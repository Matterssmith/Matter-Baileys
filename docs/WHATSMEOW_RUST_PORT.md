# WhatsMeow Rust Matter — port, compatibilidad y actualizaciones

**Estado:** Fase 1 implementada en rama aislada. No es un port completo de Go ni una conexión real de WhatsApp. La sesión Rust existente de Base-Matter sigue intacta.

## Investigación — 20/11/2026

- Oficial tulir/whatsmeow: Go, licencia MPL-2.0. Mensajes, grupos, estados, media, presencia, sincronización y confirmaciones; no presume llamadas y broadcast list send.
- Rust whatsmeow 0.1.4 (crates.io): bindings Rust sobre Go mediante whatsmeow-sys. No es motor nativo. Su último build docs.rs no estaba exitoso.
- YASSERRMD/Whatsmeow-rust: port parcial puro Rust; reconoce que Signal, conexión WebSocket y grupos requieren terminarse y verificarse.
- oxidezap/whatsapp-rust 0.7.0: biblioteca nativa Rust con E2EE, pareado, multimedia, grupos y más. Se utilizará como base técnica, no como sustituto ficticio de una conversión literal del código Go.
- WhiskeySockets/Baileys: JS/TS, referencia para tipos de contenido interactivo, pero renderizar botones en WhatsApp depende del dispositivo y la versión.

## Arquitectura Matter

- WhatsApp-Matter es dueño de los proveedores; Base-Matter conserva el proveedor actual hasta que termine un corte controlado. Bot-Matter sigue bloqueado.
- providers/whatsmeow/src/engine.rs contiene ingreso estricto de eventos de texto (sin conexión), aislamiento de identidad por socket, validación de storage privado existente, filtro antireplay volátil y prohibición de mutaciones mediante datos no autenticados.
- providers/whatsmeow/src/features.rs lleva el inventario de funcionalidades. Estado offline no equivale a disponibilidad en red; ninguna capacidad sensible se marca como comprobada hasta E2E real.
- El port final utilizará el stack rust nativo (whatsapp-rust) con compatibilidad WhatsMeow y extras Matter. Un worker Go oficial puede usarse opcionalmente como comparador independiente en pruebas, nunca como requisito oculto para reclamar Rust puro.
- El permiso de owner global debe provenir de decisiones verificadas por Vault, no de eventos JSON o datos ingresados por usuarios.
- Nunca reutilizar el almacén de Base, claves Signal, sesiones o grants; su ciclo de vida es independiente.

## Lista de capacidades obligatorias

1. Protocolo: Noise, Signal, Protobuf, sincronización, E2EE, reintentos, reconnect, history sync, device keys.
2. Cuenta: pairing por código telefónico, PN/LID, sesiones persistentes, multicuenta y revocación.
3. Mensajería: texto, citas, menciones, ediciones, eliminación, reacciones, temporales, receipts, leído, privacidad.
4. Multimedia: fotos, stickers, audio, voz, video, documentos, GIF, cifrado, descarga y upload.
5. Grupos, comunidades, invitaciones, permisos de admin, canales/newsletters, moderación.
6. Interactividad: botones NativeFlow, CTA URL, open_webview, encuestas, listas, carouseles, menús y sesiones de prueba.
7. Extras Matter: roles globales autorizados, filtros Anti a ediciones/camuflaje, sandbox de plugins, bots de Rust/Go/TS/JS, registro web y Vault-Matter.
8. Extras opcionales: llamadas, negocio, privacidad avanzada, estados; se habilitan solo si el stack real y el protocolo los soportan.

## Cada actualización oficial: procedimiento obligatorio

1. Consultar los HEAD exactos de tulir/whatsmeow, WhiskeySockets/Baileys y oxidezap/whatsapp-rust mediante el monitor; comparar con config/upstream-sources.json. Esos SHAs son observaciones y no aprobaciones de seguridad.
2. Registrar SHA previo, nuevo SHA, fecha y URL de comparación. El workflow whatsmeow-upstream-watch.yml podrá crear un issue de revisión cuando cambie alguno. Las tareas programadas solo se ejecutan tras integrar el workflow a la rama default de GitHub; en la rama feature no hay vigilancia automática activa.
3. Evaluar compatibilidad de cambios: Protobuf, Signal/Noise, mensajes editados, PN/LID, multimedia, grupos, NativeFlow, limitaciones de cliente, errores y seguridad.
4. Respetar licencias: WhatsMeow MPL-2.0 y Rust de base MIT. No copiar código MPL dentro de archivos MIT sin cumplir sus obligaciones. Preferir implementación Rust propia sobre interfaces y especificación.
5. Actualizar la matriz de capacidades y casos sintéticos, Rustfmt, cargo tests, Clippy con -D warnings y CI Linux/Windows. Hacer pruebas en dispositivos reales para funciones interactivas.
6. Solo tras revisión y evidencia de pruebas, actualizar SHAs de observación en config/upstream-sources.json y dependencias con pin exacto, en PR revisado sin auto-merge.
7. Revalidar instalación, rollback, aislamiento de socket, restauración y acceso Vault; no desplegar sobre WhatsApp conectado sin migración autorizada.
8. En cada commit/PR actualizar README.md, AGENTS.md y docs/WORK_CONTINUITY.md, con SHA, archivos, pruebas, pendientes y etapa siguiente.

## Entrega por fases

F1: contrato offline, aislamiento, paridad de características y vigilancia (implementado, sujeto a CI).
F2: backend Rust real con whatsapp-rust, pairing-code, envío y E2EE probados en socket nuevo.
F3: igualdad WhatsMeow (grupos, media, app state, newsletters).
F4: compatibilidad ampliada Baileys, UI avanzada y casos de clientes.
F5: autoridad Vault, anti-spoofing y test multicliente.
F6: despliegue aislado, recuperación y sustitución opcional del proveedor actual.

**No se puede declarar paridad completa todavía.** Ninguna API de WhatsApp no oficial garantiza compatibilidad futura ni el derecho a eludir límites de plataforma.

Fuentes:
- https://github.com/tulir/whatsmeow
- https://github.com/oxidezap/whatsapp-rust
- https://github.com/WhiskeySockets/Baileys
- https://github.com/YASSERRMD/Whatsmeow-rust

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
