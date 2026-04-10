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
    /// True when `User-agent: *` has `Disallow: /` — blocks everything
    pub has_wildcard_disallow_all: bool,
    /// True when any group blocks AI crawlers specifically
    pub blocks_ai_crawlers: bool,
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
    /// Verified AI trainer bot (GPTBot, ClaudeBot, Google-Extended, …)
    VerifiedAi,
    /// Unverified / scrapers often used for AI training (ByteSpider, CCBot, …)
    UnknownAi,
    /// General / known-good (archive.org, etc.)
    General,
    /// Not in any known list
    Unknown,
}

impl std::fmt::Display for BotClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotClass::Wildcard => write!(f, "Alle Crawler (*)"),
            BotClass::SearchEngine => write!(f, "Suchmaschine"),
            BotClass::VerifiedAi => write!(f, "KI-Crawler (verifiziert)"),
            BotClass::UnknownAi => write!(f, "KI-Crawler (unbekannt)"),
            BotClass::General => write!(f, "Allgemein"),
            BotClass::Unknown => write!(f, "Nicht klassifiziert"),
        }
    }
}

// ─── Bot Registry ────────────────────────────────────────────────────────────

const SEARCH_BOTS: &[&str] = &[
    "googlebot",
    "bingbot",
    "slurp", // Yahoo
    "duckduckbot",
    "baiduspider",
    "yandexbot",
    "sogou",
    "exabot",
    "ia_archiver", // Wayback Machine
    "msnbot",
    "teoma",
    "ask jeeves",
];

const VERIFIED_AI_BOTS: &[&str] = &[
    "gptbot",          // OpenAI
    "chatgpt-user",    // OpenAI browsing
    "google-extended", // Google AI training
    "claudebot",       // Anthropic
    "anthropic-ai",    // Anthropic
    "cohere-ai",
    "perplexitybot",
    "youbot", // You.com
    "amazonbot",
    "meta-externalagent", // Meta
    "facebookbot",
    "applebot-extended", // Apple AI
    "diffbot",
    "omgili",
];

const UNKNOWN_AI_BOTS: &[&str] = &[
    "bytespider", // TikTok / ByteDance
    "ccbot",      // Common Crawl (used for AI training)
    "dotbot",
    "petalbot", // Huawei
    "wpbot",
    "semrushbot",
    "ahrefsbot",
    "mj12bot",
    "rogerbot",
];

fn classify_bot(ua: &str) -> BotClass {
    if ua == "*" {
        return BotClass::Wildcard;
    }
    let lower = ua.to_lowercase();
    for pat in SEARCH_BOTS {
        if lower.contains(pat) {
            return BotClass::SearchEngine;
        }
    }
    for pat in VERIFIED_AI_BOTS {
        if lower.contains(pat) {
            return BotClass::VerifiedAi;
        }
    }
    for pat in UNKNOWN_AI_BOTS {
        if lower.contains(pat) {
            return BotClass::UnknownAi;
        }
    }
    BotClass::Unknown
}

// ─── Fetch ───────────────────────────────────────────────────────────────────

/// Fetch and parse the robots.txt for the given URL's domain.
/// Never returns an error — failures are recorded in `RobotsAudit.error`.
pub async fn audit_robots_txt(url: &str) -> RobotsAudit {
    let base = extract_base(url);
    let robots_url = format!("{}/robots.txt", base.trim_end_matches('/'));

    let client = match Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("auditmysite/1.0")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return RobotsAudit {
                error: Some(format!("HTTP-Client konnte nicht erstellt werden: {e}")),
                ..Default::default()
            }
        }
    };

    let text = match client.get(&robots_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                return RobotsAudit {
                    error: Some(format!("robots.txt konnte nicht gelesen werden: {e}")),
                    ..Default::default()
                }
            }
        },
        Ok(resp) => {
            return RobotsAudit {
                error: Some(format!("HTTP {}", resp.status())),
                ..Default::default()
            }
        }
        Err(e) => {
            return RobotsAudit {
                error: Some(format!("Netzwerkfehler: {e}")),
                ..Default::default()
            }
        }
    };

    let mut audit = parse_robots_txt(&text);
    audit.fetched = true;
    audit
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

