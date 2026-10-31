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
    "EvaluateParams::new(",
    "CallFunctionOnParams::new(",
    "AddScriptToEvaluateOnNewDocumentParams::new(",
];

enum Coverage {
    /// The failure of these sites ends up in these inventoried collectors, and
    /// the file itself names one of their `CollectorSpec` statics.
    Collectors(&'static [&'static str]),
    /// As `Collectors`, but the record is written elsewhere: one of the
    /// listed files must name the file's module (`stem::`).
    Via(&'static [&'static str], &'static [&'static str]),
    /// No collector reports these sites, for the reason given.
    Exempt(&'static str),
}
use Coverage::{Collectors, Exempt, Via};

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

const RULE_EXECUTION: &[&str] = &["accessibility.rule_execution"];
const RULES: Coverage = Via(RULE_EXECUTION, &["src/wcag/rules/mod.rs"]);

/// file → (number of evaluate sites outside test code, coverage)
fn known_sites() -> BTreeMap<&'static str, (usize, Coverage)> {
    BTreeMap::from([
        (
            "src/a11y_journey/form_error.rs",
            (1, Via(&["accessibility_journey.form_error"], &["src/a11y_journey/mod.rs"])),
        ),
        ("src/a11y_journey/mod.rs", (2, Via(JOURNEYS, &["src/audit/pipeline/mod.rs"]))),
        (
            "src/a11y_journey/quantity_stepper.rs",
            (2, Via(&["accessibility_journey.quantity_stepper"], &["src/a11y_journey/mod.rs"])),
        ),
        (
            "src/a11y_journey/skip_link.rs",
            (2, Via(&["accessibility_journey.skip_link"], &["src/a11y_journey/mod.rs"])),
        ),
        (
            "src/a11y_journey/spa_navigation.rs",
            (2, Via(&["accessibility_journey.spa_navigation"], &["src/a11y_journey/mod.rs"])),
        ),
        ("src/interaction/focus.rs", (4, Via(JOURNEYS, &["src/a11y_journey/tab_walk.rs"]))),
        ("src/interaction/pointer.rs", (1, Via(JOURNEYS, &["src/a11y_journey/menu_journey.rs"]))),
        ("src/interaction/live_regions.rs", (1, Via(JOURNEYS, &["src/a11y_journey/add_to_cart.rs"]))),
        (
            "src/interaction/stability.rs",
            (3, Via(&["accessibility.page_stability"], &["src/audit/pipeline/mod.rs"])),
        ),
        (
            "src/audit/exclusion.rs",
            (3, Via(&["accessibility.exclusion_scope"], &["src/audit/pipeline/mod.rs"])),
        ),
        (
            "src/audit/frames.rs",
            (1, Via(&["accessibility.frames"], &["src/audit/pipeline/mod.rs"])),
        ),
        (
            "src/browser/consent.rs",
            (3, Via(&["accessibility.consent_banner"], &["src/audit/pipeline/mod.rs"])),
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
        ("src/wcag/types.rs", (1, Via(RULE_EXECUTION, &["src/wcag/engine.rs"]))),
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
                7,
                Exempt(
                    "navigation plumbing: readyState polling, window.stop(), meta-refresh lookup and the webdriver/DNT injections on a new page; the navigation result itself is the navigation_snapshot collector",
                ),
            ),
        ),
        (
            "src/browser/state.rs",
            (
                1,
                Exempt(
                    "registers the declared localStorage on new documents; a failure is logged, and the declared state is part of the measurement profile, not a measured value",
                ),
            ),
        ),
        (
            "src/display/mod.rs",
            (
                2,
                Exempt(
                    "display-mode observer and stored choice injected before navigation; a failure leaves the site default, which DisplayModesInfo::active_mode reports",
                ),
            ),
        ),
        (
            "src/best_practices/console_errors.rs",
            (1, Collectors(&["best_practices.console"])),
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

/// The code of a source file without its unit-test module: everything before
/// the first `#[cfg(test)]` that is directly followed by a module (an item
/// with the attribute in the middle of the file does not end the scan).
fn non_test_code(source: &str) -> &str {
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut offset = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "#[cfg(test)]" {
            let next = lines.get(i + 1).map_or("", |l| l.trim_start());
            let is_module = ["mod ", "pub mod ", "pub(crate) mod "]
                .iter()
                .any(|prefix| next.starts_with(prefix));
            if is_module {
                return &source[..offset];
            }
        }
        offset += line.len();
    }
    source
}

/// Evaluate sites of one source file, ignoring comments and the unit tests
/// at the end of the file.
fn sites_in(source: &str) -> usize {
    non_test_code(source)
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| {
            SITE_MARKERS
                .iter()
                .map(|marker| line.matches(marker).count())
                .sum::<usize>()
        })
        .sum()
}

