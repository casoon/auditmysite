//! Collection contract (#777): every collector and subcheck reports what it
//! actually measured.
//!
//! Raw data enters the audit from outside the process — JavaScript
//! evaluation, CDP commands, HTTP requests, DNS lookups. A failure at that
//! boundary must never turn into a regular domain value: no "missing header"
//! from a request that never completed, no 100 points from a collector that
//! returned nothing. [`Collected`] carries the state through the analysis
//! code; [`CollectionLog`] records it per collector so the report can say
//! which measurements exist ([`CollectionRecord`] in
//! `module_runs[].subchecks[]`).
//!
//! Every collector is declared once in [`COLLECTORS`]. The inventory test
//! requires counter-cases for each entry, so a new collector without them
//! fails the build's tests.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Version of the collection contract itself. Bump it when the meaning of a
/// [`CollectionState`] changes.
pub const COLLECTION_CONTRACT_VERSION: u32 = 1;

/// How a collector obtains its raw data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionMethod {
    /// `Runtime.evaluate` of a collection script.
    JavaScript,
    /// A CDP domain command or event (Network, CSS, Profiler, …).
    Cdp,
    /// An HTTP request outside the browser.
    Http,
    /// A DNS lookup.
    Dns,
}

/// Outcome of one collection, as published in the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionState {
    /// The collector ran and returned a valid, non-empty result.
    Measured,
    /// The collector ran and the page validly has nothing to report
    /// (no third-party origins, no images, …).
    Empty,
    /// Part of the collection succeeded; `gaps` names what is missing.
    Partial,
    /// The collector ran and failed technically (transport error, invalid
    /// result, wrong schema). No domain value may be derived from it.
    Failed,
    /// The environment cannot perform this collection (CDP domain missing,
    /// emulation not available).
    Unsupported,
    /// The collection does not apply to this page (e.g. no web manifest is
    /// linked, so there is nothing to fetch).
    NotApplicable,
}

impl CollectionState {
    /// Whether the collection produced a measurement the analysis may use.
    pub fn has_value(self) -> bool {
        matches!(self, Self::Measured | Self::Empty | Self::Partial)
    }

    /// Whether the module lost (part of) a measurement it would otherwise
    /// have: failed, unsupported or partial. `NotApplicable` is no gap.
    pub fn is_gap(self) -> bool {
        matches!(self, Self::Partial | Self::Failed | Self::Unsupported)
    }

    /// The execution status this state projects to in `module_runs`.
    pub fn execution_status(self) -> super::ExecutionStatus {
        use super::ExecutionStatus;
        match self {
            Self::Measured | Self::Empty => ExecutionStatus::Completed,
            Self::Partial => ExecutionStatus::Partial,
            Self::Failed => ExecutionStatus::Failed,
            Self::Unsupported => ExecutionStatus::Skipped,
            Self::NotApplicable => ExecutionStatus::NotApplicable,
        }
    }
}

/// Static description of one collector. Declared once per collection call
/// site and listed in [`COLLECTORS`].
#[derive(Debug)]
pub struct CollectorSpec {
    /// Stable id, `<module>.<collector>`; appears in the report.
    pub id: &'static str,
    /// Module whose run the record belongs to (`module_runs[].module`).
    pub module: &'static str,
    pub method: CollectionMethod,
    /// Version of this collector's method (script, request shape). Bump it
    /// when the collected data changes meaning.
    pub method_version: u32,
    /// Whether the module score uses this collection. A failed scoring
    /// collector makes the module score partial and non-contributing.
    pub scoring: bool,
}

