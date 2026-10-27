//! AI Visibility Analysis
//!
//! Evaluates how well a website's content is prepared for discovery,
//! extraction, and citation by AI systems (LLMs, RAG pipelines, AI search).
//!
//! Five dimensions are scored:
//! - **LLM-Lesbarkeit** — How well can LLMs extract and understand the content?
//! - **Zitatfähigkeit** — Structural signals that make passages quotable (no
//!   measured or predicted citation likelihood, #784).
//! - **Chunk-Qualität** — How well is content structured for RAG/embedding pipelines?
//! - **Wissensgraph** — How rich is the entity/relationship model?
//! - **AI-Policy** — How is the site configured for AI crawler access?
//!
//! This module derives its analysis from existing audit data (SEO, Security,
//! WCAG) — it does not make additional network or CDP requests.
//!
//! ## Localization (#406)
//!
//! Analysis bakes **canonical English** text into the struct (and thus JSON):
//! every dimension `name`/`label`, signal `name`/`detail`, chunk
//! `recommendation`/section heading and knowledge-graph link suggestion is
//! produced with `en = true`. Each [`AiSignal`] additionally carries a stable
//! [`AiSignalKind`] plus the raw interpolated [`AiSignalValues`], so the PDF
//! layer can re-derive localized text via [`ai_signal_text`] /
//! [`ai_dimension_name`] / [`ai_dimension_label`] / [`ai_disclaimer`] in the
//! run language.

mod chunks;
mod citation;
mod knowledge_graph;
pub mod module;
mod readability;

pub use chunks::{
    ai_chunk_recommendation, ai_chunk_section_heading, ChunkAnalysis, ChunkQuality,
    ChunkRecommendationKind, ChunkSectionKind, ContentSection,
};
pub use citation::CitationAnalysis;
pub use knowledge_graph::{
    ai_kg_link_suggestion_reason, EntityRelationship, EntitySource, GraphEntity, KgSuggestionKind,
    KnowledgeGraphAnalysis, LinkSuggestion,
};
pub use module::AiVisibilityModule;
pub use readability::ReadabilityAnalysis;

use serde::{Deserialize, Serialize};

use crate::audit::AuditReport;
use crate::seo::schema::SchemaType;

// ─── Public types ────────────────────────────────────────────────────────────

/// Complete AI visibility analysis for a single page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiVisibilityAnalysis {
    /// Overall AI visibility score (0–100)
    pub score: u32,
    /// Qualitative band for `score`, canonical English (#406). This module
    /// carries no weight in the overall score, so it gets a band word rather
    /// than a letter grade (plan 29, D2).
    pub band: String,
    /// LLM readability dimension
    pub readability: ReadabilityAnalysis,
    /// Citation likelihood dimension
    pub citation: CitationAnalysis,
    /// Content chunk quality dimension
    pub chunks: ChunkAnalysis,
    /// Knowledge graph dimension
    pub knowledge_graph: KnowledgeGraphAnalysis,
    /// AI crawler policy dimension
    pub policy: PolicyAnalysis,
    /// Disclaimer
    pub disclaimer: String,
}

/// AI crawler policy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAnalysis {
    /// Dimension score (0–100) with individual signals
    pub dimension: DimensionScore,
    /// Whether citation/search AI bots are blocked (reduces real-time AI visibility)
    pub blocks_ai_citation: bool,
    /// Whether training bots are blocked (common practice, neutral for visibility)
    pub blocks_ai_training: bool,
    /// Whether wildcard disallow blocks everything
    pub blocks_all: bool,
    /// Number of specifically blocked AI bots
    pub blocked_ai_bot_count: usize,
    /// Human-readable policy label
    pub inferred_policy: String,
}

/// Stable identifier for which AI visibility dimension a [`DimensionScore`]
/// represents.
///
/// Lets the PDF layer re-derive a localized dimension name without parsing the
/// canonical-English `name` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionKind {
    Readability,
    Citability,
    Chunks,
    KnowledgeGraph,
    Policy,
}

/// Score for a single AI visibility dimension
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
    pub signals: Vec<AiSignal>,
}

/// Stable identifier for a concrete AI visibility signal.
///
/// One variant per distinct signal text/detail shape across all sub-analyses.
/// Together with the raw values stored on [`AiSignal`] this fully reproduces
/// the human-readable `name`/`detail` strings in any language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiSignalKind {
    // Readability
    HeadingStructure,
    ContentVolume,
    ParagraphStructure,
    Lists,
    Tables,
    SchemaCoverage,
    ExtractableAnswers,
    SummaryMeta,
    LanguageDeclaration,
    DefinitionPatterns,
    // Citation
    Encryption,
    PublisherIdentity,
    ArticleStructure,
    PublicationDate,
    CanonicalUrl,
    SnippetQuality,
    QuestionAnswerPattern,
    ContentDepth,
    SharingMetadata,
    ThematicContext,
    // Chunks
    SectionCount,
    SectionLength,
    NoOversizedSections,
    FewFragments,
    HierarchicalStructure,
    SemanticHtml,
    SectionDensity,
    ContentBoundary,
    // Knowledge graph
    EntitiesDetected,
    SchemaOrgEntities,
    Relationships,
    TypeDiversity,
    BreadcrumbHierarchy,
    LinkingDensity,
    PropertyCompleteness,
    // Policy
    RobotsReachable,
    NoWildcardBlock,
    AiSearchReachable,
    ExplicitAiPolicy,
    SitemapReference,
    MetaRobotsAllowed,
}

/// The interpolated values a signal text may reference.
///
/// Stored on every [`AiSignal`] alongside `present` so that [`ai_signal_text`]
/// can reproduce the detail string for any locale. Only the fields relevant to
/// the signal's `kind` are populated.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiSignalValues {
    /// Heading count (HeadingStructure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_count: Option<u32>,
    /// Max heading depth (HeadingStructure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_heading_depth: Option<u32>,
    /// Word count (ContentVolume, ContentDepth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<u32>,
    /// Paragraph count (ParagraphStructure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_count: Option<u32>,
    /// Average paragraph length (ParagraphStructure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_paragraph_len: Option<u32>,
    /// List count (Lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_count: Option<u32>,
    /// Schema type count (SchemaCoverage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_type_count: Option<u32>,
    /// Whether FAQ schema is present (ExtractableAnswers, QuestionAnswerPattern)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_faq: Option<bool>,
    /// Whether HowTo schema is present (ExtractableAnswers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_howto: Option<bool>,
    /// Whether basic schema is present (SchemaCoverage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_schema: Option<bool>,
    /// Meta description length (SummaryMeta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_desc_len: Option<u32>,
    /// Whether a meta description exists (SummaryMeta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_meta_description: Option<bool>,
    /// Heading / section count (ContentDepth, SectionCount)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_count: Option<u32>,
    /// Whether the author schema is present (PublisherIdentity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_author: Option<bool>,
    /// Whether the org schema is present (PublisherIdentity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_org: Option<bool>,
    /// Short paragraph ratio (SnippetQuality)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_paragraph_ratio: Option<f32>,
    /// Whether lists are present (SnippetQuality)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_lists: Option<bool>,
    /// Count of optimal-sized sections (SectionLength)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimal_count: Option<u32>,
    /// Count of too-long sections (NoOversizedSections)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub too_long_count: Option<u32>,
    /// Count of too-short sections (FewFragments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub too_short_count: Option<u32>,
    /// Average words per section (SectionDensity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_words: Option<u32>,
    /// Entity count (EntitiesDetected, PropertyCompleteness)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_count: Option<u32>,
    /// Schema.org entity count (SchemaOrgEntities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_entity_count: Option<u32>,
    /// Relationship count (Relationships)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_count: Option<u32>,
    /// Unique entity type count (TypeDiversity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_type_count: Option<u32>,
    /// Internal link count (LinkingDensity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_links: Option<u32>,
    /// Entities with properties count (PropertyCompleteness)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities_with_props: Option<u32>,
    /// Inferred policy label (ExplicitAiPolicy) — already canonical/neutral
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inferred_policy: Option<String>,
    /// Blocked AI bot count (ExplicitAiPolicy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_count: Option<u32>,
    /// Whether all crawlers are blocked (AiSearchReachable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks_all: Option<bool>,
    /// Whether citation bots are blocked (AiSearchReachable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks_citation: Option<bool>,
    /// Whether robots.txt is reachable (ExplicitAiPolicy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_robots: Option<bool>,
}

