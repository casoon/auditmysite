//! Maintainer commands of the `auditmysite-dev` binary (#912): `plan`,
//! `replay`, `accname-diff`, `report-lint`, plus the `--debug-typ` and
//! `--export-snapshot` flags. None of this is part of the `auditmysite` CLI a
//! user installs.

use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command as ClapCommand, FromArgMatches, Subcommand};
use colored::Colorize;

use auditmysite::audit::{replay_report, ExpectedProfile, SnapshotArtifact, Verdict};
use auditmysite::cli::report_lint::{run_report_lint, ReportLintFailOn};
use auditmysite::cli::{Args, DevOptions};
use auditmysite::error::{AuditError, Result};

use crate::plan::{print_banner, print_batch_audit_plan, print_single_audit_plan};
use crate::runners::emit_replayed;

/// Maintainer subcommands, added on top of the `auditmysite` CLI.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum DevCommand {
    /// Show what an audit would do (mode, scope, modules, output paths) without
    /// running it.
    Plan {
        /// URL to plan an audit for (required unless --sitemap or --url-file is set)
        url: Option<String>,
    },
    /// Evaluate a stored snapshot again — no browser, no network access.
    ///
    /// The target is a `snapshot.json`, a cache entry directory, or the URL
    /// whose cached entry to replay. The report states `replay` as its
    /// provenance. A snapshot that is incomplete or was written under other
    /// versions is refused with its reason, never evaluated under today's
    /// semantics. `--session`, `--state-id` and `--storage-state` assert the
    /// measurement profile the snapshot must carry; another profile is
    /// refused, not shown as the same measurement.
    Replay {
        /// Snapshot file, cache entry directory, or URL of a cached entry
        target: String,
    },
    /// Compare the own `accname` computation against Chrome's native
    /// accessibility tree — a differential test without a screen reader.
    AccnameDiff {
        /// URL(s) to load and compare; more than one URL (together with
        /// --url-file) produces a corpus aggregate instead of a single result
        #[arg(value_name = "URL", required_unless_present = "url_file")]
        urls: Vec<String>,
        /// Read further URLs from a file (one per line, `#` comments allowed)
        #[arg(long)]
        url_file: Option<PathBuf>,
        /// Write the full JSON result here (terminal summary is always printed)
        #[arg(long)]
        output: Option<PathBuf>,
        /// Maximum stored divergence samples per kind; counts stay complete
        #[arg(long, default_value_t = auditmysite::accessibility::DEFAULT_MAX_SAMPLES)]
        max_samples: usize,
    },
    /// Run deterministic report-lint checks (#507) against a JSON report file.
    ReportLint {
        /// Path to a JSON report file (single or batch envelope)
        input: PathBuf,
        /// Minimum finding severity that causes a non-zero exit code (default: high)
        #[arg(long, value_enum)]
        fail_on: Option<ReportLintFailOn>,
        /// Path to the `--debug-typ` Typst source for the same report — when
        /// given, additionally checks the accessibility score is traceable to it
        #[arg(long)]
        typst_source: Option<PathBuf>,
    },
}

const DEV_SUBCOMMANDS: [&str; 4] = ["plan", "replay", "accname-diff", "report-lint"];

/// What the dev binary was asked to do, next to the ordinary audit arguments.
pub(crate) struct DevInvocation {
    pub args: Args,
    pub command: Option<DevCommand>,
    pub matches: ArgMatches,
}

/// The `auditmysite` command line plus the maintainer subcommands and flags.
pub(crate) fn dev_command() -> ClapCommand {
    use clap::CommandFactory;
    let command = Args::command()
        .name("auditmysite-dev")
        .about(
            "Maintainer tooling for auditmysite: everything `auditmysite` does, plus plan, \
             replay, accname-diff, report-lint, --debug-typ and --export-snapshot",
        )
        .arg(
            Arg::new("debug_typ")
                .long("debug-typ")
                .global(true)
                .action(ArgAction::SetTrue)
                .help(
                    "With `--format pdf`, also write the intermediate Typst source as a `.typ` \
                     sidecar next to the PDF (single + batch), for template/wording review",
                ),
        )
        .arg(
            Arg::new("export_snapshot")
                .long("export-snapshot")
                .value_name("PATH")
                .value_parser(clap::value_parser!(PathBuf))
                .help(
                    "Export AXTree snapshots and journey traces as YAML (Playwright's ARIA \
                     snapshot format) next to a JSON report",
                ),
        );
    DevCommand::augment_subcommands(command)
}

