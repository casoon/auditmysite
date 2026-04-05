//! Code fix generation — produces `suggested_code` for WCAG violations.
//!
//! Given the WCAG rule ID (e.g. "1.1.1"), the element's outer HTML snippet,
//! and optional context from the Violation, this module returns a concrete
//! HTML code example showing how the element should be fixed.

/// Maximum length of the html_snippet stored on a Violation.
/// Longer outer HTML is truncated with an ellipsis.
pub const HTML_SNIPPET_MAX: usize = 500;

/// Truncate outer HTML to [`HTML_SNIPPET_MAX`] characters.
pub fn truncate_html(html: String) -> String {
    if html.len() <= HTML_SNIPPET_MAX {
        return html;
    }
    let boundary = html
        .char_indices()
        .take_while(|(i, _)| *i <= HTML_SNIPPET_MAX)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}…", &html[..boundary])
}

/// Generate a concrete `suggested_code` string for a WCAG violation.
///
/// # Arguments
/// * `wcag_rule` – WCAG criterion, e.g. `"1.1.1"`, `"3.1.1"`.
/// * `html_snippet` – Outer HTML of the problematic element, if available.
/// * `role` – Accessibility role of the node, if known.
/// * `fix_suggestion` – Existing text-based fix from the rule, used as fallback.
pub fn generate_suggested_code(
    wcag_rule: &str,
    html_snippet: Option<&str>,
    role: Option<&str>,
    fix_suggestion: Option<&str>,
) -> Option<String> {
    match wcag_rule {
        "1.1.1" => suggest_alt_text(html_snippet, role),
        "2.4.2" => Some("<title>Aussagekräftiger Seitentitel</title>".to_string()),
        "3.1.1" => suggest_lang(html_snippet),
        "1.4.3" | "1.4.6" => suggest_contrast(fix_suggestion),
        "4.1.2" => suggest_name_role_value(html_snippet, role),
        "2.4.4" | "2.4.9" => suggest_link_purpose(html_snippet),
        "1.3.1" => suggest_form_label(html_snippet, role),
        "2.1.1" | "2.1.3" => suggest_keyboard(html_snippet, role),
        "1.3.5" => suggest_autocomplete(html_snippet),
        _ => None,
    }
}

// ─── Rule-specific generators ────────────────────────────────────────────────

fn suggest_alt_text(html_snippet: Option<&str>, role: Option<&str>) -> Option<String> {
    let html = html_snippet?;

    // SVG: recommend <title> inside the element
    if role == Some("graphics-document") || role == Some("graphics-symbol") || html.contains("<svg")
    {
        return Some(
            "<svg aria-label=\"[Grafik beschreiben]\" role=\"img\">\n  <title>[Grafik beschreiben]</title>\n  …\n</svg>"
                .to_string(),
        );
    }

    // img tag: inject alt attribute
    if html.trim_start().starts_with("<img") {
        let fixed = inject_attribute(html, "alt", "[Bildbeschreibung ergänzen]");
        return Some(fixed);
    }

    // icon / other img-role element: recommend aria-label
    if let Some(fixed) = inject_attribute_if_missing(html, "aria-label", "[Beschreibung ergänzen]")
    {
        return Some(fixed);
    }

    None
}

fn suggest_lang(html_snippet: Option<&str>) -> Option<String> {
    if let Some(html) = html_snippet {
        if html.contains("<html") {
            let fixed = inject_attribute(html, "lang", "de");
            return Some(fixed);
        }
    }
    Some("<html lang=\"de\">".to_string())
}

fn suggest_contrast(fix_suggestion: Option<&str>) -> Option<String> {
    // Contrast fixes require knowing the actual colors; provide a commented hint
    let base = "/* Kontrastverhältnis erhöhen — mindestens 4.5:1 für normalen Text, 3:1 für großen Text */";
    if let Some(fix) = fix_suggestion {
        Some(format!("{}\n/* Hinweis: {} */", base, fix))
    } else {
        Some(base.to_string())
    }
}

fn suggest_name_role_value(html_snippet: Option<&str>, role: Option<&str>) -> Option<String> {
    let html = html_snippet?;

    // Button without accessible name
    if html.contains("<button") || role == Some("button") {
        return Some(inject_attribute(
            html,
            "aria-label",
            "[Schaltfläche beschreiben]",
        ));
    }

    // Input without label — show label wrapper
    if html.contains("<input") || role == Some("textbox") || role == Some("combobox") {
        let id_hint = extract_attr(html, "id")
            .or_else(|| extract_attr(html, "name"))
            .unwrap_or("field-id");
        return Some(format!(
            "<label for=\"{id}\">[Feldbeschriftung]</label>\n{html}",
            id = id_hint,
            html = ensure_attr(html, "id", id_hint),
        ));
    }

    // Generic interactive element
    if let Some(fixed) =
        inject_attribute_if_missing(html, "aria-label", "[Zugänglichen Namen ergänzen]")
    {
        return Some(fixed);
    }

    None
}

fn suggest_link_purpose(html_snippet: Option<&str>) -> Option<String> {
    let html = html_snippet?;
    if html.contains("<a ") || html.starts_with("<a>") {
        let fixed = inject_attribute(html, "aria-label", "[Linkziel beschreiben]");
        return Some(fixed);
    }
    None
}

