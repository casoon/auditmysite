//! Consent-based bootstrap of the managed browser (#855).
//!
//! An audit that finds no usable browser may offer a one-time managed
//! Chrome for Testing install, and only an interactive terminal on a
//! supported platform ever gets that offer. Non-interactive, headless and CI
//! runs never download and never ask: they fail with the existing
//! actionable error. `browser ensure --yes` is the explicit opt-in for them.

use std::io::IsTerminal;
use std::path::PathBuf;

use super::install_plan::{Artifact, Platform};
use super::install_txn::{InstallPreview, InstallRequest, Installer};
use super::installer::BrowserInstaller;
use super::resolver::{resolve_browser, BrowserResolveOptions};
use crate::error::AuditError;

/// Why no install was offered or performed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("{0}")]
    UnsupportedPlatform(String),
    #[error("No browser found and offline mode forbids downloading one. Run `auditmysite browser install` with network access once")]
    Offline,
    #[error("No browser found. This is not an interactive session, so nothing is downloaded and nothing is asked. Opt in with `auditmysite browser ensure --yes` (or `auditmysite browser install`), or pass --browser-path")]
    NonInteractive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Install; ask for consent first when `prompt`.
    Install {
        prompt: bool,
    },
    Refuse(Refusal),
}

/// The consent policy as a pure function.
pub fn decide(
    platform: &Result<Platform, String>,
    interactive: bool,
    assume_yes: bool,
    offline: bool,
) -> Decision {
    if let Err(msg) = platform {
        return Decision::Refuse(Refusal::UnsupportedPlatform(msg.clone()));
    }
    if offline {
        return Decision::Refuse(Refusal::Offline);
    }
    if assume_yes {
        return Decision::Install { prompt: false };
    }
    if interactive {
        Decision::Install { prompt: true }
    } else {
        Decision::Refuse(Refusal::NonInteractive)
    }
}

/// A run may prompt only when a person is at the terminal: stdin and stderr
/// are terminals, `CI` is not set, and the run is not `--quiet`.
pub fn is_interactive(quiet: bool) -> bool {
    interactive_from(
        std::io::stdin().is_terminal(),
        std::io::stderr().is_terminal(),
        std::env::var("CI").ok().as_deref(),
        quiet,
    )
}

pub(crate) fn interactive_from(
    stdin_tty: bool,
    stderr_tty: bool,
    ci_env: Option<&str>,
    quiet: bool,
) -> bool {
    let ci = ci_env.is_some_and(|v| !matches!(v.trim(), "" | "0" | "false"));
    stdin_tty && stderr_tty && !ci && !quiet
}

/// The consent prompt: dependency, source, destination, size, how to decline.
pub fn consent_text(p: &InstallPreview) -> String {
    let size = p
        .size
        .map(|b| format!("about {} MB", b.div_ceil(1_000_000)))
        .unwrap_or_else(|| "size unknown".to_string());
    format!(
        "auditmysite needs a Chromium-based browser to read the page's accessibility tree, and none was found.\n\
         It can download a managed copy of Chrome for Testing {version} once:\n\
         \x20 from        {host} (Google's Chrome for Testing archive)\n\
         \x20 to          {dest}\n\
         \x20 download    {size}, no administrator rights, nothing installed system-wide\n\
         Upstream publishes no checksum or signature for it; the download is checked for\n\
         size, safe extraction and a browser start before it replaces anything.\n\
         Declining changes nothing: install a browser yourself or pass --browser-path.",
        version = p.version,
        host = p.url.host_str().unwrap_or("?"),
        dest = p.destination.display(),
    )
}

