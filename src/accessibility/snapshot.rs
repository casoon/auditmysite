//! AXTree snapshot with focus and context for the Accessibility-Journey-Layer.
//!
//! A single static AXTree is what the WCAG rule engine checks against. The
//! Journey-Layer instead works with *sequences* of snapshots taken before
//! and after interactions, so state transitions become visible.
//!
//! Phase 1 ships the data types; capture and diffing are wired up by callers
//! starting with Phase 2.

use serde::{Deserialize, Serialize};

use super::tree::AXTree;

/// One captured snapshot of the page at a defined point in a journey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AXSnapshot {
    /// Human-readable label such as `"initial"` or `"after_modal_open"`.
    pub label: String,
    /// Wall-clock ms since page navigation started.
    pub timestamp_ms: u64,
    /// URL at time of capture. Distinguishes SPA navigations from full reloads.
    pub url: String,
    /// `document.title` at time of capture.
    pub document_title: String,
    /// AXTree as returned by CDP at capture time.
    pub tree: AXTree,
    /// Focus state at capture time.
    pub focus: FocusSnapshot,
}

/// Focus state captured alongside an AXTree.
///
/// AXTree deltas alone miss many real issues: focus may technically rest on
/// an element that is visually invisible, scrolled off-screen, or occluded
/// by an overlay. This struct combines focus identity, computed visibility,
/// and focus-indicator detection so the journey layer can reason about
/// "is the keyboard user actually seeing where they are?".
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FocusSnapshot {
    /// CDP backend node id of `document.activeElement`, if any.
    pub active_backend_node_id: Option<i64>,
    /// Matching AXTree node id, if mappable.
    pub ax_node_id: Option<String>,
    /// CSS selector of the focused element (best effort).
    pub selector: Option<String>,
    /// Computed-style visibility: display/visibility/opacity all permit visibility.
    pub visible: bool,
    /// Bounding box intersects the current viewport.
    pub in_viewport: bool,
    /// Tri-state focus-indicator detection. `None` = not evaluated.
    pub focus_indicator: Option<FocusIndicatorStatus>,
    /// Bounding box of the focused element, if available.
    pub bounding_box: Option<Rect>,
    /// Selector of an overlay occluding the focused element, if any.
    pub obscured_by: Option<String>,
}

/// Detection of a visible focus indicator.
///
/// Auto-detection is fundamentally imprecise — focus can be conveyed via
/// background color, pseudo-elements, or complex component states. We
/// therefore model the result as tri-state, not boolean:
/// - `Detected` — clear style delta between unfocused and focused.
/// - `NotDetected` — no detectable delta at all; eligible for "violation".
/// - `Ambiguous` — some delta but not unambiguously attributable to focus;
///   reviewer should verify visually ("warning" in reports).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusIndicatorStatus {
    Detected,
    NotDetected,
    Ambiguous,
}

/// Pixel bounding rectangle in CSS pixels, viewport coordinates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Whether this rectangle overlaps the given viewport (origin 0,0).
    pub fn intersects_viewport(&self, viewport_width: f32, viewport_height: f32) -> bool {
        self.x < viewport_width
            && self.y < viewport_height
            && self.x + self.width > 0.0
            && self.y + self.height > 0.0
    }
}

impl AXSnapshot {
    /// Construct a snapshot from the given label and components.
    pub fn new(
        label: impl Into<String>,
        url: impl Into<String>,
        document_title: impl Into<String>,
        timestamp_ms: u64,
        tree: AXTree,
        focus: FocusSnapshot,
    ) -> Self {
        Self {
            label: label.into(),
            timestamp_ms,
            url: url.into(),
            document_title: document_title.into(),
            tree,
            focus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_intersects_viewport() {
        let r = Rect {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 50.0,
        };
        assert!(r.intersects_viewport(800.0, 600.0));
        let off = Rect {
            x: 900.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(!off.intersects_viewport(800.0, 600.0));
    }

    #[test]
    fn snapshot_constructs_with_defaults() {
        let snap = AXSnapshot::new(
            "initial",
            "https://example.com",
            "Example",
            0,
            AXTree::new(),
            FocusSnapshot::default(),
        );
        assert_eq!(snap.label, "initial");
        assert!(snap.focus.focus_indicator.is_none());
    }
}
