//! SARIF 2.1.0 output for GitHub Code Scanning and other SARIF consumers.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::audit::normalized::NormalizedReport;
use crate::wcag::Severity;

const SARIF_SCHEMA_URI: &str =
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";
const TOOL_NAME: &str = "auditmysite";
const TOOL_INFO_URI: &str = "https://github.com/casoon/auditmysite";
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize)]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Debug, Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Debug, Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Debug, Serialize)]
struct SarifDriver {
    name: &'static str,
    #[serde(rename = "informationUri")]
    information_uri: &'static str,
    version: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(Debug, Serialize)]
struct SarifRule {
    id: String,
    name: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifText,
    #[serde(rename = "fullDescription")]
    full_description: SarifText,
    #[serde(rename = "helpUri", skip_serializing_if = "Option::is_none")]
    help_uri: Option<String>,
    properties: SarifRuleProperties,
}

#[derive(Debug, Serialize)]
struct SarifRuleProperties {
    tags: Vec<String>,
    #[serde(rename = "wcagCriterion", skip_serializing_if = "Option::is_none")]
    wcag_criterion: Option<String>,
}

#[derive(Debug, Serialize)]
struct SarifText {
    text: String,
}

#[derive(Debug, Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: &'static str,
    message: SarifText,
    locations: Vec<SarifLocation>,
}

#[derive(Debug, Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
}

#[derive(Debug, Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

fn sarif_level(severity: &Severity) -> &'static str {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
    }
}

fn location_for(url: &str) -> SarifLocation {
    SarifLocation {
        physical_location: SarifPhysicalLocation {
            artifact_location: SarifArtifactLocation {
                uri: url.to_string(),
            },
        },
    }
}

/// Converts normalized findings across one or more pages into a single SARIF
/// 2.1.0 log, for GitHub Code Scanning and other SARIF consumers.
///
/// There is no source file to point at, so each result's location uses the
/// audited page URL as the artifact URI; element context (selector) is
/// folded into the message text instead of a text region.
pub fn format_sarif(reports: &[&NormalizedReport]) -> anyhow::Result<String> {
    let mut rules: BTreeMap<String, SarifRule> = BTreeMap::new();
    let mut results = Vec::new();

    for report in reports {
        for finding in &report.findings {
            rules
                .entry(finding.rule_id.clone())
                .or_insert_with(|| SarifRule {
                    id: finding.rule_id.clone(),
                    name: finding.title.clone(),
                    short_description: SarifText {
                        text: finding.title.clone(),
                    },
                    full_description: SarifText {
                        text: finding.description.clone(),
                    },
                    help_uri: finding.help_url.clone(),
                    properties: SarifRuleProperties {
                        tags: vec![finding.category.clone(), finding.dimension.clone()],
                        wcag_criterion: if finding.wcag_criterion.is_empty() {
                            None
                        } else {
                            Some(finding.wcag_criterion.clone())
                        },
                    },
                });

            let level = sarif_level(&finding.severity);

            if finding.occurrences.is_empty() {
                results.push(SarifResult {
                    rule_id: finding.rule_id.clone(),
                    level,
                    message: SarifText {
                        text: finding.description.clone(),
                    },
                    locations: vec![location_for(&report.url)],
                });
                continue;
            }

            for occurrence in &finding.occurrences {
                let text = match &occurrence.selector {
                    Some(selector) => format!("{} (selector: {selector})", occurrence.message),
                    None => occurrence.message.clone(),
                };
                results.push(SarifResult {
                    rule_id: finding.rule_id.clone(),
                    level,
                    message: SarifText { text },
                    locations: vec![location_for(&report.url)],
                });
            }
        }
    }

    let log = SarifLog {
        schema: SARIF_SCHEMA_URI,
        version: SARIF_VERSION,
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: TOOL_NAME,
                    information_uri: TOOL_INFO_URI,
                    version: TOOL_VERSION,
                    rules: rules.into_values().collect(),
                },
            },
            results,
        }],
    };

    serde_json::to_string_pretty(&log).map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::normalize;
    use crate::audit::AuditReport;
    use crate::cli::WcagLevel;
    use crate::wcag::{Violation, WcagResults};

    fn sample_report(url: &str) -> AuditReport {
        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Images must have alternate text",
            WcagLevel::A,
            Severity::Critical,
            "Image is missing an alt attribute",
            "node-1",
        ));
        AuditReport::new(url.to_string(), WcagLevel::AA, results, 100)
    }

    #[test]
    fn emits_valid_sarif_shape() {
        let report = sample_report("https://example.com");
        let ctx = normalize(&report);
        let sarif = format_sarif(&[&ctx.normalized]).expect("sarif serializes");

        let value: serde_json::Value = serde_json::from_str(&sarif).expect("valid json");
        assert_eq!(value["version"], "2.1.0");
        assert!(value["runs"][0]["tool"]["driver"]["rules"].is_array());
        assert!(value["runs"][0]["results"].is_array());
    }

    #[test]
    fn merges_rules_across_multiple_pages() {
        let report_a = sample_report("https://example.com/a");
        let report_b = sample_report("https://example.com/b");
        let ctx_a = normalize(&report_a);
        let ctx_b = normalize(&report_b);

        let sarif =
            format_sarif(&[&ctx_a.normalized, &ctx_b.normalized]).expect("sarif serializes");
        let value: serde_json::Value = serde_json::from_str(&sarif).expect("valid json");

        let uris: Vec<&str> = value["runs"][0]["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .unwrap()
            })
            .collect();
        assert!(uris.contains(&"https://example.com/a"));
        assert!(uris.contains(&"https://example.com/b"));
    }
}
