//! Batch consistency analysis (issues #44, #45, #46).
//!
//! Aggregates per-page signals across a `BatchReport` and reports whether
//! shared structural elements are consistent: navigation landmarks, heading
//! hierarchy starts, canonical domain variant.
//!
//! These checks complement WCAG 3.2.3 (Consistent Navigation) and 3.2.4
//! (Consistent Identification) without requiring runtime interaction.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::audit::report::{AuditReport, BatchReport};

/// Aggregated consistency analysis across all pages in a batch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchConsistencyAnalysis {
    pub navigation: NavigationConsistency,
    pub headings: HeadingConsistency,
    pub canonical: CanonicalConsistency,
    pub orphan_pages: OrphanPageAnalysis,
    pub schema_graph: SchemaGraphAnalysis,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrphanPageAnalysis {
    /// Pages not linked from any other audited page.
    pub orphan_urls: Vec<String>,
    pub total_pages: usize,
    pub findings: Vec<String>,
}

/// A conflict between two pages for the same schema `@id` entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEntityConflict {
    pub entity_id: String,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaGraphAnalysis {
    pub conflicts: Vec<SchemaEntityConflict>,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NavigationConsistency {
    /// Pages where the MainNavigation pattern was recognized.
    pub pages_with_main_nav: usize,
    /// Pages where the SkipLink pattern was recognized.
    pub pages_with_skip_link: usize,
    pub total_pages: usize,
    /// Human-readable notes about inconsistencies.
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadingConsistency {
    /// Pages with exactly one H1.
    pub pages_with_single_h1: usize,
    /// Pages with zero H1.
    pub pages_with_no_h1: usize,
    /// Pages with multiple H1s.
    pub pages_with_multiple_h1: usize,
    pub total_pages: usize,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanonicalConsistency {
    /// Number of pages canonicalizing to a www.* host.
    pub www_count: usize,
    /// Number of pages canonicalizing to a non-www host.
    pub non_www_count: usize,
    /// Number of pages with no canonical URL set.
    pub missing_count: usize,
    pub total_pages: usize,
    pub findings: Vec<String>,
}

/// Run all consistency analyses against a `BatchReport`. Returns
/// `None` when the batch has fewer than 2 pages (consistency is not
/// meaningful for a single page).
pub fn analyze(batch: &BatchReport) -> Option<BatchConsistencyAnalysis> {
    if batch.reports.len() < 2 {
        return None;
    }
    Some(BatchConsistencyAnalysis {
        navigation: analyze_navigation(&batch.reports),
        headings: analyze_headings(&batch.reports),
        canonical: analyze_canonical(&batch.reports),
        orphan_pages: analyze_orphan_pages(&batch.reports),
        schema_graph: analyze_schema_graph(&batch.reports),
    })
}

fn analyze_navigation(reports: &[AuditReport]) -> NavigationConsistency {
    let total_pages = reports.len();
    let mut pages_with_main_nav = 0;
    let mut pages_with_skip_link = 0;
    let mut missing_nav = Vec::new();
    let mut missing_skip = Vec::new();

    for r in reports {
        let recognized = r
            .patterns
            .as_ref()
            .map(|p| p.recognized.as_slice())
            .unwrap_or(&[]);
        let has_nav = recognized.iter().any(|p| p.pattern == "MainNavigation");
        let has_skip = recognized.iter().any(|p| p.pattern == "SkipLink");
        if has_nav {
            pages_with_main_nav += 1;
        } else {
            missing_nav.push(r.url.clone());
        }
        if has_skip {
            pages_with_skip_link += 1;
        } else {
            missing_skip.push(r.url.clone());
        }
    }

    let mut findings = Vec::new();
    if pages_with_main_nav > 0 && pages_with_main_nav < total_pages {
        let missing = total_pages - pages_with_main_nav;
        findings.push(format!(
            "{missing} of {total_pages} {} have no recognized main navigation landmark — inconsistent navigation structure across the site.",
            if total_pages == 1 { "page" } else { "pages" }
        ));
    }
    if pages_with_skip_link > 0 && pages_with_skip_link < total_pages {
        let missing = total_pages - pages_with_skip_link;
        findings.push(format!(
            "Skip link present on {pages_with_skip_link} of {total_pages} {}; missing on {missing}. Skip links should appear on every page.",
            if total_pages == 1 { "page" } else { "pages" }
        ));
    }

    NavigationConsistency {
        pages_with_main_nav,
        pages_with_skip_link,
        total_pages,
        findings,
    }
}

fn analyze_headings(reports: &[AuditReport]) -> HeadingConsistency {
    let total_pages = reports.len();
    let mut single = 0;
    let mut none = 0;
    let mut multi = 0;
    let mut findings = Vec::new();

    for r in reports {
        let h1_count = r
            .discoverability
            .seo
            .as_ref()
            .map(|s| s.headings.h1_count)
            .unwrap_or(0);
        match h1_count {
            0 => none += 1,
            1 => single += 1,
            _ => multi += 1,
        }
    }

    if none > 0 {
        findings.push(format!(
            "{none} of {total_pages} {} have no H1 heading. Every page should start with a single H1.",
            if total_pages == 1 { "page" } else { "pages" }
        ));
    }
    if multi > 0 {
        findings.push(format!(
            "{multi} of {total_pages} {} have multiple H1 headings. Use exactly one H1 per page.",
            if total_pages == 1 { "page" } else { "pages" }
        ));
    }

    HeadingConsistency {
        pages_with_single_h1: single,
        pages_with_no_h1: none,
        pages_with_multiple_h1: multi,
        total_pages,
        findings,
    }
}

fn analyze_canonical(reports: &[AuditReport]) -> CanonicalConsistency {
    let total_pages = reports.len();
    let mut www = 0;
    let mut non_www = 0;
    let mut missing = 0;
    let mut findings = Vec::new();

    for r in reports {
        let canonical = r
            .discoverability
            .seo
            .as_ref()
            .and_then(|s| s.technical.canonical_url.as_deref());
        match canonical {
            None => missing += 1,
            Some(url) => match canonical_host(url) {
                Some(host) if host.starts_with("www.") => www += 1,
                Some(_) => non_www += 1,
                None => missing += 1,
            },
        }
    }

    if www > 0 && non_www > 0 {
        findings.push(format!(
            "Mixed canonical strategy: {www} {} canonicalize to www, {non_www} to non-www. Pick one variant and use it everywhere.",
            if www == 1 { "page" } else { "pages" }
        ));
    }
    if missing > 0 {
        findings.push(format!(
            "{missing} of {total_pages} {} have no canonical URL. Set <link rel=\"canonical\"> on every page.",
            if total_pages == 1 { "page" } else { "pages" }
        ));
    }

    CanonicalConsistency {
        www_count: www,
        non_www_count: non_www,
        missing_count: missing,
        total_pages,
        findings,
    }
}

fn analyze_orphan_pages(reports: &[AuditReport]) -> OrphanPageAnalysis {
    let total_pages = reports.len();

    // Collect all internal link targets from every page, normalised.
    let mut all_targets: HashSet<String> = HashSet::new();
    for r in reports {
        if let Some(seo) = r.discoverability.seo.as_ref() {
            for target in &seo.technical.internal_link_targets {
                all_targets.insert(normalise_url(target));
            }
        }
    }

    // A page is an orphan if its own URL is not referenced by any other page.
    let orphan_urls: Vec<String> = reports
        .iter()
        .filter(|r| !all_targets.contains(&normalise_url(&r.url)))
        .map(|r| r.url.clone())
        .collect();

    let mut findings = Vec::new();
    if !orphan_urls.is_empty() {
        findings.push(format!(
            "{} of {total_pages} {} are not linked from any other audited page: {}",
            orphan_urls.len(),
            if orphan_urls.len() == 1 {
                "page"
            } else {
                "pages"
            },
            orphan_urls.join(", ")
        ));
    }

    OrphanPageAnalysis {
        orphan_urls,
        total_pages,
        findings,
    }
}

fn analyze_schema_graph(reports: &[AuditReport]) -> SchemaGraphAnalysis {
    // entity_id → Vec<(page_url, schema_type, name)>
    let mut entities: HashMap<String, Vec<(String, String, String)>> = HashMap::new();

    for r in reports {
        let json_ld = match r
            .discoverability
            .seo
            .as_ref()
            .map(|s| &s.structured_data.json_ld)
        {
            Some(v) => v,
            None => continue,
        };
        for schema in json_ld {
            collect_schema_entities(&schema.content, &r.url, &mut entities);
        }
    }

    let mut conflicts = Vec::new();
    for (entity_id, occurrences) in &entities {
        if occurrences.len() < 2 {
            continue;
        }
        let first_type = &occurrences[0].1;
        let first_name = &occurrences[0].2;
        let type_conflict = occurrences.iter().any(|(_, t, _)| t != first_type);
        let name_conflict = occurrences
            .iter()
            .any(|(_, _, n)| !n.is_empty() && !first_name.is_empty() && n != first_name);

        if type_conflict || name_conflict {
            let mut msgs = Vec::new();
            if type_conflict {
                let types: Vec<&str> = occurrences.iter().map(|(_, t, _)| t.as_str()).collect();
                msgs.push(format!("@type conflict: {}", types.join(" vs ")));
            }
            if name_conflict {
                let names: Vec<&str> = occurrences
                    .iter()
                    .filter(|(_, _, n)| !n.is_empty())
                    .map(|(_, _, n)| n.as_str())
                    .collect();
                msgs.push(format!("name conflict: {}", names.join(" vs ")));
            }
            conflicts.push(SchemaEntityConflict {
                entity_id: entity_id.clone(),
                conflicts: msgs,
            });
        }
    }

    let mut findings = Vec::new();
    if !conflicts.is_empty() {
        findings.push(format!(
            "{} schema entity/entities have conflicting @type or name across pages — review structured data consistency.",
            conflicts.len()
        ));
    }

    SchemaGraphAnalysis {
        conflicts,
        findings,
    }
}

/// Recursively collect entities with `@id` from a JSON-LD value.
fn collect_schema_entities(
    value: &serde_json::Value,
    page_url: &str,
    out: &mut HashMap<String, Vec<(String, String, String)>>,
) {
    match value {
        serde_json::Value::Object(obj) => {
            if let Some(id) = obj.get("@id").and_then(|v| v.as_str()) {
                let schema_type = obj
                    .get("@type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                out.entry(id.to_string()).or_default().push((
                    page_url.to_string(),
                    schema_type,
                    name,
                ));
            }
            // Recurse into nested objects (e.g. @graph array items)
            for v in obj.values() {
                collect_schema_entities(v, page_url, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_schema_entities(v, page_url, out);
            }
        }
        _ => {}
    }
}

fn normalise_url(url: &str) -> String {
    url.trim_end_matches('/').to_lowercase()
}

fn canonical_host(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::report::AuditReport;
    use crate::cli::WcagLevel;
    use crate::patterns::{PatternAnalysis, PatternConfidence, RecognizedPattern};
    use crate::seo::technical::TechnicalSeo;
    use crate::seo::{HeadingStructure, SeoAnalysis};
    use crate::wcag::WcagResults;

    fn make_report(
        url: &str,
        h1_count: usize,
        canonical: Option<&str>,
        recognized: Vec<&str>,
    ) -> AuditReport {
        let mut report = AuditReport::new(url.into(), WcagLevel::AA, WcagResults::new(), 100);
        report.discoverability.seo = Some(SeoAnalysis {
            headings: HeadingStructure {
                h1_count,
                ..Default::default()
            },
            technical: TechnicalSeo {
                canonical_url: canonical.map(String::from),
                ..Default::default()
            },
            ..Default::default()
        });
        report.patterns = Some(PatternAnalysis {
            recognized: recognized
                .into_iter()
                .map(|p| RecognizedPattern {
                    pattern: p.to_string(),
                    message: "test".to_string(),
                    confidence: PatternConfidence::Strong,
                })
                .collect(),
            violations: vec![],
            journey_candidates: vec![],
        });
        report
    }

    #[test]
    fn test_consistent_pages_no_findings() {
        let reports = vec![
            make_report(
                "https://a.com/",
                1,
                Some("https://a.com/"),
                vec!["MainNavigation", "SkipLink"],
            ),
            make_report(
                "https://a.com/x",
                1,
                Some("https://a.com/x"),
                vec!["MainNavigation", "SkipLink"],
            ),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a.navigation.findings.is_empty());
        assert!(a.headings.findings.is_empty());
        assert!(a.canonical.findings.is_empty());
    }

    #[test]
    fn test_mixed_canonical_strategy_flagged() {
        let reports = vec![
            make_report("https://a.com/", 1, Some("https://www.a.com/"), vec![]),
            make_report("https://a.com/x", 1, Some("https://a.com/x"), vec![]),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a
            .canonical
            .findings
            .iter()
            .any(|f| f.contains("Mixed canonical")));
    }

    #[test]
    fn test_inconsistent_navigation_flagged() {
        let reports = vec![
            make_report(
                "https://a.com/",
                1,
                None,
                vec!["MainNavigation", "SkipLink"],
            ),
            make_report("https://a.com/x", 1, None, vec!["MainNavigation"]),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a
            .navigation
            .findings
            .iter()
            .any(|f| f.contains("Skip link")));
    }

    #[test]
    fn test_missing_h1_flagged() {
        let reports = vec![
            make_report("https://a.com/", 0, None, vec![]),
            make_report("https://a.com/x", 1, None, vec![]),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a.headings.findings.iter().any(|f| f.contains("no H1")));
    }

    #[test]
    fn test_single_page_returns_none() {
        let reports = vec![make_report("https://a.com/", 1, None, vec![])];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        assert!(analyze(&batch).is_none());
    }

    fn make_report_with_links(url: &str, link_targets: Vec<&str>) -> AuditReport {
        let mut report = AuditReport::new(url.into(), WcagLevel::AA, WcagResults::new(), 100);
        report.discoverability.seo = Some(SeoAnalysis {
            technical: TechnicalSeo {
                internal_link_targets: link_targets.into_iter().map(String::from).collect(),
                ..Default::default()
            },
            ..Default::default()
        });
        report
    }

    #[test]
    fn test_orphan_page_detected() {
        // page-a links to page-b but not page-c; page-b links to nothing
        let reports = vec![
            make_report_with_links("https://a.com/a", vec!["https://a.com/b"]),
            make_report_with_links("https://a.com/b", vec![]),
            make_report_with_links("https://a.com/c", vec![]),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        // page-a and page-c are not linked from anyone
        assert!(a
            .orphan_pages
            .orphan_urls
            .contains(&"https://a.com/a".to_string()));
        assert!(a
            .orphan_pages
            .orphan_urls
            .contains(&"https://a.com/c".to_string()));
        assert!(!a
            .orphan_pages
            .orphan_urls
            .contains(&"https://a.com/b".to_string()));
    }

    #[test]
    fn test_no_orphans_when_all_linked() {
        let reports = vec![
            make_report_with_links("https://a.com/a", vec!["https://a.com/b"]),
            make_report_with_links("https://a.com/b", vec!["https://a.com/a"]),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a.orphan_pages.orphan_urls.is_empty());
        assert!(a.orphan_pages.findings.is_empty());
    }

    #[test]
    fn test_schema_graph_conflict_detected() {
        use crate::seo::schema::{JsonLdSchema, StructuredData};
        use serde_json::json;

        let make_schema_report = |url: &str, schema_type: &str| {
            let mut report = AuditReport::new(url.into(), WcagLevel::AA, WcagResults::new(), 100);
            report.discoverability.seo = Some(SeoAnalysis {
                structured_data: StructuredData {
                    json_ld: vec![JsonLdSchema {
                        schema_type: schema_type.to_string(),
                        schema_types: vec![schema_type.to_string()],
                        content: json!({
                            "@id": "https://a.com/#org",
                            "@type": schema_type,
                            "name": "Acme"
                        }),
                        is_valid: true,
                    }],
                    has_structured_data: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            report
        };

        let reports = vec![
            make_schema_report("https://a.com/", "Organization"),
            make_schema_report("https://a.com/about", "LocalBusiness"),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(!a.schema_graph.conflicts.is_empty());
        assert!(a.schema_graph.conflicts[0].entity_id == "https://a.com/#org");
        assert!(!a.schema_graph.findings.is_empty());
    }

    #[test]
    fn test_schema_graph_no_conflict_same_type() {
        use crate::seo::schema::{JsonLdSchema, StructuredData};
        use serde_json::json;

        let make_schema_report = |url: &str| {
            let mut report = AuditReport::new(url.into(), WcagLevel::AA, WcagResults::new(), 100);
            report.discoverability.seo = Some(SeoAnalysis {
                structured_data: StructuredData {
                    json_ld: vec![JsonLdSchema {
                        schema_type: "Organization".to_string(),
                        schema_types: vec!["Organization".to_string()],
                        content: json!({
                            "@id": "https://a.com/#org",
                            "@type": "Organization",
                            "name": "Acme"
                        }),
                        is_valid: true,
                    }],
                    has_structured_data: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            report
        };

        let reports = vec![
            make_schema_report("https://a.com/"),
            make_schema_report("https://a.com/about"),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a.schema_graph.conflicts.is_empty());
        assert!(a.schema_graph.findings.is_empty());
    }
}
