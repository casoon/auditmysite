//! Content Chunk Optimizer
//!
//! Analyzes page content structure for optimal chunking in RAG/embedding
//! pipelines. Evaluates heading-based segmentation, section lengths,
//! and semantic coherence heuristics.

use serde::{Deserialize, Serialize};

use super::{
    build_dimension, AiSignal, AiSignalKind, AiSignalValues, DimensionKind, DimensionScore,
};

/// Which chunk-strategy recommendation applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkRecommendationKind {
    /// Content well suited — heading-based chunking
    WellSuited,
    /// Oversized sections should be split (uses `too_long_count`)
    SplitOversized,
    /// Too little structure for effective chunking
    TooLittleStructure,
    /// Mixed structure (uses optimal/too_short/too_long counts)
    Mixed,
}

/// Content chunk analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkAnalysis {
    /// Dimension score (0–100) with individual signals
    pub dimension: DimensionScore,
    /// Detected content sections with their properties
    pub sections: Vec<ContentSection>,
    /// Which recommendation applies (for localized re-derivation)
    pub recommendation_kind: ChunkRecommendationKind,
    /// Counts referenced by `recommendation` (optimal, too_short, too_long)
    pub recommendation_counts: (u32, u32, u32),
    /// Recommended chunk strategy (canonical English)
    pub recommendation: String,
}

/// A synthetic (non-content-derived) section heading that needs localization.
///
/// Real headings come straight from page content and are language-agnostic;
/// these two are tool-generated labels and must be re-derivable for the PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkSectionKind {
    /// "Entire content" — the whole page when no headings exist
    EntireContent,
    /// "Introduction" — content before the first heading
    Introduction,
}

/// A detected content section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSection {
    /// Synthetic heading kind, if this is a tool-generated label (for
    /// localized re-derivation). `None` for real content headings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_kind: Option<ChunkSectionKind>,
    /// Section heading (canonical English for synthetic labels, otherwise the
    /// page's own heading text)
    pub heading: String,
    /// Heading level (1-6, 0 for intro)
    pub level: u32,
    /// Estimated word count
    pub word_count: u32,
    /// Quality assessment
    pub quality: ChunkQuality,
}

/// Quality of a content section as a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkQuality {
    /// Optimal size (100–800 words)
    Optimal,
    /// Too short for meaningful embedding (< 100 words)
    TooShort,
    /// Too long, should be split (> 800 words)
    TooLong,
    /// Acceptable but not ideal
    Acceptable,
}

impl std::fmt::Display for ChunkQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkQuality::Optimal => write!(f, "Optimal"),
            ChunkQuality::TooShort => write!(f, "Zu kurz"),
            ChunkQuality::TooLong => write!(f, "Zu lang"),
            ChunkQuality::Acceptable => write!(f, "Akzeptabel"),
        }
    }
}

/// Input data for chunk analysis
pub(crate) struct ChunkInput {
    pub headings: Vec<HeadingInfo>,
    pub total_word_count: u32,
    pub has_semantic_html: bool,
    pub has_article_tag: bool,
    pub has_section_tags: bool,
}

/// A heading found in the content
pub(crate) struct HeadingInfo {
    pub text: String,
    pub level: u32,
    pub word_count_after: u32,
}

