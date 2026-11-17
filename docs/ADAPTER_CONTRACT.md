# Contrato del experimento WhatsApp-Matter

## Estado

Repositorio de plataforma independiente. No es Bot-Matter.
Solo se hacen correcciones para eliminar las rutas de QR y \`#ping\` que
contradecían los requisitos del proyecto. El desarrollo posterior queda
congelado hasta completar Base-Matter.

El historial de Base-Matter contiene un proveedor anterior que ya
empleaba \`whatsapp-rust\` y PairCodeOptions:
https://github.com/Matterssmith/Matter-Base/blob/3c36d97bb/src/providers/whatsapp/mod.rs

## Pairing code obligatorio

Se pide el código únicamente para sesiones no registradas. El número del
propietario debe incluir el prefijo internacional y solamente dígitos.
No se muestra QR y no hay alternativa QR. La biblioteca puede emitir el
evento de QR como señal interna de disponibilidad del transporte, incluso
en modo pairing code; no se procesa ni se muestra como QR.

El adaptador está restringido al almacenamiento privado e ignorado
\`.local-auth/\`, sin enviar mensajes. La prueba no implementa \`=/>on\`,
\`=/>set role\` ni comandos de Bot-Matter. Estos son contratos exclusivos
de Base-Matter, con políticas de socket.

La futura conexión real requiere verificar identidad, consentimiento,
propiedad del socket, revocación, persistencia protegida de credenciales,
reconexión segura, límites entre procesos y transporte autenticado con Base.
No fusionar esta beta a estable ni activar Bot-Matter antes de cerrar Base.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
