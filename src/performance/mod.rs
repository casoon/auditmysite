//! Performance analysis module
//!
//! Provides Core Web Vitals collection and performance scoring.

mod vitals;
mod scoring;
mod content_weight;

pub use vitals::{extract_web_vitals, WebVitals, VitalMetric};
pub use scoring::{calculate_performance_score, PerformanceGrade, PerformanceScore};
pub use content_weight::{analyze_content_weight, ContentWeight, ResourceBreakdown};
