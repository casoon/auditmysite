//! WCAG 3.3.2 Labels or Instructions
//!
//! Labels or instructions are provided when content requires user input.
//! Level A

use crate::accessibility::{AXNode, AXTree, NameSource};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 3.3.2
pub const INSTRUCTIONS_RULE: RuleMetadata = RuleMetadata {
    id: "3.3.2",
    name: "Labels or Instructions",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Labels or instructions are provided when content requires user input",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html",
    axe_id: "label",
    tags: &["wcag2a", "wcag332", "cat.forms"],
};

/// Check for labels and instructions on form controls
pub fn check_instructions(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        results.nodes_checked += 1;
        let role_lower = node.role.as_deref().unwrap_or("").to_lowercase();

        // Check form inputs
        if is_form_input(&role_lower) && !in_native_date_time_input(node, tree) {
            let has_label = has_accessible_label(node);
            let has_instructions = has_instructions_or_hint(node);

            if !has_label {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Critical,
                    format!("Form control '{}' has no accessible label", role_lower),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Add a <label> element, aria-label, or aria-labelledby attribute")
                .with_help_url(INSTRUCTIONS_RULE.help_url)
                .with_rule_id(INSTRUCTIONS_RULE.axe_id);

                results.add_violation(violation);
                continue;
            }

            // Placeholder used as the only label: Chrome computes the
            // accessible name FROM the placeholder in this case, so `name`
            // is non-empty (has_label above is true) — name_source is the
            // only reliable signal, not a `placeholder` AX property (which
            // doesn't exist; the old `has_placeholder_text` check based on
            // it was dead code, on top of being structurally unreachable
            // here since it re-tested `!has_label` after already requiring
            // `has_label` above) (#QA-030).
            if node.name_source == Some(NameSource::Placeholder) && !has_instructions {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Medium,
                    "Placeholder used as only label",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Add a visible <label> element. Placeholder should supplement, not replace, labels")
                .with_help_url(INSTRUCTIONS_RULE.help_url)
            .with_rule_id(INSTRUCTIONS_RULE.axe_id);

                results.add_violation(violation);
            }

            // Check for required fields without indication
            if is_required(node) && !indicates_required(node) {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Medium,
                    "Required field not clearly indicated",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Add visual indicator (e.g., asterisk *) and screen reader text for required fields")
                .with_help_url(INSTRUCTIONS_RULE.help_url)
            .with_rule_id(INSTRUCTIONS_RULE.axe_id);

                results.add_violation(violation);
            }

            // Check for inputs with format requirements. Guessed from the
            // label wording alone, so it can only ask for review, never
            // confirm a failure (#643).
            if needs_format_instructions(node) && !has_format_hint(node) {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Low,
                    format!("Input '{}' may require format instructions", role_lower),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Consider adding format instructions (e.g., 'DD/MM/YYYY' for dates)")
                .with_help_url(INSTRUCTIONS_RULE.help_url)
                .with_rule_id(INSTRUCTIONS_RULE.axe_id)
                .as_warning();

                results.add_violation(violation);
            }

            // If no violations found for this input, count as pass
            if has_label {
                results.passes += 1;
            }
        }

        // Check fieldsets without legends.
        // Chrome exposes many generic HTML elements (<details>, <address>, <hgroup>, ...)
        // as role="group" — these are NOT form groups and must be excluded.
        // Only flag actual form grouping: explicit role="radiogroup" or an htmlTag of
        // FIELDSET, or a group that contains form controls.
        if role_lower == "radiogroup" || (role_lower == "group" && is_form_group(node, tree)) {
            if !has_group_label(node) {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Medium,
                    "Form group has no legend or label",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix("Use <fieldset> with <legend>, or add aria-labelledby to the group")
                .with_help_url(INSTRUCTIONS_RULE.help_url)
                .with_rule_id(INSTRUCTIONS_RULE.axe_id);

                results.add_violation(violation);
            } else {
                results.passes += 1;
            }
        }
    }

    results
}

