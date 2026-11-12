use crate::output::localized::is_english;
use fluent_bundle::FluentValue;
use renderreport::components::advanced::{ChecklistPanel, ChecklistRow, KeyValueList, List};
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::audit::BatchReport;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::short_url;

use super::super::helpers::severity_label_i18n;

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
    let score = summary
        .average_score
        .expect(super::UNMEASURED_REFUSED)
        .round() as u32;
    // Score band is the primary signal; severity only refines wording within a
    // band. A low average must never yield a reassuring label (mirrors the
    // single-report logic in builder/single/executive.rs, #355).
    let key = if score < 40 {
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
    };
    i18n.t(key)
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
    let en = is_english(i18n);

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
        let label = super::portfolio::localized_module_name(&module, en).to_string();
        let (worst_url, worst_score) = scored[0];
        let (best_url, best_score) = scored[scored.len() - 1];
        // No variation: a best and a weakest URL would be an arbitrary pick
        // (plan 86).
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

pub(super) fn render_batch_decision_actions(
    builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("batch-management-col-action")).with_width("44%"),
        TableColumn::new(i18n.t("batch-col-risk")).with_width("12%"),
        // The WCAG level instead of the expected-impact sentence: since plan
        // 99 that sentence only repeated the reach column and "score effect
        // not quantified" on every row (plan 108).
        TableColumn::new("WCAG").with_width("10%"),
        TableColumn::new(i18n.t("batch-management-col-scope")).with_width("14%"),
        TableColumn::new(i18n.t("batch-management-col-reach")).with_width("20%"),
    ])
    .with_title(i18n.t("batch-management-actions-title"));

    let mut count = 0;
    for group in pres
        .top_issues
        .iter()
        .take(crate::audit::normalized::BATCH_TOP_ACTION_COUNT)
    {
        // A verified template cluster for this rule overrides the loose
        // occurrence-count heuristic with the evidence-backed claim (and its
        // confirmed/likely distinction) instead of the vague "likely shared
        // component/template" label.
        let cluster = pres
            .template_clusters
            .iter()
            .find(|c| c.rule_id == group.finding.rule_id);
        let root: String = if let Some(cluster) = cluster {
            // Name the place: the element of the rule's most widespread
            // pattern. The full CSS path stays in the template list; a long
            // path in this narrow column broke the layout before (#518).
            format!(
                "{}: {}",
                selector_tail(&cluster.selector),
                cluster.decision_label
            )
        } else if group.affected_urls.len() > 1 {
            // Recurring is observed; a shared component is not (plan 85).
            i18n.t_args(
                "batch-management-root-recurring",
                &[("count", FluentValue::from(group.affected_urls.len() as i64))],
            )
        } else if group.finding.occurrence_count > 1 {
            // Repetition within one page says nothing about other pages
            // ("auf 1 Seiten, Ursache offen", plan 99).
            i18n.t_args(
                "batch-management-root-one-page",
                &[(
                    "count",
                    FluentValue::from(group.finding.occurrence_count as i64),
                )],
            )
        } else {
            i18n.t("batch-management-root-page-specific")
        };
        table = table.add_row(vec![
            format!("{} — {}", group.title, root),
            severity_label_i18n(group.finding.severity, i18n),
            if group.finding.wcag_level.is_empty() {
                "–".to_string()
            } else {
                group.finding.wcag_level.clone()
            },
            // The column shows the scope derived from the data, the same value
            // as `top_actions[].complexity` in the JSON — not the per-rule
            // effort estimate, which read "low" where the JSON said "high".
            change_scope_label(&group.finding.complexity, i18n),
            i18n.t_args(
                "batch-management-reach",
                &[
                    (
                        "occurrences",
                        FluentValue::from(group.finding.occurrence_count as i64),
                    ),
                    ("urls", FluentValue::from(group.affected_urls.len() as i64)),
                ],
            ),
        ]);
        count += 1;
    }
    if count == 0 {
        return builder;
    }
    builder.add_component(table)
}

/// The last compound of a CSS selector — the element itself:
/// `nav > div.menu a.mainnav-toggle` → `a.mainnav-toggle`.
fn selector_tail(selector: &str) -> &str {
    selector
        .rsplit(|c: char| c.is_whitespace() || c == '>' || c == '+' || c == '~')
        .find(|part| !part.is_empty())
        .unwrap_or(selector)
}

fn change_scope_label(complexity: &str, i18n: &I18n) -> String {
    i18n.t(match complexity {
        "high" => "batch-management-scope-high",
        "medium" => "batch-management-scope-medium",
        _ => "batch-management-scope-low",
    })
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
    // groups they form — two different counts, both named (plan 79).
    let ch = dist.critical + dist.high;
    let occ = &pres.portfolio_summary.occurrence_distribution;
    let ch_occurrences = occ.critical + occ.high;
    if ch > 0 {
        points.push(i18n.t_args(
            "batch-management-point-severe",
            &[
                ("occurrences", FluentValue::from(ch_occurrences as i64)),
                (
                    "urls",
                    FluentValue::from(pres.portfolio_summary.total_urls as i64),
                ),
                ("findings", FluentValue::from(ch as i64)),
            ],
        ));
    }

    // Point 2: Dominant/recurring issue
    if let Some(top) = pres.top_issues.first() {
        points.push(i18n.t_args(
            "batch-management-point-main-issue",
            &[
                ("title", FluentValue::from(top.title.as_str())),
                (
                    "occurrences",
                    FluentValue::from(top.finding.occurrence_count as i64),
                ),
                ("urls", FluentValue::from(top.affected_urls.len() as i64)),
            ],
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

#[cfg(test)]
mod tests {
    use super::selector_tail;

    #[test]
    fn selector_tail_names_the_element() {
        assert_eq!(
            selector_tail("nav > div.menu a.mainnav-toggle"),
            "a.mainnav-toggle"
        );
        assert_eq!(selector_tail("a#skip"), "a#skip");
        assert_eq!(selector_tail("ul > li + li"), "li");
    }
}
