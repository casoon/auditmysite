//! The "problem profile" diagnosis: how concentrated a page's findings are,
//! how broad the damage is across the weighted modules, and the two sentences
//! the Management Summary and the Action Plan derive from that.
//!
//! Extracted verbatim from `single_report.rs` (plan 27, hotspot 1), which
//! mixed this with risks/strengths, the score-driver table, the subcategory
//! breakdown, the appendix, findings, root-cause analysis and the roadmap.
//! The cluster was the cleanest cut available: nothing in it was referenced
//! from outside `single_report.rs`, so only three functions had to widen from
//! private to `pub(super)`. No logic changed in the move.

use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

/// Number of top causes considered for the "problem concentration"
/// percentage (plan/3-problem-concentration-diagnosis-narrative.md's
/// suggested "kleinere, z. B. Top-4-Teilmenge" — deliberately smaller than
/// `ROOT_CAUSE_SHOWN` (6), which sizes the *list* further down, not this
/// one-sentence diagnosis).
const PROBLEM_CONCENTRATION_TOP_N: usize = 4;

/// Below this many total occurrences, "X% trace back to N recurring causes"
/// is statistically meaningless (e.g. "1 occurrence — 100% trace back to 1
/// recurring cause") — feedback: the concentration sentence read as
/// nonsensical/machine-generated on a near-clean audit. Below the threshold,
/// `build_problem_concentration_note` falls back to a plain occurrence count
/// instead, and `build_overall_leverage_note` is suppressed entirely.
const PROBLEM_CONCENTRATION_MIN_OCCURRENCES: usize = 3;

/// The raw numbers behind the "problem concentration" diagnosis
/// (plan/3-problem-concentration-diagnosis-narrative.md): how many
/// occurrences exist in total, how many of the top `PROBLEM_CONCENTRATION_TOP_N`
/// causes they concentrate into, and which cause is affected most. A single
/// calculation shared by two different sentences on two different pages
/// (`build_problem_concentration_note` for the Management Summary,
/// `build_overall_leverage_note` for the Action Plan, plan/7-management-
/// summary-consolidation.md) — same source number, not two independently
/// computed ones that could silently drift apart.
struct ProblemConcentration {
    total_occurrences: usize,
    top_n: usize,
    concentration_pct: i64,
    top1_title: String,
}

fn compute_problem_concentration(vm: &ReportViewModel) -> Option<ProblemConcentration> {
    // WCAG-only (`all_findings` also carries SEO findings, #406/report_model
    // doc comment) — matches `vm.severity.total` ("N WCAG occurrences" on the
    // cover/executive dashboard), so this sentence's total can't silently
    // diverge from the headline count a reader already saw (feedback:
    // "121 WCAG-Vorkommen" vs. "122 Vorkommen wurden erkannt" a page later).
    let mut findings: Vec<&FindingGroup> = vm
        .findings
        .all_findings
        .iter()
        .filter(|f| f.finding.occurrence_count > 0 && !f.finding.wcag_criterion.is_empty())
        .collect();
    if findings.is_empty() {
        return None;
    }
    findings.sort_by_key(|f| std::cmp::Reverse(f.finding.occurrence_count));

    let total_occurrences: usize = findings.iter().map(|f| f.finding.occurrence_count).sum();
    let top_n = findings.len().min(PROBLEM_CONCENTRATION_TOP_N);
    let top_occurrences: usize = findings[..top_n]
        .iter()
        .map(|f| f.finding.occurrence_count)
        .sum();
    let concentration_pct = if total_occurrences > 0 {
        (top_occurrences as f64 * 100.0 / total_occurrences as f64).round() as i64
    } else {
        0
    };

    Some(ProblemConcentration {
        total_occurrences,
        top_n,
        concentration_pct,
        top1_title: findings[0].title.clone(),
    })
}

