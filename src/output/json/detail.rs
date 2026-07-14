use super::helpers::*;
use super::*;

#[derive(Default)]
pub(super) struct DetailContext {
    pub(super) budget_violations: Vec<serde_json::Value>,
    pub(super) screenshot_status: Option<serde_json::Value>,
    pub(super) collection_errors: Vec<ReportError>,
}

/// Build a [`PageEntry`]. `audit_ctx` `Some` builds the full single-report `detail`
/// using raw module data; `None` builds a minimal cached detail.
/// `detail_ctx` `None` leaves `detail` unset (batch callers attach a compact detail).
pub(super) fn build_page(
    normalized: &NormalizedReport,
    audit_ctx: Option<&AuditContext<'_>>,
    detail_ctx: Option<DetailContext>,
) -> PageEntry {
    let detail = detail_ctx.map(|d| {
        if let Some(ctx) = audit_ctx {
            build_detail(ctx, d)
        } else {
            build_detail_cached(normalized, d)
        }
    });

    PageEntry {
        url: normalized.url.clone(),
        accessibility_score: normalized.score,
        overall_score: normalized.overall_score,
        grade: normalized.grade.clone(),
        certificate: normalized.certificate.clone(),
        // Counts cover all finding categories (WCAG + SEO), matching the
        // contents of `findings` and `detail.fix_guidance` (issues #254, #255).
        // `severity_counts` stays WCAG-only (legal/risk semantics, see spec).
        violation_count: normalized.findings.iter().map(|f| f.occurrence_count).sum(),
        violated_rule_count: distinct_rule_count(&normalized.findings),
        severity_counts: normalized.severity_counts.clone(),
        severity_counts_scope: "wcag_only".to_string(),
        occurrence_counts: all_category_occurrence_counts(&normalized.findings),
        nodes_analyzed: normalized.nodes_analyzed,
        duration_ms: normalized.duration_ms,
        module_scores: normalized.module_scores.clone(),
        performance_score: normalized_module_score(normalized, "Performance"),
        seo_score: normalized_module_score(normalized, "SEO"),
        security_score: normalized_module_score(normalized, "Security"),
        mobile_score: normalized_module_score(normalized, "Mobile"),
        ux_score: normalized_module_score(normalized, "UX"),
        journey_score: normalized_module_score(normalized, "Journey"),
        score_calculation_method: normalized.score_calculation_method.clone(),
        score_breakdown: normalized.score_breakdown.clone(),
        risk: normalized.risk.clone(),
        principle_coverage: normalized.principle_coverage.clone(),
        findings: normalized.findings.clone(),
        audit_flags: normalized.audit_flags.clone(),
        consent_privacy: normalized.consent_privacy.clone(),
        interactive_findings: normalized.interactive_findings.clone(),
        accessibility_journey: normalized.accessibility_journey.clone(),
        screen_reader: normalized.screen_reader.clone(),
        detail,
    }
}

/// Compact per-page detail for batch reports: actionable `fix_guidance` only,
/// without the heavy module blob. Keeps batch reports from devolving into a
/// stack of single-page reports (see CLAUDE.md "Report Intent") while honouring
/// the contract that `detail.fix_guidance` is always present (issue #256).
pub(super) fn build_batch_detail(normalized: &NormalizedReport) -> PageDetail {
    PageDetail {
        fix_guidance: build_fix_guidance(normalized),
        en301549_annex: build_en301549_annex(&normalized.findings),
        modules: ModuleBlob::default(),
        confidence_summary: Vec::new(),
        capabilities: Vec::new(),
        viewport_scores: None,
        budget_violations: Vec::new(),
        throttled_performance: Vec::new(),
        screenshot_status: None,
        collection_errors: Vec::new(),
    }
}

/// Minimal detail for the cached/from-normalized path — no raw module data available.
pub(super) fn build_detail_cached(
    normalized: &NormalizedReport,
    detail_ctx: DetailContext,
) -> PageDetail {
    PageDetail {
        fix_guidance: build_fix_guidance(normalized),
        en301549_annex: build_en301549_annex(&normalized.findings),
        modules: ModuleBlob::default(),
        confidence_summary: Vec::new(),
        capabilities: Vec::new(),
        viewport_scores: normalized.viewport_scores.clone(),
        budget_violations: detail_ctx.budget_violations,
        throttled_performance: Vec::new(),
        screenshot_status: detail_ctx.screenshot_status,
        collection_errors: detail_ctx.collection_errors,
    }
}

