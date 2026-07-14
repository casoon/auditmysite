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

/// Stable identifier for which UX dimension a [`UxDimension`] is.
///
/// Lets the PDF layer re-derive a localized `name`/`summary` without parsing
/// the canonical-English text fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UxDimensionKind {
    CtaClarity,
    VisualHierarchy,
    ContentClarity,
    TrustSignals,
    CognitiveLoad,
}

/// A scored UX dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxDimension {
    /// Which dimension this is (for localized re-derivation)
    pub kind: UxDimensionKind,
    /// Dimension name (canonical English)
    pub name: String,
    pub score: u32,
    pub weight: f64,
    /// Score-band summary (canonical English; derived from `kind` + `score`)
    pub summary: String,
}

/// Stable identifier for a concrete UX issue.
///
/// One variant per distinct problem/impact/recommendation shape. Together with
/// the raw values stored on [`UxIssue`] this fully reproduces the
/// human-readable strings in any language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UxIssueKind {
    // CTA Clarity
    NoCta,
    CompetingCtas,
    GenericLinks,
    // Visual Hierarchy
    NoH1,
    MultipleH1,
    HeadingSkips,
    LargeDom,
    // Content Clarity
    LittleText,
    NoSubheadings,
    // Trust Signals
    NoContact,
    NoImprint,
    NoPrivacy,
    // Cognitive Load
    TooManyLinks,
    TooManyInteractive,
}

impl UxIssueKind {
    /// Which dimension this issue belongs to.
    pub fn dimension(self) -> UxDimensionKind {
        use UxIssueKind::*;
        match self {
            NoCta | CompetingCtas | GenericLinks => UxDimensionKind::CtaClarity,
            NoH1 | MultipleH1 | HeadingSkips | LargeDom => UxDimensionKind::VisualHierarchy,
            LittleText | NoSubheadings => UxDimensionKind::ContentClarity,
            NoContact | NoImprint | NoPrivacy => UxDimensionKind::TrustSignals,
            TooManyLinks | TooManyInteractive => UxDimensionKind::CognitiveLoad,
        }
    }
}

/// The interpolated values a UX issue text may reference.
///
/// Stored on every [`UxIssue`] so that [`ux_issue_text`] can reproduce the
/// strings in any locale. Only the field relevant to the issue's `kind` is
/// populated.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UxIssueValues {
    /// A generic count (CompetingCtas, GenericLinks, MultipleH1, HeadingSkips,
    /// LargeDom, TooManyLinks, TooManyInteractive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Approximate word count (LittleText).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<u32>,
}

/// A single UX issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxIssue {
    /// Stable issue identifier (for localized re-derivation)
    pub kind: UxIssueKind,
    /// Dimension name (canonical English)
    pub dimension: String,
    pub severity: String,
    /// Problem statement (canonical English)
    pub problem: String,
    /// Impact statement (canonical English)
    pub impact: String,
    /// Recommendation (canonical English)
    pub recommendation: String,
    /// Interpolated values needed to reproduce the text in another language
    #[serde(default)]
    pub values: UxIssueValues,
}

impl UxIssue {
    /// Build an issue, baking canonical-English `dimension`/`problem`/`impact`/
    /// `recommendation` from its kind + values.
    fn new(kind: UxIssueKind, values: UxIssueValues) -> Self {
        let (problem, impact, recommendation) = ux_issue_text(kind, &values, true);
        UxIssue {
            kind,
            dimension: ux_dimension_name(kind.dimension(), true).to_string(),
            severity: ux_issue_severity(kind).to_string(),
            problem,
            impact,
            recommendation,
            values,
        }
    }
}

// ── Localized text (single source of truth) ─────────────────────────

/// Severity of a UX issue (stable, locale-independent).
fn ux_issue_severity(kind: UxIssueKind) -> &'static str {
    use UxIssueKind::*;
    match kind {
        NoCta | NoH1 | LittleText | NoContact => "high",
        CompetingCtas | GenericLinks | MultipleH1 | HeadingSkips | NoSubheadings | NoImprint
        | NoPrivacy | TooManyLinks | TooManyInteractive => "medium",
        LargeDom => "low",
    }
}

