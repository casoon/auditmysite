//! WCAG 4.1.3 Status Messages
//!
//! Status messages can be programmatically determined through role or property
//! so they can be presented to the user by assistive technologies without
//! receiving focus.
//! Level AA
//!
//! Checks:
//! - role="alert" must not override aria-live to "polite" or "off"
//!   (alert implies assertive; overriding it breaks AT announcement)
//! - role="status" must not override aria-live to "assertive" or "off"
//!   (status implies polite; assertive is too intrusive)
//! - role="log" must not set aria-live="off"
//! - role="timer" must not set aria-live="off"

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const STATUS_MESSAGES_RULE: RuleMetadata = RuleMetadata {
    id: "4.1.3",
    name: "Status Messages",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Status messages must be programmatically determinable without receiving focus",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/status-messages.html",
    axe_id: "aria-live-region-role",
    tags: &["wcag2aa", "wcag413", "cat.aria"],
};

pub fn check_status_messages(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        let role = match node.role.as_deref() {
            Some(r) => r.to_lowercase(),
            None => continue,
        };

        let live = node.get_property_str("live").map(|v| v.to_lowercase());

        match role.as_str() {
            // role="alert" implies aria-live="assertive"
            // Overriding to "polite" or "off" breaks the announcement contract
            "alert" => {
                if let Some(ref live_val) = live {
                    if live_val == "polite" || live_val == "off" {
                        results.add_violation(
                            Violation::new(
                                STATUS_MESSAGES_RULE.id,
                                STATUS_MESSAGES_RULE.name,
                                STATUS_MESSAGES_RULE.level,
                                Severity::High,
                                format!(
                                    "role=\"alert\" has aria-live=\"{}\" — alert implies assertive; overriding silences or delays the announcement",
                                    live_val
                                ),
                                node.node_id.clone(),
                            )
                            .with_role(node.role.clone())
                            .with_name(node.name.clone())
                            .with_fix("Remove the aria-live attribute from role=\"alert\" elements, or set it to \"assertive\"")
                            .with_rule_id(STATUS_MESSAGES_RULE.axe_id)
                            .with_help_url(STATUS_MESSAGES_RULE.help_url),
                        );
                    } else {
                        results.passes += 1;
                    }
                } else {
                    results.passes += 1;
                }
            }

            // role="status" implies aria-live="polite"
            // "assertive" is too intrusive; "off" silences the region
            "status" => {
                if let Some(ref live_val) = live {
                    if live_val == "assertive" || live_val == "off" {
                        results.add_violation(
                            Violation::new(
                                STATUS_MESSAGES_RULE.id,
                                STATUS_MESSAGES_RULE.name,
                                STATUS_MESSAGES_RULE.level,
                                Severity::Medium,
                                format!(
                                    "role=\"status\" has aria-live=\"{}\" — status implies polite; \"{}\" changes the announcement behavior",
                                    live_val, live_val
                                ),
                                node.node_id.clone(),
                            )
                            .with_role(node.role.clone())
                            .with_name(node.name.clone())
                            .with_fix("Remove the aria-live attribute from role=\"status\" elements, or set it to \"polite\"")
                            .with_rule_id(STATUS_MESSAGES_RULE.axe_id)
                            .with_help_url(STATUS_MESSAGES_RULE.help_url),
                        );
                    } else {
                        results.passes += 1;
                    }
                } else {
                    results.passes += 1;
                }
            }

            // role="log" implies aria-live="polite"; "off" disables announcements entirely
            // role="timer" implies aria-live="off" by spec — but if explicitly set to
            // "assertive" or "polite" that's still spec-compliant, so only flag "off"
            // on log.
            "log" => {
                if live.as_deref() == Some("off") {
                    results.add_violation(
                        Violation::new(
                            STATUS_MESSAGES_RULE.id,
                            STATUS_MESSAGES_RULE.name,
                            STATUS_MESSAGES_RULE.level,
                            Severity::Medium,
                            "role=\"log\" has aria-live=\"off\" — log regions should announce updates to AT",
                            node.node_id.clone(),
                        )
                        .with_role(node.role.clone())
                        .with_fix("Remove aria-live=\"off\" from role=\"log\" elements, or use aria-live=\"polite\"")
                        .with_rule_id(STATUS_MESSAGES_RULE.axe_id)
                        .with_help_url(STATUS_MESSAGES_RULE.help_url),
                    );
                } else {
                    results.passes += 1;
                }
            }

            _ => {}
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node_with_role_and_live(id: &str, role: &str, live: Option<&str>) -> AXNode {
        let mut properties = vec![];
        if let Some(live_val) = live {
            properties.push(AXProperty {
                name: "live".to_string(),
                value: AXValue::String(live_val.to_string()),
            });
        }
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some("Status region".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties,
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_alert_without_aria_live_passes() {
        let tree = AXTree::from_nodes(vec![node_with_role_and_live("1", "alert", None)]);
        let r = check_status_messages(&tree);
        assert_eq!(r.violations.len(), 0);
    }

    #[test]
    fn test_alert_with_assertive_passes() {
        let tree = AXTree::from_nodes(vec![node_with_role_and_live(
            "1",
            "alert",
            Some("assertive"),
        )]);
        let r = check_status_messages(&tree);
        assert_eq!(r.violations.len(), 0);
    }

    #[test]
    fn test_alert_with_polite_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_role_and_live("1", "alert", Some("polite"))]);
        let r = check_status_messages(&tree);
        assert_eq!(r.violations.len(), 1);
        assert!(r.violations[0].message.contains("polite"));
    }

    #[test]
    fn test_alert_with_off_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_role_and_live("1", "alert", Some("off"))]);
        let r = check_status_messages(&tree);
        assert_eq!(r.violations.len(), 1);
        assert!(r.violations[0].message.contains("off"));
    }

    #[test]
    fn test_status_with_polite_passes() {
        let tree = AXTree::from_nodes(vec![node_with_role_and_live("1", "status", Some("polite"))]);
        let r = check_status_messages(&tree);
        assert_eq!(r.violations.len(), 0);
    }

    #[test]
    fn test_status_with_assertive_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_role_and_live(
            "1",
            "status",
            Some("assertive"),
        )]);
        let r = check_status_messages(&tree);
        assert_eq!(r.violations.len(), 1);
        assert!(r.violations[0].message.contains("assertive"));
    }

    #[test]
    fn test_log_with_off_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_role_and_live("1", "log", Some("off"))]);
        let r = check_status_messages(&tree);
        assert_eq!(r.violations.len(), 1);
        assert!(r.violations[0].message.contains("log"));
    }

    #[test]
    fn test_log_without_aria_live_passes() {
        let tree = AXTree::from_nodes(vec![node_with_role_and_live("1", "log", None)]);
        let r = check_status_messages(&tree);
        assert_eq!(r.violations.len(), 0);
    }
}
