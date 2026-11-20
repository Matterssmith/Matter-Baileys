# Interoperabilidad para desarrolladores

JavaScript: bindings/javascript/index.mjs, describe.mjs, index.test.mjs.
TypeScript: bindings/typescript/src/index.ts, tsconfig.json.
Go: bindings/go/contract/contract.go, contrato y pruebas.
Rust: crates/whatsapp-contracts.

Solo se consulta el descriptor de proveedor mediante operación describe.
No hay envío real, sesión, administración ni acceso a Vault desde bindings.
Más capacidades requerirán autorización del runtime.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
