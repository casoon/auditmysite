//! Shared helper functions for PDF rendering.

use crate::i18n::I18n;
use crate::output::report_model::*;

/// Concrete VoiceOver/NVDA self-verification instructions (plan/16) for the
/// handful of WCAG criteria where a non-expert can personally confirm a
/// suspected screen-reader issue in a few minutes, rather than trusting a
/// generic "manual review" hint. Shared across the screen-reader findings
/// table, WCAG violation finding cards, and the not-testable/warning
/// checklist — the same criterion can surface through any of those three
/// paths. Keyboard shortcuts themselves stay identical across locales; only
/// the surrounding explanation is translated. Returns `None` for criteria
/// with no documented self-test (most of them — this is deliberately not a
/// general-purpose annotation).
pub(super) fn manual_recheck_instruction(wcag_criterion: &str, en: bool) -> Option<&'static str> {
    match wcag_criterion {
        "1.3.6" => Some(if en {
            "Self-check: open VoiceOver's Rotor (VO+U, macOS) or NVDA's Elements List \
             (Insert+F7) and switch to \"Landmarks\" — listen whether the areas you'd expect \
             (header, navigation, footer) are actually announced."
        } else {
            "Selbsttest: VoiceOver-Rotor (VO+U, macOS) oder NVDA-Elementliste (Einfg+F7) öffnen \
             und auf „Landmarken\" umschalten — anhören, ob die erwarteten Bereiche (Header, \
             Navigation, Footer) tatsächlich angekündigt werden."
        }),
        "1.3.1" => Some(if en {
            "Self-check: use VoiceOver's Rotor (VO+U) or NVDA's Elements List (Insert+F7) and \
             check both the \"Headings\" and \"Landmarks\" views — confirm the order/uniqueness \
             you hear matches what this finding describes."
        } else {
            "Selbsttest: VoiceOver-Rotor (VO+U) oder NVDA-Elementliste (Einfg+F7) nutzen und \
             sowohl „Überschriften\" als auch „Landmarken\" prüfen — abgleichen, ob \
             Reihenfolge/Eindeutigkeit dem hier beschriebenen Befund entspricht."
        }),
        "3.3.2" => Some(if en {
            "Self-check: tab into the form with VoiceOver (VO+Cmd+J jumps between form \
             controls) or NVDA (F jumps between fields) and listen whether every field \
             announces a name — not just a generic \"edit text\"."
        } else {
            "Selbsttest: mit VoiceOver (VO+Cmd+J springt zwischen Formularfeldern) oder NVDA \
             (F springt zwischen Feldern) durch das Formular tabben und hören, ob jedes Feld \
             einen Namen ansagt — nicht nur ein generisches „Textfeld\"."
        }),
        "4.1.3" => Some(if en {
            "Self-check: trigger the message (e.g. submit the form with an invalid value) \
             while VoiceOver or NVDA is running and confirm it is announced automatically, \
             without moving keyboard focus to it."
        } else {
            "Selbsttest: die Meldung auslösen (z. B. Formular mit ungültigem Wert absenden), \
             während VoiceOver oder NVDA läuft, und prüfen, ob sie automatisch angesagt wird, \
             ohne dass der Tastaturfokus dorthin springt."
        }),
        _ => None,
    }
}

pub(super) fn extract_domain(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = without_scheme.split('/').next().unwrap_or(without_scheme);
    host.trim_start_matches("www.").to_string()
}

const SECTION_HEADER_SPLIT_TEMPLATE: &str = include_str!("templates/section_header_split.typ");
const METRIC_STRIP_TEMPLATE: &str = include_str!("templates/metric_strip.typ");

// Theme fonts, bundled so the PDF renders identically regardless of which
// fonts happen to be installed on the machine running auditmysite (system
// fonts are not a reliable dependency: they differ across dev machines and
// CI, and scanning/parsing all of them cost several GB of RAM per PDF —
// see `EngineConfig::use_system_fonts(false)` below). All three are
// OFL-licensed and metric/style equivalents of the previous system fonts:
// Arimo for Helvetica/Arial (body), PT Serif for Georgia (headings, not
// redistributable), JetBrains Mono unchanged (already OFL, now bundled
// instead of relying on it being locally installed). See
// `assets/fonts/*/OFL.txt` for the license texts.
const ARIMO_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/arimo/Arimo-Regular.ttf");
const ARIMO_BOLD: &[u8] = include_bytes!("../../../assets/fonts/arimo/Arimo-Bold.ttf");
const ARIMO_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/arimo/Arimo-Italic.ttf");
const ARIMO_BOLD_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/arimo/Arimo-BoldItalic.ttf");
const PT_SERIF_REGULAR: &[u8] =
    include_bytes!("../../../assets/fonts/ptserif/PT_Serif-Web-Regular.ttf");
const PT_SERIF_BOLD: &[u8] = include_bytes!("../../../assets/fonts/ptserif/PT_Serif-Web-Bold.ttf");
const PT_SERIF_ITALIC: &[u8] =
    include_bytes!("../../../assets/fonts/ptserif/PT_Serif-Web-Italic.ttf");
