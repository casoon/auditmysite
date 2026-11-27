//! The "problem profile" diagnosis: how concentrated a page's findings are,
//! how broad the damage is across the weighted modules, and the two sentences
//! the Management Summary and the Action Plan derive from that.
//!
//! Extracted verbatim from `single_report.rs` (hotspot 1), which
//! mixed this with risks/strengths, the score-driver table, the subcategory
//! breakdown, the appendix, findings, root-cause analysis and the roadmap.
//! The cluster was the cleanest cut available: nothing in it was referenced
//! from outside `single_report.rs`, so only three functions had to widen from
//! private to `pub(super)`. No logic changed in the move.

use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

/// Number of top causes considered for the "problem concentration"
/// percentage (a small subset, e.g. the top 4 — deliberately smaller than
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

/// A module below this score is "Kritisch" (the FIVE_BAND cutoff). Named in the
/// profile text so the label next to a low accessibility verdict explains
/// itself (#1208).
const CRITICAL_MODULE_BELOW: u32 = 40;

/// A module below this score is no longer in the "Verbesserungswuerdig" band
/// but "Ausbaufaehig" or worse (FIVE_BAND): the severity cutoff of the profile.
const LOW_MODULE_BELOW: u32 = 60;

/// A module below this score is outside the "Good" band: the reach cutoff.
const GOOD_BAND_FROM: u32 = 75;

/// The raw numbers behind the "problem concentration" diagnosis:
/// how many
/// occurrences exist in total, how many of the top `PROBLEM_CONCENTRATION_TOP_N`
/// causes they concentrate into, and which cause is affected most. A single
/// calculation shared by two different sentences on two different pages
/// (`build_problem_concentration_note` for the Management Summary,
/// `build_overall_leverage_note` for the Action Plan) — same source number, not two independently
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
/// summary, directly beneath the overall verdict.
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

/// Modules that contribute to the weighted site-quality score, carry a
/// measurement (a `not_measured` entry is no evidence either way) and score
/// below `limit`. Sorted by drag impact descending ((100 - score) * weight).
fn modules_below(
    vm: &ReportViewModel,
    limit: u32,
) -> Vec<&crate::audit::normalized::ModuleScoreEntry> {
    let mut below: Vec<&crate::audit::normalized::ModuleScoreEntry> = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| {
            m.contributes_to_site_quality_score
                && m.measurement_type != "not_measured"
                && m.score < limit
        })
        .collect();
    below.sort_by_key(|m| std::cmp::Reverse((100 - m.score) * m.weight_pct));
    below
}

/// Reach: modules below the "Good" band (< 75).
fn weak_contributing_modules(
    vm: &ReportViewModel,
) -> Vec<&crate::audit::normalized::ModuleScoreEntry> {
    modules_below(vm, GOOD_BAND_FROM)
}

/// Severity: modules "Kritisch" (< 40).
fn critical_contributing_modules(
    vm: &ReportViewModel,
) -> Vec<&crate::audit::normalized::ModuleScoreEntry> {
    modules_below(vm, CRITICAL_MODULE_BELOW)
}

/// The modules that "determine the need for action": the critical ones, else
/// those below 60, else those below the good band. The tier says how far the
/// damage reaches; this names where it is deepest.
fn driving_modules(vm: &ReportViewModel) -> Vec<&crate::audit::normalized::ModuleScoreEntry> {
    let critical = critical_contributing_modules(vm);
    if !critical.is_empty() {
        return critical;
    }
    let low = modules_below(vm, LOW_MODULE_BELOW);
    if !low.is_empty() {
        return low;
    }
    weak_contributing_modules(vm)
}

