use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
use crate::output::report_model::{
    ExampleBlock, FindingGroup, FindingPatternCluster, NarrativeArc, PositiveAspect,
    RepresentativeOccurrence, RoleAssignment,
};

pub(super) fn derive_positive_aspects_from_normalized(
    i18n: &I18n,
    normalized: &AuditContext<'_>,
) -> Vec<PositiveAspect> {
    let mut positives = Vec::new();
    let a11y_score = normalized.normalized.score as f32;

    let area_a11y = i18n.t("module-accessibility");

    if normalized.normalized.findings.is_empty() {
        positives.push(PositiveAspect {
            area: area_a11y.clone(),
            description: i18n.t("report-builder-aspect-no-violations"),
        });
    } else if a11y_score >= 80.0 {
        positives.push(PositiveAspect {
            area: area_a11y,
            description: i18n.t("report-builder-aspect-stable-base"),
        });
    }

    if let Some(perf) = normalized.raw_performance {
        if perf.score.overall >= 80 {
            positives.push(PositiveAspect {
                area: "Performance".into(),
                description: i18n.t("report-builder-aspect-performance"),
            });
        }
    }
    if let Some(seo) = normalized.raw_seo {
        if seo.score >= 80 {
            positives.push(PositiveAspect {
                area: "SEO".into(),
                description: i18n.t("report-builder-aspect-seo"),
            });
        }
    }
    if let Some(sec) = normalized.raw_security {
        if sec.score >= 80 {
            positives.push(PositiveAspect {
                area: i18n.t("module-security"),
                description: i18n.t("report-builder-aspect-security"),
            });
        }
    }
    if let Some(mobile) = normalized.raw_mobile {
        if mobile.score >= 80 {
            positives.push(PositiveAspect {
                area: "Mobile".into(),
                description: i18n.t("report-builder-aspect-mobile"),
            });
        }
    }

    if positives.is_empty() {
        positives.push(PositiveAspect {
            area: i18n.t("report-builder-aspect-base-area"),
            description: i18n.t("report-builder-aspect-base"),
        });
    }
    positives
}

// ─── Clone implementations ──────────────────────────────────────────────────

impl Clone for FindingGroup {
    fn clone(&self) -> Self {
        FindingGroup {
            finding: self.finding.clone(),
            title: self.title.clone(),
            priority: self.priority,
            customer_description: self.customer_description.clone(),
            user_impact: self.user_impact.clone(),
            business_impact: self.business_impact.clone(),
            typical_cause: self.typical_cause.clone(),
            recommendation: self.recommendation.clone(),
            technical_note: self.technical_note.clone(),
            affected_urls: self.affected_urls.clone(),
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
                    evidence: occ.evidence.clone(),
                    evidence_screenshot: occ.evidence_screenshot.clone(),
                    evidence_viewport: occ.evidence_viewport,
                })
                .collect(),
            responsible_role: self.responsible_role,
            effort: self.effort,
            execution_priority: self.execution_priority,
            examples: self.examples.clone(),
            structural_cause: self.structural_cause.clone(),
            is_component_issue: self.is_component_issue,
            criticality_tier: self.criticality_tier,
            narrative: NarrativeArc {
                diagnose: self.narrative.diagnose.clone(),
                ursache: self.narrative.ursache.clone(),
                wirkung: self.narrative.wirkung.clone(),
                umsetzung: self.narrative.umsetzung.clone(),
            },
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
