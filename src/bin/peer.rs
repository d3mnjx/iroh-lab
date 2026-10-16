use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use clap::{Parser, Subcommand};
use fs2::FileExt;
use iroh::{
    Endpoint, SecretKey,
    endpoint::{Connection, presets},
};
use iroh_lab::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use subtle::ConstantTimeEq;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, Notify, Semaphore},
    time::timeout,
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "runtime", global = true)]
    state_dir: PathBuf,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Serve {
        #[arg(long, default_value = "https://d3mnjx.github.io/iroh-lab/")]
        base_url: String,
    },
    Send {
        text: String,
    },
    Status,
}
#[derive(Serialize, Deserialize)]
struct Control {
    address: String,
    token: String,
}
struct History {
    messages: VecDeque<Message>,
    last: u64,
}
struct State {
    token: String,
    history: Mutex<History>,
    changed: Notify,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}
fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    Ok(())
}
fn identity(path: &Path) -> Result<SecretKey> {
    match fs::read(path) {
        Ok(bytes) => Ok(SecretKey::from_bytes(&bytes.try_into().map_err(|_| {
            anyhow::anyhow!("Corrupt endpoint key; refusing to replace identity")
        })?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let key = SecretKey::generate();
            private_write(path, &key.to_bytes())?;
            Ok(key)
        }
        Err(e) => Err(e.into()),
    }
}
impl State {
    fn authorized(&self, token: &str) -> bool {
        self.token.as_bytes().ct_eq(token.as_bytes()).into()
    }
    async fn add(&self, from: &str, text: String) -> Result<Message> {
        validate_text(&text)?;
        let mut history = self.history.lock().await;
        history.last += 1;
        let message = Message {
            id: history.last,
            from: from.into(),
            text,
            at_ms: now(),
        };
        history.messages.push_back(message.clone());
        while history.messages.len() > HISTORY_SIZE {
            history.messages.pop_front();
        }
        drop(history);
        self.changed.notify_waiters();
        println!("message {} from {}", message.id, from);
        Ok(message)
    }
    async fn snapshot(&self, after: u64) -> Response {
        let history = self.history.lock().await;
        let messages: Vec<_> = history
            .messages
            .iter()
            .filter(|m| m.id > after)
            .take(BATCH_SIZE)
            .cloned()
            .collect();
        let cursor = messages.last().map(|m| m.id).unwrap_or(history.last);
        Response {
            ok: true,
            cursor,
            messages,
            history_lost: history
                .messages
                .front()
                .is_some_and(|m| after.saturating_add(1) < m.id),
            server_time_ms: now(),
            peer: "Sandbox".into(),
            ..Default::default()
        }
    }
    async fn handle(&self, request: Request, local: bool) -> Response {
        if !self.authorized(&request.token) {
            return failure("Invitation rejected. Ask for a fresh invitation.");
        }
        match request.action {
            Action::Hello | Action::Ping => Response {
                ok: true,
                cursor: self.history.lock().await.last,
                server_time_ms: now(),
                peer: "Sandbox".into(),
                ..Default::default()
            },
            Action::Send { text } => match self
                .add(if local { "sandbox" } else { "phone" }, text)
                .await
            {
                Ok(message) => Response {
                    ok: true,
                    cursor: message.id,
                    messages: vec![message],
                    server_time_ms: now(),
                    ..Default::default()
                },
                Err(e) => failure(&e.to_string()),
            },
            Action::Poll { after } => {
                let notified = self.changed.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                let snapshot = self.snapshot(after).await;
                if !snapshot.messages.is_empty() || local {
                    return snapshot;
                }
                let _ = timeout(Duration::from_secs(12), notified).await;
                self.snapshot(after).await
            }
        }
    }
}
fn failure(message: &str) -> Response {
    Response {
        error: Some(message.into()),
        ..Default::default()
    }
}
async fn handle_connection(conn: Connection, state: Arc<State>) -> Result<()> {
    println!("connected peer {}", conn.remote_id());
    let slots = Arc::new(Semaphore::new(8));
    while let Ok((mut send, mut recv)) = conn.accept_bi().await {
        let Ok(permit) = slots.clone().try_acquire_owned() else {
            conn.close(429u32.into(), b"too many streams");
            break;
        };
        let state = state.clone();
        let conn = conn.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let result: Result<()> = async {
                let bytes =
                    timeout(Duration::from_secs(8), recv.read_to_end(MAX_REQUEST)).await??;
                let response = match parse_request(&bytes) {
                    Ok(request) => state.handle(request, false).await,
                    Err(_) => failure("Invalid request"),
                };
                let rejected = !response.ok;
                let bytes = serde_json::to_vec(&response)?;
                anyhow::ensure!(bytes.len() <= MAX_RESPONSE, "Response too large");
                timeout(Duration::from_secs(5), send.write_all(&bytes)).await??;
                send.finish()?;
                // Wait for acknowledged delivery before closing rejected connections.
                if rejected {
                    let _ = timeout(Duration::from_secs(2), send.stopped()).await;
                    conn.close(403u32.into(), b"request rejected");
                }
                Ok(())
            }
            .await;
            if result.is_err() {
                conn.close(400u32.into(), b"invalid or timed out stream");
            }
        });
    }
    println!("disconnected peer {}", conn.remote_id());
    Ok(())
}
async fn control_connection(mut socket: TcpStream, state: Arc<State>) -> Result<()> {
    let mut bytes = Vec::new();
    timeout(
        Duration::from_secs(5),
        (&mut socket)
            .take((MAX_REQUEST + 1) as u64)
            .read_to_end(&mut bytes),
    )
    .await??;
    let response = match parse_request(&bytes) {
        Ok(request) => state.handle(request, true).await,
        Err(_) => failure("Invalid control request"),
    };
    socket.write_all(&serde_json::to_vec(&response)?).await?;
    socket.shutdown().await?;
    Ok(())
}
async fn serve(directory: PathBuf, base_url: String) -> Result<()> {
    anyhow::ensure!(
        base_url.starts_with("https://")
            || base_url.starts_with("http://localhost:")
            || base_url.starts_with("http://127.0.0.1:"),
        "Use an HTTPS Pages URL or a local development URL"
    );
    fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
    }
    let lock = File::create(directory.join("peer.lock"))?;
    lock.try_lock_exclusive()
        .context("A peer is already using this state directory")?;
    let key = identity(&directory.join("endpoint.key"))?;
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(key)
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await?;
    timeout(Duration::from_secs(35), endpoint.online())
        .await
        .context("No Iroh relay became reachable within 35 seconds")?;
    let address = endpoint.addr();
    let relay = address
        .relay_urls()
        .next()
        .context("No home relay")?
        .to_string();
    let token = hex(&SecretKey::generate().to_bytes());
    let invitation = Invitation {
        version: 1,
        endpoint_id: endpoint.id().to_string(),
        relay_url: relay.clone(),
        token: token.clone(),
    };
    let encoded = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&invitation)?);
    let pairing_url = format!("{}#invite={encoded}", base_url.trim_end_matches('#'));
    private_write(
        &directory.join("invitation.json"),
        &serde_json::to_vec_pretty(&invitation)?,
    )?;
    private_write(&directory.join("pairing-url.txt"), pairing_url.as_bytes())?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    private_write(
        &directory.join("control.json"),
        &serde_json::to_vec(&Control {
            address: listener.local_addr()?.to_string(),
            token: token.clone(),
        })?,
    )?;
    let state = Arc::new(State {
        token,
        history: Mutex::new(History {
            messages: VecDeque::new(),
            last: 0,
        }),
        changed: Notify::new(),
    });
    state
        .add(
            "sandbox",
            "Sandbox is ready. Send a message from your phone, or try Ping.".into(),
        )
        .await?;
    println!(
        "Endpoint: {}\nRelay: {relay}\nPrivate pairing link: {}\nReady. Use `iroh-lab-peer send <text>` from another terminal.",
        endpoint.id(),
        directory.join("pairing-url.txt").display()
    );
    let slots = Arc::new(Semaphore::new(32));
    loop {
        tokio::select! {
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else { break; };
                let Ok(permit) = slots.clone().try_acquire_owned() else { incoming.refuse(); continue; };
                let state = state.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    if let Ok(Ok(conn)) = timeout(Duration::from_secs(10), incoming).await {
                        let _ = handle_connection(conn, state).await;
                    }
                });
            }
            accepted = listener.accept() => {
                let (socket, _) = accepted?;
                let Ok(permit) = slots.clone().try_acquire_owned() else { continue; };
                let state = state.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    let _ = timeout(Duration::from_secs(10), control_connection(socket, state)).await;
                });
            }
            _ = tokio::signal::ctrl_c() => { break; }
        }
    }
    endpoint.close().await;
    Ok(())
}
async fn control(directory: &Path, action: Action) -> Result<()> {
    let control: Control = serde_json::from_slice(
        &fs::read(directory.join("control.json")).context("Start the sandbox peer first")?,
    )?;
    let request = serde_json::to_vec(&Request {
        token: control.token,
        action,
    })?;
    anyhow::ensure!(request.len() <= MAX_REQUEST, "Message too large");
    let mut socket = TcpStream::connect(&control.address).await?;
    socket.write_all(&request).await?;
    socket.shutdown().await?;
    let mut bytes = Vec::new();
    socket
        .take((MAX_RESPONSE + 1) as u64)
        .read_to_end(&mut bytes)
        .await?;
    let response: Response = serde_json::from_slice(&bytes)?;
    anyhow::ensure!(response.ok, "{}", response.error.unwrap_or_default());
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();
    let args = Args::parse();
    match args.command {
        Command::Serve { base_url } => serve(args.state_dir, base_url).await,
        Command::Send { text } => {
            timeout(
                Duration::from_secs(10),
                control(&args.state_dir, Action::Send { text }),
            )
            .await?
        }
        Command::Status => {
            timeout(
                Duration::from_secs(10),
                control(&args.state_dir, Action::Poll { after: 0 }),
            )
            .await?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn authorization_precedes_any_message_access() {
        let state = State {
            token: "a".repeat(64),
            history: Mutex::new(History {
                messages: VecDeque::new(),
                last: 0,
            }),
            changed: Notify::new(),
        };
        let response = state
            .handle(
                Request {
                    token: "b".repeat(64),
                    action: Action::Send { text: "no".into() },
                },
                false,
            )
            .await;
        assert!(!response.ok);
        assert_eq!(state.history.lock().await.last, 0);
        let response = state
            .handle(
                Request {
                    token: "a".repeat(64),
                    action: Action::Send { text: "yes".into() },
                },
                false,
            )
            .await;
        assert!(response.ok);
        assert_eq!(response.messages[0].from, "phone");
    }
    #[tokio::test]
    async fn history_reports_eviction_and_batches_without_skipping() {
        let state = State {
            token: "a".repeat(64),
            history: Mutex::new(History {
                messages: VecDeque::new(),
                last: 0,
            }),
            changed: Notify::new(),
        };
        for n in 0..HISTORY_SIZE + 3 {
            state.add("sandbox", n.to_string()).await.unwrap();
        }
        let response = state.snapshot(0).await;
        assert!(response.history_lost);
        assert_eq!(response.messages.len(), BATCH_SIZE);
        assert_eq!(response.messages[0].id, 4);
        let next = state.snapshot(response.cursor).await;
        assert_eq!(next.messages[0].id, response.cursor + 1);
    }
}
