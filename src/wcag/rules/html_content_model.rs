//! WCAG 1.3.1 - HTML Content Model
//!
//! Complements `list_structure.rs` (also SC 1.3.1, but AXTree-role-based).
//! Browsers "heal" invalid HTML when building the accessibility tree —
//! a content-model violation (e.g. an `<img>` sibling incorrectly placed
//! inside a `<dl>`'s inner `<div>`) can be silently absorbed into a
//! superficially valid role structure, hiding the real markup defect from
//! an AXTree-only check (#579).
//!
//! This rule instead re-scans the raw page HTML for `html-conform`'s
//! `schema.html5` (RelaxNG content-model) findings and determines whether
//! each one occurred inside a structurally significant element family
//! (`dl`, `table`, `select`, `fieldset`, `ul`/`ol`, …) whose content model
//! screen-reader semantics depend on. `html-conform`'s findings carry no
//! parent-element context, only a source location — the enclosing tag
//! stack is reconstructed here via a small, best-effort scanner (not a
//! full HTML5 parse; see `enclosing_tag_stack`).

use crate::cli::WcagLevel;
use crate::html_conform::HtmlConformFinding;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships - HTML Content Model",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Structural elements (lists, tables, forms, selects) must not contain content that violates their HTML5 content model, which can break screen-reader semantics even when the resulting accessibility tree looks superficially valid",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "html-content-model",
    tags: &["wcag2a", "wcag131", "cat.structure"],
};

/// Element families whose content model is directly relied on for
/// screen-reader semantics (issue #579's scope: dl/dt/dd, the table
/// family, ul/ol/li, select/option/optgroup, fieldset/legend). Only the
/// container elements are listed here since those are what appear on the
/// open-tag stack.
const STRUCTURAL_FAMILIES: &[&str] = &[
    "dl", "table", "thead", "tbody", "tfoot", "select", "optgroup", "fieldset", "ul", "ol",
];

/// Checks `html-conform`'s `schema.html5` findings for violations that
/// occurred inside a structurally significant element family.
pub fn check_html_content_model(html: &str, findings: &[HtmlConformFinding]) -> Vec<Violation> {
    let mut violations = Vec::new();

    for finding in findings {
        if finding.rule_id != "schema.html5" {
            continue;
        }
        let Some(byte_offset) = finding.byte_offset else {
            continue;
        };

        let stack = enclosing_tag_stack(html, byte_offset);
        let Some(family) = stack
            .iter()
            .rev()
            .find(|tag| STRUCTURAL_FAMILIES.contains(&tag.as_str()))
        else {
            continue;
        };

        let message = format!(
            "Invalid HTML content model inside a <{family}> element: {}",
            finding.message
        );

        violations.push(
            Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                RULE_META.severity,
                message,
                "document",
            )
            .with_selector(family)
            .with_fix(
                "Fix the markup so it matches the HTML5 content model for this element \
                 (e.g. a <dl> may only directly contain <dt>/<dd> groups, <div>, <script> or \
                 <template>). Invalid children can be silently 'healed' by the browser when \
                 building the accessibility tree, hiding the defect from role-based checks \
                 while still confusing assistive technology.",
            )
            .with_rule_id(RULE_META.axe_id)
            .with_help_url(RULE_META.help_url),
        );
    }

    violations
}

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_void_element(name: &str) -> bool {
    VOID_ELEMENTS.contains(&name)
}

