//! Customer-facing Search Experience score.
//!
//! This is a presentation-level composite: it does not collect new data, but
//! combines existing SEO, UX, AI visibility, content visibility and mobile
//! signals so the front report does not overstate classic technical SEO.

#[cfg(feature = "experimental")]
use crate::assessment::AssessmentLevel;
use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
#[cfg(feature = "experimental")]
use crate::output::localized::is_english;
use crate::output::report_model::{SearchExperienceComponent, SearchExperiencePresentation};
use crate::seo::profile::PageType;

const W_TECHNICAL_SEO: u32 = 35;
const W_CONTENT: u32 = 25;
const W_TRUST: u32 = 15;
const W_AI: u32 = 10;
const W_STRUCTURE: u32 = 10;
const W_MOBILE: u32 = 5;

pub fn build_search_experience(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<SearchExperiencePresentation> {
    let seo = normalized.raw_seo?;
    #[cfg(feature = "experimental")]
    let en = is_english(i18n);

    let mut components = Vec::new();

    components.push(SearchExperienceComponent {
        label: i18n.t("seo-score-card"),
        score: seo.score,
        weight_pct: W_TECHNICAL_SEO,
        explanation: i18n.t("pdf-analysis-search-technical-seo-explanation"),
    });

    #[cfg(feature = "experimental")]
    let ux_ran = push_ux_components(&mut components, normalized, i18n, en);
    #[cfg(not(feature = "experimental"))]
    let ux_ran = false;
    if let Some(profile) = seo.content_profile.as_ref().filter(|_| !ux_ran) {
        components.push(SearchExperienceComponent {
            label: i18n.t("pdf-analysis-search-content-clarity"),
            score: profile.page_classification.content_depth_score,
            weight_pct: W_CONTENT,
            explanation: i18n.t("pdf-analysis-search-content-estimated"),
        });
        components.push(SearchExperienceComponent {
            label: i18n.t("pdf-analysis-search-structure"),
            score: profile.signal_strength.overall_pct,
            weight_pct: W_STRUCTURE,
            explanation: i18n.t("pdf-analysis-search-structure-estimated"),
        });
    }

    #[cfg(feature = "experimental")]
    if let Some(ai) = normalized.raw_ai_visibility {
        let ai_score = weighted_average(&[
            (ai.readability.dimension.score, 40),
            (ai.citation.dimension.score, 30),
            (ai.chunks.dimension.score, 30),
        ]);
        components.push(SearchExperienceComponent {
            label: i18n.t("pdf-analysis-search-ai"),
            score: ai_score,
            weight_pct: W_AI,
            explanation: i18n.t("pdf-analysis-search-ai-explanation"),
        });
    }

    if let Some(mobile) = normalized.raw_mobile {
        components.push(SearchExperienceComponent {
            label: i18n.t("pdf-analysis-search-mobile"),
            score: mobile.score,
            weight_pct: W_MOBILE,
            explanation: i18n.t("pdf-analysis-search-mobile-explanation"),
        });
    }

    let score = composite_score(&components);
    let not_measured = [
        ("pdf-analysis-search-content-clarity", W_CONTENT),
        ("pdf-analysis-search-trust", W_TRUST),
        ("pdf-analysis-search-structure", W_STRUCTURE),
        ("pdf-analysis-search-ai", W_AI),
        ("pdf-analysis-search-mobile", W_MOBILE),
    ]
    .iter()
    .map(|(key, _)| i18n.t(key))
    .filter(|label| !components.iter().any(|c| &c.label == label))
    .collect();
    apply_effective_weights(&mut components);
    let mut warnings = build_warnings(normalized, score, i18n);
    warnings.truncate(5);

    Some(SearchExperiencePresentation {
        score,
        label: i18n.t("pdf-analysis-search-label"),
        interpretation: interpretation(score, seo.score, &components, i18n),
        components,
        not_measured,
        warnings,
    })
}

/// The `ux` indicator's share of the composite: content clarity, trust and
/// structure. Returns whether the indicator ran; without it the SEO content
/// profile stands in for the estimate.
#[cfg(feature = "experimental")]
fn push_ux_components(
    components: &mut Vec<SearchExperienceComponent>,
    normalized: &AuditContext<'_>,
    i18n: &I18n,
    en: bool,
) -> bool {
    let Some(ux) = normalized.raw_ux else {
        return false;
    };
    let has_issue_for = |kind: crate::ux::UxDimensionKind| {
        ux.issues.iter().any(|issue| issue.kind.dimension() == kind)
    };
    components.push(SearchExperienceComponent {
        label: i18n.t("pdf-analysis-search-content-clarity"),
        score: ux.content_clarity.score,
        weight_pct: W_CONTENT,
        // Re-derive in the run locale; the stored summary is canonical
        // English and would otherwise leak into the German report (#406).
        explanation: crate::ux::ux_dimension_summary(
            ux.content_clarity.kind,
            ux.content_clarity.score,
            en,
            has_issue_for(ux.content_clarity.kind),
        ),
    });
    components.push(SearchExperienceComponent {
        label: i18n.t("pdf-analysis-search-trust"),
        score: trust_score(normalized, ux.trust_signals.score),
        weight_pct: W_TRUST,
        explanation: crate::ux::ux_dimension_summary(
            ux.trust_signals.kind,
            ux.trust_signals.score,
            en,
            has_issue_for(ux.trust_signals.kind),
        ),
    });
    components.push(SearchExperienceComponent {
        label: i18n.t("pdf-analysis-search-structure"),
        score: structure_score(normalized, ux.visual_hierarchy.score),
        weight_pct: W_STRUCTURE,
        explanation: crate::ux::ux_dimension_summary(
            ux.visual_hierarchy.kind,
            ux.visual_hierarchy.score,
            en,
            has_issue_for(ux.visual_hierarchy.kind),
        ),
    });
    true
}

fn composite_score(components: &[SearchExperienceComponent]) -> u32 {
    let total_weight: u32 = components.iter().map(|c| c.weight_pct).sum();
    if total_weight == 0 {
        return 0;
    }
    let weighted: u32 = components.iter().map(|c| c.score * c.weight_pct).sum();
    ((weighted as f64 / total_weight as f64).round() as u32).min(100)
}

/// Replaces the nominal weights by the weights the composite actually used
/// (renormalized over the present components), as whole percents summing to
/// exactly 100 (largest remainder), so the score can be recomputed from the rows.
fn apply_effective_weights(components: &mut [SearchExperienceComponent]) {
    let total: u32 = components.iter().map(|c| c.weight_pct).sum();
    if total == 0 {
        return;
    }
    let mut effective: Vec<(u32, u32)> = components
        .iter()
        .map(|c| (c.weight_pct * 100 / total, c.weight_pct * 100 % total))
        .collect();
    let mut missing = 100 - effective.iter().map(|(w, _)| w).sum::<u32>();
    let mut order: Vec<usize> = (0..effective.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(effective[i].1));
    for i in order {
        if missing == 0 {
            break;
        }
        effective[i].0 += 1;
        missing -= 1;
    }
    for (component, (weight, _)) in components.iter_mut().zip(effective) {
        component.weight_pct = weight;
    }
}

#[cfg(feature = "experimental")]
fn weighted_average(items: &[(u32, u32)]) -> u32 {
    let total_weight: u32 = items.iter().map(|(_, w)| *w).sum();
    if total_weight == 0 {
        return 0;
    }
    let sum: u32 = items.iter().map(|(score, weight)| score * weight).sum();
    ((sum as f64 / total_weight as f64).round() as u32).min(100)
}

#[cfg(feature = "experimental")]
fn trust_score(normalized: &AuditContext<'_>, ux_score: u32) -> u32 {
    let local_business_penalty = normalized
        .raw_content_visibility
        .map(|cv| {
            cv.local_business
                .iter()
                .filter(|s| {
                    matches!(
                        s.level,
                        AssessmentLevel::Warning | AssessmentLevel::Violation
                    )
                })
                .count() as u32
                * 8
        })
        .unwrap_or(0);
    ux_score.saturating_sub(local_business_penalty.min(24))
}

#[cfg(feature = "experimental")]
fn structure_score(normalized: &AuditContext<'_>, ux_score: u32) -> u32 {
    let seo_structure = normalized
        .raw_seo
        .and_then(|seo| seo.content_profile.as_ref())
        .map(|profile| {
            let heading = category_score(profile, "Überschriften")
                .or_else(|| category_score(profile, "Headings"))
                .unwrap_or(profile.signal_strength.overall_pct);
            let schema = category_score(profile, "Strukturierte Daten")
                .or_else(|| category_score(profile, "Structured data"))
                .unwrap_or(profile.signal_strength.overall_pct);
            weighted_average(&[(heading, 55), (schema, 45)])
        });
    match seo_structure {
        Some(score) => weighted_average(&[(ux_score, 55), (score, 45)]),
        None => ux_score,
    }
}

#[cfg(feature = "experimental")]
fn category_score(profile: &crate::seo::profile::SeoContentProfile, name: &str) -> Option<u32> {
    profile
        .signal_strength
        .categories
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.pct())
}

