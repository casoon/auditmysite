//! Canonical metric registry (#506).
//!
//! Gives every specialized number surfaced in JSON, PDF, and docs a single
//! machine-readable definition instead of scattered renderer/doc logic.
//! Seeded first from `src/output/json.rs`'s `metric_context()` (Phase 1 step
//! 1 of the Report Quality Layer plan). The ~19 independent score→label/grade
//! definitions found elsewhere in the codebase (taxonomy, PDF renderers,
//! module-specific label functions) migrate to reference this registry in
//! later, separately reviewed steps — they are not touched yet.

mod bands;
mod metrics;
mod paths;

pub use bands::{
    BandSet, BAR_COLOR_BAND, BATCH_GRADE, CERTIFICATE, COVER_PHRASE, FIVE_BAND, FIVE_BAND_LETTERS,
    LETTER_GRADE, MEDAL, SCORE_RANGE, SECURITY_GRADE, SEO_BAND,
};
pub use metrics::REGISTRY;
pub use paths::json_path_candidates;

/// What kind of specialized number a `MetricSpec` describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    /// 0-100, higher is better.
    Score,
    /// 0-100, higher means MORE risk (inverted direction vs. `Score`).
    Risk,
    /// 0-1 share/coverage value.
    Ratio,
    /// An unbounded relative-ordering value (not a percentage or score).
    Ranking,
    /// A rule-level scoring-input constant (e.g. penalty points), not itself
    /// a report verdict.
    Impact,
    /// A distinct occurrence/finding count.
    Count,
    /// A label/grade/certificate derived from an underlying score.
    Classification,
}

/// Which direction of change is an improvement for this metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    HigherIsBetter,
    LowerIsBetter,
    /// Direction is not meaningful (e.g. a config constant or an unbounded
    /// ranking value where "higher" means "more urgent", not "better").
    Neutral,
}

/// Which report types carry this metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Single,
    Batch,
    Both,
}

/// How a metric is combined across pages in a batch report, if at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregation {
    None,
    Average,
    WeightedAverage,
    Sum,
    Max,
}

/// One canonical specialized number: id, JSON location, unit/scale, and the
/// documentation it must stay in sync with.
#[derive(Debug, Clone, Copy)]
pub struct MetricSpec {
    /// Stable identifier, snake_case, unique within `REGISTRY`.
    pub id: &'static str,
    /// JSON path (or, for entries kept compatible with the pre-registry
    /// `metric_context()` text, a short compound description) locating this
    /// value in the report envelope.
    pub json_path: &'static str,
    pub kind: MetricKind,
    pub unit: &'static str,
    /// `None` when no shared band/label ladder applies to this metric.
    pub band_set: Option<&'static BandSet>,
    pub direction: Direction,
    pub scope: Scope,
    pub aggregation: Aggregation,
    pub needs_explanation: bool,
    pub docs_url: &'static str,
    pub reference_urls: &'static [&'static str],
    /// ISO date (`YYYY-MM-DD`) of the last fachlicher review of this entry.
    pub reviewed_at: &'static str,
    /// Informal PDF component name, for traceability only — not validated by
    /// the registry contract test.
    pub pdf_component: Option<&'static str>,
    pub meaning: &'static str,
    /// Overrides `meaning` for batch reports when the wording differs.
    pub meaning_batch_override: Option<&'static str>,
}

impl MetricSpec {
    /// The `meaning` text for a given `report_type` (`"single"` or `"batch"`).
    pub fn meaning_for(&self, report_type: &str) -> &'static str {
        if report_type == "batch" {
            self.meaning_batch_override.unwrap_or(self.meaning)
        } else {
            self.meaning
        }
    }
}
