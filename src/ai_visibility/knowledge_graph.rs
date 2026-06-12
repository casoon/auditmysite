//! Lightweight Knowledge Graph Builder
//!
//! Builds a knowledge graph from structured data (Schema.org) and internal
//! link structure. Detects entity relationships, suggests internal linking
//! opportunities, and exports as JSON-LD.

use serde::{Deserialize, Serialize};

use super::{
    build_dimension, AiSignal, AiSignalKind, AiSignalValues, DimensionKind, DimensionScore,
};

/// Knowledge graph analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphAnalysis {
    /// Dimension score (0–100) with individual signals
    pub dimension: DimensionScore,
    /// Discovered entities
    pub entities: Vec<GraphEntity>,
    /// Detected relationships between entities
    pub relationships: Vec<EntityRelationship>,
    /// Internal linking suggestions based on entity co-occurrence
    pub link_suggestions: Vec<LinkSuggestion>,
}

/// An entity extracted from structured data or page structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEntity {
    /// Entity name
    pub name: String,
    /// Entity type (Schema.org type or inferred)
    pub entity_type: String,
    /// Source of extraction
    pub source: EntitySource,
    /// Additional properties extracted
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<(String, String)>,
}

/// How an entity was discovered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitySource {
    /// From JSON-LD / Schema.org markup
    SchemaOrg,
    /// From heading structure
    Heading,
    /// From meta tags
    Meta,
}

impl std::fmt::Display for EntitySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntitySource::SchemaOrg => write!(f, "Schema.org"),
            EntitySource::Heading => write!(f, "Heading"),
            EntitySource::Meta => write!(f, "Meta"),
        }
    }
}

/// A relationship between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelationship {
    /// Subject entity name
    pub subject: String,
    /// Relationship type
    pub predicate: String,
    /// Object entity name
    pub object: String,
    /// Source of the relationship
    pub source: EntitySource,
}

/// Which kind of link suggestion reason applies (for localized re-derivation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KgSuggestionKind {
    /// Too few internal links between entities (uses `internal_links`)
    FewInternalLinks,
    /// A topic only recognized as a heading (uses the entity name)
    TopicOnlyHeading,
}

/// Suggestion for internal linking based on entity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSuggestion {
    /// Which reason this is (for localized re-derivation)
    pub kind: KgSuggestionKind,
    /// Internal link count referenced by `reason` (FewInternalLinks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_links: Option<u32>,
    /// Entity that should be linked (canonical English for synthetic labels)
    pub entity: String,
    /// Reason for the suggestion (canonical English)
    pub reason: String,
}

/// Input data for knowledge graph building
pub(crate) struct KnowledgeGraphInput {
    /// JSON-LD schemas found on the page
    pub schemas: Vec<SchemaEntity>,
    /// Page title from meta
    pub page_title: Option<String>,
    /// Site name from meta/schema
    pub site_name: Option<String>,
    /// Internal link targets found on the page
    pub internal_links: u32,
    /// Whether breadcrumb schema exists
    pub has_breadcrumb: bool,
}

/// A schema entity extracted from JSON-LD
pub(crate) struct SchemaEntity {
    pub schema_type: String,
    pub name: Option<String>,
    pub properties: Vec<(String, String)>,
}

