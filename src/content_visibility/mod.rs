//! Content Visibility Analysis — issue #54.
//!
//! Aggregates signals from SEO, Source Quality, AI Visibility and Pattern
//! Detection into five thematic areas. Each area produces [`ContentSignal`]
//! items with structured evidence so every claim is traceable to its source.
//!
//! # Areas
//! 1. **Organic Visibility** — title, description, H1, canonical, HTTPS, rich snippets
//! 2. **Local Business** — NAP completeness, geo, sameAs, aggregateRating
//! 3. **E-E-A-T** — author/date in schema, organization, authority signals
//! 4. **Content Depth** — word count, internal links, language
//! 5. **Topical Authority** — schema coverage, breadcrumb, FAQ, heuristic note
//!
//! # Localization (#406)
//!
//! Analysis bakes **canonical English** text into every signal `title`/`detail`
//! (and thus JSON). Each signal additionally carries a stable
//! [`ContentVisibilitySignalKind`] plus the raw interpolated
//! [`ContentSignalValues`], so the PDF layer can re-derive localized text via
//! [`content_visibility_signal_text`] in the run language.

pub mod module;
pub use module::ContentVisibilityModule;

use serde::{Deserialize, Serialize};

use crate::assessment::{
    AssessmentLevel, ContentArea, ContentEvidence, ContentSignal, EvidenceConfidence,
    EvidenceSource,
};
use crate::audit::AuditReport;
use crate::seo::profile::SchemaExtracted;
use crate::seo::SchemaType;

// ─── Top-level struct ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentVisibilityAnalysis {
    /// Signals about search-engine-facing metadata and indexability.
    pub organic_visibility: Vec<ContentSignal>,
    /// Signals derived from LocalBusiness schema (skipped if absent).
    pub local_business: Vec<ContentSignal>,
    /// E-E-A-T indicators: author, date, organization, authority.
    pub eeat: Vec<ContentSignal>,
    /// Content depth and engagement signals.
    pub content_depth: Vec<ContentSignal>,
    /// Topical authority heuristics (single-run, not definitive).
    pub topical_authority: Vec<ContentSignal>,
    /// Total number of signals across all areas.
    pub signal_count: usize,
    /// Number of signals with level == Violation or Warning.
    pub problem_count: usize,
}

impl ContentVisibilityAnalysis {
    fn finish(mut self) -> Self {
        let all: Vec<&ContentSignal> = self
            .organic_visibility
            .iter()
            .chain(&self.local_business)
            .chain(&self.eeat)
            .chain(&self.content_depth)
            .chain(&self.topical_authority)
            .collect();
        self.signal_count = all.len();
        self.problem_count = all.iter().filter(|s| s.level.is_problem()).count();
        self
    }
}

// ─── Signal kind + values ────────────────────────────────────────────────────

/// Stable identifier for a concrete content-visibility signal.
///
/// One variant per distinct `title`/`detail` shape. Together with the raw
/// [`ContentSignalValues`] stored on the signal this fully reproduces the
/// human-readable strings in any language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentVisibilitySignalKind {
    // Organic visibility
    TitleMissing,
    TitleTooShort,
    TitleTooLong,
    TitleOptimal,
    DescriptionMissing,
    DescriptionLength,
    DescriptionOptimal,
    H1Missing,
    H1Exactly,
    H1Multiple,
    HttpsActive,
    HttpsMissing,
    CanonicalSet,
    CanonicalMissing,
    RichSnippetPotential,
    // Local business
    LocalBusinessNotFound,
    NapComplete,
    NapIncomplete,
    GeoPresent,
    GeoMissing,
    SameAsPresent,
    SameAsMissing,
    AggregateRating,
    // E-E-A-T
    OrganizationSchema,
    OrganizationSchemaMissing,
    ArticleAuthor,
    ArticleAuthorMissing,
    PublicationDate,
    PublicationDateMissing,
    AuthorityStrong,
    AuthorityWeak,
    AuthorityHardlyAny,
    // Content depth
    WordCountGood,
    WordCountSufficient,
    WordCountLow,
    InternalLinksGood,
    InternalLinksPresent,
    InternalLinksFew,
    LanguageDefined,
    LanguageNotDefined,
    SubstanceGood,
    SubstanceWeak,
    // Topical authority
    SchemaBroad,
    SchemaPresent,
    FaqPage,
    BreadcrumbList,
    Hreflang,
    LlmChunkQuality,
    LinkMesh,
    LinkClusterPresent,
    LinkClusterNone,
    HeadingDiversityBroad,
    HeadingDiversityBasic,
    HeadingDiversityFlat,
    ContentTypeSchema,
    StructuredContentSections,
    TrueTopicalAuthorityNotTestable,
}

/// The interpolated values a content-visibility signal text may reference.
///
/// Stored on every content-visibility [`ContentSignal`] so that
/// [`content_visibility_signal_text`] can reproduce the strings for any locale.
/// Only the fields relevant to the signal's kind are populated.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentSignalValues {
    /// A character/word/element/link count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    /// A 0–100 sub-score (authority, substance, AI chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<u32>,
    /// A free-form text fragment (missing-field list, author name, schema label).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl ContentSignalValues {
    pub fn is_empty(&self) -> bool {
        self.count.is_none() && self.score.is_none() && self.text.is_none()
    }

    fn count(count: usize) -> Self {
        Self {
            count: Some(count),
            ..Default::default()
        }
    }

    fn score(score: u32) -> Self {
        Self {
            score: Some(score),
            ..Default::default()
        }
    }

    fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }
}

// ─── Localized text (single source of truth) ─────────────────────────────────

