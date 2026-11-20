/* Original low-volume Matterssmith 0.45-second two-note test tone.
 * A real Ogg/Opus audio message, never an automatic HTML sound.
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
pub const MENU_TONE_OGG: &[u8] = include_bytes!("../assets/menu-tone.ogg");

pub fn menu_tone_bytes() -> &'static [u8] {
    MENU_TONE_OGG
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menu_tone_has_valid_ogg_opus_header_and_size_bound() {
        assert!(MENU_TONE_OGG.starts_with(b"OggS"));
        assert!(MENU_TONE_OGG.windows(8).any(|part| part == b"OpusHead"));
        assert!((500..4096).contains(&MENU_TONE_OGG.len()));
    }
}
