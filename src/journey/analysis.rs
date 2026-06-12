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
    // English
    "more",
    "click here",
    "read more",
    "learn more",
    "details",
    "info",
    "link",
    "here",
    "view",
    "see more",
    "see all",
    // German
    "mehr",
    "hier",
    "klicken",
    "weiter",
    "hier klicken",
    "mehr erfahren",
    "weiterlesen",
    "alle anzeigen",
    "ansehen",
    "jetzt lesen",
    "öffnen",
    // French
    "ici",
    "cliquez ici",
    "en savoir plus",
    "lire la suite",
    "lire plus",
    "voir plus",
    "voir tout",
    "télécharger",
    // Spanish
    "aquí",
    "haz clic aquí",
    "leer más",
    "saber más",
    "ver más",
    "ver todo",
    // Italian
    "qui",
    "clicca qui",
    "leggi di più",
    "scopri di più",
    "vedi di più",
    "vedi tutto",
    // Portuguese
    "aqui",
    "clique aqui",
    "leia mais",
    "saiba mais",
    "ver mais",
    // Dutch
    "hier",
    "klik hier",
    "meer lezen",
    "lees meer",
    "bekijk meer",
    "volgende",
    // Swedish
    "här",
    "klicka här",
    "läs mer",
    // Norwegian
    "her",
    "klikk her",
    "les mer",
    // Polish
    "tutaj",
    "kliknij tutaj",
    "czytaj więcej",
    "więcej",
    // Turkish
    "devamını oku",
    "daha fazla",
];

const CTA_KEYWORDS: &[&str] = &[
    // English
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
    "download",
    "demo",
    "newsletter",
    // German
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
    "termin",
    "beratung",
    "angebot",
    "warenkorb",
    "kasse",
    "herunterladen",
    // French
    "acheter",
    "commander",
    "contacter",
    "commencer",
    "s'inscrire",
    "réserver",
    "gratuit",
    "essai",
    "télécharger",
    "abonner",
    "panier",
    // Spanish
    "comprar",
    "pedir",
    "contactar",
    "empezar",
    "registrarse",
    "reservar",
    "gratis",
    "prueba",
    "descargar",
    "suscribirse",
    "carrito",
    // Italian
    "acquistare",
    "ordinare",
    "contattare",
    "iniziare",
    "registrarsi",
    "prenotare",
    "gratuito",
    "prova",
    "scaricare",
    "iscriversi",
    "carrello",
    // Portuguese
    "comprar",
    "pedir",
    "contactar",
    "começar",
    "registrar",
    "reservar",
    "grátis",
    "baixar",
    "assinar",
    "carrinho",
    // Dutch
    "kopen",
    "bestellen",
    "contact",
    "starten",
    "registreren",
    "boeken",
    "gratis",
    "proberen",
    "downloaden",
    "abonneren",
    "winkelwagen",
    // Swedish
    "köpa",
    "beställa",
    "kontakta",
    "starta",
    "registrera",
    "boka",
    "gratis",
    "ladda ner",
    "prenumerera",
    // Polish
    "kupić",
    "zamówić",
    "kontakt",
    "zarejestrować",
    "zarezerwować",
    "bezpłatny",
    "pobierz",
    "subskrybować",
    // Turkish
    "satın al",
    "sipariş ver",
    "iletişim",
    "başla",
    "kayıt ol",
    "rezervasyon",
    "ücretsiz",
    "indir",
    "abone ol",
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

/// Stable identifier for which journey dimension a [`JourneyDimension`] is.
///
/// Lets the PDF layer re-derive a localized `name`/`summary` without parsing
/// the canonical-English text fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JourneyDimensionKind {
    EntryClarity,
    Orientation,
    Navigation,
    Interaction,
    Conversion,
}

/// A scored journey dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyDimension {
    /// Which dimension this is (for localized re-derivation)
    pub kind: JourneyDimensionKind,
    /// Dimension name (canonical English)
    pub name: String,
    pub score: u32,
    pub weight: f64,
    /// Score-band summary (canonical English; derived from `kind` + `score`)
    pub summary: String,
}

