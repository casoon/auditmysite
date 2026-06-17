//! Source Quality Analysis
//!
//! Interprets data from existing audit modules (Accessibility, SEO, Security,
//! UX) to assess the quality of a website **as an information source**.
//!
//! Three dimensions are scored:
//! - **Substance** — Does the site treat its content as valuable?
//! - **Consistency** — Does the site maintain its own standards?
//! - **Authority** — Does the site present itself as a trustworthy source?
//!
//! **Disclaimer**: This is a purely technical assessment based on structural,
//! semantic, and metadata signals. It does NOT evaluate whether the content
//! itself is factually correct, complete, or up to date.
//!
//! ## Localization (#406)
//!
//! Analysis bakes **canonical English** text into the struct (and thus JSON):
//! every `name`/`detail`/`label`/`disclaimer` is produced with `en = true`.
//! Each signal additionally carries a stable [`QualitySignalKind`] plus the raw
//! interpolated values, so the PDF layer can re-derive localized text via
//! [`source_quality_signal_text`] / [`source_quality_dimension_label`] /
//! [`source_quality_disclaimer`] in the run language.

pub mod module;
pub use module::SourceQualityModule;

use serde::{Deserialize, Serialize};

use crate::audit::AuditReport;
use crate::seo::schema::SchemaType;
use crate::taxonomy::module_score_grade;

// ─── Public types ────────────────────────────────────────────────────────────

/// Complete source quality analysis for a single page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceQualityAnalysis {
    /// Overall source quality score (0–100)
    pub score: u32,
    /// Letter grade (A–F)
    pub grade: String,
    /// Substance dimension
    pub substance: DimensionScore,
    /// Consistency dimension (limited for single page, full in batch)
    pub consistency: DimensionScore,
    /// Authority dimension
    pub authority: DimensionScore,
    /// Always-present disclaimer (canonical English)
    pub disclaimer: String,
}

/// Stable identifier for which dimension a [`DimensionScore`] represents.
///
/// Lets the PDF layer re-derive a localized dimension name without parsing the
/// canonical-English `name` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionKind {
    Substance,
    Consistency,
    Authority,
}

/// Score for a single quality dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// Which dimension this is (for localized re-derivation)
    pub kind: DimensionKind,
    /// Dimension name (canonical English)
    pub name: String,
    /// Score (0–100)
    pub score: u32,
    /// Short assessment (canonical English; derived from `score`)
    pub label: String,
    /// Individual signals evaluated
    pub signals: Vec<QualitySignal>,
}

/// Stable identifier for a concrete quality signal.
///
/// One variant per distinct signal text/detail shape. Together with the raw
/// values stored on [`QualitySignal`] this fully reproduces the human-readable
/// `name`/`detail` strings in any language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualitySignalKind {
    // Substance
    HeadingStructure,
    ContentVolume,
    StructuredData,
    MetaDescription,
    LanguageDeclaration,
    ImageDescriptions,
    SemanticStructure,
    // Authority
    Https,
    SecurityHeaders,
    PublisherIdentity,
    CanonicalUrl,
    SocialMeta,
    Accessibility,
    TrustSignals,
    // Consistency (single page)
    HeadingHierarchy,
    NamedControls,
    NoCriticalErrors,
    LanguageConsistency,
    // Consistency (batch / cross-page)
    ScoreStability,
    MetaDescriptionCoverage,
    StructuredDataCoverage,
    LanguageDeclarationCoverage,
    HstsCoverage,
    ErrorFreePages,
}

/// The interpolated values a signal text may reference.
///
/// Stored on every [`QualitySignal`] alongside `present` so that
/// [`source_quality_signal_text`] can reproduce the detail string for any
/// locale. Only the fields relevant to the signal's `kind` are populated.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalValues {
    /// Heading depth (HeadingStructure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    /// Whether an H1 is present (HeadingStructure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_h1: Option<bool>,
    /// Word count (ContentVolume)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<u32>,
    /// Detected Schema.org types (StructuredData, present case)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub schema_types: Vec<String>,
    /// Declared language code (LanguageDeclaration, present case)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// A generic count (ImageDescriptions, SemanticStructure, HeadingHierarchy,
    /// NamedControls, NoCriticalErrors, SecurityHeaders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Whether <main> is missing in the AX tree (SemanticStructure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_main: Option<bool>,
    /// An accessibility/UX score (Accessibility, TrustSignals)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    /// A cross-page coverage percentage (*Coverage, ErrorFreePages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent: Option<u32>,
    /// Cross-page standard deviation (ScoreStability)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub std_dev: Option<f32>,
}

/// A single measurable signal contributing to a dimension score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySignal {
    /// Stable signal identifier (for localized re-derivation)
    pub kind: QualitySignalKind,
    /// What was checked (canonical English)
    pub name: String,
    /// Whether the signal is positive
    pub present: bool,
    /// Weight of this signal within its dimension (0.0–1.0)
    pub weight: f32,
    /// Human-readable detail (canonical English)
    pub detail: String,
    /// Interpolated values needed to reproduce `detail` in another language
    #[serde(default)]
    pub values: SignalValues,
}

