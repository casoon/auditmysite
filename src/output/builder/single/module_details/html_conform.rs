use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
use crate::output::report_model::HtmlConformPresentation;

use super::{module_interpretation, normalized_module_score};

/// Collapses raw conformance findings into one row per *distinct* defect,
/// carrying an occurrence count and the first location as an example.
///
/// The score is charged per distinct defect (`html_conform::score_findings`),
/// so a table listing raw occurrences would contradict it — twelve identical
/// rows for one templated element read as twelve problems. Insertion order
/// is preserved (the checker emits findings in document order) and the cap
/// applies to defects, not occurrences, so twenty rows now mean twenty real
/// things to fix.
fn distinct_html_conform_rows(
    findings: &[crate::html_conform::HtmlConformFinding],
    severity_label: &impl Fn(&str) -> String,
    en: bool,
) -> Vec<(String, String, String, String, u32)> {
    let mut order: Vec<String> = Vec::new();
    let mut rows: std::collections::HashMap<String, (String, String, String, String, u32)> =
        std::collections::HashMap::new();

    for finding in findings {
        let key = crate::html_conform::defect_key(finding);
        match rows.get_mut(&key) {
            Some(row) => row.4 += 1,
            None => {
                order.push(key.clone());
                rows.insert(
                    key,
                    (
                        finding.rule_id.clone(),
                        severity_label(&finding.severity),
                        // Trailing " at line:column" stripped: it duplicates the
                        // Location column, and on a row standing for N
                        // occurrences it would name just one of them as if it
                        // were the whole finding.
                        // The run language replaces the checker's English
                        // prose (#902).
                        crate::html_conform::message::message_text(
                            &finding.rule_id,
                            finding
                                .message
                                .split(" at ")
                                .next()
                                .unwrap_or(&finding.message)
                                .trim(),
                            finding.detail.as_ref(),
                            en,
                        ),
                        finding.location.clone().unwrap_or_else(|| "—".to_string()),
                        1,
                    ),
                );
            }
        }
    }

    order
        .into_iter()
        .take(20)
        .filter_map(|key| rows.remove(&key))
        .collect()
}

pub(super) fn build_html_conform_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<HtmlConformPresentation> {
    let locale = i18n.locale();
    normalized.raw_html_conform.map(|hc| {
        let score =
            normalized_module_score(&normalized.normalized, "HTML Conformance").unwrap_or(hc.score);
        let severity_label = |severity: &str| match severity {
            "error" => i18n.t("severity-error"),
            "warning" => i18n.t("severity-warning"),
            _ => i18n.t("severity-info"),
        };
        let rows = distinct_html_conform_rows(&hc.findings, &severity_label, locale == "en");
        let error_label = i18n.t("severity-error");
        let warning_label = i18n.t("severity-warning");
        let severity_rank = |label: &str| -> u8 {
            if label == error_label {
                0
            } else if label == warning_label {
                1
            } else {
                2
            }
        };
        let mut ranked: Vec<&(String, String, String, String, u32)> = rows.iter().collect();
        ranked.sort_by(|a, b| {
            severity_rank(&a.1)
                .cmp(&severity_rank(&b.1))
                .then(b.4.cmp(&a.4))
        });
        let recommendations = ranked.into_iter().take(4).map(|r| r.2.clone()).collect();

        // One measure per rule family, at the family's highest severity.
        let mut measures: Vec<(String, crate::wcag::Severity, String)> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for finding in &hc.findings {
            let severity = match finding.severity.as_str() {
                "error" => crate::wcag::Severity::High,
                "warning" => crate::wcag::Severity::Medium,
                _ => crate::wcag::Severity::Low,
            };
            let (title, fix) =
                crate::html_conform::recommendation::recommendation(&finding.rule_id, i18n);
            match seen.iter().position(|t| *t == title) {
                Some(idx) => {
                    if severity_order(severity) < severity_order(measures[idx].1) {
                        measures[idx].1 = severity;
                    }
                }
                None => {
                    seen.push(title.clone());
                    measures.push((title, severity, fix));
                }
            }
        }
        measures.sort_by_key(|m| severity_order(m.1));

        HtmlConformPresentation {
            score,
            checked: hc.checked,
            interpretation: module_interpretation(&normalized.normalized, "html_conform", locale),
            error_count: hc.error_count,
            warning_count: hc.warning_count,
            info_count: hc.info_count,
            distinct_defect_count: hc.distinct_defect_count,
            findings: rows,
            recommendations,
            measures,
        }
    })
}

fn severity_order(severity: crate::wcag::Severity) -> u8 {
    use crate::wcag::Severity;
    match severity {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
    }
}

#[cfg(test)]
mod html_conform_recommendations_tests {
    use super::*;
    use crate::cli::WcagLevel;
    use crate::html_conform::{HtmlConformAnalysis, HtmlConformFinding};
    use crate::wcag::WcagResults;

