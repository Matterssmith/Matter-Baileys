// Base-Matter — standalone WhatsApp provider control-frame presentation.
// This module is intentionally independent of Base-Matter's command registry.
// It never executes actions, grants permissions or accesses controller data.
use anyhow::{bail, Result};

pub fn decorate(body: &str) -> Result<String> {
    if body.is_empty() || body.len() > 3800 || body.chars().any(|ch| ch.is_control() && ch != '\n')
    {
        bail!("Invalid control response");
    }
    let template =
        include_str!("../../../templates/matterssmith-control.txt").replace("\r\n", "\n");
    let legacy_menu =
        include_str!("../../../templates/matterssmith-menu.txt").replace("\r\n", "\n");
    if body.trim_end_matches('\n') == legacy_menu.trim_end_matches('\n') {
        return Ok(legacy_menu.trim_end_matches('\n').to_owned());
    }
    let (prefix, suffix) = template
        .split_once("{{BODY}}")
        .ok_or_else(|| anyhow::anyhow!("Invalid control template"))?;
    let prefix = prefix
        .replace("{{TITLE}}", "Base-Matter")
        .replace("{{ACCENT}}", "✦");
    let suffix = suffix
        .replace("{{TITLE}}", "Base-Matter")
        .replace("{{ACCENT}}", "✦");
    if body.starts_with(&prefix) && body.trim_end().ends_with(suffix.trim_end()) {
        return Ok(body.to_owned());
    }
    if body.len() > 3200 {
        bail!("Control body exceeds template allowance");
    }
    let result = template
        .replace("{{TITLE}}", "Base-Matter")
        .replace("{{ACCENT}}", "✦")
        .replace("{{BODY}}", body.trim_end_matches('\n'));
    if result.len() > 3800 {
        bail!("Themed control reply exceeds transport limit");
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_template_decoration_is_idempotent() {
        let text = decorate("Listo").unwrap();
        assert!(text.contains("Listo"));
        assert_eq!(decorate(&text).unwrap(), text);
        assert!(decorate("").is_err());
        assert!(decorate("contains\u{0000}control").is_err());
    }
}
