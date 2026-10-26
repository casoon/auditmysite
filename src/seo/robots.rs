//! robots.txt audit — informational, no score impact.
//!
//! Fetches and parses the robots.txt of the audited domain. Classifies known
//! bots by category (search engines, verified AI crawlers, unverified AI
//! crawlers, generic crawlers) and surfaces key findings in the report.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// robots.txt audit result. Never affects the SEO score — informational only.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RobotsAudit {
    /// Whether robots.txt was fetched successfully
    pub fetched: bool,
    /// HTTP or parse error, if any
    pub error: Option<String>,
    /// Parsed rule groups (one per User-agent block)
    pub groups: Vec<RobotsGroup>,
    /// Sitemap directives found in robots.txt
    pub sitemaps: Vec<String>,
    /// (user-agent, delay-seconds) pairs
    pub crawl_delays: Vec<(String, u32)>,
    /// True when the audited page has noindex AND appears in the sitemap.xml
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noindex_in_sitemap: Option<bool>,
    /// True when `User-agent: *` has `Disallow: /` — blocks everything
    pub has_wildcard_disallow_all: bool,
    /// True when any AI crawler (any sub-type) is blocked — legacy alias
    pub blocks_ai_crawlers: bool,
    /// True when training-only bots (GPTBot, Google-Extended, …) are blocked
    pub blocks_ai_training: bool,
    /// True when citation/search AI bots (PerplexityBot, Amazonbot, …) are blocked
    pub blocks_ai_citation: bool,
    /// Human-readable policy label inferred from the rules
    pub inferred_policy: String,
}

/// A single rule group in robots.txt (one or more User-agents sharing Allow/Disallow rules)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsGroup {
    pub user_agent: String,
    pub bot_class: BotClass,
    pub allows: Vec<String>,
    pub disallows: Vec<String>,
}

/// Bot category, used for display grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotClass {
    /// Wildcard (*) rule
    Wildcard,
    /// Verified search engine crawler (Googlebot, Bingbot, …)
    SearchEngine,
    /// AI training bot — blocking is standard practice (GPTBot, Google-Extended, CCBot, …)
    AiTraining,
    /// AI citation / search bot — blocking is unusual (PerplexityBot, Amazonbot, …)
    AiCitation,
    /// AI bot with mixed purpose: both training and citation (ClaudeBot, meta-externalagent, …)
    AiMixed,
    /// Unverified scrapers, SEO bots (dotbot, semrushbot, …)
    UnknownAi,
    /// General / known-good (archive.org, etc.)
    General,
    /// Not in any known list
    Unknown,
}

impl BotClass {
    /// Human-readable, report-visible label for this bot class.
    pub fn label(&self, en: bool) -> &'static str {
        match self {
            BotClass::Wildcard => {
                if en {
                    "All crawlers (*)"
                } else {
                    "Alle Crawler (*)"
                }
            }
            BotClass::SearchEngine => {
                if en {
                    "Search engine"
                } else {
                    "Suchmaschine"
                }
            }
            BotClass::AiTraining => {
                if en {
                    "AI training"
                } else {
                    "KI-Training"
                }
            }
            BotClass::AiCitation => {
                if en {
                    "AI search / citation"
                } else {
                    "KI-Suche / Zitation"
                }
            }
            BotClass::AiMixed => {
                if en {
                    "AI training & search"
                } else {
                    "KI-Training & Suche"
                }
            }
            BotClass::UnknownAi => {
                if en {
                    "Unknown scraper"
                } else {
                    "Unbekannter Scraper"
                }
            }
            BotClass::General => {
                if en {
                    "General"
                } else {
                    "Allgemein"
                }
            }
            BotClass::Unknown => {
                if en {
                    "Unclassified"
                } else {
                    "Nicht klassifiziert"
                }
            }
        }
    }
}

impl std::fmt::Display for BotClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label(false))
    }
}

// ─── Bot Registry ────────────────────────────────────────────────────────────

