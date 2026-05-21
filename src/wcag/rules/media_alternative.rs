//! WCAG 1.2.8 Media Alternative (Prerecorded) (Level AAA)
//!
//! An alternative for time-based media is provided for all prerecorded
//! synchronized media and for all prerecorded video-only media.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation, WcagResults};

pub const MEDIA_ALTERNATIVE_RULE: RuleMetadata = RuleMetadata {
    id: "1.2.8",
    name: "Media Alternative (Prerecorded)",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Prerecorded media has a text alternative or description track",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/media-alternative-prerecorded.html",
    axe_id: "media-alt",
    tags: &["wcag2aaa", "wcag128", "cat.time-and-media"],
};

pub fn check_media_alternative(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    let has_video = tree.iter().any(|n| {
        !n.ignored
            && matches!(
                n.role.as_deref(),
                Some("video") | Some("VideoElement") | Some("EmbeddedObject")
            )
    });

    if has_video {
        results.add_violation(
            Violation::new(
                MEDIA_ALTERNATIVE_RULE.id,
                MEDIA_ALTERNATIVE_RULE.name,
                MEDIA_ALTERNATIVE_RULE.level,
                Severity::Medium,
                "Video content detected. Verify that a full text alternative or audio description \
                 track exists for all prerecorded video content.",
                "video",
            )
            .with_fix(
                "Provide a text transcript or synchronized audio description for prerecorded \
                 video. Add a <track kind=\"descriptions\"> element or link to a text alternative.",
            )
            .with_rule_id(MEDIA_ALTERNATIVE_RULE.axe_id)
            .with_help_url(MEDIA_ALTERNATIVE_RULE.help_url)
            .with_kind(FindingKind::NotTestable),
        );
    } else {
        // No video found — emit a general notice anyway since the AXTree
        // may not capture all embedded media (iframes, objects).
        results.add_violation(
            Violation::new(
                MEDIA_ALTERNATIVE_RULE.id,
                MEDIA_ALTERNATIVE_RULE.name,
                MEDIA_ALTERNATIVE_RULE.level,
                Severity::Low,
                "WCAG 1.2.8 requires manual verification that all prerecorded video and \
                 audio content has a full text alternative.",
                "page",
            )
            .with_fix(
                "Ensure every prerecorded multimedia element has a complete text transcript \
                 and, where applicable, an audio description.",
            )
            .with_rule_id(MEDIA_ALTERNATIVE_RULE.axe_id)
            .with_help_url(MEDIA_ALTERNATIVE_RULE.help_url)
            .with_kind(FindingKind::NotTestable),
        );
    }

    results
}
