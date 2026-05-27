//! Browser interaction primitives for the Accessibility-Journey-Layer.
//!
//! Real CDP events (keyboard + mouse) over synthetic JS clicks — see
//! `pointer::synthetic_click_backend` for the documented fallback.
//!
//! This module is *actions only*: no evaluation, no journey logic. Journey
//! orchestration lives in `crate::a11y_journey`.

pub mod focus;
pub mod keyboard;
pub mod pointer;
pub mod stability;
