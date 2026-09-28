//! Rich-result rule tables for structured data.
//!
//! The tables, the JSON-LD normalization and the assessment live in
//! `web_checks::structured_data` (web-checks 0.4), shared with astro-post-audit.
//! This module keeps what is auditmysite's own: the report wording for
//! features, requirement status and manual-review points (#406 — the
//! assessment carries canonical kinds, the texts are derived here).

pub use web_checks::structured_data::{
    assess_node, inventory_fields, ManualReview, ProductRuleContext, SchemaFeature,
    SchemaFeatureAvailability, SchemaRequirementStatus, SchemaRuleAssessment, RULESET_VERSION,
};

/// Display name of a rich-result feature.
pub fn feature_label(feature: SchemaFeature, en: bool) -> &'static str {
    match feature {
        SchemaFeature::ProductSnippet => "Product Snippet",
        SchemaFeature::MerchantListing => "Merchant Listing",
        SchemaFeature::Article => "Article",
        SchemaFeature::Breadcrumb => {
            if en {
                "Breadcrumb"
            } else {
                "Breadcrumb-Navigation"
            }
        }
        SchemaFeature::Organization => "Organization",
        SchemaFeature::LocalBusiness => "LocalBusiness",
        SchemaFeature::Faq => "FAQ",
        SchemaFeature::Event => "Event",
        SchemaFeature::Recipe => "Recipe",
        SchemaFeature::Video => "Video",
        SchemaFeature::JobPosting => "Job Posting",
        SchemaFeature::SoftwareApplication => "Software Application",
        SchemaFeature::ProfilePage => "Profile Page",
        SchemaFeature::ItemList => "Item List",
        SchemaFeature::WebPage => "WebPage",
        SchemaFeature::WebSite => "WebSite",
        SchemaFeature::Person => "Person",
    }
}

/// Requirement status of an assessment as report text.
pub fn status_text(assessment: &SchemaRuleAssessment, en: bool) -> &'static str {
    match (assessment.requirement_status, en) {
        (SchemaRequirementStatus::MeetsRequiredProperties, true) => "Required properties met",
        (SchemaRequirementStatus::MeetsRequiredProperties, false) => "Pflichtangaben erfüllt",
        (SchemaRequirementStatus::MissingRequiredProperties, true) => "Required properties missing",
        (SchemaRequirementStatus::MissingRequiredProperties, false) => "Pflichtangaben fehlen",
        (SchemaRequirementStatus::RecommendationsOnly, true) => "No required properties",
        (SchemaRequirementStatus::RecommendationsOnly, false) => "Keine Pflichtangaben",
        (SchemaRequirementStatus::NotEvaluated, true) => "Context not established",
        (SchemaRequirementStatus::NotEvaluated, false) => "Kontext nicht nachgewiesen",
    }
}

/// A manual-review point as report text.
pub fn manual_review_text(review: ManualReview, en: bool) -> &'static str {
    match (review, en) {
        (ManualReview::MerchantListingContext, true) => "Confirm that this page is a purchasable product detail page before applying merchant-listing requirements.",
        (ManualReview::MerchantListingContext, false) => "Prüfen, ob dies eine kaufbare Produktdetailseite ist, bevor Merchant-Listing-Anforderungen angewendet werden.",
        (ManualReview::CancelledEventKeepsDetails, true) => "Confirm that the cancelled event retains its original startDate and location.",
        (ManualReview::CancelledEventKeepsDetails, false) => "Prüfen, ob die abgesagte Veranstaltung ihr ursprüngliches startDate und ihren ursprünglichen Ort beibehält.",
        (ManualReview::EventIsSinglePublicEvent, true) => "Confirm that the event is a single bookable public event and all marked-up details are visible on the page.",
        (ManualReview::EventIsSinglePublicEvent, false) => "Prüfen, ob es sich um eine einzelne buchbare öffentliche Veranstaltung handelt und alle ausgezeichneten Angaben sichtbar sind.",
        (ManualReview::RecipeCookAndPrepTime, true) => "Google recommends using cookTime and prepTime together; verify the missing duration.",
        (ManualReview::RecipeCookAndPrepTime, false) => "Google empfiehlt cookTime und prepTime gemeinsam; die fehlende Dauer ist zu prüfen.",
        (ManualReview::RecipeMatchesVisibleRecipe, true) => "Confirm that the page describes preparation of one dish and that ingredients and instructions match the visible recipe.",
        (ManualReview::RecipeMatchesVisibleRecipe, false) => "Prüfen, ob die Seite die Zubereitung eines Gerichts beschreibt und Zutaten sowie Anleitung dem sichtbaren Rezept entsprechen.",
        (ManualReview::JobPostingIsOpenPosition, true) => "Confirm that this is one currently open position with a visible application path; expired jobs must be removed or marked with a past validThrough date.",
        (ManualReview::JobPostingIsOpenPosition, false) => "Prüfen, ob genau eine aktuell offene Stelle mit sichtbarem Bewerbungsweg vorliegt; abgelaufene Stellen müssen entfernt oder mit einem vergangenen validThrough-Datum gekennzeichnet werden.",
        (ManualReview::SoftwareRatingIsGenuine, true) => "Confirm that the rating or review is visible, genuine, and specifically about this application.",
        (ManualReview::SoftwareRatingIsGenuine, false) => "Prüfen, ob Bewertung oder Rezension sichtbar, authentisch und eindeutig dieser Anwendung zugeordnet ist.",
        (ManualReview::ProfilePageSubject, true) => "Confirm that the page is primarily about one person or organization affiliated with the site.",
        (ManualReview::ProfilePageSubject, false) => "Prüfen, ob die Seite hauptsächlich eine mit der Website verbundene Person oder Organisation beschreibt.",
        (ManualReview::ItemListCanonicalUrls, true) => "Confirm that each listed URL is unique and points to a canonical detail page represented by the visible list.",
        (ManualReview::ItemListCanonicalUrls, false) => "Prüfen, ob jede aufgeführte URL eindeutig ist und auf eine kanonische Detailseite verweist, die in der sichtbaren Liste enthalten ist.",
    }
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
