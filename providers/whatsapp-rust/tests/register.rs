/*
 * WhatsApp-Matter — exact community REGISTER description marker.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
use whatsapp_matter_rust_provider::official_register_description;

#[test]
fn register_must_begin_at_first_line_with_the_exact_parent_group_id() {
    let parent = "120363000000000@g.us";
    let valid = "MATTER:OFFICIAL:register:120363000000000@g.us\nGrupo oficial";
    assert!(official_register_description(valid, parent));
    assert!(official_register_description(
        "MATTER:OFFICIAL:register:120363000000000@g.us",
        parent
    ));
    for invalid in [
        "MATTER:OFFICIAL:register:120363000000000@g.us-suffix",
        "otro\nMATTER:OFFICIAL:register:120363000000000@g.us",
        " MATTER:OFFICIAL:register:120363000000000@g.us",
        "MATTER:OFFICIAL:register:120363000000000@g.us ",
        "MATTER:OFFICIAL:register:120363999999999@g.us",
        "MATTER:OFFICIAL:register:120363000000000@g.us\r\nRegistro",
    ] {
        assert!(!official_register_description(invalid, parent));
    }
    assert!(!official_register_description(valid, "../unsafe"));
    assert!(!official_register_description(valid, "not-a-community@lid"));
}
