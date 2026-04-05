//! Journey Analysis — user-flow evaluation from AXTree data
//!
//! Evaluates how well a page supports a typical user journey:
//! Entry → Orientation → Navigation → Interaction → Conversion.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::accessibility::AXTree;
use crate::journey::page_intent::{detect_page_intent, PageIntent};
use crate::journey::scoring::{journey_dimension_score, weighted_average_with_intent};
use crate::ux::saturating_penalty;

// ── Generic / CTA keyword lists (shared with UX but scoped here) ───

const GENERIC_LINK_LABELS: &[&str] = &[
    "mehr",
    "hier",
    "klicken",
    "weiter",
    "link",
    "more",
    "click here",
    "read more",
    "learn more",
    "hier klicken",
    "mehr erfahren",
    "details",
    "weiterlesen",
    "info",
];

const CTA_KEYWORDS: &[&str] = &[
    "kaufen",
    "bestellen",
    "kontakt",
    "anfrage",
    "starten",
    "registrieren",
    "anmelden",
    "buchen",
    "jetzt",
    "kostenlos",
    "testen",
    "demo",
    "termin",
    "beratung",
    "angebot",
    "download",
    "newsletter",
    "warenkorb",
    "kasse",
    "buy",
    "order",
    "contact",
    "start",
    "register",
    "sign up",
    "book",
    "free",
    "trial",
    "get started",
    "subscribe",
    "cart",
    "checkout",
];

// ── Public types ────────────────────────────────────────────────────

/// Complete journey analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyAnalysis {
    /// Overall journey score (0–100)
    pub score: u32,
    /// Grade (A–F)
    pub grade: String,
    /// Detected page intent
    pub page_intent: PageIntent,
    /// Per-dimension results
    pub entry_clarity: JourneyDimension,
    pub orientation: JourneyDimension,
    pub navigation: JourneyDimension,
    pub interaction: JourneyDimension,
    pub conversion: JourneyDimension,
    /// Friction points found along the journey
    pub friction_points: Vec<FrictionPoint>,
}

/// A scored journey dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyDimension {
    pub name: String,
    pub score: u32,
    pub weight: f64,
    pub summary: String,
}

/// A point of friction in the user journey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionPoint {
    pub step: String,
    pub severity: String,
    pub problem: String,
    pub impact: String,
    pub recommendation: String,
}

// ── Analysis entry point ────────────────────────────────────────────

/// Analyze user journey quality from the Accessibility Tree.
/// Runs purely on already-extracted AXTree data — no CDP calls needed.
pub fn analyze_journey(tree: &AXTree) -> JourneyAnalysis {
    info!("Analyzing user journey...");

    let page_intent = detect_page_intent(tree);
    let (w_entry, w_orient, w_nav, w_interact, w_convert) = page_intent.weights();

    let mut friction_points = Vec::new();

    let entry_clarity = analyze_entry_clarity(tree, &mut friction_points);
    let orientation = analyze_orientation(tree, &mut friction_points);
    let navigation = analyze_navigation(tree, &mut friction_points);
    let interaction = analyze_interaction(tree, &mut friction_points);
    let conversion = analyze_conversion(tree, &mut friction_points);

    let score = weighted_average_with_intent(&[
        (entry_clarity.score, w_entry),
        (orientation.score, w_orient),
        (navigation.score, w_nav),
        (interaction.score, w_interact),
        (conversion.score, w_convert),
    ]);

    let grade = match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
    .to_string();

    info!(
        "Journey analysis: score={}, intent={:?}, friction_points={}",
        score,
        page_intent,
        friction_points.len()
    );

    // Update dimensions with intent-adjusted weights for display
    let mut entry_clarity = entry_clarity;
    entry_clarity.weight = w_entry;
    let mut orientation = orientation;
    orientation.weight = w_orient;
    let mut navigation = navigation;
    navigation.weight = w_nav;
    let mut interaction = interaction;
    interaction.weight = w_interact;
    let mut conversion = conversion;
    conversion.weight = w_convert;

    JourneyAnalysis {
        score,
        grade,
        page_intent,
        entry_clarity,
        orientation,
        navigation,
        interaction,
        conversion,
        friction_points,
    }
}

