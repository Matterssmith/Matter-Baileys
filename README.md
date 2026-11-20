## 2026-11-19 — Política obligatoria Vault-D019: protección equivalente a credenciales

- **Regla permanente del ecosistema:** TODOS los activos administrados por Vault — roles, permisos, decisiones, credenciales, políticas, metadatos, evidencia, auditoría, caché, respaldos y estados — deben cifrarse/autenticarse en reposo y transmitirse por canales cifrados y autenticados. Descifrado temporal sólo en memoria confiable; claves fuera de Git, nunca textos sensibles en logs, respuestas públicas ni archivos temporales.
- Cada proveedor/adaptador puede manejar transporte y eventos, pero no recibir ni persistir en claro claves, grants, identidad Vault, sesión ajena ni credenciales compartidas. Vincular servicio Vault sólo por canal autenticado y cifrado; cifrar cualquier estado gestionado por Vault, incluyendo audit/replay/metadata relevante, y denegar si falla el canal. El proveedor no interpreta un JSON, alias o prefijo como permiso. Cifrado en tránsito NO sustituye resguardo de sesión ni cifrado Vault. No cambiar las sesiones actuales.
- La norma principal se mantiene en la política privada VAULT-D019. Esta actualización es documental y no acredita gateway Vault, ejecución privada o migración de sesiones. No conceder permisos sin verificación y no afirmar pruebas funcionales.
- Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].

## 2026-11-19 — Reparación de CI del proveedor extraído

- En GitHub Actions (run 44647572370) fallaban rustfmt de `crates/whatsapp-contracts/src/lib.rs` y dos tests de Rich HTML en Windows por los finales de línea CRLF incluidos desde `templates/matterssmith-menu.txt`. Se normaliza CRLF a LF **solo para la vista previa offline**, manteniendo rechazo de otros caracteres de control.
- No se cambian sesiones, comunicación real ni pairing; Baileys/whatsmeow siguen como adaptadores Rust de normalización, NO transportes completos.
- Confirmar tests Linux/Windows tras el commit y conservar proveedor en Base hasta completar corte controlado de origen. Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].

# WhatsApp-Matter — Rust nativo con interoperabilidad Go, TypeScript y JavaScript

**Estado (2026-11-19): extracción en curso, NO corte de producción.**
Los fuentes, tests y recursos del proveedor whatsapp-rust actualmente usado por
Base-Matter ya se copiaron a providers/whatsapp-rust/. Base-Matter conserva
su proveedor original y no se modificaron sus sesiones ni el pairing code.

## Directorios

    providers/whatsapp-rust/       Proveedor Rust real (biblioteca whatsapp-rust 0.7.0)
    providers/baileys/             Adaptador Rust, NO transporte de Baileys portado
    providers/whatsmeow/           Adaptador Rust, NO transporte whatsmeow portado
    crates/whatsapp-contracts/     Protocolo Rust versionado de solo lectura
    bindings/javascript/          Cliente JS y pruebas
    bindings/typescript/          Cliente TypeScript
    bindings/go/                  Cliente Go y pruebas
    templates/                    Plantillas del proveedor nativo

Todas las familias tendrán una interfaz Rust común. Baileys se origina en
TypeScript/JavaScript y whatsmeow en Go. Sus directorios actuales son de
normalización Rust; aún NO contienen implementaciones funcionales del
protocolo WhatsApp. No se ha importado código de terceros ni se afirma
una conversión completa. Cualquier port exige revisión de licencia y tests.

## Contrato de interoperabilidad

El binario Rust matter_whatsapp_bridge procesa JSON por líneas desde stdin.
Únicamente acepta la operación describe; NO envía mensajes, no conecta
cuentas, no consulta secretos y no autoriza comandos. Go, JS y TS tienen
bindings para este contrato; el servicio de mensajería autenticado es futuro.

Pruebas offline desde la raíz:

    cargo fmt --all -- --check
    cargo test -p matter-whatsapp-contracts -p matter-whatsapp-baileys-adapter -p matter-whatsapp-whatsmeow-adapter --all-targets
    cargo test -p whatsapp-matter-rust-provider --all-targets
    node --test bindings/javascript/index.test.mjs
    node bindings/javascript/describe.mjs whatsapp-rust

Para Go, desde bindings/go:

    go test ./...

No ejecutar dos proveedores con el MISMO almacén de sesión ni copiar SQLite,
keys o tokens al repositorio. El traslado del proceso de Base solo podrá
realizarse tras CI, autenticación del puente y pairing de sesión de prueba
aislada. Vault-Matter sigue siendo autoridad de permisos; Bot-Matter permanece
bloqueado hasta cumplir los requisitos de Base-Matter.

Documentos: docs/PROVIDER_MIGRATION.md y docs/WIRE_PROTOCOL.md.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
