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
        wcag_version: crate::wcag::coverage::WCAG_VERSION,
        level: format!("WCAG {} {level}", crate::wcag::coverage::WCAG_VERSION),
        automated_criteria: automated,
        manual_review_criteria: crate::wcag::coverage::manual_review_criteria().len(),
        total_wcag_aa_criteria: total,
        note: "Automated score covers detectable criteria only; context-dependent WCAG criteria require manual review.".to_string(),
    }
}

/// Break the accessibility score down by area, using the scorer's own
/// per-rule penalties as the single source of truth.
///
/// The previous implementation was a second, independent scoring model: flat
/// severity weights (20/14/8/4) that existed nowhere else, a per-area cap at
/// 90, and fixed "importance" weights (15/15/15/10/15/15/8/7) that the scorer
/// knows nothing about. On a real report the two disagreed by a factor of ~3 —
/// inros-lackner-de 2026-09-19 broke a reported accessibility score of 20 into
/// areas whose weighted average was 57 (plan 37).
///
/// The scorer's global steps — `diversity_factor`, the square-root compression
/// above `COMPRESSION_KNEE`, `apply_soft_floor` and the critical/high score cap
/// — are all non-linear and depend on the finding set as a whole, so no fixed
/// weighting of per-area sub-scores can reproduce the headline number. Rather
/// than invent a decomposition that cannot exist, this distributes the loss the
/// score *actually* took (`100 - accessibility_score`) across the areas in
/// proportion to each area's share of the raw per-rule penalty.
///
/// That makes one identity exact and checkable, which `lint` enforces:
///
/// ```text
/// sum(estimated_lost_points) == 100 - accessibility_score
/// ```
///
/// `weight_pct` is consequently each area's share of that loss (summing to 100
/// whenever anything was lost), not an invented importance weight.
pub(super) fn build_accessibility_score_breakdown(
    reports: &[NormalizedReport],
    accessibility_score: u32,
) -> Vec<AccessibilityScoreComponent> {
    const AREAS: [&str; 8] = [
        "Semantics",
        "Forms",
        "Keyboard",
        "Focus management",
        "Images / alternative text",
        "ARIA",
        "Heading structure",
        "Landmarks / page structure",
    ];

    // Raw penalty per area, from the same `ScoreImpact` the scorer applies.
    // `NormalizedFinding` carries the taxonomy rule id, so it resolves via
    // `by_id` where the scorer (which sees legacy WCAG ids on `Violation`)
    // uses `by_legacy_wcag_id` — same registry, same numbers.
    let mut raw_penalty = [0f64; AREAS.len()];
    let mut drivers: [Option<(&str, usize)>; AREAS.len()] = Default::default();
    let mut criteria: [std::collections::HashSet<&str>; AREAS.len()] = Default::default();
    let mut critical_occ = [0usize; AREAS.len()];
    let mut urgent_occ = [0usize; AREAS.len()];

    for finding in reports.iter().flat_map(|report| report.findings.iter()) {
        let Some(area) = score_area_for_finding(finding) else {
            continue;
        };
        let Some(idx) = AREAS.iter().position(|a| *a == area) else {
            continue;
        };

        let impact = crate::taxonomy::RuleLookup::by_id(&finding.rule_id)
            .map(|r| r.score_impact)
            .unwrap_or_else(|| crate::audit::default_impact(finding.severity));
        raw_penalty[idx] += impact.calculate_penalty(finding.occurrence_count.max(1)) as f64;
        criteria[idx].insert(finding.wcag_criterion.as_str());

        let occ = finding.occurrence_count.max(1);
        match finding.severity {
            crate::taxonomy::Severity::Critical => {
                critical_occ[idx] += occ;
                urgent_occ[idx] += occ;
            }
            crate::taxonomy::Severity::High => urgent_occ[idx] += occ,
            _ => {}
        }

        if drivers[idx]
            .map(|(_, count)| finding.occurrence_count > count)
            .unwrap_or(true)
        {
            drivers[idx] = Some((&finding.title, finding.occurrence_count));
        }
    }

    let total_raw: f64 = raw_penalty.iter().sum();
    let total_loss = 100u32.saturating_sub(accessibility_score);

    // Largest-remainder apportionment, so the parts sum to `total_loss`
    // exactly instead of drifting by a point or two through rounding.
    let lost_points = apportion(&raw_penalty, total_raw, total_loss);
    let share_pct = apportion(
        &raw_penalty,
        total_raw,
        if total_raw > 0.0 { 100 } else { 0 },
    );

    AREAS
        .iter()
        .enumerate()
        .map(|(idx, area)| AccessibilityScoreComponent {
            area: (*area).to_string(),
            // The area scored on its own terms, through the scorer's real
            // pipeline — "what would this page score if this area were its
            // only problem". Deliberately NOT `100 - estimated_lost_points`:
            // that would make the field a restatement of the apportionment
            // rather than a statement about the area.
            score: crate::audit::score_from_penalties(
                raw_penalty[idx] as f32,
                criteria[idx].len(),
                critical_occ[idx],
                urgent_occ[idx],
            )
            .round() as u32,
            weight_pct: share_pct[idx],
            estimated_lost_points: lost_points[idx],
            main_driver: drivers[idx]
                .map(|(title, count)| format!("{title} ({count} occurrences)"))
                .unwrap_or_else(|| "No detected driver".to_string()),
        })
        .collect()
}