/// Whether the cover verdict (`verdict-tier-*`, the accessibility level) and
/// the problem profile (the distribution across the modules) can stand on the
/// same page. They are two axes: the verdict makes no claim about structure or
/// distribution, the profile none about the accessibility level; the one pair
/// that still clashes is a profile that calls the state "very good" next to a
/// verdict naming barriers. `all_modules_good` is the profile text
/// `pdf-analysis-profile-isolated-good`. `None` when the pair agrees (#1208).
#[cfg(test)]
pub(super) fn verdict_profile_contradiction(
    verdict_key: &str,
    kind: ProblemProfileKind,
    all_modules_good: bool,
) -> Option<&'static str> {
    let names_barriers = matches!(
        verdict_key,
        "verdict-tier-stable" | "verdict-tier-deficient" | "verdict-tier-critical"
    );
    (names_barriers && kind == ProblemProfileKind::Punktuell && all_modules_good)
        .then_some("the profile calls the state very good next to a verdict naming barriers")
}

/// Pure classification of the problem profile from severity, reach and
/// detection confidence (#1228) — never from one threshold.
///
/// * Reach: `weak`, the weighted modules below 75 (the good band).
/// * Severity: `low`, the weighted modules below 60, and `critical`, those
///   below 40.
/// * Confidence: a heuristic module counts toward reach, but not toward the
///   escalation to "broad" (it is no measured damage); a `not_measured` entry
///   counts nowhere.
///
/// Rules, first match wins:
/// 1. **broad & systemic** - three or more *confident* modules below 60, and
///    either three or more modules below 40 or four or more modules below 75.
/// 2. **concentrated & systemic** - three or more modules below 75; or one
///    module below 60 together with another below 75; or one module below 40
///    next to systemic (recurring) WCAG findings.
/// 3. **isolated** - everything else, so at most two modules below 75 and, if
///    there are two, neither below 60 (or a single weak module).
///
/// "Isolated" is therefore never reached with three modules below 75, or with
/// one below 60 next to another below 75.
pub(super) fn classify_problem_profile(vm: &ReportViewModel) -> ProblemProfileKind {
    let weak = weak_contributing_modules(vm);
    let low = modules_below(vm, LOW_MODULE_BELOW);
    let critical = critical_contributing_modules(vm);
    let confident_low = low
        .iter()
        .filter(|m| m.measurement_type != "heuristic")
        .count();
    let has_systemic_wcag = match compute_problem_concentration(vm) {
        Some(c) => {
            c.total_occurrences > PROBLEM_CONCENTRATION_MIN_OCCURRENCES && c.concentration_pct >= 50
        }
        None => false,
    };

    if confident_low >= 3 && (critical.len() >= 3 || weak.len() >= 4) {
        ProblemProfileKind::BreitSystemisch
    } else if weak.len() >= 3
        || (!low.is_empty() && weak.len() >= 2)
        || (critical.len() == 1 && has_systemic_wcag)
    {
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
    // The named modules come from the same sets `classify_problem_profile`
    // tiers on, so the badge text and its tier cannot diverge.
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
                        ("critical", CRITICAL_MODULE_BELOW.to_string()),
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
            let drivers = driving_modules(vm);
            let names = format_module_names(&drivers, i18n);
            let several = if drivers.len() > 1 { "yes" } else { "no" };
            let others = other_weak_modules(vm, &drivers);
            let args = [("names", names), ("several", several.to_string())];
            let mut text = if has_systemic_wcag {
                i18n.t_args("pdf-analysis-profile-concentrated-systemic", &args)
            } else if others.is_empty() {
                i18n.t_args("pdf-analysis-profile-concentrated-stable", &args)
            } else {
                i18n.t_args("pdf-analysis-profile-concentrated-driver", &args)
            };
            push_also_below(&mut text, &others, i18n);
            text
        }
        ProblemProfileKind::BreitSystemisch => {
            let drivers = driving_modules(vm);
            let names = format_module_names(&drivers, i18n);
            let mut text = if names.is_empty() {
                i18n.t("pdf-analysis-profile-broad-unnamed")
            } else {
                i18n.t_args("pdf-analysis-profile-broad-named", &[("names", names)])
            };
            push_also_below(&mut text, &other_weak_modules(vm, &drivers), i18n);
            text
        }
    }
}

