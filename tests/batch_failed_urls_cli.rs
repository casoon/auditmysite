//! #768 / #788: exit codes and CLI rejection of the batch path, through the
//! real binary.

use std::process::Command;

fn binary(dir: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_auditmysite"));
    command.current_dir(dir);
    command
}

/// Unsupported combinations are refused before any browser starts.
#[test]
fn unsupported_extra_check_combinations_exit_with_a_reason() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("urls.txt"), "http://127.0.0.1:9/a\n").unwrap();
    let cases: [(&[&str], &str); 3] = [
        (
            &["--url-file", "urls.txt", "--ai-transparency"],
            "--ai-transparency is single-URL only",
        ),
        (
            &[
                "--url-file",
                "urls.txt",
                "--seo",
                "--isolate-third-party-impact",
            ],
            "--isolate-third-party-impact needs performance checking",
        ),
        (
            &[
                "http://127.0.0.1:9/",
                "--performance",
                "--check-ssr-content",
            ],
            "--check-ssr-content needs SEO checking",
        ),
    ];
    for (argv, reason) in cases {
        let out = binary(dir.path()).args(argv).output().expect("binary runs");
        assert!(!out.status.success(), "{argv:?} must be refused");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains(reason), "{argv:?}: {stderr}");
    }
}

/// A batch in which every URL fails exits with the FAIL code (2) and carries
/// no score of 0, only the counts.
#[test]
#[ignore = "needs Chrome"]
fn batch_with_only_failed_urls_exits_2_without_a_score() {
    let dir = tempfile::tempdir().unwrap();
    // Port 9 (discard) on loopback refuses the connection.
    std::fs::write(
        dir.path().join("urls.txt"),
        "http://127.0.0.1:9/a\nhttp://127.0.0.1:9/b\n",
    )
    .unwrap();
    let out_file = dir.path().join("batch.json");
    let mut command = binary(dir.path());
    command
        .args(["--url-file", "urls.txt", "--format", "json", "-o"])
        .arg(&out_file)
        .args([
            "-q",
            "--progress",
            "never",
            "--timeout",
            "10",
            "--concurrency",
            "1",
        ]);
    if std::env::var("CI").is_ok() {
        command.arg("--no-sandbox");
    }
    let out = command.output().expect("binary runs");
    assert_eq!(
        out.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_file).expect("batch JSON written"))
            .unwrap();
    let summary = &json["summary"];
    assert!(summary["accessibility_score"].is_null(), "{summary}");
    assert_eq!(summary["coverage"]["succeeded"], 0);
    assert_eq!(summary["coverage"]["attempted"], 2);
    assert_eq!(summary["coverage"]["failed"], 2);
}