impl QualitySignal {
    /// Build a signal, baking canonical-English `name`/`detail` from its kind +
    /// values via [`source_quality_signal_text`].
    fn new(kind: QualitySignalKind, present: bool, weight: f32, values: SignalValues) -> Self {
        let (name, detail) = source_quality_signal_text(kind, present, &values, true);
        QualitySignal {
            kind,
            name,
            present,
            weight,
            detail,
            values,
        }
    }
}

// ─── Localized text (single source of truth) ─────────────────────────────────

const DISCLAIMER_DE: &str = "Diese Bewertung basiert ausschließlich auf technischen Signalen \
    (Struktur, Semantik, Metadaten, Sicherheit). Sie beurteilt nicht, ob die \
    dargestellten Inhalte inhaltlich korrekt, vollständig oder aktuell sind.";

const DISCLAIMER_EN: &str = "This assessment is based solely on technical signals \
    (structure, semantics, metadata, security). It does not judge whether the \
    presented content is factually correct, complete or up to date.";

/// The always-present disclaimer in the requested language.
pub fn source_quality_disclaimer(en: bool) -> String {
    if en { DISCLAIMER_EN } else { DISCLAIMER_DE }.to_string()
}

/// Localized dimension name for a [`DimensionKind`].
pub fn source_quality_dimension_name(kind: DimensionKind, en: bool) -> &'static str {
    match (kind, en) {
        (DimensionKind::Substance, true) => "Substance",
        (DimensionKind::Substance, false) => "Substanz",
        (DimensionKind::Consistency, true) => "Consistency",
        (DimensionKind::Consistency, false) => "Konsistenz",
        (DimensionKind::Authority, true) => "Authority",
        (DimensionKind::Authority, false) => "Autorität",
    }
}

/// Localized "no data" label used when a dimension has no signals.
pub fn source_quality_no_data_label(en: bool) -> String {
    if en { "No data" } else { "Keine Daten" }.to_string()
}

/// Localized band label for a dimension score (single source of truth).
pub fn source_quality_dimension_label(score: u32, en: bool) -> String {
    if en {
        match score {
            90..=100 => "Excellent",
            75..=89 => "Good",
            60..=74 => "Needs improvement",
            40..=59 => "Inadequate",
            _ => "Critical",
        }
    } else {
        match score {
            90..=100 => "Sehr gut",
            75..=89 => "Gut",
            60..=74 => "Verbesserungswürdig",
            40..=59 => "Ausbaufähig",
            _ => "Kritisch",
        }
    }
    .to_string()
}

