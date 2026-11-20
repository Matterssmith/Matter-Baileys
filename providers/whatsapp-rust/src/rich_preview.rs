/*
 * WhatsApp-Matter — offline-only research of experimental Rich HTML envelopes.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use anyhow::{bail, Result};
use base64::Engine;
use serde_json::{json, Value};

const MAX_TEXT_BYTES: usize = 3800;
const MAX_HTML_BYTES: usize = 16 * 1024;

/// Build a safe, fully offline HTML preview of the verified original menu.
///
/// This is a browser preview, NOT an HTML message sent to WhatsApp.
pub fn menu_html_preview(menu: &str) -> Result<String> {
    // Git checkouts on Windows may change checked-in template LF into CRLF.
    // Normalize paired line endings ONLY for the offline HTML preview.
    let normalized = menu.replace("\r\n", "\n");
    let menu = normalized.as_str();
    if menu.is_empty()
        || menu.len() > MAX_TEXT_BYTES
        || menu.chars().any(|c| c.is_control() && c != '\n')
    {
        bail!("Invalid Matterssmith menu for HTML preview");
    }
    let mut escaped = String::with_capacity(menu.len() + 128);
    for ch in menu.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    let audio_data =
        base64::engine::general_purpose::STANDARD.encode(crate::menu_sound::menu_tone_bytes());
    let result = format!(
        r#"<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Base-Matter · Vista previa experimental</title>
<style>
:root{{color-scheme:dark}}
*{{box-sizing:border-box}}
body{{margin:0;background:#090d18;color:#f3f7ff;font-family:system-ui,sans-serif;padding:18px}}
main{{max-width:470px;margin:0 auto;padding:22px 16px;border:1px solid #31466d;border-radius:17px;background:linear-gradient(150deg,#101c38,#14111f)}}
pre{{font-family:inherit;font-size:12px;line-height:1.55;white-space:pre-wrap;overflow-wrap:anywhere;margin:0}}
.controls{{display:grid;grid-template-columns:repeat(3,1fr);gap:8px;margin-top:18px}}
button{{border-radius:10px;border:1px solid #576d9d;background:#273554;color:#fff;padding:11px 3px;opacity:.8}}
small{{display:block;margin:15px 0;color:#a5b8d8}}
</style>
</head>
<body><main>
<pre>{escaped}</pre>
<div class="controls">
<button type="button" disabled>Audio</button>
<button type="button" disabled>MP4</button>
<button type="button" disabled>Enlace</button>
</div>
<audio controls preload="none" aria-label="Sonido original"><source src="data:audio/ogg;base64,{audio_data}" type="audio/ogg; codecs=opus"></audio>
<small>Vista HTML experimental. Audio pulsable; nunca autoplay. Los botones activos son mensajes nativos separados.</small>
</main></body></html>"#
    );
    if result.len() > MAX_HTML_BYTES {
        bail!("HTML preview exceeds offline budget");
    }
    Ok(result)
}

/// Reconstruct *only* the research JSON hierarchy present in the supplied
/// Go/JS examples. This function never makes or sends a WA Message.
///
/// The experimental Meta AI names are preserved for compatibility analysis.
/// In particular we never wrap the result in botForwardedMessage or assert
/// that the sender is `0@bot`.
pub fn experimental_rich_schema(menu: &str) -> Result<Value> {
    let html = menu_html_preview(menu)?;
    let output = json!({
        "response_id": "matterssmith-preview-v1",
        "sections": [{
            "view_model": {
                "__typename": "GenAISingleLayoutViewModel",
                "primitive": {
                    "__typename": "GenAIMarkdownTextUXPrimitive",
                    "text": "Base-Matter · Vista previa local"
                }
            }
        }],
        "embedded_screens": [{
            "title": "Base-Matter · Vista previa",
            "content": [{
                "__typename": "FOAIDNixelButtonSheets",
                "tabs": [{
                    "id": "matterssmith-control",
                    "tab_header": "Matterssmith",
                    "sections": [{
                        "view_model": {
                            "__typename": "GenAISingleLayoutViewModel",
                            "primitive": {
                                "__typename": "GenAIaeacdsnwHtmlPrimitive",
                                "payload": html,
                                "trusted_sources": []
                            }
                        }
                    }]
                }]
            }]
        }]
    });
    // Defensive size bound for future local tooling that consumes the
    // serializer; no external network, scripts or dynamic embeds.
    if output.to_string().len() > MAX_HTML_BYTES * 2 {
        bail!("Rich preview structure exceeds offline budget");
    }
    Ok(output)
}

pub fn original_menu() -> &'static str {
    include_str!("../../../templates/matterssmith-menu.txt").trim_ascii_end()
}
