//! Citation Likelihood Checker
//!
//! Heuristic scoring of how likely a page's content is to be cited or quoted
//! by LLMs. Evaluates authority signals, statement clarity, snippet quality,
//! and source attribution.

use serde::{Deserialize, Serialize};

use super::{build_dimension, AiSignal, DimensionScore};

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
    signals.push(AiSignal {
        name: "Verschlüsselung".into(),
        present: input.has_https,
        weight: 0.08,
        detail: if input.has_https {
            "HTTPS — Vertrauenssignal für Zitierwürdigkeit".into()
        } else {
            "Kein HTTPS — mindert Vertrauen in die Quelle".into()
        },
    });

    // 2. Author / Organization identity
    let has_identity = input.has_author_schema || input.has_org_schema;
    signals.push(AiSignal {
        name: "Herausgeber-Identität".into(),
        present: has_identity,
        weight: 0.15,
        detail: if input.has_author_schema && input.has_org_schema {
            "Autor + Organisation per Schema.org identifiziert — starkes Autoritätssignal".into()
        } else if input.has_author_schema {
            "Autor per Schema.org identifiziert".into()
        } else if input.has_org_schema {
            "Organisation per Schema.org identifiziert".into()
        } else {
            "Kein Herausgeber-Markup — Autorität nicht maschinell prüfbar".into()
        },
    });

    // 3. Article structured data
    signals.push(AiSignal {
        name: "Artikelstruktur".into(),
        present: input.has_article_schema,
        weight: 0.10,
        detail: if input.has_article_schema {
            "Article/BlogPosting-Schema — als zitierfähiger Inhalt markiert".into()
        } else {
            "Kein Artikel-Schema — Content-Typ nicht maschinenlesbar".into()
        },
    });

    // 4. Publication date
    signals.push(AiSignal {
        name: "Publikationsdatum".into(),
        present: input.has_date_published,
        weight: 0.08,
        detail: if input.has_date_published {
            "Veröffentlichungsdatum vorhanden — Aktualität prüfbar".into()
        } else {
            "Kein Publikationsdatum — Aktualität nicht einschätzbar".into()
        },
    });

    // 5. Canonical URL
    signals.push(AiSignal {
        name: "Kanonische URL".into(),
        present: input.has_canonical,
        weight: 0.07,
        detail: if input.has_canonical {
            "Canonical URL gesetzt — eindeutige Quellenreferenz".into()
        } else {
            "Keine Canonical-URL — Duplikate möglich".into()
        },
    });

    // 6. Snippet quality — short paragraphs and lists = quotable chunks
    let good_snippet_structure = input.short_paragraph_ratio >= 0.4 || input.has_lists;
    signals.push(AiSignal {
        name: "Snippet-Qualität".into(),
        present: good_snippet_structure,
        weight: 0.15,
        detail: if input.short_paragraph_ratio >= 0.4 && input.has_lists {
            format!(
                "{:.0}% kurze Absätze + Listen — viele zitierfähige Textblöcke",
                input.short_paragraph_ratio * 100.0
            )
        } else if input.short_paragraph_ratio >= 0.4 {
            format!(
                "{:.0}% kurze, prägnante Absätze — gute Snippet-Eignung",
                input.short_paragraph_ratio * 100.0
            )
        } else if input.has_lists {
            "Listen vorhanden — zitierfähige Aufzählungen".into()
        } else {
            "Wenig kurze Absätze, keine Listen — geringe Snippet-Eignung".into()
        },
    });

    // 7. FAQ patterns — directly quotable Q&A
    signals.push(AiSignal {
        name: "Frage-Antwort-Muster".into(),
        present: input.has_faq_schema,
        weight: 0.10,
        detail: if input.has_faq_schema {
            "FAQ-Schema — Antworten direkt als Zitat nutzbar".into()
        } else {
            "Keine FAQ-Struktur — kein direktes Zitat-Potenzial".into()
        },
    });

    // 8. Content substance
    let substantial = input.word_count >= 500 && input.heading_count >= 3;
    signals.push(AiSignal {
        name: "Inhaltliche Tiefe".into(),
        present: substantial,
        weight: 0.10,
        detail: if substantial {
            format!(
                "{} Wörter, {} Abschnitte — ausreichend Substanz für Zitate",
                input.word_count, input.heading_count
            )
        } else {
            format!(
                "{} Wörter, {} Überschriften — wenig Substanz",
                input.word_count, input.heading_count
            )
        },
    });

    // 9. Social / sharing metadata
    signals.push(AiSignal {
        name: "Teilen-Metadaten".into(),
        present: input.has_og_meta,
        weight: 0.07,
        detail: if input.has_og_meta {
            "Open Graph vorhanden — Vorschau und Referenzierung möglich".into()
        } else {
            "Keine Open-Graph-Daten — eingeschränkte Vorschau".into()
        },
    });

    // 10. Breadcrumb — provides topical context
    signals.push(AiSignal {
        name: "Thematische Einordnung".into(),
        present: input.has_breadcrumb,
        weight: 0.05,
        detail: if input.has_breadcrumb {
            "Breadcrumb-Schema — thematischer Kontext maschinell verfügbar".into()
        } else {
            "Kein Breadcrumb — thematische Einordnung fehlt".into()
        },
    });

    // 11. Technical trust
    let sec_good = input.security_score.is_none_or(|s| s >= 70);
    let a11y_good = input.a11y_score >= 80.0;
    let tech_trust = sec_good && a11y_good;
    signals.push(AiSignal {
        name: "Technisches Vertrauen".into(),
        present: tech_trust,
        weight: 0.05,
        detail: format!(
            "Security: {}, Accessibility: {:.0} — {}",
            input
                .security_score
                .map_or("n/a".to_string(), |s| format!("{}", s)),
            input.a11y_score,
            if tech_trust {
                "solide technische Basis"
            } else {
                "technische Schwächen mindern Vertrauen"
            }
        ),
    });

    CitationAnalysis {
        dimension: build_dimension("Zitierbarkeit", &signals),
    }
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
        assert_eq!(result.dimension.name, "Zitierbarkeit");
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
            .find(|s| s.name == "Herausgeber-Identität")
            .expect("signal must exist");
        assert!(identity_signal.present);
        assert!(
            identity_signal.detail.contains("Autor")
                && identity_signal.detail.contains("Organisation")
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
            .find(|s| s.name == "Snippet-Qualität")
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
            .find(|s| s.name == "Technisches Vertrauen")
            .expect("signal must exist");
        assert!(!trust_signal.present);
    }
}
