//! Module detail renderers (performance, SEO, security, mobile, AI visibility).

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

use super::helpers::{
    manual_recheck_instruction, map_severity, score_quality_color, score_quality_label,
    severity_label_i18n, truncate_with_ellipsis,
};

mod accessibility;
mod commerce;
mod experience;
mod html_conform;
mod indicators;
mod overview;
mod performance;
mod platform;
mod privacy;
mod seo;
#[cfg(test)]
pub(super) mod text_export;

#[cfg(test)]
pub(super) use accessibility::effective_bfsg_verdict;
pub(super) use accessibility::{
    bfsg_relevant_wcag_findings, render_a11y_journey_findings, render_screen_reader_section,
};
pub(super) use commerce::render_commerce;
pub(super) use experience::{render_journey, render_ux};
pub(super) use html_conform::render_html_conform;
pub(super) use indicators::{
    render_ai_visibility, render_best_practices, render_content_visibility, render_source_quality,
    render_tech_stack,
};
pub(super) use overview::{render_budget_violations, render_search_experience};
pub(super) use performance::render_performance;
pub(super) use platform::{mobile_category_label, render_mobile, render_security};
pub(super) use privacy::{
    category_label, check_short_label, kind_label, render_privacy, seat_label, spread_label,
};
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
    // `None` for a module that carries no 0-100 score at all (plan 29, D4) —
    // the score-derived weakness sentence is then simply omitted.
    score: Option<u32>,
    // Intentionally unused: the technical interpretation / heuristic disclaimer
    // is already shown in the module's "Überblick" and indicator notes. Appending
    // it here duplicated jargon (e.g. "DOM-Komplexität 20804 Knoten") into the
    // plain-language customer passage, defeating its purpose (#446 readability).
    _interpretation: &str,
) -> Label {
    // html_conform is score-neutral with a known false-positive gap (see
    // detail_modules/html_conform.rs) -- a generic "clear weakness" sentence
    // for its score would contradict the module's own heuristic-reliability
    // disclaimer immediately above it. content_visibility no longer has a
    // score to judge at all.
    let scored = match module_key {
        "content_visibility" | "html_conform" => None,
        _ => score,
    };
    let weakness = match scored {
        Some(s) if s < 50 => Some(i18n.t("pdf-details-weakness-clear")),
        Some(s) if s < 75 => Some(i18n.t("pdf-details-weakness-visible")),
        _ => None,
    };
    let context_key = match module_key {
        "performance" => Some("pdf-details-context-performance"),
        "seo" => Some("pdf-details-context-seo"),
        // The composite's own chapter opener already states what the value combines;
        // repeating it here printed two near-identical lead sentences (#952).
        "search_experience" => None,
        "security" => Some("pdf-details-context-security"),
        "html_conform" => Some("pdf-details-context-html-conform"),
        "mobile" => Some("pdf-details-context-mobile"),
        "ux" => Some("pdf-details-context-ux"),
        "journey" => Some("pdf-details-context-journey"),
        "source_quality" => Some("pdf-details-context-source-quality"),
        "ai_visibility" => Some("pdf-details-context-ai-visibility"),
        "content_visibility" => Some("pdf-details-context-content-visibility"),
        _ => Some("pdf-details-context-other"),
    };
    let mut parts = Vec::new();
    if let Some(w) = weakness {
        parts.push(w);
    }
    if let Some(key) = context_key {
        parts.push(i18n.t(key));
    }
    // No meta-label prefix — the plain-language sentence speaks for itself
    // ("Was das für Kunden bedeutet:" was redundant wording, #446 readability).
    let text = parts.join(" ");
    Label::new(text)
        .with_size("10.5pt")
        .with_color(crate::output::pdf::design::tokens::NEUTRAL)
}

