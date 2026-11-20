# Matter-Rust universal y transporte WhatsApp

**Aclaración de arquitectura — 20 noviembre 2026**

Matter-Rust es el núcleo Rust común de WhatsApp, Telegram y Discord, con Social-Matter reservado para el futuro.

**Fuente de verdad del núcleo compartido:** `Matterssmith/Base-Matter`, rama `feat/matter-rust-multiplatform`, crate independiente `crates/matter-rust`. Su contrato de registro, observaciones y separación de sesiones no está ligado a WhatsApp.

**Este repositorio WhatsApp-Matter solo implementa la parte WhatsApp.** El ejecutable experimental `providers/matter-rust` usa el transporte `whatsapp-rust` para probar la conexión WhatsApp; no es todo el motor multiplataforma ni debe tratarse como dependencia de los repos Telegram o Discord. No se ha instalado ni conectado como sustitución del proveedor actual de Base-Matter.

| Repositorio | Función |
| --- | --- |
| Base-Matter / Matter-Rust | Registro de conectores, rutas compartidas y autorización delegada |
| WhatsApp-Matter | Protocolo WhatsApp, compatibilidad WhatsMeow/Baileys, multimedia, NativeFlow |
| Telegram-Matter | Bot API de Telegram, callbacks y multimedia propios |
| Discord-Matter | Gateway y HTTP Discord, interacciones y guilds |
| Social-Matter | Protocolo y servicios sociales futuros, actualmente desactivado |
| Vault-Matter | Autoridad de identidad, permisos, cifrado y auditoría |

**Reglas:** Los adaptadores no se atribuyen roles por mensajes ni mediante números o aliases; los eventos de origen deben autenticarse y los permisos vienen de Vault. Ningún conector comparte almacenes, claves, sockets o tokens. Las funciones de WhatsMeow/Baileys que no existen de forma equivalente en Telegram/Discord tendrán degradación explícita, no emulación falsa.

**Ejecución real:** todavía no disponible a través del núcleo compartido. Las pruebas de la fase actual son offline, no una conexión efectiva a las cuatro plataformas.

**Actualizaciones oficiales:** por plataforma, vigilar cambios de API/SDK y nuevos campos, revisar licencia, actualizar matriz de capacidades y contratos, ejecutar CI Linux/Windows y E2E en cuenta aislada, aprobar y actualizar los README; nunca autoaplicar cambios en sesiones activas.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
