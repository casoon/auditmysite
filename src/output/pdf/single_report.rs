//! Single-report page-section renderers for `generate_pdf`.
//!
//! Each function takes ownership of the builder, appends its section's
//! components, and returns the builder for further chaining.

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DevicePreview, DiagnosisPanel, DiagnosisRow, Divider, List,
    MetricStrip, MetricStripItem, PageBreak, SectionHeaderSplit,
};
use renderreport::components::charts::{Chart, ChartType};
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, CardDashboard, DashboardCard, Finding, TableColumn};
use renderreport::prelude::*;

use super::design;
use super::problem_profile::{build_problem_concentration_note, render_problem_profile};
use super::score_drivers::{
    render_additional_score_drivers, render_score_driver_table, render_subcategory_breakdown,
};

use super::appendix::build_cli_snapshot_table;
use super::bik_guide::render_bik_guide_annex;
use super::detail_modules::{
    mobile_category_label, render_a11y_journey_findings, render_ai_transparency,
    render_ai_visibility, render_best_practices, render_budget_violations, render_commerce,
    render_content_visibility, render_dark_mode, render_design_quality, render_html_conform,
    render_journey, render_mobile, render_network_dns, render_performance,
    render_screen_reader_section, render_search_experience, render_security, render_seo,
    render_source_quality, render_tech_stack, render_ux,
};
use super::diagnosis::render_diagnosis_section;
use super::en301549::render_en301549_annex;
use super::findings::render_finding_technical;
use super::helpers::{
    manual_recheck_instruction, map_severity, priority_label_i18n, role_label_i18n,
};
use super::wcag_coverage::render_wcag_coverage_section;
use crate::audit::AuditReport;
use crate::cli::{AnnexKind, ReportLevel};
use crate::i18n::I18n;
use crate::output::module::active_report_modules;
use crate::output::report_model::*;

/// Render a part divider — visually separates the three report parts (#246).
///
/// Each divider produces a page break, a strong level=1 section header tagged
/// with "TEIL N / 3" (or "PART N OF 3"), an audience callout, and a contents list.
/// Part 3 is visually reinforced as a dedicated separator page before the technical
/// appendix and detailed evidence modules (plan/22-report-anhang-visual-cut.md).
pub(super) fn render_part_divider(
    mut builder: renderreport::engine::ReportBuilder,
    part_num: u8,
    title: &str,
    intro: &str,
    audience_title: &str,
    audience_body: &str,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let eyebrow = if en {
        if part_num == 3 {
            "PART 3 OF 3 · TECHNICAL APPENDIX".to_string()
        } else {
            format!("PART {} OF 3", part_num)
        }
    } else if part_num == 3 {
        "TEIL 3 VON 3 · TECHNISCHER ANHANG".to_string()
    } else {
        format!("TEIL {} VON 3", part_num)
    };
    if part_num > 1 {
        builder = builder.add_component(PageBreak::new());
    }
    builder = builder
        // Large chapter number — magazine-style opener.
        .add_component(
            Label::new(format!("{:02}", part_num))
                .with_size("72pt")
                .bold()
                .with_color(design::tokens::MUTED),
        )
        .add_component(
            SectionHeaderSplit::new(title, intro)
                .with_eyebrow(eyebrow)
                .with_level(1),
        )
        .add_component(
            Label::new(format!("{}: {}", audience_title, audience_body))
                .with_size("10.5pt")
                .with_color(design::tokens::NEUTRAL),
        );
    if part_num == 3 {
        builder = builder
            .add_component(Divider::thick().with_color(design::tokens::MUTED))
            .add_component(PageBreak::new());
    }
    builder
}

/// Scannable severity counter row for the executive dashboard — total findings
/// plus a critical/high/medium breakdown. Zero counts read as "all clear"
/// (green), non-zero counts carry their severity hue.
fn build_severity_counter_strip(vm: &ReportViewModel, i18n: &I18n) -> MetricStrip {
    let en = i18n.locale() == "en";
    let count_accent = |n: u32, hue: &'static str| {
        if n > 0 {
            hue
        } else {
            design::tokens::SUCCESS
        }
    };
    let items = vec![
        MetricStripItem::new(
            if en {
                "WCAG occurrences"
            } else {
                "WCAG-Vorkommen"
            },
            vm.severity.total.to_string(),
        )
        .with_accent(design::tokens::INK),
        MetricStripItem::new(
            if en { "Critical" } else { "Kritisch" },
            vm.severity.critical.to_string(),
        )
        .with_accent(count_accent(vm.severity.critical, design::tokens::DANGER)),
        MetricStripItem::new(
            if en { "High" } else { "Hoch" },
            vm.severity.high.to_string(),
        )
        .with_accent(count_accent(vm.severity.high, design::tokens::DANGER)),
        MetricStripItem::new(
            if en { "Medium" } else { "Mittel" },
            vm.severity.medium.to_string(),
        )
        .with_accent(count_accent(vm.severity.medium, design::tokens::WARN_DEEP)),
    ];
    MetricStrip::new(items).compact()
}

/// Build a dashboard card for one module, aligned with the report's grade bands
/// (good ≥ 75, watch 40–74, problem < 40). Carries the same non-normative
/// name-suffix qualifier as the cover gauges and the technical modules
/// overview for Heuristic/Optional-tier modules (#577).
fn module_dashboard_card(module: &ModuleScore, i18n: &I18n) -> DashboardCard {
    DashboardCard {
        name: super::helpers::module_name_with_taxonomy_suffix(
            &module.name,
            &module.measurement_type,
            i18n,
        ),
        score: module.score,
        interpretation: if module.interpretation.is_empty() {
            format!("{}/100", module.score)
        } else {
            module.interpretation.clone()
        },
        good_threshold: 75,
        warn_threshold: 40,
    }
}

/// "What works well" vs "Where optimization pays off" — splits the module
/// scores into two card groups so an unbriefed reader sees strengths and
/// priorities at a glance.
fn render_module_split_dashboards(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    // plan/7-management-summary-consolidation.md ("lighter" option): the
    // weighted, "measured" modules (Accessibility/Performance/Security/
    // Mobile/SEO) are now covered by the score-driver table above with more
    // detail (weight_pct + reasoning) — showing them a second time here as
    // strength/weakness cards was redundant. This dashboard now only covers
    // the composite/heuristic indicator modules (UX, Journey, Search
    // Experience, …) that the score-driver table deliberately excludes
    // (they don't feed the weighted overall score).
    let indicator_modules: Vec<&ModuleScore> = vm
        .modules
        .dashboard
        .iter()
        .filter(|m| m.measurement_type != "measured")
        .collect();
    let strong: Vec<DashboardCard> = indicator_modules
        .iter()
        .filter(|m| m.score >= 75)
        .map(|m| module_dashboard_card(m, i18n))
        .collect();
    let weak: Vec<DashboardCard> = indicator_modules
        .iter()
        .filter(|m| m.score < 75)
        .map(|m| module_dashboard_card(m, i18n))
        .collect();

    if !strong.is_empty() {
        builder = builder.add_component(CardDashboard::new(strong).with_title(if en {
            "What works particularly well"
        } else {
            "Was besonders gut funktioniert"
        }));
    }
    if !weak.is_empty() {
        builder = builder.add_component(CardDashboard::new(weak).with_title(if en {
            "Where optimization pays off"
        } else {
            "Wo sich Optimierung lohnt"
        }));
    } else if !indicator_modules.is_empty() {
        // Module *scores* (0-100 per module) are a different axis from the
        // risk-gated certificate (which can be downgraded to EINGESCHRÄNKT/
        // NICHT BESTANDEN by blocking operability issues or legal flags even
        // when every module score is >= 75 -- see gate_certificate_by_risk).
        // An unconditional "all good" claim here directly contradicted that
        // downgrade (confirmed live in satower-mosterei.de, 2026-08-31:
        // certificate EINGESCHRÄNKT / verdict fail / 4 blockers, right next
        // to "no module needs prioritized optimization"). Qualify the
        // message instead of suppressing it entirely, so the "module scores
        // are fine" observation stays but doesn't read as "nothing to fix".
        // Scoped to the indicator modules shown here — the weighted/measured
        // modules' status is the score-driver table's job, not this one's.
        let risk_gated = matches!(
            vm.summary.certificate.as_str(),
            "EINGESCHRÄNKT" | "NICHT BESTANDEN"
        );
        let message = if risk_gated {
            if en {
                "The additional indicator modules (UX, Journey, …) are in good shape, but operability/legal risk elsewhere in this report still requires action — see Management Summary."
            } else {
                "Die zusätzlichen Indikator-Module (UX, Journey, …) sind in gutem Zustand, dennoch besteht an anderer Stelle in diesem Report Bedienbarkeits- oder rechtliches Risiko, das Handeln erfordert — siehe Management-Zusammenfassung."
            }
        } else if en {
            "The additional indicator modules (UX, Journey, …) are in good shape — no module needs prioritized optimization."
        } else {
            "Die zusätzlichen Indikator-Module (UX, Journey, …) sind in gutem Zustand — kein Modul erfordert vorrangige Optimierung."
        };
        builder = builder.add_component(Label::new(message).with_size("10.5pt").with_color(
            if risk_gated {
                design::tokens::WARN_DEEP
            } else {
                design::tokens::SUCCESS
            },
        ));
    }
    builder
}

/// One of the five fixed report dimensions (Usability / Legal / Business /
/// SEO / Performance), evaluated to a status ("bad"/"warn"/"good") and a
/// localized description. Pure data — no rendering, no `ReportViewModel`
/// dependency — so the bucketing logic below is directly unit-testable.
struct DimensionRow {
    label: String,
    description: String,
    status: &'static str,
}

