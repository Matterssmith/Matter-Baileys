/*
 * WhatsApp-Matter — offline sticker-annotation protobuf builder.
 * Does not send messages or grant roles. Vault authorization is required
 * by the future caller before using an envelope on a live session.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use anyhow::{bail, Result};
use buffa::MessageField;
use whatsapp_rust::prelude::wa;

fn valid_jid(value: &str) -> bool {
    let domain_ok =
        value.ends_with("@g.us") || value.ends_with("@s.whatsapp.net") || value.ends_with("@lid");
    domain_ok
        && !value.starts_with('@')
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@._:-".contains(&b))
}

fn valid_message_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b))
}

/// Compose the non-transmitting WhatsApp protobuf envelope for sticker annotation.
///
/// This is a *serialization primitive*, not permission to publish a badge.
/// Never call it with unverified message keys, an untrusted sticker asset,
/// or a guessed Vault role. No production code calls this function.
pub fn offline_sticker_badge_proto(
    parent: &wa::MessageKey,
    sticker: &wa::message::StickerMessage,
) -> Result<wa::Message> {
    let remote = parent.remote_jid.as_deref().unwrap_or_default();
    let id = parent.id.as_deref().unwrap_or_default();
    if !valid_jid(remote) || !valid_message_id(id) || parent.from_me.is_none() {
        bail!("Unverified or invalid parent message key");
    }
    let group = remote.ends_with("@g.us");
    let participant = parent.participant.as_deref();
    if group != participant.is_some() || participant.is_some_and(|p| !valid_jid(p)) {
        bail!("Parent participant does not match chat scope");
    }
    let media_key = sticker.media_key.as_deref().unwrap_or_default();
    let file_sha256 = sticker.file_sha256.as_deref().unwrap_or_default();
    let file_enc_sha256 = sticker.file_enc_sha256.as_deref().unwrap_or_default();
    let direct_path = sticker.direct_path.as_deref().unwrap_or_default();
    if media_key.len() != 32
        || file_sha256.len() != 32
        || file_enc_sha256.len() != 32
        || !direct_path.starts_with('/')
        || direct_path.len() > 512
        || direct_path.chars().any(char::is_control)
        || sticker.mimetype.as_deref() != Some("image/webp")
    {
        bail!("Unverified or incomplete sticker metadata");
    }

    let association = wa::MessageAssociation {
        association_type: Some(wa::message_association::AssociationType::STICKER_ANNOTATION),
        parent_message_key: MessageField::some(parent.clone()),
        ..Default::default()
    };
    Ok(wa::Message {
        message_context_info: MessageField::some(wa::MessageContextInfo {
            message_association: MessageField::some(association),
            ..Default::default()
        }),
        sticker_message: MessageField::some(sticker.clone()),
        ..Default::default()
    })
}
