//! Social media meta tags extraction
//!
//! Extracts OpenGraph and Twitter Card meta tags.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

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
    pub fn is_complete(&self) -> bool {
        self.title.is_some()
            && self.description.is_some()
            && self.image.is_some()
            && self.url.is_some()
    }

    pub fn completeness(&self) -> u32 {
        let fields = [
            self.title.is_some(),
            self.description.is_some(),
            self.image.is_some(),
            self.url.is_some(),
            self.og_type.is_some(),
            self.site_name.is_some(),
        ];
        let count = fields.iter().filter(|&&x| x).count();
        (count * 100 / fields.len()) as u32
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
    pub fn is_complete(&self) -> bool {
        self.card.is_some() && self.title.is_some() && self.description.is_some()
    }

    pub fn completeness(&self) -> u32 {
        let fields = [
            self.card.is_some(),
            self.title.is_some(),
            self.description.is_some(),
            self.image.is_some(),
        ];
        let count = fields.iter().filter(|&&x| x).count();
        (count * 100 / fields.len()) as u32
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

    let json_str = js_result
        .value()
        .and_then(|v| v.as_str())
        .unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    // Parse OpenGraph
    let og = &parsed["og"];
    let open_graph = if og.is_object() && !og.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        Some(OpenGraph {
            title: og["title"].as_str().map(String::from),
            description: og["description"].as_str().map(String::from),
            image: og["image"].as_str().map(String::from),
            url: og["url"].as_str().map(String::from),
            og_type: og["type"].as_str().map(String::from),
            site_name: og["site_name"].as_str().map(String::from),
            locale: og["locale"].as_str().map(String::from),
        })
    } else {
        None
    };

    // Parse Twitter Card
    let tw = &parsed["twitter"];
    let twitter_card = if tw.is_object() && !tw.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        Some(TwitterCard {
            card: tw["card"].as_str().map(String::from),
            title: tw["title"].as_str().map(String::from),
            description: tw["description"].as_str().map(String::from),
            image: tw["image"].as_str().map(String::from),
            site: tw["site"].as_str().map(String::from),
            creator: tw["creator"].as_str().map(String::from),
        })
    } else {
        None
    };

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
}