/// Die Einordnung selbst kommt aus [`web_checks::robots`] — ein Register für
/// beide Werkzeuge, damit dieselbe Seite nicht zwei Antworten bekommt. Hier
/// bleibt nur die Beschriftung, die sprachabhängig ist.
impl From<web_checks::robots::BotClass> for BotClass {
    fn from(class: web_checks::robots::BotClass) -> Self {
        use web_checks::robots::BotClass as W;
        match class {
            W::Wildcard => BotClass::Wildcard,
            W::SearchEngine => BotClass::SearchEngine,
            W::AiTraining => BotClass::AiTraining,
            W::AiCitation => BotClass::AiCitation,
            W::AiMixed => BotClass::AiMixed,
            W::UnknownAi => BotClass::UnknownAi,
            W::General => BotClass::General,
            W::Unknown => BotClass::Unknown,
        }
    }
}

// ─── Fetch ───────────────────────────────────────────────────────────────────

/// robots.txt of the audited origin.
pub(crate) static ROBOTS_TXT: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "seo.robots_txt",
        module: "seo",
        method: crate::audit::collection::CollectionMethod::Http,
        method_version: 1,
        scoring: false,
    };

/// The first sitemap declared in robots.txt, read for the noindex check.
pub(crate) static SITEMAP_NOINDEX: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "seo.sitemap_noindex",
        module: "seo",
        method: crate::audit::collection::CollectionMethod::Http,
        method_version: 1,
        scoring: false,
    };

/// Fetch and parse the robots.txt for the given URL's domain.
/// If `canonical_url` is provided and `is_noindex` is true, also checks
/// whether the page appears in the first declared sitemap.
///
/// `None` when robots.txt could not be requested: an unanswered request is
/// no evidence of a missing robots.txt (#777). An error status is a
/// measured result (`fetched: false`).
pub async fn audit_robots_txt(
    url: &str,
    canonical_url: Option<&str>,
    is_noindex: bool,
    locale: &str,
    log: &crate::audit::collection::CollectionLog,
) -> Option<RobotsAudit> {
    use crate::audit::collection::{reason, Collected};
    let audit = fetch_robots_txt(url, locale).await;
    let mut audit = log.record(&ROBOTS_TXT, audit).into_value()?;

    // noindex-in-sitemap check: only run when page is noindex and we have a canonical URL
    if is_noindex && audit.fetched {
        if let Some(canon) = canonical_url {
            let checked = match audit.sitemaps.first() {
                None => Collected::NotApplicable("no_sitemap_declared"),
                Some(sitemap_url) => match Client::builder()
                    .timeout(Duration::from_secs(10))
                    .user_agent("auditmysite/1.0")
                    .build()
                {
                    Ok(client) => check_noindex_in_sitemap(sitemap_url, canon, &client).await,
                    Err(_) => Collected::Failed(reason::HTTP_REQUEST_FAILED),
                },
            };
            audit.noindex_in_sitemap = log.record(&SITEMAP_NOINDEX, checked).into_value();
        }
    }

    Some(audit)
}

async fn fetch_robots_txt(
    url: &str,
    locale: &str,
) -> crate::audit::collection::Collected<RobotsAudit> {
    use crate::audit::collection::{reason, Collected};
    let base = extract_base(url);
    let robots_url = format!("{}/robots.txt", base.trim_end_matches('/'));

    let client = match Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("auditmysite/1.0")
        .build()
    {
        Ok(c) => c,
        Err(_) => return Collected::Failed(reason::HTTP_REQUEST_FAILED),
    };

    let text = match client.get(&robots_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(t) => t,
            Err(_) => return Collected::Failed(reason::HTTP_REQUEST_FAILED),
        },
        // The server answered: there is no usable robots.txt. That is a
        // measurement, not a failure.
        Ok(resp) => {
            return Collected::Measured(RobotsAudit {
                error: Some(format!("HTTP {}", resp.status())),
                ..Default::default()
            })
        }
        Err(_) => return Collected::Failed(reason::HTTP_REQUEST_FAILED),
    };

    let mut audit = parse_robots_txt(&text, locale);
    audit.fetched = true;
    Collected::Measured(audit)
}

