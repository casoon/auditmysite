//! Analyses for report tests (#872): a page with trackers, a clean one, a
//! partly measured one and one that could not be measured.

use super::findings::*;
use crate::audit::collection::CollectionState;

fn provider(id: &str, name: &str, category: &str, jurisdiction: &str) -> ProviderRef {
    ProviderRef {
        id: id.into(),
        name: name.into(),
        category: category.into(),
        jurisdiction: jurisdiction.into(),
    }
}

fn evidence(source: EvidenceSource, value: &str) -> Evidence {
    Evidence {
        source,
        value: value.into(),
    }
}

fn finding(
    kind: PrivacyFindingKind,
    confidence: ConfidenceClass,
    provider: Option<ProviderRef>,
    document: Option<PolicyDocument>,
    evidence: Vec<Evidence>,
) -> PrivacyFinding {
    let name = provider.as_ref().map(|p| p.name.as_str()).unwrap_or("");
    PrivacyFinding {
        kind,
        confidence,
        message: finding_text(kind, name, document, true),
        provider,
        document,
        evidence,
    }
}

fn scope() -> PrivacyScope {
    PrivacyScope {
        phase: "A".into(),
        interaction: false,
        fresh_context: true,
        privacy_signals_sent: false,
        settle_ms: 2000,
    }
}

fn measurement(state: CollectionState) -> MeasurementState {
    MeasurementState {
        state,
        gaps: Vec::new(),
        reason: None,
    }
}

fn checks(outcomes: [(CheckOutcome, Option<&str>); 6]) -> Vec<PrivacyCheckResult> {
    PrivacyCheck::ALL
        .iter()
        .zip(outcomes)
        .map(|(&check, (outcome, reason))| PrivacyCheckResult {
            check,
            outcome,
            reason: reason.map(str::to_string),
        })
        .collect()
}

/// Analytics + fonts contacted, a cookie set, no consent platform.
pub(crate) fn tracked() -> PrivacyAnalysis {
    let ga = provider("google-analytics", "Google Analytics", "analytics", "us");
    let fonts = provider("google-fonts", "Google Fonts", "fonts", "us");
    use CheckOutcome::Found;
    PrivacyAnalysis {
        ruleset_version: "test-1".into(),
        scope: scope(),
        measurement: measurement(CollectionState::Measured),
        checks: checks([
            (Found, None),
            (Found, None),
            (Found, None),
            (Found, None),
            (Found, None),
            (Found, None),
        ]),
        findings: vec![
            finding(
                PrivacyFindingKind::TrackingRequestBeforeConsent,
                ConfidenceClass::Evidence,
                Some(ga.clone()),
                None,
                vec![evidence(
                    EvidenceSource::Request,
                    "https://www.google-analytics.com/g/collect",
                )],
            ),
            finding(
                PrivacyFindingKind::NonEssentialStorageBeforeConsent,
                ConfidenceClass::Evidence,
                Some(ga.clone()),
                None,
                vec![
                    evidence(EvidenceSource::Cookie, "_ga"),
                    evidence(EvidenceSource::Cookie, "_ga_ABC123"),
                ],
            ),
            finding(
                PrivacyFindingKind::VisitorIpTransfer,
                ConfidenceClass::Evidence,
                Some(fonts.clone()),
                None,
                vec![evidence(
                    EvidenceSource::Request,
                    "https://fonts.gstatic.com/s/inter/v12/font.woff2",
                )],
            ),
            finding(
                PrivacyFindingKind::ThirdCountryProvider,
                ConfidenceClass::Indication,
                None,
                None,
                vec![
                    evidence(EvidenceSource::Provider, "google-analytics (us)"),
                    evidence(EvidenceSource::Provider, "google-fonts (us)"),
                ],
            ),
            finding(
                PrivacyFindingKind::NoConsentPlatformDetected,
                ConfidenceClass::Indication,
                None,
                None,
                Vec::new(),
            ),
            finding(
                PrivacyFindingKind::PolicyLinkMissing,
                ConfidenceClass::Indication,
                None,
                Some(PolicyDocument::Imprint),
                Vec::new(),
            ),
        ],
        contacts: vec![
            ThirdPartyContact {
                provider: fonts,
                hosts: vec!["fonts.gstatic.com".into()],
                request_count: 2,
            },
            ThirdPartyContact {
                provider: ga,
                hosts: vec!["www.google-analytics.com".into()],
                request_count: 3,
            },
        ],
        unclassified_hosts: vec!["cdn.unknown.example".into()],
    }
}

/// Measured, nothing found, no third-party request.
pub(crate) fn clean() -> PrivacyAnalysis {
    use CheckOutcome::{NotApplicable, NotFound};
    PrivacyAnalysis {
        ruleset_version: "test-1".into(),
        scope: scope(),
        measurement: measurement(CollectionState::Empty),
        checks: checks([
            (NotApplicable, Some("no_third_party_requests")),
            (NotApplicable, Some("nothing_stored")),
            (NotApplicable, Some("no_third_party_requests")),
            (NotApplicable, Some("no_classified_provider")),
            (NotApplicable, Some("no_tracking_requests")),
            (NotFound, None),
        ]),
        findings: Vec::new(),
        contacts: Vec::new(),
        unclassified_hosts: Vec::new(),
    }
}

/// The network never went quiet: request checks read "not measured".
#[cfg(feature = "pdf")]
pub(crate) fn partial() -> PrivacyAnalysis {
    use CheckOutcome::{NotFound, NotMeasured};
    PrivacyAnalysis {
        ruleset_version: "test-1".into(),
        scope: scope(),
        measurement: MeasurementState {
            state: CollectionState::Partial,
            gaps: vec!["network_not_idle".into()],
            reason: None,
        },
        checks: checks([
            (NotMeasured, Some("network_not_idle")),
            (NotFound, None),
            (NotMeasured, Some("network_not_idle")),
            (NotMeasured, Some("network_not_idle")),
            (NotMeasured, Some("network_not_idle")),
            (NotFound, None),
        ]),
        findings: Vec::new(),
        contacts: Vec::new(),
        unclassified_hosts: Vec::new(),
    }
}

/// The capture failed: every check is "not measured".
pub(crate) fn not_measured() -> PrivacyAnalysis {
    PrivacyAnalysis {
        ruleset_version: "test-1".into(),
        scope: scope(),
        measurement: MeasurementState {
            state: CollectionState::Failed,
            gaps: Vec::new(),
            reason: Some("cdp_command_failed".into()),
        },
        checks: PrivacyCheck::ALL
            .iter()
            .map(|&check| PrivacyCheckResult {
                check,
                outcome: CheckOutcome::NotMeasured,
                reason: Some("cdp_command_failed".into()),
            })
            .collect(),
        findings: Vec::new(),
        contacts: Vec::new(),
        unclassified_hosts: Vec::new(),
    }
}
