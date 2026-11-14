//! Rich-result rule tables for structured data.
//!
//! The tables, the JSON-LD normalization and the assessment live in
//! `web_checks::structured_data` (web-checks 0.4), shared with astro-post-audit.
//! This module keeps what is auditmysite's own: the report wording for
//! features, requirement status and manual-review points (#406 — the
//! assessment carries canonical kinds, the texts are derived here; the
//! language-dependent ones are `seo-schema-rules-*` messages of
//! `locales/{de,en}/seo-issues.ftl`, #936).

pub use web_checks::structured_data::{
    assess_node, inventory_fields, ManualReview, ProductRuleContext, SchemaFeature,
    SchemaFeatureAvailability, SchemaRequirementStatus, SchemaRuleAssessment, RULESET_VERSION,
};

/// Display name of a rich-result feature. Only the breadcrumb name differs
/// by language (`locales/{de,en}/seo-issues.ftl`); the others are
/// language-neutral.
pub fn feature_label(feature: SchemaFeature, en: bool) -> String {
    match feature {
        SchemaFeature::ProductSnippet => "Product Snippet".to_string(),
        SchemaFeature::MerchantListing => "Merchant Listing".to_string(),
        SchemaFeature::Article => "Article".to_string(),
        SchemaFeature::Breadcrumb => crate::i18n::text("seo-schema-rules-feature-breadcrumb", en),
        SchemaFeature::Organization => "Organization".to_string(),
        SchemaFeature::LocalBusiness => "LocalBusiness".to_string(),
        SchemaFeature::Faq => "FAQ".to_string(),
        SchemaFeature::Event => "Event".to_string(),
        SchemaFeature::Recipe => "Recipe".to_string(),
        SchemaFeature::Video => "Video".to_string(),
        SchemaFeature::JobPosting => "Job Posting".to_string(),
        SchemaFeature::SoftwareApplication => "Software Application".to_string(),
        SchemaFeature::ProfilePage => "Profile Page".to_string(),
        SchemaFeature::ItemList => "Item List".to_string(),
        SchemaFeature::WebPage => "WebPage".to_string(),
        SchemaFeature::WebSite => "WebSite".to_string(),
        SchemaFeature::Person => "Person".to_string(),
    }
}

/// Requirement status of an assessment as report text
/// (`locales/{de,en}/seo-issues.ftl`).
pub fn status_text(assessment: &SchemaRuleAssessment, en: bool) -> String {
    let key = match assessment.requirement_status {
        SchemaRequirementStatus::MeetsRequiredProperties => {
            "seo-schema-rules-status-meets-required"
        }
        SchemaRequirementStatus::MissingRequiredProperties => {
            "seo-schema-rules-status-missing-required"
        }
        SchemaRequirementStatus::RecommendationsOnly => {
            "seo-schema-rules-status-recommendations-only"
        }
        SchemaRequirementStatus::NotEvaluated => "seo-schema-rules-status-not-evaluated",
    };
    crate::i18n::text(key, en)
}

/// A manual-review point as report text (`locales/{de,en}/seo-issues.ftl`).
pub fn manual_review_text(review: ManualReview, en: bool) -> String {
    let key = match review {
        ManualReview::MerchantListingContext => "seo-schema-rules-review-merchant-listing-context",
        ManualReview::CancelledEventKeepsDetails => {
            "seo-schema-rules-review-cancelled-event-keeps-details"
        }
        ManualReview::EventIsSinglePublicEvent => {
            "seo-schema-rules-review-event-is-single-public-event"
        }
        ManualReview::RecipeCookAndPrepTime => "seo-schema-rules-review-recipe-cook-and-prep-time",
        ManualReview::RecipeMatchesVisibleRecipe => {
            "seo-schema-rules-review-recipe-matches-visible-recipe"
        }
        ManualReview::JobPostingIsOpenPosition => {
            "seo-schema-rules-review-job-posting-is-open-position"
        }
        ManualReview::SoftwareRatingIsGenuine => {
            "seo-schema-rules-review-software-rating-is-genuine"
        }
        ManualReview::ProfilePageSubject => "seo-schema-rules-review-profile-page-subject",
        ManualReview::ItemListCanonicalUrls => "seo-schema-rules-review-item-list-canonical-urls",
    };
    crate::i18n::text(key, en)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GERMAN: [char; 7] = ['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß'];

    #[test]
    fn english_wording_has_no_german_characters() {
        for feature in [
            SchemaFeature::ProductSnippet,
            SchemaFeature::MerchantListing,
            SchemaFeature::Article,
            SchemaFeature::Breadcrumb,
            SchemaFeature::Organization,
            SchemaFeature::LocalBusiness,
            SchemaFeature::Faq,
            SchemaFeature::Event,
            SchemaFeature::Recipe,
            SchemaFeature::Video,
            SchemaFeature::JobPosting,
            SchemaFeature::SoftwareApplication,
            SchemaFeature::ProfilePage,
            SchemaFeature::ItemList,
            SchemaFeature::WebPage,
            SchemaFeature::WebSite,
            SchemaFeature::Person,
        ] {
            assert!(!feature_label(feature, true).contains(GERMAN));
        }
        for review in [
            ManualReview::MerchantListingContext,
            ManualReview::CancelledEventKeepsDetails,
            ManualReview::EventIsSinglePublicEvent,
            ManualReview::RecipeCookAndPrepTime,
            ManualReview::RecipeMatchesVisibleRecipe,
            ManualReview::JobPostingIsOpenPosition,
            ManualReview::SoftwareRatingIsGenuine,
            ManualReview::ProfilePageSubject,
            ManualReview::ItemListCanonicalUrls,
        ] {
            assert!(!manual_review_text(review, true).contains(GERMAN));
        }
    }
}