/// Splits matches into the maintainer subcommand (if any) and the ordinary
/// audit arguments. The subcommand is taken out of the matches, so `Args`
/// sees an ordinary invocation.
fn split_matches(mut matches: ArgMatches) -> (Args, Option<DevCommand>, ArgMatches) {
    let is_dev_command = matches
        .subcommand_name()
        .is_some_and(|name| DEV_SUBCOMMANDS.contains(&name));
    let command = is_dev_command.then(|| {
        let command = DevCommand::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
        matches.remove_subcommand();
        command
    });
    let mut args = Args::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    args.dev = DevOptions {
        debug_typ: matches.get_flag("debug_typ"),
        export_snapshot: matches.get_one::<PathBuf>("export_snapshot").cloned(),
    };
    (args, command, matches)
}

/// Parses the dev command line.
pub(crate) fn parse_dev_invocation() -> DevInvocation {
    let (args, command, matches) = split_matches(dev_command().get_matches());
    DevInvocation {
        args,
        command,
        matches,
    }
}

pub(crate) async fn run_dev_command(
    command: &DevCommand,
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
    session_from_cli: bool,
) -> Result<Verdict> {
    match command {
        DevCommand::Plan { url } => run_plan_command(args, url.as_deref()).map(|_| Verdict::Pass),
        DevCommand::Replay { target } => {
            validate_replay(args)?;
            run_replay_mode(args, config, target, session_from_cli)
        }
        DevCommand::ReportLint {
            input,
            fail_on,
            typst_source,
        } => {
            let out = run_report_lint(input, *fail_on, typst_source.as_deref())?;
            print!("{out}");
            Ok(Verdict::Pass)
        }
        DevCommand::AccnameDiff {
            urls,
            url_file,
            output,
            max_samples,
        } => run_accname_diff_command(urls, url_file.as_deref(), output.as_deref(), *max_samples)
            .await
            .map(|_| Verdict::Pass),
    }
}

/// A replay evaluates a stored snapshot: a flag asking for a live check is
/// refused, and a declared state must be readable (#788).
fn validate_replay(args: &Args) -> Result<()> {
    args.load_storage_state().map_err(AuditError::ConfigError)?;
    if args.perf_runs > 1 {
        return Err(AuditError::ConfigError(
            "--perf-runs measures a live page; `replay` evaluates a stored snapshot".to_string(),
        ));
    }
    auditmysite::audit::support_matrix::ExtraCheckRequest {
        isolate_third_party_impact: args.isolate_third_party_impact,
        check_ssr_content: args.check_ssr_content,
        privacy_check: args.privacy_check,
        replay: true,
        ..Default::default()
    }
    .validate()
    .map_err(AuditError::ConfigError)
}