#[derive(Debug, Clone)]
pub struct EnsureOptions {
    pub artifact: Artifact,
    pub assume_yes: bool,
    pub interactive: bool,
    pub offline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnsureOutcome {
    /// A usable browser exists already; nothing was downloaded.
    Present(PathBuf),
    Installed(PathBuf),
    Declined,
    Refused(Refusal),
}

/// `present` is the already-resolved browser, if any. `ask` shows the consent
/// text and returns the answer; it is called only when consent is needed.
pub async fn ensure_with(
    installer: &Installer,
    present: Option<PathBuf>,
    opts: &EnsureOptions,
    ask: &dyn Fn(&str) -> bool,
) -> Result<EnsureOutcome, AuditError> {
    if let Some(path) = present {
        return Ok(EnsureOutcome::Present(path));
    }
    let platform: Result<Platform, String> = Ok(installer.platform);
    let request = InstallRequest {
        artifact: opts.artifact,
        version: None,
        force: false,
        offline: opts.offline,
    };
    match decide(&platform, opts.interactive, opts.assume_yes, opts.offline) {
        Decision::Refuse(r) => Ok(EnsureOutcome::Refused(r)),
        Decision::Install { prompt } => {
            if prompt {
                let preview = installer.preview(&request).await?;
                if !ask(&consent_text(&preview)) {
                    return Ok(EnsureOutcome::Declined);
                }
            }
            let outcome = installer.install(&request).await?;
            Ok(EnsureOutcome::Installed(outcome.binary().to_path_buf()))
        }
    }
}

/// Production entry: resolve, decide, maybe ask, maybe install.
pub async fn ensure_browser(opts: &EnsureOptions) -> Result<EnsureOutcome, AuditError> {
    if let Ok(resolved) = resolve_browser(&BrowserResolveOptions::default()) {
        return Ok(EnsureOutcome::Present(resolved.browser.path));
    }
    let installer = match Installer::official(BrowserInstaller::browsers_dir()?, opts.artifact) {
        Ok(i) => i,
        Err(e) => {
            return Ok(EnsureOutcome::Refused(Refusal::UnsupportedPlatform(
                e.to_string(),
            )))
        }
    };
    ensure_with(&installer, None, opts, &|text| {
        eprintln!("\n{text}\n");
        dialoguer::Confirm::new()
            .with_prompt("  Download Chrome for Testing now?")
            .default(false)
            .interact_on(&dialoguer::console::Term::stderr())
            .unwrap_or(false)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_policy() {
        let ok: Result<Platform, String> = Ok(Platform::MacArm64);
        let win: Result<Platform, String> = Err("Windows uses Edge".to_string());
        assert_eq!(
            decide(&ok, true, false, false),
            Decision::Install { prompt: true }
        );
        assert_eq!(
            decide(&ok, false, true, false),
            Decision::Install { prompt: false }
        );
        // Never a silent download without a person or the flag.
        assert_eq!(
            decide(&ok, false, false, false),
            Decision::Refuse(Refusal::NonInteractive)
        );
        assert_eq!(
            decide(&ok, true, true, true),
            Decision::Refuse(Refusal::Offline)
        );
        // Windows is never offered a download, not even with --yes.
        assert!(matches!(
            decide(&win, true, true, false),
            Decision::Refuse(Refusal::UnsupportedPlatform(_))
        ));
    }

    #[test]
    fn interactive_only_for_a_person_at_a_terminal() {
        assert!(interactive_from(true, true, None, false));
        assert!(!interactive_from(false, true, None, false), "stdin piped");
        assert!(!interactive_from(true, false, None, false), "stderr piped");
        assert!(!interactive_from(true, true, Some("true"), false), "CI");
        assert!(!interactive_from(true, true, Some("1"), false), "CI");
        assert!(interactive_from(true, true, Some("false"), false));
        assert!(interactive_from(true, true, Some(""), false));
        assert!(!interactive_from(true, true, None, true), "--quiet");
    }

    #[test]
    fn consent_text_names_source_destination_size_and_decline() {
        let p = InstallPreview {
            version: "156.0.8078.80".to_string(),
            url: url::Url::parse(
                "https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.80/mac-arm64/chrome-mac-arm64.zip",
            )
            .unwrap(),
            size: Some(195_027_080),
            destination: PathBuf::from("/home/u/.auditmysite/browsers/chrome-for-testing"),
        };
        let t = consent_text(&p);
        assert!(t.contains("156.0.8078.80"));
        assert!(t.contains("storage.googleapis.com"));
        assert!(t.contains("/home/u/.auditmysite/browsers/chrome-for-testing"));
        assert!(t.contains("about 196 MB"), "{t}");
        assert!(t.contains("Declining changes nothing"));
        assert!(t.contains("no checksum or signature"));
    }
}
