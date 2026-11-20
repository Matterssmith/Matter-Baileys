## 2026-11-19 — Migración inicial Rust y contratos Go/TypeScript/JavaScript

- Copia completa de proveedor Rust desde Base-Matter alpha (fuentes, tests, plantillas y audio); Base sigue ejecutando su proveedor original.
- Workspace Rust con compatibilidad Baileys/whatsmeow de normalización únicamente. NO transportes portados ni sesiones nuevas.
- Contratos y bindings JSON v1 describe solo lectura para Rust, Go, TS y JS, con pruebas automáticas y reglas de CI.
- Guiar la continuación mediante docs/PROVIDER_MIGRATION.md y docs/WIRE_PROTOCOL.md. No hay Vault operativo ni capacidad de mutación externa.
- CI y pairing reales pendientes al escribir este commit. Mantener Bot-Matter bloqueado; README revisado.
- Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].

# Adaptador WhatsApp y pruebas de interfaz del archivo

**Fecha:** 2026-11-19. **Estado:** especificación a validar, sin código activo. **Requisitos:** MAT-ARCH-013,014,017; MAT-COM-003,017. [Fuente clasificada](https://github.com/Matterssmith/Docs-Matter/blob/matter/docs/ARCHIVE_IDEAS_SOURCE_INDEX.md).

## Necesidades e instrucciones

1. La prueba de menú nativo/NativeFlow y botones depende del proveedor WhatsApp Rust realmente instalado en Base-Matter. No duplicar Baileys antiguo ni anunciar funciones cliente no verificadas.
2. AutoAccept del archivo histórico (líneas 5980+): requerirá autorización del dueño de socket/grupo, OFF por defecto, invitaciones seguras, rate limits y auditoría. No aceptar solicitudes automáticamente solo por escribir una orden en un chat público. No implementar todavía: Bot-Matter permanece bloqueado.
3. Music Menu Generator (líneas 6223+): utilizar solo audio **original o licenciado**; WhatsApp no ejecuta HTML arbitrario ni sonidos automáticos en un menú. Si se prueba música se envía un audio reproducible y nunca se afirma autoplay.
4. Sticker System (líneas 11894+): separar conversión/pack básico de automatizaciones avanzadas, copyright y límites por tamaño, formato y recursos. No activar generación sin tests y controles.
5. Integraciones futuras de Marketplace-Matter permitirán estado de pedido únicamente con usuario autenticado, consentimiento y recibos mínimos; mensajes no pueden conceder propiedad ni ejecutar pagos. Invitados sí pueden usar comandos de lectura cuando su chat está habilitado.
6. Sin duplicar roles, anti, lista de dominios o gating ya definidos en Base-Matter. Continuidad de menú y HMAC protegidos.
**Aceptación:** UI confirmada desde dispositivo cliente, callback interactivo autenticado, no replay, rechazo de escalada, grupos OFF por defecto, música legal y no fuga de datos.

## Flujo responsable

Equipo responsable deberá documentar actor/permiso, compatibilidad, secretos, coste, límites y tests antes de cualquier habilitación. No modificar Bot-Matter, no importar datos privados del ZIP, no confundir una demo con una función compatible en todos los móviles.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