/// Localized dimension name for a [`UxDimensionKind`].
pub fn ux_dimension_name(kind: UxDimensionKind, _en: bool) -> &'static str {
    match kind {
        UxDimensionKind::CtaClarity => "CTA Clarity",
        UxDimensionKind::VisualHierarchy => "Visual Hierarchy",
        UxDimensionKind::ContentClarity => "Content Clarity",
        UxDimensionKind::TrustSignals => "Trust Signals",
        UxDimensionKind::CognitiveLoad => "Cognitive Load",
    }
}

/// Localized score-band summary for a UX dimension.
///
/// The single source of truth for dimension `summary` text. Analysis calls it
/// with `en = true` to bake canonical English; the PDF layer re-derives in the
/// run language.
pub fn ux_dimension_summary(
    kind: UxDimensionKind,
    score: u32,
    en: bool,
    has_flagged_issue: bool,
) -> String {
    use UxDimensionKind::*;
    let high = score >= 85;
    let mid = score >= 60;
    let base = match kind {
        CtaClarity => {
            if high {
                if en {
                    "Call-to-actions are clear and understandable"
                } else {
                    "Call-to-Actions sind klar und verständlich"
                }
            } else if mid {
                if en {
                    "CTAs present, but partly unclear or competing"
                } else {
                    "CTAs vorhanden, aber teilweise unklar oder konkurrierend"
                }
            } else if en {
                "CTAs are missing or not recognizable"
            } else {
                "CTAs fehlen oder sind nicht erkennbar"
            }
        }
        VisualHierarchy => {
            if high {
                if en {
                    "Clear visual hierarchy with a logical heading structure"
                } else {
                    "Klare visuelle Hierarchie mit logischer Heading-Struktur"
                }
            } else if mid {
                if en {
                    "Basic structure present, but heading hierarchy has gaps"
                } else {
                    "Grundstruktur vorhanden, aber Heading-Hierarchie lückenhaft"
                }
            } else if en {
                "Weak visual structure — page focus not recognizable"
            } else {
                "Schwache visuelle Struktur — Seitenfokus nicht erkennbar"
            }
        }
        ContentClarity => {
            if high {
                if en {
                    "Content is clearly structured and present in adequate volume"
                } else {
                    "Inhalte sind klar strukturiert und in angemessenem Umfang vorhanden"
                }
            } else if mid {
                if en {
                    "Content present, but structure or volume needs improvement"
                } else {
                    "Inhalte vorhanden, aber Struktur oder Umfang verbesserungswürdig"
                }
            } else if en {
                "Insufficient content or missing text structure"
            } else {
                "Unzureichende Inhalte oder fehlende Textstruktur"
            }
        }
        TrustSignals => {
            if high {
                if en {
                    "Trust signals present (contact, imprint, privacy policy)"
                } else {
                    "Vertrauenssignale vorhanden (Kontakt, Impressum, Datenschutz)"
                }
            } else if mid {
                if en {
                    "Basic trust signals partially present"
                } else {
                    "Grundlegende Vertrauenssignale teilweise vorhanden"
                }
            } else if en {
                "Important trust signals are missing"
            } else {
                "Wichtige Vertrauenssignale fehlen"
            }
        }
        CognitiveLoad => {
            if high {
                if en {
                    "Appropriate complexity — the page is well-organized"
                } else {
                    "Angemessene Komplexität — Seite ist übersichtlich"
                }
            } else if mid {
                if en {
                    "Slightly elevated complexity — navigation still manageable"
                } else {
                    "Leicht erhöhte Komplexität — Navigation noch handhabbar"
                }
            } else if en {
                "High complexity — the page feels cluttered"
            } else {
                "Hohe Komplexität — Seite wirkt überladen"
            }
        }
    };

    // A high score can still coexist with a flagged issue for the exact same
    // dimension (score thresholds and issue detection are independent
    // signals) — say so explicitly rather than reading as an unqualified,
    // contradicted "all clear" (#QA-039 report review).
    if high && has_flagged_issue {
        format!(
            "{} {}",
            base,
            if en {
                "— with one flagged exception below."
            } else {
                "— mit einer unten aufgeführten Ausnahme."
            }
        )
    } else {
        base.to_string()
    }
}

