//! UX Analysis — heuristic evaluation from AXTree data
//!
//! Extracts CTA clarity, visual hierarchy, content clarity,
//! trust signals, and cognitive load metrics from the Accessibility Tree.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::accessibility::AXTree;
use crate::ux::scoring::{dimension_score, saturating_penalty, weighted_average};

// ── CTA detection keywords ──────────────────────────────────────────
const CTA_KEYWORDS_DE: &[&str] = &[
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
    "herunterladen",
    "newsletter",
    "abonnieren",
    "warenkorb",
    "kasse",
];

const CTA_KEYWORDS_EN: &[&str] = &[
    "buy",
    "order",
    "contact",
    "start",
    "register",
    "sign up",
    "book",
    "free",
    "trial",
    "demo",
    "schedule",
    "get started",
    "download",
    "subscribe",
    "cart",
    "checkout",
    "request",
    "apply",
];

const GENERIC_LABELS: &[&str] = &[
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

const TRUST_KEYWORDS: &[&str] = &[
    "kontakt",
    "impressum",
    "datenschutz",
    "agb",
    "über uns",
    "about",
    "partner",
    "referenz",
    "kunden",
    "zertifikat",
    "auszeichnung",
    "bewertung",
    "erfahrung",
    "garantie",
    "sicherheit",
    "ssl",
    "tüv",
    "iso",
    "dsgvo",
    "privacy",
    "terms",
    "imprint",
    "contact",
];

// ── Dimension weights ───────────────────────────────────────────────
const W_CTA: f64 = 0.30;
const W_HIERARCHY: f64 = 0.20;
const W_CONTENT: f64 = 0.20;
const W_TRUST: f64 = 0.15;
const W_COGNITIVE: f64 = 0.15;

// ── Public types ────────────────────────────────────────────────────

/// Complete UX analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxAnalysis {
    /// Overall UX score (0–100)
    pub score: u32,
    /// Grade (A–F)
    pub grade: String,
    /// Per-dimension results
    pub cta_clarity: UxDimension,
    pub visual_hierarchy: UxDimension,
    pub content_clarity: UxDimension,
    pub trust_signals: UxDimension,
    pub cognitive_load: UxDimension,
    /// All issues found
    pub issues: Vec<UxIssue>,
}

/// A scored UX dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxDimension {
    pub name: String,
    pub score: u32,
    pub weight: f64,
    pub summary: String,
}

/// A single UX issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxIssue {
    pub dimension: String,
    pub severity: String,
    pub problem: String,
    pub impact: String,
    pub recommendation: String,
}

// ── Analysis entry point ────────────────────────────────────────────

/// Analyze UX quality from the Accessibility Tree.
/// This runs purely on already-extracted AXTree data — no CDP calls needed.
pub fn analyze_ux(tree: &AXTree) -> UxAnalysis {
    info!("Analyzing UX heuristics...");

    let mut issues = Vec::new();

    // ── 1. CTA Clarity ──────────────────────────────────────────────
    let cta_clarity = analyze_cta_clarity(tree, &mut issues);

    // ── 2. Visual Hierarchy ─────────────────────────────────────────
    let visual_hierarchy = analyze_visual_hierarchy(tree, &mut issues);

    // ── 3. Content Clarity ──────────────────────────────────────────
    let content_clarity = analyze_content_clarity(tree, &mut issues);

    // ── 4. Trust Signals ────────────────────────────────────────────
    let trust_signals = analyze_trust_signals(tree, &mut issues);

    // ── 5. Cognitive Load ───────────────────────────────────────────
    let cognitive_load = analyze_cognitive_load(tree, &mut issues);

    // ── Overall score ───────────────────────────────────────────────
    let score = weighted_average(&[
        (cta_clarity.score, W_CTA),
        (visual_hierarchy.score, W_HIERARCHY),
        (content_clarity.score, W_CONTENT),
        (trust_signals.score, W_TRUST),
        (cognitive_load.score, W_COGNITIVE),
    ]);

    let grade = match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
    .to_string();

    info!("UX analysis: score={}, issues={}", score, issues.len());

    UxAnalysis {
        score,
        grade,
        cta_clarity,
        visual_hierarchy,
        content_clarity,
        trust_signals,
        cognitive_load,
        issues,
    }
}

// ── Dimension analyzers ─────────────────────────────────────────────

