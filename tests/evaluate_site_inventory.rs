//! #777: the collector inventory is derived from the code, not trusted.
//!
//! `COLLECTORS` is a hand-kept list; a list like that looks complete however
//! many collection sites it misses. This test scans `src/` for every place
//! that evaluates script in the browser and requires each file to be either
//! tied to inventoried collectors or on an explicit exemption list with its
//! reason. The number of sites per file is pinned, so a new evaluate call in
//! an already known file fails too: whoever adds it must decide which
//! collector reports its failure, or why none has to.

use auditmysite::audit::collection::collector;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What an evaluate call is, for the scan.
const SITE_MARKERS: &[&str] = &[
    ".evaluate(",
    ".evaluate_on_new_document(",
    "EvaluateParams::builder",
    "CallFunctionOnParams::builder",
];

enum Coverage {
    /// The failure of these sites ends up in these inventoried collectors.
    Collectors(&'static [&'static str]),
    /// No collector reports these sites, for the reason given.
    Exempt(&'static str),
}
use Coverage::{Collectors, Exempt};

const JOURNEYS: &[&str] = &[
    "accessibility_journey.tab_walk",
    "accessibility_journey.skip_link",
    "accessibility_journey.disclosure",
    "accessibility_journey.modal",
    "accessibility_journey.tabs",
    "accessibility_journey.menu",
    "accessibility_journey.form_error",
    "accessibility_journey.add_to_cart",
    "accessibility_journey.quantity_stepper",
    "accessibility_journey.spa_navigation",
];

const RULES: Coverage = Collectors(&["accessibility.rule_execution"]);

/// file → (number of evaluate sites outside test code, coverage)
fn known_sites() -> BTreeMap<&'static str, (usize, Coverage)> {
    BTreeMap::from([
        (
            "src/a11y_journey/form_error.rs",
            (1, Collectors(&["accessibility_journey.form_error"])),
        ),
        ("src/a11y_journey/mod.rs", (2, Collectors(JOURNEYS))),
        (
            "src/a11y_journey/quantity_stepper.rs",
            (2, Collectors(&["accessibility_journey.quantity_stepper"])),
        ),
        (
            "src/a11y_journey/skip_link.rs",
            (2, Collectors(&["accessibility_journey.skip_link"])),
        ),
        (
            "src/a11y_journey/spa_navigation.rs",
            (2, Collectors(&["accessibility_journey.spa_navigation"])),
        ),
        ("src/interaction/focus.rs", (4, Collectors(JOURNEYS))),
        ("src/interaction/pointer.rs", (1, Collectors(JOURNEYS))),
        ("src/interaction/live_regions.rs", (1, Collectors(JOURNEYS))),
        (
            "src/interaction/stability.rs",
            (3, Collectors(&["accessibility.page_stability"])),
        ),
        (
            "src/audit/exclusion.rs",
            (3, Collectors(&["accessibility.exclusion_scope"])),
        ),
        (
            "src/audit/frames.rs",
            (1, Collectors(&["accessibility.frames"])),
        ),
        (
            "src/browser/consent.rs",
            (3, Collectors(&["accessibility.consent_banner"])),
        ),
        (
            "src/seo/technical.rs",
            (1, Collectors(&["seo.service_worker_registrations"])),
        ),
        (
            "src/performance/vitals.rs",
            (
                4,
                Collectors(&[
                    "performance.observers",
                    "performance.cdp_metrics",
                    "performance.navigation_timing",
                    "performance.lcp",
                    "performance.fcp",
                    "performance.cls",
                    "performance.tbt",
                ]),
            ),
        ),
        ("src/wcag/types.rs", (1, RULES)),
        ("src/wcag/rules/focus_not_obscured_minimum.rs", (1, RULES)),
        ("src/wcag/rules/iframe_rules.rs", (1, RULES)),
        ("src/wcag/rules/media_alternative.rs", (1, RULES)),
        ("src/wcag/rules/media_rules.rs", (2, RULES)),
        ("src/wcag/rules/modern_attributes.rs", (1, RULES)),
        ("src/wcag/rules/page_titled.rs", (1, RULES)),
        ("src/wcag/rules/reflow.rs", (1, RULES)),
        ("src/wcag/rules/text_spacing.rs", (1, RULES)),
        (
            "src/audit/collection.rs",
            (
                2,
                Exempt(
                    "the shared JavaScript helpers: every caller records the result under its own CollectorSpec, and CollectionLog::record rejects a spec outside COLLECTORS",
                ),
            ),
        ),
        (
            "src/audit/access_block.rs",
            (
                1,
                Exempt(
                    "challenge-page detection before the audit; when all attempts fail the page counts as regular and the pipeline checks the document status again",
                ),
            ),
        ),
        (
            "src/audit/pipeline/mod.rs",
            (
                2,
                Exempt(
                    "scroll reset and viewport metadata of the screenshot evidence; no result is derived from them",
                ),
            ),
        ),
        (
            "src/accessibility/dom_document.rs",
            (
                4,
                Exempt(
                    "DOM capture of the accessibility pass; a failure aborts the page audit with AXTreeExtractionFailed instead of producing a value",
                ),
            ),
        ),
        (
            "src/accessibility/element_capture.rs",
            (
                3,
                Exempt(
                    "evidence screenshots for findings that already exist; a failed capture drops the picture, not a result",
                ),
            ),
        ),
        (
            "src/accessibility/enrichment.rs",
            (
                2,
                Exempt(
                    "selector and location enrichment of existing violations; a failure leaves the violation without a selector",
                ),
            ),
        ),
        (
            "src/browser/manager.rs",
            (
                5,
                Exempt(
                    "navigation plumbing: readyState polling, window.stop(), meta-refresh lookup; the navigation result itself is the navigation_snapshot collector",
                ),
            ),
        ),
        (
            "src/cli/commands.rs",
            (
                1,
                Exempt(
                    "accname-diff, a measurement tool outside the audit pipeline that reports no score",
                ),
            ),
        ),
    ])
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Evaluate sites of one source file, ignoring comments and the unit tests
/// at the end of the file.
fn sites_in(source: &str) -> usize {
    let code = source.split("#[cfg(test)]\nmod ").next().unwrap_or(source);
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| {
            SITE_MARKERS
                .iter()
                .map(|marker| line.matches(marker).count())
                .sum::<usize>()
        })
        .sum()
}

