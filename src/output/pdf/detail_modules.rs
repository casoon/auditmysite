//! Module detail renderers (performance, SEO, security, mobile, dark mode, AI visibility).

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, Grid, KeyValueList, List, MetricStrip, MetricStripItem, PageBreak,
};
use renderreport::components::charts::{Gauge, GaugeThreshold};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::{AuditTable, Finding, ScoreCard, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::localized::{is_english, pick};
use crate::output::report_model::*;

use super::helpers::{map_severity, score_quality_color, score_quality_label};

mod accessibility;
mod dark_mode;
mod experience;
mod indicators;
mod overview;
mod performance;
mod platform;
mod seo;

pub(super) use accessibility::{render_a11y_journey_findings, render_screen_reader_section};
pub(super) use dark_mode::render_dark_mode;
pub(super) use experience::{render_journey, render_ux};
pub(super) use indicators::{
    render_ai_visibility, render_best_practices, render_content_visibility, render_source_quality,
    render_tech_stack,
};
pub(super) use overview::{render_budget_violations, render_search_experience};
pub(super) use performance::render_performance;
pub(super) use platform::{render_mobile, render_security};
pub(super) use seo::render_seo;

/// A neutral per-section line shown when a module produced no findings, so a
/// reader can distinguish "checked and clean" from "not checked" instead of the
/// section silently collapsing to nothing (#446).
fn clean_section_note(i18n: &I18n) -> Label {
    Label::new(i18n.t("pdf-section-clean"))
        .with_size("10.5pt")
        .with_color(crate::output::pdf::design::tokens::NEUTRAL)
}

fn module_customer_context(
    i18n: &I18n,
    module_key: &str,
    score: u32,
    // Intentionally unused: the technical interpretation / heuristic disclaimer
    // is already shown in the module's "Überblick" and indicator notes. Appending
    // it here duplicated jargon (e.g. "DOM-Komplexität 20804 Knoten") into the
    // plain-language customer passage, defeating its purpose (#446 readability).
    _interpretation: &str,
) -> Label {
    let en = is_english(i18n);
    let weakness = if module_key == "content_visibility" {
        None
    } else if score < 50 {
        Some(pick(
            i18n,
            "Der Score zeigt in diesem Bereich eine deutliche Schwäche.",
            "The score indicates a clear weakness in this area.",
        ))
    } else if score < 75 {
        Some(pick(
            i18n,
            "Der Score zeigt in diesem Bereich erkennbares Verbesserungspotenzial.",
            "The score indicates visible improvement potential in this area.",
        ))
    } else {
        None
    };
    let module_text = match (module_key, en) {
        ("performance", true) => "Visitors may experience delays, unstable rendering or unnecessary data transfer before the page feels usable.",
        ("performance", false) => "Besucher können Verzögerungen, instabiles Rendering oder unnötige Datenmenge erleben, bevor die Seite nutzbar wirkt.",
        ("seo", true) => "Search engines and AI systems need clear titles, headings, structured data and enough readable content to understand the page.",
        ("seo", false) => "Suchmaschinen und KI-Systeme benötigen klare Titel, Überschriften, strukturierte Daten und ausreichend lesbaren Inhalt, um die Seite zu verstehen.",
        ("search_experience", true) => "This score combines technical findability with whether users, search engines and AI systems can actually understand and trust the content.",
        ("search_experience", false) => "Dieser Wert verbindet technische Auffindbarkeit mit der Frage, ob Nutzer, Suchmaschinen und KI-Systeme die Inhalte tatsächlich verstehen und ihnen vertrauen können.",
        ("security", true) => "Security headers and HTTPS signals influence visible trust and reduce avoidable browser-side risk in the checked scope.",
        ("security", false) => "Security Header und HTTPS-Signale beeinflussen sichtbares Vertrauen und reduzieren vermeidbare Browser-Risiken im geprüften Umfang.",
        ("mobile", true) => "Mobile visitors depend on readable text, fitting content and controls that are easy to tap on small screens.",
        ("mobile", false) => "Mobile Besucher sind auf lesbaren Text, passende Inhaltsbreiten und gut antippbare Bedienelemente angewiesen.",
        ("ux", true) => "This indicator estimates whether the page feels understandable, consistent and low-friction for common visitor tasks.",
        ("ux", false) => "Dieser Indikator schätzt, ob die Seite für typische Besucheraufgaben verständlich, konsistent und reibungsarm wirkt.",
        ("journey", true) => "This indicator estimates whether visitors can move through the page intent without avoidable friction.",
        ("journey", false) => "Dieser Indikator schätzt, ob Besucher ohne vermeidbare Reibung durch den Zweck der Seite kommen.",
        ("source_quality", true) => "Source quality signals show whether content appears substantial, consistent and trustworthy enough to support decisions.",
        ("source_quality", false) => "Quellenqualität zeigt, ob Inhalte substanziell, konsistent und vertrauenswürdig genug wirken, um Entscheidungen zu stützen.",
        ("ai_visibility", true) => "AI visibility indicates whether content can be parsed, chunked, attributed and cited by AI-assisted systems.",
        ("ai_visibility", false) => "KI-Sichtbarkeit zeigt, ob Inhalte von KI-gestützten Systemen gelesen, gegliedert, zugeordnet und zitiert werden können.",
        ("content_visibility", true) => "Content visibility combines discoverability, trust and topical depth signals; it is an indicator, not a guarantee of reach.",
        ("content_visibility", false) => "Content Visibility verbindet Auffindbarkeit, Vertrauen und inhaltliche Tiefe; es ist ein Indikator, keine Reichweiten-Garantie.",
        _ if en => "This module describes customer-facing quality beyond pure accessibility findings.",
        _ => "Dieses Modul beschreibt kundennahe Qualität über reine Accessibility-Befunde hinaus.",
    };
    let mut parts = Vec::new();
    if let Some(w) = weakness {
        parts.push(w);
    }
    parts.push(module_text);
    // No meta-label prefix — the plain-language sentence speaks for itself
    // ("Was das für Kunden bedeutet:" was redundant wording, #446 readability).
    let text = parts.join(" ");
    Label::new(text)
        .with_size("10.5pt")
        .with_color(crate::output::pdf::design::tokens::NEUTRAL)
}

/// Gauge color bands matching `design::score_color`'s 40/75 thresholds
/// (higher is better). `Gauge`'s own defaults assume the opposite — a
/// value climbing towards `max` reads as *more* severe — so every score
/// gauge needs these explicit thresholds, not the component default.
fn score_gauge_thresholds() -> Vec<GaugeThreshold> {
    vec![
        GaugeThreshold {
            value: 0.0,
            color: crate::output::pdf::design::tokens::DANGER.to_string(),
        },
        GaugeThreshold {
            value: 40.0,
            color: crate::output::pdf::design::tokens::WARN_DEEP.to_string(),
        },
        GaugeThreshold {
            value: 75.0,
            color: crate::output::pdf::design::tokens::SUCCESS.to_string(),
        },
    ]
}

/// Opens a module as a level-2 chapter: a page break (unless it is the first
/// module) plus a level-2 header carrying the module name and a one-line key
/// takeaway, so the module appears in the table of contents as its own chapter
/// and the reader leaves the page with one core message (#15).
pub(super) fn module_chapter_opener(
    mut builder: renderreport::engine::ReportBuilder,
    title: &str,
    takeaway: &str,
    is_first: bool,
) -> renderreport::engine::ReportBuilder {
    use renderreport::components::advanced::{PageBreak, SectionHeaderSplit};
    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }
    builder.add_component(SectionHeaderSplit::new(title, takeaway).with_level(2))
}