// ── Dimension analyzers ─────────────────────────────────────────────

/// Entry Clarity: Is the purpose of this page immediately clear?
fn analyze_entry_clarity(tree: &AXTree, friction: &mut Vec<FrictionPoint>) -> JourneyDimension {
    let headings = tree.headings();
    let mut penalties = Vec::new();

    // H1 presence and clarity
    let h1s: Vec<_> = headings
        .iter()
        .filter(|h| h.heading_level() == Some(1))
        .collect();

    if h1s.is_empty() {
        penalties.push(45.0);
        friction.push(FrictionPoint {
            step: "Entry".into(),
            severity: "high".into(),
            problem: "Keine H1-Überschrift — Seitenzweck nicht sofort erkennbar".into(),
            impact: "Nutzer verstehen nicht, worum es auf dieser Seite geht".into(),
            recommendation: "Eine aussagekräftige H1-Überschrift setzen".into(),
        });
    } else if h1s.len() > 1 {
        penalties.push(15.0);
        friction.push(FrictionPoint {
            step: "Entry".into(),
            severity: "medium".into(),
            problem: format!("{} H1-Überschriften — unklarer Seitenfokus", h1s.len()),
            impact: "Nutzer wissen nicht, was der Hauptinhalt ist".into(),
            recommendation: "Genau eine H1-Überschrift pro Seite verwenden".into(),
        });
    } else {
        // Check H1 quality: very short or generic?
        if let Some(name) = h1s[0].name.as_deref() {
            if name.len() < 5 {
                penalties.push(20.0);
            }
        }
    }

    // Page title (check for title-like node in tree)
    let has_title = tree
        .iter()
        .any(|n| n.role.as_deref() == Some("RootWebArea") && n.name.is_some());
    if !has_title {
        penalties.push(20.0);
        friction.push(FrictionPoint {
            step: "Entry".into(),
            severity: "medium".into(),
            problem: "Kein Seitentitel erkannt".into(),
            impact: "Tab-Titel ist leer — Nutzer verlieren den Kontext beim Tabwechsel".into(),
            recommendation: "Aussagekräftigen <title> setzen".into(),
        });
    }

    // Early content: check if there's substantial text in the first portion of the tree
    let early_text_len: usize = tree
        .iter()
        .take(50)
        .filter(|n| matches!(n.role.as_deref(), Some("StaticText" | "heading")))
        .filter_map(|n| n.name.as_ref())
        .map(|n| n.len())
        .sum();

    if early_text_len < 30 {
        penalties.push(20.0);
        friction.push(FrictionPoint {
            step: "Entry".into(),
            severity: "medium".into(),
            problem: "Wenig sichtbarer Text im oberen Seitenbereich".into(),
            impact: "Nutzer erhalten keine sofortige Orientierung".into(),
            recommendation: "Relevanten Einleitungstext above-the-fold platzieren".into(),
        });
    }

    let score = journey_dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Seitenzweck ist sofort klar erkennbar".into()
    } else if score >= 60 {
        "Einstieg grundsätzlich verständlich, aber verbesserungswürdig".into()
    } else {
        "Seitenzweck ist nicht auf den ersten Blick erkennbar".into()
    };

    JourneyDimension {
        name: "Entry Clarity".into(),
        score,
        weight: 0.20,
        summary,
    }
}

