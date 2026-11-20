## 2026-11-19 — Separación Vault y proveedores WhatsApp

- La clave maestra y datos privados quedan exclusivamente bajo custodia del servicio Vault, no en proveedores ni bindings Go/JS/TS ni Bot-Matter. El transporte sólo entrega eventos verificables, sin otorgar autoridad por texto o prefijo.
- Vault ya tiene un núcleo de cifrado local probado en Rust; gateway autenticado remoto aún pendiente. Ningún proveedor puede descifrar Vault ni reutilizar identidades de otro socket.
- Continuidad: docs/VAULT_BOUNDARY.md. No se traslada sesión ni se declara port real de Baileys/whatsmeow, ni se toca código ejecutable. Revisado README; Bot-Matter bloqueado. Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].

## 2026-11-19 — Política obligatoria Vault-D019: protección equivalente a credenciales

- **Regla permanente del ecosistema:** TODOS los activos administrados por Vault — roles, permisos, decisiones, credenciales, políticas, metadatos, evidencia, auditoría, caché, respaldos y estados — deben cifrarse/autenticarse en reposo y transmitirse por canales cifrados y autenticados. Descifrado temporal sólo en memoria confiable; claves fuera de Git, nunca textos sensibles en logs, respuestas públicas ni archivos temporales.
- Cada proveedor/adaptador puede manejar transporte y eventos, pero no recibir ni persistir en claro claves, grants, identidad Vault, sesión ajena ni credenciales compartidas. Vincular servicio Vault sólo por canal autenticado y cifrado; cifrar cualquier estado gestionado por Vault, incluyendo audit/replay/metadata relevante, y denegar si falla el canal. El proveedor no interpreta un JSON, alias o prefijo como permiso. Cifrado en tránsito NO sustituye resguardo de sesión ni cifrado Vault. No cambiar las sesiones actuales.
- La norma principal se mantiene en la política privada VAULT-D019. Esta actualización es documental y no acredita gateway Vault, ejecución privada o migración de sesiones. No conceder permisos sin verificación y no afirmar pruebas funcionales.
- Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].

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

# Instrucciones WhatsApp-Matter

Leer docs/PROJECT_IDEA_REQUIREMENTS.md. Todos los cambios nuevos necesitan autorización, pruebas de cliente real, controles de permisos, licencia de contenido, aislamiento y actualización README/continuidad. Bot-Matter bloqueado y separación de roles Vault innegociable.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