pub(super) fn build_detail(ctx: &AuditContext<'_>, detail_ctx: DetailContext) -> PageDetail {
    // JSON is canonical English (#406) — always build the view model with "en".
    let vm = build_view_model(
        ctx,
        &ReportConfig {
            locale: "en".to_string(),
            ..ReportConfig::default()
        },
    );
    let normalized: &NormalizedReport = &ctx.normalized;
    let mut errors = detail_ctx.collection_errors;

    let wcag_findings: Vec<_> = normalized
        .findings
        .iter()
        .filter(|f| f.category == "wcag")
        .collect();
    let seo_findings: Vec<_> = normalized
        .findings
        .iter()
        .filter(|f| f.category == "seo")
        .collect();

    let tech_stack = ctx.raw_tech_stack.and_then(|m| {
        serde_json::to_value(m)
            .map_err(|e| {
                errors.push(ReportError {
                    module: "tech_stack",
                    error_type: "serialization_failed",
                    reason: e.to_string(),
                })
            })
            .ok()
    });

    let search_experience = vm.module_details.search_experience.as_ref().map(|sx| {
        serde_json::json!({
            "score": sx.score,
            "label": sx.label.clone(),
            "interpretation": sx.interpretation.clone(),
            "components": sx.components.iter().map(|component| {
                serde_json::json!({
                    "label": component.label.clone(),
                    "score": component.score,
                    "weight_pct": component.weight_pct,
                    "explanation": component.explanation.clone(),
                })
            }).collect::<Vec<_>>(),
            "warnings": sx.warnings.clone(),
            "measurement_type": "composite",
        })
    });

    let dual_viewport = ctx.raw_dual_viewport.map(|dual| {
        serde_json::json!({
            "desktop": viewport_detail_summary(&dual.desktop),
            "mobile": viewport_detail_summary(&dual.mobile),
        })
    });

    let patterns = ctx.raw_patterns.map(|m| {
        let total = m.recognized.len() + m.violations.len();
        let pattern_score: u32 = if total > 0 {
            (m.recognized.len() as u32 * 100) / total as u32
        } else {
            75
        };
        match serde_json::to_value(m) {
            Ok(mut v) => {
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("score".to_string(), serde_json::json!(pattern_score));
                    obj.insert("grade".to_string(), serde_json::json!(
                        crate::audit::AccessibilityScorer::calculate_grade(pattern_score as f32)
                    ));
                }
                v
            }
            Err(e) => {
                errors.push(ReportError {
                    module: "patterns",
                    error_type: "serialization_failed",
                    reason: e.to_string(),
                });
                serde_json::json!({
                    "score": pattern_score,
                    "grade": crate::audit::AccessibilityScorer::calculate_grade(pattern_score as f32)
                })
            }
        }
    });

    let throttled_performance: Vec<serde_json::Value> = {
        let mut acc = Vec::new();
        for v in ctx.raw_throttled_performance.iter() {
            match serde_json::to_value(v) {
                Ok(json) => acc.push(json),
                Err(e) => errors.push(ReportError {
                    module: "throttled_performance",
                    error_type: "serialization_failed",
                    reason: e.to_string(),
                }),
            }
        }
        acc
    };

    let seo = ctx.raw_seo.map(|m| {
        let mut v = with_normalized_score(m.to_json(), normalized, "SEO");
        let findings_value = match serde_json::to_value(&seo_findings) {
            Ok(json) => json,
            Err(e) => {
                errors.push(ReportError {
                    module: "seo",
                    error_type: "findings_serialization_failed",
                    reason: e.to_string(),
                });
                serde_json::json!([])
            }
        };
        if let Some(obj) = v.as_object_mut() {
            obj.insert("findings".to_string(), findings_value);
        }
        v
    });

    let modules = ModuleBlob {
        accessibility: Some(serde_json::json!({
            "score": normalized.score,
            "grade": normalized.grade,
            "severity_counts": normalized.severity_counts,
            "principle_coverage": normalized.principle_coverage,
            "findings": wcag_findings,
        })),
        dual_viewport,
        search_experience,
        performance: ctx.raw_performance.map(|m| {
            inject_unused_js_bytes(
                with_normalized_score(m.to_json(), normalized, "Performance"),
                m,
            )
        }),
        seo,
        security: ctx
            .raw_security
            .map(|m| with_normalized_score(m.to_json(), normalized, "Security")),
        mobile: ctx
            .raw_mobile
            .map(|m| with_normalized_score(m.to_json(), normalized, "Mobile")),
        ux: ctx
            .raw_ux
            .map(|m| with_normalized_score(m.to_json(), normalized, "UX")),
        journey: ctx
            .raw_journey
            .map(|m| with_normalized_score(m.to_json(), normalized, "Journey")),
        dark_mode: ctx
            .raw_dark_mode
            .map(|m| inject_grade(m.to_json(), m.score)),
        source_quality: ctx
            .raw_source_quality
            .map(|m| with_measurement_type(m.to_json(), "heuristic")),
        ai_visibility: ctx
            .raw_ai_visibility
            .map(|m| with_measurement_type(m.to_json(), "heuristic")),
        content_visibility: ctx.raw_content_visibility.and_then(|m| {
            // Only emit when SEO data was available — all signal sections are empty
            // without --full, which is misleading.
            if m.signal_count == 0 {
                return None;
            }
            let cv_score = (m.signal_count.saturating_sub(m.problem_count) as u32 * 100)
                / m.signal_count as u32;
            let mut v = with_measurement_type(m.to_json(), "heuristic");
            if let Some(obj) = v.as_object_mut() {
                obj.insert("score".to_string(), serde_json::json!(cv_score));
                obj.insert(
                    "grade".to_string(),
                    serde_json::json!(crate::audit::AccessibilityScorer::calculate_grade(
                        cv_score as f32
                    )),
                );
            }
            Some(v)
        }),
        tech_stack,
        patterns,
        best_practices: ctx
            .raw_best_practices
            .map(|m| with_normalized_score(m.to_json(), normalized, "Best Practices")),
        commerce: ctx.raw_commerce.map(|m| m.to_json()),
    };

    PageDetail {
        fix_guidance: build_fix_guidance(normalized),
        en301549_annex: build_en301549_annex(&normalized.findings),
        modules,
        confidence_summary: vm
            .methodology
            .confidence_summary
            .iter()
            .map(|(signal, assessment)| OutputConfidenceSignal {
                signal: signal.clone(),
                assessment: assessment.clone(),
            })
            .collect(),
        capabilities: vm
            .methodology
            .capabilities
            .iter()
            .map(|cap| OutputCapabilitySignal {
                signal: cap.signal.clone(),
                source: cap.source.clone(),
                confidence: cap.confidence.clone(),
                surfaces: cap.surfaces.clone(),
                note: cap.note.clone(),
            })
            .collect(),
        viewport_scores: normalized.viewport_scores.clone(),
        budget_violations: detail_ctx.budget_violations,
        throttled_performance,
        screenshot_status: detail_ctx.screenshot_status,
        collection_errors: errors,
    }
}