/// Collected value or the reason there is none.
#[derive(Debug, Clone, PartialEq)]
pub enum Collected<T> {
    Measured(T),
    /// A valid, empty measurement; the value is the empty result.
    Empty(T),
    Partial {
        value: T,
        gaps: Vec<&'static str>,
    },
    Failed(&'static str),
    Unsupported(&'static str),
    NotApplicable(&'static str),
}

impl<T> Collected<T> {
    pub fn state(&self) -> CollectionState {
        match self {
            Self::Measured(_) => CollectionState::Measured,
            Self::Empty(_) => CollectionState::Empty,
            Self::Partial { .. } => CollectionState::Partial,
            Self::Failed(_) => CollectionState::Failed,
            Self::Unsupported(_) => CollectionState::Unsupported,
            Self::NotApplicable(_) => CollectionState::NotApplicable,
        }
    }

    /// The measured value, `None` when nothing was measured.
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Measured(v) | Self::Empty(v) | Self::Partial { value: v, .. } => Some(v),
            Self::Failed(_) | Self::Unsupported(_) | Self::NotApplicable(_) => None,
        }
    }

    pub fn into_value(self) -> Option<T> {
        match self {
            Self::Measured(v) | Self::Empty(v) | Self::Partial { value: v, .. } => Some(v),
            Self::Failed(_) | Self::Unsupported(_) | Self::NotApplicable(_) => None,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Collected<U> {
        match self {
            Self::Measured(v) => Collected::Measured(f(v)),
            Self::Empty(v) => Collected::Empty(f(v)),
            Self::Partial { value, gaps } => Collected::Partial {
                value: f(value),
                gaps,
            },
            Self::Failed(r) => Collected::Failed(r),
            Self::Unsupported(r) => Collected::Unsupported(r),
            Self::NotApplicable(r) => Collected::NotApplicable(r),
        }
    }

    /// The value, or the state without one retyped — for collectors that
    /// combine several collections and stop at the first without a value.
    pub fn into_measurement<U>(self) -> Result<T, Collected<U>> {
        match self {
            Self::Measured(v) | Self::Empty(v) | Self::Partial { value: v, .. } => Ok(v),
            Self::Failed(r) => Err(Collected::Failed(r)),
            Self::Unsupported(r) => Err(Collected::Unsupported(r)),
            Self::NotApplicable(r) => Err(Collected::NotApplicable(r)),
        }
    }

    /// For collectors whose caller cannot go on without the value: a
    /// missing measurement becomes an error naming the collector.
    pub fn into_result(self, spec: &CollectorSpec) -> crate::error::Result<T> {
        let reason = self.reason_code().unwrap_or("no_value");
        self.into_value()
            .ok_or_else(|| crate::error::AuditError::CdpError(format!("{}: {reason}", spec.id)))
    }

    /// Reclassify a measurement as [`Collected::Empty`] when `is_empty`
    /// holds for it.
    pub fn empty_if(self, is_empty: impl FnOnce(&T) -> bool) -> Self {
        match self {
            Self::Measured(v) if is_empty(&v) => Self::Empty(v),
            other => other,
        }
    }

    fn reason_code(&self) -> Option<&'static str> {
        match self {
            Self::Failed(r) | Self::Unsupported(r) | Self::NotApplicable(r) => Some(r),
            Self::Measured(_) | Self::Empty(_) | Self::Partial { .. } => None,
        }
    }

    fn gaps(&self) -> &[&'static str] {
        match self {
            Self::Partial { gaps, .. } => gaps,
            _ => &[],
        }
    }
}

/// Reason codes shared by the adapters. Collectors may use their own codes
/// for domain-specific failures; all codes are stable and safe for JSON.
pub mod reason {
    pub const NO_RESULT: &str = "collector_returned_no_result";
    pub const INVALID_JSON: &str = "collector_returned_invalid_json";
    pub const NOT_STRUCTURED: &str = "collector_returned_no_structured_result";
    pub const SCHEMA_MISMATCH: &str = "collector_result_schema_mismatch";
    pub const EVALUATION_FAILED: &str = "evaluation_failed";
    pub const CDP_COMMAND_FAILED: &str = "cdp_command_failed";
    pub const HTTP_REQUEST_FAILED: &str = "http_request_failed";
    pub const DNS_LOOKUP_FAILED: &str = "dns_lookup_failed";
}

/// Decode a collection script's result: the script returns a JSON string.
/// Missing, malformed, scalar or schema-violating results are failures,
/// never an empty measurement.
pub fn decode_js<T: serde::de::DeserializeOwned>(
    value: Option<&serde_json::Value>,
) -> Collected<T> {
    let Some(text) = value.and_then(serde_json::Value::as_str) else {
        return Collected::Failed(reason::NO_RESULT);
    };
    let Ok(decoded) = serde_json::from_str::<serde_json::Value>(text) else {
        return Collected::Failed(reason::INVALID_JSON);
    };
    if !decoded.is_object() && !decoded.is_array() {
        return Collected::Failed(reason::NOT_STRUCTURED);
    }
    match serde_json::from_value(decoded) {
        Ok(v) => Collected::Measured(v),
        Err(_) => Collected::Failed(reason::SCHEMA_MISMATCH),
    }
}

/// Evaluate a collection script that returns a JSON string and decode it
/// (see [`decode_js`]). An evaluation error is a failure, not an empty page.
pub async fn evaluate_js<T: serde::de::DeserializeOwned>(
    page: &chromiumoxide::Page,
    script: &str,
) -> Collected<T> {
    match page.evaluate(script).await {
        Ok(result) => decode_js(result.value()),
        Err(_) => Collected::Failed(reason::EVALUATION_FAILED),
    }
}

/// Evaluate a script that returns a plain value (a boolean, a count). A
/// missing, `null` or differently typed value is a failure — never `false`
/// or `0`.
pub async fn evaluate_js_value<T: serde::de::DeserializeOwned>(
    page: &chromiumoxide::Page,
    script: &str,
) -> Collected<T> {
    let Ok(result) = page.evaluate(script).await else {
        return Collected::Failed(reason::EVALUATION_FAILED);
    };
    decode_js_value(result.value())
}

/// See [`evaluate_js_value`].
pub fn decode_js_value<T: serde::de::DeserializeOwned>(
    value: Option<&serde_json::Value>,
) -> Collected<T> {
    match value {
        None | Some(serde_json::Value::Null) => Collected::Failed(reason::NO_RESULT),
        Some(value) => match serde_json::from_value(value.clone()) {
            Ok(v) => Collected::Measured(v),
            Err(_) => Collected::Failed(reason::SCHEMA_MISMATCH),
        },
    }
}

/// An object-shaped collection result, for scripts whose fields are read
/// one by one. An array or scalar is a schema mismatch.
pub type JsObject = serde_json::Map<String, serde_json::Value>;

/// One collector's outcome in the report (`module_runs[].subchecks[]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub collector: String,
    pub module: String,
    pub state: CollectionState,
    pub method: CollectionMethod,
    pub method_version: u32,
    pub scoring: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<String>,
}

/// Records every collection of one page in one viewport. Handed to the
/// modules through [`super::ModuleContext`]; the pipeline folds the records
/// into the module runs afterwards.
#[derive(Debug, Default)]
pub struct CollectionLog {
    records: Mutex<Vec<CollectionRecord>>,
}

impl CollectionLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record `collected` for `spec` and hand it back.
    pub fn record<T>(&self, spec: &'static CollectorSpec, collected: Collected<T>) -> Collected<T> {
        debug_assert!(
            collector(spec.id).is_some_and(|known| std::ptr::eq(known, spec)),
            "collector {} is not listed in COLLECTORS",
            spec.id
        );
        let record = CollectionRecord {
            collector: spec.id.to_string(),
            module: spec.module.to_string(),
            state: collected.state(),
            method: spec.method,
            method_version: spec.method_version,
            scoring: spec.scoring,
            reason_code: collected.reason_code().map(str::to_string),
            gaps: collected.gaps().iter().map(|g| g.to_string()).collect(),
        };
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(record);
        collected
    }

