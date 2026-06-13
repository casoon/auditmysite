//! Action plan derivation helpers.

use std::collections::HashMap;

use crate::output::report_model::{
    ActionItem, ActionPlan, Effort, ExecutionPriority, FindingGroup, NarrativeArc, Priority, Role,
    RoleAssignment,
};
use crate::wcag::Severity;

use crate::i18n::I18n;

/// Re-derive the locale-specific title / user impact / technical impact for a
/// finding. The stored `NormalizedFinding` fields are canonical English (#406);
/// for non-English locales we look the rule up in the taxonomy and return the
/// German strings. Falls back to the (English) stored values when no taxonomy
/// entry matches. Used only by the fallback branch — when an explanation exists
/// the localized text comes from the explanation system instead.
pub(super) fn localized_finding_text(
    locale: &str,
    finding: &crate::audit::normalized::NormalizedFinding,
) -> (String, String, String) {
    if locale == "en" {
        return (
            finding.title.clone(),
            finding.user_impact.clone(),
            finding.technical_impact.clone(),
        );
    }
    // SEO heading findings are not in the WCAG taxonomy; re-derive German from
    // their own single-source text function (#406).
    if let Some(issue_type) = finding.rule_id.strip_prefix("seo.headings.") {
        let (title, technical_impact) = crate::audit::normalized::seo_heading_finding_text(
            issue_type,
            false,
            &finding.technical_impact,
        );
        return (title, finding.user_impact.clone(), technical_impact);
    }
    use crate::taxonomy::RuleLookup;
    let rule = RuleLookup::by_id(&finding.rule_id)
        .or_else(|| RuleLookup::by_wcag(&finding.wcag_criterion))
        .or_else(|| RuleLookup::by_legacy_wcag_id(&finding.wcag_criterion));
    match rule {
        Some(r) => (
            r.title.to_string(),
            r.user_impact.to_string(),
            r.technical_impact.to_string(),
        ),
        None => (
            finding.title.clone(),
            finding.user_impact.clone(),
            finding.technical_impact.clone(),
        ),
    }
}

pub(super) fn derive_action_plan(i18n: &I18n, finding_groups: &[FindingGroup]) -> ActionPlan {
    let mut quick_wins = Vec::new();
    let mut medium_term = Vec::new();
    let mut structural = Vec::new();

    for group in finding_groups {
        let item = ActionItem {
            action: humanize_action_text(i18n, &group.recommendation),
            benefit: group.business_impact.clone(),
            role: group.responsible_role,
            priority: group.priority,
            execution_priority: group.execution_priority,
            effort: group.effort,
        };
        match group.effort {
            Effort::Quick => quick_wins.push(item),
            Effort::Medium => medium_term.push(item),
            Effort::Structural => structural.push(item),
        }
    }

    quick_wins.sort_by_key(|b| std::cmp::Reverse(b.execution_priority));
    medium_term.sort_by_key(|b| std::cmp::Reverse(b.execution_priority));
    structural.sort_by_key(|b| std::cmp::Reverse(b.execution_priority));

    // Deduplicate by action text across ALL phases (keep first occurrence = highest phase/priority)
    let mut seen_actions: std::collections::HashSet<String> = std::collections::HashSet::new();
    let dedup =
        |items: Vec<ActionItem>, seen: &mut std::collections::HashSet<String>| -> Vec<ActionItem> {
            items
                .into_iter()
                .filter(|i| seen.insert(i.action.clone()))
                .collect()
        };
    let quick_wins = dedup(quick_wins, &mut seen_actions);
    let medium_term = dedup(medium_term, &mut seen_actions);
    let structural = dedup(structural, &mut seen_actions);

    let mut role_map: HashMap<Role, Vec<String>> = HashMap::new();
    for group in finding_groups {
        role_map
            .entry(group.responsible_role)
            .or_default()
            .push(group.title.clone());
    }
    let pm_extras = [
        i18n.t("pm-extra-prioritize"),
        i18n.t("pm-extra-qa"),
        i18n.t("pm-extra-responsibilities"),
    ];
    role_map
        .entry(Role::ProjectManagement)
        .or_default()
        .extend(pm_extras.iter().map(|s| s.to_string()));

    let role_assignments: Vec<RoleAssignment> = role_map
        .into_iter()
        .map(|(role, mut responsibilities)| {
            responsibilities.dedup();
            RoleAssignment {
                role,
                responsibilities,
            }
        })
        .collect();

    ActionPlan {
        quick_wins,
        medium_term,
        structural,
        role_assignments,
    }
}