/// The single source of truth for content-visibility signal `title`/`detail`.
///
/// Returns `(title, detail)` in German or English for the given `kind` and
/// interpolated `values`. Analysis calls it with `en = true` to bake canonical
/// English; the PDF layer calls it with the run language to re-derive localized
/// text.
pub fn content_visibility_signal_text(
    kind: ContentVisibilitySignalKind,
    values: &ContentSignalValues,
    en: bool,
) -> (String, String) {
    use ContentVisibilitySignalKind::*;

    let count = values.count.unwrap_or(0);
    let score = values.score.unwrap_or(0);
    let text = values.text.as_deref().unwrap_or("");

    match (kind, en) {
        // ── Organic visibility ───────────────────────────────────────────
        (TitleMissing, true) => (
            "Title missing".into(),
            "No <title> element found — the page appears in SERPs without a title.".into(),
        ),
        (TitleMissing, false) => (
            "Title fehlt".into(),
            "Kein <title>-Element gefunden — Seite wird in SERPs ohne Titel angezeigt.".into(),
        ),
        (TitleTooShort, true) => (
            "Title too short".into(),
            format!("Title has {count} characters (recommended: 30–60)."),
        ),
        (TitleTooShort, false) => (
            "Title zu kurz".into(),
            format!("Title hat {count} Zeichen (empfohlen: 30–60)."),
        ),
        (TitleTooLong, true) => (
            "Title too long".into(),
            format!("Title has {count} characters — may be truncated in SERPs."),
        ),
        (TitleTooLong, false) => (
            "Title zu lang".into(),
            format!("Title hat {count} Zeichen — wird in SERPs möglicherweise abgeschnitten."),
        ),
        (TitleOptimal, true) => ("Title present & optimal".into(), format!("{count} characters.")),
        (TitleOptimal, false) => ("Title vorhanden & optimal".into(), format!("{count} Zeichen.")),
        (DescriptionMissing, true) => (
            "Description missing".into(),
            "No meta description — Google may auto-generate a snippet.".into(),
        ),
        (DescriptionMissing, false) => (
            "Description fehlt".into(),
            "Keine Meta-Description — Google generiert ggf. automatisch einen Snippet.".into(),
        ),
        (DescriptionLength, true) => (
            "Description outside optimal length".into(),
            format!("{count} characters (recommended: 120–160)."),
        ),
        (DescriptionLength, false) => (
            "Description außerhalb optimaler Länge".into(),
            format!("{count} Zeichen (empfohlen: 120–160)."),
        ),
        (DescriptionOptimal, true) => {
            ("Description present & optimal".into(), format!("{count} characters."))
        }
        (DescriptionOptimal, false) => {
            ("Description vorhanden & optimal".into(), format!("{count} Zeichen."))
        }
        (H1Missing, true) => (
            "H1 missing".into(),
            "No H1 heading — the page's primary topic is not structurally marked up.".into(),
        ),
        (H1Missing, false) => (
            "H1 fehlt".into(),
            "Keine H1-Überschrift — primäres Thema der Seite nicht strukturell ausgezeichnet."
                .into(),
        ),
        (H1Exactly, true) => ("Exactly one H1".into(), text.to_string()),
        (H1Exactly, false) => ("Genau eine H1".into(), text.to_string()),
        (H1Multiple, true) => (
            "Multiple H1 elements".into(),
            format!("{count} H1 tags found — exactly one H1 per page is recommended."),
        ),
        (H1Multiple, false) => (
            "Mehrere H1-Elemente".into(),
            format!("{count} H1-Tags gefunden — empfohlen ist genau eine H1 pro Seite."),
        ),
        (HttpsActive, true) => (
            "HTTPS active".into(),
            "The page is served over an encrypted connection.".into(),
        ),
        (HttpsActive, false) => (
            "HTTPS aktiv".into(),
            "Seite wird über eine verschlüsselte Verbindung ausgeliefert.".into(),
        ),
        (HttpsMissing, true) => (
            "No HTTPS".into(),
            "Page not encrypted — ranking disadvantage and a browser warning for users.".into(),
        ),
        (HttpsMissing, false) => (
            "Kein HTTPS".into(),
            "Seite nicht verschlüsselt — Ranking-Nachteil, Browser-Warnung für Nutzer.".into(),
        ),
        (CanonicalSet, true) => ("Canonical URL set".into(), text.to_string()),
        (CanonicalSet, false) => ("Canonical URL gesetzt".into(), text.to_string()),
        (CanonicalMissing, true) => (
            "Canonical missing".into(),
            "No <link rel=\"canonical\"> — duplicate-content risk across URL variants.".into(),
        ),
        (CanonicalMissing, false) => (
            "Canonical fehlt".into(),
            "Kein <link rel=\"canonical\"> — Duplicate-Content-Risiko bei URL-Varianten.".into(),
        ),
        (RichSnippetPotential, true) => (
            "Rich-result type detected".into(),
            format!(
                "Structured data contains types associated with enhanced search appearances: {text}. Type detection alone does not confirm eligibility."
            ),
        ),
        (RichSnippetPotential, false) => (
            "Rich-Result-Typ erkannt".into(),
            format!(
                "Die strukturierten Daten enthalten Typen für mögliche erweiterte Suchdarstellungen: {text}. Die Typ-Erkennung allein bestätigt keine Eignung."
            ),
        ),
        // ── Local business ───────────────────────────────────────────────
        (LocalBusinessNotFound, true) => (
            "LocalBusiness schema not found".into(),
            "No LocalBusiness JSON-LD on this page — local search signals cannot be checked."
                .into(),
        ),
        (LocalBusinessNotFound, false) => (
            "LocalBusiness-Schema nicht gefunden".into(),
            "Kein LocalBusiness JSON-LD auf dieser Seite — lokale Suchsignale nicht prüfbar."
                .into(),
        ),
        (NapComplete, true) => (
            "NAP complete (name, address, phone)".into(),
            "LocalBusiness contains name, street, postal code, city and phone number.".into(),
        ),
        (NapComplete, false) => (
            "NAP vollständig (Name, Adresse, Telefon)".into(),
            "LocalBusiness enthält Name, Straße, PLZ, Ort und Telefonnummer.".into(),
        ),
        (NapIncomplete, true) => {
            ("NAP incomplete".into(), format!("Missing fields: {text}."))
        }
        (NapIncomplete, false) => {
            ("NAP unvollständig".into(), format!("Fehlende Felder: {text}."))
        }
        (GeoPresent, true) => (
            "Geo coordinates present".into(),
            "latitude and longitude set in LocalBusiness.geo.".into(),
        ),
        (GeoPresent, false) => (
            "Geo-Koordinaten vorhanden".into(),
            "latitude und longitude in LocalBusiness.geo gesetzt.".into(),
        ),
        (GeoMissing, true) => (
            "Geo coordinates missing".into(),
            "LocalBusiness.geo.latitude/longitude not set — maps integration limited.".into(),
        ),
        (GeoMissing, false) => (
            "Geo-Koordinaten fehlen".into(),
            "LocalBusiness.geo.latitude/longitude nicht gesetzt — Maps-Integration eingeschränkt."
                .into(),
        ),
        (SameAsPresent, true) => (
            "sameAs authority links present".into(),
            "LocalBusiness references external authority sources (Wikidata, social, etc.).".into(),
        ),
        (SameAsPresent, false) => (
            "sameAs-Autoritätslinks vorhanden".into(),
            "LocalBusiness verweist auf externe Autoritätsquellen (Wikidata, Social, etc.).".into(),
        ),
        (SameAsMissing, true) => (
            "sameAs missing".into(),
            "No sameAs links — knowledge panel linking to external sources is not possible.".into(),
        ),
        (SameAsMissing, false) => (
            "sameAs fehlt".into(),
            "Keine sameAs-Links — Knowledge-Panel-Verknüpfung mit externen Quellen nicht möglich."
                .into(),
        ),
        (AggregateRating, true) => (
            "aggregateRating present".into(),
            "Rating data in schema — enables a star snippet in SERPs.".into(),
        ),
        (AggregateRating, false) => (
            "aggregateRating vorhanden".into(),
            "Bewertungsdaten in Schema — ermöglicht Sterne-Snippet in SERPs.".into(),
        ),
        // ── E-E-A-T ──────────────────────────────────────────────────────
        (OrganizationSchema, true) => (
            "Organization schema detected".into(),
            "E-E-A-T indicator: the entity is identified as an organization.".into(),
        ),
        (OrganizationSchema, false) => (
            "Organization-Schema erkannt".into(),
            "Hinweis auf E-E-A-T: Entität ist als Organisation ausgewiesen.".into(),
        ),
        (OrganizationSchemaMissing, true) => (
            "No Organization schema".into(),
            "No indicator of institutional authority.".into(),
        ),
        (OrganizationSchemaMissing, false) => (
            "Kein Organization-Schema".into(),
            "Fehlender Hinweis auf institutionelle Autorität.".into(),
        ),
        (ArticleAuthor, true) => (
            "Author in Article schema".into(),
            format!(
                "E-E-A-T indicator: the author is identified{}",
                if text.is_empty() {
                    String::new()
                } else {
                    format!(" ({text})")
                }
            ),
        ),
        (ArticleAuthor, false) => (
            "Autor in Article-Schema".into(),
            format!(
                "Hinweis auf E-E-A-T: Autor ist ausgewiesen{}",
                if text.is_empty() {
                    String::new()
                } else {
                    format!(" ({text})")
                }
            ),
        ),
        (ArticleAuthorMissing, true) => (
            "No author in Article schema".into(),
            "Article schema without an author field — E-E-A-T signal missing.".into(),
        ),
        (ArticleAuthorMissing, false) => (
            "Kein Autor in Article-Schema".into(),
            "Article-Schema ohne author-Feld — E-E-A-T-Signal fehlt.".into(),
        ),
        (PublicationDate, true) => (
            "Publication date present".into(),
            "datePublished set in Article schema — recency can be signaled.".into(),
        ),
        (PublicationDate, false) => (
            "Veröffentlichungsdatum vorhanden".into(),
            "datePublished in Article-Schema gesetzt — Aktualität signalisierbar.".into(),
        ),
        (PublicationDateMissing, true) => (
            "No publication date".into(),
            "datePublished missing — Google cannot derive recency from the schema.".into(),
        ),
        (PublicationDateMissing, false) => (
            "Kein Veröffentlichungsdatum".into(),
            "datePublished fehlt — Google kann Aktualität nicht aus Schema ableiten.".into(),
        ),
        (AuthorityStrong, true) => (
            "Strong authority signals".into(),
            format!("Source quality authority: {score}/100."),
        ),
        (AuthorityStrong, false) => (
            "Starke Autoritätssignale".into(),
            format!("Source-Quality-Autorität: {score}/100."),
        ),
        (AuthorityWeak, true) => (
            "Weak authority signals".into(),
            format!("Source quality authority: {score}/100 — review imprint, linking or schema."),
        ),
        (AuthorityWeak, false) => (
            "Schwache Autoritätssignale".into(),
            format!(
                "Source-Quality-Autorität: {score}/100 — Impressum, Verlinkung oder Schema prüfen."
            ),
        ),
        (AuthorityHardlyAny, true) => (
            "Hardly any authority signals detected".into(),
            format!("Source quality authority: {score}/100."),
        ),
        (AuthorityHardlyAny, false) => (
            "Kaum Autoritätssignale erkannt".into(),
            format!("Source-Quality-Autorität: {score}/100."),
        ),
        // ── Content depth ────────────────────────────────────────────────
        (WordCountGood, true) => (
            "Good content depth".into(),
            format!("{count} words — in-depth content recognizable."),
        ),
        (WordCountGood, false) => (
            "Gute Inhaltstiefe".into(),
            format!("{count} Wörter — tiefergehender Inhalt erkennbar."),
        ),
        (WordCountSufficient, true) => {
            ("Sufficient word count".into(), format!("{count} words."))
        }
        (WordCountSufficient, false) => {
            ("Ausreichende Wortanzahl".into(), format!("{count} Wörter."))
        }
        (WordCountLow, true) => (
            "Little content".into(),
            format!("{count} words — thin-content risk (recommended: ≥ 300)."),
        ),
        (WordCountLow, false) => (
            "Wenig Inhalt".into(),
            format!("{count} Wörter — Thin-Content-Risiko (empfohlen: ≥ 300)."),
        ),
        (InternalLinksGood, true) => (
            "Good internal linking".into(),
            format!("{count} internal links — helps crawlers discover related pages."),
        ),
        (InternalLinksGood, false) => (
            "Gute interne Verlinkung".into(),
            format!("{count} interne Links — hilft Crawlern beim Entdecken verwandter Seiten."),
        ),
        (InternalLinksPresent, true) => {
            ("Internal linking present".into(), format!("{count} internal links."))
        }
        (InternalLinksPresent, false) => {
            ("Interne Verlinkung vorhanden".into(), format!("{count} interne Links."))
        }
        (InternalLinksFew, true) => (
            "Hardly any internal links".into(),
            format!("{count} {} — the page appears isolated.", if count == 1 { "internal link" } else { "internal links" }),
        ),
        (InternalLinksFew, false) => (
            "Kaum interne Links".into(),
            format!("{count} {} — Seite wirkt isoliert.", if count == 1 { "interner Link" } else { "interne Links" }),
        ),
        (LanguageDefined, true) => ("Language defined".into(), text.to_string()),
        (LanguageDefined, false) => ("Sprache definiert".into(), text.to_string()),
        (LanguageNotDefined, true) => (
            "Language not defined".into(),
            "No lang attribute on the <html> element — internationalization issue.".into(),
        ),
        (LanguageNotDefined, false) => (
            "Sprache nicht definiert".into(),
            "Kein lang-Attribut am <html>-Element — Internationalisierungsproblem.".into(),
        ),
        (SubstanceGood, true) => (
            "Good content substance".into(),
            format!("Substance score: {score}/100."),
        ),
        (SubstanceGood, false) => (
            "Gute inhaltliche Substanz".into(),
            format!("Substanz-Score: {score}/100."),
        ),
        (SubstanceWeak, true) => (
            "Weak content substance".into(),
            format!("Substance score: {score}/100 — review content, structure or sources."),
        ),
        (SubstanceWeak, false) => (
            "Schwache inhaltliche Substanz".into(),
            format!("Substanz-Score: {score}/100 — Inhalt, Struktur oder Quellen prüfen."),
        ),
        // ── Topical authority ────────────────────────────────────────────
        (SchemaBroad, true) => (
            "Broad schema coverage".into(),
            format!("{count} schemas — thematic diversity recognizable."),
        ),
        (SchemaBroad, false) => (
            "Breite Schema-Abdeckung".into(),
            format!("{count} Schemas — thematische Vielfalt erkennbar."),
        ),
        (SchemaPresent, true) => (
            "Schema data present".into(),
            format!("{count} {} found.", if count == 1 { "schema" } else { "schemas" }),
        ),
        (SchemaPresent, false) => (
            "Schema-Daten vorhanden".into(),
            format!("{count} {} gefunden.", if count == 1 { "Schema" } else { "Schemas" }),
        ),
        (FaqPage, true) => (
            "FAQPage schema present".into(),
            "Topic coverage signaled by FAQ questions — good for featured snippets.".into(),
        ),
        (FaqPage, false) => (
            "FAQPage-Schema vorhanden".into(),
            "Themenabdeckung durch FAQ-Fragen signalisiert — gut für Featured Snippets.".into(),
        ),
        (BreadcrumbList, true) => (
            "BreadcrumbList detected".into(),
            "Page is embedded in a content cluster — indicator of thematic structure.".into(),
        ),
        (BreadcrumbList, false) => (
            "BreadcrumbList erkannt".into(),
            "Seite ist in einem Inhalts-Cluster eingebettet — Hinweis auf thematische Struktur."
                .into(),
        ),
        (Hreflang, true) => (
            "Multilingual content (hreflang)".into(),
            format!("{count} language variants linked."),
        ),
        (Hreflang, false) => (
            "Mehrsprachige Inhalte (hreflang)".into(),
            format!("{count} Sprachvarianten verknüpft."),
        ),
        (LlmChunkQuality, true) => (
            "Good LLM chunk quality".into(),
            format!(
                "AI chunk score: {score}/100 — content well structured for AI citability."
            ),
        ),
        (LlmChunkQuality, false) => (
            "Gute LLM-Chunk-Qualität".into(),
            format!(
                "AI-Chunk-Score: {score}/100 — Inhalt für KI-Zitierbarkeit gut strukturiert."
            ),
        ),
        (LinkMesh, true) => (
            "Internal link mesh recognizable".into(),
            format!("{count} internal links — the page is embedded in a topic cluster."),
        ),
        (LinkMesh, false) => (
            "Internes Linkgeflecht erkennbar".into(),
            format!("{count} interne Links — Seite ist in ein Themencluster eingebettet."),
        ),
        (LinkClusterPresent, true) => (
            "Internal linking present".into(),
            format!("{count} internal links found."),
        ),
        (LinkClusterPresent, false) => (
            "Interne Verlinkung vorhanden".into(),
            format!("{count} interne Links gefunden."),
        ),
        (LinkClusterNone, true) => (
            "No internal links".into(),
            "Page appears isolated — missing cluster embedding weakens topical authority.".into(),
        ),
        (LinkClusterNone, false) => (
            "Keine internen Links".into(),
            "Seite wirkt isoliert — fehlende Cluster-Einbettung schwächt topische Autorität.".into(),
        ),
        (HeadingDiversityBroad, true) => (
            "Thematic structure recognizable".into(),
            format!(
                "{count} subheadings (H2+) — broad topic coverage visible in the single run."
            ),
        ),
        (HeadingDiversityBroad, false) => (
            "Thematische Gliederung erkennbar".into(),
            format!(
                "{count} Unterüberschriften (H2+) — breite Themenabdeckung im Single-Durchlauf sichtbar."
            ),
        ),
        (HeadingDiversityBasic, true) => (
            "Basic structure via headings present".into(),
            format!("{count} subheadings (H2+) found."),
        ),
        (HeadingDiversityBasic, false) => (
            "Grundstruktur durch Überschriften vorhanden".into(),
            format!("{count} Unterüberschriften (H2+) gefunden."),
        ),
        (HeadingDiversityFlat, true) => (
            "Flat content structure".into(),
            "Fewer than 2 subheadings — topic coverage not recognizable in the single run.".into(),
        ),
        (HeadingDiversityFlat, false) => (
            "Flache Inhaltsstruktur".into(),
            "Weniger als 2 Unterüberschriften — Themenabdeckung im Single-Durchlauf nicht erkennbar."
                .into(),
        ),
        (ContentTypeSchema, true) => (
            "Content type identified via schema".into(),
            format!(
                "{text} schema present — page type clearly recognizable for search engines."
            ),
        ),
        (ContentTypeSchema, false) => (
            "Inhaltstyp durch Schema identifiziert".into(),
            format!(
                "{text}-Schema vorhanden — Seitentyp für Suchmaschinen eindeutig erkennbar."
            ),
        ),
        (StructuredContentSections, true) => (
            "Structured content sections detected".into(),
            "Accordion or disclosure pattern found — indicator of thematically structured content."
                .into(),
        ),
        (StructuredContentSections, false) => (
            "Strukturierte Inhaltsabschnitte erkannt".into(),
            "Accordion oder Disclosure-Pattern gefunden — Hinweis auf thematisch gegliederte Inhalte."
                .into(),
        ),
        (TrueTopicalAuthorityNotTestable, true) => (
            "True topical authority (not testable in the single run)".into(),
            "Backlinks, SERP positions, historical performance and domain age \
             cannot be assessed automatically."
                .into(),
        ),
        (TrueTopicalAuthorityNotTestable, false) => (
            "Echte Topical Authority (nicht prüfbar im Single-Durchlauf)".into(),
            "Backlinks, SERP-Positionen, historische Performance und Domain-Alter \
             können automatisiert nicht bewertet werden."
                .into(),
        ),
    }
}

