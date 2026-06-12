//! LLM-Readability Analyzer
//!
//! Heuristic scoring of how well page content can be extracted and understood
//! by large language models. Evaluates structure, clarity, entity density,
//! redundancy, and the presence of extractable answers.

use serde::{Deserialize, Serialize};

use super::{
    build_dimension, AiSignal, AiSignalKind, AiSignalValues, DimensionKind, DimensionScore,
};

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
    signals.push(AiSignal::new(
        AiSignalKind::HeadingStructure,
        good_depth,
        0.15,
        AiSignalValues {
            heading_count: Some(input.heading_count as u32),
            max_heading_depth: Some(input.max_heading_depth),
            ..Default::default()
        },
    ));

    // 2. Content density — enough substance for extraction
    let substantial = input.word_count >= 300;
    let very_long = input.word_count > 5000;
    signals.push(AiSignal::new(
        AiSignalKind::ContentVolume,
        substantial && !very_long,
        0.10,
        AiSignalValues {
            word_count: Some(input.word_count),
            ..Default::default()
        },
    ));

    // 3. Paragraph structure — short, focused paragraphs aid extraction
    let good_paragraphs =
        input.paragraph_count >= 3 && input.avg_paragraph_len > 0 && input.avg_paragraph_len <= 150;
    signals.push(AiSignal::new(
        AiSignalKind::ParagraphStructure,
        good_paragraphs,
        0.12,
        AiSignalValues {
            paragraph_count: Some(input.paragraph_count),
            avg_paragraph_len: Some(input.avg_paragraph_len),
            ..Default::default()
        },
    ));

    // 4. List usage — lists are highly extractable for LLMs
    signals.push(AiSignal::new(
        AiSignalKind::Lists,
        input.has_lists,
        0.10,
        AiSignalValues {
            list_count: Some(input.list_count),
            ..Default::default()
        },
    ));

    // 5. Tables — structured data LLMs can parse
    signals.push(AiSignal::new(
        AiSignalKind::Tables,
        input.has_tables,
        0.08,
        AiSignalValues::default(),
    ));

    // 6. Schema.org entities — LLMs leverage structured data for understanding
    signals.push(AiSignal::new(
        AiSignalKind::SchemaCoverage,
        input.has_schema,
        0.12,
        AiSignalValues {
            has_schema: Some(input.has_schema),
            schema_type_count: Some(input.schema_type_count as u32),
            ..Default::default()
        },
    ));

    // 7. FAQ / HowTo patterns — directly extractable Q&A
    let has_qa_pattern = input.has_faq_schema || input.has_howto_schema;
    signals.push(AiSignal::new(
        AiSignalKind::ExtractableAnswers,
        has_qa_pattern,
        0.13,
        AiSignalValues {
            has_faq: Some(input.has_faq_schema),
            has_howto: Some(input.has_howto_schema),
            ..Default::default()
        },
    ));

    // 8. Meta description quality — used as summary by LLMs
    let good_meta = input.has_meta_description && input.meta_desc_len >= 80;
    signals.push(AiSignal::new(
        AiSignalKind::SummaryMeta,
        good_meta,
        0.10,
        AiSignalValues {
            has_meta_description: Some(input.has_meta_description),
            meta_desc_len: Some(input.meta_desc_len as u32),
            ..Default::default()
        },
    ));

    // 9. Language declaration — helps LLMs with language-specific processing
    signals.push(AiSignal::new(
        AiSignalKind::LanguageDeclaration,
        input.has_lang,
        0.05,
        AiSignalValues::default(),
    ));

    // 10. Definition patterns — explicit definitions aid understanding
    signals.push(AiSignal::new(
        AiSignalKind::DefinitionPatterns,
        input.has_definition_patterns,
        0.05,
        AiSignalValues::default(),
    ));

    ReadabilityAnalysis {
        dimension: build_dimension(DimensionKind::Readability, &signals),
    }
}

// ─── Signal detail text (single source of truth) ─────────────────────────────