pub(crate) fn analyze_knowledge_graph(input: &KnowledgeGraphInput) -> KnowledgeGraphAnalysis {
    let en = true; // bake canonical English; PDF re-derives via stored kinds
    let mut entities = Vec::new();
    let mut relationships = Vec::new();
    let mut link_suggestions = Vec::new();

    // Extract entities from Schema.org
    for schema in &input.schemas {
        if let Some(name) = &schema.name {
            entities.push(GraphEntity {
                name: name.clone(),
                entity_type: schema.schema_type.clone(),
                source: EntitySource::SchemaOrg,
                properties: schema.properties.clone(),
            });
        }
    }

    // Extract entities from page structure
    if let Some(title) = &input.page_title {
        if !title.is_empty() && !entities.iter().any(|e| e.name == *title) {
            entities.push(GraphEntity {
                name: title.clone(),
                entity_type: "WebPage".into(),
                source: EntitySource::Meta,
                properties: vec![],
            });
        }
    }

    if let Some(site) = &input.site_name {
        if !site.is_empty() && !entities.iter().any(|e| e.name == *site) {
            entities.push(GraphEntity {
                name: site.clone(),
                entity_type: "WebSite".into(),
                source: EntitySource::Meta,
                properties: vec![],
            });
        }
    }

    // Build relationships from Schema.org data
    build_schema_relationships(&entities, &input.schemas, &mut relationships);

    // Build relationships from page/site structure
    if let (Some(title), Some(site)) = (&input.page_title, &input.site_name) {
        if !title.is_empty() && !site.is_empty() {
            relationships.push(EntityRelationship {
                subject: title.clone(),
                predicate: "isPartOf".into(),
                object: site.clone(),
                source: EntitySource::Meta,
            });
        }
    }

    // Breadcrumb relationships
    if input.has_breadcrumb {
        if let Some(title) = &input.page_title {
            relationships.push(EntityRelationship {
                subject: title.clone(),
                predicate: "breadcrumbPath".into(),
                object: "BreadcrumbList".into(),
                source: EntitySource::SchemaOrg,
            });
        }
    }

    // Generate link suggestions for entities without rich schema
    if entities.len() > 1 && input.internal_links < 5 {
        link_suggestions.push(LinkSuggestion {
            kind: KgSuggestionKind::FewInternalLinks,
            internal_links: Some(input.internal_links),
            entity: ai_kg_suggestion_entity(KgSuggestionKind::FewInternalLinks, "", en),
            reason: ai_kg_link_suggestion_reason(
                KgSuggestionKind::FewInternalLinks,
                "",
                input.internal_links,
                en,
            ),
        });
    }

    for entity in &entities {
        if entity.source == EntitySource::Heading && entity.entity_type == "Topic" {
            link_suggestions.push(LinkSuggestion {
                kind: KgSuggestionKind::TopicOnlyHeading,
                internal_links: None,
                entity: entity.name.clone(),
                reason: ai_kg_link_suggestion_reason(
                    KgSuggestionKind::TopicOnlyHeading,
                    &entity.name,
                    0,
                    en,
                ),
            });
        }
    }

    // Score signals
    let mut signals = Vec::new();

    // 1. Entity count
    let good_entity_count = entities.len() >= 3;
    signals.push(AiSignal::new(
        AiSignalKind::EntitiesDetected,
        good_entity_count,
        0.20,
        AiSignalValues {
            entity_count: Some(entities.len() as u32),
            ..Default::default()
        },
    ));

    // 2. Schema.org entity ratio
    let schema_entities = entities
        .iter()
        .filter(|e| e.source == EntitySource::SchemaOrg)
        .count();
    let good_schema_ratio = schema_entities >= 2;
    signals.push(AiSignal::new(
        AiSignalKind::SchemaOrgEntities,
        good_schema_ratio,
        0.20,
        AiSignalValues {
            schema_entity_count: Some(schema_entities as u32),
            ..Default::default()
        },
    ));

    // 3. Relationship count
    let good_relationships = relationships.len() >= 2;
    signals.push(AiSignal::new(
        AiSignalKind::Relationships,
        good_relationships,
        0.20,
        AiSignalValues {
            relationship_count: Some(relationships.len() as u32),
            ..Default::default()
        },
    ));

    // 4. Entity types diversity
    let unique_types: std::collections::HashSet<&str> =
        entities.iter().map(|e| e.entity_type.as_str()).collect();
    let diverse = unique_types.len() >= 3;
    signals.push(AiSignal::new(
        AiSignalKind::TypeDiversity,
        diverse,
        0.10,
        AiSignalValues {
            unique_type_count: Some(unique_types.len() as u32),
            ..Default::default()
        },
    ));

    // 5. Breadcrumb hierarchy
    signals.push(AiSignal::new(
        AiSignalKind::BreadcrumbHierarchy,
        input.has_breadcrumb,
        0.10,
        AiSignalValues::default(),
    ));

    // 6. Internal linking density (for graph connectivity)
    let good_linking = input.internal_links >= 5;
    signals.push(AiSignal::new(
        AiSignalKind::LinkingDensity,
        good_linking,
        0.10,
        AiSignalValues {
            internal_links: Some(input.internal_links),
            ..Default::default()
        },
    ));

    // 7. Properties completeness
    let entities_with_props = entities.iter().filter(|e| !e.properties.is_empty()).count();
    let good_props = entities_with_props >= 1 && schema_entities >= 1;
    signals.push(AiSignal::new(
        AiSignalKind::PropertyCompleteness,
        good_props,
        0.10,
        AiSignalValues {
            entities_with_props: Some(entities_with_props as u32),
            entity_count: Some(entities.len() as u32),
            ..Default::default()
        },
    ));

    KnowledgeGraphAnalysis {
        dimension: build_dimension(DimensionKind::KnowledgeGraph, &signals),
        entities,
        relationships,
        link_suggestions,
    }
}