/// Build a content-visibility signal, baking canonical-English
/// `title`/`detail` from its kind + values via [`content_visibility_signal_text`].
fn signal(
    area: ContentArea,
    level: AssessmentLevel,
    confidence: EvidenceConfidence,
    kind: ContentVisibilitySignalKind,
    values: ContentSignalValues,
) -> ContentSignal {
    let (title, detail) = content_visibility_signal_text(kind, &values, true);
    ContentSignal::new(area, level, confidence, title, detail).with_cv(kind, values)
}

// ─── Entry point ─────────────────────────────────────────────────────────────

/// Derive content visibility from an existing audit report (single page).
///
/// Produces canonical-English text in every signal (and thus JSON).
pub fn analyze_content_visibility(report: &AuditReport) -> ContentVisibilityAnalysis {
    let mut out = ContentVisibilityAnalysis::default();

    if let Some(seo) = &report.discoverability.seo {
        out.organic_visibility = build_organic_visibility(seo);

        let lb_signals = build_local_business(seo);
        out.local_business = lb_signals;

        out.eeat = build_eeat(seo, report.discoverability.source_quality.as_ref());
        out.content_depth =
            build_content_depth(seo, report.discoverability.source_quality.as_ref());
        out.topical_authority = build_topical_authority(
            seo,
            report.discoverability.ai_visibility.as_ref(),
            report.patterns.as_ref(),
        );
    }

    out.finish()
}