const PT_SERIF_BOLD_ITALIC: &[u8] =
    include_bytes!("../../../assets/fonts/ptserif/PT_Serif-Web-BoldItalic.ttf");
const JETBRAINS_MONO_REGULAR: &[u8] =
    include_bytes!("../../../assets/fonts/jetbrainsmono/JetBrainsMono-Regular.ttf");
const JETBRAINS_MONO_BOLD: &[u8] =
    include_bytes!("../../../assets/fonts/jetbrainsmono/JetBrainsMono-Bold.ttf");

const THEME_FONTS: &[&[u8]] = &[
    ARIMO_REGULAR,
    ARIMO_BOLD,
    ARIMO_ITALIC,
    ARIMO_BOLD_ITALIC,
    PT_SERIF_REGULAR,
    PT_SERIF_BOLD,
    PT_SERIF_ITALIC,
    PT_SERIF_BOLD_ITALIC,
    JETBRAINS_MONO_REGULAR,
    JETBRAINS_MONO_BOLD,
];

/// Built once per process and shared from then on: every caller only reads
/// from the engine afterwards (`render_pdf`/`render_typ` take `&self`), and
/// building one is deterministic (embedded fonts/templates/theme, no I/O or
/// input-dependent state) — so re-running `build_engine()` per PDF, and
/// especially per test in a suite that renders dozens of PDFs, bought
/// nothing but repeated font-cache construction cost.
static SHARED_ENGINE: std::sync::LazyLock<std::sync::Arc<renderreport::Engine>> =
    std::sync::LazyLock::new(|| {
        std::sync::Arc::new(build_engine().expect("PDF engine should build"))
    });

/// Shared engine with proper font configuration for German text.
pub(super) fn create_engine() -> std::sync::Arc<renderreport::Engine> {
    std::sync::Arc::clone(&SHARED_ENGINE)
}

fn build_engine() -> anyhow::Result<renderreport::Engine> {
    use renderreport::components::ComponentId;
    use renderreport::engine::EngineConfig;
    use renderreport::theme::{Theme, TokenValue};

    // No system fonts: a full scan+parse of every installed font (500+
    // files on a typical macOS dev machine, more on CI images) cost several
    // GB of RAM per engine and made the rendered PDF depend on whatever
    // happened to be installed locally. THEME_FONTS above covers every
    // family the theme references, so nothing is lost by disabling this.
    let config = EngineConfig::builder().use_system_fonts(false).build();
    let mut engine = renderreport::Engine::with_config_and_fonts(config, THEME_FONTS)?;

    engine.components_mut().register(
        ComponentId::new("section-header-split"),
        SECTION_HEADER_SPLIT_TEMPLATE.to_string(),
    );
    engine.components_mut().register(
        ComponentId::new("metric-strip"),
        METRIC_STRIP_TEMPLATE.to_string(),
    );

    let mut theme = Theme::default_theme();
    theme
        .tokens
        .set("font.body", TokenValue::Font("Arimo".into()));
    theme
        .tokens
        .set("font.heading", TokenValue::Font("PT Serif".into()));
    theme
        .tokens
        .set("font.mono", TokenValue::Font("JetBrains Mono".into()));
    engine.set_default_theme(theme);

    Ok(engine)
}

/// Map our severity to renderreport severity
pub(super) fn map_severity(severity: &crate::wcag::Severity) -> renderreport::prelude::Severity {
    use renderreport::prelude::Severity;
    match severity {
        crate::wcag::Severity::Critical => Severity::Critical,
        crate::wcag::Severity::High => Severity::High,
        crate::wcag::Severity::Medium => Severity::Medium,
        crate::wcag::Severity::Low => Severity::Low,
    }
}

pub(super) fn severity_label_i18n(severity: crate::wcag::Severity, i18n: &I18n) -> String {
    match severity {
        crate::wcag::Severity::Critical => i18n.t("severity-critical"),
        crate::wcag::Severity::High => i18n.t("severity-high"),
        crate::wcag::Severity::Medium => i18n.t("severity-medium"),
        crate::wcag::Severity::Low => i18n.t("severity-low"),
    }
}

pub(super) fn priority_label_i18n(priority: Priority, i18n: &I18n) -> String {
    match priority {
        Priority::Critical => i18n.t("priority-critical"),
        Priority::High => i18n.t("priority-high"),
        Priority::Medium => i18n.t("priority-medium"),
        Priority::Low => i18n.t("priority-low"),
    }
}

pub(super) fn role_label_i18n(role: Role, i18n: &I18n) -> String {
    match role {
        Role::Development => i18n.t("role-development"),
        Role::Editorial => i18n.t("role-editorial"),
        Role::DesignUx => i18n.t("role-designux"),
        Role::ProjectManagement => i18n.t("role-projectmanagement"),
    }
}

pub(super) fn effort_label_i18n(effort: Effort, i18n: &I18n) -> String {
    match effort {
        Effort::Quick => i18n.t("effort-quick"),
        Effort::Medium => i18n.t("effort-medium"),
        Effort::Structural => i18n.t("effort-structural"),
    }
}

