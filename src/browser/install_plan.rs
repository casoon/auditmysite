//! Install plan for the managed Chrome for Testing (CfT) browser (#855).
//!
//! Pure decisions, no I/O: which platforms are supported, which publisher
//! manifest entry is the download, and which URLs are trusted.
//!
//! # What upstream publishes (researched 2026-11-02)
//!
//! The CfT JSON endpoints (`last-known-good-versions-with-downloads.json`,
//! `known-good-versions-with-downloads.json`) list per version and platform
//! only `platform` and `url`. They carry **no checksum and no signature**,
//! and no `.sha256`/`.sig` file exists next to the archives. The bucket
//! (`storage.googleapis.com`) returns `x-goog-hash` (MD5, CRC32C) and an
//! `ETag`, but those come from the same origin as the bytes, so they detect
//! transfer corruption and prove nothing about the publisher. A hash computed
//! from the received download authenticates nothing either.
//!
//! Therefore this installer does **not** claim publisher verification. The
//! trust anchor is TLS to two fixed hosts (see [`SourcePolicy::official`]),
//! the exact manifest entry for the exact platform, and the integrity checks
//! of the transaction (announced size, safe extraction, version, CDP smoke).
//! If upstream starts publishing a digest or signature, it belongs here.

use std::path::PathBuf;

use serde_json::Value;
use url::Url;

use super::install_txn::InstallError;
use super::types::InstallTarget;

/// Platforms with a managed Chrome for Testing build (platform contract of
/// v2.0: macOS ARM64 only, Linux x86_64 and ARM64; Windows uses Edge).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacArm64,
    LinuxX64,
    LinuxArm64,
}

/// The archive kind of a managed install.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Artifact {
    Chrome,
    HeadlessShell,
}

impl From<InstallTarget> for Artifact {
    fn from(target: InstallTarget) -> Self {
        match target {
            InstallTarget::ChromeForTesting => Artifact::Chrome,
            InstallTarget::HeadlessShell => Artifact::HeadlessShell,
        }
    }
}

impl Artifact {
    /// Directory name below `~/.auditmysite/browsers/`.
    pub fn dir_name(self) -> &'static str {
        match self {
            Artifact::Chrome => "chrome-for-testing",
            Artifact::HeadlessShell => "headless-shell",
        }
    }

    /// Key in the manifest's `downloads` object.
    fn manifest_key(self) -> &'static str {
        match self {
            Artifact::Chrome => "chrome",
            Artifact::HeadlessShell => "chrome-headless-shell",
        }
    }

    fn archive_stem(self) -> &'static str {
        match self {
            Artifact::Chrome => "chrome",
            Artifact::HeadlessShell => "chrome-headless-shell",
        }
    }
}

impl Platform {
    /// The platform this binary runs on, or why it has no managed browser.
    pub fn detect() -> Result<Self, InstallError> {
        if let Some(reason) = super::platform::Platform::current().managed_unavailable_reason() {
            return Err(InstallError::UnsupportedPlatform(reason));
        }
        Self::from_target(std::env::consts::OS, std::env::consts::ARCH)
    }

    pub fn from_target(os: &str, arch: &str) -> Result<Self, InstallError> {
        match (os, arch) {
            ("macos", "aarch64") => Ok(Platform::MacArm64),
            ("linux", "x86_64") => Ok(Platform::LinuxX64),
            ("linux", "aarch64") => Ok(Platform::LinuxArm64),
            ("windows", _) => Err(InstallError::UnsupportedPlatform(
                "Windows uses an existing Microsoft Edge only; no browser is downloaded there. \
                 Install or repair Edge (https://www.microsoft.com/edge) or pass --browser-path \
                 to its msedge.exe"
                    .to_string(),
            )),
            ("macos", _) => Err(InstallError::UnsupportedPlatform(format!(
                "macOS is supported on Apple Silicon (ARM64) only, this is {arch}"
            ))),
            _ => Err(InstallError::UnsupportedPlatform(format!(
                "no managed browser for {os}/{arch}; supported: macOS ARM64, Linux x86_64 and ARM64"
            ))),
        }
    }

