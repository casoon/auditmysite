//! WCAG Rule Coverage Tests — browser-free
//!
//! Verifies that every rule registered in the engine has:
//! - Valid rule metadata (non-empty id, name, axe_id)
//! - A defined WCAG level
//! - A help URL
//!
//! Also verifies that the engine's `RuleFilterConfig` works correctly.
//!
//! Run with:
//!   cargo test --test wcag_coverage

use auditmysite::accessibility::{AXNode, AXProperty, AXTree, AXValue};
use auditmysite::cli::WcagLevel;
use auditmysite::wcag::engine::{check_all_with_config, RuleFilterConfig};
use auditmysite::wcag::rules::{
    check_accessible_name, check_focus_visible, check_keyboard, check_link_purpose,
    check_text_alternatives,
};
use auditmysite::wcag::WcagResults;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_tree() -> AXTree {
    AXTree::new()
}

fn minimal_tree() -> AXTree {
    AXTree::from_nodes(vec![AXNode {
        node_id: "root".to_string(),
        ignored: false,
        ignored_reasons: vec![],
        role: Some("RootWebArea".to_string()),
        name: Some("Coverage Test Page".to_string()),
        name_source: None,
        description: None,
        value: None,
        properties: vec![AXProperty {
            name: "lang".to_string(),
            value: AXValue::String("en".to_string()),
        }],
        child_ids: vec![],
        parent_id: None,
        backend_dom_node_id: None,
    }])
}

// ---------------------------------------------------------------------------
// Each rule function must run without panicking on an empty tree
// and on a minimal well-formed tree.
// ---------------------------------------------------------------------------

macro_rules! rule_smoke_test {
    ($test_name:ident, $fn:ident) => {
        #[test]
        fn $test_name() {
            // Must not panic on empty input
            let r1 = $fn(&empty_tree());
            // violations + passes must be non-negative (trivially true for usize, just checks compilation)
            let _ = r1.violations.len() + r1.passes;

            // Must not panic on a minimal valid tree
            let r2 = $fn(&minimal_tree());
            let _ = r2.violations.len() + r2.passes;
        }
    };
}

rule_smoke_test!(smoke_check_text_alternatives, check_text_alternatives);
rule_smoke_test!(smoke_check_keyboard, check_keyboard);
rule_smoke_test!(smoke_check_link_purpose, check_link_purpose);
rule_smoke_test!(smoke_check_accessible_name, check_accessible_name);
// non_text_contrast.rs was replaced by non_text_contrast_css.rs (a `_with_page`
// CDP-based check) — like the other `_with_page` rules, it has no smoke test
// here (this file is browser-free/AXTree-only); it has its own unit tests.
rule_smoke_test!(smoke_check_focus_visible, check_focus_visible);

// ---------------------------------------------------------------------------
// RuleFilterConfig — engine filtering logic
// ---------------------------------------------------------------------------

#[test]
fn test_filter_config_default_runs_all_rules() {
    let filter = RuleFilterConfig::default();
    assert!(filter.should_run("image-alt"));
    assert!(filter.should_run("html-has-lang"));
    assert!(filter.should_run("link-name"));
    assert!(filter.should_run("aria-roles"));
    assert!(filter.should_run("heading-order"));
}

#[test]
fn test_filter_config_disabled_rule_skipped() {
    let filter = RuleFilterConfig {
        disabled_rules: vec!["image-alt".to_string()],
        enabled_only_rules: vec![],
    };
    assert!(
        !filter.should_run("image-alt"),
        "Disabled rule should not run"
    );
    assert!(
        filter.should_run("html-has-lang"),
        "Non-disabled rule should still run"
    );
}

#[test]
fn test_filter_config_enabled_only_restricts_to_list() {
    let filter = RuleFilterConfig {
        disabled_rules: vec![],
        enabled_only_rules: vec!["image-alt".to_string(), "html-has-lang".to_string()],
    };
    assert!(filter.should_run("image-alt"));
    assert!(filter.should_run("html-has-lang"));
    assert!(
        !filter.should_run("link-name"),
        "Rule not in enabled_only list should not run"
    );
    assert!(!filter.should_run("aria-roles"));
}