fn all_sources() -> BTreeMap<String, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    files
        .into_iter()
        .map(|file| {
            let relative = file
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let code = non_test_code(&std::fs::read_to_string(&file).unwrap()).to_string();
            (relative, code)
        })
        .collect()
}

fn mentions(code: &str, token: &str) -> bool {
    let is_name_char = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices(token).any(|(at, _)| {
        let before_ok = !token.starts_with(is_name_char)
            || !code[..at].chars().next_back().is_some_and(is_name_char);
        let after_ok = !token.ends_with(is_name_char)
            || !code[at + token.len()..]
                .chars()
                .next()
                .is_some_and(is_name_char);
        before_ok && after_ok
    })
}

/// The name of the `static` that declares collector `id`, found by looking
/// for the id literal right behind a `CollectorSpec` declaration.
fn static_of(sources: &BTreeMap<String, String>, id: &str) -> Option<String> {
    let literal = format!("\"{id}\"");
    for code in sources.values() {
        for (at, _) in code.match_indices(&literal) {
            let before = &code[..at];
            let window = &before[before.len().saturating_sub(300)..];
            let Some(decl) = window.rfind("CollectorSpec") else {
                continue;
            };
            let head = &window[..decl];
            let keyword = head.rfind("static ").or_else(|| head.rfind("const "))?;
            let name: String = head[keyword..]
                .split_whitespace()
                .nth(1)?
                .trim_end_matches(':')
                .to_string();
            return Some(name);
        }
    }
    None
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
            Collectors(ids) | Via(ids, _) => {
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

/// The mapping file → collector is checked against the code, not only
/// declared: the collector's `CollectorSpec` static is used by code that
/// records it, and the evaluating file either names that static itself or is
/// reached by a file that does (`Via`).
#[test]
fn every_declared_file_to_collector_mapping_holds_in_the_code() {
    let sources = all_sources();
    let mut problems = Vec::new();
    for (file, (_, coverage)) in known_sites() {
        let (ids, via): (&[&str], Option<&[&str]>) = match coverage {
            Collectors(ids) => (ids, None),
            Via(ids, via) => (ids, Some(via)),
            Exempt(_) => continue,
        };
        let code = &sources[file];
        let mut names_own = false;
        for id in ids {
            let Some(name) = static_of(&sources, id) else {
                problems.push(format!("{file}: no CollectorSpec static declares {id}"));
                continue;
            };
            // The declaration, the inventory list and at least one use that
            // records or maps it.
            let users: Vec<_> = sources
                .iter()
                .filter(|(path, code)| {
                    path.as_str() != "src/audit/collection.rs" && mentions(code, &name)
                })
                .map(|(path, code)| (path, code.matches(name.as_str()).count()))
                .collect();
            let uses: usize = users.iter().map(|(_, n)| n).sum();
            if uses < 2 {
                problems.push(format!(
                    "{file}: {id} ({name}) is declared but never used to record anything"
                ));
            }
            names_own |= mentions(code, &name);
        }
        match via {
            None => {
                if !names_own {
                    problems.push(format!(
                        "{file}: declared as covered by {ids:?}, but the file names none of their statics"
                    ));
                }
            }
            Some(files) => {
                let stem = Path::new(file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|stem| {
                        if stem == "mod" {
                            Path::new(file)
                                .parent()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or(stem)
                        } else {
                            stem
                        }
                    })
                    .unwrap();
                let reached = files
                    .iter()
                    .any(|via| mentions(&sources[*via], &format!("{stem}::")));
                if !reached {
                    problems.push(format!(
                        "{file}: not reached by {files:?} (none mentions `{stem}::`)"
                    ));
                }
            }
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn the_mapping_check_rejects_a_mapping_the_code_does_not_support() {
    let sources = all_sources();
    // A real collector that nothing in this file touches.
    let name = static_of(&sources, "seo.service_worker_registrations").unwrap();
    assert!(!mentions(&sources["src/audit/frames.rs"], &name));
    // `mentions` matches whole names only.
    assert!(mentions("use X::LCP;", "LCP"));
    assert!(!mentions("use X::LCP_BUDGET;", "LCP"));
    assert!(static_of(&sources, "no.such.collector").is_none());
}

#[test]
fn the_scan_counts_the_script_injection_marker_and_ends_only_at_a_test_module() {
    assert_eq!(
        sites_in("page.execute(AddScriptToEvaluateOnNewDocumentParams::new(js)).await;"),
        1
    );
    // An attribute on an item does not hide the code behind it.
    assert_eq!(
        sites_in("#[cfg(test)]\nfn helper() {}\nfn f() { page.evaluate(js); }\n#[cfg(test)]\nmod tests {}"),
        1
    );
    assert_eq!(
        sites_in("#[cfg(test)]\npub(crate) mod cases { fn t() { page.evaluate(js); } }"),
        0
    );
}