pub(super) fn score_quality_label(score: u32) -> &'static str {
    match score {
        85..=100 => "Stark",
        70..=84 => "Solide",
        50..=69 => "Uneinheitlich",
        _ => "Schwach",
    }
}

pub(super) fn score_quality_color(score: u32) -> &'static str {
    use super::design::tokens;
    match score {
        85..=100 => tokens::SUCCESS,
        70..=84 => tokens::SUCCESS,
        50..=69 => tokens::WARN_DEEP,
        _ => tokens::DANGER,
    }
}

/// Appends a non-normative qualifier to a module's display name wherever it
/// renders with the same visual weight as a Compliance/Measured module
/// (cover gauges, dashboard cards, technical modules overview). Single
/// shared implementation of the "(Indikator)"/"(Indicator)" suffix that
/// previously only covered the technical modules overview panel — extended
/// here to also cover Optional-tier modules (e.g. Dark Mode) and reused
/// across every render surface instead of being re-implemented per call
/// site (#577).
pub(super) fn module_name_with_taxonomy_suffix(
    name: &str,
    measurement_type: &str,
    i18n: &I18n,
) -> String {
    use crate::output::report_model::ModuleTaxonomyClass;
    let class = ModuleTaxonomyClass::from_measurement_type(measurement_type, name);
    if !class.needs_suffix_qualifier() {
        return name.to_string();
    }
    let en = i18n.locale() == "en";
    let suffix = match class {
        ModuleTaxonomyClass::Optional => "Optional",
        _ => {
            if en {
                "Indicator"
            } else {
                "Indikator"
            }
        }
    };
    format!("{name} ({suffix})")
}

#[cfg(test)]
mod tests {
    use super::{METRIC_STRIP_TEMPLATE, SECTION_HEADER_SPLIT_TEMPLATE};

    // These two assert directly on the embedded template source (see the
    // `include_str!` constants above `create_engine`) instead of building a
    // full renderreport::Engine — Engine::new() loads every system font
    // (fontdb, face by face) purely to satisfy the default theme, which costs
    // several GB of RSS per call and made these two string-contains checks
    // the single heaviest part of `cargo test` on this crate.

    #[test]
    fn section_eyebrow_uses_light_spacious_typography() {
        assert!(SECTION_HEADER_SPLIT_TEMPLATE.contains("weight: \"regular\""));
        assert!(SECTION_HEADER_SPLIT_TEMPLATE.contains("tracking: 0.20em"));
        assert!(SECTION_HEADER_SPLIT_TEMPLATE.contains("#v(spacing-3)"));
    }

    #[test]
    fn metric_value_and_context_share_a_bottom_alignment() {
        assert!(METRIC_STRIP_TEMPLATE.contains("align: bottom + left"));
        assert!(!METRIC_STRIP_TEMPLATE.contains("pad(top: 3pt)"));
    }

    #[test]
    fn taxonomy_suffix_leaves_compliance_and_measured_names_untouched() {
        let de = crate::i18n::I18n::new("de").expect("i18n");
        assert_eq!(
            super::module_name_with_taxonomy_suffix("Accessibility", "measured", &de),
            "Accessibility"
        );
        assert_eq!(
            super::module_name_with_taxonomy_suffix("Performance", "measured", &de),
            "Performance"
        );
        assert_eq!(
            super::module_name_with_taxonomy_suffix("Search Experience", "composite", &de),
            "Search Experience"
        );
    }

    #[test]
    fn taxonomy_suffix_marks_heuristic_modules_as_indicator() {
        let de = crate::i18n::I18n::new("de").expect("i18n");
        let en = crate::i18n::I18n::new("en").expect("i18n");
        assert_eq!(
            super::module_name_with_taxonomy_suffix("UX", "heuristic", &de),
            "UX (Indikator)"
        );
        assert_eq!(
            super::module_name_with_taxonomy_suffix("UX", "heuristic", &en),
            "UX (Indicator)"
        );
    }

    #[test]
    fn taxonomy_suffix_left_off_html_conformance_as_measured() {
        // HTML Conformance carried "heuristic" (and so an "(Indikator)"
        // qualifier) while its score was distorted by a since-closed upstream
        // false-positive gap and a per-occurrence penalty. Both are fixed (see
        // src/audit/normalized.rs), it is weighted and "measured" again, and a
        // measured module takes no qualifier.
        let de = crate::i18n::I18n::new("de").expect("i18n");
        assert_eq!(
            super::module_name_with_taxonomy_suffix("HTML Conformance", "measured", &de),
            "HTML Conformance"
        );
    }

    #[test]
    fn taxonomy_suffix_marks_optional_modules_as_optional() {
        let de = crate::i18n::I18n::new("de").expect("i18n");
        let en = crate::i18n::I18n::new("en").expect("i18n");
        assert_eq!(
            super::module_name_with_taxonomy_suffix("Dark Mode", "optional", &de),
            "Dark Mode (Optional)"
        );
        assert_eq!(
            super::module_name_with_taxonomy_suffix("Dark Mode", "optional", &en),
            "Dark Mode (Optional)"
        );
    }
}
