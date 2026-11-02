//! #854: browser discovery against a real Chrome -- the headless CDP probe
//! (launch, connection, cleanup) and the diagnosis `browser detect` and
//! `doctor` print. Needs Chrome (or Edge): run with `--ignored`.

use auditmysite::browser::{
    diagnose_browsers, probe_cdp, resolve_browser, BrowserResolveOptions, CandidateVerdict,
};
use std::time::Duration;

fn profile_prefix() -> String {
    format!("auditmysite-chrome-{}-", std::process::id())
}

fn profile_dirs_of_this_process() -> usize {
    let prefix = profile_prefix();
    std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .count()
}

/// Command lines of all running processes: `ps` on Unix, CIM on Windows.
fn process_command_lines() -> Vec<String> {
    #[cfg(unix)]
    let out = std::process::Command::new("ps")
        .args(["-axo", "command"])
        .output()
        .unwrap();
    #[cfg(windows)]
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_Process | ForEach-Object { $_.CommandLine }",
        ])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

fn processes_using_our_profiles() -> usize {
    let prefix = profile_prefix();
    process_command_lines()
        .iter()
        .filter(|l| l.contains(&prefix))
        .count()
}

/// CI runners (Ubuntu 23.10+) have no usable Chrome sandbox; the other browser
/// suites run with `--no-sandbox` there too.
fn in_ci() -> bool {
    std::env::var_os("CI").is_some()
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn probe_launches_connects_and_cleans_up() {
    let resolved = resolve_browser(&BrowserResolveOptions::default()).expect("a browser");
    let before = profile_dirs_of_this_process();

    let product = probe_cdp(&resolved.browser.path, in_ci(), Duration::from_secs(30))
        .await
        .expect("a real browser must pass the probe");

    assert!(
        product.contains("Chrome") || product.contains("Edg"),
        "unexpected CDP product: {product}"
    );
    assert_eq!(profile_dirs_of_this_process(), before, "profile removed");
    assert_eq!(processes_using_our_profiles(), 0, "browser process ended");
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn diagnosis_selects_a_probed_browser_and_explains_the_rest() {
    let diagnosis = diagnose_browsers(&BrowserResolveOptions::default(), in_ci()).await;

    assert!(diagnosis.problem.is_none());
    assert!(diagnosis.probed);
    let selected = diagnosis.selected().expect("a usable browser");
    assert!(
        selected.cdp_product.is_some(),
        "selection rests on a CDP probe"
    );
    // Every candidate has a verdict; a rejected one says why.
    for candidate in &diagnosis.candidates {
        if let CandidateVerdict::Rejected { detail, .. } = &candidate.verdict {
            assert!(!detail.is_empty());
        }
    }
    assert_eq!(profile_dirs_of_this_process(), 0);
}