#[test]
fn test_filter_disabled_rule_does_not_produce_violations() {
    // Tree with missing image alt — would normally produce 1.1.1 violations
    let tree = AXTree::from_nodes(vec![AXNode {
        node_id: "1".to_string(),
        ignored: false,
        ignored_reasons: vec![],
        role: Some("image".to_string()),
        name: None,
        name_source: None,
        description: None,
        value: None,
        properties: vec![],
        child_ids: vec![],
        parent_id: None,
        backend_dom_node_id: None,
    }]);

    let filter = RuleFilterConfig {
        disabled_rules: vec!["image-alt".to_string()],
        enabled_only_rules: vec![],
    };

    let results = check_all_with_config(&tree, WcagLevel::A, &filter);
    let alt_violations: Vec<_> = results
        .violations
        .iter()
        .filter(|v| v.rule == "1.1.1")
        .collect();
    assert!(
        alt_violations.is_empty(),
        "Disabled rule should produce no violations"
    );
}

#[test]
fn test_enabled_only_runs_exactly_those_rules() {
    // Ein Baum, auf den ohne Filter zwei Regeln anschlagen: 1.1.1 (Bild ohne
    // Alternativtext) und `keyboard` (der seitenweite `UNTESTED`-Vermerk zur
    // Tastaturfalle, `keyboard-trap`).
    //
    // Davor war das zweite Beispiel bypass (keine einzige Ueberschrift); die
    // Seite ohne Ueberschriften laeuft seit #694 als `headings/none` im
    // geteilten Bestand.
    //
    // Davor war es landmark-main-present; die fehlende
    // main-Landmark laeuft seit #690 als `landmarks/main-missing` im
    // geteilten Bestand.
    //
    // Davor war das zweite Beispiel 2.4.2 (Dokument ohne Titel); 2.4.2 laeuft
    // inzwischen nur noch als DOM-Seitenregel (`check_page_titled_with_page`).
    //
    // Frueher war das zweite Beispiel 3.1.1 (fehlendes lang). Seit 3.1.1 als
    // geteilte Regel gegen den DOM laeuft (`document/lang-missing`, siehe
    // `wcag::shared`), kann das AX-basierte `check_all_with_config` dazu
    // nichts mehr finden -- der Test haette dann nur noch bestaetigt, dass
    // die Regel verschwunden ist, statt die Filterlogik zu pruefen.
    let tree = AXTree::from_nodes(vec![
        AXNode {
            node_id: "root".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("RootWebArea".to_string()),
            name: None, // kein Titel -> 2.4.2
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec!["img1".to_string(), "p1".to_string()],
            parent_id: None,
            backend_dom_node_id: None,
        },
        AXNode {
            node_id: "img1".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".to_string()),
            name: None, // missing alt
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("root".to_string()),
            backend_dom_node_id: None,
        },
        AXNode {
            node_id: "p1".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("paragraph".to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("root".to_string()),
            backend_dom_node_id: None,
        },
    ]);

    // Ohne Filter fallen beide an. Ohne diese Haelfte waere die Zusicherung
    // unten tautologisch: Sie wuerde auch halten, wenn `keyboard` hier gar
    // nicht anschlaegt.
    let ungefiltert = check_all_with_config(&tree, WcagLevel::A, &RuleFilterConfig::default());
    assert!(
        ungefiltert.violations.iter().any(|v| v.rule == "1.1.1"),
        "1.1.1 muss ohne Filter anfallen"
    );
    assert!(
        ungefiltert
            .not_testables
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("keyboard-trap")),
        "keyboard-trap muss ohne Filter anfallen"
    );

    // Mit enabled_only bleibt genau die eine Regel uebrig.
    let filter = RuleFilterConfig {
        disabled_rules: vec![],
        enabled_only_rules: vec!["image-alt".to_string()],
    };
    let results = check_all_with_config(&tree, WcagLevel::A, &filter);

    assert!(
        results.violations.iter().any(|v| v.rule == "1.1.1"),
        "1.1.1 steht auf der enabled_only-Liste und muss laufen"
    );
    assert!(
        !results
            .not_testables
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("keyboard-trap")),
        "keyboard steht nicht auf der enabled_only-Liste und muss unterdrueckt sein"
    );
}