pub(crate) fn detail_heading_structure(present: bool, v: &AiSignalValues, en: bool) -> String {
    let heading_count = v.heading_count.unwrap_or(0);
    let max_depth = v.max_heading_depth.unwrap_or(0);
    if present {
        if en {
            format!(
                "{} headings down to H{} — good structure for section extraction",
                heading_count, max_depth
            )
        } else {
            format!(
                "{} Überschriften bis H{} — gute Gliederung für Abschnittsextraktion",
                heading_count, max_depth
            )
        }
    } else if heading_count == 0 {
        if en {
            "No headings — LLMs cannot recognize sections".into()
        } else {
            "Keine Überschriften — LLMs können keine Abschnitte erkennen".into()
        }
    } else if en {
        format!(
            "Flat outline ({} headings, max H{}) — hinders chunk formation",
            heading_count, max_depth
        )
    } else {
        format!(
            "Flache Gliederung ({} Überschriften, max H{}) — erschwert Chunk-Bildung",
            heading_count, max_depth
        )
    }
}

pub(crate) fn detail_content_volume(present: bool, v: &AiSignalValues, en: bool) -> String {
    let word_count = v.word_count.unwrap_or(0);
    let substantial = word_count >= 300;
    let very_long = word_count > 5000;
    let _ = present;
    if !substantial {
        if en {
            format!(
                "{} words — too little for substantial LLM extraction (recommended ≥ 300)",
                word_count
            )
        } else {
            format!(
                "{} Wörter — zu wenig für gehaltvolle LLM-Extraktion (empfohlen ≥ 300)",
                word_count
            )
        }
    } else if very_long {
        if en {
            format!(
                "{} words — very long, may exceed the context window without good chunk structure",
                word_count
            )
        } else {
            format!(
                "{} Wörter — sehr lang, kann Kontext-Fenster sprengen ohne gute Chunk-Struktur",
                word_count
            )
        }
    } else if en {
        format!("{} words — good volume for LLM processing", word_count)
    } else {
        format!("{} Wörter — guter Umfang für LLM-Verarbeitung", word_count)
    }
}

pub(crate) fn detail_paragraph_structure(v: &AiSignalValues, en: bool) -> String {
    let paragraph_count = v.paragraph_count.unwrap_or(0);
    let avg_len = v.avg_paragraph_len.unwrap_or(0);
    if paragraph_count < 3 {
        if en {
            "Fewer than 3 paragraphs — hardly any structure for chunk extraction".into()
        } else {
            "Weniger als 3 Absätze — kaum Gliederung für Chunk-Extraktion".into()
        }
    } else if avg_len > 150 {
        if en {
            format!(
                "Avg {} words/paragraph — too long, hinders targeted extraction",
                avg_len
            )
        } else {
            format!(
                "Ø {} Wörter/Absatz — zu lang, erschwert gezielte Extraktion",
                avg_len
            )
        }
    } else if en {
        format!(
            "{} paragraphs with avg {} words — well structured",
            paragraph_count, avg_len
        )
    } else {
        format!(
            "{} Absätze mit Ø {} Wörtern — gut strukturiert",
            paragraph_count, avg_len
        )
    }
}

pub(crate) fn detail_lists(present: bool, v: &AiSignalValues, en: bool) -> String {
    let list_count = v.list_count.unwrap_or(0);
    if present {
        if en {
            format!("{} lists found — good for fact extraction", list_count)
        } else {
            format!("{} Listen gefunden — gut für Fakten-Extraktion", list_count)
        }
    } else if en {
        "No lists — bullet points improve LLM extractability".into()
    } else {
        "Keine Listen — Aufzählungen verbessern LLM-Extrahierbarkeit".into()
    }
}

pub(crate) fn detail_tables(present: bool, en: bool) -> String {
    if present {
        if en {
            "Tabular data present — good for structured extraction".into()
        } else {
            "Tabellarische Daten vorhanden — gut für strukturierte Extraktion".into()
        }
    } else if en {
        "No tables (not always needed)".into()
    } else {
        "Keine Tabellen (nicht immer nötig)".into()
    }
}

