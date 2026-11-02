//! Doctor command - system diagnostics
//!
//! Checks browser availability, permissions, disk space, and configuration.

use colored::Colorize;

use crate::browser::{
    diagnose_browsers, BrowserDiagnosis, BrowserResolveOptions, CandidateReport, CandidateVerdict,
};

/// Run all diagnostic checks and print results
pub async fn run_doctor(no_sandbox: bool) {
    println!("{}", "Running diagnostics...".cyan().bold());
    println!();

    let mut has_errors = false;

    // 1. Browser detection: every candidate with the reason it is or is not usable
    let diagnosis = diagnose_browsers(&BrowserResolveOptions::default(), no_sandbox).await;
    if let Some(problem) = &diagnosis.problem {
        print_check("Platform", CheckStatus::Error, problem);
        has_errors = true;
    } else {
        print_check("Platform", CheckStatus::Ok, &diagnosis.platform.describe());
        if diagnosis.platform.is_edge_only() {
            print_check(
                "Browser policy",
                CheckStatus::Ok,
                "Microsoft Edge only (Chrome and managed browsers are not supported on Windows)",
            );
        }
    }
    if diagnosis.problem.is_none() && diagnosis.candidates.is_empty() {
        print_check("Browser", CheckStatus::Error, "No compatible browser found");
        println!("         Install one:");
        println!("           brew install --cask google-chrome   (macOS)");
        println!("           sudo apt install chromium           (Linux)");
        println!("           Microsoft Edge                      (Windows)");
        println!(
            "           auditmysite browser install         (macOS ARM64, Linux x86_64/ARM64)"
        );
        has_errors = true;
    }
    let selected = diagnosis.selected().map(|c| c.browser.path.clone());
    for candidate in &diagnosis.candidates {
        let is_selected = selected.as_ref() == Some(&candidate.browser.path);
        let (status, detail) = describe_candidate(candidate, is_selected);
        print_check(&format!("{}", candidate.browser.kind), status, &detail);
    }

    // 2. Active browser resolution
    match diagnosis.selected() {
        Some(active) => {
            print_check(
                "Active browser",
                CheckStatus::Ok,
                &format!(
                    "{} v{} ({})",
                    active.browser.kind,
                    active.browser.version.as_deref().unwrap_or("unknown"),
                    active.browser.source,
                ),
            );
            if crate::browser::resolver::stalls_on_keyboard_journeys(&active.browser) {
                print_check(
                    "Batch concurrency",
                    CheckStatus::Warning,
                    "full browser on macOS: batches with journeys audit one page at a time \
                     (parallel keyboard journeys can stall it); --concurrency overrides",
                );
            }
        }
        None => {
            print_check(
                "Active browser",
                CheckStatus::Error,
                "No browser can be resolved",
            );
            has_errors = true;
        }
    }

    // 3. Legacy install check
    let managed_dir = dirs::home_dir().map(|h| h.join(".auditmysite").join("browsers"));
    let legacy_dir = dirs::home_dir().map(|h| h.join(".auditmysite").join("chromium"));
    if let Some(ref dir) = legacy_dir {
        if dir.exists() {
            print_check(
                "Legacy install",
                CheckStatus::Warning,
                "~/.auditmysite/chromium/ exists (migrate with: auditmysite browser install && rm -rf ~/.auditmysite/chromium/)",
            );
        }
    }

    // 4. Config file
    let config_path = std::path::Path::new("auditmysite.toml");
    if config_path.exists() {
        print_check("Config", CheckStatus::Ok, "auditmysite.toml found");
    } else {
        print_check("Config", CheckStatus::Ok, "No config file (using defaults)");
    }

    // 5. Disk space
    if let Some(ref dir) = managed_dir {
        if let Ok(metadata) = dir_size(dir) {
            print_check(
                "Disk usage",
                CheckStatus::Ok,
                &format!("~/.auditmysite/browsers/: {} MB", metadata / 1_000_000),
            );
        }
    }

    // Summary
    println!();
    if has_errors {
        println!(
            "{} Some checks failed. See above for details.",
            "WARN:".yellow().bold()
        );
    } else {
        println!("{} All checks passed.", "OK:".green().bold());
    }
}

