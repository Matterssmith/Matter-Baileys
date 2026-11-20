/*
 * WhatsApp-Matter — authenticated encrypted community recovery snapshots.
 * Credits are project-owned. No WhatsApp sessions or plaintext member lists
 * are written by this module.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use anyhow::{anyhow, bail, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

const DOMAIN: &str = "Base-Matter:Community-Recovery:v1";
const MAX_MEMBERS: usize = 15_000;
const MAX_BYTES: usize = 5 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryMember {
    pub jid: String,
    pub phone_jid: Option<String>,
    pub lid_jid: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverySnapshot {
    pub version: u8,
    pub socket_id: String,
    pub owner_jid: String,
    pub community_jid: String,
    pub community_name: String,
    pub captured_at: u64,
    pub members: Vec<RecoveryMember>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EncryptedEnvelope {
    version: u8,
    nonce: String,
    ciphertext: String,
}

fn aad(socket_id: &str, owner_jid: &str) -> Vec<u8> {
    format!("{DOMAIN}\0{socket_id}\0{owner_jid}").into_bytes()
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.chars().any(char::is_control)
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@._:-".contains(&b))
}

fn validate(snapshot: &RecoverySnapshot) -> Result<()> {
    if snapshot.version != 1
        || !valid_id(&snapshot.socket_id)
        || !valid_id(&snapshot.owner_jid)
        || !snapshot.community_jid.ends_with("@g.us")
        || !valid_id(&snapshot.community_jid)
        || snapshot.community_name.is_empty()
        || snapshot.community_name.len() > 256
        || snapshot.community_name.chars().any(char::is_control)
        || snapshot.members.len() > MAX_MEMBERS
    {
        bail!("Invalid recovery snapshot");
    }
    for member in &snapshot.members {
        if !valid_id(&member.jid)
            || member.phone_jid.as_deref().is_some_and(|id| !valid_id(id))
            || member.lid_jid.as_deref().is_some_and(|id| !valid_id(id))
        {
            bail!("Invalid recovery member");
        }
    }
    Ok(())
}

/// Separate 256-bit recovery key. Never derive it from the WhatsApp session
/// or use the same key as the authenticated control bridge.
pub fn recovery_key_from_env(bridge_key: &[u8; 32]) -> Result<Zeroizing<[u8; 32]>> {
    let raw = Zeroizing::new(
        std::env::var("MATTER_WA_RECOVERY_KEY_HEX")
            .map_err(|_| anyhow!("Recovery key is not configured"))?,
    );
    if raw.len() != 64 || !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("Invalid recovery key");
    }
    let mut key = Zeroizing::new([0_u8; 32]);
    hex::decode_to_slice(raw.as_bytes(), &mut key[..])
        .map_err(|_| anyhow!("Invalid recovery key"))?;
    if key.iter().all(|b| *b == 0) || &*key == bridge_key {
        bail!("Recovery key must be nonzero and separate from the bridge key");
    }
    Ok(key)
}

fn verify_directory(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.file_type().is_dir() {
        bail!("Recovery directory must be ordinary");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            bail!("Recovery directory must be owner-only");
        }
    }
    Ok(())
}

/// Save a new ciphertext-only, randomly named file beside the private session
/// DB. No existing snapshot is overwritten. The key is never persisted here.
pub fn save_encrypted(
    session_db: &Path,
    secret: &[u8; 32],
    snapshot: &RecoverySnapshot,
) -> Result<PathBuf> {
    validate(snapshot)?;
    if secret.iter().all(|b| *b == 0) {
        bail!("Invalid recovery key");
    }
    let parent = session_db
        .parent()
        .ok_or_else(|| anyhow!("Session directory missing"))?;
    verify_directory(parent)?;

    let cleartext = Zeroizing::new(serde_json::to_vec(snapshot)?);
    if cleartext.len() > MAX_BYTES {
        bail!("Recovery snapshot exceeds encrypted storage limit");
    }
    let mut nonce = [0_u8; 24];
    getrandom::fill(&mut nonce).map_err(|_| anyhow!("Recovery nonce unavailable"))?;
    let cipher = XChaCha20Poly1305::new_from_slice(secret)
        .map_err(|_| anyhow!("Recovery encryption unavailable"))?;
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &cleartext,
                aad: &aad(&snapshot.socket_id, &snapshot.owner_jid),
            },
        )
        .map_err(|_| anyhow!("Recovery encryption failed"))?;
    let envelope = serde_json::to_vec(&EncryptedEnvelope {
        version: 1,
        nonce: hex::encode(nonce),
        ciphertext: hex::encode(ciphertext),
    })?;
    let id_digest = Sha256::digest(snapshot.community_jid.as_bytes());
    let filename = format!(
        "community-recovery-{}-{}-{}.enc.json",
        hex::encode(&id_digest[..8]),
        snapshot.captured_at,
        hex::encode(&nonce[..8])
    );
    let destination = parent.join(filename);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&destination)
        .map_err(|_| anyhow!("Recovery snapshot could not be created"))?;
    file.write_all(&envelope)?;
    file.sync_all()?;
    Ok(destination)
}

/// Read only an authenticated snapshot for this exact socket/owner.
/// A wrong key, altered ciphertext or mismatched owner fails closed.
pub fn read_encrypted(
    path: &Path,
    secret: &[u8; 32],
    expected_socket: &str,
    expected_owner: &str,
) -> Result<RecoverySnapshot> {
    if !valid_id(expected_socket) || !valid_id(expected_owner) {
        bail!("Invalid recovery identity");
    }
    let meta = fs::symlink_metadata(path)?;
    if !meta.file_type().is_file() || meta.len() > (MAX_BYTES as u64 * 2 + 2048) {
        bail!("Unsafe recovery file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            bail!("Recovery file is not private");
        }
    }
    let raw = fs::read(path)?;
    let envelope: EncryptedEnvelope =
        serde_json::from_slice(&raw).map_err(|_| anyhow!("Invalid encrypted snapshot"))?;
    if envelope.version != 1 {
        bail!("Unsupported recovery snapshot version");
    }
    let bytes = hex::decode(&envelope.nonce).map_err(|_| anyhow!("Invalid recovery nonce"))?;
    if bytes.len() != 24 {
        bail!("Invalid recovery nonce");
    }
    let ciphertext =
        hex::decode(&envelope.ciphertext).map_err(|_| anyhow!("Invalid recovery ciphertext"))?;
    let cipher =
        XChaCha20Poly1305::new_from_slice(secret).map_err(|_| anyhow!("Recovery key invalid"))?;
    let cleartext = Zeroizing::new(
        cipher
            .decrypt(
                XNonce::from_slice(&bytes),
                Payload {
                    msg: &ciphertext,
                    aad: &aad(expected_socket, expected_owner),
                },
            )
            .map_err(|_| anyhow!("Recovery authentication failed"))?,
    );
    let snapshot: RecoverySnapshot =
        serde_json::from_slice(&cleartext).map_err(|_| anyhow!("Invalid recovery payload"))?;
    validate(&snapshot)?;
    if snapshot.socket_id != expected_socket || snapshot.owner_jid != expected_owner {
        bail!("Recovery owner does not match");
    }
    Ok(snapshot)
}
