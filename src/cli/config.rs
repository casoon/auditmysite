//! Configuration file support
//!
//! Loads `auditmysite.toml` from the current directory or parent directories.

use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

use super::args::{Args, OutputFormat, WcagLevel};

const CONFIG_FILENAME: &str = "auditmysite.toml";

/// Configuration file structure
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub audit: AuditConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub modules: ModulesConfig,
    #[serde(default)]
    pub rules: RulesConfig,
    #[serde(default)]
    pub thresholds: ThresholdsConfig,
    #[serde(default)]
    pub budgets: BudgetConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct AuditConfig {
    /// WCAG level: "a", "aa", or "aaa"
    pub level: Option<String>,
    /// Page load timeout in seconds
    pub timeout: Option<u64>,
    /// Number of concurrent browser tabs
    pub concurrency: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OutputConfig {
    /// Default output format: "table", "json", or "pdf"
    pub format: Option<String>,
    /// Default output path
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ModulesConfig {
    /// Enable performance analysis
    pub performance: Option<bool>,
    /// Enable SEO analysis
    pub seo: Option<bool>,
    /// Enable security analysis
    pub security: Option<bool>,
    /// Enable mobile analysis
    pub mobile: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RulesConfig {
    /// List of rule IDs to ignore
    pub ignore: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ThresholdsConfig {
    /// Minimum score to pass (exit code 0)
    pub min_score: Option<f64>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct BudgetConfig {
    /// Maximum LCP in milliseconds (good: ≤ 2500)
    pub max_lcp_ms: Option<f64>,
    /// Maximum FCP in milliseconds (good: ≤ 1800)
    pub max_fcp_ms: Option<f64>,
    /// Maximum CLS (good: ≤ 0.1)
    pub max_cls: Option<f64>,
    /// Maximum TBT in milliseconds (good: ≤ 200)
    pub max_tbt_ms: Option<f64>,
    /// Maximum total JavaScript size in KB
    pub max_js_kb: Option<f64>,
    /// Maximum total CSS size in KB
    pub max_css_kb: Option<f64>,
    /// Maximum total page size in KB
    pub max_total_kb: Option<f64>,
    /// Maximum number of render-blocking scripts
    pub max_blocking_scripts: Option<u32>,
    /// Maximum third-party transfer size in KB
    pub max_third_party_kb: Option<f64>,
    /// Maximum number of HTTP requests
    pub max_request_count: Option<u32>,
}

impl BudgetConfig {
    pub fn is_empty(&self) -> bool {
        self.max_lcp_ms.is_none()
            && self.max_fcp_ms.is_none()
            && self.max_cls.is_none()
            && self.max_tbt_ms.is_none()
            && self.max_js_kb.is_none()
            && self.max_css_kb.is_none()
            && self.max_total_kb.is_none()
            && self.max_blocking_scripts.is_none()
            && self.max_third_party_kb.is_none()
            && self.max_request_count.is_none()
    }
}

impl Config {
    /// Load config from `auditmysite.toml`, searching up from the current directory.
    /// Returns `None` if no config file is found.
    pub fn load() -> Option<Self> {
        let path = find_config_file()?;
        info!("Loading config from: {}", path.display());

        let content = std::fs::read_to_string(&path).ok()?;
        match toml::from_str::<Config>(&content) {
            Ok(config) => Some(config),
            Err(e) => {
                tracing::warn!("Failed to parse {}: {}", path.display(), e);
                None
            }
        }
    }

    /// Apply config file defaults to CLI args.
    /// CLI args always take precedence over config file values.
    pub fn apply_to_args(&self, args: &mut Args) {
        // Audit settings (only if CLI didn't override)
        if let Some(ref level_str) = self.audit.level {
            // Only apply if user didn't specify --level explicitly
            // clap default is "aa", so we check if it's still the default
            match level_str.to_lowercase().as_str() {
                "a" => args.level = WcagLevel::A,
                "aa" => args.level = WcagLevel::AA,
                "aaa" => args.level = WcagLevel::AAA,
                _ => tracing::warn!("Invalid level in config: {}", level_str),
            }
        }

        if let Some(timeout) = self.audit.timeout {
            if args.timeout == 30 {
                // default
                args.timeout = timeout;
            }
        }

        if let Some(concurrency) = self.audit.concurrency {
            if args.concurrency == 3 {
                // default
                args.concurrency = concurrency;
            }
        }

        // Output settings
        if let Some(ref fmt) = self.output.format {
            if args.format.is_none() {
                match fmt.to_lowercase().as_str() {
                    "json" => args.format = Some(OutputFormat::Json),
                    "pdf" => args.format = Some(OutputFormat::Pdf),
                    "table" => args.format = Some(OutputFormat::Table),
                    _ => tracing::warn!("Invalid format in config: {}", fmt),
                }
            }
        }

        if let Some(ref path) = self.output.path {
            if args.output.is_none() {
                args.output = Some(PathBuf::from(path));
            }
        }

        // Module settings
        if !args.full {
            if let Some(true) = self.modules.performance {
                args.performance = true;
            }
            if let Some(true) = self.modules.seo {
                args.seo = true;
            }
            if let Some(true) = self.modules.security {
                args.security = true;
            }
            if let Some(true) = self.modules.mobile {
                args.mobile = true;
            }
        }
    }
}

/// Search for config file starting from current dir, walking up to root.
fn find_config_file() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Get the minimum score threshold from config, if set.
pub fn get_min_score_threshold(config: &Option<Config>) -> Option<f64> {
    config.as_ref().and_then(|c| c.thresholds.min_score)
}

/// Get the list of ignored rules from config, if set.
pub fn get_ignored_rules(config: &Option<Config>) -> Vec<String> {
    config
        .as_ref()
        .and_then(|c| c.rules.ignore.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[audit]
level = "aaa"
timeout = 60
concurrency = 5

[output]
format = "pdf"
path = "reports/"

[modules]
performance = true
seo = true
security = true
mobile = true

[rules]
ignore = ["1.4.3"]

[thresholds]
min_score = 70

# [budgets]
# max_lcp_ms = 2500
# max_fcp_ms = 1800
# max_cls = 0.1
# max_tbt_ms = 200
# max_js_kb = 400
# max_css_kb = 100
# max_total_kb = 1500
# max_blocking_scripts = 0
# max_third_party_kb = 200
# max_request_count = 80
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.audit.level.as_deref(), Some("aaa"));
        assert_eq!(config.audit.timeout, Some(60));
        assert_eq!(config.audit.concurrency, Some(5));
        assert_eq!(config.output.format.as_deref(), Some("pdf"));
        assert_eq!(config.output.path.as_deref(), Some("reports/"));
        assert!(config.modules.performance.unwrap());
        assert_eq!(config.rules.ignore.as_ref().unwrap().len(), 1);
        assert_eq!(config.thresholds.min_score, Some(70.0));
    }

    #[test]
    fn test_empty_config() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.audit.level.is_none());
        assert!(config.modules.performance.is_none());
    }

    #[test]
    fn test_partial_config() {
        let toml_str = r#"
[modules]
seo = true
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.modules.seo.unwrap());
        assert!(config.modules.performance.is_none());
    }
}
