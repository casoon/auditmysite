//! Privacy indicators (tracking issue #875).
//!
//! Collection (#870): what a first visit to a page leaves behind in a clean
//! browser profile - requests, cookies, storage keys, consent and legal-page
//! traces of the document. Findings (#871): [`analyze_privacy`] classifies
//! that capture with `web-checks` (barrierlab#91) and reports technical
//! observations with a confidence class - never a compliance score or a
//! legal verdict. The report (#872) builds on [`PrivacyAnalysis`].

pub mod aggregate;
pub mod findings;
pub mod first_visit;
#[cfg(test)]
pub(crate) mod fixtures;
pub mod module;

pub use aggregate::{aggregate_privacy, PrivacyBatchAggregate};
pub use module::PrivacyModule;

pub use first_visit::{
    collect_first_visit, CaptureProfile, CapturedCookie, CapturedLink, CapturedRequest,
    CapturedStorageKey, FirstVisitCapture, FirstVisitOptions, PageSignals, StorageArea,
    FIRST_VISIT,
};

pub use findings::{
    analyze_privacy, finding_text, CheckOutcome, ConfidenceClass, Evidence, EvidenceSource,
    MeasurementState, PolicyDocument, PrivacyAnalysis, PrivacyCheck, PrivacyCheckResult,
    PrivacyFinding, PrivacyFindingKind, PrivacyScope, ProviderRef, ThirdPartyContact,
};
