//! SARIF 2.1.0 output for GitHub Code Scanning and other SARIF consumers.
//!
//! # Location strategy (#977)
//!
//! An audited page is a URL, not a file in a repository, and no finding
//! carries a source position (line, column): the browser reports elements by
//! selector. This writer therefore never invents a `region`. Each result is
//! located like this:
//!
//! - `physicalLocation.artifactLocation.uri` is a **relative pseudo-path**
//!   derived from the page URL (`audited-pages/<host>/<path>`, see
//!   [`pseudo_path`]). GitHub resolves absolute URIs against the checkout's
//!   `file://` root and rejects a root with a different scheme, so an `https://`
//!   URI would risk the whole upload; a relative path cannot. It matches no
//!   committed file, so GitHub lists the alert without a code view.
//! - The audited URL itself stays in `run.artifacts[].description`, in
//!   `properties.pageUrl` and in the fingerprint.
//! - The element is a `logicalLocations` entry (`kind: element`, the CSS
//!   selector) and part of the message.
//!
//! # Fingerprints
//!
//! Every result carries `partialFingerprints`. `auditmysite/v1` holds
//! `SHA-256(rule_id NUL page_url NUL kind NUL value)` as 32 hex digits plus
//! `:<n>`; `primaryLocationLineHash` repeats it because that is the only key
//! GitHub reads. `page_url` is the URL without fragment; `kind`/`value` is the
//! selector when the occurrence has one, else the whitespace-normalised
//! message. The selector wins over the message because messages carry measured
//! values (a contrast ratio) that change when the page is only edited. `n`
//! counts results with an identical hash in one run, in report order, so equal
//! hashes stay distinct.
//!
//! # Not run
//!
//! A check that did not run is no result. Rules that were disabled, lacked a
//! capability or errored, and modules that failed, were skipped or only ran
//! partially, are listed in `invocations[].toolExecutionNotifications`.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::audit::finding_contract::FindingOutcome;
use crate::audit::normalized::NormalizedReport;
use crate::audit::ExecutionStatus;
use crate::wcag::{NotRun, Severity};

const SARIF_SCHEMA_URI: &str =
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";
const TOOL_NAME: &str = "auditmysite";
const TOOL_INFO_URI: &str = "https://github.com/casoon/auditmysite";
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Key of the documented fingerprint; bump the suffix when the derivation changes.
const FINGERPRINT_KEY: &str = "auditmysite/v1";
/// The key GitHub Code Scanning reads for de-duplication across runs.
const GITHUB_FINGERPRINT_KEY: &str = "primaryLocationLineHash";
const PSEUDO_PATH_PREFIX: &str = "audited-pages";
const LOCATION_STRATEGY: &str = "page-pseudo-path";

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
    invocations: Vec<SarifInvocation>,
    artifacts: Vec<SarifArtifact>,
    results: Vec<SarifResult>,
    properties: SarifRunProperties,
}

#[derive(Debug, Serialize)]
struct SarifRunProperties {
    #[serde(rename = "locationStrategy")]
    location_strategy: &'static str,
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
    help: SarifText,
    #[serde(rename = "helpUri", skip_serializing_if = "Option::is_none")]
    help_uri: Option<String>,
    #[serde(rename = "defaultConfiguration")]
    default_configuration: SarifRuleConfiguration,
    properties: SarifRuleProperties,
}

#[derive(Debug, Serialize)]
struct SarifRuleConfiguration {
    level: &'static str,
}

#[derive(Debug, Serialize)]
struct SarifRuleProperties {
    tags: Vec<String>,
    #[serde(rename = "wcagCriterion", skip_serializing_if = "Option::is_none")]
    wcag_criterion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    precision: Option<String>,
}

#[derive(Debug, Serialize)]
struct SarifText {
    text: String,
}

#[derive(Debug, Serialize)]
struct SarifInvocation {
    #[serde(rename = "executionSuccessful")]
    execution_successful: bool,
    #[serde(rename = "toolExecutionNotifications")]
    tool_execution_notifications: Vec<SarifNotification>,
}

#[derive(Debug, Serialize)]
struct SarifNotification {
    level: &'static str,
    message: SarifText,
    descriptor: SarifDescriptorRef,
    #[serde(rename = "associatedRule", skip_serializing_if = "Option::is_none")]
    associated_rule: Option<SarifDescriptorRef>,
    locations: Vec<SarifLocation>,
    properties: SarifNotificationProperties,
}

#[derive(Debug, Serialize)]
struct SarifDescriptorRef {
    id: String,
}