/// A single measurable signal contributing to a dimension score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSignal {
    /// Stable signal identifier (for localized re-derivation)
    pub kind: AiSignalKind,
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
    pub values: AiSignalValues,
    /// Batch only: the page whose `values` illustrate this aggregated signal
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example_url: Option<String>,
}

impl AiSignal {
    /// Build a signal, baking canonical-English `name`/`detail` from its kind +
    /// values via [`ai_signal_text`].
    pub(crate) fn new(
        kind: AiSignalKind,
        present: bool,
        weight: f32,
        values: AiSignalValues,
    ) -> Self {
        let (name, detail) = ai_signal_text(kind, present, &values, true);
        AiSignal {
            kind,
            name,
            present,
            weight,
            detail,
            values,
            example_url: None,
        }
    }
}

const DISCLAIMER_DE: &str = "Diese Bewertung basiert auf heuristischen Signalen zur \
    AI-Sichtbarkeit. Sie zählt Struktursignale, die die maschinelle Extraktion \
    erleichtern (Überschriften, Listen, strukturierte Daten, Crawler-Zugang). Ob \
    KI-Systeme die Seite abrufen oder zitieren, misst sie nicht und sagt es nicht \
    voraus; Schwellen und Gewichte sind nicht extern validiert. Über die inhaltliche \
    Qualität sagt sie nichts.";

const DISCLAIMER_EN: &str = "This assessment is based on heuristic signals for \
    AI visibility. It counts structural signals that make machine extraction \
    easier (headings, lists, structured data, crawler access). It does not measure \
    or predict whether AI systems retrieve or cite the page; its thresholds and \
    weights are not externally validated. It says nothing about factual quality.";

/// The always-present disclaimer in the requested language.
pub fn ai_disclaimer(en: bool) -> String {
    if en { DISCLAIMER_EN } else { DISCLAIMER_DE }.to_string()
}

// ─── Localized text (single source of truth) ─────────────────────────────────

/// Localized dimension name for a [`DimensionKind`].
pub fn ai_dimension_name(kind: DimensionKind, en: bool) -> &'static str {
    match (kind, en) {
        (DimensionKind::Readability, true) => "AI readability",
        (DimensionKind::Readability, false) => "KI-Lesbarkeit",
        (DimensionKind::Citability, true) => "Citability",
        (DimensionKind::Citability, false) => "Zitierbarkeit",
        (DimensionKind::Chunks, true) => "Technical AI readability",
        (DimensionKind::Chunks, false) => "Technische KI-Lesbarkeit",
        (DimensionKind::KnowledgeGraph, true) => "Structured data",
        (DimensionKind::KnowledgeGraph, false) => "Strukturierte Daten",
        (DimensionKind::Policy, true) => "AI policy",
        (DimensionKind::Policy, false) => "AI-Policy",
    }
}

/// Localized "no data" label used when a dimension has no signals.
pub fn ai_no_data_label(en: bool) -> String {
    if en { "No data" } else { "Keine Daten" }.to_string()
}

/// Localized band label for a dimension score (single source of truth).
pub fn ai_dimension_label(score: u32, en: bool) -> String {
    crate::registry::FIVE_BAND
        .label(score as f32, en)
        .to_string()
}