/// Stable identifier for a concrete friction point.
///
/// One variant per distinct problem/impact/recommendation shape. Together with
/// the raw values stored on [`FrictionPoint`] this fully reproduces the
/// human-readable strings in any language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrictionKind {
    // Entry
    NoH1,
    MultipleH1,
    NoPageTitle,
    LittleEarlyText,
    // Orientation
    NavigationMissing,
    MainHiddenFromAx,
    MainMissing,
    FewSubheadings,
    // Navigation
    GenericLinks,
    EmptyLinks,
    DuplicateLinks,
    // Interaction
    UnnamedButtons,
    UnlabeledForms,
    GenericButtons,
    // Conversion
    NoCallToAction,
    BlockingDialog,
    ComplexForm,
}

/// The interpolated values a friction text may reference.
///
/// Stored on every [`FrictionPoint`] so that [`journey_friction_text`] can
/// reproduce the strings in any locale. Only the field relevant to the
/// friction's `kind` is populated.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrictionValues {
    /// A generic count (MultipleH1, GenericLinks, EmptyLinks, DuplicateLinks,
    /// UnnamedButtons, UnlabeledForms, GenericButtons, ComplexForm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
}

/// A point of friction in the user journey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionPoint {
    /// Stable friction identifier (for localized re-derivation)
    pub kind: FrictionKind,
    pub step: String,
    pub severity: String,
    /// Problem statement (canonical English)
    pub problem: String,
    /// Impact statement (canonical English)
    pub impact: String,
    /// Recommendation (canonical English)
    pub recommendation: String,
    /// Interpolated values needed to reproduce the text in another language
    #[serde(default)]
    pub values: FrictionValues,
}

impl FrictionPoint {
    /// Build a friction point, baking canonical-English `problem`/`impact`/
    /// `recommendation` from its kind + values via [`journey_friction_text`].
    fn new(kind: FrictionKind, values: FrictionValues) -> Self {
        let (problem, impact, recommendation) = journey_friction_text(kind, &values, true);
        FrictionPoint {
            kind,
            step: friction_step(kind).to_string(),
            severity: friction_severity(kind).to_string(),
            problem,
            impact,
            recommendation,
            values,
        }
    }
}

// ── Localized text (single source of truth) ─────────────────────────

/// Journey step the friction belongs to (stable, locale-independent).
fn friction_step(kind: FrictionKind) -> &'static str {
    use FrictionKind::*;
    match kind {
        NoH1 | MultipleH1 | NoPageTitle | LittleEarlyText => "Entry",
        NavigationMissing | MainHiddenFromAx | MainMissing | FewSubheadings => "Orientation",
        GenericLinks | EmptyLinks | DuplicateLinks => "Navigation",
        UnnamedButtons | UnlabeledForms | GenericButtons => "Interaction",
        NoCallToAction | BlockingDialog | ComplexForm => "Conversion",
    }
}

/// Severity of a friction point (stable, locale-independent).
fn friction_severity(kind: FrictionKind) -> &'static str {
    use FrictionKind::*;
    match kind {
        NoH1 | NavigationMissing | EmptyLinks | UnnamedButtons | UnlabeledForms
        | NoCallToAction => "high",
        MultipleH1 | NoPageTitle | LittleEarlyText | MainMissing | BlockingDialog | ComplexForm => {
            "medium"
        }
        MainHiddenFromAx | FewSubheadings | DuplicateLinks | GenericLinks | GenericButtons => "low",
    }
}

/// Localized dimension name for a [`JourneyDimensionKind`].
pub fn journey_dimension_name(kind: JourneyDimensionKind, en: bool) -> &'static str {
    match (kind, en) {
        (JourneyDimensionKind::EntryClarity, _) => "Entry Clarity",
        (JourneyDimensionKind::Orientation, _) => "Orientation",
        (JourneyDimensionKind::Navigation, _) => "Navigation",
        (JourneyDimensionKind::Interaction, _) => "Interaction",
        (JourneyDimensionKind::Conversion, _) => "Conversion",
    }
}

