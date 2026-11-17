# WhatsApp-Matter -> Base-Matter: control autenticado v1

Este es un contrato experimental para eventos del **propietario autenticado**
y su chat privado. No forma parte de Bot-Matter. No entrega respuestas a
WhatsApp y no reemplaza la vinculación por código.

## Variables locales obligatorias

- WA_BRIDGE_ADDR = 127.0.0.1:PORT (únicamente loopback)
- WA_BRIDGE_KEY_HEX = secreto no nulo de 32 bytes (64 caracteres hex)
- WA_BRIDGE_SOCKET_ID = ID del socket creado en Base-Matter
- WA_BRIDGE_OWNER_ID = ID del dueño registrado en Base-Matter
- WA_BRIDGE_OWNER_JID = chat privado de la cuenta vinculada, con
  @s.whatsapp.net o @lid

La clave se almacena únicamente en el proceso autorizado. No se incluye en
Git ni en Vault; no se imprime. Cada socket requiere una clave distinta.

## Wire

Codificación JSON UTF-8 terminada en LF, máximo 16 KiB. Contiene versión 1,
socket_id, actor_id, owner_jid, chat_jid, message_id, text, from_me=true,
timestamp en segundos Unix, signature en HMAC-SHA256 hex.

La entrada del MAC se documenta en
[Base-Matter](https://github.com/Matterssmith/Matter-Base/blob/matter-beta/docs/WHATSAPP_BRIDGE.md).
Rust y Node usan el mismo vector de firma.

El adaptador solo observa eventos fromMe con JID privado propio y mensajes
que comienzan con el prefijo canónico \`=/>\`. Grupos, terceros, documentos,
archivos, otros prefijos, \`#ping\` y eventos ambiguos se ignoran.

## Límites

La sonda no guarda mensajes ni los reenvía a otros chats. No hay
persistencia de mensajes por el proveedor: Base-Matter debe aplicar su
InboundReplayStore transaccional antes de modificar el control. El transporte
local todavía requiere un listener de integración independiente. Se deben
probar revocación de claves, emparejamiento real y recuperación de sesiones.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