/// The run's rating and CI verdict, with the reasons that drove it.
///
/// Neither reached the PDF before: `summary.certificate` was only read as a
/// gate, and `verdict`/`verdict_reasons` appeared zero times in the rendered
/// output, so the CLI printed "FAIL — legal_flags: 5, blocking_issues: 36"
/// while the document said nothing (plan 34).
///
/// Labelled "Gesamteinstufung" / "Overall rating", not "Zertifikat":
/// `calculate_certificate` maps the overall score onto a band label and
/// certifies nothing, and this document cites BFSG and EN 301 549 (plan 44).
fn render_overall_rating(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::audit::verdict::Verdict;
    let en = i18n.locale() == "en";

    let rating = super::cover::certificate_label_localized(&vm.summary.certificate, i18n.locale());
    let verdict_word = match (vm.summary.ci_verdict, en) {
        (Verdict::Pass, true) => "passed",
        (Verdict::Pass, false) => "bestanden",
        (Verdict::Warn, true) => "passed with reservations",
        (Verdict::Warn, false) => "mit Vorbehalt bestanden",
        (Verdict::Fail, true) => "not passed",
        (Verdict::Fail, false) => "nicht bestanden",
    };

    let mut body = if en {
        format!("{rating} — automated check {verdict_word}.")
    } else {
        format!("{rating} — automatisierte Prüfung {verdict_word}.")
    };

    // The partial-run reason is dropped here when the provisional sentence
    // below already carries it — otherwise the same fact is stated twice in
    // one paragraph.
    let reason_kinds: Vec<_> = vm
        .summary
        .ci_verdict_reasons
        .iter()
        .filter(|kind| {
            !(vm.summary.run_is_partial
                && **kind == crate::audit::verdict::VerdictReasonKind::AuditQualityNotComplete)
        })
        .collect();

    if !reason_kinds.is_empty() {
        let reasons = reason_kinds
            .iter()
            .map(|kind| kind.text(en))
            .collect::<Vec<_>>()
            .join(", ");
        body.push(' ');
        body.push_str(&if en {
            format!("Reasons: {reasons}.")
        } else {
            format!("Gründe: {reasons}.")
        });
    }

    // A run that did not complete may still state a rating, but not as a
    // settled one — and it has to say so here, not on page ~60 (plan 44).
    if vm.summary.run_is_partial {
        body.push(' ');
        body.push_str(if en {
            "This run did not complete in full, so the rating is provisional and describes only what was measured successfully."
        } else {
            "Dieser Lauf ist nicht vollständig durchgelaufen; die Einstufung ist daher vorläufig und beschreibt nur den erfolgreich gemessenen Umfang."
        });
    }

    let title = if en {
        "Overall rating"
    } else {
        "Gesamteinstufung"
    };
    let callout = match vm.summary.ci_verdict {
        Verdict::Fail => Callout::warning(&body),
        Verdict::Warn => Callout::info(&body),
        Verdict::Pass => Callout::success(&body),
    };
    builder = builder.add_component(callout.with_title(title));
    builder.add_component(
        Label::new(if en {
            "The rating is a band label for the accessibility score within the automated scope, not a conformance certificate."
        } else {
            "Die Einstufung ist ein Bandlabel für den Barrierefreiheits-Wert im automatisierten Prüfumfang, kein Konformitätsnachweis."
        })
        .with_size("8.5pt")
        .with_color(design::tokens::MUTED),
    )
}

/// Renders the risks block (only "bad"/"warn" dimensions, 0..5 rows,
/// zero-risk empty state when all dimensions are "good") and, directly
/// below it, a separate strengths block for the "good" dimensions (#576).
/// Replaces the previous `build_top_risks_checklist`, which unconditionally
/// rendered all 5 dimensions — including positive ones — under a fixed
/// "Die 5 wichtigsten Risiken" title.
pub(super) fn render_risks_and_strengths(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    // The dimensions come from `audit::management_risk`, the same derivation
    // the JSON publishes as `summary.management_risks`. This used to be a
    // second heuristic (`compute_dimension_rows`) over severity counts and
    // three module scores: it carried different dimensions, graded them
    // differently, and the JSON's evidence-bearing rationales never reached
    // the PDF reader at all (plan 35). On inros-lackner-de the JSON held four
    // `high` risks while the panel showed three "bad" and two "warn", and two
    // `high` dimensions were missing entirely.
    //
    // The text is re-derived here in the run language from the same kind, so
    // the German PDF never shows the canonical English stored in the JSON
    // (#406).
    let dimensions: Vec<DimensionRow> = vm
        .management_risks
        .iter()
        // A module that was not run is neither a risk nor a strength, so it
        // belongs in neither panel. Without this an unrun module would be
        // partitioned into the risks bucket purely for not being "good".
        .filter(|kind| kind.tier() != crate::audit::management_risk::RiskTier::Unknown)
        .map(|kind| DimensionRow {
            label: kind.dimension(en),
            description: kind.rationale(en),
            status: kind.tier().status(),
        })
        .collect();

    let (mut risks, strengths): (Vec<DimensionRow>, Vec<DimensionRow>) =
        dimensions.into_iter().partition(|d| d.status != "good");
    // Within the risks block, "bad" outranks "warn" — a low-effort ordering
    // improvement by user impact without redesigning the dimension model.
    // `sort_by_key` is stable, so relative dimension order within each group
    // is preserved.
    risks.sort_by_key(|d| if d.status == "bad" { 0u8 } else { 1u8 });

    if risks.is_empty() {
        // Zero-risk empty state (#576) — reuses the same automated-scope
        // caveat mechanism/wording as the clean-run verdict callout (#572)
        // instead of inventing new phrasing.
        let (automated, total) = crate::wcag::coverage::coverage_stats();
        let empty_state = if en {
            format!(
                "No priority risks identified within the automated audit scope. This result applies to the automated audit scope only ({automated} of about {total} testable WCAG 2.2 AA criteria); criteria requiring manual review are listed in the appendix."
            )
        } else {
            format!(
                "Keine prioritären Risiken im automatisierten Prüfumfang erkannt. Diese Einschätzung bezieht sich ausschließlich auf den automatisierten Prüfumfang ({automated} von ca. {total} testbaren WCAG-2.2-AA-Kriterien); Kriterien mit manuellem Prüfbedarf sind im Anhang aufgeführt."
            )
        };
        builder = builder.add_component(Callout::success(&empty_state).with_title(if en {
            "Key Risks"
        } else {
            "Wichtigste Risiken"
        }));
    } else {
        let title = if en {
            format!("Key Risks ({} identified)", risks.len())
        } else {
            format!("Wichtigste Risiken ({} erkannt)", risks.len())
        };
        let rows: Vec<ChecklistRow> = risks
            .into_iter()
            .map(|d| ChecklistRow::new(d.label, d.description).with_status(d.status))
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(title));
    }

    if !strengths.is_empty() {
        let rows: Vec<ChecklistRow> = strengths
            .into_iter()
            .map(|d| ChecklistRow::new(d.label, d.description).with_status("good"))
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Strengths within the audited scope"
        } else {
            "Stärken im geprüften Umfang"
        }));
    }

    builder
}

/// One entry of the management-summary measures list.
///
/// `module` is the localized module label, `title` the thing to fix and
/// `detail` the specific recommendation. Title and detail stay separate so
/// several findings that resolve into the same action — three CSP
/// misconfigurations are one "fix the CSP" — can be collapsed into one entry.
pub(super) struct TopMeasure {
    module: String,
    title: String,
    detail: String,
}

impl TopMeasure {
    fn render(&self, index: usize, extra: usize, en: bool) -> String {
        let more = match (extra, en) {
            (0, _) => String::new(),
            (n, true) => format!(" (+{n} more of the same kind)"),
            (n, false) => format!(" (+{n} weitere gleicher Art)"),
        };
        if self.title.is_empty() {
            format!("{}. {} — {}{more}", index, self.module, self.detail)
        } else {
            format!(
                "{}. {} — {}: {}{more}",
                index, self.module, self.title, self.detail
            )
        }
    }
}

/// Measures from every module that is not accessibility, ordered by severity
/// first and by how many overall points the module currently costs second.
/// Each entry carries the number of further findings collapsed into it.
///
/// Without this the list was fed exclusively from WCAG finding groups, so a
/// page with no WCAG violations printed "no urgent actions required" although
/// the very same report documented performance, SEO and security work
/// further down. Everything here is already-localized view-model data; no new
/// analysis is derived at render time.
pub(super) fn cross_module_measures(vm: &ReportViewModel, i18n: &I18n) -> Vec<(TopMeasure, usize)> {
    use crate::wcag::Severity;

    let details = &vm.module_details;
    let module_label = |key: &str| -> String {
        let translated = i18n.t(&format!("module-{key}"));
        if translated == format!("module-{key}") {
            key.to_string()
        } else {
            translated
        }
    };

    // Points this module currently costs the overall score — the tie-breaker
    // between modules whose issues share a severity.
    let cost_of = |name: &str| -> u32 {
        vm.modules
            .module_scores
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(name) && m.contributes_to_overall)
            .map(|m| (100u32.saturating_sub(m.score)) * m.weight_pct)
            .unwrap_or(0)
    };

    let severity_rank = |s: Severity| match s {
        Severity::Critical => 0u8,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
    };

    // (severity, module cost, measure)
    let mut ranked: Vec<(u8, u32, TopMeasure)> = Vec::new();

    if let Some(sec) = details.security.as_ref() {
        let cost = cost_of("Security");
        for (title, severity, message) in &sec.issues {
            ranked.push((
                severity_rank(*severity),
                cost,
                TopMeasure {
                    module: module_label("security"),
                    title: title.clone(),
                    detail: message.clone(),
                },
            ));
        }
    }
    if let Some(mobile) = details.mobile.as_ref() {
        let cost = cost_of("Mobile");
        for (category, severity, message) in &mobile.issues {
            ranked.push((
                severity_rank(*severity),
                cost,
                TopMeasure {
                    module: module_label("mobile"),
                    // Same localization the mobile module section uses — the
                    // raw snake_case category must not reach the report.
                    title: mobile_category_label(category, i18n),
                    detail: message.clone(),
                },
            ));
        }
    }
    if let Some(seo) = details.seo.as_ref() {
        let cost = cost_of("SEO");
        for (title, severity, message) in &seo.meta_issues {
            ranked.push((
                severity_rank(*severity),
                cost,
                TopMeasure {
                    module: module_label("seo"),
                    title: title.clone(),
                    detail: message.clone(),
                },
            ));
        }
        // Technical SEO issues carry a localized severity *label* rather than a
        // typed severity, so they rank below the typed ones as Medium.
        for (_issue_type, message, _severity_label) in &seo.technical_issues {
            ranked.push((
                severity_rank(Severity::Medium),
                cost,
                TopMeasure {
                    module: module_label("seo"),
                    title: String::new(),
                    detail: message.clone(),
                },
            ));
        }
    }
    // Performance recommendations are already prioritized by their own module
    // and carry no per-item severity; they rank as Medium.
    if let Some(perf) = details.performance.as_ref() {
        let cost = cost_of("Performance");
        for recommendation in &perf.recommendations {
            ranked.push((
                severity_rank(Severity::Medium),
                cost,
                TopMeasure {
                    module: module_label("performance"),
                    title: String::new(),
                    detail: recommendation.clone(),
                },
            ));
        }
    }

    ranked.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));

    // Collapse findings that resolve into the same action. Three separate CSP
    // misconfigurations are three findings but one measure — listing them
    // individually filled a five-slot management list with one topic.
    let mut collapsed: Vec<(TopMeasure, usize)> = Vec::new();
    for (_, _, measure) in ranked {
        let duplicate = !measure.title.is_empty()
            && collapsed
                .iter()
                .any(|(kept, _)| kept.module == measure.module && kept.title == measure.title);
        if duplicate {
            if let Some(entry) = collapsed
                .iter_mut()
                .find(|(kept, _)| kept.module == measure.module && kept.title == measure.title)
            {
                entry.1 += 1;
            }
        } else {
            collapsed.push((measure, 0));
        }
    }
    collapsed
}

