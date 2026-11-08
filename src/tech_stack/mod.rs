//! Tech Stack Detection
//!
//! Detects CMS/frameworks/libraries from in-page signals (JS globals, meta
//! generator, script URL patterns). Read-only and score-neutral: the module
//! sends no request of its own to the audited host.

pub mod module;
pub use module::TechStackModule;

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

const DETECT_JS: &str = r#"
(function() {
  const result = {};
  const metaGenerator = (document.querySelector('meta[name="generator"]') || {}).content || '';
  const scripts = Array.from(document.querySelectorAll('script[src]')).map(s => s.src);
  const links   = Array.from(document.querySelectorAll('link[href]')).map(l => l.href);
  const allSrc  = scripts.concat(links);

  // WordPress
  const wpMeta   = /WordPress\s*([\d.]+)?/i.exec(metaGenerator);
  const wpSrc    = allSrc.some(s => /\/wp-content\/|\/wp-includes\//.test(s));
  const wpGlobal = typeof window.wp !== 'undefined';
  const wooSrc   = allSrc.some(s => /woocommerce/.test(s));
  const wooClass = !!document.querySelector('.woocommerce, .wc-block');
  if (wpMeta || wpSrc || wpGlobal) {
    result.wordpress = {
      version: wpMeta ? (wpMeta[1] || null) : null,
      signals: [wpMeta ? 'meta generator' : null, wpSrc ? 'wp-content/wp-includes URLs' : null, wpGlobal ? 'window.wp' : null].filter(Boolean),
    };
    if (wooSrc || wooClass) {
      result.woocommerce = { signals: [wooSrc ? 'woocommerce script URLs' : null, wooClass ? '.woocommerce CSS class' : null].filter(Boolean) };
    }
  }

  // Drupal
  const drupalMeta = /Drupal\s*([\d.]+)?/i.exec(metaGenerator);
  const drupalGlob = typeof window.Drupal !== 'undefined';
  const drupalSrc  = allSrc.some(s => /\/sites\/default\/|\/modules\/|drupal/.test(s));
  if (drupalMeta || drupalGlob || drupalSrc) {
    result.drupal = {
      version: drupalMeta ? (drupalMeta[1] || null) : null,
      signals: [drupalMeta ? 'meta generator' : null, drupalGlob ? 'window.Drupal' : null, drupalSrc ? 'Drupal asset URLs' : null].filter(Boolean),
    };
  }

  // Joomla
  const joomlaMeta = /Joomla[!]?\s*([\d.]+)?/i.exec(metaGenerator);
  const joomlaSrc  = allSrc.some(s => /\/media\/jui\/|\/media\/system\//.test(s));
  if (joomlaMeta || joomlaSrc) {
    result.joomla = {
      version: joomlaMeta ? (joomlaMeta[1] || null) : null,
      signals: [joomlaMeta ? 'meta generator' : null, joomlaSrc ? 'Joomla media URLs' : null].filter(Boolean),
    };
  }

  // Next.js
  const nextData  = typeof window.__NEXT_DATA__ !== 'undefined';
  const nextSrc   = scripts.some(s => /\/_next\//.test(s));
  if (nextData || nextSrc) {
    const buildId = (window.__NEXT_DATA__ || {}).buildId || null;
    result.nextjs = { buildId, signals: [nextData ? 'window.__NEXT_DATA__' : null, nextSrc ? '_next script URLs' : null].filter(Boolean) };
  }

  // Nuxt.js
  const nuxtData = typeof window.__NUXT__ !== 'undefined';
  const nuxtSrc  = scripts.some(s => /\/_nuxt\//.test(s));
  if (nuxtData || nuxtSrc) {
    result.nuxt = { signals: [nuxtData ? 'window.__NUXT__' : null, nuxtSrc ? '_nuxt script URLs' : null].filter(Boolean) };
  }

  // Astro
  const astroMeta = /Astro\s*v?([\d.]+)?/i.exec(metaGenerator);
  const astroSrc  = allSrc.some(s => /\/_astro\//.test(s));
  if (astroMeta || astroSrc) {
    result.astro = {
      version: astroMeta ? (astroMeta[1] || null) : null,
      signals: [astroMeta ? 'meta generator' : null, astroSrc ? '_astro asset URLs' : null].filter(Boolean),
    };
  }

  // Gatsby
  const gatsbyGlob = typeof window.___gatsby !== 'undefined';
  const gatsbySrc  = scripts.some(s => /gatsby/.test(s));
  if (gatsbyGlob || gatsbySrc) {
    result.gatsby = { signals: [gatsbyGlob ? 'window.___gatsby' : null, gatsbySrc ? 'gatsby script URLs' : null].filter(Boolean) };
  }

  // Shopify
  const shopifyGlob = typeof window.Shopify !== 'undefined';
  const shopifyMeta = !!document.querySelector('meta[name="shopify-checkout-api-token"]');
  if (shopifyGlob || shopifyMeta) {
    result.shopify = { signals: [shopifyGlob ? 'window.Shopify' : null, shopifyMeta ? 'shopify meta tag' : null].filter(Boolean) };
  }

  // React (standalone — not via Next/Gatsby which already capture it)
  const reactHook = typeof window.__REACT_DEVTOOLS_GLOBAL_HOOK__ !== 'undefined';
  const reactRoot = !!document.querySelector('[data-reactroot]');
  if (!result.nextjs && !result.gatsby && (reactHook || reactRoot)) {
    result.react = { signals: [reactHook ? '__REACT_DEVTOOLS_GLOBAL_HOOK__' : null, reactRoot ? 'data-reactroot attribute' : null].filter(Boolean) };
  }

  // Vue.js
  const vueHook  = typeof window.__vue_devtools_global_hook__ !== 'undefined';
  const vueAttr  = !!document.querySelector('[data-v-app]');
  const vue3glob = typeof window.__VUE__ !== 'undefined';
  if (!result.nuxt && (vueHook || vueAttr || vue3glob)) {
    result.vue = { signals: [vue3glob ? 'window.__VUE__' : null, vueAttr ? 'data-v-app attribute' : null, vueHook ? 'Vue devtools hook' : null].filter(Boolean) };
  }

  // Angular
  const ngVersion = document.querySelector('[ng-version]');
  if (ngVersion) {
    result.angular = { version: ngVersion.getAttribute('ng-version'), signals: ['ng-version attribute'] };
  }

  // Svelte
  const svelteClass = document.querySelector('[class*="svelte-"]');
  if (svelteClass) {
    result.svelte = { signals: ['svelte-* CSS classes'] };
  }

  return result;
})()
"#;

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
pub async fn analyze_tech_stack(
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