    pub fn into_records(self) -> Vec<CollectionRecord> {
        self.records
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Every collector the audit runs. A collector missing here is a contract
/// violation (see the inventory tests).
pub static COLLECTORS: &[&CollectorSpec] = &[
    &crate::seo::meta::META,
    &crate::seo::headings::HEADINGS,
    &crate::seo::social::SOCIAL,
    &crate::seo::technical::TECHNICAL,
    &crate::seo::technical::COOKIE_INVENTORY,
    &crate::seo::technical::BROKEN_LINKS,
    &crate::seo::technical::SERVICE_WORKERS,
    &crate::seo::technical::WEB_MANIFEST,
    &crate::seo::technical::WWW_REDIRECT,
    &crate::seo::schema::STRUCTURED_DATA,
    &crate::seo::schema_parity::VISIBLE_FACTS,
    &crate::seo::robots::ROBOTS_TXT,
    &crate::seo::robots::SITEMAP_NOINDEX,
    &crate::seo::image_efficiency::IMAGE_EFFICIENCY,
    &crate::seo::page_health::dom::DOM_INSPECTION,
    &crate::seo::page_health::probes::HEADERS,
    &crate::seo::page_health::probes::REDIRECT_CHAIN,
    &crate::seo::page_health::probes::CUSTOM_404,
    &crate::seo::page_health::probes::RESOURCE_CACHE,
    &crate::seo::page_health::probes::WWW_CONSOLIDATION,
    &crate::seo::page_health::probes::URL_CANONICALIZATION,
    &crate::mobile::MOBILE,
    &crate::best_practices::console_errors::CONSOLE,
    &crate::best_practices::vulnerable_libs::LIBRARIES,
    &crate::design_quality::CANDIDATES,
    &crate::dark_mode::STATIC_SUPPORT,
    &crate::dark_mode::PRINT_STYLESHEET,
    &crate::dark_mode::PRINT_LAYOUT,
    &crate::dark_mode::FORCED_COLORS_STYLESHEET,
    &crate::dark_mode::FORCED_COLORS_LAYOUT,
    &crate::dark_mode::CONTRAST,
    &crate::dark_mode::LINK_INDISTINCT,
];

/// Test support: [`assert_js_contract`] for scripts that return a plain
/// value instead of a JSON string.
#[cfg(test)]
pub(crate) fn assert_js_value_contract<T: serde::de::DeserializeOwned>(valid: serde_json::Value) {
    for input in [
        None,
        Some(serde_json::Value::Null),
        Some(serde_json::json!("text")),
    ] {
        assert!(
            decode_js_value::<T>(input.as_ref()).value().is_none(),
            "{input:?} must fail"
        );
    }
    assert!(
        decode_js_value::<T>(Some(&valid)).value().is_some(),
        "valid result must decode"
    );
}

/// Test support: the generic counter-cases of a JavaScript collector. The
/// script result missing, invalid, scalar or of the wrong shape is a
/// failure; `valid` decodes to a measurement.
#[cfg(test)]
pub(crate) fn assert_js_contract<T: serde::de::DeserializeOwned>(valid: serde_json::Value) {
    let as_result = |v: serde_json::Value| serde_json::Value::String(v.to_string());
    let wrong_shape = if valid.is_array() {
        serde_json::json!({"unexpected": true})
    } else {
        serde_json::json!([[true]])
    };
    for (input, expected) in [
        (None, reason::NO_RESULT),
        (Some(serde_json::json!("{not json")), reason::INVALID_JSON),
        (
            Some(as_result(serde_json::json!(42))),
            reason::NOT_STRUCTURED,
        ),
        (Some(as_result(wrong_shape)), reason::SCHEMA_MISMATCH),
    ] {
        let state = decode_js::<T>(input.as_ref());
        assert_eq!(
            state.reason_code(),
            Some(expected),
            "{input:?} must fail with {expected}"
        );
    }
    assert!(
        decode_js::<T>(Some(&as_result(valid))).value().is_some(),
        "valid result must decode"
    );
}

/// Look up a collector by id.
pub fn collector(id: &str) -> Option<&'static CollectorSpec> {
    COLLECTORS.iter().copied().find(|spec| spec.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Deserialize)]
    struct Sample {
        count: u32,
    }

