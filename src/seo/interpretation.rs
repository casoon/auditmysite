//! SEO interpretation — locale-neutral decision logic plus the single localized
//! text source (#406).
//!
//! The classifiers (`classify_seo_score`, `classify_page_type_fit`) are pure
//! decisions over numbers; the `*_text(.., en)` functions are the ONLY place the
//! wording lives. Callers pass `en` (JSON canonical English uses `true`, the PDF
//! uses the run locale), so both surfaces share one source instead of the
//! builder carrying hardcoded de/en branches.

use crate::i18n::{text, text_args};
use crate::seo::profile::SeoContentProfile;
use crate::seo::SeoAnalysis;
use fluent_bundle::FluentValue;

/// SEO score band used to pick the lead interpretation sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeoScoreBand {
    Excellent,
    Good,
    Partial,
    Insufficient,
    Critical,
}

pub fn classify_seo_score(score: u32) -> SeoScoreBand {
    match crate::registry::SEO_BAND.label(score as f32, false) {
        "excellent" => SeoScoreBand::Excellent,
        "good" => SeoScoreBand::Good,
        "partial" => SeoScoreBand::Partial,
        "insufficient" => SeoScoreBand::Insufficient,
        _ => SeoScoreBand::Critical,
    }
}

fn seo_score_lead(band: SeoScoreBand, en: bool) -> String {
    let key = match band {
        SeoScoreBand::Excellent => "audit-interpretation-seo-lead-excellent",
        SeoScoreBand::Good => "audit-interpretation-seo-lead-good",
        SeoScoreBand::Partial => "audit-interpretation-seo-lead-partial",
        SeoScoreBand::Insufficient => "audit-interpretation-seo-lead-insufficient",
        SeoScoreBand::Critical => "audit-interpretation-seo-lead-critical",
    };
    text(key, en)
}

/// Full SEO interpretation: lead sentence, optional page-type context and a
/// content-depth caveat when technical SEO is strong but content is thin.
pub fn seo_interpretation_text(seo: &SeoAnalysis, en: bool) -> String {
    let lead = seo_score_lead(classify_seo_score(seo.score), en);

    let Some(profile) = &seo.content_profile else {
        return lead;
    };

    let page_type = profile.page_classification.primary_type.label(en);
    let content_depth = profile.page_classification.content_depth_score;

    // The page type is stated, not scored against a "reference value".
    // `intent_fit_score` used to be presented here as the score expected for
    // this page type — but it is one of eleven constants the tool defines, not
    // an observed value from any reference population, and it was compared
    // against `seo.score`, which a different function produces on a different
    // scale. The comparison also inverted for thin pages: `ThinContent` has
    // the lowest constant (28), so a thin page scoring 30 was told it met
    // expectations for its type (plan 39).
    //
    // `intent_fit_score` itself is kept — as a quality reading it is sound and
    // is still shown as the "Intent-Fit" metric and used by the
    // `intent_fit_score < 65` advice gate below.
    let context = text_args(
        "audit-interpretation-seo-page-type",
        &[("page_type", FluentValue::from(page_type))],
        en,
    );

    // When technical SEO is strong but content depth is weak, make the gap
    // explicit so readers don't interpret a high SEO score as endorsing content.
    let mut interpretation = format!("{lead} {context}");
    if seo.score >= 80 && content_depth < 55 {
        interpretation.push(' ');
        interpretation.push_str(&text_args(
            "audit-interpretation-seo-depth-note",
            &[("depth", FluentValue::from(content_depth.to_string()))],
            en,
        ));
    }
    interpretation
}

