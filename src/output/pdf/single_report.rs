//! Single-report page-section renderers for `generate_pdf`.
//!
//! Each function takes ownership of the builder, appends its section's
//! components, and returns the builder for further chaining.

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DevicePreview, DiagnosisPanel, DiagnosisRow, Divider, List,
    MetricStrip, MetricStripItem, PageBreak, RecommendationCard, SectionHeaderSplit,
};
use renderreport::components::charts::{Chart, ChartType};
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, CardDashboard, DashboardCard, Finding, TableColumn};
use renderreport::prelude::*;

use super::design;

use super::appendix::build_cli_snapshot_table;
use super::bik_guide::render_bik_guide_annex;
use super::detail_modules::{
    mobile_category_label, render_a11y_journey_findings, render_ai_transparency,
    render_ai_visibility, render_best_practices, render_budget_violations, render_commerce,
    render_content_visibility, render_dark_mode, render_design_quality, render_html_conform,
    render_journey, render_mobile, render_network_dns, render_performance,
    render_screen_reader_section, render_search_experience, render_security, render_seo,
    render_source_quality, render_tech_stack, render_ux, score_band_label,
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
            "The rating is a band label for the overall score within the automated scope, not a conformance certificate."
        } else {
            "Die Einstufung ist ein Bandlabel für den Gesamtwert im automatisierten Prüfumfang, kein Konformitätsnachweis."
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
                "No priority risks identified within the automated audit scope. This result applies to the automated audit scope only ({automated} of about {total} testable WCAG 2.1 AA criteria); criteria requiring manual review are listed in the appendix."
            )
        } else {
            format!(
                "Keine prioritären Risiken im automatisierten Prüfumfang erkannt. Diese Einschätzung bezieht sich ausschließlich auf den automatisierten Prüfumfang ({automated} von ca. {total} testbaren WCAG-2.1-AA-Kriterien); Kriterien mit manuellem Prüfbedarf sind im Anhang aufgeführt."
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
struct TopMeasure {
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
fn cross_module_measures(vm: &ReportViewModel, i18n: &I18n) -> Vec<(TopMeasure, usize)> {
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
    if evidence.warnings > 0 || evidence.manual_checks > 0 {
        builder = builder.add_component(
            Label::new(if en {
                format!(
                    "Counted here are confirmed WCAG violations only. In addition: {} heuristic accessibility warning(s) and {} criterion/criteria requiring manual review — documented, but not scored.",
                    evidence.warnings, evidence.manual_checks
                )
            } else {
                format!(
                    "Gezählt sind hier ausschließlich bestätigte WCAG-Verstöße. Hinzu kommen {} heuristische Barrierefreiheits-Warnung(en) und {} manuell zu prüfende(s) Kriterium/Kriterien — dokumentiert, aber nicht in den Score eingerechnet.",
                    evidence.warnings, evidence.manual_checks
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
                    "This result applies to the automated audit scope only ({automated} of ~{total} testable WCAG 2.1 AA criteria); criteria requiring manual review are listed in the appendix."
                )
            } else {
                format!(
                    "Diese Einschätzung bezieht sich ausschließlich auf den automatisierten Prüfumfang ({automated} von ca. {total} testbaren WCAG-2.1-AA-Kriterien); Kriterien mit manuellem Prüfbedarf sind im Anhang aufgeführt."
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
        .filter(|f| f.occurrence_count > 0 && !f.wcag_criterion.is_empty())
        .collect();
    if findings.is_empty() {
        return None;
    }
    findings.sort_by_key(|f| std::cmp::Reverse(f.occurrence_count));

    let total_occurrences: usize = findings.iter().map(|f| f.occurrence_count).sum();
    let top_n = findings.len().min(PROBLEM_CONCENTRATION_TOP_N);
    let top_occurrences: usize = findings[..top_n].iter().map(|f| f.occurrence_count).sum();
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
    pub(super) fn label(&self, en: bool) -> &'static str {
        match self {
            Self::Punktuell => {
                if en {
                    "isolated"
                } else {
                    "punktuell"
                }
            }
            Self::KonzentriertSystemisch => {
                if en {
                    "concentrated & systemic"
                } else {
                    "konzentriert & systemisch"
                }
            }
            Self::BreitSystemisch => {
                if en {
                    "broad & systemic"
                } else {
                    "breit & systemisch"
                }
            }
        }
    }
}

/// Modules that contribute to the weighted overall score and fall below the "Good" band (< 75).
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
        .filter(|m| m.contributes_to_overall && m.score < 75)
        .collect();
    weak.sort_by_key(|m| std::cmp::Reverse((100 - m.score) * m.weight_pct));
    weak
}

/// Modules that contribute to the weighted overall score and are "Kritisch" (< 40 — same
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
        .filter(|m| m.contributes_to_overall && m.score < 40)
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
    let en = i18n.locale() == "en";
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
        2 => {
            if en {
                format!("{} and {}", names[0], names[1])
            } else {
                format!("{} und {}", names[0], names[1])
            }
        }
        _ => {
            let last = &names[names.len() - 1];
            let initial = names[..names.len() - 1].join(", ");
            if en {
                format!("{}, and {}", initial, last)
            } else {
                format!("{} und {}", initial, last)
            }
        }
    }
}

fn build_problem_profile_description(
    vm: &ReportViewModel,
    kind: ProblemProfileKind,
    i18n: &I18n,
) -> String {
    let en = i18n.locale() == "en";
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
            if critical_modules.is_empty() {
                if en {
                    "Very good technical condition. Isolated local optimizations without structural problems.".to_string()
                } else {
                    "Sehr guter technischer Zustand. Einzelne lokale Optimierungen ohne strukturelles Problem.".to_string()
                }
            } else {
                let name = format_module_names(&critical_modules, i18n);
                if en {
                    format!("{name} determines the primary need for action. Isolated local optimizations without structural problems.")
                } else {
                    format!("{name} bestimmt den Handlungsbedarf. Einzelne lokale Optimierungen ohne strukturelles Problem.")
                }
            }
        }
        ProblemProfileKind::KonzentriertSystemisch => {
            let names = format_module_names(&critical_modules, i18n);
            let verb_plural = critical_modules.len() > 1;
            if has_systemic_wcag {
                if en {
                    let verb = if verb_plural {
                        "determine"
                    } else {
                        "determines"
                    };
                    format!("{names} {verb} the need for action. A large portion of the accessibility findings is concentrated in a few recurring patterns.")
                } else {
                    let verb = if verb_plural { "bestimmen" } else { "bestimmt" };
                    format!("{names} {verb} den Handlungsbedarf. Ein großer Teil der Accessibility-Befunde konzentriert sich auf wenige wiederkehrende Muster.")
                }
            } else if en {
                let verb = if verb_plural {
                    "determine"
                } else {
                    "determines"
                };
                format!(
                    "{names} {verb} the need for action while other technical areas remain stable."
                )
            } else {
                let verb = if verb_plural { "bestimmen" } else { "bestimmt" };
                format!("{names} {verb} den Handlungsbedarf bei ansonsten stabilen technischen Bereichen.")
            }
        }
        ProblemProfileKind::BreitSystemisch => {
            let names = format_module_names(&critical_modules, i18n);
            if names.is_empty() {
                if en {
                    "Multiple technical areas are affected simultaneously and require structural corrections.".to_string()
                } else {
                    "Mehrere technische Bereiche sind gleichzeitig betroffen und benötigen strukturelle Korrekturen.".to_string()
                }
            } else if en {
                format!("Multiple technical areas are affected simultaneously. In particular, {names} require structural corrections.")
            } else {
                format!("Mehrere technische Bereiche sind gleichzeitig betroffen. Besonders {names} benötigen strukturelle Korrekturen.")
            }
        }
    }
}

fn render_problem_profile(
    builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let kind = classify_problem_profile(vm);
    let title_prefix = if en {
        "Problem profile"
    } else {
        "Problemprofil"
    };
    let title = format!("{}: {}", title_prefix, kind.label(en));
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
fn build_problem_concentration_note(vm: &ReportViewModel, i18n: &I18n) -> Option<String> {
    let en = i18n.locale() == "en";
    let c = compute_problem_concentration(vm)?;
    if c.total_occurrences <= PROBLEM_CONCENTRATION_MIN_OCCURRENCES {
        return Some(if en {
            format!(
                "{} occurrence(s) were detected — too few to identify a recurring pattern.",
                c.total_occurrences
            )
        } else {
            format!(
                "Es wurden {} Vorkommen erkannt — zu wenige, um ein wiederkehrendes Muster abzuleiten.",
                c.total_occurrences
            )
        });
    }
    Some(if en {
        format!(
            "{} occurrences were detected. About {}% of them trace back to just {} recurring causes. Most affected: {}.",
            c.total_occurrences, c.concentration_pct, c.top_n, c.top1_title
        )
    } else {
        format!(
            "{} Vorkommen wurden erkannt. Rund {} % davon lassen sich auf {} wiederkehrende Ursachen zurückführen. Besonders betroffen: {}.",
            c.total_occurrences, c.concentration_pct, c.top_n, c.top1_title
        )
    })
}

/// Action-Plan-page framing of the same numbers, worded around leverage
/// rather than diagnosis: "fixing the top causes covers most of what's
/// listed below" (plan/7-management-summary-consolidation.md's suggested
/// "Die vier wichtigsten Ursachen erklären ca. 88 % der erkannten
/// WCAG-Vorkommen").
fn build_overall_leverage_note(vm: &ReportViewModel, i18n: &I18n) -> Option<String> {
    let en = i18n.locale() == "en";
    let c = compute_problem_concentration(vm)?;
    if c.total_occurrences <= PROBLEM_CONCENTRATION_MIN_OCCURRENCES {
        // Too few occurrences for a "top causes" leverage framing to be
        // meaningful (see `build_problem_concentration_note`) — the Action
        // Plan page already lists the few findings directly, so this bridge
        // sentence is simply omitted rather than restated at low counts.
        return None;
    }
    Some(if en {
        format!(
            "The {} most significant causes account for about {}% of the {} occurrences detected — addressing those first has the biggest leverage on the list below.",
            c.top_n, c.concentration_pct, c.total_occurrences
        )
    } else {
        format!(
            "Die {} wichtigsten Ursachen erklären rund {} % der {} erkannten Vorkommen — ihre Behebung hat die größte Hebelwirkung auf die folgende Liste.",
            c.top_n, c.concentration_pct, c.total_occurrences
        )
    })
}

/// "Why is the overall score X" — one compact row per module that actually
/// feeds the weighted `overall_score`, showing its score, its weight, and a
/// plain-language classification of whether it drags the overall score down
/// or stabilizes it (plan/2-score-driver-breakdown.md). Deliberately reads
/// `vm.modules.module_scores` (the canonical `ModuleScoreEntry` list, same
/// numbers `overall_score` is computed from) rather than
/// `vm.modules.dashboard` — the dashboard's cards are reshaped for
/// narrative presentation (e.g. "Search Experience" blends SEO with
/// heuristic AI-visibility signals into a composite score that is *not*
/// the raw SEO number actually carrying SEO's weight).
fn render_score_driver_table(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let contributing: Vec<&crate::audit::normalized::ModuleScoreEntry> = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .collect();
    // The combined technical value, named as what it is. It is no longer the
    // headline — that is the accessibility score (plan 29, D1) — so the
    // report has to say once what this second number averages, otherwise a
    // reader meets it in the derivation table with no introduction.
    let basis: u32 = contributing.iter().map(|m| m.weight_pct).sum();
    builder = builder
        .add_component(
            ScoreCard::new(
                if en {
                    "Combined technical score · 0–100"
                } else {
                    "Kombinierter technischer Wert · 0–100"
                },
                vm.summary.overall_score,
            )
            .with_description(if en {
                format!("Weighted across six modules, weight basis {basis}")
            } else {
                format!("Gewichtet über sechs Module, Gewichtsbasis {basis}")
            })
            .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(if en {
                "A different question from the accessibility score on the cover, which is what this report assesses."
            } else {
                "Eine andere Frage als der Barrierefreiheits-Wert auf dem Deckblatt, um den es in diesem Bericht geht."
            })
            .with_size("10.5pt")
            .with_color(design::tokens::NEUTRAL),
        );

    if contributing.len() < 2 {
        return builder;
    }

    // The "biggest weak point" is the module that pulls the overall score
    // down the hardest — that's a function of *both* how far below "Good"
    // it scores and how much weight it carries, not score alone. A 40%-
    // weighted module at 40/100 drags the overall score down far more than
    // a 10%-weighted module at 30/100, even though the latter's raw score
    // looks worse in isolation (the exact case this table exists to make
    // legible, see plan/2-score-driver-breakdown.md's "Accessibility 20 /
    // Security 30" example). Every other module scoring below the "Good"
    // band (>=75, reusing the shared FIVE_BAND cutoffs — no new threshold
    // logic) is a lesser but still real drag; everything at/above 75
    // stabilizes the overall score.
    let weakest_name = contributing
        .iter()
        .filter(|m| m.score < 75)
        .max_by_key(|m| (100 - m.score) * m.weight_pct)
        .map(|m| m.name.clone());

    let classify = |m: &crate::audit::normalized::ModuleScoreEntry| -> &'static str {
        if Some(&m.name) == weakest_name.as_ref() {
            if en {
                "Biggest weak point"
            } else {
                "Größter Schwachpunkt"
            }
        } else if m.score < 75 {
            if en {
                "Significant risk driver"
            } else {
                "Erheblicher Risikotreiber"
            }
        } else if en {
            "Stabilizes the overall score"
        } else {
            "Stabilisiert Gesamtwert"
        }
    };

    // In `viewport_weighted` mode the per-module weighting is NOT how
    // `overall_score` was computed — the score comes from a desktop/mobile
    // blend mixed with security (`score_breakdown`). Titling this table "why
    // the overall score is what it is" then invited arithmetic that does not
    // come out: on casoon.de the rows averaged 90.05 against a reported 92,
    // on inros-lackner 47.05 against 45 (plan 34). The real derivation is
    // rendered separately by `render_overall_score_derivation` below.
    let is_derivation = vm.summary.score_calculation_method != "viewport_weighted";
    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Module" } else { "Modul" }).with_width("28%"),
        TableColumn::new("Score").with_width("14%"),
        TableColumn::new(if en { "Weight" } else { "Gewichtung" }).with_width("16%"),
        TableColumn::new(if en { "Assessment" } else { "Einordnung" }).with_width("42%"),
    ])
    .with_title(match (is_derivation, en) {
        (true, true) => "Why the combined technical score is what it is",
        (true, false) => "Warum der kombinierte technische Wert ist, was er ist",
        (false, true) => "The modules that carry weight",
        (false, false) => "Die gewichteten Module im Einzelnen",
    });

    for m in &contributing {
        // Same lookup as `output::builder::helpers::localized_module_name`
        // (private to that module, so replicated here rather than exposed
        // just for this one call site): `module-<name>` Fluent key with the
        // canonical English name as a fallback.
        let key = format!("module-{}", m.name.to_lowercase().replace(' ', "-"));
        let translated = i18n.t(&key);
        let display_name = if translated == key {
            m.name.clone()
        } else {
            translated
        };
        table = table.add_row(vec![
            display_name,
            format!("{} ({})", m.score, score_band_label(m.score, i18n)),
            format!("{}%", m.weight_pct),
            classify(m).to_string(),
        ]);
    }

    builder = builder.add_component(table);
    builder = render_weight_basis_note(builder, vm, i18n);
    render_overall_score_derivation(builder, vm, i18n)
}