pub(crate) fn detail_schema_coverage(v: &AiSignalValues, en: bool) -> String {
    let has_schema = v.has_schema.unwrap_or(false);
    let count = v.schema_type_count.unwrap_or(0);
    let rich_schema = count >= 2;
    if rich_schema {
        if en {
            format!("{} schema types — rich entity information", count)
        } else {
            format!(
                "{} Schema-Typen — reichhaltige Entitätsinformationen",
                count
            )
        }
    } else if has_schema {
        if en {
            "Basic Schema.org present".into()
        } else {
            "Grundlegendes Schema.org vorhanden".into()
        }
    } else if en {
        "No Schema.org data — entities are not machine-readable for LLMs".into()
    } else {
        "Keine Schema.org-Daten — Entitäten sind für LLMs nicht maschinenlesbar".into()
    }
}

pub(crate) fn detail_extractable_answers(v: &AiSignalValues, en: bool) -> String {
    let has_faq = v.has_faq.unwrap_or(false);
    let has_howto = v.has_howto.unwrap_or(false);
    if has_faq && has_howto {
        if en {
            "FAQ + HowTo schema — optimal question-answer extraction".into()
        } else {
            "FAQ + HowTo-Schema — optimale Frage-Antwort-Extraktion".into()
        }
    } else if has_faq {
        if en {
            "FAQ schema present — questions directly extractable".into()
        } else {
            "FAQ-Schema vorhanden — Fragen direkt extrahierbar".into()
        }
    } else if has_howto {
        if en {
            "HowTo schema present — instructions directly extractable".into()
        } else {
            "HowTo-Schema vorhanden — Anleitungen direkt extrahierbar".into()
        }
    } else if en {
        "No FAQ/HowTo structure — answers not directly extractable".into()
    } else {
        "Keine FAQ/HowTo-Struktur — Antworten nicht direkt extrahierbar".into()
    }
}

pub(crate) fn detail_summary_meta(present: bool, v: &AiSignalValues, en: bool) -> String {
    let has_meta = v.has_meta_description.unwrap_or(false);
    let len = v.meta_desc_len.unwrap_or(0);
    if present {
        if en {
            format!(
                "Meta description ({} characters) — good summary for LLM context",
                len
            )
        } else {
            format!(
                "Meta-Beschreibung ({} Zeichen) — gute Kurzfassung für LLM-Kontext",
                len
            )
        }
    } else if has_meta {
        if en {
            format!(
                "Meta description too short ({} characters) — little context",
                len
            )
        } else {
            format!(
                "Meta-Beschreibung zu kurz ({} Zeichen) — wenig Kontext",
                len
            )
        }
    } else if en {
        "No meta description — LLMs lack a page summary".into()
    } else {
        "Keine Meta-Beschreibung — LLMs fehlt ein Seiten-Summary".into()
    }
}

pub(crate) fn detail_language_declaration(present: bool, en: bool) -> String {
    if present {
        if en {
            "Language declared — correct tokenization possible".into()
        } else {
            "Sprache deklariert — korrekte Tokenisierung möglich".into()
        }
    } else if en {
        "No language declaration — hinders language-specific processing".into()
    } else {
        "Keine Sprachdeklaration — erschwert sprachspezifische Verarbeitung".into()
    }
}

pub(crate) fn detail_definition_patterns(present: bool, en: bool) -> String {
    if present {
        if en {
            "Clear definition/explanation patterns detected".into()
        } else {
            "Klare Definitions-/Erklärungsmuster erkannt".into()
        }
    } else if en {
        "No explicit definition patterns detected".into()
    } else {
        "Keine expliziten Definitionsmuster erkannt".into()
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
        // Struct carries canonical English.
        assert_eq!(result.dimension.name, "AI readability");
        assert_eq!(result.dimension.kind, DimensionKind::Readability);
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
            .find(|s| s.kind == AiSignalKind::ExtractableAnswers)
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
            .find(|s| s.kind == AiSignalKind::SummaryMeta)
            .expect("signal must exist");
        assert!(!meta_signal.present);
    }

    #[test]
    fn detail_re_derives_german() {
        let (name, detail) = super::super::ai_signal_text(
            AiSignalKind::ContentVolume,
            false,
            &AiSignalValues {
                word_count: Some(120),
                ..Default::default()
            },
            false,
        );
        assert_eq!(name, "Inhaltsumfang");
        assert!(detail.contains("Wörter"));
    }
}
