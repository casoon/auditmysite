//! CLI subcommand handlers.
//!
//! Handles the browser subcommands, the plan dry-run command, and the
//! legacy --detect-chrome flag. No mode-runner or audit logic here.

use colored::Colorize;

use auditmysite::browser::{
    detect_all_browsers, find_chrome, resolve_browser, BrowserInstaller, BrowserResolveOptions,
    InstallTarget,
};
use auditmysite::cli::{Args, BrowserAction, Command};
use auditmysite::error::{AuditError, Result};

use crate::plan::{print_banner, print_batch_audit_plan, print_single_audit_plan};

pub async fn handle_command(command: &Command, args: &Args) -> Result<f64> {
    match command {
        Command::Browser { action } => handle_browser_command(action).await,
        Command::Doctor => {
            auditmysite::cli::doctor::run_doctor();
            Ok(0.0)
        }
        Command::Plan { url } => run_plan_command(args, url.as_deref()),
    }
}

fn run_plan_command(args: &Args, url: Option<&str>) -> Result<f64> {
    let mut effective = args.clone();
    if let Some(u) = url {
        effective.url = Some(u.to_string());
    }

    if effective.url.is_none() && effective.sitemap.is_none() && effective.url_file.is_none() {
        return Err(AuditError::ConfigError(
            "auditmysite plan requires a URL or --sitemap/--url-file.".to_string(),
        ));
    }

    if !effective.quiet {
        print_banner();
    }

    if let Some(ref sitemap) = effective.sitemap {
        let url_count = effective.max_pages.max(1);
        println!("{} {}", "Sitemap plan:".cyan().bold(), sitemap);
        print_batch_audit_plan(&effective, url_count);
    } else if effective.url_file.is_some() {
        println!("{} URL file", "Plan:".cyan().bold());
        print_batch_audit_plan(&effective, effective.max_pages.max(1));
    } else if effective.crawl {
        println!("{} Crawl", "Plan:".cyan().bold());
        print_batch_audit_plan(&effective, effective.max_pages.max(1));
    } else if let Some(ref single_url) = effective.url {
        print_single_audit_plan(&effective, single_url);
    }

    Ok(0.0)
}

async fn handle_browser_command(action: &BrowserAction) -> Result<f64> {
    match action {
        BrowserAction::Detect => {
            println!("{}", "Detecting browsers...".cyan().bold());
            println!();

            let browsers = detect_all_browsers();
            if browsers.is_empty() {
                println!("  No browsers found.");
                println!();
                println!("  Installation:");
                println!("    brew install --cask google-chrome");
                println!("    auditmysite browser install");
            } else {
                for browser in &browsers {
                    println!(
                        "  {} {:<25} {:<15} {}",
                        "✓".green(),
                        browser.kind.display_name(),
                        browser.version.as_deref().unwrap_or("unknown"),
                        browser.path.display()
                    );
                }
            }

            // Check managed installs
            if let Some(home) = dirs::home_dir() {
                let browsers_dir = home.join(".auditmysite").join("browsers");
                if browsers_dir.exists() {
                    let cft = browsers_dir.join("chrome-for-testing");
                    let hs = browsers_dir.join("headless-shell");
                    if cft.exists() {
                        let version = std::fs::read_to_string(cft.join("version.txt"))
                            .unwrap_or_else(|_| "unknown".to_string());
                        println!(
                            "  {} {:<25} {:<15} {}",
                            "✓".green(),
                            "Chrome for Testing",
                            version.trim(),
                            cft.display()
                        );
                    }
                    if hs.exists() {
                        let version = std::fs::read_to_string(hs.join("version.txt"))
                            .unwrap_or_else(|_| "unknown".to_string());
                        println!(
                            "  {} {:<25} {:<15} {}",
                            "✓".green(),
                            "Headless Shell",
                            version.trim(),
                            hs.display()
                        );
                    }
                }
            }

            // Show active browser
            println!();
            let opts = BrowserResolveOptions::default();
            match resolve_browser(&opts) {
                Ok(resolved) => {
                    println!(
                        "  {} Active: {} v{} ({})",
                        "→".cyan(),
                        resolved.browser.kind.display_name(),
                        resolved.browser.version.as_deref().unwrap_or("unknown"),
                        resolved.browser.source,
                    );
                }
                Err(_) => {
                    println!("  {} No browser can be resolved for auditing.", "✗".red());
                }
            }

            Ok(0.0)
        }

        BrowserAction::Install {
            headless_shell,
            version,
            force,
        } => {
            let target = if *headless_shell {
                InstallTarget::HeadlessShell
            } else {
                InstallTarget::ChromeForTesting
            };
            BrowserInstaller::install(target, version.as_deref(), *force).await?;
            Ok(0.0)
        }

        BrowserAction::Remove { all } => {
            if *all {
                BrowserInstaller::remove_all()?;
            } else {
                BrowserInstaller::remove(InstallTarget::ChromeForTesting)?;
            }
            Ok(0.0)
        }

        BrowserAction::Path => {
            let opts = BrowserResolveOptions::default();
            match resolve_browser(&opts) {
                Ok(resolved) => {
                    println!("{}", resolved.browser.path.display());
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
            Ok(0.0)
        }
    }
}

pub fn detect_chrome_command(args: &Args) -> Result<f64> {
    println!("{}", "Searching for Chrome/Chromium...".cyan().bold());
    println!();

    match find_chrome(args.chrome_path.as_deref()) {
        Ok(info) => {
            println!("{} Chrome found!", "Done:".green().bold());
            println!("  Path:    {}", info.path.display());
            println!(
                "  Version: {}",
                info.version.as_deref().unwrap_or("unknown")
            );
            println!("  Methode: {:?}", info.detection_method);
            Ok(0.0)
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}
