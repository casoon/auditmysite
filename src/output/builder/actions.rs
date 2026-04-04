//! Action plan derivation helpers.

use std::collections::HashMap;

use crate::output::report_model::{
    ActionItem, ActionPlan, Effort, ExecutionPriority, FindingGroup, Priority, Role,
    RoleAssignment,
};
use crate::wcag::Severity;

pub(super) fn derive_action_plan(finding_groups: &[FindingGroup]) -> ActionPlan {
    let mut quick_wins = Vec::new();
    let mut medium_term = Vec::new();
    let mut structural = Vec::new();

    for group in finding_groups {
        let item = ActionItem {
            action: humanize_action_text(&group.recommendation),
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

    quick_wins.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
    medium_term.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
    structural.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));

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
    role_map
        .entry(Role::ProjectManagement)
        .or_default()
        .extend([
            "Priorisierung der Maßnahmen".to_string(),
            "Qualitätssicherung und Testing".to_string(),
            "Verantwortlichkeiten festlegen".to_string(),
        ]);

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
    user_impact: &str,
    dimension: &str,
    severity: Severity,
    subcategory: Option<&str>,
) -> String {
    match dimension {
        "SEO" => "Kann Sichtbarkeit in Suchmaschinen reduzieren und organischen Traffic senken."
            .to_string(),
        "Security" => "Erhöht Angriffsfläche und Risiko für Datenverlust.".to_string(),
        "Performance" => {
            "Verschlechtert Ladezeit und Nutzererlebnis, erhöht Absprungrate.".to_string()
        }
        "Mobile" => "Beeinträchtigt mobile Nutzbarkeit für die Mehrheit der Nutzer.".to_string(),
        "Accessibility" => {
            if subcategory == Some("Visuelle Darstellung")
                || user_impact.contains("Kontrast")
                || user_impact.contains("Lesbarkeit")
            {
                "Beeinträchtigt Lesbarkeit für Nutzer mit Sehschwäche.".to_string()
            } else {
                match severity {
                    Severity::Critical | Severity::High => {
                        "Kann Nutzer ausschließen und rechtliches Risiko erhöhen.".to_string()
                    }
                    _ if user_impact.contains("Sprachsteuerung") => {
                        "Kann Nutzungshürden erhöhen und Interaktionen mit zentralen Elementen verhindern."
                            .to_string()
                    }
                    _ => "Beeinträchtigt Qualität und Nutzererlebnis der Website.".to_string(),
                }
            }
        }
        _ => match severity {
            Severity::Critical | Severity::High => {
                "Kann Nutzer ausschließen und rechtliches Risiko erhöhen.".to_string()
            }
            _ if user_impact.contains("Sprachsteuerung") => {
                "Kann Nutzungshürden erhöhen und Interaktionen mit zentralen Elementen verhindern."
                    .to_string()
            }
            _ => "Beeinträchtigt Qualität und Nutzererlebnis der Website.".to_string(),
        },
    }
}