    #[test]
    fn decode_js_never_turns_a_failure_into_a_measurement() {
        for (value, reason) in [
            (None, reason::NO_RESULT),
            (Some(serde_json::Value::Null), reason::NO_RESULT),
            (Some(serde_json::json!("not json")), reason::INVALID_JSON),
            (Some(serde_json::json!("null")), reason::NOT_STRUCTURED),
            (
                Some(serde_json::json!("{\"count\":\"x\"}")),
                reason::SCHEMA_MISMATCH,
            ),
        ] {
            assert_eq!(
                decode_js::<Sample>(value.as_ref()),
                Collected::Failed(reason),
                "{value:?}"
            );
        }
        assert_eq!(
            decode_js::<Sample>(Some(&serde_json::json!("{\"count\":2}"))),
            Collected::Measured(Sample { count: 2 })
        );
    }

    #[test]
    fn js_object_rejects_arrays() {
        assert_eq!(
            decode_js::<JsObject>(Some(&serde_json::json!("[]"))),
            Collected::Failed(reason::SCHEMA_MISMATCH)
        );
        assert!(decode_js::<JsObject>(Some(&serde_json::json!("{}")))
            .value()
            .is_some());
    }

    #[test]
    fn empty_measurement_is_not_a_failure() {
        let collected = decode_js::<Vec<u32>>(Some(&serde_json::json!("[]")))
            .empty_if(|v: &Vec<u32>| v.is_empty());
        assert_eq!(collected.state(), CollectionState::Empty);
        assert_eq!(collected.value(), Some(&Vec::new()));
    }

