//! Action plan derivation helpers.

use std::collections::HashMap;

use crate::output::report_model::{
    ActionItem, ActionPlan, Effort, ExecutionPriority, FindingGroup, Priority, Role, RoleAssignment,
};
use crate::wcag::Severity;

#[inline]
fn is_en(locale: &str) -> bool {
    locale == "en"
}

pub(super) fn derive_action_plan(locale: &str, finding_groups: &[FindingGroup]) -> ActionPlan {
    let mut quick_wins = Vec::new();
    let mut medium_term = Vec::new();
    let mut structural = Vec::new();

    for group in finding_groups {
        let item = ActionItem {
            action: humanize_action_text(locale, &group.recommendation),
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
    let pm_extras: [&str; 3] = if is_en(locale) {
        [
            "Prioritize the action plan",
            "Quality assurance and testing",
            "Define responsibilities",
        ]
    } else {
        [
            "Priorisierung der Maßnahmen",
            "Qualitätssicherung und Testing",
            "Verantwortlichkeiten festlegen",
        ]
    };
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
    locale: &str,
    user_impact: &str,
    dimension: &str,
    severity: Severity,
    subcategory: Option<&str>,
) -> String {
    let en = is_en(locale);
    match dimension {
        "SEO" => {
            if en {
                "Can reduce search engine visibility and decrease organic traffic.".to_string()
            } else {
                "Kann Sichtbarkeit in Suchmaschinen reduzieren und organischen Traffic senken."
                    .to_string()
            }
        }
        "Security" => {
            if en {
                "Increases attack surface and risk of data loss.".to_string()
            } else {
                "Erhöht Angriffsfläche und Risiko für Datenverlust.".to_string()
            }
        }
        "Performance" => {
            if en {
                "Worsens load time and user experience, increases bounce rate.".to_string()
            } else {
                "Verschlechtert Ladezeit und Nutzererlebnis, erhöht Absprungrate.".to_string()
            }
        }
        "Mobile" => {
            if en {
                "Impairs mobile usability for the majority of users.".to_string()
            } else {
                "Beeinträchtigt mobile Nutzbarkeit für die Mehrheit der Nutzer.".to_string()
            }
        }
        "Accessibility" => {
            if subcategory == Some("Visuelle Darstellung")
                || user_impact.contains("Kontrast")
                || user_impact.contains("Lesbarkeit")
                || user_impact.contains("contrast")
                || user_impact.contains("readability")
            {
                if en {
                    "Impairs readability for users with visual impairment.".to_string()
                } else {
                    "Beeinträchtigt Lesbarkeit für Nutzer mit Sehschwäche.".to_string()
                }
            } else {
                match severity {
                    Severity::Critical | Severity::High => {
                        if en {
                            "Can exclude users and increase legal risk.".to_string()
                        } else {
                            "Kann Nutzer ausschließen und rechtliches Risiko erhöhen.".to_string()
                        }
                    }
                    _ if user_impact.contains("Sprachsteuerung")
                        || user_impact.contains("voice control") =>
                    {
                        if en {
                            "Can raise usage barriers and prevent interactions with key elements."
                                .to_string()
                        } else {
                            "Kann Nutzungshürden erhöhen und Interaktionen mit zentralen Elementen verhindern.".to_string()
                        }
                    }
                    _ => {
                        if en {
                            "Impairs site quality and user experience.".to_string()
                        } else {
                            "Beeinträchtigt Qualität und Nutzererlebnis der Website.".to_string()
                        }
                    }
                }
            }
        }
        _ => match severity {
            Severity::Critical | Severity::High => {
                if en {
                    "Can exclude users and increase legal risk.".to_string()
                } else {
                    "Kann Nutzer ausschließen und rechtliches Risiko erhöhen.".to_string()
                }
            }
            _ if user_impact.contains("Sprachsteuerung")
                || user_impact.contains("voice control") =>
            {
                if en {
                    "Can raise usage barriers and prevent interactions with key elements."
                        .to_string()
                } else {
                    "Kann Nutzungshürden erhöhen und Interaktionen mit zentralen Elementen verhindern.".to_string()
                }
            }
            _ => {
                if en {
                    "Impairs site quality and user experience.".to_string()
                } else {
                    "Beeinträchtigt Qualität und Nutzererlebnis der Website.".to_string()
                }
            }
        },
    }
}

pub(super) fn humanize_action_text(locale: &str, action: &str) -> String {
    let en = is_en(locale);
    let lower = action.to_lowercase();
    if lower.contains("aria-label") || lower.contains("aria_label") {
        return if en {
            "Name interactive elements (buttons, links) clearly".to_string()
        } else {
            "Interaktive Elemente (Buttons, Links) verständlich benennen".to_string()
        };
    }
    if (lower.contains("alt-text") || lower.contains("alt text") || lower.contains("alt-attribut"))
        && !lower.contains("kein")
    {
        return if en {
            "Add descriptive alternative text to images".to_string()
        } else {
            "Bilder mit beschreibendem Alternativtext versehen".to_string()
        };
    }
    if lower.contains("kontrast") || lower.contains("contrast") {
        return if en {
            "Improve color contrast for text and UI elements".to_string()
        } else {
            "Farbkontraste für Text und UI-Elemente verbessern".to_string()
        };
    }
    if (lower.contains("label") || lower.contains("beschriftung"))
        && (lower.contains("formular")
            || lower.contains("input")
            || lower.contains("feld")
            || lower.contains("form")
            || lower.contains("field"))
    {
        return if en {
            "Label form fields unambiguously".to_string()
        } else {
            "Formularfelder eindeutig beschriften".to_string()
        };
    }
    if lower.contains("überschrift") || (lower.contains("heading") && lower.contains("struktur")) {
        return if en {
            "Structure heading hierarchy logically".to_string()
        } else {
            "Überschriften-Hierarchie logisch strukturieren".to_string()
        };
    }
    if lower.contains("tastatur")
        || lower.contains("keyboard")
        || lower.contains("fokus-reihenfolge")
        || lower.contains("focus order")
    {
        return if en {
            "Ensure keyboard navigation and focus order".to_string()
        } else {
            "Tastaturnavigation und Fokus-Reihenfolge sicherstellen".to_string()
        };
    }
    if lower.contains("sprunglink") || lower.contains("skip link") || lower.contains("skip-link") {
        return if en {
            "Add skip links for screen-reader users".to_string()
        } else {
            "Sprunglinks für Screenreader-Nutzer ergänzen".to_string()
        };
    }
    if lower.contains("lang-attribut")
        || lower.contains("lang attribute")
        || (lower.contains("sprache") && lower.contains("attribut"))
        || (lower.contains("language") && lower.contains("attribute"))
    {
        return if en {
            "Mark page language correctly in HTML".to_string()
        } else {
            "Seitensprache korrekt im HTML auszeichnen".to_string()
        };
    }
    if lower.contains("seitentitel")
        || lower.contains("page title")
        || (lower.contains("title") && lower.contains("tag"))
    {
        return if en {
            "Provide a meaningful page title".to_string()
        } else {
            "Aussagekräftigen Seitentitel vergeben".to_string()
        };
    }
    if lower.contains("linktext")
        || (lower.contains("link") && (lower.contains("beschrift") || lower.contains("label")))
    {
        return if en {
            "Label links clearly and unambiguously".to_string()
        } else {
            "Links verständlich und eindeutig beschriften".to_string()
        };
    }
    if lower.contains("landmark") || (lower.contains("aria") && lower.contains("role")) {
        return if en {
            "Mark page structure with landmarks".to_string()
        } else {
            "Seitenstruktur mit Orientierungspunkten auszeichnen".to_string()
        };
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

pub(super) fn derive_user_effect_from_action(locale: &str, action: &str, effort: Effort) -> String {
    let en = is_en(locale);
    let a = action.to_lowercase();
    if a.contains("buttons")
        || a.contains("schaltflächen")
        || a.contains("interactive elements")
        || a.contains("interaktive elemente")
    {
        if en {
            "Users grasp controls instantly — fewer mis-clicks".to_string()
        } else {
            "Nutzer verstehen Schaltflächen sofort — weniger Fehlklicks".to_string()
        }
    } else if a.contains("links verständlich")
        || a.contains("links eindeutig")
        || a.contains("label links")
    {
        if en {
            "Clearer navigation — users find targets faster".to_string()
        } else {
            "Navigation klarer — Nutzer finden Ziele schneller".to_string()
        }
    } else if a.contains("aria-label") || a.contains("name interactive") {
        if en {
            "All controls clearly named — screen-reader use without guessing".to_string()
        } else {
            "Alle Bedienelemente klar benannt — Screenreader-Nutzung ohne Ratespiel".to_string()
        }
    } else if a.contains("bilder")
        || a.contains("alternativtext")
        || a.contains("alt-text")
        || a.contains("alternative text")
        || a.contains("images")
    {
        if en {
            "Images understandable for users without sight".to_string()
        } else {
            "Bilder verständlich für Nutzer ohne Sehvermögen".to_string()
        }
    } else if a.contains("kontrast") || a.contains("contrast") {
        if en {
            "Text readable for everyone — even in poor lighting".to_string()
        } else {
            "Text für alle Nutzer gut lesbar — auch bei schlechten Lichtverhältnissen".to_string()
        }
    } else if (a.contains("formular") || a.contains("form")) && a.contains("label") {
        if en {
            "Forms fillable without confusion — fewer drop-offs".to_string()
        } else {
            "Formulare ausfüllbar ohne Verwirrung — weniger Abbrüche".to_string()
        }
    } else if a.contains("überschrift") || a.contains("heading") {
        if en {
            "Content structure instantly graspable — faster orientation".to_string()
        } else {
            "Inhaltsstruktur sofort erfassbar — schnellere Orientierung".to_string()
        }
    } else if a.contains("sprunglink") || a.contains("skip") {
        if en {
            "Keyboard users jump directly to main content".to_string()
        } else {
            "Tastaturnutzer gelangen direkt zum Hauptinhalt".to_string()
        }
    } else if a.contains("tastatur")
        || a.contains("keyboard")
        || a.contains("fokus")
        || a.contains("focus")
    {
        if en {
            "Full operability without a mouse".to_string()
        } else {
            "Vollständige Bedienbarkeit ohne Maus".to_string()
        }
    } else if a.contains("sprache") || a.contains("lang-attribut") || a.contains("language") {
        if en {
            "Screen reader pronounces content in correct language and inflection".to_string()
        } else {
            "Screenreader liest Inhalte in korrekter Sprache und Betonung".to_string()
        }
    } else if a.contains("seitentitel") || a.contains("page title") || a.contains("title") {
        if en {
            "Page clearly identifiable in browser tab and search".to_string()
        } else {
            "Seite klar identifizierbar in Browser-Tab und Suche".to_string()
        }
    } else if a.contains("landmark") || a.contains("orientierungspunkt") {
        if en {
            "Screen-reader users navigate the page with structure".to_string()
        } else {
            "Screenreader-Nutzer navigieren strukturiert durch die Seite".to_string()
        }
    } else {
        match (effort, en) {
            (Effort::Quick, true) => {
                "Direct, noticeable improvement of user experience".to_string()
            }
            (Effort::Quick, false) => {
                "Direkte, spürbare Verbesserung der Nutzererfahrung".to_string()
            }
            (Effort::Medium, true) => "Noticeable improvement for affected user groups".to_string(),
            (Effort::Medium, false) => {
                "Merkliche Verbesserung für betroffene Nutzergruppen".to_string()
            }
            (Effort::Structural, true) => {
                "Long-term, more inclusive experience for everyone".to_string()
            }
            (Effort::Structural, false) => {
                "Langfristig inklusivere Nutzererfahrung für alle".to_string()
            }
        }
    }
}

pub(super) fn derive_conversion_effect_from_action(
    locale: &str,
    action: &str,
    effort: Effort,
) -> String {
    let en = is_en(locale);
    let a = action.to_lowercase();
    if a.contains("link") || a.contains("navigation") {
        if en {
            "Clearer navigation → fewer drop-offs".to_string()
        } else {
            "Klarere Navigation → weniger Absprünge".to_string()
        }
    } else if a.contains("kontrast") || a.contains("contrast") {
        if en {
            "Better readability → higher dwell time".to_string()
        } else {
            "Bessere Lesbarkeit → höhere Verweildauer".to_string()
        }
    } else if a.contains("heading") || a.contains("h1") || a.contains("überschrift") {
        if en {
            "Structural clarity → faster orientation".to_string()
        } else {
            "Strukturklarheit → schnellere Orientierung".to_string()
        }
    } else if a.contains("lang") || a.contains("language") {
        if en {
            "Correct speech output → no drop-offs from misreadings".to_string()
        } else {
            "Korrekte Sprachausgabe → keine Abbrüche durch Vorlesefehler".to_string()
        }
    } else {
        match (effort, en) {
            (Effort::Quick, true) => "Quick to take effect — measurable within days".to_string(),
            (Effort::Quick, false) => "Schnell wirksam — messbar innerhalb von Tagen".to_string(),
            (Effort::Medium, true) => "Medium-term measurable UX improvement".to_string(),
            (Effort::Medium, false) => "Mittelfristig messbare UX-Verbesserung".to_string(),
            (Effort::Structural, true) => "Solid technical baseline for future growth".to_string(),
            (Effort::Structural, false) => {
                "Solide technische Basis für weiteres Wachstum".to_string()
            }
        }
    }
}
