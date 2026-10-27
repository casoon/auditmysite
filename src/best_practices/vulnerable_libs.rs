//! Vulnerable JavaScript library detection (#124).
//!
//! Scans script elements and known window globals to detect JS libraries
//! and compares their versions against a hardcoded list of known-vulnerable
//! version ranges.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

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
    /// Known libraries loaded via more than one distinct version at once
    /// (#547) — detected from `<script src>` URLs carrying different version
    /// strings for the same library name. Global-namespace detection
    /// (`window.jQuery`, etc.) can only ever observe the last value written,
    /// so this signal is URL-based only and limited to the already-cataloged
    /// libraries in `detect_from_url`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duplicate_libraries: Vec<DuplicateLibraryVersions>,
    /// Whether detection ran. `false` means the empty lists are no
    /// measurement (plan 69); older reports carry no field.
    #[serde(default = "crate::best_practices::collected_by_default")]
    pub checked: bool,
}

/// A known library loaded under more than one distinct version at once.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateLibraryVersions {
    pub name: String,
    /// Distinct version strings found, in first-seen order
    pub versions: Vec<String>,
}

/// Detect JS libraries and check for known vulnerabilities.
/// Library globals and `<script src>` URLs; the best-practices score reads
/// the vulnerable libraries found in them.
pub(crate) static LIBRARIES: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "best_practices.libraries",
        module: "best_practices",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 1,
        scoring: true,
    };

#[derive(serde::Deserialize)]
pub(crate) struct RawLib {
    name: String,
    version: String,
    source: String,
}