    #[test]
    fn log_records_state_reason_and_gaps() {
        let spec = &crate::seo::page_health::probes::URL_CANONICALIZATION;
        let log = CollectionLog::new();
        log.record(spec, Collected::<()>::Failed(reason::HTTP_REQUEST_FAILED));
        log.record(
            spec,
            Collected::Partial {
                value: (),
                gaps: vec!["second_request"],
            },
        );
        let records = log.into_records();
        assert_eq!(records[0].state, CollectionState::Failed);
        assert_eq!(
            records[0].reason_code.as_deref(),
            Some(reason::HTTP_REQUEST_FAILED)
        );
        assert_eq!(records[1].gaps, vec!["second_request".to_string()]);
    }

    /// The counter-cases of every collector, by id. A collector in
    /// [`COLLECTORS`] without an entry here fails
    /// `every_collector_has_counter_cases`.
    fn counter_cases() -> Vec<(&'static str, fn())> {
        use crate::seo::page_health::probes::counter_cases as probes;
        use crate::seo::technical::counter_cases as technical;
        vec![
            ("seo.meta", crate::seo::meta::counter_cases),
            ("seo.headings", crate::seo::headings::counter_cases),
            ("seo.social", crate::seo::social::counter_cases),
            ("seo.technical", crate::seo::technical::counter_cases),
            ("seo.cookie_inventory", technical::browser_state),
            ("seo.broken_links", technical::broken_links),
            ("seo.service_worker_registrations", technical::browser_state),
            ("seo.web_manifest", technical::web_manifest),
            ("seo.www_redirect", technical::www_redirect),
            ("seo.structured_data", crate::seo::schema::counter_cases),
            (
                "seo.schema_visible_facts",
                crate::seo::schema_parity::counter_cases,
            ),
            ("seo.robots_txt", crate::seo::robots::counter_cases),
            ("seo.sitemap_noindex", crate::seo::robots::counter_cases),
            (
                "seo.image_efficiency",
                crate::seo::image_efficiency::counter_cases,
            ),
            (
                "seo.page_health.dom",
                crate::seo::page_health::dom::counter_cases,
            ),
            ("seo.page_health.headers", probes::transport_failure),
            ("seo.page_health.redirect_chain", probes::transport_failure),
            ("seo.page_health.custom_404", probes::transport_failure),
            ("seo.page_health.resource_cache", probes::valid_empty),
            ("seo.page_health.www_consolidation", probes::not_applicable),
            (
                "seo.page_health.url_canonicalization",
                probes::partial_result,
            ),
            ("mobile.friendliness", crate::mobile::counter_cases),
            (
                "best_practices.console",
                crate::best_practices::console_errors::counter_cases,
            ),
            (
                "best_practices.libraries",
                crate::best_practices::vulnerable_libs::counter_cases,
            ),
            (
                "design_quality.candidates",
                crate::design_quality::counter_cases,
            ),
            ("dark_mode.static_support", crate::dark_mode::counter_cases),
            (
                "dark_mode.print_stylesheet",
                crate::dark_mode::counter_cases,
            ),
            ("dark_mode.print_layout", crate::dark_mode::counter_cases),
            (
                "dark_mode.forced_colors_stylesheet",
                crate::dark_mode::counter_cases,
            ),
            (
                "dark_mode.forced_colors_layout",
                crate::dark_mode::counter_cases,
            ),
            ("dark_mode.contrast", crate::dark_mode::counter_cases),
            ("dark_mode.link_indistinct", crate::dark_mode::counter_cases),
        ]
    }

    #[test]
    fn every_collector_has_counter_cases() {
        let cases = counter_cases();
        for spec in COLLECTORS {
            let Some((_, case)) = cases.iter().find(|(id, _)| *id == spec.id) else {
                panic!("collector {} has no counter-cases", spec.id);
            };
            case();
        }
        for (id, _) in &cases {
            assert!(
                collector(id).is_some(),
                "counter-cases for unknown collector {id}"
            );
        }
    }

    #[test]
    fn inventory_ids_are_unique_and_scoped_to_their_module() {
        let mut seen = std::collections::HashSet::new();
        for spec in COLLECTORS {
            assert!(seen.insert(spec.id), "duplicate collector id {}", spec.id);
            assert!(
                spec.id.starts_with(&format!("{}.", spec.module)),
                "{} must start with its module {}",
                spec.id,
                spec.module
            );
        }
    }
}
