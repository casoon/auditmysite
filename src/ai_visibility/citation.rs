//! Citation Likelihood Checker
//!
//! Heuristic scoring of how likely a page's content is to be cited or quoted
//! by LLMs. Evaluates authority signals, statement clarity, snippet quality,
//! and source attribution.

use serde::{Deserialize, Serialize};

use super::{
    build_dimension, AiSignal, AiSignalKind, AiSignalValues, DimensionKind, DimensionScore,
};

/// Citation likelihood analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationAnalysis {
    /// Dimension score (0–100) with individual signals
    pub dimension: DimensionScore,
}

/// Input data for citation analysis
pub(crate) struct CitationInput {
    pub has_https: bool,
    pub has_author_schema: bool,
    pub has_org_schema: bool,
    pub has_article_schema: bool,
    pub has_canonical: bool,
    pub has_og_meta: bool,
    pub word_count: u32,
    pub heading_count: usize,
    pub security_score: Option<u32>,
    pub a11y_score: f32,
    pub has_faq_schema: bool,
    pub has_lists: bool,
    pub short_paragraph_ratio: f32,
    pub has_date_published: bool,
    pub has_breadcrumb: bool,
}

pub(crate) fn analyze_citation(input: &CitationInput) -> CitationAnalysis {
    let mut signals = Vec::new();

    // 1. Authority: HTTPS
    signals.push(AiSignal::new(
        AiSignalKind::Encryption,
        input.has_https,
        0.08,
        AiSignalValues::default(),
    ));

    // 2. Author / Organization identity
    let has_identity = input.has_author_schema || input.has_org_schema;
    signals.push(AiSignal::new(
        AiSignalKind::PublisherIdentity,
        has_identity,
        0.15,
        AiSignalValues {
            has_author: Some(input.has_author_schema),
            has_org: Some(input.has_org_schema),
            ..Default::default()
        },
    ));

    // 3. Article structured data
    signals.push(AiSignal::new(
        AiSignalKind::ArticleStructure,
        input.has_article_schema,
        0.10,
        AiSignalValues::default(),
    ));

    // 4. Publication date
    signals.push(AiSignal::new(
        AiSignalKind::PublicationDate,
        input.has_date_published,
        0.08,
        AiSignalValues::default(),
    ));

    // 5. Canonical URL
    signals.push(AiSignal::new(
        AiSignalKind::CanonicalUrl,
        input.has_canonical,
        0.07,
        AiSignalValues::default(),
    ));

    // 6. Snippet quality — short paragraphs and lists = quotable chunks
    let good_snippet_structure = input.short_paragraph_ratio >= 0.4 || input.has_lists;
    signals.push(AiSignal::new(
        AiSignalKind::SnippetQuality,
        good_snippet_structure,
        0.15,
        AiSignalValues {
            short_paragraph_ratio: Some(input.short_paragraph_ratio),
            has_lists: Some(input.has_lists),
            ..Default::default()
        },
    ));

    // 7. FAQ patterns — directly quotable Q&A
    signals.push(AiSignal::new(
        AiSignalKind::QuestionAnswerPattern,
        input.has_faq_schema,
        0.10,
        AiSignalValues::default(),
    ));

    // 8. Content substance
    let substantial = input.word_count >= 500 && input.heading_count >= 3;
    signals.push(AiSignal::new(
        AiSignalKind::ContentDepth,
        substantial,
        0.10,
        AiSignalValues {
            word_count: Some(input.word_count),
            section_count: Some(input.heading_count as u32),
            ..Default::default()
        },
    ));

    // 9. Social / sharing metadata
    signals.push(AiSignal::new(
        AiSignalKind::SharingMetadata,
        input.has_og_meta,
        0.07,
        AiSignalValues::default(),
    ));

    // 10. Breadcrumb — provides topical context
    signals.push(AiSignal::new(
        AiSignalKind::ThematicContext,
        input.has_breadcrumb,
        0.05,
        AiSignalValues::default(),
    ));

    // 11. Technical trust
    let sec_good = input.security_score.is_none_or(|s| s >= 70);
    let a11y_good = input.a11y_score >= 80.0;
    let tech_trust = sec_good && a11y_good;
    signals.push(AiSignal::new(
        AiSignalKind::TechnicalTrust,
        tech_trust,
        0.05,
        AiSignalValues {
            security_score: input.security_score,
            a11y_score: Some(input.a11y_score),
            ..Default::default()
        },
    ));

    CitationAnalysis {
        dimension: build_dimension(DimensionKind::Citability, &signals),
    }
}

// ─── Signal detail text (single source of truth) ─────────────────────────────