/// The single source of truth for UX issue `problem`/`impact`/`recommendation`.
///
/// Returns `(problem, impact, recommendation)` in German or English for the
/// given `kind` and interpolated `values`. Analysis calls it with `en = true`
/// to bake canonical English; the PDF layer re-derives in the run language.
pub fn ux_issue_text(
    kind: UxIssueKind,
    values: &UxIssueValues,
    en: bool,
) -> (String, String, String) {
    use UxIssueKind::*;
    let count = values.count.unwrap_or(0);

    let (problem, impact, recommendation): (String, String, String) = match kind {
        NoCta => (
            if en {
                "No recognizable call-to-action found".into()
            } else {
                "Kein erkennbarer Call-to-Action gefunden".into()
            },
            if en {
                "Users cannot tell what the next step is".into()
            } else {
                "Nutzer wissen nicht, was der nächste Schritt ist".into()
            },
            if en {
                "Clearly emphasize the primary CTA and give it an unambiguous label".into()
            } else {
                "Primären CTA klar hervorheben und eindeutig benennen".into()
            },
        ),
        CompetingCtas => (
            if en {
                format!("{} competing call-to-actions found", count)
            } else {
                format!("{} konkurrierende Call-to-Actions gefunden", count)
            },
            if en {
                "Too many equally weighted calls to action confuse users".into()
            } else {
                "Zu viele gleichwertige Handlungsaufforderungen verwirren Nutzer".into()
            },
            if en {
                "Prioritize one primary CTA and visually de-emphasize secondary ones".into()
            } else {
                "Einen primären CTA priorisieren, sekundäre visuell zurücknehmen".into()
            },
        ),
        GenericLinks => (
            if en {
                format!(
                    "{} generic link texts (\"more\", \"here\", \"click\")",
                    count
                )
            } else {
                format!(
                    "{} generische Linktexte (\"mehr\", \"hier\", \"klicken\")",
                    count
                )
            },
            if en {
                "Users cannot distinguish link targets".into()
            } else {
                "Nutzer können Ziele nicht unterscheiden".into()
            },
            if en {
                "Give links descriptive texts that name their target".into()
            } else {
                "Links mit beschreibenden Texten versehen, die das Ziel benennen".into()
            },
        ),
        NoH1 => (
            if en {
                "No H1 heading present".into()
            } else {
                "Keine H1-Überschrift vorhanden".into()
            },
            if en {
                "The page topic is not recognizable for users and search engines".into()
            } else {
                "Seitenthema ist für Nutzer und Suchmaschinen nicht erkennbar".into()
            },
            if en {
                "Set exactly one H1 heading with the page's main topic".into()
            } else {
                "Genau eine H1-Überschrift mit dem Hauptthema der Seite setzen".into()
            },
        ),
        MultipleH1 => (
            if en {
                format!("{} H1 headings found", count)
            } else {
                format!("{} H1-Überschriften gefunden", count)
            },
            if en {
                "The page has no clear primary focus".into()
            } else {
                "Seite hat keinen klaren Hauptfokus".into()
            },
            if en {
                "Use only one H1 heading per page".into()
            } else {
                "Nur eine H1-Überschrift pro Seite verwenden".into()
            },
        ),
        HeadingSkips => (
            if en {
                format!("Heading hierarchy skipped {} times (e.g. H2 → H4)", count)
            } else {
                format!(
                    "Heading-Hierarchie {} mal übersprungen (z. B. H2 → H4)",
                    count
                )
            },
            if en {
                "The page structure is unclear for screen readers and users".into()
            } else {
                "Seitenstruktur ist für Screenreader und Nutzer unklar".into()
            },
            if en {
                "Build heading levels without gaps (H1 → H2 → H3)".into()
            } else {
                "Heading-Ebenen lückenlos aufbauen (H1 → H2 → H3)".into()
            },
        ),
        LargeDom => (
            if en {
                format!("Very large DOM with {} nodes", count)
            } else {
                format!("Sehr großer DOM mit {} Knoten", count)
            },
            if en {
                "High visual complexity can overwhelm users".into()
            } else {
                "Hohe visuelle Komplexität kann Nutzer überfordern".into()
            },
            if en {
                "Simplify the page structure, fewer nested elements".into()
            } else {
                "Seitenstruktur vereinfachen, weniger verschachtelte Elemente".into()
            },
        ),
        LittleText => {
            let word_count = values.word_count.unwrap_or(0);
            (
                if en {
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
                if en {
                    "Users do not receive enough information to make a decision".into()
                } else {
                    "Nutzer erhalten nicht genügend Information für eine Entscheidung".into()
                },
                if en {
                    "Add relevant content that clearly conveys the page's purpose".into()
                } else {
                    "Relevanten Inhalt ergänzen, der den Seitenzweck klar vermittelt".into()
                },
            )
        }
        NoSubheadings => (
            if en {
                "A lot of text without sufficient subheadings".into()
            } else {
                "Viel Text ohne ausreichende Zwischenüberschriften".into()
            },
            if en {
                "Users cannot scan the content and fail to find relevant passages".into()
            } else {
                "Nutzer können Inhalte nicht scannen und finden relevante Stellen nicht".into()
            },
            if en {
                "Structure the text with subheadings (H2, H3)".into()
            } else {
                "Text mit Zwischenüberschriften (H2, H3) gliedern".into()
            },
        ),
        NoContact => (
            if en {
                "No contact link recognizable".into()
            } else {
                "Kein Kontakt-Link erkennbar".into()
            },
            if en {
                "No contact link recognizable on this page (heuristic — contact may be intentionally placed elsewhere).".into()
            } else {
                "Kein Kontakt-Link auf dieser Seite erkennbar (heuristisch — Kontakt könnte bewusst ausgelagert sein).".into()
            },
            if en {
                "Link the contact page or contact information clearly visible".into()
            } else {
                "Kontaktseite oder Kontaktinformationen gut sichtbar verlinken".into()
            },
        ),
        NoImprint => (
            if en {
                "No imprint link recognizable".into()
            } else {
                "Kein Impressum-Link erkennbar".into()
            },
            if en {
                "Legally required in DACH — signals a lack of credibility".into()
            } else {
                "Rechtlich erforderlich in DACH — signalisiert mangelnde Seriosität".into()
            },
            if en {
                "Link the imprint in the footer".into()
            } else {
                "Impressum im Footer verlinken".into()
            },
        ),
        NoPrivacy => (
            if en {
                "No privacy policy link recognizable".into()
            } else {
                "Kein Datenschutz-Link erkennbar".into()
            },
            if en {
                "Required under GDPR, strengthens user trust".into()
            } else {
                "DSGVO-Pflicht, stärkt Nutzervertrauen".into()
            },
            if en {
                "Link the privacy policy in the footer".into()
            } else {
                "Datenschutzerklärung im Footer verlinken".into()
            },
        ),
        TooManyLinks => (
            if en {
                format!("{} links on the page", count)
            } else {
                format!("{} Links auf der Seite", count)
            },
            if en {
                "High link density overwhelms users when orienting themselves".into()
            } else {
                "Hohe Linkdichte überfordert Nutzer bei der Orientierung".into()
            },
            if en {
                "Simplify navigation, prioritize and group links".into()
            } else {
                "Navigation vereinfachen, Links priorisieren und gruppieren".into()
            },
        ),
        TooManyInteractive => (
            if en {
                format!("{} interactive elements on the page", count)
            } else {
                format!("{} interaktive Elemente auf der Seite", count)
            },
            if en {
                "Too many interaction options make orientation harder".into()
            } else {
                "Zu viele Interaktionsmöglichkeiten erschweren die Orientierung".into()
            },
            if en {
                "Reduce interactive elements or group them into sections".into()
            } else {
                "Interaktive Elemente reduzieren oder in Abschnitte gruppieren".into()
            },
        ),
    };

    (problem, impact, recommendation)
}

/// Build a dimension, baking canonical-English `name`/`summary`.
fn build_dimension(kind: UxDimensionKind, score: u32, weight: f64) -> UxDimension {
    UxDimension {
        kind,
        name: ux_dimension_name(kind, true).to_string(),
        score,
        weight,
        // `false`: this bakes the canonical JSON summary for this dimension in
        // isolation, before the aggregated `issues` list exists to check
        // against. The PDF layer re-derives with the real flag (#406).
        summary: ux_dimension_summary(kind, score, true, false),
    }
}

// ── Analysis entry point ────────────────────────────────────────────

/// Analyze UX quality from the Accessibility Tree.
/// This runs purely on already-extracted AXTree data — no CDP calls needed.
///
/// Produces canonical-English text in the struct (and thus JSON).
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
    use UxIssueKind::*;
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
        issues.push(UxIssue::new(NoCta, UxIssueValues::default()));
    } else if cta_count > 5 {
        let p = saturating_penalty((cta_count - 5) as f64, 15.0, 5.0);
        penalties.push(p);
        issues.push(UxIssue::new(
            CompetingCtas,
            UxIssueValues {
                count: Some(cta_count as u32),
                ..Default::default()
            },
        ));
    }

    if generic_count > 0 {
        let p = saturating_penalty(generic_count as f64, 20.0, 5.0);
        penalties.push(p);
        if generic_count >= 3 {
            issues.push(UxIssue::new(
                GenericLinks,
                UxIssueValues {
                    count: Some(generic_count as u32),
                    ..Default::default()
                },
            ));
        }
    }

    let score = dimension_score(&penalties, 100.0);
    build_dimension(UxDimensionKind::CtaClarity, score, W_CTA)
}

