//! WCAG 2.4.4 Link Purpose (In Context)
//!
//! Ensures the purpose of each link can be determined from the link text alone
//! or from the link text together with its context.
//! Level A

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.4.4
pub const LINK_PURPOSE_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.4",
    name: "Link Purpose (In Context)",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The purpose of each link can be determined from the link text or context",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-in-context.html",
    axe_id: "link-name",
    tags: &["wcag2a", "wcag244", "cat.links"],
};

/// Check for link purpose issues
pub fn check_link_purpose(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored || node.role.as_deref() != Some("link") {
            continue;
        }

        results.nodes_checked += 1;
        let link_text = node.name.as_deref().unwrap_or("").trim();

        // Empty links (no accessible text at all) are already covered by
        // a11y.name_role.missing (WCAG 4.1.2). Reporting them here too would
        // cause the same element to appear in two separate findings and inflate
        // severity_counts. Skip empty links in this rule.
        if link_text.is_empty() {
            continue;
        }

        // Check for generic/ambiguous link text
        if is_generic_link_text(link_text) {
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                LINK_PURPOSE_RULE.severity,
                format!("Link has generic text: '{}'", link_text),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Use descriptive link text that explains where the link goes")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation);
        } else if looks_like_url(link_text) && !has_link_context(node, tree) {
            // Check for URL-only link text
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                Severity::Low,
                "Link text appears to be a raw URL",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Replace URL with descriptive text")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation.as_warning());
        } else if link_text.len() == 1
            && !link_text
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            // Check for single character links
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                LINK_PURPOSE_RULE.severity,
                format!("Link has single character text: '{}'", link_text),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Expand single character links to be more descriptive")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }

        // Check for links that open in new window without warning
        if opens_new_window(node) && !indicates_new_window(link_text) {
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                Severity::Low,
                "Link opens in new window without indication",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Add '(opens in new window)' to link text")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation);
        }
    }

    results
}

