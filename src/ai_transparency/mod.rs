//! AI-transparency analysis: EU AI Act Art. 50 image-provenance check (opt-in,
//! `--ai-transparency`, requires the `ai-transparency` Cargo feature to do
//! anything — see `module.rs`).
//!
//! Reads any C2PA ("Content Credentials") manifest already embedded in an
//! image's bytes and, if it asserts AI/algorithmic generation, produces a
//! **manual-review advisory** finding — never a hard violation, never
//! score-affecting (own finding type, never touches `push_indicator`/
//! `ModuleScoreEntry`, same construction-level guarantee as `design_quality`).
//!
//! Deliberately does **not** attempt to detect whether a human-visible
//! disclosure already exists near the image (selector proximity/alt-text
//! scanning) — too fragile to automate reliably (same class of problem as
//! chat-widget disclosure detection). This module only reports the
//! machine-readable provenance signal itself; the human judgment of whether
//! disclosure is adequate is left to the reviewer.
//!
//! Also deliberately does **not** fall back to EXIF `Software`-tag string
//! matching — C2PA is the standard the EU AI Act's machine-readable-marking
//! duty (Art. 50(2)) actually references; an EXIF heuristic would be a
//! second, much weaker detection path for uncertain benefit.

#[cfg(feature = "ai-transparency")]
mod image_provenance;
pub mod module;

pub use module::AiTransparencyModule;

use serde::{Deserialize, Serialize};

/// Subset of C2PA/IPTC `digitalSourceType` values that indicate at least
/// partial AI/algorithmic generation — the only signal this module surfaces.
///
/// Deliberately a local, plain enum rather than re-exporting `c2pa::
/// DigitalSourceType` directly: this type (and the whole JSON/PDF surface
/// built on it) must compile and serialize identically regardless of whether
/// the `ai-transparency` Cargo feature (which pulls in the `c2pa` crate) is
/// enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProvenanceKind {
    /// IPTC `trainedAlgorithmicMedia` — media created by an AI model trained
    /// on captured content (the clearest "AI-generated image" case).
    TrainedAlgorithmicMedia,
    /// IPTC `compositeWithTrainedAlgorithmicMedia` — generative-AI-assisted
    /// augmentation/correction (e.g. inpainting/outpainting).
    CompositeWithTrainedAlgorithmicMedia,
    /// IPTC `compositeSynthetic` — a mix of elements where at least one is
    /// generative AI.
    CompositeSynthetic,
    /// IPTC `virtualRecording` — a virtual event recording based on
    /// generative AI and/or captured elements.
    VirtualRecording,
    /// C2PA-specific `trainedAlgorithmicData` — algorithmically produced data
    /// (not itself a media type), included for completeness.
    TrainedAlgorithmicData,
}

impl AiProvenanceKind {
    /// Stable short label for `evidence` text — not the sentence-level
    /// message (see `finding_message_text`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TrainedAlgorithmicMedia => "trained_algorithmic_media",
            Self::CompositeWithTrainedAlgorithmicMedia => {
                "composite_with_trained_algorithmic_media"
            }
            Self::CompositeSynthetic => "composite_synthetic",
            Self::VirtualRecording => "virtual_recording",
            Self::TrainedAlgorithmicData => "trained_algorithmic_data",
        }
    }
}

/// Mirrors `c2pa::ValidationState` (Invalid/Valid/Trusted) without depending
/// on the crate for the same reason as `AiProvenanceKind`. Folded into every
/// finding's evidence so an unsigned/failed-validation manifest is never
/// presented with the same confidence as a validated one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestValidation {
    Invalid,
    Valid,
    Trusted,
}

impl ManifestValidation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "invalid",
            Self::Valid => "valid",
            Self::Trusted => "trusted",
        }
    }
}

/// A single image found to carry an AI-generation-asserting C2PA manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageProvenanceFinding {
    /// Stable identifier, currently always `"ai_transparency.c2pa_provenance"`
    /// (kept as a field, not a constant, so a future second rule can reuse
    /// this same finding shape without a breaking change).
    pub rule_id: String,
    pub image_url: String,
    pub provenance: AiProvenanceKind,
    pub validation: ManifestValidation,
    /// Generator tool name/version from the manifest's claim generator or
    /// action software agent, when present (e.g. `"Adobe Firefly"`).
    pub generator: Option<String>,
    /// Always `"c2pa_manifest"` — a read structured metadata fact, not a
    /// heuristic guess and not a conformance-grade measurement.
    pub measurement_type: String,
    /// Bounded evidence: provenance kind, generator, validation state.
    pub evidence: String,
    /// Canonical English message (#406 pattern — JSON stays English, the PDF
    /// builder re-derives with the run locale via `finding_message_text`).
    pub message: String,
}

/// Results of the ai-transparency analysis for one page.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiTransparencyAnalysis {
    pub findings: Vec<ImageProvenanceFinding>,
    /// Number of distinct images actually fetched and checked (bounded by
    /// `image_provenance::MAX_IMAGES_CHECKED`) — lets the report distinguish
    /// "checked, none flagged" from "nothing to check".
    pub images_checked: usize,
    /// What happened to every image on the page (#789).
    #[serde(default)]
    pub coverage: ImageCoverage,
}