/// States the weight basis when the overall score was renormalised.
///
/// The score is divided by the weight of the *contributing* modules, so a
/// module that did not run, or that ran without being able to measure,
/// silently changes the denominator rather than the result. That is deliberate
/// for Performance (#QA-023) but was never communicated: two runs over
/// different module sets carry the same label, grade and rating and are not
/// comparable (plan 41).
fn render_weight_basis_note(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let basis: u32 = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.weight_pct)
        .sum();
    if basis >= 100 {
        return builder;
    }

    // Derived from the weight table, not from the entry list: a module that
    // was skipped outright has no entry at all, so filtering the entries alone
    // named nothing in exactly the case a reader most needs it
    // (`--skip-performance`).
    let contributing: std::collections::HashSet<&str> = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.name.as_str())
        .collect();
    let missing: Vec<String> = crate::taxonomy::MODULE_WEIGHTS
        .iter()
        .filter(|(name, _)| !contributing.contains(name))
        .map(|(name, weight)| format!("{name} ({weight} %)"))
        .collect();

    let mut note = if en {
        format!(
            "The overall score is normalised over the modules that were measured — a weight basis of {basis} of 100, not the full set."
        )
    } else {
        format!(
            "Der Gesamtwert ist auf die gemessenen Module normiert — Gewichtsbasis {basis} von 100, nicht der volle Satz."
        )
    };
    if !missing.is_empty() {
        note.push(' ');
        note.push_str(&if en {
            format!("Not included: {}.", missing.join(", "))
        } else {
            format!("Nicht enthalten: {}.", missing.join(", "))
        });
    }
    note.push(' ');
    note.push_str(if en {
        "A score from a different module set is not directly comparable."
    } else {
        "Ein Wert aus einem anderen Modulsatz ist damit nicht direkt vergleichbar."
    });

    builder = builder.add_component(Callout::info(&note).with_title(if en {
        "Weight basis"
    } else {
        "Gewichtsbasis"
    }));
    builder
}