pub(super) fn derive_execution_priority(
    severity: Severity,
    effort: Effort,
    dimension: &str,
) -> ExecutionPriority {
    match (severity, effort, dimension) {
        (Severity::Critical, _, _) => ExecutionPriority::Immediate,
        (Severity::High, _, "Accessibility") => ExecutionPriority::Immediate,
        (Severity::High, Effort::Quick, _) => ExecutionPriority::Important,
        (Severity::High, _, _) => ExecutionPriority::Important,
        (Severity::Medium, Effort::Quick, _) => ExecutionPriority::Important,
        _ => ExecutionPriority::Optional,
    }
}

pub(super) fn derive_business_impact(
    i18n: &I18n,
    user_impact: &str,
    dimension: &str,
    severity: Severity,
    subcategory: Option<&str>,
    occurrence_count: usize,
) -> String {
    let base = derive_business_impact_base(i18n, user_impact, dimension, severity, subcategory);
    let prefix = match occurrence_count {
        n if n >= 20 => i18n.t("impact-prefix-widespread"),
        n if n >= 5 => i18n.t("impact-prefix-frequent"),
        _ => "".to_string(),
    };
    if prefix.is_empty() {
        base
    } else {
        format!("{}{}", prefix, base)
    }
}

fn derive_business_impact_base(
    i18n: &I18n,
    user_impact: &str,
    dimension: &str,
    severity: Severity,
    subcategory: Option<&str>,
) -> String {
    match dimension {
        "SEO" => i18n.t("impact-base-seo"),
        "Security" => i18n.t("impact-base-security"),
        "Performance" => i18n.t("impact-base-performance"),
        "Mobile" => i18n.t("impact-base-mobile"),
        "Accessibility" => {
            if subcategory == Some("Visuelle Darstellung")
                || user_impact.contains("Kontrast")
                || user_impact.contains("Lesbarkeit")
                || user_impact.contains("contrast")
                || user_impact.contains("readability")
            {
                i18n.t("impact-base-accessibility-readability")
            } else {
                match severity {
                    Severity::Critical | Severity::High => {
                        i18n.t("impact-base-accessibility-exclude")
                    }
                    _ if user_impact.contains("Sprachsteuerung")
                        || user_impact.contains("voice control") =>
                    {
                        i18n.t("impact-base-accessibility-voice")
                    }
                    _ => i18n.t("impact-base-accessibility-default"),
                }
            }
        }
        _ => match severity {
            Severity::Critical | Severity::High => i18n.t("impact-base-accessibility-exclude"),
            _ if user_impact.contains("Sprachsteuerung")
                || user_impact.contains("voice control") =>
            {
                i18n.t("impact-base-accessibility-voice")
            }
            _ => i18n.t("impact-base-accessibility-default"),
        },
    }
}

pub(super) fn humanize_action_text(i18n: &I18n, action: &str) -> String {
    let lower = action.to_lowercase();
    if lower.contains("aria-label") || lower.contains("aria_label") {
        return i18n.t("action-human-aria-label");
    }
    if (lower.contains("alt-text") || lower.contains("alt text") || lower.contains("alt-attribut"))
        && !lower.contains("kein")
    {
        return i18n.t("action-human-alt-text");
    }
    if lower.contains("kontrast") || lower.contains("contrast") {
        return i18n.t("action-human-contrast");
    }
    if (lower.contains("label") || lower.contains("beschriftung"))
        && (lower.contains("formular")
            || lower.contains("input")
            || lower.contains("feld")
            || lower.contains("form")
            || lower.contains("field"))
    {
        return i18n.t("action-human-form-label");
    }
    if lower.contains("überschrift") || (lower.contains("heading") && lower.contains("struktur")) {
        return i18n.t("action-human-heading");
    }
    if lower.contains("tastatur")
        || lower.contains("keyboard")
        || lower.contains("fokus-reihenfolge")
        || lower.contains("focus order")
    {
        return i18n.t("action-human-keyboard");
    }
    if lower.contains("sprunglink") || lower.contains("skip link") || lower.contains("skip-link") {
        return i18n.t("action-human-skip-link");
    }
    if lower.contains("lang-attribut")
        || lower.contains("lang attribute")
        || (lower.contains("sprache") && lower.contains("attribut"))
        || (lower.contains("language") && lower.contains("attribute"))
    {
        return i18n.t("action-human-lang-attr");
    }
    if lower.contains("seitentitel")
        || lower.contains("page title")
        || (lower.contains("title") && lower.contains("tag"))
    {
        return i18n.t("action-human-page-title");
    }
    if lower.contains("linktext")
        || (lower.contains("link") && (lower.contains("beschrift") || lower.contains("label")))
    {
        return i18n.t("action-human-link-text");
    }
    if lower.contains("landmark") || (lower.contains("aria") && lower.contains("role")) {
        return i18n.t("action-human-landmark");
    }
    action.to_string()
}