fn analyze_cta_clarity(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    let buttons = tree.nodes_with_role("button");
    let links = tree.links();

    // Find primary CTAs (buttons or links with CTA keywords)
    let mut cta_count = 0;
    let mut generic_count = 0;
    let mut primary_found = false;

    for node in buttons.iter().chain(links.iter()) {
        let name = node.name.as_deref().unwrap_or("").to_lowercase();
        if name.is_empty() {
            continue;
        }

        let is_cta = CTA_KEYWORDS_DE
            .iter()
            .chain(CTA_KEYWORDS_EN.iter())
            .any(|kw| name.contains(kw));
        let is_generic = GENERIC_LABELS
            .iter()
            .any(|g| name.trim() == *g || name.starts_with(g));

        if is_cta {
            cta_count += 1;
            primary_found = true;
        }
        if is_generic {
            generic_count += 1;
        }
    }

    let mut penalties = Vec::new();

    if !primary_found {
        penalties.push(45.0);
        issues.push(UxIssue {
            dimension: "CTA Clarity".into(),
            severity: "high".into(),
            problem: "Kein erkennbarer Call-to-Action gefunden".into(),
            impact: "Nutzer wissen nicht, was der nächste Schritt ist".into(),
            recommendation: "Primären CTA klar hervorheben und eindeutig benennen".into(),
        });
    } else if cta_count > 5 {
        let p = saturating_penalty((cta_count - 5) as f64, 15.0, 5.0);
        penalties.push(p);
        issues.push(UxIssue {
            dimension: "CTA Clarity".into(),
            severity: "medium".into(),
            problem: format!("{} konkurrierende Call-to-Actions gefunden", cta_count),
            impact: "Zu viele gleichwertige Handlungsaufforderungen verwirren Nutzer".into(),
            recommendation: "Einen primären CTA priorisieren, sekundäre visuell zurücknehmen"
                .into(),
        });
    }

    if generic_count > 0 {
        let p = saturating_penalty(generic_count as f64, 20.0, 5.0);
        penalties.push(p);
        if generic_count >= 3 {
            issues.push(UxIssue {
                dimension: "CTA Clarity".into(),
                severity: "medium".into(),
                problem: format!(
                    "{} generische Linktexte (\"mehr\", \"hier\", \"klicken\")",
                    generic_count
                ),
                impact: "Nutzer können Ziele nicht unterscheiden".into(),
                recommendation: "Links mit beschreibenden Texten versehen, die das Ziel benennen"
                    .into(),
            });
        }
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Call-to-Actions sind klar und verständlich".into()
    } else if score >= 60 {
        "CTAs vorhanden, aber teilweise unklar oder konkurrierend".into()
    } else {
        "CTAs fehlen oder sind nicht erkennbar".into()
    };

    UxDimension {
        name: "CTA Clarity".into(),
        score,
        weight: W_CTA,
        summary,
    }
}

