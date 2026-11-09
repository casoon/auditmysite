//! The command-line surface of the `auditmysite` binary (#912, #913, #910,
//! #916): what a user sees in `--help`, and what the maintainer binary adds.

use std::process::Command;

fn help_of_user_binary() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_auditmysite"))
        .arg("--help")
        .output()
        .expect("failed to run auditmysite --help");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn user_help_lists_no_maintainer_tooling() {
    let help = help_of_user_binary();
    for word in [
        "accname-diff",
        "report-lint",
        "replay",
        "--export-snapshot",
        "--debug-typ",
    ] {
        assert!(!help.contains(word), "`{word}` is in `auditmysite --help`");
    }
    // `plan` is a subcommand name; match it as a command line, not as a word.
    assert!(!help.contains("\n  plan "), "{help}");
}

#[test]
fn maintainer_commands_are_unknown_to_the_user_binary() {
    for args in [
        &["report-lint", "x.json"][..],
        &["accname-diff", "https://example.com"],
        &["replay", "https://example.com"],
        &["https://example.com", "--debug-typ"],
        &["https://example.com", "--export-snapshot", "x.yaml"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_auditmysite"))
            .args(args)
            .output()
            .expect("failed to run auditmysite");
        assert!(
            !output.status.success(),
            "`auditmysite {}` was accepted",
            args.join(" ")
        );
    }
}

#[cfg(feature = "dev-tools")]
mod dev_binary {
    use std::path::Path;
    use std::process::Command;

    fn report_lint(fixture: &str) -> (Option<i32>, String) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/lint_fixtures")
            .join(fixture);
        let output = Command::new(env!("CARGO_BIN_EXE_auditmysite-dev"))
            .arg("report-lint")
            .arg(path)
            .output()
            .expect("failed to run auditmysite-dev report-lint");
        let mut text = String::from_utf8_lossy(&output.stdout).to_string();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        (output.status.code(), text)
    }

    #[test]
    fn report_lint_runs_through_the_dev_binary() {
        let (code, text) = report_lint("clean_single.json");
        assert_eq!(code, Some(0), "{text}");
        assert!(text.contains("no findings"), "{text}");

        let (code, text) = report_lint("broken_grade_mismatch.json");
        assert_eq!(code, Some(3), "{text}");
        assert!(text.contains("grade_matches_accessibility_score"), "{text}");
    }

    #[test]
    fn dev_help_lists_the_maintainer_tooling() {
        let output = Command::new(env!("CARGO_BIN_EXE_auditmysite-dev"))
            .arg("--help")
            .output()
            .expect("failed to run auditmysite-dev --help");
        let help = String::from_utf8_lossy(&output.stdout);
        for word in [
            "accname-diff",
            "report-lint",
            "replay",
            "--export-snapshot",
            "--debug-typ",
        ] {
            assert!(help.contains(word), "`{word}` missing from {help}");
        }
    }
}