pub(super) fn viewport_detail_summary(data: &crate::audit::ViewportAuditData) -> serde_json::Value {
    serde_json::json!({
        "accessibility_score": data.accessibility_score.round().max(0.0) as u32,
        "wcag": {
            "violations": data.wcag_results.violations.len(),
            "warnings": data.wcag_results.warnings.len(),
            "positives": data.wcag_results.positives.len(),
            "not_testables": data.wcag_results.not_testables.len(),
            "nodes_checked": data.wcag_results.nodes_checked,
        },
        "modules": {
            "performance": data.performance.as_ref().map(|p| p.score.overall),
            "seo": data.seo.as_ref().map(|s| s.score),
            "mobile": data.mobile.as_ref().map(|m| m.score),
            "ux": data.ux.as_ref().map(|u| u.score),
            "journey": data.journey.as_ref().map(|j| j.score),
        },
        "has_screenshot": data.screenshot.is_some(),
    })
}

/// Build fix guidance entries from normalized findings + explanation database.
pub(super) fn build_fix_guidance(normalized: &NormalizedReport) -> Vec<FixGuidance> {
    normalized
        .findings
        .iter()
        .map(|finding| {
            let expl = get_explanation(&finding.rule_id);

            let mut seen = std::collections::HashSet::new();
            // Accept any non-empty selector that isn't a raw numeric node ID.
            // The old CSS-character filter was too strict and dropped plain tag
            // selectors (e.g. "a") from rules like color_link_indicator.
            let affected_selectors: Vec<String> = finding
                .occurrences
                .iter()
                .filter_map(|o| o.selector.clone())
                .filter(|s| {
                    !s.is_empty()
                        && !s.chars().all(|c| c.is_ascii_digit())
                        && seen.insert(s.clone())
                })
                .take(10)
                .collect();

            let code_example = expl.and_then(|e| match (e.example_bad, e.example_good) {
                (Some(bad), Some(good)) => Some(CodeExample {
                    bad: bad.to_string(),
                    good: good.to_string(),
                }),
                _ => None,
            });

            FixGuidance {
                rule_id: finding.rule_id.clone(),
                title: expl
                    .map(|e| e.customer_title_en.to_string())
                    .unwrap_or_else(|| finding.title.clone()),
                wcag_criterion: finding.wcag_criterion.clone(),
                severity: format!("{:?}", finding.severity).to_lowercase(),
                risk: format!("{:?}", finding.severity).to_lowercase(),
                remediation_priority: finding.remediation_priority.clone(),
                complexity: finding.complexity.clone(),
                complexity_reason: finding.complexity_reason.clone(),
                confidence: finding.confidence.clone(),
                false_positive_risk: finding.false_positive_risk.clone(),
                verification: finding.verification.clone(),
                expected_impact: finding.expected_impact.clone(),
                bfsg_relevance: finding.bfsg_relevance.clone(),
                occurrence_count: finding.occurrence_count,
                problem: expl
                    .map(|e| e.customer_description_en.to_string())
                    .unwrap_or_else(|| finding.description.clone()),
                user_impact: expl.map(|e| e.user_impact_en.to_string()).or_else(|| {
                    if finding.user_impact.is_empty() {
                        None
                    } else {
                        Some(finding.user_impact.clone())
                    }
                }),
                typical_cause: expl
                    .map(|e| e.typical_cause_en.to_string())
                    .filter(|s| !s.is_empty()),
                recommendation: expl
                    .map(|e| e.recommendation_en.to_string())
                    .filter(|s| !s.is_empty()),
                technical_note: expl
                    .map(|e| e.technical_note_en.to_string())
                    .filter(|s| !s.is_empty()),
                code_example,
                affected_selectors,
            }
        })
        .collect()
}