/// The single source of truth for signal `name`/`detail` text.
///
/// Returns `(name, detail)` in German or English for the given `kind`,
/// `present` flag, and interpolated `values`. Analysis calls it with
/// `en = true` to bake canonical English; the PDF layer calls it with the run
/// language to re-derive localized text.
pub fn source_quality_signal_text(
    kind: QualitySignalKind,
    present: bool,
    values: &SignalValues,
    en: bool,
) -> (String, String) {
    use QualitySignalKind::*;

    let name: String = match (kind, en) {
        (HeadingStructure, true) => "Heading structure".into(),
        (HeadingStructure, false) => "Überschriftenstruktur".into(),
        (ContentVolume, true) => "Content volume".into(),
        (ContentVolume, false) => "Inhaltsumfang".into(),
        (StructuredData, true) | (StructuredDataCoverage, true) => "Structured data".into(),
        (StructuredData, false) | (StructuredDataCoverage, false) => "Strukturierte Daten".into(),
        (MetaDescription, true) => "Meta description".into(),
        (MetaDescription, false) => "Meta-Beschreibung".into(),
        (LanguageDeclaration, true) | (LanguageDeclarationCoverage, true) => {
            "Language declaration".into()
        }
        (LanguageDeclaration, false) | (LanguageDeclarationCoverage, false) => {
            "Sprachdeklaration".into()
        }
        (ImageDescriptions, true) => "Image descriptions".into(),
        (ImageDescriptions, false) => "Bildbeschreibungen".into(),
        (SemanticStructure, true) => "Semantic structure".into(),
        (SemanticStructure, false) => "Semantische Struktur".into(),
        (Https, _) => "HTTPS".into(),
        (SecurityHeaders, true) => "Security headers".into(),
        (SecurityHeaders, false) => "Sicherheits-Header".into(),
        (PublisherIdentity, true) => "Publisher identity".into(),
        (PublisherIdentity, false) => "Herausgeber-Identität".into(),
        (CanonicalUrl, _) => "Canonical URL".into(),
        (SocialMeta, true) => "Social meta".into(),
        (SocialMeta, false) => "Social-Meta".into(),
        (Accessibility, true) => "Accessibility".into(),
        (Accessibility, false) => "Barrierefreiheit".into(),
        (TrustSignals, true) => "Trust signals".into(),
        (TrustSignals, false) => "Vertrauenssignale".into(),
        (HeadingHierarchy, true) => "Heading hierarchy".into(),
        (HeadingHierarchy, false) => "Überschriften-Hierarchie".into(),
        (NamedControls, true) => "Named controls".into(),
        (NamedControls, false) => "Benannte Bedienelemente".into(),
        (NoCriticalErrors, true) => "No critical errors".into(),
        (NoCriticalErrors, false) => "Keine kritischen Fehler".into(),
        (LanguageConsistency, true) => "Language consistency".into(),
        (LanguageConsistency, false) => "Sprachkonsistenz".into(),
        (ScoreStability, true) => "Score stability".into(),
        (ScoreStability, false) => "Score-Stabilität".into(),
        (MetaDescriptionCoverage, true) => "Meta descriptions".into(),
        (MetaDescriptionCoverage, false) => "Meta-Beschreibungen".into(),
        (HstsCoverage, true) => "HSTS coverage".into(),
        (HstsCoverage, false) => "HSTS-Abdeckung".into(),
        (ErrorFreePages, true) => "Error-free pages".into(),
        (ErrorFreePages, false) => "Fehlerfreie Seiten".into(),
    };

    let count = values.count.unwrap_or(0);
    let percent = values.percent.unwrap_or(0);

    let detail: String = match kind {
        HeadingStructure => {
            let depth = values.depth.unwrap_or(0);
            let has_h1 = values.has_h1.unwrap_or(false);
            if has_h1 && depth >= 3 {
                if en {
                    format!("Structured outline down to H{}", depth)
                } else {
                    format!("Strukturierte Gliederung bis H{}", depth)
                }
            } else if !has_h1 {
                if en {
                    "No H1 heading present".into()
                } else {
                    "Keine H1-Überschrift vorhanden".into()
                }
            } else if en {
                format!("Flat outline (only down to H{})", depth)
            } else {
                format!("Flache Gliederung (nur bis H{})", depth)
            }
        }
        ContentVolume => {
            let word_count = values.word_count.unwrap_or(0);
            if en {
                format!(
                    "{} words{}",
                    word_count,
                    if present {
                        ""
                    } else {
                        " (heuristic: typically ≥ 300 words recommended)"
                    }
                )
            } else {
                format!(
                    "{} Wörter{}",
                    word_count,
                    if present {
                        ""
                    } else {
                        " (Heuristik: typisch ≥ 300 Wörter empfohlen)"
                    }
                )
            }
        }
        StructuredData => {
            if present {
                format!("Schema.org: {}", values.schema_types.join(", "))
            } else if en {
                "No structured data".into()
            } else {
                "Keine strukturierten Daten".into()
            }
        }
        MetaDescription => {
            if present {
                if en {
                    "Meaningful meta description present".into()
                } else {
                    "Aussagekräftige Meta-Beschreibung vorhanden".into()
                }
            } else if en {
                "Missing or too short meta description".into()
            } else {
                "Keine oder zu kurze Meta-Beschreibung".into()
            }
        }
        LanguageDeclaration => {
            if present {
                let lang = values.lang.as_deref().unwrap_or("?");
                if en {
                    format!("Language declared: {}", lang)
                } else {
                    format!("Sprache deklariert: {}", lang)
                }
            } else if en {
                "No language declaration".into()
            } else {
                "Keine Sprachdeklaration".into()
            }
        }
        ImageDescriptions => {
            if present {
                if en {
                    "All images have alternative text".into()
                } else {
                    "Alle Bilder haben Alternativtexte".into()
                }
            } else if en {
                format!("{} images without alternative text", count)
            } else {
                format!("{} Bilder ohne Alternativtext", count)
            }
        }
        SemanticStructure => {
            let missing_main = values.missing_main.unwrap_or(false);
            if present {
                if en {
                    "Correct landmark regions".into()
                } else {
                    "Korrekte Landmark-Regionen".into()
                }
            } else if missing_main {
                if en {
                    "<main> landmark not detectable in the accessibility tree".into()
                } else {
                    "<main>-Landmark im Accessibility Tree nicht nachweisbar".into()
                }
            } else if en {
                format!("{} structural issues", count)
            } else {
                format!("{} Strukturprobleme", count)
            }
        }
        Https => {
            if present {
                if en {
                    "Encrypted connection".into()
                } else {
                    "Verschlüsselte Verbindung".into()
                }
            } else if en {
                "No HTTPS encryption".into()
            } else {
                "Keine HTTPS-Verschlüsselung".into()
            }
        }
        SecurityHeaders => {
            if en {
                format!("{}/4 relevant security headers set", count)
            } else {
                format!("{}/4 relevante Security-Header gesetzt", count)
            }
        }
        PublisherIdentity => {
            if present {
                if en {
                    "Organization/publisher identified via Schema.org".into()
                } else {
                    "Organisation/Herausgeber per Schema.org identifiziert".into()
                }
            } else if en {
                "No publisher markup".into()
            } else {
                "Kein Herausgeber-Markup".into()
            }
        }
        CanonicalUrl => {
            if present {
                if en {
                    "Canonical URL declared".into()
                } else {
                    "Kanonische URL deklariert".into()
                }
            } else if en {
                "No canonical URL".into()
            } else {
                "Keine Canonical-URL".into()
            }
        }
        SocialMeta => {
            if present {
                if en {
                    "Open Graph metadata present".into()
                } else {
                    "Open Graph Metadaten vorhanden".into()
                }
            } else if en {
                "Incomplete social metadata".into()
            } else {
                "Unvollständige Social-Metadaten".into()
            }
        }
        Accessibility => {
            let score = values.score.unwrap_or(0.0);
            if en {
                format!(
                    "Accessibility score: {:.0}{}",
                    score,
                    if present { "" } else { " (low)" }
                )
            } else {
                format!(
                    "Accessibility-Score: {:.0}{}",
                    score,
                    if present { "" } else { " (niedrig)" }
                )
            }
        }
        TrustSignals => {
            let score = values.score.unwrap_or(0.0);
            if en {
                format!(
                    "UX trust score: {:.0}{}",
                    score,
                    if present { "" } else { " (weak)" }
                )
            } else {
                format!(
                    "UX Trust-Score: {:.0}{}",
                    score,
                    if present { "" } else { " (schwach)" }
                )
            }
        }
        HeadingHierarchy => {
            if present {
                if en {
                    "Gapless heading hierarchy".into()
                } else {
                    "Lückenlose Überschriften-Hierarchie".into()
                }
            } else if en {
                format!("{} hierarchy issues", count)
            } else {
                format!("{} Hierarchie-Probleme", count)
            }
        }
        NamedControls => {
            if present {
                if en {
                    "All interactive elements correctly named".into()
                } else {
                    "Alle interaktiven Elemente korrekt benannt".into()
                }
            } else if en {
                format!("{} elements without an accessible name", count)
            } else {
                format!("{} Elemente ohne zugänglichen Namen", count)
            }
        }
        NoCriticalErrors => {
            if present {
                if en {
                    "No critical accessibility violations".into()
                } else {
                    "Keine kritischen Accessibility-Verstöße".into()
                }
            } else if en {
                format!("{} critical violations", count)
            } else {
                format!("{} kritische Verstöße", count)
            }
        }
        LanguageConsistency => {
            if present {
                if en {
                    "Language correctly declared".into()
                } else {
                    "Sprache korrekt deklariert".into()
                }
            } else if en {
                "Missing language declaration".into()
            } else {
                "Fehlende Sprachdeklaration".into()
            }
        }
        ScoreStability => {
            let std_dev = values.std_dev.unwrap_or(0.0);
            if en {
                format!(
                    "Standard deviation: {:.1}{}",
                    std_dev,
                    if present {
                        " (stable)"
                    } else {
                        " (inconsistent)"
                    }
                )
            } else {
                format!(
                    "Standardabweichung: {:.1}{}",
                    std_dev,
                    if present {
                        " (stabil)"
                    } else {
                        " (inkonsistent)"
                    }
                )
            }
        }
        MetaDescriptionCoverage => {
            if en {
                format!("{}% of pages with a meta description", percent)
            } else {
                format!("{}% der Seiten mit Meta-Beschreibung", percent)
            }
        }
        StructuredDataCoverage => {
            if en {
                format!("{}% of pages with Schema.org", percent)
            } else {
                format!("{}% der Seiten mit Schema.org", percent)
            }
        }
        LanguageDeclarationCoverage => {
            if en {
                format!("{}% of pages with a language declaration", percent)
            } else {
                format!("{}% der Seiten mit Sprachdeklaration", percent)
            }
        }
        HstsCoverage => {
            if en {
                format!("{}% of pages with HSTS", percent)
            } else {
                format!("{}% der Seiten mit HSTS", percent)
            }
        }
        ErrorFreePages => {
            if en {
                format!("{}% of pages without critical errors", percent)
            } else {
                format!("{}% der Seiten ohne kritische Fehler", percent)
            }
        }
    };

    (name, detail)
}