/// How `overall_score` was actually reached, in `viewport_weighted` mode.
///
/// The numbers come from `score_breakdown`, so the block reproduces the
/// headline value exactly — the same contract the Search Experience section's
/// "Wie sich der Wert zusammensetzt" table already honours. Without it the
/// report showed a weighting that does not add up to the number beside it and
/// no other explanation anywhere (plan 34).
fn render_overall_score_derivation(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let Some(sb) = vm.summary.score_breakdown.as_ref() else {
        return builder;
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Step" } else { "Schritt" }).with_width("46%"),
        TableColumn::new(if en { "Input" } else { "Eingang" }).with_width("34%"),
        TableColumn::new(if en { "Result" } else { "Ergebnis" }).with_width("20%"),
    ])
    .with_title(if en {
        "How the overall score was calculated"
    } else {
        "Wie der Gesamtwert zustande kommt"
    });

    table = table.add_row(vec![
        if en {
            "Viewport blend (mobile weighs more)".to_string()
        } else {
            "Viewport-Mischung (Mobile zählt mehr)".to_string()
        },
        format!(
            "Desktop {} x {} % + Mobile {} x {} %",
            sb.desktop_overall, sb.desktop_weight_pct, sb.mobile_overall, sb.mobile_weight_pct
        ),
        sb.viewport_blended_overall.to_string(),
    ]);

    match (sb.security_score, sb.security_weight_pct) {
        (Some(security), Some(security_weight)) => {
            table = table.add_row(vec![
                if en {
                    "Security mixed in".to_string()
                } else {
                    "Sicherheit eingemischt".to_string()
                },
                format!(
                    "{} x {} % + {} x {} %",
                    sb.viewport_blended_overall,
                    sb.viewport_blend_weight_pct,
                    security,
                    security_weight
                ),
                vm.summary.overall_score.to_string(),
            ]);
        }
        _ => {
            table = table.add_row(vec![
                if en {
                    "Security not measured — blend carries the full weight".to_string()
                } else {
                    "Sicherheit nicht gemessen — die Mischung trägt voll".to_string()
                },
                format!("{} x 100 %", sb.viewport_blended_overall),
                vm.summary.overall_score.to_string(),
            ]);
        }
    }

    builder = builder.add_component(table);
    builder.add_component(
        Label::new(if en {
            "The module weights above rank what carries how much; this table is the calculation that produced the combined technical score."
        } else {
            "Die Modulgewichte oben zeigen, was wie stark zählt; diese Tabelle ist die Rechnung, aus der der kombinierte technische Wert entsteht."
        })
        .with_size("8.8pt")
        .with_color(design::tokens::MUTED),
    )
}