/// Three-tier semantic condensation of the entire audit for the executive
/// summary, directly beneath the overall verdict (plan/20-problemprofil-badge.md).
/// Combines breadth (count of weighted modules scoring below 75) and systemics
/// (recurring WCAG problem concentration).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProblemProfileKind {
    Punktuell,
    KonzentriertSystemisch,
    BreitSystemisch,
}

impl ProblemProfileKind {
    pub(super) fn label(&self, i18n: &I18n) -> String {
        i18n.t(match self {
            Self::Punktuell => "pdf-analysis-profile-kind-isolated",
            Self::KonzentriertSystemisch => "pdf-analysis-profile-kind-concentrated",
            Self::BreitSystemisch => "pdf-analysis-profile-kind-broad",
        })
    }
}

/// Modules that contribute to the weighted site-quality score and fall below the "Good" band (< 75).
/// Sorted by drag impact descending ((100 - score) * weight). Supplementary context only (e.g.
/// deciding success-vs-info styling for an otherwise "isolated" profile) — NOT the breadth count
/// for the Problem-Profile tier itself, see `critical_contributing_modules`.
fn weak_contributing_modules(
    vm: &ReportViewModel,
) -> Vec<&crate::audit::normalized::ModuleScoreEntry> {
    let mut weak: Vec<&crate::audit::normalized::ModuleScoreEntry> = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_site_quality_score && m.score < 75)
        .collect();
    weak.sort_by_key(|m| std::cmp::Reverse((100 - m.score) * m.weight_pct));
    weak
}

/// Modules that contribute to the weighted site-quality score and are "Kritisch" (< 40 — same
/// FIVE_BAND cutoff used elsewhere in the report, e.g. `score_band_label`). This, not the
/// broader < 75 "Good" band, decides how many modules "determine the need for action" for the
/// Problem-Profile breadth tiering (feedback 2026-09-07: the < 75 threshold pulled in merely
/// "Verbesserungswürdig" modules — e.g. Performance 64 / SEO 65 on inros-lackner.de — alongside
/// the two genuinely critical ones (Accessibility 20 / Security 30), producing "breit &
/// systemisch" where the real driver was two critical modules, not four weak ones).
/// Sorted by drag impact descending, same as `weak_contributing_modules`.
fn critical_contributing_modules(
    vm: &ReportViewModel,
) -> Vec<&crate::audit::normalized::ModuleScoreEntry> {
    let mut critical: Vec<&crate::audit::normalized::ModuleScoreEntry> = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_site_quality_score && m.score < 40)
        .collect();
    critical.sort_by_key(|m| std::cmp::Reverse((100 - m.score) * m.weight_pct));
    critical
}

/// Pure classification function combining breadth (< 40 "Kritisch" module count) and systemics
/// (plan/20-problemprofil-badge.md).
pub(super) fn classify_problem_profile(vm: &ReportViewModel) -> ProblemProfileKind {
    let critical = critical_contributing_modules(vm);
    let critical_count = critical.len();
    let concentration = compute_problem_concentration(vm);
    let has_systemic_wcag = match &concentration {
        Some(c) => {
            c.total_occurrences > PROBLEM_CONCENTRATION_MIN_OCCURRENCES && c.concentration_pct >= 50
        }
        None => false,
    };

    if critical_count >= 3 {
        ProblemProfileKind::BreitSystemisch
    } else if critical_count == 2 || (critical_count == 1 && has_systemic_wcag) {
        ProblemProfileKind::KonzentriertSystemisch
    } else {
        ProblemProfileKind::Punktuell
    }
}

fn format_module_names(
    modules: &[&crate::audit::normalized::ModuleScoreEntry],
    i18n: &I18n,
) -> String {
    let names: Vec<String> = modules
        .iter()
        .map(|m| {
            let key = format!("module-{}", m.name.to_lowercase().replace(' ', "-"));
            let translated = i18n.t(&key);
            if translated == key {
                m.name.clone()
            } else {
                translated
            }
        })
        .collect();

    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        2 => i18n.t_args(
            "pdf-analysis-profile-names-two",
            &[("first", names[0].clone()), ("second", names[1].clone())],
        ),
        _ => i18n.t_args(
            "pdf-analysis-profile-names-many",
            &[
                ("initial", names[..names.len() - 1].join(", ")),
                ("last", names[names.len() - 1].clone()),
            ],
        ),
    }
}

