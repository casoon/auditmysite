//! WCAG coverage section for PDF reports (issue #37).

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, KeyValueList, SectionHeaderSplit,
};
use renderreport::components::text::Label;
use renderreport::components::TagCloud;

use crate::audit::{AccessibilityScorer, AuditReport, CoverageRatio};
use crate::i18n::I18n;

/// Render a WCAG coverage section (issue #37).
///
/// Lists the criteria this tool automatically checks, plus the criteria
/// that fundamentally require manual review. Communicates audit scope
/// transparently so users avoid a false sense of security.
pub(super) fn render_wcag_coverage_section(
    mut builder: renderreport::engine::ReportBuilder,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::wcag::coverage::{automated_criteria, coverage_stats, manual_review_criteria};

    let en = i18n.locale() == "en";

    // Principle coverage — informative secondary indicator (#99).
    let coverage =
        AccessibilityScorer::calculate_coverage(&report.accessibility.wcag_results.violations);
    let cov_title = if en {
        "Principle coverage (criteria passed)"
    } else {
        "Prinzip-Abdeckung (bestandene Kriterien)"
    };
    let fmt_ratio =
        |r: &CoverageRatio| format!("{}/{} ({:.0} %)", r.passed, r.total, r.ratio * 100.0);
    let (p_perc, p_op, p_und, p_rob) = if en {
        ("Perceivable", "Operable", "Understandable", "Robust")
    } else {
        ("Wahrnehmbar", "Bedienbar", "Verständlich", "Robust")
    };
    builder = builder.add_component(
        KeyValueList::new()
            .with_title(cov_title)
            .add(p_perc, fmt_ratio(&coverage.perceivable))
            .add(p_op, fmt_ratio(&coverage.operable))
            .add(p_und, fmt_ratio(&coverage.understandable))
            .add(p_rob, fmt_ratio(&coverage.robust)),
    );
    let (automated, total) = coverage_stats();
    // Criteria this tool checks automatically that are WCAG 2.2-only (didn't
    // exist in WCAG 2.1) — counted separately from `automated`/`total`, which
    // stay scoped to WCAG 2.1's 50 A/AA criteria (#572). Stating this
    // explicitly here, rather than leaving those criteria to appear as
    // undifferentiated "AA" entries, commits the report to its actual scope
    // decision: WCAG 2.1 AA, plus a small set of additional WCAG 2.2 AA
    // criteria this tool happens to already check.
    let wcag22_criteria: Vec<&str> = automated_criteria()
        .iter()
        .filter(|(id, _)| crate::wcag::coverage::is_wcag22_only(id))
        .map(|(id, _)| *id)
        .collect();
    let title = if en { "Audit scope" } else { "Prüfumfang" };
    let intro = if wcag22_criteria.is_empty() {
        if en {
            format!(
                "This audit covers {automated} of ~{total} testable WCAG 2.1 AA criteria automatically. The criteria listed below require manual review."
            )
        } else {
            format!(
                "Dieses Audit prüft {automated} von ca. {total} WCAG-2.1-AA-Kriterien automatisch. Die unten aufgeführten Kriterien benötigen manuelle Prüfung."
            )
        }
    } else {
        let wcag22_list = wcag22_criteria.join(", ");
        if en {
            format!(
                "This audit covers {automated} of ~{total} testable WCAG 2.1 AA criteria automatically, plus {} selected WCAG 2.2 AA criteria ({wcag22_list}, marked \"WCAG 2.2\" below and on individual findings) that are outside the WCAG 2.1-scoped ratio above. The criteria listed below require manual review.",
                wcag22_criteria.len()
            )
        } else {
            format!(
                "Dieses Audit prüft {automated} von ca. {total} WCAG-2.1-AA-Kriterien automatisch, ergänzt um {} ausgewählte WCAG-2.2-AA-Kriterien ({wcag22_list}, unten und bei einzelnen Befunden mit „WCAG 2.2\" gekennzeichnet), die außerhalb der oben genannten WCAG-2.1-Quote liegen. Die unten aufgeführten Kriterien benötigen manuelle Prüfung.",
                wcag22_criteria.len()
            )
        }
    };

    builder = builder.add_component(SectionHeaderSplit::new(title, &intro).with_level(2));

    // The cloud lists every criterion this tool checks; `automated` counts
    // only the WCAG 2.1 A/AA ones the ratio above is scoped to. Titling the
    // cloud with `automated` put "Automatisch geprüft (36)" above 56 entries,
    // with nothing reconciling the two numbers (plan 36 §1).
    let listed = automated_criteria().len();
    // Three disjoint groups, so they add up to `listed`. They must not
    // overlap: one AAA criterion is also WCAG 2.2-only, and counting it in
    // both groups produced 36 + 4 + 17 = 57 against a cloud of 56 — the same
    // class of unreconciled total this section is fixing.
    let aaa_listed = automated_criteria()
        .iter()
        .filter(|(_, level)| *level == "AAA")
        .count();
    let wcag22_ab_listed = automated_criteria()
        .iter()
        .filter(|(id, level)| *level != "AAA" && crate::wcag::coverage::is_wcag22_only(id))
        .count();
    debug_assert_eq!(
        automated + wcag22_ab_listed + aaa_listed,
        listed,
        "the three groups must partition the listed criteria"
    );
    let automated_title = if en {
        format!("Automatically checked ({listed})")
    } else {
        format!("Automatisch geprüft ({listed})")
    };
    let listed_note = if en {
        format!(
            "{listed} criteria in total: the {automated} WCAG 2.1 A/AA criteria the ratio above is scoped to, plus {wcag22_ab_listed} A/AA criteria added by WCAG 2.2 and {aaa_listed} AAA criteria this tool also checks."
        )
    } else {
        format!(
            "{listed} Kriterien insgesamt: die {automated} WCAG-2.1-A/AA-Kriterien, auf die sich die Quote oben bezieht, dazu {wcag22_ab_listed} von WCAG 2.2 ergänzte A/AA-Kriterien und {aaa_listed} AAA-Kriterien, die dieses Werkzeug ebenfalls prüft."
        )
    };
    let mut tag_cloud = TagCloud::new().with_title(&automated_title).with_gap("5pt");
    for (c, l) in automated_criteria().iter() {
        let tag = if crate::wcag::coverage::is_wcag22_only(c) {
            format!("WCAG {} ({}, WCAG 2.2)", c, l)
        } else {
            format!("WCAG {} ({})", c, l)
        };
        tag_cloud = tag_cloud.add(tag, "good");
    }
    builder = builder.add_component(tag_cloud);
    builder = builder.add_component(
        Label::new(listed_note)
            .with_size("8.8pt")
            .with_color(crate::output::pdf::design::tokens::MUTED),
    );

    let manual_title = if en {
        format!(
            "Requires manual review ({})",
            manual_review_criteria().len()
        )
    } else {
        format!(
            "Manuelle Prüfung erforderlich ({})",
            manual_review_criteria().len()
        )
    };
    let mut manual_cloud = TagCloud::new().with_title(&manual_title).with_gap("5pt");
    for (c, l, name) in manual_review_criteria().iter() {
        let localized_name = if en {
            *name
        } else {
            crate::wcag::coverage::manual_review_criterion_name_de(c, name)
        };
        manual_cloud = manual_cloud.add(format!("{c} ({l}) – {localized_name}"), "info");
    }
    builder = builder.add_component(manual_cloud);

    // Practical testing guide — how to test the manual criteria above
    let how_title = if en {
        "How to test manually"
    } else {
        "So testen Sie manuell"
    };
    let items: &[(&str, &str)] = if en {
        &[
            (
                "Keyboard navigation",
                "Tab through the entire page. No focus loss, no keyboard trap, every interactive element reachable.",
            ),
            (
                "Screen reader",
                "Test with NVDA/JAWS (Windows) or VoiceOver (Mac/iOS) — landmark navigation and form interaction.",
            ),
            (
                "400% zoom",
                "At 400% browser zoom: page operable without horizontal scrolling, no content lost.",
            ),
            (
                "Reduced motion",
                "Enable the OS 'reduce motion' setting and verify animations are disabled or significantly diminished.",
            ),
            (
                "Modal / dropdown interaction",
                "Full keyboard interaction: Tab, Enter, Space, Escape, Arrow keys. Focus returns to the trigger on close.",
            ),
            (
                "Color blindness simulation",
                "Use a tool like 'Color Oracle' to verify information is conveyed by more than color alone.",
            ),
        ]
    } else {
        &[
            (
                "Tastaturnavigation",
                "Komplette Seite per Tab navigieren. Kein Fokus verloren, kein Keyboard-Trap, jedes interaktive Element erreichbar.",
            ),
            (
                "Screenreader",
                "Test mit NVDA/JAWS (Windows) oder VoiceOver (Mac/iOS) — Landmark-Navigation und Formular-Interaktion.",
            ),
            (
                "400% Zoom",
                "Bei 400% Browser-Zoom: Seite ohne horizontales Scrollen bedienbar, kein Inhalt verloren.",
            ),
            (
                "Reduced Motion",
                "Betriebssystem-Einstellung „Bewegung reduzieren\" aktivieren und prüfen, ob Animationen deaktiviert oder reduziert werden.",
            ),
            (
                "Modal- / Dropdown-Interaktion",
                "Vollständige Tastaturbedienung: Tab, Enter, Space, Escape, Pfeiltasten. Fokus kehrt nach Schließen zum Trigger zurück.",
            ),
            (
                "Farbenblindheit",
                "Mit einem Werkzeug wie „Color Oracle\" prüfen, ob Informationen nicht ausschließlich über Farbe vermittelt werden.",
            ),
        ]
    };
    let rows: Vec<ChecklistRow> = items
        .iter()
        .map(|(t, d)| ChecklistRow::new(*t, *d).with_status("info"))
        .collect();
    builder.add_component(ChecklistPanel::new(rows).with_title(how_title))
}
