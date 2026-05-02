//! AI Visibility Analysis
//!
//! Evaluates how well a website's content is prepared for discovery,
//! extraction, and citation by AI systems (LLMs, RAG pipelines, AI search).
//!
//! Five dimensions are scored:
//! - **LLM-Lesbarkeit** — How well can LLMs extract and understand the content?
//! - **Zitatfähigkeit** — How likely is the content to be cited by AI systems?
//! - **Chunk-Qualität** — How well is content structured for RAG/embedding pipelines?
//! - **Wissensgraph** — How rich is the entity/relationship model?
//! - **AI-Policy** — How is the site configured for AI crawler access?
//!
//! This module derives its analysis from existing audit data (SEO, Security,
//! WCAG) — it does not make additional network or CDP requests.

mod chunks;
mod citation;
mod knowledge_graph;
mod readability;

pub use chunks::{ChunkAnalysis, ChunkQuality, ContentSection};
pub use citation::CitationAnalysis;
pub use knowledge_graph::{
    EntityRelationship, EntitySource, GraphEntity, KnowledgeGraphAnalysis, LinkSuggestion,
};
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
    /// Letter grade (A–F)
    pub grade: String,
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

/// Score for a single AI visibility dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// Dimension name
    pub name: String,
    /// Score (0–100)
    pub score: u32,
    /// Short assessment
    pub label: String,
    /// Individual signals evaluated
    pub signals: Vec<AiSignal>,
}

/// A single measurable signal contributing to a dimension score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSignal {
    /// What was checked
    pub name: String,
    /// Whether the signal is positive
    pub present: bool,
    /// Weight of this signal within its dimension (0.0–1.0)
    pub weight: f32,
    /// Human-readable detail
    pub detail: String,
}

const DISCLAIMER_DE: &str = "Diese Bewertung basiert auf heuristischen Signalen zur \
    AI-Sichtbarkeit. Sie bewertet, wie gut Inhalte für die maschinelle Extraktion \
    und Zitierung durch KI-Systeme aufbereitet sind — nicht deren inhaltliche Qualität.";

// ─── Analysis entry points ──────────────────────────────────────────────────

/// Derive AI visibility analysis from an existing audit report (single page).
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
        grade: score_to_grade(score),
        readability,
        citation,
        chunks,
        knowledge_graph,
        policy,
        disclaimer: DISCLAIMER_DE.to_string(),
    }
}

