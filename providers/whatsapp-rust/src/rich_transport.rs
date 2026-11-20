/*
 * WhatsApp-Matter — opt-in RichResponse transport experiment.
 *
 * This path serializes independently-generated HTML preview data into the
 * public whatsapp-rust protobuf. It never claims to be Meta AI, never
 * forwards a non-forwarded message, and never operates without the owner's
 * explicit environment opt-in and authenticated private self-chat context.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use crate::rich_preview::experimental_rich_schema;
use anyhow::{anyhow, bail, Result};
use buffa::MessageField;
use whatsapp_rust::prelude::wa;

const MAX_WIRE_BYTES: usize = 16_384;

/// Build, but do not send, a non-forged experimental RichResponse message.
/// A client can ignore or discard these unsupported UI primitives even when
/// the API reports a successful send. NativeFlow/text remain the fallback.
pub fn experimental_self_chat_rich_message(menu: &str) -> Result<wa::Message> {
    let mut payload = experimental_rich_schema(menu)?;
    let mut nonce = [0_u8; 16];
    getrandom::fill(&mut nonce).map_err(|_| anyhow!("Rich response ID unavailable"))?;
    payload["response_id"] = format!("matterssmith-{}", hex::encode(nonce)).into();
    let bytes = serde_json::to_vec(&payload)?;
    if bytes.is_empty() || bytes.len() > MAX_WIRE_BYTES {
        bail!("Experimental HTML response exceeds transport limit");
    }
    let rich = wa::AIRichResponseMessage {
        message_type: Some(wa::AIRichResponseMessageType::AiRichResponseTypeStandard),
        submessages: vec![wa::AIRichResponseSubMessage {
            message_type: Some(wa::AIRichResponseSubMessageType::AiRichResponseText),
            message_text: Some("Matterssmith · Base-Matter".to_owned()),
            ..Default::default()
        }],
        unified_response: MessageField::some(wa::AIRichResponseUnifiedResponse {
            data: Some(bytes),
        }),
        ..Default::default()
    };
    // Deliberately NOT a nested BotForwardedMessage. No fabricated 0@bot,
    // forwarded flag, Meta AI origin, or disguised trusted source.
    Ok(wa::Message {
        rich_response_message: MessageField::some(rich),
        ..Default::default()
    })
}