// ─── Area builders ───────────────────────────────────────────────────────────

fn build_organic_visibility(seo: &crate::seo::SeoAnalysis) -> Vec<ContentSignal> {
    use ContentVisibilitySignalKind::*;
    let mut signals = Vec::new();
    let meta = &seo.meta;
    let tech = &seo.technical;

    // Title
    match meta.title.as_deref() {
        None => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Violation,
                EvidenceConfidence::High,
                TitleMissing,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.title"),
            ),
        ),
        Some(t) if t.len() < 30 => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Warning,
                EvidenceConfidence::High,
                TitleTooShort,
                ContentSignalValues::count(t.len()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.title")
                    .with_value(t),
            ),
        ),
        Some(t) if t.len() > 60 => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Warning,
                EvidenceConfidence::High,
                TitleTooLong,
                ContentSignalValues::count(t.len()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.title")
                    .with_value(t),
            ),
        ),
        Some(t) => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Pass,
                EvidenceConfidence::High,
                TitleOptimal,
                ContentSignalValues::count(t.len()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.title")
                    .with_value(t),
            ),
        ),
    }

    // Description
    match meta.description.as_deref() {
        None => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Warning,
                EvidenceConfidence::High,
                DescriptionMissing,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.description"),
            ),
        ),
        Some(d) if d.len() < 120 || d.len() > 160 => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Warning,
                EvidenceConfidence::High,
                DescriptionLength,
                ContentSignalValues::count(d.len()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("meta.description"),
            ),
        ),
        Some(d) => signals.push(signal(
            ContentArea::Seo,
            AssessmentLevel::Pass,
            EvidenceConfidence::High,
            DescriptionOptimal,
            ContentSignalValues::count(d.len()),
        )),
    }

    // H1
    match seo.headings.h1_count {
        0 => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Violation,
                EvidenceConfidence::High,
                H1Missing,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::High)
                    .with_field("heading.h1"),
            ),
        ),
        1 => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Pass,
                EvidenceConfidence::High,
                H1Exactly,
                ContentSignalValues::text(seo.headings.h1_text.clone().unwrap_or_default()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::High)
                    .with_field("heading.h1")
                    .with_value(seo.headings.h1_text.as_deref().unwrap_or("")),
            ),
        ),
        n => signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Warning,
                EvidenceConfidence::High,
                H1Multiple,
                ContentSignalValues::count(n),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::High)
                    .with_field("heading.h1"),
            ),
        ),
    }

    // HTTPS
    signals.push(if tech.https {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Pass,
            EvidenceConfidence::High,
            HttpsActive,
            ContentSignalValues::default(),
        )
        .with_evidence(ContentEvidence::new(
            EvidenceSource::HttpHeader,
            EvidenceConfidence::High,
        ))
    } else {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Violation,
            EvidenceConfidence::High,
            HttpsMissing,
            ContentSignalValues::default(),
        )
        .with_evidence(ContentEvidence::new(
            EvidenceSource::HttpHeader,
            EvidenceConfidence::High,
        ))
    });

    // Canonical
    signals.push(if tech.has_canonical {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Pass,
            EvidenceConfidence::High,
            CanonicalSet,
            ContentSignalValues::text(tech.canonical_url.clone().unwrap_or_default()),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                .with_field("link[rel=canonical]")
                .with_value(tech.canonical_url.as_deref().unwrap_or("")),
        )
    } else {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Warning,
            EvidenceConfidence::High,
            CanonicalMissing,
            ContentSignalValues::default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                .with_field("link[rel=canonical]"),
        )
    });

    // Rich snippets
    if !seo.structured_data.rich_snippets_potential.is_empty() {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::Medium,
                RichSnippetPotential,
                ContentSignalValues::text(seo.structured_data.rich_snippets_potential.join(", ")),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data.rich_snippets"),
            ),
        );
    }

    signals
}