/// Reader-facing summary of a page's content profile (type + structural quality).
pub fn page_profile_summary_text(profile: &SeoContentProfile, en: bool) -> String {
    let classification = &profile.page_classification;
    let avg = crate::seo::average_page_semantic_score(classification);
    let quality = text(
        match avg {
            85..=100 => "audit-interpretation-seo-quality-coherent",
            70..=84 => "audit-interpretation-seo-quality-supported",
            50..=69 => "audit-interpretation-seo-quality-partial",
            _ => "audit-interpretation-seo-quality-weak",
        },
        en,
    );

    let mut traits = classification.attributes.clone();
    if traits.is_empty() {
        traits.push(text("audit-interpretation-seo-no-attributes", en));
    }

    text_args(
        "audit-interpretation-seo-profile-summary",
        &[
            (
                "page_type",
                FluentValue::from(classification.primary_type.label(en)),
            ),
            ("quality", FluentValue::from(quality)),
            ("traits", FluentValue::from(traits.join(", "))),
        ],
        en,
    )
}

/// The single biggest content-optimization lever for a page profile.
pub fn page_profile_optimization_note_text(profile: &SeoContentProfile, en: bool) -> String {
    let classification = &profile.page_classification;
    if classification.content_depth_score < 45 {
        return text("audit-interpretation-seo-note-depth", en);
    }
    if classification.structural_richness_score < 55 {
        return text("audit-interpretation-seo-note-structure", en);
    }
    if classification.media_text_balance_score < 55 {
        return text("audit-interpretation-seo-note-media", en);
    }
    // Only when the intent-fit value actually read a measured signal. For
    // MediaHeavy, Utility and ThinContent it is a constant keyed on the page
    // type, so gating this sentence on it would assert something about *this
    // page* on the strength of its classification alone — a media-heavy page
    // would receive it unconditionally, however good it is (plan 39). Pages
    // with a genuinely poor balance are already caught by the
    // `media_text_balance_score` gate above, with a measured reason.
    if crate::seo::profile::intent_fit_is_signal_derived(&classification.primary_type)
        && classification.intent_fit_score < 65
    {
        return text("audit-interpretation-seo-note-intent", en);
    }
    text("audit-interpretation-seo-note-fine", en)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seo::profile::PageType;

    /// Minimal `SeoAnalysis` carrying just the page classification the
    /// interpretation reads. Built through serde so the fixture does not have
    /// to name every unrelated nested field.
    fn profile_with(
        page_type: &PageType,
        intent_fit: u32,
    ) -> crate::seo::profile::SeoContentProfile {
        let mut profile = seo_with_page_type(70, page_type)
            .content_profile
            .expect("fixture has a profile");
        profile.page_classification.intent_fit_score = intent_fit;
        // Keep the earlier, measured gates satisfied so the intent-fit gate is
        // the one under test.
        profile.page_classification.content_depth_score = 80;
        profile.page_classification.structural_richness_score = 80;
        profile.page_classification.media_text_balance_score = 80;
        profile
    }

    fn seo_with_page_type(score: u32, page_type: &PageType) -> SeoAnalysis {
        let primary_type = serde_json::to_value(page_type).expect("page type serializes");
        let profile: crate::seo::profile::SeoContentProfile =
            serde_json::from_value(serde_json::json!({
                "content_identity": {
                    "summary": "",
                    "site_name": "",
                    "content_type": "",
                    "language": "en",
                    "category_hints": [],
                },
                "page_classification": {
                    "primary_type": primary_type,
                    "attributes": [],
                    "content_depth_score": 60,
                    "structural_richness_score": 60,
                    "media_text_balance_score": 60,
                    "intent_fit_score": 82,
                },
                "schema_inventory": { "schemas": [], "total_count": 0 },
                "signal_strength": { "categories": [], "overall_pct": 0 },
                "maturity": "basic",
                "maturity_techniques": 0,
            }))
            .expect("profile fixture deserializes");

        SeoAnalysis {
            score,
            content_profile: Some(profile),
            ..Default::default()
        }
    }

    #[test]
    fn score_bands_classify_at_boundaries() {
        assert_eq!(classify_seo_score(90), SeoScoreBand::Excellent);
        assert_eq!(classify_seo_score(70), SeoScoreBand::Good);
        assert_eq!(classify_seo_score(55), SeoScoreBand::Partial);
        assert_eq!(classify_seo_score(35), SeoScoreBand::Insufficient);
        assert_eq!(classify_seo_score(34), SeoScoreBand::Critical);
    }

    /// Plan 39: the page-type sentence must not claim a reference value.
    /// `intent_fit_score` is one of eleven constants the tool defines, not an
    /// observed benchmark, and it was being compared against `seo.score`,
    /// which is produced by a different function on a different scale.
    #[test]
    fn page_type_context_claims_no_reference_value() {
        for (seo_score, page_type) in [
            (30u32, PageType::ThinContent),
            (65, PageType::NavigationHub),
            (95, PageType::Editorial),
        ] {
            let page_type = &page_type;
            for en in [false, true] {
                let text = seo_interpretation_text(&seo_with_page_type(seo_score, page_type), en);
                for claim in [
                    "Erwartungswert",
                    "erwarteten Bereich",
                    "Referenz",
                    "reference value",
                    "reference for this page type",
                ] {
                    assert!(
                        !text.contains(claim),
                        "{page_type:?}/{seo_score} (en={en}) still claims a benchmark: {text}",
                    );
                }
            }
        }
    }

    /// The page type itself is still reported — dropping the comparison must
    /// not drop the classification.
    #[test]
    fn page_type_is_still_named() {
        let text =
            seo_interpretation_text(&seo_with_page_type(65, &PageType::NavigationHub), false);
        assert!(
            text.contains(&PageType::NavigationHub.label(false)),
            "page type missing: {text}",
        );
    }

    /// Plan 39: the "does not serve its page type" advice must not fire on a
    /// value that is a pure page-type constant. `MediaHeavy` scores a fixed
    /// 62, so before this it received the sentence unconditionally.
    #[test]
    fn type_constant_intent_fit_does_not_drive_page_advice() {
        for page_type in [
            PageType::MediaHeavy,
            PageType::Utility,
            PageType::ThinContent,
        ] {
            assert!(
                !crate::seo::profile::intent_fit_is_signal_derived(&page_type),
                "{page_type:?} is expected to be a type constant",
            );

            // All measured sub-scores healthy, intent-fit below the gate.
            let profile = profile_with(&page_type, 62);
            for en in [false, true] {
                let note = page_profile_optimization_note_text(&profile, en);
                for claim in [
                    "bedient ihren Seitentyp noch nicht sauber",
                    "does not yet serve its page type cleanly",
                ] {
                    assert!(!note.contains(claim), "{page_type:?}: {note}");
                }
            }
        }
    }

    /// ...while the page types whose intent-fit does read a measured signal
    /// must keep the advice.
    #[test]
    fn signal_derived_intent_fit_still_drives_page_advice() {
        let profile = profile_with(&PageType::NavigationHub, 60);
        let note = page_profile_optimization_note_text(&profile, false);
        assert!(
            note.contains("bedient ihren Seitentyp noch nicht sauber"),
            "{note}",
        );
    }

    /// The inversion this fix removes: a thin page must never read as being
    /// in good shape for its type. `ThinContent` carried the lowest constant
    /// (28), so a thin page scoring 30 used to "meet expectations".
    #[test]
    fn a_thin_page_never_reads_as_meeting_expectations() {
        for en in [false, true] {
            let text = seo_interpretation_text(&seo_with_page_type(30, &PageType::ThinContent), en);
            for praise in ["liegt im erwarteten Bereich", "meets the reference"] {
                assert!(!text.contains(praise), "thin page praised: {text}");
            }
        }
    }

    #[test]
    fn english_lead_has_no_german_umlauts() {
        // #406 guard: EN output must not leak German characters.
        for band in [
            SeoScoreBand::Excellent,
            SeoScoreBand::Good,
            SeoScoreBand::Partial,
            SeoScoreBand::Insufficient,
            SeoScoreBand::Critical,
        ] {
            let lead = seo_score_lead(band, true);
            assert!(
                !lead.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "EN lead contains German umlaut: {lead}"
            );
        }
    }
}