/// Distribute `total` over `parts` in proportion to each part's share of
/// `sum`, using largest-remainder so the result sums to exactly `total`.
fn apportion<const N: usize>(parts: &[f64; N], sum: f64, total: u32) -> [u32; N] {
    let mut out = [0u32; N];
    if total == 0 || sum <= 0.0 {
        return out;
    }

    let mut remainders: Vec<(usize, f64)> = Vec::with_capacity(N);
    let mut assigned = 0u32;
    for (idx, part) in parts.iter().enumerate() {
        let exact = part / sum * total as f64;
        let floor = exact.floor();
        out[idx] = floor as u32;
        assigned += floor as u32;
        remainders.push((idx, exact - floor));
    }

    // Hand the rounding residue to the largest remainders first.
    remainders.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (idx, _) in remainders.into_iter().take((total - assigned) as usize) {
        out[idx] += 1;
    }
    out
}

/// Which score-breakdown area a finding belongs to, or `None` when it does
/// not belong in the accessibility breakdown at all.
///
/// Reads the answer from the taxonomy (`taxonomy::score_area`) instead of
/// searching the finding's prose for keywords. The old text matching kept
/// mis-filing findings in shipped reports — a landmark rule whose description
/// mentioned `'image'` was published under "Images / alternative text", and an
/// SEO multiple-H1 finding drove the "Heading structure" area of the
/// *accessibility* breakdown (plan 38).
///
/// Non-WCAG findings return `None`: `accessibility_score` is computed from
/// WCAG violations only, so letting an SEO finding into its breakdown made the
/// breakdown describe a different population than the score it explains.
pub(crate) fn score_area_for_finding(
    finding: &crate::audit::normalized::NormalizedFinding,
) -> Option<&'static str> {
    if finding.category != "wcag" {
        return None;
    }
    Some(
        crate::taxonomy::score_area_for_rule(&finding.rule_id)
            // Only reachable for findings whose rule_id is not in the rule
            // register at all; `every_accessibility_rule_has_a_score_area`
            // keeps registered rules off this path.
            .unwrap_or_else(|| {
                crate::taxonomy::score_area_for_subcategory(finding.subcategory_kind)
            })
            .label(),
    )
}