pub(super) fn humanize_action_text(action: &str) -> String {
    let lower = action.to_lowercase();
    if lower.contains("aria-label") || lower.contains("aria_label") {
        return "Interaktive Elemente (Buttons, Links) verständlich benennen".to_string();
    }
    if (lower.contains("alt-text") || lower.contains("alt text") || lower.contains("alt-attribut"))
        && !lower.contains("kein")
    {
        return "Bilder mit beschreibendem Alternativtext versehen".to_string();
    }
    if lower.contains("kontrast") || lower.contains("contrast") {
        return "Farbkontraste für Text und UI-Elemente verbessern".to_string();
    }
    if (lower.contains("label") || lower.contains("beschriftung"))
        && (lower.contains("formular") || lower.contains("input") || lower.contains("feld"))
    {
        return "Formularfelder eindeutig beschriften".to_string();
    }
    if lower.contains("überschrift") || (lower.contains("heading") && lower.contains("struktur")) {
        return "Überschriften-Hierarchie logisch strukturieren".to_string();
    }
    if lower.contains("tastatur")
        || lower.contains("keyboard")
        || lower.contains("fokus-reihenfolge")
    {
        return "Tastaturnavigation und Fokus-Reihenfolge sicherstellen".to_string();
    }
    if lower.contains("sprunglink") || lower.contains("skip link") || lower.contains("skip-link") {
        return "Sprunglinks für Screenreader-Nutzer ergänzen".to_string();
    }
    if lower.contains("lang-attribut") || (lower.contains("sprache") && lower.contains("attribut"))
    {
        return "Seitensprache korrekt im HTML auszeichnen".to_string();
    }
    if lower.contains("seitentitel") || (lower.contains("title") && lower.contains("tag")) {
        return "Aussagekräftigen Seitentitel vergeben".to_string();
    }
    if lower.contains("linktext") || (lower.contains("link") && lower.contains("beschrift")) {
        return "Links verständlich und eindeutig beschriften".to_string();
    }
    if lower.contains("landmark") || (lower.contains("aria") && lower.contains("role")) {
        return "Seitenstruktur mit Orientierungspunkten auszeichnen".to_string();
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

pub(super) fn derive_user_effect_from_action(action: &str, effort: Effort) -> String {
    let a = action.to_lowercase();
    if a.contains("buttons") || a.contains("schaltflächen") {
        "Nutzer verstehen Schaltflächen sofort — weniger Fehlklicks".to_string()
    } else if a.contains("links verständlich") || a.contains("links eindeutig") {
        "Navigation klarer — Nutzer finden Ziele schneller".to_string()
    } else if a.contains("interaktive elemente") || a.contains("aria-label") {
        "Alle Bedienelemente klar benannt — Screenreader-Nutzung ohne Ratespiel".to_string()
    } else if a.contains("bilder") || a.contains("alternativtext") || a.contains("alt-text") {
        "Bilder verständlich für Nutzer ohne Sehvermögen".to_string()
    } else if a.contains("kontrast") || a.contains("farbkontrast") {
        "Text für alle Nutzer gut lesbar — auch bei schlechten Lichtverhältnissen".to_string()
    } else if a.contains("formular") && a.contains("beschrift") {
        "Formulare ausfüllbar ohne Verwirrung — weniger Abbrüche".to_string()
    } else if a.contains("überschrift") || a.contains("heading") {
        "Inhaltsstruktur sofort erfassbar — schnellere Orientierung".to_string()
    } else if a.contains("sprunglink") || a.contains("skip") {
        "Tastaturnutzer gelangen direkt zum Hauptinhalt".to_string()
    } else if a.contains("tastatur") || a.contains("keyboard") || a.contains("fokus") {
        "Vollständige Bedienbarkeit ohne Maus".to_string()
    } else if a.contains("sprache") || a.contains("lang-attribut") {
        "Screenreader liest Inhalte in korrekter Sprache und Betonung".to_string()
    } else if a.contains("seitentitel") || a.contains("title") {
        "Seite klar identifizierbar in Browser-Tab und Suche".to_string()
    } else if a.contains("landmark") || a.contains("orientierungspunkt") {
        "Screenreader-Nutzer navigieren strukturiert durch die Seite".to_string()
    } else {
        match effort {
            Effort::Quick => "Direkte, spürbare Verbesserung der Nutzererfahrung".to_string(),
            Effort::Medium => "Merkliche Verbesserung für betroffene Nutzergruppen".to_string(),
            Effort::Structural => "Langfristig inklusivere Nutzererfahrung für alle".to_string(),
        }
    }
}

pub(super) fn derive_conversion_effect_from_action(action: &str, effort: Effort) -> String {
    let action_lower = action.to_lowercase();
    if action_lower.contains("link") || action_lower.contains("navigation") {
        "Klarere Navigation → weniger Absprünge".to_string()
    } else if action_lower.contains("kontrast") || action_lower.contains("contrast") {
        "Bessere Lesbarkeit → höhere Verweildauer".to_string()
    } else if action_lower.contains("heading") || action_lower.contains("h1") {
        "Strukturklarheit → schnellere Orientierung".to_string()
    } else if action_lower.contains("lang") {
        "Korrekte Sprachausgabe → keine Abbrüche durch Vorlesefehler".to_string()
    } else {
        match effort {
            Effort::Quick => "Schnell wirksam — messbar innerhalb von Tagen".to_string(),
            Effort::Medium => "Mittelfristig messbare UX-Verbesserung".to_string(),
            Effort::Structural => "Solide technische Basis für weiteres Wachstum".to_string(),
        }
    }
}
