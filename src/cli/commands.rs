//! CLI subcommand handlers.
//!
//! Handles the browser subcommands, the plan dry-run command, and the
//! legacy --detect-chrome flag. No mode-runner or audit logic here.

use colored::Colorize;

use auditmysite::browser::{
    detect_all_browsers, find_chrome, resolve_browser, BrowserInstaller, BrowserResolveOptions,
    InstallTarget,
};
use auditmysite::cli::{Args, BrowserAction, Command, ReportLintFailOn};
use auditmysite::error::{AuditError, Result};
use auditmysite::lint::lint;
use auditmysite::taxonomy::Severity;

use crate::plan::{print_banner, print_batch_audit_plan, print_single_audit_plan};

pub async fn handle_command(command: &Command, args: &Args) -> Result<f64> {
    match command {
        Command::Browser { action } => handle_browser_command(action).await,
        Command::Doctor => {
            auditmysite::cli::doctor::run_doctor();
            Ok(0.0)
        }
        Command::Plan { url } => run_plan_command(args, url.as_deref()),
        Command::ReportLint {
            input,
            fail_on,
            typst_source,
        } => run_report_lint_command(input, *fail_on, typst_source.as_deref()),
        Command::AccnameDiff {
            urls,
            url_file,
            output,
            max_samples,
        } => {
            run_accname_diff_command(urls, url_file.as_deref(), output.as_deref(), *max_samples)
                .await
        }
    }
}

