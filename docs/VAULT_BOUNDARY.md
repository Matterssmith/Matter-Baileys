# WhatsApp-Matter — límites de credenciales y capacidades

Estado: requisito contractual; no conexión Vault real.

Los proveedores whatsapp-rust, baileys y whatsmeow son transportes y
adaptadores. Su responsabilidad es identificar eventos del protocolo
WhatsApp con procedencia comprobable, no descifrar registros de Vault.

Los bindings Go, TypeScript y JavaScript nunca reciben la clave maestra,
llaves de base de datos privada, credenciales GitHub o autorización para
actuar sobre sockets de otros bots. Los servicios consumidores deberán
autenticarse de manera independiente ante la autoridad protegida.

Un mensaje WhatsApp que contenga un comando no es una autorización.
Los datos del ingreso deben vincularse a una sesión real, socket, remitente,
chat, ID de mensaje y prueba contra replay. El canal protegido entre
componentes se implementará fuera de los clientes públicos.

El motor de Vault ya puede cifrar registros locales en Rust, con
pruebas sintéticas. No hay gateway autorizado desplegado. Por ello el
proveedor no debe cambiar la autenticación de Base ni enviar credenciales,
sesiones o solicitudes de control a Bot-Matter como intermediario.

Conservar pairing por código y el proveedor activo en Base hasta
concluir la migración validada y reversible. Los puertos de Baileys y
whatsmeow siguen pendientes: un adaptador que normaliza eventos no es
una implementación Rust completa de conectividad.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