/// Findings the "key risks" list counts as project risk (reach of 10+ elements
/// or a structural fix) — the same value, not a second threshold.
fn structural_finding_count(vm: &ReportViewModel) -> usize {
    vm.management_risks
        .iter()
        .filter_map(|kind| match kind {
            crate::audit::management_risk::ManagementRiskKind::Project { count, .. } => {
                Some(*count)
            }
            _ => None,
        })
        .sum()
}

fn build_problem_profile_description(
    vm: &ReportViewModel,
    kind: ProblemProfileKind,
    i18n: &I18n,
) -> String {
    // Only "Kritisch" (< 40) modules are named as "determining the need for
    // action" — same set `classify_problem_profile` tiers on, so the badge
    // text and the tier it's attached to can't silently diverge (feedback
    // 2026-09-07: naming from the broader < 75 band pulled in merely
    // "Verbesserungswürdig" modules that didn't actually drive the tier).
    let critical_modules = critical_contributing_modules(vm);
    let concentration = compute_problem_concentration(vm);
    let has_systemic_wcag = match &concentration {
        Some(c) => {
            c.total_occurrences > PROBLEM_CONCENTRATION_MIN_OCCURRENCES && c.concentration_pct >= 50
        }
        None => false,
    };

    match kind {
        ProblemProfileKind::Punktuell => {
            // The risk list's "project risk" is the one definition of "wide
            // reach or structural fix"; the badge must not claim "no
            // structural problem" while that list names one (#952-4).
            let tail = i18n.count(
                "pdf-analysis-profile-structural",
                structural_finding_count(vm),
            );
            // "Very good technical condition" is only true while no module is
            // below the "good" band; a module at 49 or 40 is no such thing
            // even though it is not "critical" (< 40) (#1131). Those modules
            // are named instead.
            let weak_modules = weak_contributing_modules(vm);
            if critical_modules.is_empty() && weak_modules.is_empty() {
                i18n.t_args("pdf-analysis-profile-isolated-good", &[("tail", tail)])
            } else if critical_modules.is_empty() {
                let several = if weak_modules.len() > 1 { "yes" } else { "no" };
                i18n.t_args(
                    "pdf-analysis-profile-isolated-weak",
                    &[
                        ("names", format_module_names(&weak_modules, i18n)),
                        ("several", several.to_string()),
                    ],
                )
            } else {
                let names = format_module_names(&critical_modules, i18n);
                i18n.t_args(
                    "pdf-analysis-profile-isolated-named",
                    &[("names", names), ("tail", tail)],
                )
            }
        }
        ProblemProfileKind::KonzentriertSystemisch => {
            let names = format_module_names(&critical_modules, i18n);
            let several = if critical_modules.len() > 1 {
                "yes"
            } else {
                "no"
            };
            i18n.t_args(
                if has_systemic_wcag {
                    "pdf-analysis-profile-concentrated-systemic"
                } else {
                    "pdf-analysis-profile-concentrated-stable"
                },
                &[("names", names), ("several", several.to_string())],
            )
        }
        ProblemProfileKind::BreitSystemisch => {
            let names = format_module_names(&critical_modules, i18n);
            if names.is_empty() {
                i18n.t("pdf-analysis-profile-broad-unnamed")
            } else {
                i18n.t_args("pdf-analysis-profile-broad-named", &[("names", names)])
            }
        }
    }
}