#[derive(Debug, Serialize)]
struct SarifNotificationProperties {
    #[serde(rename = "pageUrl")]
    page_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    viewport: Option<String>,
    status: String,
}

#[derive(Debug, Serialize)]
struct SarifArtifact {
    location: SarifArtifactLocation,
    description: SarifText,
    roles: [&'static str; 1],
}

#[derive(Debug, Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    #[serde(rename = "ruleIndex")]
    rule_index: usize,
    level: &'static str,
    message: SarifText,
    locations: Vec<SarifLocation>,
    #[serde(rename = "partialFingerprints")]
    partial_fingerprints: BTreeMap<&'static str, String>,
    properties: SarifResultProperties,
}

#[derive(Debug, Serialize)]
struct SarifResultProperties {
    #[serde(rename = "pageUrl")]
    page_url: String,
    /// `fail`, or `review` for a heuristic finding that needs a human.
    outcome: &'static str,
    severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
}

#[derive(Debug, Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
    #[serde(rename = "logicalLocations", skip_serializing_if = "Vec::is_empty")]
    logical_locations: Vec<SarifLogicalLocation>,
}

#[derive(Debug, Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
}

#[derive(Debug, Serialize)]
struct SarifArtifactLocation {
    uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
}

#[derive(Debug, Serialize)]
struct SarifLogicalLocation {
    name: String,
    #[serde(rename = "fullyQualifiedName")]
    fully_qualified_name: String,
    kind: &'static str,
}

fn sarif_level(severity: &Severity) -> &'static str {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
    }
}

/// A heuristic finding is never presented as a confirmed defect, so it is
/// capped at `note`.
fn result_level(severity: &Severity, outcome: FindingOutcome) -> &'static str {
    match outcome {
        FindingOutcome::Fail => sarif_level(severity),
        FindingOutcome::Review => "note",
    }
}

/// The audited URL without fragment, scheme and host in canonical form.
fn normalize_url(url: &str) -> String {
    match url::Url::parse(url) {
        Ok(mut parsed) => {
            parsed.set_fragment(None);
            parsed.to_string()
        }
        Err(_) => url.trim().to_string(),
    }
}

/// The relative pseudo-path a page is reported under:
/// `audited-pages/<host>[:port]/<path>`. An empty path or one ending in `/`
/// gets `index`; a query string becomes `__q-<8 hex>` of its SHA-256, so two
/// URLs that differ only in the query stay two artifacts.
fn pseudo_path(url: &str) -> String {
    let Ok(parsed) = url::Url::parse(url) else {
        let cleaned: String = url
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        return format!("{PSEUDO_PATH_PREFIX}/{cleaned}");
    };
    let mut host = parsed.host_str().unwrap_or("unknown-host").to_string();
    if let Some(port) = parsed.port() {
        host = format!("{host}-{port}");
    }
    let mut path = parsed.path().trim_start_matches('/').to_string();
    if path.is_empty() || path.ends_with('/') {
        path.push_str("index");
    }
    let mut out = format!("{PSEUDO_PATH_PREFIX}/{host}/{path}");
    if let Some(query) = parsed.query() {
        let digest = format!("{:x}", Sha256::digest(query.as_bytes()));
        out.push_str(&format!("__q-{}", &digest[..8]));
    }
    out
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// See the module docs: SHA-256 over rule, page, and selector-or-message.
fn fingerprint_hash(
    rule_id: &str,
    page_url: &str,
    selector: Option<&str>,
    message: &str,
) -> String {
    let (kind, value) = match selector {
        Some(selector) => ("selector", selector.trim().to_string()),
        None => ("message", collapse_whitespace(message)),
    };
    let mut hasher = Sha256::new();
    for part in [rule_id, page_url, kind, value.as_str()] {
        hasher.update(part.as_bytes());
        hasher.update([0u8]);
    }
    let hex = format!("{:x}", hasher.finalize());
    hex[..32].to_string()
}

fn fingerprints(hash: &str, occurrence: usize) -> BTreeMap<&'static str, String> {
    let value = format!("{hash}:{occurrence}");
    BTreeMap::from([
        (FINGERPRINT_KEY, value.clone()),
        (GITHUB_FINGERPRINT_KEY, value),
    ])
}

fn location_for(
    artifact_uri: &str,
    artifact_index: usize,
    page_url: &str,
    selector: Option<&str>,
) -> SarifLocation {
    SarifLocation {
        physical_location: SarifPhysicalLocation {
            artifact_location: SarifArtifactLocation {
                uri: artifact_uri.to_string(),
                index: Some(artifact_index),
            },
        },
        logical_locations: selector
            .map(|selector| {
                vec![SarifLogicalLocation {
                    name: selector.to_string(),
                    fully_qualified_name: format!("{page_url} {selector}"),
                    kind: "element",
                }]
            })
            .unwrap_or_default(),
    }
}