/// Localized score-band summary for a journey dimension.
///
/// The single source of truth for dimension `summary` text. Analysis calls it
/// with `en = true` to bake canonical English; the PDF layer re-derives in the
/// run language.
pub fn journey_dimension_summary(kind: JourneyDimensionKind, score: u32, en: bool) -> String {
    use JourneyDimensionKind::*;
    let high = score >= 85;
    let mid = score >= 60;
    match kind {
        EntryClarity => {
            if high {
                if en {
                    "The page purpose is immediately clear"
                } else {
                    "Seitenzweck ist sofort klar erkennbar"
                }
            } else if mid {
                if en {
                    "The entry point is broadly understandable, but could be improved"
                } else {
                    "Einstieg grundsätzlich verständlich, aber verbesserungswürdig"
                }
            } else if en {
                "The page purpose is not recognizable at first glance"
            } else {
                "Seitenzweck ist nicht auf den ersten Blick erkennbar"
            }
        }
        Orientation => {
            if high {
                if en {
                    "Good orientation through navigation, landmarks and structure"
                } else {
                    "Gute Orientierung durch Navigation, Landmarks und Struktur"
                }
            } else if mid {
                if en {
                    "Basic orientation present, structure could be expanded"
                } else {
                    "Grundlegende Orientierung vorhanden, Struktur ausbaufähig"
                }
            } else if en {
                "Insufficient orientation — navigation or landmarks are missing"
            } else {
                "Mangelnde Orientierung — Navigation oder Landmarks fehlen"
            }
        }
        Navigation => {
            if high {
                if en {
                    "Links are understandable, unambiguous and well structured"
                } else {
                    "Links sind verständlich, eindeutig und gut strukturiert"
                }
            } else if mid {
                if en {
                    "Navigation is usable, but some links are unclear or redundant"
                } else {
                    "Navigation nutzbar, aber einige Links sind unklar oder redundant"
                }
            } else if en {
                "Navigation problems: unclear, empty or redundant links"
            } else {
                "Navigationsprobleme: unklare, leere oder redundante Links"
            }
        }
        Interaction => {
            if high {
                if en {
                    "Interactive elements are clearly labeled and operable"
                } else {
                    "Interaktive Elemente sind klar beschriftet und bedienbar"
                }
            } else if mid {
                if en {
                    "Basic interaction is possible, but some labels are unclear"
                } else {
                    "Grundlegende Interaktion möglich, aber Beschriftungen teilweise unklar"
                }
            } else if en {
                "Significant interaction problems: unlabeled buttons or form fields"
            } else {
                "Erhebliche Interaktionsprobleme: unbeschriftete Buttons oder Formularfelder"
            }
        }
        Conversion => {
            if high {
                if en {
                    "Clear conversion path with a reachable CTA"
                } else {
                    "Klarer Conversion-Pfad mit erreichbarem CTA"
                }
            } else if mid {
                if en {
                    "Conversion path present, but impaired by obstacles"
                } else {
                    "Conversion-Pfad vorhanden, aber durch Hindernisse beeinträchtigt"
                }
            } else if en {
                "No clear conversion path detectable"
            } else {
                "Kein klarer Conversion-Pfad erkennbar"
            }
        }
    }
    .to_string()
}