// ---------------------------------------------------------------------------
// Level gating — AA rules only run when level >= AA
// ---------------------------------------------------------------------------

#[test]
fn test_level_a_does_not_run_aa_rules() {
    // A tree an AA-only rule reacts to: 2.4.7 Focus Visible notes a page with
    // no focusable element at all (a review hint since 2.4.7 has nothing to
    // apply to there). (1.4.4, 1.4.11 and most recently 1.3.5 used to be the
    // example here; all three are now DOM/CDP `_with_page` rules with their
    // own level-gating coverage in page_rules.rs, so this AXTree-only engine
    // test needs a still-AXTree-based AA rule.)
    let node = |id: &str, role: &str| AXNode {
        node_id: id.to_string(),
        ignored: false,
        ignored_reasons: vec![],
        role: Some(role.to_string()),
        name: None,
        name_source: None,
        description: None,
        value: None,
        properties: vec![],
        child_ids: vec![],
        parent_id: None,
        backend_dom_node_id: None,
    };
    let tree = AXTree::from_nodes(vec![
        node("1", "RootWebArea"),
        node("2", "heading"),
        node("3", "paragraph"),
        node("4", "paragraph"),
        node("5", "generic"),
        node("6", "generic"),
    ]);

    let results_a = check_all_with_config(&tree, WcagLevel::A, &RuleFilterConfig::default());
    let results_aa = check_all_with_config(&tree, WcagLevel::AA, &RuleFilterConfig::default());

    let focus_visible = |r: &WcagResults| r.warnings.iter().any(|v| v.rule == "2.4.7");
    assert!(
        !focus_visible(&results_a),
        "Level A should not check 2.4.7 (AA rule)"
    );
    assert!(focus_visible(&results_aa), "Level AA should check 2.4.7");
}

#[test]
fn test_nodes_checked_counter_is_populated() {
    let tree = AXTree::from_nodes(vec![AXNode {
        node_id: "1".to_string(),
        ignored: false,
        ignored_reasons: vec![],
        role: Some("image".to_string()),
        name: Some("Logo".to_string()),
        name_source: None,
        description: None,
        value: None,
        properties: vec![],
        child_ids: vec![],
        parent_id: None,
        backend_dom_node_id: None,
    }]);
    let results = check_all_with_config(&tree, WcagLevel::A, &RuleFilterConfig::default());
    assert!(
        results.nodes_checked > 0,
        "nodes_checked should be non-zero after checking a non-empty tree"
    );
}

// ---------------------------------------------------------------------------
// #QA-032 — guard against the CDP-property-name mismatch bug class
//
// A large share of Phase 2's audit findings (#QA-001, #QA-006, #QA-030) came
// from tree-based rules reading AX property names that CDP never emits (most
// often the `aria-`-prefixed HTML attribute name instead of the real,
// unprefixed CDP property name). Every occurrence was invisible to its own
// unit tests because the test fixtures were hand-built with the same wrong
// name. This test scans the actual rule source for every
// `get_property_str`/`get_property_bool`/`get_property_int`/`has_property`/
// `get_property_idref`/`get_property_idrefs` call and asserts the string
// literal is either a real, empirically-or-spec-confirmed CDP property name,
// or an explicitly documented, reasoned exception — so a new instance of
// this bug class can't be introduced silently.
// ---------------------------------------------------------------------------