fn precision(confidence: &str) -> Option<String> {
    match confidence {
        "very_high" | "high" | "medium" | "low" => Some(confidence.replace('_', "-")),
        _ => None,
    }
}

fn not_run_notifications(
    report: &NormalizedReport,
    page_url: &str,
    artifact_uri: &str,
    artifact_index: usize,
    out: &mut Vec<SarifNotification>,
) {
    let location = || location_for(artifact_uri, artifact_index, page_url, None);
    for run in &report.rule_outcomes {
        let Some(not_run) = run.not_run else { continue };
        let (level, status) = match not_run {
            NotRun::Errored => ("error", "errored"),
            NotRun::CapabilityMissing => ("warning", "capability_missing"),
            NotRun::Disabled => ("note", "disabled"),
            // Nothing to check on this page: neither tested nor untested.
            NotRun::NotApplicable => continue,
            _ => ("warning", "not_run"),
        };
        let mut text = format!("Rule {} did not run on {page_url} ({status})", run.rule_id);
        if let Some(reason) = &run.reason {
            text.push_str(&format!(": {reason}"));
        }
        out.push(SarifNotification {
            level,
            message: SarifText { text },
            descriptor: SarifDescriptorRef {
                id: "auditmysite.rule_not_run".to_string(),
            },
            associated_rule: Some(SarifDescriptorRef {
                id: run.rule_id.clone(),
            }),
            locations: vec![location()],
            properties: SarifNotificationProperties {
                page_url: page_url.to_string(),
                viewport: run.viewport.clone(),
                status: status.to_string(),
            },
        });
    }
    for run in &report.execution.module_runs {
        let (level, status) = match run.status {
            ExecutionStatus::Failed => ("error", "failed"),
            ExecutionStatus::Partial => ("warning", "partial"),
            ExecutionStatus::Skipped => ("note", "skipped"),
            _ => continue,
        };
        let mut text = format!("Module {} was {status} on {page_url}", run.module);
        if let Some(message) = run.message.as_deref().or(run.reason_code.as_deref()) {
            text.push_str(&format!(": {message}"));
        }
        out.push(SarifNotification {
            level,
            message: SarifText { text },
            descriptor: SarifDescriptorRef {
                id: "auditmysite.module_not_run".to_string(),
            },
            associated_rule: None,
            locations: vec![location()],
            properties: SarifNotificationProperties {
                page_url: page_url.to_string(),
                viewport: None,
                status: status.to_string(),
            },
        });
    }
}

