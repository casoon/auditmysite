//! Written recommendations for `html-conform` findings (#899).
//!
//! The checker's rule set is open-ended (about 280 Schematron ids plus a few
//! fixed layers), and its messages are third-party English prose. A finding
//! group without a written recommendation is left out of the action plan and
//! the key-measures list, so every rule id resolves here to a *family* with a
//! localized title and a concrete fix hint (`html-conform-<family>-title` /
//! `-fix` in `locales/`). Ids a newer checker version adds fall back to a
//! generic hint; `every_rule_id_of_the_checker_has_a_family` fails for those
//! at the next version bump so they get a family of their own.

use crate::i18n::I18n;

/// Rule families, keyed like the locale entries.
const FAMILIES: &[&str] = &[
    "parser",
    "schema",
    "ids",
    "headings",
    "language",
    "image-alt",
    "responsive-images",
    "image-maps",
    "nested-interactive",
    "forms",
    "document-meta",
    "link-rel",
    "scripts",
    "csp",
    "tables",
    "aria-redundant-role",
    "aria-naming",
    "aria-state",
    "aria-structure",
    "obsolete",
    "microdata",
    "meter-progress",
    "sectioning",
    "media-text",
    "text-normalization",
];

/// The checker's rule id as the report carries it (`html_conform.` prefix of
/// the finding contract and `assertion.` prefix of the checker stripped).
pub(crate) fn bare_id(rule_id: &str) -> &str {
    let id = rule_id.strip_prefix("html_conform.").unwrap_or(rule_id);
    id.strip_prefix("assertion.").unwrap_or(id)
}

/// The family of a checker rule id, `None` for an id no family covers.
pub fn family(rule_id: &str) -> Option<&'static str> {
    let id = bare_id(rule_id);
    let name = id.strip_prefix("elements.");
    let starts = |prefix: &str| id.starts_with(prefix);
    let el = |prefix: &str| name.is_some_and(|n| n.starts_with(prefix));

    let family = if id == "parser.html5" {
        "parser"
    } else if id == "schema.html5" {
        "schema"
    } else if starts("ids.") {
        "ids"
    } else if starts("headings.") || starts("headingoffset-range.") {
        "headings"
    } else if id == "elements.html-missing-lang" || id == "attributes.lang-xml-lang-sameness" {
        "language"
    } else if matches!(
        id,
        "elements.img-missing-alt"
            | "elements.img-missing-alt-in-figure"
            | "elements.img-controls-needs-alt"
            | "aria.img-alt-role-none-presentation"
            | "presentation-role.img-empty-alt-with-aria"
            | "presentational.no-button-in-img"
    ) || starts("img-aria-needs-accessible-name.")
        || starts("img-role-needs-accessible-name.")
    {
        "image-alt"
    } else if el("img-ismap")
        || el("img-usemap")
        || el("map-")
        || el("area-")
        || el("object-usemap")
    {
        "image-maps"
    } else if el("img-picture-srcset")
        || el("img-sizes")
        || el("sizes-")
        || el("source-")
        || el("srcset-")
    {
        "responsive-images"
    } else if name.is_some_and(|n| n.ends_with("-in-a") || n.ends_with("-in-button"))
        || id == "elements.svg-a-in-svg-a"
        || id == "elements.interactive-role-in-a"
        || id == "elements.tabindex-in-a"
        || starts("a-href-forbidden-inside-widget-role.")
        || starts("tabindex-forbidden-inside-role.")
    {
        "nested-interactive"
    } else if el("input-")
        || el("label-")
        || el("select-")
        || el("selectedcontent-")
        || el("option-")
        || el("optgroup-")
        || el("form-")
        || el("autofocus-")
        || starts("attributes.form-ref")
    {
        "forms"
    } else if el("meta-") || el("title-") || el("base-") {
        "document-meta"
    } else if el("link-") {
        "link-rel"
    } else if el("script-") || starts("scripts.") {
        "scripts"
    } else if starts("csp.") {
        "csp"
    } else if starts("tables.") {
        "tables"
    } else if starts("roles.unnecessary-")
        || starts("unnecessary-role-extra.")
        || starts("aria-disabled-redundant.")
        || id == "select-listbox-role.redundant"
    {
        "aria-redundant-role"
    } else if starts("naming-prohibited.") || starts("label-role-restrictions.") {
        "aria-naming"
    } else if starts("aria-")
        || starts("aria.")
        || starts("a-href-aria-disabled.")
        || starts("checkbox.")
        || starts("roles-properties-global.")
        || starts("commandfor-target-exists.")
    {
        "aria-state"
    } else if starts("author-requirements.")
        || starts("li-role-by-ancestor.")
        || starts("role-")
        || starts("presentation-role.")
        || starts("presentational")
        || starts("dl-div-child-role.")
        || starts("multiple-main-with-role.")
        || starts("summary-role.")
        || starts("select-listbox-role.")
    {
        "aria-structure"
    } else if starts("obsolete-elements.") {
        "obsolete"
    } else if starts("microdata.") || el("itemprop-") || el("itemref-") {
        "microdata"
    } else if el("meter-") || el("progress-") {
        "meter-progress"
    } else if el("address-")
        || el("dl-")
        || el("dt-")
        || el("figure-")
        || el("main-")
        || el("multiple-main")
        || el("no-footer")
        || el("no-header")
    {
        "sectioning"
    } else if el("track-") || el("rtc-") || el("ruby-") || el("bdo-") || el("custom-element-") {
        "media-text"
    } else if id == "text.normalization" {
        "text-normalization"
    } else {
        return None;
    };
    debug_assert!(FAMILIES.contains(&family));
    Some(family)
}