/// Chrome renders `<input type=date|time|month|week|datetime-local>` with
/// internal day/month/year/hour fields exposed as `spinbutton`s below the
/// input's own `Date`/`DateTime`/`InputTime` node. Those fields and their
/// format belong to the browser, not the author (#656).
fn in_native_date_time_input(node: &AXNode, tree: &AXTree) -> bool {
    let mut current = node.parent_id.as_deref();
    while let Some(parent) = current.and_then(|id| tree.get_node(id)) {
        if matches!(
            parent.role.as_deref(),
            Some("Date" | "DateTime" | "InputTime")
        ) {
            return true;
        }
        current = parent.parent_id.as_deref();
    }
    false
}

/// Check if role is a form input
fn is_form_input(role: &str) -> bool {
    matches!(
        role,
        "textbox"
            | "searchbox"
            | "combobox"
            | "listbox"
            | "spinbutton"
            | "slider"
            | "checkbox"
            | "radio"
            | "switch"
            | "textarea"
    )
}

/// Check if node has an accessible label
fn has_accessible_label(node: &AXNode) -> bool {
    if let Some(name) = &node.name {
        if !name.trim().is_empty() {
            return true;
        }
    }
    false
}

/// Check if node has placeholder
fn has_placeholder(node: &AXNode) -> bool {
    node.get_property_str("placeholder")
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

/// Check if node has instructions or hint text
fn has_instructions_or_hint(node: &AXNode) -> bool {
    if let Some(desc) = &node.description {
        if !desc.trim().is_empty() {
            return true;
        }
    }
    false
}

/// Check if field is marked as required
fn is_required(node: &AXNode) -> bool {
    node.get_property_bool("required").unwrap_or(false)
}

/// Check if required status is indicated
fn indicates_required(node: &AXNode) -> bool {
    if let Some(name) = &node.name {
        let name_lower = name.to_lowercase();
        let required_terms = [
            "required",
            // German
            "pflichtfeld",
            "pflicht",
            "erforderlich",
            // French
            "obligatoire",
            "requis",
            "champ obligatoire",
            // Spanish
            "obligatorio",
            "requerido",
            "campo requerido",
            // Italian
            "obbligatorio",
            "richiesto",
            // Portuguese
            "obrigatório",
            "campo obrigatório",
            // Dutch
            "verplicht",
            "vereist",
            // Swedish
            "obligatoriskt",
            "krävs",
            // Polish
            "wymagane",
            // Turkish
            "zorunlu",
        ];
        if required_terms
            .iter()
            .any(|t| name_lower.contains(t) || name_lower.contains('*'))
        {
            return true;
        }
    }

    if let Some(desc) = &node.description {
        let desc_lower = desc.to_lowercase();
        if desc_lower.contains("required")
            || desc_lower.contains("pflichtfeld")
            || desc_lower.contains("obligatoire")
            || desc_lower.contains("obligatorio")
            || desc_lower.contains("obbligatorio")
            || desc_lower.contains("obrigatório")
            || desc_lower.contains("verplicht")
        {
            return true;
        }
    }

    false
}

/// Check if the label asks for data with a format the user has to know.
///
/// Decided from the label alone. The `spinbutton` role is no signal of its
/// own: WCAG 3.3.2 asks for instructions where input must follow a format
/// the user cannot infer, and a spinbutton — native `<input type=number>` or
/// custom `role="spinbutton"` — holds a number the widget itself constrains
/// and steps with the arrow keys. A spinbutton whose label asks for a
/// format-sensitive value ("Postal code") is still caught by the label (#656).
fn needs_format_instructions(node: &AXNode) -> bool {
    let name = node.name.as_deref().unwrap_or("").to_lowercase();

    let format_sensitive = [
        // English
        "date",
        "phone",
        "tel",
        "zip",
        "postal",
        "credit card",
        "ssn",
        "social security",
        "passport",
        "account",
        "routing",
        // German
        "datum",
        "telefon",
        "postleitzahl",
        "plz",
        "kreditkarte",
        "reisepass",
        "kontonummer",
        // French
        "téléphone",
        "code postal",
        "carte de crédit",
        "passeport",
        "numéro de compte",
        // Spanish
        "teléfono",
        "código postal",
        "tarjeta de crédito",
        "pasaporte",
        "número de cuenta",
        // Italian
        "telefono",
        "codice postale",
        "carta di credito",
        "passaporto",
        "numero di conto",
        // Portuguese
        "telefone",
        "código postal",
        "cartão de crédito",
        "passaporte",
        // Dutch
        "telefoon",
        "postcode",
        "creditcard",
        "paspoort",
        "rekeningnummer",
        // Swedish
        "telefon",
        "postnummer",
        "kreditkort",
        "pass",
        // Polish
        "telefon",
        "kod pocztowy",
        "karta kredytowa",
        "paszport",
        // Turkish
        "telefon",
        "posta kodu",
        "kredi kartı",
        "pasaport",
    ];

    format_sensitive
        .iter()
        .any(|&term| contains_format_term(&name, term))
}

/// Terms shorter than five characters ("pass", "tel", "date", "zip", "plz",
/// "ssn") only count as whole words: as substrings they fire on unrelated
/// labels — "Was ist passiert?" is not a passport field, "Hotel" not a
/// phone number (#643). Longer terms keep substring matching so compounds
/// such as "Geburtsdatum" or "Telefonnummer" still count.
fn contains_format_term(name: &str, term: &str) -> bool {
    if term.chars().count() >= 5 {
        return name.contains(term);
    }
    name.match_indices(term).any(|(start, _)| {
        let before = name[..start].chars().next_back();
        let after = name[start + term.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// Check if format hint is provided.
///
/// Any accessible description counts: `aria-describedby` is exactly how
/// instructions are attached to a field, and whether its text spells out the
/// format cannot be judged by keyword matching (#643).
fn has_format_hint(node: &AXNode) -> bool {
    if has_instructions_or_hint(node) {
        return true;
    }

    let format_patterns = [
        "format:",
        "example:",
        "e.g.",
        "(",
        "mm/dd",
        "yyyy",
        // German
        "z.b.",
        "bsp.",
        "beispiel:",
        // French
        "ex.",
        "exemple:",
        "par ex.",
        // Spanish
        "ej.",
        "ejemplo:",
        "p.ej.",
        // Italian
        "es.",
        "esempio:",
        // Portuguese
        "ex.:",
        "exemplo:",
        // Dutch
        "bijv.",
        "voorbeeld:",
        // Swedish/Norwegian
        "t.ex.",
        // Polish
        "np.",
        "przykład:",
    ];

    if let Some(name) = &node.name {
        let name_lower = name.to_lowercase();
        if format_patterns.iter().any(|p| name_lower.contains(p)) {
            return true;
        }
    }

    has_placeholder(node)
}

/// A node is a real form group only when it has fieldset semantics OR
/// contains at least one descendant that is a form control. This filters
/// out <details>, <address>, <hgroup> and other generic groupings that
/// Chrome exposes as role="group" but are not form-related.
fn is_form_group(node: &AXNode, tree: &AXTree) -> bool {
    // Explicit fieldset: htmlTag check
    if let Some(tag) = node.get_property_str("htmlTag") {
        let tag_up = tag.to_uppercase();
        if tag_up == "FIELDSET" {
            return true;
        }
        // Common non-form groups exposed as role="group" by Chrome
        if matches!(
            tag_up.as_str(),
            "DETAILS" | "ADDRESS" | "HGROUP" | "FIGURE" | "ARTICLE" | "SECTION" | "ASIDE"
        ) {
            return false;
        }
    }

    // Fallback: group counts as a form group only if it contains a form control
    has_form_control_descendant(node, tree, 0)
}

fn has_form_control_descendant(node: &AXNode, tree: &AXTree, depth: usize) -> bool {
    if depth > 8 {
        return false;
    }
    for child_id in &node.child_ids {
        if let Some(child) = tree.nodes.get(child_id) {
            let role = child.role.as_deref().unwrap_or("").to_lowercase();
            if is_form_input(&role) {
                return true;
            }
            if has_form_control_descendant(child, tree, depth + 1) {
                return true;
            }
        }
    }
    false
}

/// Check if group has a label
fn has_group_label(node: &AXNode) -> bool {
    if let Some(name) = &node.name {
        if !name.trim().is_empty() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXProperty, AXValue};

    fn create_input(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    fn create_input_with_required(
        id: &str,
        role: &str,
        name: Option<&str>,
        required: bool,
    ) -> AXNode {
        let mut node = create_input(id, role, name);
        if required {
            node.properties.push(AXProperty {
                name: "required".to_string(),
                value: AXValue::Bool(true),
            });
        }
        node
    }

    #[test]
    fn test_instructions_rule_metadata() {
        assert_eq!(INSTRUCTIONS_RULE.id, "3.3.2");
        assert_eq!(INSTRUCTIONS_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_is_form_input() {
        assert!(is_form_input("textbox"));
        assert!(is_form_input("checkbox"));
        assert!(is_form_input("combobox"));
        assert!(!is_form_input("button"));
        assert!(!is_form_input("link"));
    }

    #[test]
    fn test_input_without_label() {
        let tree = AXTree::from_nodes(vec![create_input("1", "textbox", None)]);
        let results = check_instructions(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no accessible label")));
    }

    #[test]
    fn test_input_with_label() {
        let tree = AXTree::from_nodes(vec![create_input("1", "textbox", Some("Email address"))]);
        let results = check_instructions(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("no accessible label")));
    }

    #[test]
    fn test_placeholder_only_label_flagged() {
        // Regression check for #QA-030: name_source == Placeholder must be
        // detected even though the AX tree's `name` is non-empty (Chrome
        // computes it from the placeholder) and there is no real AX
        // "placeholder" property to read.
        let mut node = create_input("1", "textbox", Some("Email address"));
        node.name_source = Some(NameSource::Placeholder);
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_instructions(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("Placeholder used as only label")));
    }

    #[test]
    fn test_labelled_input_not_flagged_as_placeholder_only() {
        let mut node = create_input("1", "textbox", Some("Email address"));
        node.name_source = Some(NameSource::RelatedElement);
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_instructions(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("Placeholder used as only label")));
    }

    #[test]
    fn test_required_without_indication() {
        let tree = AXTree::from_nodes(vec![create_input_with_required(
            "1",
            "textbox",
            Some("Name"),
            true,
        )]);
        let results = check_instructions(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("Required field")));
    }

    #[test]
    fn test_required_with_indication() {
        let tree = AXTree::from_nodes(vec![create_input_with_required(
            "1",
            "textbox",
            Some("Name (required)"),
            true,
        )]);
        let results = check_instructions(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("Required field not clearly indicated")));
    }

    fn format_findings(results: &WcagResults) -> Vec<&Violation> {
        results
            .violations
            .iter()
            .chain(results.warnings.iter())
            .filter(|v| v.message.contains("may require format instructions"))
            .collect()
    }

    /// #643: a field without any instruction whose label asks for a date is
    /// still reported — as needs-review, not as a confirmed violation.
    #[test]
    fn test_format_sensitive_field_without_hint_is_warning() {
        let tree = AXTree::from_nodes(vec![create_input("1", "textbox", Some("Date"))]);
        let results = check_instructions(&tree);
        assert_eq!(format_findings(&results).len(), 1);
        assert!(results.violations.is_empty());
        assert_eq!(results.warnings.len(), 1);
    }

    /// #643: `aria-describedby` instructions count, whatever their wording
    /// ("Day of travel" on barrierlab.eu's accessible-form pattern).
    #[test]
    fn test_described_field_needs_no_format_hint() {
        let mut node = create_input("1", "textbox", Some("Date"));
        node.description = Some("Day of travel".to_string());
        let tree = AXTree::from_nodes(vec![node]);
        assert!(format_findings(&check_instructions(&tree)).is_empty());
    }

    /// #643: "Was ist passiert?" contains "pass" but is no passport field.
    #[test]
    fn test_short_terms_match_whole_words_only() {
        let needs =
            |name: &str| needs_format_instructions(&create_input("1", "textbox", Some(name)));
        assert!(!needs("Was ist passiert?"));
        assert!(!needs("Hotel"));
        assert!(!needs("Last update"));
        assert!(needs("Date of birth"));
        assert!(needs("Tel."));
        assert!(needs("Reisepass"));
        assert!(needs("Geburtsdatum"));
    }

    /// #656: the day/month/year spinbuttons Chrome renders inside a native
    /// date input are the browser's own fields, with the browser's format.
    #[test]
    fn test_native_date_time_subfields_are_not_checked() {
        for native_role in ["Date", "DateTime", "InputTime"] {
            let input = create_input("d", native_role, Some("Order date"));
            let mut wrapper = create_input("w", "generic", None);
            wrapper.parent_id = Some("d".into());
            let mut day = create_input("s", "spinbutton", Some("Day"));
            day.parent_id = Some("w".into());
            let tree = AXTree::from_nodes(vec![input, wrapper, day]);
            let results = check_instructions(&tree);
            assert!(format_findings(&results).is_empty(), "{native_role}");
            assert!(results.violations.is_empty(), "{native_role}");
        }
    }

    /// #656: a spinbutton, native or custom, holds a number the widget
    /// constrains — no format hint just for the role; the label still counts.
    #[test]
    fn test_spinbutton_needs_format_hint_only_by_label() {
        let quantity = AXTree::from_nodes(vec![create_input("s", "spinbutton", Some("Quantity"))]);
        assert!(format_findings(&check_instructions(&quantity)).is_empty());
        let postal = AXTree::from_nodes(vec![create_input("s", "spinbutton", Some("Postal code"))]);
        assert_eq!(format_findings(&check_instructions(&postal)).len(), 1);
    }

    fn group_with_html_tag(id: &str, tag: &str, name: Option<&str>) -> AXNode {
        let mut n = create_input(id, "group", name);
        n.properties.push(AXProperty {
            name: "htmlTag".into(),
            value: AXValue::String(tag.into()),
        });
        n
    }

    #[test]
    fn test_details_not_flagged_as_form_group() {
        let tree = AXTree::from_nodes(vec![group_with_html_tag("1", "DETAILS", None)]);
        let results = check_instructions(&tree);
        assert!(
            !results
                .violations
                .iter()
                .any(|v| v.message.contains("Form group has no legend")),
            "<details> must not be flagged as a form group missing a legend"
        );
    }

    #[test]
    fn test_address_not_flagged_as_form_group() {
        let tree = AXTree::from_nodes(vec![group_with_html_tag("1", "ADDRESS", None)]);
        let results = check_instructions(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("Form group has no legend")));
    }

    #[test]
    fn test_fieldset_without_legend_still_flagged() {
        // A FIELDSET htmlTag with no name must still trigger the form-group rule.
        let tree = AXTree::from_nodes(vec![group_with_html_tag("1", "FIELDSET", None)]);
        let results = check_instructions(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("Form group has no legend")));
    }

    #[test]
    fn test_generic_group_with_form_control_flagged() {
        // A generic "group" that contains a form control must be flagged when
        // it has no accessible name.
        let mut group = create_input("g", "group", None);
        group.child_ids.push("input".into());
        let input = create_input("input", "textbox", Some("Email"));
        let tree = AXTree::from_nodes(vec![group, input]);
        let results = check_instructions(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("Form group has no legend")));
    }
}
