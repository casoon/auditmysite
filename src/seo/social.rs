//! Social media meta tags extraction
//!
//! Extracts OpenGraph and Twitter Card meta tags.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};
use crate::seo::meta::MetaValidation;
use crate::taxonomy::Severity;
use web_checks::social;

/// Social media meta tags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialTags {
    /// OpenGraph tags
    pub open_graph: Option<OpenGraph>,
    /// Twitter Card tags
    pub twitter_card: Option<TwitterCard>,
    /// Completeness score (0-100)
    pub completeness: u32,
}

/// OpenGraph meta tags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenGraph {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub url: Option<String>,
    pub og_type: Option<String>,
    pub site_name: Option<String>,
    pub locale: Option<String>,
}

impl OpenGraph {
    /// Inhalt eines Tags nach seinem Namen (`og:title`, …).
    fn content(&self, tag: &str) -> Option<&str> {
        match tag {
            "og:title" => self.title.as_deref(),
            "og:description" => self.description.as_deref(),
            "og:image" => self.image.as_deref(),
            "og:url" => self.url.as_deref(),
            "og:type" => self.og_type.as_deref(),
            "og:site_name" => self.site_name.as_deref(),
            "og:locale" => self.locale.as_deref(),
            _ => None,
        }
    }

    pub fn is_complete(&self) -> bool {
        social::is_complete(&social::OPEN_GRAPH_REQUIRED, |t| self.content(t))
    }

    pub fn completeness(&self) -> u32 {
        social::completeness(&social::OPEN_GRAPH_FIELDS, |t| self.content(t))
    }
}

/// Twitter Card meta tags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TwitterCard {
    pub card: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub site: Option<String>,
    pub creator: Option<String>,
}

impl TwitterCard {
    /// Inhalt eines Tags nach seinem Namen (`twitter:card`, …).
    fn content(&self, tag: &str) -> Option<&str> {
        match tag {
            "twitter:card" => self.card.as_deref(),
            "twitter:title" => self.title.as_deref(),
            "twitter:description" => self.description.as_deref(),
            "twitter:image" => self.image.as_deref(),
            "twitter:site" => self.site.as_deref(),
            "twitter:creator" => self.creator.as_deref(),
            _ => None,
        }
    }

    pub fn is_complete(&self) -> bool {
        social::is_complete(&social::TWITTER_REQUIRED, |t| self.content(t))
    }

    pub fn completeness(&self) -> u32 {
        social::completeness(&social::TWITTER_FIELDS, |t| self.content(t))
    }
}

impl SocialTags {
    /// Wertprüfungen aus `web_checks::social`: ein `twitter:card`-Typ, den
    /// Twitter nicht kennt, und ein relatives `og:image`, das Plattformen nicht
    /// gegen die Seite auflösen.
    pub fn validate(&self) -> Vec<MetaValidation> {
        let mut issues = Vec::new();
        if let Some(card) = self.twitter_card.as_ref().and_then(|t| t.card.as_deref()) {
            if !social::is_valid_twitter_card(card) {
                issues.push(MetaValidation {
                    field: "twitter:card".to_string(),
                    message: format!(
                        "Invalid twitter:card value \"{}\" (allowed: {})",
                        card.trim(),
                        social::TWITTER_CARD_TYPES.join(", ")
                    ),
                    severity: Severity::Medium,
                    suggestion: Some("Use summary_large_image".to_string()),
                });
            }
        }
        if let Some(image) = self.open_graph.as_ref().and_then(|o| o.image.as_deref()) {
            if !social::is_absolute_url(image) {
                issues.push(MetaValidation {
                    field: "og:image".to_string(),
                    message: format!("og:image is not an absolute URL: \"{}\"", image.trim()),
                    severity: Severity::Medium,
                    suggestion: Some(
                        "Use an absolute URL (https://...) so social platforms can fetch the image"
                            .to_string(),
                    ),
                });
            }
        }
        issues
    }
}