/// First sentence of an interpretation text — used as the chapter's one-line
/// key takeaway so the opener stays concise even when the full interpretation
/// runs to several sentences.
pub(super) fn first_sentence(text: &str) -> String {
    let t = text.trim();
    match t.find(". ") {
        Some(idx) => t[..=idx].trim().to_string(),
        None => t.to_string(),
    }
}

/// Generic label for a module's headline score card. The descriptive module
/// name lives in the level-2 chapter heading, so the card only needs a short
/// "overall score" caption (avoids printing the title twice).
pub(super) fn module_score_caption(i18n: &I18n) -> &'static str {
    if is_english(i18n) {
        "Overall score · 0–100"
    } else {
        "Gesamtwertung · 0–100"
    }
}

/// Plain-language grade band for a module score, aligned with the report's
/// bands (Sehr gut ≥ 90 … Kritisch < 40). Replaces the rejected A–F letter
/// grade as the score-card description. Localized de/en.
pub(super) fn score_band_label(score: u32, i18n: &I18n) -> &'static str {
    crate::registry::FIVE_BAND.label(score as f32, is_english(i18n))
}

fn vital_status(rating: &str) -> &'static str {
    match rating {
        "good" => "good",
        "needs-improvement" => "warn",
        "poor" => "bad",
        _ => "info",
    }
}

/// Localizes the canonical English rating token ("good"/"needs-improvement"/
/// "poor") — printing it raw leaked untranslated English into German report
/// tables (#406).
fn vital_rating_label(rating: &str, en: bool) -> &'static str {
    match (rating, en) {
        ("good", true) => "Good",
        ("needs-improvement", true) => "Needs improvement",
        ("poor", true) => "Poor",
        ("good", false) => "Gut",
        ("needs-improvement", false) => "Verbesserungswürdig",
        ("poor", false) => "Schlecht",
        _ => "—",
    }
}

fn vital_color(rating: &str) -> &'static str {
    match rating {
        "good" => "#0f766e",
        "needs-improvement" => "#d97706",
        "poor" => "#dc2626",
        _ => "#2563eb",
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
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

fn journey_category_label(category: &str, i18n: &I18n) -> String {
    let en = is_english(i18n);
    let label = match category {
        "TabOrder" => Some(if en { "Tab order" } else { "Tab-Reihenfolge" }),
        "FocusTrap" => Some(if en { "Focus trap" } else { "Fokus-Falle" }),
        "StateTransition" => Some(if en {
            "State transition"
        } else {
            "Zustandswechsel"
        }),
        "FocusRestoration" => Some(if en {
            "Focus restoration"
        } else {
            "Fokus-Wiederherstellung"
        }),
        "FormError" => Some(if en {
            "Form error announcement"
        } else {
            "Formularfehler-Ansage"
        }),
        "SpaNavigation" => Some(if en {
            "SPA navigation"
        } else {
            "SPA-Navigation"
        }),
        "HiddenFocusable" => Some(if en {
            "Hidden focusable element"
        } else {
            "Verstecktes fokussierbares Element"
        }),
        "SkipLink" => Some(if en { "Skip link" } else { "Skip-Link" }),
        "FocusIndicator" => Some(if en {
            "Focus indicator"
        } else {
            "Fokus-Indikator"
        }),
        "MenuJourney" => Some(if en {
            "Menu navigation"
        } else {
            "Menü-Navigation"
        }),
        "TabsJourney" => Some(if en {
            "Tab navigation"
        } else {
            "Tab-Navigation"
        }),
        _ => None,
    };
    if let Some(label) = label {
        return label.to_string();
    }
    let mut out = String::new();
    for (i, ch) in category.char_indices() {
        if ch == '_' || ch == '-' {
            out.push(' ');
        } else if ch.is_uppercase() && i > 0 && !out.ends_with(' ') {
            out.push(' ');
            out.push(ch);
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests;
