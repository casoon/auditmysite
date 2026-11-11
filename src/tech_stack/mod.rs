//! Tech Stack Detection
//!
//! Detects CMS/frameworks/libraries from in-page signals (JS globals, meta
//! generator, script URL patterns). Read-only and score-neutral: the module
//! sends no request of its own to the audited host.

pub(crate) mod module;
pub(crate) use module::TechStackModule;

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::audit::collection::{Collected, CollectionLog, CollectionMethod, CollectorSpec};
use crate::error::Result;

/// Technology fingerprints on the page (globals, markup, asset paths).
pub(crate) static DETECTION: CollectorSpec = CollectorSpec {
    id: "tech_stack.detection",
    module: "tech_stack",
    method: CollectionMethod::JavaScript,
    method_version: 1,
    scoring: false,
};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackAnalysis {
    /// All technologies detected on the page.
    pub detected: Vec<DetectedTech>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTech {
    pub name: String,
    pub category: TechCategory,
    /// Detected version string, if exposed.
    pub version: Option<String>,
    pub confidence: Confidence,
    /// Human-readable detection signals.
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TechCategory {
    Cms,
    Framework,
    StaticSiteGenerator,
    Ecommerce,
    JsLibrary,
}

impl TechCategory {
    /// Display label — identical in German and English (proper nouns/acronyms).
    pub fn label(&self) -> &'static str {
        match self {
            TechCategory::Cms => "CMS",
            TechCategory::Framework => "Framework",
            TechCategory::StaticSiteGenerator => "Static Site Generator",
            TechCategory::Ecommerce => "E-Commerce",
            TechCategory::JsLibrary => "JS Library",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

// ── Detection JavaScript ──────────────────────────────────────────────────────

const DETECT_JS: &str = include_str!("js/detect.js");

// ── Detection ─────────────────────────────────────────────────────────────────

/// The detected technologies. A script that did not answer is a failure,
/// not a page without a recognizable stack (#777).
async fn detect_technologies(page: &Page) -> Collected<Vec<DetectedTech>> {
    let val = match crate::audit::collection::evaluate_js_value::<
        serde_json::Map<String, serde_json::Value>,
    >(page, DETECT_JS)
    .await
    .into_measurement()
    {
        Ok(map) => serde_json::Value::Object(map),
        Err(failure) => {
            warn!("Tech stack JS detection failed");
            return failure;
        }
    };
    Collected::Measured(technologies_from(&val)).empty_if(Vec::is_empty)
}

fn technologies_from(val: &serde_json::Value) -> Vec<DetectedTech> {
    let mut detected = Vec::new();

    macro_rules! detect {
        ($key:literal, $name:literal, $cat:expr, $conf_high:expr) => {
            if let Some(obj) = val.get($key) {
                let version = obj
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let signals: Vec<String> = obj
                    .get("signals")
                    .and_then(|s| s.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                let confidence = if $conf_high || signals.len() > 1 {
                    Confidence::High
                } else {
                    Confidence::Medium
                };
                detected.push(DetectedTech {
                    name: $name.to_string(),
                    category: $cat,
                    version,
                    confidence,
                    signals,
                });
            }
        };
    }

    detect!("wordpress", "WordPress", TechCategory::Cms, false);
    detect!("woocommerce", "WooCommerce", TechCategory::Ecommerce, false);
    detect!("drupal", "Drupal", TechCategory::Cms, false);
    detect!("joomla", "Joomla", TechCategory::Cms, false);
    detect!("shopify", "Shopify", TechCategory::Ecommerce, false);
    detect!("nextjs", "Next.js", TechCategory::Framework, true);
    detect!("nuxt", "Nuxt.js", TechCategory::Framework, false);
    detect!("astro", "Astro", TechCategory::StaticSiteGenerator, false);
    detect!("gatsby", "Gatsby", TechCategory::StaticSiteGenerator, false);
    detect!("react", "React", TechCategory::JsLibrary, false);
    detect!("vue", "Vue.js", TechCategory::JsLibrary, false);
    detect!("angular", "Angular", TechCategory::Framework, true);
    detect!("svelte", "Svelte", TechCategory::JsLibrary, false);

    detected
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Detect the technology stack of the loaded page from in-page signals.
pub(crate) async fn analyze_tech_stack(
    page: &Page,
    base_url: &str,
    log: &CollectionLog,
) -> Result<TechStackAnalysis> {
    info!("Running tech stack detection for {}", base_url);

    let detected = log
        .record(&DETECTION, detect_technologies(page).await)
        .into_result(&DETECTION)?;

    info!("Tech stack: {} technologies detected", detected.len());

    Ok(TechStackAnalysis { detected })
}

/// Counter-case of [`DETECTION`] (#777): a script that did not answer is a
/// failure, not a page without a recognizable stack.
#[cfg(test)]
pub(crate) fn counter_cases() {
    use crate::audit::collection::assert_js_value_contract;
    assert_js_value_contract::<serde_json::Map<String, serde_json::Value>>(serde_json::json!({}));
}

#[cfg(test)]
mod tests {
    use super::TechCategory;

    /// Regression test for a debug-format leak found via a report-critic eval
    /// (#511 corpus): the PDF technology table used to render `{:?}` on
    /// `TechCategory` directly, so a real report showed raw Rust identifier
    /// text like "StaticSiteGenerator" instead of a readable label. Checks
    /// the multi-word variants, where the Debug spelling is clearly not a
    /// presentable label (single-word variants like `Framework` coincide
    /// with their Debug form and aren't a useful signal either way).
    #[test]
    fn tech_category_label_is_a_readable_string_not_the_debug_identifier() {
        assert_eq!(TechCategory::Cms.label(), "CMS");
        assert_eq!(
            TechCategory::StaticSiteGenerator.label(),
            "Static Site Generator"
        );
        assert_eq!(TechCategory::Ecommerce.label(), "E-Commerce");
        assert_eq!(TechCategory::JsLibrary.label(), "JS Library");
    }
}
