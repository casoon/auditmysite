use crate::output::localized::is_english;
use fluent_bundle::FluentValue;
use renderreport::components::advanced::{ChecklistPanel, ChecklistRow, KeyValueList, List};
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::audit::BatchReport;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::short_url;

// ─── Helper: Batch Report Assessment & Key Points ──────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BatchAuditFlagSummary {
    pub(super) kind: String,
    pub(super) affected_pages: usize,
    example: String,
}

pub(super) fn aggregate_audit_flags(
    reports: &[crate::audit::AuditReport],
) -> Vec<BatchAuditFlagSummary> {
    let mut by_kind: std::collections::BTreeMap<String, (usize, String)> =
        std::collections::BTreeMap::new();

    for report in reports {
        let normalized = crate::audit::normalized::normalize(report);
        let mut seen_on_page = std::collections::BTreeSet::new();
        for flag in &normalized.normalized.audit_flags {
            if !seen_on_page.insert(flag.kind.clone()) {
                continue;
            }
            by_kind
                .entry(flag.kind.clone())
                .and_modify(|(count, _)| *count += 1)
                .or_insert((1, flag.message.clone()));
        }
    }

    let mut summaries: Vec<_> = by_kind
        .into_iter()
        .map(|(kind, (affected_pages, example))| BatchAuditFlagSummary {
            kind,
            affected_pages,
            example,
        })
        .collect();
    summaries.sort_by(|a, b| {
        b.affected_pages
            .cmp(&a.affected_pages)
            .then_with(|| a.kind.cmp(&b.kind))
    });
    summaries
}

pub(super) fn audit_flag_batch_title(kind: &str, i18n: &I18n) -> String {
    i18n.t(match kind {
        "consent_banner" => "batch-management-flag-consent-banner",
        "bypass_blocks_untested" => "batch-management-flag-bypass-blocks-untested",
        "conflicting_signal" => "batch-management-flag-conflicting-signal",
        "viewport_gap" => "batch-management-flag-viewport-gap",
        "incomplete_audit" => "batch-management-flag-incomplete-audit",
        "consent_wall_artifact" => "batch-management-flag-consent-wall-artifact",
        _ => "batch-management-flag-other",
    })
}

pub(super) fn render_batch_audit_flags(
    builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let summaries = aggregate_audit_flags(&batch.reports);
    if summaries.is_empty() {
        return builder;
    }

    let mut rows = Vec::new();
    for summary in summaries {
        let pages = i18n.count("batch-management-flag-pages", summary.affected_pages);
        rows.push(
            ChecklistRow::new(
                audit_flag_batch_title(&summary.kind, i18n),
                format!("{} — {}", pages, summary.example),
            )
            .with_status("warn"),
        );
    }

    builder
        .add_component(ChecklistPanel::new(rows).with_title(i18n.t("batch-management-flags-title")))
}

/// Clear batch assessment — no score, just interpretation
pub(super) fn build_batch_assessment(
    summary: &crate::output::report_model::PortfolioSummary,
    dist: &SeverityDistribution,
    i18n: &I18n,
) -> String {
    i18n.t(batch_assessment_key(summary, dist))
}

pub(super) fn batch_assessment_key(
    summary: &crate::output::report_model::PortfolioSummary,
    dist: &SeverityDistribution,
) -> &'static str {
    let score = summary
        .average_score
        .expect(super::UNMEASURED_REFUSED)
        .round() as u32;
    // Score band is the primary signal; severity only refines wording within a
    // band. A low average must never yield a reassuring label (mirrors the
    // single-report logic in builder/single/executive.rs, #355).
    if score < 40 {
        "batch-management-assessment-critical"
    } else if score < 60 {
        if dist.critical > 0 {
            "batch-management-assessment-serious"
        } else {
            "batch-management-assessment-substantial-gaps"
        }
    } else if score < 75 {
        if dist.critical > 0 {
            "batch-management-assessment-usable-critical"
        } else if dist.high > 0 {
            "batch-management-assessment-usable-high"
        } else {
            "batch-management-assessment-needs-improvement"
        }
    } else if dist.critical > 0 {
        // From 75 up the severity refines the band label; above 90 too, where
        // "largely accessible" used to stand next to open high findings and a
        // failing verdict (#1039).
        "batch-management-assessment-stable-critical"
    } else if dist.high > 0 {
        // What the automated check found, not a conformance claim: the
        // rating note right above says the band is no statement of
        // conformance (#1039).
        "batch-management-assessment-good-high"
    } else if score < 90 {
        "batch-management-assessment-fine-tuning"
    } else {
        "batch-management-assessment-polish"
    }
}

