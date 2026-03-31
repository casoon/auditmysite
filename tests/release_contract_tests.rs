//! Release contract tests.
//!
//! These tests keep the installer and release workflow aligned so published
//! artifacts can actually be installed.

use std::path::PathBuf;

fn read_repo_file(path: &str) -> String {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", full_path.display(), e))
}

#[test]
fn test_installer_and_release_workflow_agree_on_unix_archive_name() {
    let install_sh = read_repo_file("install.sh");
    let release_yml = read_repo_file(".github/workflows/release.yml");

    assert!(
        install_sh.contains("archive_name=\"${BINARY_NAME}-${version}-${platform}.tar.gz\""),
        "install.sh must construct versioned Unix archive names"
    );
    assert!(
        release_yml.contains("ARCHIVE_NAME=\"${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz\""),
        "release workflow must publish versioned Unix archive names"
    );
}

#[test]
fn test_installer_and_release_workflow_agree_on_checksum_name() {
    let install_sh = read_repo_file("install.sh");
    let release_yml = read_repo_file(".github/workflows/release.yml");

    assert!(
        install_sh.contains("checksum_name=\"${archive_name}.sha256\""),
        "install.sh must fetch archive checksum files"
    );
    assert!(
        release_yml.contains("${{ env.ARCHIVE_NAME }}.sha256"),
        "release workflow must upload checksum files next to the archive"
    );
}

#[test]
fn test_readme_documents_curl_installer() {
    let readme = read_repo_file("README.md");

    assert!(readme.contains("curl -fsSL"));
    assert!(readme.contains("install.sh | bash"));
    assert!(readme.contains(".sha256"));
}
