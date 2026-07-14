use crate::audit::occurrence_analysis::{
    build_location_hints, build_pattern_clusters, build_representative_occurrences,
};
use crate::audit::prioritization::{derive_execution_priority, severity_to_priority};
use crate::i18n::I18n;
use crate::output::explanations::get_explanation;
use crate::output::report_model::{classify_criticality_tier, Effort, FindingGroup, Role};

use super::super::actions::{build_narrative_arc, derive_business_impact, localized_finding_text};

pub(super) fn finding_group_from_normalized(
    i18n: &I18n,
    f: &crate::audit::normalized::NormalizedFinding,
) -> FindingGroup {
    let locale = i18n.locale();
    // Try the taxonomy rule_id first (e.g. "a11y.aria_hidden_focus.invalid"),
    // then fall back to the WCAG criterion. Some rules carry their localized
    // explanation under the taxonomy key, not the WCAG number — looking up by
    // wcag_criterion alone left those findings with the raw English fix (#357).
    let explanation = get_explanation(&f.rule_id).or_else(|| get_explanation(&f.wcag_criterion));

    let (
        title,
        customer_desc,
        user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        role,
        effort,
        execution_priority,
    ) = if let Some(expl) = explanation {
        (
            expl.customer_title_for(locale).to_string(),
            expl.customer_description_for(locale).to_string(),
            expl.user_impact_for(locale).to_string(),
            derive_business_impact(
                i18n,
                expl.user_impact_for(locale),
                f.dimension.as_str(),
                f.severity,
                Some(f.subcategory_kind.label(false)),
                f.occurrence_count,
            ),
            expl.typical_cause_for(locale).to_string(),
            expl.recommendation_for(locale).to_string(),
            expl.technical_note_for(locale).to_string(),
            expl.responsible_role,
            expl.effort_estimate,
            derive_execution_priority(f.severity, expl.effort_estimate, f.dimension.as_str()),
        )
    } else {
        // JSON-stored title/user_impact are canonical English (#406); re-derive
        // the runtime-locale text from the taxonomy for the report.
        let (title_loc, user_impact_loc, _technical_loc) = localized_finding_text(locale, f);
        (
            title_loc,
            f.description.clone(),
            user_impact_loc.clone(),
            derive_business_impact(
                i18n,
                &user_impact_loc,
                f.dimension.as_str(),
                f.severity,
                Some(f.subcategory_kind.label(false)),
                f.occurrence_count,
            ),
            String::new(),
            f.occurrences
                .first()
                .and_then(|o| o.fix_suggestion.clone())
                .unwrap_or_default(),
            String::new(),
            Role::Development,
            Effort::Medium,
            derive_execution_priority(f.severity, Effort::Medium, f.dimension.as_str()),
        )
    };

    let examples = explanation.map(|e| e.examples()).unwrap_or_default();
    let location_hints = build_location_hints(&f.occurrences);
    let representative_occurrences = build_representative_occurrences(&f.occurrences);
    let pattern_clusters = build_pattern_clusters(&f.occurrences);
    let additional_occurrences = f
        .occurrence_count
        .saturating_sub(representative_occurrences.len());

    let narrative = build_narrative_arc(
        i18n,
        f.occurrence_count,
        f.severity,
        f.dimension.as_str(),
        &customer_desc,
        &user_impact_text,
        &business_impact,
        &typical_cause,
        &recommendation,
        effort,
        role,
    );

    FindingGroup {
        title,
        rule_id: f.rule_id.clone(),
        wcag_criterion: f.wcag_criterion.clone(),
        wcag_level: f.wcag_level.clone(),
        help_url: f.help_url.clone(),
        dimension: Some(f.dimension.clone()),
        subcategory: Some(f.subcategory.clone()),
        issue_class: Some(f.issue_class.clone()),
        severity: f.severity,
        priority: severity_to_priority(f.severity),
        customer_description: customer_desc,
        user_impact: user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        confidence: f.confidence.clone(),
        false_positive_risk: f.false_positive_risk.clone(),
        verification: f.verification.clone(),
        complexity: f.complexity.clone(),
        complexity_reason: f.complexity_reason.clone(),
        complexity_kind: f.complexity_kind,
        expected_impact: f.expected_impact.clone(),
        expected_impact_kind: f.expected_impact_kind.clone(),
        bfsg_relevance: f.bfsg_relevance.clone(),
        remediation_priority: f.remediation_priority.clone(),
        occurrence_count: f.occurrence_count,
        affected_urls: Vec::new(),
        affected_elements: f.occurrence_count,
        additional_occurrences,
        pattern_clusters,
        location_hints,
        representative_occurrences,
        responsible_role: role,
        effort,
        execution_priority,
        examples,
        structural_cause: if f.occurrence_count >= 10 {
            Some(i18n.t_args(
                "finding-structural-cause-component",
                &[("count", f.occurrence_count.to_string())],
            ))
        } else if f.occurrence_count >= 5 {
            Some(i18n.t_args(
                "finding-structural-cause-shared",
                &[("count", f.occurrence_count.to_string())],
            ))
        } else {
            None
        },
        is_component_issue: f.occurrence_count >= 10,
        criticality_tier: classify_criticality_tier(&f.category, &f.wcag_level),
        narrative,
    }
}

/// Recompute the fields that `finding_group_from_normalized` derives from
/// `occurrence_count` after a post-hoc title-based merge changes it (see
/// `mod.rs`'s dedup pass) — without this, a merged card shows the new summed
/// `occurrence_count` in its header while `structural_cause`/`affected_elements`
/// still reflect only the first source finding's original, smaller count,
/// producing a self-contradictory card (e.g. header "36 Vorkommen" next to a
/// "Root Cause" callout that says "22 Vorkommen"). Mirrors the exact threshold
/// logic in `finding_group_from_normalized` above.
///
/// `expected_impact`/`complexity_reason` (the raw English `String` fields) are
/// NOT rewritten here: the PDF layer (`pdf/findings.rs`) never reads them
/// directly — it always re-derives the displayed sentence from
/// `complexity_kind`/`expected_impact_kind` in the run language (#406). What
/// DOES need recomputing after a merge is the `occurrence_count` embedded in
/// those `..._kind` values, so the re-derived sentence reflects the merged
/// total instead of the pre-merge count.
pub(super) fn recompute_occurrence_derived_fields(group: &mut FindingGroup, i18n: &I18n) {
    group.affected_elements = group.occurrence_count;
    group.additional_occurrences = group
        .occurrence_count
        .saturating_sub(group.representative_occurrences.len());
    group.complexity_kind = group
        .complexity_kind
        .with_occurrence_count(group.occurrence_count);
    group.expected_impact_kind = group
        .expected_impact_kind
        .clone()
        .with_occurrence_count(group.occurrence_count);
    group.structural_cause = if group.occurrence_count >= 10 {
        Some(i18n.t_args(
            "finding-structural-cause-component",
            &[("count", group.occurrence_count.to_string())],
        ))
    } else if group.occurrence_count >= 5 {
        Some(i18n.t_args(
            "finding-structural-cause-shared",
            &[("count", group.occurrence_count.to_string())],
        ))
    } else {
        None
    };
    group.is_component_issue = group.occurrence_count >= 10;
}