// ─── Link suggestion text (single source of truth) ───────────────────────────

/// Localized entity label for a link suggestion. Real topic names pass through;
/// the synthetic FewInternalLinks "Page" label localizes.
pub(crate) fn ai_kg_suggestion_entity(kind: KgSuggestionKind, name: &str, en: bool) -> String {
    match kind {
        KgSuggestionKind::FewInternalLinks => {
            if en {
                "Page".into()
            } else {
                "Seite".into()
            }
        }
        KgSuggestionKind::TopicOnlyHeading => name.to_string(),
    }
}

/// Localized reason text for a link suggestion (single source of truth).
pub fn ai_kg_link_suggestion_reason(
    kind: KgSuggestionKind,
    entity_name: &str,
    internal_links: u32,
    en: bool,
) -> String {
    match kind {
        KgSuggestionKind::FewInternalLinks => {
            if en {
                format!(
                    "Only {} internal links — entities should be linked to each other",
                    internal_links
                )
            } else {
                format!(
                    "Nur {} interne Links — Entitäten sollten untereinander verlinkt werden",
                    internal_links
                )
            }
        }
        KgSuggestionKind::TopicOnlyHeading => {
            if en {
                format!(
                    "Topic '{}' only recognized as a heading — Schema.org markup or internal linking recommended",
                    entity_name
                )
            } else {
                format!(
                    "Thema '{}' nur als Überschrift erkannt — Schema.org-Markup oder interne Verlinkung empfohlen",
                    entity_name
                )
            }
        }
    }
}

// ─── Signal detail text (single source of truth) ─────────────────────────────

pub(crate) fn detail_entities_detected(present: bool, v: &AiSignalValues, en: bool) -> String {
    let count = v.entity_count.unwrap_or(0);
    if en {
        format!(
            "{} entities extracted — {}",
            count,
            if present {
                "rich knowledge model"
            } else {
                "few machine-readable entities"
            }
        )
    } else {
        format!(
            "{} Entitäten extrahiert — {}",
            count,
            if present {
                "reichhaltiges Wissensmodell"
            } else {
                "wenig maschinenlesbare Entitäten"
            }
        )
    }
}

pub(crate) fn detail_schema_entities(present: bool, v: &AiSignalValues, en: bool) -> String {
    let count = v.schema_entity_count.unwrap_or(0);
    if en {
        format!(
            "{} entities from Schema.org — {}",
            count,
            if present {
                "good machine readability"
            } else {
                "few structured entities"
            }
        )
    } else {
        format!(
            "{} Entitäten aus Schema.org — {}",
            count,
            if present {
                "gute Maschinenlesbarkeit"
            } else {
                "wenig strukturierte Entitäten"
            }
        )
    }
}