pub(crate) fn analyze_chunks(input: &ChunkInput) -> ChunkAnalysis {
    // Build sections from headings
    let sections = build_sections(input);
    let section_count = sections.len();

    let optimal_count = sections
        .iter()
        .filter(|s| s.quality == ChunkQuality::Optimal)
        .count();
    let too_long_count = sections
        .iter()
        .filter(|s| s.quality == ChunkQuality::TooLong)
        .count();
    let too_short_count = sections
        .iter()
        .filter(|s| s.quality == ChunkQuality::TooShort)
        .count();

    let mut signals = Vec::new();

    // 1. Section count — enough sections for meaningful chunks
    let good_section_count = (3..=30).contains(&section_count);
    signals.push(AiSignal::new(
        AiSignalKind::SectionCount,
        good_section_count,
        0.15,
        AiSignalValues {
            section_count: Some(section_count as u32),
            ..Default::default()
        },
    ));

    // 2. Optimal chunk ratio
    let optimal_ratio = if section_count > 0 {
        optimal_count as f32 / section_count as f32
    } else {
        0.0
    };
    signals.push(AiSignal::new(
        AiSignalKind::SectionLength,
        optimal_ratio >= 0.5,
        0.20,
        AiSignalValues {
            optimal_count: Some(optimal_count as u32),
            section_count: Some(section_count as u32),
            ..Default::default()
        },
    ));

    // 3. No oversized sections
    let no_oversized = too_long_count == 0;
    signals.push(AiSignal::new(
        AiSignalKind::NoOversizedSections,
        no_oversized,
        0.15,
        AiSignalValues {
            too_long_count: Some(too_long_count as u32),
            ..Default::default()
        },
    ));

    // 4. Minimal fragment ratio
    let low_fragment = too_short_count as f32 / (section_count.max(1) as f32) < 0.3;
    signals.push(AiSignal::new(
        AiSignalKind::FewFragments,
        low_fragment,
        0.10,
        AiSignalValues {
            too_short_count: Some(too_short_count as u32),
            ..Default::default()
        },
    ));

    // 5. Heading hierarchy — clean hierarchy enables tree-based chunking
    let has_hierarchy =
        input.headings.iter().any(|h| h.level >= 2) && input.headings.iter().any(|h| h.level >= 3);
    signals.push(AiSignal::new(
        AiSignalKind::HierarchicalStructure,
        has_hierarchy,
        0.10,
        AiSignalValues::default(),
    ));

    // 6. Semantic HTML usage
    signals.push(AiSignal::new(
        AiSignalKind::SemanticHtml,
        input.has_semantic_html,
        0.10,
        AiSignalValues::default(),
    ));

    // 7. Content/word density per section
    let avg_words = if section_count > 0 {
        input.total_word_count / section_count as u32
    } else {
        input.total_word_count
    };
    let good_density = (100..=500).contains(&avg_words);
    signals.push(AiSignal::new(
        AiSignalKind::SectionDensity,
        good_density,
        0.10,
        AiSignalValues {
            avg_words: Some(avg_words),
            ..Default::default()
        },
    ));

    // 8. Article/section tags
    let has_article = input.has_article_tag || input.has_section_tags;
    signals.push(AiSignal::new(
        AiSignalKind::ContentBoundary,
        has_article,
        0.10,
        AiSignalValues::default(),
    ));

    // Build recommendation
    let recommendation_kind = if optimal_ratio >= 0.7 && no_oversized {
        ChunkRecommendationKind::WellSuited
    } else if too_long_count > 0 && has_hierarchy {
        ChunkRecommendationKind::SplitOversized
    } else if section_count < 3 {
        ChunkRecommendationKind::TooLittleStructure
    } else {
        ChunkRecommendationKind::Mixed
    };
    let recommendation_counts = (
        optimal_count as u32,
        too_short_count as u32,
        too_long_count as u32,
    );
    let recommendation = ai_chunk_recommendation(recommendation_kind, recommendation_counts, true);

    ChunkAnalysis {
        dimension: build_dimension(DimensionKind::Chunks, &signals),
        sections,
        recommendation_kind,
        recommendation_counts,
        recommendation,
    }
}

/// Localized chunk-strategy recommendation (single source of truth).
pub fn ai_chunk_recommendation(
    kind: ChunkRecommendationKind,
    counts: (u32, u32, u32),
    en: bool,
) -> String {
    let (optimal_count, too_short_count, too_long_count) = counts;
    match kind {
        ChunkRecommendationKind::WellSuited => {
            if en {
                "Content is well suited for RAG/embedding pipelines. Heading-based chunking recommended."
                    .to_string()
            } else {
                "Content ist gut für RAG/Embedding-Pipelines geeignet. Heading-basiertes Chunking empfohlen."
                    .to_string()
            }
        }
        ChunkRecommendationKind::SplitOversized => {
            if en {
                format!(
                    "{} oversized sections should be split at H3/H4 boundaries. \
                     Recursive splitting by heading level recommended.",
                    too_long_count
                )
            } else {
                format!(
                    "{} übergroße Abschnitte sollten an H3/H4-Grenzen aufgeteilt werden. \
                     Rekursives Splitting nach Heading-Level empfohlen.",
                    too_long_count
                )
            }
        }
        ChunkRecommendationKind::TooLittleStructure => {
            if en {
                "Too little structure for effective chunking. \
                 Additional subheadings would improve extractability."
                    .to_string()
            } else {
                "Zu wenig Gliederung für effektives Chunking. \
                 Zusätzliche Zwischenüberschriften würden die Extrahierbarkeit verbessern."
                    .to_string()
            }
        }
        ChunkRecommendationKind::Mixed => {
            if en {
                format!(
                    "Mixed content structure: {} sections optimal, {} too short, {} too long. \
                     More subheadings improve readability for AI systems.",
                    optimal_count, too_short_count, too_long_count
                )
            } else {
                format!(
                    "Gemischte Inhaltsstruktur: {} Abschnitte optimal, {} zu kurz, {} zu lang. \
                     Mehr Zwischenüberschriften verbessern die Lesbarkeit für KI-Systeme.",
                    optimal_count, too_short_count, too_long_count
                )
            }
        }
    }
}

