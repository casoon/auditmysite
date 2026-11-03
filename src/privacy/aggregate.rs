//! Cross-page roll-up of privacy findings for batch reports (#872).
//!
//! Pure data: counts which pages show which indicator and which providers
//! recur. No text, no legal verdict, no score. Denominators are the pages
//! whose first visit was measured; a page whose capture failed is counted as
//! not measured and never as clean.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::findings::{
    CheckOutcome, PolicyDocument, PrivacyAnalysis, PrivacyCheck, PrivacyFindingKind, ProviderRef,
};
use crate::audit::collection::CollectionState;

/// How widely an indicator or provider shows up among the measured pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Spread {
    /// Every measured page (and there are at least two).
    AllPages,
    /// Two or more pages, not all.
    SeveralPages,
    SinglePage,
}

fn spread(pages: usize, measured: usize) -> Spread {
    if pages == measured && measured >= 2 {
        Spread::AllPages
    } else if pages >= 2 {
        Spread::SeveralPages
    } else {
        Spread::SinglePage
    }
}

/// One indicator across the measured pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KindPrevalence {
    pub kind: PrivacyFindingKind,
    /// Only for the policy-link kinds.
    pub document: Option<PolicyDocument>,
    pub pages: usize,
    pub spread: Spread,
}

/// One provider across the measured pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderPrevalence {
    pub provider: ProviderRef,
    /// Pages on which the provider was contacted.
    pub pages: usize,
    /// Requests to the provider, summed over the pages.
    pub requests: usize,
    pub spread: Spread,
}

/// How often a check ended in each outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckDistribution {
    pub check: PrivacyCheck,
    pub found: usize,
    pub not_found: usize,
    pub not_applicable: usize,
    pub not_measured: usize,
}

/// One cell of the per-page matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MatrixCell {
    pub outcome: CheckOutcome,
    /// Findings of the check on this page.
    pub findings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatrixRow {
    pub url: String,
    /// Whether the first visit was measured at all.
    pub measured: bool,
    /// In [`PrivacyCheck::ALL`] order.
    pub cells: Vec<MatrixCell>,
    pub findings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrivacyBatchAggregate {
    /// Pages that carry a privacy analysis (measured or not).
    pub pages_with_data: usize,
    /// First visit measured (fully or partly).
    pub pages_measured: usize,
    pub pages_not_measured: usize,
    /// Measured pages with at least one finding.
    pub pages_with_findings: usize,
    /// Sorted by affected pages, then kind.
    pub kinds: Vec<KindPrevalence>,
    /// Sorted by pages, requests, id.
    pub providers: Vec<ProviderPrevalence>,
    /// In [`PrivacyCheck::ALL`] order, over the measured pages.
    pub checks: Vec<CheckDistribution>,
    /// Sorted by findings (most first), then URL.
    pub matrix: Vec<MatrixRow>,
}

/// Finding kinds a check produces.
fn check_kinds(check: PrivacyCheck) -> &'static [PrivacyFindingKind] {
    use PrivacyFindingKind as K;
    match check {
        PrivacyCheck::TrackingRequests => &[K::TrackingRequestBeforeConsent],
        PrivacyCheck::NonEssentialStorage => &[K::NonEssentialStorageBeforeConsent],
        PrivacyCheck::VisitorIpTransfer => &[K::VisitorIpTransfer],
        PrivacyCheck::ThirdCountryProvider => &[K::ThirdCountryProvider],
        PrivacyCheck::ConsentPlatform => &[K::NoConsentPlatformDetected],
        PrivacyCheck::PolicyLinks => &[K::PolicyLinkMissing, K::PolicyLinkUnusable],
    }
}

fn is_measured(a: &PrivacyAnalysis) -> bool {
    matches!(
        a.measurement.state,
        CollectionState::Measured | CollectionState::Empty | CollectionState::Partial
    )
}