/// Real CDP `Accessibility.AXPropertyName` values (Chrome DevTools Protocol),
/// plus a few properties this codebase confirmed Chrome exposes in practice
/// beyond the strictly documented enum (empirically verified 2026-07-13 —
/// see plans/quality-audit-backlog.md's QA-032 entry for the repro).
const REAL_CDP_PROPERTIES: &[&str] = &[
    // Value-type AX properties
    "busy",
    "disabled",
    "editable",
    "focusable",
    "focused",
    "hidden",
    "hiddenRoot",
    "invalid",
    "keyshortcuts",
    "settable",
    "roledescription",
    "live",
    "atomic",
    "relevant",
    "root",
    "autocomplete",
    "hasPopup",
    "level",
    "multiselectable",
    "orientation",
    "multiline",
    "readonly",
    "required",
    "valuemin",
    "valuemax",
    "valuenow",
    "valuetext",
    "checked",
    "expanded",
    "modal",
    "pressed",
    "selected",
    // Relationship (idref / idref-list) AX properties
    "activedescendant",
    "controls",
    "describedby",
    "details",
    "errormessage",
    "flowto",
    "labelledby",
    "owns",
    // Empirically confirmed beyond the documented enum, this session:
    // - "language": populated on RootWebArea even without an author lang
    //   attribute (the root cause of #QA-001's false negative).
    // - "htmlTag": confirmed live via summary_name.rs (<details>/<summary>,
    //   since #692 a shared rule) and instructions.rs (<fieldset>, since #693
    //   a shared rule); read by landmark_granular.rs until #694.
    // - "url": a link's href target (landmark_granular.rs / region.rs until
    //   #694).
    "language",
    "htmlTag",
    "url",
];

/// Known residual reads of a non-real property name: documented, low
/// priority, intentionally tolerated rather than silently broken. Each is a
/// harmless redundant branch alongside a working primary check, or a
/// tracked, deliberately-deferred item — not a silent gap.
///
/// Empty since #693: both entries (`placeholder` in instructions.rs,
/// `title` in label_title_only.rs) left with their files for the shared
/// `forms/*` rules, which read the attributes from the DOM.
const KNOWN_EXCEPTIONS: &[(&str, &str)] = &[];

/// Scan `content` for `.<method>("<name>")` calls and return every extracted
/// `name`. Deliberately simple substring scanning (no regex dependency) —
/// this only needs to catch literal string-argument calls, which is the
/// entire universe of how these accessors are used in this codebase.
///
/// Known blind spot: a call like `["a", "b"].iter().any(|n|
/// node.get_property_str(n))` passes a bound *variable*, not a literal, so
/// it isn't caught here. One real instance of this pattern was found and
/// fixed by hand while building this test (label_title_only.rs); if a
/// similar pattern reappears, it won't be caught automatically.
fn extract_property_literals(content: &str) -> Vec<String> {
    const METHODS: &[&str] = &[
        "get_property_str",
        "get_property_bool",
        "get_property_int",
        "get_property_idrefs",
        "get_property_idref",
        "has_property",
    ];

    let mut found = Vec::new();
    for method in METHODS {
        let pattern = format!(".{method}(\"");
        let mut start = 0;
        while let Some(rel_pos) = content[start..].find(&pattern) {
            let quote_start = start + rel_pos + pattern.len();
            if let Some(rel_end) = content[quote_start..].find('"') {
                found.push(content[quote_start..quote_start + rel_end].to_string());
                start = quote_start + rel_end;
            } else {
                break;
            }
        }
    }
    found
}

#[test]
fn all_get_property_calls_use_real_cdp_property_names() {
    let rules_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/wcag/rules");
    let mut violations: Vec<String> = Vec::new();
    let mut seen_exceptions: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for entry in std::fs::read_dir(&rules_dir).expect("src/wcag/rules must exist") {
        let entry = entry.expect("readable dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("readable rule source file");
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        for literal in extract_property_literals(&content) {
            if REAL_CDP_PROPERTIES.contains(&literal.as_str()) {
                continue;
            }
            if let Some((name, _reason)) =
                KNOWN_EXCEPTIONS.iter().find(|(name, _)| *name == literal)
            {
                seen_exceptions.insert(name);
                continue;
            }
            violations.push(format!("{file_name}: \"{literal}\""));
        }
    }

    assert!(
        violations.is_empty(),
        "Found get_property_*/has_property calls using a name that is neither a \
         known-real CDP AXPropertyName nor a documented exception (#QA-032). \
         Either the name is dead (CDP never emits it — check via a live fixture \
         before assuming) or REAL_CDP_PROPERTIES/KNOWN_EXCEPTIONS in this test \
         needs updating:\n{}",
        violations.join("\n")
    );

    // If every documented exception has since been fixed, this test should
    // be tightened by removing the stale entry — surface that opportunity
    // rather than letting the exceptions list grow stale silently.
    let stale: Vec<&str> = KNOWN_EXCEPTIONS
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| !seen_exceptions.contains(name))
        .collect();
    assert!(
        stale.is_empty(),
        "KNOWN_EXCEPTIONS entries no longer found in src/wcag/rules/ — remove \
         them from the list: {stale:?}"
    );
}