fn build_local_business(seo: &crate::seo::SeoAnalysis) -> Vec<ContentSignal> {
    use ContentVisibilitySignalKind::*;

    let lb_schema = seo.content_profile.as_ref().and_then(|cp| {
        cp.schema_inventory
            .schemas
            .iter()
            .find(|s| s.schema_type == "LocalBusiness")
    });

    let raw = seo
        .structured_data
        .json_ld
        .iter()
        .find(|s| s.schema_type == "LocalBusiness")
        .map(|s| &s.content);

    if raw.is_none()
        && !seo
            .structured_data
            .types
            .contains(&SchemaType::LocalBusiness)
    {
        return vec![signal(
            ContentArea::Seo,
            AssessmentLevel::NotTestable,
            EvidenceConfidence::High,
            LocalBusinessNotFound,
            ContentSignalValues::default(),
        )];
    }

    let mut signals = Vec::new();
    let v = match raw {
        Some(r) => r,
        None => return signals,
    };

    // Extract fields from LocalBusiness extracted data if available
    let (postal_ok, locality_ok, phone_ok, has_geo, has_same_as, has_rating) =
        if let Some(schema) = lb_schema {
            if let SchemaExtracted::LocalBusiness {
                postal_code,
                locality,
                phone,
                latitude,
                longitude,
                same_as,
                aggregate_rating,
                ..
            } = &schema.extracted
            {
                (
                    postal_code.is_some(),
                    locality.is_some(),
                    phone.is_some(),
                    latitude.is_some() || longitude.is_some(),
                    !same_as.is_empty(),
                    aggregate_rating.is_some(),
                )
            } else {
                (
                    !v["address"]["postalCode"].is_null(),
                    !v["address"]["addressLocality"].is_null(),
                    !v["telephone"].is_null(),
                    !v["geo"]["latitude"].is_null() || !v["geo"]["longitude"].is_null(),
                    !v["sameAs"].is_null(),
                    !v["aggregateRating"].is_null(),
                )
            }
        } else {
            (
                !v["address"]["postalCode"].is_null(),
                !v["address"]["addressLocality"].is_null(),
                !v["telephone"].is_null(),
                !v["geo"]["latitude"].is_null() || !v["geo"]["longitude"].is_null(),
                !v["sameAs"].is_null(),
                !v["aggregateRating"].is_null(),
            )
        };

    let has_name = !v["name"].is_null();
    let has_street = !v["address"]["streetAddress"].is_null();
    let nap_complete = has_name && has_street && postal_ok && locality_ok && phone_ok;

    signals.push(if nap_complete {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Pass,
            EvidenceConfidence::High,
            NapComplete,
            ContentSignalValues::default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.address"),
        )
    } else {
        let missing: Vec<&str> = [
            (!has_name).then_some("name"),
            (!has_street).then_some("streetAddress"),
            (!postal_ok).then_some("postalCode"),
            (!locality_ok).then_some("addressLocality"),
            (!phone_ok).then_some("telephone"),
        ]
        .into_iter()
        .flatten()
        .collect();
        signal(
            ContentArea::Seo,
            AssessmentLevel::Warning,
            EvidenceConfidence::High,
            NapIncomplete,
            ContentSignalValues::text(missing.join(", ")),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.address"),
        )
    });

    signals.push(if has_geo {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Pass,
            EvidenceConfidence::High,
            GeoPresent,
            ContentSignalValues::default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.geo"),
        )
    } else {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Warning,
            EvidenceConfidence::Medium,
            GeoMissing,
            ContentSignalValues::default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::Medium)
                .with_field("LocalBusiness.geo"),
        )
    });

    signals.push(if has_same_as {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Positive,
            EvidenceConfidence::High,
            SameAsPresent,
            ContentSignalValues::default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("LocalBusiness.sameAs"),
        )
    } else {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Warning,
            EvidenceConfidence::Medium,
            SameAsMissing,
            ContentSignalValues::default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::Medium)
                .with_field("LocalBusiness.sameAs"),
        )
    });

    if has_rating {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::High,
                AggregateRating,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("LocalBusiness.aggregateRating"),
            ),
        );
    }

    signals
}