/// Roll-up over `(url, analysis)` pairs. `None` when no page has data.
pub fn aggregate_privacy<'a>(
    pages: impl IntoIterator<Item = (&'a str, &'a PrivacyAnalysis)>,
) -> Option<PrivacyBatchAggregate> {
    let pages: Vec<(&str, &PrivacyAnalysis)> = pages.into_iter().collect();
    if pages.is_empty() {
        return None;
    }
    let measured_pages: Vec<&(&str, &PrivacyAnalysis)> =
        pages.iter().filter(|(_, a)| is_measured(a)).collect();
    let measured = measured_pages.len();

    let mut kind_pages: BTreeMap<(PrivacyFindingKind, Option<u8>), usize> = BTreeMap::new();
    let mut provider_pages: BTreeMap<String, (ProviderRef, usize, usize)> = BTreeMap::new();
    let mut pages_with_findings = 0usize;
    for (_, a) in &measured_pages {
        if !a.findings.is_empty() {
            pages_with_findings += 1;
        }
        let kinds: BTreeSet<_> = a
            .findings
            .iter()
            .map(|f| (f.kind, f.document.map(doc_rank)))
            .collect();
        for key in kinds {
            *kind_pages.entry(key).or_default() += 1;
        }
        for c in &a.contacts {
            let e = provider_pages
                .entry(c.provider.id.clone())
                .or_insert_with(|| (c.provider.clone(), 0, 0));
            e.1 += 1;
            e.2 += c.request_count;
        }
    }

    let mut kinds: Vec<KindPrevalence> = kind_pages
        .into_iter()
        .map(|((kind, document), pages)| KindPrevalence {
            kind,
            document: document.map(rank_doc),
            pages,
            spread: spread(pages, measured),
        })
        .collect();
    kinds.sort_by(|a, b| {
        b.pages
            .cmp(&a.pages)
            .then(a.kind.cmp(&b.kind))
            .then(doc_key(a.document).cmp(&doc_key(b.document)))
    });

    let mut providers: Vec<ProviderPrevalence> = provider_pages
        .into_values()
        .map(|(provider, pages, requests)| ProviderPrevalence {
            provider,
            pages,
            requests,
            spread: spread(pages, measured),
        })
        .collect();
    providers.sort_by(|a, b| {
        b.pages
            .cmp(&a.pages)
            .then(b.requests.cmp(&a.requests))
            .then(a.provider.id.cmp(&b.provider.id))
    });

    let checks = PrivacyCheck::ALL
        .iter()
        .map(|&check| {
            let mut d = CheckDistribution {
                check,
                found: 0,
                not_found: 0,
                not_applicable: 0,
                not_measured: 0,
            };
            for (_, a) in &measured_pages {
                match outcome_of(a, check) {
                    CheckOutcome::Found => d.found += 1,
                    CheckOutcome::NotFound => d.not_found += 1,
                    CheckOutcome::NotApplicable => d.not_applicable += 1,
                    CheckOutcome::NotMeasured => d.not_measured += 1,
                }
            }
            d
        })
        .collect();

    let mut matrix: Vec<MatrixRow> = pages
        .iter()
        .map(|(url, a)| MatrixRow {
            url: (*url).to_string(),
            measured: is_measured(a),
            cells: PrivacyCheck::ALL
                .iter()
                .map(|&check| MatrixCell {
                    outcome: outcome_of(a, check),
                    findings: a
                        .findings
                        .iter()
                        .filter(|f| check_kinds(check).contains(&f.kind))
                        .count(),
                })
                .collect(),
            findings: a.findings.len(),
        })
        .collect();
    matrix.sort_by(|a, b| b.findings.cmp(&a.findings).then(a.url.cmp(&b.url)));

    Some(PrivacyBatchAggregate {
        pages_with_data: pages.len(),
        pages_measured: measured,
        pages_not_measured: pages.len() - measured,
        pages_with_findings,
        kinds,
        providers,
        checks,
        matrix,
    })
}