/// The management risk dimensions of the batch: the same derivation the JSON
/// publishes as `summary.management_risks` (`audit::management_risk`), its
/// text re-derived in the run language. A second heuristic here graded legal
/// risk "medium" where the JSON said "high" and counted 8 recurring patterns
/// where the JSON counted 12 rules (#1039).
pub(super) fn render_batch_management_risks(
    builder: renderreport::engine::ReportBuilder,
    normalized_reports: &[crate::audit::normalized::NormalizedReport],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::audit::management_risk::RiskTier;
    let en = is_english(i18n);
    let mut kv = KeyValueList::new().with_title(i18n.t("batch-management-risks-title"));
    for kind in crate::audit::management_risk::build_management_risk_kinds(normalized_reports) {
        let tier = match kind.tier() {
            RiskTier::High => i18n.t("risk-level-high"),
            RiskTier::Medium => i18n.t("risk-level-medium"),
            RiskTier::Low => i18n.t("risk-level-low"),
            // A module that was not run is neither a risk nor a strength
            // (as in the single report).
            RiskTier::Unknown => continue,
        };
        kv = kv.add(
            kind.dimension(en),
            format!("{tier} — {}", kind.rationale(en)),
        );
    }
    builder.add_component(kv)
}

pub(super) fn render_batch_internal_comparison(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // The internal comparison and outlier detection only carry meaning across a
    // real sample. With one or two URLs "best vs weakest" is identical or
    // trivial, so we show a caveat instead of a misleading table (#450).
    if pres.url_details.len() < 3 {
        let note = i18n.t_args(
            "batch-management-comparison-too-few",
            &[("count", FluentValue::from(pres.url_details.len() as i64))],
        );
        return builder.add_component(Callout::info(note));
    }

    let mut rows = Vec::new();
    let mut module_names = std::collections::BTreeSet::new();
    for detail in &pres.url_details {
        for (module, _) in &detail.module_scores {
            module_names.insert(module.clone());
        }
    }
    for module in module_names {
        let mut scored: Vec<_> = pres
            .url_details
            .iter()
            .filter_map(|detail| {
                detail
                    .module_scores
                    .iter()
                    .find(|(name, _)| name == &module)
                    .map(|(_, score)| (detail.url.as_str(), *score))
            })
            .collect();
        if scored.is_empty() {
            continue;
        }
        scored.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));
        let label = super::portfolio::localized_module_name(&module, i18n);
        let (worst_url, worst_score) = scored[0];
        let (best_url, best_score) = scored[scored.len() - 1];
        // No variation: a best and a weakest URL would be an arbitrary pick.
        if best_score == worst_score {
            let same = i18n.t_args(
                "batch-management-comparison-same",
                &[
                    ("score", FluentValue::from(best_score as i64)),
                    ("count", FluentValue::from(scored.len() as i64)),
                ],
            );
            rows.push(vec![label, same, "—".to_string()]);
            continue;
        }
        let tied = |score: u32| scored.iter().filter(|(_, s)| *s == score).count();
        rows.push(vec![
            label,
            compared_url_text(best_url, best_score, tied(best_score), i18n),
            compared_url_text(worst_url, worst_score, tied(worst_score), i18n),
        ]);
    }
    if !rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-module")).with_width("20%"),
            TableColumn::new(i18n.t("batch-management-col-best-url")).with_width("40%"),
            TableColumn::new(i18n.t("batch-management-col-weakest-url")).with_width("40%"),
        ])
        .with_title(i18n.t("batch-management-comparison-title"));
        for row in rows {
            table = table.add_row(row);
        }
        builder = builder.add_component(table);
    }

    let avg = pres
        .portfolio_summary
        .average_score
        .expect(super::UNMEASURED_REFUSED)
        .round() as i32;
    let outliers: Vec<_> = pres
        .url_ranking
        .iter()
        .filter_map(|url| {
            let delta = url.score.round() as i32 - avg;
            (delta <= -15).then(|| {
                i18n.t_args(
                    "batch-management-outlier",
                    &[
                        ("url", FluentValue::from(short_url(&url.url, 70))),
                        ("score", FluentValue::from(url.score.round() as u32 as i64)),
                        ("delta", FluentValue::from(delta.abs() as i64)),
                    ],
                )
            })
        })
        .take(6)
        .collect();
    if !outliers.is_empty() {
        let mut list = List::new().with_title(i18n.t("batch-management-outliers-title"));
        for item in outliers {
            list = list.add_item(item);
        }
        builder = builder.add_component(list);
    }

    builder
}