/// Fetch coverage of the image check: how many images were attempted,
/// checked, left out by the per-page limit, and why the rest failed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageCoverage {
    pub attempted: usize,
    pub checked: usize,
    /// Images beyond the per-page limit, not fetched.
    pub skipped_over_limit: usize,
    /// Failed fetches by reason (`redirect_not_followed`, `oversized`,
    /// `timeout`, `host_not_public`, …).
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub failed: std::collections::BTreeMap<String, usize>,
}

/// The `<img>` source URLs of the page.
pub(crate) static IMAGE_URLS: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "ai_transparency.image_urls",
        module: "ai_transparency",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 1,
        scoring: false,
    };

/// Hardened fetches of those images for their C2PA manifests.
pub(crate) static IMAGE_FETCHES: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "ai_transparency.image_fetches",
        module: "ai_transparency",
        method: crate::audit::collection::CollectionMethod::Http,
        method_version: 2,
        scoring: false,
    };

/// The collection state of the image fetches: every image checked is a
/// measurement, any failure or image beyond the limit makes it partial, no
/// image checked at all a failure (#789).
#[cfg(any(feature = "ai-transparency", test))]
pub(crate) fn image_fetch_state(
    coverage: &ImageCoverage,
) -> crate::audit::collection::Collected<()> {
    use crate::audit::collection::Collected;
    let failed: usize = coverage.failed.values().sum();
    if coverage.attempted == 0 && coverage.skipped_over_limit == 0 {
        Collected::Empty(())
    } else if coverage.checked == 0 && failed > 0 {
        Collected::Failed("image_fetch_failed")
    } else if failed > 0 || coverage.skipped_over_limit > 0 {
        let mut gaps = Vec::new();
        if failed > 0 {
            gaps.push("image_fetch");
        }
        if coverage.skipped_over_limit > 0 {
            gaps.push("image_limit");
        }
        Collected::Partial { value: (), gaps }
    } else {
        Collected::Measured(())
    }
}

/// Counter-cases of [`IMAGE_URLS`] and [`IMAGE_FETCHES`] (#789). The fetch
/// itself is tested against local fixtures under the `ai-transparency`
/// feature (`image_provenance` tests).
#[cfg(test)]
pub(crate) fn counter_cases() {
    use crate::audit::collection::CollectionState;
    let coverage = |attempted, checked, skipped, failed: &[(&str, usize)]| ImageCoverage {
        attempted,
        checked,
        skipped_over_limit: skipped,
        failed: failed.iter().map(|(r, n)| (r.to_string(), *n)).collect(),
    };
    assert_eq!(
        image_fetch_state(&coverage(0, 0, 0, &[])).state(),
        CollectionState::Empty
    );
    assert_eq!(
        image_fetch_state(&coverage(3, 3, 0, &[])).state(),
        CollectionState::Measured
    );
    assert_eq!(
        image_fetch_state(&coverage(3, 2, 0, &[("timeout", 1)])).state(),
        CollectionState::Partial
    );
    assert_eq!(
        image_fetch_state(&coverage(15, 15, 4, &[])).state(),
        CollectionState::Partial
    );
    assert_eq!(
        image_fetch_state(&coverage(2, 0, 0, &[("oversized", 2)])).state(),
        CollectionState::Failed
    );
}

/// Localized presentation text for an ai-transparency finding (#406
/// message-baked-struct pattern: the analysis layer bakes
/// `ImageProvenanceFinding.message` by calling this with `en=true` — JSON
/// stays canonical English — the PDF builder calls it again with the run
/// locale). Per-instance specifics (provenance kind, generator, validation)
/// live in `evidence`, not in this generic prose.
pub fn finding_message_text(rule_id: &str, en: bool) -> String {
    match rule_id {
        "ai_transparency.c2pa_provenance" => if en {
            "This image carries a C2PA Content Credentials manifest asserting AI/algorithmic generation. Verify a human-visible disclosure exists for this image (EU AI Act Art. 50 transparency duties)."
        } else {
            "Dieses Bild trägt ein C2PA-\"Content Credentials\"-Manifest, das eine KI-/algorithmische Erzeugung ausweist. Prüfen Sie, ob für dieses Bild eine für Menschen wahrnehmbare Kennzeichnung existiert (EU-AI-Act-Transparenzpflichten, Art. 50)."
        }
        .to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_message_text_has_no_known_german_leaks() {
        // #406 guard: canonical (en=true) message text must be real English,
        // never a German sentence with the umlaut/ß left in.
        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));
        let msg = finding_message_text("ai_transparency.c2pa_provenance", true);
        assert!(!msg.is_empty());
        assert!(!has_umlaut(&msg), "EN message leaks German: {msg}");
    }

    #[test]
    fn german_message_text_differs_from_english() {
        let en = finding_message_text("ai_transparency.c2pa_provenance", true);
        let de = finding_message_text("ai_transparency.c2pa_provenance", false);
        assert_ne!(en, de);
    }

    #[test]
    fn unknown_rule_id_yields_empty_message() {
        assert_eq!(finding_message_text("ai_transparency.unknown", true), "");
    }
}