/// JSON view of the shared risk dimensions.
///
/// The derivation itself lives in `audit::management_risk` so the PDF renders
/// the same dimensions instead of a second heuristic of its own (plan 35).
/// Canonical English is baked in here (#406); the PDF re-derives the text in
/// the run language from the same `ManagementRiskKind`.
pub(super) fn build_management_risks(reports: &[NormalizedReport]) -> Vec<ManagementRisk> {
    crate::audit::management_risk::build_management_risk_kinds(reports)
        .into_iter()
        .map(|kind| ManagementRisk {
            dimension: kind.dimension(true),
            level: kind.tier().as_str().to_string(),
            rationale: kind.rationale(true),
        })
        .collect()
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

/// Sum of the module weights that actually fed `overall_score`.
///
/// The score is divided by the weight of the *contributing* modules, so a
/// module that did not run — or that ran but could not measure, as Performance
/// does when no Core Web Vital came back (#QA-023) — silently changes the
/// denominator rather than the result. Publishing the basis lets a consumer
/// see that two runs are not comparable (plan 41).
pub(super) fn overall_score_weight_basis(reports: &[NormalizedReport]) -> u32 {
    // Batch: the lowest basis across the audited pages, so the published value
    // never overstates how much of the model backs the average. When the pages
    // disagree, `weight_basis_is_uniform` is what says the average mixes
    // different bases — this number alone cannot (plan 41).
    page_weight_bases(reports).into_iter().min().unwrap_or(0)
}

/// Whether every audited page contributed the same weight basis. `false` means
/// `overall_score` values were averaged across different module sets and are
/// not directly comparable.
pub(super) fn weight_basis_is_uniform(reports: &[NormalizedReport]) -> bool {
    let bases = page_weight_bases(reports);
    bases.windows(2).all(|pair| pair[0] == pair[1])
}

fn page_weight_bases(reports: &[NormalizedReport]) -> Vec<u32> {
    reports
        .iter()
        .map(|report| {
            report
                .module_scores
                .iter()
                .filter(|m| m.contributes_to_overall)
                .map(|m| m.weight_pct)
                .sum()
        })
        .collect()
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
        // A letter grade only where one exists — a module of weight 0 carries
        // the band word alone (plan 29, D2). `remove` rather than leaving the
        // module's own grade in place: the raw module payload may carry a
        // pre-adjustment grade (UX/Journey before the a11y penalty), and a
        // stale letter is worse than none.
        match &entry.grade {
            Some(grade) => {
                obj.insert("grade".to_string(), serde_json::json!(grade));
            }
            None => {
                obj.remove("grade");
            }
        }
        obj.insert("band".to_string(), serde_json::json!(entry.band));
    }

    value
}

