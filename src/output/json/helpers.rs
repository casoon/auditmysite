use super::*;

pub(super) fn build_wcag_coverage_summary(normalized: &NormalizedReport) -> WcagCoverageSummary {
    build_wcag_coverage_for_level(&normalized.wcag_level.to_string())
}

fn en301549_status_kind(
    status: crate::wcag::en301549::ClauseStatus,
) -> crate::output::json::En301549ClauseStatusKind {
    use crate::output::json::En301549ClauseStatusKind as Kind;
    use crate::wcag::en301549::ClauseStatus;
    match status {
        ClauseStatus::ViolationsFound => Kind::ViolationsFound,
        ClauseStatus::NoViolationsAutomated => Kind::NoViolationsAutomated,
        ClauseStatus::ManualReviewRequired => Kind::ManualReviewRequired,
    }
}

/// Build the per-page EN 301 549 annex — always present, single AND batch
/// (same rule as `fix_guidance`, issue #256). Pure projection over the
/// page's own findings; nothing stored in `NormalizedReport`.
pub(super) fn build_en301549_annex(
    normalized: &NormalizedReport,
) -> crate::output::json::En301549Annex {
    use crate::output::json::{
        En301549Annex, En301549ClauseEntry, En301549FindingRef, OutOfScopeChapterEntry,
    };
    use crate::wcag::en301549::{
        derive_annex, out_of_standard_finding_count, EN301549_DISCLAIMER_EN,
        EN301549_MAPPING_VERSION, EN301549_VERSION, OUT_OF_SCOPE_CHAPTERS,
    };

    let clauses = derive_annex(&normalized.findings)
        .into_iter()
        .map(|r| {
            let failed = normalized.rule_outcomes.iter().any(|outcome| {
                outcome.wcag.iter().any(|c| c == r.clause.wcag)
                    && crate::wcag::rule_run_errored(outcome)
            });
            let completed = normalized.rule_outcomes.iter().any(|outcome| {
                outcome.wcag.iter().any(|c| c == r.clause.wcag)
                    && !crate::wcag::rule_run_errored(outcome)
            });
            let base = en301549_status_kind(r.status);
            let status = if base == crate::output::json::En301549ClauseStatusKind::ViolationsFound {
                base
            } else if failed && completed {
                crate::output::json::En301549ClauseStatusKind::PartialAutomatedEvaluation
            } else if failed {
                crate::output::json::En301549ClauseStatusKind::NotEvaluated
            } else {
                base
            };
            En301549ClauseEntry {
                en_clause: r.clause.en_clause,
                wcag: r.clause.wcag,
                level: r.clause.wcag_level,
                title: r.clause.title_en,
                status,
                findings: r
                    .findings
                    .into_iter()
                    .map(|f| En301549FindingRef {
                        rule_id: f.rule_id,
                        occurrences: f.occurrences,
                    })
                    .collect(),
            }
        })
        .collect();

    En301549Annex {
        standard_version: EN301549_VERSION,
        mapping_version: EN301549_MAPPING_VERSION,
        scope: "web_chapter_9_automated_subset",
        clauses,
        out_of_standard_findings: out_of_standard_finding_count(&normalized.findings),
        out_of_scope_chapters: OUT_OF_SCOPE_CHAPTERS
            .iter()
            .map(|c| OutOfScopeChapterEntry {
                chapter: c.chapter,
                title: c.title_en,
            })
            .collect(),
        disclaimer: EN301549_DISCLAIMER_EN.to_string(),
    }
}