/// The single source of truth for signal `name`/`detail` text.
///
/// Returns `(name, detail)` in German or English for the given `kind`,
/// `present` flag, and interpolated `values`. Analysis calls it with
/// `en = true` to bake canonical English; the PDF layer calls it with the run
/// language to re-derive localized text.
pub fn ai_signal_text(
    kind: AiSignalKind,
    present: bool,
    values: &AiSignalValues,
    en: bool,
) -> (String, String) {
    use AiSignalKind::*;

    let name: String = match (kind, en) {
        (HeadingStructure, true) => "Heading structure".into(),
        (HeadingStructure, false) => "Überschriftenstruktur".into(),
        (ContentVolume, true) => "Content volume".into(),
        (ContentVolume, false) => "Inhaltsumfang".into(),
        (ParagraphStructure, true) => "Paragraph structure".into(),
        (ParagraphStructure, false) => "Absatzstruktur".into(),
        (Lists, true) => "Lists / bullet points".into(),
        (Lists, false) => "Listen / Aufzählungen".into(),
        (Tables, true) => "Tables".into(),
        (Tables, false) => "Tabellen".into(),
        (SchemaCoverage, true) => "Schema coverage".into(),
        (SchemaCoverage, false) => "Schema-Abdeckung".into(),
        (ExtractableAnswers, true) => "Extractable answers".into(),
        (ExtractableAnswers, false) => "Extrahierbare Antworten".into(),
        (SummaryMeta, true) => "Summary (meta)".into(),
        (SummaryMeta, false) => "Zusammenfassung (Meta)".into(),
        (LanguageDeclaration, true) => "Language declaration".into(),
        (LanguageDeclaration, false) => "Sprachdeklaration".into(),
        (DefinitionPatterns, true) => "Definition patterns".into(),
        (DefinitionPatterns, false) => "Definitionsmuster".into(),
        (Encryption, true) => "Encryption".into(),
        (Encryption, false) => "Verschlüsselung".into(),
        (PublisherIdentity, true) => "Publisher identity".into(),
        (PublisherIdentity, false) => "Herausgeber-Identität".into(),
        (ArticleStructure, true) => "Article structure".into(),
        (ArticleStructure, false) => "Artikelstruktur".into(),
        (PublicationDate, true) => "Publication date".into(),
        (PublicationDate, false) => "Publikationsdatum".into(),
        (CanonicalUrl, true) => "Canonical URL".into(),
        (CanonicalUrl, false) => "Kanonische URL".into(),
        (SnippetQuality, true) => "Snippet quality".into(),
        (SnippetQuality, false) => "Snippet-Qualität".into(),
        (QuestionAnswerPattern, true) => "Question-answer pattern".into(),
        (QuestionAnswerPattern, false) => "Frage-Antwort-Muster".into(),
        (ContentDepth, true) => "Content depth".into(),
        (ContentDepth, false) => "Inhaltliche Tiefe".into(),
        (SharingMetadata, true) => "Sharing metadata".into(),
        (SharingMetadata, false) => "Teilen-Metadaten".into(),
        (ThematicContext, true) => "Thematic context".into(),
        (ThematicContext, false) => "Thematische Einordnung".into(),
        (SectionCount, true) => "Section count".into(),
        (SectionCount, false) => "Abschnittszahl".into(),
        (SectionLength, true) => "Heuristic: section length".into(),
        (SectionLength, false) => "Heuristik: Abschnittslänge".into(),
        (NoOversizedSections, true) => "No oversized sections".into(),
        (NoOversizedSections, false) => "Keine Übergroßen Abschnitte".into(),
        (FewFragments, true) => "Few fragments".into(),
        (FewFragments, false) => "Wenig Fragmente".into(),
        (HierarchicalStructure, true) => "Hierarchical structure".into(),
        (HierarchicalStructure, false) => "Hierarchische Gliederung".into(),
        (SemanticHtml, true) => "Semantic HTML".into(),
        (SemanticHtml, false) => "Semantisches HTML".into(),
        (SectionDensity, true) => "Section density".into(),
        (SectionDensity, false) => "Abschnittsdichte".into(),
        (ContentBoundary, true) => "Content boundary".into(),
        (ContentBoundary, false) => "Content-Begrenzung".into(),
        (EntitiesDetected, true) => "Entities detected".into(),
        (EntitiesDetected, false) => "Entitäten erkannt".into(),
        (SchemaOrgEntities, true) => "Schema.org entities".into(),
        (SchemaOrgEntities, false) => "Schema.org-Entitäten".into(),
        (Relationships, true) => "Relationships".into(),
        (Relationships, false) => "Beziehungen".into(),
        (TypeDiversity, true) => "Type diversity".into(),
        (TypeDiversity, false) => "Typen-Vielfalt".into(),
        (BreadcrumbHierarchy, true) => "Breadcrumb hierarchy".into(),
        (BreadcrumbHierarchy, false) => "Breadcrumb-Hierarchie".into(),
        (LinkingDensity, true) => "Linking density".into(),
        (LinkingDensity, false) => "Verlinkungsdichte".into(),
        (PropertyCompleteness, true) => "Property completeness".into(),
        (PropertyCompleteness, false) => "Eigenschafts-Vollständigkeit".into(),
        (RobotsReachable, true) => "robots.txt reachable".into(),
        (RobotsReachable, false) => "robots.txt erreichbar".into(),
        (NoWildcardBlock, true) => "No wildcard block".into(),
        (NoWildcardBlock, false) => "Kein Wildcard-Block".into(),
        (AiSearchReachable, true) => "AI search reachable".into(),
        (AiSearchReachable, false) => "KI-Suche erreichbar".into(),
        (ExplicitAiPolicy, true) => "Explicit AI policy".into(),
        (ExplicitAiPolicy, false) => "Explizite AI-Policy".into(),
        (SitemapReference, true) => "Sitemap reference".into(),
        (SitemapReference, false) => "Sitemap-Verweis".into(),
        (MetaRobotsAllowed, true) => "Meta robots allowed".into(),
        (MetaRobotsAllowed, false) => "Meta-Robots freigegeben".into(),
    };

    let detail = ai_signal_detail(kind, present, values, en);
    (name, detail)
}

fn ai_signal_detail(
    kind: AiSignalKind,
    present: bool,
    values: &AiSignalValues,
    en: bool,
) -> String {
    use AiSignalKind::*;
    match kind {
        HeadingStructure => readability::detail_heading_structure(present, values, en),
        ContentVolume => readability::detail_content_volume(present, values, en),
        ParagraphStructure => readability::detail_paragraph_structure(values, en),
        Lists => readability::detail_lists(present, values, en),
        Tables => readability::detail_tables(present, en),
        SchemaCoverage => readability::detail_schema_coverage(values, en),
        ExtractableAnswers => readability::detail_extractable_answers(values, en),
        SummaryMeta => readability::detail_summary_meta(present, values, en),
        LanguageDeclaration => readability::detail_language_declaration(present, en),
        DefinitionPatterns => readability::detail_definition_patterns(present, en),
        Encryption => citation::detail_encryption(present, en),
        PublisherIdentity => citation::detail_publisher_identity(values, en),
        ArticleStructure => citation::detail_article_structure(present, en),
        PublicationDate => citation::detail_publication_date(present, en),
        CanonicalUrl => citation::detail_canonical_url(present, en),
        SnippetQuality => citation::detail_snippet_quality(values, en),
        QuestionAnswerPattern => citation::detail_question_answer(present, en),
        ContentDepth => citation::detail_content_depth(present, values, en),
        SharingMetadata => citation::detail_sharing_metadata(present, en),
        ThematicContext => citation::detail_thematic_context(present, en),
        SectionCount => chunks::detail_section_count(values, en),
        SectionLength => chunks::detail_section_length(values, en),
        NoOversizedSections => chunks::detail_no_oversized(present, values, en),
        FewFragments => chunks::detail_few_fragments(present, values, en),
        HierarchicalStructure => chunks::detail_hierarchical(present, en),
        SemanticHtml => chunks::detail_semantic_html(present, en),
        SectionDensity => chunks::detail_section_density(present, values, en),
        ContentBoundary => chunks::detail_content_boundary(present, en),
        EntitiesDetected => knowledge_graph::detail_entities_detected(present, values, en),
        SchemaOrgEntities => knowledge_graph::detail_schema_entities(present, values, en),
        Relationships => knowledge_graph::detail_relationships(present, values, en),
        TypeDiversity => knowledge_graph::detail_type_diversity(present, values, en),
        BreadcrumbHierarchy => knowledge_graph::detail_breadcrumb(present, en),
        LinkingDensity => knowledge_graph::detail_linking_density(present, values, en),
        PropertyCompleteness => knowledge_graph::detail_property_completeness(present, values, en),
        RobotsReachable => detail_robots_reachable(present, en),
        NoWildcardBlock => detail_no_wildcard_block(present, en),
        AiSearchReachable => detail_ai_search_reachable(values, en),
        ExplicitAiPolicy => detail_explicit_ai_policy(present, values, en),
        SitemapReference => detail_sitemap_reference(present, en),
        MetaRobotsAllowed => detail_meta_robots_allowed(present, en),
    }
}

// ─── Analysis entry points ──────────────────────────────────────────────────

