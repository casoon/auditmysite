use crate::output::localized::is_english;
use renderreport::components::advanced::SectionHeaderSplit;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;

// ─── Interactive Journey Summary ────────────────────────────────────────────

pub(super) fn translate_interactive_category(category: &str, en: bool) -> String {
    match (category, en) {
        ("TabOrder", true) => "Tab Order",
        ("TabOrder", false) => "Tab-Reihenfolge",
        ("FocusTrap", true) => "Focus Trap (Modal)",
        ("FocusTrap", false) => "Fokus-Falle (Modal)",
        ("StateTransition", true) => "State Transitions",
        ("StateTransition", false) => "Zustandswechsel",
        ("FocusRestoration", true) => "Focus Restoration",
        ("FocusRestoration", false) => "Fokus-Rückführung",
        ("FormError", true) => "Form Error Announcement",
        ("FormError", false) => "Formularfehler-Ankündigung",
        ("SpaNavigation", true) => "SPA Navigation",
        ("SpaNavigation", false) => "SPA-Navigation",
        ("HiddenFocusable", true) => "Hidden Focusable",
        ("HiddenFocusable", false) => "Verstecktes fokussierbares Element",
        ("SkipLink", true) => "Skip Link",
        ("SkipLink", false) => "Skip-Link",
        ("FocusIndicator", true) => "Focus Indicator",
        ("FocusIndicator", false) => "Fokus-Indikator",
        ("MenuJourney", true) => "Menu / Navigation",
        ("MenuJourney", false) => "Menü / Navigation",
        ("TabsJourney", true) => "Tab Widget",
        ("TabsJourney", false) => "Tab-Widget",
        ("LinkText", true) => "Link Text",
        ("LinkText", false) => "Linktext",
        ("Landmark", true) => "Landmark",
        ("Landmark", false) => "Landmarke",
        ("MediaControls", true) => "Media Controls",
        ("MediaControls", false) => "Medienbedienung",
        ("HeadingOutline", true) => "Heading Outline",
        ("HeadingOutline", false) => "Überschriftengliederung",
        _ => category,
    }
    .to_string()
}

/// The summary sentence; verb and noun agree with the counts (#1039).
pub(super) fn interactive_callout_text(
    summary: &crate::output::report_model::InteractiveJourneySummary,
    i18n: &I18n,
) -> String {
    let total = summary.total_pages_tested as i64;
    if summary.pages_with_issues == 0 {
        return i18n.t_args("batch-interactive-none", &[("total", total)]);
    }
    let with_issues = summary.pages_with_issues as i64;
    i18n.t_args(
        "batch-interactive-some",
        &[
            ("count", with_issues),
            ("total", total),
            ("clean", total - with_issues),
        ],
    )
}

pub(super) fn render_batch_interactive_summary(
    mut builder: renderreport::engine::ReportBuilder,
    summary: &crate::output::report_model::InteractiveJourneySummary,
    total_urls: usize,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);

    let title = if en {
        "Keyboard Accessibility Journey"
    } else {
        "Tastatur-Journey zur Barrierefreiheit"
    };
    let intro = if en {
        "Interactive tests verify whether focus management, state transitions, and keyboard navigation behave correctly — not just whether the initial page tree is well-structured."
    } else {
        "Interaktive Tests prüfen, ob Fokusführung, Zustandswechsel und Tastaturnavigation korrekt funktionieren — nicht nur ob der initiale Seitenbaum korrekt ausgezeichnet ist."
    };
    builder = builder.add_component(SectionHeaderSplit::new(title, intro).with_level(1));

    let callout_text = interactive_callout_text(summary, i18n);
    builder = builder.add_component(if summary.has_critical {
        Callout::warning(&callout_text)
    } else if summary.pages_with_issues > 0 {
        Callout::info(&callout_text)
    } else {
        Callout::success(&callout_text)
    });

    if !summary.categories.is_empty() {
        let category_col = if en { "Category" } else { "Kategorie" };
        let affected_col = if en {
            "Affected URLs"
        } else {
            "Betroffene URLs"
        };
        let share_col = if en { "Share" } else { "Anteil" };
        let sev_col = if en { "Max. Severity" } else { "Max. Schwere" };
        let mut table = AuditTable::new(vec![
            TableColumn::new(category_col),
            TableColumn::new(affected_col),
            TableColumn::new(share_col),
            TableColumn::new(sev_col),
        ])
        .with_title(if en {
            "Issues by Category"
        } else {
            "Befunde nach Kategorie"
        });
        for row in &summary.categories {
            let pct = (row.affected_urls * 100)
                .checked_div(total_urls)
                .unwrap_or(0);
            let sev_label = match row.max_severity {
                crate::wcag::Severity::Critical => {
                    if en {
                        "Critical"
                    } else {
                        "Kritisch"
                    }
                }
                crate::wcag::Severity::High => {
                    if en {
                        "High"
                    } else {
                        "Hoch"
                    }
                }
                crate::wcag::Severity::Medium => {
                    if en {
                        "Medium"
                    } else {
                        "Mittel"
                    }
                }
                crate::wcag::Severity::Low => {
                    if en {
                        "Low"
                    } else {
                        "Niedrig"
                    }
                }
            };
            let category_label = translate_interactive_category(&row.category, en);
            table = table.add_row(vec![
                category_label,
                row.affected_urls.to_string(),
                format!("{pct}%"),
                sev_label.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}
