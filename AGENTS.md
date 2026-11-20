## 2026-11-19 — Migración inicial Rust y contratos Go/TypeScript/JavaScript

- Copia completa de proveedor Rust desde Base-Matter alpha (fuentes, tests, plantillas y audio); Base sigue ejecutando su proveedor original.
- Workspace Rust con compatibilidad Baileys/whatsmeow de normalización únicamente. NO transportes portados ni sesiones nuevas.
- Contratos y bindings JSON v1 describe solo lectura para Rust, Go, TS y JS, con pruebas automáticas y reglas de CI.
- Guiar la continuación mediante docs/PROVIDER_MIGRATION.md y docs/WIRE_PROTOCOL.md. No hay Vault operativo ni capacidad de mutación externa.
- CI y pairing reales pendientes al escribir este commit. Mantener Bot-Matter bloqueado; README revisado.
- Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].

# Instrucciones WhatsApp-Matter

Leer docs/PROJECT_IDEA_REQUIREMENTS.md. Todos los cambios nuevos necesitan autorización, pruebas de cliente real, controles de permisos, licencia de contenido, aislamiento y actualización README/continuidad. Bot-Matter bloqueado y separación de roles Vault innegociable.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
