# WhatsApp-Matter — Protocolo Rust / Go / TypeScript / JavaScript

Versión: 1. JSON Lines UTF-8 por stdin/stdout, una petición por línea.
Tamaño máximo de solicitud: 8192 bytes. Solo lectura informativa.

Petición:
    {"version":1,"operation":"describe","provider":"whatsapp-rust"}

Respuesta ilustrativa:
    {"version":1,"ok":true,"provider":{"name":"whatsapp-rust","runtime":"Rust","original_ecosystem":"Rust","status":"native_rust_migration","operations":["describe"],"control_authority":false}}

También se admite provider="baileys" y provider="whatsmeow".
Unknown provider, versión incorrecta o campos adicionales => rechazo.
Enviar mensajes, administrar grupos y activar antis NO está implementado.
No debe enviarse ningún secreto. El protocolo informa compatibilidad,
no representa una prueba de autenticación o permisos.

Rust:
    cargo run -p matter-whatsapp-contracts --bin matter_whatsapp_bridge

JavaScript:
    node bindings/javascript/describe.mjs whatsapp-rust

La evolución a mensajes con efectos requerirá otro gateway de transporte,
autenticación del actor y Vault con decisiones vinculadas a operaciones.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
