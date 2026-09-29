//! Golden-render harness (plan 66, WP0).
//!
//! Insta snapshots pin report *shapes*; this renders the full report *text*
//! from a frozen set of cached audits so a refactoring PR can prove it changes
//! no output: render on `main`, render on the branch, `diff -r` must be empty.
//!
//! Input (`AUDITMYSITE_GOLDEN_DIR`): one subdirectory per site holding a copy
//! of a cache entry as written by `save_artifacts`
//! (`~/.auditmysite/cache/<domain>/<hash>/v<VERSION>/`): `report.json`,
//! `snapshot.json` and `audit.json` are read; other files are ignored.
//!
//! Output (`AUDITMYSITE_GOLDEN_OUT`), deterministic names:
//!   `<site>/report.json`        normalized single-page JSON (as `-f json`)
//!   `<site>/report.<lang>.typ`  PDF Typst source (as `--debug-typ`), de + en
//!   `batch/batch.json`          batch JSON over all sites (as batch `-f json`)
//!   `batch/batch.<lang>.typ`    batch PDF Typst source, de + en
//!
//! Each cached report is rehydrated exactly as the CLI does on a cache hit
//! (`hydrate_cached_report` with the run language). Two values differ by design
//! between runs or checkouts and are replaced by placeholders: the batch render
//! date (`<RENDER-DATE>`) and the JSON `build_id` (`<BUILD-ID>`). Cached page
//! screenshots are stripped, so no temp screenshot paths (`ams-*-<ts>.png`)
//! reach the Typst source.
//!
//! Usage (see also `scripts/golden-diff.sh`):
//!   AUDITMYSITE_GOLDEN_DIR=reports/golden AUDITMYSITE_GOLDEN_OUT=/tmp/golden-main \
//!     cargo test --release --all-features --test golden_render -- --ignored

#![cfg(feature = "pdf")]

use std::fs;
use std::path::{Path, PathBuf};

use auditmysite::audit::{
    compute_batch_verdict, compute_verdict, hydrate_cached_report, normalize, NormalizedReport,
    SampleMetadata, SnapshotArtifact,
};
use auditmysite::output::report_model::ReportConfig;
use auditmysite::output::{generate_batch_typ, generate_typ, UnifiedReport};
use auditmysite::{AuditReport, BatchReport};

const LOCALES: [&str; 2] = ["de", "en"];
/// The CLI's default `--lang`; the JSON outputs are rendered with it.
const JSON_LOCALE: &str = "de";

struct CachedSite {
    name: String,
    report: AuditReport,
    snapshot: SnapshotArtifact,
    audit: NormalizedReport,
}

fn env_dir(name: &str) -> PathBuf {
    PathBuf::from(
        std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set (see module docs)")),
    )
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn load_sites(dir: &Path) -> Vec<CachedSite> {
    let mut site_dirs: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.is_dir())
        .collect();
    site_dirs.sort();
    site_dirs
        .into_iter()
        .map(|path| CachedSite {
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            report: read_json(&path.join("report.json")),
            snapshot: read_json(&path.join("snapshot.json")),
            audit: read_json(&path.join("audit.json")),
        })
        .collect()
}

/// The report as the CLI renders it on a cache hit with `--lang <locale>`.
fn hydrated(site: &CachedSite, locale: &str) -> AuditReport {
    let mut report = site.report.clone();
    hydrate_cached_report(&mut report, &site.snapshot, locale);
    report
}

fn report_config(locale: &str) -> ReportConfig {
    ReportConfig {
        locale: locale.to_string(),
        ..Default::default()
    }
}

/// Replace the batch render date (`output/builder/batch.rs` stamps the cover,
/// footer and scope table with the current UTC date, en and de format) with a
/// fixed placeholder. Single reports use the audit timestamp and need nothing.
fn normalize_render_date(text: &str) -> String {
    let now = chrono::Utc::now();
    ["%Y-%m-%d", "%d.%m.%Y"]
        .iter()
        .fold(text.to_string(), |text, format| {
            text.replace(&now.format(format).to_string(), "<RENDER-DATE>")
        })
}

/// Replace the JSON `build_id` (git commit + dirty flag of the rendering
/// checkout), which by design differs between the two checkouts compared.
fn normalize_build_id(json: &str) -> String {
    const KEY: &str = "\"build_id\": \"";
    let Some(start) = json.find(KEY).map(|i| i + KEY.len()) else {
        return json.to_string();
    };
    let end = start + json[start..].find('"').expect("closing quote of build_id");
    format!("{}<BUILD-ID>{}", &json[..start], &json[end..])
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).expect("create output dir");
    fs::write(path, content).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

#[test]
#[ignore = "needs AUDITMYSITE_GOLDEN_DIR/AUDITMYSITE_GOLDEN_OUT; run via scripts/golden-diff.sh"]
fn golden_render() {
    let input = env_dir("AUDITMYSITE_GOLDEN_DIR");
    let out = env_dir("AUDITMYSITE_GOLDEN_OUT");
    let sites = load_sites(&input);
    assert!(
        !sites.is_empty(),
        "no site directories in {}",
        input.display()
    );

    for site in &sites {
        let site_out = out.join(&site.name);
        let verdict = compute_verdict(&site.audit, &Default::default());

        let report = hydrated(site, JSON_LOCALE);
        let normalized = normalize(&report);
        let json = UnifiedReport::single(&normalized, &report)
            .with_verdict(&verdict)
            .to_json(true)
            .expect("single JSON");
        write(&site_out.join("report.json"), &normalize_build_id(&json));

        for locale in LOCALES {
            let typ = generate_typ(&hydrated(site, locale), &report_config(locale))
                .unwrap_or_else(|e| panic!("{} typ ({locale}): {e}", site.name));
            write(&site_out.join(format!("report.{locale}.typ")), &typ);
        }
    }

    // One batch over all sites, shaped like a `--url-file` run that audited
    // every listed URL.
    let batch_for = |locale: &str| {
        let reports: Vec<AuditReport> = sites.iter().map(|s| hydrated(s, locale)).collect();
        let total_duration_ms = reports.iter().map(|r| r.duration_ms).sum();
        BatchReport::from_reports(reports, Vec::new(), total_duration_ms).with_sample(
            SampleMetadata {
                source: "url_file".to_string(),
                total_discovered: sites.len(),
                audited: sites.len(),
                sample_limit: None,
                selection: "all".to_string(),
                is_sample: false,
            },
        )
    };
    let batch_out = out.join("batch");

    let batch = batch_for(JSON_LOCALE);
    let verdict = compute_batch_verdict(&batch.summary, &Default::default());
    let json = UnifiedReport::batch(&batch)
        .with_verdict(&verdict)
        .to_json(true)
        .expect("batch JSON");
    write(&batch_out.join("batch.json"), &normalize_build_id(&json));

    for locale in LOCALES {
        let typ = generate_batch_typ(&batch_for(locale), &report_config(locale))
            .unwrap_or_else(|e| panic!("batch typ ({locale}): {e}"));
        write(
            &batch_out.join(format!("batch.{locale}.typ")),
            &normalize_render_date(&typ),
        );
    }
}
