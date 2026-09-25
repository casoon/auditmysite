//! WCAG 2.5.4 Motion Actuation (Level A)
//!
//! Functionality that can be operated by device motion or user motion can also
//! be operated by user interface components, and responding to the motion can
//! be disabled to prevent accidental actuation.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub const MOTION_ACTUATION_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.4",
    name: "Motion Actuation",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Functionality triggered by device motion can also be activated by UI components",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/motion-actuation.html",
    axe_id: "motion-actuation",
    tags: &["wcag2a", "wcag254", "cat.sensory-and-visual-cues"],
};

pub async fn check_motion_actuation_with_page(_page: &Page) -> Vec<Violation> {
    vec![Violation::new(
        MOTION_ACTUATION_RULE.id,
        MOTION_ACTUATION_RULE.name,
        MOTION_ACTUATION_RULE.level,
        Severity::Low,
        "WCAG 2.5.4 requires manual testing to verify that all motion-based interactions \
         have UI alternatives and can be disabled.",
        "page",
    )
    .with_fix(
        "For any device motion interaction, provide equivalent button controls and allow \
         users to disable motion actuation.",
    )
    .with_rule_id(MOTION_ACTUATION_RULE.axe_id)
    .with_help_url(MOTION_ACTUATION_RULE.help_url)
    .with_kind(Outcome::Untested)]
}