pub(super) fn build_top_measures_list(vm: &ReportViewModel, i18n: &I18n) -> List {
    let en = i18n.locale() == "en";
    let list_title = if en {
        "5 Key Measures"
    } else {
        "Die 5 wichtigsten Maßnahmen"
    };
    let mut list = List::new().with_title(list_title);

    let accessibility = module_label_accessibility(i18n);
    let mut measures: Vec<(TopMeasure, usize)> = vm
        .findings
        .top_findings
        .iter()
        .take(5)
        .map(|group| {
            (
                TopMeasure {
                    module: accessibility.clone(),
                    title: group.title.clone(),
                    detail: group.recommendation.clone(),
                },
                0,
            )
        })
        .collect();

    if measures.len() < 5 {
        let missing = 5 - measures.len();
        measures.extend(cross_module_measures(vm, i18n).into_iter().take(missing));
    }

    if measures.is_empty() {
        let no_measures = if en {
            "No urgent actions required."
        } else {
            "Keine dringenden Maßnahmen erforderlich."
        };
        list = list.add_item(no_measures);
        return list;
    }

    for (idx, (measure, extra)) in measures.iter().enumerate() {
        list = list.add_item(measure.render(idx + 1, *extra, en));
    }

    list
}

fn module_label_accessibility(i18n: &I18n) -> String {
    let translated = i18n.t("module-accessibility");
    if translated == "module-accessibility" {
        "Accessibility".to_string()
    } else {
        translated
    }
}

/// Scope line (#575): this report only ever covers exactly one audited URL,
/// checked with both desktop and mobile viewports (the pipeline always runs
/// both — see `pipeline.rs`'s unconditional dual-viewport pass), and never
/// crawls the rest of the site.
///
/// Shown on the cover *and* at the top of the management summary, from this
/// one source: a reader who only ever sees the cover would otherwise read
/// "Barrierefreiheit 100" as a statement about the whole site.
pub(super) fn audit_scope_line(en: bool) -> String {
    if en {
        "Audited: 1 URL · Desktop and Mobile · no website crawl".to_string()
    } else {
        "Geprüft: 1 URL · Desktop und Mobile · kein Website-Crawl".to_string()
    }
}

pub(super) fn render_management_page(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let mgt_title = if en {
        "Management Summary"
    } else {
        "Management-Sicht"
    };
    let mgt_subtitle = if en {
        "Overall status, key risks, prioritized actions, and leverage at a glance."
    } else {
        "Gesamtstatus, Hauptrisiken, wichtigste Maßnahmen und Hebel auf einen Blick."
    };

    builder = builder.add_component(
        SectionHeaderSplit::new(mgt_title, mgt_subtitle)
            .with_eyebrow(if en {
                "MANAGEMENT SUMMARY"
            } else {
                "MANAGEMENT-SUMMARY"
            })
            .with_level(1),
    );

    builder = builder.add_component(
        Label::new(audit_scope_line(en))
            .with_size("10.5pt")
            .bold()
            .with_color(design::tokens::NEUTRAL),
    );

    // 1. Scannable severity counters — the 20-second top line
    builder = builder.add_component(build_severity_counter_strip(vm, i18n));
    // Short definition right at first use, so a reader doesn't have to reach
    // the methodology appendix to learn "Vorkommen" ≠ "distinct rule" — the
    // full breakdown (finding groups vs. occurrences, all categories) still
    // lives only in the appendix, this is just the one-line pointer (#572).
    builder = builder.add_component(
        Label::new(if en {
            "Occurrences = individual affected elements, not the number of distinct rules — see the appendix for the full breakdown."
        } else {
            "Vorkommen = einzelne betroffene Elemente, nicht die Anzahl unterschiedlicher Regeln — vollständige Aufschlüsselung im Anhang."
        })
        .with_size("8.5pt")
        .with_color(design::tokens::MUTED),
    );

    // The strip counts confirmed WCAG occurrences only. Without this line a
    // reader took "0" for "nothing found" while the journey, screen-reader and
    // manual-check sections listed findings further in.
    let evidence = &vm.findings.evidence;
    // Page-level manual checks are not the criteria that are manual-only by
    // nature: "5 manuell zu prüfende Kriterien" stood next to an appendix
    // listing 21.
    let manual_only = crate::wcag::coverage::manual_review_criteria().len();
    if evidence.warnings > 0 || evidence.manual_checks > 0 {
        builder = builder.add_component(
            Label::new(if en {
                format!(
                    "Counted here are confirmed WCAG violations only. In addition: {} heuristic accessibility warning(s) and {} point(s) on this page to confirm by hand — documented, but not scored. A further {} WCAG criteria can only be checked manually (appendix).",
                    evidence.warnings, evidence.manual_checks, manual_only
                )
            } else {
                format!(
                    "Gezählt sind hier ausschließlich bestätigte WCAG-Verstöße. Hinzu kommen {} heuristische Barrierefreiheits-Warnung(en) und {} Stelle(n) auf dieser Seite, die von Hand zu bestätigen sind — dokumentiert, aber nicht in den Score eingerechnet. Weitere {} WCAG-Kriterien lassen sich nur manuell prüfen (Anhang).",
                    evidence.warnings, evidence.manual_checks, manual_only
                )
            })
            .with_size("8.5pt")
            .with_color(design::tokens::MUTED),
        );
    }

    // 2. Overall verdict (the single core sentence)
    builder = builder.add_component(Callout::info(&vm.summary.verdict).with_title(if en {
        "Overall Verdict"
    } else {
        "Gesamturteil"
    }));

    builder = render_overall_rating(builder, vm, i18n);

    // Problem profile badge (plan/20-problemprofil-badge.md)
    builder = render_problem_profile(builder, vm, i18n);

    // Surface a partial/insufficient audit run right here, not only in the
    // methodology appendix (#575-adjacent review finding, 2026-09-01): a
    // reader who never reaches the appendix must still see that the scores
    // above describe a downgraded run.
    if let Some(note) = &vm.summary.audit_quality_note {
        // Insufficient (failed rule checks) is a genuine data-quality problem
        // and stays a warning; Partial (a stability/retry budget hit) is a
        // normal automated-audit coverage limitation and is framed calmer,
        // as "Coverage" rather than "Audit Quality" (feedback: a plain
        // "audit incomplete" warning next to a clean run read as if the
        // tool itself had failed, see `audit_quality_severe`).
        let callout = if vm.summary.audit_quality_severe {
            Callout::warning(note).with_title(if en {
                "Audit Quality"
            } else {
                "Audit-Qualität"
            })
        } else {
            Callout::info(note).with_title(if en { "Coverage" } else { "Prüfabdeckung" })
        };
        builder = builder.add_component(callout);
    }
    // For a clean automated run (no findings), state the automated-scope
    // caveat directly alongside the headline verdict — not only in the
    // appendix — so "0 findings" doesn't read as a full WCAG conformance
    // claim (#572).
    if !vm.severity.has_issues {
        let (automated, total) = crate::wcag::coverage::coverage_stats();
        builder = builder.add_component(
            Label::new(if en {
                format!(
                    "This result applies to the automated audit scope only ({automated} of ~{total} testable WCAG 2.2 AA criteria); criteria requiring manual review are listed in the appendix."
                )
            } else {
                format!(
                    "Diese Einschätzung bezieht sich ausschließlich auf den automatisierten Prüfumfang ({automated} von ca. {total} testbaren WCAG-2.2-AA-Kriterien); Kriterien mit manuellem Prüfbedarf sind im Anhang aufgeführt."
                )
            })
            .with_size("9pt")
            .with_color(design::tokens::MUTED),
        );
    }

    // 2b. Quality profile radar — balance across all dimensions at a glance.
    let radar_data: Vec<(String, f64)> = vm
        .modules
        .dashboard
        .iter()
        .map(|m| {
            let short = m
                .name
                .split([' ', '&'])
                .next()
                .unwrap_or(m.name.as_str())
                .trim()
                .to_string();
            (short, m.score as f64)
        })
        .collect();
    if radar_data.len() >= 3 {
        builder = builder.add_component(
            Chart::new(
                if en {
                    "Quality profile"
                } else {
                    "Qualitätsprofil"
                },
                ChartType::Radar,
            )
            .add_series("scores", radar_data),
        );
    }

    // 2c. Score-driver table — the radar shows *balance*, this answers
    // "why is the overall score X": which modules pull it down vs.
    // stabilize it, and how much each one actually weighs
    // (plan/2-score-driver-breakdown.md).
    builder = render_score_driver_table(builder, vm, i18n);

    // 2d. Problem-concentration diagnosis — the bridge between the score
    // level above and the technical root-cause list on a later page: "how
    // concentrated are the occurrences on a few recurring causes"
    // (plan/3-problem-concentration-diagnosis-narrative.md).
    if let Some(note) = build_problem_concentration_note(vm, i18n) {
        builder = builder.add_component(Label::new(note).with_size("10.5pt"));
    }

    // 2d-bis. Additional score drivers beyond Accessibility — the note above
    // only explains the WCAG occurrences; on sites where Performance or
    // Security are equally significant drags on the overall score, they were
    // left unexplained next to their number in the table above (feedback:
    // management summary read as accessibility-only, e.g. "Performance 16"
    // and "Security 22" had no plain-language reason next to them).
    builder = render_additional_score_drivers(builder, vm, i18n);

    // 2e. Accessibility/Security sub-category breakdown — a low module
    // score explained at a finer grain than one number (plan/5-module-
    // accessibility-security-driver-detail.md).
    let en_label = if en {
        "Accessibility"
    } else {
        "Barrierefreiheit"
    };
    let accessibility_entries: Vec<(String, u32)> = vm
        .modules
        .accessibility_subcategory_scores
        .iter()
        .map(|s| (s.subcategory_kind.label(en).to_string(), s.score))
        .collect();
    builder = render_subcategory_breakdown(builder, en_label, &accessibility_entries, i18n);

    let security_label = if en { "Security" } else { "Sicherheit" };
    let security_entries: Vec<(String, u32)> = vm
        .modules
        .security_category_scores
        .iter()
        .map(|s| (s.category_kind.label(en).to_string(), s.score))
        .collect();
    builder = render_subcategory_breakdown(builder, security_label, &security_entries, i18n);

    // 3. Strengths vs. priorities as card groups
    builder = render_module_split_dashboards(builder, vm, i18n);

    // 4. Risks & strengths
    builder = render_risks_and_strengths(builder, vm, i18n);

    // 5. Measures
    builder = builder.add_component(build_top_measures_list(vm, i18n));

    builder
}
/// Sections 5b + 6+ — diagnosis, findings by severity tier, module metrics, appendix.
pub(super) fn render_tech_details(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    // Technical modules overview DiagnosisPanel
    if !vm.modules.dashboard.is_empty() {
        let diag_rows: Vec<DiagnosisRow> = vm
            .modules
            .dashboard
            .iter()
            .map(|module| {
                let status = if module.score >= 80 {
                    "good"
                } else if module.score >= 50 {
                    "warn"
                } else {
                    "bad"
                };
                let display_name = super::helpers::module_name_with_taxonomy_suffix(
                    &module.name,
                    &module.measurement_type,
                    i18n,
                );
                DiagnosisRow::new(&display_name, format!("{}/100", module.score))
                    .with_status(status)
            })
            .collect();
        builder = builder.add_component(
            DiagnosisPanel::new(diag_rows).with_title(i18n.t("panel-modules-overview")),
        );
    }

    if vm.severity.has_issues {
        builder = render_diagnosis_section(builder, &vm.diagnosis, i18n);
    }
    // Element-evidence crop temp files are named `ams-evidence-{ts}-{n}.png`,
    // keyed off the report timestamp (matches `cleanup_screenshot_temps`'s
    // desktop/mobile naming) plus a per-report sequence number.
    let report_ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
    let mut evidence_seq: usize = 0;
    let mut acronyms_expanded = false;
    builder = render_findings_section(
        builder,
        vm,
        en,
        i18n,
        report_ts,
        &mut evidence_seq,
        &mut acronyms_expanded,
    );

    // Interactive Accessibility-Journey findings (Phase 2+)
    if !report.interactive_findings.is_empty() {
        builder = render_a11y_journey_findings(
            builder,
            &report.interactive_findings,
            report.accessibility_journey.as_ref(),
            i18n,
        );
    }

    // Screen-reader reading-order audit (#411)
    if let Some(sr) = report.screen_reader_audit.as_ref() {
        builder = render_screen_reader_section(builder, sr, report.patterns.as_ref(), i18n);
    }

    builder
}