/// A best or weakest URL of the module comparison. When other pages share
/// its score it says so: "agb (95/100)" read as the one weakest page where
/// four pages scored 95 (#1039).
pub(super) fn compared_url_text(
    url: &str,
    score: u32,
    pages_at_score: usize,
    i18n: &I18n,
) -> String {
    let url = short_url(url, 34);
    let named = if pages_at_score > 1 {
        i18n.t_args(
            "batch-dup-pages-more",
            &[
                ("list", fluent_bundle::FluentValue::from(url)),
                (
                    "count",
                    fluent_bundle::FluentValue::from((pages_at_score - 1) as i64),
                ),
            ],
        )
    } else {
        url
    };
    format!("{named} ({score}/100)")
}

/// The cause of a cross-page problem and how sure the report is about it. A
/// verified template cluster for the rule names the place and carries the
/// confirmed or likely claim; without one, recurrence is observed but a shared
/// component is not, and repetition within one page says nothing about other
/// pages.
pub(super) fn batch_cause_text(
    group: &FindingGroup,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> String {
    let cluster = pres
        .template_clusters
        .iter()
        .find(|c| c.rule_id == group.finding.rule_id && c.confidence != "open");
    if let Some(cluster) = cluster {
        return cluster.headline.clone();
    }
    if group.affected_urls.len() > 1 {
        i18n.t_args(
            "batch-management-root-recurring",
            &[("count", FluentValue::from(group.affected_urls.len() as i64))],
        )
    } else if group.finding.occurrence_count > 1 {
        i18n.t_args(
            "batch-management-root-one-page",
            &[(
                "count",
                FluentValue::from(group.finding.occurrence_count as i64),
            )],
        )
    } else {
        i18n.t("batch-management-root-page-specific")
    }
}

/// The score effect of fixing one problem on every page (#1169), the same
/// value as `top_actions[].score_gain.accessibility` in the JSON; a problem
/// that would not move the score, or whose gain is not quantified, gets no
/// line.
pub(super) fn batch_score_effect(group: &FindingGroup, i18n: &I18n) -> Option<String> {
    group
        .finding
        .expected_impact_kind
        .score_gain()
        .filter(|gain| gain.accessibility > 0)
        .map(|gain| {
            i18n.t_args(
                "batch-management-score-gain",
                &[("points", FluentValue::from(i64::from(gain.accessibility)))],
            )
        })
}

/// The URLs that carry at least one critical or high WCAG finding: the number
/// the "critical/high occurrences on N URLs" key point names. It was the count
/// of all audited URLs, so two failing pages of five read "on 5 URLs" (#1039).
fn urls_with_critical_or_high(pres: &BatchPresentation) -> i64 {
    pres.url_ranking
        .iter()
        .filter(|url| url.critical_violations > 0)
        .count() as i64
}

fn is_wcag_group(group: &FindingGroup) -> bool {
    !group.finding.wcag_criterion.is_empty()
}

fn batch_issue_point(key: &str, group: &FindingGroup, i18n: &I18n) -> String {
    i18n.t_args(
        key,
        &[
            ("title", FluentValue::from(group.title.as_str())),
            (
                "occurrences",
                FluentValue::from(group.finding.occurrence_count as i64),
            ),
            ("urls", FluentValue::from(group.affected_urls.len() as i64)),
        ],
    )
}

/// What the automated WCAG check found on the audited pages: the factual
/// headline of the batch status. A finding of another module (HTML, SEO,
/// security) never decides it (#1208).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BatchWcagHeadline {
    /// No WCAG violation on any audited page.
    NoneFound,
    /// WCAG violations on `affected` of the audited pages.
    SomePages { affected: usize },
    /// WCAG violations on every audited page.
    AllPages,
}

