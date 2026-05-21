//! Pattern Detection — structural recognition of UI patterns in the AXTree.
//!
//! Detects common patterns (MainNavigation, DisclosureMenu, ModalDialog,
//! TabList, SkipLink, Accordion) and produces:
//! - `recognized`: positive signals when the pattern is well-formed
//! - `violations`: WCAG findings when the pattern is broken
//!
//! Patterns are inferred from AXTree role/name/property structure, not from
//! runtime interaction. Anything requiring a behavioral test (e.g. actual
//! keyboard focus traversal) is out of scope and remains a manual-review item.

mod accordion;
mod disclosure_menu;
mod main_navigation;
mod modal_dialog;
mod skip_link;
mod tab_list;

use serde::{Deserialize, Serialize};

use crate::accessibility::AXTree;
use crate::wcag::types::Violation;

/// Result of running pattern detection against an AXTree.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternAnalysis {
    /// Patterns that were recognized in the page (positive signals).
    pub recognized: Vec<RecognizedPattern>,
    /// Violations emitted when a pattern was found but broken.
    pub violations: Vec<Violation>,
}

/// A pattern that was recognized in the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizedPattern {
    /// Pattern name, e.g. "MainNavigation".
    pub pattern: String,
    /// Human-readable summary of what was recognized.
    pub message: String,
    /// How confident the detection is.
    pub confidence: PatternConfidence,
}

/// Confidence level of a pattern detection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternConfidence {
    /// All structural criteria matched.
    Strong,
    /// Some criteria matched; others uncertain or missing.
    Partial,
}

impl PatternAnalysis {
    pub fn add_recognized(
        &mut self,
        pattern: &str,
        message: impl Into<String>,
        confidence: PatternConfidence,
    ) {
        self.recognized.push(RecognizedPattern {
            pattern: pattern.to_string(),
            message: message.into(),
            confidence,
        });
    }
}

/// Run all pattern detectors against an AXTree.
pub fn analyze(tree: &AXTree) -> PatternAnalysis {
    let mut result = PatternAnalysis::default();
    main_navigation::detect(tree, &mut result);
    disclosure_menu::detect(tree, &mut result);
    modal_dialog::detect(tree, &mut result);
    tab_list::detect(tree, &mut result);
    skip_link::detect(tree, &mut result);
    accordion::detect(tree, &mut result);
    result
}
