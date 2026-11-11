//! Text export of the security issue texts (#936): every text of
//! `security_issue_text` in both languages across an enumerated input space,
//! as records in `reports/security-issue-texts.json` (gitignored; the path can
//! be set with `SECURITY_ISSUE_TEXT_EXPORT`). Run it before and after a change
//! of how the texts are stored and compare the two files byte for byte.

use super::{security_issue_text, SecurityIssueKind, SecurityIssueValues};
use serde_json::json;

const TEXTS: [&str; 6] = [
    "",
    "https://example.com/app.js.map",
    " padded value ",
    "{braces} and {$var}",
    "äöüß — „quotes“ \"x\" <b>; semi",
    "https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/and-going/and-going/and-going/and-going/bundle.min.js.map?v=1234567890",
];

/// Value sets for every kind: no values, all values, and each group of values
/// varied on its own.
pub(super) fn security_values() -> Vec<SecurityIssueValues> {
    let mut out = vec![
        SecurityIssueValues::default(),
        SecurityIssueValues {
            hsts_max_age: Some("100".to_string()),
            hsts_include_subdomains: Some(false),
            source_map_count: Some(3),
            source_map_example: Some(TEXTS[1].to_string()),
            csp_directive: Some("base-uri".to_string()),
        },
    ];
    let max_ages = [
        None,
        Some(""),
        Some("0"),
        Some("31536000"),
        Some(TEXTS[3]),
        Some(TEXTS[4]),
    ];
    for max_age in max_ages {
        for subdomains in [None, Some(false), Some(true)] {
            out.push(SecurityIssueValues {
                hsts_max_age: max_age.map(String::from),
                hsts_include_subdomains: subdomains,
                ..Default::default()
            });
        }
    }
    let counts = [None, Some(0), Some(1), Some(2), Some(5), Some(1_234_567)];
    let examples = [
        None,
        Some(TEXTS[0]),
        Some(TEXTS[1]),
        Some(TEXTS[2]),
        Some(TEXTS[3]),
        Some(TEXTS[4]),
        Some(TEXTS[5]),
    ];
    for count in counts {
        for example in examples {
            out.push(SecurityIssueValues {
                source_map_count: count,
                source_map_example: example.map(String::from),
                ..Default::default()
            });
        }
    }
    for directive in [
        None,
        Some(""),
        Some("object-src"),
        Some("base-uri"),
        Some("frame-ancestors"),
        Some(TEXTS[3]),
        Some(TEXTS[4]),
    ] {
        out.push(SecurityIssueValues {
            csp_directive: directive.map(String::from),
            ..Default::default()
        });
    }
    out
}

#[test]
#[ignore = "writes reports/security-issue-texts.json; run before and after a change of the text storage"]
fn export_security_issue_texts() {
    let mut records = Vec::new();
    for kind in SecurityIssueKind::ALL {
        for values in security_values() {
            for (lang, en) in [("en", true), ("de", false)] {
                records.push(json!([
                    format!("{kind:?}"),
                    values,
                    lang,
                    security_issue_text(kind, &values, en)
                ]));
            }
        }
    }
    let path = std::env::var("SECURITY_ISSUE_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/security-issue-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} security records written to {path}", records.len());
}