/// Batch-only domain-wide roll-up: worst status per clause across all pages,
/// plus the count of pages with a confirmed violation for it. Built next to
/// `build_management_risks`' `legal_flags` aggregation below, since both fold
/// over the same `&[NormalizedReport]` batch slice. Thin wrapper over
/// `wcag::en301549::derive_batch_rollup`, shared with the batch PDF annex.
pub(super) fn build_en301549_batch_rollup(
    reports: &[NormalizedReport],
) -> Vec<crate::output::json::En301549BatchClauseRollup> {
    use crate::output::json::En301549BatchClauseRollup;

    crate::wcag::en301549::derive_batch_rollup(reports.iter().map(|r| r.findings.as_slice()))
        .into_iter()
        .map(|r| {
            let failed = reports.iter().any(|report| {
                report.rule_outcomes.iter().any(|outcome| {
                    outcome.wcag.iter().any(|c| c == r.clause.wcag)
                        && crate::wcag::rule_run_errored(outcome)
                })
            });
            let completed = reports.iter().any(|report| {
                report.rule_outcomes.iter().any(|outcome| {
                    outcome.wcag.iter().any(|c| c == r.clause.wcag)
                        && !crate::wcag::rule_run_errored(outcome)
                })
            });
            let base = en301549_status_kind(r.status);
            let status = if base == crate::output::json::En301549ClauseStatusKind::ViolationsFound {
                base
            } else if failed && completed {
                crate::output::json::En301549ClauseStatusKind::PartialAutomatedEvaluation
            } else if failed {
                crate::output::json::En301549ClauseStatusKind::NotEvaluated
            } else {
                base
            };
            En301549BatchClauseRollup {
                en_clause: r.clause.en_clause,
                wcag: r.clause.wcag,
                level: r.clause.wcag_level,
                title: r.clause.title_en,
                status,
                affected_pages: r.affected_pages,
            }
        })
        .collect()
}

fn bik_status_kind(
    status: crate::wcag::bik_guide::BikChapterStatus,
) -> crate::output::json::BikChapterStatusKind {
    use crate::output::json::BikChapterStatusKind as Kind;
    use crate::wcag::bik_guide::BikChapterStatus;
    match status {
        BikChapterStatus::FindingsPresent => Kind::FindingsPresent,
        BikChapterStatus::NoFindingsDetected => Kind::NoFindingsDetected,
        BikChapterStatus::NotChecked => Kind::NotChecked,
    }
}

fn bik_chapter_id_str(id: crate::wcag::bik_guide::BikChapterId) -> &'static str {
    use crate::wcag::bik_guide::BikChapterId;
    match id {
        BikChapterId::Images => "images",
        BikChapterId::LinkText => "link_text",
        BikChapterId::Structure => "structure",
        BikChapterId::EasyLanguage => "easy_language",
        BikChapterId::Pdf => "pdf",
        BikChapterId::Videos => "videos",
    }
}

/// Build the per-page "BIK für Alle" guide mapping — always present, single
/// AND batch (same rule as `fix_guidance`/`en301549_annex`). Pure projection;
/// nothing stored in `NormalizedReport`. `design_quality_findings`/
/// `seo_technical_issues`/`easy_language_detected` are only available from a
/// live single-report audit's raw module data — callers without it (cached,
/// batch) pass empty slices / `false`, see `wcag::bik_guide` module doc.
pub(super) fn build_bik_guide_annex(
    normalized: &NormalizedReport,
    design_quality_findings: &[crate::design_quality::DesignQualityFinding],
    seo_technical_issues: &[crate::seo::technical::TechnicalIssue],
    easy_language_detected: bool,
) -> crate::output::json::BikGuideAnnex {
    use crate::output::json::{BikChapterEntry, BikFindingRefEntry, BikGuideAnnex};
    use crate::wcag::bik_guide::{derive_bik_chapters, BIK_GUIDE_SOURCE};

    let screen_reader_issues: &[crate::screen_reader::SrAuditIssue] = normalized
        .screen_reader
        .as_ref()
        .map(|sr| sr.issues.as_slice())
        .unwrap_or(&[]);
    let chapters = derive_bik_chapters(
        &normalized.findings,
        &normalized.interactive_findings,
        screen_reader_issues,
        design_quality_findings,
        seo_technical_issues,
        easy_language_detected,
        &normalized.accessibility_assessments,
    )
    .into_iter()
    .map(|r| BikChapterEntry {
        id: bik_chapter_id_str(r.chapter.id),
        title: r.chapter.title_en,
        status: bik_status_kind(r.status),
        findings: r
            .findings
            .into_iter()
            .map(|f| BikFindingRefEntry {
                label: f.label,
                occurrences: f.occurrences,
            })
            .collect(),
    })
    .collect();

    BikGuideAnnex {
        guide: BIK_GUIDE_SOURCE,
        chapters,
    }
}

pub(super) fn build_wcag_coverage_for_level(level: &str) -> WcagCoverageSummary {
    let (automated, total) = crate::wcag::coverage::coverage_stats();
    WcagCoverageSummary {
        level: format!("WCAG 2.1 {level}"),
        automated_criteria: automated,
        manual_review_criteria: crate::wcag::coverage::manual_review_criteria().len(),
        total_wcag_aa_criteria: total,
        note: "Automated score covers detectable criteria only; context-dependent WCAG criteria require manual review.".to_string(),
    }
}

