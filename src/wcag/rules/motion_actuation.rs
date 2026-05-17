//! WCAG 2.5.4 Motion Actuation (Level AAA)
//!
//! Functionality that can be operated by device motion or user motion can also
//! be operated by user interface components, and responding to the motion can
//! be disabled to prevent accidental actuation.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const MOTION_ACTUATION_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.4",
    name: "Motion Actuation",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Functionality triggered by device motion can also be activated by UI components",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/motion-actuation.html",
    axe_id: "motion-actuation",
    tags: &["wcag2aaa", "wcag254", "cat.sensory-and-visual-cues"],
};

const MOTION_ACTUATION_JS: &str = r#"
(function() {
  var motionPatterns = ['deviceorientation', 'devicemotion', 'shake'];
  var found = [];

  var scripts = document.querySelectorAll('script:not([src])');
  for (var i = 0; i < scripts.length; i++) {
    var content = scripts[i].textContent || '';
    for (var j = 0; j < motionPatterns.length; j++) {
      if (content.indexOf(motionPatterns[j]) !== -1) {
        found.push(motionPatterns[j]);
      }
    }
  }

  // Check inline handlers
  var allElements = document.querySelectorAll('[ondeviceorientation], [ondevicemotion]');
  if (allElements.length > 0) {
    found.push('inline-motion-handler');
  }

  return { found: found };
})()
"#;

pub async fn check_motion_actuation_with_page(page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
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
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(MOTION_ACTUATION_JS).await {
        Ok(r) => r,
        Err(_) => return vec![not_testable],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![not_testable],
    };

    let found: Vec<String> = val
        .get("found")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut findings = vec![not_testable];

    if !found.is_empty() {
        let patterns: Vec<String> = found
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        findings.push(
            Violation::new(
                MOTION_ACTUATION_RULE.id,
                MOTION_ACTUATION_RULE.name,
                MOTION_ACTUATION_RULE.level,
                Severity::Medium,
                format!(
                    "Device motion event listeners detected ({}). Verify that UI alternatives \
                     exist and motion actuation can be disabled.",
                    patterns.join(", ")
                ),
                "page",
            )
            .with_fix(
                "Provide button-based alternatives for all device-motion interactions and \
                 include a setting to disable motion-based control.",
            )
            .with_rule_id(MOTION_ACTUATION_RULE.axe_id)
            .with_help_url(MOTION_ACTUATION_RULE.help_url),
        );
    }

    findings
}
