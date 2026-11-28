//! robots.txt audit — informational, no score impact.
//!
//! Fetches and parses the robots.txt of the audited domain. Classifies known
//! bots by category (search engines, verified AI crawlers, unverified AI
//! crawlers, generic crawlers) and surfaces key findings in the report.

use crate::audit::collection::{reason, Collected, CollectionLog, CollectionMethod, CollectorSpec};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Status of `/llms.txt` on the audited origin (#1270).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmsTxtStatus {
    /// `/llms.txt` returned HTTP 2xx with plausible Markdown/text content.
    PresentValid,
    /// `/llms.txt` returned HTTP 2xx but looks empty or like an HTML fallback page.
    PresentInvalid,
    /// `/llms.txt` returned 404/410 or another non-2xx status.
    Missing,
}

/// robots.txt audit result (#1270).
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
    /// `Content-Signal:` directives (IETF AI Preferences / Cloudflare)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_signals: Vec<String>,
    /// (user-agent, delay-seconds) pairs
    pub crawl_delays: Vec<(String, u32)>,
    /// True when the audited page has noindex AND appears in the sitemap.xml
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noindex_in_sitemap: Option<bool>,
    /// True when `User-agent: *` has `Disallow: /` — blocks everything
    pub has_wildcard_disallow_all: bool,
    /// True when any AI crawler (any sub-type) is blocked — legacy alias
    pub blocks_ai_crawlers: bool,
    /// True when training-only bots (GPTBot, ClaudeBot, CCBot, …) are blocked
    pub blocks_ai_training: bool,
    /// True when citation/search or mixed AI bots (PerplexityBot, OAI-SearchBot, meta-externalagent, …) are blocked
    pub blocks_ai_citation: bool,
    /// True when user-triggered AI fetch bots (ChatGPT-User, Claude-User, …) are blocked
    #[serde(default)]
    pub blocks_user_fetch: bool,
    /// True when web-archive crawlers (ia_archiver, archive.org_bot) are blocked
    #[serde(default)]
    pub blocks_archive: bool,
    /// True when search-engine crawlers (Googlebot, Bingbot, …) are fully or partially blocked
    #[serde(default)]
    pub blocks_search_engines: bool,
    /// True when the audited page path itself is disallowed for Googlebot or Bingbot
    #[serde(default)]
    pub audited_path_blocked_for_search: bool,
    /// Result of checking `/llms.txt` on the audited origin
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llms_txt: Option<LlmsTxtStatus>,
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
    /// AI training bot — blocking is standard practice (GPTBot, ClaudeBot, CCBot, …)
    AiTraining,
    /// AI citation / search bot (PerplexityBot, OAI-SearchBot, Claude-SearchBot, …)
    AiCitation,
    /// User-triggered live fetch by AI assistants (ChatGPT-User, Claude-User, Perplexity-User, …)
    UserFetch,
    /// AI bot with mixed purpose: both training and citation (Google-Extended, meta-externalagent, …)
    AiMixed,
    /// Web archive crawler (ia_archiver, archive.org_bot)
    Archive,
    /// Unverified scrapers, SEO bots (dotbot, semrushbot, …)
    UnknownAi,
    /// General / known-good (social preview, etc.)
    General,
    /// Not in any known list
    Unknown,
}

impl BotClass {
    /// Human-readable, report-visible label for this bot class: the
    /// `seo-robots-bot-class-*` message of `locales/{en,de}/seo-issues.ftl`.
    pub fn label(&self, en: bool) -> String {
        let key = match self {
            BotClass::Wildcard => "seo-robots-bot-class-wildcard",
            BotClass::SearchEngine => "seo-robots-bot-class-search-engine",
            BotClass::AiTraining => "seo-robots-bot-class-ai-training",
            BotClass::AiCitation => "seo-robots-bot-class-ai-citation",
            BotClass::UserFetch => "seo-robots-bot-class-user-fetch",
            BotClass::AiMixed => "seo-robots-bot-class-ai-mixed",
            BotClass::Archive => "seo-robots-bot-class-archive",
            BotClass::UnknownAi => "seo-robots-bot-class-unknown-ai",
            BotClass::General => "seo-robots-bot-class-general",
            BotClass::Unknown => "seo-robots-bot-class-unknown",
        };
        crate::i18n::text(key, en)
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
            W::UserFetch => BotClass::UserFetch,
            W::AiMixed => BotClass::AiMixed,
            W::Archive => BotClass::Archive,
            W::UnknownAi => BotClass::UnknownAi,
            W::General => BotClass::General,
            W::Unknown => BotClass::Unknown,
        }
    }
}