pub(super) fn build_accessibility_score_breakdown(
    reports: &[NormalizedReport],
) -> Vec<AccessibilityScoreComponent> {
    const AREAS: [(&str, u32); 8] = [
        ("Semantics", 15),
        ("Forms", 15),
        ("Keyboard", 15),
        ("Focus management", 10),
        ("Images / alternative text", 15),
        ("ARIA", 15),
        ("Heading structure", 8),
        ("Landmarks / page structure", 7),
    ];

    AREAS
        .iter()
        .map(|(area, weight_pct)| {
            // Logarithmic occurrence penalty with a soft floor (#485). The old
            // linear `severity_weight * occurrence_count` saturated at 100 after a
            // handful of findings, collapsing whole areas to 0 on large, mostly
            // compliant pages (e.g. gov.uk Forms). Each additional occurrence now
            // contributes progressively less, and the per-area loss is capped below
            // 100 so areas stay diagnostic — 1 vs. 100 violations remain
            // distinguishable instead of all flatlining at 0.
            let mut penalty = 0f64;
            let mut driver: Option<(&str, usize)> = None;

            for finding in reports.iter().flat_map(|report| report.findings.iter()) {
                if score_area_for_finding(finding) != *area {
                    continue;
                }
                let severity_weight = match finding.severity {
                    crate::taxonomy::Severity::Critical => 20.0,
                    crate::taxonomy::Severity::High => 14.0,
                    crate::taxonomy::Severity::Medium => 8.0,
                    crate::taxonomy::Severity::Low => 4.0,
                };
                let occ = finding.occurrence_count.max(1) as f64;
                penalty += severity_weight * (1.0 + occ.ln());
                if driver
                    .map(|(_, count)| finding.occurrence_count > count)
                    .unwrap_or(true)
                {
                    driver = Some((&finding.title, finding.occurrence_count));
                }
            }

            // Soft cap at 90: the worst areas floor at a score of 10 rather than 0.
            let estimated_lost_points = (penalty.round() as u32).min(90);
            AccessibilityScoreComponent {
                area: (*area).to_string(),
                score: 100u32.saturating_sub(estimated_lost_points),
                weight_pct: *weight_pct,
                estimated_lost_points,
                main_driver: driver
                    .map(|(title, count)| format!("{title} ({count} occurrences)"))
                    .unwrap_or_else(|| "No detected driver".to_string()),
            }
        })
        .collect()
}

/// Whether `key` contains `prefix` as the start of a "word" (a run of
/// alphanumeric characters delimited by any non-alphanumeric character, or
/// by the string boundary) rather than merely as a substring anywhere.
/// Prevents e.g. "conformance" (contains "form" mid-word) or "querformat"
/// (contains "form" mid-word) from being misread as a forms-related match,
/// while still matching legitimate plurals/compounds like "forms",
/// "landmarks", "inputs" that genuinely start with the needle (see
/// score-area-substring-misclassification in the regression corpus).
pub(crate) fn key_has_word_starting_with(key: &str, prefix: &str) -> bool {
    key.split(|c: char| !c.is_alphanumeric())
        .any(|word| word.starts_with(prefix))
}