/// Converts normalized findings across one or more pages into a single SARIF
/// 2.1.0 log, for GitHub Code Scanning and other SARIF consumers. Location,
/// fingerprint and not-run handling are described in the module docs.
pub fn format_sarif(reports: &[&NormalizedReport]) -> anyhow::Result<String> {
    for report in reports {
        report.validate()?;
    }
    let mut rules: Vec<SarifRule> = Vec::new();
    let mut rule_index: HashMap<String, usize> = HashMap::new();
    let mut artifacts: Vec<SarifArtifact> = Vec::new();
    let mut artifact_index: HashMap<String, usize> = HashMap::new();
    let mut results = Vec::new();
    let mut notifications = Vec::new();
    let mut seen_hashes: HashMap<String, usize> = HashMap::new();
    let mut seen_pages = BTreeSet::new();

    for report in reports {
        let page_url = normalize_url(&report.url);
        let artifact_uri = pseudo_path(&report.url);
        let index = *artifact_index
            .entry(artifact_uri.clone())
            .or_insert_with(|| {
                artifacts.push(SarifArtifact {
                    location: SarifArtifactLocation {
                        uri: artifact_uri.clone(),
                        index: None,
                    },
                    description: SarifText {
                        text: format!("Audited page {page_url}"),
                    },
                    roles: ["analysisTarget"],
                });
                artifacts.len() - 1
            });
        // The same page twice in one batch must not report not-run twice.
        if seen_pages.insert(page_url.clone()) {
            not_run_notifications(report, &page_url, &artifact_uri, index, &mut notifications);
        }

        for finding in &report.findings {
            let rule_idx = *rule_index
                .entry(finding.rule_id.clone())
                .or_insert_with(|| {
                    let help = finding
                        .occurrences
                        .iter()
                        .find_map(|o| o.fix_suggestion.clone())
                        .unwrap_or_else(|| finding.description.clone());
                    rules.push(SarifRule {
                        id: finding.rule_id.clone(),
                        name: finding.title.clone(),
                        short_description: SarifText {
                            text: finding.title.clone(),
                        },
                        full_description: SarifText {
                            text: finding.description.clone(),
                        },
                        help: SarifText { text: help },
                        help_uri: finding.help_url.clone(),
                        default_configuration: SarifRuleConfiguration {
                            level: result_level(&finding.severity, finding.contract.outcome),
                        },
                        properties: SarifRuleProperties {
                            tags: vec![finding.category.clone(), finding.dimension.clone()],
                            wcag_criterion: if finding.wcag_criterion.is_empty() {
                                None
                            } else {
                                Some(finding.wcag_criterion.clone())
                            },
                            precision: precision(&finding.confidence),
                        },
                    });
                    rules.len() - 1
                });

            let level = result_level(&finding.severity, finding.contract.outcome);
            let outcome = match finding.contract.outcome {
                FindingOutcome::Fail => "fail",
                FindingOutcome::Review => "review",
            };

            // A finding without occurrences is one result for the whole page.
            let whole_page = [crate::audit::normalized::OccurrenceDetail {
                message: finding.description.clone(),
                ..Default::default()
            }];
            let occurrences = if finding.occurrences.is_empty() {
                &whole_page[..]
            } else {
                &finding.occurrences[..]
            };

            for occurrence in occurrences {
                let selector = occurrence.selector.as_deref();
                let text = match selector {
                    Some(selector) => format!("{} (selector: {selector})", occurrence.message),
                    None => occurrence.message.clone(),
                };
                let hash =
                    fingerprint_hash(&finding.rule_id, &page_url, selector, &occurrence.message);
                let n = seen_hashes.entry(hash.clone()).or_insert(0);
                *n += 1;
                results.push(SarifResult {
                    rule_id: finding.rule_id.clone(),
                    rule_index: rule_idx,
                    level,
                    message: SarifText { text },
                    locations: vec![location_for(&artifact_uri, index, &page_url, selector)],
                    partial_fingerprints: fingerprints(&hash, *n),
                    properties: SarifResultProperties {
                        page_url: page_url.clone(),
                        outcome,
                        severity: format!("{:?}", finding.severity).to_lowercase(),
                        selector: selector.map(str::to_string),
                    },
                });
            }
        }
    }

    let execution_successful = !notifications
        .iter()
        .any(|n: &SarifNotification| n.level == "error");
    let log = SarifLog {
        schema: SARIF_SCHEMA_URI,
        version: SARIF_VERSION,
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: TOOL_NAME,
                    information_uri: TOOL_INFO_URI,
                    version: TOOL_VERSION,
                    rules,
                },
            },
            invocations: vec![SarifInvocation {
                execution_successful,
                tool_execution_notifications: notifications,
            }],
            artifacts,
            results,
            properties: SarifRunProperties {
                location_strategy: LOCATION_STRATEGY,
            },
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
        assert!(uris.contains(&"audited-pages/example.com/a"));
        assert!(uris.contains(&"audited-pages/example.com/b"));
    }

    #[test]
    fn pseudo_path_shapes() {
        assert_eq!(
            pseudo_path("https://Example.com"),
            "audited-pages/example.com/index"
        );
        assert_eq!(
            pseudo_path("https://example.com/a/b/"),
            "audited-pages/example.com/a/b/index"
        );
        assert_eq!(
            pseudo_path("http://localhost:8080/x#frag"),
            "audited-pages/localhost-8080/x"
        );
        let q1 = pseudo_path("https://example.com/p?a=1");
        let q2 = pseudo_path("https://example.com/p?a=2");
        assert!(q1.starts_with("audited-pages/example.com/p__q-") && q1 != q2);
        assert_eq!(pseudo_path("not a url"), "audited-pages/not-a-url");
    }

    #[test]
    fn fingerprint_ignores_fragment_and_message_noise_when_a_selector_exists() {
        let a = fingerprint_hash(
            "r",
            &normalize_url("https://e.com/#a"),
            Some("img.x"),
            "m 1",
        );
        let b = fingerprint_hash(
            "r",
            &normalize_url("https://e.com/#b"),
            Some("img.x"),
            "m 2",
        );
        assert_eq!(a, b);
        let c = fingerprint_hash("r", "https://e.com/", None, "a  b");
        let d = fingerprint_hash("r", "https://e.com/", None, "a b");
        assert_eq!(c, d, "whitespace is normalised");
        assert_ne!(c, fingerprint_hash("r", "https://e.com/", None, "a c"));
    }
}
