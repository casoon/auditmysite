//! Lightweight Knowledge Graph Builder
//!
//! Builds a knowledge graph from structured data (Schema.org) and internal
//! link structure. Detects entity relationships, suggests internal linking
//! opportunities, and exports as JSON-LD.

use serde::{Deserialize, Serialize};

use super::{build_dimension, AiSignal, DimensionScore};

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

/// Suggestion for internal linking based on entity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSuggestion {
    /// Entity that should be linked
    pub entity: String,
    /// Reason for the suggestion
    pub reason: String,
}

/// Input data for knowledge graph building
pub(crate) struct KnowledgeGraphInput {
    /// JSON-LD schemas found on the page
    pub schemas: Vec<SchemaEntity>,
    /// Headings from the page
    #[allow(dead_code)]
    pub headings: Vec<String>,
    /// Page title from meta
    pub page_title: Option<String>,
    /// Site name from meta/schema
    pub site_name: Option<String>,
    /// Internal link targets found on the page
    pub internal_links: u32,
    /// Whether breadcrumb schema exists
    pub has_breadcrumb: bool,
    /// Schema types present
    #[allow(dead_code)]
    pub schema_types: Vec<String>,
}

/// A schema entity extracted from JSON-LD
pub(crate) struct SchemaEntity {
    pub schema_type: String,
    pub name: Option<String>,
    pub properties: Vec<(String, String)>,
}

pub(crate) fn analyze_knowledge_graph(input: &KnowledgeGraphInput) -> KnowledgeGraphAnalysis {
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
            entity: "Seite".into(),
            reason: format!(
                "Nur {} interne Links — Entitäten sollten untereinander verlinkt werden",
                input.internal_links
            ),
        });
    }

    for entity in &entities {
        if entity.source == EntitySource::Heading && entity.entity_type == "Topic" {
            link_suggestions.push(LinkSuggestion {
                entity: entity.name.clone(),
                reason: format!(
                    "Thema '{}' nur als Überschrift erkannt — Schema.org-Markup oder interne Verlinkung empfohlen",
                    entity.name
                ),
            });
        }
    }

    // Score signals
    let mut signals = Vec::new();

    // 1. Entity count
    let good_entity_count = entities.len() >= 3;
    signals.push(AiSignal {
        name: "Entitäten erkannt".into(),
        present: good_entity_count,
        weight: 0.20,
        detail: format!(
            "{} Entitäten extrahiert — {}",
            entities.len(),
            if good_entity_count {
                "reichhaltiges Wissensmodell"
            } else {
                "wenig maschinenlesbare Entitäten"
            }
        ),
    });

    // 2. Schema.org entity ratio
    let schema_entities = entities
        .iter()
        .filter(|e| e.source == EntitySource::SchemaOrg)
        .count();
    let good_schema_ratio = schema_entities >= 2;
    signals.push(AiSignal {
        name: "Schema.org-Entitäten".into(),
        present: good_schema_ratio,
        weight: 0.20,
        detail: format!(
            "{} Entitäten aus Schema.org — {}",
            schema_entities,
            if good_schema_ratio {
                "gute Maschinenlesbarkeit"
            } else {
                "wenig strukturierte Entitäten"
            }
        ),
    });

    // 3. Relationship count
    let good_relationships = relationships.len() >= 2;
    signals.push(AiSignal {
        name: "Beziehungen".into(),
        present: good_relationships,
        weight: 0.20,
        detail: format!(
            "{} Beziehungen zwischen Entitäten — {}",
            relationships.len(),
            if good_relationships {
                "Wissensnetz erkennbar"
            } else {
                "isolierte Entitäten, kein Netz"
            }
        ),
    });

    // 4. Entity types diversity
    let unique_types: std::collections::HashSet<&str> =
        entities.iter().map(|e| e.entity_type.as_str()).collect();
    let diverse = unique_types.len() >= 3;
    signals.push(AiSignal {
        name: "Typen-Vielfalt".into(),
        present: diverse,
        weight: 0.10,
        detail: format!(
            "{} verschiedene Entitätstypen — {}",
            unique_types.len(),
            if diverse {
                "vielfältiges Wissensmodell"
            } else {
                "wenig Typen-Diversität"
            }
        ),
    });

    // 5. Breadcrumb hierarchy
    signals.push(AiSignal {
        name: "Breadcrumb-Hierarchie".into(),
        present: input.has_breadcrumb,
        weight: 0.10,
        detail: if input.has_breadcrumb {
            "Breadcrumb vorhanden — thematische Einordnung im Graph möglich".into()
        } else {
            "Kein Breadcrumb — Seite ist isoliert im Wissensgraph".into()
        },
    });

    // 6. Internal linking density (for graph connectivity)
    let good_linking = input.internal_links >= 5;
    signals.push(AiSignal {
        name: "Verlinkungsdichte".into(),
        present: good_linking,
        weight: 0.10,
        detail: format!(
            "{} interne Links — {}",
            input.internal_links,
            if good_linking {
                "gute Vernetzung im Graph"
            } else {
                "schwache Vernetzung"
            }
        ),
    });

    // 7. Properties completeness
    let entities_with_props = entities.iter().filter(|e| !e.properties.is_empty()).count();
    let good_props = entities_with_props >= 1 && schema_entities >= 1;
    signals.push(AiSignal {
        name: "Eigenschafts-Vollständigkeit".into(),
        present: good_props,
        weight: 0.10,
        detail: format!(
            "{}/{} Entitäten mit Eigenschaften — {}",
            entities_with_props,
            entities.len(),
            if good_props {
                "Entitäten sind beschrieben"
            } else {
                "Entitäten ohne Details"
            }
        ),
    });

    KnowledgeGraphAnalysis {
        dimension: build_dimension("Wissensgraph", &signals),
        entities,
        relationships,
        link_suggestions,
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
