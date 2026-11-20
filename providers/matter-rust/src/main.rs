/*
 * Matter-Rust — unified native Rust WhatsApp provider executable.
 *
 * The single connected engine is the existing audited Rust WhatsApp
 * runtime from providers/whatsapp-rust/src/main.rs, reused in this
 * compiled executable, not started as another process.
 *
 * Baileys and WhatsMeow are format compatibility modules. Neither
 * Node nor Go is launched, and no existing WhatsApp session is reused
 * unless the host explicitly provisions and authorizes that session.
 *
 * The source include is a temporary migration strategy to prevent
 * two independently maintained live implementations.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
include!("../../whatsapp-rust/src/main.rs");
