# WhatsApp-Matter — Migración de proveedores por etapas

Estado: copia de fuentes y contrato inicial, NO migración final ni
conexión nueva verificada. Origen: Matterssmith/Base-Matter alpha
SHA 5d0897cd8c71a895d5e7dc76777494e2ce7ffceb.

## Separación

Base-Matter mantiene control, admisión y runtime de bots, no protocolos
WhatsApp internos. WhatsApp-Matter contiene conectividad y eventos. Vault
conserva autoridad sobre roles/operaciones protegidas. Bot-Matter utiliza
los servicios únicamente cuando Base cumpla la aceptación. SDK-Matter
documenta cómo utilizar los contratos públicos.

## Trabajo ya guardado

- providers/whatsapp-rust contiene una copia completa del Rust nativo de
  Base alpha, tests existentes, templates y archivo audio. Su dependencia
  externa whatsapp-rust=0.7.0 no ha sido reimplementada.
- providers/baileys y providers/whatsmeow tienen paquetes Rust con
  normalización estrictamente no autenticada. NO establecen conexión,
  pairing, envío ni moderación. Ningún código externo se copió.
- crates/whatsapp-contracts declara una API Rust de lectura y normaliza
  texto sin atribuir identidad verificada.
- bindings/go, bindings/javascript y bindings/typescript implementan
  el protocolo v1 de consulta descriptiva. Ninguno tiene permisos Vault.
- CI prueba los contratos y el proveedor copiado offline.

## Contrato futuro para migración real

1. Registrar capabilities y formatos de eventos de la plataforma.
2. Garantizar correspondencia de firmas HMAC, identidad socket, grupo,
   nonce y replay contra Base; no copiar secretos a clientes.
3. Demostrar tests de compilación y proveedor Linux/Windows al SHA exacto.
4. Probar sesión WhatsApp mediante pairing code en almacenamiento nuevo
   separado; NO levantar simultáneamente la misma sesión de Base.
5. Cambiar el lanzador de Base para consumir un paquete versionado del
   nuevo repositorio con opción de rollback, sin borrar sesiones.
6. Tras validar transporte y recuperación real, retirar el duplicado
   de Base en otro commit, preservando compatibilidad API.
7. Portar las capacidades de las familias Baileys (TS/JS) y
   whatsmeow (Go) a Rust mediante adaptadores/implementaciones
   independientes, solo cuando exista autorización, licencia compatible
   y prueba real. No prometer portar automáticamente protocolos ajenos.

## Límites

La información JSON entrante no es prueba de propiedad de un socket ni de
pertenencia a grupo. La autenticación se efectúa en el ingreso y Vault.
Una implementación privada se ejecuta en un servicio protegido, no se
entrega al desarrollador. Todos los antis siguen inactivos por defecto
hasta habilitación del titular autenticado.

Sin CI final y prueba real no declarar WhatsApp-Matter listo para reemplazar
el proveedor de Base.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