// ─── Analysis entry point ────────────────────────────────────────────────────

/// Derive source quality from an existing audit report (single page).
///
/// Produces canonical-English text in the struct (and thus JSON).
pub fn analyze_source_quality(report: &AuditReport) -> SourceQualityAnalysis {
    let substance = evaluate_substance(report);
    let consistency = evaluate_single_page_consistency(report);
    let authority = evaluate_authority(report);

    let score = weighted_average(&[
        (substance.score, 40),
        (consistency.score, 25),
        (authority.score, 35),
    ]);

    SourceQualityAnalysis {
        score,
        grade: module_score_grade(score).to_string(),
        substance,
        consistency,
        authority,
        disclaimer: source_quality_disclaimer(true),
    }
}

/// Derive source quality for batch mode with cross-page consistency.
///
/// Produces canonical-English text in the struct (and thus JSON).
pub fn analyze_source_quality_batch(reports: &[AuditReport]) -> SourceQualityAnalysis {
    if reports.is_empty() {
        return empty_analysis();
    }

    // Average substance and authority across pages
    let substance_scores: Vec<DimensionScore> = reports.iter().map(evaluate_substance).collect();
    let authority_scores: Vec<DimensionScore> = reports.iter().map(evaluate_authority).collect();

    let avg_substance = average_dimensions(&substance_scores, DimensionKind::Substance);
    let avg_authority = average_dimensions(&authority_scores, DimensionKind::Authority);

    // Cross-page consistency (the real batch value)
    let consistency = evaluate_cross_page_consistency(reports);

    let score = weighted_average(&[
        (avg_substance.score, 35),
        (consistency.score, 30),
        (avg_authority.score, 35),
    ]);

    SourceQualityAnalysis {
        score,
        grade: module_score_grade(score).to_string(),
        substance: avg_substance,
        consistency,
        authority: avg_authority,
        disclaimer: source_quality_disclaimer(true),
    }
}

