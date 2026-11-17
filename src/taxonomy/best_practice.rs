//! Per-rule decision for every best-practice rule (#919).
//!
//! A rule tagged `best-practice` is anchored to the nearest WCAG criterion
//! without proving a breach of it. What such a rule reports (a failure or a
//! review hint) and whether its finding may carry BFSG relevance or a legal
//! flag (#704) is decided here, one row per rule id, and nowhere else.
//! `is_convention_rule` reads this table.
//!
//! The table is the single source of `docs/reference/best-practice-rules.md`
//! (generated and held in step by `tests/best_practice_decisions.rs`). The
//! outcome column records what the rule does today; moving a rule from
//! `Fail` to `Review` is a separate decision per rule and happens in the
//! rule's owner (the shared library for [`Source::Barrierlab`]), not here.

/// What the engine reports for a rule today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeClass {
    /// A confirmed violation (`Outcome::Fail`), counted in the score.
    Fail,
    /// A review hint (`Outcome::Review`), no violation.
    Review,
    /// Both, depending on the case (`display/text-hidden`).
    FailAndReview,
}

impl OutcomeClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fail => "fail",
            Self::Review => "review",
            Self::FailAndReview => "fail + review",
        }
    }
}

/// Where the detection lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Shared library (`a11y-rules`, `SHARED_RULES`): changed in barrierlab,
    /// never patched here.
    Barrierlab,
    /// Rule of this repository (needs the running browser or the AX tree).
    Local,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Self::Barrierlab => "barrierlab",
            Self::Local => "local",
        }
    }
}

/// The decision for one best-practice rule id.
#[derive(Debug, Clone, Copy)]
pub struct BestPracticeDecision {
    pub id: &'static str,
    /// What the rule reports today.
    pub outcome: OutcomeClass,
    pub source: Source,
    /// The finding carries the `best-practice` keyword. `false` for a rule
    /// that is a best practice by decision (#711) without the library
    /// tagging it.
    pub keyword: bool,
    /// Convention rule (#704): never BFSG-relevant, raises no legal flag.
    pub convention: bool,
    /// Why the rule is not a confirmed WCAG breach. `None` for a rule of the
    /// library's own best-practice list (`a11y_rules::BEST_PRACTICE_RULES`),
    /// whose reason is that list's.
    pub reason: Option<&'static str>,
}

const CONVENTION_REASON: &str = "BarrierLab display-mode convention (#653): not a WCAG \
    requirement, anchored to the nearest criterion and reported only on pages that use the \
    convention.";

const fn row(
    id: &'static str,
    outcome: OutcomeClass,
    source: Source,
    keyword: bool,
    convention: bool,
    reason: Option<&'static str>,
) -> BestPracticeDecision {
    BestPracticeDecision {
        id,
        outcome,
        source,
        keyword,
        convention,
        reason,
    }
}

use OutcomeClass::{Fail, FailAndReview, Review};
use Source::{Barrierlab, Local};

/// One row per best-practice rule id, ordered by id.
///
/// `convention` is `true` for exactly the rules `is_convention_rule` exempted
/// before this table existed (`display/*`, `viz/*`, `names/title-only`,
/// `description-duplicates-name`, `zoom/viewport-scale-limited`); the other
/// rows keep their classification (#919 changes no outcome).
pub const BEST_PRACTICE_DECISIONS: &[BestPracticeDecision] = &[
    row("aria/role-redundant", Review, Barrierlab, true, false, None),
    row(
        "description-duplicates-name",
        Fail,
        Local,
        true,
        true,
        Some(
            "The element has a name; a description that repeats it is redundant, not a missing \
             name (#713). Anchored to 4.1.2 without breaching it.",
        ),
    ),
    row(
        "display/init-missing",
        Fail,
        Local,
        true,
        true,
        Some(CONVENTION_REASON),
    ),
    row(
        "display/text-hidden",
        FailAndReview,
        Local,
        true,
        true,
        Some(
            "BarrierLab display-mode convention (#653): not a WCAG requirement. A text layer \
             removed from assistive technology is a fail; one that a rendered element still \
             references by aria-describedby or aria-details is already a review note (#704).",
        ),
    ),
    row(
        "display/text-media-visible",
        Fail,
        Local,
        true,
        true,
        Some(CONVENTION_REASON),
    ),
    row(
        "display/text-not-visible",
        Fail,
        Local,
        true,
        true,
        Some(CONVENTION_REASON),
    ),
    row(
        "display/toggle-missing",
        Fail,
        Barrierlab,
        true,
        true,
        Some(CONVENTION_REASON),
    ),
    row("headings/h1-missing", Review, Barrierlab, true, false, None),
    row("headings/skip-level", Review, Barrierlab, true, false, None),
    row(
        "landmarks/banner-duplicate",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row(
        "landmarks/content-outside",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row(
        "landmarks/contentinfo-duplicate",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row(
        "landmarks/main-duplicate",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row(
        "landmarks/main-missing",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row(
        "landmarks/not-top-level",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row("lists/empty", Review, Barrierlab, true, false, None),
    row(
        "lists/role-redundant",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row(
        "names/title-only",
        Review,
        Barrierlab,
        false,
        true,
        Some(
            "`title` is a valid name source (H65), so 4.1.2 is met; the tooltip is only \
             reachable by mouse hover (#711). The library reports a review hint without the \
             best-practice keyword.",
        ),
    ),
    row(
        "patterns/tooltip-unreferenced",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row(
        "viz/caption-missing",
        Fail,
        Barrierlab,
        true,
        true,
        Some(CONVENTION_REASON),
    ),
    row(
        "viz/static-missing",
        Fail,
        Barrierlab,
        true,
        true,
        Some(CONVENTION_REASON),
    ),
    row(
        "viz/table-missing",
        Review,
        Barrierlab,
        true,
        true,
        Some(
            "BarrierLab display-mode convention (#653): not a WCAG requirement; a review hint \
             because a few values are also carried by a sentence.",
        ),
    ),
    row(
        "viz/text-missing",
        Fail,
        Barrierlab,
        true,
        true,
        Some(CONVENTION_REASON),
    ),
    row(
        "zoom/viewport-missing",
        Review,
        Barrierlab,
        true,
        false,
        None,
    ),
    row(
        "zoom/viewport-scale-limited",
        Review,
        Barrierlab,
        true,
        true,
        None,
    ),
];

/// The decision row of a rule id, if it is a best-practice rule.
pub fn best_practice_decision(rule_id: &str) -> Option<&'static BestPracticeDecision> {
    BEST_PRACTICE_DECISIONS.iter().find(|d| d.id == rule_id)
}
