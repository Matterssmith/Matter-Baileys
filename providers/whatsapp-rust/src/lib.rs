/*
 * WhatsApp-Matter — standalone Rust provider, bridge signing and session lease.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
pub mod help_reply;
pub mod menu_sound;
pub mod recovery;
pub mod rich_preview;
pub mod rich_transport;
pub mod sticker_badge;
pub mod ui;

// Provider-only fallback frame. The command registry lives in the
// independent Base-Matter core; no core modules are imported here.
mod provider_control_style;

pub fn render_provider_control(body: &str) -> Result<String> {
    provider_control_style::decorate(body)
}

use anyhow::{anyhow, bail, Result};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;

const DOMAIN: &[u8] = b"Base-Matter:WhatsApp-Control:v1\0";

pub struct Config {
    pub phone: String,
    pub owner_jid: String,
    pub socket_id: String,
    pub actor_id: String,
    pub session_db: PathBuf,
    pub bridge_addr: SocketAddrV4,
    pub bridge_key: Zeroizing<[u8; 32]>,
}

#[derive(Debug)]
pub struct Event<'a> {
    pub chat_jid: &'a str,
    pub sender_jid: &'a str,
    pub message_id: &'a str,
    pub text: &'a str,
    pub from_me: bool,
    pub is_group: bool,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct SignedFrame {
    pub version: u8,
    pub socket_id: String,
    pub actor_id: String,
    pub owner_jid: String,
    pub chat_jid: String,
    pub message_id: String,
    pub text: String,
    pub from_me: bool,
    pub timestamp: i64,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlAck {
    Applied,
    AppliedWithMessage(String),
    Duplicate,
    Ignored,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlAckWire {
    ok: bool,
    outcome: String,
    #[serde(default)]
    message: Option<String>,
}

/// Interpret the signed-control IPC acknowledgment conservatively.
/// A duplicate is acknowledged as a replay without claiming another effect.
pub fn parse_control_ack(input: &[u8]) -> Result<ControlAck> {
    if input.is_empty() || input.len() > 8192 {
        bail!("Invalid controller acknowledgment");
    }
    let response: ControlAckWire =
        serde_json::from_slice(input).map_err(|_| anyhow!("Invalid controller acknowledgment"))?;
    match (response.ok, response.outcome.as_str(), response.message) {
        (true, "applied", None) => Ok(ControlAck::Applied),
        (true, "applied", Some(message))
            if !message.is_empty()
                && message.len() <= 3800
                && !message.chars().any(|c| c.is_control() && c != '\n') =>
        {
            Ok(ControlAck::AppliedWithMessage(message))
        }
        (true, "duplicate", None) => Ok(ControlAck::Duplicate),
        (true, "ignored", None) => Ok(ControlAck::Ignored),
        _ => bail!("Controller denied the operation"),
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Self::from_reader(|name| env::var(name).ok())
    }

    pub fn from_reader(mut get: impl FnMut(&str) -> Option<String>) -> Result<Self> {
        let mut required = |name: &str| {
            let value = get(name).ok_or_else(|| anyhow!("Missing provider configuration"))?;
            if value.is_empty() {
                bail!("Empty provider configuration");
            }
            Ok(value)
        };
        let phone = required("MATTER_WA_PHONE")?;
        if !(8..=15).contains(&phone.len())
            || phone.starts_with('0')
            || !phone.bytes().all(|b| b.is_ascii_digit())
        {
            bail!("Invalid international pairing phone");
        }
        let owner_jid = required("MATTER_WA_OWNER_JID")?;
        let user = owner_jid.split('@').next().unwrap_or("");
        if user.is_empty()
            || owner_jid.len() > 128
            || !(owner_jid.ends_with("@s.whatsapp.net") || owner_jid.ends_with("@lid"))
            || !user
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
        {
            bail!("Invalid owner private JID");
        }
        let socket_id = required("MATTER_WA_SOCKET_ID")?;
        let actor_id = required("MATTER_WA_OWNER_ID")?;
        for value in [&socket_id, &actor_id] {
            if value.len() > 128
                || !value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
            {
                bail!("Invalid provider socket or actor ID");
            }
        }
        let session_db = private_path(&required("MATTER_WA_SESSION_DB")?)?;
        let addr: SocketAddrV4 = required("MATTER_WA_BRIDGE_ADDR")?
            .parse()
            .map_err(|_| anyhow!("Invalid loopback bridge address"))?;
        if *addr.ip() != Ipv4Addr::LOCALHOST || addr.port() == 0 {
            bail!("Bridge must be bound to 127.0.0.1 with a nonzero port");
        }
        let raw = Zeroizing::new(required("MATTER_WA_BRIDGE_KEY_HEX")?);
        if raw.len() != 64 || !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
            bail!("Invalid 256-bit bridge key");
        }
        let mut bridge_key = Zeroizing::new([0_u8; 32]);
        hex::decode_to_slice(raw.as_bytes(), &mut bridge_key[..])
            .map_err(|_| anyhow!("Invalid bridge key encoding"))?;
        if bridge_key.iter().all(|b| *b == 0) {
            bail!("Bridge key must not be zero");
        }
        Ok(Self {
            phone,
            owner_jid,
            socket_id,
            actor_id,
            session_db,
            bridge_addr: addr,
            bridge_key,
        })
    }

    /// Accept the PN and LID aliases that belong to the *authenticated*
    /// WhatsApp client, never arbitrary aliases supplied by an event.
    ///
    /// The caller must pass canonical, device-free JIDs obtained from
    /// Client::pn(), Client::lid(), and MessageSource in the same session.
    /// The signed bridge frame stays bound to the configured owner, because
    /// Base-Matter checks its authenticated HMAC and canonical owner JID.
    pub fn signed_verified_control(
        &self,
        event: &Event<'_>,
        session_pn: Option<&str>,
        session_lid: Option<&str>,
    ) -> Result<Option<SignedFrame>> {
        if !event.from_me || !event.text.starts_with("=/>") {
            return Ok(None);
        }
        // A group command is an owner-authored, explicitly scoped control.
        // No ordinary bot handler or third-party message is enabled here.
        let command = event.text.split_whitespace().next().unwrap_or_default();
        let allowed = if event.is_group {
            matches!(
                command,
                "=/>group"
                    | "=/>grupo"
                    | "=/>menu"
                    | "=/>menú"
                    | "=/>help"
                    | "=/>h"
                    | "=/>ayuda"
                    | "=/>status"
                    | "=/>estado"
                    | "=/>bot"
                    | "=/>start"
                    | "=/>add"
                    | "=/>add++"
                    | "=/>anti"
                    | "=/>set"
                    | "=/>credits"
                    | "=/>creditos"
                    | "=/>créditos"
                    | "=/>test"
                    | "=/>register"
            )
        } else {
            matches!(
                command,
                "=/>status"
                    | "=/>estado"
                    | "=/>bot"
                    | "=/>menu"
                    | "=/>menú"
                    | "=/>help"
                    | "=/>h"
                    | "=/>ayuda"
                    | "=/>set"
                    | "=/>unset"
                    | "=/>role"
                    | "=/>roles"
                    | "=/>audit"
                    | "=/>matter"
                    | "=/>test"
                    | "=/>register"
                    | "=/>anti"
                    | "=/>credits"
                    | "=/>creditos"
                    | "=/>créditos"
            )
        };
        if !allowed {
            return Ok(None);
        }

        let expected_pn = format!("{}@s.whatsapp.net", self.phone);
        // Proof of the linked account comes from the connected client,
        // not the phone value typed at setup or the outgoing message itself.
        if session_pn != Some(expected_pn.as_str()) {
            bail!("Authenticated WhatsApp account does not match pairing owner");
        }
        let verified_lid = session_lid
            .filter(|jid| jid.ends_with("@lid") && !jid.starts_with('@') && jid.len() <= 128);
        // A preconfigured LID must itself match this session's verified LID.
        if self.owner_jid != expected_pn && Some(self.owner_jid.as_str()) != verified_lid {
            bail!("Configured WhatsApp owner differs from authenticated account");
        }

        let belongs_to_owner = |jid: &str| jid == expected_pn.as_str() || verified_lid == Some(jid);
        if !belongs_to_owner(event.sender_jid) {
            return Ok(None);
        }
        if event.is_group {
            if !event.chat_jid.ends_with("@g.us")
                || event.chat_jid.len() > 128
                || !event
                    .chat_jid
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"@._:-".contains(&b))
            {
                return Ok(None);
            }
        } else if !belongs_to_owner(event.chat_jid) {
            return Ok(None);
        }

        // Only identity fields proven above are canonicalized. Message ID,
        // text, timestamp and from_me retain the original callback values.
        let canonical = Event {
            chat_jid: &self.owner_jid,
            sender_jid: &self.owner_jid,
            message_id: event.message_id,
            text: event.text,
            from_me: event.from_me,
            // Scope was verified above. Reuse the legacy private signer
            // before replacing chat_jid with the signed group destination.
            is_group: false,
            timestamp: event.timestamp,
        };
        let mut signed = self.signed_control(&canonical)?;
        if event.is_group {
            if let Some(frame) = signed.as_mut() {
                // The group JID is derived from the authenticated WhatsApp
                // callback and is covered by the HMAC signature.
                frame.chat_jid = event.chat_jid.to_owned();
                frame.signature = sign_frame(frame, &self.bridge_key)?;
            }
        }
        Ok(signed)
    }

    /// Sign incoming, strictly READ-ONLY chat requests.
    /// The callback must supply real sender and destination JIDs, never
    /// sender IDs typed by a person in a WhatsApp message.
    /// This method cannot sign group power, socket roles, or any mutations.
    pub fn signed_public_readonly(
        &self,
        event: &Event<'_>,
        session_pn: Option<&str>,
        session_lid: Option<&str>,
    ) -> Result<Option<SignedFrame>> {
        if event.from_me || !event.text.starts_with("=/>") {
            return Ok(None);
        }
        let parts: Vec<_> = event.text.split_whitespace().collect();
        // Exactly the approved 21 visible entries, plus legacy aliases.
        // These names only authorize READ-ONLY routing; never elevation.
        let public_category = |value: &str| {
            matches!(
                value,
                "ia" | "anti"
                    | "all"
                    | "config"
                    | "downloads"
                    | "economy"
                    | "fun"
                    | "gacha"
                    | "games"
                    | "group"
                    | "info"
                    | "links"
                    | "main"
                    | "mode"
                    | "owner"
                    | "profile"
                    | "reactions"
                    | "search"
                    | "sockets"
                    | "system"
                    | "utils"
                    | "ai"
                    | "antis"
                    | "set"
                    | "bot"
                    | "roles"
                    | "status"
                    | "navigation"
                    | "sandbox"
            )
        };
        let safe_help_name = |name: &str| {
            !name.is_empty()
                && name.len() <= 48
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
        };
        let allowed = match parts.as_slice() {
            ["=/>status" | "=/>estado"]
            | ["=/>status" | "=/>estado", "--mod"]
            | ["=/>status" | "=/>estado", "--global"]
            | ["=/>menu" | "=/>menú"]
            | ["=/>bot"]
            | ["=/>credits" | "=/>creditos" | "=/>créditos"] => true,
            ["=/>help" | "=/>h" | "=/>ayuda"] => true,
            ["=/>help", "--quoted", payload] => {
                !payload.is_empty()
                    && payload.len() <= 1500
                    && payload
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
            }
            ["=/>test"] | ["=/>test", "ui" | "row"] => true,
            ["=/>register"] | ["=/>register", "start"] => true,
            ["=/>test", "ack", "a" | "b" | "one" | "two" | "audio" | "mp4" | "link"] => true,
            ["=/>help" | "=/>h" | "=/>ayuda", category] => {
                public_category(category) || *category == "all" || safe_help_name(category)
            }
            ["=/>help" | "=/>h" | "=/>ayuda", category, page] => {
                (public_category(category)
                    && page.parse::<usize>().is_ok_and(|v| (1..=8).contains(&v)))
                    || (*category == "set" && *page == "role")
                    || (*category == "anti" && *page == "audit")
            }
            ["=/>help" | "=/>h" | "=/>ayuda", "set", "anti", "link"] => true,
            ["=/>anti", arguments @ ..] if arguments.len() <= 2 => {
                let scope_count = arguments
                    .iter()
                    .filter(|argument| matches!(**argument, "--mod" | "--global"))
                    .count();
                let filters = arguments
                    .iter()
                    .filter(|argument| !matches!(**argument, "--mod" | "--global"))
                    .copied()
                    .collect::<Vec<_>>();
                scope_count <= 1
                    && filters.len() <= 1
                    && filters.iter().all(|filter| {
                        (1..=24).contains(&filter.len())
                            && filter
                                .bytes()
                                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                    })
            }
            ["=/>menu" | "=/>menú", category] => public_category(category) || *category == "all",
            ["=/>menu" | "=/>menú", "page", page] => matches!(*page, "1" | "2" | "3"),
            ["=/>menu" | "=/>menú", category, page] => {
                public_category(category)
                    && page
                        .parse::<usize>()
                        .is_ok_and(|value| (1..=8).contains(&value))
            }
            _ => false,
        };
        if !allowed {
            return Ok(None);
        }
        let valid_sender = |value: &str| {
            value
                .strip_suffix("@s.whatsapp.net")
                .or_else(|| value.strip_suffix("@lid"))
                .is_some_and(|account| {
                    (8..=20).contains(&account.len()) && account.bytes().all(|b| b.is_ascii_digit())
                })
        };
        if !valid_sender(event.sender_jid) {
            return Ok(None);
        }
        let valid_group = event.chat_jid.ends_with("@g.us")
            && event.chat_jid.len() <= 128
            && event
                .chat_jid
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"@._:-".contains(&b));
        if event.is_group {
            if !valid_group {
                return Ok(None);
            }
        } else if event.chat_jid != event.sender_jid {
            // No unverified LID/PN identity linking for third parties.
            return Ok(None);
        }
        let own_pn = format!("{}@s.whatsapp.net", self.phone);
        if session_pn != Some(own_pn.as_str())
            || (self.owner_jid != own_pn && Some(self.owner_jid.as_str()) != session_lid)
        {
            bail!("Authenticated WhatsApp session identity mismatch");
        }
        if event.message_id.is_empty()
            || event.message_id.len() > 256
            || event.message_id.chars().any(char::is_control)
            || event.text.is_empty()
            || event.text.len() > 4096
        {
            bail!("Invalid readonly WhatsApp event");
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        if now.abs_diff(event.timestamp) > 120 {
            bail!("Expired readonly WhatsApp event");
        }
        let mut frame = SignedFrame {
            version: 1,
            socket_id: self.socket_id.clone(),
            // For readonly ingress actor_id is the verified sender JID,
            // never the privileged configured socket-owner actor ID.
            actor_id: event.sender_jid.to_owned(),
            owner_jid: self.owner_jid.clone(),
            chat_jid: event.chat_jid.to_owned(),
            message_id: event.message_id.to_owned(),
            text: event.text.to_owned(),
            from_me: false,
            timestamp: event.timestamp,
            signature: String::new(),
        };
        frame.signature = sign_frame(&frame, &self.bridge_key)?;
        Ok(Some(frame))
    }

    /// Emit a signed, READ-ONLY Anti observation for a real group callback.
    /// The invisible internal prefix cannot be supplied as a control command
    /// and the signed observation never gives sender admin permissions.
    pub fn signed_anti_observation(
        &self,
        event: &Event<'_>,
        edit_state: &str,
    ) -> Result<Option<SignedFrame>> {
        if !event.is_group
            || !event.chat_jid.ends_with("@g.us")
            || event.chat_jid.len() > 128
            || !event
                .chat_jid
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"@._:-".contains(&byte))
            || !matches!(edit_state, "original" | "edited" | "opaque")
        {
            return Ok(None);
        }
        let valid_sender = event
            .sender_jid
            .strip_suffix("@s.whatsapp.net")
            .or_else(|| event.sender_jid.strip_suffix("@lid"))
            .is_some_and(|account| {
                (8..=20).contains(&account.len())
                    && account.bytes().all(|byte| byte.is_ascii_digit())
            });
        if !valid_sender
            || event.message_id.is_empty()
            || event.message_id.len() > 180
            || event.message_id.chars().any(char::is_control)
            || event.text.len() > 4096
            || (edit_state == "opaque" && !event.text.is_empty())
        {
            return Ok(None);
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        if now.abs_diff(event.timestamp) > 120 {
            return Ok(None);
        }
        let mut signed = SignedFrame {
            version: 1,
            socket_id: self.socket_id.clone(),
            actor_id: if event.from_me {
                self.actor_id.clone()
            } else {
                event.sender_jid.to_owned()
            },
            owner_jid: self.owner_jid.clone(),
            chat_jid: event.chat_jid.to_owned(),
            message_id: event.message_id.to_owned(),
            text: format!("\u{001f}matter-anti:{edit_state}:{}", event.text),
            from_me: event.from_me,
            timestamp: event.timestamp,
            signature: String::new(),
        };
        signed.signature = sign_frame(&signed, &self.bridge_key)?;
        Ok(Some(signed))
    }
    /// Signs only recent, self-authored messages in the owner's own private
    /// chat. All identifiers come from the trusted whatsapp-rust callback.
    pub fn signed_control(&self, event: &Event<'_>) -> Result<Option<SignedFrame>> {
        if !event.from_me
            || event.is_group
            || event.chat_jid != self.owner_jid
            || event.sender_jid != self.owner_jid
            || !event.text.starts_with("=/>")
        {
            return Ok(None);
        }
        if event.message_id.is_empty()
            || event.message_id.len() > 256
            || event.message_id.chars().any(char::is_control)
            || event.text.len() > 4096
        {
            bail!("Invalid WhatsApp control data");
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        if now.abs_diff(event.timestamp) > 120 {
            bail!("Expired WhatsApp control data");
        }
        let mut result = SignedFrame {
            version: 1,
            socket_id: self.socket_id.clone(),
            actor_id: self.actor_id.clone(),
            owner_jid: self.owner_jid.clone(),
            chat_jid: self.owner_jid.clone(),
            message_id: event.message_id.to_owned(),
            text: event.text.to_owned(),
            from_me: true,
            timestamp: event.timestamp,
            signature: String::new(),
        };
        result.signature = sign_frame(&result, &self.bridge_key)?;
        Ok(Some(result))
    }
}

pub fn sign_frame(frame: &SignedFrame, secret: &[u8; 32]) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .map_err(|_| anyhow!("Bridge signature unavailable"))?;
    mac.update(DOMAIN);
    let values = [
        frame.version.to_string(),
        frame.socket_id.clone(),
        frame.actor_id.clone(),
        frame.owner_jid.clone(),
        frame.chat_jid.clone(),
        frame.message_id.clone(),
        frame.text.clone(),
    ];
    for value in &values {
        let bytes = value.as_bytes();
        mac.update(&(u32::try_from(bytes.len())?).to_be_bytes());
        mac.update(bytes);
    }
    mac.update(&[u8::from(frame.from_me)]);
    mac.update(&frame.timestamp.to_be_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

/// Require a pre-existing private directory; never use a relative database.
pub fn private_path(raw: &str) -> Result<PathBuf> {
    let path = PathBuf::from(raw);
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
        || path.file_name().is_none()
    {
        bail!("Session path must be absolute without parent traversal");
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("Session directory missing"))?;
    let info = fs::symlink_metadata(parent)?;
    if !info.is_dir() || info.file_type().is_symlink() {
        bail!("Session directory must be an ordinary directory");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if info.permissions().mode() & 0o077 != 0 {
            bail!("Session directory must be accessible to the owner only");
        }
    }
    if let Ok(info) = fs::symlink_metadata(&path) {
        if !info.is_file() || info.file_type().is_symlink() {
            bail!("Session database must be an ordinary file");
        }
    }
    Ok(path)
}

const SESSION_LEASE_MARKER: &[u8] = b"Base-Matter:WhatsApp-Session-Lease:v2\n";

/// A stable, version-marked lock file. The OS lock is released on process
/// termination, including a crash, without unlinking/replacing its inode.
pub struct SessionLease {
    _file: File,
}

impl SessionLease {
    fn existing(path: &Path) -> Result<File> {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.file_type().is_file() || metadata.len() != SESSION_LEASE_MARKER.len() as u64 {
            bail!("Legacy or unsafe session lock requires verified recovery");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                bail!("Session lock must be private");
            }
        }
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut marker = Vec::new();
        file.read_to_end(&mut marker)?;
        if marker != SESSION_LEASE_MARKER {
            bail!("Legacy or unsafe session lock requires verified recovery");
        }
        Ok(file)
    }

    /// No writes or file creation during preflight.
    pub fn check_available(database: &Path) -> Result<()> {
        let path = database.with_extension("owner.lock");
        match fs::symlink_metadata(&path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => bail!("Session lock cannot be inspected"),
            Ok(_) => {}
        };
        let file = Self::existing(&path)?;
        file.try_lock()
            .map_err(|_| anyhow!("Session already owned by another process"))?;
        Ok(())
    }

    pub fn acquire(database: &Path) -> Result<Self> {
        let path = database.with_extension("owner.lock");
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let (mut file, fresh) = match options.open(&path) {
            Ok(file) => (file, true),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                (Self::existing(&path)?, false)
            }
            Err(_) => bail!("Session lock could not be opened"),
        };
        file.try_lock()
            .map_err(|_| anyhow!("Session already owned by another process"))?;
        if fresh {
            file.write_all(SESSION_LEASE_MARKER)?;
            file.sync_all()?;
        }
        Ok(Self { _file: file })
    }
}

/// An exact first-line marker for the official REGISTER subgroup.
/// Community membership and admin identity MUST be independently verified
/// from authenticated WhatsApp metadata before this predicate is used.
pub fn official_register_description(description: &str, community_jid: &str) -> bool {
    if community_jid.is_empty()
        || community_jid.len() > 128
        || !community_jid.ends_with("@g.us")
        || !community_jid
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"@._:-".contains(&byte))
    {
        return false;
    }
    let expected = format!("MATTER:OFFICIAL:register:{community_jid}");
    description.split('\n').next() == Some(expected.as_str())
}
