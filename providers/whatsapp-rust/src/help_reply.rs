/*
 * WhatsApp-Matter — read-only help from replied-to command text.
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use whatsapp_rust::prelude::{wa, MessageExt};

/// Only the user's literal '=/>help' turns the quoted message into metadata.
/// Any quotation that points to another chat is ignored. The quoted payload
/// never becomes a command invocation; it is encoded data for the help parser.
pub fn help_from_quoted_message(
    message: &wa::Message,
    text: &str,
    verified_chat: &str,
) -> Option<String> {
    if !matches!(text.trim(), "=/>help" | "=/>h" | "=/>ayuda") {
        return None;
    }
    let base = message.get_base_message();
    let context = base
        .extended_text_message
        .as_option()?
        .context_info
        .as_option()?;
    if context
        .remote_jid
        .as_deref()
        .is_some_and(|chat| chat != verified_chat)
    {
        return None;
    }
    let quoted = context.quoted_message.as_option()?.text_content()?;
    if quoted.len() > 1000 || quoted.is_empty() || quoted.contains('\0') {
        return None;
    }
    let encoded = URL_SAFE_NO_PAD.encode(quoted);
    Some(format!("=/>help --quoted {encoded}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::MessageField;

    fn quoted(text: &str, remote: Option<&str>) -> wa::Message {
        wa::Message {
            extended_text_message: MessageField::some(wa::message::ExtendedTextMessage {
                text: Some("=/>help".into()),
                context_info: MessageField::some(wa::ContextInfo {
                    stanza_id: Some("original-1".into()),
                    remote_jid: remote.map(str::to_owned),
                    quoted_message: MessageField::some(wa::Message {
                        conversation: Some(text.to_owned()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn quoting_a_command_builds_only_a_help_query() {
        let chat = "123456789@g.us";
        let message = quoted("=/>set anti link add t.me", Some(chat));
        let result = help_from_quoted_message(&message, "=/>help", chat).unwrap();
        assert!(result.starts_with("=/>help --quoted "));
        let raw = URL_SAFE_NO_PAD
            .decode(result.split_whitespace().last().unwrap())
            .unwrap();
        assert_eq!(raw, b"=/>set anti link add t.me");
        assert!(help_from_quoted_message(&message, "=/>bot on", chat).is_none());
        assert!(help_from_quoted_message(&message, "=/>help", "other@g.us").is_none());
    }

    #[test]
    fn no_quote_does_not_synthesize_help() {
        let message = wa::Message {
            conversation: Some("=/>help".into()),
            ..Default::default()
        };
        assert!(help_from_quoted_message(&message, "=/>help", "123@g.us").is_none());
    }
}
