//! WCAG 2.2.4 Interruptions (Level AAA)
//!
//! Interruptions can be postponed or suppressed by the user, except interruptions
//! involving an emergency.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const NO_INTERRUPTIONS_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.4",
    name: "Interruptions",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Interruptions can be postponed or suppressed by the user",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/interruptions.html",
    axe_id: "no-interruptions",
    tags: &["wcag2aaa", "wcag224", "cat.time-and-media"],
};

const INTERRUPTIONS_JS: &str = r#"
(function() {
  var hasAlertDialog = !!document.querySelector('[role="alertdialog"]');
  var hasAutoPopup = !!document.querySelector('[aria-live="assertive"]:not([role="alert"])');
  return { hasAlertDialog: hasAlertDialog, hasAutoPopup: hasAutoPopup };
})()
"#;

pub async fn check_no_interruptions_with_page(page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
        NO_INTERRUPTIONS_RULE.id,
        NO_INTERRUPTIONS_RULE.name,
        NO_INTERRUPTIONS_RULE.level,
        Severity::Low,
        "WCAG 2.2.4 (Interruptions) requires manual testing to verify that users can \
         postpone or suppress non-emergency interruptions such as alerts, popups, and \
         live region updates.",
        "page",
    )
    .with_fix(
        "Provide users with settings to control or suppress non-essential notifications \
         and interruptions.",
    )
    .with_rule_id(NO_INTERRUPTIONS_RULE.axe_id)
    .with_help_url(NO_INTERRUPTIONS_RULE.help_url)
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(INTERRUPTIONS_JS).await {
        Ok(r) => r,
        Err(_) => return vec![not_testable],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![not_testable],
    };

    let has_alert_dialog = val
        .get("hasAlertDialog")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut findings = vec![not_testable];

    if has_alert_dialog {
        findings.push(
            Violation::new(
                NO_INTERRUPTIONS_RULE.id,
                NO_INTERRUPTIONS_RULE.name,
                NO_INTERRUPTIONS_RULE.level,
                Severity::Low,
                "Page contains role=\"alertdialog\" elements. Verify that users can \
                 dismiss or suppress these interruptions unless they represent an emergency.",
                "[role=\"alertdialog\"]",
            )
            .with_selector("[role=\"alertdialog\"]")
            .with_fix(
                "Allow users to dismiss or delay alert dialogs. Provide a preference to \
                 suppress non-emergency notifications.",
            )
            .with_rule_id(NO_INTERRUPTIONS_RULE.axe_id)
            .with_help_url(NO_INTERRUPTIONS_RULE.help_url)
            .with_kind(FindingKind::Warning),
        );
    }

    findings
}