/// States that an indicator re-reads other modules rather than measuring
/// anything new (plan 29, D3). `None` for a module that measures its own
/// subject, so the note only appears where it is true.
///
/// Without it, a reader sees SEO, AI Visibility and Source Quality agree and
/// takes that for three independent assessments. They largely share inputs.
pub(super) fn derived_from_note(i18n: &I18n, module_name: &str) -> Option<Label> {
    let sources = crate::taxonomy::module_derived_from(module_name);
    if sources.is_empty() {
        return None;
    }
    let names: Vec<String> = sources
        .iter()
        .map(|key| {
            let translated = i18n.t(&format!("module-{key}"));
            if translated == format!("module-{key}") {
                key.to_string()
            } else {
                translated
            }
        })
        .collect();
    let list = names.join(", ");
    let text = i18n.t_args("pdf-details-derived-from", &[("list", list)]);
    Some(
        Label::new(text)
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    )
}

/// Names the points the Accessibility result cost this indicator.
///
/// The penalty is defensible — for users with disabilities, accessibility
/// *is* the UX — but it was folded into the score with nothing said, so the
/// number looked like an independent reading of the page (plan 29, D3).
pub(super) fn a11y_penalty_note(
    builder: renderreport::engine::ReportBuilder,
    penalty: u32,
    score_before: u32,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if penalty == 0 {
        return builder;
    }
    let text = i18n.t_args(
        "pdf-details-a11y-penalty",
        &[
            ("penalty", i64::from(penalty)),
            ("before", i64::from(score_before)),
        ],
    );
    builder.add_component(
        Label::new(text)
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    )
}

/// The interpretation, but only where it says more than the chapter opener
/// already did.
///
/// The opener's takeaway is `first_sentence(interpretation)`, and a
/// single-sentence interpretation therefore arrived twice: once in the
/// `section-header-split` body, once as a standalone label two components
/// later, with only a score card between them (plan 36 §3).
pub(super) fn interpretation_beyond_takeaway<'a>(
    interpretation: &'a str,
    takeaway: &str,
) -> Option<&'a str> {
    let rest = interpretation
        .trim()
        .strip_prefix(takeaway.trim())
        .unwrap_or(interpretation)
        .trim();
    (!rest.is_empty()).then_some(rest)
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
/// A canonical severity string (`high`, `medium`, ...) in the run language.
/// Text that is no known severity is returned unchanged.
pub(super) fn severity_text(severity: &str, i18n: &I18n) -> String {
    let key = format!("severity-{}", severity.to_lowercase());
    let text = i18n.t(&key);
    if text == key {
        severity.to_string()
    } else {
        text
    }
}

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
pub(super) fn module_score_caption(i18n: &I18n) -> String {
    i18n.t("pdf-details-score-caption")
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
fn vital_rating_label(rating: &str, en: bool) -> String {
    let key = match rating {
        "good" => "pdf-details-vital-good",
        "needs-improvement" => "pdf-details-vital-needs-improvement",
        "poor" => "pdf-details-vital-poor",
        _ => return "—".to_string(),
    };
    crate::i18n::text(key, en)
}

fn vital_color(rating: &str) -> &'static str {
    crate::output::pdf::design::status_color(vital_status(rating))
}

fn journey_category_label(category: &str, i18n: &I18n) -> String {
    let key = match category {
        "TabOrder" => Some("pdf-details-journey-tab-order"),
        "FocusTrap" => Some("pdf-details-journey-focus-trap"),
        "StateTransition" => Some("pdf-details-journey-state-transition"),
        "FocusRestoration" => Some("pdf-details-journey-focus-restoration"),
        "FormError" => Some("pdf-details-journey-form-error"),
        "SpaNavigation" => Some("a11y-details-journey-label-spa-navigation"),
        "HiddenFocusable" => Some("pdf-details-journey-hidden-focusable"),
        "SkipLink" => Some("pdf-details-journey-skip-link"),
        "FocusIndicator" => Some("pdf-details-journey-focus-indicator"),
        "MenuJourney" => Some("pdf-details-journey-menu"),
        "TabsJourney" => Some("pdf-details-journey-tabs"),
        "LinkText" => Some("pdf-details-journey-link-text"),
        "Landmark" => Some("pdf-details-journey-landmark"),
        "MediaControls" => Some("pdf-details-journey-media-controls"),
        "AddToCart" => Some("pdf-details-journey-add-to-cart"),
        "QuantityStepper" => Some("a11y-details-journey-label-quantity-stepper"),
        "HeadingOutline" => Some("pdf-details-journey-heading-outline"),
        _ => None,
    };
    if let Some(key) = key {
        return i18n.t(key);
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