fn analyze_visual_hierarchy(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    let headings = tree.headings();
    let mut penalties = Vec::new();

    // Check H1
    let h1_count = headings
        .iter()
        .filter(|h| h.heading_level() == Some(1))
        .count();

    if h1_count == 0 {
        penalties.push(40.0);
        issues.push(UxIssue {
            dimension: "Visual Hierarchy".into(),
            severity: "high".into(),
            problem: "Keine H1-Überschrift vorhanden".into(),
            impact: "Seitenthema ist für Nutzer und Suchmaschinen nicht erkennbar".into(),
            recommendation: "Genau eine H1-Überschrift mit dem Hauptthema der Seite setzen".into(),
        });
    } else if h1_count > 1 {
        penalties.push(15.0);
        issues.push(UxIssue {
            dimension: "Visual Hierarchy".into(),
            severity: "medium".into(),
            problem: format!("{} H1-Überschriften gefunden", h1_count),
            impact: "Seite hat keinen klaren Hauptfokus".into(),
            recommendation: "Nur eine H1-Überschrift pro Seite verwenden".into(),
        });
    }

    // Check heading order
    let mut last_level: u8 = 0;
    let mut skip_count = 0;
    for h in &headings {
        if let Some(level) = h.heading_level() {
            if last_level > 0 && level > last_level + 1 {
                skip_count += 1;
            }
            last_level = level;
        }
    }
    if skip_count > 0 {
        let p = saturating_penalty(skip_count as f64, 30.0, 3.0);
        penalties.push(p);
        if skip_count >= 2 {
            issues.push(UxIssue {
                dimension: "Visual Hierarchy".into(),
                severity: "medium".into(),
                problem: format!(
                    "Heading-Hierarchie {} mal übersprungen (z. B. H2 → H4)",
                    skip_count
                ),
                impact: "Seitenstruktur ist für Screenreader und Nutzer unklar".into(),
                recommendation: "Heading-Ebenen lückenlos aufbauen (H1 → H2 → H3)".into(),
            });
        }
    }

    // Check DOM depth (very large trees = visual overload)
    let dom_size = tree.len();
    if dom_size > 2000 {
        let excess = (dom_size - 2000) as f64;
        let p = saturating_penalty(excess, 20.0, 2000.0);
        penalties.push(p);
        if dom_size > 4000 {
            issues.push(UxIssue {
                dimension: "Visual Hierarchy".into(),
                severity: "low".into(),
                problem: format!("Sehr großer DOM mit {} Knoten", dom_size),
                impact: "Hohe visuelle Komplexität kann Nutzer überfordern".into(),
                recommendation: "Seitenstruktur vereinfachen, weniger verschachtelte Elemente"
                    .into(),
            });
        }
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Klare visuelle Hierarchie mit logischer Heading-Struktur".into()
    } else if score >= 60 {
        "Grundstruktur vorhanden, aber Heading-Hierarchie lückenhaft".into()
    } else {
        "Schwache visuelle Struktur — Seitenfokus nicht erkennbar".into()
    };

    UxDimension {
        name: "Visual Hierarchy".into(),
        score,
        weight: W_HIERARCHY,
        summary,
    }
}

fn analyze_content_clarity(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    let headings = tree.headings();
    let mut penalties = Vec::new();

    // Count text content (approximation from AXTree names)
    let mut total_text_len = 0usize;
    let mut _text_node_count = 0usize;
    for node in tree.iter() {
        if let Some(role) = node.role.as_deref() {
            if matches!(
                role,
                "StaticText" | "paragraph" | "listitem" | "cell" | "heading"
            ) {
                if let Some(name) = &node.name {
                    total_text_len += name.len();
                    _text_node_count += 1;
                }
            }
        }
    }

    // Approximate word count (German avg ~6 chars/word)
    let word_count = total_text_len / 6;

    if word_count < 50 {
        penalties.push(40.0);
        issues.push(UxIssue {
            dimension: "Content Clarity".into(),
            severity: "high".into(),
            problem: format!("Sehr wenig Textinhalt (~{} Wörter)", word_count),
            impact: "Nutzer erhalten nicht genügend Information für eine Entscheidung".into(),
            recommendation: "Relevanten Inhalt ergänzen, der den Seitenzweck klar vermittelt"
                .into(),
        });
    } else if word_count < 100 {
        penalties.push(20.0);
    }

    // Subheadings: content without structure
    if word_count > 200 && headings.len() < 3 {
        penalties.push(25.0);
        issues.push(UxIssue {
            dimension: "Content Clarity".into(),
            severity: "medium".into(),
            problem: "Viel Text ohne ausreichende Zwischenüberschriften".into(),
            impact: "Nutzer können Inhalte nicht scannen und finden relevante Stellen nicht".into(),
            recommendation: "Text mit Zwischenüberschriften (H2, H3) gliedern".into(),
        });
    }

    // Very long page without structure
    if word_count > 1000 && headings.len() < 5 {
        penalties.push(15.0);
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Inhalte sind klar strukturiert und in angemessenem Umfang vorhanden".into()
    } else if score >= 60 {
        "Inhalte vorhanden, aber Struktur oder Umfang verbesserungswürdig".into()
    } else {
        "Unzureichende Inhalte oder fehlende Textstruktur".into()
    };

    UxDimension {
        name: "Content Clarity".into(),
        score,
        weight: W_CONTENT,
        summary,
    }
}

