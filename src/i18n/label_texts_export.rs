//! Text export of the enum labels of slice 4 of #936: every text of
//! `security::SecurityCategory::label`, `seo::profile::SeoMaturityLevel::label`
//! and `::description`, `seo::profile::PageType::label`,
//! `seo::serp::SerpSignalStatus::label`, `page_flow::PageIntent::label`,
//! `audit::prioritization::{Priority, Effort, ExecutionPriority}::label`,
//! `Effort::short_label`,
//! `output::report_model::Role::label`, `taxonomy::IssueClass::label` (and its
//! `Display`) and `taxonomy::Subcategory::label`, for every variant in both
//! languages, as records in `reports/label-texts.json` (gitignored; the path
//! can be set with `LABEL_TEXT_EXPORT`). Run it before and after a change of
//! how the texts are stored and compare the two files byte for byte.

use crate::audit::prioritization::{Effort, ExecutionPriority, Priority};
use crate::output::report_model::Role;
use crate::page_flow::PageIntent;
use crate::security::SecurityCategory;
use crate::seo::profile::{PageType, SeoMaturityLevel};
use crate::seo::serp::SerpSignalStatus;
use crate::taxonomy::{IssueClass, Subcategory};
use serde_json::{json, Value};

pub(crate) const SEO_MATURITY_LEVELS: [SeoMaturityLevel; 4] = [
    SeoMaturityLevel::Basic,
    SeoMaturityLevel::Standard,
    SeoMaturityLevel::Advanced,
    SeoMaturityLevel::Professional,
];

pub(crate) fn page_types() -> [PageType; 7] {
    [
        PageType::Editorial,
        PageType::StructuredContent,
        PageType::MarketingLanding,
        PageType::MediaHeavy,
        PageType::Utility,
        PageType::NavigationHub,
        PageType::ThinContent,
    ]
}

pub(crate) fn serp_statuses() -> [SerpSignalStatus; 3] {
    [
        SerpSignalStatus::Ok,
        SerpSignalStatus::Warning,
        SerpSignalStatus::Fail,
    ]
}

pub(crate) const PAGE_INTENTS: [PageIntent; 7] = [
    PageIntent::Shop,
    PageIntent::LeadGen,
    PageIntent::Editorial,
    PageIntent::Marketing,
    PageIntent::Corporate,
    PageIntent::Hub,
    PageIntent::Unknown,
];

pub(crate) const PRIORITIES: [Priority; 4] = [
    Priority::Low,
    Priority::Medium,
    Priority::High,
    Priority::Critical,
];

pub(crate) const EFFORTS: [Effort; 3] = [Effort::Quick, Effort::Medium, Effort::Structural];

pub(crate) const EXECUTION_PRIORITIES: [ExecutionPriority; 3] = [
    ExecutionPriority::Optional,
    ExecutionPriority::Important,
    ExecutionPriority::Immediate,
];

pub(crate) const ROLES: [Role; 4] = [
    Role::Development,
    Role::Editorial,
    Role::DesignUx,
    Role::ProjectManagement,
];

pub(crate) const ISSUE_CLASSES: [IssueClass; 6] = [
    IssueClass::Missing,
    IssueClass::Invalid,
    IssueClass::Weak,
    IssueClass::Risk,
    IssueClass::Opportunity,
    IssueClass::Informational,
];

pub(crate) const SUBCATEGORIES: [Subcategory; 26] = [
    Subcategory::ContentAlternatives,
    Subcategory::StructureSemantics,
    Subcategory::NavigationInteraction,
    Subcategory::FormsInteraction,
    Subcategory::LanguageClarity,
    Subcategory::TechnicalRobustness,
    Subcategory::VisualPresentation,
    Subcategory::LoadBehavior,
    Subcategory::Interactivity,
    Subcategory::VisualStability,
    Subcategory::ResourceUsage,
    Subcategory::TechnicalComplexity,
    Subcategory::SnippetMetadata,
    Subcategory::ContentStructure,
    Subcategory::Indexability,
    Subcategory::Linking,
    Subcategory::SemanticSignals,
    Subcategory::Transport,
    Subcategory::Headers,
    Subcategory::BrowserProtection,
    Subcategory::ServerConfiguration,
    Subcategory::Viewport,
    Subcategory::TouchUsability,
    Subcategory::Readability,
    Subcategory::ResponsiveLayout,
    Subcategory::ContentSizing,
];

/// `[function, variant, language, text]` for every variant in both languages.
fn records<T: std::fmt::Debug, S: AsRef<str>>(
    function: &str,
    variants: impl IntoIterator<Item = T>,
    text: impl Fn(&T, bool) -> S,
) -> Vec<Value> {
    let mut out = Vec::new();
    for variant in variants {
        for (lang, en) in [("en", true), ("de", false)] {
            out.push(json!([
                function,
                format!("{variant:?}"),
                lang,
                text(&variant, en).as_ref()
            ]));
        }
    }
    out
}

#[test]
#[ignore = "writes reports/label-texts.json; run before and after a change of the text storage"]
fn export_label_texts() {
    let mut labels = Vec::new();
    labels.extend(records(
        "SecurityCategory::label",
        SecurityCategory::all(),
        |c, en| c.label(en),
    ));
    labels.extend(records(
        "SeoMaturityLevel::label",
        SEO_MATURITY_LEVELS,
        |l, en| l.label(en),
    ));
    labels.extend(records(
        "SeoMaturityLevel::description",
        SEO_MATURITY_LEVELS,
        |l, en| l.description(en),
    ));
    labels.extend(records("PageType::label", page_types(), |t, en| {
        t.label(en)
    }));
    labels.extend(records(
        "SerpSignalStatus::label",
        serp_statuses(),
        |s, en| s.label(en),
    ));
    labels.extend(records("PageIntent::label", PAGE_INTENTS, |i, en| {
        i.label(en)
    }));
    labels.extend(records("Priority::label", PRIORITIES, |p, en| p.label(en)));
    labels.extend(records("Effort::label", EFFORTS, |e, en| e.label(en)));
    labels.extend(records("Effort::short_label", EFFORTS, |e, en| {
        e.short_label(en)
    }));
    labels.extend(records(
        "ExecutionPriority::label",
        EXECUTION_PRIORITIES,
        |p, en| p.label(en),
    ));
    labels.extend(records("Role::label", ROLES, |r, en| r.label(en)));
    labels.extend(records("IssueClass::label", ISSUE_CLASSES, |c, en| {
        c.label(en)
    }));
    for class in ISSUE_CLASSES {
        labels.push(json!([
            "IssueClass::fmt",
            format!("{class:?}"),
            "display",
            class.to_string()
        ]));
    }
    labels.extend(records("Subcategory::label", SUBCATEGORIES, |s, en| {
        s.label(en)
    }));
    let path = std::env::var("LABEL_TEXT_EXPORT")
        .unwrap_or_else(|_| format!("{}/reports/label-texts.json", env!("CARGO_MANIFEST_DIR")));
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&labels).unwrap()).unwrap();
    println!("{} label records written to {path}", labels.len());
}