/// Extract social media meta tags
pub async fn extract_social_tags(page: &Page) -> Result<SocialTags> {
    info!("Extracting social media tags...");

    let js_code = r#"
    (() => {
        const result = { og: {}, twitter: {} };

        // OpenGraph tags
        const ogTags = ['title', 'description', 'image', 'url', 'type', 'site_name', 'locale'];
        ogTags.forEach(tag => {
            const el = document.querySelector(`meta[property="og:${tag}"]`);
            if (el) result.og[tag] = el.getAttribute('content');
        });

        // Twitter Card tags
        const twitterTags = ['card', 'title', 'description', 'image', 'site', 'creator'];
        twitterTags.forEach(tag => {
            const el = document.querySelector(`meta[name="twitter:${tag}"]`);
            if (el) result.twitter[tag] = el.getAttribute('content');
        });

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Social tags extraction failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    let (open_graph, twitter_card) = parse_social_tags(&parsed);

    // Calculate completeness
    let og_score = open_graph.as_ref().map(|o| o.completeness()).unwrap_or(0);
    let tw_score = twitter_card.as_ref().map(|t| t.completeness()).unwrap_or(0);
    let completeness = (og_score + tw_score) / 2;

    info!(
        "Social tags: OG={}, Twitter={}, completeness={}%",
        open_graph.is_some(),
        twitter_card.is_some(),
        completeness
    );

    Ok(SocialTags {
        open_graph,
        twitter_card,
        completeness,
    })
}

/// Liest die Tags aus dem Ergebnis der Seitenabfrage. Ein Tag mit leerem
/// `content` gilt als fehlend (`web_checks::social::is_present`); eine Gruppe
/// ohne ein vorhandenes Tag als nicht vorhanden.
fn parse_social_tags(parsed: &serde_json::Value) -> (Option<OpenGraph>, Option<TwitterCard>) {
    let read = |group: &serde_json::Value, key: &str| {
        group[key]
            .as_str()
            .filter(|v| social::is_present(Some(v)))
            .map(String::from)
    };
    let og = &parsed["og"];
    let open_graph = OpenGraph {
        title: read(og, "title"),
        description: read(og, "description"),
        image: read(og, "image"),
        url: read(og, "url"),
        og_type: read(og, "type"),
        site_name: read(og, "site_name"),
        locale: read(og, "locale"),
    };
    let tw = &parsed["twitter"];
    let twitter_card = TwitterCard {
        card: read(tw, "card"),
        title: read(tw, "title"),
        description: read(tw, "description"),
        image: read(tw, "image"),
        site: read(tw, "site"),
        creator: read(tw, "creator"),
    };
    let og_any = [
        "og:title",
        "og:description",
        "og:image",
        "og:url",
        "og:type",
        "og:site_name",
        "og:locale",
    ]
    .iter()
    .any(|t| open_graph.content(t).is_some());
    let tw_any = [
        "twitter:card",
        "twitter:title",
        "twitter:description",
        "twitter:image",
        "twitter:site",
        "twitter:creator",
    ]
    .iter()
    .any(|t| twitter_card.content(t).is_some());
    (og_any.then_some(open_graph), tw_any.then_some(twitter_card))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opengraph_completeness() {
        let og = OpenGraph {
            title: Some("Title".to_string()),
            description: Some("Description".to_string()),
            image: Some("image.jpg".to_string()),
            url: Some("https://example.com".to_string()),
            og_type: Some("website".to_string()),
            site_name: Some("Example".to_string()),
            locale: None,
        };

        assert!(og.is_complete());
        assert!(og.completeness() >= 80);
    }

    #[test]
    fn test_twitter_card_completeness() {
        let tw = TwitterCard {
            card: Some("summary_large_image".to_string()),
            title: Some("Title".to_string()),
            description: Some("Description".to_string()),
            image: Some("image.jpg".to_string()),
            site: None,
            creator: None,
        };

        assert!(tw.is_complete());
        assert_eq!(tw.completeness(), 100);
    }

    #[test]
    fn leerer_inhalt_gilt_als_fehlend() {
        let parsed = serde_json::json!({
            "og": {"title": "T", "image": "  "},
            "twitter": {"card": ""}
        });
        let (og, tw) = parse_social_tags(&parsed);
        let og = og.expect("og:title ist vorhanden");
        assert_eq!(og.image, None);
        assert!(
            tw.is_none(),
            "nur leere Twitter-Tags zählen als keine Karte"
        );
    }

    #[test]
    fn wertpruefungen_melden_ungueltige_karte_und_relatives_bild() {
        let tags = SocialTags {
            open_graph: Some(OpenGraph {
                image: Some("/og.png".to_string()),
                ..Default::default()
            }),
            twitter_card: Some(TwitterCard {
                card: Some("large".to_string()),
                ..Default::default()
            }),
            completeness: 0,
        };
        let fields: Vec<_> = tags.validate().into_iter().map(|i| i.field).collect();
        assert_eq!(fields, ["twitter:card", "og:image"]);

        let ok = SocialTags {
            open_graph: Some(OpenGraph {
                image: Some("https://example.com/og.png".to_string()),
                ..Default::default()
            }),
            twitter_card: Some(TwitterCard {
                card: Some("summary".to_string()),
                ..Default::default()
            }),
            completeness: 0,
        };
        assert!(ok.validate().is_empty());
    }
}
