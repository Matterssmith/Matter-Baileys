/*
 * WhatsApp-Matter — offline sticker association tests.
 * No WhatsApp session, provider credentials or Vault role grant is created.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use whatsapp_matter_rust_provider::sticker_badge::offline_sticker_badge_proto;
use whatsapp_rust::prelude::wa;

fn source() -> wa::MessageKey {
    wa::MessageKey {
        remote_jid: Some("120363123456789@g.us".to_owned()),
        from_me: Some(false),
        id: Some("ABC123".to_owned()),
        participant: Some("12345678901@s.whatsapp.net".to_owned()),
    }
}

fn sticker() -> wa::message::StickerMessage {
    wa::message::StickerMessage {
        mimetype: Some("image/webp".to_owned()),
        direct_path: Some("/v/t62.15575-24/test".to_owned()),
        media_key: Some(vec![7_u8; 32]),
        file_sha256: Some(vec![8_u8; 32]),
        file_enc_sha256: Some(vec![9_u8; 32]),
        ..Default::default()
    }
}

#[test]
fn packs_sticker_annotation_with_exact_parent_and_no_other_content() {
    let key = source();
    let body = sticker();
    let packet = offline_sticker_badge_proto(&key, &body).unwrap();
    let ctx = packet.message_context_info.as_option().unwrap();
    let association = ctx.message_association.as_option().unwrap();
    assert_eq!(
        association.association_type,
        Some(wa::message_association::AssociationType::STICKER_ANNOTATION)
    );
    assert_eq!(
        association.parent_message_key.as_option().unwrap().id,
        key.id
    );
    assert!(packet.sticker_message.as_option().is_some());
    assert!(packet.conversation.is_none());
}

#[test]
fn rejects_invalid_parent_or_missing_sticker_crypto() {
    let mut key = source();
    key.remote_jid = Some("bad.example".to_owned());
    assert!(offline_sticker_badge_proto(&key, &sticker()).is_err());

    let mut key = source();
    key.participant = None;
    assert!(offline_sticker_badge_proto(&key, &sticker()).is_err());

    let mut key = source();
    key.id = Some("".to_owned());
    assert!(offline_sticker_badge_proto(&key, &sticker()).is_err());

    let mut bad = sticker();
    bad.media_key = None;
    assert!(offline_sticker_badge_proto(&source(), &bad).is_err());

    let mut bad = sticker();
    bad.direct_path = None;
    assert!(offline_sticker_badge_proto(&source(), &bad).is_err());
}

#[test]
fn private_chat_parent_does_not_require_participant() {
    let mut key = source();
    key.remote_jid = Some("12345678901@s.whatsapp.net".to_owned());
    key.participant = None;
    let packet = offline_sticker_badge_proto(&key, &sticker()).unwrap();
    assert!(packet.message_context_info.as_option().is_some());
}