/// Check if the link has a descriptive context
fn has_link_context(node: &AXNode, tree: &AXTree) -> bool {
    if let Some(desc) = &node.description {
        if !desc.trim().is_empty() {
            return true;
        }
    }

    if let Some(parent_id) = &node.parent_id {
        if let Some(parent) = tree.get_node(parent_id) {
            for child_id in &parent.child_ids {
                if child_id == &node.node_id {
                    continue;
                }
                if let Some(sibling) = tree.get_node(child_id) {
                    if !sibling.ignored {
                        let text = sibling.name.as_deref().unwrap_or("").trim();
                        if !text.is_empty()
                            && matches!(
                                sibling.role.as_deref(),
                                Some("StaticText" | "text" | "paragraph")
                            )
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Check if link text is generic/ambiguous
fn is_generic_link_text(text: &str) -> bool {
    let generic_phrases = [
        // English
        "click here",
        "click",
        "here",
        "read more",
        "more",
        "learn more",
        "info",
        "information",
        "details",
        "link",
        "this link",
        "go",
        "continue",
        "download",
        "view",
        "see more",
        "see all",
        "read",
        "start",
        "begin",
        "submit",
        "next",
        "previous",
        // German
        "weiterlesen",
        "mehr erfahren",
        "mehr",
        "hier klicken",
        "klicken sie hier",
        "hier",
        "weiter",
        "alle anzeigen",
        "ansehen",
        "jetzt lesen",
        "öffnen",
        // French
        "cliquez ici",
        "ici",
        "en savoir plus",
        "lire la suite",
        "lire plus",
        "plus",
        "voir plus",
        "voir tout",
        "continuer",
        "télécharger",
        "suivant",
        "précédent",
        "commencer",
        // Spanish
        "haz clic aquí",
        "aquí",
        "leer más",
        "más",
        "saber más",
        "ver más",
        "ver todo",
        "continuar",
        "descargar",
        "siguiente",
        "anterior",
        "empezar",
        // Italian
        "clicca qui",
        "qui",
        "leggi di più",
        "scopri di più",
        "più",
        "vedi di più",
        "vedi tutto",
        "continua",
        "scarica",
        "successivo",
        "precedente",
        "inizia",
        // Portuguese
        "clique aqui",
        "aqui",
        "leia mais",
        "saiba mais",
        "mais",
        "ver mais",
        "ver tudo",
        "baixar",
        "próximo",
        "começar",
        // Dutch
        "klik hier",
        "meer lezen",
        "lees meer",
        "meer",
        "bekijk meer",
        "volgende",
        "vorige",
        "downloaden",
        "beginnen",
        // Swedish
        "klicka här",
        "här",
        "läs mer",
        "mer",
        "nästa",
        "föregående",
        "ladda ner",
        "börja",
        // Norwegian
        "klikk her",
        "her",
        "les mer",
        "neste",
        "forrige",
        "last ned",
        // Danish
        "klik her",
        "læs mere",
        "mere",
        "næste",
        // Finnish
        "klikkaa tästä",
        "lue lisää",
        "lisää",
        "seuraava",
        "edellinen",
        "lataa",
        // Polish
        "kliknij tutaj",
        "tutaj",
        "czytaj więcej",
        "więcej",
        "następny",
        "poprzedni",
        "pobierz",
        // Turkish
        "buraya tıklayın",
        "devamını oku",
        "daha fazla",
        "sonraki",
        "önceki",
        "indir",
        // Czech / Slovak
        "klikněte zde",
        "zde",
        "číst více",
        "více",
        "další",
        "předchozí",
        "stáhnout",
        // Romanian
        "citește mai mult",
        "mai mult",
        "următor",
        "descărcați",
        // Hungarian
        "tovább olvas",
        "több",
        "következő",
        "előző",
        "letöltés",
        // Symbols
        "...",
        ">",
        ">>",
        "→",
    ];

    let text_lower = text.to_lowercase();
    generic_phrases.iter().any(|&phrase| text_lower == phrase)
}

/// Check if text looks like a URL
fn looks_like_url(text: &str) -> bool {
    text.starts_with("http://")
        || text.starts_with("https://")
        || text.starts_with("www.")
        || (text.contains(".com") && !text.contains(' '))
        || (text.contains(".org") && !text.contains(' '))
        || (text.contains(".net") && !text.contains(' '))
}

/// Check if link opens in new window
fn opens_new_window(node: &AXNode) -> bool {
    node.properties
        .iter()
        .any(|p| p.name.to_lowercase() == "haspopup" && p.value.as_bool().unwrap_or(false))
}

/// Check if link text indicates it opens in new window
fn indicates_new_window(text: &str) -> bool {
    let indicators = [
        "new window",
        "new tab",
        "opens in",
        "(external)",
        "external link",
        "[external]",
    ];
    let text_lower = text.to_lowercase();
    indicators.iter().any(|&ind| text_lower.contains(ind))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_link(id: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("link".to_string()),
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

    #[test]
    fn test_link_purpose_rule_metadata() {
        assert_eq!(LINK_PURPOSE_RULE.id, "2.4.4");
        assert_eq!(LINK_PURPOSE_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_generic_link_text() {
        assert!(is_generic_link_text("click here"));
        assert!(is_generic_link_text("Read more"));
        assert!(is_generic_link_text("HERE"));
        assert!(is_generic_link_text("Weiterlesen"));
        assert!(is_generic_link_text("Mehr erfahren"));
        assert!(is_generic_link_text("weiter"));
        assert!(!is_generic_link_text("View product specifications"));
        assert!(!is_generic_link_text("Artikel über HBOT lesen"));
    }

    #[test]
    fn test_looks_like_url() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("www.example.com"));
        assert!(!looks_like_url("Visit our website"));
    }

    #[test]
    fn test_empty_link() {
        // Empty links are handled by a11y.name_role.missing (4.1.2).
        // This rule (2.4.4) skips them to avoid double-reporting.
        let tree = AXTree::from_nodes(vec![create_link("1", None)]);
        let results = check_link_purpose(&tree);
        assert!(
            results.violations.is_empty(),
            "Empty links must not be reported by link_purpose — covered by name_role.missing"
        );
    }

    #[test]
    fn test_generic_link() {
        let tree = AXTree::from_nodes(vec![create_link("1", Some("click here"))]);
        let results = check_link_purpose(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("generic text")));
    }

    #[test]
    fn test_good_link_text() {
        let tree = AXTree::from_nodes(vec![create_link(
            "1",
            Some("View our accessibility statement"),
        )]);
        let results = check_link_purpose(&tree);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_url_as_link_text() {
        let tree = AXTree::from_nodes(vec![create_link("1", Some("https://example.com/page"))]);
        let results = check_link_purpose(&tree);
        assert!(results
            .warnings
            .iter()
            .any(|v| v.message.contains("raw URL")));
    }

    #[test]
    fn test_url_as_link_text_with_context() {
        let parent = AXNode {
            node_id: "parent".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("paragraph".to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec!["text1".to_string(), "link1".to_string()],
            parent_id: None,
            backend_dom_node_id: None,
        };
        let text_node = AXNode {
            node_id: "text1".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("StaticText".to_string()),
            name: Some("Go to ".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("parent".to_string()),
            backend_dom_node_id: None,
        };
        let link_node = AXNode {
            node_id: "link1".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("link".to_string()),
            name: Some("https://example.com".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("parent".to_string()),
            backend_dom_node_id: None,
        };
        let tree = AXTree::from_nodes(vec![parent, text_node, link_node]);
        let results = check_link_purpose(&tree);
        assert!(results.warnings.is_empty());
        assert!(results.violations.is_empty());
    }
}
