//! Performance analysis module
//!
//! Provides Core Web Vitals collection and performance scoring.

mod animations;
mod content_weight;
mod coverage;
mod critical_chain;
mod minification;
mod render_blocking;
mod scoring;
mod third_party;
mod vitals;

pub use animations::{
    analyze_non_composited_animations, AnimationAnalysis, NonCompositedAnimation,
};
pub use content_weight::{analyze_content_weight, ContentWeight, ResourceBreakdown};
pub use coverage::{
    prepare_coverage_collection, take_coverage_results, CoverageAnalysis, ScriptCoverageEntry,
    UnusedCssAnalysis, UnusedJsAnalysis,
};
pub use critical_chain::{analyze_critical_chain, ChainNode, CriticalChain};
pub use minification::{analyze_minification, MinificationAnalysis, UnminifiedAsset};
pub use render_blocking::{analyze_render_blocking, BlockingResource, RenderBlockingAnalysis};
pub use scoring::{calculate_performance_score, PerformanceGrade, PerformanceScore};
pub use third_party::{analyze_third_party_attribution, ThirdPartyAttribution, ThirdPartyOrigin};
pub use vitals::{
    extract_web_vitals, finalize_lcp, prepare_vitals_collection, validate_metrics, ClsShift,
    ClsSource, MeasurementContext, ShiftRect, VitalMetric, WebVitals,
};