fn analyze_visual_hierarchy(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    use UxIssueKind::*;
    let headings = tree.headings();
    let mut penalties = Vec::new();

    // Check H1
    let h1_count = headings
        .iter()
        .filter(|h| h.heading_level() == Some(1))
        .count();

    if h1_count == 0 {
        penalties.push(40.0);
        issues.push(UxIssue::new(NoH1, UxIssueValues::default()));
    } else if h1_count > 1 {
        penalties.push(15.0);
        issues.push(UxIssue::new(
            MultipleH1,
            UxIssueValues {
                count: Some(h1_count as u32),
                ..Default::default()
            },
        ));
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
            issues.push(UxIssue::new(
                HeadingSkips,
                UxIssueValues {
                    count: Some(skip_count as u32),
                    ..Default::default()
                },
            ));
        }
    }

    // Check DOM depth (very large trees = visual overload)
    let dom_size = tree.len();
    if dom_size > 2000 {
        let excess = (dom_size - 2000) as f64;
        let p = saturating_penalty(excess, 20.0, 2000.0);
        penalties.push(p);
        // Emit the finding whenever the penalty applies, so the Visual
        // Hierarchy score never drops without a stated reason (the penalty
        // already reaches ~8 points by 3000 nodes). Severity is "low".
        issues.push(UxIssue::new(
            LargeDom,
            UxIssueValues {
                count: Some(dom_size as u32),
                ..Default::default()
            },
        ));
    }

    let score = dimension_score(&penalties, 100.0);
    build_dimension(UxDimensionKind::VisualHierarchy, score, W_HIERARCHY)
}