// ─── Substance ───────────────────────────────────────────────────────────────

fn evaluate_substance(report: &AuditReport) -> DimensionScore {
    use QualitySignalKind::*;
    let mut signals = Vec::new();

    // 1. Heading structure depth
    if let Some(seo) = &report.seo {
        let has_h1 = seo.headings.h1_count > 0;
        let depth = seo
            .headings
            .headings
            .iter()
            .map(|h| h.level)
            .max()
            .unwrap_or(0);
        let good_depth = depth >= 3;

        signals.push(QualitySignal::new(
            HeadingStructure,
            has_h1 && good_depth,
            0.20,
            SignalValues {
                depth: Some(depth as u32),
                has_h1: Some(has_h1),
                ..Default::default()
            },
        ));

        // 2. Word count / content density
        let word_count = seo.technical.word_count;
        signals.push(QualitySignal::new(
            ContentVolume,
            word_count >= 300,
            0.15,
            SignalValues {
                word_count: Some(word_count),
                ..Default::default()
            },
        ));

        // 3. Schema.org structured data
        let has_schema = seo.structured_data.has_structured_data;
        let schema_types: Vec<String> = seo
            .structured_data
            .types
            .iter()
            .map(|t| t.as_str().to_string())
            .collect();
        signals.push(QualitySignal::new(
            StructuredData,
            has_schema,
            0.20,
            SignalValues {
                schema_types,
                ..Default::default()
            },
        ));

        // 4. Meta description
        let has_meta_desc = seo.meta.description.as_ref().is_some_and(|d| d.len() >= 50);
        signals.push(QualitySignal::new(
            MetaDescription,
            has_meta_desc,
            0.10,
            SignalValues::default(),
        ));

        // 5. Language declaration
        let has_lang = seo.technical.has_lang;
        signals.push(QualitySignal::new(
            LanguageDeclaration,
            has_lang,
            0.10,
            SignalValues {
                lang: seo.technical.lang.clone(),
                ..Default::default()
            },
        ));
    }

    // 6. Accessibility — image alt text coverage
    let image_violations = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule == "1.1.1")
        .count();
    signals.push(QualitySignal::new(
        ImageDescriptions,
        image_violations == 0,
        0.15,
        SignalValues {
            count: Some(image_violations as u32),
            ..Default::default()
        },
    ));

    // 7. Landmark structure
    let landmark_violations = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule_name.contains("Landmark"))
        .count();
    let missing_main_in_ax = report
        .interactive_findings
        .iter()
        .any(|f| f.category == "Landmark" && f.message.contains("Kein <main>-Landmark"));
    let good_landmarks = landmark_violations == 0 && !missing_main_in_ax;
    signals.push(QualitySignal::new(
        SemanticStructure,
        good_landmarks,
        0.10,
        SignalValues {
            count: Some(landmark_violations as u32),
            missing_main: Some(missing_main_in_ax),
            ..Default::default()
        },
    ));

    build_dimension(DimensionKind::Substance, signals)
}

// ─── Authority ───────────────────────────────────────────────────────────────

