//! Content Chunk Optimizer
//!
//! Analyzes page content structure for optimal chunking in RAG/embedding
//! pipelines. Evaluates heading-based segmentation, section lengths,
//! and semantic coherence heuristics.

use serde::{Deserialize, Serialize};

use super::{build_dimension, AiSignal, DimensionScore};

/// Content chunk analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkAnalysis {
    /// Dimension score (0–100) with individual signals
    pub dimension: DimensionScore,
    /// Detected content sections with their properties
    pub sections: Vec<ContentSection>,
    /// Recommended chunk strategy
    pub recommendation: String,
}

/// A detected content section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSection {
    /// Section heading (or "Einleitung" for content before first heading)
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
    signals.push(AiSignal {
        name: "Abschnittszahl".into(),
        present: good_section_count,
        weight: 0.15,
        detail: if section_count < 3 {
            format!(
                "Nur {} Abschnitte — zu wenig für granulare Chunk-Bildung",
                section_count
            )
        } else if section_count > 30 {
            format!(
                "{} Abschnitte — sehr fragmentiert, kann Kontext verteilen",
                section_count
            )
        } else {
            format!(
                "{} Abschnitte — gute Granularität für Chunks",
                section_count
            )
        },
    });

    // 2. Optimal chunk ratio
    let optimal_ratio = if section_count > 0 {
        optimal_count as f32 / section_count as f32
    } else {
        0.0
    };
    signals.push(AiSignal {
        name: "Optimale Chunk-Größe".into(),
        present: optimal_ratio >= 0.5,
        weight: 0.20,
        detail: format!(
            "{}/{} Abschnitte haben optimale Länge (100–800 Wörter) — {:.0}%",
            optimal_count,
            section_count,
            optimal_ratio * 100.0
        ),
    });

    // 3. No oversized sections
    let no_oversized = too_long_count == 0;
    signals.push(AiSignal {
        name: "Keine Übergroßen Abschnitte".into(),
        present: no_oversized,
        weight: 0.15,
        detail: if no_oversized {
            "Kein Abschnitt über 800 Wörter — gut für Token-Limits".into()
        } else {
            format!(
                "{} Abschnitte über 800 Wörter — sollten aufgeteilt werden",
                too_long_count
            )
        },
    });

    // 4. Minimal fragment ratio
    let low_fragment = too_short_count as f32 / (section_count.max(1) as f32) < 0.3;
    signals.push(AiSignal {
        name: "Wenig Fragmente".into(),
        present: low_fragment,
        weight: 0.10,
        detail: if low_fragment {
            format!(
                "Nur {} Kurzabschnitte (<100 Wörter) — wenig Informationsverlust",
                too_short_count
            )
        } else {
            format!(
                "{} Kurzabschnitte — viele Fragmente können Kontext verlieren",
                too_short_count
            )
        },
    });

    // 5. Heading hierarchy — clean hierarchy enables tree-based chunking
    let has_hierarchy =
        input.headings.iter().any(|h| h.level >= 2) && input.headings.iter().any(|h| h.level >= 3);
    signals.push(AiSignal {
        name: "Hierarchische Gliederung".into(),
        present: has_hierarchy,
        weight: 0.10,
        detail: if has_hierarchy {
            "Mehrstufige Heading-Hierarchie — rekursive Chunk-Strategien möglich".into()
        } else {
            "Flache Heading-Struktur — nur sequenzielles Chunking möglich".into()
        },
    });

    // 6. Semantic HTML usage
    signals.push(AiSignal {
        name: "Semantisches HTML".into(),
        present: input.has_semantic_html,
        weight: 0.10,
        detail: if input.has_semantic_html {
            "Semantische Elemente (article, section, nav) — erleichtert Bereichs-Erkennung".into()
        } else {
            "Kaum semantisches HTML — Chunks nur heading-basiert möglich".into()
        },
    });

    // 7. Content/word density per section
    let avg_words = if section_count > 0 {
        input.total_word_count / section_count as u32
    } else {
        input.total_word_count
    };
    let good_density = (100..=500).contains(&avg_words);
    signals.push(AiSignal {
        name: "Abschnittsdichte".into(),
        present: good_density,
        weight: 0.10,
        detail: format!(
            "Ø {} Wörter/Abschnitt — {}",
            avg_words,
            if good_density {
                "optimaler Bereich für Embeddings"
            } else if avg_words < 100 {
                "zu dünn für gehaltvolle Embeddings"
            } else {
                "zu dicht, Split empfohlen"
            }
        ),
    });

    // 8. Article/section tags
    let has_article = input.has_article_tag || input.has_section_tags;
    signals.push(AiSignal {
        name: "Content-Begrenzung".into(),
        present: has_article,
        weight: 0.10,
        detail: if has_article {
            "article/section-Tags vorhanden — Hauptinhalt abgrenzbar".into()
        } else {
            "Kein article/section — Hauptinhalt nicht klar abgegrenzt".into()
        },
    });

    // Build recommendation
    let recommendation = if optimal_ratio >= 0.7 && no_oversized {
        "Content ist gut für RAG/Embedding-Pipelines geeignet. Heading-basiertes Chunking empfohlen."
            .to_string()
    } else if too_long_count > 0 && has_hierarchy {
        format!(
            "{} übergroße Abschnitte sollten an H3/H4-Grenzen aufgeteilt werden. \
             Rekursives Splitting nach Heading-Level empfohlen.",
            too_long_count
        )
    } else if section_count < 3 {
        "Zu wenig Gliederung für effektives Chunking. \
         Zusätzliche Zwischenüberschriften würden die Extrahierbarkeit verbessern."
            .to_string()
    } else {
        format!(
            "Gemischte Inhaltsstruktur: {} Abschnitte optimal, {} zu kurz, {} zu lang. \
             Mehr Zwischenüberschriften verbessern die Lesbarkeit für KI-Systeme.",
            optimal_count, too_short_count, too_long_count
        )
    };

    ChunkAnalysis {
        dimension: build_dimension("Technische KI-Lesbarkeit", &signals),
        sections,
        recommendation,
    }
}

fn build_sections(input: &ChunkInput) -> Vec<ContentSection> {
    let mut sections = Vec::new();

    if input.headings.is_empty() {
        // No headings: the entire content is one chunk
        sections.push(ContentSection {
            heading: "Gesamter Inhalt".into(),
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
                heading: "Einleitung".into(),
                level: 0,
                word_count: intro_words,
                quality: classify_chunk_size(intro_words),
            });
        }

        // Each heading starts a section
        for h in &input.headings {
            sections.push(ContentSection {
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
        assert_eq!(result.dimension.name, "Technische KI-Lesbarkeit");
        assert!(!result.sections.is_empty());
    }

    #[test]
    fn minimal_input_produces_single_section_low_score() {
        let result = analyze_chunks(&minimal_input());
        // No headings → one section covering all 0 words
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].heading, "Gesamter Inhalt");
        assert_eq!(result.sections[0].quality, ChunkQuality::TooShort);
        assert!(result.dimension.score <= 30);
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
            .find(|s| s.name == "Hierarchische Gliederung")
            .expect("signal must exist");
        // Only H2, no H3 → flat
        assert!(!hier_signal.present);
    }
}