/// Derive AI visibility for batch mode (average across pages).
pub fn analyze_ai_visibility_batch(reports: &[AuditReport]) -> AiVisibilityAnalysis {
    if reports.is_empty() {
        return empty_analysis();
    }

    let analyses: Vec<AiVisibilityAnalysis> = reports.iter().map(analyze_ai_visibility).collect();

    let avg_score = analyses.iter().map(|a| a.score).sum::<u32>() / analyses.len() as u32;

    // Use the first report's detailed analysis as template, with averaged scores
    let mut result = analyses.into_iter().next().unwrap();
    result.score = avg_score;
    result.grade = score_to_grade(avg_score);

    result
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
    ) = if let Some(seo) = &report.seo {
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
                "Keine robots.txt".to_string(),
            )
        }
    } else {
        (
            false,
            false,
            false,
            false,
            0,
            false,
            "Keine robots.txt".to_string(),
        )
    };

    // 1. robots.txt accessible
    signals.push(AiSignal {
        name: "robots.txt erreichbar".into(),
        present: has_robots,
        weight: 0.15,
        detail: if has_robots {
            "robots.txt vorhanden und lesbar".into()
        } else {
            "robots.txt nicht erreichbar — AI-Crawler haben keine Policy-Info".into()
        },
    });

    // 2. Not blocking all crawlers
    signals.push(AiSignal {
        name: "Kein Wildcard-Block".into(),
        present: !blocks_all,
        weight: 0.20,
        detail: if blocks_all {
            "Disallow: / für alle Crawler — blockiert auch AI-Systeme".into()
        } else {
            "Kein globaler Block — Crawler haben grundsätzlich Zugang".into()
        },
    });

    // 3. Citation/search AI bots accessible — only citation bots matter for real-time AI visibility.
    //    Blocking training bots (GPTBot, Google-Extended etc.) is standard practice and not penalized.
    signals.push(AiSignal {
        name: "KI-Suche erreichbar".into(),
        present: !blocks_citation,
        weight: 0.25,
        detail: if blocks_all {
            "Alle Crawler gesperrt (Disallow: *) — auch KI-Suche blockiert".into()
        } else if blocks_citation {
            "KI-Suchbots (PerplexityBot, Amazonbot etc.) blockiert — Inhalte erscheinen nicht in KI-Antworten".into()
        } else {
            "KI-Suchbots nicht blockiert — Inhalte für KI-Antworten verfügbar".into()
        },
    });

    // 4. Explicit AI policy defined — having any policy is better than silence
    let has_explicit_policy = has_robots && blocked_count > 0;
    signals.push(AiSignal {
        name: "Explizite AI-Policy".into(),
        present: has_explicit_policy,
        weight: 0.15,
        detail: if has_explicit_policy {
            format!(
                "Policy definiert: {} — {} AI-Bots adressiert",
                inferred_policy, blocked_count
            )
        } else if has_robots {
            "Keine explizite AI-Crawler-Regelung in robots.txt".into()
        } else {
            "Keine robots.txt — keine AI-Policy definierbar".into()
        },
    });

    // 5. Sitemap in robots.txt (helps AI crawlers discover content)
    signals.push(AiSignal {
        name: "Sitemap-Verweis".into(),
        present: has_sitemap_in_robots,
        weight: 0.15,
        detail: if has_sitemap_in_robots {
            "Sitemap in robots.txt verlinkt — erleichtert AI-Crawling".into()
        } else {
            "Keine Sitemap in robots.txt — AI-Crawler müssen Seiten selbst entdecken".into()
        },
    });

    // 6. Meta robots (check for noindex/nofollow which affects AI visibility)
    let has_meta_robots_issue = report.seo.as_ref().is_some_and(|seo| {
        seo.technical
            .issues
            .iter()
            .any(|i| i.issue_type.contains("noindex") || i.issue_type.contains("nofollow"))
    });
    signals.push(AiSignal {
        name: "Meta-Robots freigegeben".into(),
        present: !has_meta_robots_issue,
        weight: 0.10,
        detail: if has_meta_robots_issue {
            "noindex/nofollow erkannt — Seite wird von AI-Crawlern gemieden".into()
        } else {
            "Keine Indexierungs-Blockade via Meta-Tags".into()
        },
    });

    PolicyAnalysis {
        dimension: build_dimension("AI-Policy", &signals),
        blocks_ai_citation: blocks_citation,
        blocks_ai_training: blocks_training,
        blocks_all,
        blocked_ai_bot_count: blocked_count,
        inferred_policy,
    }
}

// ─── Input builders ─────────────────────────────────────────────────────────

fn build_readability_input(report: &AuditReport) -> readability::ReadabilityInput {
    let seo = report.seo.as_ref();

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
    let seo = report.seo.as_ref();
    let schema_types = seo.map_or(&[][..], |s| &s.structured_data.types);

    let has_author_schema = schema_types.iter().any(|t| matches!(t, SchemaType::Person));
    let has_org_schema = schema_types
        .iter()
        .any(|t| matches!(t, SchemaType::Organization | SchemaType::LocalBusiness));
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

    let has_meta_description = seo
        .and_then(|s| s.meta.description.as_ref())
        .is_some_and(|d| !d.is_empty());
    let meta_desc_len = seo
        .and_then(|s| s.meta.description.as_ref())
        .map_or(0, |d| d.len());

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
        has_meta_description,
        meta_desc_len,
        security_score: report.security.as_ref().map(|s| s.score),
        a11y_score: report.score,
        has_faq_schema,
        has_lists,
        short_paragraph_ratio,
        has_date_published,
        has_breadcrumb,
    }
}

