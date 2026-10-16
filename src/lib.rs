use serde::{Deserialize, Serialize};

pub const ALPN: &[u8] = b"d3mnjx/iroh-lab/1";
pub const MAX_TEXT: usize = 4096;
pub const MAX_REQUEST: usize = 8192;
// Forty 4 KiB messages can expand up to sixfold under JSON escaping.
pub const MAX_RESPONSE: usize = 1_048_576;
pub const BATCH_SIZE: usize = 40;
pub const HISTORY_SIZE: usize = 200;

#[derive(Serialize, Deserialize, Clone)]
pub struct Invitation {
    pub version: u8,
    pub endpoint_id: String,
    pub relay_url: String,
    pub token: String,
}
impl Invitation {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.version == 1, "Unsupported invitation version");
        anyhow::ensure!(
            self.token.len() == 64 && self.token.bytes().all(|b| b.is_ascii_hexdigit()),
            "Invalid invitation token"
        );
        anyhow::ensure!(
            self.endpoint_id.len() == 64 && self.endpoint_id.bytes().all(|b| b.is_ascii_hexdigit()),
            "Invalid endpoint ID"
        );
        anyhow::ensure!(
            self.relay_url.starts_with("https://") && self.relay_url.len() < 2048,
            "Invalid relay URL"
        );
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub token: String,
    #[serde(flatten)]
    pub action: Action,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Hello,
    Ping,
    Send { text: String },
    Poll { after: u64 },
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: u64,
    pub from: String,
    pub text: String,
    pub at_ms: u64,
}
#[derive(Serialize, Deserialize, Default)]
pub struct Response {
    pub ok: bool,
    pub error: Option<String>,
    pub cursor: u64,
    pub messages: Vec<Message>,
    pub history_lost: bool,
    pub server_time_ms: u64,
    pub peer: String,
}
/// Use a browser TLS hostname while preserving HTTPS and certificate validation.
/// Iroh's default FQDN spelling has a final dot that WebKit rejects for TLS.
pub fn browser_relay_url(input: &str) -> anyhow::Result<iroh::RelayUrl> {
    let relay: iroh::RelayUrl = input.parse()?;
    let mut url = (*relay).clone();
    anyhow::ensure!(
        url.scheme() == "https" && url.username().is_empty() && url.password().is_none(),
        "Invalid secure relay URL"
    );
    if let Some(host) = url
        .host_str()
        .and_then(|host| host.strip_suffix('.'))
        .map(str::to_owned)
    {
        url.set_host(Some(&host))?;
    }
    Ok(url.into())
}

pub fn validate_text(text: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!text.trim().is_empty(), "Message is empty");
    anyhow::ensure!(
        text.len() <= MAX_TEXT,
        "Message is larger than 4096 UTF-8 bytes"
    );
    Ok(())
}
pub fn parse_request(bytes: &[u8]) -> anyhow::Result<Request> {
    anyhow::ensure!(bytes.len() <= MAX_REQUEST, "Request too large");
    Ok(serde_json::from_slice(bytes)?)
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod browser;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_relay_uses_a_certificate_hostname_without_disabling_tls() {
        let url = browser_relay_url("https://relay.example.:8443/").unwrap();
        assert_eq!(url.as_str(), "https://relay.example:8443/");
        assert_eq!(
            browser_relay_url("https://relay.example/")
                .unwrap()
                .as_str(),
            "https://relay.example/"
        );
        assert!(browser_relay_url("http://relay.example/").is_err());
        assert!(browser_relay_url("https://user:password@relay.example/").is_err());
    }
    #[test]
    fn reject_empty_and_oversized_unicode_messages() {
        assert!(validate_text("  \n").is_err());
        assert!(validate_text(&"é".repeat(MAX_TEXT / 2)).is_ok());
        assert!(validate_text(&"é".repeat(MAX_TEXT / 2 + 1)).is_err());
    }
    #[test]
    fn request_parser_bounds_untrusted_frames() {
        assert!(parse_request(&vec![b' '; MAX_REQUEST + 1]).is_err());
        assert!(parse_request(br#"{"token":"abc","op":"shell"}"#).is_err());
        let request = parse_request(br#"{"token":"abc","op":"poll","after":42}"#).unwrap();
        assert!(matches!(request.action, Action::Poll { after: 42 }));
    }
    #[test]
    fn invitation_rejects_unknown_versions_and_plaintext_relay() {
        let mut invite = Invitation {
            version: 1,
            endpoint_id: "a".repeat(64),
            relay_url: "https://relay.example".into(),
            token: "b".repeat(64),
        };
        assert!(invite.validate().is_ok());
        invite.version = 2;
        assert!(invite.validate().is_err());
        invite.version = 1;
        invite.relay_url = "http://relay.example".into();
        assert!(invite.validate().is_err());
    }
}