/// Derive AI visibility analysis from an existing audit report (single page).
///
/// Bakes canonical English into the struct (JSON canonical). The PDF layer
/// re-derives localized text from each signal's stored `kind` + `values`.
pub fn analyze_ai_visibility(report: &AuditReport) -> AiVisibilityAnalysis {
    let readability_input = build_readability_input(report);
    let readability = readability::analyze_readability(&readability_input);

    let citation_input = build_citation_input(report);
    let citation = citation::analyze_citation(&citation_input);

    let chunk_input = build_chunk_input(report);
    let chunks = chunks::analyze_chunks(&chunk_input);

    let kg_input = build_knowledge_graph_input(report);
    let knowledge_graph = knowledge_graph::analyze_knowledge_graph(&kg_input);

    let policy = analyze_policy(report);

    let score = weighted_average(&[
        (readability.dimension.score, 25),
        (citation.dimension.score, 25),
        (chunks.dimension.score, 20),
        (knowledge_graph.dimension.score, 15),
        (policy.dimension.score, 15),
    ]);

    AiVisibilityAnalysis {
        score,
        band: crate::registry::FIVE_BAND
            .label(score as f32, true)
            .to_string(),
        readability,
        citation,
        chunks,
        knowledge_graph,
        policy,
        disclaimer: ai_disclaimer(true),
    }
}

/// Derive AI visibility for batch mode.
///
/// Overall score, dimension scores and signals all describe the same page
/// set: dimension scores are the mean over pages, the overall score is the
/// weighted average of those dimension means, and each signal is the majority
/// outcome over the pages that carry it. A signal's interpolated values cannot
/// be averaged, so they come from an example page named in `example_url`.
/// Per-page lists (sections, entities, relationships, link suggestions) have
/// no aggregate and stay empty. Pages are processed in URL order, so the
/// result does not depend on the order of `reports`.
pub fn analyze_ai_visibility_batch(reports: &[AuditReport]) -> AiVisibilityAnalysis {
    if reports.is_empty() {
        return empty_analysis();
    }

    let mut pages: Vec<(&str, AiVisibilityAnalysis)> = reports
        .iter()
        .map(|r| (r.url.as_str(), analyze_ai_visibility(r)))
        .collect();
    pages.sort_by(|a, b| a.0.cmp(b.0));

    let aggregate = |kind: DimensionKind, pick: fn(&AiVisibilityAnalysis) -> &DimensionScore| {
        aggregate_dimension(kind, pages.iter().map(|(url, a)| (*url, pick(a))).collect())
    };
    let readability = aggregate(DimensionKind::Readability, |a| &a.readability.dimension);
    let citation = aggregate(DimensionKind::Citability, |a| &a.citation.dimension);
    let chunk_dim = aggregate(DimensionKind::Chunks, |a| &a.chunks.dimension);
    let kg_dim = aggregate(DimensionKind::KnowledgeGraph, |a| {
        &a.knowledge_graph.dimension
    });
    let policy_dim = aggregate(DimensionKind::Policy, |a| &a.policy.dimension);

    let score = weighted_average(&[
        (readability.score, 25),
        (citation.score, 25),
        (chunk_dim.score, 20),
        (kg_dim.score, 15),
        (policy_dim.score, 15),
    ]);

    let n = pages.len();
    let majority =
        |f: fn(&PolicyAnalysis) -> bool| pages.iter().filter(|(_, a)| f(&a.policy)).count() * 2 > n;
    let mode = |mut values: Vec<String>| {
        values.sort();
        let mut best = (String::new(), 0);
        for v in &values {
            let count = values.iter().filter(|x| *x == v).count();
            if count > best.1 {
                best = (v.clone(), count);
            }
        }
        best.0
    };

    // Most frequent recommendation; ties go to the earlier entry of this list.
    let kinds = [
        chunks::ChunkRecommendationKind::WellSuited,
        chunks::ChunkRecommendationKind::SplitOversized,
        chunks::ChunkRecommendationKind::TooLittleStructure,
        chunks::ChunkRecommendationKind::Mixed,
    ];
    let recommendation_kind = kinds
        .into_iter()
        .rev()
        .max_by_key(|k| {
            pages
                .iter()
                .filter(|(_, a)| a.chunks.recommendation_kind == *k)
                .count()
        })
        .expect("kinds is non-empty");
    let recommendation_counts = pages.iter().fold((0, 0, 0), |acc, (_, a)| {
        let c = a.chunks.recommendation_counts;
        (acc.0 + c.0, acc.1 + c.1, acc.2 + c.2)
    });

    AiVisibilityAnalysis {
        score,
        band: crate::registry::FIVE_BAND
            .label(score as f32, true)
            .to_string(),
        readability: ReadabilityAnalysis {
            dimension: readability,
        },
        citation: CitationAnalysis {
            dimension: citation,
        },
        chunks: ChunkAnalysis {
            dimension: chunk_dim,
            sections: vec![],
            recommendation_kind,
            recommendation_counts,
            recommendation: chunks::ai_chunk_recommendation(
                recommendation_kind,
                recommendation_counts,
                true,
            ),
        },
        knowledge_graph: KnowledgeGraphAnalysis {
            dimension: kg_dim,
            entities: vec![],
            relationships: vec![],
            link_suggestions: vec![],
        },
        policy: PolicyAnalysis {
            dimension: policy_dim,
            blocks_ai_citation: majority(|p| p.blocks_ai_citation),
            blocks_ai_training: majority(|p| p.blocks_ai_training),
            blocks_all: majority(|p| p.blocks_all),
            blocked_ai_bot_count: (pages
                .iter()
                .map(|(_, a)| a.policy.blocked_ai_bot_count)
                .sum::<usize>() as f64
                / n as f64)
                .round() as usize,
            inferred_policy: mode(
                pages
                    .iter()
                    .map(|(_, a)| a.policy.inferred_policy.clone())
                    .collect(),
            ),
        },
        disclaimer: ai_disclaimer(true),
    }
}

/// One dimension over a URL-sorted page set: mean score, majority signals.
fn aggregate_dimension(kind: DimensionKind, pages: Vec<(&str, &DimensionScore)>) -> DimensionScore {
    let score = (pages.iter().map(|(_, d)| d.score as f64).sum::<f64>() / pages.len() as f64)
        .round() as u32;

    let mut kinds: Vec<AiSignalKind> = Vec::new();
    for (_, d) in &pages {
        for s in &d.signals {
            if !kinds.contains(&s.kind) {
                kinds.push(s.kind);
            }
        }
    }
    kinds.sort_by_key(|k| *k as u32);

    let signals = kinds
        .into_iter()
        .map(|k| {
            let carrying: Vec<(&str, &AiSignal)> = pages
                .iter()
                .filter_map(|(url, d)| d.signals.iter().find(|s| s.kind == k).map(|s| (*url, s)))
                .collect();
            let present = carrying.iter().filter(|(_, s)| s.present).count() * 2 > carrying.len();
            let weight =
                carrying.iter().map(|(_, s)| s.weight).sum::<f32>() / carrying.len() as f32;
            let (url, example) = carrying
                .iter()
                .find(|(_, s)| s.present == present)
                .expect("a majority outcome is carried by at least one page");
            let mut signal = AiSignal::new(k, present, weight, example.values.clone());
            signal.example_url = Some((*url).to_string());
            signal
        })
        .collect();

    DimensionScore {
        kind,
        name: ai_dimension_name(kind, true).to_string(),
        score,
        label: ai_dimension_label(score, true),
        signals,
    }
}