pub(super) fn render_problem_profile(
    builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let kind = classify_problem_profile(vm);
    let title = i18n.t_args("pdf-analysis-profile-title", &[("kind", kind.label(i18n))]);
    let desc = build_problem_profile_description(vm, kind, i18n);

    let callout = match kind {
        ProblemProfileKind::Punktuell => {
            // Context-only use of the broader < 75 band (not the < 40
            // "critical" band the tiering itself uses, see
            // `classify_problem_profile`): a lone merely-"Verbesserungswürdig"
            // module still earns an `info` callout instead of `success`, even
            // though it doesn't change the tier.
            if weak_contributing_modules(vm).is_empty() {
                Callout::success(desc).with_title(title)
            } else {
                Callout::info(desc).with_title(title)
            }
        }
        ProblemProfileKind::KonzentriertSystemisch => Callout::info(desc).with_title(title),
        ProblemProfileKind::BreitSystemisch => Callout::warning(desc).with_title(title),
    };
    builder.add_component(callout)
}

/// "N occurrences were found; ~X% trace back to M recurring causes,
/// especially <top cause>." — a single generated (not free LLM) sentence
/// bridging the score level above and the technical root-cause list further
/// down. Scope is Accessibility + SEO/Optimization combined (deliberate
/// choice, see the plan file's "Offene Frage" — broader than
/// `mandatory_root_causes`, which stays WCAG-only to match the "N
/// Accessibility-Befunde" header count it feeds).
pub(super) fn build_problem_concentration_note(
    vm: &ReportViewModel,
    i18n: &I18n,
) -> Option<String> {
    let c = compute_problem_concentration(vm)?;
    if c.total_occurrences <= PROBLEM_CONCENTRATION_MIN_OCCURRENCES {
        return Some(i18n.t_args(
            "pdf-analysis-profile-few",
            &[
                ("count", c.total_occurrences.to_string()),
                (
                    "found",
                    i18n.count("count-occurrences-found", c.total_occurrences),
                ),
            ],
        ));
    }
    Some(i18n.t_args(
        "pdf-analysis-profile-concentration",
        &[
            ("total", c.total_occurrences.to_string()),
            ("pct", c.concentration_pct.to_string()),
            ("top", c.top_n.to_string()),
            ("title", c.top1_title),
        ],
    ))
}

