//! Customer-facing Search Experience score.
//!
//! This is a presentation-level composite: it does not collect new data, but
//! combines existing SEO, UX, AI visibility, content visibility and mobile
//! signals so the front report does not overstate classic technical SEO.

use crate::assessment::AssessmentLevel;
use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
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
    let en = i18n.locale() == "en";

    let mut components = Vec::new();

    components.push(SearchExperienceComponent {
        label: if en {
            "Technical SEO".into()
        } else {
            "Technisches SEO".into()
        },
        score: seo.score,
        weight_pct: W_TECHNICAL_SEO,
        explanation: if en {
            "Title, description, canonical, sitemap, structured data and crawlable links."
        } else {
            "Title, Description, Canonical, Sitemap, strukturierte Daten und crawlbare Links."
        }
        .into(),
    });

    if let Some(ux) = normalized.raw_ux {
        let has_issue_for = |kind: crate::ux::UxDimensionKind| {
            ux.issues.iter().any(|issue| issue.kind.dimension() == kind)
        };
        components.push(SearchExperienceComponent {
            label: if en {
                "Content clarity".into()
            } else {
                "Inhaltsverständlichkeit".into()
            },
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
            label: if en {
                "Trust signals".into()
            } else {
                "Vertrauenssignale".into()
            },
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
            label: if en {
                "Structure and semantics".into()
            } else {
                "Struktur und Semantik".into()
            },
            score: structure_score(normalized, ux.visual_hierarchy.score),
            weight_pct: W_STRUCTURE,
            explanation: crate::ux::ux_dimension_summary(
                ux.visual_hierarchy.kind,
                ux.visual_hierarchy.score,
                en,
                has_issue_for(ux.visual_hierarchy.kind),
            ),
        });
    } else if let Some(profile) = seo.content_profile.as_ref() {
        components.push(SearchExperienceComponent {
            label: if en {
                "Content clarity".into()
            } else {
                "Inhaltsverständlichkeit".into()
            },
            score: profile.page_classification.content_depth_score,
            weight_pct: W_CONTENT,
            explanation: if en {
                "Estimated from page type and content-depth signals."
            } else {
                "Aus Seitentyp und Content-Tiefe abgeleitet."
            }
            .into(),
        });
        components.push(SearchExperienceComponent {
            label: if en {
                "Structure and semantics".into()
            } else {
                "Struktur und Semantik".into()
            },
            score: profile.signal_strength.overall_pct,
            weight_pct: W_STRUCTURE,
            explanation: if en {
                "Estimated from headings, structured data and page profile signals."
            } else {
                "Aus Überschriften, strukturierten Daten und Seitenprofil abgeleitet."
            }
            .into(),
        });
    }

    if let Some(ai) = normalized.raw_ai_visibility {
        let ai_score = weighted_average(&[
            (ai.readability.dimension.score, 40),
            (ai.citation.dimension.score, 30),
            (ai.chunks.dimension.score, 30),
        ]);
        components.push(SearchExperienceComponent {
            label: if en {
                "AI readability".into()
            } else {
                "KI-Lesbarkeit".into()
            },
            score: ai_score,
            weight_pct: W_AI,
            explanation: if en {
                "Readability, chunk quality and citation likelihood for AI-assisted systems."
            } else {
                "Lesbarkeit, Abschnittsqualität und Zitierbarkeit für KI-gestützte Systeme."
            }
            .into(),
        });
    }

    if let Some(mobile) = normalized.raw_mobile {
        components.push(SearchExperienceComponent {
            label: if en {
                "Mobile readability".into()
            } else {
                "Mobile Lesbarkeit".into()
            },
            score: mobile.score,
            weight_pct: W_MOBILE,
            explanation: if en {
                "Mobile usability, readable text and tappable controls."
            } else {
                "Mobile Nutzbarkeit, lesbarer Text und gut antippbare Bedienelemente."
            }
            .into(),
        });
    }

    let score = composite_score(&components);
    let mut warnings = build_warnings(normalized, score, en);
    warnings.truncate(5);

    Some(SearchExperiencePresentation {
        score,
        label: if en {
            "Search Experience".into()
        } else {
            "Sichtbarkeit & Nutzerverständnis".into()
        },
        interpretation: interpretation(score, seo.score, &components, en),
        components,
        warnings,
    })
}

fn composite_score(components: &[SearchExperienceComponent]) -> u32 {
    let total_weight: u32 = components.iter().map(|c| c.weight_pct).sum();
    if total_weight == 0 {
        return 0;
    }
    let weighted: u32 = components.iter().map(|c| c.score * c.weight_pct).sum();
    ((weighted as f64 / total_weight as f64).round() as u32).min(100)
}

