//! WCAG 3.3.5 Help (Level AAA)
//!
//! Context-sensitive help is available. For form inputs, this typically means
//! instructions, labels, or additional guidance is accessible to the user.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation, WcagResults};

pub const HELP_RULE: RuleMetadata = RuleMetadata {
    id: "3.3.5",
    name: "Help",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Context-sensitive help is available for form inputs",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/help.html",
    axe_id: "help",
    tags: &["wcag2aaa", "wcag335", "cat.forms"],
};

pub fn check_help(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    let mut missing_help = 0usize;

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        let role = node.role.as_deref().unwrap_or("");
        if !matches!(role, "textbox" | "searchbox") {
            continue;
        }

        results.nodes_checked += 1;

        let has_describedby = node
            .get_property_str("describedby")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);

        let has_description = node
            .description
            .as_ref()
            .map(|d| !d.trim().is_empty())
            .unwrap_or(false);

        if !has_describedby && !has_description {
            missing_help += 1;
            if missing_help <= 5 {
                results.add_violation(
                    Violation::new(
                        HELP_RULE.id,
                        HELP_RULE.name,
                        HELP_RULE.level,
                        Severity::Low,
                        format!(
                            "Input '{}' (role={}) has no aria-describedby or description. \
                             Consider providing context-sensitive help text.",
                            node.name.as_deref().unwrap_or(&node.node_id),
                            role
                        ),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_name(node.name.clone())
                    .with_fix(
                        "Add aria-describedby pointing to a help text element, or include \
                         inline instructions near the input.",
                    )
                    .with_rule_id(HELP_RULE.axe_id)
                    .with_help_url(HELP_RULE.help_url),
                );
            }
        } else {
            results.passes += 1;
        }
    }

    // Always emit a not-testable notice since help can be provided in ways
    // that are invisible to the AXTree (tooltips, inline text, external docs).
    results.add_violation(
        Violation::new(
            HELP_RULE.id,
            HELP_RULE.name,
            HELP_RULE.level,
            Severity::Low,
            "WCAG 3.3.5 requires manual review. Context-sensitive help may be provided via \
             tooltips, inline instructions, or support links that are not visible in the \
             accessibility tree.",
            "page",
        )
        .with_fix(
            "Provide context-sensitive help for all form controls: instructions, examples, \
             tooltips, or links to help documentation.",
        )
        .with_rule_id(HELP_RULE.axe_id)
        .with_help_url(HELP_RULE.help_url)
        .with_kind(FindingKind::NotTestable),
    );

    results
}
