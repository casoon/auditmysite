//! Vulnerable JavaScript library detection (#124).
//!
//! Scans script elements and known window globals to detect JS libraries
//! and compares their versions against a hardcoded list of known-vulnerable
//! version ranges.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

/// A detected JavaScript library with its version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedLibrary {
    /// Library name (e.g. "jQuery")
    pub name: String,
    /// Detected version string
    pub version: String,
    /// How the version was detected: "global" or "url"
    pub detection_source: String,
}

/// A detected library that has known security vulnerabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerableLibrary {
    /// Library name
    pub name: String,
    /// Detected version
    pub version: String,
    /// Vulnerability severity: "high", "medium", or "low"
    pub severity: String,
    /// Short description of the vulnerability
    pub description: String,
    /// Recommended minimum safe version
    pub safe_version: String,
}

/// Vulnerable library analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerableLibrariesAnalysis {
    /// All detected libraries (name + version)
    pub detected: Vec<DetectedLibrary>,
    /// Subset with known vulnerabilities
    pub vulnerable: Vec<VulnerableLibrary>,
    /// True if any vulnerable libraries were found
    pub has_vulnerabilities: bool,
}

/// Detect JS libraries and check for known vulnerabilities.
pub async fn analyze_vulnerable_libraries(page: &Page) -> Result<VulnerableLibrariesAnalysis> {
    info!("Analyzing vulnerable JavaScript libraries...");

    // Read versions from well-known window globals
    let globals_js = r#"
    (() => {
        var out = [];
        try {
            if (typeof jQuery !== 'undefined' && jQuery.fn && jQuery.fn.jquery)
                out.push({name: 'jQuery', version: jQuery.fn.jquery, source: 'global'});
        } catch(e) {}
        try {
            if (typeof $ !== 'undefined' && $ && $.fn && $.fn.jquery)
                out.push({name: 'jQuery', version: $.fn.jquery, source: 'global'});
        } catch(e) {}
        try {
            if (typeof angular !== 'undefined' && angular.version && angular.version.full)
                out.push({name: 'AngularJS', version: angular.version.full, source: 'global'});
        } catch(e) {}
        try {
            if (typeof Handlebars !== 'undefined' && Handlebars.VERSION)
                out.push({name: 'Handlebars', version: Handlebars.VERSION, source: 'global'});
        } catch(e) {}
        try {
            if (typeof _ !== 'undefined' && _.VERSION && typeof _.chunk === 'function')
                out.push({name: 'Lodash', version: _.VERSION, source: 'global'});
        } catch(e) {}
        try {
            if (typeof _ !== 'undefined' && _.VERSION && typeof _.chunk === 'undefined')
                out.push({name: 'Underscore', version: _.VERSION, source: 'global'});
        } catch(e) {}
        try {
            if (typeof Prototype !== 'undefined' && Prototype.Version)
                out.push({name: 'Prototype', version: Prototype.Version, source: 'global'});
        } catch(e) {}
        try {
            if (typeof MooTools !== 'undefined' && MooTools.version)
                out.push({name: 'MooTools', version: MooTools.version, source: 'global'});
        } catch(e) {}
        try {
            if (typeof moment !== 'undefined' && moment.version)
                out.push({name: 'Moment.js', version: moment.version, source: 'global'});
        } catch(e) {}
        // Bootstrap: window.bootstrap (v5+) or data-bs-version attribute
        try {
            if (typeof bootstrap !== 'undefined' && bootstrap.Tooltip && bootstrap.Tooltip.VERSION)
                out.push({name: 'Bootstrap', version: bootstrap.Tooltip.VERSION, source: 'global'});
            else {
                var bsEl = document.querySelector('[data-bs-version]');
                if (bsEl) out.push({name: 'Bootstrap', version: bsEl.getAttribute('data-bs-version'), source: 'global'});
            }
        } catch(e) {}
        return JSON.stringify(out);
    })()
    "#;

    let globals_result = page
        .evaluate(globals_js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Vulnerable libs globals JS failed: {e}")))?;

    let globals_str = globals_result
        .value()
        .and_then(|v| v.as_str())
        .unwrap_or("[]");

    #[derive(serde::Deserialize)]
    struct RawLib {
        name: String,
        version: String,
        source: String,
    }

    let mut global_libs: Vec<RawLib> = serde_json::from_str(globals_str).unwrap_or_default();

    // Also scan script URLs for version patterns
    let scripts_js = r#"
    (() => {
        var scripts = Array.from(document.querySelectorAll('script[src]'));
        return JSON.stringify(scripts.map(s => s.src.substring(0, 200)));
    })()
    "#;

    let scripts_result = page
        .evaluate(scripts_js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Vulnerable libs scripts JS failed: {e}")))?;

    let scripts_str = scripts_result
        .value()
        .and_then(|v| v.as_str())
        .unwrap_or("[]");

    let script_urls: Vec<String> = serde_json::from_str(scripts_str).unwrap_or_default();

    for url in &script_urls {
        if let Some(lib) = detect_from_url(url) {
            // Avoid duplicates with globals-detected libraries
            let already = global_libs.iter().any(|g| g.name == lib.name);
            if !already {
                global_libs.push(RawLib {
                    name: lib.name,
                    version: lib.version,
                    source: lib.source,
                });
            }
        }
    }

    // Deduplicate by name (prefer global detection over URL)
    let mut seen_names = std::collections::HashSet::new();
    let detected: Vec<DetectedLibrary> = global_libs
        .into_iter()
        .filter(|lib| !lib.version.is_empty() && seen_names.insert(lib.name.clone()))
        .map(|lib| DetectedLibrary {
            name: lib.name,
            version: lib.version,
            detection_source: lib.source,
        })
        .collect();

    let vulnerable: Vec<VulnerableLibrary> = detected
        .iter()
        .filter_map(|lib| check_vulnerability(&lib.name, &lib.version))
        .collect();

    let has_vulnerabilities = !vulnerable.is_empty();

    info!(
        "Vulnerable libs: {} detected, {} vulnerable",
        detected.len(),
        vulnerable.len()
    );

    Ok(VulnerableLibrariesAnalysis {
        detected,
        vulnerable,
        has_vulnerabilities,
    })
}

struct RawLibOwned {
    name: String,
    version: String,
    source: String,
}

/// Attempt to detect a library name and version from a script URL.
fn detect_from_url(url: &str) -> Option<RawLibOwned> {
    let lower = url.to_lowercase();

    // jQuery: jquery-1.11.3.min.js, jquery/3.6.0/jquery.min.js
    if lower.contains("jquery") {
        if let Some(version) = extract_version_from_url(url) {
            return Some(RawLibOwned {
                name: "jQuery".to_string(),
                version,
                source: "url".to_string(),
            });
        }
    }

    // Bootstrap
    if lower.contains("bootstrap") {
        if let Some(version) = extract_version_from_url(url) {
            return Some(RawLibOwned {
                name: "Bootstrap".to_string(),
                version,
                source: "url".to_string(),
            });
        }
    }

    // Angular.js (not Angular 2+)
    if lower.contains("angular") && !lower.contains("angular/") {
        if let Some(version) = extract_version_from_url(url) {
            return Some(RawLibOwned {
                name: "AngularJS".to_string(),
                version,
                source: "url".to_string(),
            });
        }
    }

    // Lodash
    if lower.contains("lodash") {
        if let Some(version) = extract_version_from_url(url) {
            return Some(RawLibOwned {
                name: "Lodash".to_string(),
                version,
                source: "url".to_string(),
            });
        }
    }

    // Handlebars
    if lower.contains("handlebars") {
        if let Some(version) = extract_version_from_url(url) {
            return Some(RawLibOwned {
                name: "Handlebars".to_string(),
                version,
                source: "url".to_string(),
            });
        }
    }

    None
}

/// Extract a version string (e.g. "3.6.0") following a keyword in a URL path segment.
fn extract_version_from_url(url: &str) -> Option<String> {
    find_version_pattern(url)
}

fn find_version_pattern(text: &str) -> Option<String> {
    // Manual pattern matching for version strings like -1.12.3. or /3.6.0/
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if (chars[i] == '-' || chars[i] == '/' || chars[i] == '@')
            && i + 1 < len
            && chars[i + 1].is_ascii_digit()
        {
            // Try to read a version: digits, dots, digits
            let start = i + 1;
            let mut j = start;
            let mut dots = 0;
            while j < len && (chars[j].is_ascii_digit() || chars[j] == '.') {
                if chars[j] == '.' {
                    dots += 1;
                }
                j += 1;
            }
            if dots >= 1 && j > start + 2 {
                let version: String = chars[start..j].iter().collect();
                // Trim trailing dots
                let version = version.trim_end_matches('.').to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        i += 1;
    }
    None
}

/// Check a library version against the known-vulnerable database.
fn check_vulnerability(name: &str, version: &str) -> Option<VulnerableLibrary> {
    let v = parse_semver(version)?;

    match name {
        "jQuery" => {
            // jQuery < 3.5.0: XSS via HTML processing (CVE-2020-11022, CVE-2020-11023)
            // jQuery < 1.12.0 / 2.x < 2.2.0: multiple XSS issues
            if v < (3, 5, 0) {
                return Some(VulnerableLibrary {
                    name: name.to_string(),
                    version: version.to_string(),
                    severity: if v < (1, 12, 0) || (v.0 == 2 && v < (2, 2, 0)) {
                        "high"
                    } else {
                        "medium"
                    }
                    .to_string(),
                    description: "Cross-site scripting (XSS) via HTML string processing (CVE-2020-11022, CVE-2020-11023).".to_string(),
                    safe_version: "3.5.0+".to_string(),
                });
            }
        }
        "Bootstrap" => {
            // Bootstrap 3.x < 3.4.1 and 4.x < 4.3.1: XSS via data attributes
            if (v.0 == 3 && v < (3, 4, 1)) || (v.0 == 4 && v < (4, 3, 1)) {
                return Some(VulnerableLibrary {
                    name: name.to_string(),
                    version: version.to_string(),
                    severity: "medium".to_string(),
                    description: "Cross-site scripting (XSS) via tooltip/popover data attributes (CVE-2019-8331).".to_string(),
                    safe_version: "3.4.1+ or 4.3.1+".to_string(),
                });
            }
        }
        "AngularJS" => {
            // AngularJS < 1.8.0: client-side template injection / XSS
            if v.0 == 1 && v < (1, 8, 0) {
                return Some(VulnerableLibrary {
                    name: name.to_string(),
                    version: version.to_string(),
                    severity: "high".to_string(),
                    description:
                        "Client-side template injection and cross-site scripting vulnerabilities."
                            .to_string(),
                    safe_version: "1.8.0+".to_string(),
                });
            }
        }
        "Handlebars" => {
            // Handlebars < 4.5.3: prototype pollution / RCE in template compilation
            if v < (4, 5, 3) {
                return Some(VulnerableLibrary {
                    name: name.to_string(),
                    version: version.to_string(),
                    severity: "high".to_string(),
                    description: "Prototype pollution and potential remote code execution in template compilation (CVE-2019-19919).".to_string(),
                    safe_version: "4.5.3+".to_string(),
                });
            }
        }
        "Lodash" => {
            // Lodash < 4.17.21: prototype pollution (CVE-2021-23337, CVE-2020-8203)
            if v < (4, 17, 21) {
                return Some(VulnerableLibrary {
                    name: name.to_string(),
                    version: version.to_string(),
                    severity: "medium".to_string(),
                    description:
                        "Prototype pollution vulnerabilities (CVE-2021-23337, CVE-2020-8203)."
                            .to_string(),
                    safe_version: "4.17.21+".to_string(),
                });
            }
        }
        "Underscore" => {
            // Underscore < 1.13.0: arbitrary code execution (CVE-2021-23358)
            if v < (1, 13, 0) {
                return Some(VulnerableLibrary {
                    name: name.to_string(),
                    version: version.to_string(),
                    severity: "high".to_string(),
                    description:
                        "Arbitrary code execution via template injection (CVE-2021-23358)."
                            .to_string(),
                    safe_version: "1.13.0+".to_string(),
                });
            }
        }
        "Prototype" | "MooTools" => {
            // These libraries are unmaintained and inherently risky
            return Some(VulnerableLibrary {
                name: name.to_string(),
                version: version.to_string(),
                severity: "medium".to_string(),
                description: "This library is no longer maintained and may contain unpatched security vulnerabilities.".to_string(),
                safe_version: "Migrate to a maintained library".to_string(),
            });
        }
        _ => {}
    }
    None
}

/// Parse a version string like "3.6.0" into (major, minor, patch).
/// Returns None if the string cannot be parsed.
fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.trim().split('.').collect();
    if parts.is_empty() {
        return None;
    }
    let major = parts.first().and_then(|s| s.parse::<u32>().ok())?;
    let minor = parts
        .get(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let patch = parts
        .get(2)
        .and_then(|s| {
            // Strip pre-release suffixes like "0-rc1"
            s.split('-').next()?.parse::<u32>().ok()
        })
        .unwrap_or(0);
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("3.6.0"), Some((3, 6, 0)));
        assert_eq!(parse_semver("1.12.4"), Some((1, 12, 4)));
        assert_eq!(parse_semver("4"), Some((4, 0, 0)));
        assert_eq!(parse_semver("invalid"), None);
    }

    #[test]
    fn test_jquery_vulnerable() {
        assert!(check_vulnerability("jQuery", "1.11.3").is_some());
        assert!(check_vulnerability("jQuery", "3.4.1").is_some());
    }

    #[test]
    fn test_jquery_safe() {
        assert!(check_vulnerability("jQuery", "3.5.0").is_none());
        assert!(check_vulnerability("jQuery", "3.7.1").is_none());
    }

    #[test]
    fn test_lodash_vulnerable() {
        assert!(check_vulnerability("Lodash", "4.17.20").is_some());
    }

    #[test]
    fn test_lodash_safe() {
        assert!(check_vulnerability("Lodash", "4.17.21").is_none());
    }

    #[test]
    fn test_extract_version_from_url() {
        assert_eq!(
            extract_version_from_url("https://code.jquery.com/jquery-3.6.0.min.js"),
            Some("3.6.0".to_string())
        );
        assert_eq!(
            extract_version_from_url("https://cdn.jsdelivr.net/npm/lodash@4.17.20/lodash.min.js"),
            Some("4.17.20".to_string())
        );
    }
}
