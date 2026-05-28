//! Semantic AI evaluation (Phase 4).
//!
//! Two optional providers:
//! - `FasembedEvaluator` — local embeddings for link-text quality (feature `semantic-eval`)
//! - `MistralEvaluator` — Mistral API for heading outline + blind-user perspective
//!
//! Results go into `NormalizedReport.advisory_findings`. They are purely
//! advisory — they never influence score or risk level.

pub mod fastembed_eval;
pub mod mistral;
pub mod prompts;

use crate::audit::normalized::AdvisoryFinding;
use tracing::warn;

/// Configuration for semantic AI evaluation.
#[derive(Debug, Clone)]
pub struct SemanticEvalConfig {
    /// Whether semantic evaluation is enabled at all (`--semantic-eval`).
    pub enabled: bool,
    /// Optional Mistral API key. When `None`, the Mistral evaluator is skipped.
    pub mistral_api_key: Option<String>,
    /// Mistral model to use.
    pub mistral_model: String,
    /// Cosine-similarity threshold for the fastembed link-text evaluator.
    pub similarity_threshold: f32,
}

impl Default for SemanticEvalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mistral_api_key: None,
            mistral_model: "mistral-small-latest".to_string(),
            similarity_threshold: 0.62,
        }
    }
}

/// Run all enabled semantic evaluators and collect advisory findings.
///
/// Returns an empty vec immediately when `config.enabled` is false.
pub async fn run(
    config: &SemanticEvalConfig,
    ax_tree: &crate::accessibility::AXTree,
) -> Vec<AdvisoryFinding> {
    if !config.enabled {
        return Vec::new();
    }

    let mut findings: Vec<AdvisoryFinding> = Vec::new();

    // ── Fastembed (feature-gated) ─────────────────────────────────────────────
    #[cfg(feature = "semantic-eval")]
    {
        match fastembed_eval::FasembedEvaluator::try_new(config.similarity_threshold) {
            Ok(mut evaluator) => {
                let link_findings = evaluator.evaluate_link_texts(ax_tree);
                findings.extend(link_findings);
            }
            Err(e) => {
                warn!("FasembedEvaluator init failed: {}", e);
            }
        }
    }

    // ── Mistral (always compiled, skipped when no key) ────────────────────────
    if let Some(api_key) = &config.mistral_api_key {
        let evaluator =
            mistral::MistralEvaluator::new(api_key.clone(), config.mistral_model.clone());

        match evaluator.evaluate_heading_outline(ax_tree).await {
            Ok(f) => findings.extend(f),
            Err(e) => warn!("Mistral heading_outline evaluation failed: {}", e),
        }

        match evaluator.evaluate_blind_user_perspective(ax_tree).await {
            Ok(f) => findings.extend(f),
            Err(e) => warn!("Mistral blind_user_perspective evaluation failed: {}", e),
        }
    }

    findings
}
