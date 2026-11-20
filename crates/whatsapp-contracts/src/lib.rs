/*
 * WhatsApp-Matter — Neutral provider contracts and bounded read-only bridge.
 * No sessions, moderation, authorization, network connection or Vault keys.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_REQUEST_BYTES: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    NativeRustMigration,
    CompatibilityOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderDescriptor {
    pub name: String,
    pub runtime: String,
    pub original_ecosystem: String,
    pub status: ProviderStatus,
    pub operations: Vec<String>,
    pub control_authority: bool,
}

/// This describes code availability, not a live authenticated WhatsApp connection.
pub fn describe_provider(name: &str) -> Option<ProviderDescriptor> {
    let (original_ecosystem, status) = match name {
        "whatsapp-rust" => ("Rust", ProviderStatus::NativeRustMigration),
        "baileys" => ("TypeScript/JavaScript", ProviderStatus::CompatibilityOnly),
        "whatsmeow" => ("Go", ProviderStatus::CompatibilityOnly),
        _ => return None,
    };
    Some(ProviderDescriptor {
        name: name.to_owned(),
        runtime: "Rust".to_owned(),
        original_ecosystem: original_ecosystem.to_owned(),
        status,
        operations: vec!["describe".to_owned()],
        control_authority: false,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRequest {
    pub version: u16,
    pub operation: String,
    pub provider: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireResponse {
    pub version: u16,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl WireResponse {
    pub fn denied(reason: &str) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            ok: false,
            provider: None,
            error: Some(reason.to_owned()),
        }
    }
}

/// The public bridge intentionally supports DESCRIBE ONLY.
/// A future mutating gateway must be separately authenticated via Vault.
pub fn handle_json(line: &str) -> WireResponse {
    if line.len() > MAX_REQUEST_BYTES {
        return WireResponse::denied("request_too_large");
    }
    let request: WireRequest = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => return WireResponse::denied("invalid_request"),
    };
    if request.version != PROTOCOL_VERSION {
        return WireResponse::denied("unsupported_version");
    }
    if request.operation != "describe" {
        return WireResponse::denied("operation_not_allowed");
    }
    let Some(provider) = describe_provider(&request.provider) else {
        return WireResponse::denied("unknown_provider");
    };
    WireResponse {
        version: PROTOCOL_VERSION,
        ok: true,
        provider: Some(provider),
        error: None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UntrustedTextEvent {
    pub chat_id: String,
    pub sender_id: String,
    pub message_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedTextEvent {
    pub provider: String,
    pub chat_id: String,
    pub sender_id: String,
    pub message_id: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NormalizeError {
    InvalidIdentity,
    InvalidContent,
}

/// An event transformed by an adapter is STILL UNVERIFIED.
/// This type deliberately contains no permission or "authenticated" bit.
pub fn normalize_untrusted(
    provider: &str,
    event: UntrustedTextEvent,
) -> Result<NormalizedTextEvent, NormalizeError> {
    fn valid_id(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"@._:-".contains(&b))
    }
    if describe_provider(provider).is_none()
        || !valid_id(&event.chat_id)
        || !valid_id(&event.sender_id)
        || !valid_id(&event.message_id)
    {
        return Err(NormalizeError::InvalidIdentity);
    }
    if event.text.len() > 4096 || event.text.chars().any(|c| c == '\0') {
        return Err(NormalizeError::InvalidContent);
    }
    Ok(NormalizedTextEvent {
        provider: provider.to_owned(),
        chat_id: event.chat_id,
        sender_id: event.sender_id,
        message_id: event.message_id,
        text: event.text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognized_providers_are_explicitly_non_authoritative() {
        for provider in ["whatsapp-rust", "baileys", "whatsmeow"] {
            assert!(!describe_provider(provider).unwrap().control_authority);
        }
    }

    #[test]
    fn read_only_describe_roundtrip() {
        let input = serde_json::to_string(&WireRequest {
            version: PROTOCOL_VERSION,
            operation: "describe".into(),
            provider: "whatsapp-rust".into(),
        })
        .unwrap();
        let response = handle_json(&input);
        assert!(response.ok);
        assert_eq!(response.provider.unwrap().runtime, "Rust");
    }

    #[test]
    fn no_privileged_operation_or_unexpected_fields() {
        assert!(!handle_json(r#"{"version":1,"operation":"send","provider":"baileys"}"#).ok);
        assert!(!handle_json(r#"{"version":1,"operation":"describe","provider":"baileys","token":"secret"}"#).ok);
        assert!(!handle_json(r#"{"version":2,"operation":"describe","provider":"baileys"}"#).ok);
    }

    #[test]
    fn unknown_provider_denied() {
        assert_eq!(
            handle_json(r#"{"version":1,"operation":"describe","provider":"other"}"#)
                .error
                .as_deref(),
            Some("unknown_provider")
        );
    }

    #[test]
    fn untrusted_events_cannot_claim_authority() {
        let msg = UntrustedTextEvent {
            chat_id: "group@g.us".into(),
            sender_id: "user@s.whatsapp.net".into(),
            message_id: "msg:1".into(),
            text: "hola".into(),
        };
        let normalized = normalize_untrusted("baileys", msg).unwrap();
        assert_eq!(normalized.provider, "baileys");
    }
}

// Créditos:
// Hecho por Lournyx Matterssmith [Lou-Nyx].
