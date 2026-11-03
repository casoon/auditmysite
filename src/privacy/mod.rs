//! Privacy indicators (tracking issue #875).
//!
//! Collection only so far (#870): what a first visit to a page leaves behind
//! in a clean browser profile — requests, cookies, storage keys. Findings
//! (#871) and the report (#872) build on [`FirstVisitCapture`]; the
//! classification of hosts and cookie names (provider, category,
//! jurisdiction) belongs to `web-checks` in barrierlab (casoon/barrierlab#91)
//! and is not done here.

pub mod first_visit;

pub use first_visit::{
    collect_first_visit, CaptureProfile, CapturedCookie, CapturedRequest, CapturedStorageKey,
    FirstVisitCapture, FirstVisitOptions, StorageArea, FIRST_VISIT,
};
