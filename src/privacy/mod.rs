//! Privacy indicators (tracking issue #875).
//!
//! Collection (#870): what a first visit to a page leaves behind in a clean
//! browser profile — requests, cookies, storage keys. Analysis (#871):
//! [`analyze_first_visit`] classifies that capture with `web-checks`
//! (casoon/barrierlab#91) and states technical findings, never a verdict.
//! The report (#872) builds on [`PrivacyAnalysis`].

pub mod analysis;
pub mod first_visit;

pub use analysis::{
    analyze_first_visit, privacy_finding_text, Confidence, PrivacyAnalysis, PrivacyFinding,
    PrivacyFindingKind, ProviderContact, ProviderStorage,
};

pub use first_visit::{
    collect_first_visit, CaptureProfile, CapturedCookie, CapturedRequest, CapturedStorageKey,
    FirstVisitCapture, FirstVisitOptions, StorageArea, FIRST_VISIT,
};