// ─── Policy analysis ────────────────────────────────────────────────────────

fn analyze_policy(report: &AuditReport) -> PolicyAnalysis {
    let mut signals = Vec::new();

    let (
        has_robots,
        blocks_citation,
        blocks_training,
        blocks_all,
        blocked_count,
        has_sitemap_in_robots,
        inferred_policy,
    ) = if let Some(seo) = &report.discoverability.seo {
        if let Some(robots) = &seo.robots {
            let blocked = robots
                .groups
                .iter()
                .filter(|g| {
                    matches!(
                        g.bot_class,
                        crate::seo::robots::BotClass::AiTraining
                            | crate::seo::robots::BotClass::AiCitation
                            | crate::seo::robots::BotClass::AiMixed
                            | crate::seo::robots::BotClass::UnknownAi
                    ) && g.disallows.iter().any(|d| d == "/")
                })
                .count();
            (
                robots.fetched,
                robots.blocks_ai_citation,
                robots.blocks_ai_training,
                robots.has_wildcard_disallow_all,
                blocked,
                !robots.sitemaps.is_empty(),
                robots.inferred_policy.clone(),
            )
        } else {
            (
                false,
                false,
                false,
                false,
                0,
                false,
                "No robots.txt".to_string(),
            )
        }
    } else {
        (false, false, false, false, 0, false, String::new())
    };

    // robots.txt that could not be requested is unknown, not absent: the
    // crawler-access signals then stay out of the dimension (#777).
    let robots_known = report
        .discoverability
        .seo
        .as_ref()
        .is_some_and(|seo| seo.robots.is_some());

    // 1. robots.txt accessible
    if robots_known {
        signals.push(AiSignal::new(
            AiSignalKind::RobotsReachable,
            has_robots,
            0.15,
            AiSignalValues::default(),
        ));

        // 2. Not blocking all crawlers
        signals.push(AiSignal::new(
            AiSignalKind::NoWildcardBlock,
            !blocks_all,
            0.20,
            AiSignalValues::default(),
        ));

        // 3. Citation/search AI bots accessible — only citation bots matter for real-time AI visibility.
        //    Blocking training bots (GPTBot, Google-Extended etc.) is standard practice and not penalized.
        signals.push(AiSignal::new(
            AiSignalKind::AiSearchReachable,
            !blocks_citation,
            0.25,
            AiSignalValues {
                blocks_all: Some(blocks_all),
                blocks_citation: Some(blocks_citation),
                ..Default::default()
            },
        ));

        // 4. Explicit AI policy defined — having any policy is better than silence
        let has_explicit_policy = has_robots && blocked_count > 0;
        signals.push(AiSignal::new(
            AiSignalKind::ExplicitAiPolicy,
            has_explicit_policy,
            0.15,
            AiSignalValues {
                has_robots: Some(has_robots),
                blocked_count: Some(blocked_count as u32),
                inferred_policy: Some(inferred_policy.clone()),
                ..Default::default()
            },
        ));

        // 5. Sitemap in robots.txt (helps AI crawlers discover content)
        signals.push(AiSignal::new(
            AiSignalKind::SitemapReference,
            has_sitemap_in_robots,
            0.15,
            AiSignalValues::default(),
        ));
    }

    // 6. Meta robots (check for noindex/nofollow which affects AI visibility)
    let has_meta_robots_issue = report.discoverability.seo.as_ref().is_some_and(|seo| {
        seo.technical
            .issues
            .iter()
            .any(|i| i.issue_type.contains("noindex") || i.issue_type.contains("nofollow"))
    });
    signals.push(AiSignal::new(
        AiSignalKind::MetaRobotsAllowed,
        !has_meta_robots_issue,
        0.10,
        AiSignalValues::default(),
    ));

    PolicyAnalysis {
        dimension: build_dimension(DimensionKind::Policy, &signals),
        blocks_ai_citation: blocks_citation,
        blocks_ai_training: blocks_training,
        blocks_all,
        blocked_ai_bot_count: blocked_count,
        inferred_policy,
    }
}

// ─── Policy signal detail text ───────────────────────────────────────────────

fn detail_robots_reachable(present: bool, en: bool) -> String {
    if present {
        if en {
            "robots.txt present and readable".into()
        } else {
            "robots.txt vorhanden und lesbar".into()
        }
    } else if en {
        "robots.txt not reachable — AI crawlers have no policy info".into()
    } else {
        "robots.txt nicht erreichbar — AI-Crawler haben keine Policy-Info".into()
    }
}

fn detail_no_wildcard_block(present: bool, en: bool) -> String {
    // present == !blocks_all
    if !present {
        if en {
            "Disallow: / for all crawlers — blocks AI systems too".into()
        } else {
            "Disallow: / für alle Crawler — blockiert auch AI-Systeme".into()
        }
    } else if en {
        "No global block — crawlers generally have access".into()
    } else {
        "Kein globaler Block — Crawler haben grundsätzlich Zugang".into()
    }
}

fn detail_ai_search_reachable(values: &AiSignalValues, en: bool) -> String {
    let blocks_all = values.blocks_all.unwrap_or(false);
    let blocks_citation = values.blocks_citation.unwrap_or(false);
    if blocks_all {
        if en {
            "All crawlers blocked (Disallow: *) — AI search blocked too".into()
        } else {
            "Alle Crawler gesperrt (Disallow: *) — auch KI-Suche blockiert".into()
        }
    } else if blocks_citation {
        if en {
            "AI search bots (PerplexityBot, Amazonbot etc.) blocked — content does not appear in AI answers".into()
        } else {
            "KI-Suchbots (PerplexityBot, Amazonbot etc.) blockiert — Inhalte erscheinen nicht in KI-Antworten".into()
        }
    } else if en {
        "AI search bots not blocked — content available for AI answers".into()
    } else {
        "KI-Suchbots nicht blockiert — Inhalte für KI-Antworten verfügbar".into()
    }
}

fn detail_explicit_ai_policy(present: bool, values: &AiSignalValues, en: bool) -> String {
    let has_robots = values.has_robots.unwrap_or(false);
    let blocked_count = values.blocked_count.unwrap_or(0);
    let inferred_policy = values.inferred_policy.as_deref().unwrap_or("");
    if present {
        if en {
            format!(
                "Policy defined: {} — {} AI bots addressed",
                inferred_policy, blocked_count
            )
        } else {
            format!(
                "Policy definiert: {} — {} AI-Bots adressiert",
                inferred_policy, blocked_count
            )
        }
    } else if has_robots {
        if en {
            "No explicit AI crawler rules in robots.txt".into()
        } else {
            "Keine explizite AI-Crawler-Regelung in robots.txt".into()
        }
    } else if en {
        "No robots.txt — no AI policy definable".into()
    } else {
        "Keine robots.txt — keine AI-Policy definierbar".into()
    }
}

