//! Text export of the mobile issue texts (#936): every text of
//! `mobile_issue_text` and `mobile_context_label` in both languages across an
//! enumerated input space, as records in `reports/mobile-issue-texts.json`
//! (gitignored; the path can be set with `MOBILE_ISSUE_TEXT_EXPORT`). Run it
//! before and after a change of how the texts are stored and compare the two
//! files byte for byte.

use super::{mobile_context_label, mobile_issue_text, MobileIssueKind, MobileIssueValues};
use serde_json::json;

pub(super) const MOBILE_KINDS: [MobileIssueKind; 6] = [
    MobileIssueKind::MissingViewport,
    MobileIssueKind::ImproperViewport,
    MobileIssueKind::NotScalable,
    MobileIssueKind::SmallTargets,
    MobileIssueKind::SmallFonts,
    MobileIssueKind::HorizontalScroll,
];

/// The context keys of the page script plus unknown ones, which pass through.
pub(super) const CONTEXTS: [&str; 13] = [
    "navigation",
    "footer",
    "header",
    "sidebar",
    "social/utility",
    "form",
    "button",
    "other",
    "",
    "brand-new-context",
    "Navigation",
    "{braces} and {$var}",
    "äöüß",
];

const SAMPLES: [&str; 5] = [
    "nav > a.icon (24×24px)",
    "",
    "{braces} and {$var} (1×1px)",
    "button.äöüß — „x“ (40×43px)",
    "https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/and-going/and-going/and-going/and-going/page#anchor > a (0×0px)",
];

/// Value sets for every kind: the touch-target values combined, the font
/// sizes on their own.
pub(super) fn mobile_values() -> Vec<MobileIssueValues> {
    let mut out = Vec::new();
    let counts = [None, Some(0), Some(1), Some(2), Some(5), Some(1_234_567)];
    let context_sets: Vec<Vec<(String, u32)>> = vec![
        vec![],
        vec![("navigation".to_string(), 1)],
        CONTEXTS[..8]
            .iter()
            .enumerate()
            .map(|(i, c)| (c.to_string(), [0, 1, 2, 5, 1_234_567][i % 5]))
            .collect(),
        CONTEXTS[8..].iter().map(|c| (c.to_string(), 2)).collect(),
        vec![
            ("form".to_string(), 3),
            ("brand-new-context".to_string(), 1),
        ],
    ];
    let sample_sets: Vec<Vec<String>> = vec![
        vec![],
        vec![SAMPLES[0].to_string()],
        SAMPLES[..3].iter().map(|s| s.to_string()).collect(),
        SAMPLES.iter().map(|s| s.to_string()).collect(),
    ];
    for count in counts {
        for contexts in &context_sets {
            for samples in &sample_sets {
                out.push(MobileIssueValues {
                    small_target_count: count,
                    small_by_context: contexts.clone(),
                    small_target_samples: samples.clone(),
                    smallest_font_px: None,
                });
            }
        }
    }
    for px in [
        None,
        Some(0.0),
        Some(9.25),
        Some(11.0),
        Some(11.94),
        Some(11.95),
        Some(12.0),
        Some(-0.5),
        Some(1_234_567.8),
        Some(f32::NAN),
        Some(f32::INFINITY),
    ] {
        out.push(MobileIssueValues {
            smallest_font_px: px,
            ..Default::default()
        });
    }
    out
}

#[test]
#[ignore = "writes reports/mobile-issue-texts.json; run before and after a change of the text storage"]
fn export_mobile_issue_texts() {
    let mut issues = Vec::new();
    for kind in MOBILE_KINDS {
        for values in mobile_values() {
            for (lang, en) in [("en", true), ("de", false)] {
                issues.push(json!([
                    format!("{kind:?}"),
                    values.small_target_count,
                    values.small_by_context,
                    values.small_target_samples,
                    values.smallest_font_px.map(|px| format!("{px:?}")),
                    lang,
                    mobile_issue_text(kind, &values, en)
                ]));
            }
        }
    }
    let mut contexts = Vec::new();
    for context in CONTEXTS {
        for (lang, en) in [("en", true), ("de", false)] {
            contexts.push(json!([context, lang, mobile_context_label(context, en)]));
        }
    }
    let path = std::env::var("MOBILE_ISSUE_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/mobile-issue-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    let export = json!({ "issues": issues, "contexts": contexts });
    std::fs::write(&path, serde_json::to_string_pretty(&export).unwrap()).unwrap();
    println!(
        "{} mobile issue and {} context records written to {path}",
        issues.len(),
        contexts.len()
    );
}
