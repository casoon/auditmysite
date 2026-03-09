//! Doctor command - system diagnostics
//!
//! Checks browser availability, permissions, disk space, and configuration.

use colored::Colorize;

use crate::browser::{detect_all_browsers, resolve_browser, BrowserResolveOptions};

/// Run all diagnostic checks and print results
pub fn run_doctor() {
    println!("{}", "Running diagnostics...".cyan().bold());
    println!();

    let mut has_errors = false;

    // 1. Browser detection
    let browsers = detect_all_browsers();
    if browsers.is_empty() {
        print_check("Browser", CheckStatus::Error, "No compatible browser found");
        println!("         Install one:");
        println!("           brew install --cask google-chrome");
        println!("           auditmysite browser install");
        has_errors = true;
    } else {
        for browser in &browsers {
            print_check(
                &format!("{}", browser.kind),
                CheckStatus::Ok,
                &format!(
                    "v{} at {}",
                    browser.version.as_deref().unwrap_or("unknown"),
                    browser.path.display()
                ),
            );
        }
    }

    // 2. Active browser resolution
    let opts = BrowserResolveOptions::default();
    match resolve_browser(&opts) {
        Ok(resolved) => {
            print_check(
                "Active browser",
                CheckStatus::Ok,
                &format!(
                    "{} v{} ({})",
                    resolved.browser.kind,
                    resolved.browser.version.as_deref().unwrap_or("unknown"),
                    resolved.browser.source,
                ),
            );
        }
        Err(_) => {
            print_check("Active browser", CheckStatus::Error, "No browser can be resolved");
            has_errors = true;
        }
    }

    // 3. Managed installs
    let managed_dir = dirs::home_dir().map(|h| h.join(".auditmysite").join("browsers"));
    if let Some(ref dir) = managed_dir {
        if dir.exists() {
            let cft = dir.join("chrome-for-testing");
            let hs = dir.join("headless-shell");

            if cft.exists() {
                let version = std::fs::read_to_string(cft.join("version.txt"))
                    .unwrap_or_else(|_| "unknown".to_string());
                print_check("Managed: Chrome for Testing", CheckStatus::Ok, version.trim());
            }
            if hs.exists() {
                let version = std::fs::read_to_string(hs.join("version.txt"))
                    .unwrap_or_else(|_| "unknown".to_string());
                print_check("Managed: Headless Shell", CheckStatus::Ok, version.trim());
            }
        }
    }

    // Legacy install check
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
        println!(
            "{} All checks passed.",
            "OK:".green().bold()
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
