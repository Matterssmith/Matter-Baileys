/*
 * WhatsApp-Matter — self-chat-only experimental rich transport regression.
 * Offline. No session, connection, impersonation or privilege change.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use whatsapp_matter_rust_provider::{
    rich_preview::original_menu, rich_transport::experimental_self_chat_rich_message,
};

#[test]
fn rich_transport_wraps_original_owner_menu_without_fake_meta_ai_origin() {
    let message = experimental_self_chat_rich_message(original_menu()).unwrap();
    assert!(message.bot_forwarded_message.as_option().is_none());
    assert!(message.message_context_info.as_option().is_none());
    let rich = message.rich_response_message.as_option().unwrap();
    assert!(rich.context_info.as_option().is_none());
    assert_eq!(rich.submessages.len(), 1);
    let body = rich
        .unified_response
        .as_option()
        .unwrap()
        .data
        .as_deref()
        .unwrap();
    assert!(body.len() <= 16_384);
    let json: serde_json::Value = serde_json::from_slice(body).unwrap();
    assert!(json["response_id"]
        .as_str()
        .unwrap()
        .starts_with("matterssmith-"));
    let html = json["embedded_screens"][0]["content"][0]["tabs"][0]["sections"][0]["view_model"]
        ["primitive"]["payload"]
        .as_str()
        .unwrap();
    assert!(html.contains("Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx]."));
    assert!(html.contains("꧁✪⃢.✪ 𝐌𝐚𝐭𝐭𝐞𝐫𝐬𝐬𝐦𝐢𝐭𝐡"));
    assert!(html.contains("disabled>Audio</button>"));
    assert!(html.contains("disabled>MP4</button>"));
    assert!(html.contains("disabled>Enlace</button>"));
    assert!(html.contains("data:audio/ogg;base64,"));
    let wire = json.to_string();
    for marker in [
        "0@bot",
        "botForwardedMessage",
        "forwardedAiBotMessageInfo",
        "isForwarded",
        "<script",
    ] {
        assert!(!wire.contains(marker), "forbidden marker: {marker}");
    }
}

#[test]
fn rich_transport_bounds_inputs_and_uses_fresh_message_ids() {
    for invalid in ["", "NUL\u{0000}", "\u{001b}"] {
        assert!(experimental_self_chat_rich_message(invalid).is_err());
    }
    assert!(experimental_self_chat_rich_message(&"x".repeat(4000)).is_err());
    let first = experimental_self_chat_rich_message(original_menu()).unwrap();
    let second = experimental_self_chat_rich_message(original_menu()).unwrap();
    let get_id = |message: whatsapp_rust::prelude::wa::Message| {
        let rich = message.rich_response_message;
        let data = rich
            .as_option()
            .unwrap()
            .unified_response
            .as_option()
            .unwrap()
            .data
            .as_deref()
            .unwrap()
            .to_vec();
        let value: serde_json::Value = serde_json::from_slice(&data).unwrap();
        value["response_id"].as_str().unwrap().to_owned()
    };
    assert_ne!(get_id(first), get_id(second));
}