// ─── Fetch ───────────────────────────────────────────────────────────────────

/// robots.txt of the audited origin.
pub(crate) static ROBOTS_TXT: CollectorSpec = CollectorSpec {
    id: "seo.robots_txt",
    module: "seo",
    method: CollectionMethod::Http,
    method_version: 1,
    scoring: false,
};

/// The first sitemap declared in robots.txt, read for the noindex check.
pub(crate) static SITEMAP_NOINDEX: CollectorSpec = CollectorSpec {
    id: "seo.sitemap_noindex",
    module: "seo",
    method: CollectionMethod::Http,
    method_version: 1,
    scoring: false,
};

/// `/llms.txt` of the audited origin (#1270).
pub(crate) static LLMS_TXT: CollectorSpec = CollectorSpec {
    id: "seo.llms_txt",
    module: "seo",
    method: CollectionMethod::Http,
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
    log: &CollectionLog,
) -> Option<RobotsAudit> {
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

    // /llms.txt presence & basic validity check (#1270)
    let llms = fetch_llms_txt(url).await;
    audit.llms_txt = log.record(&LLMS_TXT, llms).into_value();

    Some(audit)
}

async fn fetch_llms_txt(url: &str) -> Collected<LlmsTxtStatus> {
    let base = extract_base(url);
    let llms_url = format!("{}/llms.txt", base.trim_end_matches('/'));
    let client = match Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent("auditmysite/1.0")
        .build()
    {
        Ok(c) => c,
        Err(_) => return Collected::Failed(reason::HTTP_REQUEST_FAILED),
    };

    match client.get(&llms_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let is_html = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|ct| ct.to_ascii_lowercase().contains("text/html"));
            let body = resp.text().await.unwrap_or_default();
            let trimmed = body.trim();
            if is_html
                || trimmed.is_empty()
                || trimmed.starts_with("<!DOCTYPE")
                || trimmed.starts_with("<!doctype")
                || trimmed.starts_with("<html")
            {
                Collected::Measured(LlmsTxtStatus::PresentInvalid)
            } else if trimmed.starts_with('#')
                || trimmed.contains("http://")
                || trimmed.contains("https://")
            {
                Collected::Measured(LlmsTxtStatus::PresentValid)
            } else {
                Collected::Measured(LlmsTxtStatus::PresentInvalid)
            }
        }
        Ok(_) => Collected::Measured(LlmsTxtStatus::Missing),
        Err(_) => Collected::Failed(reason::HTTP_REQUEST_FAILED),
    }
}