pub(super) fn batch_wcag_headline(pres: &BatchPresentation) -> BatchWcagHeadline {
    let affected: std::collections::BTreeSet<&String> = pres
        .top_issues
        .iter()
        .filter(|g| is_wcag_group(g) && g.finding.occurrence_count > 0)
        .flat_map(|g| g.affected_urls.iter())
        .collect();
    if affected.is_empty() {
        BatchWcagHeadline::NoneFound
    } else if affected.len() >= pres.portfolio_summary.total_urls {
        BatchWcagHeadline::AllPages
    } else {
        BatchWcagHeadline::SomePages {
            affected: affected.len(),
        }
    }
}

pub(super) fn batch_wcag_headline_text(
    headline: BatchWcagHeadline,
    total_urls: usize,
    i18n: &I18n,
) -> String {
    let total = FluentValue::from(total_urls as i64);
    match headline {
        BatchWcagHeadline::NoneFound => {
            i18n.t_args("batch-management-headline-none", &[("total", total)])
        }
        BatchWcagHeadline::AllPages => {
            i18n.t_args("batch-management-headline-all", &[("total", total)])
        }
        BatchWcagHeadline::SomePages { affected } => i18n.t_args(
            "batch-management-headline-some",
            &[
                ("total", total),
                ("affected", FluentValue::from(affected as i64)),
            ],
        ),
    }
}

/// A headline and an assessment label that cannot stand together: barriers
/// named although the headline says no WCAG violation was found (#1208).
/// `None` when the pair agrees.
#[cfg(test)]
pub(super) fn batch_headline_contradiction(
    headline: BatchWcagHeadline,
    assessment_key: &str,
) -> Option<&'static str> {
    let names_barriers = !matches!(
        assessment_key,
        "batch-management-assessment-fine-tuning" | "batch-management-assessment-polish"
    );
    (headline == BatchWcagHeadline::NoneFound && names_barriers)
        .then_some("the assessment names barriers although no WCAG violation was found")
}

/// 3 key takeaways for batch report
pub(super) fn build_batch_key_points(
    pres: &BatchPresentation,
    dist: &SeverityDistribution,
    legal_flags: usize,
    i18n: &I18n,
) -> Vec<String> {
    let mut points = Vec::with_capacity(3);

    // Point 1: critical/high WCAG occurrences, with the per-page finding
    // groups they form — two different counts, both named.
    let ch = dist.critical + dist.high;
    let occ = &pres.portfolio_summary.occurrence_distribution;
    let ch_occurrences = occ.critical + occ.high;
    if ch > 0 {
        points.push(i18n.t_args(
            "batch-management-point-severe",
            &[
                ("occurrences", FluentValue::from(ch_occurrences as i64)),
                ("urls", FluentValue::from(urls_with_critical_or_high(pres))),
                ("findings", FluentValue::from(ch as i64)),
            ],
        ));
    }

    // Point 2: the dominant WCAG problem. `top_issues` carries the findings of
    // every module, so its first entry may be an HTML or SEO finding that has
    // nothing to do with the accessibility verdict above it (#1208).
    if let Some(top) = pres.top_issues.iter().find(|g| is_wcag_group(g)) {
        points.push(batch_issue_point(
            "batch-management-point-main-issue",
            top,
            i18n,
        ));
    }

    // Point 2b: the most important finding of the other modules, on its own
    // line and named as such.
    if let Some(top) = pres.top_issues.iter().find(|g| !is_wcag_group(g)) {
        points.push(batch_issue_point(
            "batch-management-point-other-module-issue",
            top,
            i18n,
        ));
    }

    // Point 3: Legal status — from the legal flags the batch decision uses
    // (high or critical Level A/AA rules), as the single report does; keyed on
    // critical findings alone it read "no Level A/AA violations" next to a
    // high AA finding (#1039).
    points.push(i18n.t(if legal_flags > 0 {
        "batch-management-point-legal-flags"
    } else if dist.high > 0 {
        "batch-management-point-structural"
    } else {
        "batch-management-point-no-barriers"
    }));

    points
}
