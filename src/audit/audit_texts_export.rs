//! Text export of the audit layer (#936): the SEO heading-issue titles and
//! technical impacts (`normalized::seo_heading_finding_text`), the 3.2.6
//! assessment basis, region names and deviation sentences
//! (`batch_consistency`) and the management risk dimension names and
//! rationales (`management_risk`) in both languages across an enumerated set
//! of inputs, as records in `reports/audit-texts.json` (gitignored; the path
//! can be set with `AUDIT_TEXT_EXPORT`). Run it before and after a change of
//! how the texts are stored and compare the two files byte for byte.

use super::batch_consistency::{
    consistent_help_basis, help_deviation_text, help_region_name, HelpConsistency, HelpDeviation,
};
use super::management_risk::{blocking_controls_text, ManagementRiskKind};
use super::normalized::seo_heading_finding_text;
use crate::patterns::help_mechanisms::HelpRegion;
use serde_json::{json, Value};

pub(crate) const HEADING_ISSUE_TYPES: [&str; 6] = [
    "long_heading",
    "missing_h1",
    "multiple_h1",
    "skipped_level",
    "empty_heading",
    "something_else",
];

pub(crate) const HELP_REGIONS: [HelpRegion; 6] = [
    HelpRegion::Header,
    HelpRegion::Navigation,
    HelpRegion::Complementary,
    HelpRegion::Footer,
    HelpRegion::Floating,
    HelpRegion::Other,
];

fn region_deviation(
    mechanism: &str,
    expected: &[HelpRegion],
    found: &[HelpRegion],
) -> HelpDeviation {
    HelpDeviation::Region {
        url: "https://example.com/a".to_string(),
        mechanism: mechanism.to_string(),
        expected: expected.to_vec(),
        found: found.to_vec(),
    }
}

fn order_deviation(region: HelpRegion) -> HelpDeviation {
    HelpDeviation::Order {
        url: "https://example.com/b".to_string(),
        region,
        first: "contact".to_string(),
        second: "chat".to_string(),
    }
}

/// The help-consistency states that pick each branch of the 3.2.6 basis, with
/// singular and plural counts of mechanisms, deviations and affected pages.
pub(crate) fn help_consistencies() -> Vec<(&'static str, HelpConsistency)> {
    let page = |url: &str, mechanism: &str| HelpDeviation::Region {
        url: url.to_string(),
        mechanism: mechanism.to_string(),
        expected: vec![HelpRegion::Footer],
        found: vec![HelpRegion::Header],
    };
    let consistency = |repeated: usize, deviations: Vec<HelpDeviation>| HelpConsistency {
        pages_with_help: 3,
        total_pages: 4,
        repeated_mechanisms: repeated,
        deviations,
    };
    vec![
        ("none repeated", consistency(0, vec![])),
        ("one repeated, consistent", consistency(1, vec![])),
        ("three repeated, consistent", consistency(3, vec![])),
        (
            "one repeated, one deviation",
            consistency(1, vec![page("https://example.com/a", "contact")]),
        ),
        (
            "two repeated, two deviations on one page",
            consistency(
                2,
                vec![
                    page("https://example.com/a", "contact"),
                    page("https://example.com/a", "chat"),
                ],
            ),
        ),
        (
            "four repeated, three deviations on two pages",
            consistency(
                4,
                vec![
                    page("https://example.com/a", "contact"),
                    page("https://example.com/b", "chat"),
                    order_deviation(HelpRegion::Footer),
                ],
            ),
        ),
        (
            "large counts",
            consistency(
                1234,
                (0..1100)
                    .map(|i| page(&format!("https://example.com/{i}"), "contact"))
                    .collect(),
            ),
        ),
    ]
}

/// One deviation per sentence shape: regions joined from one, two and three
/// names, and the reversed order in every region.
pub(crate) fn help_deviations() -> Vec<(String, HelpDeviation)> {
    let mut out = vec![
        (
            "region, one and one".to_string(),
            region_deviation("contact", &[HelpRegion::Footer], &[HelpRegion::Header]),
        ),
        (
            "region, two and one".to_string(),
            region_deviation(
                "chat",
                &[HelpRegion::Footer, HelpRegion::Floating],
                &[HelpRegion::Navigation],
            ),
        ),
        (
            "region, one and three".to_string(),
            region_deviation(
                "phone",
                &[HelpRegion::Other],
                &[
                    HelpRegion::Header,
                    HelpRegion::Complementary,
                    HelpRegion::Footer,
                ],
            ),
        ),
    ];
    for region in HELP_REGIONS {
        out.push((format!("order in {region:?}"), order_deviation(region)));
    }
    out
}

