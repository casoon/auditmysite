//! The CLI contract that needs no browser (#119): version, help and refused input, through
//! the real binary. Exit codes follow docs/reference/cli-and-output.md: 0 pass, 1 warn,
//! 2 fail. Input that never reaches an audit must not exit with 0 or 1, otherwise a CI gate
//! would read a typo as "pass" or "warn".

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    let dir = tempfile::tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_auditmysite"))
        .current_dir(dir.path())
        .args(args)
        .output()
        .expect("binary runs")
}

fn code(out: &Output) -> i32 {
    out.status.code().expect("exited normally, not by signal")
}

fn assert_refused(args: &[&str]) -> Output {
    let out = run(args);
    assert!(
        code(&out) >= 2,
        "{args:?} must not look like pass or warn, got exit {}",
        code(&out)
    );
    assert!(
        !out.stderr.is_empty(),
        "{args:?} must say why on stderr, stdout was: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    out
}

#[test]
fn version_prints_the_crate_version_and_exits_0() {
    let out = run(&["--version"]);
    assert_eq!(code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
}

#[test]
fn help_exits_0_and_documents_the_modes() {
    let out = run(&["--help"]);
    assert_eq!(code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    for needle in ["Usage", "--sitemap", "--url-file", "--format"] {
        assert!(stdout.contains(needle), "--help lacks {needle}");
    }
}

#[test]
fn no_target_is_refused() {
    assert_refused(&[]);
}

#[test]
fn an_unparseable_url_is_refused() {
    assert_refused(&["not-a-url"]);
}

#[test]
fn a_missing_url_file_is_refused_and_names_the_file() {
    let out = assert_refused(&["--url-file", "no-such-file.txt"]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("no-such-file.txt"));
}

#[test]
fn an_unknown_format_is_refused() {
    assert_refused(&["--format", "nope", "https://example.com"]);
}

#[test]
fn an_unknown_flag_is_refused() {
    assert_refused(&["--no-such-flag", "https://example.com"]);
}