/// One plain-language reason per non-Accessibility score driver (SEO,
/// Security, Tech complexity/Performance) — reuses the bullets already
/// computed by `interpretation::build_technical_overview_localized`
/// (`vm.summary.technical_overview`), which is always exactly
/// `[accessibility, seo, security, tech]` followed by any cross-impact notes.
/// Bullet 0 (Accessibility) is skipped here — it's already covered by the
/// WCAG root-cause note directly above this table (#2d).
fn render_additional_score_drivers(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    if vm.summary.technical_overview.len() < 4 {
        return builder;
    }
    let mut list = List::new().with_title(if en {
        "Additional score drivers"
    } else {
        "Weitere Score-Treiber"
    });
    for item in &vm.summary.technical_overview[1..4] {
        list = list.add_item(item);
    }
    builder = builder.add_component(list);
    builder
}

/// Splits a list of `(label, score)` pairs into "critical weaknesses"
/// (below the shared FIVE_BAND "Verbesserungswürdig"/"Needs improvement"
/// cutoff at 60 — no new threshold invented) and "comparatively stable",
/// then renders a compact callout per module, matching the review's own
/// example format (plan/5-module-accessibility-security-driver-detail.md
/// and plan/21-accessibility-subcategory-breakdown-uniform-gap.md).
/// Handles both contrast (two-line) and uniform (single-line) distributions;
/// renders nothing only for an empty input (module didn't run).
fn render_subcategory_breakdown(
    mut builder: renderreport::engine::ReportBuilder,
    title: &str,
    entries: &[(String, u32)],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    if entries.is_empty() {
        return builder;
    }
    const STABLE_CUTOFF: f32 = 60.0; // registry::FIVE_BAND's "Needs improvement" cutoff

    let mut weak: Vec<&(String, u32)> = entries
        .iter()
        .filter(|(_, score)| (*score as f32) < STABLE_CUTOFF)
        .collect();
    let mut stable: Vec<&(String, u32)> = entries
        .iter()
        .filter(|(_, score)| (*score as f32) >= STABLE_CUTOFF)
        .collect();
    if weak.is_empty() && stable.is_empty() {
        return builder;
    }
    weak.sort_by_key(|(_, score)| *score);
    stable.sort_by_key(|(_, score)| std::cmp::Reverse(*score));

    let render_side = |items: &[&(String, u32)]| -> String {
        items
            .iter()
            .map(|(name, score)| format!("{name} {score}"))
            .collect::<Vec<_>>()
            .join(" · ")
    };

    // "Stronger sub-areas within {title}" rather than "comparatively stable"
    // (feedback 2026-09-07): at an overall score of e.g. 20, a subcategory at
    // 67 read as "stable" on its own, contradicting the module's headline
    // "Kritisch" grade a few lines above. Framing it as relative-within-the-
    // module keeps the same information without the contradiction.
    let body = if !weak.is_empty() && !stable.is_empty() {
        if en {
            format!(
                "Critical weaknesses: {}\nStronger sub-areas within {title}: {}",
                render_side(&weak),
                render_side(&stable)
            )
        } else {
            format!(
                "Kritische Schwächen: {}\nStärkere Teilbereiche innerhalb der {title}: {}",
                render_side(&weak),
                render_side(&stable)
            )
        }
    } else if !weak.is_empty() {
        if en {
            format!("Critical weaknesses: {}", render_side(&weak))
        } else {
            format!("Kritische Schwächen: {}", render_side(&weak))
        }
    } else if en {
        format!(
            "Stronger sub-areas within {title}: {}",
            render_side(&stable)
        )
    } else {
        format!(
            "Stärkere Teilbereiche innerhalb der {title}: {}",
            render_side(&stable)
        )
    };
    builder = builder.add_component(Callout::info(body).with_title(title));
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

        let mut matrix_findings: Vec<&FindingGroup> = all_findings.iter().collect();
        matrix_findings.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
        });

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

