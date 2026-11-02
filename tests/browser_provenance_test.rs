//! Browser provenance of the measurement profile (#787).
//!
//! A measurement names the browser that took it -- product, version, origin,
//! platform, headless mode and failover -- without paths. Missing or
//! contradictory provenance breaks the contract; a batch that mixes browsers
//! says so instead of averaging them away. No Chrome needed: the live record
//! is checked in `session_policy_test` and `state_changes_test` (`--ignored`).

// `normalize` is the deprecated 1.x compatibility API; the batch needs the model only.
#![allow(deprecated)]

use auditmysite::audit::{
    normalize_checked, profile_differences, validate_batch, AuditReport, BatchReport,
    MEASUREMENT_PROFILE_VERSION,
};
use auditmysite::browser::{
    BrowserOrigin, BrowserProduct, BrowserProvenance, HeadlessModeKind, PlatformArch, PlatformOs,
};
use auditmysite::cli::WcagLevel;
use auditmysite::wcag::WcagResults;

fn provenance() -> BrowserProvenance {
    BrowserProvenance {
        product: BrowserProduct::Chrome,
        version: Some("131.0.6778.85".into()),
        origin: BrowserOrigin::System,
        os: PlatformOs::Macos,
        arch: PlatformArch::Aarch64,
        headless_mode: HeadlessModeKind::New,
        failover: false,
        discarded_candidates: 0,
    }
}

fn report_with(browser: Option<BrowserProvenance>) -> AuditReport {
    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        10,
    );
    let execution = &mut report.accessibility.execution;
    execution.scope.measurement_profile.version = MEASUREMENT_PROFILE_VERSION;
    execution.scope.measurement_profile.read_only = true;
    if let Some(browser) = &browser {
        execution.environment.headless = browser.headless_mode.is_headless();
        execution.environment.browser_version = browser.version.clone();
    }
    execution.scope.measurement_profile.browser = browser;
    report
}

fn codes(report: &AuditReport) -> Vec<&'static str> {
    match normalize_checked(report) {
        Ok(_) => Vec::new(),
        Err(error) => error.violations.iter().map(|v| v.code).collect(),
    }
}

#[test]
fn a_complete_consistent_provenance_passes() {
    assert_eq!(codes(&report_with(Some(provenance()))), Vec::<&str>::new());
}

#[test]
fn a_measurement_without_provenance_breaks_the_contract() {
    assert_eq!(codes(&report_with(None)), ["browser_provenance"]);
}

#[test]
fn reports_from_before_the_field_stay_readable() {
    let mut old = report_with(None);
    old.accessibility
        .execution
        .scope
        .measurement_profile
        .version = 2;
    assert!(codes(&old).is_empty());
}

#[test]
fn a_missing_version_is_a_violation() {
    let mut browser = provenance();
    browser.version = None;
    assert_eq!(codes(&report_with(Some(browser))), ["browser_provenance"]);
}

#[test]
fn contradictory_provenance_is_a_violation() {
    type Mutation = Box<dyn Fn(&mut BrowserProvenance)>;
    let cases: [(&str, Mutation); 5] = [
        (
            "managed product from a system install",
            Box::new(|b| b.product = BrowserProduct::ChromeForTesting),
        ),
        (
            "system chrome claiming a managed origin",
            Box::new(|b| b.origin = BrowserOrigin::Managed),
        ),
        (
            "failover without a discarded candidate",
            Box::new(|b| b.failover = true),
        ),
        (
            "discarded candidates without failover",
            Box::new(|b| b.discarded_candidates = 2),
        ),
        (
            "Chrome on Windows",
            Box::new(|b| {
                b.os = PlatformOs::Windows;
                b.arch = PlatformArch::X86_64;
            }),
        ),
    ];
    for (name, mutate) in cases {
        let mut browser = provenance();
        mutate(&mut browser);
        assert_eq!(
            codes(&report_with(Some(browser))),
            ["browser_provenance"],
            "{name}"
        );
    }
}

#[test]
fn macos_intel_is_outside_the_platform_contract() {
    let mut browser = provenance();
    browser.arch = PlatformArch::X86_64;
    assert_eq!(codes(&report_with(Some(browser))), ["browser_provenance"]);
}

