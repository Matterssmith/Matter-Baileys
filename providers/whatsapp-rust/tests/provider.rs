/*
 * WhatsApp-Matter — offline provider signing, configuration and lease tests.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use std::{collections::HashMap, fs};
use whatsapp_matter_rust_provider::{
    parse_control_ack, sign_frame, Config, ControlAck, Event, SessionLease, SignedFrame,
};

const OWNER: &str = "5215512345678@s.whatsapp.net";

fn fixture() -> (HashMap<String, String>, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "whatsapp-matter-provider-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let mut values = HashMap::new();
    for (name, value) in [
        ("MATTER_WA_PHONE", "5215512345678"),
        ("MATTER_WA_OWNER_JID", OWNER),
        ("MATTER_WA_SOCKET_ID", "socket-1"),
        ("MATTER_WA_OWNER_ID", "owner"),
        ("MATTER_WA_BRIDGE_ADDR", "127.0.0.1:39819"),
    ] {
        values.insert(name.to_owned(), value.to_owned());
    }
    values.insert(
        "MATTER_WA_SESSION_DB".into(),
        root.join("session.db").to_string_lossy().into_owned(),
    );
    values.insert("MATTER_WA_BRIDGE_KEY_HEX".into(), "07".repeat(32));
    (values, root)
}

fn event<'a>(text: &'a str) -> Event<'a> {
    Event {
        chat_jid: OWNER,
        sender_jid: OWNER,
        message_id: "msg-1",
        text,
        from_me: true,
        is_group: false,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    }
}

#[test]
fn standalone_provider_uses_exact_same_external_matterssmith_template() {
    let text = whatsapp_matter_rust_provider::render_provider_control(
        "REGISTER: solo lectura, sin cambios en grupos.",
    )
    .unwrap();
    assert!(text.starts_with("╭─💠 𝗖𝗼𝗻𝘁𝗿𝗼𝗹"));
    assert!(text.contains("REGISTER: solo lectura"));
    assert!(text.contains('\n'));
    assert!(!text.contains("\\n"));
    assert!(!text.contains("Lournyx Matterssmith [Lou-Nyx]"));
    assert_eq!(
        whatsapp_matter_rust_provider::render_provider_control(&text).unwrap(),
        text
    );
}

#[test]
fn creates_valid_signed_owner_control_only_for_private_self_chat() {
    let (vars, root) = fixture();
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    let frame = c.signed_control(&event("=/>status")).unwrap().unwrap();
    assert_eq!(frame.owner_jid, OWNER);
    assert_eq!(frame.text, "=/>status");
    assert_eq!(frame.signature.len(), 64);
    let mut bad = event("=/>bot on");
    bad.from_me = false;
    assert!(c.signed_control(&bad).unwrap().is_none());
    bad.from_me = true;
    bad.is_group = true;
    assert!(c.signed_control(&bad).unwrap().is_none());
    bad.is_group = false;
    bad.chat_jid = "other@s.whatsapp.net";
    assert!(c.signed_control(&bad).unwrap().is_none());
    assert!(c.signed_control(&event("#ping")).unwrap().is_none());
    // The outbound owner-chat acknowledgment must not be re-interpreted as a control.
    assert!(c
        .signed_control(&event("Base-Matter: solicitud procesada."))
        .unwrap()
        .is_none());
    fs::remove_dir_all(root).unwrap();
}

const VERIFIED_LID: &str = "987654321012345@lid";

#[test]
fn verified_client_accepts_self_chat_in_pn_or_lid_namespaces() {
    let (vars, root) = fixture();
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    for (chat, sender) in [
        (OWNER, OWNER),
        (VERIFIED_LID, VERIFIED_LID),
        (OWNER, VERIFIED_LID),
        (VERIFIED_LID, OWNER),
    ] {
        let mut message = event("=/>status");
        message.chat_jid = chat;
        message.sender_jid = sender;
        let signed = c
            .signed_verified_control(&message, Some(OWNER), Some(VERIFIED_LID))
            .unwrap()
            .expect("verified private owner event should be signed");
        assert_eq!(signed.chat_jid, OWNER);
        assert_eq!(signed.owner_jid, OWNER);
        assert_eq!(signed.text, "=/>status");
        assert_eq!(signed.signature.len(), 64);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn verified_client_rejects_wrong_phone_foreign_chat_and_forged_lid() {
    let (vars, root) = fixture();
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    let message = event("=/>bot on");
    assert!(c
        .signed_verified_control(&message, None, Some(VERIFIED_LID))
        .is_err());
    assert!(c
        .signed_verified_control(
            &message,
            Some("999999999999@s.whatsapp.net"),
            Some(VERIFIED_LID),
        )
        .is_err());

    let mut forged = event("=/>bot on");
    forged.chat_jid = "other@s.whatsapp.net";
    assert!(c
        .signed_verified_control(&forged, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());

    forged.chat_jid = VERIFIED_LID;
    forged.sender_jid = "other@s.whatsapp.net";
    assert!(c
        .signed_verified_control(&forged, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());

    forged.sender_jid = VERIFIED_LID;
    forged.from_me = false;
    assert!(c
        .signed_verified_control(&forged, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());

    forged.from_me = true;
    forged.is_group = true;
    assert!(c
        .signed_verified_control(&forged, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());

    forged.is_group = false;
    forged.text = "#ping";
    assert!(c
        .signed_verified_control(&forged, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn verified_client_allows_configured_lid_only_when_it_matches_session() {
    let (mut vars, root) = fixture();
    vars.insert("MATTER_WA_OWNER_JID".to_owned(), VERIFIED_LID.to_owned());
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    let mut message = event("=/>status");
    message.chat_jid = VERIFIED_LID;
    message.sender_jid = VERIFIED_LID;

    assert!(c
        .signed_verified_control(&message, Some(OWNER), None)
        .is_err());
    let signed = c
        .signed_verified_control(&message, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .unwrap();
    assert_eq!(signed.owner_jid, VERIFIED_LID);
    assert_eq!(signed.chat_jid, VERIFIED_LID);

    // Menu is available in the authenticated private context, subject
    // to the controller's effective bot activation policy.
    message.text = "=/>menu";
    assert!(c
        .signed_verified_control(&message, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_some());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn verified_owner_group_controls_are_signed_for_exact_group_jid() {
    let (vars, root) = fixture();
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    let group = "120363123456789@g.us";
    let mut event = event("=/>bot on");
    event.chat_jid = group;
    event.sender_jid = VERIFIED_LID;
    event.is_group = true;
    let frame = c
        .signed_verified_control(&event, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .expect("group owner control should be delivered");
    assert_eq!(frame.chat_jid, group);
    assert_eq!(frame.owner_jid, OWNER);
    assert_eq!(frame.signature, sign_frame(&frame, &c.bridge_key).unwrap());
    event.text = "=/>menu";
    assert!(c
        .signed_verified_control(&event, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_some());
    for probe in ["=/>test", "=/>test ui", "=/>test row"] {
        event.text = probe;
        assert!(c
            .signed_verified_control(&event, Some(OWNER), Some(VERIFIED_LID))
            .unwrap()
            .is_some());
    }
    event.text = "=/>on";
    assert!(c
        .signed_verified_control(&event, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());
    event.text = "=/>bot on";
    event.sender_jid = "foreign@s.whatsapp.net";
    assert!(c
        .signed_verified_control(&event, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());
    event.sender_jid = OWNER;
    event.from_me = false;
    assert!(c
        .signed_verified_control(&event, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());
    event.from_me = true;
    event.chat_jid = "other@s.whatsapp.net";
    assert!(c
        .signed_verified_control(&event, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn verified_private_owner_chat_does_not_sign_group_only_commands() {
    let (vars, root) = fixture();
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    for command in ["=/>on", "=/>off", "=/>start", "=/>add++", "=/>group on"] {
        assert!(c
            .signed_verified_control(&event(command), Some(OWNER), Some(VERIFIED_LID))
            .unwrap()
            .is_none());
    }
    assert!(c
        .signed_verified_control(&event("=/>status"), Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_some());
    for allowed in [
        "=/>menu",
        "=/>menú",
        "=/>help",
        "=/>h",
        "=/>help all",
        "=/>help antis",
        "=/>menu set",
        "=/>menu antis",
        "=/>menu antis 2",
        "=/>menu system",
        "=/>test",
        "=/>test ui",
        "=/>test row",
        "=/>test html",
        "=/>test ack a",
        "=/>bot",
        "=/>bot on",
        "=/>bot off",
        "=/>estado",
        "=/>credits",
    ] {
        assert!(c
            .signed_verified_control(&event(allowed), Some(OWNER), Some(VERIFIED_LID))
            .unwrap()
            .is_some());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn incoming_readonly_commands_bind_real_sender_and_never_gain_owner_authority() {
    let (vars, root) = fixture();
    let config = Config::from_reader(|key| vars.get(key).cloned()).unwrap();
    let public_id = "987654321012346@lid";
    let mut incoming = event("=/>status");
    incoming.from_me = false;
    incoming.is_group = true;
    incoming.chat_jid = "120363111222333@g.us";
    incoming.sender_jid = public_id;
    // Every canonical category and all three selector pages must be
    // accepted for authenticated public readers without granting writes.
    for command in [
        "=/>menu ia",
        "=/>menu anti",
        "=/>menu all",
        "=/>menu config",
        "=/>menu downloads",
        "=/>menu economy",
        "=/>menu fun",
        "=/>menu gacha",
        "=/>menu games",
        "=/>menu group",
        "=/>menu info",
        "=/>menu links",
        "=/>menu main",
        "=/>menu mode",
        "=/>menu owner",
        "=/>menu profile",
        "=/>menu reactions",
        "=/>menu search",
        "=/>menu sockets",
        "=/>menu system",
        "=/>menu utils",
        "=/>menu page 1",
        "=/>menu page 2",
        "=/>menu page 3",
    ] {
        incoming.text = command;
        assert!(
            config
                .signed_public_readonly(&incoming, Some(OWNER), Some(VERIFIED_LID))
                .unwrap()
                .is_some(),
            "read-only menu was rejected: {command}"
        );
    }
    for text in [
        "=/>status",
        "=/>estado",
        "=/>status --global",
        "=/>menu",
        "=/>menú",
        "=/>bot",
        "=/>credits",
        "=/>help antilink",
        "=/>help menu",
        "=/>help all",
        "=/>help antis 2",
        "=/>test",
        "=/>test ui",
        "=/>test row",
        "=/>test ack audio",
    ] {
        incoming.text = text;
        let signed = config
            .signed_public_readonly(&incoming, Some(OWNER), Some(VERIFIED_LID))
            .unwrap()
            .expect("read-only public command should be signed");
        assert_eq!(signed.actor_id, public_id);
        assert_eq!(signed.chat_jid, incoming.chat_jid);
        assert!(!signed.from_me);
        assert_eq!(signed.owner_jid, OWNER);
        assert_eq!(
            signed.signature,
            sign_frame(&signed, &config.bridge_key).unwrap()
        );
    }
    for text in [
        "=/>status --debug",
        "=/>group on",
        "=/>bot on",
        "=/>set role 12345678901 admin",
        "=/>menu unknown",
        "=/>menu antis 999",
        "=/>menu unset",
        "=/>menu config extra",
        "=/>help run(unknown)",
        "=/>test html",
        "=/>test run",
        "=/>test ack invalid",
        "#ping",
    ] {
        incoming.text = text;
        assert!(config
            .signed_public_readonly(&incoming, Some(OWNER), Some(VERIFIED_LID))
            .unwrap()
            .is_none());
    }
    incoming.text = "=/>status";
    assert!(config
        .signed_public_readonly(&incoming, None, Some(VERIFIED_LID))
        .is_err());
    incoming.sender_jid = "someone@s.whatsapp.net";
    assert!(config
        .signed_public_readonly(&incoming, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());
    incoming.sender_jid = public_id;
    incoming.chat_jid = "other@s.whatsapp.net";
    assert!(config
        .signed_public_readonly(&incoming, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());
    incoming.chat_jid = public_id;
    incoming.is_group = false;
    assert!(config
        .signed_public_readonly(&incoming, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_some());
    incoming.chat_jid = "another@s.whatsapp.net";
    assert!(config
        .signed_public_readonly(&incoming, Some(OWNER), Some(VERIFIED_LID))
        .unwrap()
        .is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn signature_matches_rust_base_control_protocol_fixture() {
    let value = SignedFrame {
        version: 1,
        socket_id: "socket-1".into(),
        actor_id: "owner".into(),
        owner_jid: OWNER.into(),
        chat_jid: OWNER.into(),
        message_id: "msg-1".into(),
        text: "=/>status".into(),
        from_me: true,
        timestamp: 1_794_873_600,
        signature: String::new(),
    };
    let mut key = [0; 32];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = (index + 1) as u8;
    }
    assert_eq!(
        sign_frame(&value, &key).unwrap(),
        "747dc915a2734e90d1f2042cc42df7e360f848efcacda76baa8f27925b3124c7"
    );
}

#[test]
fn rejects_missing_reused_or_public_bridge_credentials() {
    let (mut vars, root) = fixture();
    for (name, bad) in [
        ("MATTER_WA_PHONE", "+5215512345678"),
        ("MATTER_WA_OWNER_JID", "123@g.us"),
        ("MATTER_WA_BRIDGE_ADDR", "0.0.0.0:39819"),
        ("MATTER_WA_SOCKET_ID", "socket/1"),
    ] {
        let old = vars.insert(name.to_owned(), bad.to_owned()).unwrap();
        assert!(Config::from_reader(|name| vars.get(name).cloned()).is_err());
        vars.insert(name.to_owned(), old);
    }
    let old = vars.insert("MATTER_WA_BRIDGE_KEY_HEX".into(), "00".repeat(32));
    assert!(Config::from_reader(|name| vars.get(name).cloned()).is_err());
    vars.insert("MATTER_WA_BRIDGE_KEY_HEX".into(), old.unwrap());
    vars.remove("MATTER_WA_SESSION_DB");
    assert!(Config::from_reader(|name| vars.get(name).cloned()).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn exclusive_session_lock_never_removes_sqlite_file() {
    let (vars, root) = fixture();
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    fs::write(&c.session_db, b"existing session").unwrap();
    let lease = SessionLease::acquire(&c.session_db).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(c.session_db.with_extension("owner.lock"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o077, 0, "Session lock must be private at creation");
    }
    assert!(SessionLease::acquire(&c.session_db).is_err());
    assert!(SessionLease::check_available(&c.session_db).is_err());
    drop(lease);
    assert!(c.session_db.with_extension("owner.lock").exists());
    SessionLease::check_available(&c.session_db).unwrap();
    let resumed = SessionLease::acquire(&c.session_db).unwrap();
    assert!(SessionLease::acquire(&c.session_db).is_err());
    drop(resumed);
    assert_eq!(fs::read(&c.session_db).unwrap(), b"existing session");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn old_or_malformed_control_events_are_rejected() {
    let (vars, root) = fixture();
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    let mut old = event("=/>bot off");
    old.timestamp -= 500;
    assert!(c.signed_control(&old).is_err());
    let mut invalid = event("=/>bot on");
    invalid.message_id = "";
    assert!(c.signed_control(&invalid).is_err());
    fs::remove_dir_all(root).unwrap();
}

fn provider_binary() -> std::path::PathBuf {
    option_env!("CARGO_BIN_EXE_whatsapp-matter-rust-provider")
        .map(Into::into)
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_whatsapp-matter-rust-provider").map(Into::into))
        .expect("Cargo did not supply the WhatsApp Rust provider binary")
}

#[test]
fn preflight_validates_configuration_without_writing_a_session() {
    use std::process::Command;
    let (config, dir) = fixture();
    let result = Command::new(provider_binary())
        .arg("--check")
        .envs(&config)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "Rust provider preflight failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!dir.join("session.db").exists());
    assert!(!dir.join("session.owner.lock").exists());
    let mut invalid = config;
    invalid.insert("MATTER_WA_PHONE".into(), "+5215512345678".into());
    let denied = Command::new(provider_binary())
        .arg("--check")
        .envs(&invalid)
        .output()
        .unwrap();
    assert!(!denied.status.success());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn controller_ack_distinguishes_applied_and_duplicate_without_false_success() {
    assert_eq!(
        parse_control_ack(br#"{"ok":true,"outcome":"applied"}"#).unwrap(),
        ControlAck::Applied
    );
    assert_eq!(
        parse_control_ack(br#"{"ok":true,"outcome":"duplicate"}"#).unwrap(),
        ControlAck::Duplicate
    );
    assert_eq!(
        parse_control_ack(br#"{"ok":true,"outcome":"ignored"}"#).unwrap(),
        ControlAck::Ignored
    );
    assert_eq!(
        parse_control_ack(br#"{"ok":true,"outcome":"applied","message":"Estado"}"#).unwrap(),
        ControlAck::AppliedWithMessage("Estado".to_owned())
    );
    for invalid in [
        br#"{"ok":false,"outcome":"denied"}"#.as_slice(),
        br#"{"ok":true,"outcome":"denied"}"#,
        br#"{"ok":false,"outcome":"applied"}"#,
        br#"{"ok":true,"outcome":"duplicate","message":"bad"}"#,
        br#"{"ok":true,"outcome":"applied","message":""}"#,
        br#"{"ok":true,"outcome":"applied","message":"hi\ncontrol\u0000"}"#,
        br#"{"ok":true}"#,
        br#"{"ok":"true","outcome":"applied"}"#,
        br#"{"ok":true,"outcome":"applied","unexpected":1}"#,
        br#"{}"#,
        b"".as_slice(),
    ] {
        assert!(parse_control_ack(invalid).is_err());
    }
    assert!(parse_control_ack(&vec![b'a'; 1025]).is_err());
}

#[test]
fn long_category_menu_ack_is_bounded_and_does_not_accept_control_characters() {
    let menu = "✦ Menú Matterssmith\\n".repeat(70);
    assert!(menu.len() > 1024);
    assert!(menu.len() < 3800);
    let value = serde_json::json!({
        "ok": true, "outcome": "applied", "message": menu
    });
    let wire = serde_json::to_vec(&value).unwrap();
    assert_eq!(
        parse_control_ack(&wire).unwrap(),
        ControlAck::AppliedWithMessage(value["message"].as_str().unwrap().to_owned())
    );
    let too_long = serde_json::json!({
        "ok": true, "outcome": "applied", "message": "X".repeat(3801)
    });
    assert!(parse_control_ack(&serde_json::to_vec(&too_long).unwrap()).is_err());
    let unsafe_text = serde_json::json!({
        "ok": true, "outcome": "applied", "message": "hello\u{0000}"
    });
    assert!(parse_control_ack(&serde_json::to_vec(&unsafe_text).unwrap()).is_err());
}

#[test]
fn legacy_session_lock_is_not_silently_stolen() {
    let (vars, root) = fixture();
    let c = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    let lock = c.session_db.with_extension("owner.lock");
    fs::write(&lock, b"12345\n").unwrap();
    assert!(SessionLease::check_available(&c.session_db).is_err());
    assert!(SessionLease::acquire(&c.session_db).is_err());
    assert_eq!(fs::read(&lock).unwrap(), b"12345\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn group_anti_observation_is_signed_without_owner_privileges() {
    let (vars, root) = fixture();
    let config = Config::from_reader(|name| vars.get(name).cloned()).unwrap();
    let msg = Event {
        chat_jid: "120363777@g.us",
        sender_jid: "987654321012345@lid",
        message_id: "anti-original-1",
        text: "https://t․me/invite",
        from_me: false,
        is_group: true,
        timestamp: event("hello").timestamp,
    };
    let signed = config
        .signed_anti_observation(&msg, "original")
        .unwrap()
        .unwrap();
    assert!(signed.text.starts_with("\u{001f}matter-anti:original:"));
    assert_eq!(signed.actor_id, msg.sender_jid);
    assert!(!signed.from_me);
    assert_eq!(signed.chat_jid, msg.chat_jid);
    assert_eq!(
        signed.signature,
        sign_frame(&signed, &config.bridge_key).unwrap()
    );
    let mut self_edit = msg;
    self_edit.text = "";
    assert!(config
        .signed_anti_observation(&self_edit, "opaque")
        .unwrap()
        .is_some());
    self_edit.text = "=/>bot on";
    assert!(config
        .signed_anti_observation(&self_edit, "opaque")
        .unwrap()
        .is_none());
    self_edit.is_group = false;
    assert!(config
        .signed_anti_observation(&self_edit, "original")
        .unwrap()
        .is_none());
    fs::remove_dir_all(root).unwrap();
}
