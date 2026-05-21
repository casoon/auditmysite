//! WCAG coverage section for PDF reports (issue #37).

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, KeyValueList, SectionHeaderSplit,
};
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
    use crate::wcag::coverage::{coverage_stats, AUTOMATED_CRITERIA, MANUAL_REVIEW_CRITERIA};

    let en = i18n.locale() == "en";

    // Principle coverage — informative secondary indicator (#99).
    let coverage = AccessibilityScorer::calculate_coverage(&report.wcag_results.violations);
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
    let title = if en { "Audit scope" } else { "Prüfumfang" };
    let intro = if en {
        format!(
            "This audit covers {automated} of ~{total} testable WCAG 2.1 AA criteria automatically. The criteria listed below require manual review."
        )
    } else {
        format!(
            "Dieses Audit prüft {automated} von ca. {total} WCAG-2.1-AA-Kriterien automatisch. Die unten aufgeführten Kriterien benötigen manuelle Prüfung."
        )
    };

    builder = builder.add_component(SectionHeaderSplit::new(title, &intro).with_level(2));

    let automated_title = if en {
        format!("Automatically checked ({})", automated)
    } else {
        format!("Automatisch geprüft ({})", automated)
    };
    let mut tag_cloud = TagCloud::new().with_title(&automated_title).with_gap("5pt");
    for (c, l) in AUTOMATED_CRITERIA.iter() {
        tag_cloud = tag_cloud.add(format!("WCAG {} ({})", c, l), "good");
    }
    builder = builder.add_component(tag_cloud);

    let manual_title = if en {
        format!("Requires manual review ({})", MANUAL_REVIEW_CRITERIA.len())
    } else {
        format!(
            "Manuelle Prüfung erforderlich ({})",
            MANUAL_REVIEW_CRITERIA.len()
        )
    };
    let mut manual_cloud = TagCloud::new().with_title(&manual_title).with_gap("5pt");
    for (c, l, name) in MANUAL_REVIEW_CRITERIA.iter() {
        manual_cloud = manual_cloud.add(format!("{c} ({l}) – {name}"), "info");
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