/// `auditmysite-dev replay <target>`: the stored snapshot, evaluated again
/// without browser or network. `session_from_cli` says whether `--session`
/// was given: its default is not an assertion about the snapshot.
fn run_replay_mode(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
    target: &str,
    session_from_cli: bool,
) -> Result<Verdict> {
    let path = std::path::Path::new(target);
    let snapshot = if path.exists() {
        SnapshotArtifact::read(path)?
    } else if url::Url::parse(target).is_ok() {
        SnapshotArtifact::read_cached(target)?
    } else {
        return Err(AuditError::ConfigError(format!(
            "replay target '{target}' is neither an existing snapshot/cache directory nor a URL"
        )));
    };
    let expected = ExpectedProfile {
        session_policy: session_from_cli.then(|| args.session.as_str().to_string()),
        state_id: args.state_id.clone(),
        storage_state_sha256: args
            .load_storage_state()
            .map_err(AuditError::ConfigError)?
            .map(|state| state.digest),
        // Replay refuses extra-check flags (support matrix), so nothing to assert.
        extra_checks: None,
        // A replay measures nothing, so it asserts no browser.
        browser: None,
    };
    let refuse = |e: auditmysite::audit::ReplayError| AuditError::ConfigError(e.to_string());
    snapshot.check_measurement(&expected).map_err(refuse)?;
    let report = replay_report(&snapshot).map_err(refuse)?;
    if !args.quiet {
        println!(
            "{} {}",
            "Replay:".cyan().bold(),
            format!("{} (no browser, no network)", report.url).dimmed()
        );
    }
    emit_replayed(report, args, config).map(|o| o.verdict)
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
        accname_case_only_candidates, compare_accname_with_text_transform,
        extract_ax_tree_with_name_sources, fetch_dom_document_with_layout,
    };

    let page = manager.new_page().await?;
    let result = async {
        manager.navigate(&page, url).await?;
        let (ax_tree, name_sources) = extract_ax_tree_with_name_sources(&page).await?;
        let doc = fetch_dom_document_with_layout(&page, &ax_tree)
            .await?
            .with_chrome_name_sources(&name_sources);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(argv: &[&str]) -> std::result::Result<(Args, Option<DevCommand>), clap::Error> {
        let matches = dev_command().try_get_matches_from(argv)?;
        let (args, command, _) = split_matches(matches);
        Ok((args, command))
    }

    #[test]
    fn replay_takes_a_target_and_refuses_live_check_flags() {
        let (args, command) = parse(&["auditmysite-dev", "replay", "https://example.com"]).unwrap();
        assert!(
            matches!(command, Some(DevCommand::Replay { ref target }) if target == "https://example.com")
        );
        assert!(validate_replay(&args).is_ok());

        let (args, _) = parse(&[
            "auditmysite-dev",
            "--privacy-check",
            "replay",
            "https://example.com",
        ])
        .unwrap();
        let e = validate_replay(&args).unwrap_err().to_string();
        assert!(e.contains("--privacy-check") && e.contains("replay"), "{e}");

        let (args, _) = parse(&[
            "auditmysite-dev",
            "--perf-runs",
            "3",
            "replay",
            "https://example.com",
        ])
        .unwrap();
        let e = validate_replay(&args).unwrap_err().to_string();
        assert!(e.contains("replay"), "{e}");

        assert!(parse(&["auditmysite-dev", "replay"]).is_err());
    }

    #[test]
    fn accname_diff_nimmt_mehrere_urls_oder_url_datei() {
        let (_, command) = parse(&[
            "auditmysite-dev",
            "accname-diff",
            "https://a.example",
            "https://b.example",
        ])
        .unwrap();
        match command {
            Some(DevCommand::AccnameDiff { urls, url_file, .. }) => {
                assert_eq!(urls.len(), 2);
                assert!(url_file.is_none());
            }
            _ => panic!("accname-diff erwartet"),
        }

        let (_, command) =
            parse(&["auditmysite-dev", "accname-diff", "--url-file", "urls.txt"]).unwrap();
        assert!(matches!(
            command,
            Some(DevCommand::AccnameDiff { ref urls, url_file: Some(_), .. }) if urls.is_empty()
        ));

        assert!(parse(&["auditmysite-dev", "accname-diff"]).is_err());
    }

    #[test]
    fn dev_flags_and_ordinary_audit_arguments_parse_together() {
        let (args, command) = parse(&[
            "auditmysite-dev",
            "https://example.com",
            "-f",
            "pdf",
            "--debug-typ",
            "--export-snapshot",
            "snap.yaml",
        ])
        .unwrap();
        assert!(command.is_none());
        assert_eq!(args.url.as_deref(), Some("https://example.com"));
        assert!(args.dev.debug_typ);
        assert_eq!(args.dev.export_snapshot, Some(PathBuf::from("snap.yaml")));
    }

    #[test]
    fn plan_reads_the_audit_flags_given_after_it() {
        let (args, command) = parse(&[
            "auditmysite-dev",
            "plan",
            "https://example.com",
            "--lang",
            "en",
        ])
        .unwrap();
        assert!(matches!(command, Some(DevCommand::Plan { url: Some(_) })));
        assert_eq!(args.lang, "en");
    }
}