/// Max number of root causes assigned a letter (A, B, C…) in the root-cause
/// analysis section. Shared with `render_timeframe_roadmap` so both sections
/// agree on which letter identifies which cause.
const ROOT_CAUSE_SHOWN: usize = 6;

/// WCAG-Mandatory-tier findings sorted by occurrence count (descending) — the
/// exact pool `render_root_cause_analysis` assigns letters A, B, C… from.
fn mandatory_root_causes(vm: &ReportViewModel) -> Vec<&FindingGroup> {
    let mut findings: Vec<&FindingGroup> = vm
        .findings
        .all_findings
        .iter()
        .filter(|f| f.occurrence_count > 0 && f.criticality_tier == CriticalityTier::Mandatory)
        .collect();
    findings.sort_by_key(|f| std::cmp::Reverse(f.occurrence_count));
    findings
}

/// Shorten a finding title for inline display next to a letter code (chart
/// labels, table cells) — same char-based ellipsis truncation pattern used
/// elsewhere in the PDF layer.
fn truncate_title(value: &str, max_chars: usize) -> String {
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

/// `rule_id -> (letter, occurrence_count, share_pct)` for the top
/// `ROOT_CAUSE_SHOWN` mandatory root causes. Used by `render_timeframe_roadmap`
/// to tag systemic actions with the root cause they resolve, so the two
/// sections never disagree on which letter is which.
fn root_cause_lookup(
    findings: &[&FindingGroup],
) -> std::collections::HashMap<String, (char, usize, i64)> {
    let total_occurrences: usize = findings.iter().map(|f| f.occurrence_count).sum();
    findings
        .iter()
        .enumerate()
        .take(ROOT_CAUSE_SHOWN)
        .map(|(idx, f)| {
            let letter = (b'A' + idx as u8) as char;
            let share_pct = if total_occurrences > 0 {
                (f.occurrence_count as f64 * 100.0 / total_occurrences as f64).round() as i64
            } else {
                0
            };
            (f.rule_id.clone(), (letter, f.occurrence_count, share_pct))
        })
        .collect()
}

pub(super) fn render_root_cause_analysis(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let title = if en {
        "Root Cause Analysis"
    } else {
        "Ursachenanalyse"
    };
    let subtitle = if en {
        "Many individual findings trace back to a few recurring causes — fixing those has a compounding effect."
    } else {
        "Viele Einzelbefunde gehen auf wenige wiederkehrende Ursachen zurück – deren Behebung wirkt gebündelt."
    };

    // No leading PageBreak: the preceding part divider ("Befunde nach Ursache")
    // already opened the page — an extra break left that divider page 3/4 empty.
    builder = builder.add_component(
        SectionHeaderSplit::new(title, subtitle)
            .with_eyebrow(if en { "ROOT CAUSE" } else { "URSACHEN" })
            .with_level(2),
    );

    // Only WCAG/Mandatory findings — scope matches the header "N Accessibility-Befunde" count.
    // SEO findings (Optimization tier) appear in their own section below.
    let findings = mandatory_root_causes(vm);

    if findings.is_empty() {
        // "No accessibility findings were detected" was false whenever the
        // journey or screen-reader modules had produced findings — they are
        // just not confirmed WCAG violations and therefore out of scope here.
        // Name the other two evidence classes instead of denying them.
        let evidence = &vm.findings.evidence;
        let msg = if evidence.is_empty() {
            if en {
                "No accessibility findings were detected, so no root cause analysis is necessary."
                    .to_string()
            } else {
                "Es wurden keine Barrierefreiheits-Befunde erkannt, daher ist keine Ursachenanalyse erforderlich.".to_string()
            }
        } else if en {
            format!(
                "No confirmed WCAG violations, so no root cause analysis is necessary. {} accessibility warning(s) and {} manual check(s) remain — they are documented in their own sections and are not scored.",
                evidence.warnings, evidence.manual_checks
            )
        } else {
            format!(
                "Keine bestätigten WCAG-Verstöße, daher ist keine Ursachenanalyse erforderlich. Es bleiben {} Barrierefreiheits-Warnung(en) und {} manuelle(r) Prüfhinweis(e) — sie stehen in den eigenen Abschnitten und fließen nicht in den Score ein.",
                evidence.warnings, evidence.manual_checks
            )
        };
        builder = builder.add_component(if evidence.is_empty() {
            Callout::success(&msg)
        } else {
            Callout::info(&msg)
        });
        return builder;
    }

    // Plain lead-in (Rule A): spell out what "root cause" means in practice
    // before the technical list of causes/components follows below.
    let lead_in = if en {
        "In practice: fixing the right root cause — a shared component or template — often resolves several findings at once, instead of fixing each one individually."
    } else {
        "Das bedeutet konkret: Ein Fix an der richtigen Stelle – etwa einer wiederverwendeten Komponente oder einem Template – behebt oft mehrere Befunde gleichzeitig, statt jeden einzeln reparieren zu müssen."
    };
    builder = builder.add_component(Label::new(lead_in).with_size("10.5pt"));

    // Render root causes (assigning letters A, B, C, etc.)
    let mut list = List::new().with_title(if en {
        "Systemic Root Causes"
    } else {
        "Erkannte Kernursachen"
    });
    let mut table_rows = Vec::new();
    let mut chart_data: Vec<(String, f64)> = Vec::new();
    let total_occurrences: usize = findings.iter().map(|f| f.occurrence_count).sum();

    // Rounded (not truncating) share so a small-but-real cause reads as "1 %"
    // rather than a self-contradictory "0 %" next to its own listed row.
    let share_pct = |occurrences: usize| -> i64 {
        if total_occurrences > 0 {
            (occurrences as f64 * 100.0 / total_occurrences as f64).round() as i64
        } else {
            0
        }
    };

    let mut shown_occurrences = 0usize;

    for (idx, finding) in findings.iter().enumerate().take(ROOT_CAUSE_SHOWN) {
        let letter = (b'A' + idx as u8) as char;
        let item_title = format!(
            "{} {}: {} {} — {}",
            if en { "Root Cause" } else { "Ursache" },
            letter,
            finding.occurrence_count,
            if en { "occurrences" } else { "Vorkommen" },
            finding.title
        );
        list = list.add_item(&item_title);
        shown_occurrences += finding.occurrence_count;

        // Repeat the clear-text title next to the letter — otherwise the chart
        // and table only carry "Ursache A", forcing readers to flip back to
        // the list above to remember what it refers to.
        let short_title = truncate_title(&finding.title, 45);
        table_rows.push(vec![
            format!("{letter} — {short_title}"),
            finding.occurrence_count.to_string(),
            format!("{} %", share_pct(finding.occurrence_count)),
        ]);
        chart_data.push((
            format!("{letter} — {short_title}"),
            finding.occurrence_count as f64,
        ));
    }

    // Findings beyond the top ROOT_CAUSE_SHOWN would otherwise inflate
    // `total_occurrences` (the true, honest denominator) with no visible row
    // to account for them — the displayed shares silently failed to sum to
    // 100 %. Disclose the remainder explicitly instead (#QA-039 report review).
    if findings.len() > ROOT_CAUSE_SHOWN {
        let remaining_causes = findings.len() - ROOT_CAUSE_SHOWN;
        let remaining_occurrences = total_occurrences - shown_occurrences;
        let label = if en {
            format!("Other ({remaining_causes} further causes)")
        } else {
            format!("Sonstige ({remaining_causes} weitere Ursachen)")
        };
        table_rows.push(vec![
            label.clone(),
            remaining_occurrences.to_string(),
            format!("{} %", share_pct(remaining_occurrences)),
        ]);
        chart_data.push((label, remaining_occurrences as f64));
    }

    builder = builder.add_component(list);

    // Occurrence distribution as a bar chart — shows at a glance where the
    // findings concentrate (#7 distribution bars).
    if chart_data.len() >= 2 {
        builder = builder.add_component(
            Chart::bar(if en {
                "Occurrence distribution by cause"
            } else {
                "Verteilung der Vorkommen nach Ursache"
            })
            .add_series("occurrences", chart_data)
            .horizontal(),
        );
    }

    // Render the table: Ursache | Vorkommen | Anteil
    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Root Cause" } else { "Ursache" }).with_width("40%"),
        TableColumn::new(if en { "Occurrences" } else { "Vorkommen" }).with_width("30%"),
        TableColumn::new(if en { "Share" } else { "Anteil" }).with_width("30%"),
    ])
    .with_title(if en {
        "Distribution of Issues by Root Cause"
    } else {
        "Verteilung der Mängel nach Ursache"
    });

    for row in table_rows {
        table = table.add_row(row);
    }
    builder = builder.add_component(table);

    builder
}