/// The single source of truth for friction `problem`/`impact`/`recommendation`.
///
/// Returns `(problem, impact, recommendation)` in German or English for the
/// given `kind` and interpolated `values`. Analysis calls it with `en = true`
/// to bake canonical English; the PDF layer re-derives in the run language.
pub fn journey_friction_text(
    kind: FrictionKind,
    values: &FrictionValues,
    en: bool,
) -> (String, String, String) {
    use FrictionKind::*;
    let count = values.count.unwrap_or(0);

    let (problem, impact, recommendation): (String, String, String) = match kind {
        NoH1 => (
            if en {
                "No H1 heading — the page purpose is not immediately recognizable".into()
            } else {
                "Keine H1-Überschrift — Seitenzweck nicht sofort erkennbar".into()
            },
            if en {
                "Users do not understand what this page is about".into()
            } else {
                "Nutzer verstehen nicht, worum es auf dieser Seite geht".into()
            },
            if en {
                "Set a meaningful H1 heading".into()
            } else {
                "Eine aussagekräftige H1-Überschrift setzen".into()
            },
        ),
        MultipleH1 => (
            if en {
                format!("{} H1 headings — unclear page focus", count)
            } else {
                format!("{} H1-Überschriften — unklarer Seitenfokus", count)
            },
            if en {
                "Users cannot tell what the main content is".into()
            } else {
                "Nutzer wissen nicht, was der Hauptinhalt ist".into()
            },
            if en {
                "Use exactly one H1 heading per page".into()
            } else {
                "Genau eine H1-Überschrift pro Seite verwenden".into()
            },
        ),
        NoPageTitle => (
            if en {
                "No page title detected".into()
            } else {
                "Kein Seitentitel erkannt".into()
            },
            if en {
                "The tab title is empty — users lose context when switching tabs".into()
            } else {
                "Tab-Titel ist leer — Nutzer verlieren den Kontext beim Tabwechsel".into()
            },
            if en {
                "Set a meaningful <title>".into()
            } else {
                "Aussagekräftigen <title> setzen".into()
            },
        ),
        LittleEarlyText => (
            if en {
                "Little visible text in the upper area of the page".into()
            } else {
                "Wenig sichtbarer Text im oberen Seitenbereich".into()
            },
            if en {
                "Users get no immediate orientation".into()
            } else {
                "Nutzer erhalten keine sofortige Orientierung".into()
            },
            if en {
                "Place relevant introductory text above the fold".into()
            } else {
                "Relevanten Einleitungstext above-the-fold platzieren".into()
            },
        ),
        NavigationMissing => (
            if en {
                "Navigation landmark not reachable in the accessibility tree".into()
            } else {
                "Navigation-Landmark nicht im Accessibility-Tree erreichbar".into()
            },
            if en {
                "Screen reader users cannot recognize the main navigation as such or jump to it directly — often caused by aria-hidden on the <nav> element or an ancestor.".into()
            } else {
                "Screenreader-Nutzer können die Hauptnavigation nicht als solche erkennen oder gezielt ansteuern — häufig durch aria-hidden auf dem <nav>-Element oder einem Vorfahren verursacht.".into()
            },
            if en {
                "Check whether aria-hidden=\"true\" is set on or above the <nav> element. The navigation landmark must be visible in the accessibility tree (role=\"navigation\").".into()
            } else {
                "Prüfen, ob aria-hidden=\"true\" auf oder oberhalb des <nav>-Elements gesetzt ist. Das Navigation-Landmark muss im Accessibility-Tree sichtbar sein (role=\"navigation\").".into()
            },
        ),
        MainHiddenFromAx => (
            if en {
                "<main> element present, but not visible in the accessibility tree".into()
            } else {
                "<main>-Element vorhanden, aber im Accessibility-Tree nicht sichtbar".into()
            },
            if en {
                "Screen reader users may not be able to jump to the main content directly on this viewport".into()
            } else {
                "Screenreader-Nutzer können den Hauptinhalt auf diesem Viewport möglicherweise nicht direkt anspringen".into()
            },
            if en {
                "Check whether an overlay or aria-hidden hides the <main> area on this viewport"
                    .into()
            } else {
                "Prüfen, ob ein Overlay oder aria-hidden den <main>-Bereich auf diesem Viewport verbirgt".into()
            },
        ),
        MainMissing => (
            if en {
                "No main content area (<main>) detected".into()
            } else {
                "Kein Hauptinhaltsbereich (<main>) erkannt".into()
            },
            if en {
                "Screen reader users cannot jump to the main content directly".into()
            } else {
                "Screenreader-Nutzer können den Hauptinhalt nicht direkt anspringen".into()
            },
            if en {
                "Wrap the main content in a <main> element".into()
            } else {
                "Hauptinhalt in ein <main>-Element einschließen".into()
            },
        ),
        FewSubheadings => (
            if en {
                "Hardly any subheadings for orientation".into()
            } else {
                "Kaum Zwischenüberschriften zur Orientierung".into()
            },
            if en {
                "Users cannot scan the content".into()
            } else {
                "Nutzer können Inhalte nicht scannen".into()
            },
            if en {
                "Structure the content with H2/H3 headings".into()
            } else {
                "Inhalte mit H2/H3-Überschriften gliedern".into()
            },
        ),
        GenericLinks => (
            if en {
                format!("{} links with generic text (\"more\", \"here\")", count)
            } else {
                format!(
                    "{} Links mit generischen Texten (\"mehr\", \"hier\")",
                    count
                )
            },
            if en {
                "Users cannot distinguish where links lead".into()
            } else {
                "Nutzer können nicht unterscheiden, wohin Links führen".into()
            },
            if en {
                "Write descriptive link text that names the destination".into()
            } else {
                "Linktexte beschreibend formulieren, die das Ziel benennen".into()
            },
        ),
        EmptyLinks => (
            if en {
                format!("{} links without recognizable text", count)
            } else {
                format!("{} Links ohne erkennbaren Text", count)
            },
            if en {
                "Screen reader users do not learn what the link does".into()
            } else {
                "Screenreader-Nutzer erfahren nicht, was der Link tut".into()
            },
            if en {
                "Give every link descriptive text or an aria-label".into()
            } else {
                "Alle Links mit beschreibendem Text oder aria-label versehen".into()
            },
        ),
        DuplicateLinks => (
            if en {
                format!("{} duplicate link texts on the page", count)
            } else {
                format!("{} doppelte Linktexte auf der Seite", count)
            },
            if en {
                "The same label for different destinations confuses users".into()
            } else {
                "Gleiche Beschriftung für unterschiedliche Ziele verwirrt Nutzer".into()
            },
            if en {
                "Make link texts unambiguous or differentiate them with aria-label".into()
            } else {
                "Linktexte eindeutig formulieren oder mit aria-label differenzieren".into()
            },
        ),
        UnnamedButtons => (
            if en {
                format!("{} buttons without a recognizable label", count)
            } else {
                format!("{} Buttons ohne erkennbare Beschriftung", count)
            },
            if en {
                "Users do not know what a button triggers".into()
            } else {
                "Nutzer wissen nicht, was ein Button auslöst".into()
            },
            if en {
                "Give every button descriptive text or an aria-label".into()
            } else {
                "Alle Buttons mit beschreibendem Text oder aria-label versehen".into()
            },
        ),
        UnlabeledForms => (
            if en {
                format!("{} form fields without a label", count)
            } else {
                format!("{} Formularfelder ohne Label", count)
            },
            if en {
                "Users do not know what input is expected".into()
            } else {
                "Nutzer wissen nicht, welche Eingabe erwartet wird".into()
            },
            if en {
                "Connect every form field to a visible <label> or aria-label".into()
            } else {
                "Jedes Formularfeld mit sichtbarem <label> oder aria-label verbinden".into()
            },
        ),
        GenericButtons => (
            if en {
                format!("{} buttons with generic labels (\"OK\", \"Submit\")", count)
            } else {
                format!(
                    "{} Buttons mit generischen Labels (\"OK\", \"Submit\")",
                    count
                )
            },
            if en {
                "The context of the action is unclear".into()
            } else {
                "Kontext der Aktion ist nicht klar".into()
            },
            if en {
                "Label buttons with action-describing text (e.g. \"Send message\")".into()
            } else {
                "Buttons mit handlungsbeschreibenden Texten benennen (z. B. \"Nachricht senden\")"
                    .into()
            },
        ),
        NoCallToAction => (
            if en {
                "No recognizable call-to-action on the page".into()
            } else {
                "Kein erkennbarer Call-to-Action auf der Seite".into()
            },
            if en {
                "Users have no clear prompt to act".into()
            } else {
                "Nutzer haben keine klare Handlungsaufforderung".into()
            },
            if en {
                "Define a primary CTA and place it prominently".into()
            } else {
                "Einen primären CTA definieren und prominent platzieren".into()
            },
        ),
        BlockingDialog => (
            if en {
                "Dialog/overlay detected that can interrupt the user path".into()
            } else {
                "Dialog/Overlay erkannt, der den Nutzerpfad unterbrechen kann".into()
            },
            if en {
                "Cookie banners or modals can obscure the CTA".into()
            } else {
                "Cookie-Banner oder Modals können den CTA verdecken".into()
            },
            if en {
                "Ensure overlays are easy to close and do not block the CTA".into()
            } else {
                "Sicherstellen, dass Overlays einfach schließbar sind und den CTA nicht blockieren"
                    .into()
            },
        ),
        ComplexForm => (
            if en {
                format!("{} form fields — high input barrier", count)
            } else {
                format!("{} Formularfelder — hohe Eingabehürde", count)
            },
            if en {
                "Complex forms reduce the completion rate".into()
            } else {
                "Komplexe Formulare reduzieren die Abschlussrate".into()
            },
            if en {
                "Reduce the form to essential fields or split it into steps".into()
            } else {
                "Formular auf wesentliche Felder reduzieren oder in Schritte aufteilen".into()
            },
        ),
    };

    (problem, impact, recommendation)
}