/// Every management risk kind with the values that pick a different wording:
/// singular and plural counts, no blockers, unnamed-only, keyboard-only and
/// both kinds of blockers, scores measured and not measured, a multi-page run
/// and a single URL.
pub(crate) fn risk_kinds() -> Vec<(String, ManagementRiskKind)> {
    let mut out: Vec<(String, ManagementRiskKind)> = Vec::new();
    // (total blocking, keyboard subset): none, unnamed only (1 and 3),
    // keyboard only (1 and 2), both (1+1, 2+3).
    let blocking = [(0, 0), (1, 0), (3, 0), (1, 1), (2, 2), (2, 1), (5, 3)];
    for legal_flags in [0, 1, 2, 4] {
        for (blocking_issues, keyboard_blocking) in blocking {
            out.push((
                format!("legal {legal_flags} blocking {blocking_issues}/{keyboard_blocking}"),
                ManagementRiskKind::Legal {
                    legal_flags,
                    critical: 2,
                    high: 5,
                    blocking_issues,
                    keyboard_blocking,
                },
            ));
        }
    }
    for (critical, high) in [(0, 0), (1, 0), (0, 3), (2, 4)] {
        for (blocking_issues, keyboard_blocking) in blocking {
            out.push((
                format!(
                    "assistive {critical}/{high} blocking {blocking_issues}/{keyboard_blocking}"
                ),
                ManagementRiskKind::AssistiveTechnology {
                    critical,
                    high,
                    blocking_issues,
                    keyboard_blocking,
                },
            ));
        }
    }
    for (performance, mobile) in [
        (Some(81), Some(64)),
        (None, Some(64)),
        (Some(81), None),
        (None, None),
    ] {
        for (blocking_issues, keyboard_blocking) in blocking {
            out.push((
                format!(
                    "conversion {performance:?}/{mobile:?} blocking {blocking_issues}/{keyboard_blocking}"
                ),
                ManagementRiskKind::Conversion {
                    accessibility: 77,
                    performance,
                    mobile,
                    blocking_issues,
                    keyboard_blocking,
                },
            ));
        }
    }
    for score in [Some(0), Some(95), None] {
        out.push((format!("seo {score:?}"), ManagementRiskKind::Seo { score }));
    }
    for (performance, mobile) in [
        (Some(81), Some(64)),
        (None, Some(64)),
        (Some(81), None),
        (None, None),
    ] {
        for high_findings in [0, 1, 4, 1234] {
            out.push((
                format!("performance/mobile {performance:?}/{mobile:?} high {high_findings}"),
                ManagementRiskKind::PerformanceMobile {
                    performance,
                    mobile,
                    high_findings,
                },
            ));
        }
    }
    for (accessibility, critical, high) in [(100, 0, 0), (61, 1, 1), (35, 12, 40)] {
        out.push((
            format!("trust {accessibility}/{critical}/{high}"),
            ManagementRiskKind::Trust {
                accessibility,
                critical,
                high,
            },
        ));
    }
    for count in [0, 1, 2, 17] {
        for pages in [None, Some(2), Some(40)] {
            out.push((
                format!("project {count} pages {pages:?}"),
                ManagementRiskKind::Project { count, pages },
            ));
        }
    }
    out
}

/// Every text of the audit layer in one language.
pub(crate) fn audit_texts(en: bool) -> Vec<Value> {
    let mut out = Vec::new();
    for issue_type in HEADING_ISSUE_TYPES {
        let (title, technical_impact) =
            seo_heading_finding_text(issue_type, en, "fallback message");
        out.push(json!(["heading_title", issue_type, title]));
        out.push(json!([
            "heading_technical_impact",
            issue_type,
            technical_impact
        ]));
    }
    for (label, help) in help_consistencies() {
        out.push(json!([
            "help_basis",
            label,
            consistent_help_basis(&help, en)
        ]));
    }
    for region in HELP_REGIONS {
        out.push(json!([
            "help_region_name",
            format!("{region:?}"),
            help_region_name(region, en)
        ]));
    }
    for (label, deviation) in help_deviations() {
        out.push(json!([
            "help_deviation_text",
            label,
            help_deviation_text(&deviation, en)
        ]));
    }
    for (label, kind) in risk_kinds() {
        out.push(json!(["risk_dimension", label, kind.dimension(en)]));
        out.push(json!(["risk_rationale", label, kind.rationale(en)]));
    }
    for total in 0..4 {
        for keyboard in 0..=total {
            out.push(json!([
                "blocking_controls_text",
                format!("{total}/{keyboard}"),
                blocking_controls_text(total, keyboard, en)
            ]));
        }
    }
    out
}

#[test]
#[ignore = "writes reports/audit-texts.json; run before and after a change of the text storage"]
fn export_audit_texts() {
    let mut records = Vec::new();
    for lang in ["en", "de"] {
        for text in audit_texts(lang == "en") {
            records.push(json!({ "lang": lang, "text": text }));
        }
    }
    let path = std::env::var("AUDIT_TEXT_EXPORT")
        .unwrap_or_else(|_| format!("{}/reports/audit-texts.json", env!("CARGO_MANIFEST_DIR")));
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} audit text records written to {path}", records.len());
}