async fn fetch_robots_txt(url: &str, locale: &str) -> Collected<RobotsAudit> {
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

    let mut audit = parse_robots_txt_for_url(&text, Some(url), locale);
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
) -> Collected<bool> {
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

#[cfg(test)]
fn parse_robots_txt(text: &str, locale: &str) -> RobotsAudit {
    parse_robots_txt_for_url(text, None, locale)
}

pub fn parse_robots_txt_for_url(text: &str, url: Option<&str>, locale: &str) -> RobotsAudit {
    use web_checks::robots::BlockScope;
    let en = locale == "en";

    // Grammatik und Bot-Einordnung kommen aus web-checks; astro-post-audit
    // benutzt dieselbe Auswertung an einem gebauten dist/.
    let parsed = web_checks::robots::parse(text);
    let sitemaps = parsed.sitemaps.clone();
    let content_signals = parsed.content_signals.clone();
    let crawl_delays = parsed.crawl_delays();

    let is_blocked_any = |g: &web_checks::robots::Group| {
        matches!(g.block_scope(), BlockScope::Full | BlockScope::Partial(_))
    };

    let has_wildcard_disallow_all = parsed.groups.iter().any(|g| {
        g.bot_class == web_checks::robots::BotClass::Wildcard && g.block_scope() == BlockScope::Full
    });

    let blocks_ai_training = parsed
        .groups
        .iter()
        .any(|g| g.bot_class == web_checks::robots::BotClass::AiTraining && is_blocked_any(g));

    let blocks_ai_mixed = parsed
        .groups
        .iter()
        .any(|g| g.bot_class == web_checks::robots::BotClass::AiMixed && is_blocked_any(g));

    let blocks_ai_citation = blocks_ai_mixed
        || parsed
            .groups
            .iter()
            .any(|g| g.bot_class == web_checks::robots::BotClass::AiCitation && is_blocked_any(g));

    let blocks_user_fetch = parsed
        .groups
        .iter()
        .any(|g| g.bot_class == web_checks::robots::BotClass::UserFetch && is_blocked_any(g));

    let blocks_archive = parsed
        .groups
        .iter()
        .any(|g| g.bot_class == web_checks::robots::BotClass::Archive && is_blocked_any(g));

    let blocks_search_engines = parsed
        .groups
        .iter()
        .any(|g| g.bot_class == web_checks::robots::BotClass::SearchEngine && is_blocked_any(g));

    let audited_path_blocked_for_search = url
        .and_then(|u| url::Url::parse(u).ok())
        .map(|u| {
            let path = u.path();
            ["Googlebot", "Bingbot"].iter().any(|bot| {
                parsed
                    .effective_group(bot)
                    .is_some_and(|g| g.disallows_path(path))
            })
        })
        .unwrap_or(false);

    let blocks_ai_crawlers = blocks_ai_training
        || blocks_ai_citation
        || blocks_user_fetch
        || parsed
            .groups
            .iter()
            .any(|g| g.bot_class == web_checks::robots::BotClass::UnknownAi && is_blocked_any(g));

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

    let mut audit = RobotsAudit {
        fetched: false, // set by caller after network step
        error: None,
        groups,
        sitemaps,
        content_signals,
        crawl_delays,
        has_wildcard_disallow_all,
        blocks_ai_crawlers,
        blocks_ai_training,
        blocks_ai_citation,
        blocks_user_fetch,
        blocks_archive,
        blocks_search_engines,
        audited_path_blocked_for_search,
        llms_txt: None,
        inferred_policy: String::new(),
        noindex_in_sitemap: None,
    };

    // Bake the policy label in the requested language. The PDF re-derives a
    // localized label from the same pure function at presentation time (#406).
    audit.inferred_policy = infer_robots_policy(&audit, en);
    audit
}

/// Classification of a robots.txt / llms.txt finding (#1269, #1270).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotsFindingKind {
    Error,
    Policy,
    Hint,
}

/// A finding derived from [`RobotsAudit`] for the unified finding contract (#1270).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotsFinding {
    pub check: &'static str,
    pub kind: RobotsFindingKind,
    pub severity: crate::taxonomy::Severity,
    pub message: String,
}

