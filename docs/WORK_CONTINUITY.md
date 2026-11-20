## 2026-11-19 — Reparación de CI del proveedor extraído

- En GitHub Actions (run 44647572370) fallaban rustfmt de `crates/whatsapp-contracts/src/lib.rs` y dos tests de Rich HTML en Windows por los finales de línea CRLF incluidos desde `templates/matterssmith-menu.txt`. Se normaliza CRLF a LF **solo para la vista previa offline**, manteniendo rechazo de otros caracteres de control.
- No se cambian sesiones, comunicación real ni pairing; Baileys/whatsmeow siguen como adaptadores Rust de normalización, NO transportes completos.
- Confirmar tests Linux/Windows tras el commit y conservar proveedor en Base hasta completar corte controlado de origen. Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].

## 2026-11-19 — Migración inicial Rust y contratos Go/TypeScript/JavaScript

- Copia completa de proveedor Rust desde Base-Matter alpha (fuentes, tests, plantillas y audio); Base sigue ejecutando su proveedor original.
- Workspace Rust con compatibilidad Baileys/whatsmeow de normalización únicamente. NO transportes portados ni sesiones nuevas.
- Contratos y bindings JSON v1 describe solo lectura para Rust, Go, TS y JS, con pruebas automáticas y reglas de CI.
- Guiar la continuación mediante docs/PROVIDER_MIGRATION.md y docs/WIRE_PROTOCOL.md. No hay Vault operativo ni capacidad de mutación externa.
- CI y pairing reales pendientes al escribir este commit. Mantener Bot-Matter bloqueado; README revisado.
- Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].

## 2026-11-19 — Enlace corregido

- SHA anterior: 8dd6044ec27630d6d60de7747dbda99a45788db8.
- Corregido únicamente el enlace relativo Markdown del índice de ideas en README; sin cambios de funcionalidad.
- README revisado y validado; CI de código no aplicable.

# Continuidad — archivo Matter

- Fecha 2026-11-19; SHA previo 57226de377c1dfeecd9de49415fb593c8ac01936; rama matter.
- Se crearon requisitos MAT-ARCH-013,014,017; MAT-COM-003,017 y apuntadores README/AGENTS; no se cambió código de proveedor ni runtime.
- Fuente: Proyecto-Matter.zip, no publicado por contener información privada.
- No hay tests funcionales de este documento; estado idea.
- Siguiente: revisión responsable, pruebas seguras y aprobación por fase. Bot-Matter bloqueado.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