#[test]
fn every_evaluate_site_is_inventoried_or_exempt_with_a_reason() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    let known = known_sites();

    let mut found = BTreeMap::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let count = sites_in(&std::fs::read_to_string(&file).unwrap());
        if count > 0 {
            found.insert(relative, count);
        }
    }

    let mut problems = Vec::new();
    for (file, count) in &found {
        match known.get(file.as_str()) {
            None => problems.push(format!(
                "{file}: {count} evaluate site(s) neither inventoried nor exempt — \
                 record them under a CollectorSpec and list the file here, or exempt it with a reason"
            )),
            Some((expected, _)) if expected != count => problems.push(format!(
                "{file}: {count} evaluate site(s), the inventory knows {expected} — \
                 decide which collector reports the new site (or why none does)"
            )),
            Some(_) => {}
        }
    }
    for file in known.keys() {
        if !found.contains_key(*file) {
            problems.push(format!(
                "{file}: listed here but has no evaluate site any more — remove the entry"
            ));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn the_inventory_names_real_collectors_and_real_reasons() {
    for (file, (_, coverage)) in known_sites() {
        match coverage {
            Collectors(ids) => {
                assert!(!ids.is_empty(), "{file}: no collector named");
                for id in ids {
                    assert!(collector(id).is_some(), "{file}: {id} is not in COLLECTORS");
                }
            }
            Exempt(reason) => assert!(
                reason.split_whitespace().count() >= 8,
                "{file}: an exemption needs a real reason, got {reason:?}"
            ),
        }
    }
}

#[test]
fn the_scan_sees_a_site_that_is_not_inventoried() {
    // The scan itself: a fresh evaluate call is counted, a comment and the
    // unit-test tail are not.
    assert_eq!(sites_in("let r = page.evaluate(js).await;"), 1);
    assert_eq!(sites_in("// page.evaluate(js)\nfn f() {}"), 0);
    assert_eq!(
        sites_in("fn f() {}\n#[cfg(test)]\nmod tests { fn t() { page.evaluate(js); } }"),
        0
    );
    assert_eq!(
        sites_in("let a = EvaluateParams::builder();\nlet b = CallFunctionOnParams::builder();"),
        2
    );
}