/// Removes CSS-selector-shaped spans (a `.class`/`#id` chain glued directly
/// onto a preceding word, e.g. "div.alt-service-hero-card" or
/// "a.button.success") from `text`. Several WCAG rules embed the raw
/// `affected_selectors` CSS selector directly into their finding message
/// (e.g. `text_spacing.rs`'s "Content is clipped by '{selector}' ..."), and
/// a selector's class/id name commonly starts with an unrelated area
/// keyword by coincidence (a class named "alt-service-hero-card" has
/// nothing to do with image alt text). English prose never glues a `.`/`#`
/// directly onto a following letter without a space, so this pattern
/// reliably identifies CSS selector syntax rather than legitimate word
/// content -- see score-area-substring-misclassification in the regression
/// corpus. Scoped to `finding.description` only (the one field that embeds
/// live, page-controlled selector text); rule_id/title are fixed, curated
/// strings that never contain a real CSS selector.
pub(crate) fn strip_css_selector_spans(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((_, c)) = chars.next() {
        if (c == '.' || c == '#') && chars.peek().is_some_and(|(_, next)| next.is_alphabetic()) {
            while chars
                .peek()
                .is_some_and(|(_, next)| next.is_alphanumeric() || *next == '-' || *next == '_')
            {
                chars.next();
            }
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

pub(super) fn score_area_for_finding(
    finding: &crate::audit::normalized::NormalizedFinding,
) -> &'static str {
    // rule_id/title/description are specific to this one finding; subcategory
    // is a coarse, shared label covering many unrelated rules (see below).
    let description = strip_css_selector_spans(&finding.description.to_ascii_lowercase());
    let specific = format!(
        "{} {} {}",
        finding.rule_id.to_ascii_lowercase(),
        finding.title.to_ascii_lowercase(),
        description
    );
    let key = format!("{specific} {}", finding.subcategory.to_ascii_lowercase());
    let has = |word: &str| key_has_word_starting_with(&key, word);
    // "navigation" is checked against `specific` only, deliberately excluding
    // `subcategory`: the shared `NavigationInteraction` subcategory label
    // ("Navigation & Operation") covers ~26 unrelated rules (keyboard focus,
    // timing, click target size, pointer gestures, ...), so matching it
    // against subcategory text misrouted all of them into "Landmarks / page
    // structure" (e.g. a11y.target_size_minimum.small, confirmed live in the
    // 2026-08-31 corpus). "landmark" and "main" don't have this problem --
    // no subcategory label contains either word -- so they still match
    // against the full `key`, including the genuinely landmark-related
    // a11y.bypass_blocks.missing ("Missing bypass navigation" in its own
    // title) and a11y.landmark_main.missing ("landmark" in its own rule_id).
    let has_navigation = key_has_word_starting_with(&specific, "navigation");
    if (has("form") && !has("format")) || has("label") || has("input") {
        "Forms"
    } else if has("keyboard") || has("tastatur") {
        "Keyboard"
    } else if has("focus") || has("fokus") {
        "Focus management"
    } else if has("alt") || has("image") || has("bild") {
        "Images / alternative text"
    } else if has("aria") || has("role") {
        "ARIA"
    } else if has("heading") || has("überschrift") || has("h1") {
        "Heading structure"
    } else if has("landmark") || has("main") || has_navigation {
        "Landmarks / page structure"
    } else {
        "Semantics"
    }
}

pub(super) fn build_management_risks(reports: &[NormalizedReport]) -> Vec<ManagementRisk> {
    let legal_flags: usize = reports.iter().map(|r| r.risk.legal_flags).sum();
    let critical: usize = reports.iter().map(|r| r.severity_counts.critical).sum();
    let high: usize = reports.iter().map(|r| r.severity_counts.high).sum();
    // Buttons/forms without an accessible name (WCAG 4.1.2) -- these can be
    // "medium" severity (so they don't count toward `legal_flags`, which
    // requires High/Critical) while still making a control fully inoperable
    // for keyboard/screen-reader users. Thresholds (>=1 -> at least medium,
    // >=5 -> high) mirror `compute_risk_assessment`'s canonical risk-level
    // gating so this dimension can't silently disagree with the report's own
    // top-level risk_level/certificate. Regression (satower-mosterei.de,
    // 2026-08-31): overall risk_level "medium", certificate "EINGESCHRÄNKT",
    // verdict "fail", 4 blocking issues -- yet every management_risks
    // dimension previously read "low" because this field was never consulted.
    let blocking_issues: usize = reports.iter().map(|r| r.risk.blocking_issues).sum();
    let avg = average_accessibility_score(reports);
    let seo = average_module_score_from_reports(reports, "SEO");
    let perf = average_module_score_from_reports(reports, "Performance");
    let mobile = average_module_score_from_reports(reports, "Mobile");
    let component_findings = reports
        .iter()
        .flat_map(|r| r.findings.iter())
        .filter(|f| f.occurrence_count >= 10 || f.complexity == "high")
        .count();

    vec![
        ManagementRisk {
            dimension: "Legal / BFSG-EAA".to_string(),
            level: if legal_flags > 0 || critical > 0 || blocking_issues >= 5 {
                "high"
            } else if high > 0 || blocking_issues > 0 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: if blocking_issues > 0 {
                format!(
                    "{legal_flags} legal flags, {critical} critical and {high} high WCAG findings detected automatically; {blocking_issues} blocking interaction {} (missing accessible name/role) also affect BFSG/EAA operability requirements.",
                    if blocking_issues == 1 { "issue" } else { "issues" }
                )
            } else {
                format!(
                    "{legal_flags} legal flags, {critical} critical and {high} high WCAG findings detected automatically."
                )
            },
        },
        ManagementRisk {
            dimension: "Conversion / usability".to_string(),
            level: if avg < 60
                || critical > 0
                || perf.is_some_and(|s| s < 50)
                || blocking_issues >= 5
            {
                "high"
            } else if avg < 80 || high > 0 || mobile.is_some_and(|s| s < 75) || blocking_issues > 0
            {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: format!(
                "Average accessibility score is {avg}/100; performance {}, mobile {}.{}",
                perf.map(|score| format!("{score}/100"))
                    .unwrap_or_else(|| "not measured".to_string()),
                mobile
                    .map(|score| format!("{score}/100"))
                    .unwrap_or_else(|| "not measured".to_string()),
                if blocking_issues > 0 {
                    format!(
                        " {blocking_issues} interactive element{} block{} completion of key actions.",
                        if blocking_issues == 1 { "" } else { "s" },
                        if blocking_issues == 1 { "s" } else { "" }
                    )
                } else {
                    String::new()
                }
            ),
        },
        ManagementRisk {
            dimension: "SEO / visibility".to_string(),
            level: risk_level_from_optional_score(seo),
            rationale: seo
                .map(|score| format!("Average SEO score is {score}/100."))
                .unwrap_or_else(|| "SEO module was not run.".to_string()),
        },
        ManagementRisk {
            dimension: "Trust / brand".to_string(),
            level: if critical > 0 || avg < 50 {
                "high"
            } else if high > 0 || avg < 75 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: "Accessibility barriers can reduce perceived reliability and inclusiveness."
                .to_string(),
        },
        ManagementRisk {
            dimension: "Project risk".to_string(),
            level: if component_findings >= 3 {
                "high"
            } else if component_findings > 0 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: format!(
                "{component_findings} likely component or template {} coordinated remediation.",
                if component_findings == 1 {
                    "issue needs"
                } else {
                    "issues need"
                }
            ),
        },
    ]
}

pub(super) fn build_decision_actions(reports: &[NormalizedReport]) -> Vec<DecisionAction> {
    let mut findings: Vec<_> = reports.iter().flat_map(|r| r.findings.iter()).collect();
    findings.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
    });
    findings
        .into_iter()
        .take(8)
        .map(|finding| DecisionAction {
            title: finding.title.clone(),
            risk: format!("{:?}", finding.severity).to_lowercase(),
            priority: finding.remediation_priority.clone(),
            complexity: finding.complexity.clone(),
            occurrence_count: finding.occurrence_count,
            root_cause: if finding.occurrence_count >= 10 {
                "Likely shared component or template".to_string()
            } else {
                finding.subcategory.clone()
            },
            expected_impact: finding.expected_impact.clone(),
        })
        .collect()
}

pub(super) fn build_internal_comparison(reports: &[NormalizedReport]) -> InternalComparison {
    let module_names = {
        let mut names = std::collections::BTreeSet::new();
        for report in reports {
            for module in &report.module_scores {
                names.insert(module.name.clone());
            }
        }
        names
    };

    let module_extremes = module_names
        .into_iter()
        .filter_map(|module| {
            let mut scored: Vec<(&NormalizedReport, u32)> = reports
                .iter()
                .filter_map(|report| {
                    report
                        .module_scores
                        .iter()
                        .find(|m| m.name == module)
                        .map(|m| (report, m.score))
                })
                .collect();
            if scored.is_empty() {
                return None;
            }
            scored.sort_by_key(|(_, score)| *score);
            let (worst_report, worst_score) = scored.first().copied()?;
            let (best_report, best_score) = scored.last().copied()?;
            Some(ModuleExtreme {
                module,
                best_url: best_report.url.clone(),
                best_score,
                worst_url: worst_report.url.clone(),
                worst_score,
            })
        })
        .collect();

    let avg = average_accessibility_score(reports);
    let outlier_urls = reports
        .iter()
        .filter_map(|report| {
            let delta = report.score as i32 - avg as i32;
            (delta <= -15).then(|| UrlOutlier {
                url: report.url.clone(),
                accessibility_score: report.score,
                batch_average: avg,
                delta_points: delta,
                reason: "Accessibility score is at least 15 points below the batch average."
                    .to_string(),
            })
        })
        .collect();

    let mut root_map: std::collections::BTreeMap<
        String,
        (usize, std::collections::BTreeSet<String>),
    > = std::collections::BTreeMap::new();
    for report in reports {
        for finding in &report.findings {
            let entry = root_map
                .entry(finding.title.clone())
                .or_insert_with(|| (0, std::collections::BTreeSet::new()));
            entry.0 += finding.occurrence_count;
            entry.1.insert(report.url.clone());
        }
    }
    let mut root_causes: Vec<_> = root_map
        .into_iter()
        .map(|(title, (occurrence_count, urls))| RootCauseSummary {
            title,
            occurrence_count,
            affected_urls: urls.len(),
            classification: if urls.len() >= 2 || occurrence_count >= 10 {
                "likely_template_or_component".to_string()
            } else {
                "page_specific".to_string()
            },
        })
        .collect();
    root_causes.sort_by(|a, b| {
        b.affected_urls
            .cmp(&a.affected_urls)
            .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
    });
    root_causes.truncate(10);

    InternalComparison {
        module_extremes,
        outlier_urls,
        root_causes,
    }
}

pub(super) fn average_accessibility_score(reports: &[NormalizedReport]) -> u32 {
    if reports.is_empty() {
        0
    } else {
        reports.iter().map(|r| r.score).sum::<u32>() / reports.len() as u32
    }
}

pub(super) fn average_module_score_from_reports(
    reports: &[NormalizedReport],
    module_name: &str,
) -> Option<u32> {
    let scores: Vec<u32> = reports
        .iter()
        .filter_map(|report| {
            report
                .module_scores
                .iter()
                .find(|module| module.name == module_name)
                .map(|module| module.score)
        })
        .collect();
    (!scores.is_empty()).then(|| scores.iter().sum::<u32>() / scores.len() as u32)
}

pub(super) fn risk_level_from_optional_score(score: Option<u32>) -> String {
    match score {
        Some(score) if score < 60 => "high",
        Some(score) if score < 80 => "medium",
        Some(_) => "low",
        None => "unknown",
    }
    .to_string()
}

/// Number of distinct violated WCAG rules.
pub(super) fn distinct_wcag_rule_count(
    findings: &[crate::audit::normalized::NormalizedFinding],
) -> usize {
    findings
        .iter()
        .filter(|finding| finding.category == "wcag")
        .map(|f| f.rule_id.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len()
}

pub(super) fn aggregate_severity(pages: &[PageEntry]) -> crate::audit::normalized::SeverityCounts {
    crate::audit::normalized::SeverityCounts {
        critical: pages.iter().map(|p| p.severity_counts.critical).sum(),
        high: pages.iter().map(|p| p.severity_counts.high).sum(),
        medium: pages.iter().map(|p| p.severity_counts.medium).sum(),
        low: pages.iter().map(|p| p.severity_counts.low).sum(),
        total: pages.iter().map(|p| p.severity_counts.total).sum(),
    }
}

pub(super) fn aggregate_occurrences(
    pages: &[PageEntry],
) -> crate::audit::normalized::SeverityCounts {
    crate::audit::normalized::SeverityCounts {
        critical: pages.iter().map(|p| p.occurrence_counts.critical).sum(),
        high: pages.iter().map(|p| p.occurrence_counts.high).sum(),
        medium: pages.iter().map(|p| p.occurrence_counts.medium).sum(),
        low: pages.iter().map(|p| p.occurrence_counts.low).sum(),
        total: pages.iter().map(|p| p.occurrence_counts.total).sum(),
    }
}

pub(super) fn normalized_module_score(
    normalized: &NormalizedReport,
    module_name: &str,
) -> Option<u32> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.score)
}

