//! `auditmysite-dev` — maintainer tooling (#912).
//!
//! Everything `auditmysite` does, plus the research and QA commands that do not
//! belong in the CLI a user installs: `plan`, `replay`, `accname-diff`,
//! `report-lint`, and the `--debug-typ` / `--export-snapshot` flags.
//!
//! Built only with the `dev-tools` feature, so `cargo install auditmysite`
//! never installs it:
//!
//! ```text
//! cargo run --features dev-tools --bin auditmysite-dev -- report-lint report.json
//! ```

#![warn(unreachable_pub)]

#[path = "../cli/app.rs"]
mod app;
#[path = "../cli/batch_lifecycle.rs"]
mod batch_lifecycle;
#[path = "../cli/commands.rs"]
mod commands;
#[path = "../cli/dev_commands.rs"]
mod dev_commands;
#[path = "../cli/output_paths.rs"]
mod output_paths;
#[path = "../cli/plan.rs"]
mod plan;
#[path = "../cli/report_writers.rs"]
mod report_writers;
#[path = "../cli/runners.rs"]
mod runners;
#[path = "../cli/sitemap_suggest.rs"]
mod sitemap_suggest;

#[tokio::main]
async fn main() {
    let dev_commands::DevInvocation {
        mut args,
        command,
        matches,
    } = dev_commands::parse_dev_invocation();
    let request_mode_from_cli =
        matches.value_source("request_mode") == Some(clap::parser::ValueSource::CommandLine);
    let interactive_from_cli =
        matches.value_source("interactive") == Some(clap::parser::ValueSource::CommandLine);
    let session_from_cli =
        matches.value_source("session") == Some(clap::parser::ValueSource::CommandLine);
    app::setup_logging(&args);

    let config = auditmysite::cli::Config::load();
    if let Some(ref cfg) = config {
        cfg.apply_to_args_with_sources(&mut args, interactive_from_cli);
    }
    args.apply_technician_preset();

    let report_mode = args.report_mode;
    match command {
        Some(command) => {
            app::run_and_exit(
                dev_commands::run_dev_command(&command, &args, &config, session_from_cli),
                report_mode,
            )
            .await
        }
        None => {
            app::run_and_exit(app::run(args, &config, request_mode_from_cli), report_mode).await
        }
    }
}