    /// Platform id in the CfT manifest and download URLs.
    pub fn manifest_id(self) -> &'static str {
        match self {
            Platform::MacArm64 => "mac-arm64",
            Platform::LinuxX64 => "linux64",
            Platform::LinuxArm64 => "linux-arm64",
        }
    }

    /// Top-level directory inside the archive.
    pub fn archive_dir(self, artifact: Artifact) -> String {
        format!("{}-{}", artifact.archive_stem(), self.manifest_id())
    }

    /// Binary path relative to the install directory.
    pub fn binary_rel_path(self, artifact: Artifact) -> PathBuf {
        let dir = PathBuf::from(self.archive_dir(artifact));
        match (artifact, self) {
            (Artifact::Chrome, Platform::MacArm64) => dir
                .join("Google Chrome for Testing.app")
                .join("Contents")
                .join("MacOS")
                .join("Google Chrome for Testing"),
            (Artifact::Chrome, _) => dir.join("chrome"),
            (Artifact::HeadlessShell, _) => dir.join("chrome-headless-shell"),
        }
    }
}

/// Which URLs are trusted. Everything the manifest hands out is checked
/// against this before a byte is requested.
#[derive(Debug, Clone)]
pub struct SourcePolicy {
    pub manifest_hosts: Vec<String>,
    pub download_hosts: Vec<String>,
    /// Required prefix of the download path (the CfT bucket).
    pub download_path_prefix: String,
    /// Tests only: allow plain HTTP to a loopback host.
    pub allow_loopback_http: bool,
}

impl SourcePolicy {
    pub fn official() -> Self {
        Self {
            manifest_hosts: vec!["googlechromelabs.github.io".to_string()],
            download_hosts: vec!["storage.googleapis.com".to_string()],
            download_path_prefix: "/chrome-for-testing-public/".to_string(),
            allow_loopback_http: false,
        }
    }

    /// Policy for a local test server on `127.0.0.1`.
    pub fn loopback() -> Self {
        Self {
            manifest_hosts: vec!["127.0.0.1".to_string()],
            download_hosts: vec!["127.0.0.1".to_string()],
            download_path_prefix: "/".to_string(),
            allow_loopback_http: true,
        }
    }

    fn scheme_ok(&self, url: &Url) -> bool {
        match url.scheme() {
            "https" => true,
            "http" => self.allow_loopback_http && url.host_str() == Some("127.0.0.1"),
            _ => false,
        }
    }

    pub fn check_manifest_url(&self, url: &Url) -> Result<(), InstallError> {
        let host_ok = url
            .host_str()
            .is_some_and(|h| self.manifest_hosts.iter().any(|a| a == h));
        if self.scheme_ok(url) && host_ok {
            Ok(())
        } else {
            Err(InstallError::UntrustedUrl(url.to_string()))
        }
    }

    pub fn check_download_url(&self, url: &Url) -> Result<(), InstallError> {
        let host_ok = url
            .host_str()
            .is_some_and(|h| self.download_hosts.iter().any(|a| a == h));
        if self.scheme_ok(url)
            && host_ok
            && url.path().starts_with(&self.download_path_prefix)
            && url.username().is_empty()
            && url.password().is_none()
        {
            Ok(())
        } else {
            Err(InstallError::UntrustedUrl(url.to_string()))
        }
    }

    /// Redirect target check: downloads and manifests share one rule set, so
    /// a redirect may only land on a trusted host over an allowed scheme.
    pub fn redirect_allowed(&self, url: &Url) -> bool {
        self.scheme_ok(url)
            && url.host_str().is_some_and(|h| {
                self.manifest_hosts
                    .iter()
                    .chain(self.download_hosts.iter())
                    .any(|a| a == h)
            })
    }
}

/// Where the publisher manifests are.
#[derive(Debug, Clone)]
pub struct InstallSource {
    pub latest_manifest: String,
    pub known_good_manifest: String,
    pub policy: SourcePolicy,
}

impl InstallSource {
    pub fn official() -> Self {
        Self {
            latest_manifest: "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json".to_string(),
            known_good_manifest: "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json".to_string(),
            policy: SourcePolicy::official(),
        }
    }
}

/// The one archive selected from a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDownload {
    pub version: String,
    pub url: Url,
    pub platform: Platform,
    pub artifact: Artifact,
}