/// Action-Plan-page framing of the same numbers, worded around leverage
/// rather than diagnosis: "fixing the top causes covers most of what's
/// listed below" (plan/7-management-summary-consolidation.md's suggested
/// "Die vier wichtigsten Ursachen erklären ca. 88 % der erkannten
/// WCAG-Vorkommen").
pub(super) fn build_overall_leverage_note(vm: &ReportViewModel, i18n: &I18n) -> Option<String> {
    let c = compute_problem_concentration(vm)?;
    if c.total_occurrences <= PROBLEM_CONCENTRATION_MIN_OCCURRENCES {
        // Too few occurrences for a "top causes" leverage framing to be
        // meaningful (see `build_problem_concentration_note`) — the Action
        // Plan page already lists the few findings directly, so this bridge
        // sentence is simply omitted rather than restated at low counts.
        return None;
    }
    Some(i18n.t_args(
        "pdf-analysis-profile-leverage",
        &[
            ("top", c.top_n.to_string()),
            ("pct", c.concentration_pct.to_string()),
            ("total", c.total_occurrences.to_string()),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    // The fixture stays where the rest of the report tests use it; only these
    // three moved (plan 27).
    use super::super::single_report::risk_and_strength_tests::test_report_view_model;

    /// #952-4: the badge must not say "no structural problem" while the risk
    /// list counts a finding with wide reach / structural fix.
    #[test]
    fn punktuell_text_follows_the_project_risk_count() {
        let mut vm = test_report_view_model();
        let i18n_de = I18n::new("de").unwrap();
        let i18n_en = I18n::new("en").unwrap();
        vm.modules.module_scores = vec![crate::audit::normalized::ModuleScoreEntry::new(
            "Accessibility",
            99,
            "measured",
            true,
        )];
        vm.management_risks = vec![crate::audit::management_risk::ManagementRiskKind::Project {
            count: 0,
            pages: None,
        }];
        let kind = ProblemProfileKind::Punktuell;
        assert!(build_problem_profile_description(&vm, kind, &i18n_de)
            .contains("ohne strukturelles Problem"));

        vm.management_risks = vec![crate::audit::management_risk::ManagementRiskKind::Project {
            count: 1,
            pages: None,
        }];
        let de = build_problem_profile_description(&vm, kind, &i18n_de);
        assert!(!de.contains("ohne strukturelles Problem"), "{de}");
        assert!(de.contains("1 Befund mit großer Reichweite"), "{de}");
        let en = build_problem_profile_description(&vm, kind, &i18n_en);
        assert!(!en.contains("without structural problems"), "{en}");
        assert!(!en.chars().any(|c| "äöüß".contains(c)), "{en}");
    }

    #[test]
    fn problem_profile_casoon_scenario_is_punktuell() {
        let mut vm = test_report_view_model();
        let i18n_de = I18n::new("de").unwrap();
        let i18n_en = I18n::new("en").unwrap();

        // Real module scores from a live casoon.de audit, 2026-09-07
        // (verified via `--debug-typ`'s "Warum ist der Gesamtwert"-table) — 0
        // critical (< 40) modules, and in fact 0 modules below 75 at all.
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("Accessibility", 99, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Performance", 75, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("SEO", 100, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Security", 95, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Mobile", 90, "measured", true),
        ];
        // 1 occurrence — matches the real audit's single text-spacing finding.
        let mut finding = vm.findings.all_findings[0].clone();
        finding.finding.rule_id = "1.4.12".into();
        finding.title = "Inhalt wird abgeschnitten".into();
        finding.finding.occurrence_count = 1;
        finding.finding.wcag_criterion = "1.4.12".into();
        vm.findings.all_findings = vec![finding];

        let kind = classify_problem_profile(&vm);
        assert_eq!(kind, ProblemProfileKind::Punktuell);
        assert_eq!(kind.label(&i18n_de), "punktuell");
        assert_eq!(kind.label(&i18n_en), "isolated");

        let desc_de = build_problem_profile_description(&vm, kind, &i18n_de);
        assert!(desc_de.contains("Sehr guter technischer Zustand"));
        let desc_en = build_problem_profile_description(&vm, kind, &i18n_en);
        assert!(desc_en.contains("Very good technical condition"));
    }

    #[test]
    fn problem_profile_inros_lackner_scenario_is_konzentriert_systemisch() {
        let mut vm = test_report_view_model();
        let i18n_de = I18n::new("de").unwrap();
        let i18n_en = I18n::new("en").unwrap();

        // Real module scores from a live inros-lackner.de audit, 2026-09-07
        // (verified via `--debug-typ`'s "Warum ist der Gesamtwert"-table) — 4
        // of 5 weighted modules are below 75 ("Good"), but only Accessibility
        // (20) and Security (30) are actually "Kritisch" (< 40); Performance
        // (64) and SEO (65) are merely "Verbesserungswürdig". Regression
        // coverage for the 2026-09-07 fix: with the broader < 75 breadth
        // count this used to misclassify as "breit & systemisch" (4 modules)
        // instead of "konzentriert & systemisch" (2 critical modules).
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("Accessibility", 20, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Performance", 64, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("SEO", 65, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Security", 30, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Mobile", 80, "measured", true),
        ];
        let mut f1 = vm.findings.all_findings[0].clone();
        f1.finding.rule_id = "1.4.3".into();
        f1.title = "Kontrast unzureichend".into();
        f1.finding.occurrence_count = 60;
        f1.finding.wcag_criterion = "1.4.3".into();

        let mut f2 = vm.findings.all_findings[0].clone();
        f2.finding.rule_id = "4.1.2".into();
        f2.title = "Name/Rolle fehlt".into();
        f2.finding.occurrence_count = 30;
        f2.finding.wcag_criterion = "4.1.2".into();
        vm.findings.all_findings = vec![f1, f2];

        let kind = classify_problem_profile(&vm);
        assert_eq!(kind, ProblemProfileKind::KonzentriertSystemisch);
        assert_eq!(kind.label(&i18n_de), "konzentriert & systemisch");
        assert_eq!(kind.label(&i18n_en), "concentrated & systemic");

        let desc_de = build_problem_profile_description(&vm, kind, &i18n_de);
        assert!(desc_de.contains("Barrierefreiheit und Sicherheit bestimmen den Handlungsbedarf"));
        assert!(desc_de.contains("konzentriert sich auf wenige wiederkehrende Muster"));

        let desc_en = build_problem_profile_description(&vm, kind, &i18n_en);
        assert!(desc_en.contains("Accessibility and Security determine the need for action"));
        assert!(desc_en.contains("concentrated in a few recurring patterns"));
    }

    #[test]
    fn problem_profile_auto_birne_scenario_is_breit_systemisch() {
        let mut vm = test_report_view_model();
        let i18n_de = I18n::new("de").unwrap();
        let i18n_en = I18n::new("en").unwrap();

        // Real module scores from a live auto-birne.de audit, 2026-09-07
        // (verified via `--debug-typ`'s "Warum ist der Gesamtwert"-table) — 3
        // critical (< 40) modules (Accessibility, Performance, Security); SEO
        // (82) is stable and Mobile (70) is merely "Verbesserungswürdig", so
        // neither should be named as "determining the need for action".
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("Accessibility", 12, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Performance", 34, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Security", 22, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("SEO", 82, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Mobile", 70, "measured", true),
        ];

        let kind = classify_problem_profile(&vm);
        assert_eq!(kind, ProblemProfileKind::BreitSystemisch);
        assert_eq!(kind.label(&i18n_de), "breit & systemisch");
        assert_eq!(kind.label(&i18n_en), "broad & systemic");

        let desc_de = build_problem_profile_description(&vm, kind, &i18n_de);
        assert!(desc_de.contains("Mehrere technische Bereiche sind gleichzeitig betroffen"));
        assert!(desc_de.contains("Barrierefreiheit, Performance und Sicherheit"));

        let desc_en = build_problem_profile_description(&vm, kind, &i18n_en);
        assert!(desc_en.contains("Multiple technical areas are affected simultaneously"));
        assert!(desc_en.contains("Accessibility, Performance, and Security"));
    }

    /// #1131: modules at 49 and 40 are below the "good" band, so the profile
    /// never calls the technical condition "very good" and names them.
    #[test]
    fn a_module_below_the_good_band_is_never_very_good() {
        let mut vm = test_report_view_model();
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("Accessibility", 49, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Security", 40, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Performance", 90, "measured", true),
        ];
        let kind = classify_problem_profile(&vm);
        for (locale, expected) in [
            (
                "de",
                "Barrierefreiheit und Sicherheit liegen unter dem Bereich „gut“",
            ),
            (
                "en",
                "Accessibility and Security fall below the \"good\" band",
            ),
        ] {
            let i18n = I18n::new(locale).unwrap();
            let text = build_problem_profile_description(&vm, kind, &i18n);
            assert!(text.contains(expected), "{text}");
            assert!(
                !text.contains("Sehr guter") && !text.contains("Very good"),
                "{text}"
            );
        }

        // One module below the band is named in the singular.
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("Accessibility", 92, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Performance", 62, "measured", true),
        ];
        let en = build_problem_profile_description(
            &vm,
            classify_problem_profile(&vm),
            &I18n::new("en").unwrap(),
        );
        assert!(
            en.starts_with("Performance falls below the \"good\" band"),
            "{en}"
        );
    }
}