fn evaluate_authority(report: &AuditReport) -> DimensionScore {
    use QualitySignalKind::*;
    let mut signals = Vec::new();

    // 1. HTTPS
    let has_https = report.url.starts_with("https://");
    signals.push(QualitySignal::new(
        Https,
        has_https,
        0.15,
        SignalValues::default(),
    ));

    // 2. Security headers
    if let Some(sec) = &report.security {
        let header_count = [
            sec.headers.strict_transport_security.is_some(),
            sec.headers.content_security_policy.is_some(),
            sec.headers.x_content_type_options.is_some(),
            sec.headers.referrer_policy.is_some(),
        ]
        .iter()
        .filter(|&&b| b)
        .count();

        signals.push(QualitySignal::new(
            SecurityHeaders,
            header_count >= 3,
            0.15,
            SignalValues {
                count: Some(header_count as u32),
                ..Default::default()
            },
        ));
    }

    // 3. Schema.org Organization / Author
    if let Some(seo) = &report.seo {
        let has_org = seo.structured_data.types.iter().any(|t| {
            t.is_organization_like() || matches!(t, SchemaType::Person | SchemaType::WebSite)
        });
        signals.push(QualitySignal::new(
            PublisherIdentity,
            has_org,
            0.20,
            SignalValues::default(),
        ));

        // 4. Canonical URL
        let has_canonical = seo.technical.has_canonical;
        signals.push(QualitySignal::new(
            CanonicalUrl,
            has_canonical,
            0.10,
            SignalValues::default(),
        ));

        // 5. Social meta / Open Graph
        let has_og = seo
            .social
            .open_graph
            .as_ref()
            .is_some_and(|og| og.title.is_some() && og.description.is_some());
        signals.push(QualitySignal::new(
            SocialMeta,
            has_og,
            0.10,
            SignalValues::default(),
        ));
    }

    // 6. Accessibility score as quality signal
    let a11y_good = report.score >= 80.0;
    signals.push(QualitySignal::new(
        Accessibility,
        a11y_good,
        0.15,
        SignalValues {
            score: Some(report.score),
            ..Default::default()
        },
    ));

    // 7. Trust signals from UX module
    if let Some(ux) = &report.ux {
        let trust_good = ux.trust_signals.score >= 70;
        signals.push(QualitySignal::new(
            TrustSignals,
            trust_good,
            0.15,
            SignalValues {
                score: Some(ux.trust_signals.score as f32),
                ..Default::default()
            },
        ));
    }

    build_dimension(DimensionKind::Authority, signals)
}

// ─── Consistency (single page) ───────────────────────────────────────────────

fn evaluate_single_page_consistency(report: &AuditReport) -> DimensionScore {
    use QualitySignalKind::*;
    let mut signals = Vec::new();

    // 1. Heading hierarchy (no skips)
    if let Some(seo) = &report.seo {
        let issue_count = seo.headings.issues.len();
        signals.push(QualitySignal::new(
            HeadingHierarchy,
            issue_count == 0,
            0.25,
            SignalValues {
                count: Some(issue_count as u32),
                ..Default::default()
            },
        ));
    }

    // 2. All interactive elements named
    let unnamed_interactive = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule == "4.1.2" || v.rule == "1.1.1")
        .count();
    signals.push(QualitySignal::new(
        NamedControls,
        unnamed_interactive == 0,
        0.25,
        SignalValues {
            count: Some(unnamed_interactive as u32),
            ..Default::default()
        },
    ));

    // 3. No critical WCAG violations
    let critical = report.statistics.critical;
    signals.push(QualitySignal::new(
        NoCriticalErrors,
        critical == 0,
        0.25,
        SignalValues {
            count: Some(critical as u32),
            ..Default::default()
        },
    ));

    // 4. Language consistency
    if let Some(seo) = &report.seo {
        let has_lang = seo.technical.has_lang;
        signals.push(QualitySignal::new(
            LanguageConsistency,
            has_lang,
            0.25,
            SignalValues::default(),
        ));
    }

    build_dimension(DimensionKind::Consistency, signals)
}

// ─── Consistency (batch / cross-page) ────────────────────────────────────────