/// Lädt eine oder mehrere Seiten, stellt die eigene `accname`-Berechnung gegen
/// Chromes native Werte und schreibt das Ergebnis. Bei mehr als einer URL
/// entsteht ein Korpus-Aggregat; eine Seite, die nicht geladen werden kann,
/// wird dort als Fehler geführt, statt den Lauf abzubrechen.
///
/// Chrome ist hier eine zweite Implementierung, nicht die Spezifikation — der
/// Exit-Code bleibt deshalb 0, auch wenn Abweichungen gefunden werden. Das
/// Kommando misst, es urteilt nicht.
async fn run_accname_diff_command(
    urls: &[String],
    url_file: Option<&std::path::Path>,
    output: Option<&std::path::Path>,
    max_samples: usize,
) -> Result<f64> {
    use auditmysite::accessibility::{AccnameCorpus, AccnamePageFailure};
    use auditmysite::browser::BrowserManager;

    let mut all_urls = urls.to_vec();
    if let Some(path) = url_file {
        all_urls.extend(auditmysite::read_url_file(&path.to_string_lossy())?);
    }

    let manager = BrowserManager::new().await?;

    let json = if let [url] = all_urls.as_slice() {
        let diff = accname_diff_page(&manager, url, max_samples).await?;
        print_accname_diff(&diff);
        serde_json::to_string_pretty(&diff)?
    } else {
        let mut pages = Vec::new();
        let mut failures = Vec::new();
        for (i, url) in all_urls.iter().enumerate() {
            eprintln!("[{}/{}] {}", i + 1, all_urls.len(), url);
            match accname_diff_page(&manager, url, max_samples).await {
                Ok(diff) => pages.push(diff),
                Err(e) => failures.push(AccnamePageFailure {
                    url: url.clone(),
                    error: e.to_string(),
                }),
            }
        }
        let corpus = AccnameCorpus::from_pages(pages, failures, max_samples);
        print_accname_corpus(&corpus);
        serde_json::to_string_pretty(&corpus)?
    };
    manager.close().await?;

    if let Some(path) = output {
        std::fs::write(path, json).map_err(|e| AuditError::FileError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        println!("\n{} {}", "JSON:".dimmed(), path.display());
    }

    Ok(0.0)
}

/// Ein Differentiallauf über eine Seite, in einem eigenen Tab.
///
/// Zweiphasig: Erst die Kandidaten bestimmen, deren Namen sich nur in der
/// Groß-/Kleinschreibung unterscheiden, dann für genau diese den berechneten
/// Stil holen. Das ist Messung auf der Seite dieses Werkzeugs — `accname`
/// selbst bleibt unberührt.
async fn accname_diff_page(
    manager: &auditmysite::browser::BrowserManager,
    url: &str,
    max_samples: usize,
) -> Result<auditmysite::accessibility::AccnameDiff> {
    use auditmysite::accessibility::{
        accname_case_only_candidates, compare_accname_with_text_transform, extract_ax_tree,
        fetch_dom_document,
    };

    let page = manager.new_page().await?;
    let result = async {
        manager.navigate(&page, url).await?;
        let ax_tree = extract_ax_tree(&page).await?;
        let doc = fetch_dom_document(&page, &ax_tree).await?;

        let mut text_transformed = std::collections::HashSet::new();
        for id in accname_case_only_candidates(&doc) {
            if has_text_transform(&page, id).await {
                text_transformed.insert(id);
            }
        }

        let mut diff = compare_accname_with_text_transform(&doc, max_samples, &text_transformed);
        diff.url = Some(url.to_string());
        Ok(diff)
    }
    .await;
    let _ = page.close().await;
    result
}

/// Hat das Element oder ein Nachfahre ein berechnetes `text-transform`
/// ungleich `none`? Die Nachfahren zählen mit, weil ein Name aus dem Inhalt
/// den Text der Kinder übernimmt, deren Stil sich vom Element unterscheiden
/// kann. Bei einem CDP-Fehler `false` — der Fall bleibt dann `mismatch` und
/// damit sichtbar.
async fn has_text_transform(page: &chromiumoxide::Page, backend_node_id: i64) -> bool {
    use chromiumoxide::cdp::browser_protocol::dom::{BackendNodeId, ResolveNodeParams};
    use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;

    const JS: &str = "function() { return [this, ...this.querySelectorAll('*')].some(\
        e => getComputedStyle(e).textTransform !== 'none'); }";

    let resolve = ResolveNodeParams::builder()
        .backend_node_id(BackendNodeId::new(backend_node_id))
        .build();
    let Ok(resolved) = page.execute(resolve).await else {
        return false;
    };
    let Some(object_id) = resolved.result.object.object_id.clone() else {
        return false;
    };
    let Ok(call) = CallFunctionOnParams::builder()
        .function_declaration(JS)
        .object_id(object_id)
        .return_by_value(true)
        .build()
    else {
        return false;
    };
    page.execute(call)
        .await
        .ok()
        .and_then(|r| r.result.result.value)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn print_accname_corpus(corpus: &auditmysite::accessibility::AccnameCorpus) {
    println!("\n{}", "accname vs. Chrome — Korpus".bold());
    println!(
        "{:<28} {} von {}",
        "Seiten verglichen", corpus.pages_compared, corpus.pages_total
    );
    for f in &corpus.failures {
        println!("  {} {}  {}", "Fehler:".red(), f.url, f.error.dimmed());
    }
    println!("{:<28} {}", "Elemente im DOM", corpus.elements_total);
    println!(
        "{:<28} {}  ({} ohne AX-Gegenstück, {} ignoriert)",
        "verglichen",
        corpus.elements_compared,
        corpus.skipped_not_in_ax_tree,
        corpus.skipped_ignored
    );

    println!("\n{}", "Name".bold());
    println!("{:<28} {}", "gleich", corpus.names_equal);
    println!("{:<28} {}", "abweichend", corpus.names_divergent());
    for (shape, count) in &corpus.names_by_shape {
        println!("  {:<26} {}", shape, count);
    }
    if !corpus.names_by_source.is_empty() {
        println!(
            "\n{}",
            "Abweichungen nach Namensquelle (laut Chrome)".dimmed()
        );
        for (source, count) in &corpus.names_by_source {
            println!("  {:<26} {}", source, count);
        }
    }

    println!("\n{}", "Rolle".bold());
    println!("{:<28} {}", "gleich", corpus.roles_equal);
    println!("{:<28} {}", "abweichend", corpus.roles_divergent);
    println!(
        "{:<28} {}",
        "nicht vergleichbar".dimmed(),
        corpus.roles_not_comparable
    );

    println!("\n{}", "Seiten".bold());
    for page in &corpus.pages {
        println!(
            "  {:>5} verglichen  {:>3} abweichend  {}",
            page.elements_compared,
            page.names_divergent(),
            page.url.as_deref().unwrap_or("")
        );
    }

    if !corpus.name_patterns.is_empty() {
        const SHOWN: usize = 5;
        println!("\n{} — je Form die häufigsten", "Muster".bold());
        let mut shown: std::collections::BTreeMap<_, usize> = Default::default();
        for p in &corpus.name_patterns {
            let n = shown.entry(p.shape).or_insert(0);
            *n += 1;
            if *n > SHOWN {
                continue;
            }
            println!(
                "  [{}] {}× auf {} Seiten  <{}>  chrome={:?}  accname={:?}",
                p.shape.as_str(),
                p.occurrences,
                p.pages,
                p.tag,
                p.chrome.as_deref().unwrap_or(""),
                p.accname.as_deref().unwrap_or("")
            );
        }
        if !corpus.patterns_complete {
            println!(
                "  {}",
                "Beispielgrenze erreicht — Vorkommen je Muster sind Untergrenzen.".dimmed()
            );
        }
    }
}

fn print_accname_diff(diff: &auditmysite::accessibility::AccnameDiff) {
    let substantive = diff.names_divergent_substantive();

    println!("\n{}", "accname vs. Chrome".bold());
    println!("{}", diff.url.as_deref().unwrap_or("").dimmed());
    println!("\n{:<28} {}", "Elemente im DOM", diff.elements_total);
    println!(
        "{:<28} {}  ({} ohne AX-Gegenstück, {} ignoriert)",
        "verglichen", diff.elements_compared, diff.skipped_not_in_ax_tree, diff.skipped_ignored
    );

    println!("\n{}", "Name".bold());
    println!("{:<28} {}", "gleich", diff.names_equal);
    let label = format!("{substantive}");
    println!(
        "{:<28} {}",
        "abweichend (ohne Leerraum)",
        if substantive == 0 {
            label.green()
        } else {
            label.yellow()
        }
    );
    for (shape, count) in &diff.names_by_shape {
        println!("  {:<26} {}", shape, count);
    }
    if !diff.names_by_source.is_empty() {
        println!(
            "\n{}",
            "Abweichungen nach Namensquelle (laut Chrome)".dimmed()
        );
        for (source, count) in &diff.names_by_source {
            println!("  {:<26} {}", source, count);
        }
    }

    println!("\n{}", "Rolle".bold());
    println!("{:<28} {}", "gleich", diff.roles_equal);
    println!("{:<28} {}", "abweichend", diff.roles_divergent);
    println!(
        "{:<28} {}",
        "nicht vergleichbar".dimmed(),
        diff.roles_not_comparable
    );

    if !diff.name_divergences.is_empty() {
        const SHOWN: usize = 15;
        let shown = diff.name_divergences.len().min(SHOWN);
        println!(
            "\n{} — {shown} von {} Abweichungen",
            "Beispiele".bold(),
            diff.names_divergent()
        );
        for d in diff.name_divergences.iter().take(SHOWN) {
            println!(
                "  [{}] <{}>  chrome={:?}  accname={:?}",
                d.shape.as_str(),
                d.tag,
                d.chrome.as_deref().unwrap_or(""),
                d.accname.as_deref().unwrap_or("")
            );
        }
    }

    println!(
        "\n{}",
        "Chrome ist eine zweite Implementierung, nicht die Spezifikation.".dimmed()
    );
    println!(
        "{}",
        "Eine Abweichung ist ein Hinweis; sie wird gegen WPT bzw. accname 1.2 entschieden."
            .dimmed()
    );
}

fn report_lint_fail_on_to_severity(fail_on: Option<ReportLintFailOn>) -> Severity {
    match fail_on.unwrap_or(ReportLintFailOn::High) {
        ReportLintFailOn::Low => Severity::Low,
        ReportLintFailOn::Medium => Severity::Medium,
        ReportLintFailOn::High => Severity::High,
        ReportLintFailOn::Critical => Severity::Critical,
    }
}

fn run_report_lint_command(
    input: &std::path::Path,
    fail_on: Option<ReportLintFailOn>,
    typst_source: Option<&std::path::Path>,
) -> Result<f64> {
    let text = std::fs::read_to_string(input).map_err(|e| AuditError::FileError {
        path: input.to_path_buf(),
        reason: e.to_string(),
    })?;
    let report: serde_json::Value = serde_json::from_str(&text)?;

    let typst_text = typst_source
        .map(|path| {
            std::fs::read_to_string(path).map_err(|e| AuditError::FileError {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })
        })
        .transpose()?;

    let result = lint(&report, typst_text.as_deref());
    let threshold = report_lint_fail_on_to_severity(fail_on);

    if result.is_clean() {
        println!("{} {}", "report-lint:".cyan().bold(), "no findings".green());
    } else {
        println!(
            "{} {} finding(s)",
            "report-lint:".cyan().bold(),
            result.findings.len()
        );
        for finding in &result.findings {
            println!(
                "  [{}] {} — {}\n      expected: {}\n      actual:   {}",
                format!("{:?}", finding.severity).to_uppercase(),
                finding.check_id,
                finding.evidence_path,
                finding.expected,
                finding.actual,
            );
        }
    }

    match result.worst_severity() {
        Some(worst) if worst >= threshold => Err(AuditError::ConfigError(format!(
            "report-lint found a {worst:?} finding, at or above --fail-on {threshold:?}"
        ))),
        _ => Ok(0.0),
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
