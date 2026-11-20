/*
 * WhatsApp-Matter — optional native quick replies and restricted HTML styling.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use anyhow::{bail, Result};
use buffa::MessageField;
use wa::message::interactive_response_message::InteractiveResponseMessage as NativeResponse;
use whatsapp_rust::prelude::wa;

const MAX_MENU_BYTES: usize = 3800;
const VISIBLE_CATEGORIES: &str = include_str!("../../../templates/menu-visible-categories.json");

const NAV_CATEGORIES: [(&str, &str, &str); 21] = [
    ("ia", "ⴵ IA", "matter.read.menu.ia"),
    ("anti", "☾︎ Anti", "matter.read.menu.anti"),
    ("all", "✦ Todo", "matter.read.menu.all"),
    ("config", "⚙︎ Configuración", "matter.read.menu.config"),
    ("downloads", "⇩ Descargas", "matter.read.menu.downloads"),
    ("economy", "◈ Economía", "matter.read.menu.economy"),
    ("fun", "♫ Diversión", "matter.read.menu.fun"),
    ("gacha", "✰ Gacha", "matter.read.menu.gacha"),
    ("games", "🎮 Juegos", "matter.read.menu.games"),
    ("group", "♧ Grupo", "matter.read.menu.group"),
    ("info", "ⓘ Información", "matter.read.menu.info"),
    ("links", "🔗 Enlaces", "matter.read.menu.links"),
    ("main", "✪ Principal", "matter.read.menu.main"),
    ("mode", "◉ Modo", "matter.read.menu.mode"),
    ("owner", "♛ Propietario", "matter.read.menu.owner"),
    ("profile", "♙ Perfil", "matter.read.menu.profile"),
    ("reactions", "♡ Reacciones", "matter.read.menu.reactions"),
    ("search", "⌕ Búsqueda", "matter.read.menu.search"),
    ("sockets", "ⴵ Conexiones", "matter.read.menu.sockets"),
    ("system", "⚙︎ Sistema", "matter.read.menu.system"),
    ("utils", "✧ Utilidades", "matter.read.menu.utils"),
];

fn button_command(id: &str) -> Option<&'static str> {
    match id {
        "matter.read.status" => Some("=/>status"),
        "matter.read.menu" => Some("=/>menu"),
        "matter.read.antis" => Some("=/>menu antis"),
        "matter.read.help" => Some("=/>help"),
        "matter.read.menu.ia" => Some("=/>menu ia"),
        "matter.read.menu.anti" => Some("=/>menu anti"),
        "matter.read.menu.all" => Some("=/>menu all"),
        "matter.read.menu.config" => Some("=/>menu config"),
        "matter.read.menu.downloads" => Some("=/>menu downloads"),
        "matter.read.menu.economy" => Some("=/>menu economy"),
        "matter.read.menu.fun" => Some("=/>menu fun"),
        "matter.read.menu.gacha" => Some("=/>menu gacha"),
        "matter.read.menu.games" => Some("=/>menu games"),
        "matter.read.menu.group" => Some("=/>menu group"),
        "matter.read.menu.info" => Some("=/>menu info"),
        "matter.read.menu.links" => Some("=/>menu links"),
        "matter.read.menu.main" => Some("=/>menu main"),
        "matter.read.menu.mode" => Some("=/>menu mode"),
        "matter.read.menu.owner" => Some("=/>menu owner"),
        "matter.read.menu.profile" => Some("=/>menu profile"),
        "matter.read.menu.reactions" => Some("=/>menu reactions"),
        "matter.read.menu.search" => Some("=/>menu search"),
        "matter.read.menu.sockets" => Some("=/>menu sockets"),
        "matter.read.menu.system" => Some("=/>menu system"),
        "matter.read.menu.utils" => Some("=/>menu utils"),
        "matter.read.menu.page.1" => Some("=/>menu page 1"),
        "matter.read.menu.page.2" => Some("=/>menu page 2"),
        "matter.read.menu.page.3" => Some("=/>menu page 3"),
        // Historical response IDs are aliases only; no legacy categories appear.
        "matter.read.menu.ai" => Some("=/>menu ia"),
        "matter.read.menu.antis" => Some("=/>menu anti"),
        "matter.read.menu.set" => Some("=/>menu config"),
        "matter.read.menu.bot" => Some("=/>menu mode"),
        "matter.read.menu.status" => Some("=/>menu info"),
        "matter.read.menu.roles" => Some("=/>menu owner"),
        "matter.read.menu.sandbox" => Some("=/>menu utils"),
        "matter.read.menu.navigation" => Some("=/>menu main"),
        "matter.read.help.status" => Some("=/>help status"),
        "matter.read.help.bot" => Some("=/>help bot"),
        "matter.read.help.set" => Some("=/>help set"),
        "matter.read.help.antis" => Some("=/>help antis"),
        "matter.read.help.roles" => Some("=/>help roles"),
        "matter.read.help.system" => Some("=/>help system"),
        "matter.read.help.sandbox" => Some("=/>help sandbox"),
        "matter.read.help.ai" => Some("=/>help ai"),
        "matter.read.help.navigation" => Some("=/>help navigation"),
        "matter.test.button.a" => Some("=/>test ack a"),
        "matter.test.button.b" => Some("=/>test ack b"),
        "matter.test.select.one" => Some("=/>test ack one"),
        "matter.test.select.two" => Some("=/>test ack two"),
        "matter.test.row.audio" => Some("=/>test ack audio"),
        "matter.test.row.mp4" => Some("=/>test ack mp4"),
        "matter.test.row.link" => Some("=/>test ack link"),
        _ => None,
    }
}

/// Convert a *small* allowlisted HTML-style fragment to WhatsApp text
/// formatting. WhatsApp does not interpret arbitrary HTML or JavaScript.
/// Reject attributes, scripts, unknown tags and unclosed markup.
pub fn short_html_to_whatsapp(input: &str) -> Result<String> {
    if input.len() > 512 || input.chars().any(|c| c.is_control() && c != '\n') {
        bail!("HTML style fragment exceeds safe bounds");
    }
    let mut remaining = input;
    let mut stack: Vec<&str> = Vec::new();
    let mut output = String::with_capacity(input.len());
    while !remaining.is_empty() {
        let mut found = false;
        for (tag, marker) in [
            ("b", "*"),
            ("strong", "*"),
            ("i", "_"),
            ("em", "_"),
            ("s", "~"),
            ("code", "`"),
        ] {
            let opening = format!("<{tag}>");
            let closing = format!("</{tag}>");
            if remaining.starts_with(&opening) {
                stack.push(tag);
                output.push_str(marker);
                remaining = &remaining[opening.len()..];
                found = true;
                break;
            }
            if remaining.starts_with(&closing) {
                if stack.pop() != Some(tag) {
                    bail!("Mismatched HTML style tag");
                }
                output.push_str(marker);
                remaining = &remaining[closing.len()..];
                found = true;
                break;
            }
        }
        if found {
            continue;
        }
        if remaining.starts_with("<br>") {
            output.push('\n');
            remaining = &remaining[4..];
        } else if remaining.starts_with("&amp;") {
            output.push('&');
            remaining = &remaining[5..];
        } else if remaining.starts_with("&lt;") {
            output.push('<');
            remaining = &remaining[4..];
        } else if remaining.starts_with("&gt;") {
            output.push('>');
            remaining = &remaining[4..];
        } else if remaining.starts_with("&quot;") {
            output.push('"');
            remaining = &remaining[6..];
        } else {
            let ch = remaining.chars().next().expect("nonempty input");
            if ch == '<' || ch == '>' || ch == '&' {
                bail!("Unsupported HTML tag or entity");
            }
            output.push(ch);
            remaining = &remaining[ch.len_utf8()..];
        }
    }
    if !stack.is_empty() || output.len() > 800 {
        bail!("Unclosed or oversized HTML style");
    }
    Ok(output)
}

/// Construct a NativeFlow menu with exactly one Categories selector.
/// The phone must confirm visual support; no selection grants roles.
pub fn native_menu_message(menu: &str) -> Result<wa::Message> {
    native_navigation_message(menu, "=/>menu")
}

pub fn native_navigation_message(menu: &str, request: &str) -> Result<wa::Message> {
    if menu.is_empty()
        || menu.len() > MAX_MENU_BYTES
        || menu.chars().any(|c| c.is_control() && c != '\n')
    {
        bail!("Invalid authenticated native menu");
    }
    if !(request == "=/>menu"
        || request == "=/>menú"
        || request.starts_with("=/>menu ")
        || request.starts_with("=/>menú ")
        || request == "=/>help"
        || request == "=/>h"
        || request == "=/>ayuda"
        || request.starts_with("=/>help ")
        || request.starts_with("=/>h ")
        || request.starts_with("=/>ayuda "))
    {
        bail!("Native navigation is read-only");
    }
    // Shared visibility manifest: only implemented categories are selectable.
    // The core tests assert that this manifest matches implemented commands.
    let active: Vec<String> = serde_json::from_str(VISIBLE_CATEGORIES)?;
    if active.is_empty()
        || active.len() > NAV_CATEGORIES.len()
        || active.iter().any(|id| {
            !NAV_CATEGORIES
                .iter()
                .any(|(known, _, _)| *known == id.as_str())
        })
    {
        bail!("Invalid visible menu categories");
    }
    let categories: Vec<_> = NAV_CATEGORIES
        .iter()
        .filter(|(id, _, _)| active.iter().any(|shown| shown.as_str() == *id))
        .collect();
    if categories.len() != active.len() {
        bail!("Duplicate visible menu category");
    }
    const PAGE_SIZE: usize = 9;
    let pages = categories.len().div_ceil(PAGE_SIZE).max(1);
    let requested_page = match request {
        "=/>menu page 2" => 2,
        "=/>menu page 3" => 3,
        _ => 1,
    };
    // Historical page IDs are still read-only aliases, not empty pages.
    let page = requested_page.min(pages);
    let start = (page - 1) * PAGE_SIZE;
    let mut rows: Vec<_> = categories
        .iter()
        .skip(start)
        .take(PAGE_SIZE)
        .map(|(category, _, id)| {
            // WhatsApp shows the selected row's title as the outgoing
            // chat bubble. It must be the exact command, not decoration.
            serde_json::json!({
                "title": format!("=/>menu {category}"),
                "id": id
            })
        })
        .collect();
    if page > 1 {
        rows.push(serde_json::json!({
            "title": "Anterior",
            "id": format!("matter.read.menu.page.{}", page - 1)
        }));
    }
    if page < pages {
        rows.push(serde_json::json!({
            "title": "Más categorías",
            "id": format!("matter.read.menu.page.{}", page + 1)
        }));
    }
    let selector = wa::message::interactive_message::native_flow_message::NativeFlowButton {
        name: Some("single_select".to_owned()),
        button_params_json: Some(
            serde_json::json!({
                "title": "Categorías",
                "sections": [{
                    "title": "Categorías",
                    "rows": rows
                }]
            })
            .to_string(),
        ),
    };
    let buttons = vec![selector];
    Ok(wa::Message {
        interactive_message: MessageField::some(wa::message::InteractiveMessage {
            body: MessageField::some(wa::message::interactive_message::Body {
                text: Some(menu.to_owned()),
            }),
            footer: MessageField::some(wa::message::interactive_message::Footer {
                text: Some(format!("Control · Categorías {page}/{pages}")),
                ..Default::default()
            }),
            interactive_message: Some(
                wa::message::interactive_message::InteractiveMessage::NativeFlowMessage(Box::new(
                    wa::message::interactive_message::NativeFlowMessage {
                        buttons,
                        message_version: Some(1),
                        ..Default::default()
                    },
                )),
            ),
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Owner-only UI diagnostic transport. This does not verify phone rendering;
/// it supplies two quick replies and two selector rows that the owner can tap.
/// IDs map only to read-only acknowledgments after the normal HMAC bridge.
pub fn native_ui_probe_message() -> Result<wa::Message> {
    let selector = wa::message::interactive_message::native_flow_message::NativeFlowButton {
        name: Some("single_select".to_owned()),
        button_params_json: Some(
            serde_json::json!({
                "title": "Prueba de selector",
                "sections": [{
                    "title": "Selecciona una fila",
                    "rows": [
                        {"title": "Fila uno", "id": "matter.test.select.one"},
                        {"title": "Fila dos", "id": "matter.test.select.two"}
                    ]
                }]
            })
            .to_string(),
        ),
    };
    let quick_reply = |id: &str, label: &str| {
        wa::message::interactive_message::native_flow_message::NativeFlowButton {
            name: Some("quick_reply".to_owned()),
            button_params_json: Some(
                serde_json::json!({"display_text": label, "id": id}).to_string(),
            ),
        }
    };
    Ok(wa::Message {
        interactive_message: MessageField::some(wa::message::InteractiveMessage {
            body: MessageField::some(wa::message::interactive_message::Body {
                text: Some(
                    "🧪 Prueba de interfaz\nToca A, B o una fila del selector.\n\
                     Espera un acuse del controlador; esto no modifica datos."
                        .to_owned(),
                ),
            }),
            footer: MessageField::some(wa::message::interactive_message::Footer {
                text: Some("Base-Matter · prueba privada".to_owned()),
                ..Default::default()
            }),
            interactive_message: Some(
                wa::message::interactive_message::InteractiveMessage::NativeFlowMessage(Box::new(
                    wa::message::interactive_message::NativeFlowMessage {
                        buttons: vec![
                            selector,
                            quick_reply("matter.test.button.a", "Botón A"),
                            quick_reply("matter.test.button.b", "Botón B"),
                        ],
                        message_version: Some(1),
                        ..Default::default()
                    },
                )),
            ),
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Read-only button cards based on the original ts18/ts19 layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuCardVariant {
    Ts18,
    Ts19,
}

pub fn menu_lab_card(variant: MenuCardVariant) -> wa::Message {
    let button = |id: &str, name: &str| wa::message::buttons_message::Button {
        button_id: Some(id.to_owned()),
        button_text: MessageField::some(wa::message::buttons_message::button::ButtonText {
            display_text: Some(name.to_owned()),
        }),
        r#type: Some(wa::message::buttons_message::button::Type::RESPONSE),
        ..Default::default()
    };
    let mut buttons = vec![
        button("matter.test.row.audio", "Audio"),
        button("matter.test.row.mp4", "MP4"),
    ];
    if variant == MenuCardVariant::Ts19 {
        buttons.push(button("matter.test.row.link", "Enlace"));
    }
    let title = if variant == MenuCardVariant::Ts18 {
        "TS18"
    } else {
        "TS19"
    };
    wa::Message {
        buttons_message: MessageField::some(wa::message::ButtonsMessage {
            content_text: Some(format!(
                "{title} · Matterssmith\nAudio · MP4 · Enlace\nPrueba de botones, sin descargas."
            )),
            footer_text: Some("Base-Matter · interfaz de prueba".to_owned()),
            header_type: Some(wa::message::buttons_message::HeaderType::EMPTY),
            buttons,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// NativeFlow versions of the TS18/TS19 laboratory cards.
/// Unlike legacy ButtonsMessage, these use the same interactive transport
/// as the category selector already visible on Android.
/// A successful send is NOT proof of client-side rendering.
pub fn native_menu_lab_card(variant: MenuCardVariant) -> wa::Message {
    let choices: &[(&str, &str)] = if variant == MenuCardVariant::Ts18 {
        &[
            ("matter.test.row.audio", "Audio"),
            ("matter.test.row.mp4", "MP4"),
        ]
    } else {
        &[
            ("matter.test.row.audio", "Audio"),
            ("matter.test.row.mp4", "MP4"),
            ("matter.test.row.link", "Enlace"),
        ]
    };
    let buttons = choices
        .iter()
        .map(|(id, label)| {
            wa::message::interactive_message::native_flow_message::NativeFlowButton {
                name: Some("quick_reply".to_owned()),
                button_params_json: Some(
                    serde_json::json!({
                        "display_text": label,
                        "id": id
                    })
                    .to_string(),
                ),
            }
        })
        .collect();
    let title = if variant == MenuCardVariant::Ts18 {
        "TS18"
    } else {
        "TS19"
    };
    wa::Message {
        interactive_message: MessageField::some(wa::message::InteractiveMessage {
            body: MessageField::some(wa::message::interactive_message::Body {
                text: Some(
                    format!("{title} · Matterssmith\\nBotones de prueba: no ejecutan descargas.")
                        .replace("\\n", "\n"),
                ),
            }),
            footer: MessageField::some(wa::message::interactive_message::Footer {
                text: Some("Base-Matter · laboratorio NativeFlow".to_owned()),
                ..Default::default()
            }),
            interactive_message: Some(
                wa::message::interactive_message::InteractiveMessage::NativeFlowMessage(Box::new(
                    wa::message::interactive_message::NativeFlowMessage {
                        buttons,
                        message_version: Some(1),
                        ..Default::default()
                    },
                )),
            ),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// A legacy ButtonsMessage row, matching the protocol family used by
/// classic Go/JS SendButtonsRow examples. Text-only on purpose: a real
/// thumbnail needs an independently uploaded and encrypted WA media object.
/// The private owner can compare this with NativeFlow from =/>test ui.
pub fn legacy_row_probe_message() -> wa::Message {
    let response = |id: &str, label: &str| wa::message::buttons_message::Button {
        button_id: Some(id.to_owned()),
        button_text: MessageField::some(wa::message::buttons_message::button::ButtonText {
            display_text: Some(label.to_owned()),
        }),
        r#type: Some(wa::message::buttons_message::button::Type::RESPONSE),
        ..Default::default()
    };
    wa::Message {
        buttons_message: MessageField::some(wa::message::ButtonsMessage {
            content_text: Some(
                "🧪 Tarjeta de prueba (ButtonsRow)\n\
                 Elige Audio, MP4 o Enlace. No hay descargas."
                    .to_owned(),
            ),
            footer_text: Some("Base-Matter · diagnóstico privado".to_owned()),
            header_type: Some(wa::message::buttons_message::HeaderType::EMPTY),
            buttons: vec![
                response("matter.test.row.audio", "Audio"),
                response("matter.test.row.mp4", "MP4"),
                response("matter.test.row.link", "Enlace"),
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// A response is only a request to a read-only, fixed command.
/// The normal WhatsApp identity, HMAC and replay checks run after mapping.
/// Legacy button replies are recognized for Go-style ButtonsRow compatibility.
pub fn readonly_button_command(message: &wa::Message) -> Option<&'static str> {
    // WhatsApp list rows are not guaranteed to arrive as NativeFlow.
    // Select by opaque protocol IDs only; never trust visible text.
    let id = if let Some(response) = message.interactive_response_message.as_option() {
        let NativeResponse::NativeFlowResponseMessage(native) =
            response.interactive_response_message.as_ref()?;
        if !matches!(
            native.name.as_deref(),
            Some("quick_reply" | "single_select")
        ) {
            return None;
        }
        let json = native.params_json.as_deref()?;
        if json.len() > 512 {
            return None;
        }
        let value: serde_json::Value = serde_json::from_str(json).ok()?;
        value
            .get("id")
            .or_else(|| value.get("selectedRowId"))
            .or_else(|| value.get("selected_row_id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    } else if let Some(response) = message.list_response_message.as_option() {
        response
            .single_select_reply
            .as_option()?
            .selected_row_id
            .clone()
    } else if let Some(response) = message.buttons_response_message.as_option() {
        response.selected_button_id.clone()
    } else if let Some(response) = message.template_button_reply_message.as_option() {
        response.selected_id.clone()
    } else {
        None
    }?;
    if id.len() > 128 {
        return None;
    }
    button_command(&id)
}
