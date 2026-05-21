//! LLM-Readability Analyzer
//!
//! Heuristic scoring of how well page content can be extracted and understood
//! by large language models. Evaluates structure, clarity, entity density,
//! redundancy, and the presence of extractable answers.

use serde::{Deserialize, Serialize};

use super::{build_dimension, AiSignal, DimensionScore};

/// LLM readability analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityAnalysis {
    /// Dimension score (0–100) with individual signals
    pub dimension: DimensionScore,
}

/// Input data extracted from the audit report for readability analysis
pub(crate) struct ReadabilityInput {
    pub word_count: u32,
    pub heading_count: usize,
    pub max_heading_depth: u32,
    pub has_lists: bool,
    pub list_count: u32,
    pub has_tables: bool,
    pub has_schema: bool,
    pub schema_type_count: usize,
    pub has_faq_schema: bool,
    pub has_howto_schema: bool,
    pub has_meta_description: bool,
    pub meta_desc_len: usize,
    pub has_lang: bool,
    pub paragraph_count: u32,
    pub avg_paragraph_len: u32,
    pub has_definition_patterns: bool,
}

pub(crate) fn analyze_readability(input: &ReadabilityInput) -> ReadabilityAnalysis {
    let mut signals = Vec::new();

    // 1. Heading structure depth — LLMs rely on headings for section extraction
    let good_depth = input.max_heading_depth >= 3 && input.heading_count >= 3;
    signals.push(AiSignal {
        name: "Überschriftenstruktur".into(),
        present: good_depth,
        weight: 0.15,
        detail: if good_depth {
            format!(
                "{} Überschriften bis H{} — gute Gliederung für Abschnittsextraktion",
                input.heading_count, input.max_heading_depth
            )
        } else if input.heading_count == 0 {
            "Keine Überschriften — LLMs können keine Abschnitte erkennen".into()
        } else {
            format!(
                "Flache Gliederung ({} Überschriften, max H{}) — erschwert Chunk-Bildung",
                input.heading_count, input.max_heading_depth
            )
        },
    });

    // 2. Content density — enough substance for extraction
    let substantial = input.word_count >= 300;
    let very_long = input.word_count > 5000;
    signals.push(AiSignal {
        name: "Inhaltsumfang".into(),
        present: substantial && !very_long,
        weight: 0.10,
        detail: if !substantial {
            format!(
                "{} Wörter — zu wenig für gehaltvolle LLM-Extraktion (empfohlen ≥ 300)",
                input.word_count
            )
        } else if very_long {
            format!(
                "{} Wörter — sehr lang, kann Kontext-Fenster sprengen ohne gute Chunk-Struktur",
                input.word_count
            )
        } else {
            format!(
                "{} Wörter — guter Umfang für LLM-Verarbeitung",
                input.word_count
            )
        },
    });

    // 3. Paragraph structure — short, focused paragraphs aid extraction
    let good_paragraphs =
        input.paragraph_count >= 3 && input.avg_paragraph_len > 0 && input.avg_paragraph_len <= 150;
    signals.push(AiSignal {
        name: "Absatzstruktur".into(),
        present: good_paragraphs,
        weight: 0.12,
        detail: if input.paragraph_count < 3 {
            "Weniger als 3 Absätze — kaum Gliederung für Chunk-Extraktion".into()
        } else if input.avg_paragraph_len > 150 {
            format!(
                "Ø {:.0} Wörter/Absatz — zu lang, erschwert gezielte Extraktion",
                input.avg_paragraph_len
            )
        } else {
            format!(
                "{} Absätze mit Ø {:.0} Wörtern — gut strukturiert",
                input.paragraph_count, input.avg_paragraph_len
            )
        },
    });

    // 4. List usage — lists are highly extractable for LLMs
    signals.push(AiSignal {
        name: "Listen / Aufzählungen".into(),
        present: input.has_lists,
        weight: 0.10,
        detail: if input.has_lists {
            format!(
                "{} Listen gefunden — gut für Fakten-Extraktion",
                input.list_count
            )
        } else {
            "Keine Listen — Aufzählungen verbessern LLM-Extrahierbarkeit".into()
        },
    });

    // 5. Tables — structured data LLMs can parse
    signals.push(AiSignal {
        name: "Tabellen".into(),
        present: input.has_tables,
        weight: 0.08,
        detail: if input.has_tables {
            "Tabellarische Daten vorhanden — gut für strukturierte Extraktion".into()
        } else {
            "Keine Tabellen (nicht immer nötig)".into()
        },
    });

    // 6. Schema.org entities — LLMs leverage structured data for understanding
    let rich_schema = input.schema_type_count >= 2;
    signals.push(AiSignal {
        name: "Schema-Abdeckung".into(),
        present: input.has_schema,
        weight: 0.12,
        detail: if rich_schema {
            format!(
                "{} Schema-Typen — reichhaltige Entitätsinformationen",
                input.schema_type_count
            )
        } else if input.has_schema {
            "Grundlegendes Schema.org vorhanden".into()
        } else {
            "Keine Schema.org-Daten — Entitäten sind für LLMs nicht maschinenlesbar".into()
        },
    });

    // 7. FAQ / HowTo patterns — directly extractable Q&A
    let has_qa_pattern = input.has_faq_schema || input.has_howto_schema;
    signals.push(AiSignal {
        name: "Extrahierbare Antworten".into(),
        present: has_qa_pattern,
        weight: 0.13,
        detail: if input.has_faq_schema && input.has_howto_schema {
            "FAQ + HowTo-Schema — optimale Frage-Antwort-Extraktion".into()
        } else if input.has_faq_schema {
            "FAQ-Schema vorhanden — Fragen direkt extrahierbar".into()
        } else if input.has_howto_schema {
            "HowTo-Schema vorhanden — Anleitungen direkt extrahierbar".into()
        } else {
            "Keine FAQ/HowTo-Struktur — Antworten nicht direkt extrahierbar".into()
        },
    });

    // 8. Meta description quality — used as summary by LLMs
    let good_meta = input.has_meta_description && input.meta_desc_len >= 80;
    signals.push(AiSignal {
        name: "Zusammenfassung (Meta)".into(),
        present: good_meta,
        weight: 0.10,
        detail: if good_meta {
            format!(
                "Meta-Beschreibung ({} Zeichen) — gute Kurzfassung für LLM-Kontext",
                input.meta_desc_len
            )
        } else if input.has_meta_description {
            format!(
                "Meta-Beschreibung zu kurz ({} Zeichen) — wenig Kontext",
                input.meta_desc_len
            )
        } else {
            "Keine Meta-Beschreibung — LLMs fehlt ein Seiten-Summary".into()
        },
    });

    // 9. Language declaration — helps LLMs with language-specific processing
    signals.push(AiSignal {
        name: "Sprachdeklaration".into(),
        present: input.has_lang,
        weight: 0.05,
        detail: if input.has_lang {
            "Sprache deklariert — korrekte Tokenisierung möglich".into()
        } else {
            "Keine Sprachdeklaration — erschwert sprachspezifische Verarbeitung".into()
        },
    });

    // 10. Definition patterns — explicit definitions aid understanding
    signals.push(AiSignal {
        name: "Definitionsmuster".into(),
        present: input.has_definition_patterns,
        weight: 0.05,
        detail: if input.has_definition_patterns {
            "Klare Definitions-/Erklärungsmuster erkannt".into()
        } else {
            "Keine expliziten Definitionsmuster erkannt".into()
        },
    });

    ReadabilityAnalysis {
        dimension: build_dimension("KI-Lesbarkeit", &signals),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rich_input() -> ReadabilityInput {
        ReadabilityInput {
            word_count: 800,
            heading_count: 6,
            max_heading_depth: 4,
            has_lists: true,
            list_count: 3,
            has_tables: true,
            has_schema: true,
            schema_type_count: 3,
            has_faq_schema: true,
            has_howto_schema: false,
            has_meta_description: true,
            meta_desc_len: 150,
            has_lang: true,
            paragraph_count: 10,
            avg_paragraph_len: 80,
            has_definition_patterns: true,
        }
    }

    fn minimal_input() -> ReadabilityInput {
        ReadabilityInput {
            word_count: 0,
            heading_count: 0,
            max_heading_depth: 0,
            has_lists: false,
            list_count: 0,
            has_tables: false,
            has_schema: false,
            schema_type_count: 0,
            has_faq_schema: false,
            has_howto_schema: false,
            has_meta_description: false,
            meta_desc_len: 0,
            has_lang: false,
            paragraph_count: 0,
            avg_paragraph_len: 0,
            has_definition_patterns: false,
        }
    }

    #[test]
    fn rich_input_produces_high_score() {
        let result = analyze_readability(&rich_input());
        assert!(result.dimension.score >= 70);
        assert_eq!(result.dimension.name, "KI-Lesbarkeit");
    }

    #[test]
    fn minimal_input_produces_low_score() {
        let result = analyze_readability(&minimal_input());
        assert!(result.dimension.score <= 20);
    }

    #[test]
    fn very_long_content_penalized() {
        let mut input = rich_input();
        input.word_count = 6000; // above the 5000-word penalty threshold
        let rich_score = analyze_readability(&rich_input()).dimension.score;
        let long_score = analyze_readability(&input).dimension.score;
        assert!(long_score < rich_score);
    }

    #[test]
    fn faq_and_howto_both_present_detected() {
        let mut input = rich_input();
        input.has_howto_schema = true;
        let result = analyze_readability(&input);
        let qa_signal = result
            .dimension
            .signals
            .iter()
            .find(|s| s.name == "Extrahierbare Antworten")
            .expect("signal must exist");
        assert!(qa_signal.present);
        assert!(qa_signal.detail.contains("FAQ") && qa_signal.detail.contains("HowTo"));
    }

    #[test]
    fn short_meta_description_not_counted_as_good() {
        let input = ReadabilityInput {
            has_meta_description: true,
            meta_desc_len: 30, // below the 80-char threshold
            ..minimal_input()
        };
        let result = analyze_readability(&input);
        let meta_signal = result
            .dimension
            .signals
            .iter()
            .find(|s| s.name == "Zusammenfassung (Meta)")
            .expect("signal must exist");
        assert!(!meta_signal.present);
    }
}
