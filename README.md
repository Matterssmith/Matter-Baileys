# WhatsApp-Matter — beta congelada (nombre actual: Matter-Baileys)

**No iniciar Bot-Matter.** Este es un experimento aislado que pertenece al
ecosistema Matter. El usuario exige terminar **Base-Matter** antes de
desarrollar o integrar los componentes en Bot-Matter.

## Restricciones obligatorias

- **Solo pairing code.** No hay visualización ni ruta de login por QR.
  Baileys puede emitir internamente un evento \`qr\` como señal de que
  el transporte está listo, incluso al pedir código. Nunca se muestra ni
  se utiliza como método de vinculación.
- El \`#ping\` histórico fue una prueba y se retiró de esta beta.
  No hay respuesta automática, \`messages.upsert\` ni \`sendMessage\`.
- El control global del ecosistema usa \`=/>\` dentro de Base-Matter,
  independiente del futuro bot. Este prototipo no administra grupos ni roles.
- La sesión se mantiene solo en \`.local-auth/\`, carpeta ignorada por Git.
  Nunca subir contraseñas, credenciales Signal, sesiones, números privados
  ni claves a GitHub, Vault-Matter o Base-Matter.
- La misma sesión no debe abrirse desde dos procesos ni reutilizarse al
  mismo tiempo en otro servidor. No se eliminan sesiones automáticamente.

## Preparación para una prueba aislada posterior

Después de completar Base-Matter, una persona autorizada podrá probar un
número secundario propio desde un checkout de \`matter-beta\` con Node.js 22+.
El número se configura como dígitos internacionales sin el signo \`+\`:

\`\`\`sh
npm install
npm test
WA_PAIRING_PHONE=521XXXXXXXXXX npm run smoke
\`\`\`

El código temporal se muestra únicamente en la terminal del propietario.
En el teléfono: WhatsApp → Dispositivos vinculados → Vincular dispositivo →
Vincular con número de teléfono. Introducir allí el código.

La prueba no envía mensajes. Si la sesión ya estaba vinculada, no se solicita
un código nuevo. Si aparece un archivo \`.owner.lock\` tras una caída,
verificar primero que el proceso anterior realmente terminó; no borrar
automáticamente la sesión ni iniciar otra instancia concurrente.

Baileys es una librería no oficial. Las pruebas offline solo validan
dependencias, importaciones y los límites locales de código; no confirman
que una sesión real esté conectada. La implementación anterior de
\`whatsapp-rust\` por pairing code sigue disponible en el historial de
Base-Matter, commit \`3c36d97bb\`. Debe revisarse antes de escoger adaptador.

[Contrato técnico](docs/ADAPTER_CONTRACT.md)

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