pub(super) fn render_appendix_full(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    findings: &[crate::audit::normalized::NormalizedFinding],
    config: &ReportConfig,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let (app_title, app_intro) = if en {
        (
            "Appendix & Methodology",
            "Technical scope, methodology, WCAG coverage, and the complete violations list.",
        )
    } else {
        (
            "Anhang & Methodik",
            "Prüfumfang, Methodik, WCAG-Coverage und die vollständige Fundstellenliste.",
        )
    };
    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(app_title, app_intro)
            .with_eyebrow(if en { "APPENDIX" } else { "ANHANG" })
            .with_level(1),
    );

    // WCAG Coverage (issue #37)
    if vm.meta.report_level != ReportLevel::Executive {
        builder = render_wcag_coverage_section(builder, report, i18n);
        builder = render_manual_only_criteria_note(builder, i18n);

        // EN 301 549 clause annex — opt-in only (see `--annex en301549`).
        if config.annex == Some(AnnexKind::En301549) {
            builder = render_en301549_annex(
                builder,
                findings,
                &report.accessibility.wcag_results.rule_outcomes,
                i18n,
            );
        }

        // BIK-für-Alle guide chapter mapping — opt-in only (see `--annex bik`).
        if config.annex == Some(AnnexKind::Bik) {
            let design_quality_findings: &[crate::design_quality::DesignQualityFinding] = report
                .experience
                .design_quality
                .as_ref()
                .map(|m| m.findings.as_slice())
                .unwrap_or(&[]);
            let seo_technical_issues: Vec<crate::seo::technical::TechnicalIssue> = report
                .discoverability
                .seo
                .as_ref()
                .map(|s| crate::seo::collect_technical_issues(&s.technical, en))
                .unwrap_or_default();
            let easy_language_detected = report
                .patterns
                .as_ref()
                .is_some_and(|p| p.recognized.iter().any(|r| r.pattern == "EasyLanguage"));
            let screen_reader_issues: &[crate::screen_reader::SrAuditIssue] = report
                .screen_reader_audit
                .as_ref()
                .map(|sr| sr.issues.as_slice())
                .unwrap_or(&[]);
            let accessibility_assessments =
                crate::audit::normalized::normalize_assessments(&report.accessibility.wcag_results);
            builder = render_bik_guide_annex(
                builder,
                findings,
                &report.interactive_findings,
                screen_reader_issues,
                design_quality_findings,
                &seo_technical_issues,
                easy_language_detected,
                &accessibility_assessments,
                i18n,
            );
        }
    }

    builder = render_assessment_and_execution_notes(builder, report, i18n);

    // Methodology / disclaimer text blocks
    let limitations_title = i18n.t("callout-limitations-title");
    let limitations_text = format!("{}: {}", limitations_title, vm.methodology.limitations);
    builder = builder.add_component(
        Label::new(&limitations_text)
            .with_size("10.5pt")
            .with_color("#475569"),
    );

    let disclaimer_title = i18n.t("callout-note-title");
    let disclaimer_text = format!("{}: {}", disclaimer_title, vm.methodology.disclaimer);
    builder = builder.add_component(
        Label::new(&disclaimer_text)
            .with_size("10.5pt")
            .with_color("#475569"),
    );

    // Complete violations list
    builder = render_appendix_section(builder, vm, i18n);

    builder
}

/// Fixed list (plan/15) of WCAG A/AA criteria that stay outside automated
/// testing on structural grounds, not merely "not yet implemented" — the
/// contentual sensibleness of a focus order (2.4.3), the intelligibility of
/// an error message (3.3.1), the practical usability of 400% zoom (1.4.4/
/// 1.4.10), the factual correctness of an alt text (1.1.1) or a video's
/// caption/transcript summary (1.2.1/1.2.2) can never be judged from markup
/// alone, however good the tool. Deliberately not derived from this page's
/// findings (unlike the EN 301 549/BIK annexes) — a 0-finding report would
/// otherwise leave this fully invisible, which is exactly the transparency
/// gap this section closes. Renders unconditionally (no `--annex` flag),
/// same report-level gate as the WCAG coverage section it follows.
fn render_manual_only_criteria_note(
    builder: renderreport::engine::ReportBuilder,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let rows = if en {
        vec![
            ChecklistRow::new(
                "2.4.3 · Focus Order",
                "Whether the technical focus order makes sense in context depends on the page's \
                 content and cannot be assessed automatically.",
            ),
            ChecklistRow::new(
                "3.3.1 · Error Identification",
                "Whether an error message is actually understandable to people is a content \
                 question, not a structural check.",
            ),
            ChecklistRow::new(
                "1.4.4 / 1.4.10 · Resize Text & Reflow",
                "Whether the page stays practically usable at 400% zoom goes beyond the \
                 technical reflow structure and requires manual review.",
            ),
            ChecklistRow::new(
                "1.1.1 · Non-text Content",
                "Whether alt text correctly describes the image content can only be judged by \
                 a person; automated checks can only confirm that alt text is present at all.",
            ),
            ChecklistRow::new(
                "1.2.1 / 1.2.2 · Time-based Media",
                "Whether a caption or transcript summary correctly reflects a video's content \
                 cannot be checked automatically.",
            ),
        ]
    } else {
        vec![
            ChecklistRow::new(
                "2.4.3 · Fokusreihenfolge",
                "Ob die technische Fokusreihenfolge inhaltlich sinnvoll ist, hängt vom \
                 Seitenaufbau ab und lässt sich nicht automatisiert bewerten.",
            ),
            ChecklistRow::new(
                "3.3.1 · Fehlerkennzeichnung",
                "Ob eine Fehlermeldung für Menschen tatsächlich verständlich ist, ist eine \
                 inhaltliche Frage und kein struktureller Check.",
            ),
            ChecklistRow::new(
                "1.4.4 / 1.4.10 · Textgröße & Reflow",
                "Ob die Seite bei 400 % Zoom praktisch bedienbar bleibt, geht über die \
                 technische Reflow-Struktur hinaus und erfordert eine manuelle Prüfung.",
            ),
            ChecklistRow::new(
                "1.1.1 · Nicht-Text-Inhalte",
                "Ob ein Alternativtext den Bildinhalt korrekt beschreibt, kann nur ein Mensch \
                 beurteilen; automatisiert prüfbar ist nur, ob überhaupt ein Alternativtext \
                 vorhanden ist.",
            ),
            ChecklistRow::new(
                "1.2.1 / 1.2.2 · Zeitbasierte Medien",
                "Ob eine Untertitel- oder Transkript-Zusammenfassung den Videoinhalt inhaltlich \
                 korrekt wiedergibt, lässt sich nicht automatisiert prüfen.",
            ),
        ]
    };
    builder.add_component(ChecklistPanel::new(rows).with_title(if en {
        "Structurally manual-only criteria"
    } else {
        "Strukturell nur manuell prüfbare Kriterien"
    }))
}

