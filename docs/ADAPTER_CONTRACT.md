# Contrato de WhatsApp-Matter (pruebas)

El repositorio remoto sigue con su nombre antiguo hasta que se haga el
renombrado administrativo a WhatsApp-Matter. Este adaptador no incorpora el
runtime de Matter-Bot ni registra comandos permanentes.

## Arquitectura

- Baileys mantiene su propia sesión en la carpeta local ignorada;
  nunca en Base-Matter, Vault-Matter ni GitHub.
- Baileys Web es un proveedor no oficial y puede desconectarse o cambiar
  de protocolo sin previo aviso.
- Por defecto: únicamente lectura y prueba de vinculación. La única escritura
  opcional es contestar \`#ping\` en chats privados de contactos permitidos.
- Se requiere un solo proceso propietario; la presencia del archivo
  \`.owner.lock\` impide iniciar otro proceso con la misma carpeta de sesión.
- La autenticación del emisor, la traducción a identidades globales y
  la persistencia del replay de mensajes pertenecen al futuro bridge seguro.
- La prueba usa deduplicación limitada en memoria, no sustituye el
  InboundReplayStore cifrado de Base-Matter.
- WhatsApp-Matter deberá implementar los contratos Rust de InboundAuthenticator,
  InboundAdmission, TransportDriver, ConnectionDriver y SessionCredentialSource
  mediante un transporte autenticado. El controlador Node no puede crear
  por sí mismo una identidad Rust verificada.
- El test de conexión no verifica todavía envío cifrado de Vault, reconexión
  segura, grupos, permisos, sincronización de claves ni límites por proceso.

## Criterios para promover

1. npm instala una versión upstream fijada y conserva avisos/licencia.
2. Las pruebas de admisión, grupos y deduplicación pasan.
3. Una cuenta secundaria autorizada se vincula realmente mediante QR o código.
4. Recibe un mensaje directo y confirma que no responde por defecto.
5. Una lista explícita permite \`#ping\` solo desde el contacto autorizado.
6. Se prueba cierre, caída, reemplazo de conexión y reinicio sin sesiones
   duplicadas ni filtraciones de credenciales.
7. Base-Matter valida realmente la identidad del proveedor y la autorización
   antes de encolar/contestar eventos. El puente firma/autentica los eventos.
8. La propiedad y los permisos por grupo se prueban end to end.

Solo después de esos puntos debe considerarse la promoción estable.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].

La suite `test/runtime-import.test.mjs` confirma que la versión fijada
de Baileys exporta su fábrica WebSocket y el cargador de estado. No abre
conexión ni guarda credenciales. La validación de carpetas de sesión funciona
tanto en Unix como en Windows mediante `path.relative`.