fn evaluate_cross_page_consistency(reports: &[AuditReport]) -> DimensionScore {
    use QualitySignalKind::*;
    let total = reports.len() as f32;
    let mut signals = Vec::new();

    // 1. Score stability (low standard deviation = consistent)
    let scores: Vec<f32> = reports.iter().map(|r| r.score).collect();
    let mean = scores.iter().sum::<f32>() / total;
    let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f32>() / total;
    let std_dev = variance.sqrt();
    signals.push(QualitySignal::new(
        ScoreStability,
        std_dev < 15.0,
        0.20,
        SignalValues {
            std_dev: Some(std_dev),
            ..Default::default()
        },
    ));

    // 2. Meta description coverage
    let with_meta: usize = reports
        .iter()
        .filter(|r| {
            r.seo
                .as_ref()
                .and_then(|s| s.meta.description.as_ref())
                .is_some_and(|d| d.len() >= 50)
        })
        .count();
    let meta_pct = (with_meta as f32 / total * 100.0) as u32;
    signals.push(QualitySignal::new(
        MetaDescriptionCoverage,
        meta_pct >= 90,
        0.15,
        SignalValues {
            percent: Some(meta_pct),
            ..Default::default()
        },
    ));

    // 3. Schema.org coverage
    let with_schema: usize = reports
        .iter()
        .filter(|r| {
            r.seo
                .as_ref()
                .is_some_and(|s| s.structured_data.has_structured_data)
        })
        .count();
    let schema_pct = (with_schema as f32 / total * 100.0) as u32;
    signals.push(QualitySignal::new(
        StructuredDataCoverage,
        schema_pct >= 80,
        0.15,
        SignalValues {
            percent: Some(schema_pct),
            ..Default::default()
        },
    ));

    // 4. Language declaration coverage
    let with_lang: usize = reports
        .iter()
        .filter(|r| r.seo.as_ref().is_some_and(|s| s.technical.has_lang))
        .count();
    let lang_pct = (with_lang as f32 / total * 100.0) as u32;
    signals.push(QualitySignal::new(
        LanguageDeclarationCoverage,
        lang_pct >= 95,
        0.15,
        SignalValues {
            percent: Some(lang_pct),
            ..Default::default()
        },
    ));

    // 5. Security header consistency
    let with_hsts: usize = reports
        .iter()
        .filter(|r| {
            r.security
                .as_ref()
                .is_some_and(|s| s.headers.strict_transport_security.is_some())
        })
        .count();
    let hsts_pct = (with_hsts as f32 / total * 100.0) as u32;
    signals.push(QualitySignal::new(
        HstsCoverage,
        hsts_pct >= 95,
        0.15,
        SignalValues {
            percent: Some(hsts_pct),
            ..Default::default()
        },
    ));

    // 6. No pages with critical violations
    let pages_with_critical: usize = reports.iter().filter(|r| r.statistics.critical > 0).count();
    let clean_pct = ((total as usize - pages_with_critical) as f32 / total * 100.0) as u32;
    signals.push(QualitySignal::new(
        ErrorFreePages,
        pages_with_critical == 0,
        0.20,
        SignalValues {
            percent: Some(clean_pct),
            ..Default::default()
        },
    ));

    build_dimension(DimensionKind::Consistency, signals)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn build_dimension(kind: DimensionKind, signals: Vec<QualitySignal>) -> DimensionScore {
    if signals.is_empty() {
        return DimensionScore {
            kind,
            name: source_quality_dimension_name(kind, true).to_string(),
            score: 0,
            label: source_quality_no_data_label(true),
            signals: vec![],
        };
    }

    // Normalize weights to sum to 1.0
    let total_weight: f32 = signals.iter().map(|s| s.weight).sum();
    let score = if total_weight > 0.0 {
        let raw: f32 = signals
            .iter()
            .map(|s| {
                if s.present {
                    s.weight / total_weight * 100.0
                } else {
                    0.0
                }
            })
            .sum();
        raw.round() as u32
    } else {
        0
    };

    DimensionScore {
        kind,
        name: source_quality_dimension_name(kind, true).to_string(),
        score,
        label: source_quality_dimension_label(score, true),
        signals,
    }
}

fn average_dimensions(dims: &[DimensionScore], kind: DimensionKind) -> DimensionScore {
    if dims.is_empty() {
        return DimensionScore {
            kind,
            name: source_quality_dimension_name(kind, true).to_string(),
            score: 0,
            label: source_quality_no_data_label(true),
            signals: vec![],
        };
    }

    let avg = dims.iter().map(|d| d.score).sum::<u32>() / dims.len() as u32;

    // Merge signals: take the first report's signals as template, show coverage
    let signals = dims.first().map(|d| d.signals.clone()).unwrap_or_default();

    DimensionScore {
        kind,
        name: source_quality_dimension_name(kind, true).to_string(),
        score: avg,
        label: source_quality_dimension_label(avg, true),
        signals,
    }
}

fn weighted_average(items: &[(u32, u32)]) -> u32 {
    let total_weight: u32 = items.iter().map(|(_, w)| w).sum();
    if total_weight == 0 {
        return 0;
    }
    let sum: u32 = items.iter().map(|(s, w)| s * w).sum();
    (sum as f64 / total_weight as f64).round() as u32
}

fn empty_analysis() -> SourceQualityAnalysis {
    let empty = |kind: DimensionKind| DimensionScore {
        kind,
        name: source_quality_dimension_name(kind, true).to_string(),
        score: 0,
        label: source_quality_no_data_label(true),
        signals: vec![],
    };
    SourceQualityAnalysis {
        score: 0,
        grade: "F".into(),
        substance: empty(DimensionKind::Substance),
        consistency: empty(DimensionKind::Consistency),
        authority: empty(DimensionKind::Authority),
        disclaimer: source_quality_disclaimer(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::InteractiveFinding;
    use crate::audit::AuditReport;
    use crate::audit::ViolationStatistics;
    use crate::cli::WcagLevel;
    use crate::taxonomy::Severity;
    use crate::wcag::WcagResults;

    fn minimal_report() -> AuditReport {
        AuditReport {
            url: "https://example.com".into(),
            wcag_level: WcagLevel::AA,
            timestamp: chrono::Utc::now(),
            wcag_results: WcagResults::new(),
            score: 95.0,
            grade: "A".into(),
            certificate: "SEHR GUT".into(),
            statistics: ViolationStatistics {
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
                total: 0,
            },
            nodes_analyzed: 100,
            duration_ms: 1000,
            performance: None,
            seo: None,
            security: None,
            mobile: None,
            budget_violations: vec![],
            ux: None,
            journey: None,
            dark_mode: None,
            source_quality: None,
            ai_visibility: None,
            content_visibility: None,
            tech_stack: None,
            page_screenshots: None,
            dual_viewport: None,
            viewport_scores: None,
            throttled_performance: vec![],
            patterns: None,
            screenshot_status: Default::default(),
            best_practices: None,
            consent_banner_detected: false,
            consent_banner_cmp: None,
            consent_banner_dismissed: false,
            consent_privacy: None,
            accessibility_journey: None,
            interactive_findings: Vec::new(),
            screen_reader_audit: None,
        }
    }

    #[test]
    fn test_minimal_report_produces_scores() {
        let report = minimal_report();
        let analysis = analyze_source_quality(&report);
        assert!(analysis.score <= 100);
        assert!(!analysis.disclaimer.is_empty());
        assert!(!analysis.grade.is_empty());
    }

    #[test]
    fn test_empty_batch_returns_zero() {
        let analysis = analyze_source_quality_batch(&[]);
        assert_eq!(analysis.score, 0);
    }

    #[test]
    fn test_batch_with_reports() {
        let reports = vec![minimal_report(), minimal_report()];
        let analysis = analyze_source_quality_batch(&reports);
        assert!(analysis.score <= 100);
        // Canonical English in the struct.
        assert_eq!(analysis.consistency.name, "Consistency");
        assert_eq!(analysis.consistency.kind, DimensionKind::Consistency);
    }

    #[test]
    fn test_grade_mapping() {
        assert_eq!(module_score_grade(95), "A");
        assert_eq!(module_score_grade(80), "B");
        assert_eq!(module_score_grade(65), "C");
        assert_eq!(module_score_grade(45), "D");
        assert_eq!(module_score_grade(20), "F");
    }

    #[test]
    fn ax_missing_main_prevents_positive_landmark_signal() {
        let mut report = minimal_report();
        report.interactive_findings.push(InteractiveFinding {
            category: "Landmark".to_string(),
            maps_to_finding: None,
            severity: Severity::Medium,
            journey: "link_inventory".to_string(),
            before_snapshot_label: None,
            after_snapshot_label: None,
            message: "Kein <main>-Landmark gefunden.".to_string(),
            fix_suggestion: None,
        });

        let analysis = analyze_source_quality(&report);
        let signal = analysis
            .substance
            .signals
            .iter()
            .find(|s| s.kind == QualitySignalKind::SemanticStructure)
            .expect("semantic structure signal");
        assert!(!signal.present);
        // Canonical English detail in the struct.
        assert!(signal.detail.contains("accessibility tree"));
        // PDF re-derivation yields German.
        let (_name, detail) =
            source_quality_signal_text(signal.kind, signal.present, &signal.values, false);
        assert!(detail.contains("Accessibility Tree"));
    }

    #[test]
    fn test_weighted_average() {
        assert_eq!(weighted_average(&[(100, 50), (0, 50)]), 50);
        assert_eq!(weighted_average(&[(100, 100)]), 100);
        assert_eq!(weighted_average(&[]), 0);
    }

    #[test]
    fn canonical_struct_has_no_german_chars() {
        use crate::seo::{
            HeadingStructure, MetaTags, SeoAnalysis, SocialTags, StructuredData, TechnicalSeo,
        };
        // A bare SEO profile so every "missing"/"weak" branch contributes a string.
        let mut report = minimal_report();
        report.score = 50.0;
        report.seo = Some(SeoAnalysis {
            meta: MetaTags::default(),
            headings: HeadingStructure::default(),
            technical: TechnicalSeo::default(),
            social: SocialTags::default(),
            structured_data: StructuredData::default(),
            score: 40,
            content_profile: None,
            robots: None,
            page_health: None,
            serp: None,
            meta_issues: vec![],
            image_efficiency: None,
        });

        // The struct (and thus JSON) is always canonical English now.
        let analysis = analyze_source_quality(&report);
        let dims = [
            &analysis.substance,
            &analysis.consistency,
            &analysis.authority,
        ];
        for dim in dims {
            let mut texts = vec![dim.name.clone(), dim.label.clone()];
            for s in &dim.signals {
                texts.push(s.name.clone());
                texts.push(s.detail.clone());
            }
            for t in texts {
                assert!(
                    !t.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
                    "German characters in canonical struct: {t:?}"
                );
            }
        }
        assert!(!analysis
            .disclaimer
            .contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
    }

    #[test]
    fn signal_text_german_for_pdf_derivation() {
        // PDF re-derivation must yield real German for at least one variant.
        let (name, detail) = source_quality_signal_text(
            QualitySignalKind::ContentVolume,
            false,
            &SignalValues {
                word_count: Some(120),
                ..Default::default()
            },
            false,
        );
        assert_eq!(name, "Inhaltsumfang");
        assert!(detail.contains("Wörter"));
        assert!(detail.contains("Heuristik"));

        // Dimension label + disclaimer localize too.
        assert_eq!(
            source_quality_dimension_name(DimensionKind::Authority, false),
            "Autorität"
        );
        assert_eq!(source_quality_dimension_label(95, false), "Sehr gut");
        assert!(source_quality_disclaimer(false).contains("ausschließlich"));
    }
}