/// Orientation: Can the user tell where they are and where they can go?
fn analyze_orientation(tree: &AXTree, friction: &mut Vec<FrictionPoint>) -> JourneyDimension {
    let mut penalties = Vec::new();

    // Navigation landmark
    let has_nav = tree.iter().any(|n| n.role.as_deref() == Some("navigation"));
    if !has_nav {
        penalties.push(40.0);
        friction.push(FrictionPoint {
            step: "Orientation".into(),
            severity: "high".into(),
            problem: "Kein Navigationsbereich (<nav>) erkannt".into(),
            impact: "Nutzer können sich nicht orientieren und finden keine Hauptnavigation".into(),
            recommendation: "Hauptnavigation in ein <nav>-Element einschließen".into(),
        });
    }

    // Breadcrumbs or secondary nav hint
    let has_breadcrumb = tree.iter().any(|n| {
        let name = n.name.as_deref().unwrap_or("").to_lowercase();
        name.contains("breadcrumb")
            || n.role.as_deref() == Some("navigation") && name.contains("breadcrumb")
    });
    // Not penalized heavily, but bonus signals
    if !has_breadcrumb {
        penalties.push(10.0);
    }

    // Main landmark
    let has_main = tree.iter().any(|n| n.role.as_deref() == Some("main"));
    if !has_main {
        penalties.push(20.0);
        friction.push(FrictionPoint {
            step: "Orientation".into(),
            severity: "medium".into(),
            problem: "Kein Hauptinhaltsbereich (<main>) erkannt".into(),
            impact: "Screenreader-Nutzer können den Hauptinhalt nicht direkt anspringen".into(),
            recommendation: "Hauptinhalt in ein <main>-Element einschließen".into(),
        });
    }

    // Footer / complementary landmark
    let has_footer = tree
        .iter()
        .any(|n| n.role.as_deref() == Some("contentinfo"));
    if !has_footer {
        penalties.push(10.0);
    }

    // Heading structure: are there sub-sections?
    let heading_count = tree.headings().len();
    if heading_count < 2 {
        penalties.push(15.0);
        friction.push(FrictionPoint {
            step: "Orientation".into(),
            severity: "low".into(),
            problem: "Kaum Zwischenüberschriften zur Orientierung".into(),
            impact: "Nutzer können Inhalte nicht scannen".into(),
            recommendation: "Inhalte mit H2/H3-Überschriften gliedern".into(),
        });
    }

    let score = journey_dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Gute Orientierung durch Navigation, Landmarks und Struktur".into()
    } else if score >= 60 {
        "Grundlegende Orientierung vorhanden, Struktur ausbaufähig".into()
    } else {
        "Mangelnde Orientierung — Navigation oder Landmarks fehlen".into()
    };

    JourneyDimension {
        name: "Orientation".into(),
        score,
        weight: 0.20,
        summary,
    }
}

/// Navigation: Are links clear, non-redundant, and well-structured?
fn analyze_navigation(tree: &AXTree, friction: &mut Vec<FrictionPoint>) -> JourneyDimension {
    let links = tree.links();
    let mut penalties = Vec::new();

    // Generic link labels
    let generic_count = links
        .iter()
        .filter(|l| {
            let name = l.name.as_deref().unwrap_or("").to_lowercase();
            let trimmed = name.trim();
            GENERIC_LINK_LABELS
                .iter()
                .any(|g| trimmed == *g || trimmed.starts_with(g))
        })
        .count();

    if generic_count > 0 {
        let p = saturating_penalty(generic_count as f64, 30.0, 8.0);
        penalties.push(p);
        if generic_count >= 3 {
            friction.push(FrictionPoint {
                step: "Navigation".into(),
                severity: "medium".into(),
                problem: format!(
                    "{} Links mit generischen Texten (\"mehr\", \"hier\")",
                    generic_count
                ),
                impact: "Nutzer können nicht unterscheiden, wohin Links führen".into(),
                recommendation: "Linktexte beschreibend formulieren, die das Ziel benennen".into(),
            });
        }
    }

    // Empty / unnamed links
    let empty_links = links
        .iter()
        .filter(|l| l.name.as_deref().map_or(true, |n| n.trim().is_empty()))
        .count();

    if empty_links > 0 {
        let p = saturating_penalty(empty_links as f64, 35.0, 5.0);
        penalties.push(p);
        if empty_links >= 2 {
            friction.push(FrictionPoint {
                step: "Navigation".into(),
                severity: "high".into(),
                problem: format!("{} Links ohne erkennbaren Text", empty_links),
                impact: "Screenreader-Nutzer erfahren nicht, was der Link tut".into(),
                recommendation: "Alle Links mit beschreibendem Text oder aria-label versehen"
                    .into(),
            });
        }
    }

    // Duplicate link texts pointing potentially different places
    let mut link_names: Vec<String> = links
        .iter()
        .filter_map(|l| l.name.as_deref())
        .filter(|n| !n.trim().is_empty())
        .map(|n| n.to_lowercase().trim().to_string())
        .collect();
    link_names.sort();
    let total_links = link_names.len();
    link_names.dedup();
    let duplicate_count = total_links.saturating_sub(link_names.len());

    if duplicate_count > 5 {
        let p = saturating_penalty((duplicate_count - 5) as f64, 20.0, 10.0);
        penalties.push(p);
        if duplicate_count > 10 {
            friction.push(FrictionPoint {
                step: "Navigation".into(),
                severity: "low".into(),
                problem: format!("{} doppelte Linktexte auf der Seite", duplicate_count),
                impact: "Gleiche Beschriftung für unterschiedliche Ziele verwirrt Nutzer".into(),
                recommendation:
                    "Linktexte eindeutig formulieren oder mit aria-label differenzieren".into(),
            });
        }
    }

    // Link density: too many links for page size
    if links.len() > 60 {
        let excess = (links.len() - 60) as f64;
        let p = saturating_penalty(excess, 15.0, 60.0);
        penalties.push(p);
    }

    let score = journey_dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Links sind verständlich, eindeutig und gut strukturiert".into()
    } else if score >= 60 {
        "Navigation nutzbar, aber einige Links sind unklar oder redundant".into()
    } else {
        "Navigationsprobleme: unklare, leere oder redundante Links".into()
    };

    JourneyDimension {
        name: "Navigation".into(),
        score,
        weight: 0.25,
        summary,
    }
}

