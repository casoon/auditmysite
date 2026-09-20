//! Shared test-only helpers for the detection-corpus effort (#552–#558).
//! Not itself a test binary — included via `mod common;` from the test
//! files that need it (`tests/common/mod.rs` is the standard Rust idiom for
//! sharing code between separate `tests/*.rs` integration-test crates).
//! Each integration-test binary is its own crate, so `mod common;` recompiles
//! this whole module per binary — any single binary only uses part of the
//! API, which clippy's dead-code lint can't see across crate boundaries.

#![allow(dead_code)]

pub mod detection_corpus;
pub mod fixture_server;
pub mod nonwcag_rule_inventory;
pub mod rule_inventory;
