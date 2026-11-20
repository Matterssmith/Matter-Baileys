/*
 * WhatsApp-Matter — strict Rust ingress for the WhatsMeow provider family.
 * This validates untrusted events and ensures storage isolation.
 * It does NOT sign provider events, authenticate a sender or grant roles.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use crate::normalize;
use matter_whatsapp_contracts::{NormalizeError, NormalizedTextEvent, UntrustedTextEvent};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    fmt, fs,
    path::{Component, Path, PathBuf},
};

pub const BRIDGE_VERSION: u16 = 1;
pub const MAX_EVENT_BYTES: usize = 8 * 1024;
pub const MAX_REPLAY_ENTRIES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineMode {
    /// Rust event/contract mapping; zero network connections.
    OfflineRust,
    /// Official Go WhatsMeow implementation, if installed as an isolated worker.
    /// This is NOT a native Rust port and is never started implicitly.
    ExternalGo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeState {
    Offline,
    Unauthenticated,
    AwaitingExplicitPairing,
    AuthenticatedReadOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhatsmeowTextEnvelope {
    pub version: u16,
    pub provider: String,
    pub socket_id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub message_id: String,
    pub text: String,
    /// The event kind describes an observation, not permission to execute it.
    pub kind: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EngineError {
    InvalidEvent,
    EventTooLarge,
    InvalidSocket,
    UnsafeStorage,
    Replay,
    UntrustedOrigin,
    FeatureUnavailable,
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::InvalidEvent => "invalid_event",
                Self::EventTooLarge => "event_too_large",
                Self::InvalidSocket => "invalid_socket",
                Self::UnsafeStorage => "unsafe_storage",
                Self::Replay => "replay",
                Self::UntrustedOrigin => "untrusted_origin",
                Self::FeatureUnavailable => "feature_unavailable",
            }
        )
    }
}

impl std::error::Error for EngineError {}

fn limited_identifier(input: &str) -> bool {
    !input.is_empty()
        && input.len() <= 128
        && input
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@._:-".contains(&b))
}

fn valid_socket_id(input: &str) -> bool {
    limited_identifier(input)
        && !input.contains("..")
        && !input.contains('@')
        && !input.contains('/')
        && input != "wa-alpha"
        && input != "wa-test-1"
}

/// Maps only a text observation into Matter's neutral event contract.
/// The returned event has NO authenticated / authorized status.
pub fn decode_untrusted_event(
    json: &str,
    configured_socket_id: &str,
) -> Result<NormalizedTextEvent, EngineError> {
    if json.len() > MAX_EVENT_BYTES {
        return Err(EngineError::EventTooLarge);
    }
    if !valid_socket_id(configured_socket_id) {
        return Err(EngineError::InvalidSocket);
    }
    let event: WhatsmeowTextEnvelope =
        serde_json::from_str(json).map_err(|_| EngineError::InvalidEvent)?;
    if event.version != BRIDGE_VERSION
        || event.provider != "whatsmeow"
        || event.socket_id != configured_socket_id
        || !matches!(event.kind.as_str(), "original" | "edited")
        || !limited_identifier(&event.chat_id)
        || !limited_identifier(&event.sender_id)
        || !limited_identifier(&event.message_id)
        || event.text.contains('\0')
    {
        return Err(EngineError::InvalidEvent);
    }
    let raw = UntrustedTextEvent {
        chat_id: event.chat_id,
        sender_id: event.sender_id,
        message_id: event.message_id,
        text: event.text,
    };
    normalize(raw).map_err(|err| match err {
        NormalizeError::InvalidIdentity | NormalizeError::InvalidContent => {
            EngineError::InvalidEvent
        }
    })
}

/// Duplicates are rejected even if the original was edited.
/// This is an in-memory ingress limiter, NOT durable antireplay authority.
/// Authenticated control must be implemented separately in Vault/Base.
pub struct ReadOnlyReplayFilter {
    seen: HashSet<String>,
    recent: VecDeque<String>,
}

impl Default for ReadOnlyReplayFilter {
    fn default() -> Self {
        Self {
            seen: HashSet::new(),
            recent: VecDeque::new(),
        }
    }
}

impl ReadOnlyReplayFilter {
    pub fn accept(&mut self, event: &NormalizedTextEvent) -> Result<(), EngineError> {
        if event.provider != "whatsmeow" {
            return Err(EngineError::UntrustedOrigin);
        }
        let key = format!(
            "{}\0{}\0{}",
            event.chat_id, event.sender_id, event.message_id
        );
        if !self.seen.insert(key.clone()) {
            return Err(EngineError::Replay);
        }
        self.recent.push_back(key);
        if self.recent.len() > MAX_REPLAY_ENTRIES {
            if let Some(previous) = self.recent.pop_front() {
                self.seen.remove(&previous);
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.recent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recent.is_empty()
    }
}

/// WhatsMeow requires a distinct session store for every socket/provider.
/// Must be called on an operator-provisioned private directory before startup.
pub fn isolated_session_root(root: &Path, socket_id: &str) -> Result<PathBuf, EngineError> {
    if !valid_socket_id(socket_id) || !root.is_absolute() {
        return Err(EngineError::UnsafeStorage);
    }
    let mut has_whatsmeow_component = false;
    for component in root.components() {
        match component {
            Component::Normal(name) => {
                if name == "whatsmeow" {
                    has_whatsmeow_component = true;
                }
            }
            Component::ParentDir | Component::CurDir => return Err(EngineError::UnsafeStorage),
            _ => {}
        }
    }
    if !has_whatsmeow_component {
        return Err(EngineError::UnsafeStorage);
    }
    let info = fs::symlink_metadata(root).map_err(|_| EngineError::UnsafeStorage)?;
    if !info.is_dir() || info.file_type().is_symlink() {
        return Err(EngineError::UnsafeStorage);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if info.permissions().mode() & 0o077 != 0 {
            return Err(EngineError::UnsafeStorage);
        }
    }
    let state = root.join(socket_id);
    // Never create or overwrite an existing store implicitly.
    let state_info = fs::symlink_metadata(&state).map_err(|_| EngineError::UnsafeStorage)?;
    if !state_info.is_dir() || state_info.file_type().is_symlink() {
        return Err(EngineError::UnsafeStorage);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if state_info.permissions().mode() & 0o077 != 0 {
            return Err(EngineError::UnsafeStorage);
        }
    }
    Ok(state)
}

/// Pairing/login and mutating messaging are deliberately NOT callable via
/// the public descriptive bridge. Only trusted host-side setup may enable
/// an isolated experimental engine after authorization and live tests.
pub fn require_mutating_authority(
    _untrusted_event: &NormalizedTextEvent,
) -> Result<(), EngineError> {
    Err(EngineError::UntrustedOrigin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> String {
        serde_json::json!({
            "version": 1,
            "provider": "whatsmeow",
            "socket_id": "wa-whatsmeow-test",
            "chat_id": "120363000000000@g.us",
            "sender_id": "5215550000000@s.whatsapp.net",
            "message_id": "ABC:123",
            "text": "=/>set role owner --global",
            "kind": "original"
        })
        .to_string()
    }

    #[test]
    fn accepted_text_is_not_a_permission() {
        let event = decode_untrusted_event(&sample(), "wa-whatsmeow-test").unwrap();
        assert_eq!(event.provider, "whatsmeow");
        assert!(require_mutating_authority(&event).is_err());
    }

    #[test]
    fn rejects_socket_spoofing_and_extra_authority_fields() {
        assert_eq!(
            decode_untrusted_event(&sample(), "wa-alpha").unwrap_err(),
            EngineError::InvalidSocket
        );
        assert_eq!(
            decode_untrusted_event(&sample(), "wa-whatsmeow-other").unwrap_err(),
            EngineError::InvalidEvent
        );
        let forged = sample().replace(
            "\"kind\":\"original\"",
            "\"kind\":\"original\",\"owner\":true",
        );
        assert_eq!(
            decode_untrusted_event(&forged, "wa-whatsmeow-test").unwrap_err(),
            EngineError::InvalidEvent
        );
        let edited = sample().replace("\"kind\":\"original\"", "\"kind\":\"edited\"");
        assert!(decode_untrusted_event(&edited, "wa-whatsmeow-test").is_ok());
        assert_eq!(
            decode_untrusted_event(&"A".repeat(MAX_EVENT_BYTES + 1), "wa-whatsmeow-test")
                .unwrap_err(),
            EngineError::EventTooLarge
        );
    }

    #[test]
    fn bounds_replays_without_claiming_durable_security() {
        let event = decode_untrusted_event(&sample(), "wa-whatsmeow-test").unwrap();
        let mut replay = ReadOnlyReplayFilter::default();
        assert!(replay.is_empty());
        replay.accept(&event).unwrap();
        assert_eq!(replay.accept(&event).unwrap_err(), EngineError::Replay);
        assert_eq!(replay.len(), 1);
    }

    #[test]
    fn isolated_root_rejects_original_socket() {
        assert!(
            isolated_session_root(Path::new("/tmp/base-matter/whatsmeow"), "wa-alpha").is_err()
        );
        assert!(
            isolated_session_root(Path::new("/tmp/base-matter/whatsmeow"), "wa-test-1").is_err()
        );
        assert!(
            isolated_session_root(Path::new("/tmp/base-matter/whatsapp-rust"), "wa-meow-test")
                .is_err()
        );
    }
}