// ── Analysis entry point ────────────────────────────────────────────

/// Analyze user journey quality from the Accessibility Tree.
/// Runs purely on already-extracted AXTree data — no CDP calls needed.
///
/// Produces canonical-English text in the struct (and thus JSON).
pub fn analyze_journey(tree: &AXTree) -> JourneyAnalysis {
    analyze_journey_inner(tree, false)
}

/// Like [`analyze_journey`] but accepts a DOM-level hint for whether a `<main>`
/// element exists in the HTML source. Pass `true` when a JS `querySelector`
/// confirms `<main>` is present but hidden from the AX tree (e.g. by an overlay
/// on mobile viewport), so the severity is downgraded instead of flagged as
/// missing structure.
///
/// Produces canonical-English text in the struct (and thus JSON).
pub fn analyze_journey_with_dom_check(tree: &AXTree, dom_has_main: bool) -> JourneyAnalysis {
    analyze_journey_inner(tree, dom_has_main)
}

fn analyze_journey_inner(tree: &AXTree, dom_has_main: bool) -> JourneyAnalysis {
    info!("Analyzing user journey...");

    let page_intent = detect_page_intent(tree);
    let (w_entry, w_orient, w_nav, w_interact, w_convert) = page_intent.weights();

    let mut friction_points = Vec::new();

    let entry_clarity = analyze_entry_clarity(tree, &mut friction_points);
    let orientation = analyze_orientation(tree, &mut friction_points, dom_has_main);
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
    use FrictionKind::*;
    let headings = tree.headings();
    let mut penalties = Vec::new();

    // H1 presence and clarity
    let h1s: Vec<_> = headings
        .iter()
        .filter(|h| h.heading_level() == Some(1))
        .collect();

    if h1s.is_empty() {
        penalties.push(45.0);
        friction.push(FrictionPoint::new(NoH1, FrictionValues::default()));
    } else if h1s.len() > 1 {
        penalties.push(15.0);
        friction.push(FrictionPoint::new(
            MultipleH1,
            FrictionValues {
                count: Some(h1s.len() as u32),
            },
        ));
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
        friction.push(FrictionPoint::new(NoPageTitle, FrictionValues::default()));
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
        friction.push(FrictionPoint::new(
            LittleEarlyText,
            FrictionValues::default(),
        ));
    }

    let score = journey_dimension_score(&penalties, 100.0);
    build_dimension(JourneyDimensionKind::EntryClarity, score, 0.20)
}

