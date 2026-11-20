/*
 * WhatsApp-Matter — offline, no-impersonation Rich HTML compatibility tests.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use whatsapp_matter_rust_provider::rich_preview::{
    experimental_rich_schema, menu_html_preview, original_menu,
};

#[test]
fn original_matterssmith_menu_is_preserved_inside_offline_html() {
    let html = menu_html_preview(original_menu()).unwrap();
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("꧁✪⃢.✪ 𝐌𝐚𝐭𝐭𝐞𝐫𝐬𝐬𝐦𝐢𝐭𝐡"));
    assert!(html.contains("*Base-Matter* · ✦"));
    assert!(html.contains("Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx]."));
    assert!(html.contains("disabled>Audio</button>"));
    assert!(html.contains("disabled>MP4</button>"));
    assert!(html.contains("disabled>Enlace</button>"));
    assert!(html.contains("data:audio/ogg;base64,"));
    assert!(html.contains("<audio controls preload="));
    assert!(!html.contains("<script"));
    assert!(!html.contains("<iframe"));
    assert!(!html.contains("onclick="));
    assert!(!html.contains("https://"));
}

#[test]
fn untrusted_text_cannot_inject_scripts_or_html_attributes() {
    let html = menu_html_preview("<script>alert(\"x\")</script> & 'hello'").unwrap();
    assert!(html.contains("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;"));
    assert!(html.contains("&amp; &#39;hello&#39;"));
    assert!(!html.contains("<script"));
    for text in ["", "unsafe\u{0000}menu", "x\u{001b}y"] {
        assert!(menu_html_preview(text).is_err());
    }
    assert!(menu_html_preview(&"a".repeat(3801)).is_err());
}

#[test]
fn sample_rich_screen_shape_is_offline_and_does_not_spoof_meta_ai() {
    let value = experimental_rich_schema(original_menu()).unwrap();
    assert_eq!(value["response_id"], "matterssmith-preview-v1");
    assert_eq!(
        value["embedded_screens"][0]["content"][0]["__typename"],
        "FOAIDNixelButtonSheets"
    );
    assert_eq!(
        value["embedded_screens"][0]["content"][0]["tabs"][0]["sections"][0]["view_model"]
            ["primitive"]["__typename"],
        "GenAIaeacdsnwHtmlPrimitive"
    );
    let html = value["embedded_screens"][0]["content"][0]["tabs"][0]["sections"][0]["view_model"]
        ["primitive"]["payload"]
        .as_str()
        .unwrap();
    assert!(html.contains("Base-Matter"));
    let serialized = value.to_string();
    for forbidden in [
        "0@bot",
        "botForwardedMessage",
        "forwardedAiBotMessageInfo",
        "isForwarded",
        "<script",
        "https://",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "forbidden marker: {forbidden}"
        );
    }
}
