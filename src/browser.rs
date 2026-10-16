use crate::*;
use anyhow::{Context, Result};
use futures_lite::future;
use gloo_timers::future::TimeoutFuture;
use iroh::{
    Endpoint, EndpointAddr, EndpointId, RelayMode,
    endpoint::{Connection, presets},
};
use std::time::Duration;
use wasm_bindgen::prelude::*;

async fn deadline<T>(
    milliseconds: u32,
    work: impl std::future::Future<Output = Result<T>>,
) -> Result<T> {
    future::race(work, async move {
        TimeoutFuture::new(milliseconds).await;
        anyhow::bail!("Connection timed out. Check that the sandbox is running, then reconnect.")
    })
    .await
}
fn js_error(error: impl std::fmt::Display) -> JsError {
    JsError::new(&error.to_string())
}

#[wasm_bindgen]
pub struct BrowserPeer {
    endpoint: Endpoint,
    connection: Connection,
    token: String,
}
#[wasm_bindgen]
impl BrowserPeer {
    pub async fn connect(invitation_json: String) -> Result<BrowserPeer, JsError> {
        console_error_panic_hook::set_once();
        let invitation: Invitation = serde_json::from_str(&invitation_json).map_err(js_error)?;
        invitation.validate().map_err(js_error)?;
        let id: EndpointId = invitation.endpoint_id.parse().map_err(js_error)?;
        let relay = browser_relay_url(&invitation.relay_url).map_err(js_error)?;
        let address = EndpointAddr::from(id).with_relay_url(relay.clone());
        // This paired client already has the server's exact relay and identity.
        // Avoid N0's dotted default relay / lookup URLs in browser TLS requests.
        let endpoint = Endpoint::builder(presets::Minimal)
            .relay_mode(RelayMode::Custom(relay.into()))
            .bind()
            .await
            .map_err(js_error)?;
        let connection = match deadline(25_000, async {
            endpoint
                .connect(address, ALPN)
                .await
                .context("Could not reach the sandbox")
        })
        .await
        {
            Ok(connection) => connection,
            Err(error) => {
                endpoint.close().await;
                return Err(js_error(error));
            }
        };
        let peer = Self {
            endpoint,
            connection,
            token: invitation.token,
        };
        if let Err(error) = peer.request(Action::Hello).await {
            peer.connection.close(0u32.into(), b"authorization failed");
            peer.endpoint.close().await;
            return Err(js_error(error));
        }
        Ok(peer)
    }
    pub fn endpoint_id(&self) -> String {
        self.endpoint.id().to_string()
    }
    pub fn path_kind(&self) -> String {
        self.connection
            .paths()
            .iter()
            .find(|path| path.is_selected())
            .map(|path| {
                if path.is_relay() {
                    "relay"
                } else if path.is_ip() {
                    "direct"
                } else {
                    "custom"
                }
            })
            .unwrap_or("checking")
            .to_string()
    }
    pub fn remote_id(&self) -> String {
        self.connection.remote_id().to_string()
    }
    pub async fn ping(&self) -> Result<String, JsError> {
        self.request(Action::Ping).await.map_err(js_error)
    }
    pub async fn send_message(&self, text: String) -> Result<String, JsError> {
        validate_text(&text).map_err(js_error)?;
        self.request(Action::Send { text }).await.map_err(js_error)
    }
    pub async fn poll(&self, after: f64) -> Result<String, JsError> {
        if !after.is_finite()
            || after < 0.0
            || after.fract() != 0.0
            || after > 9_007_199_254_740_991.0
        {
            return Err(js_error("Invalid message cursor"));
        }
        self.request(Action::Poll {
            after: after as u64,
        })
        .await
        .map_err(js_error)
    }
    pub fn disconnect(&self) {
        self.connection.close(0u32.into(), b"user disconnected");
    }
    pub async fn close(&self) {
        self.disconnect();
        self.endpoint.close().await;
    }
}
impl BrowserPeer {
    async fn request(&self, action: Action) -> Result<String> {
        deadline(Duration::from_secs(22).as_millis() as u32, async {
            let request = serde_json::to_vec(&Request {
                token: self.token.clone(),
                action,
            })?;
            anyhow::ensure!(request.len() <= MAX_REQUEST, "Request too large");
            let (mut send, mut recv) = self.connection.open_bi().await?;
            send.write_all(&request).await?;
            send.finish()?;
            let bytes = recv.read_to_end(MAX_RESPONSE).await?;
            let response: Response = serde_json::from_slice(&bytes)?;
            anyhow::ensure!(
                response.ok,
                "{}",
                response
                    .error
                    .unwrap_or_else(|| "The sandbox rejected the request".into())
            );
            Ok(String::from_utf8(bytes)?)
        })
        .await
    }
}