pub(super) fn avg_module_score(pages: &[PageEntry], name: &str) -> Option<u32> {
    let scores: Vec<u32> = pages
        .iter()
        .flat_map(|p| p.module_scores.iter())
        .filter(|m| m.name == name)
        .map(|m| m.score)
        .collect();
    if scores.is_empty() {
        None
    } else {
        // Round rather than truncate so this matches the PDF's module-average
        // computation (builder/batch.rs) — truncation could show a different
        // score band (e.g. 74 amber vs 75 green) for the same underlying value
        // (#QA-026).
        let sum: u32 = scores.iter().sum();
        Some((sum as f64 / scores.len() as f64).round() as u32)
    }
}

pub(super) fn with_normalized_score(
    mut value: serde_json::Value,
    normalized: &NormalizedReport,
    module_name: &str,
) -> serde_json::Value {
    let Some(entry) = normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
    else {
        return value;
    };

    if let Some(obj) = value.as_object_mut() {
        if module_name == "Performance" {
            if let Some(existing) = obj.remove("score") {
                obj.insert("score_details".to_string(), existing);
            }
        }
        obj.insert("score".to_string(), serde_json::json!(entry.score));
        obj.insert("grade".to_string(), serde_json::json!(entry.grade));
    }

    value
}

pub(super) fn inject_grade(mut value: serde_json::Value, score: u32) -> serde_json::Value {
    let grade = crate::audit::AccessibilityScorer::calculate_grade(score as f32);
    if let Some(obj) = value.as_object_mut() {
        obj.insert("grade".to_string(), serde_json::json!(grade));
    }
    value
}

