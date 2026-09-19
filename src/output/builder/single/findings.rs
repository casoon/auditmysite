use crate::audit::occurrence_analysis::{
    build_location_hints, build_pattern_clusters, build_representative_occurrences,
};
use crate::audit::prioritization::{derive_execution_priority, severity_to_priority};
use crate::i18n::I18n;
use crate::output::explanations::resolve_explanation;
use crate::output::report_model::{classify_criticality_tier, Effort, FindingGroup, Role};

use super::super::actions::{build_narrative_arc, derive_business_impact, localized_finding_text};

pub(super) fn finding_group_from_normalized(
    i18n: &I18n,
    f: &crate::audit::normalized::NormalizedFinding,
) -> FindingGroup {
    let locale = i18n.locale();
    // Shared axe_id -> rule_id -> wcag_criterion order; see
    // `explanations::resolve_explanation` for why it must not be reordered
    // (#571, #357, plan 32).
    let explanation = resolve_explanation(f.axe_id.as_deref(), &f.rule_id, &f.wcag_criterion);

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
        // the runtime-locale text from the taxonomy for the report. There is no
        // dedicated "customer description" field in the taxonomy for rules
        // without a `RuleExplanation` — `technical_impact` is the closest
        // available substitute that is actually localized DE/EN (`description`
        // has no English counterpart). Using `f.description` here (canonical
        // English, #406) would leak raw English prose into German reports —
        // never do that (Rule C, plain-language report plan).
        let (title_loc, user_impact_loc, technical_loc) = localized_finding_text(locale, f);
        (
            title_loc,
            technical_loc,
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

#[cfg(test)]
mod fallback_tests {
    use super::finding_group_from_normalized;
    use crate::audit::normalized::{
        ComplexityKind, ExpectedImpactKind, NormalizedFinding, ReportVisibilityData, ScoreEffect,
        ScoreImpactData,
    };
    use crate::i18n::I18n;
    use crate::wcag::Severity;

    /// `a11y.timing.unadjustable` (WCAG 2.2.1) has no `RuleExplanation` entry
    /// in `explanations.rs` — a real, currently-uncovered rule (verified by
    /// auditing `EXPLANATIONS` against the taxonomy), so this exercises the
    /// `finding_group_from_normalized` fallback branch. `description` is set
    /// to a raw-English marker that must never surface as `customer_description`.
    fn uncovered_rule_finding() -> NormalizedFinding {
        NormalizedFinding {
            category: "wcag".into(),
            rule_id: "a11y.timing.unadjustable".into(),
            wcag_criterion: "2.2.1".into(),
            axe_id: None,
            wcag_level: "A".into(),
            dimension: "Accessibility".into(),
            subcategory: "Navigation Interaction".into(),
            issue_class: "Weak".into(),
            dimension_kind: crate::taxonomy::Dimension::Accessibility,
            subcategory_kind: crate::taxonomy::Subcategory::NavigationInteraction,
            issue_class_kind: crate::taxonomy::IssueClass::Weak,
            severity: Severity::High,
            user_impact: "Users need more time to enter data and may lose data through timeouts."
                .into(),
            technical_impact: "Missing refresh or timeout handling in the script/markup.".into(),
            score_impact: ScoreImpactData {
                base_penalty: 3.0,
                max_penalty: 10.0,
                scaling: "Logarithmic".into(),
            },
            report_visibility: ReportVisibilityData::default(),
            aggregation_key: "a11y.timing.unadjustable".into(),
            title: "Non-adjustable time limit".into(),
            description: "RAW ENGLISH CANARY — must never reach a German customer_description"
                .into(),
            help_url: None,
            occurrence_count: 1,
            priority_score: 1.0,
            confidence: "high".into(),
            false_positive_risk: "low".into(),
            verification: "automatically_confirmed".into(),
            complexity: "medium".into(),
            complexity_reason: "Test fixture".into(),
            complexity_kind: ComplexityKind::LowScope,
            expected_impact: "Test fixture".into(),
            expected_impact_kind: ExpectedImpactKind::Other {
                occurrence_count: 1,
                score_effect: ScoreEffect::Low,
            },
            bfsg_relevance: "medium".into(),
            remediation_priority: "normal".into(),
            occurrences: vec![],
        }
    }

    /// Rule C (plain-language report plan): the no-`RuleExplanation` fallback
    /// must source `customer_description` from the localized taxonomy text,
    /// never from the raw canonical-English `NormalizedFinding.description`.
    #[test]
    fn fallback_customer_description_is_localized_not_raw_english() {
        let i18n = I18n::new("de").expect("test locale should load");
        let finding = uncovered_rule_finding();
        let group = finding_group_from_normalized(&i18n, &finding);

        assert_ne!(group.customer_description, finding.description);
        assert!(!group.customer_description.is_empty());
        // The localized taxonomy `technical_impact` for this rule (German).
        assert_eq!(
            group.customer_description,
            "Fehlendes Refresh- oder Timeout-Handling im Script/Markup."
        );
    }

    /// A `region.rs` finding (WCAG 1.3.1, taxonomy id "a11y.structure.missing",
    /// axe_id "region"). Several unrelated 1.3.1 checks (tables, lists, form
    /// groups) share that same taxonomy id, so `rule_id` alone would resolve
    /// to a generic table/list/fieldset explanation — only the `axe_id` is
    /// specific enough to identify this as a landmark/region finding (#571).
    fn region_finding() -> NormalizedFinding {
        NormalizedFinding {
            rule_id: "a11y.structure.missing".into(),
            wcag_criterion: "1.3.1".into(),
            axe_id: Some("region".into()),
            title: "Region".into(),
            description: "Element with role 'link' is not contained within a landmark region"
                .into(),
            ..uncovered_rule_finding()
        }
    }

    /// Regression for #571 (example 1): `finding_group_from_normalized` must
    /// use the `axe_id`-specific "region" explanation for a region.rs
    /// finding, not the generic WCAG-1.3.1 fallback (which is written for
    /// tables/lists/fieldsets and would show a `<table>` code example for a
    /// finding about a link outside a landmark).
    #[test]
    fn region_finding_gets_landmark_example_not_table_fallback() {
        let i18n = I18n::new("de").expect("test locale should load");
        let finding = region_finding();
        let group = finding_group_from_normalized(&i18n, &finding);

        assert!(
            group.recommendation.contains("Landmark"),
            "expected the landmark-specific recommendation, got: {}",
            group.recommendation
        );
        assert!(
            !group.examples.iter().any(|ex| ex.bad.contains("<table>")),
            "region finding must not show the generic table example"
        );
    }

    /// A `landmark_granular.rs` `check_landmark_unique` finding (WCAG 1.3.1,
    /// taxonomy id "a11y.landmark_unique.invalid", axe_id "landmark-unique").
    /// This taxonomy id's `external_ref` is "WCAG 1.3.1", so
    /// `get_explanation(rule_id)`'s internal WCAG-id fallback would otherwise
    /// silently resolve to the generic 1.3.1 explanation written for
    /// tables/lists/forms — same bug class as #571 (region.rs), confirmed
    /// live against a real report (plan/1-root-cause-title-occurrence-mismatch.md).
    fn landmark_unique_finding() -> NormalizedFinding {
        NormalizedFinding {
            rule_id: "a11y.landmark_unique.invalid".into(),
            wcag_criterion: "1.3.1".into(),
            axe_id: Some("landmark-unique".into()),
            title: "Landmarks are not uniquely named".into(),
            description: "Multiple landmarks of the same role have the same accessible name."
                .into(),
            occurrence_count: 31,
            ..uncovered_rule_finding()
        }
    }

    /// Regression for plan/1: a `landmark-unique` finding must get its own
    /// title ("Landmarks nicht eindeutig benannt"), not the generic WCAG 1.3.1
    /// fallback title ("Fehlende semantische Struktur") that belongs to a
    /// different, unrelated check (tables/lists/forms structure).
    #[test]
    fn landmark_unique_finding_gets_own_title_not_generic_1_3_1_fallback() {
        let i18n = I18n::new("de").expect("test locale should load");
        let finding = landmark_unique_finding();
        let group = finding_group_from_normalized(&i18n, &finding);

        assert_eq!(group.title, "Landmarks nicht eindeutig benannt");
        assert_ne!(group.title, "Fehlende semantische Struktur");
        // Title and occurrence_count must come from the same finding, not an
        // accidental pairing across two different findings.
        assert_eq!(group.occurrence_count, 31);
    }

    /// Same fallback branch in English: no German umlauts/ß should leak in
    /// (mirrors the #406 guard-test pattern used elsewhere in the codebase).
    #[test]
    fn fallback_customer_description_en_has_no_german_umlauts() {
        let i18n = I18n::new("en").expect("test locale should load");
        let finding = uncovered_rule_finding();
        let group = finding_group_from_normalized(&i18n, &finding);

        assert!(!group.customer_description.is_empty());
        assert!(!group
            .customer_description
            .contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
    }
}
