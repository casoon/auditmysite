//! Versioned contracts for heuristic signals (#784).
//!
//! UX, Journey, AI Visibility, Source Quality, Content Visibility and Design
//! Quality publish scores and signals that no external reference has
//! validated yet. Each module states here, once, what its score is meant to
//! say, how it is computed, where it applies, what it rests on, how
//! uncertain it is and how it may be read — and which claims it does **not**
//! establish. The JSON carries the contract next to the module's result
//! (`signal_contract`), so a consumer never has to guess.
//!
//! A contract changes its `version` whenever method, weights or thresholds
//! change in a way that moves the score; the calibration reference set
//! (`docs/calibration.md`) is bound to these versions. `validation` stays
//! `unvalidated` until a pre-registered calibration run against the blind,
//! human-labelled reference set meets its thresholds. Internal consistency
//! and passing local score bands are not such a validation.

use serde::Serialize;

/// Identifier of the contract format itself.
pub const SIGNAL_CONTRACT_FORMAT: &str = "auditmysite.signal-contract/1";

/// Whether a signal's score has been validated against an independent,
/// human-labelled reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum ValidationStatus {
    /// No external validation. The score orders pages by the contract's
    /// method only; the claims in `not_established` stay unproven.
    Unvalidated,
    /// Met the pre-registered thresholds of a calibration run.
    Calibrated {
        /// Reference-set version the run used.
        reference_set: &'static str,
        /// Published result file of the run.
        results: &'static str,
    },
}

/// The contract of one heuristic signal (canonical English, #406).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SignalContract {
    /// Module id (`registry` catalog), also the JSON key of its result.
    pub id: &'static str,
    /// Contract version of this signal; bumped when the score's method,
    /// weights or thresholds change.
    pub version: u32,
    /// What the score is meant to say.
    pub target_statement: &'static str,
    /// How it is computed.
    pub method: &'static str,
    /// Which pages it applies to, and when it does not.
    pub applicability: &'static str,
    /// What it rests on.
    pub evidence: &'static str,
    /// Known sources of error.
    pub uncertainty: &'static str,
    /// How a reader may interpret the score.
    pub permitted_interpretation: &'static str,
    /// Claims the score does not establish until validated.
    pub not_established: &'static [&'static str],
    pub validation: ValidationStatus,
}

/// One contract per heuristic module. A module whose JSON carries
/// `measurement_type: "heuristic"` must have an entry (test below and in
/// `output::json`).
pub const SIGNAL_CONTRACTS: &[SignalContract] = &[
    SignalContract {
        id: "ux",
        version: 1,
        target_statement: "Whether the page shows structural patterns that usability heuristics \
            associate with clear calls to action, visual hierarchy, clear content, visible trust \
            information and low cognitive load.",
        method: "Rule-based counts over the accessibility tree in five dimensions (CTA clarity, \
            visual hierarchy, content clarity, trust signals, cognitive load), each scored 0-100 \
            by fixed thresholds and combined as a weighted average.",
        applicability: "Any rendered page with an accessibility tree. Thresholds do not depend \
            on page type; on pages without a call to action or a trust context some dimensions \
            score what the page does not need.",
        evidence: "Roles, accessible names and counts from the accessibility tree; no visual \
            rendering, no user data.",
        uncertainty: "Thresholds and weights were set by the authors and are not calibrated. \
            The accessibility tree does not show layout, colour or emphasis, so visual hierarchy \
            is inferred from structure.",
        permitted_interpretation: "A list of structural patterns found or missing, to review by \
            hand. Comparable across pages only as a structural indicator.",
        not_established: &[
            "actual user experience or satisfaction",
            "task success or completion rate",
            "conversion or revenue effect",
        ],
        validation: ValidationStatus::Unvalidated,
    },
    SignalContract {
        id: "journey",
        version: 1,
        target_statement: "Whether a visitor can plausibly enter, orient, navigate, interact \
            and complete the page's detected purpose, judged from its structure.",
        method: "Page intent is detected from structure and URL; five dimensions (entry clarity, \
            orientation, navigation, interaction, conversion) are scored by fixed thresholds and \
            combined with intent-dependent weights. Friction points are recorded per dimension.",
        applicability: "Any rendered page with an accessibility tree. The weights depend on the \
            detected intent, which is itself heuristic; a wrong intent shifts the score.",
        evidence: "Accessibility tree, presence of a main landmark in the DOM and, when \
            available, the visible text length in the first viewport.",
        uncertainty: "No real visitor paths are observed. Thresholds, weights and intent \
            detection are not calibrated against task outcomes.",
        permitted_interpretation: "Structural friction candidates to review by hand; the score \
            orders pages by these candidates, not by how visitors fare.",
        not_established: &[
            "real visitor paths or task completion",
            "conversion or drop-off rates",
            "correctness of the detected page intent",
        ],
        validation: ValidationStatus::Unvalidated,
    },
    SignalContract {
        id: "ai_visibility",
        version: 1,
        target_statement: "How far the page carries structural signals that make machine \
            extraction easier, and whether AI crawlers may fetch it.",
        method: "Five dimensions (readability, citability, chunks, knowledge graph, crawler \
            policy) scored from present/absent signals with fixed weights and combined 25/25/20/\
            15/15. Lists, tables, headings and text volume count regardless of whether they suit \
            the content.",
        applicability: "Pages meant to be found and quoted. On pages that are not (log-in, \
            checkout, legal notices) a low score is no defect.",
        evidence: "Rendered DOM structure, structured data, meta data, robots.txt and meta \
            robots. The crawler-policy dimension reads robots rules; all other dimensions are \
            structural proxies.",
        uncertainty: "No AI system is queried. Weights and thresholds are not externally \
            validated; structure that suits one content type may hurt another.",
        permitted_interpretation: "A structural readiness indicator for machine extraction and \
            a statement of what robots rules allow.",
        not_established: &[
            "whether AI systems retrieve, rank or cite the page",
            "citation likelihood or frequency",
            "factual quality of the content",
        ],
        validation: ValidationStatus::Unvalidated,
    },
    SignalContract {
        id: "source_quality",
        version: 1,
        target_statement: "Whether the page is technically prepared as an information source: \
            consistent metadata and identifiable publisher and transport security.",
        method: "Consistency checks the page can show on its own (declared language vs. \
            og:locale, title vs. og:title, structured data vs. visible content) and publisher/\
            security signals (HTTPS, headers, publisher markup, canonical, Open Graph, contact/\
            imprint/privacy links) combined with fixed weights. Structural signals are reported \
            as indicators and do not score.",
        applicability: "Pages that present information under a publisher. Applies poorly to \
            application screens and pages without editorial content.",
        evidence: "DOM, structured data, meta data and response headers of the audited page.",
        uncertainty: "Markup can be present and wrong. Weights are not calibrated against any \
            assessment of source reliability.",
        permitted_interpretation: "Technical preparation for machine processing: what is \
            declared and whether declarations agree.",
        not_established: &[
            "factual correctness, completeness or currency of the content",
            "truthfulness or reliability of the source",
            "expertise or trustworthiness of the publisher",
        ],
        validation: ValidationStatus::Unvalidated,
    },
    SignalContract {
        id: "content_visibility",
        version: 1,
        target_statement: "Which search-facing signals in five areas (organic visibility, local \
            business, E-E-A-T markup, content depth, topical authority) the page shows or lacks.",
        method: "Aggregates signals from SEO, Source Quality, AI Visibility and pattern \
            detection; the score is the share of signals that are not problems.",
        applicability: "Pages meant to be found in search. Local-business signals apply only to \
            local businesses; the module is omitted when no SEO data was collected.",
        evidence: "Each signal carries its source (meta, structured data, DOM) in its evidence \
            list.",
        uncertainty: "Every signal counts equally; the areas are not weighted by any measured \
            effect on visibility.",
        permitted_interpretation: "A checklist of search-facing signals with evidence; the \
            score is the share fulfilled, not a visibility forecast.",
        not_established: &[
            "search ranking, impressions or traffic",
            "how search engines assess E-E-A-T",
            "topical authority as search engines measure it",
        ],
        validation: ValidationStatus::Unvalidated,
    },
    SignalContract {
        id: "design_quality",
        version: 1,
        target_statement: "Whether individual elements show measurable readability or layout \
            patterns that design guidelines warn against.",
        method: "Per-element measurements in the rendered page (computed styles and geometry) \
            compared with fixed thresholds; each hit is a review finding.",
        applicability: "Rendered pages; elements outside the measured viewport are not covered.",
        evidence: "Computed styles and bounding boxes of the reported elements, each with a \
            selector.",
        uncertainty: "Thresholds come from guidelines, not from tests with readers; a hit can be \
            an intended design decision.",
        permitted_interpretation: "Element-level review candidates, never confirmed defects.",
        not_established: &[
            "aesthetic quality or brand fit",
            "reader preference or comprehension",
        ],
        validation: ValidationStatus::Unvalidated,
    },
];