fn detail_sitemap_reference(present: bool, en: bool) -> String {
    if present {
        if en {
            "Sitemap linked in robots.txt — eases AI crawling".into()
        } else {
            "Sitemap in robots.txt verlinkt — erleichtert AI-Crawling".into()
        }
    } else if en {
        "No sitemap in robots.txt — AI crawlers must discover pages themselves".into()
    } else {
        "Keine Sitemap in robots.txt — AI-Crawler müssen Seiten selbst entdecken".into()
    }
}

fn detail_meta_robots_allowed(present: bool, en: bool) -> String {
    // present == !has_meta_robots_issue
    if !present {
        if en {
            "noindex/nofollow detected — page avoided by AI crawlers".into()
        } else {
            "noindex/nofollow erkannt — Seite wird von AI-Crawlern gemieden".into()
        }
    } else if en {
        "No indexing block via meta tags".into()
    } else {
        "Keine Indexierungs-Blockade via Meta-Tags".into()
    }
}

// ─── Input builders ─────────────────────────────────────────────────────────

fn build_readability_input(report: &AuditReport) -> readability::ReadabilityInput {
    let seo = report.discoverability.seo.as_ref();

    let word_count = seo.map_or(0, |s| s.technical.word_count);
    let headings = seo.map_or(&[][..], |s| &s.headings.headings);
    let heading_count = headings.len();
    let max_heading_depth = headings.iter().map(|h| h.level as u32).max().unwrap_or(0);

    let has_schema = seo.is_some_and(|s| s.structured_data.has_structured_data);
    let schema_types = seo.map_or(&[][..], |s| &s.structured_data.types);
    let schema_type_count = schema_types.len();
    let has_faq_schema = schema_types
        .iter()
        .any(|t| matches!(t, SchemaType::FAQPage));
    let has_howto_schema = schema_types.iter().any(|t| matches!(t, SchemaType::HowTo));

    let has_meta_description = seo
        .and_then(|s| s.meta.description.as_ref())
        .is_some_and(|d| !d.is_empty());
    let meta_desc_len = seo
        .and_then(|s| s.meta.description.as_ref())
        .map_or(0, |d| d.len());

    let has_lang = seo.is_some_and(|s| s.technical.has_lang);

    // Estimate paragraph count and avg length from word count and heading count
    let paragraph_count = if heading_count > 0 {
        heading_count as u32 * 2 // rough estimate: ~2 paragraphs per section
    } else if word_count > 100 {
        (word_count / 80).max(1) // rough estimate: ~80 words per paragraph
    } else {
        1
    };
    let avg_paragraph_len = word_count
        .checked_div(paragraph_count)
        .unwrap_or(word_count);

    // Lists/tables not directly available from technical SEO — use heuristics
    // If a page has FAQ/HowTo schema it likely has lists; otherwise estimate from word count
    let has_lists = has_faq_schema || word_count > 500;
    let list_count = if has_faq_schema {
        2
    } else if word_count > 500 {
        1
    } else {
        0
    };
    let has_tables = false; // conservative: not detected without CDP

    readability::ReadabilityInput {
        word_count,
        heading_count,
        max_heading_depth,
        has_lists,
        list_count,
        has_tables,
        has_schema,
        schema_type_count,
        has_faq_schema,
        has_howto_schema,
        has_meta_description,
        meta_desc_len,
        has_lang,
        paragraph_count,
        avg_paragraph_len,
        has_definition_patterns: false, // would need content analysis
    }
}

fn build_citation_input(report: &AuditReport) -> citation::CitationInput {
    let seo = report.discoverability.seo.as_ref();
    let schema_types = seo.map_or(&[][..], |s| &s.structured_data.types);

    let has_author_schema = schema_types.iter().any(|t| matches!(t, SchemaType::Person));
    let has_org_schema = schema_types.iter().any(SchemaType::is_organization_like);
    let has_article_schema = schema_types.iter().any(|t| {
        matches!(
            t,
            SchemaType::Article | SchemaType::BlogPosting | SchemaType::NewsArticle
        )
    });
    let has_faq_schema = schema_types
        .iter()
        .any(|t| matches!(t, SchemaType::FAQPage));
    let has_breadcrumb = schema_types
        .iter()
        .any(|t| matches!(t, SchemaType::BreadcrumbList));

    let has_canonical = seo.is_some_and(|s| s.technical.has_canonical);
    let has_og_meta = seo.is_some_and(|s| {
        s.social
            .open_graph
            .as_ref()
            .is_some_and(|og| og.title.is_some() && og.description.is_some())
    });

    let word_count = seo.map_or(0, |s| s.technical.word_count);
    let heading_count = seo.map_or(0, |s| s.headings.headings.len());

    // Check for datePublished in JSON-LD
    let has_date_published = seo.is_some_and(|s| {
        s.structured_data.json_ld.iter().any(|ld| {
            ld.content.get("datePublished").is_some() || ld.content.get("dateCreated").is_some()
        })
    });

    let has_lists = seo.is_some() && word_count > 500;

    // Estimate short paragraph ratio
    let paragraph_count = if heading_count > 0 {
        heading_count as u32 * 2
    } else if word_count > 100 {
        (word_count / 80).max(1)
    } else {
        1
    };
    let avg_len = word_count
        .checked_div(paragraph_count)
        .unwrap_or(word_count);
    let short_paragraph_ratio = if avg_len <= 100 {
        0.6
    } else if avg_len <= 150 {
        0.4
    } else {
        0.2
    };

    citation::CitationInput {
        has_https: report.url.starts_with("https://"),
        has_author_schema,
        has_org_schema,
        has_article_schema,
        has_canonical,
        has_og_meta,
        word_count,
        heading_count,
        has_faq_schema,
        has_lists,
        short_paragraph_ratio,
        has_date_published,
        has_breadcrumb,
    }
}

fn build_chunk_input(report: &AuditReport) -> chunks::ChunkInput {
    let seo = report.discoverability.seo.as_ref();

    let headings: Vec<chunks::HeadingInfo> = seo.map_or(vec![], |s| {
        s.headings
            .headings
            .iter()
            .map(|heading| chunks::HeadingInfo {
                text: heading.text.clone(),
                level: heading.level as u32,
                word_count_after: heading.word_count_after,
            })
            .collect()
    });

    let total_word_count = seo.map_or(0, |s| s.technical.word_count);

    // A unique <main> region, as the SEO text extraction found it. This used
    // to be "no landmark violations" from the accessibility result — an
    // accessibility finding moving an indicator a second time (plan 105).
    let has_main_region = seo.is_some_and(|s| s.technical.text_excerpt_source == "main");
    // article/section tags not directly available from technical SEO extraction.
    // Heuristic: if page has article schema, it likely uses <article> markup.
    let has_article_tag = seo.is_some_and(|s| {
        s.structured_data.types.iter().any(|t| {
            matches!(
                t,
                SchemaType::Article | SchemaType::BlogPosting | SchemaType::NewsArticle
            )
        })
    });
    let has_section_tags = headings.len() >= 3; // pages with 3+ headings likely use <section>
    let has_semantic_html = has_main_region && (has_article_tag || has_section_tags);

    chunks::ChunkInput {
        headings,
        total_word_count,
        has_semantic_html,
        has_article_tag,
        has_section_tags,
    }
}