fn analyze_trust_signals(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    let links = tree.links();
    let mut penalties = Vec::new();

    // Scan links and text for trust keywords
    let mut contact_found = false;
    let mut impressum_found = false;
    let mut privacy_found = false;
    let mut trust_keyword_count = 0;

    for node in links.iter() {
        let name = node.name.as_deref().unwrap_or("").to_lowercase();
        if name.is_empty() {
            continue;
        }

        if name.contains("kontakt") || name.contains("contact") {
            contact_found = true;
        }
        if name.contains("impressum") || name.contains("imprint") {
            impressum_found = true;
        }
        if name.contains("datenschutz") || name.contains("privacy") {
            privacy_found = true;
        }
        if TRUST_KEYWORDS.iter().any(|kw| name.contains(kw)) {
            trust_keyword_count += 1;
        }
    }

    if !contact_found {
        penalties.push(30.0);
        issues.push(UxIssue {
            dimension: "Trust Signals".into(),
            severity: "high".into(),
            problem: "Kein Kontakt-Link erkennbar".into(),
            impact: "Nutzer können bei Fragen oder Problemen keinen Ansprechpartner finden".into(),
            recommendation: "Kontaktseite oder Kontaktinformationen gut sichtbar verlinken".into(),
        });
    }

    if !impressum_found {
        penalties.push(20.0);
        issues.push(UxIssue {
            dimension: "Trust Signals".into(),
            severity: "medium".into(),
            problem: "Kein Impressum-Link erkennbar".into(),
            impact: "Rechtlich erforderlich in DACH — signalisiert mangelnde Seriosität".into(),
            recommendation: "Impressum im Footer verlinken".into(),
        });
    }

    if !privacy_found {
        penalties.push(15.0);
        issues.push(UxIssue {
            dimension: "Trust Signals".into(),
            severity: "medium".into(),
            problem: "Kein Datenschutz-Link erkennbar".into(),
            impact: "DSGVO-Pflicht, stärkt Nutzervertrauen".into(),
            recommendation: "Datenschutzerklärung im Footer verlinken".into(),
        });
    }

    // Overall trust signal density
    if trust_keyword_count < 3 {
        penalties.push(15.0);
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Vertrauenssignale vorhanden (Kontakt, Impressum, Datenschutz)".into()
    } else if score >= 60 {
        "Grundlegende Vertrauenssignale teilweise vorhanden".into()
    } else {
        "Wichtige Vertrauenssignale fehlen".into()
    };

    UxDimension {
        name: "Trust Signals".into(),
        score,
        weight: W_TRUST,
        summary,
    }
}

fn analyze_cognitive_load(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    let link_count = tree.links().len();
    let interactive_count = tree.iter().filter(|n| n.is_interactive()).count();
    let dom_size = tree.len();

    let mut penalties = Vec::new();

    // Too many links
    if link_count > 40 {
        let excess = (link_count - 40) as f64;
        let p = saturating_penalty(excess, 30.0, 80.0);
        penalties.push(p);
        if link_count > 80 {
            issues.push(UxIssue {
                dimension: "Cognitive Load".into(),
                severity: "medium".into(),
                problem: format!("{} Links auf der Seite", link_count),
                impact: "Hohe Linkdichte überfordert Nutzer bei der Orientierung".into(),
                recommendation: "Navigation vereinfachen, Links priorisieren und gruppieren".into(),
            });
        }
    }

    // Too many interactive elements
    if interactive_count > 50 {
        let excess = (interactive_count - 50) as f64;
        let p = saturating_penalty(excess, 25.0, 50.0);
        penalties.push(p);
        if interactive_count > 100 {
            issues.push(UxIssue {
                dimension: "Cognitive Load".into(),
                severity: "medium".into(),
                problem: format!("{} interaktive Elemente auf der Seite", interactive_count),
                impact: "Zu viele Interaktionsmöglichkeiten erschweren die Orientierung".into(),
                recommendation: "Interaktive Elemente reduzieren oder in Abschnitte gruppieren"
                    .into(),
            });
        }
    }

    // Very large DOM
    if dom_size > 1500 {
        let excess = (dom_size - 1500) as f64;
        let p = saturating_penalty(excess, 20.0, 1000.0);
        penalties.push(p);
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        "Angemessene Komplexität — Seite ist übersichtlich".into()
    } else if score >= 60 {
        "Leicht erhöhte Komplexität — Navigation noch handhabbar".into()
    } else {
        "Hohe Komplexität — Seite wirkt überladen".into()
    };

    UxDimension {
        name: "Cognitive Load".into(),
        score,
        weight: W_COGNITIVE,
        summary,
    }
}