fn parse_robots_txt(text: &str) -> RobotsAudit {
    let mut groups: Vec<RobotsGroup> = Vec::new();
    let mut sitemaps: Vec<String> = Vec::new();
    let mut crawl_delays: Vec<(String, u32)> = Vec::new();

    // Accumulate current group
    let mut cur_agents: Vec<String> = Vec::new();
    let mut cur_allows: Vec<String> = Vec::new();
    let mut cur_disallows: Vec<String> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            // Empty line separates groups — flush if we have rules
            if line.is_empty()
                && !cur_agents.is_empty()
                && (!cur_allows.is_empty() || !cur_disallows.is_empty())
            {
                flush_group(
                    &mut cur_agents,
                    &mut cur_allows,
                    &mut cur_disallows,
                    &mut groups,
                );
            }
            continue;
        }

        let Some(colon_pos) = line.find(':') else {
            continue;
        };
        let key = line[..colon_pos].trim().to_lowercase();
        let value = line[colon_pos + 1..].trim().to_string();

        match key.as_str() {
            "user-agent" => {
                // New user-agent: if we have pending rules flush, else just accumulate agents
                if !cur_allows.is_empty() || !cur_disallows.is_empty() {
                    flush_group(
                        &mut cur_agents,
                        &mut cur_allows,
                        &mut cur_disallows,
                        &mut groups,
                    );
                }
                cur_agents.push(value);
            }
            "allow" if !value.is_empty() => cur_allows.push(value),
            "disallow" => cur_disallows.push(value),
            "sitemap" if !value.is_empty() => sitemaps.push(value),
            "crawl-delay" => {
                if let Ok(delay) = value.parse::<u32>() {
                    for agent in &cur_agents {
                        crawl_delays.push((agent.clone(), delay));
                    }
                }
            }
            _ => {}
        }
    }

    // Flush remaining group
    if !cur_agents.is_empty() {
        flush_group(
            &mut cur_agents,
            &mut cur_allows,
            &mut cur_disallows,
            &mut groups,
        );
    }

    // Derived signals
    let has_wildcard_disallow_all = groups
        .iter()
        .any(|g| g.bot_class == BotClass::Wildcard && g.disallows.iter().any(|d| d == "/"));

    let blocks_ai_crawlers = groups.iter().any(|g| {
        matches!(g.bot_class, BotClass::VerifiedAi | BotClass::UnknownAi)
            && g.disallows.iter().any(|d| d == "/")
    });

    RobotsAudit {
        fetched: false, // set by caller after network step
        error: None,
        groups,
        sitemaps,
        crawl_delays,
        has_wildcard_disallow_all,
        blocks_ai_crawlers,
    }
}

fn flush_group(
    agents: &mut Vec<String>,
    allows: &mut Vec<String>,
    disallows: &mut Vec<String>,
    groups: &mut Vec<RobotsGroup>,
) {
    for agent in agents.drain(..) {
        let class = classify_bot(&agent);
        groups.push(RobotsGroup {
            user_agent: agent,
            bot_class: class,
            allows: allows.clone(),
            disallows: disallows.clone(),
        });
    }
    allows.clear();
    disallows.clear();
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
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
        let audit = parse_robots_txt(SAMPLE);
        assert!(!audit.has_wildcard_disallow_all);
        assert!(audit.blocks_ai_crawlers);
        assert!(audit
            .sitemaps
            .contains(&"https://example.com/sitemap.xml".to_string()));
    }

    #[test]
    fn test_wildcard_disallow_all() {
        let text = "User-agent: *\nDisallow: /\n";
        let audit = parse_robots_txt(text);
        assert!(audit.has_wildcard_disallow_all);
    }

    #[test]
    fn test_bot_classification() {
        assert_eq!(classify_bot("GPTBot"), BotClass::VerifiedAi);
        assert_eq!(classify_bot("Googlebot"), BotClass::SearchEngine);
        assert_eq!(classify_bot("CCBot"), BotClass::UnknownAi);
        assert_eq!(classify_bot("*"), BotClass::Wildcard);
        assert_eq!(classify_bot("SomeRandomBot"), BotClass::Unknown);
        assert_eq!(classify_bot("claudebot"), BotClass::VerifiedAi);
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