pub(crate) fn detail_encryption(present: bool, en: bool) -> String {
    if present {
        if en {
            "HTTPS — trust signal for citation-worthiness".into()
        } else {
            "HTTPS — Vertrauenssignal für Zitierwürdigkeit".into()
        }
    } else if en {
        "No HTTPS — reduces trust in the source".into()
    } else {
        "Kein HTTPS — mindert Vertrauen in die Quelle".into()
    }
}

pub(crate) fn detail_publisher_identity(v: &AiSignalValues, en: bool) -> String {
    let has_author = v.has_author.unwrap_or(false);
    let has_org = v.has_org.unwrap_or(false);
    if has_author && has_org {
        if en {
            "Author + organization identified via Schema.org — strong authority signal".into()
        } else {
            "Autor + Organisation per Schema.org identifiziert — starkes Autoritätssignal".into()
        }
    } else if has_author {
        if en {
            "Author identified via Schema.org".into()
        } else {
            "Autor per Schema.org identifiziert".into()
        }
    } else if has_org {
        if en {
            "Organization identified via Schema.org".into()
        } else {
            "Organisation per Schema.org identifiziert".into()
        }
    } else if en {
        "No publisher markup — authority not machine-verifiable".into()
    } else {
        "Kein Herausgeber-Markup — Autorität nicht maschinell prüfbar".into()
    }
}

pub(crate) fn detail_article_structure(present: bool, en: bool) -> String {
    if present {
        if en {
            "Article/BlogPosting schema — marked as citable content".into()
        } else {
            "Article/BlogPosting-Schema — als zitierfähiger Inhalt markiert".into()
        }
    } else if en {
        "No article schema — content type not machine-readable".into()
    } else {
        "Kein Artikel-Schema — Content-Typ nicht maschinenlesbar".into()
    }
}

pub(crate) fn detail_publication_date(present: bool, en: bool) -> String {
    if present {
        if en {
            "Publication date present — recency verifiable".into()
        } else {
            "Veröffentlichungsdatum vorhanden — Aktualität prüfbar".into()
        }
    } else if en {
        "No publication date — recency not assessable".into()
    } else {
        "Kein Publikationsdatum — Aktualität nicht einschätzbar".into()
    }
}

pub(crate) fn detail_canonical_url(present: bool, en: bool) -> String {
    if present {
        if en {
            "Canonical URL set — unambiguous source reference".into()
        } else {
            "Canonical URL gesetzt — eindeutige Quellenreferenz".into()
        }
    } else if en {
        "No canonical URL — duplicates possible".into()
    } else {
        "Keine Canonical-URL — Duplikate möglich".into()
    }
}

pub(crate) fn detail_snippet_quality(v: &AiSignalValues, en: bool) -> String {
    let ratio = v.short_paragraph_ratio.unwrap_or(0.0);
    let has_lists = v.has_lists.unwrap_or(false);
    if ratio >= 0.4 && has_lists {
        if en {
            format!(
                "{:.0}% short paragraphs + lists — many citable text blocks",
                ratio * 100.0
            )
        } else {
            format!(
                "{:.0}% kurze Absätze + Listen — viele zitierfähige Textblöcke",
                ratio * 100.0
            )
        }
    } else if ratio >= 0.4 {
        if en {
            format!(
                "{:.0}% short, concise paragraphs — good snippet suitability",
                ratio * 100.0
            )
        } else {
            format!(
                "{:.0}% kurze, prägnante Absätze — gute Snippet-Eignung",
                ratio * 100.0
            )
        }
    } else if has_lists {
        if en {
            "Lists present — citable bullet points".into()
        } else {
            "Listen vorhanden — zitierfähige Aufzählungen".into()
        }
    } else if en {
        "Few short paragraphs, no lists — low snippet suitability".into()
    } else {
        "Wenig kurze Absätze, keine Listen — geringe Snippet-Eignung".into()
    }
}

pub(crate) fn detail_question_answer(present: bool, en: bool) -> String {
    if present {
        if en {
            "FAQ schema — answers directly usable as quotes".into()
        } else {
            "FAQ-Schema — Antworten direkt als Zitat nutzbar".into()
        }
    } else if en {
        "No FAQ structure — no direct quote potential".into()
    } else {
        "Keine FAQ-Struktur — kein direktes Zitat-Potenzial".into()
    }
}

pub(crate) fn detail_content_depth(present: bool, v: &AiSignalValues, en: bool) -> String {
    let word_count = v.word_count.unwrap_or(0);
    let section_count = v.section_count.unwrap_or(0);
    if present {
        if en {
            format!(
                "{} words, {} sections — sufficient substance for quotes",
                word_count, section_count
            )
        } else {
            format!(
                "{} Wörter, {} Abschnitte — ausreichend Substanz für Zitate",
                word_count, section_count
            )
        }
    } else if en {
        format!(
            "{} words, {} headings — little substance",
            word_count, section_count
        )
    } else {
        format!(
            "{} Wörter, {} Überschriften — wenig Substanz",
            word_count, section_count
        )
    }
}

