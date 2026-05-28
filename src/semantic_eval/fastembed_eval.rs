//! Local embedding-based link-text evaluator using fastembed.
//!
//! Only compiled when the `semantic-eval` feature is enabled.

#[cfg(feature = "semantic-eval")]
pub use inner::FasembedEvaluator;

#[cfg(feature = "semantic-eval")]
mod inner {
    use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
    use tracing::warn;

    use crate::audit::normalized::AdvisoryFinding;

    /// Reference phrases considered semantically generic for link texts (DE + EN).
    const GENERIC_REFERENCE_PHRASES: &[&str] = &[
        "mehr erfahren",
        "weiterlesen",
        "hier klicken",
        "read more",
        "learn more",
        "click here",
        "here",
        "mehr",
        "more",
        "details",
        "weiter",
    ];

    pub struct FasembedEvaluator {
        model: TextEmbedding,
        threshold: f32,
    }

    impl FasembedEvaluator {
        /// Initialise the evaluator. Downloads the model on first use (~120 MB).
        pub fn try_new(threshold: f32) -> crate::error::Result<Self> {
            let model =
                TextEmbedding::try_new(InitOptions::new(EmbeddingModel::MultilingualE5Small))
                    .map_err(|e| {
                        crate::error::AuditError::ConfigError(format!(
                            "fastembed model init failed: {e}"
                        ))
                    })?;
            Ok(Self { model, threshold })
        }

        /// Evaluate link texts in the AX tree and return advisory findings.
        ///
        /// Emits at most one finding aggregating all flagged links.
        pub fn evaluate_link_texts(
            &mut self,
            tree: &crate::accessibility::AXTree,
        ) -> Vec<AdvisoryFinding> {
            // Embed reference phrases and compute centroid
            let refs: Vec<String> = GENERIC_REFERENCE_PHRASES
                .iter()
                .map(|s| s.to_string())
                .collect();

            let ref_embeddings = match self.model.embed(refs, None) {
                Ok(e) => e,
                Err(e) => {
                    warn!("fastembed: failed to embed reference phrases: {}", e);
                    return Vec::new();
                }
            };

            let centroid = compute_centroid(&ref_embeddings);

            // Collect link names to evaluate
            let links = tree.links();
            let candidates: Vec<(String, String)> = links
                .iter()
                .filter_map(|node| {
                    let name = node.name.as_deref()?;
                    let trimmed = name.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    // Skip if already caught by Phase 3 exact match (≤3 words)
                    let word_count = trimmed.split_whitespace().count();
                    if word_count <= 3 {
                        // Check against the known exact generic list
                        let lower = trimmed.to_lowercase();
                        let already_flagged = GENERIC_REFERENCE_PHRASES
                            .iter()
                            .any(|p| p.to_lowercase() == lower);
                        if already_flagged {
                            return None;
                        }
                    }
                    Some((node.node_id.clone(), trimmed.to_string()))
                })
                .collect();

            if candidates.is_empty() {
                return Vec::new();
            }

            let texts: Vec<String> = candidates.iter().map(|(_, t)| t.clone()).collect();
            let embeddings = match self.model.embed(texts, None) {
                Ok(e) => e,
                Err(e) => {
                    warn!("fastembed: failed to embed link texts: {}", e);
                    return Vec::new();
                }
            };

            let mut flagged: Vec<(String, f32)> = Vec::new();
            for (i, emb) in embeddings.iter().enumerate() {
                let sim = cosine(emb, &centroid);
                if sim > self.threshold {
                    flagged.push((candidates[i].1.clone(), sim));
                }
            }

            if flagged.is_empty() {
                return Vec::new();
            }

            // Sort by similarity descending, take top examples
            flagged.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let max_sim = flagged[0].1;
            let examples: Vec<String> = flagged
                .iter()
                .take(5)
                .map(|(t, _)| format!("\u{201E}{t}\u{201C}"))
                .collect();
            let examples_str = examples.join(", ");
            let count = flagged.len();

            let sim_pct = (max_sim * 100.0) as u32;
            let message = format!(
                "{count} Link(s) mit semantisch generischem Text (z. B. {examples_str}). Fastembed-Ähnlichkeit zu bekannten Nicht-Beschreibungstexten: {sim_pct}%."
            );

            vec![AdvisoryFinding {
                category: "link_text".to_string(),
                message,
                source: "fastembed".to_string(),
                confidence: max_sim,
            }]
        }
    }

    /// Compute the centroid (mean) of a set of embeddings.
    fn compute_centroid(embeddings: &[Vec<f32>]) -> Vec<f32> {
        if embeddings.is_empty() {
            return Vec::new();
        }
        let dim = embeddings[0].len();
        let mut centroid = vec![0.0f32; dim];
        for emb in embeddings {
            for (i, v) in emb.iter().enumerate() {
                centroid[i] += v;
            }
        }
        let n = embeddings.len() as f32;
        for v in &mut centroid {
            *v /= n;
        }
        centroid
    }

    /// Cosine similarity between two vectors.
    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}