/// Localized heading for a synthetic section kind.
pub fn ai_chunk_section_heading(kind: ChunkSectionKind, en: bool) -> String {
    match (kind, en) {
        (ChunkSectionKind::EntireContent, true) => "Entire content".into(),
        (ChunkSectionKind::EntireContent, false) => "Gesamter Inhalt".into(),
        (ChunkSectionKind::Introduction, true) => "Introduction".into(),
        (ChunkSectionKind::Introduction, false) => "Einleitung".into(),
    }
}

// ─── Signal detail text (single source of truth) ─────────────────────────────

pub(crate) fn detail_section_count(v: &AiSignalValues, en: bool) -> String {
    let section_count = v.section_count.unwrap_or(0);
    if section_count < 3 {
        if en {
            format!(
                "Only {} sections — too few for granular chunk formation",
                section_count
            )
        } else {
            format!(
                "Nur {} Abschnitte — zu wenig für granulare Chunk-Bildung",
                section_count
            )
        }
    } else if section_count > 30 {
        if en {
            format!(
                "{} sections — very fragmented, may scatter context",
                section_count
            )
        } else {
            format!(
                "{} Abschnitte — sehr fragmentiert, kann Kontext verteilen",
                section_count
            )
        }
    } else if en {
        format!("{} sections — good granularity for chunks", section_count)
    } else {
        format!(
            "{} Abschnitte — gute Granularität für Chunks",
            section_count
        )
    }
}

pub(crate) fn detail_section_length(v: &AiSignalValues, en: bool) -> String {
    let optimal_count = v.optimal_count.unwrap_or(0);
    let section_count = v.section_count.unwrap_or(0);
    let ratio = if section_count > 0 {
        optimal_count as f32 / section_count as f32
    } else {
        0.0
    };
    if en {
        format!(
            "Heuristic: {} of {} sections fall in the 100–800 word range ({:.0}%). \
             A guideline, not a standardized metric.",
            optimal_count,
            section_count,
            ratio * 100.0
        )
    } else {
        format!(
            "Heuristik: {} von {} Abschnitten liegen im Bereich 100–800 Wörter ({:.0}%). \
             Richtwert, keine standardisierte Metrik.",
            optimal_count,
            section_count,
            ratio * 100.0
        )
    }
}

pub(crate) fn detail_no_oversized(present: bool, v: &AiSignalValues, en: bool) -> String {
    let too_long_count = v.too_long_count.unwrap_or(0);
    if present {
        if en {
            "No section over 800 words — good for token limits".into()
        } else {
            "Kein Abschnitt über 800 Wörter — gut für Token-Limits".into()
        }
    } else if en {
        format!(
            "{} sections over 800 words — should be split",
            too_long_count
        )
    } else {
        format!(
            "{} Abschnitte über 800 Wörter — sollten aufgeteilt werden",
            too_long_count
        )
    }
}

pub(crate) fn detail_few_fragments(present: bool, v: &AiSignalValues, en: bool) -> String {
    let too_short_count = v.too_short_count.unwrap_or(0);
    if present {
        if en {
            format!(
                "Only {} short sections (<100 words) — little information loss",
                too_short_count
            )
        } else {
            format!(
                "Nur {} Kurzabschnitte (<100 Wörter) — wenig Informationsverlust",
                too_short_count
            )
        }
    } else if en {
        format!(
            "{} short sections — many fragments may lose context",
            too_short_count
        )
    } else {
        format!(
            "{} Kurzabschnitte — viele Fragmente können Kontext verlieren",
            too_short_count
        )
    }
}

pub(crate) fn detail_hierarchical(present: bool, en: bool) -> String {
    if present {
        if en {
            "Multi-level heading hierarchy — recursive chunk strategies possible".into()
        } else {
            "Mehrstufige Heading-Hierarchie — rekursive Chunk-Strategien möglich".into()
        }
    } else if en {
        "Flat heading structure — only sequential chunking possible".into()
    } else {
        "Flache Heading-Struktur — nur sequenzielles Chunking möglich".into()
    }
}

