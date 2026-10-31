//! Vulnerable JavaScript library detection (#124).
//!
//! Scans script elements and known window globals to detect JS libraries
//! and compares their versions against a table of known-vulnerable version ranges
//! (`advisories.json`).

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
    /// proves the advisory range matches, so this is `unconfirmed` — or
    /// `not_applicable` when only maintenance notices matched.
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
    /// Whether the affected range was checked against `source_url`. An
    /// `unreviewed` range is carried over from earlier rules: the match is a
    /// lead to verify, not an established finding.
    #[serde(default)]
    pub review_status: ReviewStatus,
    /// Date the affected range was last checked against `source_url`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed: Option<String>,
}

/// Whether an advisory's affected range has been checked against its source.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Reviewed,
    /// Also what reports written before this field read as.
    #[default]
    Unreviewed,
}

/// Whether a matched advisory is known to be exploitable on this page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exploitability {
    /// Version is in the affected range; reachability was not examined.
    #[default]
    Unconfirmed,
    /// Every matched advisory is an end-of-maintenance notice: no specific
    /// vulnerability is named, so there is nothing to exploit or to confirm.
    /// There is no `confirmed` state: it would need a reachability probe of the
    /// page, which this check does not run.
    NotApplicable,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AdvisoryKind {
    Vulnerability,
    Unmaintained,
}

/// One affected version range: `introduced` inclusive, `fixed` exclusive.
/// `fixed: None` means no release is fixed (every later version is affected).
#[derive(Debug)]
pub(crate) struct VersionRange {
    pub introduced: (u32, u32, u32),
    pub fixed: Option<(u32, u32, u32)>,
}

impl VersionRange {
    fn contains(&self, v: (u32, u32, u32)) -> bool {
        v >= self.introduced && self.fixed.is_none_or(|f| v < f)
    }
}

/// One entry of the advisory table.
#[derive(Debug)]
pub(crate) struct Advisory {
    pub library: String,
    /// Primary identifier (GHSA id, or `local:` id when none was reviewed)
    pub id: String,
    /// CVE ids and other aliases
    pub aliases: Vec<String>,
    pub kind: AdvisoryKind,
    pub severity: String,
    pub description: String,
    pub affected: Vec<VersionRange>,
    pub safe_version: String,
    /// Where the affected ranges were taken from
    pub source_url: String,
    /// Date the bounds were last checked against `source_url`; `None` means
    /// carried over from the earlier hardcoded rules and not yet re-reviewed.
    pub reviewed: Option<String>,
}

/// Version of the data-file layout this build reads.
const ADVISORY_SCHEMA_VERSION: u32 = 1;

/// The shipped advisory data, compiled in from `advisories.json`.
const ADVISORY_DATA: &str = include_str!("advisories.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdvisoryFile {
    schema_version: u32,
    /// Date the file as a whole was last edited
    updated: String,
    advisories: Vec<AdvisoryRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdvisoryRecord {
    library: String,
    id: String,
    #[serde(default)]
    aliases: Vec<String>,
    kind: AdvisoryKind,
    severity: String,
    description: String,
    affected: Vec<RangeRecord>,
    safe_version: String,
    source_url: String,
    reviewed: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RangeRecord {
    introduced: String,
    fixed: Option<String>,
}

fn parse_date(what: &str, value: &str) -> Result<(), String> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| format!("{what}: {value:?} is not an ISO date (YYYY-MM-DD)"))
}

/// Strictly parse a range bound: exactly `major.minor.patch`, all numeric.
fn parse_bound(what: &str, value: &str) -> Result<(u32, u32, u32), String> {
    let parts: Vec<&str> = value.split('.').collect();
    let nums: Option<Vec<u32>> = parts.iter().map(|p| p.parse().ok()).collect();
    match nums.as_deref() {
        Some(&[a, b, c]) => Ok((a, b, c)),
        _ => Err(format!("{what}: {value:?} is not major.minor.patch")),
    }
}

