//! WCAG 1.4.7 Low or No Background Audio (Level AAA)
//!
//! For prerecorded audio-only content that contains speech, there is no
//! background sound, background sound can be turned off, or background sound
//! is at least 20 dB lower than the foreground speech content.
//!
//! This automated check detects audio elements with autoplay that are not muted,
//! which is a reliable structural signal for uncontrolled background audio.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const BACKGROUND_AUDIO_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.7",
    name: "Low or No Background Audio",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Background audio must be avoidable or at least 20 dB lower than speech",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/low-or-no-background-audio.html",
    axe_id: "background-audio",
    tags: &["wcag2aaa", "wcag147", "cat.time-and-media"],
};

const BACKGROUND_AUDIO_JS: &str = r#"
(function() {
  var elements = document.querySelectorAll('audio[autoplay]:not([muted])');
  var results = [];
  for (var i = 0; i < elements.length; i++) {
    var el = elements[i];
    var src = el.getAttribute('src') || (el.querySelector('source') ? el.querySelector('source').getAttribute('src') : '') || 'audio';
    results.push({ src: src });
  }
  return { elements: results };
})()
"#;

pub async fn check_background_audio_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(BACKGROUND_AUDIO_JS).await {
        Ok(r) => r,
        Err(_) => {
            return vec![crate::wcag::technical_rule_failure(
                &BACKGROUND_AUDIO_RULE,
                "page_evaluation_failed",
            )]
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &BACKGROUND_AUDIO_RULE,
                "missing_evaluation_value",
            )]
        }
    };

    let elements = match val.get("elements").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &BACKGROUND_AUDIO_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    elements
        .iter()
        .map(|item| {
            let src = item.get("src").and_then(|v| v.as_str()).unwrap_or("audio");

            Violation::new(
                BACKGROUND_AUDIO_RULE.id,
                BACKGROUND_AUDIO_RULE.name,
                BACKGROUND_AUDIO_RULE.level,
                Severity::Medium,
                format!(
                    "Audio element '{}' autoplays without being muted. Background audio that \
                     plays automatically can interfere with screen readers and disturb users.",
                    src
                ),
                "audio[autoplay]",
            )
            .with_selector("audio[autoplay]:not([muted])")
            .with_fix(
                "Remove the autoplay attribute, add the muted attribute, or provide a \
                 visible control to immediately pause/stop the audio.",
            )
            .with_rule_id(BACKGROUND_AUDIO_RULE.axe_id)
            .with_help_url(BACKGROUND_AUDIO_RULE.help_url)
        })
        .collect()
}