fn build_eeat(
    seo: &crate::seo::SeoAnalysis,
    source_quality: Option<&crate::source_quality::SourceQualityAnalysis>,
) -> Vec<ContentSignal> {
    use ContentVisibilitySignalKind::*;
    let mut signals = Vec::new();

    // Organization schema
    let has_org = seo
        .structured_data
        .types
        .iter()
        .any(SchemaType::is_organization_like);
    signals.push(if has_org {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Positive,
            EvidenceConfidence::High,
            OrganizationSchema,
            ContentSignalValues::default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                .with_field("Organization"),
        )
    } else {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Warning,
            EvidenceConfidence::Medium,
            OrganizationSchemaMissing,
            ContentSignalValues::default(),
        )
        .with_evidence(ContentEvidence::new(
            EvidenceSource::JsonLd,
            EvidenceConfidence::Medium,
        ))
    });

    // Author in Article schema
    let article_schema = seo.structured_data.json_ld.iter().find(|s| {
        matches!(
            s.schema_type.as_str(),
            "Article" | "BlogPosting" | "NewsArticle"
        )
    });
    if let Some(article) = article_schema {
        let has_author = !article.content["author"].is_null();
        let has_date = !article.content["datePublished"].is_null();

        signals.push(if has_author {
            let author_name = article.content["author"]["name"]
                .as_str()
                .or_else(|| article.content["author"].as_str())
                .unwrap_or("");
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::High,
                ArticleAuthor,
                ContentSignalValues::text(author_name),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.author")
                    .with_value(author_name),
            )
        } else {
            signal(
                ContentArea::Seo,
                AssessmentLevel::Warning,
                EvidenceConfidence::High,
                ArticleAuthorMissing,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.author"),
            )
        });

        signals.push(if has_date {
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::High,
                PublicationDate,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.datePublished")
                    .with_value(
                        article.content["datePublished"]
                            .as_str()
                            .unwrap_or("(present)"),
                    ),
            )
        } else {
            signal(
                ContentArea::Seo,
                AssessmentLevel::Warning,
                EvidenceConfidence::High,
                PublicationDateMissing,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("Article.datePublished"),
            )
        });
    }

    // Source quality authority
    if let Some(sq) = source_quality {
        let auth_score = sq.authority.score;
        signals.push(if auth_score >= 70 {
            signal(
                ContentArea::SourceQuality,
                AssessmentLevel::Positive,
                EvidenceConfidence::Medium,
                AuthorityStrong,
                ContentSignalValues::score(auth_score),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else if auth_score >= 40 {
            signal(
                ContentArea::SourceQuality,
                AssessmentLevel::Warning,
                EvidenceConfidence::Medium,
                AuthorityWeak,
                ContentSignalValues::score(auth_score),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else {
            signal(
                ContentArea::SourceQuality,
                AssessmentLevel::Violation,
                EvidenceConfidence::Medium,
                AuthorityHardlyAny,
                ContentSignalValues::score(auth_score),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        });
    }

    signals
}

fn build_content_depth(
    seo: &crate::seo::SeoAnalysis,
    source_quality: Option<&crate::source_quality::SourceQualityAnalysis>,
) -> Vec<ContentSignal> {
    use ContentVisibilitySignalKind::*;
    let mut signals = Vec::new();
    let tech = &seo.technical;

    // Word count
    let wc = tech.word_count as usize;
    signals.push(if wc >= 600 {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Positive,
            EvidenceConfidence::High,
            WordCountGood,
            ContentSignalValues::count(wc),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::High)
                .with_field("word_count")
                .with_value(wc.to_string()),
        )
    } else if wc >= 300 {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Pass,
            EvidenceConfidence::High,
            WordCountSufficient,
            ContentSignalValues::count(wc),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::High)
                .with_field("word_count")
                .with_value(wc.to_string()),
        )
    } else {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Warning,
            EvidenceConfidence::High,
            WordCountLow,
            ContentSignalValues::count(wc),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::High)
                .with_field("word_count")
                .with_value(wc.to_string()),
        )
    });

    // Internal links
    let il = tech.internal_links as usize;
    signals.push(if il >= 5 {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Positive,
            EvidenceConfidence::High,
            InternalLinksGood,
            ContentSignalValues::count(il),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                .with_field("internal_links")
                .with_value(il.to_string()),
        )
    } else if il >= 2 {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Pass,
            EvidenceConfidence::High,
            InternalLinksPresent,
            ContentSignalValues::count(il),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                .with_field("internal_links"),
        )
    } else {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Warning,
            EvidenceConfidence::High,
            InternalLinksFew,
            ContentSignalValues::count(il),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                .with_field("internal_links"),
        )
    });

    // Language
    signals.push(if tech.has_lang {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Pass,
            EvidenceConfidence::High,
            LanguageDefined,
            ContentSignalValues::text(tech.lang.clone().unwrap_or_default()),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::DomAttribute, EvidenceConfidence::High)
                .with_field("html[lang]")
                .with_value(tech.lang.as_deref().unwrap_or("")),
        )
    } else {
        signal(
            ContentArea::Seo,
            AssessmentLevel::Warning,
            EvidenceConfidence::High,
            LanguageNotDefined,
            ContentSignalValues::default(),
        )
        .with_evidence(
            ContentEvidence::new(EvidenceSource::DomAttribute, EvidenceConfidence::High)
                .with_field("html[lang]"),
        )
    });

    // Source quality substance
    if let Some(sq) = source_quality {
        let sub_score = sq.substance.score;
        signals.push(if sub_score >= 70 {
            signal(
                ContentArea::SourceQuality,
                AssessmentLevel::Positive,
                EvidenceConfidence::Medium,
                SubstanceGood,
                ContentSignalValues::score(sub_score),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        } else {
            signal(
                ContentArea::SourceQuality,
                AssessmentLevel::Warning,
                EvidenceConfidence::Medium,
                SubstanceWeak,
                ContentSignalValues::score(sub_score),
            )
            .with_evidence(ContentEvidence::new(
                EvidenceSource::Computed,
                EvidenceConfidence::Medium,
            ))
        });
    }

    signals
}

