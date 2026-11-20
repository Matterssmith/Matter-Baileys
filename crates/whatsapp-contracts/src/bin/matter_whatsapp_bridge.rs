/*
 * WhatsApp-Matter — Read-only JSON-lines protocol probe.
 * Does NOT send or receive actual WhatsApp traffic.
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use matter_whatsapp_contracts::{handle_json, WireResponse, MAX_REQUEST_BYTES};
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let response = match line {
            Ok(value) if value.len() <= MAX_REQUEST_BYTES => handle_json(&value),
            Ok(_) => WireResponse::denied("request_too_large"),
            Err(_) => WireResponse::denied("read_failed"),
        };
        let body = serde_json::to_string(&response).map_err(io::Error::other)?;
        writeln!(stdout, "{body}")?;
        stdout.flush()?;
    }
    Ok(())
}

// Créditos:
// Hecho por Lournyx Matterssmith [Lou-Nyx].