#[test]
fn a_failover_that_is_recorded_consistently_is_valid_and_visible() {
    let mut browser = provenance();
    browser.record_discarded(2);
    let report = report_with(Some(browser));
    assert!(codes(&report).is_empty());
    let json =
        serde_json::to_value(&report.accessibility.execution.scope.measurement_profile).unwrap();
    assert_eq!(json["browser"]["failover"], true);
    assert_eq!(json["browser"]["discarded_candidates"], 2);
}

#[test]
fn the_environment_cannot_contradict_the_provenance() {
    let mut report = report_with(Some(provenance()));
    report.accessibility.execution.environment.headless = false;
    assert_eq!(codes(&report), ["browser_provenance"]);
    let mut report = report_with(Some(provenance()));
    report.accessibility.execution.environment.browser_version = Some("99.0.0.0".into());
    assert_eq!(codes(&report), ["browser_provenance"]);
}

#[test]
fn the_json_carries_categories_and_no_local_paths() {
    let report = report_with(Some(provenance()));
    let json = serde_json::to_string(&report.accessibility.execution.scope).unwrap();
    for needle in ["/Users", "/home", "C:\\", "Applications", "--headless"] {
        assert!(!json.contains(needle), "{needle} in {json}");
    }
    assert!(json.contains("\"product\":\"chrome\"") && json.contains("\"origin\":\"system\""));
}

#[test]
fn a_replay_of_another_browser_is_another_measurement() {
    let a = report_with(Some(provenance()));
    let mut other = provenance();
    other.version = Some("132.0.1.1".into());
    let b = report_with(Some(other));
    let differences = profile_differences(
        &a.accessibility.execution.scope.measurement_profile,
        &b.accessibility.execution.scope.measurement_profile,
    );
    assert_eq!(differences.len(), 1);
    assert_eq!(differences[0].field, "browser");
    // The same browser found another way is the same measurement.
    let mut found_elsewhere = provenance();
    found_elsewhere.origin = BrowserOrigin::Path;
    found_elsewhere.record_discarded(1);
    let c = report_with(Some(found_elsewhere));
    assert!(profile_differences(
        &a.accessibility.execution.scope.measurement_profile,
        &c.accessibility.execution.scope.measurement_profile,
    )
    .is_empty());
}

#[test]
fn a_batch_of_one_browser_lists_it_once_with_all_pages() {
    let batch = BatchReport::from_reports(
        vec![
            report_with(Some(provenance())),
            report_with(Some(provenance())),
        ],
        Vec::new(),
        10,
    );
    let browsers = &batch.summary.measurement_browsers;
    assert_eq!(browsers.len(), 1);
    assert_eq!(browsers[0].pages, 2);
    assert!(validate_batch(&batch).is_ok());
}

#[test]
fn a_batch_mixing_browsers_says_so_instead_of_averaging() {
    let mut other = provenance();
    other.product = BrowserProduct::Chromium;
    other.version = Some("130.0.6723.58".into());
    let batch = BatchReport::from_reports(
        vec![
            report_with(Some(provenance())),
            report_with(Some(other.clone())),
            report_with(Some(provenance())),
        ],
        Vec::new(),
        10,
    );
    let browsers = &batch.summary.measurement_browsers;
    assert_eq!(browsers.len(), 2, "{browsers:?}");
    let pages_of = |wanted: &BrowserProvenance| {
        browsers
            .iter()
            .find(|e| e.browser.as_ref() == Some(wanted))
            .map(|e| e.pages)
    };
    assert_eq!(pages_of(&provenance()), Some(2));
    assert_eq!(pages_of(&other), Some(1));
    assert!(validate_batch(&batch).is_ok());

    // A stored summary that hides the mix is refused.
    let mut hidden = batch.clone();
    hidden.summary.measurement_browsers.truncate(1);
    let error = validate_batch(&hidden).err().unwrap();
    assert!(error
        .violations
        .iter()
        .any(|v| v.code == "batch_aggregation" && v.path == "summary.measurement_browsers"));
}

#[test]
fn pages_without_a_recorded_browser_are_counted_not_dropped() {
    let mut old = report_with(None);
    old.accessibility
        .execution
        .scope
        .measurement_profile
        .version = 2;
    let batch =
        BatchReport::from_reports(vec![report_with(Some(provenance())), old], Vec::new(), 10);
    let browsers = &batch.summary.measurement_browsers;
    assert_eq!(browsers.len(), 2);
    assert!(browsers[0].browser.is_some() && browsers[1].browser.is_none());
    assert_eq!(browsers[1].pages, 1);
}
