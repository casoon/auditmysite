//! Text export of the structured-data rule wording (#936): every text of
//! `schema_rules::feature_label`, `schema_rules::status_text` and
//! `schema_rules::manual_review_text` in both languages, as records in
//! `reports/seo-schema-rules-texts.json` (gitignored; the path can be set with
//! `SEO_SCHEMA_RULES_TEXT_EXPORT`). Run it before and after a change of how
//! the texts are stored and compare the two files byte for byte.

use super::schema_rules::{
    feature_label, manual_review_text, status_text, ManualReview, SchemaFeature,
    SchemaFeatureAvailability, SchemaRequirementStatus, SchemaRuleAssessment,
};
use serde_json::{json, Value};

/// Every requirement status of an assessment.
pub(crate) const REQUIREMENT_STATUSES: [SchemaRequirementStatus; 4] = [
    SchemaRequirementStatus::MeetsRequiredProperties,
    SchemaRequirementStatus::MissingRequiredProperties,
    SchemaRequirementStatus::RecommendationsOnly,
    SchemaRequirementStatus::NotEvaluated,
];

/// Every manual-review point.
pub(crate) const MANUAL_REVIEWS: [ManualReview; 9] = [
    ManualReview::MerchantListingContext,
    ManualReview::CancelledEventKeepsDetails,
    ManualReview::EventIsSinglePublicEvent,
    ManualReview::RecipeCookAndPrepTime,
    ManualReview::RecipeMatchesVisibleRecipe,
    ManualReview::JobPostingIsOpenPosition,
    ManualReview::SoftwareRatingIsGenuine,
    ManualReview::ProfilePageSubject,
    ManualReview::ItemListCanonicalUrls,
];

/// An assessment that differs only in its requirement status.
pub(crate) fn assessment(status: SchemaRequirementStatus) -> SchemaRuleAssessment {
    SchemaRuleAssessment {
        node_index: 0,
        schema_type: "Product".to_string(),
        feature: SchemaFeature::ProductSnippet,
        availability: SchemaFeatureAvailability::General,
        requirement_status: status,
        missing_required: vec![],
        missing_recommended: vec![],
        manual_review: vec![],
        quality_score: None,
        source_url: String::new(),
        reviewed_at: String::new(),
    }
}

/// Every text of the rule wording in one language.
fn schema_rule_texts(en: bool) -> Vec<Value> {
    let mut out = Vec::new();
    for feature in SchemaFeature::ALL {
        out.push(json!([
            "feature_label",
            format!("{feature:?}"),
            feature_label(feature, en)
        ]));
    }
    for status in REQUIREMENT_STATUSES {
        out.push(json!([
            "status_text",
            format!("{status:?}"),
            status_text(&assessment(status), en)
        ]));
    }
    for review in MANUAL_REVIEWS {
        out.push(json!([
            "manual_review_text",
            format!("{review:?}"),
            manual_review_text(review, en)
        ]));
    }
    out
}

#[test]
#[ignore = "writes reports/seo-schema-rules-texts.json; run before and after a change of the text storage"]
fn export_seo_schema_rule_texts() {
    let mut records = Vec::new();
    for lang in ["en", "de"] {
        for text in schema_rule_texts(lang == "en") {
            records.push(json!({ "lang": lang, "text": text }));
        }
    }
    let path = std::env::var("SEO_SCHEMA_RULES_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/seo-schema-rules-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} schema rule records written to {path}", records.len());
}
