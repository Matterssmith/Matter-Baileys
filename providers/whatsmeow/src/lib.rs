/*
 * WhatsApp-Matter — whatsmeow family event normalization in Rust.
 * Compatibility-only: NO Go transport, pairing or live client here.
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
pub mod engine;

use matter_whatsapp_contracts::{
    describe_provider, normalize_untrusted, NormalizeError, NormalizedTextEvent,
    ProviderDescriptor, UntrustedTextEvent,
};

pub fn descriptor() -> ProviderDescriptor {
    describe_provider("whatsmeow").expect("known adapter")
}

pub fn normalize(event: UntrustedTextEvent) -> Result<NormalizedTextEvent, NormalizeError> {
    normalize_untrusted("whatsmeow", event)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adapter_is_rust_only_and_does_not_grant_control() {
        let d = descriptor();
        assert_eq!(d.runtime, "Rust");
        assert!(!d.control_authority);
        assert_eq!(d.operations, ["describe"]);
    }
}

// Créditos:
// Hecho por Lournyx Matterssmith [Lou-Nyx].
