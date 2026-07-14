//! Thin delegating wrapper over the canonical `wcag::en301549` mapping.
//!
//! The WCAG↔EN 301 549 table used to live only here; it has been promoted to
//! `crate::wcag::en301549` (single canonical source, with clause titles and a
//! pure status roll-up over findings). This module keeps its existing public
//! API (`map_to_bfsg`, `wcag_21_aa_criteria`, `WcagCriterionMapping`,
//! `BfsgMapping`) so `BfsgViolation.bfsg_reference` and other consumers are
//! unaffected.
//!
//! `BFSG_PARAGRAPH_WEB` stays local and unverified — it is NOT propagated
//! into the new `wcag::en301549` module or the EN 301 549 annex; see that
//! module's doc comment.

use serde::{Deserialize, Serialize};

use crate::wcag::en301549::EN301549_WEB_CLAUSES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BfsgMapping {
    pub en_301549_clause: &'static str,
    pub bfsg_paragraph: &'static str,
    pub fix_required: bool,
}

pub const BFSG_PARAGRAPH_WEB: &str = "§12 Abs. 1";

pub fn map_to_bfsg(wcag: &str) -> Option<BfsgMapping> {
    wcag_21_aa_criteria()
        .iter()
        .find(|criterion| criterion.wcag == wcag)
        .map(|criterion| BfsgMapping {
            en_301549_clause: criterion.en_301549_clause,
            bfsg_paragraph: BFSG_PARAGRAPH_WEB,
            fix_required: true,
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WcagCriterionMapping {
    pub wcag: &'static str,
    pub en_301549_clause: &'static str,
}

pub fn wcag_21_aa_criteria() -> &'static [WcagCriterionMapping] {
    static CACHE: std::sync::OnceLock<Vec<WcagCriterionMapping>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        EN301549_WEB_CLAUSES
            .iter()
            .map(|c| WcagCriterionMapping {
                wcag: c.wcag,
                en_301549_clause: c.en_clause,
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::{map_to_bfsg, wcag_21_aa_criteria};

    #[test]
    fn maps_required_wcag_examples() {
        let mapping = map_to_bfsg("1.1.1").expect("mapped");
        assert_eq!(mapping.en_301549_clause, "9.1.1.1");
        assert_eq!(mapping.bfsg_paragraph, "§12 Abs. 1");
        assert!(mapping.fix_required);

        assert_eq!(
            map_to_bfsg("4.1.2").expect("mapped").en_301549_clause,
            "9.4.1.2"
        );
    }

    #[test]
    fn covers_wcag_21_a_and_aa_criteria() {
        assert_eq!(wcag_21_aa_criteria().len(), 50);
        assert!(wcag_21_aa_criteria()
            .iter()
            .all(|criterion| map_to_bfsg(criterion.wcag).is_some_and(|m| m.fix_required)));
    }

    #[test]
    fn aaa_and_best_practice_criteria_are_not_required() {
        assert!(map_to_bfsg("2.4.9").is_none());
        assert!(map_to_bfsg("best-practice").is_none());
    }
}
