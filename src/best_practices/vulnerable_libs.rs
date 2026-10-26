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
    /// Whether the version string could be read as a version. An unparseable
    /// version is neither affected nor cleared — it is not checked.
    #[serde(default = "crate::best_practices::collected_by_default")]
    pub version_parsed: bool,
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
    /// Advisories whose affected range contains the detected version
    #[serde(default)]
    pub advisories: Vec<AdvisoryRef>,
    /// Whether exploitability is established. A detected version only ever
    /// proves the advisory range matches, so this stays `unconfirmed`.
    #[serde(default)]
    pub exploitability: Exploitability,
}

/// Reference to a matched entry of the advisory table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryRef {
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// End-of-maintenance notice rather than a published vulnerability
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unmaintained: bool,
    pub source_url: String,
    /// Date the affected range was last checked against `source_url`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed: Option<String>,
}

/// Whether a matched advisory is known to be exploitable on this page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exploitability {
    /// Version is in the affected range; reachability was not examined.
    #[default]
    Unconfirmed,
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
            version_parsed: parse_semver(&lib.version).is_some(),
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

/// Kind of advisory: a published vulnerability, or end of maintenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdvisoryKind {
    Vulnerability,
    Unmaintained,
}

/// One affected version range: `introduced` inclusive, `fixed` exclusive.
pub(crate) struct VersionRange {
    pub introduced: (u32, u32, u32),
    pub fixed: (u32, u32, u32),
}

/// One entry of the advisory table.
pub(crate) struct Advisory {
    pub library: &'static str,
    /// Primary identifier (GHSA id, or `local:` id when none was reviewed)
    pub id: &'static str,
    /// CVE ids and other aliases
    pub aliases: &'static [&'static str],
    pub kind: AdvisoryKind,
    pub severity: &'static str,
    pub description: &'static str,
    pub affected: &'static [VersionRange],
    pub safe_version: &'static str,
    /// Where the affected ranges were taken from
    pub source_url: &'static str,
    /// Date the bounds were last checked against `source_url`; `None` means
    /// carried over from the earlier hardcoded rules and not yet re-reviewed.
    pub reviewed: Option<&'static str>,
}

const ANY_VERSION: &[VersionRange] = &[VersionRange {
    introduced: (0, 0, 0),
    fixed: (u32::MAX, 0, 0),
}];