/// `--version` must be a CfT version number, nothing that ends up in a path
/// or URL unchecked.
pub fn validate_version(version: &str) -> Result<(), InstallError> {
    let parts: Vec<&str> = version.split('.').collect();
    let ok = parts.len() == 4
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 6 && p.bytes().all(|b| b.is_ascii_digit()));
    if ok {
        Ok(())
    } else {
        Err(InstallError::InvalidVersion(version.to_string()))
    }
}

/// Pick the archive for `platform`/`artifact` from a manifest.
///
/// `version == None` takes the Stable channel of the last-known-good
/// manifest; `Some(v)` looks `v` up in the known-good manifest. The entry
/// must name exactly this platform, and its URL must be the publisher's
/// canonical `<version>/<platform>/<stem>-<platform>.zip` on a trusted host.
/// No URL is ever built by hand, and no other platform is substituted.
pub fn select_download(
    manifest: &Value,
    version: Option<&str>,
    platform: Platform,
    artifact: Artifact,
    policy: &SourcePolicy,
) -> Result<ResolvedDownload, InstallError> {
    let (resolved_version, downloads) = match version {
        None => {
            let stable = &manifest["channels"]["Stable"];
            let v = stable["version"].as_str().ok_or_else(|| {
                InstallError::ManifestInvalid("no Stable channel version".to_string())
            })?;
            (v.to_string(), &stable["downloads"])
        }
        Some(wanted) => {
            let entry = manifest["versions"]
                .as_array()
                .and_then(|all| all.iter().find(|e| e["version"].as_str() == Some(wanted)))
                .ok_or_else(|| InstallError::NotPublished {
                    platform: platform.manifest_id().to_string(),
                    version: wanted.to_string(),
                    reason: "version is not in the publisher's known-good list".to_string(),
                })?;
            (wanted.to_string(), &entry["downloads"])
        }
    };
    validate_version(&resolved_version)
        .map_err(|_| InstallError::ManifestInvalid(format!("bad version {resolved_version:?}")))?;

    let url_str = downloads[artifact.manifest_key()]
        .as_array()
        .and_then(|all| {
            all.iter()
                .find(|e| e["platform"].as_str() == Some(platform.manifest_id()))
        })
        .and_then(|e| e["url"].as_str())
        .ok_or_else(|| InstallError::NotPublished {
            platform: platform.manifest_id().to_string(),
            version: resolved_version.clone(),
            reason: "the publisher manifest lists no such asset (Linux ARM64 exists from \
                     Chrome for Testing 153.0.8001.0 on)"
                .to_string(),
        })?;

    let url = Url::parse(url_str)
        .map_err(|_| InstallError::ManifestInvalid(format!("bad download URL {url_str:?}")))?;
    policy.check_download_url(&url)?;

    let expected_tail = format!(
        "/{}/{}/{}.zip",
        resolved_version,
        platform.manifest_id(),
        platform.archive_dir(artifact)
    );
    if !url.path().ends_with(&expected_tail) {
        return Err(InstallError::ManifestInvalid(format!(
            "download URL {url} does not match {expected_tail}"
        )));
    }

    Ok(ResolvedDownload {
        version: resolved_version,
        url,
        platform,
        artifact,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest_with(platforms: &[&str]) -> Value {
        let entries = |stem: &str| -> Vec<Value> {
            platforms
                .iter()
                .map(|p| {
                    json!({"platform": p, "url": format!(
                        "https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.80/{p}/{stem}-{p}.zip")})
                })
                .collect()
        };
        json!({"channels": {"Stable": {"version": "156.0.8078.80", "downloads": {
            "chrome": entries("chrome"),
            "chrome-headless-shell": entries("chrome-headless-shell"),
        }}}})
    }

    #[test]
    fn platform_contract() {
        assert_eq!(
            Platform::from_target("macos", "aarch64").unwrap(),
            Platform::MacArm64
        );
        assert_eq!(
            Platform::from_target("linux", "x86_64").unwrap(),
            Platform::LinuxX64
        );
        assert_eq!(
            Platform::from_target("linux", "aarch64").unwrap(),
            Platform::LinuxArm64
        );
        for (os, arch) in [
            ("macos", "x86_64"),
            ("windows", "x86_64"),
            ("windows", "aarch64"),
            ("linux", "riscv64"),
            ("freebsd", "x86_64"),
        ] {
            assert!(
                matches!(
                    Platform::from_target(os, arch),
                    Err(InstallError::UnsupportedPlatform(_))
                ),
                "{os}/{arch}"
            );
        }
        let win = Platform::from_target("windows", "x86_64")
            .unwrap_err()
            .to_string();
        assert!(win.contains("Edge"), "{win}");
    }

    #[test]
    fn linux_arm64_uses_its_own_asset_and_directory() {
        let m = manifest_with(&["linux-arm64", "linux64", "mac-arm64"]);
        let d = select_download(
            &m,
            None,
            Platform::LinuxArm64,
            Artifact::Chrome,
            &SourcePolicy::official(),
        )
        .unwrap();
        assert!(d
            .url
            .as_str()
            .ends_with("/linux-arm64/chrome-linux-arm64.zip"));
        assert_eq!(
            Platform::LinuxArm64.binary_rel_path(Artifact::Chrome),
            PathBuf::from("chrome-linux-arm64/chrome")
        );
    }

    #[test]
    fn missing_platform_is_not_substituted() {
        let m = manifest_with(&["linux64", "mac-arm64"]);
        let err = select_download(
            &m,
            None,
            Platform::LinuxArm64,
            Artifact::Chrome,
            &SourcePolicy::official(),
        )
        .unwrap_err();
        assert!(matches!(err, InstallError::NotPublished { .. }), "{err}");
    }

    #[test]
    fn entry_for_another_platform_is_rejected() {
        // Manifest lies: the linux-arm64 entry points at the linux64 archive.
        let mut m = manifest_with(&["linux-arm64"]);
        m["channels"]["Stable"]["downloads"]["chrome"][0]["url"] = json!(
            "https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.80/linux64/chrome-linux64.zip"
        );
        let err = select_download(
            &m,
            None,
            Platform::LinuxArm64,
            Artifact::Chrome,
            &SourcePolicy::official(),
        )
        .unwrap_err();
        assert!(matches!(err, InstallError::ManifestInvalid(_)), "{err}");
    }

    #[test]
    fn untrusted_hosts_and_schemes_are_rejected() {
        let p = SourcePolicy::official();
        for bad in [
            "http://storage.googleapis.com/chrome-for-testing-public/x",
            "https://evil.example/chrome-for-testing-public/x",
            "https://storage.googleapis.com/other-bucket/x",
            "https://user:pw@storage.googleapis.com/chrome-for-testing-public/x",
            "ftp://storage.googleapis.com/chrome-for-testing-public/x",
            "http://127.0.0.1/chrome-for-testing-public/x",
        ] {
            assert!(
                p.check_download_url(&Url::parse(bad).unwrap()).is_err(),
                "{bad}"
            );
        }
        assert!(p
            .check_download_url(
                &Url::parse("https://storage.googleapis.com/chrome-for-testing-public/x").unwrap()
            )
            .is_ok());
    }

    #[test]
    fn version_argument_is_validated() {
        assert!(validate_version("156.0.8078.80").is_ok());
        for bad in [
            "",
            "156",
            "../../x",
            "156.0.8078.80/../x",
            "a.b.c.d",
            "1.2.3.4.5",
        ] {
            assert!(validate_version(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn exact_version_comes_from_known_good_list() {
        let m = json!({"versions": [{"version": "152.0.7000.0", "downloads": {"chrome": [
            {"platform": "linux64", "url": "https://storage.googleapis.com/chrome-for-testing-public/152.0.7000.0/linux64/chrome-linux64.zip"}]}}]});
        let p = SourcePolicy::official();
        assert!(select_download(
            &m,
            Some("152.0.7000.0"),
            Platform::LinuxX64,
            Artifact::Chrome,
            &p
        )
        .is_ok());
        // Pre-153 has no linux-arm64 asset.
        assert!(matches!(
            select_download(
                &m,
                Some("152.0.7000.0"),
                Platform::LinuxArm64,
                Artifact::Chrome,
                &p
            ),
            Err(InstallError::NotPublished { .. })
        ));
        assert!(matches!(
            select_download(
                &m,
                Some("1.2.3.4"),
                Platform::LinuxX64,
                Artifact::Chrome,
                &p
            ),
            Err(InstallError::NotPublished { .. })
        ));
    }
}