pub(crate) fn detail_relationships(present: bool, v: &AiSignalValues, en: bool) -> String {
    let count = v.relationship_count.unwrap_or(0);
    if en {
        format!(
            "{} relationships between entities — {}",
            count,
            if present {
                "knowledge network recognizable"
            } else {
                "isolated entities, no network"
            }
        )
    } else {
        format!(
            "{} Beziehungen zwischen Entitäten — {}",
            count,
            if present {
                "Wissensnetz erkennbar"
            } else {
                "isolierte Entitäten, kein Netz"
            }
        )
    }
}

pub(crate) fn detail_type_diversity(present: bool, v: &AiSignalValues, en: bool) -> String {
    let count = v.unique_type_count.unwrap_or(0);
    if en {
        format!(
            "{} different entity types — {}",
            count,
            if present {
                "diverse knowledge model"
            } else {
                "little type diversity"
            }
        )
    } else {
        format!(
            "{} verschiedene Entitätstypen — {}",
            count,
            if present {
                "vielfältiges Wissensmodell"
            } else {
                "wenig Typen-Diversität"
            }
        )
    }
}

pub(crate) fn detail_breadcrumb(present: bool, en: bool) -> String {
    if present {
        if en {
            "Breadcrumb present — thematic classification in the graph possible".into()
        } else {
            "Breadcrumb vorhanden — thematische Einordnung im Graph möglich".into()
        }
    } else if en {
        "No breadcrumb — thematic classification missing".into()
    } else {
        "Kein Breadcrumb — thematische Einordnung fehlt".into()
    }
}

pub(crate) fn detail_linking_density(present: bool, v: &AiSignalValues, en: bool) -> String {
    let count = v.internal_links.unwrap_or(0);
    if en {
        format!(
            "{} internal links — {}",
            count,
            if present {
                "good connectivity in the graph"
            } else {
                "weak connectivity"
            }
        )
    } else {
        format!(
            "{} interne Links — {}",
            count,
            if present {
                "gute Vernetzung im Graph"
            } else {
                "schwache Vernetzung"
            }
        )
    }
}

pub(crate) fn detail_property_completeness(present: bool, v: &AiSignalValues, en: bool) -> String {
    let with_props = v.entities_with_props.unwrap_or(0);
    let total = v.entity_count.unwrap_or(0);
    if en {
        format!(
            "{}/{} entities with properties — {}",
            with_props,
            total,
            if present {
                "entities are described"
            } else {
                "entities without details"
            }
        )
    } else {
        format!(
            "{}/{} Entitäten mit Eigenschaften — {}",
            with_props,
            total,
            if present {
                "Entitäten sind beschrieben"
            } else {
                "Entitäten ohne Details"
            }
        )
    }
}

