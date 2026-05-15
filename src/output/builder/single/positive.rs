use crate::audit::normalized::NormalizedReport;
use crate::output::report_model::{
    ExampleBlock, FindingGroup, FindingPatternCluster, PositiveAspect, RepresentativeOccurrence,
    RoleAssignment,
};

pub(super) fn derive_positive_aspects_from_normalized(
    locale: &str,
    normalized: &NormalizedReport,
) -> Vec<PositiveAspect> {
    let en = locale == "en";
    let mut positives = Vec::new();
    let a11y_score = normalized.score as f32;

    let area_a11y: String = if en {
        "Accessibility".into()
    } else {
        "Barrierefreiheit".into()
    };

    if normalized.findings.is_empty() {
        positives.push(PositiveAspect {
            area: area_a11y.clone(),
            description: if en {
                "No automatically detectable violations found.".into()
            } else {
                "Keine automatisch erkennbaren Verstöße gefunden.".into()
            },
        });
    } else if a11y_score >= 80.0 {
        positives.push(PositiveAspect {
            area: area_a11y,
            description: if en {
                "Solid base quality with focused, prioritizable remaining items.".into()
            } else {
                "Solide Grundqualität mit gezielt priorisierbaren Restpunkten.".into()
            },
        });
    }

    if let Some(ref perf) = normalized.raw_performance {
        if perf.score.overall >= 80 {
            positives.push(PositiveAspect {
                area: "Performance".into(),
                description: if en {
                    "Stable load times and overall responsive page build-up.".into()
                } else {
                    "Stabile Ladezeiten und insgesamt reaktionsschneller Seitenaufbau.".into()
                },
            });
        }
    }
    if let Some(ref seo) = normalized.raw_seo {
        if seo.score >= 80 {
            positives.push(PositiveAspect {
                area: "SEO".into(),
                description: if en {
                    "Clean foundation for discoverability, structure and meta data.".into()
                } else {
                    "Saubere Basis für Auffindbarkeit, Struktur und Meta-Daten.".into()
                },
            });
        }
    }
    if let Some(ref sec) = normalized.raw_security {
        if sec.score >= 80 {
            positives.push(PositiveAspect {
                area: if en {
                    "Security".into()
                } else {
                    "Sicherheit".into()
                },
                description: if en {
                    "Key security mechanisms are fundamentally in place.".into()
                } else {
                    "Wichtige Sicherheitsmechanismen sind grundsätzlich vorhanden.".into()
                },
            });
        }
    }
    if let Some(ref mobile) = normalized.raw_mobile {
        if mobile.score >= 80 {
            positives.push(PositiveAspect {
                area: "Mobile".into(),
                description: if en {
                    "The site is usable and readable on small displays.".into()
                } else {
                    "Die Seite ist auf kleinen Displays gut bedienbar und lesbar.".into()
                },
            });
        }
    }

    if positives.is_empty() {
        positives.push(PositiveAspect {
            area: if en {
                "Base structure".into()
            } else {
                "Grundstruktur".into()
            },
            description: if en {
                "The site is fundamentally functional and reachable.".into()
            } else {
                "Die Seite ist grundsätzlich funktional und erreichbar.".into()
            },
        });
    }
    positives
}

// ─── Clone implementations ──────────────────────────────────────────────────

impl Clone for FindingGroup {
    fn clone(&self) -> Self {
        FindingGroup {
            title: self.title.clone(),
            rule_id: self.rule_id.clone(),
            wcag_criterion: self.wcag_criterion.clone(),
            wcag_level: self.wcag_level.clone(),
            dimension: self.dimension.clone(),
            subcategory: self.subcategory.clone(),
            issue_class: self.issue_class.clone(),
            severity: self.severity,
            priority: self.priority,
            customer_description: self.customer_description.clone(),
            user_impact: self.user_impact.clone(),
            business_impact: self.business_impact.clone(),
            typical_cause: self.typical_cause.clone(),
            recommendation: self.recommendation.clone(),
            technical_note: self.technical_note.clone(),
            occurrence_count: self.occurrence_count,
            affected_urls: self.affected_urls.clone(),
            affected_elements: self.affected_elements,
            additional_occurrences: self.additional_occurrences,
            pattern_clusters: self
                .pattern_clusters
                .iter()
                .map(|cluster| FindingPatternCluster {
                    label: cluster.label.clone(),
                    occurrences: cluster.occurrences,
                })
                .collect(),
            location_hints: self.location_hints.clone(),
            representative_occurrences: self
                .representative_occurrences
                .iter()
                .map(|occ| RepresentativeOccurrence {
                    selector: occ.selector.clone(),
                    node_id: occ.node_id.clone(),
                    message: occ.message.clone(),
                    html_snippet: occ.html_snippet.clone(),
                    suggested_code: occ.suggested_code.clone(),
                })
                .collect(),
            responsible_role: self.responsible_role,
            effort: self.effort,
            execution_priority: self.execution_priority,
            examples: self.examples.clone(),
        }
    }
}

impl Clone for ExampleBlock {
    fn clone(&self) -> Self {
        ExampleBlock {
            bad: self.bad.clone(),
            good: self.good.clone(),
            decorative: self.decorative.clone(),
        }
    }
}

impl Clone for RoleAssignment {
    fn clone(&self) -> Self {
        RoleAssignment {
            role: self.role,
            responsibilities: self.responsibilities.clone(),
        }
    }
}