fn build_knowledge_graph_input(report: &AuditReport) -> knowledge_graph::KnowledgeGraphInput {
    let seo = report.discoverability.seo.as_ref();

    let schemas: Vec<knowledge_graph::SchemaEntity> = seo.map_or(vec![], |s| {
        s.structured_data
            .json_ld
            .iter()
            .map(|ld| {
                let name = ld
                    .content
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let mut properties = Vec::new();

                // Extract common properties
                for key in &[
                    "author",
                    "publisher",
                    "creator",
                    "provider",
                    "isPartOf",
                    "mainEntityOfPage",
                    "about",
                    "url",
                    "description",
                ] {
                    if let Some(val) = ld.content.get(*key) {
                        let val_str = if let Some(s) = val.as_str() {
                            s.to_string()
                        } else if let Some(obj) = val.as_object() {
                            obj.get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or_default()
                                .to_string()
                        } else {
                            continue;
                        };
                        if !val_str.is_empty() {
                            properties.push((key.to_string(), val_str));
                        }
                    }
                }

                knowledge_graph::SchemaEntity {
                    schema_type: ld.schema_type.clone(),
                    name,
                    properties,
                }
            })
            .collect()
    });

    let page_title = seo.and_then(|s| s.meta.title.clone());
    let site_name = seo.and_then(|s| {
        s.social
            .open_graph
            .as_ref()
            .and_then(|og| og.site_name.clone())
    });

    let internal_links = seo.map_or(0, |s| s.technical.internal_links);
    let has_breadcrumb = seo.is_some_and(|s| {
        s.structured_data
            .types
            .iter()
            .any(|t| matches!(t, SchemaType::BreadcrumbList))
    });

    knowledge_graph::KnowledgeGraphInput {
        schemas,
        page_title,
        site_name,
        internal_links,
        has_breadcrumb,
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Build a dimension, baking canonical-English name/label.
pub(crate) fn build_dimension(kind: DimensionKind, signals: &[AiSignal]) -> DimensionScore {
    let name = ai_dimension_name(kind, true).to_string();
    if signals.is_empty() {
        return DimensionScore {
            kind,
            name,
            score: 0,
            label: ai_no_data_label(true),
            signals: vec![],
        };
    }

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
        name,
        score,
        label: ai_dimension_label(score, true),
        signals: signals.to_vec(),
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

fn empty_analysis() -> AiVisibilityAnalysis {
    let empty_dim = |kind: DimensionKind| DimensionScore {
        kind,
        name: String::new(),
        score: 0,
        label: ai_no_data_label(true),
        signals: vec![],
    };

    AiVisibilityAnalysis {
        score: 0,
        band: crate::registry::FIVE_BAND.label(0.0, true).to_string(),
        readability: ReadabilityAnalysis {
            dimension: empty_dim(DimensionKind::Readability),
        },
        citation: CitationAnalysis {
            dimension: empty_dim(DimensionKind::Citability),
        },
        chunks: ChunkAnalysis {
            dimension: empty_dim(DimensionKind::Chunks),
            sections: vec![],
            recommendation_kind: chunks::ChunkRecommendationKind::TooLittleStructure,
            recommendation_counts: (0, 0, 0),
            recommendation: String::new(),
        },
        knowledge_graph: KnowledgeGraphAnalysis {
            dimension: empty_dim(DimensionKind::KnowledgeGraph),
            entities: vec![],
            relationships: vec![],
            link_suggestions: vec![],
        },
        policy: PolicyAnalysis {
            dimension: empty_dim(DimensionKind::Policy),
            blocks_ai_citation: false,
            blocks_ai_training: false,
            blocks_all: false,
            blocked_ai_bot_count: 0,
            inferred_policy: String::new(),
        },
        disclaimer: DISCLAIMER_EN.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditReport;
    use crate::audit::ViolationStatistics;
    use crate::cli::WcagLevel;
    use crate::wcag::WcagResults;

    #[test]
    fn unknown_robots_txt_leaves_crawler_signals_out() {
        // #777: robots.txt that could not be requested is unknown, not
        // missing — no "robots.txt not reachable" signal.
        let mut report = minimal_report();
        report.discoverability.seo = Some(crate::seo::SeoAnalysis::default());
        assert!(report
            .discoverability
            .seo
            .as_ref()
            .unwrap()
            .robots
            .is_none());
        let policy = analyze_policy(&report);
        assert!(!policy
            .dimension
            .signals
            .iter()
            .any(|s| s.kind == AiSignalKind::RobotsReachable));

        report.discoverability.seo.as_mut().unwrap().robots =
            Some(crate::seo::robots::RobotsAudit::default());
        let policy = analyze_policy(&report);
        assert!(policy
            .dimension
            .signals
            .iter()
            .any(|s| s.kind == AiSignalKind::RobotsReachable && !s.present));
    }

    fn minimal_report() -> AuditReport {
        AuditReport {
            run_context: Default::default(),
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
            html_conform: None,
            experience: crate::audit::ExperienceSection::default(),
            ux: None,
            journey: None,
            discoverability: crate::audit::DiscoverabilitySection {
                seo: None,
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
    fn test_minimal_report_produces_scores() {
        let report = minimal_report();
        let analysis = analyze_ai_visibility(&report);
        assert!(analysis.score <= 100);
        assert!(!analysis.disclaimer.is_empty());
        assert!(!analysis.band.is_empty());
    }

    #[test]
    fn test_empty_batch_returns_zero() {
        let analysis = analyze_ai_visibility_batch(&[]);
        assert_eq!(analysis.score, 0);
    }

    #[test]
    fn test_batch_with_reports() {
        let reports = vec![minimal_report(), minimal_report()];
        let analysis = analyze_ai_visibility_batch(&reports);
        assert!(analysis.score <= 100);
    }

    /// Three strongly different pages: no SEO data, a rich page with a
    /// reachable robots.txt, and a thin page with a robots.txt that blocks all.
    fn different_pages() -> Vec<AuditReport> {
        use crate::seo::{
            HeadingStructure, MetaTags, SeoAnalysis, SocialTags, StructuredData, TechnicalSeo,
        };
        let seo = |word_count: u32, has_lang: bool, desc: Option<&str>| {
            let technical = TechnicalSeo {
                word_count,
                has_lang,
                ..Default::default()
            };
            let meta = MetaTags {
                description: desc.map(str::to_string),
                ..Default::default()
            };
            SeoAnalysis {
                meta,
                headings: HeadingStructure::default(),
                technical,
                social: SocialTags::default(),
                structured_data: StructuredData::default(),
                score: 50,
                content_profile: None,
                robots: None,
                page_health: None,
                serp: None,
                meta_issues: vec![],
                image_efficiency: None,
                indexability: crate::seo::IndexabilityCheck::default(),
            }
        };

        let bare = minimal_report();

        let mut rich = minimal_report();
        rich.url = "https://example.com/rich".into();
        let mut rich_seo = seo(
            2400,
            true,
            Some("A long and useful description of the page."),
        );
        rich_seo.robots = Some(crate::seo::robots::RobotsAudit {
            fetched: true,
            ..Default::default()
        });
        rich.discoverability.seo = Some(rich_seo);

        let mut thin = minimal_report();
        thin.url = "https://example.com/thin".into();
        let mut thin_seo = seo(40, false, None);
        thin_seo.robots = Some(crate::seo::robots::RobotsAudit {
            fetched: true,
            has_wildcard_disallow_all: true,
            blocks_ai_citation: true,
            ..Default::default()
        });
        thin.discoverability.seo = Some(thin_seo);

        vec![bare, rich, thin]
    }

    #[test]
    fn batch_is_invariant_under_page_order_and_consistent_with_its_dimensions() {
        let reports = different_pages();
        let singles: Vec<AiVisibilityAnalysis> =
            reports.iter().map(analyze_ai_visibility).collect();
        // The pages must actually differ, or the test proves nothing.
        assert!(singles.iter().any(|a| a.score != singles[0].score));
        assert!(singles
            .iter()
            .any(|a| a.readability.dimension.score != singles[0].readability.dimension.score));

        let permutations: [[usize; 3]; 6] = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        let json_of = |order: &[usize; 3]| {
            let ordered: Vec<AuditReport> = order.iter().map(|&i| reports[i].clone()).collect();
            serde_json::to_string(&analyze_ai_visibility_batch(&ordered)).unwrap()
        };
        let expected = json_of(&permutations[0]);
        for order in &permutations {
            assert_eq!(json_of(order), expected, "order {order:?}");
        }

        let batch = analyze_ai_visibility_batch(&reports);
        let mean = |pick: fn(&AiVisibilityAnalysis) -> u32| {
            (singles.iter().map(|a| pick(a) as f64).sum::<f64>() / singles.len() as f64).round()
                as u32
        };
        assert_eq!(
            batch.readability.dimension.score,
            mean(|a| a.readability.dimension.score)
        );
        assert_eq!(
            batch.citation.dimension.score,
            mean(|a| a.citation.dimension.score)
        );
        assert_eq!(
            batch.chunks.dimension.score,
            mean(|a| a.chunks.dimension.score)
        );
        assert_eq!(
            batch.knowledge_graph.dimension.score,
            mean(|a| a.knowledge_graph.dimension.score)
        );
        assert_eq!(
            batch.policy.dimension.score,
            mean(|a| a.policy.dimension.score)
        );
        assert_eq!(
            batch.score,
            weighted_average(&[
                (batch.readability.dimension.score, 25),
                (batch.citation.dimension.score, 25),
                (batch.chunks.dimension.score, 20),
                (batch.knowledge_graph.dimension.score, 15),
                (batch.policy.dimension.score, 15),
            ])
        );
        // Same page set as the per-page mean of the overall score (rounding only).
        assert!(batch.score.abs_diff(mean(|a| a.score)) <= 1);
    }

    #[test]
    fn batch_signals_are_majority_outcomes_with_an_example_url() {
        let reports = different_pages();
        let batch = analyze_ai_visibility_batch(&reports);
        for s in batch
            .readability
            .dimension
            .signals
            .iter()
            .chain(&batch.policy.dimension.signals)
        {
            let url = s
                .example_url
                .as_deref()
                .expect("aggregated signal names an example");
            assert!(reports.iter().any(|r| r.url == url));
        }
        // Wildcard block exists on 1 of 2 pages carrying the signal: not a majority.
        let wildcard = batch
            .policy
            .dimension
            .signals
            .iter()
            .find(|s| s.kind == AiSignalKind::NoWildcardBlock)
            .unwrap();
        assert!(!wildcard.present);
        assert!(!batch.policy.blocks_all);
    }

    #[test]
    fn band_replaces_the_letter_grade_this_module_no_longer_carries() {
        // Plan 29, D2: an indicator of weight 0 gets a band word, not A-F.
        let band = |s: f32| crate::registry::FIVE_BAND.label(s, true);
        assert_eq!(band(95.0), "Excellent");
        assert_eq!(band(80.0), "Good");
        assert_eq!(band(65.0), "Needs improvement");
        assert_eq!(band(45.0), "Inadequate");
        assert_eq!(band(20.0), "Critical");
    }

    #[test]
    fn test_weighted_average() {
        assert_eq!(weighted_average(&[(100, 50), (0, 50)]), 50);
        assert_eq!(weighted_average(&[(100, 100)]), 100);
        assert_eq!(weighted_average(&[]), 0);
    }

    fn contains_german(s: &str) -> bool {
        s.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß'])
    }

    #[test]
    fn struct_is_canonical_english() {
        use crate::seo::{
            HeadingStructure, MetaTags, SeoAnalysis, SocialTags, StructuredData, TechnicalSeo,
        };
        // Bare SEO so every "missing"/"weak" branch contributes a string.
        let mut report = minimal_report();
        report.discoverability.seo = Some(SeoAnalysis {
            indexability: Default::default(),
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

        // Analysis bakes canonical English regardless of run locale.
        let analysis = analyze_ai_visibility(&report);
        assert!(!contains_german(&analysis.disclaimer));

        let dims = [
            &analysis.readability.dimension,
            &analysis.citation.dimension,
            &analysis.chunks.dimension,
            &analysis.knowledge_graph.dimension,
            &analysis.policy.dimension,
        ];
        for dim in dims {
            assert!(!contains_german(&dim.name), "dim name: {:?}", dim.name);
            assert!(!contains_german(&dim.label), "dim label: {:?}", dim.label);
            for s in &dim.signals {
                assert!(!contains_german(&s.name), "signal name: {:?}", s.name);
                assert!(!contains_german(&s.detail), "signal detail: {:?}", s.detail);
            }
        }
        assert!(!contains_german(&analysis.chunks.recommendation));
        for sec in &analysis.chunks.sections {
            assert!(!contains_german(&sec.heading));
        }
        for s in &analysis.knowledge_graph.link_suggestions {
            assert!(!contains_german(&s.reason));
        }
    }

    #[test]
    fn pdf_layer_re_derives_german() {
        // Dimension name + label + disclaimer + signal text all localize.
        assert_eq!(
            ai_dimension_name(DimensionKind::Readability, false),
            "KI-Lesbarkeit"
        );
        assert_eq!(ai_dimension_label(95, false), "Sehr gut");
        assert!(ai_disclaimer(false).contains("heuristischen"));

        let (name, detail) = ai_signal_text(
            AiSignalKind::AiSearchReachable,
            false,
            &AiSignalValues {
                blocks_all: Some(false),
                blocks_citation: Some(true),
                ..Default::default()
            },
            false,
        );
        assert_eq!(name, "KI-Suche erreichbar");
        assert!(detail.contains("KI-Suchbots"));
    }
}