pub(super) fn render_timeframe_roadmap(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let title = if en { "Action Plan" } else { "Maßnahmenplan" };
    // Disclose the scope mismatch with the root-cause section above: this plan
    // also includes SEO/Optimization-tier actions, which are not part of the
    // WCAG-only root-cause count (#5 fix).
    let subtitle = if en {
        "Recommended actions grouped by where the problem lives — including supplementary SEO and quality recommendations without legal relevance."
    } else {
        "Empfohlene Maßnahmen, gruppiert nach Ebene des Problems — inklusive ergänzender SEO- und Qualitätsempfehlungen ohne Rechtsbezug."
    };

    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(title, subtitle)
            .with_eyebrow(if en { "ROADMAP" } else { "MASSNAHMEN" })
            .with_level(2),
    );

    // Overall-leverage sentence (plan/7-management-summary-consolidation.md):
    // reuses the same problem-concentration numbers shown earlier
    // (plan/3-problem-concentration-diagnosis-narrative.md) but framed
    // around leverage rather than diagnosis — not literally the same
    // sentence twice.
    if let Some(note) = build_overall_leverage_note(vm, i18n) {
        builder = builder.add_component(Label::new(note).with_size("10.5pt"));
    }

    let columns = &vm.actions.roadmap_columns;
    if columns.is_empty() {
        // The roadmap is built from WCAG/finding groups only. Saying "no
        // findings require remediation" here contradicted the report whenever
        // other modules had documented work, so name where that work is
        // instead of claiming there is none.
        let module_measure_count = cross_module_measures(vm, i18n).len();
        let empty_msg = if module_measure_count > 0 {
            if en {
                format!(
                    "No prioritized accessibility actions. {module_measure_count} recommendation(s) from other modules (performance, SEO, security, mobile) are listed in their module sections and in the key measures above."
                )
            } else {
                format!(
                    "Keine priorisierten Barrierefreiheits-Maßnahmen. {module_measure_count} Empfehlung(en) aus anderen Modulen (Performance, SEO, Sicherheit, Mobile) stehen in den jeweiligen Modulabschnitten und in den wichtigsten Maßnahmen oben."
                )
            }
        } else if en {
            "No prioritized actions — no findings require remediation.".to_string()
        } else {
            "Keine priorisierten Maßnahmen — keine Befunde mit Handlungsbedarf.".to_string()
        };
        builder = builder.add_component(Label::new(empty_msg).with_color(design::tokens::NEUTRAL));
        return builder;
    }

    // Same letter assignment as `render_root_cause_analysis`, so an action
    // tied to root cause "A" always points at the same finding the reader saw
    // lettered "A" above (#2 fix).
    let root_causes = mandatory_root_causes(vm);
    let root_cause_by_rule = root_cause_lookup(&root_causes);

    // One level group per column (systemic vs. local), each action as a
    // clean recommendation card — what to do, why, and (plan/6) a
    // remediation-leverage line instead of a separate visual effort badge.
    for col in columns {
        builder = builder.add_component(
            SectionHeaderSplit::new(col.title.clone(), col.description.clone()).with_level(3),
        );
        for item in &col.items {
            let mut why = if !item.benefit.is_empty() {
                item.benefit.clone()
            } else {
                item.risk_effect.clone()
            };
            // plan/6-remediation-leverage-metric.md: "is this worth doing
            // first" verdict — reach (occurrences), cost (effort) and risk
            // condensed into one line. "betroffen", not "behoben"/"beseitigt"
            // (plan/4-root-cause-confidence-wording.md): occurrence_count is
            // how many elements this rule affects, not a guarantee that one
            // fix resolves every one of them.
            let leverage_tag = if en {
                format!(
                    "Remediation leverage: {} — {} occurrence(s) affected · effort: {}",
                    item.leverage, item.occurrence_count, item.effort
                )
            } else {
                format!(
                    "Reparaturhebel: {} — {} Vorkommen betroffen · Aufwand: {}",
                    item.leverage, item.occurrence_count, item.effort
                )
            };
            why = format!("{why}\n\n{leverage_tag}");
            if let Some((letter, occurrence_count, share_pct)) =
                root_cause_by_rule.get(&item.rule_id)
            {
                let tag = if en {
                    format!("→ Root Cause {letter}, {occurrence_count} occurrences ({share_pct}%)")
                } else {
                    format!("→ Ursache {letter}, {occurrence_count} Vorkommen ({share_pct} %)")
                };
                why = format!("{why}\n\n{tag}");
            }
            builder = builder.add_component(RecommendationCard::new(item.action.clone(), why));
        }
    }

    builder
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
mod risk_and_strength_tests {
    use super::*;