/// Status and one-line explanation for a candidate: selected, alternative, or why not.
fn describe_candidate(candidate: &CandidateReport, selected: bool) -> (CheckStatus, String) {
    let browser = &candidate.browser;
    let what = format!(
        "{} at {} [{}]",
        browser
            .version
            .as_deref()
            .map_or("version unknown".to_string(), |v| format!("v{v}")),
        browser.path.display(),
        browser.source
    );
    match &candidate.verdict {
        CandidateVerdict::Viable => {
            let role = if selected {
                "selected"
            } else {
                "usable alternative (lower priority, used if the selected one fails to start)"
            };
            let cdp = candidate
                .cdp_product
                .as_deref()
                .map(|p| format!(", headless CDP ok ({p})"))
                .unwrap_or_default();
            (CheckStatus::Ok, format!("{what} - {role}{cdp}"))
        }
        CandidateVerdict::Rejected { kind, detail } => (
            CheckStatus::Warning,
            format!(
                "{what} - not usable: {} ({})",
                shorten(detail, 220),
                kind.as_str()
            ),
        ),
    }
}

/// First `max` characters of a (possibly multi-line) reason, on one line.
fn shorten(text: &str, max: usize) -> String {
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.chars().count() <= max {
        return one_line;
    }
    let cut: String = one_line.chars().take(max).collect();
    format!("{cut}... (full text: run with RUST_LOG=debug)")
}

/// Print selection, alternatives and the reason for every candidate (`browser detect`).
pub fn print_browser_diagnosis(diagnosis: &BrowserDiagnosis) {
    println!("  Platform: {}", diagnosis.platform.describe());
    if let Some(problem) = &diagnosis.problem {
        println!("  {} {}", "✗".red(), problem);
        return;
    }
    if diagnosis.platform.is_edge_only() {
        println!("  Policy:   Microsoft Edge only; Chrome and managed browsers are not supported");
    }
    println!();

    if diagnosis.candidates.is_empty() {
        println!(
            "  No browsers found ({} locations searched).",
            diagnosis.searched
        );
        println!();
        println!("  Installation:");
        println!("    macOS:   brew install --cask google-chrome");
        println!("    Linux:   sudo apt install chromium");
        println!("    Windows: Microsoft Edge");
        println!("    managed: auditmysite browser install   (macOS ARM64, Linux x86_64/ARM64)");
        return;
    }

    let selected = diagnosis.selected().map(|c| c.browser.path.clone());
    println!("  Candidates in priority order:");
    for (i, candidate) in diagnosis.candidates.iter().enumerate() {
        let is_selected = selected.as_ref() == Some(&candidate.browser.path);
        let (status, detail) = describe_candidate(candidate, is_selected);
        let icon = match status {
            CheckStatus::Ok if is_selected => "→".cyan(),
            CheckStatus::Ok => "✓".green(),
            _ => "✗".red(),
        };
        println!(
            "  {} {}. {:<22} {}",
            icon,
            i + 1,
            candidate.browser.kind.display_name(),
            detail
        );
    }
    println!();
    println!("  {} locations searched.", diagnosis.searched);
    if diagnosis.probed {
        println!(
            "  Viability: headless launch and DevTools connection were tried for each candidate."
        );
    }
}

enum CheckStatus {
    Ok,
    Warning,
    Error,
}

fn print_check(name: &str, status: CheckStatus, detail: &str) {
    let icon = match status {
        CheckStatus::Ok => "✓".green(),
        CheckStatus::Warning => "⚠".yellow(),
        CheckStatus::Error => "✗".red(),
    };
    println!("  {} {:<30} {}", icon, name, detail);
}

fn dir_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut total = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path)?;
            } else {
                total += entry.metadata()?.len();
            }
        }
    }
    Ok(total)
}
