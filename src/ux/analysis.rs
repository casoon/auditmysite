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
pub fn analyze_ux(tree: &AXTree, locale: &str) -> UxAnalysis {
    info!("Analyzing UX heuristics...");

    let en = locale == "en";
    let mut issues = Vec::new();

    // ── 1. CTA Clarity ──────────────────────────────────────────────
    let cta_clarity = analyze_cta_clarity(tree, &mut issues, en);

    // ── 2. Visual Hierarchy ─────────────────────────────────────────
    let visual_hierarchy = analyze_visual_hierarchy(tree, &mut issues, en);

    // ── 3. Content Clarity ──────────────────────────────────────────
    let content_clarity = analyze_content_clarity(tree, &mut issues, en);

    // ── 4. Trust Signals ────────────────────────────────────────────
    let trust_signals = analyze_trust_signals(tree, &mut issues, en);

    // ── 5. Cognitive Load ───────────────────────────────────────────
    let cognitive_load = analyze_cognitive_load(tree, &mut issues, en);

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

fn analyze_cta_clarity(tree: &AXTree, issues: &mut Vec<UxIssue>, en: bool) -> UxDimension {
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
            problem: if en {
                "No recognizable call-to-action found".into()
            } else {
                "Kein erkennbarer Call-to-Action gefunden".into()
            },
            impact: if en {
                "Users cannot tell what the next step is".into()
            } else {
                "Nutzer wissen nicht, was der nächste Schritt ist".into()
            },
            recommendation: if en {
                "Clearly emphasize the primary CTA and give it an unambiguous label".into()
            } else {
                "Primären CTA klar hervorheben und eindeutig benennen".into()
            },
        });
    } else if cta_count > 5 {
        let p = saturating_penalty((cta_count - 5) as f64, 15.0, 5.0);
        penalties.push(p);
        issues.push(UxIssue {
            dimension: "CTA Clarity".into(),
            severity: "medium".into(),
            problem: if en {
                format!("{} competing call-to-actions found", cta_count)
            } else {
                format!("{} konkurrierende Call-to-Actions gefunden", cta_count)
            },
            impact: if en {
                "Too many equally weighted calls to action confuse users".into()
            } else {
                "Zu viele gleichwertige Handlungsaufforderungen verwirren Nutzer".into()
            },
            recommendation: if en {
                "Prioritize one primary CTA and visually de-emphasize secondary ones".into()
            } else {
                "Einen primären CTA priorisieren, sekundäre visuell zurücknehmen".into()
            },
        });
    }

    if generic_count > 0 {
        let p = saturating_penalty(generic_count as f64, 20.0, 5.0);
        penalties.push(p);
        if generic_count >= 3 {
            issues.push(UxIssue {
                dimension: "CTA Clarity".into(),
                severity: "medium".into(),
                problem: if en {
                    format!(
                        "{} generic link texts (\"more\", \"here\", \"click\")",
                        generic_count
                    )
                } else {
                    format!(
                        "{} generische Linktexte (\"mehr\", \"hier\", \"klicken\")",
                        generic_count
                    )
                },
                impact: if en {
                    "Users cannot distinguish link targets".into()
                } else {
                    "Nutzer können Ziele nicht unterscheiden".into()
                },
                recommendation: if en {
                    "Give links descriptive texts that name their target".into()
                } else {
                    "Links mit beschreibenden Texten versehen, die das Ziel benennen".into()
                },
            });
        }
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        if en {
            "Call-to-actions are clear and understandable".into()
        } else {
            "Call-to-Actions sind klar und verständlich".into()
        }
    } else if score >= 60 {
        if en {
            "CTAs present, but partly unclear or competing".into()
        } else {
            "CTAs vorhanden, aber teilweise unklar oder konkurrierend".into()
        }
    } else if en {
        "CTAs are missing or not recognizable".into()
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

fn analyze_visual_hierarchy(tree: &AXTree, issues: &mut Vec<UxIssue>, en: bool) -> UxDimension {
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
            problem: if en {
                "No H1 heading present".into()
            } else {
                "Keine H1-Überschrift vorhanden".into()
            },
            impact: if en {
                "The page topic is not recognizable for users and search engines".into()
            } else {
                "Seitenthema ist für Nutzer und Suchmaschinen nicht erkennbar".into()
            },
            recommendation: if en {
                "Set exactly one H1 heading with the page's main topic".into()
            } else {
                "Genau eine H1-Überschrift mit dem Hauptthema der Seite setzen".into()
            },
        });
    } else if h1_count > 1 {
        penalties.push(15.0);
        issues.push(UxIssue {
            dimension: "Visual Hierarchy".into(),
            severity: "medium".into(),
            problem: if en {
                format!("{} H1 headings found", h1_count)
            } else {
                format!("{} H1-Überschriften gefunden", h1_count)
            },
            impact: if en {
                "The page has no clear primary focus".into()
            } else {
                "Seite hat keinen klaren Hauptfokus".into()
            },
            recommendation: if en {
                "Use only one H1 heading per page".into()
            } else {
                "Nur eine H1-Überschrift pro Seite verwenden".into()
            },
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
                problem: if en {
                    format!(
                        "Heading hierarchy skipped {} times (e.g. H2 → H4)",
                        skip_count
                    )
                } else {
                    format!(
                        "Heading-Hierarchie {} mal übersprungen (z. B. H2 → H4)",
                        skip_count
                    )
                },
                impact: if en {
                    "The page structure is unclear for screen readers and users".into()
                } else {
                    "Seitenstruktur ist für Screenreader und Nutzer unklar".into()
                },
                recommendation: if en {
                    "Build heading levels without gaps (H1 → H2 → H3)".into()
                } else {
                    "Heading-Ebenen lückenlos aufbauen (H1 → H2 → H3)".into()
                },
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
                problem: if en {
                    format!("Very large DOM with {} nodes", dom_size)
                } else {
                    format!("Sehr großer DOM mit {} Knoten", dom_size)
                },
                impact: if en {
                    "High visual complexity can overwhelm users".into()
                } else {
                    "Hohe visuelle Komplexität kann Nutzer überfordern".into()
                },
                recommendation: if en {
                    "Simplify the page structure, fewer nested elements".into()
                } else {
                    "Seitenstruktur vereinfachen, weniger verschachtelte Elemente".into()
                },
            });
        }
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        if en {
            "Clear visual hierarchy with a logical heading structure".into()
        } else {
            "Klare visuelle Hierarchie mit logischer Heading-Struktur".into()
        }
    } else if score >= 60 {
        if en {
            "Basic structure present, but heading hierarchy has gaps".into()
        } else {
            "Grundstruktur vorhanden, aber Heading-Hierarchie lückenhaft".into()
        }
    } else if en {
        "Weak visual structure — page focus not recognizable".into()
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

fn analyze_content_clarity(tree: &AXTree, issues: &mut Vec<UxIssue>, en: bool) -> UxDimension {
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
            problem: if en {
                format!(
                    "Very little text content in the accessibility tree (~{} words, excluding purely visual or inaccessible content)",
                    word_count
                )
            } else {
                format!(
                    "Sehr wenig Textinhalt im Accessibility Tree (~{} Wörter, ohne rein visuelle oder nicht zugängliche Inhalte)",
                    word_count
                )
            },
            impact: if en {
                "Users do not receive enough information to make a decision".into()
            } else {
                "Nutzer erhalten nicht genügend Information für eine Entscheidung".into()
            },
            recommendation: if en {
                "Add relevant content that clearly conveys the page's purpose".into()
            } else {
                "Relevanten Inhalt ergänzen, der den Seitenzweck klar vermittelt".into()
            },
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
            problem: if en {
                "A lot of text without sufficient subheadings".into()
            } else {
                "Viel Text ohne ausreichende Zwischenüberschriften".into()
            },
            impact: if en {
                "Users cannot scan the content and fail to find relevant passages".into()
            } else {
                "Nutzer können Inhalte nicht scannen und finden relevante Stellen nicht".into()
            },
            recommendation: if en {
                "Structure the text with subheadings (H2, H3)".into()
            } else {
                "Text mit Zwischenüberschriften (H2, H3) gliedern".into()
            },
        });
    }

    // Very long page without structure
    if word_count > 1000 && headings.len() < 5 {
        penalties.push(15.0);
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        if en {
            "Content is clearly structured and present in adequate volume".into()
        } else {
            "Inhalte sind klar strukturiert und in angemessenem Umfang vorhanden".into()
        }
    } else if score >= 60 {
        if en {
            "Content present, but structure or volume needs improvement".into()
        } else {
            "Inhalte vorhanden, aber Struktur oder Umfang verbesserungswürdig".into()
        }
    } else if en {
        "Insufficient content or missing text structure".into()
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

fn analyze_trust_signals(tree: &AXTree, issues: &mut Vec<UxIssue>, en: bool) -> UxDimension {
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

    // Contact information may appear in headings or static text (not only as a
    // link). Check non-link nodes too so that pages with a visible "Kontakt"
    // heading or inline address block are not falsely flagged.
    if !contact_found {
        for node in tree.iter() {
            if matches!(
                node.role.as_deref(),
                Some("heading" | "StaticText" | "paragraph")
            ) {
                let name = node.name.as_deref().unwrap_or("").to_lowercase();
                if name.contains("kontakt") || name.contains("contact") {
                    contact_found = true;
                    break;
                }
            }
        }
    }

    if !contact_found {
        penalties.push(30.0);
        issues.push(UxIssue {
            dimension: "Trust Signals".into(),
            severity: "high".into(),
            problem: if en {
                "No contact link recognizable".into()
            } else {
                "Kein Kontakt-Link erkennbar".into()
            },
            impact: if en {
                "No contact link recognizable on this page (heuristic — contact may be intentionally placed elsewhere).".into()
            } else {
                "Kein Kontakt-Link auf dieser Seite erkennbar (heuristisch — Kontakt könnte bewusst ausgelagert sein).".into()
            },
            recommendation: if en {
                "Link the contact page or contact information clearly visible".into()
            } else {
                "Kontaktseite oder Kontaktinformationen gut sichtbar verlinken".into()
            },
        });
    }

    if !impressum_found {
        penalties.push(20.0);
        issues.push(UxIssue {
            dimension: "Trust Signals".into(),
            severity: "medium".into(),
            problem: if en {
                "No imprint link recognizable".into()
            } else {
                "Kein Impressum-Link erkennbar".into()
            },
            impact: if en {
                "Legally required in DACH — signals a lack of credibility".into()
            } else {
                "Rechtlich erforderlich in DACH — signalisiert mangelnde Seriosität".into()
            },
            recommendation: if en {
                "Link the imprint in the footer".into()
            } else {
                "Impressum im Footer verlinken".into()
            },
        });
    }

    if !privacy_found {
        penalties.push(15.0);
        issues.push(UxIssue {
            dimension: "Trust Signals".into(),
            severity: "medium".into(),
            problem: if en {
                "No privacy policy link recognizable".into()
            } else {
                "Kein Datenschutz-Link erkennbar".into()
            },
            impact: if en {
                "Required under GDPR, strengthens user trust".into()
            } else {
                "DSGVO-Pflicht, stärkt Nutzervertrauen".into()
            },
            recommendation: if en {
                "Link the privacy policy in the footer".into()
            } else {
                "Datenschutzerklärung im Footer verlinken".into()
            },
        });
    }

    // Overall trust signal density
    if trust_keyword_count < 3 {
        penalties.push(15.0);
    }

    let score = dimension_score(&penalties, 100.0);
    let summary = if score >= 85 {
        if en {
            "Trust signals present (contact, imprint, privacy policy)".into()
        } else {
            "Vertrauenssignale vorhanden (Kontakt, Impressum, Datenschutz)".into()
        }
    } else if score >= 60 {
        if en {
            "Basic trust signals partially present".into()
        } else {
            "Grundlegende Vertrauenssignale teilweise vorhanden".into()
        }
    } else if en {
        "Important trust signals are missing".into()
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

fn analyze_cognitive_load(tree: &AXTree, issues: &mut Vec<UxIssue>, en: bool) -> UxDimension {
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
                problem: if en {
                    format!("{} links on the page", link_count)
                } else {
                    format!("{} Links auf der Seite", link_count)
                },
                impact: if en {
                    "High link density overwhelms users when orienting themselves".into()
                } else {
                    "Hohe Linkdichte überfordert Nutzer bei der Orientierung".into()
                },
                recommendation: if en {
                    "Simplify navigation, prioritize and group links".into()
                } else {
                    "Navigation vereinfachen, Links priorisieren und gruppieren".into()
                },
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
                problem: if en {
                    format!("{} interactive elements on the page", interactive_count)
                } else {
                    format!("{} interaktive Elemente auf der Seite", interactive_count)
                },
                impact: if en {
                    "Too many interaction options make orientation harder".into()
                } else {
                    "Zu viele Interaktionsmöglichkeiten erschweren die Orientierung".into()
                },
                recommendation: if en {
                    "Reduce interactive elements or group them into sections".into()
                } else {
                    "Interaktive Elemente reduzieren oder in Abschnitte gruppieren".into()
                },
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
        if en {
            "Appropriate complexity — the page is well-organized".into()
        } else {
            "Angemessene Komplexität — Seite ist übersichtlich".into()
        }
    } else if score >= 60 {
        if en {
            "Slightly elevated complexity — navigation still manageable".into()
        } else {
            "Leicht erhöhte Komplexität — Navigation noch handhabbar".into()
        }
    } else if en {
        "High complexity — the page feels cluttered".into()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXNode;

    fn node(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.into(),
            role: Some(role.into()),
            name: name.map(|s| s.into()),
            ..Default::default()
        }
    }

    /// Guard against German leaking into EN reports (#406): build a scenario that
    /// triggers issue detectors across all UX dimensions and assert that no
    /// visible result text (dimension summaries, issue problem/impact/
    /// recommendation) contains German umlauts/ß when the locale is English.
    #[test]
    fn english_locale_carries_no_german_umlauts() {
        // Sparse tree: no H1, no contact/imprint/privacy links, no CTA, plus
        // generic and unnamed links → triggers detectors across all dimensions.
        let mut nodes = vec![node("root", "RootWebArea", Some("Page"))];
        // Generic link texts (triggers CTA-clarity generic-label issue).
        for (i, label) in ["mehr", "hier", "klicken", "weiter", "more"]
            .iter()
            .enumerate()
        {
            nodes.push(node(&format!("glink{i}"), "link", Some(label)));
        }
        // Filler links without trust/CTA keywords (no contact/imprint/privacy).
        for i in 0..6 {
            nodes.push(node(&format!("link{i}"), "link", Some("Page")));
        }
        let tree = AXTree::from_nodes(nodes);

        let analysis = analyze_ux(&tree, "en");
        assert!(
            !analysis.issues.is_empty(),
            "scenario should produce UX issues"
        );

        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));

        for dim in [
            &analysis.cta_clarity,
            &analysis.visual_hierarchy,
            &analysis.content_clarity,
            &analysis.trust_signals,
            &analysis.cognitive_load,
        ] {
            assert!(
                !has_umlaut(&dim.summary),
                "EN dimension summary contains German umlaut: {}",
                dim.summary
            );
        }

        for issue in &analysis.issues {
            assert!(
                !has_umlaut(&issue.problem),
                "EN issue problem contains German umlaut: {}",
                issue.problem
            );
            assert!(
                !has_umlaut(&issue.impact),
                "EN issue impact contains German umlaut: {}",
                issue.impact
            );
            assert!(
                !has_umlaut(&issue.recommendation),
                "EN issue recommendation contains German umlaut: {}",
                issue.recommendation
            );
        }
    }
}
