//! Browser interaction primitives for the Accessibility-Journey-Layer.
//!
//! Real CDP events (keyboard + mouse) over synthetic JS clicks — see
//! `pointer::synthetic_click_backend` for the documented fallback.
//!
//! This module is *actions only*: no evaluation, no journey logic. Journey
//! orchestration lives in `crate::a11y_journey`.
//!
//! # Boundary: detect, exercise, primitives (#938)
//!
//! The lowest of three phases: [`crate::patterns`] detects, [`crate::a11y_journey`]
//! exercises, this module provides the primitives. It imports neither of the
//! others (`tests/architecture_test.rs`); `a11y_journey` and the audit
//! pipeline import it.

pub mod focus;
pub mod keyboard;
pub mod live_regions;
pub mod pointer;
pub mod stability;