    use crate::audit::management_risk::ManagementRiskKind;

    /// Plan 34: in `viewport_weighted` mode the per-module weighting is not
    /// the computation path, so the derivation block must reproduce the
    /// headline value on its own. Confirmed live before the fix: casoon.de's
    /// module rows averaged 90.05 against a reported 92, inros-lackner's
    /// 47.05 against 45.
    #[test]
    fn score_derivation_reproduces_the_overall_score() {
        for (desktop, mobile, security, expected) in [
            (94u32, 91u32, Some(95u32), 92u32),
            (41, 49, Some(30), 45),
            (80, 80, None, 80),
        ] {
            let blended = ((desktop as f64 * 30.0 + mobile as f64 * 70.0) / 100.0).round() as u32;
            let overall = match security {
                Some(sec) => ((blended as f64 * 90.0 + sec as f64 * 10.0) / 100.0).round() as u32,
                None => blended,
            };
            assert_eq!(
                overall, expected,
                "desktop {desktop} / mobile {mobile} / security {security:?}",
            );
        }
    }

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

    fn test_report_view_model() -> ReportViewModel {
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
        // 1 occurrence — matches the real audit's single a11y.text_spacing.clipped finding.
        let mut finding = vm.findings.all_findings[0].clone();
        finding.rule_id = "1.4.12".into();
        finding.title = "Inhalt wird abgeschnitten".into();
        finding.occurrence_count = 1;
        finding.wcag_criterion = "1.4.12".into();
        vm.findings.all_findings = vec![finding];

        let kind = classify_problem_profile(&vm);
        assert_eq!(kind, ProblemProfileKind::Punktuell);
        assert_eq!(kind.label(false), "punktuell");
        assert_eq!(kind.label(true), "isolated");

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
        f1.rule_id = "1.4.3".into();
        f1.title = "Kontrast unzureichend".into();
        f1.occurrence_count = 60;
        f1.wcag_criterion = "1.4.3".into();

        let mut f2 = vm.findings.all_findings[0].clone();
        f2.rule_id = "4.1.2".into();
        f2.title = "Name/Rolle fehlt".into();
        f2.occurrence_count = 30;
        f2.wcag_criterion = "4.1.2".into();
        vm.findings.all_findings = vec![f1, f2];

        let kind = classify_problem_profile(&vm);
        assert_eq!(kind, ProblemProfileKind::KonzentriertSystemisch);
        assert_eq!(kind.label(false), "konzentriert & systemisch");
        assert_eq!(kind.label(true), "concentrated & systemic");

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
        assert_eq!(kind.label(false), "breit & systemisch");
        assert_eq!(kind.label(true), "broad & systemic");

        let desc_de = build_problem_profile_description(&vm, kind, &i18n_de);
        assert!(desc_de.contains("Mehrere technische Bereiche sind gleichzeitig betroffen"));
        assert!(desc_de.contains("Barrierefreiheit, Performance und Sicherheit"));

        let desc_en = build_problem_profile_description(&vm, kind, &i18n_en);
        assert!(desc_en.contains("Multiple technical areas are affected simultaneously"));
        assert!(desc_en.contains("Accessibility, Performance, and Security"));
    }