pub(super) fn severity_to_priority(severity: Severity) -> Priority {
    match severity {
        Severity::Critical => Priority::Critical,
        Severity::High => Priority::High,
        Severity::Medium => Priority::Medium,
        Severity::Low => Priority::Low,
    }
}

pub(super) fn score_to_priority(score: f32) -> Priority {
    if score < 50.0 {
        Priority::Critical
    } else if score < 70.0 {
        Priority::High
    } else if score < 85.0 {
        Priority::Medium
    } else {
        Priority::Low
    }
}

pub(super) fn impact_score(group: &FindingGroup) -> u32 {
    let severity_weight = match group.severity {
        Severity::Critical => 4,
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
    };
    severity_weight * group.occurrence_count as u32
}

pub(super) fn derive_user_effect_from_action(i18n: &I18n, action: &str, effort: Effort) -> String {
    let a = action.to_lowercase();
    if a.contains("buttons")
        || a.contains("schaltflächen")
        || a.contains("interactive elements")
        || a.contains("interaktive elemente")
    {
        i18n.t("effect-user-buttons")
    } else if a.contains("links verständlich")
        || a.contains("links eindeutig")
        || a.contains("label links")
    {
        i18n.t("effect-user-links")
    } else if a.contains("aria-label") || a.contains("name interactive") {
        i18n.t("effect-user-aria")
    } else if a.contains("bilder")
        || a.contains("alternativtext")
        || a.contains("alt-text")
        || a.contains("alternative text")
        || a.contains("images")
    {
        i18n.t("effect-user-images")
    } else if a.contains("kontrast") || a.contains("contrast") {
        i18n.t("effect-user-contrast")
    } else if (a.contains("formular") || a.contains("form")) && a.contains("label") {
        i18n.t("effect-user-forms")
    } else if a.contains("überschrift") || a.contains("heading") {
        i18n.t("effect-user-heading")
    } else if a.contains("sprunglink") || a.contains("skip") {
        i18n.t("effect-user-skip")
    } else if a.contains("tastatur")
        || a.contains("keyboard")
        || a.contains("fokus")
        || a.contains("focus")
    {
        i18n.t("effect-user-keyboard")
    } else if a.contains("sprache") || a.contains("lang-attribut") || a.contains("language") {
        i18n.t("effect-user-language")
    } else if a.contains("seitentitel") || a.contains("page title") || a.contains("title") {
        i18n.t("effect-user-title")
    } else if a.contains("landmark") || a.contains("orientierungspunkt") {
        i18n.t("effect-user-landmark")
    } else {
        match effort {
            Effort::Quick => i18n.t("effect-user-default-quick"),
            Effort::Medium => i18n.t("effect-user-default-medium"),
            Effort::Structural => i18n.t("effect-user-default-structural"),
        }
    }
}

pub(super) fn derive_conversion_effect_from_action(
    i18n: &I18n,
    action: &str,
    effort: Effort,
) -> String {
    let a = action.to_lowercase();
    if a.contains("link") || a.contains("navigation") {
        i18n.t("effect-conversion-links")
    } else if a.contains("kontrast") || a.contains("contrast") {
        i18n.t("effect-conversion-contrast")
    } else if a.contains("heading") || a.contains("h1") || a.contains("überschrift") {
        i18n.t("effect-conversion-heading")
    } else if a.contains("lang") || a.contains("language") {
        i18n.t("effect-conversion-language")
    } else {
        match effort {
            Effort::Quick => i18n.t("effect-conversion-default-quick"),
            Effort::Medium => i18n.t("effect-conversion-default-medium"),
            Effort::Structural => i18n.t("effect-conversion-default-structural"),
        }
    }
}