/// Qualitative band for a module that carries no weight in the overall score
/// and therefore no letter grade (plan 29, D2). Canonical English (#406).
pub(super) fn inject_band(mut value: serde_json::Value, score: u32) -> serde_json::Value {
    let band = crate::registry::FIVE_BAND.label(score as f32, true);
    if let Some(obj) = value.as_object_mut() {
        obj.remove("grade");
        obj.insert("band".to_string(), serde_json::json!(band));
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

    fn make_report(findings: Vec<NormalizedFinding>) -> crate::audit::normalized::NormalizedReport {
        let mut report = crate::audit::normalized::normalize(&crate::audit::AuditReport::new(
            "https://example.com".to_string(),
            crate::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            100,
        ))
        .normalized;
        report.findings = findings;
        report
    }

    /// Plan 37: the breakdown must account for exactly the points the score
    /// lost. The old model produced a weighted average of 57 next to a
    /// reported score of 20.
    #[test]
    fn breakdown_lost_points_sum_to_the_actual_score_loss() {
        let mut forms = make_finding("a11y.form_labels.missing", "Field without a label.");
        forms.occurrence_count = 7;
        let mut landmarks = make_finding("a11y.landmark_main.missing", "No main landmark.");
        landmarks.occurrence_count = 31;
        let mut keyboard = make_finding("a11y.focus_order.weak", "Illogical focus order.");
        keyboard.occurrence_count = 1;

        let report = make_report(vec![forms, landmarks, keyboard]);

        for score in [0u32, 20, 45, 73, 99, 100] {
            let breakdown =
                build_accessibility_score_breakdown(std::slice::from_ref(&report), score);
            let lost: u32 = breakdown.iter().map(|a| a.estimated_lost_points).sum();
            assert_eq!(
                lost,
                100 - score,
                "score {score}: lost points must sum to the actual loss",
            );
            // `score` is the area on its own terms and is explicitly NOT
            // `100 - estimated_lost_points`; only its range is guaranteed.
            for area in &breakdown {
                assert!(area.score <= 100, "{}: score out of range", area.area);
            }
        }
    }

    /// The share is of the *loss*, so with nothing lost there is nothing to
    /// share out — and no area may be blamed for points that were never lost.
    #[test]
    fn breakdown_is_all_clean_when_there_are_no_findings() {
        let report = make_report(vec![]);
        let breakdown = build_accessibility_score_breakdown(std::slice::from_ref(&report), 100);

        assert_eq!(breakdown.len(), 8);
        for area in &breakdown {
            assert_eq!(area.score, 100, "{}", area.area);
            assert_eq!(area.weight_pct, 0, "{}", area.area);
            assert_eq!(area.estimated_lost_points, 0, "{}", area.area);
            assert_eq!(area.weight_pct, 0, "{}", area.area);
            assert_eq!(area.main_driver, "No detected driver", "{}", area.area);
        }
    }

    /// Largest-remainder apportionment: the parts must always sum to the
    /// total, including the awkward cases that plain rounding drifts on.
    #[test]
    fn apportion_always_sums_to_the_total() {
        assert_eq!(
            apportion(&[1.0, 1.0, 1.0], 3.0, 100).iter().sum::<u32>(),
            100
        );
        assert_eq!(
            apportion(&[1.0, 2.0, 7.0], 10.0, 80).iter().sum::<u32>(),
            80
        );
        assert_eq!(apportion(&[0.0, 0.0, 5.0], 5.0, 37).iter().sum::<u32>(), 37);
        // Degenerate inputs must not panic or invent points.
        assert_eq!(apportion(&[0.0, 0.0], 0.0, 40), [0, 0]);
        assert_eq!(apportion(&[3.0, 1.0], 4.0, 0), [0, 0]);
        // A single loaded area takes the whole loss.
        assert_eq!(apportion(&[0.0, 9.0], 9.0, 80), [0, 80]);
    }

    /// Plan 41: the basis must reflect what actually fed the score, so a
    /// consumer can tell that two runs are not comparable. A module that did
    /// not measure changes the denominator, not the result (#QA-023).
    #[test]
    fn weight_basis_drops_when_a_weighted_module_does_not_contribute() {
        let mut report = make_report(vec![]);
        assert_eq!(
            overall_score_weight_basis(std::slice::from_ref(&report)),
            report
                .module_scores
                .iter()
                .filter(|m| m.contributes_to_overall)
                .map(|m| m.weight_pct)
                .sum::<u32>(),
        );

        let full = overall_score_weight_basis(std::slice::from_ref(&report));
        if let Some(entry) = report
            .module_scores
            .iter_mut()
            .find(|m| m.contributes_to_overall && m.weight_pct > 0)
        {
            let dropped = entry.weight_pct;
            entry.contributes_to_overall = false;
            assert_eq!(
                overall_score_weight_basis(std::slice::from_ref(&report)),
                full - dropped,
            );
        }
    }

    /// Plan 41, batch: averaging `overall_score` across pages with different
    /// module sets mixes incomparable numbers. The published basis must not
    /// overstate the backing, and the uniformity flag must say when they
    /// disagreed.
    #[test]
    fn batch_weight_basis_reports_the_lowest_and_flags_disagreement() {
        let full = make_report(vec![]);
        let mut reduced = make_report(vec![]);
        let dropped = reduced
            .module_scores
            .iter_mut()
            .find(|m| m.contributes_to_overall && m.weight_pct > 0)
            .map(|m| {
                m.contributes_to_overall = false;
                m.weight_pct
            })
            .expect("a weighted module to drop");

        let full_basis = overall_score_weight_basis(std::slice::from_ref(&full));

        assert!(weight_basis_is_uniform(&[full.clone(), full.clone()]));
        assert_eq!(
            overall_score_weight_basis(&[full.clone(), full.clone()]),
            full_basis,
        );

        let mixed = [full, reduced];
        assert!(
            !weight_basis_is_uniform(&mixed),
            "differing module sets must not read as uniform",
        );
        assert_eq!(
            overall_score_weight_basis(&mixed),
            full_basis - dropped,
            "the published basis must be the lowest, not the first page's",
        );
    }

    /// Plan 40: a single-URL run cannot know whether a template or component
    /// is behind a repeated finding — it has seen one DOM. The claim is only
    /// made where recurrence across pages was actually observed.
    #[test]
    fn project_risk_claims_a_template_cause_only_across_pages() {
        let mut widespread = make_finding("a11y.landmark_unique.invalid", "");
        widespread.occurrence_count = 31;

        let single =
            build_management_risks(std::slice::from_ref(&make_report(vec![widespread.clone()])));
        let project = single
            .iter()
            .find(|r| r.dimension == "Project risk")
            .expect("project risk dimension");
        assert!(
            !project.rationale.contains("component or template"),
            "single-page run infers a template cause: {}",
            project.rationale,
        );
        assert!(
            project.rationale.contains("audit further pages"),
            "single-page run should say what would confirm it: {}",
            project.rationale,
        );

        let batch = build_management_risks(&[
            make_report(vec![widespread.clone()]),
            make_report(vec![widespread]),
        ]);
        let project = batch
            .iter()
            .find(|r| r.dimension == "Project risk")
            .expect("project risk dimension");
        assert!(
            project.rationale.contains("component or template"),
            "recurrence across pages should support the claim: {}",
            project.rationale,
        );
    }

    /// Plan 40: "Trust / brand" was a fixed sentence citing no measurement,
    /// while its level came from the accessibility score.
    #[test]
    fn trust_risk_names_the_data_that_set_it() {
        let mut finding = make_finding("a11y.interactive_name.missing", "");
        finding.occurrence_count = 15;
        let risks = build_management_risks(std::slice::from_ref(&make_report(vec![finding])));
        let trust = risks
            .iter()
            .find(|r| r.dimension == "Trust / brand")
            .expect("trust dimension");

        assert!(
            trust.rationale.contains("/100"),
            "rationale cites no score: {}",
            trust.rationale,
        );
        assert!(
            trust
                .rationale
                .contains("No trust or brand signal is measured"),
            "rationale does not disclose that nothing brand-related is measured: {}",
            trust.rationale,
        );
        assert_ne!(
            trust.rationale,
            "Accessibility barriers can reduce perceived reliability and inclusiveness.",
            "the constant sentence is back",
        );
    }

    /// Plan 38: the areas come from the taxonomy mapping now, so the whole
    /// class of prose-matching misfires is structurally gone. These cases are
    /// kept as the record of what used to break — each one is a real report
    /// that shipped with the wrong area.
    #[test]
    fn score_area_is_taken_from_the_taxonomy_not_from_prose() {
        // "Querformat" used to match `form`.
        assert_eq!(
            score_area_for_finding(&make_finding("a11y.orientation.restricted", "")),
            Some("Semantics"),
        );
        // "conformance" (in the contrast rule's evidence text) used to match `form`.
        assert_eq!(
            score_area_for_finding(&make_finding("a11y.contrast.weak", "")),
            Some("Semantics"),
        );
        // "div.alt-service-hero-card" used to match `alt`.
        assert_eq!(
            score_area_for_finding(&make_finding("a11y.text_spacing.clipped", "")),
            Some("Semantics"),
        );
        // The shared "Navigation & Operation" subcategory label used to route
        // click-target-size findings into Landmarks.
        assert_eq!(
            score_area_for_finding(&make_finding("a11y.target_size_minimum.small", "")),
            Some("Keyboard"),
        );
        // A landmark rule whose description mentions role 'image' shipped under
        // "Images / alternative text" (inros-lackner-de, 2026-09-19).
        assert_eq!(
            score_area_for_finding(&make_finding(
                "a11y.landmark_region.missing",
                "Element with role 'image' is not contained within a landmark region",
            )),
            Some("Landmarks / page structure"),
        );
    }

    /// The genuinely-correct classifications must survive the rewrite.
    #[test]
    fn score_area_still_classifies_real_findings() {
        for (rule_id, expected) in [
            ("a11y.form_labels.missing", "Forms"),
            ("a11y.alt_text.missing", "Images / alternative text"),
            ("a11y.bypass_blocks.missing", "Landmarks / page structure"),
            ("a11y.landmark_main.missing", "Landmarks / page structure"),
            ("a11y.headings.missing", "Heading structure"),
            ("a11y.keyboard.missing", "Keyboard"),
            ("a11y.focus_visible.missing", "Focus management"),
            ("a11y.aria_roles.invalid", "ARIA"),
        ] {
            assert_eq!(
                score_area_for_finding(&make_finding(rule_id, "")),
                Some(expected),
                "{rule_id}",
            );
        }
    }

    /// Plan 38: `accessibility_score` counts WCAG violations only, so an SEO
    /// finding must not appear in its breakdown. `seo.headings.multiple_h1`
    /// was the `main_driver` of the "Heading structure" area in a shipped
    /// report (inros-lackner-de, 2026-09-19).
    #[test]
    fn score_area_excludes_non_wcag_findings() {
        let mut seo = make_finding("seo.headings.multiple_h1", "Two H1 headings found.");
        seo.category = "seo".into();
        assert_eq!(score_area_for_finding(&seo), None);
    }

    #[test]
    fn seo_findings_do_not_reach_the_accessibility_breakdown() {
        let mut seo = make_finding("seo.headings.multiple_h1", "Two H1 headings found.");
        seo.category = "seo".into();
        seo.occurrence_count = 40;
        let report = make_report(vec![seo]);

        let breakdown = build_accessibility_score_breakdown(std::slice::from_ref(&report), 100);
        for area in &breakdown {
            assert_eq!(area.estimated_lost_points, 0, "{}", area.area);
            assert_eq!(area.main_driver, "No detected driver", "{}", area.area);
        }
    }
}