fn build_warnings(normalized: &AuditContext<'_>, score: u32, i18n: &I18n) -> Vec<String> {
    #[cfg(feature = "experimental")]
    let en = is_english(i18n);
    let mut warnings = Vec::new();
    if let Some(seo) = normalized.raw_seo {
        if seo.score >= 75 && score + 10 < seo.score {
            warnings.push(i18n.t_args(
                "pdf-analysis-search-warning-seo",
                &[("score", seo.score.to_string())],
            ));
        }
        if seo.technical.word_count < 900
            && seo
                .image_efficiency
                .as_ref()
                .is_some_and(|ie| ie.total_images >= 30)
        {
            warnings.push(
                i18n.t_args(
                    "pdf-analysis-search-warning-images",
                    &[
                        ("words", seo.technical.word_count.to_string()),
                        (
                            "images",
                            seo.image_efficiency
                                .as_ref()
                                .map(|ie| ie.total_images)
                                .unwrap_or(0)
                                .to_string(),
                        ),
                    ],
                ),
            );
        }
        if seo
            .content_profile
            .as_ref()
            .is_some_and(|p| p.page_classification.primary_type == PageType::MediaHeavy)
        {
            warnings.push(i18n.t("pdf-analysis-search-warning-media"));
        }
    }

    #[cfg(feature = "experimental")]
    if let Some(ux) = normalized.raw_ux {
        if ux.content_clarity.score < 70 {
            warnings.push(format!(
                "{}: {} / 100 — {}",
                i18n.t("pdf-analysis-search-content-clarity"),
                ux.content_clarity.score,
                // Re-derive in the run locale; the stored summary is canonical
                // English and would otherwise leak into the German PDF (#406).
                crate::ux::ux_dimension_summary(
                    ux.content_clarity.kind,
                    ux.content_clarity.score,
                    en,
                    false,
                )
            ));
        }
        if ux.trust_signals.score < 70 {
            warnings.push(format!(
                "{}: {} / 100 — {}",
                i18n.t("pdf-analysis-search-trust"),
                ux.trust_signals.score,
                crate::ux::ux_dimension_summary(
                    ux.trust_signals.kind,
                    ux.trust_signals.score,
                    en,
                    false,
                )
            ));
        }
    }

    #[cfg(feature = "experimental")]
    if let Some(cv) = normalized.raw_content_visibility {
        if cv.local_business.iter().any(|s| s.level.is_problem()) {
            warnings.push(i18n.t("pdf-analysis-search-warning-local"));
        }
    }

    warnings
}