/// Interaction: Can users interact with controls effectively?
fn analyze_interaction(tree: &AXTree, friction: &mut Vec<FrictionPoint>) -> JourneyDimension {
    let buttons = tree.nodes_with_role("button");
    let form_controls = tree.form_controls();
    let mut penalties = Vec::new();

    // Buttons without accessible names
    let unnamed_buttons = buttons
        .iter()
        .filter(|b| b.name.as_deref().map_or(true, |n| n.trim().is_empty()))
        .count();

    if unnamed_buttons > 0 {
        let p = saturating_penalty(unnamed_buttons as f64, 40.0, 3.0);
        penalties.push(p);
        friction.push(FrictionPoint {
            step: "Interaction".into(),
            severity: "high".into(),
            problem: format!("{} Buttons ohne erkennbare Beschriftung", unnamed_buttons),
            impact: "Nutzer wissen nicht, was ein Button auslöst".into(),
            recommendation: "Alle Buttons mit beschreibendem Text oder aria-label versehen".into(),
        });
    }

    // Form controls without labels
    let unlabeled_forms = form_controls
        .iter()
        .filter(|fc| fc.name.as_deref().map_or(true, |n| n.trim().is_empty()))
        .count();

    if unlabeled_forms > 0 {
        let p = saturating_penalty(unlabeled_forms as f64, 35.0, 3.0);
        penalties.push(p);
        if unlabeled_forms >= 2 {
            friction.push(FrictionPoint {
                step: "Interaction".into(),
                severity: "high".into(),
                problem: format!("{} Formularfelder ohne Label", unlabeled_forms),
                impact: "Nutzer wissen nicht, welche Eingabe erwartet wird".into(),
                recommendation:
                    "Jedes Formularfeld mit sichtbarem <label> oder aria-label verbinden".into(),
            });
        }
    }

    // Generic button labels
    let generic_buttons = buttons
        .iter()
        .filter(|b| {
            let name = b.name.as_deref().unwrap_or("").to_lowercase();
            let trimmed = name.trim();
            matches!(
                trimmed,
                "ok" | "submit"
                    | "senden"
                    | "absenden"
                    | "go"
                    | "los"
                    | "weiter"
                    | "next"
                    | "click"
                    | "button"
            )
        })
        .count();

    if generic_buttons >= 2 {
        let p = saturating_penalty(generic_buttons as f64, 20.0, 5.0);
        penalties.push(p);
        friction.push(FrictionPoint {
            step: "Interaction".into(),
            severity: "low".into(),
            problem: format!(
                "{} Buttons mit generischen Labels (\"OK\", \"Submit\")",
                generic_buttons
            ),
            impact: "Kontext der Aktion ist nicht klar".into(),
            recommendation:
                "Buttons mit handlungsbeschreibenden Texten benennen (z. B. \"Nachricht senden\")"
                    .into(),
        });
    }

    let score = journey_dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Interaktive Elemente sind klar beschriftet und bedienbar".into()
    } else if score >= 60 {
        "Grundlegende Interaktion möglich, aber Beschriftungen teilweise unklar".into()
    } else {
        "Erhebliche Interaktionsprobleme: unbeschriftete Buttons oder Formularfelder".into()
    };

    JourneyDimension {
        name: "Interaction".into(),
        score,
        weight: 0.20,
        summary,
    }
}

