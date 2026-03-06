//! Performance analysis module
//!
//! Provides Core Web Vitals collection and performance scoring.

mod content_weight;
mod scoring;
mod vitals;

pub use content_weight::{analyze_content_weight, ContentWeight, ResourceBreakdown};
pub use scoring::{calculate_performance_score, PerformanceGrade, PerformanceScore};
pub use vitals::{extract_web_vitals, VitalMetric, WebVitals};