pub(crate) fn detail_semantic_html(present: bool, en: bool) -> String {
    if present {
        if en {
            "Semantic elements (article, section, nav) — eases region detection".into()
        } else {
            "Semantische Elemente (article, section, nav) — erleichtert Bereichs-Erkennung".into()
        }
    } else if en {
        "Hardly any semantic HTML — chunks only heading-based".into()
    } else {
        "Kaum semantisches HTML — Chunks nur heading-basiert möglich".into()
    }
}

pub(crate) fn detail_section_density(present: bool, v: &AiSignalValues, en: bool) -> String {
    let avg_words = v.avg_words.unwrap_or(0);
    if en {
        format!(
            "Avg {} words/section — {}",
            avg_words,
            if present {
                "optimal range for embeddings"
            } else if avg_words < 100 {
                "too thin for substantial embeddings"
            } else {
                "too dense, split recommended"
            }
        )
    } else {
        format!(
            "Ø {} Wörter/Abschnitt — {}",
            avg_words,
            if present {
                "optimaler Bereich für Embeddings"
            } else if avg_words < 100 {
                "zu dünn für gehaltvolle Embeddings"
            } else {
                "zu dicht, Split empfohlen"
            }
        )
    }
}

pub(crate) fn detail_content_boundary(present: bool, en: bool) -> String {
    if present {
        if en {
            "article/section tags present — main content delimitable".into()
        } else {
            "article/section-Tags vorhanden — Hauptinhalt abgrenzbar".into()
        }
    } else if en {
        "No article/section — main content not clearly delimited".into()
    } else {
        "Kein article/section — Hauptinhalt nicht klar abgegrenzt".into()
    }
}

fn build_sections(input: &ChunkInput) -> Vec<ContentSection> {
    let mut sections = Vec::new();

    if input.headings.is_empty() {
        // No headings: the entire content is one chunk
        sections.push(ContentSection {
            heading_kind: Some(ChunkSectionKind::EntireContent),
            heading: ai_chunk_section_heading(ChunkSectionKind::EntireContent, true),
            level: 0,
            word_count: input.total_word_count,
            quality: classify_chunk_size(input.total_word_count),
        });
        return sections;
    }

    // First heading might have content before it
    if let Some(first) = input.headings.first() {
        let intro_words = input.total_word_count.saturating_sub(
            input
                .headings
                .iter()
                .map(|h| h.word_count_after)
                .sum::<u32>(),
        );
        if intro_words > 20 {
            sections.push(ContentSection {
                heading_kind: Some(ChunkSectionKind::Introduction),
                heading: ai_chunk_section_heading(ChunkSectionKind::Introduction, true),
                level: 0,
                word_count: intro_words,
                quality: classify_chunk_size(intro_words),
            });
        }

        // Each heading starts a section
        for h in &input.headings {
            sections.push(ContentSection {
                heading_kind: None,
                heading: if h.text.chars().count() > 80 {
                    format!("{}…", h.text.chars().take(77).collect::<String>())
                } else {
                    h.text.clone()
                },
                level: h.level,
                word_count: h.word_count_after,
                quality: classify_chunk_size(h.word_count_after),
            });
        }

        let _ = first; // suppress unused warning
    }

    sections
}