fn render_assessment_and_execution_notes(
    mut builder: renderreport::engine::ReportBuilder,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let wcag = &report.accessibility.wcag_results;
    if !wcag.warnings.is_empty() || !wcag.not_testables.is_empty() {
        let mut rows = Vec::new();
        for finding in wcag.not_testables.iter().take(20) {
            let mut recommendation = finding
                .rule_id
                .as_deref()
                .and_then(crate::output::explanations::get_explanation)
                .or_else(|| crate::output::explanations::get_explanation(&finding.rule))
                .map(|explanation| explanation.recommendation_for(i18n.locale()).to_string())
                .unwrap_or_else(|| {
                    if en {
                        "Verify this criterion manually on the rendered page.".to_string()
                    } else {
                        "Dieses Kriterium manuell an der gerenderten Seite prüfen.".to_string()
                    }
                });
            if let Some(instruction) = manual_recheck_instruction(&finding.rule, en) {
                recommendation.push(' ');
                recommendation.push_str(instruction);
            }
            rows.push(
                ChecklistRow::new(
                    format!(
                        "{} · {}",
                        finding.rule,
                        if en {
                            "Manual review"
                        } else {
                            "Manuelle Prüfung"
                        }
                    ),
                    recommendation,
                )
                .with_status("warn"),
            );
        }
        for finding in wcag
            .warnings
            .iter()
            .take(20usize.saturating_sub(rows.len()))
        {
            let mut text = finding
                .rule_id
                .as_deref()
                .and_then(crate::output::explanations::get_explanation)
                .or_else(|| crate::output::explanations::get_explanation(&finding.rule))
                .map(|explanation| explanation.recommendation_for(i18n.locale()).to_string())
                .unwrap_or_else(|| {
                    if en {
                        "Confirm this heuristic signal manually.".to_string()
                    } else {
                        "Dieses heuristische Signal manuell bestätigen.".to_string()
                    }
                });
            if let Some(instruction) = manual_recheck_instruction(&finding.rule, en) {
                text.push(' ');
                text.push_str(instruction);
            }
            rows.push(
                ChecklistRow::new(
                    format!(
                        "{} · {}",
                        finding.rule,
                        if en { "Warning" } else { "Hinweis" }
                    ),
                    text,
                )
                .with_status("warn"),
            );
        }
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(if en {
            "Manual review and heuristic signals"
        } else {
            "Manuelle Prüfpunkte und heuristische Hinweise"
        }));
    }

    if let Some(journey) = report.accessibility_journey.as_ref() {
        let execution = &journey.execution;
        let text = if en {
            format!(
                "Interactive coverage: {} of {} attempted journeys completed; {} failed, {} skipped{}.",
                execution.completed,
                execution.attempted,
                execution.failed,
                execution.skipped,
                if execution.budget_exhausted { "; budget exhausted" } else { "" }
            )
        } else {
            format!(
                "Interaktive Abdeckung: {} von {} versuchten Journeys abgeschlossen; {} fehlgeschlagen, {} übersprungen{}.",
                execution.completed,
                execution.attempted,
                execution.failed,
                execution.skipped,
                if execution.budget_exhausted { "; Budget ausgeschöpft" } else { "" }
            )
        };
        builder = builder.add_component(Label::new(text).with_size("10.5pt").with_color("#475569"));
    }

    builder
}

fn render_findings_section(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    en: bool,
    i18n: &I18n,
    report_ts: i64,
    evidence_seq: &mut usize,
    acronyms_expanded: &mut bool,
) -> renderreport::engine::ReportBuilder {
    if vm.severity.has_issues {
        let (findings_title, findings_intro) = if en {
            (
                "Classification of Findings",
                "All technical findings, categorized by systemic template relevance and individual page occurrences.",
            )
        } else {
            (
                "Klassifizierung der Befunde",
                "Alle technischen Befunde, getrennt nach systemischen Komponentenfehlern und Einzelfällen.",
            )
        };
        builder = builder.add_component(PageBreak::new()).add_component(
            SectionHeaderSplit::new(findings_title, findings_intro)
                .with_eyebrow(if en { "FINDINGS" } else { "BEFUNDE" })
                .with_level(2),
        );

        let all_findings = &vm.findings.all_findings;

        let systemic_mandatory: Vec<&FindingGroup> = all_findings
            .iter()
            .filter(|f| f.is_component_issue && f.criticality_tier == CriticalityTier::Mandatory)
            .collect();

        let systemic_optimization: Vec<&FindingGroup> = all_findings
            .iter()
            .filter(|f| f.is_component_issue && f.criticality_tier == CriticalityTier::Optimization)
            .collect();

        let local_mandatory: Vec<&FindingGroup> = all_findings
            .iter()
            .filter(|f| !f.is_component_issue && f.criticality_tier == CriticalityTier::Mandatory)
            .collect();

        let local_optimization: Vec<&FindingGroup> = all_findings
            .iter()
            .filter(|f| {
                !f.is_component_issue && f.criticality_tier == CriticalityTier::Optimization
            })
            .collect();

        // Overview row so the classification header carries substance instead of
        // sitting alone on an otherwise blank page. One tile per rendered
        // category below (rather than two combined totals) so each number
        // maps 1:1 to the heading it belongs to — a combined "Einzelfälle"
        // total previously hid the fact that its optimization share is
        // rendered chapters later under a differently-named heading (#local
        // vs. #systemic classification page).
        builder = builder.add_component(
            MetricStrip::new(vec![
                MetricStripItem::new(
                    if en {
                        "Systemic (mandatory)"
                    } else {
                        "Systemisch (Pflicht)"
                    },
                    systemic_mandatory.len().to_string(),
                )
                .with_accent(if !systemic_mandatory.is_empty() {
                    design::tokens::INFO
                } else {
                    design::tokens::SUCCESS
                }),
                MetricStripItem::new(
                    if en {
                        "Systemic (optimization)"
                    } else {
                        "Systemisch (Optimierung)"
                    },
                    systemic_optimization.len().to_string(),
                )
                .with_accent(design::tokens::NEUTRAL),
                MetricStripItem::new(
                    if en {
                        "Local (mandatory)"
                    } else {
                        "Lokal (Pflicht)"
                    },
                    local_mandatory.len().to_string(),
                )
                .with_accent(if !local_mandatory.is_empty() {
                    design::tokens::INFO
                } else {
                    design::tokens::SUCCESS
                }),
                MetricStripItem::new(
                    if en {
                        "Local (optimization)"
                    } else {
                        "Lokal (Optimierung)"
                    },
                    local_optimization.len().to_string(),
                )
                .with_accent(design::tokens::NEUTRAL),
            ])
            .compact(),
        );

        // Findings matrix (#570): a single compact table listing every finding
        // group across all four buckets, sorted worst-first, so a reader can
        // prioritize and locate a specific finding without paging through the
        // full set of detail cards that follow below. Purely additive — does
        // not replace or alter the per-category cards rendered afterward.
        const FINDINGS_MATRIX_MAX_ROWS: usize = 30;

        // In the shared finding order, like the JSON (plan 58).
        let matrix_findings: Vec<&FindingGroup> = all_findings.iter().collect();

        if !matrix_findings.is_empty() {
            let (matrix_title, matrix_intro) = if en {
                (
                    "Findings Matrix",
                    "One row per finding group, sorted by priority — an overview to prioritize by before the detail cards below expand on each row.",
                )
            } else {
                (
                    "Befundmatrix",
                    "Eine Zeile je Befundgruppe, sortiert nach Priorität – als Überblick zur Priorisierung, bevor die Detailkarten unten jede Zeile vertiefen.",
                )
            };
            builder = builder.add_component(Label::new(matrix_intro).with_size("10.5pt"));

            let mut table = AuditTable::new(vec![
                TableColumn::new(if en { "Priority" } else { "Priorität" }).with_width("10%"),
                TableColumn::new("WCAG").with_width("10%"),
                TableColumn::new(if en { "Finding" } else { "Befund" }).with_width("28%"),
                TableColumn::new(if en { "Scope" } else { "Umfang" }).with_width("14%"),
                TableColumn::new(if en { "Responsible" } else { "Zuständig" }).with_width("14%"),
                TableColumn::new(if en { "Action" } else { "Maßnahme" }).with_width("24%"),
            ])
            .with_title(matrix_title);

            for group in matrix_findings.iter().take(FINDINGS_MATRIX_MAX_ROWS) {
                let wcag = if group.wcag_criterion.is_empty() {
                    "n/a".to_string()
                } else if group.wcag_level.is_empty() {
                    group.wcag_criterion.clone()
                } else {
                    format!("{} ({})", group.wcag_criterion, group.wcag_level)
                };
                let scope = if group.is_component_issue {
                    if en {
                        format!("Systemic · {}", group.occurrence_count)
                    } else {
                        format!("Systemisch · {}", group.occurrence_count)
                    }
                } else if en {
                    format!("Local · {}", group.occurrence_count)
                } else {
                    format!("Lokal · {}", group.occurrence_count)
                };
                table = table.add_row(vec![
                    priority_label_i18n(group.priority, i18n),
                    wcag,
                    truncate_title(&group.title, 48),
                    scope,
                    role_label_i18n(group.responsible_role, i18n),
                    truncate_title(&group.recommendation, 72),
                ]);
            }
            builder = builder.add_component(table);

            let remaining = matrix_findings
                .len()
                .saturating_sub(FINDINGS_MATRIX_MAX_ROWS);
            if remaining > 0 {
                let note = if en {
                    format!("{remaining} further findings are listed in the detail cards below.")
                } else {
                    format!(
                        "{remaining} weitere Befunde sind in den Detailkarten unten aufgeführt."
                    )
                };
                builder = builder.add_component(
                    Label::new(note)
                        .with_size("9pt")
                        .with_color(design::tokens::NEUTRAL),
                );
            }
        }

        // The first rendered category flows directly under the classification
        // header; only later categories start on a fresh page.
        let mut rendered_categories = 0usize;

        // 1. Systemic Mandatory
        if !systemic_mandatory.is_empty() {
            let title = if en {
                "Systemic Template & Component Issues (WCAG A/AA)"
            } else {
                "Systemische Template- & Komponentenfehler (WCAG A/AA)"
            };
            let desc = if en {
                "This pattern suggests a reused component or template — confirming that requires evidence from additional pages. A central fix would likely also resolve the cause on other pages sharing the same pattern."
            } else {
                "Dieses Muster deutet auf eine wiederverwendete Komponente oder Vorlage hin — bestätigt ist das erst mit Belegen von weiteren Seiten. Eine zentrale Behebung würde die Ursache voraussichtlich auch auf anderen Seiten mit demselben Muster beheben."
            };
            if rendered_categories > 0 {
                builder = builder.add_component(PageBreak::new());
            }
            rendered_categories += 1;
            builder = builder.add_component(
                SectionHeaderSplit::new(title, desc)
                    .with_eyebrow(if en {
                        "SYSTEMIC COMPLIANCE"
                    } else {
                        "SYSTEMISCHE PFLICHT"
                    })
                    .with_level(2),
            );
            for group in systemic_mandatory {
                builder = render_finding_technical(
                    builder,
                    group,
                    i18n,
                    report_ts,
                    evidence_seq,
                    acronyms_expanded,
                );
            }
        }

        // 2. Systemic Optimization
        if !systemic_optimization.is_empty() {
            let title = if en {
                "Systemic Quality & SEO Optimizations"
            } else {
                "Systemische Qualitäts- & SEO-Optimierungen"
            };
            let desc = if en {
                "These recommendations recur within this single page, which suggests a shared template — confirming that would require checking additional pages."
            } else {
                "Diese Empfehlungen wiederholen sich innerhalb dieser einen Seite, was auf eine gemeinsame Vorlage hindeutet — bestätigt ist das erst nach Prüfung weiterer Seiten."
            };
            if rendered_categories > 0 {
                builder = builder.add_component(PageBreak::new());
            }
            rendered_categories += 1;
            builder = builder.add_component(
                SectionHeaderSplit::new(title, desc)
                    .with_eyebrow(if en {
                        "SYSTEMIC OPTIMIZATION"
                    } else {
                        "SYSTEMISCHE OPTIMIERUNG"
                    })
                    .with_level(2),
            );
            for group in systemic_optimization {
                builder = render_finding_technical(
                    builder,
                    group,
                    i18n,
                    report_ts,
                    evidence_seq,
                    acronyms_expanded,
                );
            }
        }

        // 3. Local Mandatory
        if !local_mandatory.is_empty() {
            let title = if en {
                "Local & Editorial Findings (WCAG A/AA)"
            } else {
                "Einzelfälle & Redaktionelle Befunde (WCAG A/AA)"
            };
            let desc = if en {
                "Single instances of accessibility barriers affecting specific pages, images, or editorial content, usually requiring individual resolution."
            } else {
                "Punktuelle Barrieren, die nur einzelne Seiten, spezifische Bilder oder redaktionelle Texte betreffen und meist individuell behoben werden müssen."
            };
            if rendered_categories > 0 {
                builder = builder.add_component(PageBreak::new());
            }
            rendered_categories += 1;
            builder = builder.add_component(
                SectionHeaderSplit::new(title, desc)
                    .with_eyebrow(if en {
                        "LOCAL COMPLIANCE"
                    } else {
                        "LOKALE PFLICHT"
                    })
                    .with_level(2),
            );
            for group in local_mandatory {
                builder = render_finding_technical(
                    builder,
                    group,
                    i18n,
                    report_ts,
                    evidence_seq,
                    acronyms_expanded,
                );
            }
        }

        // 4. Local Optimization
        if !local_optimization.is_empty() {
            let title = if en {
                "Additional Quality & SEO Recommendations"
            } else {
                "Ergänzende Qualitäts- & SEO-Empfehlungen"
            };
            let desc = if en {
                "Usability, performance, or SEO recommendations for specific pages."
            } else {
                "Ergänzende Empfehlungen zur Verbesserung der Ladezeiten, Suchmaschinenoptimierung und Benutzerfreundlichkeit auf bestimmten Seiten."
            };
            if rendered_categories > 0 {
                builder = builder.add_component(PageBreak::new());
            }
            rendered_categories += 1;
            builder = builder.add_component(
                SectionHeaderSplit::new(title, desc)
                    .with_eyebrow(if en {
                        "LOCAL OPTIMIZATION"
                    } else {
                        "LOKALE OPTIMIERUNG"
                    })
                    .with_level(2),
            );
            for group in local_optimization {
                builder = render_finding_technical(
                    builder,
                    group,
                    i18n,
                    report_ts,
                    evidence_seq,
                    acronyms_expanded,
                );
            }
        }
        let _ = rendered_categories; // last write is intentional; silence dead-store lint
    }
    builder
}