/// Derive contract findings (errors, operator policies, hints) from [`RobotsAudit`].
pub fn collect_robots_findings(robots: &RobotsAudit) -> Vec<RobotsFinding> {
    use crate::taxonomy::Severity;
    let mut out = Vec::new();
    if !robots.fetched {
        return out;
    }

    if robots.has_wildcard_disallow_all {
        out.push(RobotsFinding {
            check: "robots_disallow_all",
            kind: RobotsFindingKind::Error,
            severity: Severity::Critical,
            message: "robots.txt blocks all crawlers (User-agent: * / Disallow: /) — on production domains this prevents search indexing entirely.".to_string(),
        });
        return out;
    }

    if robots.audited_path_blocked_for_search {
        out.push(RobotsFinding {
            check: "robots_audited_page_blocked",
            kind: RobotsFindingKind::Error,
            severity: Severity::High,
            message: "The audited page path is disallowed for search-engine crawlers (Googlebot/Bingbot) in robots.txt.".to_string(),
        });
    }

    if robots.blocks_search_engines {
        let bots: Vec<&str> = robots
            .groups
            .iter()
            .filter(|g| g.bot_class == BotClass::SearchEngine && !g.disallows.is_empty())
            .map(|g| g.user_agent.as_str())
            .collect();
        out.push(RobotsFinding {
            check: "robots_search_engine_blocked",
            kind: RobotsFindingKind::Error,
            severity: Severity::High,
            message: format!(
                "Search-engine crawlers ({}) are restricted in robots.txt — affected paths cannot be crawled or indexed.",
                bots.join(", ")
            ),
        });
    }

    let collect_bots = |classes: &[BotClass]| -> Vec<&str> {
        robots
            .groups
            .iter()
            .filter(|g| classes.contains(&g.bot_class) && !g.disallows.is_empty())
            .map(|g| g.user_agent.as_str())
            .collect()
    };

    if robots.blocks_ai_citation {
        let bots = collect_bots(&[BotClass::AiCitation, BotClass::AiMixed]);
        out.push(RobotsFinding {
            check: "robots_policy_ai_citation_blocked",
            kind: RobotsFindingKind::Policy,
            severity: Severity::Low,
            message: format!(
                "Operator policy: AI search/mixed crawlers ({}) are restricted in robots.txt — content will not appear in cited AI answers for those assistants.",
                bots.join(", ")
            ),
        });
    }

    if robots.blocks_user_fetch {
        let bots = collect_bots(&[BotClass::UserFetch]);
        out.push(RobotsFinding {
            check: "robots_policy_user_fetch_blocked",
            kind: RobotsFindingKind::Policy,
            severity: Severity::Low,
            message: format!(
                "Operator policy: user-triggered AI fetch bots ({}) are restricted in robots.txt — assistants cannot fetch live pages on user request.",
                bots.join(", ")
            ),
        });
    }

    if robots.blocks_ai_training {
        let bots = collect_bots(&[BotClass::AiTraining]);
        out.push(RobotsFinding {
            check: "robots_policy_ai_training_blocked",
            kind: RobotsFindingKind::Policy,
            severity: Severity::Low,
            message: format!(
                "Operator policy: AI training crawlers ({}) are restricted in robots.txt — pages are excluded from training runs of those crawlers.",
                bots.join(", ")
            ),
        });
    }

    if robots.blocks_archive {
        let bots = collect_bots(&[BotClass::Archive]);
        out.push(RobotsFinding {
            check: "robots_policy_archive_blocked",
            kind: RobotsFindingKind::Policy,
            severity: Severity::Low,
            message: format!(
                "Operator policy: web archive crawlers ({}) are restricted in robots.txt — pages are not archived by the Internet Archive.",
                bots.join(", ")
            ),
        });
    }

    match robots.llms_txt {
        Some(LlmsTxtStatus::PresentInvalid) => {
            out.push(RobotsFinding {
                check: "llms_txt_invalid",
                kind: RobotsFindingKind::Hint,
                severity: Severity::Low,
                message: "/llms.txt is reachable but does not contain valid Markdown/plain-text guidance (returned empty or HTML fallback).".to_string(),
            });
        }
        Some(LlmsTxtStatus::Missing) if !robots.blocks_ai_citation => {
            out.push(RobotsFinding {
                check: "llms_txt_missing",
                kind: RobotsFindingKind::Hint,
                severity: Severity::Low,
                message: "No /llms.txt found — optional index file to guide AI assistants to key documentation and pages.".to_string(),
            });
        }
        _ => {}
    }

    out
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

    let index = if audit.has_wildcard_disallow_all {
        0
    } else if audit.blocks_ai_citation {
        1
    } else if blocks_ai_mixed && audit.blocks_ai_training {
        2
    } else if audit.blocks_ai_training {
        3
    } else if has_any_ai_rule {
        4
    } else {
        5
    };
    crate::i18n::text(POLICY_LABELS[index], en)
}

/// The messages of the policy labels (`locales/{en,de}/seo-issues.ftl`), in
/// the order [`infer_robots_policy`] tests them.
const POLICY_LABELS: [&str; 6] = [
    "seo-robots-policy-everything-blocked",
    "seo-robots-policy-ai-fully-blocked",
    "seo-robots-policy-ai-training-blocked-conservative",
    "seo-robots-policy-ai-training-blocked-search-allowed",
    "seo-robots-policy-ai-access-open",
    "seo-robots-policy-no-explicit-ai-rule",
];