/// The contract of a heuristic module, if it has one.
pub fn signal_contract(module_id: &str) -> Option<&'static SignalContract> {
    SIGNAL_CONTRACTS.iter().find(|c| c.id == module_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_contract_names_a_catalog_module_once() {
        let catalog = crate::audit::catalog::AuditCatalog::standard();
        for (i, c) in SIGNAL_CONTRACTS.iter().enumerate() {
            assert!(
                catalog.iter().any(|m| m.id() == c.id),
                "contract for unknown module `{}`",
                c.id
            );
            assert!(
                SIGNAL_CONTRACTS[..i].iter().all(|o| o.id != c.id),
                "duplicate contract `{}`",
                c.id
            );
        }
    }

    /// An unvalidated signal must say what it does not establish; every
    /// field is filled and canonical English (#406).
    #[test]
    fn unvalidated_contracts_bound_their_claims() {
        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));
        for c in SIGNAL_CONTRACTS {
            assert!(c.version >= 1);
            if c.validation == ValidationStatus::Unvalidated {
                assert!(
                    !c.not_established.is_empty(),
                    "`{}` is unvalidated but bounds no claim",
                    c.id
                );
            }
            for text in [
                c.target_statement,
                c.method,
                c.applicability,
                c.evidence,
                c.uncertainty,
                c.permitted_interpretation,
            ]
            .iter()
            .chain(c.not_established)
            {
                assert!(!text.trim().is_empty(), "`{}` has an empty field", c.id);
                assert!(!has_umlaut(text), "`{}` is not English: {text}", c.id);
            }
        }
    }

    /// The claims #784 names stay bounded until a calibration run proves
    /// them: AI citation, visibility, user quality, source truth.
    #[test]
    fn issue_784_claims_are_marked_not_established() {
        let bounded = |id: &str, needle: &str| {
            let c = signal_contract(id).unwrap();
            assert_eq!(c.validation, ValidationStatus::Unvalidated);
            assert!(
                c.not_established.iter().any(|s| s.contains(needle)),
                "`{id}` does not bound `{needle}`"
            );
        };
        bounded("ai_visibility", "cite");
        bounded("content_visibility", "ranking");
        bounded("ux", "user experience");
        bounded("journey", "task completion");
        bounded("source_quality", "truthfulness");
    }
}
