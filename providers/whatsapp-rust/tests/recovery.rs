/*
 * WhatsApp-Matter — encrypted community recovery never leaks member JIDs.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use std::fs;
use whatsapp_matter_rust_provider::recovery::{
    read_encrypted, save_encrypted, RecoveryMember, RecoverySnapshot,
};

fn fixture() -> (std::path::PathBuf, RecoverySnapshot) {
    let mut nonce = [0_u8; 12];
    getrandom::fill(&mut nonce).unwrap();
    let path = std::env::temp_dir().join(format!(
        "base-matter-recovery-test-{}-{}",
        std::process::id(),
        hex::encode(nonce)
    ));
    fs::create_dir(&path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let snapshot = RecoverySnapshot {
        version: 1,
        socket_id: "wa-test".into(),
        owner_jid: "5215512345678@s.whatsapp.net".into(),
        community_jid: "120363000123456@g.us".into(),
        community_name: "Mi comunidad".into(),
        captured_at: 1_793_000_000,
        members: vec![RecoveryMember {
            jid: "5215598765432@s.whatsapp.net".into(),
            phone_jid: Some("5215598765432@s.whatsapp.net".into()),
            lid_jid: Some("99887766@lid".into()),
        }],
    };
    (path, snapshot)
}

#[test]
fn roundtrip_is_ciphertext_only_and_bound_to_owner() {
    let (dir, snapshot) = fixture();
    let key = [0x47_u8; 32];
    let db = dir.join("session.db");
    let path = save_encrypted(&db, &key, &snapshot).unwrap();
    let saved = fs::read(&path).unwrap();
    assert!(!saved
        .windows(b"5215598765432".len())
        .any(|w| w == b"5215598765432"));
    assert!(!saved
        .windows(b"Mi comunidad".len())
        .any(|w| w == b"Mi comunidad"));
    let recovered = read_encrypted(&path, &key, &snapshot.socket_id, &snapshot.owner_jid).unwrap();
    assert_eq!(recovered, snapshot);
    assert!(read_encrypted(&path, &[0x13; 32], &snapshot.socket_id, &snapshot.owner_jid).is_err());
    assert!(read_encrypted(&path, &key, "wrong-socket", &snapshot.owner_jid).is_err());
    assert!(read_encrypted(&path, &key, &snapshot.socket_id, "wrong-owner").is_err());
    assert!(save_encrypted(&db, &key, &snapshot).is_ok());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0);
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_untrusted_member_identity_and_file_edits() {
    let (dir, mut snapshot) = fixture();
    let key = [0x22_u8; 32];
    let db = dir.join("session.db");
    snapshot.members[0].jid = "../other".to_owned();
    assert!(save_encrypted(&db, &key, &snapshot).is_err());
    snapshot.members[0].jid = "5215598765432@s.whatsapp.net".to_owned();
    let path = save_encrypted(&db, &key, &snapshot).unwrap();
    let mut data = fs::read(&path).unwrap();
    let n = data.len() - 5;
    data[n] ^= 1;
    fs::write(&path, data).unwrap();
    assert!(read_encrypted(&path, &key, &snapshot.socket_id, &snapshot.owner_jid).is_err());
    fs::remove_dir_all(dir).unwrap();
}

fn provider_binary() -> std::path::PathBuf {
    // Cargo does not always export this while compiling all targets.
    // Prefer the compile-time value when present, otherwise resolve it
    // from the integration test's runtime environment.
    option_env!("CARGO_BIN_EXE_whatsapp-matter-rust-provider")
        .map(Into::into)
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_whatsapp-matter-rust-provider").map(Into::into))
        .expect("Cargo did not supply the WhatsApp Rust provider binary")
}

#[test]
fn offline_cli_verifies_archive_without_starting_whatsapp() {
    use std::process::Command;

    let (dir, snapshot) = fixture();
    let key = [0x52_u8; 32];
    let db = dir.join("session.db");
    let file = save_encrypted(&db, &key, &snapshot).unwrap();
    let filename = file.file_name().unwrap().to_string_lossy().into_owned();

    let binary = provider_binary();
    let mut command = Command::new(&binary);
    command
        .arg("--recovery-verify")
        .arg(&filename)
        .env("MATTER_WA_PHONE", "5215512345678")
        .env("MATTER_WA_OWNER_JID", &snapshot.owner_jid)
        .env("MATTER_WA_SOCKET_ID", &snapshot.socket_id)
        .env("MATTER_WA_OWNER_ID", "owner")
        .env("MATTER_WA_SESSION_DB", &db)
        .env("MATTER_WA_BRIDGE_ADDR", "127.0.0.1:39819")
        .env("MATTER_WA_BRIDGE_KEY_HEX", "07".repeat(32))
        .env("MATTER_WA_RECOVERY_KEY_HEX", hex::encode(key));
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "Offline archive inspection failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("1 member records"));
    assert!(!stdout.contains("5215598765432"));
    assert!(!db.exists(), "Offline verification must not open a session");

    let denied = command
        .env("MATTER_WA_RECOVERY_KEY_HEX", "ab".repeat(32))
        .output()
        .unwrap();
    assert!(!denied.status.success());

    let traversal = Command::new(&binary)
        .args(["--recovery-verify", "../unsafe"])
        .output()
        .unwrap();
    assert!(!traversal.status.success());
    fs::remove_dir_all(dir).unwrap();
}