/// The policy label stored in canonical English (AI visibility copies it from
/// the robots audit) in the requested language; any other text is returned
/// unchanged.
pub fn localize_policy_label(canonical: &str, en: bool) -> String {
    match POLICY_LABELS
        .iter()
        .find(|key| crate::i18n::text(key, true) == canonical)
    {
        Some(key) if !en => crate::i18n::text(key, false),
        _ => canonical.to_string(),
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
        assert_eq!(bot_class("claudebot"), BotClass::AiTraining);
        assert_eq!(bot_class("ChatGPT-User"), BotClass::UserFetch);
        assert_eq!(bot_class("Google-Extended"), BotClass::AiMixed);
        assert_eq!(bot_class("ia_archiver"), BotClass::Archive);
        assert_eq!(bot_class("archive.org_bot"), BotClass::Archive);
        assert_eq!(bot_class("Applebot"), BotClass::SearchEngine);
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

#[cfg(test)]
mod policy_label_tests {
    use super::*;

    #[test]
    fn every_canonical_label_has_a_german_form_without_english() {
        for key in POLICY_LABELS {
            let english = crate::i18n::text(key, true);
            let german = crate::i18n::text(key, false);
            assert_eq!(localize_policy_label(&english, true), english);
            assert_eq!(localize_policy_label(&english, false), german);
            assert_ne!(english, german);
        }
        assert_eq!(
            localize_policy_label("something else", false),
            "something else"
        );
    }

    #[test]
    fn casoon_fixture_reports_policies_without_errors() {
        let txt = "\
User-agent: GPTBot
Disallow: /

User-agent: ClaudeBot
Disallow: /

User-agent: meta-externalagent
Disallow: /

User-agent: ia_archiver
Disallow: /

User-agent: archive.org_bot
Disallow: /
";
        let mut audit = parse_robots_txt_for_url(txt, Some("https://casoon.de/"), "en");
        audit.fetched = true;
        assert!(audit.blocks_ai_training);
        assert!(audit.blocks_ai_citation);
        assert!(audit.blocks_archive);
        assert!(!audit.blocks_search_engines);
        assert!(!audit.audited_path_blocked_for_search);

        let findings = collect_robots_findings(&audit);
        assert!(
            findings.iter().all(|f| f.kind == RobotsFindingKind::Policy),
            "expected only Policy findings, got: {findings:?}"
        );
        assert!(findings
            .iter()
            .any(|f| f.check == "robots_policy_ai_training_blocked"));
        assert!(findings
            .iter()
            .any(|f| f.check == "robots_policy_ai_citation_blocked"));
        assert!(findings
            .iter()
            .any(|f| f.check == "robots_policy_archive_blocked"));
    }

    #[test]
    fn disallow_all_and_partial_search_engine_blocks_produce_errors() {
        use crate::taxonomy::Severity;

        let mut full = parse_robots_txt_for_url(
            "User-agent: *\nDisallow: /\n",
            Some("https://example.com/"),
            "en",
        );
        full.fetched = true;
        let full_findings = collect_robots_findings(&full);
        assert!(full_findings
            .iter()
            .any(|f| f.check == "robots_disallow_all"
                && f.kind == RobotsFindingKind::Error
                && f.severity == Severity::Critical));

        let mut partial = parse_robots_txt_for_url(
            "User-agent: Googlebot\nDisallow: /shop/\n",
            Some("https://example.com/shop/item-1"),
            "en",
        );
        partial.fetched = true;
        assert!(partial.blocks_search_engines);
        assert!(partial.audited_path_blocked_for_search);
        let partial_findings = collect_robots_findings(&partial);
        assert!(partial_findings
            .iter()
            .any(|f| f.check == "robots_audited_page_blocked"
                && f.kind == RobotsFindingKind::Error
                && f.severity == Severity::High));
        assert!(partial_findings
            .iter()
            .any(|f| f.check == "robots_search_engine_blocked"
                && f.kind == RobotsFindingKind::Error
                && f.severity == Severity::High));
    }

    #[test]
    fn robots_policy_report_texts_never_claim_default_or_recommended() {
        let forbidden = [
            "standardkonfiguration",
            "(standard)",
            "empfohlen",
            "kein problem",
            "recommended default",
            "(default)",
            "not a problem",
        ];
        for key in [
            "pdf-robots-limit-ai-body",
            "pdf-robots-limit-ai-title",
            "pdf-robots-training-blocked-body",
            "pdf-robots-training-blocked-title",
        ] {
            for en in [false, true] {
                let msg = crate::i18n::text(key, en).to_ascii_lowercase();
                for word in forbidden {
                    assert!(
                        !msg.contains(word),
                        "robots policy text {key} (en={en}) must not contain '{word}': {msg}"
                    );
                }
            }
        }
    }
}