/// Outcome of a check; a missing result reads `NotMeasured`, never clean.
fn outcome_of(a: &PrivacyAnalysis, check: PrivacyCheck) -> CheckOutcome {
    a.checks
        .iter()
        .find(|r| r.check == check)
        .map_or(CheckOutcome::NotMeasured, |r| r.outcome)
}

fn doc_rank(d: PolicyDocument) -> u8 {
    match d {
        PolicyDocument::Privacy => 0,
        PolicyDocument::Imprint => 1,
    }
}

fn rank_doc(r: u8) -> PolicyDocument {
    if r == 0 {
        PolicyDocument::Privacy
    } else {
        PolicyDocument::Imprint
    }
}

fn doc_key(d: Option<PolicyDocument>) -> u8 {
    d.map_or(2, doc_rank)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::privacy::fixtures;

    #[test]
    fn no_pages_yield_no_aggregate() {
        assert!(aggregate_privacy(std::iter::empty()).is_none());
    }

    #[test]
    fn counts_pages_per_kind_and_provider() {
        let tracked = fixtures::tracked();
        let clean = fixtures::clean();
        let agg = aggregate_privacy([
            ("https://a.example/", &tracked),
            ("https://a.example/b", &tracked),
            ("https://a.example/c", &clean),
        ])
        .expect("aggregate");
        assert_eq!(agg.pages_measured, 3);
        assert_eq!(agg.pages_with_findings, 2);

        let tracking = agg
            .kinds
            .iter()
            .find(|k| k.kind == PrivacyFindingKind::TrackingRequestBeforeConsent)
            .expect("tracking kind");
        assert_eq!(tracking.pages, 2);
        assert_eq!(tracking.spread, Spread::SeveralPages);

        let ga = agg
            .providers
            .iter()
            .find(|p| p.provider.id == "google-analytics")
            .expect("provider");
        assert_eq!(ga.pages, 2);
        assert_eq!(ga.requests, 6);
    }

    #[test]
    fn provider_on_every_measured_page_is_all_pages() {
        let tracked = fixtures::tracked();
        let agg = aggregate_privacy([
            ("https://a.example/", &tracked),
            ("https://a.example/b", &tracked),
        ])
        .expect("aggregate");
        assert!(agg.kinds.iter().all(|k| k.spread == Spread::AllPages));
    }

    #[test]
    fn single_measured_page_is_never_all_pages() {
        let tracked = fixtures::tracked();
        let agg = aggregate_privacy([("https://a.example/", &tracked)]).expect("aggregate");
        assert!(agg.kinds.iter().all(|k| k.spread == Spread::SinglePage));
    }

    #[test]
    fn failed_capture_is_not_measured_and_not_counted_as_clean() {
        let failed = fixtures::not_measured();
        let clean = fixtures::clean();
        let agg = aggregate_privacy([
            ("https://a.example/x", &failed),
            ("https://a.example/y", &clean),
        ])
        .expect("aggregate");
        assert_eq!(agg.pages_measured, 1);
        assert_eq!(agg.pages_not_measured, 1);
        let row = agg
            .matrix
            .iter()
            .find(|r| r.url.ends_with("/x"))
            .expect("row");
        assert!(!row.measured);
        assert!(row
            .cells
            .iter()
            .all(|c| c.outcome == CheckOutcome::NotMeasured));
        // The failed page is outside every denominator.
        assert!(agg.checks.iter().all(|c| c.found
            + c.not_found
            + c.not_applicable
            + c.not_measured
            == 1));
    }

    #[test]
    fn matrix_lists_pages_with_most_findings_first() {
        let tracked = fixtures::tracked();
        let clean = fixtures::clean();
        let agg = aggregate_privacy([
            ("https://a.example/clean", &clean),
            ("https://a.example/t", &tracked),
        ])
        .expect("aggregate");
        assert_eq!(agg.matrix[0].url, "https://a.example/t");
        assert_eq!(agg.matrix[0].findings, tracked.findings.len());
    }
}