fn render_appendix_section(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    // Appendix — full violations list, conclusion of Part 2 (#246). Only render
    // the section header when there are violations to list; otherwise it
    // promises a list that never appears (#364).
    if vm.appendix.has_violations {
        let (appendix_title, appendix_intro) = if en {
            (
                "Complete Findings List",
                "Raw audit data and the full list of detected violations.",
            )
        } else {
            (
                "Vollständige Fundstellen",
                "Rohdaten des Audits und die vollständige Liste aller erkannten Verstöße.",
            )
        };
        builder = builder.add_component(PageBreak::new()).add_component(
            SectionHeaderSplit::new(appendix_title, appendix_intro)
                .with_eyebrow(if en { "APPENDIX" } else { "ANHANG" })
                .with_level(2),
        );

        builder = builder.add_component(build_cli_snapshot_table(vm, i18n));

        // JSON hint in appendix (#219)
        let json_note = if en {
            "The accompanying JSON report contains the complete machine-readable issue list with selectors, occurrences, and all detail data for automated processing."
        } else {
            "Der begleitende JSON-Report enthält die vollständige maschinenlesbare Fehlerliste mit Selektoren, Vorkommen und allen Detaildaten für automatisierte Weiterverarbeitung."
        };
        builder = builder.add_component(Callout::info(json_note).with_title(if en {
            "Raw data & processing"
        } else {
            "Rohdaten & Weiterverarbeitung"
        }));

        if vm.meta.report_level == ReportLevel::Technical {
            for v in &vm.appendix.violations {
                let mut desc = v.message.clone();
                if let Some(ref fix) = v.fix_suggestion {
                    desc.push_str(&format!("\n\nFix: {}", fix));
                }
                desc.push_str(&format!(
                    "\n\n{} Elemente betroffen",
                    v.affected_elements.len()
                ));
                let useful_selectors: Vec<&str> = v
                    .affected_elements
                    .iter()
                    .map(|e| e.selector.as_str())
                    .filter(|s| {
                        s.contains('.')
                            || s.contains('#')
                            || s.contains('[')
                            || s.contains('>')
                            || s.contains(' ')
                    })
                    .collect();
                if !useful_selectors.is_empty() {
                    desc.push_str(&format!("\nSelektoren: {}", useful_selectors.join(", ")));
                }
                builder = builder.add_component(Finding::new(
                    format!("{} — {}", v.rule, v.rule_name),
                    map_severity(&v.severity),
                    &desc,
                ));
            }
        } else {
            let rows: Vec<ChecklistRow> = vm
                .appendix
                .violations
                .iter()
                .map(|v| {
                    let status = match v.severity {
                        crate::wcag::Severity::Critical => "bad",
                        crate::wcag::Severity::High => "warn",
                        _ => "neutral",
                    };
                    ChecklistRow::new(format!("{} — {}", v.rule, v.rule_name), v.message.clone())
                        .with_status(status)
                })
                .collect();
            builder = builder.add_component(
                ChecklistPanel::new(rows).with_title(i18n.t("section-all-violations")),
            );
        }
    }
    builder
}

fn render_positive_signals_section(
    mut builder: renderreport::engine::ReportBuilder,
    signals: &[PositiveSignal],
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if signals.is_empty() {
        return builder;
    }

    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }

    let en = i18n.locale() == "en";
    let mut rows = Vec::new();
    for signal in signals {
        let status = if signal.strong {
            if en {
                "Strong"
            } else {
                "Stark"
            }
        } else if en {
            "Present"
        } else {
            "Vorhanden"
        };
        rows.push(
            ChecklistRow::new(&signal.title, format!("{}: {}", status, signal.description))
                .with_status(if signal.strong { "good" } else { "info" }),
        );
    }

    builder.add_component(ChecklistPanel::new(rows).with_title(if en {
        "Recognized structural patterns"
    } else {
        "Erkannte Strukturmuster"
    }))
}