/// Orientation: Can the user tell where they are and where they can go?
///
/// `dom_has_main`: when the caller has verified via DOM query that a `<main>`
/// element exists in the HTML (but may be hidden from the AX tree by an overlay
/// or `display:none`), set this to `true` to downgrade the finding severity.
fn analyze_orientation(
    tree: &AXTree,
    friction: &mut Vec<FrictionPoint>,
    dom_has_main: bool,
) -> JourneyDimension {
    use FrictionKind::*;
    let mut penalties = Vec::new();

    // Navigation landmark
    let has_nav = tree.iter().any(|n| n.role.as_deref() == Some("navigation"));
    if !has_nav {
        penalties.push(40.0);
        friction.push(FrictionPoint::new(
            NavigationMissing,
            FrictionValues::default(),
        ));
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
        if dom_has_main {
            // <main> exists in HTML but is hidden from the AX tree (overlay / display:none on
            // this viewport). Score impact is small — structure exists, but AT access is impaired.
            penalties.push(8.0);
            friction.push(FrictionPoint::new(
                MainHiddenFromAx,
                FrictionValues::default(),
            ));
        } else {
            penalties.push(20.0);
            friction.push(FrictionPoint::new(MainMissing, FrictionValues::default()));
        }
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
        friction.push(FrictionPoint::new(
            FewSubheadings,
            FrictionValues::default(),
        ));
    }

    let score = journey_dimension_score(&penalties, 100.0);
    build_dimension(JourneyDimensionKind::Orientation, score, 0.20)
}

