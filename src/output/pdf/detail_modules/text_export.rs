//! The `detail_modules.rs` part of the slice 17 text export of #936 (see
//! `findings/text_export.rs`, which writes both parts into one file).

use super::*;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// Every module key `module_customer_context` knows, plus one it does not.
const MODULE_KEYS: [&str; 12] = [
    "performance",
    "seo",
    "search_experience",
    "security",
    "html_conform",
    "mobile",
    "ux",
    "journey",
    "source_quality",
    "ai_visibility",
    "content_visibility",
    "commerce",
];

/// Every category `journey_category_label` knows, plus two it spells out.
const JOURNEY_CATEGORIES: [&str; 20] = [
    "TabOrder",
    "FocusTrap",
    "StateTransition",
    "FocusRestoration",
    "FormError",
    "SpaNavigation",
    "HiddenFocusable",
    "SkipLink",
    "FocusIndicator",
    "MenuJourney",
    "TabsJourney",
    "LinkText",
    "Landmark",
    "MediaControls",
    "AddToCart",
    "QuantityStepper",
    "HeadingOutline",
    "SomethingNew",
    "snake_case-id",
    "",
];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

/// Every record of the `detail_modules.rs` part, in both languages.
pub(in crate::output::pdf) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        let en = lang == "en";
        for key in MODULE_KEYS {
            for score in [
                None,
                Some(30),
                Some(49),
                Some(50),
                Some(74),
                Some(75),
                Some(90),
            ] {
                records.push(json!([
                    "module_customer_context",
                    format!("{key} score={score:?}"),
                    lang,
                    module_customer_context(&i18n, key, score, "Interpretation.").to_data()
                ]));
            }
        }
        for module in [
            "UX",
            "Journey",
            "AI Visibility",
            "Source Quality",
            "Performance",
        ] {
            records.push(json!([
                "derived_from_note",
                module,
                lang,
                derived_from_note(&i18n, module).map(|label| label.to_data())
            ]));
        }
        for (penalty, before) in [(0, 80), (1, 100), (7, 85)] {
            records.push(json!([
                "a11y_penalty_note",
                format!("penalty={penalty} before={before}"),
                lang,
                components(a11y_penalty_note(bare(), penalty, before, &i18n))
            ]));
        }
        records.push(json!([
            "module_score_caption",
            "",
            lang,
            module_score_caption(&i18n)
        ]));
        for rating in ["good", "needs-improvement", "poor", "unknown"] {
            records.push(json!([
                "vital_rating_label",
                rating,
                lang,
                vital_rating_label(rating, en)
            ]));
        }
        for category in JOURNEY_CATEGORIES {
            records.push(json!([
                "journey_category_label",
                category,
                lang,
                journey_category_label(category, &i18n)
            ]));
        }
    }
    records
}