/// Parse and validate an advisory data file. Rejects anything that could make
/// the check silently wrong: unknown fields, unsupported schema version,
/// duplicate ids, libraries outside [`CHECKED_LIBRARIES`], empty or inverted or
/// overlapping version ranges, non-HTTPS sources and malformed dates.
pub(crate) fn load_advisories(json: &str) -> Result<Vec<Advisory>, String> {
    let file: AdvisoryFile =
        serde_json::from_str(json).map_err(|e| format!("advisory data: {e}"))?;
    if file.schema_version != ADVISORY_SCHEMA_VERSION {
        return Err(format!(
            "advisory data: schema_version {} is not supported (expected {ADVISORY_SCHEMA_VERSION})",
            file.schema_version
        ));
    }
    parse_date("updated", &file.updated)?;
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(file.advisories.len());
    for r in file.advisories {
        let id = r.id.clone();
        let err = |msg: &str| format!("advisory {id}: {msg}");
        if r.id.is_empty() || !seen.insert(r.id.clone()) {
            return Err(err("id is empty or duplicated"));
        }
        if !CHECKED_LIBRARIES.contains(&r.library.as_str()) {
            return Err(err(&format!(
                "library {:?} is not a checked library",
                r.library
            )));
        }
        if !matches!(r.severity.as_str(), "high" | "medium" | "low") {
            return Err(err("severity must be high, medium or low"));
        }
        if r.description.trim().is_empty() || r.safe_version.trim().is_empty() {
            return Err(err("description and safe_version must not be empty"));
        }
        if !r.source_url.starts_with("https://") {
            return Err(err("source_url must be an https URL"));
        }
        if let Some(d) = &r.reviewed {
            parse_date(&format!("advisory {id} reviewed"), d)?;
        }
        if r.affected.is_empty() {
            return Err(err("affected must list at least one range"));
        }
        let mut affected = Vec::with_capacity(r.affected.len());
        for range in &r.affected {
            let introduced = parse_bound(&format!("advisory {id} introduced"), &range.introduced)?;
            let fixed = range
                .fixed
                .as_deref()
                .map(|f| parse_bound(&format!("advisory {id} fixed"), f))
                .transpose()?;
            if fixed.is_some_and(|f| introduced >= f) {
                return Err(err("a range must have introduced < fixed"));
            }
            affected.push(VersionRange { introduced, fixed });
        }
        affected.sort_by_key(|r| r.introduced);
        for pair in affected.windows(2) {
            let overlaps = pair[0].fixed.is_none_or(|f| f > pair[1].introduced);
            if overlaps {
                return Err(err("ranges overlap"));
            }
        }
        out.push(Advisory {
            library: r.library,
            id: r.id,
            aliases: r.aliases,
            kind: r.kind,
            severity: r.severity,
            description: r.description,
            affected,
            safe_version: r.safe_version,
            source_url: r.source_url,
            reviewed: r.reviewed,
        });
    }
    Ok(out)
}

