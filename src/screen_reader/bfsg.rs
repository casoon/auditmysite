//! Thin delegating wrapper over the canonical `wcag::en301549` mapping.
//!
//! The WCAG↔EN 301 549 table used to live only here; it has been promoted to
//! `crate::wcag::en301549` (single canonical source, with clause titles and a
//! pure status roll-up over findings). This module keeps its existing public
//! API (`map_to_bfsg`, `wcag_21_aa_criteria`, `WcagCriterionMapping`,
//! `BfsgMapping`) so `BfsgViolation.bfsg_reference` and other consumers are
//! unaffected.
//!
//! `BFSG_PARAGRAPH_WEB` stays local — it is NOT propagated into the new
//! `wcag::en301549` module or the EN 301 549 annex; see that module's doc
//! comment.
//!
//! Citation corrected 2026-09-01 (from a report-quality review): the prior
//! value, `"§12 Abs. 1"`, cited the parent Act (BFSG) but the actual
//! website-accessibility requirement ("Webseiten ... wahrnehmbar, bedienbar,
//! verständlich und robust") lives in the implementing Ordinance, BFSGV §12
//! Nr. 3 -- confirmed against the primary source
//! (<https://www.gesetze-im-internet.de/bfsgv/__12.html>). Still flagged
//! unverified: confirmed via automated web-fetch summarization, not a legal
//! review: get a legal review before relying on this exact citation.
//!
//! BFSGV §19 ("Zusätzliche Anforderungen an Dienstleistungen im
//! elektronischen Geschäftsverkehr") -- confirmed verbatim against the
//! primary source (<https://www.gesetze-im-internet.de/bfsgv/__19.html>),
//! 2026-09-04 -- has three numbered requirements. Only Nr. 3 is incorporated,
//! and only as a presence-only, manual-review scope signal (see
//! `commerce::CommerceAnalysis::identification_function_detected` and
//! `BFSG_PARAGRAPH_ECOMMERCE`), never an automated pass/fail:
//! - **Nr. 1** ("Informationen zur Barrierefreiheit der ... Produkte ...,
//!   soweit diese ... vom verantwortlichen Wirtschaftsakteur zur Verfügung
//!   gestellt werden") is conditional on information the operator may or may
//!   not possess about a *specific product's* accessibility properties --
//!   there is no DOM signal that can tell "operator has such info but
//!   withheld it" apart from "product genuinely has nothing to disclose".
//!   Deliberately not incorporated: a presence-only check here would
//!   systematically false-flag every product without accessibility features
//!   to disclose, which is not a defect.
//! - **Nr. 2** ("Identifizierungs-, Authentifizierungs-, Sicherheits- und
//!   Zahlungsfunktionen, wenn diese ... im Rahmen einer Dienstleistung
//!   bereitgestellt werden") largely overlaps with Nr. 3 in DOM terms but
//!   adds the qualifier "as part of a service, not a product" and explicitly
//!   includes payment functions -- this tool's stateless single-page model
//!   essentially never reaches a live payment UI cold (same reasoning as the
//!   `CommercePageKind::Cart`/`::Checkout` removal), and distinguishing
//!   "service" from "product" delivery of an identification/security
//!   function from a single rendered page is not reliably possible.
//!   Deliberately not incorporated.
//! - **Nr. 3** ("Identifizierungsmethoden, Authentifizierungsmethoden,
//!   elektronische Signaturen und Zahlungsdienste ... wahrnehmbar, bedienbar,
//!   verständlich und robust") is the one sub-item with a concrete,
//!   deterministic DOM signal reachable from a single rendered page: a form
//!   control whose `autocomplete` token identifies it as
//!   `current-password`/`new-password`/`one-time-code`/`username`
//!   (`screen_reader::navigator::FormControlNavItem::is_identification_control`).
//!   Payment-method detection (`cc-number` etc.) was deliberately left out
//!   of scope for the same cold-reachability reason as Nr. 2. The signal is
//!   necessarily incomplete (a login field without a declared `autocomplete`
//!   purpose stays invisible), so it is only ever a positive-evidence
//!   presence flag, never a negative "no such field exists" conclusion --
//!   same honesty convention as `commerce::CommerceFinding`.

use serde::{Deserialize, Serialize};

use crate::wcag::en301549::EN301549_WEB_CLAUSES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BfsgMapping {
    pub en_301549_clause: &'static str,
    pub bfsg_paragraph: &'static str,
    pub fix_required: bool,
}

pub const BFSG_PARAGRAPH_WEB: &str = "§ 12 Nr. 3 BFSGV";

/// Citation for the BFSGV §19 Nr. 3 e-commerce identification/authentication
/// signal (`commerce::CommerceAnalysis::identification_function_detected`).
/// See this module's doc comment for the full sub-item scoping decision.
pub const BFSG_PARAGRAPH_ECOMMERCE: &str = "§ 19 Nr. 3 BFSGV";

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

/// Kept under its historical name for API stability: the list is now the 55
/// WCAG 2.2 A/AA criteria of EN 301 549 V4.1.1 (`EN301549_WEB_CLAUSES`).
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
        assert_eq!(mapping.bfsg_paragraph, "§ 12 Nr. 3 BFSGV");
        assert!(mapping.fix_required);

        assert_eq!(
            map_to_bfsg("4.1.2").expect("mapped").en_301549_clause,
            "9.4.1.2"
        );
    }

    #[test]
    fn covers_wcag_22_a_and_aa_criteria() {
        assert_eq!(wcag_21_aa_criteria().len(), 55);
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