fn interpretation(
    score: u32,
    technical_seo: u32,
    components: &[SearchExperienceComponent],
    i18n: &I18n,
) -> String {
    let weak: Vec<&SearchExperienceComponent> =
        components.iter().filter(|c| c.score < 70).collect();
    if technical_seo >= 75 && !weak.is_empty() && score + 10 < technical_seo {
        let names = weak
            .iter()
            .map(|c| c.label.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        i18n.t_args(
            "pdf-analysis-search-interpretation-gap",
            &[("names", names)],
        )
    } else if score >= 80 {
        i18n.t("pdf-analysis-search-interpretation-strong")
    } else if score >= 60 {
        i18n.t("pdf-analysis-search-interpretation-solid")
    } else {
        i18n.t("pdf-analysis-search-interpretation-limited")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_normalizes_missing_weights() {
        let components = vec![
            SearchExperienceComponent {
                label: "SEO".into(),
                score: 80,
                weight_pct: 35,
                explanation: String::new(),
            },
            SearchExperienceComponent {
                label: "Content".into(),
                score: 40,
                weight_pct: 25,
                explanation: String::new(),
            },
        ];

        assert_eq!(composite_score(&components), 63);
    }

    #[test]
    fn effective_weights_sum_to_100_and_reproduce_the_score() {
        let mut components: Vec<SearchExperienceComponent> =
            [(88, 35), (69, 25), (60, 10), (75, 5)]
                .iter()
                .map(|&(score, weight_pct)| SearchExperienceComponent {
                    label: String::new(),
                    score,
                    weight_pct,
                    explanation: String::new(),
                })
                .collect();
        let score = composite_score(&components);
        apply_effective_weights(&mut components);
        assert_eq!(components.iter().map(|c| c.weight_pct).sum::<u32>(), 100);
        let weighted: f64 = components
            .iter()
            .map(|c| c.score as f64 * c.weight_pct as f64 / 100.0)
            .sum();
        assert!((weighted - score as f64).abs() <= 1.0);
    }

    #[test]
    fn interpretation_mentions_gap_to_technical_seo() {
        let components = vec![
            SearchExperienceComponent {
                label: "Technical SEO".into(),
                score: 82,
                weight_pct: 35,
                explanation: String::new(),
            },
            SearchExperienceComponent {
                label: "Trust".into(),
                score: 35,
                weight_pct: 15,
                explanation: String::new(),
            },
        ];

        let text = interpretation(68, 82, &components, &I18n::new("de").unwrap());
        assert!(text.contains("Technisch auffindbar"));
        assert!(text.contains("Trust"));
    }
}