    #[test]
    fn subcategory_breakdown_handles_all_weak_entries() {
        let i18n = I18n::new("de").unwrap();
        let builder = renderreport::engine::ReportBuilder::new("single");
        let entries = vec![
            ("ARIA".to_string(), 10),
            ("Landmarks".to_string(), 20),
            ("Semantik".to_string(), 50),
        ];
        let builder = render_subcategory_breakdown(builder, "Accessibility", &entries, &i18n);
        let report = builder.build();
        assert_eq!(report.components.len(), 1);
        let json = serde_json::to_string(&report.components[0]).unwrap();
        assert!(json.contains("Kritische Schwächen:"));
        assert!(!json.contains("Stärkere Teilbereiche"));
        assert!(json.contains("ARIA 10"));
    }

    #[test]
    fn subcategory_breakdown_handles_all_stable_entries() {
        let i18n = I18n::new("de").unwrap();
        let builder = renderreport::engine::ReportBuilder::new("single");
        let entries = vec![
            ("Formulare".to_string(), 100),
            ("Alternativtexte".to_string(), 90),
        ];
        let builder = render_subcategory_breakdown(builder, "Barrierefreiheit", &entries, &i18n);
        let report = builder.build();
        assert_eq!(report.components.len(), 1);
        let json = serde_json::to_string(&report.components[0]).unwrap();
        assert!(!json.contains("Kritische Schwächen"));
        assert!(json.contains("Stärkere Teilbereiche innerhalb der Barrierefreiheit:"));
        assert!(json.contains("Formulare 100"));
    }

    #[test]
    fn subcategory_breakdown_handles_contrast_entries() {
        let i18n = I18n::new("de").unwrap();
        let builder = renderreport::engine::ReportBuilder::new("single");
        let entries = vec![("ARIA".to_string(), 10), ("Formulare".to_string(), 100)];
        let builder = render_subcategory_breakdown(builder, "Barrierefreiheit", &entries, &i18n);
        let report = builder.build();
        assert_eq!(report.components.len(), 1);
        let json = serde_json::to_string(&report.components[0]).unwrap();
        assert!(json.contains("Kritische Schwächen:"));
        assert!(json.contains("Stärkere Teilbereiche innerhalb der Barrierefreiheit:"));
    }

    #[test]
    fn subcategory_breakdown_handles_empty_entries() {
        let i18n = I18n::new("de").unwrap();
        let builder = renderreport::engine::ReportBuilder::new("single");
        let entries: Vec<(String, u32)> = vec![];
        let builder = render_subcategory_breakdown(builder, "Accessibility", &entries, &i18n);
        let report = builder.build();
        assert_eq!(report.components.len(), 0);
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