// ---------------------------------------------------------------------------
// #QA-032 item 4 / #QA-009 — guard against silent severity-escalation
// collisions in the taxonomy group-key mechanism.
//
// `audit::normalized::wcag_group_key` groups a violation by its own axe id
// when the taxonomy has a dedicated entry for it, otherwise by the raw WCAG
// success criterion (e.g. "4.1.2"). Rule files that share a raw SC and lack
// a dedicated entry get merged into one normalized finding whose severity is
// the *max* across the merged group (#QA-009) — so a new rule added under an
// already-crowded SC with a much higher severity than its neighbors silently
// escalates every finding in that group. This test parses every
// `RuleMetadata` declaration in `src/wcag/rules/`, computes each rule's
// effective group key the same way the real grouping mechanism does, and
// asserts that any group with more than one member has a single, consistent
// severity (and WCAG level) — unless the group is a documented, reviewed
// exception.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct RuleMetaEntry {
    file: String,
    const_name: String,
    id: String,
    severity: String,
    level: String,
    axe_id: String,
}

/// Parse every `NAME: RuleMetadata = RuleMetadata { ... };` block in `content`.
/// Simple substring scanning (no regex dependency), consistent with
/// `extract_property_literals` above. Relies on the codebase's consistent
/// field layout (id/name/level/severity/description/help_url/axe_id/tags),
/// verified by sampling before writing this test.
fn extract_rule_metadata_entries(file: &str, content: &str) -> Vec<RuleMetaEntry> {
    const MARKER: &str = ": RuleMetadata = RuleMetadata {";
    let mut entries = Vec::new();
    let mut search_from = 0;

    while let Some(rel_marker) = content[search_from..].find(MARKER) {
        let marker_pos = search_from + rel_marker;
        let const_name = content[..marker_pos]
            .rsplit(|c: char| c.is_whitespace())
            .find(|s| !s.is_empty())
            .unwrap_or("")
            .to_string();
        let block_start = marker_pos + MARKER.len();
        let Some(rel_end) = content[block_start..].find("};") else {
            break;
        };
        let block = &content[block_start..block_start + rel_end];
        search_from = block_start + rel_end + 2;

        let field_str = |field: &str| -> Option<String> {
            let pat = format!("{field}: \"");
            let start = block.find(&pat)? + pat.len();
            let end = block[start..].find('"')?;
            Some(block[start..start + end].to_string())
        };
        let field_enum = |field: &str, prefix: &str| -> Option<String> {
            let pat = format!("{field}: {prefix}");
            let start = block.find(&pat)? + pat.len();
            let end = block[start..].find(|c: char| c == ',' || c.is_whitespace())?;
            Some(block[start..start + end].trim_end_matches(',').to_string())
        };

        if let (Some(id), Some(severity), Some(level), Some(axe_id)) = (
            field_str("id"),
            field_enum("severity", "Severity::"),
            field_enum("level", "WcagLevel::"),
            field_str("axe_id"),
        ) {
            entries.push(RuleMetaEntry {
                file: file.to_string(),
                const_name,
                id,
                severity,
                level,
                axe_id,
            });
        }
    }
    entries
}