/// Detected libraries and their known vulnerabilities. A script result that
/// is missing or malformed is no measurement — not a page without
/// libraries (#777).
pub async fn analyze_vulnerable_libraries(
    page: &Page,
) -> crate::audit::collection::Collected<VulnerableLibrariesAnalysis> {
    use crate::audit::collection::{evaluate_js, Collected};
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
            // Both Lodash and Underscore expose `_` with `_.VERSION` and (in
            // modern versions) `_.chunk`. `_.runInContext` is Lodash-exclusive,
            // so use it as the distinguisher — otherwise a Lodash 4.x build was
            // mislabelled "Underscore 4.17.5" (Underscore is at 1.x).
            if (typeof _ !== 'undefined' && _.VERSION && typeof _.runInContext === 'function')
                out.push({name: 'Lodash', version: _.VERSION, source: 'global'});
        } catch(e) {}
        try {
            if (typeof _ !== 'undefined' && _.VERSION && typeof _.runInContext !== 'function')
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

    let mut global_libs: Vec<RawLib> = match evaluate_js(page, globals_js).await.into_measurement()
    {
        Ok(libs) => libs,
        Err(failure) => return failure,
    };

    // Also scan script URLs for version patterns
    let scripts_js = r#"
    (() => {
        var scripts = Array.from(document.querySelectorAll('script[src]'));
        return JSON.stringify(scripts.map(s => s.src.substring(0, 200)));
    })()
    "#;

    let script_urls: Vec<String> = match evaluate_js(page, scripts_js).await.into_measurement() {
        Ok(urls) => urls,
        Err(failure) => return failure,
    };

    // Every URL-based match, kept even when a name repeats — needed to spot
    // two different versions of the same library loaded at once (#547).
    // Global-namespace detection above can't see this: a second `window.X =`
    // assignment silently overwrites the first, so only the URL scan (which
    // sees every <script src> tag) can observe both.
    let mut url_libs: Vec<RawLibOwned> = Vec::new();
    for url in &script_urls {
        if let Some(lib) = detect_from_url(url) {
            url_libs.push(lib);
        }
    }

    let duplicate_libraries = find_duplicate_versions(&url_libs);

    for lib in &url_libs {
        // Avoid duplicates with globals-detected libraries
        let already = global_libs.iter().any(|g| g.name == lib.name);
        if !already {
            global_libs.push(RawLib {
                name: lib.name.clone(),
                version: lib.version.clone(),
                source: lib.source.clone(),
            });
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

    Collected::Measured(VulnerableLibrariesAnalysis {
        detected,
        vulnerable,
        has_vulnerabilities,
        duplicate_libraries,
        checked: true,
    })
}

/// Group URL-detected libraries by name and return those with more than one
/// distinct version string (#547).
fn find_duplicate_versions(url_libs: &[RawLibOwned]) -> Vec<DuplicateLibraryVersions> {
    let mut by_name: Vec<(String, Vec<String>)> = Vec::new();
    for lib in url_libs {
        match by_name.iter_mut().find(|(name, _)| name == &lib.name) {
            Some((_, versions)) => {
                if !versions.contains(&lib.version) {
                    versions.push(lib.version.clone());
                }
            }
            None => by_name.push((lib.name.clone(), vec![lib.version.clone()])),
        }
    }
    by_name
        .into_iter()
        .filter(|(_, versions)| versions.len() > 1)
        .map(|(name, versions)| DuplicateLibraryVersions { name, versions })
        .collect()
}

struct RawLibOwned {
    name: String,
    version: String,
    source: String,
}

/// Whether a script's file name is the core build of `library` itself
/// (`jquery.min.js`, `jquery-3.6.0.js`, `bootstrap.bundle.min.js`) rather than
/// a plugin named after it (`jquery-ui.min.js`, `jquery.validate.js`,
/// `bootstrap-datepicker.min.js`), whose version is the plugin's own.
fn is_core_build(lower_path: &str, library: &str) -> bool {
    let file = lower_path.rsplit('/').next().unwrap_or("");
    let Some(rest) = file.strip_prefix(library) else {
        return false;
    };
    let rest = rest
        .strip_prefix('-')
        .filter(|r| r.starts_with(|c: char| c.is_ascii_digit()))
        .map(|r| r.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.'))
        .map(|r| format!(".{r}"))
        .unwrap_or_else(|| rest.to_string());
    matches!(
        rest.as_str(),
        ".js" | ".min.js" | ".slim.js" | ".slim.min.js" | ".bundle.js" | ".bundle.min.js"
    )
}

/// Attempt to detect a library name and version from a script URL. Only the
/// path is read: a host such as `127.0.0.1` is no version.
fn detect_from_url(url: &str) -> Option<RawLibOwned> {
    let path = url::Url::parse(url)
        .map(|u| u.path().to_string())
        .unwrap_or_else(|_| url.to_string());
    let url = path.as_str();
    let lower = url.to_lowercase();

    // jQuery: jquery-1.11.3.min.js, jquery/3.6.0/jquery.min.js
    if lower.contains("jquery") && is_core_build(&lower, "jquery") {
        if let Some(version) = extract_version_from_url(url) {
            return Some(RawLibOwned {
                name: "jQuery".to_string(),
                version,
                source: "url".to_string(),
            });
        }
    }

    // Bootstrap
    if lower.contains("bootstrap") && is_core_build(&lower, "bootstrap") {
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
/// The libraries whose versions are checked against known vulnerabilities —
/// the whole scope of the check. "No known vulnerability" means none among
/// these, not that the page has no vulnerable script (plan 69).
pub const CHECKED_LIBRARIES: &[&str] = &[
    "jQuery",
    "Bootstrap",
    "AngularJS",
    "Handlebars",
    "Lodash",
    "Underscore",
    "Prototype",
    "MooTools",
];

/// Check a library version against the known-vulnerable database.
/// Advisory boundaries follow the GitHub/NVD advisories as collected by
/// retire.js (`jsrepository-v4.json`, reviewed 2026-10-26, #783). A version
/// is flagged when at least one advisory covers it; end-of-life lines without
/// a fixed release (AngularJS, Bootstrap ≤ 3, Prototype, MooTools) are
/// flagged at every version (OWASP Top 10 A06: unsupported components).
fn check_vulnerability(name: &str, version: &str) -> Option<VulnerableLibrary> {
    let v = parse_semver(version)?;
    let found = |severity: &str, description: &str, safe_version: &str| {
        Some(VulnerableLibrary {
            name: name.to_string(),
            version: version.to_string(),
            severity: severity.to_string(),
            description: description.to_string(),
            safe_version: safe_version.to_string(),
        })
    };

    match name {
        "jQuery"
            // jQuery < 3.5.0: XSS via HTML processing (CVE-2020-11022, CVE-2020-11023)
            // jQuery < 1.12.0 / 2.x < 2.2.0: multiple XSS issues
            if v < (3, 5, 0) => {
                return found(
                    if v < (1, 12, 0) || (v.0 == 2 && v < (2, 2, 0)) {
                        "high"
                    } else {
                        "medium"
                    },
                    "Cross-site scripting (XSS) via HTML string processing (CVE-2020-11022, CVE-2020-11023).",
                    "3.5.0+",
                );
            }
        "Bootstrap"
            // Bootstrap 3 and earlier: end of life, every community release
            // affected (CVE-2024-6485 < 3.4.2, CVE-2025-1647 = 3.4.1).
            // Bootstrap 4.x < 4.3.1: CVE-2019-8331.
            if v < (4, 0, 0) || (v.0 == 4 && v < (4, 3, 1)) => {
                return found(
                    "medium",
                    if v < (4, 0, 0) {
                        "End-of-life Bootstrap line; cross-site scripting via data attributes and the carousel/button plugins (CVE-2019-8331, CVE-2024-6485, CVE-2025-1647)."
                    } else {
                        "Cross-site scripting (XSS) via tooltip/popover data attributes (CVE-2019-8331)."
                    },
                    "4.3.1+",
                );
            }
        "AngularJS" => {
            // End of life since January 2022; every release is affected
            // (CVE-2022-25844 for >= 1.7.0, CVE-2025-4690 < 1.9.9, which was
            // never released; CVE-2023-26116..26118 < 1.8.4).
            return found(
                "high",
                "AngularJS is end of life and every release is affected by published advisories (CVE-2022-25844, CVE-2023-26116, CVE-2025-4690).",
                "Migrate to a maintained framework",
            );
        }
        "Handlebars"
            // CVE-2026-106445 / CVE-2026-106446 (< 4.7.10), CVE-2026-33937
            // (< 4.7.9), CVE-2021-23369 (< 4.7.7).
            if v < (4, 7, 10) => {
                return found(
                    "high",
                    "Version affected by template compilation and prototype-access advisories that allow code execution when untrusted templates are compiled (CVE-2021-23369, CVE-2026-33937, CVE-2026-106445). Detection of the version does not establish exploitability.",
                    "4.7.10+",
                );
            }
        "Lodash"
            // Lodash < 4.18.0: code injection in _.template (CVE-2026-4800),
            // prototype pollution (CVE-2026-2950, CVE-2025-13465, CVE-2020-8203).
            if v < (4, 18, 0) => {
                return found(
                    "medium",
                    "Prototype pollution and template code injection vulnerabilities (CVE-2020-8203, CVE-2021-23337, CVE-2025-13465, CVE-2026-2950, CVE-2026-4800).",
                    "4.18.0+",
                );
            }
        "Underscore"
            // Underscore < 1.13.8: unbounded recursion DoS (CVE-2026-27601);
            // < 1.12.1: arbitrary code execution (CVE-2021-23358).
            if v < (1, 13, 8) => {
                return found(
                    if v < (1, 12, 1) { "high" } else { "medium" },
                    if v < (1, 12, 1) {
                        "Arbitrary code execution via template injection (CVE-2021-23358) and denial of service via unbounded recursion (CVE-2026-27601)."
                    } else {
                        "Denial of service via unbounded recursion in _.flatten and _.isEqual (CVE-2026-27601)."
                    },
                    "1.13.8+",
                );
            }
        "Prototype" | "MooTools" => {
            // Unmaintained: Prototype's last release 1.7.3 is affected by
            // CVE-2020-27511 (< 1.7.4, never released); MooTools 1.6.0 (the
            // last release) has no fix line for future issues.
            return found(
                "medium",
                "This library is no longer maintained and may contain unpatched security vulnerabilities.",
                "Migrate to a maintained library",
            );
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

/// Counter-cases of [`LIBRARIES`] (#777).
#[cfg(test)]
pub(crate) fn counter_cases() {
    use crate::audit::collection::assert_js_contract;
    assert_js_contract::<Vec<RawLib>>(
        serde_json::json!([{"name": "jQuery", "version": "1.0.0", "source": "global"}]),
    );
    assert_js_contract::<Vec<String>>(serde_json::json!([]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foundation_handlebars_advisory_boundaries_are_preserved() {
        // Independent advisory boundaries: GHSA-f2jv-r9rf-7988 (<4.7.7),
        // CVE-2026-33937 (<4.7.9), CVE-2026-106445 (<4.7.10).
        for version in [
            "4.5.2", "4.5.3", "4.6.0", "4.7.6", "4.7.7", "4.7.8", "4.7.9",
        ] {
            let finding = check_vulnerability("Handlebars", version).expect(version);
            assert!(finding.description.contains("CVE-2021-23369"));
        }
        for version in ["4.7.10", "4.7.11"] {
            assert!(check_vulnerability("Handlebars", version).is_none());
        }
        assert!(check_vulnerability("Handlebars", "unknown").is_none());
    }

    #[test]
    fn advisory_boundaries_of_the_other_libraries() {
        for (name, vulnerable, safe) in [
            ("Bootstrap", "2.3.2", "4.3.1"),
            ("Bootstrap", "3.4.1", "5.3.3"),
            ("Bootstrap", "4.3.0", "4.6.2"),
            ("Lodash", "4.17.21", "4.18.0"),
            ("Underscore", "1.13.7", "1.13.8"),
        ] {
            assert!(
                check_vulnerability(name, vulnerable).is_some(),
                "{name} {vulnerable}"
            );
            assert!(check_vulnerability(name, safe).is_none(), "{name} {safe}");
        }
        assert!(check_vulnerability("AngularJS", "1.8.3").is_some());
    }

    #[test]
    fn url_version_comes_from_the_path_not_the_host() {
        let lib = detect_from_url("http://127.0.0.1:8080/js/jquery-1.12.4.min.js").unwrap();
        assert_eq!(lib.version, "1.12.4");
    }

    #[test]
    fn plugins_named_after_a_library_are_not_the_library() {
        assert!(detect_from_url("https://code.jquery.com/ui/1.13.2/jquery-ui.min.js").is_none());
        assert!(detect_from_url("https://example.com/js/jquery.validate-1.19.5.js").is_none());
        assert!(
            detect_from_url("https://example.com/js/bootstrap-datepicker-1.9.0.min.js").is_none()
        );
        for url in [
            "https://ajax.googleapis.com/ajax/libs/jquery/3.6.0/jquery.min.js",
            "https://cdn.jsdelivr.net/npm/jquery@3.6.0/dist/jquery.slim.min.js",
            "https://example.com/js/jquery-3.6.0.min.js",
        ] {
            assert_eq!(detect_from_url(url).unwrap().version, "3.6.0", "{url}");
        }
        assert_eq!(
            detect_from_url(
                "https://cdn.jsdelivr.net/npm/bootstrap@4.1.3/dist/js/bootstrap.bundle.min.js"
            )
            .unwrap()
            .name,
            "Bootstrap"
        );
    }

    #[test]
    fn find_duplicate_versions_flags_two_distinct_versions() {
        let libs = vec![
            RawLibOwned {
                name: "jQuery".to_string(),
                version: "1.9.1".to_string(),
                source: "url".to_string(),
            },
            RawLibOwned {
                name: "jQuery".to_string(),
                version: "3.6.0".to_string(),
                source: "url".to_string(),
            },
        ];
        let dupes = find_duplicate_versions(&libs);
        assert_eq!(dupes.len(), 1);
        assert_eq!(dupes[0].name, "jQuery");
        assert_eq!(dupes[0].versions, vec!["1.9.1", "3.6.0"]);
    }

    #[test]
    fn find_duplicate_versions_ignores_repeated_same_version() {
        let libs = vec![
            RawLibOwned {
                name: "jQuery".to_string(),
                version: "3.6.0".to_string(),
                source: "url".to_string(),
            },
            RawLibOwned {
                name: "jQuery".to_string(),
                version: "3.6.0".to_string(),
                source: "url".to_string(),
            },
        ];
        assert!(find_duplicate_versions(&libs).is_empty());
    }

    #[test]
    fn find_duplicate_versions_ignores_different_libraries() {
        let libs = vec![
            RawLibOwned {
                name: "jQuery".to_string(),
                version: "3.6.0".to_string(),
                source: "url".to_string(),
            },
            RawLibOwned {
                name: "Bootstrap".to_string(),
                version: "5.0.0".to_string(),
                source: "url".to_string(),
            },
        ];
        assert!(find_duplicate_versions(&libs).is_empty());
    }

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
        assert!(check_vulnerability("Lodash", "4.17.21").is_some());
    }

    #[test]
    fn test_lodash_safe() {
        assert!(check_vulnerability("Lodash", "4.18.0").is_none());
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