fn analyze_content_clarity(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    use UxIssueKind::*;
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
        issues.push(UxIssue::new(
            LittleText,
            UxIssueValues {
                word_count: Some(word_count as u32),
                ..Default::default()
            },
        ));
    } else if word_count < 100 {
        penalties.push(20.0);
    }

    // Subheadings: content without structure
    if word_count > 200 && headings.len() < 3 {
        penalties.push(25.0);
        issues.push(UxIssue::new(NoSubheadings, UxIssueValues::default()));
    }

    // Very long page without structure
    if word_count > 1000 && headings.len() < 5 {
        penalties.push(15.0);
    }

    let score = dimension_score(&penalties, 100.0);
    build_dimension(UxDimensionKind::ContentClarity, score, W_CONTENT)
}

fn analyze_trust_signals(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    use UxIssueKind::*;
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
        issues.push(UxIssue::new(NoContact, UxIssueValues::default()));
    }

    if !impressum_found {
        penalties.push(20.0);
        issues.push(UxIssue::new(NoImprint, UxIssueValues::default()));
    }

    if !privacy_found {
        penalties.push(15.0);
        issues.push(UxIssue::new(NoPrivacy, UxIssueValues::default()));
    }

    // Overall trust signal density
    if trust_keyword_count < 3 {
        penalties.push(15.0);
    }

    let score = dimension_score(&penalties, 100.0);
    build_dimension(UxDimensionKind::TrustSignals, score, W_TRUST)
}