// ─── Narrative Arc ──────────────────────────────────────────────────────────

/// Build a four-stage narrative arc (Diagnose → Ursache → Wirkung → Umsetzung)
/// for a finding. Called by both single and batch report builders.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_narrative_arc(
    i18n: &I18n,
    occurrence_count: usize,
    severity: Severity,
    dimension: &str,
    customer_description: &str,
    user_impact: &str,
    business_impact: &str,
    typical_cause: &str,
    recommendation: &str,
    effort: Effort,
    role: Role,
) -> NarrativeArc {
    // Diagnose: occurrence-enriched version of the customer description
    let diagnose = if occurrence_count > 1 {
        let desc = narrative_first_sentence(customer_description);
        i18n.t_args(
            "narrative-diagnose-multiple",
            &[
                ("count", occurrence_count.to_string()),
                ("desc", desc.to_string()),
            ],
        )
    } else {
        customer_description.to_string()
    };

    // Ursache: typical_cause or dimension-based fallback
    let ursache = if !typical_cause.is_empty() {
        typical_cause.to_string()
    } else {
        narrative_cause_fallback(i18n, dimension, severity)
    };

    // Wirkung: user_impact + business_impact combined into one statement
    let wirkung = match (
        narrative_first_sentence(user_impact),
        narrative_first_sentence(business_impact),
    ) {
        ("", "") => narrative_impact_fallback(i18n, severity),
        (u, "") => u.to_string(),
        ("", b) => b.to_string(),
        (u, b) => format!("{u} {b}"),
    };

    // Umsetzung: recommendation enriched with effort + role context
    let effort_context = match effort {
        Effort::Quick => i18n.t("narrative-effort-quick"),
        Effort::Medium => i18n.t("narrative-effort-medium"),
        Effort::Structural => i18n.t("narrative-effort-structural"),
    };
    let role_context = match role {
        Role::Development => i18n.t("narrative-owner-dev"),
        Role::Editorial => i18n.t("narrative-owner-editorial"),
        Role::DesignUx => i18n.t("narrative-owner-designux"),
        Role::ProjectManagement => i18n.t("narrative-owner-pm"),
    };
    let umsetzung = if !recommendation.is_empty() {
        i18n.t_args(
            "narrative-implementation-format",
            &[
                ("recommendation", recommendation.to_string()),
                ("effort", effort_context),
                ("owner", role_context),
            ],
        )
    } else {
        i18n.t_args(
            "narrative-implementation-empty",
            &[("effort", effort_context), ("owner", role_context)],
        )
    };

    NarrativeArc {
        diagnose,
        ursache,
        wirkung,
        umsetzung,
    }
}

/// Extract the first sentence (skips common abbreviations like "z. B.").
pub(super) fn narrative_first_sentence(text: &str) -> &str {
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(". ") {
        let pos = search_from + rel;
        if pos >= 2 {
            let bytes = text.as_bytes();
            let b0 = bytes[pos - 2];
            let b1 = bytes[pos - 1];
            if b0 == b' ' && b1.is_ascii_alphabetic() {
                search_from = pos + 2;
                continue;
            }
        }
        return &text[..pos + 1];
    }
    text
}

fn narrative_cause_fallback(i18n: &I18n, dimension: &str, severity: Severity) -> String {
    let severity_note = match severity {
        Severity::Critical | Severity::High => i18n.t("narrative-cause-suffix-high"),
        _ => "".to_string(),
    };
    let base = match dimension {
        "Accessibility" => i18n.t("narrative-cause-accessibility"),
        "SEO" => i18n.t("narrative-cause-seo"),
        "Performance" => i18n.t("narrative-cause-performance"),
        "Security" => i18n.t("narrative-cause-security"),
        "Mobile" => i18n.t("narrative-cause-mobile"),
        _ => i18n.t("narrative-cause-default"),
    };
    format!("{base}{severity_note}")
}

fn narrative_impact_fallback(i18n: &I18n, severity: Severity) -> String {
    match severity {
        Severity::Critical => i18n.t("narrative-impact-critical"),
        Severity::High => i18n.t("narrative-impact-high"),
        Severity::Medium => i18n.t("narrative-impact-medium"),
        Severity::Low => i18n.t("narrative-impact-low"),
    }
}