fn render_dual_viewport_summary_section(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let has_scores = vm.cover.desktop_score.is_some() && vm.cover.mobile_score.is_some();
    let has_screenshot_status = report.page_screenshots.is_some()
        || matches!(
            report.screenshot_status,
            crate::audit::ScreenshotStatus::Failed(_)
        );
    if !has_scores && !has_screenshot_status {
        return builder;
    }

    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }

    let en = i18n.locale() == "en";
    if report.page_screenshots.is_some() {
        builder = builder.add_component(
            DevicePreview::new(
                super::PAGE_DESKTOP_SCREENSHOT_ASSET,
                super::PAGE_MOBILE_SCREENSHOT_ASSET,
            )
            .with_height(210.0),
        );
    }

    // A bordered panel here reads as its own boxed widget that then trails
    // into the blank rest of the (short) divider page — a compact strip
    // reads as part of the divider's own content instead, and names the
    // metric ("Accessibility-Score", not a bare "93/100") so it doesn't need
    // a caller to already know what dual-viewport scoring means.
    builder = builder.add_component(
        Label::new(if en {
            "Dual viewport summary"
        } else {
            "Dual-Viewport-Zusammenfassung"
        })
        .bold()
        .with_size("10.5pt"),
    );

    let mut items = Vec::new();
    if let (Some(desktop), Some(mobile)) = (vm.cover.desktop_score, vm.cover.mobile_score) {
        items.push(
            MetricStripItem::new(
                if en {
                    "Accessibility - Desktop"
                } else {
                    "Barrierefreiheit - Desktop"
                },
                format!("{desktop} / 100"),
            )
            .with_unit(score_range_label(desktop, en))
            .with_accent(design::score_color(desktop as u8)),
        );
        items.push(
            MetricStripItem::new(
                if en {
                    "Accessibility - Mobile"
                } else {
                    "Barrierefreiheit - Mobile"
                },
                format!("{mobile} / 100"),
            )
            .with_unit(score_range_label(mobile, en))
            .with_accent(design::score_color(mobile as u8)),
        );
        items.push(
            MetricStripItem::new(
                if en {
                    "Accessibility - overall"
                } else {
                    "Barrierefreiheit - Gesamt"
                },
                format!("{} / 100", vm.summary.score),
            )
            .with_unit(if en {
                "70/30 weighted"
            } else {
                "70/30 gewichtet"
            })
            .with_accent(design::score_color(vm.summary.score as u8)),
        );
    }
    if !has_scores
        && matches!(
            report.screenshot_status,
            crate::audit::ScreenshotStatus::Captured
        )
    {
        items.push(
            MetricStripItem::new(
                if en { "Preview" } else { "Vorschau" },
                if en { "Captured" } else { "Erfasst" },
            )
            .with_accent(design::tokens::SUCCESS),
        );
    }
    if !items.is_empty() {
        builder = builder.add_component(MetricStrip::new(items).compact());
    }

    if let crate::audit::ScreenshotStatus::Failed(reason) = &report.screenshot_status {
        builder = builder.add_component(Callout::warning(if en {
            format!("Screenshot capture failed: {reason}")
        } else {
            format!("Screenshot-Erfassung fehlgeschlagen: {reason}")
        }));
    }

    let occurrence_context = report
        .dual_viewport
        .as_ref()
        .map(|dual| {
            let desktop = dual.desktop.wcag_results.violations.len();
            let mobile = dual.mobile.wcag_results.violations.len();
            let combined = vm.severity.total;
            if en {
                format!("WCAG occurrences before cross-viewport consolidation: {desktop} on desktop and {mobile} on mobile. The normalized combined finding list contains {combined} occurrences; this count is used for evidence and remediation, not as a third score. ")
            } else {
                format!("WCAG-Vorkommen vor der ansichtsübergreifenden Zusammenführung: {desktop} auf Desktop und {mobile} auf Mobile. Die normalisierte gemeinsame Befundliste enthält {combined} Vorkommen; diese Anzahl dient Evidenz und Maßnahmenplanung, nicht als dritter Score. ")
            }
        })
        .unwrap_or_default();
    let explanation = if en {
        format!(
            "Each viewport accessibility score reflects the severity and variety of automatically detected WCAG findings; 100 means no issue was detected within the automated scope. Lower values mean more remediation pressure. {occurrence_context}The displayed accessibility overall score is calculated from mobile at 70% and desktop at 30%."
        )
    } else {
        format!(
            "Jeder Barrierefreiheits-Score bewertet Schwere und Vielfalt der automatisiert erkannten WCAG-Befunde seiner Ansicht; 100 bedeutet, dass im automatisierten Prüfumfang kein Problem erkannt wurde. Niedrigere Werte bedeuten höheren Handlungsdruck. {occurrence_context}Der ausgewiesene Barrierefreiheits-Gesamtwert wird mit 70 % Mobile und 30 % Desktop berechnet."
        )
    };

    builder.add_component(
        Label::new(explanation)
            .with_size("9pt")
            .with_color(design::tokens::MUTED),
    )
}

fn score_range_label(score: u32, en: bool) -> &'static str {
    crate::registry::SCORE_RANGE.label(score as f32, en)
}

pub(super) fn render_module_sections(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let mut is_first = true;
    let has_dual_viewport_scores =
        vm.cover.desktop_score.is_some() && vm.cover.mobile_score.is_some();
    let has_screenshot_status = report.page_screenshots.is_some()
        || matches!(
            report.screenshot_status,
            crate::audit::ScreenshotStatus::Failed(_)
        );
    if has_dual_viewport_scores || has_screenshot_status {
        builder = render_dual_viewport_summary_section(builder, vm, report, is_first, i18n);
        is_first = false;
    }
    if let Some(ref sx) = vm.module_details.search_experience {
        builder = render_search_experience(builder, sx, is_first, i18n);
        is_first = false;
    }

    // #14: Source Quality, AI Visibility and Content Visibility are merged into
    // one "KI & Vertrauen" / "AI & Trust" chapter so the three trust- and
    // discoverability-indicator modules read as one section instead of three
    // fragmented ones. They render as level-3 sub-sections under one opener.
    let en = i18n.locale() == "en";
    let mut ki_opened = false;
    let mut in_ki_chapter = false;
    for module in active_report_modules(report) {
        let key = module.module_key();
        let is_trust = matches!(
            key,
            "source_quality" | "ai_visibility" | "content_visibility"
        );
        if is_trust && !ki_opened {
            builder = builder.add_component(PageBreak::new()).add_component(
                SectionHeaderSplit::new(
                    if en { "AI & Trust" } else { "KI & Vertrauen" },
                    if en {
                        "Source quality, AI readability and content visibility — trust and discoverability indicators."
                    } else {
                        "Quellenqualität, KI-Lesbarkeit und Content-Sichtbarkeit — Vertrauens- und Auffindbarkeits-Indikatoren."
                    },
                )
                .with_eyebrow(if en { "TRUST" } else { "VERTRAUEN" })
                .with_level(2),
            );
            ki_opened = true;
            in_ki_chapter = true;
            is_first = true; // the trio's first sub-section flows under the opener
        } else if !is_trust && in_ki_chapter {
            // Close the trust chapter before the next, unrelated module.
            builder = builder.add_component(PageBreak::new());
            in_ki_chapter = false;
            is_first = true;
        }
        let (next_builder, rendered) =
            render_active_module_section(builder, key, vm, report, is_first, i18n);
        builder = next_builder;
        if rendered {
            is_first = false;
        }
    }

    builder
}

fn render_active_module_section(
    mut builder: renderreport::engine::ReportBuilder,
    module_key: &str,
    vm: &ReportViewModel,
    report: &AuditReport,
    is_first: bool,
    i18n: &I18n,
) -> (renderreport::engine::ReportBuilder, bool) {
    match module_key {
        "performance" => {
            if let Some(ref perf) = vm.module_details.performance {
                builder = render_performance(builder, perf, is_first, i18n);
                if !report.experience.budget_violations.is_empty() {
                    builder = render_budget_violations(
                        builder,
                        &report.experience.budget_violations,
                        i18n,
                    );
                }
                return (builder, true);
            }
        }
        "seo" => {
            if let Some(ref seo) = vm.module_details.seo {
                return (render_seo(builder, seo, is_first, i18n), true);
            }
        }
        "security" => {
            if let Some(ref sec) = vm.module_details.security {
                return (render_security(builder, sec, is_first, i18n), true);
            }
        }
        "html_conform" => {
            if let Some(ref hc) = vm.module_details.html_conform {
                return (render_html_conform(builder, hc, is_first, i18n), true);
            }
        }
        "commerce" => {
            if let Some(ref c) = vm.module_details.commerce {
                return (render_commerce(builder, c, is_first, i18n), true);
            }
        }
        "mobile" => {
            if let Some(ref mobile) = vm.module_details.mobile {
                return (render_mobile(builder, mobile, is_first, i18n), true);
            }
        }
        "ux" => {
            if let Some(ref ux) = vm.module_details.ux {
                return (render_ux(builder, ux, is_first, i18n), true);
            }
        }
        "journey" => {
            if let Some(ref journey) = vm.module_details.journey {
                return (render_journey(builder, journey, is_first, i18n), true);
            }
        }
        "dark_mode" => {
            if let Some(ref dm) = vm.module_details.dark_mode {
                return (render_dark_mode(builder, dm, is_first, i18n), true);
            }
        }
        "design_quality" => {
            if let Some(ref dq) = vm.module_details.design_quality {
                return (render_design_quality(builder, dq, is_first, i18n), true);
            }
        }
        "ai_transparency" => {
            if let Some(ref at) = vm.module_details.ai_transparency {
                return (render_ai_transparency(builder, at, is_first, i18n), true);
            }
        }
        "network_dns" => {
            if let Some(ref dns) = vm.module_details.network_dns {
                return (render_network_dns(builder, dns, is_first, i18n), true);
            }
        }
        "source_quality" => {
            if let Some(ref sq) = vm.module_details.source_quality {
                return (render_source_quality(builder, sq, is_first, i18n), true);
            }
        }
        "ai_visibility" => {
            if let Some(ref av) = vm.module_details.ai_visibility {
                return (render_ai_visibility(builder, av, is_first, i18n), true);
            }
        }
        "content_visibility" => {
            if let Some(ref cv) = vm.module_details.content_visibility {
                return (render_content_visibility(builder, cv, is_first, i18n), true);
            }
        }
        "best_practices" => {
            if let Some(ref bp) = vm.module_details.best_practices {
                return (render_best_practices(builder, bp, is_first, i18n), true);
            }
        }
        "tech_stack" => {
            if let Some(ref ts) = vm.module_details.tech_stack {
                return (render_tech_stack(builder, ts, is_first, i18n), true);
            }
        }
        "patterns" if !vm.positive_signals.is_empty() => {
            return (
                render_positive_signals_section(builder, &vm.positive_signals, is_first, i18n),
                true,
            );
        }
        _ => {}
    }

    (builder, false)
}