fn build_chunk_input(report: &AuditReport) -> chunks::ChunkInput {
    let seo = report.seo.as_ref();

    let headings: Vec<chunks::HeadingInfo> = seo.map_or(vec![], |s| {
        let h = &s.headings.headings;
        h.iter()
            .map(|heading| {
                // Estimate word count between headings
                let total_words = s.technical.word_count;
                let n = h.len() as u32;
                let words_per_section = total_words.checked_div(n).unwrap_or(total_words);

                chunks::HeadingInfo {
                    text: heading.text.clone(),
                    level: heading.level as u32,
                    word_count_after: words_per_section,
                }
            })
            .collect()
    });

    let total_word_count = seo.map_or(0, |s| s.technical.word_count);
    let paragraph_count = if !headings.is_empty() {
        headings.len() as u32 * 2
    } else if total_word_count > 100 {
        (total_word_count / 80).max(1)
    } else {
        1
    };

    // Check for semantic HTML from accessibility tree or mobile analysis
    let has_nav_landmarks = report
        .wcag_results
        .violations
        .iter()
        .all(|v| !v.rule_name.contains("Landmark"));
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
    let has_semantic_html = has_nav_landmarks && (has_article_tag || has_section_tags);

    chunks::ChunkInput {
        headings,
        total_word_count,
        paragraph_count,
        has_semantic_html,
        has_nav_landmarks,
        has_article_tag,
        has_section_tags,
    }
}

fn build_knowledge_graph_input(report: &AuditReport) -> knowledge_graph::KnowledgeGraphInput {
    let seo = report.seo.as_ref();

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

    let headings: Vec<String> = seo.map_or(vec![], |s| {
        s.headings.headings.iter().map(|h| h.text.clone()).collect()
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

    let schema_types: Vec<String> = seo.map_or(vec![], |s| {
        s.structured_data
            .types
            .iter()
            .map(|t| format!("{:?}", t))
            .collect()
    });

    knowledge_graph::KnowledgeGraphInput {
        schemas,
        headings,
        page_title,
        site_name,
        internal_links,
        has_breadcrumb,
        schema_types,
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn build_dimension(name: &str, signals: &[AiSignal]) -> DimensionScore {
    if signals.is_empty() {
        return DimensionScore {
            name: name.to_string(),
            score: 0,
            label: "Keine Daten".into(),
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
        name: name.to_string(),
        score,
        label: score_to_label(score),
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

fn score_to_grade(score: u32) -> String {
    match score {
        90..=100 => "A",
        75..=89 => "B",
        60..=74 => "C",
        40..=59 => "D",
        _ => "F",
    }
    .to_string()
}

fn score_to_label(score: u32) -> String {
    match score {
        90..=100 => "Sehr gut",
        75..=89 => "Gut",
        60..=74 => "Befriedigend",
        40..=59 => "Ausbaufähig",
        _ => "Kritisch",
    }
    .to_string()
}

fn empty_analysis() -> AiVisibilityAnalysis {
    let empty_dim = DimensionScore {
        name: String::new(),
        score: 0,
        label: "Keine Daten".into(),
        signals: vec![],
    };

    AiVisibilityAnalysis {
        score: 0,
        grade: "F".into(),
        readability: ReadabilityAnalysis {
            dimension: empty_dim.clone(),
        },
        citation: CitationAnalysis {
            dimension: empty_dim.clone(),
        },
        chunks: ChunkAnalysis {
            dimension: empty_dim.clone(),
            sections: vec![],
            recommendation: String::new(),
        },
        knowledge_graph: KnowledgeGraphAnalysis {
            dimension: empty_dim.clone(),
            entities: vec![],
            relationships: vec![],
            link_suggestions: vec![],
        },
        policy: PolicyAnalysis {
            dimension: empty_dim,
            blocks_ai_citation: false,
            blocks_ai_training: false,
            blocks_all: false,
            blocked_ai_bot_count: 0,
            inferred_policy: String::new(),
        },
        disclaimer: DISCLAIMER_DE.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditReport;
    use crate::audit::ViolationStatistics;
    use crate::cli::WcagLevel;
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
            page_screenshots: None,
        }
    }

    #[test]
    fn test_minimal_report_produces_scores() {
        let report = minimal_report();
        let analysis = analyze_ai_visibility(&report);
        assert!(analysis.score <= 100);
        assert!(!analysis.disclaimer.is_empty());
        assert!(!analysis.grade.is_empty());
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

    #[test]
    fn test_grade_mapping() {
        assert_eq!(score_to_grade(95), "A");
        assert_eq!(score_to_grade(80), "B");
        assert_eq!(score_to_grade(65), "C");
        assert_eq!(score_to_grade(45), "D");
        assert_eq!(score_to_grade(20), "F");
    }

    #[test]
    fn test_weighted_average() {
        assert_eq!(weighted_average(&[(100, 50), (0, 50)]), 50);
        assert_eq!(weighted_average(&[(100, 100)]), 100);
        assert_eq!(weighted_average(&[]), 0);
    }
}