/// Navigation: Are links clear, non-redundant, and well-structured?
fn analyze_navigation(tree: &AXTree, friction: &mut Vec<FrictionPoint>) -> JourneyDimension {
    use FrictionKind::*;
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
            friction.push(FrictionPoint::new(
                GenericLinks,
                FrictionValues {
                    count: Some(generic_count as u32),
                },
            ));
        }
    }

    // Empty / unnamed links
    let empty_links = links
        .iter()
        .filter(|l| l.name.as_deref().is_none_or(|n| n.trim().is_empty()))
        .count();

    if empty_links > 0 {
        let p = saturating_penalty(empty_links as f64, 35.0, 5.0);
        penalties.push(p);
        if empty_links >= 2 {
            friction.push(FrictionPoint::new(
                EmptyLinks,
                FrictionValues {
                    count: Some(empty_links as u32),
                },
            ));
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
            friction.push(FrictionPoint::new(
                DuplicateLinks,
                FrictionValues {
                    count: Some(duplicate_count as u32),
                },
            ));
        }
    }

    // Link density: too many links for page size
    if links.len() > 60 {
        let excess = (links.len() - 60) as f64;
        let p = saturating_penalty(excess, 15.0, 60.0);
        penalties.push(p);
    }

    let score = journey_dimension_score(&penalties, 100.0);
    build_dimension(JourneyDimensionKind::Navigation, score, 0.25)
}

/// Interaction: Can users interact with controls effectively?
fn analyze_interaction(tree: &AXTree, friction: &mut Vec<FrictionPoint>) -> JourneyDimension {
    use FrictionKind::*;
    let buttons = tree.nodes_with_role("button");
    let form_controls = tree.form_controls();
    let mut penalties = Vec::new();

    // Buttons without accessible names
    let unnamed_buttons = buttons
        .iter()
        .filter(|b| b.name.as_deref().is_none_or(|n| n.trim().is_empty()))
        .count();

    if unnamed_buttons > 0 {
        let p = saturating_penalty(unnamed_buttons as f64, 40.0, 3.0);
        penalties.push(p);
        friction.push(FrictionPoint::new(
            UnnamedButtons,
            FrictionValues {
                count: Some(unnamed_buttons as u32),
            },
        ));
    }

    // Form controls without labels
    let unlabeled_forms = form_controls
        .iter()
        .filter(|fc| fc.name.as_deref().is_none_or(|n| n.trim().is_empty()))
        .count();

    if unlabeled_forms > 0 {
        let p = saturating_penalty(unlabeled_forms as f64, 35.0, 3.0);
        penalties.push(p);
        if unlabeled_forms >= 2 {
            friction.push(FrictionPoint::new(
                UnlabeledForms,
                FrictionValues {
                    count: Some(unlabeled_forms as u32),
                },
            ));
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
                    | "go"
                    | "next"
                    | "click"
                    | "button"
                    | "send"
                    // German
                    | "senden"
                    | "absenden"
                    | "los"
                    | "weiter"
                    | "schicken"
                    // French
                    | "envoyer"
                    | "suivant"
                    | "continuer"
                    | "valider"
                    // Spanish
                    | "enviar"
                    | "siguiente"
                    | "continuar"
                    | "aceptar"
                    // Italian
                    | "invia"
                    | "successivo"
                    | "continua"
                    | "conferma"
                    // Portuguese
                    | "próximo"
                    | "confirmar"
                    // Dutch
                    | "verzenden"
                    | "volgende"
                    | "doorgaan"
                    | "bevestigen"
                    // Swedish
                    | "skicka"
                    | "nästa"
                    | "fortsätt"
                    // Polish
                    | "wyślij"
                    | "następny"
                    | "kontynuuj"
                    // Turkish
                    | "gönder"
                    | "ileri"
                    | "devam"
            )
        })
        .count();

    if generic_buttons >= 2 {
        let p = saturating_penalty(generic_buttons as f64, 20.0, 5.0);
        penalties.push(p);
        friction.push(FrictionPoint::new(
            GenericButtons,
            FrictionValues {
                count: Some(generic_buttons as u32),
            },
        ));
    }

    let score = journey_dimension_score(&penalties, 100.0);
    build_dimension(JourneyDimensionKind::Interaction, score, 0.20)
}