fn classify_chunk_size(words: u32) -> ChunkQuality {
    match words {
        0..=50 => ChunkQuality::TooShort,
        51..=99 => ChunkQuality::Acceptable,
        100..=800 => ChunkQuality::Optimal,
        801..=1200 => ChunkQuality::Acceptable,
        _ => ChunkQuality::TooLong,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rich_input() -> ChunkInput {
        ChunkInput {
            headings: vec![
                HeadingInfo {
                    text: "Introduction".into(),
                    level: 1,
                    word_count_after: 250,
                },
                HeadingInfo {
                    text: "Core Concepts".into(),
                    level: 2,
                    word_count_after: 400,
                },
                HeadingInfo {
                    text: "Deep Dive".into(),
                    level: 3,
                    word_count_after: 300,
                },
                HeadingInfo {
                    text: "Conclusion".into(),
                    level: 2,
                    word_count_after: 150,
                },
            ],
            total_word_count: 1100,
            has_semantic_html: true,
            has_article_tag: true,
            has_section_tags: true,
        }
    }

    fn minimal_input() -> ChunkInput {
        ChunkInput {
            headings: vec![],
            total_word_count: 0,
            has_semantic_html: false,
            has_article_tag: false,
            has_section_tags: false,
        }
    }

    #[test]
    fn rich_input_produces_high_score() {
        let result = analyze_chunks(&rich_input());
        assert!(result.dimension.score >= 60);
        // Struct carries canonical English.
        assert_eq!(result.dimension.name, "Technical AI readability");
        assert_eq!(result.dimension.kind, DimensionKind::Chunks);
        assert!(!result.sections.is_empty());
    }

    #[test]
    fn minimal_input_produces_single_section_low_score() {
        let result = analyze_chunks(&minimal_input());
        // No headings → one section covering all 0 words
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].heading, "Entire content");
        assert_eq!(
            result.sections[0].heading_kind,
            Some(ChunkSectionKind::EntireContent)
        );
        assert_eq!(result.sections[0].quality, ChunkQuality::TooShort);
        assert!(result.dimension.score <= 30);
        // PDF re-derives the synthetic heading in German.
        assert_eq!(
            ai_chunk_section_heading(ChunkSectionKind::EntireContent, false),
            "Gesamter Inhalt"
        );
    }

    #[test]
    fn oversized_section_penalizes_score() {
        let small_input = ChunkInput {
            headings: vec![
                HeadingInfo {
                    text: "Intro".into(),
                    level: 1,
                    word_count_after: 300,
                },
                HeadingInfo {
                    text: "Body".into(),
                    level: 2,
                    word_count_after: 300,
                },
                HeadingInfo {
                    text: "End".into(),
                    level: 2,
                    word_count_after: 300,
                },
            ],
            total_word_count: 900,
            has_semantic_html: true,
            has_article_tag: true,
            has_section_tags: true,
        };
        let large_input = ChunkInput {
            headings: vec![
                HeadingInfo {
                    text: "Intro".into(),
                    level: 1,
                    word_count_after: 2000,
                },
                HeadingInfo {
                    text: "Body".into(),
                    level: 2,
                    word_count_after: 300,
                },
                HeadingInfo {
                    text: "End".into(),
                    level: 2,
                    word_count_after: 300,
                },
            ],
            total_word_count: 2600,
            has_semantic_html: true,
            has_article_tag: true,
            has_section_tags: true,
        };
        let small_score = analyze_chunks(&small_input).dimension.score;
        let large_score = analyze_chunks(&large_input).dimension.score;
        assert!(small_score > large_score);
    }

    #[test]
    fn classify_chunk_size_boundaries() {
        assert_eq!(classify_chunk_size(0), ChunkQuality::TooShort);
        assert_eq!(classify_chunk_size(50), ChunkQuality::TooShort);
        assert_eq!(classify_chunk_size(51), ChunkQuality::Acceptable);
        assert_eq!(classify_chunk_size(100), ChunkQuality::Optimal);
        assert_eq!(classify_chunk_size(800), ChunkQuality::Optimal);
        assert_eq!(classify_chunk_size(801), ChunkQuality::Acceptable);
        assert_eq!(classify_chunk_size(1200), ChunkQuality::Acceptable);
        assert_eq!(classify_chunk_size(1201), ChunkQuality::TooLong);
    }

    #[test]
    fn hierarchy_requires_both_h2_and_h3() {
        let flat_input = ChunkInput {
            headings: vec![
                HeadingInfo {
                    text: "A".into(),
                    level: 2,
                    word_count_after: 200,
                },
                HeadingInfo {
                    text: "B".into(),
                    level: 2,
                    word_count_after: 200,
                },
                HeadingInfo {
                    text: "C".into(),
                    level: 2,
                    word_count_after: 200,
                },
            ],
            total_word_count: 600,
            has_semantic_html: false,
            has_article_tag: false,
            has_section_tags: false,
        };
        let result = analyze_chunks(&flat_input);
        let hier_signal = result
            .dimension
            .signals
            .iter()
            .find(|s| s.kind == AiSignalKind::HierarchicalStructure)
            .expect("signal must exist");
        // Only H2, no H3 → flat
        assert!(!hier_signal.present);
    }

    #[test]
    fn recommendation_re_derives_german() {
        let de = ai_chunk_recommendation(ChunkRecommendationKind::SplitOversized, (0, 0, 2), false);
        assert!(de.contains("übergroße Abschnitte"));
    }
}