fn build_topical_authority(
    seo: &crate::seo::SeoAnalysis,
    ai_visibility: Option<&crate::ai_visibility::AiVisibilityAnalysis>,
    patterns: Option<&crate::patterns::PatternAnalysis>,
) -> Vec<ContentSignal> {
    use ContentVisibilitySignalKind::*;

    let mut signals = Vec::new();

    // Schema coverage (thematic richness)
    let schema_count = seo
        .structured_data
        .json_ld
        .iter()
        .filter(|schema| schema.is_valid && !schema.schema_types.is_empty())
        .count();
    if schema_count >= 3 {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::Medium,
                SchemaBroad,
                ContentSignalValues::count(schema_count),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data")
                    .with_value(schema_count.to_string()),
            ),
        );
    } else if schema_count > 0 {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Pass,
                EvidenceConfidence::Medium,
                SchemaPresent,
                ContentSignalValues::count(schema_count),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data"),
            ),
        );
    }

    // FAQ schema → topic coverage signal
    let has_faq = seo.structured_data.types.contains(&SchemaType::FAQPage);
    if has_faq {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::High,
                FaqPage,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("FAQPage"),
            ),
        );
    }

    // BreadcrumbList → content hierarchy
    let has_breadcrumb = seo
        .structured_data
        .types
        .contains(&SchemaType::BreadcrumbList);
    if has_breadcrumb {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::High,
                BreadcrumbList,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("BreadcrumbList"),
            ),
        );
    }

    // Hreflang → multilingual content authority
    if seo.technical.has_hreflang {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::High,
                Hreflang,
                ContentSignalValues::count(seo.technical.hreflang.len()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Meta, EvidenceConfidence::High)
                    .with_field("link[rel=alternate][hreflang]"),
            ),
        );
    }

    // AI visibility chunk quality → LLM-citability
    if let Some(ai) = ai_visibility {
        let chunk_score = ai.chunks.dimension.score;
        if chunk_score >= 70 {
            signals.push(
                signal(
                    ContentArea::AiVisibility,
                    AssessmentLevel::Positive,
                    EvidenceConfidence::Medium,
                    LlmChunkQuality,
                    ContentSignalValues::score(chunk_score),
                )
                .with_evidence(ContentEvidence::new(
                    EvidenceSource::Computed,
                    EvidenceConfidence::Medium,
                )),
            );
        }
    }

    // Internal link cluster — proxy for site embeddedness
    let internal_links = seo.technical.internal_links as usize;
    if internal_links >= 10 {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::Medium,
                LinkMesh,
                ContentSignalValues::count(internal_links),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                    .with_field("internal_links")
                    .with_value(internal_links.to_string()),
            ),
        );
    } else if internal_links >= 3 {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Pass,
                EvidenceConfidence::Medium,
                LinkClusterPresent,
                ContentSignalValues::count(internal_links),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                    .with_field("internal_links")
                    .with_value(internal_links.to_string()),
            ),
        );
    } else if internal_links == 0 {
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Warning,
                EvidenceConfidence::Medium,
                LinkClusterNone,
                ContentSignalValues::default(),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::Link, EvidenceConfidence::High)
                    .with_field("internal_links")
                    .with_value("0".to_string()),
            ),
        );
    }

    // Heading diversity — proxy for semantic topic coverage
    let h2_plus_count = seo
        .headings
        .headings
        .iter()
        .filter(|h| h.level >= 2)
        .count();
    if h2_plus_count >= 5 {
        signals.push(
            signal(
                ContentArea::Content,
                AssessmentLevel::Positive,
                EvidenceConfidence::Medium,
                HeadingDiversityBroad,
                ContentSignalValues::count(h2_plus_count),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::Medium)
                    .with_field("headings.h2_plus")
                    .with_value(h2_plus_count.to_string()),
            ),
        );
    } else if h2_plus_count >= 2 {
        signals.push(
            signal(
                ContentArea::Content,
                AssessmentLevel::Pass,
                EvidenceConfidence::Medium,
                HeadingDiversityBasic,
                ContentSignalValues::count(h2_plus_count),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::Medium)
                    .with_field("headings.h2_plus")
                    .with_value(h2_plus_count.to_string()),
            ),
        );
    } else {
        signals.push(
            signal(
                ContentArea::Content,
                AssessmentLevel::Warning,
                EvidenceConfidence::Low,
                HeadingDiversityFlat,
                ContentSignalValues::count(h2_plus_count),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::VisibleText, EvidenceConfidence::Medium)
                    .with_field("headings.h2_plus")
                    .with_value(h2_plus_count.to_string()),
            ),
        );
    }

    // Content-type schema — identifies what kind of content this page represents
    let has_content_schema = seo.structured_data.types.iter().any(|t| {
        matches!(
            t,
            SchemaType::Article
                | SchemaType::BlogPosting
                | SchemaType::NewsArticle
                | SchemaType::HowTo
                | SchemaType::WebPage
                | SchemaType::Product
                | SchemaType::Recipe
                | SchemaType::Event
                | SchemaType::VideoObject
        )
    });
    if has_content_schema {
        let schema_label = seo
            .structured_data
            .types
            .iter()
            .find(|t| {
                matches!(
                    t,
                    SchemaType::Article
                        | SchemaType::BlogPosting
                        | SchemaType::NewsArticle
                        | SchemaType::HowTo
                        | SchemaType::WebPage
                        | SchemaType::Product
                        | SchemaType::Recipe
                        | SchemaType::Event
                        | SchemaType::VideoObject
                )
            })
            .map(|t| format!("{t:?}"))
            .unwrap_or_default();
        signals.push(
            signal(
                ContentArea::Seo,
                AssessmentLevel::Positive,
                EvidenceConfidence::High,
                ContentTypeSchema,
                ContentSignalValues::text(schema_label.clone()),
            )
            .with_evidence(
                ContentEvidence::new(EvidenceSource::JsonLd, EvidenceConfidence::High)
                    .with_field("structured_data.type")
                    .with_value(schema_label),
            ),
        );
    }

    // Accordion/DisclosureMenu patterns → structured FAQ-like topic coverage
    if let Some(pats) = patterns {
        let has_structured_content = pats
            .recognized
            .iter()
            .any(|p| p.pattern == "Accordion" || p.pattern == "DisclosureMenu");
        if has_structured_content {
            signals.push(
                signal(
                    ContentArea::Content,
                    AssessmentLevel::Pass,
                    EvidenceConfidence::Medium,
                    StructuredContentSections,
                    ContentSignalValues::default(),
                )
                .with_evidence(
                    ContentEvidence::new(EvidenceSource::AxTree, EvidenceConfidence::Medium)
                        .with_field("patterns.recognized"),
                ),
            );
        }
    }

    // Always: not-testable for true authority
    signals.push(signal(
        ContentArea::Content,
        AssessmentLevel::NotTestable,
        EvidenceConfidence::High,
        TrueTopicalAuthorityNotTestable,
        ContentSignalValues::default(),
    ));

    signals
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::AssessmentLevel;

    fn minimal_seo() -> crate::seo::SeoAnalysis {
        use crate::seo::{
            HeadingStructure, MetaTags, SeoAnalysis, SocialTags, StructuredData, TechnicalSeo,
        };
        SeoAnalysis {
            meta: MetaTags {
                title: Some("Test Page Example Site Accessibility".to_string()),
                description: Some("A test description that is long enough for SEO purposes and fits perfectly within the recommended 120 to 160 character limit for meta descriptions.".to_string()),
                ..Default::default()
            },
            headings: HeadingStructure {
                h1_count: 1,
                h1_text: Some("Test Page".to_string()),
                ..Default::default()
            },
            technical: TechnicalSeo {
                https: true,
                has_canonical: true,
                canonical_url: Some("https://example.com/test".to_string()),
                has_lang: true,
                lang: Some("de".to_string()),
                word_count: 450,
                internal_links: 4,
                ..Default::default()
            },
            social: SocialTags::default(),
            structured_data: StructuredData::default(),
            score: 80,
            content_profile: None,
            robots: None,
            page_health: None,
            serp: None,
            meta_issues: vec![],
            image_efficiency: None,
        }
    }

    #[test]
    fn organic_visibility_all_pass() {
        let seo = minimal_seo();
        let signals = build_organic_visibility(&seo);
        let problems: Vec<_> = signals.iter().filter(|s| s.level.is_problem()).collect();
        assert!(
            problems.is_empty(),
            "Expected no problems, got: {:?}",
            problems.iter().map(|s| &s.title).collect::<Vec<_>>()
        );
    }

    #[test]
    fn organic_visibility_missing_title() {
        let mut seo = minimal_seo();
        seo.meta.title = None;
        let signals = build_organic_visibility(&seo);
        assert!(signals.iter().any(|s| s.level == AssessmentLevel::Violation
            && s.cv_kind == Some(ContentVisibilitySignalKind::TitleMissing)));
    }

    #[test]
    fn local_business_not_testable_when_absent() {
        let seo = minimal_seo();
        let signals = build_local_business(&seo);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].level, AssessmentLevel::NotTestable);
    }

    #[test]
    fn topical_authority_always_has_not_testable() {
        let seo = minimal_seo();
        let signals = build_topical_authority(&seo, None, None);
        assert!(signals
            .iter()
            .any(|s| s.level == AssessmentLevel::NotTestable));
    }

    #[test]
    fn content_depth_word_count_signals() {
        let mut seo = minimal_seo();
        seo.technical.word_count = 50;
        let signals = build_content_depth(&seo, None);
        assert!(signals.iter().any(|s| s.level.is_problem()));
    }

    #[test]
    fn finish_counts_signals_and_problems() {
        let analysis = ContentVisibilityAnalysis {
            organic_visibility: vec![
                ContentSignal::pass(ContentArea::Seo, EvidenceConfidence::High, "A", ""),
                ContentSignal::violation(ContentArea::Seo, EvidenceConfidence::High, "B", ""),
            ],
            ..ContentVisibilityAnalysis::default()
        };
        let analysis = analysis.finish();
        assert_eq!(analysis.signal_count, 2);
        assert_eq!(analysis.problem_count, 1);
    }

    #[test]
    fn organic_visibility_title_too_short_is_warning() {
        let mut seo = minimal_seo();
        seo.meta.title = Some("Short".to_string()); // 5 chars < 30
        let signals = build_organic_visibility(&seo);
        assert!(signals.iter().any(|s| s.level == AssessmentLevel::Warning
            && s.cv_kind == Some(ContentVisibilitySignalKind::TitleTooShort)));
    }

    #[test]
    fn organic_visibility_title_too_long_is_warning() {
        let mut seo = minimal_seo();
        seo.meta.title = Some("A".repeat(61)); // 61 chars > 60
        let signals = build_organic_visibility(&seo);
        assert!(signals.iter().any(|s| s.level == AssessmentLevel::Warning
            && s.cv_kind == Some(ContentVisibilitySignalKind::TitleTooLong)));
    }

    #[test]
    fn organic_visibility_description_too_short_is_warning() {
        let mut seo = minimal_seo();
        seo.meta.description = Some("Too short".to_string()); // < 120
        let signals = build_organic_visibility(&seo);
        assert!(signals.iter().any(|s| s.level == AssessmentLevel::Warning
            && s.cv_kind == Some(ContentVisibilitySignalKind::DescriptionLength)));
    }

    #[test]
    fn organic_visibility_description_too_long_is_warning() {
        let mut seo = minimal_seo();
        seo.meta.description = Some("A".repeat(161)); // > 160
        let signals = build_organic_visibility(&seo);
        assert!(signals.iter().any(|s| s.level == AssessmentLevel::Warning
            && s.cv_kind == Some(ContentVisibilitySignalKind::DescriptionLength)));
    }

    #[test]
    fn eeat_no_organization_schema_is_warning() {
        let seo = minimal_seo();
        let signals = build_eeat(&seo, None);
        assert!(signals.iter().any(|s| s.level == AssessmentLevel::Warning
            && s.cv_kind == Some(ContentVisibilitySignalKind::OrganizationSchemaMissing)));
    }

    fn make_json_ld(
        schema_type: &str,
        content: serde_json::Value,
    ) -> crate::seo::schema::JsonLdSchema {
        crate::seo::schema::JsonLdSchema {
            schema_type: schema_type.to_string(),
            schema_types: vec![schema_type.to_string()],
            content,
            is_valid: true,
        }
    }

    #[test]
    fn eeat_with_organization_schema_is_positive() {
        use crate::seo::schema::StructuredData;
        use crate::seo::SchemaType;
        let mut seo = minimal_seo();
        seo.structured_data = StructuredData {
            types: vec![SchemaType::Organization],
            json_ld: vec![make_json_ld(
                "Organization",
                serde_json::json!({"name": "Casoon"}),
            )],
            ..Default::default()
        };
        let signals = build_eeat(&seo, None);
        assert!(signals.iter().any(|s| s.level == AssessmentLevel::Positive
            && s.cv_kind == Some(ContentVisibilitySignalKind::OrganizationSchema)));
    }

    #[test]
    fn local_business_nap_incomplete_is_warning() {
        use crate::seo::schema::StructuredData;
        use crate::seo::SchemaType;
        let mut seo = minimal_seo();
        seo.structured_data = StructuredData {
            types: vec![SchemaType::LocalBusiness],
            json_ld: vec![make_json_ld(
                "LocalBusiness",
                serde_json::json!({"name": "Testfirma"}), // missing address fields
            )],
            ..Default::default()
        };
        let signals = build_local_business(&seo);
        assert!(signals.iter().any(|s| s.level == AssessmentLevel::Warning
            && s.cv_kind == Some(ContentVisibilitySignalKind::NapIncomplete)));
    }

    fn report_with_seo(seo: crate::seo::SeoAnalysis) -> AuditReport {
        use crate::audit::ViolationStatistics;
        use crate::cli::WcagLevel;
        use crate::wcag::WcagResults;
        AuditReport {
            url: "https://example.com".into(),
            wcag_level: WcagLevel::AA,
            timestamp: chrono::Utc::now(),
            accessibility: crate::audit::AccessibilitySection {
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
                execution: Default::default(),
            },
            duration_ms: 1000,
            performance: None,
            security: None,
            experience: crate::audit::ExperienceSection::default(),
            ux: None,
            journey: None,
            discoverability: crate::audit::DiscoverabilitySection {
                seo: Some(seo),
                ai_visibility: None,
                content_visibility: None,
                source_quality: None,
                tech_stack: None,
            },
            page_screenshots: None,
            dual_viewport: None,
            viewport_scores: None,
            throttled_performance: vec![],
            patterns: None,
            screenshot_status: Default::default(),
            best_practices: None,
            commerce: None,
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
    fn canonical_struct_has_no_german_chars() {
        // Use a deliberately problematic SEO profile so every code path
        // (violations, warnings, passes) contributes visible strings.
        let mut seo = minimal_seo();
        seo.meta.title = None;
        seo.meta.description = None;
        seo.headings.h1_count = 0;
        seo.technical.https = false;
        seo.technical.has_canonical = false;
        seo.technical.has_lang = false;
        seo.technical.word_count = 50;
        seo.technical.internal_links = 0;
        let report = report_with_seo(seo);

        // The struct (and thus JSON) is always canonical English now.
        let analysis = analyze_content_visibility(&report);
        let all = analysis
            .organic_visibility
            .iter()
            .chain(&analysis.local_business)
            .chain(&analysis.eeat)
            .chain(&analysis.content_depth)
            .chain(&analysis.topical_authority);
        for sig in all {
            let combined = format!("{}{}", sig.title, sig.detail);
            assert!(
                !combined.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
                "German characters in canonical struct: {combined:?}"
            );
        }
    }

    #[test]
    fn signal_text_german_for_pdf_derivation() {
        // PDF re-derivation must yield real German for at least one variant.
        let (title, detail) = content_visibility_signal_text(
            ContentVisibilitySignalKind::WordCountLow,
            &ContentSignalValues::count(120),
            false,
        );
        assert_eq!(title, "Wenig Inhalt");
        assert!(detail.contains("Wörter"));
        assert!(detail.contains("Thin-Content"));

        // English is the canonical baked variant.
        let (title_en, detail_en) = content_visibility_signal_text(
            ContentVisibilitySignalKind::WordCountLow,
            &ContentSignalValues::count(120),
            true,
        );
        assert_eq!(title_en, "Little content");
        assert!(detail_en.contains("words"));
    }
}
