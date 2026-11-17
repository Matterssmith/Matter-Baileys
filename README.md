# WhatsApp-Matter (beta, repositorio en transición)

> Nombre anterior en GitHub: \`Matter-Baileys\`. El nombre previsto es
> \`WhatsApp-Matter\`; el repositorio aún **no ha sido renombrado**.
> Matter-Bot continúa bloqueado hasta que Base-Matter cumpla sus requisitos.

## Estado

Adaptador **experimental** de WhatsApp Web con Baileys, para comprobar el
emparejamiento y la recepción de mensajes con un número secundario autorizado.
No es Matter-Bot, no registra comandos de producción, no conecta todavía con
Base-Matter y no debe usarse para administrar grupos ni múltiples sockets.

Baileys es una biblioteca no oficial. Meta ofrece la Cloud API oficial como
alternativa para números de WhatsApp Business.

## Prueba de emparejamiento

Requisitos: Node.js 22+, conexión a Internet, número **secundario propio**
que pueda vincular dispositivos, un solo proceso dueño de esa sesión.

En una copia de la rama \`matter-beta\`:

\`\`\`sh
npm install
npm test
npm run smoke
\`\`\`

La primera ejecución genera un QR en la terminal para vincular un dispositivo.
Si necesitas código de vinculación, establece \`WA_PAIRING_PHONE\` con los dígitos
del número autorizado (prefijo internacional, sin \`+\`) y vuelve a ejecutar.
Usa únicamente el código generado por tu propia sesión.

Los archivos sensibles se guardan exclusivamente bajo \`.local-auth/\` fuera de
Git; la ruta está ignorada. El estado de múltiples archivos provisto por
Baileys es solo para pruebas: no es un almacén seguro para producción.
No subas esa carpeta ni la compartas, incluso si cambias el nombre del repo.

De forma predeterminada el adaptador solo observa eventos y no envía nada.
Para autorizar **únicamente** una respuesta \`#ping\` -> \`pong\` a contactos
de prueba que dieron permiso, configura:

\`\`\`sh
WA_TEST_SEND=1 WA_ALLOWED_JIDS=521XXXXXXXXXX@s.whatsapp.net npm run smoke
\`\`\`

No hay envío a grupos. No se envía nada si no defines la lista permitida. Si
WhatsApp entrega identificadores LID, usa el identificador \`@lid\` conocido del
contacto en la lista; no conviertas identificadores desconocidos en teléfonos.

## Comandos disponibles en modo de prueba

Solo \`#ping\` en chat privado explícitamente permitido. Los demás mensajes se
descartan y no se registran contenidos ni JID completos en los logs.
La app no envía nada por defecto ni responde automáticamente al mismo
dispositivo con \`fromMe=true\`.

## Límites y siguientes requisitos

- La conexión real requiere vinculación QR/código con un teléfono autorizado.
  No puedo completar ese paso desde GitHub Actions ni desde ChatGPT.
- No existe aún puente autenticado hacia el InboundGateway/MessageService de
  Base-Matter. El test de Baileys no demuestra integración completa de Base.
- El proceso no reconecta automáticamente cuando se revoca/reemplaza la sesión,
  evitando competir con otro socket. Ante errores, revisa el estado y
  reinicia manualmente. No reutilices una sesión en varios servidores.
- Antes de producción se necesita un almacén de claves Signal autenticado,
  aislamiento por socket, revocación, control de generación, auditoría,
  consentimiento, cifrado de configuración y pruebas completas de grupos.
- GitHub Actions ejecuta pruebas unitarias sin credenciales y verifica que el
  proyecto instala dependencias. No realiza emparejamiento real.
- Los límites y el contrato previsto con Rust están en
  [docs/ADAPTER_CONTRACT.md](docs/ADAPTER_CONTRACT.md).

## Créditos

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].

La biblioteca Baileys conserva su propia licencia y autoría; esta adaptación
no afirma ser oficial de WhatsApp.