/// The advisory table — the single source for which detected versions count as
/// affected. Data lives in `advisories.json`; the update process is in
/// `docs/advisory-data.md`. A match means the *version* is affected; it never
/// proves the vulnerable code path is reachable on the page.
pub(crate) static ADVISORIES: std::sync::LazyLock<Vec<Advisory>> = std::sync::LazyLock::new(|| {
    load_advisories(ADVISORY_DATA).expect("embedded advisories.json must pass validation")
});

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
        .filter(|a| a.affected.iter().any(|r| r.contains(v)))
        .collect();
    let first = matches.first()?;
    let severity = matches
        .iter()
        .map(|a| a.severity.as_str())
        .max_by_key(|s| severity_rank(s))?;
    Some(VulnerableLibrary {
        name: name.to_string(),
        version: version.to_string(),
        severity: severity.to_string(),
        description: matches
            .iter()
            .map(|a| a.description.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        safe_version: first.safe_version.clone(),
        advisories: matches
            .iter()
            .map(|a| AdvisoryRef {
                id: a.id.clone(),
                aliases: a.aliases.clone(),
                unmaintained: a.kind == AdvisoryKind::Unmaintained,
                source_url: a.source_url.clone(),
                review_status: if a.reviewed.is_some() {
                    ReviewStatus::Reviewed
                } else {
                    ReviewStatus::Unreviewed
                },
                reviewed: a.reviewed.clone(),
            })
            .collect(),
        exploitability: if matches.iter().all(|a| a.kind == AdvisoryKind::Unmaintained) {
            Exploitability::NotApplicable
        } else {
            Exploitability::Unconfirmed
        },
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

    fn ids(name: &str, version: &str) -> Option<Vec<String>> {
        check_vulnerability(name, version).map(|f| f.advisories.into_iter().map(|a| a.id).collect())
    }

    /// (library, last affected, first fixed) per advisory-backed library.
    const BOUNDS: &[(&str, &str, &str)] = &[
        ("jQuery", "3.4.1", "3.5.0"),
        ("jQuery", "1.11.3", "3.5.0"),
        ("Bootstrap", "3.4.1", "4.3.1"),
        ("Bootstrap", "4.3.0", "4.3.1"),
        ("Handlebars", "4.7.9", "4.7.10"),
        ("Lodash", "4.17.21", "4.18.0"),
        ("Underscore", "1.13.7", "1.13.8"),
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
    }

    #[test]
    fn handlebars_ranges_end_at_4_7_7_and_4_7_10() {
        // GHSA-f2jv-r9rf-7988 / CVE-2021-23369 (< 4.7.7), not only < 4.5.3;
        // CVE-2026-33937 (< 4.7.9) and CVE-2026-106445 (< 4.7.10).
        for version in ["4.5.2", "4.5.3", "4.6.0", "4.7.6"] {
            let found = ids("Handlebars", version).expect(version);
            assert_eq!(
                found,
                vec![
                    "GHSA-f2jv-r9rf-7988",
                    "local:handlebars-template-compilation"
                ]
            );
        }
        for version in ["4.7.7", "4.7.8", "4.7.9"] {
            let found = ids("Handlebars", version).expect(version);
            assert_eq!(found, vec!["local:handlebars-template-compilation"]);
        }
        for version in ["4.7.10", "4.7.11", "5.0.0"] {
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
        assert_eq!(f.advisories[0].review_status, ReviewStatus::Reviewed);
        assert!(f.advisories[0]
            .aliases
            .contains(&"CVE-2021-23369".to_string()));
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["exploitability"], "unconfirmed");
    }

    #[test]
    fn shipped_advisory_data_passes_validation() {
        let table = load_advisories(ADVISORY_DATA).expect("advisories.json");
        assert_eq!(table.len(), ADVISORIES.len());
        // Every checked library is backed by at least one advisory.
        for lib in CHECKED_LIBRARIES {
            assert!(table.iter().any(|a| a.library == *lib), "{lib}");
        }
    }

    /// The shipped file with one field of the first advisory replaced.
    fn data_with(find: &str, replace: &str) -> String {
        assert_eq!(ADVISORY_DATA.matches(find).count(), 1, "{find}");
        ADVISORY_DATA.replacen(find, replace, 1)
    }

    #[test]
    fn loader_rejects_data_that_would_make_the_check_wrong() {
        let cases = [
            ("\"schema_version\": 1", "\"schema_version\": 2", "schema_version"),
            ("\"updated\": \"2026-10-26\"", "\"updated\": \"26.10.2026\"", "ISO date"),
            ("\"id\": \"local:jquery-legacy-xss\"", "\"id\": \"local:jquery-html-xss\"", "duplicated"),
            ("\"library\": \"Bootstrap\",\n      \"id\": \"local:bootstrap-tooltip-xss\"", "\"library\": \"React\",\n      \"id\": \"local:bootstrap-tooltip-xss\"", "checked library"),
            ("\"severity\": \"high\",\n      \"description\": \"Multiple", "\"severity\": \"critical\",\n      \"description\": \"Multiple", "severity"),
            ("\"source_url\": \"https://nvd.nist.gov/vuln/detail/CVE-2020-11022\"", "\"source_url\": \"http://nvd.nist.gov/\"", "https"),
            ("\"fixed\": \"3.5.0\"\n        }\n      ],\n      \"safe_version\": \"3.5.0+\",\n      \"source_url\": \"https://nvd", "\"fixed\": \"0.0.0\"\n        }\n      ],\n      \"safe_version\": \"3.5.0+\",\n      \"source_url\": \"https://nvd", "introduced < fixed"),
            ("\"fixed\": \"3.5.0\"\n        }\n      ],\n      \"safe_version\": \"3.5.0+\",\n      \"source_url\": \"https://nvd", "\"fixed\": \"3.5\"\n        }\n      ],\n      \"safe_version\": \"3.5.0+\",\n      \"source_url\": \"https://nvd", "major.minor.patch"),
        ];
        for (find, replace, expect) in cases {
            let err = load_advisories(&data_with(find, replace))
                .err()
                .unwrap_or_default();
            assert!(err.contains(expect), "{expect}: got {err:?}");
        }
        assert!(load_advisories("{}").is_err());
        assert!(load_advisories(&data_with("\"library\": \"jQuery\",\n      \"id\": \"local:jquery-html-xss\"", "\"unknown\": 1,\n      \"library\": \"jQuery\",\n      \"id\": \"local:jquery-html-xss\"")).is_err());
    }

    #[test]
    fn loader_rejects_empty_and_overlapping_ranges_and_bad_review_dates() {
        let one = |affected: &str, reviewed: &str| {
            format!(
                r#"{{"schema_version":1,"updated":"2026-10-31","advisories":[{{"library":"Lodash","id":"x","kind":"vulnerability","severity":"low","description":"d","affected":{affected},"safe_version":"1","source_url":"https://example.com","reviewed":{reviewed}}}]}}"#
            )
        };
        assert!(
            load_advisories(&one(r#"[{"introduced":"0.0.0","fixed":"1.0.0"}]"#, "null")).is_ok()
        );
        assert!(load_advisories(&one("[]", "null"))
            .unwrap_err()
            .contains("at least one"));
        assert!(load_advisories(&one(
            r#"[{"introduced":"0.0.0","fixed":"2.0.0"},{"introduced":"1.0.0","fixed":"3.0.0"}]"#,
            "null"
        ))
        .unwrap_err()
        .contains("overlap"));
        assert!(load_advisories(&one(
            r#"[{"introduced":"0.0.0","fixed":null},{"introduced":"1.0.0","fixed":"3.0.0"}]"#,
            "null"
        ))
        .unwrap_err()
        .contains("overlap"));
        assert!(load_advisories(&one(
            r#"[{"introduced":"0.0.0","fixed":"1.0.0"}]"#,
            r#""yesterday""#
        ))
        .unwrap_err()
        .contains("ISO date"));
    }

    #[test]
    fn unreviewed_advisories_are_marked_unreviewed_in_the_finding() {
        // jQuery < 3.5.0 matches only advisories carried over without review.
        let f = check_vulnerability("jQuery", "3.4.1").unwrap();
        assert_eq!(f.advisories[0].review_status, ReviewStatus::Unreviewed);
        assert!(f.advisories[0].reviewed.is_none());
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["advisories"][0]["review_status"], "unreviewed");
        let g = check_vulnerability("Lodash", "4.17.21").unwrap();
        assert_eq!(g.advisories[0].review_status, ReviewStatus::Reviewed);
        assert_eq!(
            serde_json::to_value(&g).unwrap()["advisories"][0]["review_status"],
            "reviewed"
        );
        // A report written before the field existed reads as unreviewed.
        let old: AdvisoryRef =
            serde_json::from_str(r#"{"id":"x","source_url":"https://e.com"}"#).unwrap();
        assert_eq!(old.review_status, ReviewStatus::Unreviewed);
    }

    #[test]
    fn maintenance_notices_have_no_exploitability_to_confirm() {
        for lib in ["Prototype", "MooTools"] {
            let f = check_vulnerability(lib, "1.7.3").unwrap();
            assert_eq!(f.exploitability, Exploitability::NotApplicable, "{lib}");
            assert_eq!(
                serde_json::to_value(&f).unwrap()["exploitability"],
                "not_applicable"
            );
        }
        // One real advisory among the matches keeps it unconfirmed.
        let f = check_vulnerability("AngularJS", "1.8.3").unwrap();
        assert_eq!(f.exploitability, Exploitability::Unconfirmed);
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