/// Fetch the declared sitemap and check whether `canonical_url` appears as a
/// `<loc>` entry. A sitemap that could not be read says nothing about the
/// page (#777).
async fn check_noindex_in_sitemap(
    sitemap_url: &str,
    canonical_url: &str,
    client: &Client,
) -> crate::audit::collection::Collected<bool> {
    use crate::audit::collection::{reason, Collected};
    let Ok(resp) = client
        .get(sitemap_url)
        .timeout(Duration::from_secs(8))
        .send()
        .await
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };
    if !resp.status().is_success() {
        return Collected::Failed("sitemap_not_readable");
    }
    let Ok(body) = resp.text().await else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    // Normalise URL for comparison: strip trailing slash
    let norm = |s: &str| s.trim_end_matches('/').to_lowercase();
    let target = norm(canonical_url);

    // Simple <loc>…</loc> scan — avoids an XML parser dependency
    let mut pos = 0;
    while let Some(start) = body[pos..].find("<loc>") {
        let abs = pos + start + 5;
        if let Some(end) = body[abs..].find("</loc>") {
            let loc = norm(&body[abs..abs + end]);
            if loc == target {
                return Collected::Measured(true);
            }
            pos = abs + end + 6;
        } else {
            break;
        }
    }
    Collected::Measured(false)
}

fn extract_base(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("https://") {
        let host = rest.split('/').next().unwrap_or(rest);
        return format!("https://{}", host);
    }
    if let Some(rest) = url.strip_prefix("http://") {
        let host = rest.split('/').next().unwrap_or(rest);
        return format!("http://{}", host);
    }
    url.to_string()
}

// ─── Parser ──────────────────────────────────────────────────────────────────

fn parse_robots_txt(text: &str, locale: &str) -> RobotsAudit {
    let en = locale == "en";

    // Grammatik und Bot-Einordnung kommen aus web-checks; astro-post-audit
    // benutzt dieselbe Auswertung an einem gebauten dist/.
    let parsed = web_checks::robots::parse(text);
    let sitemaps = parsed.sitemaps.clone();
    let crawl_delays = parsed.crawl_delays();
    let groups: Vec<RobotsGroup> = parsed
        .groups
        .into_iter()
        .map(|g| RobotsGroup {
            user_agent: g.user_agent,
            bot_class: g.bot_class.into(),
            allows: g.allows,
            disallows: g.disallows,
        })
        .collect();

    // Derived signals
    let has_wildcard_disallow_all = groups
        .iter()
        .any(|g| g.bot_class == BotClass::Wildcard && g.disallows.iter().any(|d| d == "/"));

    let is_fully_blocked = |g: &RobotsGroup| g.disallows.iter().any(|d| d == "/");

    let blocks_ai_training = groups
        .iter()
        .any(|g| g.bot_class == BotClass::AiTraining && is_fully_blocked(g));

    let blocks_ai_citation = groups
        .iter()
        .any(|g| g.bot_class == BotClass::AiCitation && is_fully_blocked(g));

    let blocks_ai_mixed = groups
        .iter()
        .any(|g| g.bot_class == BotClass::AiMixed && is_fully_blocked(g));

    let blocks_ai_crawlers = blocks_ai_training
        || blocks_ai_citation
        || blocks_ai_mixed
        || groups
            .iter()
            .any(|g| g.bot_class == BotClass::UnknownAi && is_fully_blocked(g));

    let mut audit = RobotsAudit {
        fetched: false, // set by caller after network step
        error: None,
        groups,
        sitemaps,
        crawl_delays,
        has_wildcard_disallow_all,
        blocks_ai_crawlers,
        blocks_ai_training,
        blocks_ai_citation,
        inferred_policy: String::new(),
        noindex_in_sitemap: None,
    };

    // Bake the policy label in the requested language. The PDF re-derives a
    // localized label from the same pure function at presentation time (#406).
    audit.inferred_policy = infer_robots_policy(&audit, en);
    audit
}