/// Versioned advisory table — the single source for which detected versions
/// count as affected.
///
/// Updating: for each library, read the advisory at `source_url`, copy its
/// affected ranges (`introduced` inclusive, `fixed` exclusive) and fixed
/// version, set `reviewed` to the date of the check, and add a test case for
/// the last affected and the first fixed version in `mod tests`. Add new
/// libraries to `CHECKED_LIBRARIES` and the detection scripts as well. A match
/// means the *version* is affected; it never proves the vulnerable code path
/// is reachable on the page.
pub(crate) const ADVISORIES: &[Advisory] = &[
    Advisory {
        library: "jQuery",
        id: "local:jquery-html-xss",
        aliases: &["CVE-2020-11022", "CVE-2020-11023"],
        kind: AdvisoryKind::Vulnerability,
        severity: "medium",
        description: "Cross-site scripting (XSS) via HTML string processing (CVE-2020-11022, CVE-2020-11023).",
        affected: &[VersionRange { introduced: (0, 0, 0), fixed: (3, 5, 0) }],
        safe_version: "3.5.0+",
        source_url: "https://nvd.nist.gov/vuln/detail/CVE-2020-11022",
        reviewed: None,
    },
    Advisory {
        library: "jQuery",
        id: "local:jquery-legacy-xss",
        aliases: &[],
        kind: AdvisoryKind::Vulnerability,
        severity: "high",
        description: "Multiple XSS issues in jQuery before 1.12.0 and 2.x before 2.2.0.",
        affected: &[
            VersionRange { introduced: (0, 0, 0), fixed: (1, 12, 0) },
            VersionRange { introduced: (2, 0, 0), fixed: (2, 2, 0) },
        ],
        safe_version: "3.5.0+",
        source_url: "https://blog.jquery.com/2016/01/08/jquery-2-2-and-1-12-released/",
        reviewed: None,
    },
    Advisory {
        library: "Bootstrap",
        id: "local:bootstrap-tooltip-xss",
        aliases: &["CVE-2019-8331"],
        kind: AdvisoryKind::Vulnerability,
        severity: "medium",
        description: "Cross-site scripting (XSS) via tooltip/popover data attributes (CVE-2019-8331).",
        affected: &[
            VersionRange { introduced: (3, 0, 0), fixed: (3, 4, 1) },
            VersionRange { introduced: (4, 0, 0), fixed: (4, 3, 1) },
        ],
        safe_version: "3.4.1+ or 4.3.1+",
        source_url: "https://nvd.nist.gov/vuln/detail/CVE-2019-8331",
        reviewed: None,
    },
    Advisory {
        library: "AngularJS",
        id: "local:angularjs-pre-1.8",
        aliases: &[],
        kind: AdvisoryKind::Vulnerability,
        severity: "high",
        description: "Client-side template injection and cross-site scripting vulnerabilities.",
        affected: &[VersionRange { introduced: (1, 0, 0), fixed: (1, 8, 0) }],
        safe_version: "1.8.0+",
        source_url: "https://docs.angularjs.org/misc/version-support-status",
        reviewed: None,
    },
    Advisory {
        library: "Handlebars",
        id: "GHSA-f2jv-r9rf-7988",
        aliases: &["CVE-2021-23369"],
        kind: AdvisoryKind::Vulnerability,
        severity: "high",
        description: "Version affected by CVE-2021-23369: remote code execution when untrusted templates are compiled with certain options. Detection of the version does not establish exploitability.",
        affected: &[VersionRange { introduced: (0, 0, 0), fixed: (4, 7, 7) }],
        safe_version: "4.7.7+ (for CVE-2021-23369)",
        source_url: "https://github.com/advisories/GHSA-f2jv-r9rf-7988",
        reviewed: Some("2026-10-26"),
    },
    Advisory {
        library: "Lodash",
        id: "local:lodash-prototype-pollution",
        aliases: &["CVE-2021-23337", "CVE-2020-8203"],
        kind: AdvisoryKind::Vulnerability,
        severity: "medium",
        description: "Prototype pollution vulnerabilities (CVE-2021-23337, CVE-2020-8203).",
        affected: &[VersionRange { introduced: (0, 0, 0), fixed: (4, 17, 21) }],
        safe_version: "4.17.21+",
        source_url: "https://nvd.nist.gov/vuln/detail/CVE-2021-23337",
        reviewed: None,
    },
    Advisory {
        library: "Underscore",
        id: "local:underscore-template-rce",
        aliases: &["CVE-2021-23358"],
        kind: AdvisoryKind::Vulnerability,
        severity: "high",
        description: "Arbitrary code execution via template injection (CVE-2021-23358).",
        affected: &[VersionRange { introduced: (0, 0, 0), fixed: (1, 13, 0) }],
        safe_version: "1.13.0+",
        source_url: "https://nvd.nist.gov/vuln/detail/CVE-2021-23358",
        reviewed: None,
    },
    Advisory {
        library: "Prototype",
        id: "local:prototype-unmaintained",
        aliases: &[],
        kind: AdvisoryKind::Unmaintained,
        severity: "medium",
        description: "This library is no longer maintained and may contain unpatched security vulnerabilities.",
        affected: ANY_VERSION,
        safe_version: "Migrate to a maintained library",
        source_url: "https://prototypejs.org/",
        reviewed: None,
    },
    Advisory {
        library: "MooTools",
        id: "local:mootools-unmaintained",
        aliases: &[],
        kind: AdvisoryKind::Unmaintained,
        severity: "medium",
        description: "This library is no longer maintained and may contain unpatched security vulnerabilities.",
        affected: ANY_VERSION,
        safe_version: "Migrate to a maintained library",
        source_url: "https://mootools.net/",
        reviewed: None,
    },
];

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "high" => 2,
        "medium" => 1,
        _ => 0,
    }
}

/// Check a library version against the advisory table. A version that does
/// not parse matches nothing: it is neither affected nor cleared.
fn check_vulnerability(name: &str, version: &str) -> Option<VulnerableLibrary> {
    let v = parse_semver(version)?;
    let matches: Vec<&Advisory> = ADVISORIES
        .iter()
        .filter(|a| a.library == name)
        .filter(|a| a.affected.iter().any(|r| v >= r.introduced && v < r.fixed))
        .collect();
    let first = matches.first()?;
    let severity = matches
        .iter()
        .map(|a| a.severity)
        .max_by_key(|s| severity_rank(s))?;
    Some(VulnerableLibrary {
        name: name.to_string(),
        version: version.to_string(),
        severity: severity.to_string(),
        description: matches
            .iter()
            .map(|a| a.description)
            .collect::<Vec<_>>()
            .join(" "),
        safe_version: first.safe_version.to_string(),
        advisories: matches
            .iter()
            .map(|a| AdvisoryRef {
                id: a.id.to_string(),
                aliases: a.aliases.iter().map(|s| s.to_string()).collect(),
                unmaintained: a.kind == AdvisoryKind::Unmaintained,
                source_url: a.source_url.to_string(),
                reviewed: a.reviewed.map(str::to_string),
            })
            .collect(),
        exploitability: Exploitability::Unconfirmed,
    })
}

