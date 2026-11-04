use crate::audit::normalized::{AuditContext, NormalizedReport};
use crate::i18n::I18n;
use crate::output::report_model::{ModuleScore, ModulesBlock};
use crate::output::search_experience::build_search_experience;

use super::super::helpers::localized_module_name;
use super::super::modules::{
    derive_accessibility_card_context, derive_accessibility_context, derive_accessibility_lever,
    derive_mobile_card_context, derive_mobile_context, derive_mobile_lever,
    derive_performance_card_context, derive_performance_context, derive_performance_lever,
    derive_security_card_context, derive_security_context, derive_security_lever,
};
use super::module_details::normalized_module_score;

fn module_interpretation(normalized: &NormalizedReport, module: &str, locale: &str) -> String {
    normalized
        .interpretation
        .as_ref()
        .and_then(|i| i.per_module.get(module))
        .map(|t| t.for_locale(locale).to_string())
        .unwrap_or_default()
}

pub(super) fn build_modules_block_from_normalized(
    i18n: &I18n,
    normalized: &AuditContext<'_>,
) -> ModulesBlock {
    let locale = i18n.locale();
    let a11y_score = normalized.normalized.score as f32;
    let mut dashboard = vec![ModuleScore {
        name: localized_module_name("Accessibility", i18n),
        score: a11y_score.round() as u32,
        measurement_type: "measured".into(),
        interpretation: module_interpretation(&normalized.normalized, "accessibility", locale),
        card_context: derive_accessibility_card_context(i18n, &normalized.normalized),
        score_context: derive_accessibility_context(i18n, &normalized.normalized),
        key_lever: derive_accessibility_lever(i18n, &normalized.normalized),
        good_threshold: 75,
        warn_threshold: 50,
    }];

    // No Web Vitals measured: no gauge or card — a 0 there would read as
    // "unusably slow" (plan 69, #QA-023 for the overall score).
    let performance_measured = normalized
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Performance")
        .is_none_or(|m| m.measurement_type != "not_measured");
    if let Some(p) = normalized.raw_performance.filter(|_| performance_measured) {
        let score = normalized_module_score(&normalized.normalized, "Performance")
            .unwrap_or(p.score.overall);
        dashboard.push(ModuleScore {
            name: "Performance".into(),
            score,
            measurement_type: "measured".into(),
            interpretation: module_interpretation(&normalized.normalized, "performance", locale),
            card_context: derive_performance_card_context(i18n, p),
            score_context: derive_performance_context(i18n, p),
            key_lever: derive_performance_lever(i18n, p),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    // SEO is one of the six weighted modules; the "Sichtbarkeit &
    // Nutzerverständnis" composite used to stand in its place, so the cover
    // showed 71 where the overall score counted 65 (plan 105). The composite
    // follows below with the indicators.
    if let Some((seo, score)) = normalized.raw_seo.and_then(|seo| {
        normalized_module_score(&normalized.normalized, "SEO").map(|score| (seo, score))
    }) {
        // The shared interpretation has no SEO entry; the audit-data row read
        // "80 / 100 — " without one (#902).
        let mut interpretation = module_interpretation(&normalized.normalized, "seo", locale);
        if interpretation.is_empty() {
            interpretation =
                crate::seo::interpretation::seo_interpretation_text(seo, locale == "en");
        }
        dashboard.push(ModuleScore {
            name: localized_module_name("SEO", i18n),
            score,
            measurement_type: "measured".into(),
            card_context: interpretation.clone(),
            score_context: interpretation.clone(),
            key_lever: interpretation.clone(),
            interpretation,
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(s) = normalized.raw_security {
        let score = normalized_module_score(&normalized.normalized, "Security").unwrap_or(s.score);
        dashboard.push(ModuleScore {
            name: localized_module_name("Security", i18n),
            score,
            measurement_type: "measured".into(),
            interpretation: module_interpretation(&normalized.normalized, "security", locale),
            card_context: derive_security_card_context(i18n, s),
            score_context: derive_security_context(i18n, s),
            key_lever: derive_security_lever(i18n, s),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(m) = normalized.raw_mobile {
        let score = normalized_module_score(&normalized.normalized, "Mobile").unwrap_or(m.score);
        dashboard.push(ModuleScore {
            name: "Mobile".into(),
            score,
            measurement_type: "measured".into(),
            interpretation: module_interpretation(&normalized.normalized, "mobile", locale),
            card_context: derive_mobile_card_context(i18n, m),
            score_context: derive_mobile_context(i18n, m),
            key_lever: derive_mobile_lever(i18n, m),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(score) = normalized
        .raw_html_conform
        .and_then(|_| normalized_module_score(&normalized.normalized, "HTML Conformance"))
    {
        let interpretation = module_interpretation(&normalized.normalized, "html_conform", locale);
        dashboard.push(ModuleScore {
            name: localized_module_name("HTML Conformance", i18n),
            score,
            measurement_type: "measured".into(),
            card_context: interpretation.clone(),
            score_context: interpretation.clone(),
            key_lever: interpretation.clone(),
            interpretation,
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(search_experience) = build_search_experience(normalized, i18n) {
        dashboard.push(ModuleScore {
            name: search_experience.label.clone(),
            score: search_experience.score,
            measurement_type: "composite".into(),
            interpretation: search_experience.interpretation.clone(),
            card_context: search_experience.interpretation.clone(),
            score_context: search_experience
                .components
                .iter()
                .map(|c| format!("{} {}%: {}/100", c.label, c.weight_pct, c.score))
                .collect::<Vec<_>>()
                .join(", "),
            key_lever: search_experience
                .warnings
                .first()
                .cloned()
                .unwrap_or_else(|| search_experience.interpretation.clone()),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(u) = normalized.raw_ux {
        let ux_score = normalized_module_score(&normalized.normalized, "UX").unwrap_or(u.score);
        let en = locale == "en";
        let ux_context = {
            use crate::ux::{ux_dimension_name, UxDimensionKind::*};
            [
                (CtaClarity, u.cta_clarity.score),
                (VisualHierarchy, u.visual_hierarchy.score),
                (ContentClarity, u.content_clarity.score),
                (TrustSignals, u.trust_signals.score),
                (CognitiveLoad, u.cognitive_load.score),
            ]
            .iter()
            .map(|(kind, score)| format!("{} {}/100", ux_dimension_name(*kind, en), score))
            .collect::<Vec<_>>()
            .join(", ")
        };
        let ux_lever: String = if u.cta_clarity.score < 60 {
            i18n.t("ux-lever-cta")
        } else if u.trust_signals.score < 60 {
            i18n.t("ux-lever-trust")
        } else if u.visual_hierarchy.score < 60 {
            i18n.t("ux-lever-hierarchy")
        } else {
            i18n.t("ux-lever-default")
        };
        dashboard.push(ModuleScore {
            name: "UX".into(),
            score: ux_score,
            measurement_type: "heuristic".into(),
            interpretation: module_interpretation(&normalized.normalized, "ux", locale),
            card_context: ux_context.clone(),
            score_context: ux_context,
            key_lever: ux_lever,
            good_threshold: 80,
            warn_threshold: 55,
        });
    }
    if let Some(j) = normalized.raw_journey {
        let journey_score =
            normalized_module_score(&normalized.normalized, "Journey").unwrap_or(j.score);
        let journey_dimensions = {
            use crate::journey::{journey_dimension_name, JourneyDimensionKind::*};
            [
                (EntryClarity, j.entry_clarity.score),
                (Orientation, j.orientation.score),
                (Navigation, j.navigation.score),
                (Interaction, j.interaction.score),
                (Conversion, j.conversion.score),
            ]
            .iter()
            .map(|(kind, score)| {
                format!(
                    "{} {}/100",
                    journey_dimension_name(*kind, locale == "en"),
                    score
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
        };
        let journey_context = format!(
            "{}: {} · {}",
            i18n.t("journey-intent-label"),
            j.page_intent.label(locale == "en"),
            journey_dimensions
        );
        let journey_lever = j
            .friction_points
            .first()
            .map(|fp| fp.recommendation.clone())
            .unwrap_or_else(|| i18n.t("journey-lever-default"));
        dashboard.push(ModuleScore {
            name: "Journey".into(),
            score: journey_score,
            measurement_type: "heuristic".into(),
            interpretation: module_interpretation(&normalized.normalized, "journey", locale),
            card_context: journey_context.clone(),
            score_context: journey_context,
            key_lever: journey_lever,
            good_threshold: 80,
            warn_threshold: 55,
        });
    }

    let has_multiple = dashboard.len() > 1;
    let overall_score = if has_multiple {
        Some(normalized.normalized.overall_score)
    } else {
        None
    };
    let overall_interpretation =
        overall_score.map(|_| build_overall_score_explanation(i18n, &normalized.normalized));

    ModulesBlock {
        dashboard,
        overall_score,
        overall_interpretation,
        module_scores: normalized.normalized.module_scores.clone(),
        accessibility_subcategory_scores: normalized
            .normalized
            .accessibility_subcategory_scores
            .clone(),
        security_category_scores: normalized.normalized.security_category_scores.clone(),
    }
}

fn build_overall_score_explanation(i18n: &I18n, normalized: &NormalizedReport) -> String {
    let locale = i18n.locale();
    let en = locale == "en";
    let indicator_names: Vec<String> = normalized
        .module_scores
        .iter()
        .filter(|m| !m.contributes_to_overall || m.measurement_type == "heuristic")
        .map(|m| localized_module_name(&m.name, i18n))
        .collect();
    let indicator_note = if indicator_names.is_empty() {
        String::new()
    } else if en {
        format!(
            " Indicator modules ({}) are shown separately and do not change the combined technical score.",
            indicator_names.join(", ")
        )
    } else {
        format!(
            " Indikator-Module ({}) werden separat ausgewiesen und verändern den kombinierten technischen Wert nicht.",
            indicator_names.join(", ")
        )
    };

    if normalized.viewport_scores.is_some() {
        if en {
            format!(
                "The combined technical score uses the dual-viewport result: 70% mobile and 30% desktop. \
                 Security contributes 10% when active.{indicator_note}"
            )
        } else {
            format!(
                "Der kombinierte technische Wert nutzt das Dual-Viewport-Ergebnis: 70% Mobile und 30% Desktop. \
                 Sicherheit fließt mit 10% ein, wenn aktiv.{indicator_note}"
            )
        }
    } else {
        let contributing: Vec<String> = normalized
            .module_scores
            .iter()
            .filter(|m| m.contributes_to_overall)
            .map(|m| {
                let name = localized_module_name(&m.name, i18n);
                if en {
                    format!("{name} {}%", m.weight_pct)
                } else {
                    format!("{name} {} %", m.weight_pct)
                }
            })
            .collect();
        let weights = if contributing.is_empty() {
            if en {
                "Accessibility 100%".to_string()
            } else {
                "Barrierefreiheit 100 %".to_string()
            }
        } else {
            contributing.join(", ")
        };
        if en {
            format!("The overall rating is the weighted result of the assessed areas: {weights}.{indicator_note}")
        } else {
            format!("Die Gesamtbewertung ergibt sich aus den gewichteten Einzelmodulen: {weights}.{indicator_note}")
        }
    }
}
