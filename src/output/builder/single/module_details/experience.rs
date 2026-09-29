use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
use crate::output::report_model::{
    FrictionPointPresentation, JourneyDimensionPresentation, JourneyPresentation,
    UxDimensionPresentation, UxIssuePresentation, UxPresentation,
};

use super::{module_interpretation, normalized_module_grade, normalized_module_score};

pub(super) fn build_ux_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<UxPresentation> {
    let locale = i18n.locale();
    normalized.raw_ux.map(|u| {
        let ux_score = normalized_module_score(&normalized.normalized, "UX").unwrap_or(u.score);
        UxPresentation {
            score: ux_score,
            a11y_penalty: u.score.saturating_sub(ux_score),
            score_before_a11y_penalty: u.score,
            grade: normalized_module_grade(&normalized.normalized, "UX")
                .unwrap_or_else(|| u.grade.clone()),
            interpretation: module_interpretation(&normalized.normalized, "ux", locale),
            dimensions: vec![
                UxDimensionPresentation {
                    kind: u.cta_clarity.kind,
                    name: u.cta_clarity.name.clone(),
                    score: u.cta_clarity.score,
                    summary: u.cta_clarity.summary.clone(),
                },
                UxDimensionPresentation {
                    kind: u.visual_hierarchy.kind,
                    name: u.visual_hierarchy.name.clone(),
                    score: u.visual_hierarchy.score,
                    summary: u.visual_hierarchy.summary.clone(),
                },
                UxDimensionPresentation {
                    kind: u.content_clarity.kind,
                    name: u.content_clarity.name.clone(),
                    score: u.content_clarity.score,
                    summary: u.content_clarity.summary.clone(),
                },
                UxDimensionPresentation {
                    kind: u.trust_signals.kind,
                    name: u.trust_signals.name.clone(),
                    score: u.trust_signals.score,
                    summary: u.trust_signals.summary.clone(),
                },
                UxDimensionPresentation {
                    kind: u.cognitive_load.kind,
                    name: u.cognitive_load.name.clone(),
                    score: u.cognitive_load.score,
                    summary: u.cognitive_load.summary.clone(),
                },
            ],
            issues: u
                .issues
                .iter()
                .map(|i| UxIssuePresentation {
                    kind: i.kind,
                    dimension: i.dimension.clone(),
                    severity: i.severity.clone(),
                    problem: i.problem.clone(),
                    impact: i.impact.clone(),
                    recommendation: i.recommendation.clone(),
                    values: i.values.clone(),
                })
                .collect(),
        }
    })
}

pub(super) fn build_journey_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<JourneyPresentation> {
    let locale = i18n.locale();
    let en = locale == "en";
    normalized.raw_journey.map(|j| {
        let journey_score =
            normalized_module_score(&normalized.normalized, "Journey").unwrap_or(j.score);
        // Detect page type mismatch between SEO profile and Journey module
        let seo_type: Option<String> = normalized
            .raw_seo
            .and_then(|s| s.content_profile.as_ref())
            .map(|cp| cp.page_classification.primary_type.label(en).to_lowercase());
        let journey_type = j.page_intent.label(en).to_lowercase();
        let type_note = match seo_type {
            Some(ref st) if !st.is_empty() && !journey_type.is_empty() && st != &journey_type => {
                if locale == "en" {
                    format!(
                        " (Primary classification: {}. Secondary signals point to {}.)",
                        st, journey_type
                    )
                } else {
                    format!(
                        " (Primäre Einordnung: {}. Sekundäre Signale deuten auf {} hin.)",
                        st, journey_type
                    )
                }
            }
            _ => String::new(),
        };
        let base_journey = module_interpretation(&normalized.normalized, "journey", locale);
        let journey_interpretation = if type_note.is_empty() {
            base_journey
        } else {
            format!("{}{}", base_journey, type_note)
        };
        JourneyPresentation {
            score: journey_score,
            a11y_penalty: j.score.saturating_sub(journey_score),
            score_before_a11y_penalty: j.score,
            grade: normalized_module_grade(&normalized.normalized, "Journey")
                .unwrap_or_else(|| j.grade.clone()),
            page_intent: j.page_intent.label(en).to_string(),
            interpretation: journey_interpretation,
            dimensions: vec![
                JourneyDimensionPresentation {
                    kind: j.entry_clarity.kind,
                    name: j.entry_clarity.name.clone(),
                    score: j.entry_clarity.score,
                    weight_pct: (j.entry_clarity.weight * 100.0).round() as u32,
                    summary: j.entry_clarity.summary.clone(),
                },
                JourneyDimensionPresentation {
                    kind: j.orientation.kind,
                    name: j.orientation.name.clone(),
                    score: j.orientation.score,
                    weight_pct: (j.orientation.weight * 100.0).round() as u32,
                    summary: j.orientation.summary.clone(),
                },
                JourneyDimensionPresentation {
                    kind: j.navigation.kind,
                    name: j.navigation.name.clone(),
                    score: j.navigation.score,
                    weight_pct: (j.navigation.weight * 100.0).round() as u32,
                    summary: j.navigation.summary.clone(),
                },
                JourneyDimensionPresentation {
                    kind: j.interaction.kind,
                    name: j.interaction.name.clone(),
                    score: j.interaction.score,
                    weight_pct: (j.interaction.weight * 100.0).round() as u32,
                    summary: j.interaction.summary.clone(),
                },
                JourneyDimensionPresentation {
                    kind: j.conversion.kind,
                    name: j.conversion.name.clone(),
                    score: j.conversion.score,
                    weight_pct: (j.conversion.weight * 100.0).round() as u32,
                    summary: j.conversion.summary.clone(),
                },
            ],
            friction_points: j
                .friction_points
                .iter()
                .map(|fp| FrictionPointPresentation {
                    kind: fp.kind,
                    step: fp.step.clone(),
                    severity: fp.severity.clone(),
                    problem: fp.problem.clone(),
                    impact: fp.impact.clone(),
                    recommendation: fp.recommendation.clone(),
                    values: fp.values.clone(),
                })
                .collect(),
        }
    })
}