/// SC groups where multiple rule files legitimately share a raw success
/// criterion with differing severity/level, already reviewed and accepted —
/// each reason states why the mix is fine, not just that it exists.
// 2026-09-06 (plan/1-root-cause-title-occurrence-mismatch.md, "Mechanism 2"):
// the "2.1.1", "1.4.4", "1.1.1", "4.1.2", and "1.3.1" groups formerly listed
// here were exactly QA-009's deferred content-authoring task — every axe_id
// in those groups that previously lacked its own `LEGACY_WCAG_MAP` entry
// (and therefore fell back to the shared raw-criterion group key, mixing
// unrelated checks' severities/occurrence counts into one finding) now has
// a dedicated taxonomy `Rule` + `LEGACY_WCAG_MAP` entry, so the "mixed
// severity in one group" premise these exceptions documented no longer
// applies — removed as directed by this test's own staleness check.
// "3.3.2" was the last one: instructions.rs/form_rules.rs shared the axe_id
// "label" with differing severities. Both left for the shared `forms/*`
// rules in #693, each with its own taxonomy group.
const ALLOWED_MIXED_SEVERITY_GROUPS: &[(&str, &str)] = &[];

#[test]
fn no_undocumented_severity_collisions_in_group_key_mechanism() {
    use auditmysite::taxonomy::RuleLookup;
    use std::collections::HashMap;

    let rules_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/wcag/rules");
    let mut all_entries: Vec<RuleMetaEntry> = Vec::new();

    for entry in std::fs::read_dir(&rules_dir).expect("src/wcag/rules must exist") {
        let entry = entry.expect("readable dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("readable rule source file");
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        all_entries.extend(extract_rule_metadata_entries(&file_name, &content));
    }

    // Sanity floor, not a pin: the count shrinks as rules move to the shared
    // `a11y-rules` bestand (#693 left 77, #694 67, #695 64, #696 58, #697 49, area/audio 47, stylesheets 46).
    assert!(
        all_entries.len() >= 40,
        "Expected ~46 RuleMetadata declarations across src/wcag/rules/, found {}. \
         The parser in extract_rule_metadata_entries may have broken (field \
         layout changed?) — verify before trusting this test's other assertions.",
        all_entries.len()
    );

    // Effective group key: the rule's own axe_id if the taxonomy has a
    // dedicated entry for it, otherwise the raw SC id — mirrors
    // audit::normalized::wcag_group_key exactly.
    let mut groups: HashMap<String, Vec<RuleMetaEntry>> = HashMap::new();
    for e in all_entries {
        let key = if RuleLookup::by_legacy_wcag_id(&e.axe_id).is_some() {
            e.axe_id.clone()
        } else {
            e.id.clone()
        };
        groups.entry(key).or_default().push(e);
    }

    let mut violations = Vec::new();
    let mut seen_exceptions: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for (key, members) in &groups {
        if members.len() < 2 {
            continue;
        }
        let severities: std::collections::HashSet<&str> =
            members.iter().map(|m| m.severity.as_str()).collect();
        let levels: std::collections::HashSet<&str> =
            members.iter().map(|m| m.level.as_str()).collect();
        if severities.len() <= 1 && levels.len() <= 1 {
            continue;
        }

        if let Some((name, _reason)) = ALLOWED_MIXED_SEVERITY_GROUPS
            .iter()
            .find(|(name, _)| name == key)
        {
            seen_exceptions.insert(name);
            continue;
        }

        let detail = members
            .iter()
            .map(|m| {
                format!(
                    "{} {}::{} (severity={}, level={})",
                    m.file, m.const_name, m.id, m.severity, m.level
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        violations.push(format!("group '{key}': {detail}"));
    }

    assert!(
        violations.is_empty(),
        "Found rule files sharing a normalized group key with inconsistent \
         severity/level (#QA-009 escalation risk) that isn't a documented \
         exception in ALLOWED_MIXED_SEVERITY_GROUPS. Either add a dedicated \
         taxonomy entry for the new/changed rule (preferred — see QA-009), \
         or add a reasoned exception if the mix is genuinely intentional:\n{}",
        violations.join("\n")
    );

    let stale: Vec<&str> = ALLOWED_MIXED_SEVERITY_GROUPS
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| !seen_exceptions.contains(name))
        .collect();
    assert!(
        stale.is_empty(),
        "ALLOWED_MIXED_SEVERITY_GROUPS entries no longer show a real \
         severity/level mismatch — remove them from the list: {stale:?}"
    );
}