/// Conversion: Can the user reach and complete the page's primary goal?
fn analyze_conversion(tree: &AXTree, friction: &mut Vec<FrictionPoint>) -> JourneyDimension {
    let buttons = tree.nodes_with_role("button");
    let links = tree.links();
    let mut penalties = Vec::new();

    // CTA presence
    let cta_found = buttons.iter().chain(links.iter()).any(|n| {
        let name = n.name.as_deref().unwrap_or("").to_lowercase();
        CTA_KEYWORDS.iter().any(|kw| name.contains(kw))
    });

    if !cta_found {
        penalties.push(40.0);
        friction.push(FrictionPoint {
            step: "Conversion".into(),
            severity: "high".into(),
            problem: "Kein erkennbarer Call-to-Action auf der Seite".into(),
            impact: "Nutzer haben keine klare Handlungsaufforderung".into(),
            recommendation: "Einen primären CTA definieren und prominent platzieren".into(),
        });
    }

    // Are there interactive blockers before CTA? (modals, cookie banners hinted by roles)
    let has_dialog = tree
        .iter()
        .any(|n| matches!(n.role.as_deref(), Some("dialog" | "alertdialog")));
    if has_dialog {
        penalties.push(15.0);
        friction.push(FrictionPoint {
            step: "Conversion".into(),
            severity: "medium".into(),
            problem: "Dialog/Overlay erkannt, der den Nutzerpfad unterbrechen kann".into(),
            impact: "Cookie-Banner oder Modals können den CTA verdecken".into(),
            recommendation:
                "Sicherstellen, dass Overlays einfach schließbar sind und den CTA nicht blockieren"
                    .into(),
        });
    }

    // Form complexity: too many fields can reduce conversion
    let form_controls = tree.form_controls();
    if form_controls.len() > 10 {
        let excess = (form_controls.len() - 10) as f64;
        let p = saturating_penalty(excess, 20.0, 10.0);
        penalties.push(p);
        if form_controls.len() > 15 {
            friction.push(FrictionPoint {
                step: "Conversion".into(),
                severity: "medium".into(),
                problem: format!("{} Formularfelder — hohe Eingabehürde", form_controls.len()),
                impact: "Komplexe Formulare reduzieren die Abschlussrate".into(),
                recommendation:
                    "Formular auf wesentliche Felder reduzieren oder in Schritte aufteilen".into(),
            });
        }
    }

    // CTA competition: multiple equally strong CTAs dilute focus
    let cta_count: usize = buttons
        .iter()
        .chain(links.iter())
        .filter(|n| {
            let name = n.name.as_deref().unwrap_or("").to_lowercase();
            CTA_KEYWORDS.iter().any(|kw| name.contains(kw))
        })
        .count();

    if cta_count > 5 {
        let p = saturating_penalty((cta_count - 5) as f64, 15.0, 5.0);
        penalties.push(p);
    }

    let score = journey_dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Klarer Conversion-Pfad mit erreichbarem CTA".into()
    } else if score >= 60 {
        "Conversion-Pfad vorhanden, aber durch Hindernisse beeinträchtigt".into()
    } else {
        "Kein klarer Conversion-Pfad erkennbar".into()
    };

    JourneyDimension {
        name: "Conversion".into(),
        score,
        weight: 0.15,
        summary,
    }
}