pub(crate) fn detail_sharing_metadata(present: bool, en: bool) -> String {
    if present {
        if en {
            "Open Graph present — preview and referencing possible".into()
        } else {
            "Open Graph vorhanden — Vorschau und Referenzierung möglich".into()
        }
    } else if en {
        "No Open Graph data — limited preview".into()
    } else {
        "Keine Open-Graph-Daten — eingeschränkte Vorschau".into()
    }
}

pub(crate) fn detail_thematic_context(present: bool, en: bool) -> String {
    if present {
        if en {
            "Breadcrumb schema — thematic context machine-available".into()
        } else {
            "Breadcrumb-Schema — thematischer Kontext maschinell verfügbar".into()
        }
    } else if en {
        "No breadcrumb — thematic context missing".into()
    } else {
        "Kein Breadcrumb — thematische Einordnung fehlt".into()
    }
}

pub(crate) fn detail_technical_trust(present: bool, v: &AiSignalValues, en: bool) -> String {
    let security_score = v.security_score;
    let a11y_score = v.a11y_score.unwrap_or(0.0);
    format!(
        "Security: {}, Accessibility: {:.0} — {}",
        security_score.map_or("n/a".to_string(), |s| format!("{}", s)),
        a11y_score,
        if present {
            if en {
                "stable technical foundation"
            } else {
                "stabile technische Basis"
            }
        } else if en {
            "technical weaknesses reduce trust"
        } else {
            "technische Schwächen mindern Vertrauen"
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rich_input() -> CitationInput {
        CitationInput {
            has_https: true,
            has_author_schema: true,
            has_org_schema: true,
            has_article_schema: true,
            has_canonical: true,
            has_og_meta: true,
            word_count: 800,
            heading_count: 5,
            security_score: Some(90),
            a11y_score: 95.0,
            has_faq_schema: true,
            has_lists: true,
            short_paragraph_ratio: 0.6,
            has_date_published: true,
            has_breadcrumb: true,
        }
    }

    fn minimal_input() -> CitationInput {
        CitationInput {
            has_https: false,
            has_author_schema: false,
            has_org_schema: false,
            has_article_schema: false,
            has_canonical: false,
            has_og_meta: false,
            word_count: 0,
            heading_count: 0,
            security_score: None,
            a11y_score: 0.0,
            has_faq_schema: false,
            has_lists: false,
            short_paragraph_ratio: 0.0,
            has_date_published: false,
            has_breadcrumb: false,
        }
    }

    #[test]
    fn rich_input_produces_high_score() {
        let result = analyze_citation(&rich_input());
        assert!(result.dimension.score >= 80);
        // Struct carries canonical English.
        assert_eq!(result.dimension.name, "Citability");
        assert_eq!(result.dimension.kind, DimensionKind::Citability);
    }

    #[test]
    fn minimal_input_produces_low_score() {
        let result = analyze_citation(&minimal_input());
        // Only signal that might be present is technical trust (no security score = None → sec_good=true, a11y=0 → bad)
        assert!(result.dimension.score <= 10);
    }

    #[test]
    fn author_and_org_both_present_detected() {
        let result = analyze_citation(&rich_input());
        let identity_signal = result
            .dimension
            .signals
            .iter()
            .find(|s| s.kind == AiSignalKind::PublisherIdentity)
            .expect("signal must exist");
        assert!(identity_signal.present);
        // Canonical English detail.
        assert!(
            identity_signal.detail.contains("Author")
                && identity_signal.detail.contains("organization")
        );
    }

    #[test]
    fn snippet_quality_with_lists_only() {
        let input = CitationInput {
            has_lists: true,
            short_paragraph_ratio: 0.1, // below 0.4 threshold
            ..minimal_input()
        };
        let result = analyze_citation(&input);
        let snippet_signal = result
            .dimension
            .signals
            .iter()
            .find(|s| s.kind == AiSignalKind::SnippetQuality)
            .expect("signal must exist");
        assert!(snippet_signal.present);
    }

    #[test]
    fn low_a11y_score_triggers_tech_trust_failure() {
        let input = CitationInput {
            a11y_score: 50.0, // below 80.0 threshold
            security_score: Some(90),
            ..minimal_input()
        };
        let result = analyze_citation(&input);
        let trust_signal = result
            .dimension
            .signals
            .iter()
            .find(|s| s.kind == AiSignalKind::TechnicalTrust)
            .expect("signal must exist");
        assert!(!trust_signal.present);
    }
}
