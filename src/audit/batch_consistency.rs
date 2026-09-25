//! Batch consistency analysis (issues #44, #45, #46).
//!
//! Aggregates per-page signals across a `BatchReport` and reports whether
//! shared structural elements are consistent: navigation landmarks, heading
//! hierarchy starts, canonical domain variant.
//!
//! These checks complement WCAG 3.2.3 (Consistent Navigation) and 3.2.4
//! (Consistent Identification) without requiring runtime interaction, and
//! carry the only automated evidence for 3.2.6 (Consistent Help), which a
//! single page cannot violate.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::audit::report::{AuditReport, BatchReport};
use crate::patterns::help_mechanisms::HelpRegion;

/// Aggregated consistency analysis across all pages in a batch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchConsistencyAnalysis {
    pub navigation: NavigationConsistency,
    /// WCAG 3.2.6: recurring help mechanisms compared across pages.
    #[serde(default)]
    pub help: HelpConsistency,
    pub headings: HeadingConsistency,
    pub canonical: CanonicalConsistency,
    pub orphan_pages: OrphanPageAnalysis,
    pub schema_graph: SchemaGraphAnalysis,
    pub structured_data: StructuredDataConsistency,
    /// Criteria that require comparison across multiple pages. These are not
    /// emitted as single-page conformance claims.
    pub wcag_cross_page: Vec<CrossPageCriterionAssessment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPageCriterionAssessment {
    pub criterion: String,
    pub status: String,
    pub basis: String,
    pub affected_pages: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredDataConsistency {
    pub type_distribution: Vec<SchemaTypeCount>,
    pub recurring_blockers: Vec<RecurringSchemaFinding>,
    pub parity_mismatches: Vec<RecurringSchemaFinding>,
    pub page_type_matrix: Vec<SchemaPageMatrixRow>,
    pub identity_findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaTypeCount {
    pub schema_type: String,
    pub pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringSchemaFinding {
    pub key: String,
    pub affected_pages: usize,
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPageMatrixRow {
    pub url: String,
    pub page_kind: String,
    pub coverage_status: String,
    pub expected_types: Vec<String>,
    pub detected_types: Vec<String>,
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

/// Cross-page comparison of help mechanisms for WCAG 3.2.6 Consistent Help.
///
/// Only mechanisms found on at least two pages are constrained by the
/// criterion. For those, two things are compared: the landmark regions the
/// mechanism sits in, and — within one region — the order of two mechanisms
/// relative to each other. Both are judged against the placement most pages
/// use. A region deviation needs *disjoint* region sets, so a page that
/// merely repeats the phone number in its header on top of the shared footer
/// is not flagged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelpConsistency {
    /// Pages with at least one help mechanism outside `main`.
    pub pages_with_help: usize,
    pub total_pages: usize,
    /// Distinct mechanisms found on at least two pages.
    pub repeated_mechanisms: usize,
    pub deviations: Vec<HelpDeviation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HelpDeviation {
    /// The mechanism sits only in regions where most pages don't place it.
    Region {
        url: String,
        mechanism: String,
        expected: Vec<HelpRegion>,
        found: Vec<HelpRegion>,
    },
    /// Within `region`, most pages place `first` before `second`; this page
    /// reverses them.
    Order {
        url: String,
        region: HelpRegion,
        first: String,
        second: String,
    },
}

impl HelpDeviation {
    pub fn url(&self) -> &str {
        match self {
            Self::Region { url, .. } | Self::Order { url, .. } => url,
        }
    }
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
        help: analyze_help(&batch.reports),
        headings: analyze_headings(&batch.reports),
        canonical: analyze_canonical(&batch.reports),
        orphan_pages: analyze_orphan_pages(&batch.reports),
        schema_graph: analyze_schema_graph(&batch.reports),
        structured_data: analyze_structured_data(&batch.reports),
        wcag_cross_page: analyze_cross_page_criteria(&batch.reports),
    })
}

fn analyze_cross_page_criteria(reports: &[AuditReport]) -> Vec<CrossPageCriterionAssessment> {
    let navigation = analyze_navigation(reports);
    let help = analyze_help(reports);
    let orphan_pages = analyze_orphan_pages(reports);
    let inconsistent_navigation = navigation.total_pages - navigation.pages_with_main_nav;
    vec![
        CrossPageCriterionAssessment {
            criterion: "3.2.3 Consistent Navigation".to_string(),
            status: if navigation.findings.is_empty() {
                "no_inconsistency_detected"
            } else {
                "warning"
            }
            .to_string(),
            basis: "Main-navigation and skip-link presence compared across the audited page set"
                .to_string(),
            affected_pages: inconsistent_navigation,
        },
        CrossPageCriterionAssessment {
            criterion: "3.2.4 Consistent Identification".to_string(),
            status: "manual_review".to_string(),
            basis: "Component identity requires accessible-name comparison across equivalent controls; current evidence is insufficient for a conformance claim"
                .to_string(),
            affected_pages: 0,
        },
        CrossPageCriterionAssessment {
            criterion: "3.2.6 Consistent Help".to_string(),
            status: consistent_help_status(&help).to_string(),
            basis: consistent_help_basis(&help, true),
            affected_pages: help
                .deviations
                .iter()
                .map(HelpDeviation::url)
                .collect::<HashSet<_>>()
                .len(),
        },
        CrossPageCriterionAssessment {
            criterion: "2.4.5 Multiple Ways".to_string(),
            status: if orphan_pages.orphan_urls.is_empty() {
                "manual_review"
            } else {
                "warning"
            }
            .to_string(),
            basis: "Inbound links within the audited set are checked; search, sitemap and process-step exceptions still require manual confirmation"
                .to_string(),
            affected_pages: orphan_pages.orphan_urls.len(),
        },
    ]
}

/// Status of the 3.2.6 assessment. Without a recurring mechanism there is
/// nothing to compare — but also no evidence that the site offers no help in a
/// form the inventory doesn't recognise, so that stays a manual check.
fn consistent_help_status(help: &HelpConsistency) -> &'static str {
    if help.repeated_mechanisms == 0 {
        "manual_review"
    } else if help.deviations.is_empty() {
        "no_inconsistency_detected"
    } else {
        "warning"
    }
}

/// Assessment basis for 3.2.6 — the single text source for the JSON
/// (`en = true`) and the localized batch PDF (#406).
pub fn consistent_help_basis(help: &HelpConsistency, en: bool) -> String {
    let affected = help
        .deviations
        .iter()
        .map(HelpDeviation::url)
        .collect::<HashSet<_>>()
        .len();
    let count =
        |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    let mechanisms = if en {
        count(
            help.repeated_mechanisms,
            "recurring help mechanism",
            "recurring help mechanisms",
        )
    } else {
        count(
            help.repeated_mechanisms,
            "wiederkehrender Hilfemechanismus",
            "wiederkehrende Hilfemechanismen",
        )
    };
    match (consistent_help_status(help), en) {
        ("manual_review", true) => "No help mechanism (contact or help link, e-mail, phone, known chat widget) recurs outside the main content on two or more audited pages; other forms of help must be checked manually.".to_string(),
        ("manual_review", false) => "Kein Hilfemechanismus (Kontakt- oder Hilfe-Link, E-Mail, Telefon, bekanntes Chat-Widget) wiederholt sich außerhalb des Hauptinhalts auf mindestens zwei geprüften Seiten; andere Formen von Hilfe sind manuell zu prüfen.".to_string(),
        ("no_inconsistency_detected", true) => format!(
            "{mechanisms} compared across the audited pages by landmark region and relative order; no deviation found."
        ),
        ("no_inconsistency_detected", false) => format!(
            "{mechanisms} über die geprüften Seiten nach Seitenbereich und relativer Reihenfolge verglichen; keine Abweichung gefunden."
        ),
        (_, true) => format!(
            "{mechanisms} compared by landmark region and relative order; {} on {}.",
            count(help.deviations.len(), "deviation", "deviations"),
            count(affected, "page", "pages")
        ),
        (_, false) => format!(
            "{mechanisms} nach Seitenbereich und relativer Reihenfolge verglichen; {} auf {}.",
            count(help.deviations.len(), "Abweichung", "Abweichungen"),
            count(affected, "Seite", "Seiten")
        ),
    }
}

/// Display name of a landmark region — single text source for JSON consumers
/// and the localized batch PDF (#406).
pub fn help_region_name(region: HelpRegion, en: bool) -> &'static str {
    match (region, en) {
        (HelpRegion::Header, true) => "header",
        (HelpRegion::Header, false) => "Kopfbereich",
        (HelpRegion::Navigation, true) => "navigation",
        (HelpRegion::Navigation, false) => "Navigation",
        (HelpRegion::Complementary, true) => "sidebar",
        (HelpRegion::Complementary, false) => "Seitenleiste",
        (HelpRegion::Footer, true) => "footer",
        (HelpRegion::Footer, false) => "Fußbereich",
        (HelpRegion::Floating, true) => "floating overlay",
        (HelpRegion::Floating, false) => "schwebende Einblendung",
        (HelpRegion::Other, true) => "page body outside the main content",
        (HelpRegion::Other, false) => "Seitenkörper außerhalb des Hauptinhalts",
    }
}

/// One-sentence description of a 3.2.6 deviation (#406: pure text function,
/// the PDF calls it with the run locale).
pub fn help_deviation_text(deviation: &HelpDeviation, en: bool) -> String {
    let regions = |list: &[HelpRegion]| {
        list.iter()
            .map(|r| help_region_name(*r, en))
            .collect::<Vec<_>>()
            .join(if en { " and " } else { " und " })
    };
    match deviation {
        HelpDeviation::Region {
            mechanism,
            expected,
            found,
            ..
        } => {
            if en {
                format!(
                    "'{mechanism}' sits in the {} instead of the {} used on most pages.",
                    regions(found),
                    regions(expected)
                )
            } else {
                format!(
                    "„{mechanism}“ steht im Bereich {} statt wie auf den meisten Seiten im Bereich {}.",
                    regions(found),
                    regions(expected)
                )
            }
        }
        HelpDeviation::Order {
            region,
            first,
            second,
            ..
        } => {
            if en {
                format!(
                    "In the {}, '{second}' comes before '{first}'; most pages use the reverse order.",
                    help_region_name(*region, true)
                )
            } else {
                format!(
                    "Im Bereich {} steht „{second}“ vor „{first}“; die meisten Seiten nutzen die umgekehrte Reihenfolge.",
                    help_region_name(*region, false)
                )
            }
        }
    }
}

fn analyze_help(reports: &[AuditReport]) -> HelpConsistency {
    let pages: Vec<(&str, &[crate::patterns::help_mechanisms::HelpMechanism])> = reports
        .iter()
        .map(|r| {
            (
                r.url.as_str(),
                r.patterns
                    .as_ref()
                    .map(|p| p.help_mechanisms.as_slice())
                    .unwrap_or(&[]),
            )
        })
        .collect();

    // Mechanism → (page → regions it occupies there).
    let mut placements: BTreeMap<&str, BTreeMap<&str, BTreeSet<HelpRegion>>> = BTreeMap::new();
    for (url, mechanisms) in &pages {
        for m in *mechanisms {
            placements
                .entry(m.key.as_str())
                .or_default()
                .entry(url)
                .or_default()
                .insert(m.region);
        }
    }
    placements.retain(|_, per_page| per_page.len() >= 2);

    let mut deviations = Vec::new();

    for (key, per_page) in &placements {
        let mut counts: BTreeMap<&BTreeSet<HelpRegion>, usize> = BTreeMap::new();
        for regions in per_page.values() {
            *counts.entry(regions).or_default() += 1;
        }
        // Most common placement; ties resolve to the first set in order, so
        // the result doesn't depend on page order.
        let Some(expected) = counts
            .iter()
            .fold(
                None::<(&BTreeSet<HelpRegion>, usize)>,
                |best, (set, n)| match best {
                    Some((_, best_n)) if best_n >= *n => best,
                    _ => Some((set, *n)),
                },
            )
            .map(|(set, _)| set)
        else {
            continue;
        };
        for (url, regions) in per_page {
            if regions.is_disjoint(expected) {
                deviations.push(HelpDeviation::Region {
                    url: url.to_string(),
                    mechanism: key.to_string(),
                    expected: expected.iter().copied().collect(),
                    found: regions.iter().copied().collect(),
                });
            }
        }
    }

    // (region, a, b) with a < b → pages placing a first / pages placing b first.
    type Pair<'a> = (HelpRegion, &'a str, &'a str);
    let mut orders: BTreeMap<Pair<'_>, (Vec<&str>, Vec<&str>)> = BTreeMap::new();
    for (url, mechanisms) in &pages {
        let mut by_region: BTreeMap<HelpRegion, Vec<&str>> = BTreeMap::new();
        for m in *mechanisms {
            if placements.contains_key(m.key.as_str()) {
                by_region.entry(m.region).or_default().push(m.key.as_str());
            }
        }
        for (region, keys) in by_region {
            for (i, earlier) in keys.iter().enumerate() {
                for later in &keys[i + 1..] {
                    let (a, b, forward) = if earlier < later {
                        (*earlier, *later, true)
                    } else {
                        (*later, *earlier, false)
                    };
                    let entry = orders.entry((region, a, b)).or_default();
                    if forward {
                        entry.0.push(url);
                    } else {
                        entry.1.push(url);
                    }
                }
            }
        }
    }
    for ((region, a, b), (a_first, b_first)) in orders {
        if a_first.is_empty() || b_first.is_empty() {
            continue;
        }
        let (first, second, deviating) = if a_first.len() >= b_first.len() {
            (a, b, b_first)
        } else {
            (b, a, a_first)
        };
        for url in deviating {
            deviations.push(HelpDeviation::Order {
                url: url.to_string(),
                region,
                first: first.to_string(),
                second: second.to_string(),
            });
        }
    }

    deviations.sort_by(|x, y| x.url().cmp(y.url()));

    HelpConsistency {
        pages_with_help: pages.iter().filter(|(_, m)| !m.is_empty()).count(),
        total_pages: reports.len(),
        repeated_mechanisms: placements.len(),
        deviations,
    }
}

fn analyze_structured_data(reports: &[AuditReport]) -> StructuredDataConsistency {
    let mut distribution: HashMap<String, HashSet<String>> = HashMap::new();
    let mut blockers: HashMap<String, HashSet<String>> = HashMap::new();
    let mut parity: HashMap<String, HashSet<String>> = HashMap::new();
    let mut page_type_matrix = Vec::new();
    let mut organization_ids = HashSet::new();
    let mut organization_pages = 0usize;
    let mut organization_pages_without_id = 0usize;

    for report in reports {
        let Some(seo) = report.discoverability.seo.as_ref() else {
            continue;
        };
        for schema_type in &seo.structured_data.types {
            distribution
                .entry(schema_type.as_str().to_string())
                .or_default()
                .insert(report.url.clone());
        }
        for assessment in &seo.structured_data.rule_assessments {
            for property in &assessment.missing_required {
                blockers
                    .entry(format!(
                        "{} / {} / {}",
                        assessment.schema_type,
                        assessment.feature.key(),
                        property
                    ))
                    .or_default()
                    .insert(report.url.clone());
            }
        }
        for assessment in &seo.structured_data.content_parity {
            if assessment.status == crate::seo::schema_parity::ContentParityStatus::Mismatch {
                parity
                    .entry(format!(
                        "{} / {}",
                        assessment.schema_type, assessment.property
                    ))
                    .or_default()
                    .insert(report.url.clone());
            }
        }
        if let Some(fit) = &seo.structured_data.fit_assessment {
            page_type_matrix.push(SchemaPageMatrixRow {
                url: report.url.clone(),
                page_kind: enum_key(&fit.page_kind),
                coverage_status: enum_key(&fit.coverage_status),
                expected_types: fit.expected_primary_types.clone(),
                detected_types: fit.detected_primary_types.clone(),
            });
        }
        for schema in &seo.structured_data.json_ld {
            if schema
                .schema_types
                .iter()
                .any(|schema_type| schema_type == "Organization" || schema_type == "LocalBusiness")
            {
                organization_pages += 1;
                if let Some(id) = schema
                    .content
                    .get("@id")
                    .and_then(serde_json::Value::as_str)
                {
                    organization_ids.insert(id.trim_end_matches('/').to_string());
                } else {
                    organization_pages_without_id += 1;
                }
            }
        }
    }

    let mut type_distribution = distribution
        .into_iter()
        .map(|(schema_type, urls)| SchemaTypeCount {
            schema_type,
            pages: urls.len(),
        })
        .collect::<Vec<_>>();
    type_distribution.sort_by(|left, right| {
        right
            .pages
            .cmp(&left.pages)
            .then_with(|| left.schema_type.cmp(&right.schema_type))
    });

    let mut recurring_blockers = recurring_findings(blockers);
    let mut parity_mismatches = recurring_findings(parity);
    recurring_blockers.retain(|finding| finding.affected_pages >= 2);
    parity_mismatches.retain(|finding| finding.affected_pages >= 2);

    let mut identity_findings = Vec::new();
    if organization_ids.len() > 1 {
        identity_findings.push(format!(
            "Organization identity uses {} different @id values across the audited pages.",
            organization_ids.len()
        ));
    }
    if organization_pages > 1 && organization_pages_without_id > 0 {
        identity_findings.push(format!(
            "{organization_pages_without_id} of {organization_pages} organization nodes have no stable @id for cross-page identity validation."
        ));
    }

    StructuredDataConsistency {
        type_distribution,
        recurring_blockers,
        parity_mismatches,
        page_type_matrix,
        identity_findings,
    }
}

fn recurring_findings(findings: HashMap<String, HashSet<String>>) -> Vec<RecurringSchemaFinding> {
    let mut result = findings
        .into_iter()
        .map(|(key, urls)| {
            let mut urls = urls.into_iter().collect::<Vec<_>>();
            urls.sort();
            RecurringSchemaFinding {
                key,
                affected_pages: urls.len(),
                urls,
            }
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| {
        right
            .affected_pages
            .cmp(&left.affected_pages)
            .then_with(|| left.key.cmp(&right.key))
    });
    result
}

fn enum_key<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
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
            help_mechanisms: vec![],
        });
        report
    }

    fn help_report(url: &str, mechanisms: &[(&str, HelpRegion)]) -> AuditReport {
        use crate::patterns::help_mechanisms::{HelpKind, HelpMechanism};
        let mut report = make_report(url, 1, None, vec![]);
        report.patterns.as_mut().unwrap().help_mechanisms = mechanisms
            .iter()
            .map(|(key, region)| HelpMechanism {
                kind: HelpKind::ContactPage,
                key: key.to_string(),
                region: *region,
            })
            .collect();
        report
    }

    fn cross_page_326(a: &BatchConsistencyAnalysis) -> &CrossPageCriterionAssessment {
        a.wcag_cross_page
            .iter()
            .find(|c| c.criterion.starts_with("3.2.6"))
            .expect("3.2.6 assessment")
    }

    #[test]
    fn consistent_help_passes_when_placement_matches() {
        use HelpRegion::*;
        let layout = [("page:a.com/kontakt", Header), ("tel:+4930123", Footer)];
        let reports = vec![
            help_report("https://a.com/", &layout),
            help_report("https://a.com/x", &layout),
        ];
        let a = analyze(&BatchReport::from_reports(reports, vec![], 100)).unwrap();
        assert_eq!(a.help.repeated_mechanisms, 2);
        assert!(a.help.deviations.is_empty());
        assert_eq!(cross_page_326(&a).status, "no_inconsistency_detected");
    }

    #[test]
    fn consistent_help_flags_mechanism_moved_to_another_region() {
        use HelpRegion::*;
        let reports = vec![
            help_report("https://a.com/", &[("page:a.com/kontakt", Header)]),
            help_report("https://a.com/x", &[("page:a.com/kontakt", Header)]),
            help_report("https://a.com/y", &[("page:a.com/kontakt", Footer)]),
        ];
        let a = analyze(&BatchReport::from_reports(reports, vec![], 100)).unwrap();
        assert_eq!(
            a.help.deviations,
            vec![HelpDeviation::Region {
                url: "https://a.com/y".into(),
                mechanism: "page:a.com/kontakt".into(),
                expected: vec![Header],
                found: vec![Footer],
            }]
        );
        let c = cross_page_326(&a);
        assert_eq!(c.status, "warning");
        assert_eq!(c.affected_pages, 1);
    }

    #[test]
    fn consistent_help_accepts_an_additional_region() {
        use HelpRegion::*;
        let reports = vec![
            help_report("https://a.com/", &[("tel:+4930123", Footer)]),
            help_report(
                "https://a.com/x",
                &[("tel:+4930123", Header), ("tel:+4930123", Footer)],
            ),
        ];
        let a = analyze(&BatchReport::from_reports(reports, vec![], 100)).unwrap();
        assert!(a.help.deviations.is_empty(), "{:?}", a.help.deviations);
    }

    #[test]
    fn consistent_help_flags_reversed_order_within_a_region() {
        use HelpRegion::*;
        let usual = [("mailto:info@a.com", Footer), ("tel:+4930123", Footer)];
        let reports = vec![
            help_report("https://a.com/", &usual),
            help_report("https://a.com/x", &usual),
            help_report(
                "https://a.com/y",
                &[("tel:+4930123", Footer), ("mailto:info@a.com", Footer)],
            ),
        ];
        let a = analyze(&BatchReport::from_reports(reports, vec![], 100)).unwrap();
        assert_eq!(
            a.help.deviations,
            vec![HelpDeviation::Order {
                url: "https://a.com/y".into(),
                region: Footer,
                first: "mailto:info@a.com".into(),
                second: "tel:+4930123".into(),
            }]
        );
    }

    #[test]
    fn consistent_help_stays_manual_without_a_recurring_mechanism() {
        use HelpRegion::*;
        let reports = vec![
            help_report("https://a.com/", &[("page:a.com/kontakt", Header)]),
            help_report("https://a.com/x", &[]),
        ];
        let a = analyze(&BatchReport::from_reports(reports, vec![], 100)).unwrap();
        assert_eq!(a.help.repeated_mechanisms, 0);
        assert_eq!(cross_page_326(&a).status, "manual_review");
    }

    #[test]
    fn consistent_help_basis_en_has_no_german() {
        let help = HelpConsistency {
            repeated_mechanisms: 2,
            ..Default::default()
        };
        let deviations = [
            HelpDeviation::Region {
                url: "https://a.com/".into(),
                mechanism: "tel:+4930123".into(),
                expected: vec![HelpRegion::Footer],
                found: vec![HelpRegion::Header, HelpRegion::Other],
            },
            HelpDeviation::Order {
                url: "https://a.com/".into(),
                region: HelpRegion::Footer,
                first: "a".into(),
                second: "b".into(),
            },
        ];
        for text in [
            consistent_help_basis(&help, true),
            consistent_help_basis(&HelpConsistency::default(), true),
            help_deviation_text(&deviations[0], true),
            help_deviation_text(&deviations[1], true),
        ] {
            assert!(!text.chars().any(|c| "äöüÄÖÜß".contains(c)), "{text}");
        }
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