/// Shorten a finding title for inline display next to a letter code (chart
/// labels, table cells) — same char-based ellipsis truncation pattern used
/// elsewhere in the PDF layer.
pub(super) fn truncate_title(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>()
        + "…"
}

#[cfg(test)]
mod top_measure_tests {
    use super::*;

    fn measure(module: &str, title: &str, detail: &str) -> TopMeasure {
        TopMeasure {
            module: module.to_string(),
            title: title.to_string(),
            detail: detail.to_string(),
        }
    }

    #[test]
    fn collapsed_measure_reports_how_many_it_stands_for() {
        let rendered =
            measure("Sicherheit", "Content-Security-Policy", "unsafe-inline").render(1, 2, false);
        assert_eq!(
            rendered,
            "1. Sicherheit — Content-Security-Policy: unsafe-inline (+2 weitere gleicher Art)"
        );
    }

    #[test]
    fn single_measure_has_no_collapse_suffix() {
        let rendered =
            measure("Security", "Content-Security-Policy", "unsafe-inline").render(3, 0, true);
        assert_eq!(
            rendered,
            "3. Security — Content-Security-Policy: unsafe-inline"
        );
    }

    /// Performance and technical-SEO measures carry no separate title; the
    /// recommendation is the whole text and must not gain a stray colon.
    #[test]
    fn titleless_measure_renders_without_separator() {
        let rendered = measure("Performance", "", "DOM-Struktur verschlanken.").render(2, 0, false);
        assert_eq!(rendered, "2. Performance — DOM-Struktur verschlanken.");
    }
}

#[cfg(test)]
pub(super) mod risk_and_strength_tests {
    use super::*;

    use crate::audit::management_risk::ManagementRiskKind;

    fn rows(kinds: &[ManagementRiskKind], en: bool) -> Vec<DimensionRow> {
        kinds
            .iter()
            .map(|kind| DimensionRow {
                label: kind.dimension(en),
                description: kind.rationale(en),
                status: kind.tier().status(),
            })
            .collect()
    }

    /// #576 acceptance scenario 1, carried over to the shared risk
    /// dimensions (plan 35): a mixed report must keep genuinely good
    /// dimensions out of the risks bucket. The cited bug was a positive SEO
    /// line ("Sehr gute Auffindbarkeit …") rendered under "Wichtigste Risiken".
    #[test]
    fn mixed_report_buckets_bad_dimensions_as_risks_and_good_ones_as_strengths() {
        let kinds = [
            ManagementRiskKind::Legal {
                legal_flags: 2,
                critical: 2,
                high: 1,
                blocking_issues: 0,
            },
            ManagementRiskKind::Conversion {
                accessibility: 45,
                performance: Some(95),
                mobile: Some(95),
                blocking_issues: 0,
            },
            ManagementRiskKind::Seo { score: Some(95) },
            ManagementRiskKind::PerformanceMobile {
                performance: Some(95),
                mobile: Some(95),
            },
        ];
        let dimensions = rows(&kinds, false);

        let risks: Vec<&DimensionRow> = dimensions.iter().filter(|d| d.status != "good").collect();
        let strengths: Vec<&DimensionRow> =
            dimensions.iter().filter(|d| d.status == "good").collect();

        assert_eq!(risks.len(), 2, "legal and conversion should be risks");
        assert!(
            risks
                .iter()
                .all(|d| d.label != ManagementRiskKind::Seo { score: Some(95) }.dimension(false)),
            "a good SEO score must never appear in the risks bucket",
        );
        assert_eq!(
            strengths.len(),
            2,
            "SEO and performance/mobile should be strengths",
        );
    }

    /// #576 acceptance scenario 2: a report where nothing is a risk must be
    /// reachable — every dimension needs a reachable "good" branch, otherwise
    /// the zero-risk empty state could never render.
    #[test]
    fn clean_report_puts_every_dimension_in_strengths() {
        let kinds = [
            ManagementRiskKind::Legal {
                legal_flags: 0,
                critical: 0,
                high: 0,
                blocking_issues: 0,
            },
            ManagementRiskKind::AssistiveTechnology {
                critical: 0,
                high: 0,
                blocking_issues: 0,
            },
            ManagementRiskKind::Conversion {
                accessibility: 95,
                performance: Some(95),
                mobile: Some(95),
                blocking_issues: 0,
            },
            ManagementRiskKind::Seo { score: Some(95) },
            ManagementRiskKind::PerformanceMobile {
                performance: Some(95),
                mobile: Some(95),
            },
            ManagementRiskKind::Trust {
                accessibility: 95,
                critical: 0,
                high: 0,
            },
            ManagementRiskKind::Project {
                count: 0,
                pages: None,
            },
        ];
        let dimensions = rows(&kinds, false);
        assert!(
            dimensions.iter().all(|d| d.status == "good"),
            "all dimensions should be good, got: {:?}",
            dimensions
                .iter()
                .map(|d| (&d.label, d.status))
                .collect::<Vec<_>>(),
        );
    }

    /// Ordering: "bad" sorts before "warn", stably within each group.
    #[test]
    fn risks_partition_sorts_bad_before_warn_stably() {
        let kinds = [
            ManagementRiskKind::Legal {
                legal_flags: 1,
                critical: 0,
                high: 0,
                blocking_issues: 0,
            },
            ManagementRiskKind::AssistiveTechnology {
                critical: 0,
                high: 1,
                blocking_issues: 0,
            },
            ManagementRiskKind::Seo { score: Some(50) },
            ManagementRiskKind::PerformanceMobile {
                performance: Some(70),
                mobile: Some(95),
            },
        ];
        let (mut risks, _strengths): (Vec<DimensionRow>, Vec<DimensionRow>) = rows(&kinds, true)
            .into_iter()
            .partition(|d| d.status != "good");
        risks.sort_by_key(|d| if d.status == "bad" { 0u8 } else { 1u8 });

        let statuses: Vec<&str> = risks.iter().map(|d| d.status).collect();
        assert_eq!(statuses, vec!["bad", "bad", "warn", "warn"]);
        // Stable within "bad": Legal came before SEO in the input order.
        assert_eq!(risks[0].label, "Legal / BFSG-EAA");
        assert_eq!(risks[1].label, "SEO / visibility");
    }

    pub(in crate::output::pdf) fn test_report_view_model() -> ReportViewModel {
        let mut results = crate::wcag::WcagResults::new();
        results.nodes_checked = 50;
        results.passes = 10;
        results.add_violation(
            crate::wcag::Violation::new(
                "1.1.1",
                "Non-text Content",
                crate::cli::WcagLevel::A,
                crate::wcag::Severity::High,
                "Image missing alt",
                "node-img",
            )
            .with_selector("img.hero")
            .with_html_snippet("<img class=\"hero\">")
            .with_fix("Add alt"),
        );
        let report = AuditReport::new(
            "https://example.com".into(),
            crate::cli::WcagLevel::AA,
            results,
            100,
        );
        let normalized = crate::audit::normalized::normalize(&report);
        let config = ReportConfig::default();
        crate::output::builder::build_view_model(&normalized, &config)
    }

    #[test]
    fn appendix_table_does_not_duplicate_interpretation_when_card_context_matches() {
        // plan/32-appendix-duplicated-interpretation-sentence.md: Search
        // Experience's dashboard card falls back to `card_context ==
        // interpretation` when it has no distinct short blurb (no derived
        // warning) — the appendix row must then print the sentence once,
        // not as "score / 100 — sentence. sentence".
        let mut vm = test_report_view_model();
        vm.modules.dashboard = vec![crate::output::report_model::ModuleScore {
            name: "Sichtbarkeit & Nutzerverständnis".into(),
            score: 56,
            measurement_type: "composite".into(),
            interpretation:
                "Eingeschränkte Search Experience: Die Seite ist nicht ausreichend verständlich."
                    .into(),
            card_context:
                "Eingeschränkte Search Experience: Die Seite ist nicht ausreichend verständlich."
                    .into(),
            score_context: String::new(),
            key_lever: String::new(),
            good_threshold: 75,
            warn_threshold: 50,
        }];
        let i18n = I18n::new("de").unwrap();

        let table = build_cli_snapshot_table(&vm, &i18n);
        let row = table
            .rows
            .iter()
            .find(|r| r.get(1).and_then(|v| v.as_str()) == Some("Sichtbarkeit & Nutzerverständnis"))
            .expect("search experience row must be present");
        let value = row[2].as_str().unwrap();

        assert_eq!(
            value.matches("Eingeschränkte Search Experience").count(),
            1,
            "interpretation sentence must appear exactly once, got: {value}"
        );
    }

    #[test]
    fn part_divider_part_1_and_2_do_not_have_trailing_break() {
        let i18n = I18n::new("de").unwrap();
        let b1 = renderreport::engine::ReportBuilder::new("single");
        let r1 = render_part_divider(b1, 1, "T1", "I1", "Z1", "B1", &i18n).build();
        assert_eq!(r1.components.len(), 3);

        let b2 = renderreport::engine::ReportBuilder::new("single");
        let r2 = render_part_divider(b2, 2, "T2", "I2", "Z2", "B2", &i18n).build();
        assert_eq!(r2.components.len(), 4);
    }

    #[test]
    fn part_divider_part_3_acts_as_standalone_trennseite() {
        let i18n = I18n::new("de").unwrap();
        let b3 = renderreport::engine::ReportBuilder::new("single");
        let r3 = render_part_divider(b3, 3, "T3", "I3", "Z3", "B3", &i18n).build();
        assert_eq!(r3.components.len(), 6);
        let json = serde_json::to_string(&r3.components).unwrap();
        assert!(json.contains("TECHNISCHER ANHANG"));
        assert!(json.contains("divider"));
    }

    #[test]
    fn part_divider_part_3_english_eyebrow() {
        let i18n = I18n::new("en").unwrap();
        let b3 = renderreport::engine::ReportBuilder::new("single");
        let r3 = render_part_divider(b3, 3, "T3", "I3", "Z3", "B3", &i18n).build();
        let json = serde_json::to_string(&r3.components).unwrap();
        assert!(json.contains("TECHNICAL APPENDIX"));
    }
}
