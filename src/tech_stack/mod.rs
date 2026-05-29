//! Tech Stack Detection and Stack-Specific Auditing
//!
//! Detects CMS/frameworks/libraries from in-page signals (JS globals, meta
//! generator, script URL patterns) and performs targeted HTTP probes for
//! stack-specific security/privacy issues (admin panel exposure, user
//! enumeration, version disclosure, etc.).

pub mod module;
pub use module::TechStackModule;

use std::time::Duration;

use chromiumoxide::Page;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::Result;
use crate::taxonomy::Severity;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackAnalysis {
    /// All technologies detected on the page.
    pub detected: Vec<DetectedTech>,
    /// Stack-specific security and privacy findings.
    pub findings: Vec<StackFinding>,
    /// Hardening score 0–100 (100 = no issues found).
    pub score: u32,
    /// Letter grade derived from score.
    pub grade: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFinding {
    /// Which technology this finding relates to.
    pub tech: String,
    /// Short title.
    pub title: String,
    /// Detailed description.
    pub detail: String,
    pub severity: Severity,
    pub fix: Option<String>,
    /// The URL that was probed (for admin-panel checks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_checked: Option<String>,
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

async fn detect_technologies(page: &Page) -> Vec<DetectedTech> {
    let js_result = match page.evaluate(DETECT_JS).await {
        Ok(r) => r,
        Err(e) => {
            warn!("Tech stack JS detection failed: {}", e);
            return vec![];
        }
    };

    let val = match js_result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

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

// ── Stack-specific HTTP probes ────────────────────────────────────────────────

async fn run_stack_audits(detected: &[DetectedTech], base_url: &str) -> Vec<StackFinding> {
    let client = match Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("auditmysite-stackcheck/1.0")
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warn!("Could not build HTTP client for stack audits: {}", e);
            return vec![];
        }
    };

    let base = base_url.trim_end_matches('/');
    let mut findings = Vec::new();

    for tech in detected {
        let mut new_findings = match tech.name.as_str() {
            "WordPress" => audit_wordpress(&client, base, tech).await,
            "Drupal" => audit_drupal(&client, base, tech).await,
            "Joomla" => audit_joomla(&client, base, tech).await,
            "Next.js" => audit_nextjs(&client, base).await,
            _ => vec![],
        };
        findings.append(&mut new_findings);
    }

    findings
}

/// Perform a HEAD/GET request and return the status code, or None on error.
async fn probe(client: &Client, url: &str) -> Option<u16> {
    match client.get(url).send().await {
        Ok(r) => Some(r.status().as_u16()),
        Err(_) => None,
    }
}

async fn audit_wordpress(client: &Client, base: &str, tech: &DetectedTech) -> Vec<StackFinding> {
    let mut findings = Vec::new();

    // Version disclosure via meta generator
    if let Some(ref version) = tech.version {
        findings.push(StackFinding {
            tech: "WordPress".into(),
            title: "WordPress version disclosed via meta generator".into(),
            detail: format!(
                "The page exposes WordPress version {version} in the <meta name=\"generator\"> tag. \
                 Attackers can target known vulnerabilities for this exact version."
            ),
            severity: Severity::Low,
            fix: Some(
                "Remove the generator meta tag. Add `remove_action('wp_head', 'wp_generator');` \
                 to functions.php or use a security plugin."
                    .into(),
            ),
            url_checked: None,
        });
    }

    // /wp-admin/ — login page exposure
    let admin_url = format!("{base}/wp-admin/");
    if let Some(status) = probe(client, &admin_url).await {
        if status == 200 || (300..400).contains(&(status as u32)) {
            findings.push(StackFinding {
                tech: "WordPress".into(),
                title: "WordPress admin login accessible".into(),
                detail: format!(
                    "/wp-admin/ is reachable (HTTP {status}). The login form is publicly \
                     accessible, enabling brute-force and credential-stuffing attacks."
                ),
                severity: Severity::Medium,
                fix: Some(
                    "Restrict /wp-admin/ by IP (nginx/Apache), use two-factor authentication, \
                     and rate-limit login attempts (e.g. Wordfence, Limit Login Attempts)."
                        .into(),
                ),
                url_checked: Some(admin_url),
            });
        }
    }

    // /wp-json/wp/v2/users — user enumeration
    let users_url = format!("{base}/wp-json/wp/v2/users");
    if let Some(200) = probe(client, &users_url).await {
        findings.push(StackFinding {
            tech: "WordPress".into(),
            title: "WordPress REST API exposes user list".into(),
            detail: format!(
                "{users_url} returns HTTP 200 and leaks usernames and user IDs. \
                 Attackers can enumerate accounts for targeted login attacks."
            ),
            severity: Severity::High,
            fix: Some(
                "Disable user enumeration via the REST API. Add \
                 `add_filter('rest_endpoints', function($e) {{ unset($e['/wp/v2/users']); \
                 unset($e['/wp/v2/users/(?P<id>[\\d]+)']); return $e; }});` \
                 or use a security plugin."
                    .into(),
            ),
            url_checked: Some(users_url),
        });
    }

    // /xmlrpc.php — XML-RPC interface
    let xmlrpc_url = format!("{base}/xmlrpc.php");
    if let Some(status) = probe(client, &xmlrpc_url).await {
        // xmlrpc.php returns 200 + XML body or 405 when accessible
        if status == 200 || status == 405 {
            findings.push(StackFinding {
                tech: "WordPress".into(),
                title: "WordPress XML-RPC interface is accessible".into(),
                detail: format!(
                    "/xmlrpc.php responded with HTTP {status}. XML-RPC is a legacy remote \
                     procedure call interface that can be abused for brute-force login \
                     amplification and DDoS reflection."
                ),
                severity: Severity::Medium,
                fix: Some(
                    "Disable XML-RPC unless required: add `add_filter('xmlrpc_enabled', '__return_false');` \
                     to functions.php, or block the endpoint at the web server level."
                        .into(),
                ),
                url_checked: Some(xmlrpc_url),
            });
        }
    }

    findings
}

async fn audit_drupal(client: &Client, base: &str, tech: &DetectedTech) -> Vec<StackFinding> {
    let mut findings = Vec::new();

    if let Some(ref version) = tech.version {
        findings.push(StackFinding {
            tech: "Drupal".into(),
            title: "Drupal version disclosed via meta generator".into(),
            detail: format!(
                "The page exposes Drupal version {version} in <meta name=\"generator\">. \
                 This allows targeted exploitation of known CVEs."
            ),
            severity: Severity::Low,
            fix: Some(
                "Disable the generator meta tag via Drupal's security settings or a hardening module."
                    .into(),
            ),
            url_checked: None,
        });
    }

    let login_url = format!("{base}/user/login");
    if let Some(200) = probe(client, &login_url).await {
        findings.push(StackFinding {
            tech: "Drupal".into(),
            title: "Drupal user login page accessible".into(),
            detail: "/user/login is publicly accessible. Brute-force and credential-stuffing \
                 attacks are possible without additional rate limiting."
                .to_string(),
            severity: Severity::Medium,
            fix: Some(
                "Enable Drupal's built-in flood control. Add IP-based rate limiting via \
                 web server rules or a security module. Consider moving admin access to a \
                 non-standard path."
                    .into(),
            ),
            url_checked: Some(login_url),
        });
    }

    let admin_url = format!("{base}/admin/");
    if let Some(status) = probe(client, &admin_url).await {
        if status == 200 {
            findings.push(StackFinding {
                tech: "Drupal".into(),
                title: "Drupal admin interface accessible without redirect".into(),
                detail: "/admin/ returned HTTP 200. Verify that unauthenticated access is not possible."
                    .into(),
                severity: Severity::High,
                fix: Some(
                    "Ensure /admin/ requires authentication. Check Drupal's access control settings \
                     and consider restricting by IP at the web server level."
                        .into(),
                ),
                url_checked: Some(admin_url),
            });
        }
    }

    findings
}

async fn audit_joomla(client: &Client, base: &str, tech: &DetectedTech) -> Vec<StackFinding> {
    let mut findings = Vec::new();

    if let Some(ref version) = tech.version {
        findings.push(StackFinding {
            tech: "Joomla".into(),
            title: "Joomla version disclosed via meta generator".into(),
            detail: format!(
                "The page exposes Joomla version {version} in <meta name=\"generator\">. \
                 This allows targeted exploitation of known vulnerabilities."
            ),
            severity: Severity::Low,
            fix: Some(
                "Disable the generator tag in Joomla's Global Configuration under the \
                 Metadata Settings."
                    .into(),
            ),
            url_checked: None,
        });
    }

    let admin_url = format!("{base}/administrator/");
    if let Some(status) = probe(client, &admin_url).await {
        if status == 200 || (300..400).contains(&(status as u32)) {
            findings.push(StackFinding {
                tech: "Joomla".into(),
                title: "Joomla administrator panel accessible".into(),
                detail: format!(
                    "/administrator/ is reachable (HTTP {status}). The admin login form is \
                     publicly accessible."
                ),
                severity: Severity::Medium,
                fix: Some(
                    "Restrict /administrator/ by IP using .htaccess or web server configuration. \
                     Enable two-factor authentication for administrator accounts."
                        .into(),
                ),
                url_checked: Some(admin_url),
            });
        }
    }

    findings
}

async fn audit_nextjs(client: &Client, base: &str) -> Vec<StackFinding> {
    let mut findings = Vec::new();

    // /__nextjs_original-stack-frames — should only be active in dev mode
    let debug_url = format!("{base}/__nextjs_original-stack-frames");
    if let Some(status) = probe(client, &debug_url).await {
        if status != 404 {
            findings.push(StackFinding {
                tech: "Next.js".into(),
                title: "Next.js debug endpoint reachable in production".into(),
                detail: format!(
                    "/__nextjs_original-stack-frames responded with HTTP {status}. \
                     This endpoint is intended for development-mode error overlays and \
                     should not be accessible in production — it can leak internal \
                     file paths and source structure."
                ),
                severity: Severity::Medium,
                fix: Some(
                    "Ensure NODE_ENV=production when building and deploying. This endpoint \
                     is automatically disabled in production builds."
                        .into(),
                ),
                url_checked: Some(debug_url),
            });
        }
    }

    findings
}

// ── Scoring ───────────────────────────────────────────────────────────────────

fn calculate_score(findings: &[StackFinding]) -> u32 {
    let deduction: u32 = findings
        .iter()
        .map(|f| match f.severity {
            Severity::Critical => 30,
            Severity::High => 20,
            Severity::Medium => 10,
            Severity::Low => 5,
        })
        .sum();
    100u32.saturating_sub(deduction)
}

fn score_to_grade(score: u32) -> String {
    match score {
        90..=100 => "A",
        75..=89 => "B",
        60..=74 => "C",
        45..=59 => "D",
        _ => "F",
    }
    .to_string()
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Detect the technology stack of the loaded page and run stack-specific audits.
pub async fn analyze_tech_stack(page: &Page, base_url: &str) -> Result<TechStackAnalysis> {
    info!("Running tech stack detection for {}", base_url);

    let detected = detect_technologies(page).await;
    let findings = run_stack_audits(&detected, base_url).await;
    let score = calculate_score(&findings);
    let grade = score_to_grade(score);

    info!(
        "Tech stack: {} technologies detected, {} findings (score {})",
        detected.len(),
        findings.len(),
        score
    );

    Ok(TechStackAnalysis {
        detected,
        findings,
        score,
        grade,
    })
}