/// Best-effort scan of `html` up to `byte_offset`, returning the stack of
/// open element tag names enclosing that position (lowercased).
///
/// Deliberately not a spec-compliant parser — this is a heuristic for an
/// advisory WCAG finding, not a rendering engine. It favors robustness
/// (never panics, never infinite-loops, degrades gracefully on malformed
/// input) over parsing fidelity: comments are skipped wholesale, raw-text
/// elements (`script`/`style`/`textarea`/`title`) have their content
/// skipped verbatim, void elements never push onto the stack, and
/// mismatched/unclosed tags are handled best-effort (a close tag pops up
/// to the matching open tag by name; an unmatched close tag is ignored).
fn enclosing_tag_stack(html: &str, byte_offset: usize) -> Vec<String> {
    // The offset comes from an external parser and can land inside a multi-byte
    // character; walk back to the character it belongs to rather than slicing
    // through it. One byte either way is irrelevant to the tag heuristic.
    let mut end = byte_offset.min(html.len());
    while end > 0 && !html.is_char_boundary(end) {
        end -= 1;
    }
    let mut stack: Vec<String> = Vec::new();
    let mut raw_text_tag: Option<String> = None;
    let mut i = 0usize;

    while i < end {
        let Some(rel_lt) = html[i..end].find('<') else {
            break;
        };
        i += rel_lt;

        if let Some(raw_tag) = &raw_text_tag {
            // Inside a raw-text element: only a matching closing tag ends it.
            let close = format!("</{raw_tag}");
            // Byte comparison: `close` is ASCII, but its length can run into
            // the middle of a multi-byte character, which `str` slicing would
            // panic on.
            let rest = html[i..].as_bytes();
            if rest.len() >= close.len()
                && rest[..close.len()].eq_ignore_ascii_case(close.as_bytes())
            {
                raw_text_tag = None;
                // Fall through to normal tag handling below for the close tag.
            } else {
                // Literal '<' inside raw-text content — not markup, skip past it.
                i += 1;
                continue;
            }
        }

        if html[i..].starts_with("<!--") {
            match html[i..].find("-->") {
                Some(off) => i += off + 3,
                None => break,
            }
            continue;
        }
        if html[i..].starts_with("<!") {
            match html[i..].find('>') {
                Some(off) => i += off + 1,
                None => break,
            }
            continue;
        }

        let Some(rel_gt) = html[i..].find('>') else {
            break;
        };
        let tag_inner = &html[i + 1..i + rel_gt];
        let next_i = i + rel_gt + 1;

        if let Some(name) = tag_inner.strip_prefix('/') {
            let name = name
                .trim()
                .split(|c: char| c.is_whitespace())
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if let Some(pos) = stack.iter().rposition(|t| *t == name) {
                stack.truncate(pos);
            }
        } else {
            let self_closing = tag_inner.trim_end().ends_with('/');
            let name = tag_inner
                .trim_start()
                .split(|c: char| c.is_whitespace() || c == '/')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if !name.is_empty() && !self_closing && !is_void_element(&name) {
                stack.push(name.clone());
                if matches!(name.as_str(), "script" | "style" | "textarea" | "title") {
                    raw_text_tag = Some(name);
                }
            }
        }

        i = next_i;
    }

    stack
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(rule_id: &str, byte_offset: Option<usize>) -> HtmlConformFinding {
        HtmlConformFinding {
            rule_id: rule_id.to_string(),
            severity: "error".to_string(),
            message: "unexpected element `img` at 12:5".to_string(),
            location: None,
            byte_offset,
        }
    }

    // ── enclosing_tag_stack ────────────────────────────────────────────

    #[test]
    fn nested_elements_produce_full_stack() {
        let html = "<html><body><div><dl><div>HERE";
        let offset = html.len();
        let stack = enclosing_tag_stack(html, offset);
        assert_eq!(stack, vec!["html", "body", "div", "dl", "div"]);
    }

    #[test]
    fn void_elements_do_not_push_onto_stack() {
        let html = "<div><br><img src=\"x\"><input type=\"text\">HERE";
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div"]);
    }

    #[test]
    fn self_closing_tags_do_not_push_onto_stack() {
        let html = "<div><custom-el/>HERE";
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div"]);
    }

    #[test]
    fn close_tag_pops_matching_open_tag() {
        let html = "<div><dl></dl>HERE";
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div"]);
    }

    #[test]
    fn comments_are_skipped_entirely() {
        let html = "<div><!-- <dl><table> --><span>HERE";
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div", "span"]);
    }

    #[test]
    fn offset_inside_a_multi_byte_character_does_not_panic() {
        // \u{a0} occupies two bytes; the offset points at its second one.
        let html = "<div><span>Preis:\u{a0}9 EUR";
        let offset = html.len() - "\u{a0}9 EUR".len() + 1;
        assert!(!html.is_char_boundary(offset));
        assert_eq!(enclosing_tag_stack(html, offset), vec!["div", "span"]);
    }

    #[test]
    fn multi_byte_character_after_raw_text_open_tag_does_not_panic() {
        let html = "<div><script>\u{a0}</script><span>HERE";
        assert_eq!(enclosing_tag_stack(html, html.len()), vec!["div", "span"]);
    }

    #[test]
    fn unterminated_comment_stops_gracefully_without_panicking() {
        let html = "<div><!-- unterminated";
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div"]);
    }

    #[test]
    fn script_content_with_literal_angle_brackets_is_skipped_verbatim() {
        let html = "<div><script>if (a < b && b > c) { x(); }</script><span>HERE";
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div", "span"]);
    }

    #[test]
    fn style_content_is_skipped_verbatim() {
        let html = "<div><style>.a > .b { color: red; }</style><span>HERE";
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div", "span"]);
    }

    #[test]
    fn mismatched_unclosed_tags_do_not_panic_or_loop() {
        let html = "<div><dl><span></div><p>HERE";
        // Best-effort: closing </div> pops past <span> and <dl> since
        // "div" is on the stack below them.
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["p"]);
    }

    #[test]
    fn unmatched_close_tag_is_ignored() {
        let html = "<div></span><p>HERE";
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div", "p"]);
    }

    #[test]
    fn byte_offset_beyond_html_length_is_clamped() {
        let html = "<div>content</div>";
        let stack = enclosing_tag_stack(html, html.len() + 1000);
        assert!(stack.is_empty());
    }

    #[test]
    fn byte_offset_landing_inside_a_tags_own_span() {
        let html = "<div><im"; // offset lands mid-tag, no '>' yet
        let stack = enclosing_tag_stack(html, html.len());
        assert_eq!(stack, vec!["div"]);
    }

    #[test]
    fn malformed_html_never_panics() {
        let inputs = [
            "<<<<<<<",
            "<div",
            "</",
            "<!--",
            "<script>",
            "",
            "<a href=\"<script>\">",
        ];
        for input in inputs {
            let _ = enclosing_tag_stack(input, input.len());
        }
    }

    // ── check_html_content_model ──────────────────────────────────────

    #[test]
    fn flags_schema_html5_finding_inside_dl() {
        let html = "<dl><div><img src=\"x\">HERE";
        let offset = html.len();
        let findings = vec![finding("schema.html5", Some(offset))];
        let violations = check_html_content_model(html, &findings);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("<dl>"));
        assert_eq!(violations[0].selector.as_deref(), Some("dl"));
    }

    #[test]
    fn ignores_non_schema_html5_rule_ids() {
        let html = "<table><div><img src=\"x\">HERE";
        let offset = html.len();
        let findings = vec![
            finding("parser.html5", Some(offset)),
            finding("assertion.aria.hidden-not-focusable", Some(offset)),
        ];
        let violations = check_html_content_model(html, &findings);
        assert!(violations.is_empty());
    }

    #[test]
    fn ignores_findings_without_a_byte_offset() {
        let html = "<dl><div><img src=\"x\">HERE";
        let findings = vec![finding("schema.html5", None)];
        let violations = check_html_content_model(html, &findings);
        assert!(violations.is_empty());
    }

    #[test]
    fn does_not_flag_when_outside_any_structural_family() {
        let html = "<body><div><img src=\"x\">HERE";
        let offset = html.len();
        let findings = vec![finding("schema.html5", Some(offset))];
        let violations = check_html_content_model(html, &findings);
        assert!(violations.is_empty());
    }

    #[test]
    fn innermost_structural_family_wins_when_nested() {
        let html = "<table><tbody><div>HERE";
        let offset = html.len();
        let findings = vec![finding("schema.html5", Some(offset))];
        let violations = check_html_content_model(html, &findings);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].selector.as_deref(), Some("tbody"));
    }
}