pub(super) fn inject_unused_js_bytes(
    mut value: serde_json::Value,
    raw: &crate::audit::PerformanceResults,
) -> serde_json::Value {
    let Some(cov) = &raw.coverage else {
        return value;
    };
    if let Some(obj) = value.as_object_mut() {
        if let Some(cov_val) = obj.get_mut("coverage") {
            if let Some(cov_obj) = cov_val.as_object_mut() {
                cov_obj.insert(
                    "unused_js_bytes".to_string(),
                    serde_json::json!(cov.unused_js.unused_bytes),
                );
            }
        }
    }
    value
}

pub(super) fn with_measurement_type(
    mut value: serde_json::Value,
    measurement_type: &str,
) -> serde_json::Value {
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "measurement_type".to_string(),
            serde_json::json!(measurement_type),
        );
    }
    value
}

pub(super) fn batch_report_timestamp(batch_report: &BatchReport) -> DateTime<Utc> {
    batch_report
        .reports
        .iter()
        .map(|report| report.timestamp)
        .max()
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::{
        ComplexityKind, ExpectedImpactKind, NormalizedFinding, ReportVisibilityData, ScoreEffect,
        ScoreImpactData,
    };
    use crate::taxonomy::Severity;

    fn make_finding(rule_id: &str, description: &str) -> NormalizedFinding {
        NormalizedFinding {
            category: "wcag".into(),
            rule_id: rule_id.into(),
            wcag_criterion: "1.3.4".into(),
            axe_id: None,
            wcag_level: "AA".into(),
            dimension: "Accessibility".into(),
            subcategory: "Structure & Semantics".into(),
            issue_class: "Weak".into(),
            dimension_kind: crate::taxonomy::Dimension::Accessibility,
            subcategory_kind: crate::taxonomy::Subcategory::StructureSemantics,
            issue_class_kind: crate::taxonomy::IssueClass::Weak,
            severity: Severity::High,
            user_impact: String::new(),
            technical_impact: String::new(),
            score_impact: ScoreImpactData {
                base_penalty: 3.0,
                max_penalty: 6.0,
                scaling: "Fixed".into(),
            },
            report_visibility: ReportVisibilityData::default(),
            aggregation_key: rule_id.into(),
            title: "Restricted screen orientation".into(),
            description: description.into(),
            help_url: None,
            occurrence_count: 1,
            priority_score: 1.0,
            confidence: "very_high".into(),
            false_positive_risk: "very_low".into(),
            verification: "automatically_confirmed".into(),
            complexity: "low".into(),
            complexity_reason: "Test fixture".into(),
            complexity_kind: ComplexityKind::LowScope,
            expected_impact: "Test fixture".into(),
            expected_impact_kind: ExpectedImpactKind::Wcag {
                occurrence_count: 1,
                score_effect: ScoreEffect::Low,
                wcag_level: "AA".into(),
            },
            bfsg_relevance: "medium".into(),
            remediation_priority: "normal".into(),
            occurrences: vec![],
        }
    }

    fn make_finding_with_subcategory(
        rule_id: &str,
        subcategory: &str,
        subcategory_kind: crate::taxonomy::Subcategory,
        title: &str,
        description: &str,
    ) -> NormalizedFinding {
        let mut finding = make_finding(rule_id, description);
        finding.subcategory = subcategory.into();
        finding.subcategory_kind = subcategory_kind;
        finding.title = title.into();
        finding
    }

    #[test]
    fn score_area_for_finding_does_not_classify_orientation_lock_as_forms() {
        // Regression: the German description "... (Hoch- oder Querformat)"
        // must not be misclassified as "Forms" via a bare "form" substring
        // match against "Querformat".
        let finding = make_finding(
            "a11y.orientation.restricted",
            "Die Seite erzwingt eine bestimmte Bildschirmausrichtung (Hoch- oder Querformat).",
        );
        assert_ne!(score_area_for_finding(&finding), "Forms");
    }

    #[test]
    fn score_area_for_finding_still_classifies_real_forms_findings() {
        let finding = make_finding(
            "a11y.form_labels.missing",
            "Ein Formularfeld hat kein zugeordnetes Label.",
        );
        assert_eq!(score_area_for_finding(&finding), "Forms");
    }

    #[test]
    fn score_area_for_finding_does_not_classify_target_size_as_landmarks() {
        // Regression (sauerstoffzentrum-nordost.de / shop.satower-mosterei.de,
        // 2026-08-31): a11y.target_size_minimum.small carries the shared
        // NavigationInteraction subcategory ("Navigation & Operation"), whose
        // label alone used to satisfy the `contains("navigation")` check and
        // misattribute click-target-size findings to "Landmarks / page
        // structure" -- a click target has nothing to do with landmarks.
        let finding = make_finding_with_subcategory(
            "a11y.target_size_minimum.small",
            "Navigation & Operation",
            crate::taxonomy::Subcategory::NavigationInteraction,
            "Insufficient click target size",
            "Interactive elements such as buttons, links, or icons have a clickable area \
             smaller than 24x24 CSS pixels.",
        );
        assert_ne!(
            score_area_for_finding(&finding),
            "Landmarks / page structure"
        );
    }

    #[test]
    fn score_area_for_finding_still_classifies_bypass_navigation_as_landmarks() {
        // The skip-link rule's own title says "navigation" -- unlike the
        // target-size case above, this must still classify as Landmarks
        // because the match comes from the finding's own title, not merely
        // from the shared subcategory label.
        let finding = make_finding_with_subcategory(
            "a11y.bypass_blocks.missing",
            "Navigation & Operation",
            crate::taxonomy::Subcategory::NavigationInteraction,
            "Missing bypass navigation",
            "No skip link or mechanism to bypass repeated blocks.",
        );
        assert_eq!(
            score_area_for_finding(&finding),
            "Landmarks / page structure"
        );
    }

    #[test]
    fn score_area_for_finding_still_classifies_landmark_main_as_landmarks() {
        let finding = make_finding_with_subcategory(
            "a11y.landmark_main.missing",
            "Navigation & Operation",
            crate::taxonomy::Subcategory::NavigationInteraction,
            "Missing main landmark",
            "The page has no distinct main-content region marked up in the accessibility tree.",
        );
        assert_eq!(
            score_area_for_finding(&finding),
            "Landmarks / page structure"
        );
    }

    #[test]
    fn score_area_for_finding_does_not_classify_contrast_conformance_note_as_forms() {
        // Regression (satower-mosterei.de / xn--sfte-loa-com, 2026-08-31): the
        // contrast rule's evidence text ends with "... (supplementary, not a
        // conformance gate)". A naive `contains("form")` check misreads
        // "conformance" as forms-related and misattributes the accessibility
        // score breakdown's "Forms" area driver to a contrast finding.
        let finding = make_finding(
            "a11y.contrast.weak",
            "Insufficient color contrast ratio: 2.10:1 (text, requires 4.5:1). \
             APCA Lc 12.3 (supplementary, not a conformance gate).",
        );
        assert_ne!(score_area_for_finding(&finding), "Forms");
    }

    #[test]
    fn score_area_for_finding_does_not_classify_selector_embedded_alt_as_images() {
        // Regression (score-area-substring-misclassification corpus entry):
        // text_spacing.rs embeds the raw CSS selector into its message
        // ("Content is clipped by '{selector}' ..."). A class name like
        // "alt-service-hero-card" starts with "alt" purely by coincidence
        // and has nothing to do with image alternative text.
        let finding = make_finding(
            "a11y.text_spacing.clipped",
            "Content is clipped by 'div.alt-service-hero-card' when WCAG-minimum text \
             spacing is applied.",
        );
        assert_ne!(
            score_area_for_finding(&finding),
            "Images / alternative text"
        );
    }

    #[test]
    fn score_area_for_finding_still_classifies_real_alt_text_findings() {
        let finding = make_finding(
            "a11y.alt_text.missing",
            "Image is missing alternative text.",
        );
        assert_eq!(
            score_area_for_finding(&finding),
            "Images / alternative text"
        );
    }
}