fn weighted_average(items: &[(u32, u32)]) -> u32 {
    let total_weight: u32 = items.iter().map(|(_, w)| *w).sum();
    if total_weight == 0 {
        return 0;
    }
    let sum: u32 = items.iter().map(|(score, weight)| score * weight).sum();
    ((sum as f64 / total_weight as f64).round() as u32).min(100)
}

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

fn category_score(profile: &crate::seo::profile::SeoContentProfile, name: &str) -> Option<u32> {
    profile
        .signal_strength
        .categories
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.score_pct)
}

fn build_warnings(normalized: &AuditContext<'_>, score: u32, en: bool) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(seo) = normalized.raw_seo {
        if seo.score >= 75 && score + 10 < seo.score {
            warnings.push(if en {
                format!(
                    "Technical SEO is {} / 100, but content, trust or AI signals reduce the customer-facing visibility score.",
                    seo.score
                )
            } else {
                format!(
                    "Technisches SEO liegt bei {} / 100, aber Content-, Trust- oder KI-Signale senken die kundennahe Sichtbarkeit.",
                    seo.score
                )
            });
        }
        if seo.technical.word_count < 900
            && seo
                .image_efficiency
                .as_ref()
                .is_some_and(|ie| ie.total_images >= 30)
        {
            warnings.push(if en {
                format!(
                    "{} words with {} images: important messages may be image-led rather than readable HTML.",
                    seo.technical.word_count,
                    seo.image_efficiency.as_ref().map(|ie| ie.total_images).unwrap_or(0)
                )
            } else {
                format!(
                    "{} Wörter bei {} Bildern: wichtige Aussagen können eher im Bild als im lesbaren HTML liegen.",
                    seo.technical.word_count,
                    seo.image_efficiency.as_ref().map(|ie| ie.total_images).unwrap_or(0)
                )
            });
        }
        if seo
            .content_profile
            .as_ref()
            .is_some_and(|p| p.page_classification.primary_type == PageType::MediaHeavy)
        {
            warnings.push(if en {
                "The page profile is media-heavy; image text should be backed by readable HTML copy."
                    .into()
            } else {
                "Das Seitenprofil ist medienlastig; Bildtext sollte durch lesbaren HTML-Text abgesichert werden."
                    .into()
            });
        }
    }

    if let Some(ux) = normalized.raw_ux {
        if ux.content_clarity.score < 70 {
            warnings.push(format!(
                "{}: {} / 100 — {}",
                if en {
                    "Content clarity"
                } else {
                    "Inhaltsverständlichkeit"
                },
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
                if en {
                    "Trust signals"
                } else {
                    "Vertrauenssignale"
                },
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

    if let Some(cv) = normalized.raw_content_visibility {
        if cv.local_business.iter().any(|s| s.level.is_problem()) {
            warnings.push(if en {
                "LocalBusiness or local trust data is incomplete or not machine-readable."
                    .into()
            } else {
                "LocalBusiness- oder lokale Vertrauensdaten sind unvollstaendig oder nicht maschinenlesbar."
                    .into()
            });
        }
    }

    warnings
}

fn interpretation(
    score: u32,
    technical_seo: u32,
    components: &[SearchExperienceComponent],
    en: bool,
) -> String {
    let weak: Vec<&SearchExperienceComponent> =
        components.iter().filter(|c| c.score < 70).collect();
    if technical_seo >= 75 && !weak.is_empty() && score + 10 < technical_seo {
        let names = weak
            .iter()
            .map(|c| c.label.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if en {
            format!(
                "Technically findable, but not equally strong for users and AI systems. Weak parts: {names}."
            )
        } else {
            format!(
                "Technisch auffindbar, aber für Nutzer und KI-Systeme nicht gleich stark verständlich. Schwächere Bestandteile: {names}."
            )
        }
    } else if score >= 80 {
        if en {
            "Strong search experience: technical visibility, content clarity and trust signals align."
        } else {
            "Starke Search Experience: technische Auffindbarkeit, Inhaltsverständlichkeit und Vertrauen passen zusammen."
        }
        .into()
    } else if score >= 60 {
        if en {
            "Solid base with visible gaps in content, trust or machine readability."
        } else {
            "Solide Basis mit sichtbaren Lücken bei Inhalt, Vertrauen oder maschineller Lesbarkeit."
        }
        .into()
    } else {
        if en {
            "Limited search experience: the page is not sufficiently understandable, trustworthy or extractable."
        } else {
            "Eingeschränkte Search Experience: Die Seite ist nicht ausreichend verständlich, vertrauensstark oder extrahierbar."
        }
        .into()
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

        let text = interpretation(68, 82, &components, false);
        assert!(text.contains("Technisch auffindbar"));
        assert!(text.contains("Trust"));
    }
}