fn analyze_cognitive_load(tree: &AXTree, issues: &mut Vec<UxIssue>) -> UxDimension {
    use UxIssueKind::*;
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
            issues.push(UxIssue::new(
                TooManyLinks,
                UxIssueValues {
                    count: Some(link_count as u32),
                    ..Default::default()
                },
            ));
        }
    }

    // Too many interactive elements
    if interactive_count > 50 {
        let excess = (interactive_count - 50) as f64;
        let p = saturating_penalty(excess, 25.0, 50.0);
        penalties.push(p);
        if interactive_count > 100 {
            issues.push(UxIssue::new(
                TooManyInteractive,
                UxIssueValues {
                    count: Some(interactive_count as u32),
                    ..Default::default()
                },
            ));
        }
    }

    // Very large DOM
    if dom_size > 1500 {
        let excess = (dom_size - 1500) as f64;
        let p = saturating_penalty(excess, 20.0, 1000.0);
        penalties.push(p);
    }

    let score = dimension_score(&penalties, 100.0);
    build_dimension(UxDimensionKind::CognitiveLoad, score, W_COGNITIVE)
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

    /// Guard against German leaking into the canonical struct/JSON (#406): the
    /// analysis bakes English, so no visible result text (dimension summaries,
    /// issue problem/impact/recommendation) may contain German umlauts/ß.
    #[test]
    fn canonical_struct_has_no_german_chars() {
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

        let analysis = analyze_ux(&tree);
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
                !has_umlaut(&dim.name),
                "canonical dimension name contains German umlaut: {}",
                dim.name
            );
            assert!(
                !has_umlaut(&dim.summary),
                "canonical dimension summary contains German umlaut: {}",
                dim.summary
            );
        }

        for issue in &analysis.issues {
            assert!(
                !has_umlaut(&issue.problem),
                "canonical issue problem contains German umlaut: {}",
                issue.problem
            );
            assert!(
                !has_umlaut(&issue.impact),
                "canonical issue impact contains German umlaut: {}",
                issue.impact
            );
            assert!(
                !has_umlaut(&issue.recommendation),
                "canonical issue recommendation contains German umlaut: {}",
                issue.recommendation
            );
        }
    }

    /// PDF re-derivation: the pure text functions must yield real German.
    #[test]
    fn issue_text_german_for_pdf_derivation() {
        let (problem, impact, recommendation) =
            ux_issue_text(UxIssueKind::NoH1, &UxIssueValues::default(), false);
        assert_eq!(problem, "Keine H1-Überschrift vorhanden");
        assert!(impact.contains("Suchmaschinen"));
        assert!(recommendation.contains("Hauptthema"));

        // A count-interpolated variant localizes too.
        let (problem, _, _) = ux_issue_text(
            UxIssueKind::TooManyLinks,
            &UxIssueValues {
                count: Some(120),
                ..Default::default()
            },
            false,
        );
        assert_eq!(problem, "120 Links auf der Seite");

        // Dimension summary localizes.
        assert_eq!(
            ux_dimension_summary(UxDimensionKind::TrustSignals, 95, false, false),
            "Vertrauenssignale vorhanden (Kontakt, Impressum, Datenschutz)"
        );
    }

    #[test]
    fn dimension_summary_notes_a_flagged_exception_on_an_otherwise_high_score() {
        let plain = ux_dimension_summary(UxDimensionKind::CtaClarity, 91, false, false);
        let with_exception = ux_dimension_summary(UxDimensionKind::CtaClarity, 91, false, true);
        assert_ne!(plain, with_exception);
        assert!(with_exception.starts_with(&plain));
    }
}