/// The modules below the good band that are not among the named drivers: they
/// are part of the reach and must not be described as "stable".
fn other_weak_modules<'a>(
    vm: &'a ReportViewModel,
    drivers: &[&crate::audit::normalized::ModuleScoreEntry],
) -> Vec<&'a crate::audit::normalized::ModuleScoreEntry> {
    weak_contributing_modules(vm)
        .into_iter()
        .filter(|m| !drivers.iter().any(|d| d.name == m.name))
        .collect()
}

fn push_also_below(
    text: &mut String,
    others: &[&crate::audit::normalized::ModuleScoreEntry],
    i18n: &I18n,
) {
    if others.is_empty() {
        return;
    }
    text.push(' ');
    text.push_str(&i18n.t_args(
        "pdf-analysis-profile-also-below",
        &[
            ("names", format_module_names(others, i18n)),
            (
                "several",
                if others.len() > 1 { "yes" } else { "no" }.to_string(),
            ),
        ],
    ));
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
/// choice — broader than
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
/// listed below" (e.g.
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
    // three moved.
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
            severe_wide: 0,
            pages: None,
        }];
        let kind = ProblemProfileKind::Punktuell;
        assert!(build_problem_profile_description(&vm, kind, &i18n_de)
            .contains("ohne strukturelles Problem"));

        vm.management_risks = vec![crate::audit::management_risk::ManagementRiskKind::Project {
            count: 1,
            severe_wide: 0,
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
    fn problem_profile_corporate_site_scenario_is_konzentriert_systemisch() {
        let mut vm = test_report_view_model();
        let i18n_de = I18n::new("de").unwrap();
        let i18n_en = I18n::new("en").unwrap();

        // Real module scores from a live audit of a corporate site, 2026-09-07
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
    fn problem_profile_dealer_site_scenario_is_breit_systemisch() {
        let mut vm = test_report_view_model();
        let i18n_de = I18n::new("de").unwrap();
        let i18n_en = I18n::new("en").unwrap();

        // Real module scores from a live audit of a dealer site, 2026-09-07
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

    /// #1131/#1228: modules at 49 and 40 are below the "good" band, so the
    /// profile never calls the technical condition "very good", is not
    /// "isolated" (one module below 60 next to another below 75) and names them.
    #[test]
    fn a_module_below_the_good_band_is_never_very_good() {
        let mut vm = test_report_view_model();
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("Accessibility", 49, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Security", 40, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Performance", 90, "measured", true),
        ];
        let kind = classify_problem_profile(&vm);
        assert_eq!(kind, ProblemProfileKind::KonzentriertSystemisch);
        for (locale, expected) in [
            (
                "de",
                "Barrierefreiheit und Sicherheit bestimmen den Handlungsbedarf",
            ),
            (
                "en",
                "Accessibility and Security determine the need for action",
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

    const VERDICT_TIERS: [&str; 9] = [
        "verdict-tier-excellent",
        "verdict-tier-excellent-medium",
        "verdict-tier-excellent-high",
        "verdict-tier-excellent-critical",
        "verdict-tier-solid",
        "verdict-tier-stable",
        "verdict-tier-violations",
        "verdict-tier-deficient",
        "verdict-tier-critical",
    ];
    const PROFILE_KINDS: [ProblemProfileKind; 3] = [
        ProblemProfileKind::Punktuell,
        ProblemProfileKind::KonzentriertSystemisch,
        ProblemProfileKind::BreitSystemisch,
    ];

    /// #1208: over every verdict tier and profile, only the pair "very good
    /// state" next to a barrier verdict is a contradiction, and it cannot
    /// arise from a score: a low accessibility score is itself a module below
    /// the "good" band, so the profile names it instead of calling the state
    /// very good.
    #[test]
    fn verdict_and_profile_never_contradict() {
        for tier in VERDICT_TIERS {
            for kind in PROFILE_KINDS {
                for good in [false, true] {
                    let expected = matches!(
                        tier,
                        "verdict-tier-stable" | "verdict-tier-deficient" | "verdict-tier-critical"
                    ) && kind == ProblemProfileKind::Punktuell
                        && good;
                    assert_eq!(
                        verdict_profile_contradiction(tier, kind, good).is_some(),
                        expected,
                        "{tier} {kind:?} good={good}"
                    );
                }
            }
        }
        for score in 0..=100u32 {
            let tier = crate::audit::interpretation::verdict_tier_key("verdict-tier", score);
            for kind in PROFILE_KINDS {
                for others_good in [false, true] {
                    // "Very good" needs every module at 75 or more.
                    let all_modules_good = score >= 75 && others_good;
                    assert_eq!(
                        verdict_profile_contradiction(&tier, kind, all_modules_good),
                        None,
                        "score {score} {kind:?}"
                    );
                }
            }
        }
    }

    /// #1208: the verdict is the accessibility level and says nothing about
    /// structure; only the problem profile does. "Structural backlog" next to
    /// "isolated" was the contradiction.
    #[test]
    fn verdict_texts_below_the_top_tier_claim_no_structure() {
        for locale in ["de", "en"] {
            let i18n = I18n::new(locale).unwrap();
            let args = [("url", "u".to_string()), ("score", "50".to_string())];
            for tier in VERDICT_TIERS
                .iter()
                .filter(|t| !t.starts_with("verdict-tier-excellent"))
            {
                let text = i18n.t_args(tier, &args).to_lowercase();
                for claim in ["struktur", "structural", "detailprobleme", "isolated"] {
                    assert!(!text.contains(claim), "{locale} {tier}: {text}");
                }
            }
        }
    }

    /// #1208: the profile label names what it describes and, next to a low
    /// accessibility verdict, why the profile is still "isolated".
    #[test]
    fn profile_title_names_its_axis_and_the_critical_threshold() {
        let mut vm = test_report_view_model();
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("Accessibility", 70, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Security", 72, "measured", true),
        ];
        for (locale, title, threshold) in [
            ("de", "Verteilung über die Module", "unter 40 Punkten"),
            ("en", "distribution across modules", "below 40 points"),
        ] {
            let i18n = I18n::new(locale).unwrap();
            let kind = classify_problem_profile(&vm);
            assert_eq!(kind, ProblemProfileKind::Punktuell);
            assert!(i18n
                .t_args("pdf-analysis-profile-title", &[("kind", kind.label(&i18n))])
                .contains(title));
            let text = build_problem_profile_description(&vm, kind, &i18n);
            assert!(text.contains(threshold), "{locale}: {text}");
        }
    }

    fn profile_of(scores: &[(&str, u32)]) -> ProblemProfileKind {
        let mut vm = test_report_view_model();
        vm.modules.module_scores = scores
            .iter()
            .map(|(name, score)| {
                crate::audit::normalized::ModuleScoreEntry::new(name, *score, "measured", true)
            })
            .collect();
        classify_problem_profile(&vm)
    }

    /// #1228: INROS (security 40, accessibility 49, HTML 61, SEO 67) is no
    /// "isolated" profile. All four modules are named: the two deepest as the
    /// drivers, the other two as also below the good band, none as "stable".
    #[test]
    fn inros_scenario_is_concentrated_and_names_the_whole_reach() {
        let mut vm = test_report_view_model();
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("Security", 40, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("Accessibility", 49, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new(
                "HTML Conformance",
                61,
                "measured",
                true,
            ),
            crate::audit::normalized::ModuleScoreEntry::new("SEO", 67, "measured", true),
        ];
        let kind = classify_problem_profile(&vm);
        assert_eq!(kind, ProblemProfileKind::KonzentriertSystemisch);
        let en = build_problem_profile_description(&vm, kind, &I18n::new("en").unwrap());
        assert!(
            en.contains("Accessibility and Security determine the need for action"),
            "{en}"
        );
        assert!(en.contains("below the \"good\" band as well"), "{en}");
        assert!(en.contains("SEO and HTML conformance fall"), "{en}");
        assert!(!en.contains("remain stable"), "{en}");
        assert!(!en.chars().any(|c| "äöüß".contains(c)), "{en}");
        let de = build_problem_profile_description(&vm, kind, &I18n::new("de").unwrap());
        assert!(de.contains("ebenfalls unter dem Bereich „gut“"), "{de}");
    }

    /// #1228: the documented rule, exactly at its edges.
    #[test]
    fn the_profile_follows_reach_and_severity() {
        use ProblemProfileKind::*;
        // Reach alone: three modules below 75, none of them low.
        assert_eq!(
            profile_of(&[
                ("Accessibility", 70),
                ("SEO", 72),
                ("Security", 74),
                ("Mobile", 90)
            ]),
            KonzentriertSystemisch
        );
        // Two modules below 75, neither below 60: isolated.
        assert_eq!(
            profile_of(&[("Accessibility", 60), ("SEO", 74), ("Security", 90)]),
            Punktuell
        );
        // One below 60 together with another below 75: not isolated.
        assert_eq!(
            profile_of(&[("Accessibility", 59), ("SEO", 74), ("Security", 90)]),
            KonzentriertSystemisch
        );
        // One module below 60 alone: isolated.
        assert_eq!(
            profile_of(&[("Accessibility", 20), ("SEO", 90), ("Security", 90)]),
            Punktuell
        );
        // Four below 75 but only two below 60: concentrated, not broad.
        assert_eq!(
            profile_of(&[("A", 50), ("B", 55), ("C", 65), ("D", 70), ("E", 90)]),
            KonzentriertSystemisch
        );
        // Three below 60 and four below 75: broad.
        assert_eq!(
            profile_of(&[("A", 50), ("B", 55), ("C", 58), ("D", 70), ("E", 90)]),
            BreitSystemisch
        );
        // Three below 40: broad.
        assert_eq!(
            profile_of(&[("A", 10), ("B", 20), ("C", 30), ("D", 90), ("E", 90)]),
            BreitSystemisch
        );
    }

    /// #1228: over a grid of scores the profile is never "isolated" with
    /// three weighted modules below 75, or one below 60 next to another
    /// below 75.
    #[test]
    fn the_profile_is_never_isolated_for_a_wide_or_deep_shortfall() {
        let grid = [10u32, 39, 40, 59, 60, 74, 75, 90];
        for a in grid {
            for b in grid {
                for c in grid {
                    let scores = [a, b, c];
                    let weak = scores.iter().filter(|s| **s < 75).count();
                    let low = scores.iter().any(|s| *s < 60);
                    let kind = profile_of(&[("Accessibility", a), ("SEO", b), ("Security", c)]);
                    if weak >= 3 || (low && weak >= 2) {
                        assert_ne!(kind, ProblemProfileKind::Punktuell, "{scores:?}");
                    }
                }
            }
        }
    }

    /// #1228: a heuristic module counts toward the reach but cannot lift the
    /// profile to "broad"; an entry that was not measured counts nowhere.
    #[test]
    fn detection_confidence_gates_the_escalation_to_broad() {
        let mut vm = test_report_view_model();
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("A", 50, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("B", 55, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("C", 58, "heuristic", true),
            crate::audit::normalized::ModuleScoreEntry::new("D", 70, "measured", true),
        ];
        assert_eq!(
            classify_problem_profile(&vm),
            ProblemProfileKind::KonzentriertSystemisch
        );
        vm.modules.module_scores = vec![
            crate::audit::normalized::ModuleScoreEntry::new("A", 90, "measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("B", 0, "not_measured", true),
            crate::audit::normalized::ModuleScoreEntry::new("C", 0, "not_measured", true),
        ];
        assert_eq!(classify_problem_profile(&vm), ProblemProfileKind::Punktuell);
    }
}