fn suggest_form_label(html_snippet: Option<&str>, role: Option<&str>) -> Option<String> {
    let html = html_snippet?;

    if html.contains("<input")
        || html.contains("<select")
        || html.contains("<textarea")
        || role == Some("textbox")
        || role == Some("combobox")
        || role == Some("listbox")
    {
        let id_hint = extract_attr(html, "id")
            .or_else(|| extract_attr(html, "name"))
            .unwrap_or("field-id");
        let tagged = ensure_attr(html, "id", id_hint);
        return Some(format!(
            "<label for=\"{id}\">[Feldbeschriftung]</label>\n{html}",
            id = id_hint,
            html = tagged,
        ));
    }

    None
}

fn suggest_keyboard(html_snippet: Option<&str>, role: Option<&str>) -> Option<String> {
    let html = html_snippet?;

    // Non-interactive element used as button/link
    if role == Some("button") && !html.contains("<button") {
        return Some(inject_attribute(html, "tabindex", "0"));
    }

    Some(format!(
        "<!-- Element mit tabindex=\"0\" und keydown-Handler versehen -->\n{}",
        inject_attribute(html, "tabindex", "0")
    ))
}

fn suggest_autocomplete(html_snippet: Option<&str>) -> Option<String> {
    let html = html_snippet?;
    if html.contains("<input") {
        let fixed = inject_attribute(html, "autocomplete", "[z. B. name, email, tel]");
        return Some(fixed);
    }
    None
}

// ─── HTML attribute helpers ───────────────────────────────────────────────────

/// Add or overwrite `name="value"` on the first tag in `html`.
/// If the attribute already exists with a non-empty value it is left unchanged.
fn inject_attribute(html: &str, name: &str, value: &str) -> String {
    // Find end of opening tag
    if let Some(close) = html.find('>') {
        let (tag, _rest) = html.split_at(close);

        // If attribute already present with a value, leave it
        let attr_pattern = format!("{}=", name);
        if tag.contains(&attr_pattern) {
            return html.to_string();
        }

        // Self-closing tag: insert before ' />' or '>'
        let insert_point = if tag.ends_with(" /") || tag.ends_with('/') {
            close - 1
        } else {
            close
        };

        let (before, after_slash) = html.split_at(insert_point);
        format!(
            "{before} {name}=\"{value}\"{after_slash}",
            before = before,
            name = name,
            value = value,
            after_slash = after_slash,
        )
    } else {
        html.to_string()
    }
}

/// Like `inject_attribute`, but only inject when the attribute is absent.
/// Returns `None` when the attribute already exists.
fn inject_attribute_if_missing(html: &str, name: &str, value: &str) -> Option<String> {
    let attr_pattern = format!("{}=", name);
    if html.contains(&attr_pattern) {
        return None;
    }
    Some(inject_attribute(html, name, value))
}

/// Ensure an element has the given `id` attribute. If it already has one,
/// return the html unchanged; otherwise inject `id="<value>"`.
fn ensure_attr(html: &str, attr: &str, value: &str) -> String {
    let pattern = format!("{}=", attr);
    if html.contains(&pattern) {
        html.to_string()
    } else {
        inject_attribute(html, attr, value)
    }
}

/// Extract the value of a simple attribute from an HTML snippet.
fn extract_attr<'a>(html: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{}=\"", attr);
    let start = html.find(&needle)? + needle.len();
    let end = html[start..].find('"')? + start;
    let val = &html[start..end];
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_alt_into_img() {
        let html = "<img src=\"/logo.png\" class=\"logo\">";
        let result = inject_attribute(html, "alt", "Logo");
        assert!(result.contains("alt=\"Logo\""));
        assert!(result.contains("src=\"/logo.png\""));
    }

    #[test]
    fn inject_does_not_overwrite_existing() {
        let html = "<img src=\"/x.png\" alt=\"existing\">";
        let result = inject_attribute(html, "alt", "new");
        assert!(result.contains("alt=\"existing\""));
        assert!(!result.contains("alt=\"new\""));
    }

    #[test]
    fn suggest_alt_for_img() {
        let code = generate_suggested_code(
            "1.1.1",
            Some("<img src=\"/hero.jpg\">"),
            Some("image"),
            None,
        );
        assert!(code.is_some());
        let c = code.unwrap();
        assert!(c.contains("alt="));
    }

    #[test]
    fn suggest_lang_html_tag() {
        let code = generate_suggested_code("3.1.1", Some("<html>"), None, None);
        let c = code.unwrap();
        assert!(c.contains("lang=\"de\""));
    }

    #[test]
    fn suggest_lang_fallback() {
        let code = generate_suggested_code("3.1.1", None, None, None);
        assert_eq!(code.unwrap(), "<html lang=\"de\">");
    }

    #[test]
    fn suggest_page_title() {
        let code = generate_suggested_code("2.4.2", None, None, None);
        assert!(code.unwrap().contains("<title>"));
    }

    #[test]
    fn truncate_long_html() {
        let long = "x".repeat(600);
        let result = truncate_html(long);
        assert!(result.len() <= HTML_SNIPPET_MAX + 4); // ellipsis is multi-byte
    }

    #[test]
    fn extract_attr_value() {
        let html = "<input id=\"email\" type=\"text\">";
        assert_eq!(extract_attr(html, "id"), Some("email"));
        assert_eq!(extract_attr(html, "type"), Some("text"));
        assert_eq!(extract_attr(html, "name"), None);
    }
}