/// Parse a version string like "3.6.0" into (major, minor, patch). Every
/// present component must be numeric (a `-rc1` suffix on the last is
/// stripped); anything else is `None`, never a guessed version.
fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let mut parts = version.trim().split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = match parts.next() {
        Some(s) => s.split('-').next()?.parse::<u32>().ok()?,
        None => 0,
    };
    let patch = match parts.next() {
        Some(s) => s.split('-').next()?.parse::<u32>().ok()?,
        None => 0,
    };
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

    fn ids(name: &str, version: &str) -> Option<Vec<String>> {
        check_vulnerability(name, version).map(|f| f.advisories.into_iter().map(|a| a.id).collect())
    }

    /// (library, last affected, first fixed) per advisory-backed library.
    const BOUNDS: &[(&str, &str, &str)] = &[
        ("jQuery", "3.4.1", "3.5.0"),
        ("jQuery", "1.11.3", "3.5.0"),
        ("Bootstrap", "3.4.0", "3.4.1"),
        ("Bootstrap", "4.3.0", "4.3.1"),
        ("AngularJS", "1.7.9", "1.8.0"),
        ("Handlebars", "4.7.6", "4.7.7"),
        ("Lodash", "4.17.20", "4.17.21"),
        ("Underscore", "1.12.1", "1.13.0"),
    ];

    #[test]
    fn affected_and_fixed_versions_per_library() {
        for (lib, affected, fixed) in BOUNDS {
            assert!(
                check_vulnerability(lib, affected).is_some(),
                "{lib} {affected} must be affected"
            );
            assert!(
                check_vulnerability(lib, fixed).is_none(),
                "{lib} {fixed} must not be affected"
            );
        }
        // Outside every affected range of the library
        assert!(check_vulnerability("Bootstrap", "5.3.3").is_none());
        assert!(check_vulnerability("AngularJS", "1.8.3").is_none());
        assert!(check_vulnerability("Underscore", "1.13.7").is_none());
    }

    #[test]
    fn handlebars_range_is_below_4_7_7_not_below_4_5_3() {
        // GHSA-f2jv-r9rf-7988 / CVE-2021-23369
        for version in ["4.5.2", "4.5.3", "4.6.0", "4.7.6"] {
            let found = ids("Handlebars", version).expect(version);
            assert_eq!(found, vec!["GHSA-f2jv-r9rf-7988"]);
        }
        for version in ["4.7.7", "4.7.8", "5.0.0"] {
            assert!(check_vulnerability("Handlebars", version).is_none());
        }
    }

    #[test]
    fn unmaintained_libraries_match_any_parsed_version_as_notice() {
        for lib in ["Prototype", "MooTools"] {
            let f = check_vulnerability(lib, "1.7.3").expect(lib);
            assert!(f.advisories.iter().all(|a| a.unmaintained));
        }
    }

    #[test]
    fn unparseable_versions_are_neither_affected_nor_cleared() {
        for lib in CHECKED_LIBRARIES {
            for bad in ["unknown", "", "x.y.z", "4.x", "4.7.x", "latest", "4..7"] {
                assert!(check_vulnerability(lib, bad).is_none(), "{lib} {bad:?}");
                assert_eq!(parse_semver(bad), None, "{bad:?}");
            }
        }
    }

    #[test]
    fn unlisted_library_is_never_reported() {
        assert!(check_vulnerability("React", "0.0.1").is_none());
    }

    #[test]
    fn jquery_reports_highest_severity_and_all_matching_advisories() {
        let old = check_vulnerability("jQuery", "1.11.3").unwrap();
        assert_eq!(old.severity, "high");
        assert_eq!(old.advisories.len(), 2);
        let mid = check_vulnerability("jQuery", "3.4.1").unwrap();
        assert_eq!(mid.severity, "medium");
        assert_eq!(mid.advisories.len(), 1);
    }

    #[test]
    fn match_keeps_exploitability_unconfirmed_and_cites_a_source() {
        let f = check_vulnerability("Handlebars", "4.7.6").unwrap();
        assert_eq!(f.exploitability, Exploitability::Unconfirmed);
        assert!(f.advisories[0].source_url.starts_with("https://"));
        assert_eq!(f.advisories[0].reviewed.as_deref(), Some("2026-10-26"));
        assert!(f.advisories[0]
            .aliases
            .contains(&"CVE-2021-23369".to_string()));
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["exploitability"], "unconfirmed");
    }

    #[test]
    fn advisory_table_is_well_formed() {
        for a in ADVISORIES {
            assert!(CHECKED_LIBRARIES.contains(&a.library), "{}", a.id);
            assert!(!a.affected.is_empty(), "{}", a.id);
            for r in a.affected {
                assert!(r.introduced < r.fixed, "{}", a.id);
            }
            assert!(a.source_url.starts_with("https://"), "{}", a.id);
        }
        let mut seen = std::collections::HashSet::new();
        assert!(ADVISORIES.iter().all(|a| seen.insert(a.id)));
    }

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("3.6.0"), Some((3, 6, 0)));
        assert_eq!(parse_semver("1.12.4"), Some((1, 12, 4)));
        assert_eq!(parse_semver("4"), Some((4, 0, 0)));
        assert_eq!(parse_semver("1.8.0-rc1"), Some((1, 8, 0)));
        assert_eq!(parse_semver("invalid"), None);
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