/// Conversion: Can the user reach and complete the page's primary goal?
fn analyze_conversion(tree: &AXTree, friction: &mut Vec<FrictionPoint>) -> JourneyDimension {
    use FrictionKind::*;
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
        friction.push(FrictionPoint::new(
            NoCallToAction,
            FrictionValues::default(),
        ));
    }

    // Are there interactive blockers before CTA? (modals, cookie banners hinted by roles)
    let has_dialog = tree
        .iter()
        .any(|n| matches!(n.role.as_deref(), Some("dialog" | "alertdialog")));
    if has_dialog {
        penalties.push(15.0);
        friction.push(FrictionPoint::new(
            BlockingDialog,
            FrictionValues::default(),
        ));
    }

    // Form complexity: too many fields can reduce conversion
    let form_controls = tree.form_controls();
    if form_controls.len() > 10 {
        let excess = (form_controls.len() - 10) as f64;
        let p = saturating_penalty(excess, 20.0, 10.0);
        penalties.push(p);
        if form_controls.len() > 15 {
            friction.push(FrictionPoint::new(
                ComplexForm,
                FrictionValues {
                    count: Some(form_controls.len() as u32),
                },
            ));
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
    build_dimension(JourneyDimensionKind::Conversion, score, 0.15)
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Build a dimension, baking canonical-English `name`/`summary`.
fn build_dimension(kind: JourneyDimensionKind, score: u32, weight: f64) -> JourneyDimension {
    JourneyDimension {
        kind,
        name: journey_dimension_name(kind, true).to_string(),
        score,
        weight,
        summary: journey_dimension_summary(kind, score, true),
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

    /// Guard against German leaking into the canonical struct/JSON (#406): the
    /// analysis bakes English, so no visible result text (dimension summaries,
    /// friction problems/impacts/recommendations) may contain German umlauts/ß.
    #[test]
    fn canonical_struct_has_no_german_chars() {
        // Sparse tree: missing H1, no navigation, no <main>, no CTA, plus
        // unnamed links and buttons → triggers detectors across all dimensions.
        let mut nodes = vec![node("root", "RootWebArea", Some("Page"))];
        for i in 0..4 {
            nodes.push(node(&format!("link{i}"), "link", None));
        }
        for i in 0..3 {
            nodes.push(node(&format!("btn{i}"), "button", None));
        }
        let tree = AXTree::from_nodes(nodes);

        let analysis = analyze_journey(&tree);
        assert!(
            !analysis.friction_points.is_empty(),
            "scenario should produce friction points"
        );

        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));

        for dim in [
            &analysis.entry_clarity,
            &analysis.orientation,
            &analysis.navigation,
            &analysis.interaction,
            &analysis.conversion,
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

        for fp in &analysis.friction_points {
            assert!(
                !has_umlaut(&fp.problem),
                "canonical friction problem contains German umlaut: {}",
                fp.problem
            );
            assert!(
                !has_umlaut(&fp.impact),
                "canonical friction impact contains German umlaut: {}",
                fp.impact
            );
            assert!(
                !has_umlaut(&fp.recommendation),
                "canonical friction recommendation contains German umlaut: {}",
                fp.recommendation
            );
        }
    }

    /// PDF re-derivation: the pure text functions must yield real German.
    #[test]
    fn friction_text_german_for_pdf_derivation() {
        let (problem, impact, recommendation) = journey_friction_text(
            FrictionKind::NoH1,
            &FrictionValues::default(),
            false, // German
        );
        assert!(problem.contains("H1-Überschrift"));
        assert!(impact.contains("Nutzer"));
        assert!(recommendation.contains("aussagekräftige"));

        // A count-interpolated variant localizes too.
        let (problem, _, _) = journey_friction_text(
            FrictionKind::EmptyLinks,
            &FrictionValues { count: Some(3) },
            false,
        );
        assert_eq!(problem, "3 Links ohne erkennbaren Text");

        // Dimension summary localizes.
        assert_eq!(
            journey_dimension_summary(JourneyDimensionKind::Conversion, 95, false),
            "Klarer Conversion-Pfad mit erreichbarem CTA"
        );
    }
}
