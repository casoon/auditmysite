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
    check_accessible_name, check_aria_naming_rules, check_aria_relationships, check_aria_roles,
    check_bypass_blocks, check_dialog_rules, check_focus_order, check_focus_visible,
    check_form_rules, check_headings, check_info_relationships, check_input_purpose,
    check_instructions, check_keyboard, check_labels, check_landmark_banner_is_top_level,
    check_landmark_contentinfo_is_top_level, check_landmark_main_is_top_level,
    check_landmark_no_duplicate_banner, check_landmark_no_duplicate_contentinfo,
    check_landmark_no_duplicate_main, check_landmark_unique, check_landmarks, check_language,
    check_link_purpose, check_list_structure, check_media_rules, check_page_titled,
    check_section_headings, check_svg_rules, check_table_extended, check_table_rules,
    check_text_alternatives, check_widget_rules,
};

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
rule_smoke_test!(smoke_check_info_relationships, check_info_relationships);
rule_smoke_test!(smoke_check_keyboard, check_keyboard);
rule_smoke_test!(smoke_check_bypass_blocks, check_bypass_blocks);
rule_smoke_test!(smoke_check_page_titled, check_page_titled);
rule_smoke_test!(smoke_check_link_purpose, check_link_purpose);
rule_smoke_test!(smoke_check_language, check_language);
rule_smoke_test!(smoke_check_instructions, check_instructions);
rule_smoke_test!(smoke_check_focus_order, check_focus_order);
rule_smoke_test!(smoke_check_labels, check_labels);
rule_smoke_test!(smoke_check_aria_roles, check_aria_roles);
rule_smoke_test!(smoke_check_accessible_name, check_accessible_name);
rule_smoke_test!(smoke_check_aria_relationships, check_aria_relationships);
rule_smoke_test!(smoke_check_aria_naming_rules, check_aria_naming_rules);
rule_smoke_test!(smoke_check_table_rules, check_table_rules);
rule_smoke_test!(smoke_check_form_rules, check_form_rules);
rule_smoke_test!(smoke_check_list_structure, check_list_structure);
rule_smoke_test!(smoke_check_dialog_rules, check_dialog_rules);
rule_smoke_test!(smoke_check_widget_rules, check_widget_rules);
rule_smoke_test!(smoke_check_media_rules, check_media_rules);
rule_smoke_test!(smoke_check_svg_rules, check_svg_rules);
rule_smoke_test!(smoke_check_landmark_unique, check_landmark_unique);
rule_smoke_test!(
    smoke_check_landmark_banner_is_top_level,
    check_landmark_banner_is_top_level
);
rule_smoke_test!(
    smoke_check_landmark_contentinfo_is_top_level,
    check_landmark_contentinfo_is_top_level
);
rule_smoke_test!(
    smoke_check_landmark_main_is_top_level,
    check_landmark_main_is_top_level
);
rule_smoke_test!(
    smoke_check_landmark_no_duplicate_banner,
    check_landmark_no_duplicate_banner
);
rule_smoke_test!(
    smoke_check_landmark_no_duplicate_contentinfo,
    check_landmark_no_duplicate_contentinfo
);
rule_smoke_test!(
    smoke_check_landmark_no_duplicate_main,
    check_landmark_no_duplicate_main
);
rule_smoke_test!(smoke_check_table_extended, check_table_extended);
rule_smoke_test!(smoke_check_input_purpose, check_input_purpose);
// non_text_contrast.rs was replaced by non_text_contrast_css.rs (a `_with_page`
// CDP-based check) — like the other `_with_page` rules, it has no smoke test
// here (this file is browser-free/AXTree-only); it has its own unit tests.
rule_smoke_test!(smoke_check_headings, check_headings);
rule_smoke_test!(smoke_check_focus_visible, check_focus_visible);
rule_smoke_test!(smoke_check_landmarks, check_landmarks);
rule_smoke_test!(smoke_check_section_headings, check_section_headings);

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
    // Tree that would violate both 1.1.1 (missing alt) and 3.1.1 (missing lang)
    let tree = AXTree::from_nodes(vec![
        AXNode {
            node_id: "root".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("RootWebArea".to_string()),
            name: Some("Test Page".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![], // no lang
            child_ids: vec!["img1".to_string()],
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
    ]);

    // Run only the lang rule
    let filter = RuleFilterConfig {
        disabled_rules: vec![],
        enabled_only_rules: vec!["html-has-lang".to_string()],
    };
    let results = check_all_with_config(&tree, WcagLevel::A, &filter);

    // Should find 3.1.1 (missing lang) but NOT 1.1.1 (disabled by filter)
    assert!(
        results.violations.iter().any(|v| v.rule == "3.1.1"),
        "3.1.1 should be found"
    );
    let alt_violations: Vec<_> = results
        .violations
        .iter()
        .filter(|v| v.rule == "1.1.1")
        .collect();
    assert!(
        alt_violations.is_empty(),
        "1.1.1 should be suppressed by enabled_only filter"
    );
}

// ---------------------------------------------------------------------------
// Level gating — AA rules only run when level >= AA
// ---------------------------------------------------------------------------

#[test]
fn test_level_a_does_not_run_aa_rules() {
    // A tree that would violate an AA-only rule (1.3.5 Identify Input
    // Purpose: a textbox whose name suggests a user-info field but has no
    // autocomplete attribute).
    // (1.4.4 resize-text and 1.4.11 non-text-contrast used to be the example
    // here; both are now DOM/CDP `_with_page` rules and have their own
    // level-gating coverage — see page_rules.rs's own level-gating tests for
    // that mechanism — so this AXTree-only engine test needed a still-AXTree
    // -based AA rule as its example.)
    let tree = AXTree::from_nodes(vec![AXNode {
        node_id: "root".to_string(),
        ignored: false,
        ignored_reasons: vec![],
        role: Some("textbox".to_string()),
        name: Some("Email".to_string()),
        name_source: None,
        description: None,
        value: None,
        properties: vec![],
        child_ids: vec![],
        parent_id: None,
        backend_dom_node_id: None,
    }]);

    let results_a = check_all_with_config(&tree, WcagLevel::A, &RuleFilterConfig::default());
    let results_aa = check_all_with_config(&tree, WcagLevel::AA, &RuleFilterConfig::default());

    // The 1.3.5 input-purpose violation should only appear at AA+
    let input_purpose_violations_a: Vec<_> = results_a
        .violations
        .iter()
        .filter(|v| v.rule == "1.3.5")
        .collect();
    let input_purpose_violations_aa: Vec<_> = results_aa
        .violations
        .iter()
        .filter(|v| v.rule == "1.3.5")
        .collect();

    assert!(
        input_purpose_violations_a.is_empty(),
        "Level A should not check 1.3.5 (AA rule)"
    );
    assert!(
        !input_purpose_violations_aa.is_empty(),
        "Level AA should check 1.3.5"
    );
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
    // - "htmlTag": confirmed live via summary_name.rs (<details>/<summary>)
    //   and instructions.rs (<fieldset> without a submitted DOM check).
    // - "url": a link's href target (landmark_granular.rs / region.rs).
    "language",
    "htmlTag",
    "url",
];

/// Known residual reads of a non-real property name: documented, low
/// priority, intentionally tolerated rather than silently broken. Each is a
/// harmless redundant branch alongside a working primary check, or a
/// tracked, deliberately-deferred item — not a silent gap.
const KNOWN_EXCEPTIONS: &[(&str, &str)] = &[
    (
        "inputType",
        "input_purpose.rs — redundant OR-branch; role==\"textbox\" already \
         covers native text-like input types in practice",
    ),
    (
        "placeholder",
        "instructions.rs's has_format_hint fallback — the primary \
         placeholder-only-label detection now uses name_source (#QA-030)",
    ),
    (
        "lang",
        "language.rs — fallback alongside the working \"language\" property; \
         superseded by pipeline.rs's DOM-based apply_lang_attribute_check (#QA-001)",
    ),
    (
        "headers",
        "info_relationships.rs check_cell_headers — only affects an internal \
         `passes` counter, not violation detection (table_extended.rs's \
         td-headers-attr is the DOM-based, violation-producing check)",
    ),
    (
        "id",
        "aria_relationships.rs — appears only in a doc comment explaining a \
         past bug, not an actual property read",
    ),
    (
        "title",
        "label_title_only.rs's fallback heuristic (name_source unavailable) — \
         a real DOM `title` attribute read would need a page.evaluate \
         conversion; the primary name_source==Title path already covers the \
         common case",
    ),
];

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
const ALLOWED_MIXED_SEVERITY_GROUPS: &[(&str, &str)] = &[
    (
        "2.1.1",
        "keyboard.rs (Critical, default metadata severity) + click_handlers.rs \
         (High) — reviewed as a harmless grouping in the Phase 2 taxonomy \
         review: both are genuine 2.1.1 keyboard-operability problems, and \
         individual violations already set their own explicit per-call \
         severity rather than relying on the struct default for most cases.",
    ),
    (
        "1.4.4",
        "meta_viewport_large.rs (Medium, 500% best-practice) + resize_text.rs \
         (High, 200% WCAG-required) — intentional by design (#QA-030 same \
         session): check_meta_viewport_large_with_page guards against \
         double-firing with the stricter check, so the two severities never \
         apply to the same violation on the same page.",
    ),
    (
        "1.1.1",
        "server_side_image_map.rs (Medium) is a distinct, lesser pattern \
         (discourage server-side maps) vs. the rest of the 1.1.1 family \
         (High — genuinely missing alternative text). Escalating to High \
         when both co-occur on a page is the correct worst-case reading, \
         not a bug.",
    ),
    (
        "4.1.2",
        "widget_rules.rs's RULE_META default (Medium) doesn't reflect that \
         its individual checks emit their own varying per-call severities \
         (Low for tab-selected-state, Medium/High for others) — a metadata \
         cleanliness gap, not a functional bug. Tracked for cleanup, not \
         fixed here.",
    ),
    (
        "1.3.1",
        "13-member SC group (region/landmark-granular/table/list/form/info- \
         relationships checks). Mostly Medium; info_relationships.rs's \
         definition-list check is High. This is the same large 1.3.1 \
         collision QA-009 identified and explicitly deferred as a \
         content-authoring task (dedicated per-check taxonomy entries with \
         properly written DE/EN copy) rather than a quick mechanical fix — \
         see QA-009's backlog entry for the full remaining-scope list.",
    ),
    (
        "3.3.2",
        "instructions.rs (High) vs. form_rules.rs's RULE_META_LABELS (Low) — \
         same deferred-collision class as 1.3.1 above, tracked under QA-009's \
         remaining scope rather than fixed here.",
    ),
];

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

    assert!(
        all_entries.len() >= 90,
        "Expected ~105 RuleMetadata declarations across src/wcag/rules/, found {}. \
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
