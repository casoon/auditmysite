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
    /// The check ids the flags of this kind name, deduplicated and sorted;
    /// they only select the sentence, they are never printed.
    sources: Vec<String>,
}

pub(super) fn aggregate_audit_flags(
    reports: &[crate::audit::AuditReport],
) -> Vec<BatchAuditFlagSummary> {
    let mut by_kind: std::collections::BTreeMap<
        String,
        (usize, std::collections::BTreeSet<String>),
    > = std::collections::BTreeMap::new();

    for report in reports {
        let normalized = crate::audit::normalized::normalize(report);
        let mut seen_on_page = std::collections::BTreeSet::new();
        for flag in &normalized.normalized.audit_flags {
            let entry = by_kind.entry(flag.kind.clone()).or_default();
            entry.1.extend(flag.source.split(',').map(str::to_string));
            if seen_on_page.insert(flag.kind.clone()) {
                entry.0 += 1;
            }
        }
    }

    let mut summaries: Vec<_> = by_kind
        .into_iter()
        .map(|(kind, (affected_pages, sources))| BatchAuditFlagSummary {
            kind,
            affected_pages,
            sources: sources.into_iter().collect(),
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
        "side_check_unanswered" => "pdf-frame-flag-title-side-check-unanswered",
        _ => "batch-management-flag-other",
    })
}

/// The plain sentence of an audit caveat in the run language: what it means
/// for reading the scores. The check ids of the JSON (`audit_flags[].source`)
/// only select which checks are named, by their plain names (#1213).
pub(super) fn audit_flag_batch_text(summary: &BatchAuditFlagSummary, i18n: &I18n) -> String {
    match summary.kind.as_str() {
        "consent_banner" => i18n.t("pdf-frame-flag-text-consent-banner"),
        "bypass_blocks_untested" => i18n.t("pdf-frame-flag-text-bypass-blocks-untested"),
        "conflicting_signal" => i18n.t("pdf-frame-flag-text-conflicting-signal"),
        "viewport_gap" => i18n.t("pdf-frame-flag-text-viewport-gap"),
        "incomplete_audit" => i18n.t("pdf-frame-flag-text-incomplete-audit"),
        "consent_wall_artifact" => i18n.t("batch-flag-text-consent-wall-artifact"),
        "side_check_unanswered" => {
            super::super::side_check_unanswered_text(&summary.sources.join(","), i18n)
        }
        _ => i18n.t("batch-flag-text-other"),
    }
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
                format!("{} — {}", pages, audit_flag_batch_text(&summary, i18n)),
            )
            .with_status("warn"),
        );
    }

    builder
        .add_component(ChecklistPanel::new(rows).with_title(i18n.t("batch-management-flags-title")))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// #1213: an audit caveat of a batch is one plain sentence in the run
    /// language; the check ids of the JSON and the English message the
    /// analysis stores never reach the German report.
    #[test]
    fn audit_caveats_are_plain_sentences_without_check_ids() {
        let summary = BatchAuditFlagSummary {
            kind: "side_check_unanswered".to_string(),
            affected_pages: 2,
            sources: vec![
                "security.certificate".to_string(),
                "seo.www_redirect".to_string(),
            ],
        };
        for locale in ["de", "en"] {
            let i18n = I18n::new(locale).unwrap();
            let title = audit_flag_batch_title(&summary.kind, &i18n);
            let text = audit_flag_batch_text(&summary, &i18n);
            for leaked in [
                "seo.www_redirect",
                "security.certificate",
                "Side checks without an answer",
            ] {
                assert!(!text.contains(leaked), "{locale}: {text}");
                assert!(!title.contains(leaked), "{locale}: {title}");
            }
            assert!(
                !text.contains("batch-") && !text.contains("pdf-frame-"),
                "{text}"
            );
        }
        let de = I18n::new("de").unwrap();
        let text = audit_flag_batch_text(&summary, &de);
        assert!(text.contains("Zertifikatsdetails"), "{text}");
        assert!(text.contains("www-/non-www"), "{text}");
        assert_eq!(
            audit_flag_batch_title("side_check_unanswered", &de),
            "Zusatzprüfung ohne Antwort"
        );
        // Every kind has a sentence of its own in both languages, and a kind
        // the report does not know falls back to a plain note, not a key.
        for kind in [
            "consent_banner",
            "bypass_blocks_untested",
            "conflicting_signal",
            "viewport_gap",
            "incomplete_audit",
            "consent_wall_artifact",
            "something_new",
        ] {
            let summary = BatchAuditFlagSummary {
                kind: kind.to_string(),
                affected_pages: 1,
                sources: vec![],
            };
            for locale in ["de", "en"] {
                let i18n = I18n::new(locale).unwrap();
                let text = audit_flag_batch_text(&summary, &i18n);
                assert!(
                    !text.is_empty()
                        && !text.starts_with("batch-")
                        && !text.starts_with("pdf-frame-"),
                    "{locale} {kind}: {text}"
                );
            }
        }
    }
}