/// Localized `(title, fix hint)` for a checker rule id. An id without a family
/// gets the generic pair, so a finding never lacks a recommendation.
pub fn recommendation(rule_id: &str, i18n: &I18n) -> (String, String) {
    let family = family(rule_id).unwrap_or("generic");
    (
        i18n.t(&format!("html-conform-{family}-title")),
        i18n.t(&format!("html-conform-{family}-fix")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Rule ids the checker can emit: the fixed layers plus every `id` of an
    /// `assert`/`report` in the Schematron files of the exact crate version
    /// this build resolved to.
    fn checker_rule_ids() -> Vec<String> {
        let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
            .expect("Cargo.lock");
        let version = lock
            .split("[[package]]")
            .find(|p| p.contains("name = \"html-conform\""))
            .and_then(|p| p.lines().find_map(|l| l.strip_prefix("version = \"")))
            .and_then(|v| v.strip_suffix('"'))
            .expect("html-conform in Cargo.lock");
        let home = std::env::var("CARGO_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap()).join(".cargo"));
        let src = home.join("registry/src");
        let crate_dir = std::fs::read_dir(&src)
            .expect("cargo registry")
            .filter_map(Result::ok)
            .map(|index| index.path().join(format!("html-conform-{version}")))
            .find(|dir| dir.join("rules").is_dir())
            .unwrap_or_else(|| panic!("html-conform {version} sources not in {src:?}"));

        let mut ids: Vec<String> = [
            "parser.html5",
            "schema.html5",
            "tables.integrity",
            "scripts.import-map",
            "scripts.speculation-rules",
            "csp.meta-enforcement",
            "text.normalization",
        ]
        .iter()
        .map(|id| id.to_string())
        .collect();
        for entry in std::fs::read_dir(crate_dir.join("rules")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "sch") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for tag in text.split('<').skip(1) {
                if !(tag.starts_with("assert ") || tag.starts_with("report ")) {
                    continue;
                }
                let head = tag.split('>').next().unwrap();
                let id = head
                    .split(" id=\"")
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .unwrap_or_else(|| panic!("check without id in {path:?}: {head}"));
                ids.push(format!("assertion.{id}"));
            }
        }
        assert!(ids.len() > 250, "only {} rule ids found", ids.len());
        ids
    }

    #[test]
    fn every_rule_id_of_the_checker_has_a_family() {
        let missing: Vec<String> = checker_rule_ids()
            .into_iter()
            .filter(|id| family(&format!("html_conform.{id}")).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "rule ids without a recommendation family: {missing:?}"
        );
    }

    #[test]
    fn every_family_has_a_complete_text_in_both_locales() {
        for locale in ["de", "en"] {
            let i18n = I18n::new(locale).unwrap();
            for family in FAMILIES.iter().copied().chain(["generic"]) {
                for part in ["title", "fix"] {
                    let key = format!("html-conform-{family}-{part}");
                    let text = i18n.t(&key);
                    assert_ne!(text, key, "{locale}: {key} missing");
                    assert!(text.trim().len() > 10, "{locale}: {key} empty");
                    if locale == "en" {
                        assert!(!text.contains(['ä', 'ö', 'ü', 'ß']), "{key}: {text}");
                    }
                }
            }
        }
    }

    #[test]
    fn every_family_is_used() {
        let ids = checker_rule_ids();
        for family_name in FAMILIES {
            assert!(
                ids.iter().any(|id| family(id) == Some(*family_name)),
                "family {family_name} matches no rule id"
            );
        }
    }

    #[test]
    fn unknown_id_gets_the_generic_hint_not_an_empty_one() {
        let i18n = I18n::new("en").unwrap();
        let (title, fix) = recommendation("html_conform.assertion.future.rule", &i18n);
        assert!(!title.is_empty() && !fix.is_empty());
        assert!(!fix.starts_with("html-conform-"));
    }
}