    fn finding(rule_id: &str, severity: &str, message: &str) -> HtmlConformFinding {
        HtmlConformFinding {
            rule_id: rule_id.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            location: None,
            byte_offset: None,
            detail: None,
        }
    }

    #[test]
    fn recommendations_rank_errors_before_warnings_and_by_occurrence() {
        // plan/31-html-conform-missing-fix-guidance.md: the HTML Conformance
        // module must get a "Verbesserungsvorschläge" list like every other
        // risk-driver module — built from its own distinct-defect messages,
        // errors first, then by occurrence count, capped at 4.
        let mut report = crate::audit::AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.html_conform = Some(HtmlConformAnalysis {
            score: 64,
            checked: true,
            error_count: 3,
            warning_count: 1,
            info_count: 0,
            distinct_defect_count: 3,
            findings: vec![
                finding(
                    "assertion.headings.no-top-level",
                    "warning",
                    "This document has heading elements but none of them has a computed heading level",
                ),
                finding(
                    "assertion.elements.script-module-defer",
                    "error",
                    "A script element with type module must not have a defer attribute",
                ),
                finding(
                    "assertion.elements.script-module-defer",
                    "error",
                    "A script element with type module must not have a defer attribute",
                ),
                finding(
                    "assertion.elements.link-as-missing-rel",
                    "error",
                    "A link element with an as attribute must have a rel attribute",
                ),
            ],
            raw_html: None,
        });

        let ctx = crate::audit::normalized::normalize(&report);
        let i18n = crate::i18n::I18n::new("de").unwrap();
        let presentation = build_html_conform_details(&ctx, &i18n)
            .expect("html_conform data must produce a presentation");

        assert_eq!(
            presentation.recommendations[0], "Skript-Attribute und Skriptarten korrigieren",
            "the error with the most occurrences must rank first"
        );
        assert!(
            presentation
                .recommendations
                .contains(&"link-Elemente im Kopf korrigieren".to_string()),
            "the other distinct error must be included"
        );
        let warning_pos = presentation
            .recommendations
            .iter()
            .position(|r| r == "Überschriftenstruktur korrigieren")
            .expect("the warning must still be included when under the cap of 4");
        assert!(
            warning_pos > 0,
            "the warning must rank after the errors, got position {warning_pos}"
        );
    }

    /// #899: HTML conformance findings carry a written recommendation per rule
    /// family, so they enter the action plan and the key measures with a title
    /// and a concrete fix hint instead of being dropped.
    #[test]
    fn findings_enter_action_plan_and_measures_with_a_written_fix() {
        let mut report = crate::audit::AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.html_conform = Some(HtmlConformAnalysis {
            score: 70,
            checked: true,
            error_count: 3,
            warning_count: 0,
            info_count: 0,
            distinct_defect_count: 3,
            findings: vec![
                finding("assertion.elements.script-module-defer", "error", "m1"),
                finding("assertion.elements.script-module-nomodule", "error", "m2"),
                finding("assertion.ids.duplicate", "error", "m3"),
            ],
            raw_html: None,
        });
        let ctx = crate::audit::normalized::normalize(&report);
        for locale in ["de", "en"] {
            let i18n = crate::i18n::I18n::new(locale).unwrap();

            let groups: Vec<_> = ctx
                .normalized
                .findings
                .iter()
                .filter(|f| f.rule_id.starts_with("html_conform."))
                .map(|f| {
                    crate::output::builder::single::findings::finding_group_from_normalized(
                        &i18n, f,
                    )
                })
                .collect();
            assert_eq!(groups.len(), 3);
            assert!(groups.iter().all(|g| !g.recommendation.trim().is_empty()));
            let plan = crate::output::builder::actions::derive_action_plan(&i18n, &groups);
            let actions: Vec<_> = plan
                .quick_wins
                .iter()
                .chain(&plan.medium_term)
                .chain(&plan.structural)
                .collect();
            // Two script rules share one family and one action.
            assert_eq!(actions.len(), 2, "{locale}");
            // Title and hint are separate: the card title is the title only (#902).
            assert!(actions
                .iter()
                .all(|a| !a.action.contains(": ") && !a.hint.is_empty()));

            let presentation = build_html_conform_details(&ctx, &i18n).unwrap();
            assert_eq!(presentation.measures.len(), 2);
            assert!(presentation
                .measures
                .iter()
                .all(|(title, _, fix)| !title.is_empty() && !fix.is_empty()));
        }
    }

    #[test]
    fn recommendations_empty_when_no_findings() {
        let mut report = crate::audit::AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.html_conform = Some(HtmlConformAnalysis {
            score: 100,
            checked: true,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            distinct_defect_count: 0,
            findings: vec![],
            raw_html: None,
        });

        let ctx = crate::audit::normalized::normalize(&report);
        let i18n = crate::i18n::I18n::new("de").unwrap();
        let presentation = build_html_conform_details(&ctx, &i18n)
            .expect("html_conform data must produce a presentation");

        assert!(presentation.recommendations.is_empty());
    }
}