/// Re-derive the human-readable robots policy label in the requested language.
///
/// Pure function of the stored audit fields — used both at analysis time
/// (canonical English, for JSON) and by the PDF to localize without re-fetching
/// robots.txt (#406).
pub fn infer_robots_policy(audit: &RobotsAudit, en: bool) -> String {
    let is_fully_blocked = |g: &RobotsGroup| g.disallows.iter().any(|d| d == "/");

    let blocks_ai_mixed = audit
        .groups
        .iter()
        .any(|g| g.bot_class == BotClass::AiMixed && is_fully_blocked(g));

    let has_any_ai_rule = audit.groups.iter().any(|g| {
        matches!(
            g.bot_class,
            BotClass::AiTraining | BotClass::AiCitation | BotClass::AiMixed
        )
    });

    if audit.has_wildcard_disallow_all {
        if en {
            "Everything blocked (Disallow: *)".to_string()
        } else {
            "Alles gesperrt (Disallow: *)".to_string()
        }
    } else if audit.blocks_ai_citation {
        if en {
            "AI fully blocked".to_string()
        } else {
            "KI vollständig blockiert".to_string()
        }
    } else if blocks_ai_mixed && audit.blocks_ai_training {
        if en {
            "AI training blocked (conservative)".to_string()
        } else {
            "KI-Training blockiert (konservativ)".to_string()
        }
    } else if audit.blocks_ai_training {
        if en {
            "AI training blocked, AI search allowed".to_string()
        } else {
            "KI-Training blockiert, KI-Suche erlaubt".to_string()
        }
    } else if has_any_ai_rule {
        if en {
            "AI access open".to_string()
        } else {
            "KI-Zugang offen".to_string()
        }
    } else if en {
        "No explicit AI rule".to_string()
    } else {
        "Keine explizite KI-Regel".to_string()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

/// Counter-cases of [`ROBOTS_TXT`] and [`SITEMAP_NOINDEX`] (#777).
#[cfg(test)]
pub(crate) fn counter_cases() {
    use crate::audit::collection::{CollectionLog, CollectionState};
    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(f)
    }
    // Nothing answers on the discard port: no robots.txt verdict at all.
    let log = CollectionLog::new();
    let audit = block_on(audit_robots_txt(
        "http://127.0.0.1:9/page",
        Some("http://127.0.0.1:9/page"),
        true,
        "en",
        &log,
    ));
    assert!(
        audit.is_none(),
        "an unanswered request is no missing robots.txt"
    );
    let records = log.into_records();
    assert_eq!(records.len(), 1, "no sitemap check without robots.txt");
    assert_eq!(records[0].state, CollectionState::Failed);

    // An unreadable sitemap says nothing about the page.
    let client = Client::new();
    let checked = block_on(check_noindex_in_sitemap(
        "http://127.0.0.1:9/sitemap.xml",
        "http://127.0.0.1:9/page",
        &client,
    ));
    assert_eq!(checked.state(), CollectionState::Failed);
}

#[cfg(test)]
mod tests {
    /// Die Einordnung selbst liegt in web-checks und ist dort geprüft; hier
    /// steht nur, dass die Zuordnung auf diese Typen stimmt.
    fn bot_class(ua: &str) -> BotClass {
        web_checks::robots::classify_bot(ua).into()
    }

    use super::*;

    const SAMPLE: &str = r#"
User-agent: *
Disallow: /admin/
Allow: /

User-agent: GPTBot
Disallow: /

User-agent: Googlebot
Allow: /

Sitemap: https://example.com/sitemap.xml
Crawl-delay: 10
"#;

    #[test]
    fn test_parse_basic() {
        let audit = parse_robots_txt(SAMPLE, "de");
        assert!(!audit.has_wildcard_disallow_all);
        assert!(audit.blocks_ai_crawlers);
        assert!(audit
            .sitemaps
            .contains(&"https://example.com/sitemap.xml".to_string()));
    }

    #[test]
    fn test_wildcard_disallow_all() {
        let text = "User-agent: *\nDisallow: /\n";
        let audit = parse_robots_txt(text, "de");
        assert!(audit.has_wildcard_disallow_all);
    }

    #[test]
    fn test_bot_classification() {
        assert_eq!(bot_class("GPTBot"), BotClass::AiTraining);
        assert_eq!(bot_class("Googlebot"), BotClass::SearchEngine);
        assert_eq!(bot_class("CCBot"), BotClass::AiTraining);
        assert_eq!(bot_class("*"), BotClass::Wildcard);
        assert_eq!(bot_class("SomeRandomBot"), BotClass::Unknown);
        assert_eq!(bot_class("claudebot"), BotClass::AiMixed);
        assert_eq!(bot_class("PerplexityBot"), BotClass::AiCitation);
        assert_eq!(bot_class("semrushbot"), BotClass::UnknownAi);
    }

    #[test]
    fn test_extract_base() {
        assert_eq!(
            extract_base("https://example.com/path?q=1"),
            "https://example.com"
        );
        assert_eq!(
            extract_base("http://sub.example.com/"),
            "http://sub.example.com"
        );
    }
}