fn build_schema_relationships(
    entities: &[GraphEntity],
    schemas: &[SchemaEntity],
    relationships: &mut Vec<EntityRelationship>,
) {
    // Find author/publisher relationships
    for schema in schemas {
        let subject = match &schema.name {
            Some(n) => n.clone(),
            None => continue,
        };

        for (key, value) in &schema.properties {
            match key.as_str() {
                "author" | "creator" => {
                    relationships.push(EntityRelationship {
                        subject: subject.clone(),
                        predicate: key.clone(),
                        object: value.clone(),
                        source: EntitySource::SchemaOrg,
                    });
                }
                "publisher" | "provider" => {
                    relationships.push(EntityRelationship {
                        subject: subject.clone(),
                        predicate: key.clone(),
                        object: value.clone(),
                        source: EntitySource::SchemaOrg,
                    });
                }
                "isPartOf" | "mainEntityOfPage" | "about" => {
                    relationships.push(EntityRelationship {
                        subject: subject.clone(),
                        predicate: key.clone(),
                        object: value.clone(),
                        source: EntitySource::SchemaOrg,
                    });
                }
                _ => {}
            }
        }
    }

    // Connect entities of type Organization to the site
    let org_entities: Vec<&GraphEntity> = entities
        .iter()
        .filter(|e| e.entity_type == "Organization" || e.entity_type == "LocalBusiness")
        .collect();

    let article_entities: Vec<&GraphEntity> = entities
        .iter()
        .filter(|e| {
            e.entity_type == "Article"
                || e.entity_type == "BlogPosting"
                || e.entity_type == "NewsArticle"
        })
        .collect();

    // Infer publisher relationship if not explicit
    for article in &article_entities {
        let has_publisher = relationships
            .iter()
            .any(|r| r.subject == article.name && r.predicate == "publisher");
        if !has_publisher {
            if let Some(org) = org_entities.first() {
                relationships.push(EntityRelationship {
                    subject: article.name.clone(),
                    predicate: "publisher (inferred)".into(),
                    object: org.name.clone(),
                    source: EntitySource::SchemaOrg,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rich_input() -> KnowledgeGraphInput {
        KnowledgeGraphInput {
            schemas: vec![
                SchemaEntity {
                    schema_type: "Article".into(),
                    name: Some("Understanding Rust".into()),
                    properties: vec![
                        ("author".into(), "Jane Doe".into()),
                        ("publisher".into(), "TechBlog".into()),
                    ],
                },
                SchemaEntity {
                    schema_type: "Organization".into(),
                    name: Some("TechBlog".into()),
                    properties: vec![("url".into(), "https://techblog.example".into())],
                },
                SchemaEntity {
                    schema_type: "Person".into(),
                    name: Some("Jane Doe".into()),
                    properties: vec![],
                },
            ],
            page_title: Some("Understanding Rust".into()),
            site_name: Some("TechBlog".into()),
            internal_links: 10,
            has_breadcrumb: true,
        }
    }

    fn minimal_input() -> KnowledgeGraphInput {
        KnowledgeGraphInput {
            schemas: vec![],
            page_title: None,
            site_name: None,
            internal_links: 0,
            has_breadcrumb: false,
        }
    }

    #[test]
    fn rich_input_produces_high_score() {
        let result = analyze_knowledge_graph(&rich_input());
        // Should score well with 3 schemas + breadcrumb + many links
        assert!(result.dimension.score >= 60);
        assert!(!result.entities.is_empty());
        assert!(!result.relationships.is_empty());
    }

    #[test]
    fn minimal_input_produces_low_score() {
        let result = analyze_knowledge_graph(&minimal_input());
        assert_eq!(result.dimension.score, 0);
        assert!(result.entities.is_empty());
        assert!(result.relationships.is_empty());
        assert!(result.link_suggestions.is_empty());
    }

    #[test]
    fn page_and_site_titles_become_entities() {
        let input = KnowledgeGraphInput {
            schemas: vec![],
            page_title: Some("My Page".into()),
            site_name: Some("My Site".into()),
            internal_links: 0,
            has_breadcrumb: false,
        };
        let result = analyze_knowledge_graph(&input);
        let names: Vec<&str> = result.entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"My Page"));
        assert!(names.contains(&"My Site"));
        // A page+site relationship should be inferred
        assert!(result
            .relationships
            .iter()
            .any(|r| r.predicate == "isPartOf"));
    }

    #[test]
    fn low_internal_links_triggers_suggestion() {
        // 2 entities + < 5 links → link suggestion
        let input = KnowledgeGraphInput {
            schemas: vec![SchemaEntity {
                schema_type: "Article".into(),
                name: Some("Article One".into()),
                properties: vec![],
            }],
            page_title: Some("Article One".into()),
            site_name: Some("Blog".into()),
            internal_links: 2,
            has_breadcrumb: false,
        };
        let result = analyze_knowledge_graph(&input);
        assert!(!result.link_suggestions.is_empty());
    }

    #[test]
    fn breadcrumb_adds_relationship() {
        let input = KnowledgeGraphInput {
            schemas: vec![],
            page_title: Some("Leaf Page".into()),
            site_name: None,
            internal_links: 5,
            has_breadcrumb: true,
        };
        let result = analyze_knowledge_graph(&input);
        assert!(result
            .relationships
            .iter()
            .any(|r| r.predicate == "breadcrumbPath"));
    }
}
