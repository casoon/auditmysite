//! Page-type fit for structured data.
//!
//! This layer deliberately starts from visible-page intent and URL structure,
//! not from the schema it is judging. Existing schema types are consulted only
//! after the page kind has been classified.

use serde::{Deserialize, Serialize};

use crate::journey::PageIntent;
use crate::seo::schema::StructuredData;
use crate::seo::schema_parity::VisibleSchemaFacts;
use crate::seo::schema_rules::ProductRuleContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaPageKind {
    MerchantProductDetail,
    ProductCollection,
    ServiceDetail,
    SoftwareDetail,
    JobDetail,
    JobListing,
    EventDetail,
    EventListing,
    FaqPage,
    PersonProfile,
    LocationDetail,
    EditorialProductReview,
    EditorialContent,
    MarketingLandingPage,
    CorporatePage,
    HubPage,
    LeadGenerationPage,
    Indeterminate,
}

impl SchemaPageKind {
    pub fn label(self, en: bool) -> &'static str {
        match self {
            Self::MerchantProductDetail => {
                if en {
                    "Purchasable product detail"
                } else {
                    "Kaufbare Produktdetailseite"
                }
            }
            Self::ProductCollection => {
                if en {
                    "Product collection"
                } else {
                    "Produktübersicht"
                }
            }
            Self::ServiceDetail => {
                if en {
                    "Service detail"
                } else {
                    "Leistungsdetailseite"
                }
            }
            Self::SoftwareDetail => "Software / SaaS",
            Self::JobDetail => {
                if en {
                    "Job detail"
                } else {
                    "Stellendetailseite"
                }
            }
            Self::JobListing => {
                if en {
                    "Job listing"
                } else {
                    "Stellenübersicht"
                }
            }
            Self::EventDetail => {
                if en {
                    "Event detail"
                } else {
                    "Veranstaltungsdetailseite"
                }
            }
            Self::EventListing => {
                if en {
                    "Event listing"
                } else {
                    "Veranstaltungsübersicht"
                }
            }
            Self::FaqPage => "FAQ",
            Self::PersonProfile => {
                if en {
                    "Person profile"
                } else {
                    "Personenprofil"
                }
            }
            Self::LocationDetail => {
                if en {
                    "Location detail"
                } else {
                    "Standortdetailseite"
                }
            }
            Self::EditorialProductReview => {
                if en {
                    "Editorial product review"
                } else {
                    "Redaktioneller Produkttest"
                }
            }
            Self::EditorialContent => {
                if en {
                    "Editorial content"
                } else {
                    "Redaktioneller Inhalt"
                }
            }
            Self::MarketingLandingPage => "Marketing / Landing Page",
            Self::CorporatePage => {
                if en {
                    "Corporate page"
                } else {
                    "Unternehmensseite"
                }
            }
            Self::HubPage => "Hub / Portal",
            Self::LeadGenerationPage => {
                if en {
                    "Lead-generation page"
                } else {
                    "Lead-Generierungsseite"
                }
            }
            Self::Indeterminate => {
                if en {
                    "Not determined"
                } else {
                    "Nicht bestimmt"
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFitStatus {
    Matched,
    Plausible,
    Mismatch,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaCoverageStatus {
    Complete,
    Opportunity,
    ManualReview,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaFitAssessment {
    pub page_kind: SchemaPageKind,
    pub confidence: u8,
    pub fit_status: SchemaFitStatus,
    pub coverage_status: SchemaCoverageStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_primary_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detected_primary_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

impl SchemaFitAssessment {
    pub fn product_context(&self) -> ProductRuleContext {
        match self.page_kind {
            SchemaPageKind::MerchantProductDetail => ProductRuleContext::MerchantProductDetail,
            SchemaPageKind::EditorialContent | SchemaPageKind::EditorialProductReview
                if self
                    .detected_primary_types
                    .iter()
                    .any(|schema_type| schema_type == "Product") =>
            {
                ProductRuleContext::EditorialProduct
            }
            _ => ProductRuleContext::Indeterminate,
        }
    }

    pub fn fit_text(&self, en: bool) -> &'static str {
        match self.fit_status {
            SchemaFitStatus::Matched => {
                if en {
                    "Primary schema matches the detected page type"
                } else {
                    "Haupt-Schema passt zum erkannten Seitentyp"
                }
            }
            SchemaFitStatus::Plausible => {
                if en {
                    "Schema is plausible; page type is not specific enough for a strict requirement"
                } else {
                    "Schema ist plausibel; der Seitentyp ist für eine strikte Anforderung nicht spezifisch genug"
                }
            }
            SchemaFitStatus::Mismatch => {
                if en {
                    "Primary schema does not match the strongly indicated page type"
                } else {
                    "Haupt-Schema passt nicht zum deutlich erkannten Seitentyp"
                }
            }
            SchemaFitStatus::Indeterminate => {
                if en {
                    "Page-to-schema fit requires manual review"
                } else {
                    "Seitentyp und Schema-Abdeckung müssen manuell geprüft werden"
                }
            }
        }
    }
}

pub fn assess_schema_fit(
    url: &str,
    page_intent: PageIntent,
    structured_data: &StructuredData,
) -> SchemaFitAssessment {
    assess_schema_fit_with_facts(
        url,
        page_intent,
        structured_data,
        &VisibleSchemaFacts::default(),
    )
}

pub fn assess_schema_fit_with_facts(
    url: &str,
    page_intent: PageIntent,
    structured_data: &StructuredData,
    facts: &VisibleSchemaFacts,
) -> SchemaFitAssessment {
    let path = url_path(url);
    let product_path = contains_any(
        &path,
        &["/product/", "/produkt/", "/p/", "/dp/", "/artikel/"],
    );
    let collection_path = contains_any(
        &path,
        &[
            "/category/",
            "/kategorie/",
            "/collection",
            "/collections/",
            "/shop/",
            "/kategorien/",
        ],
    );
    let visible_heading = facts
        .h1
        .as_ref()
        .or(facts.document_title.as_ref())
        .map(|value| value.to_lowercase())
        .unwrap_or_default();
    let job_path = contains_any(&path, &["/job/", "/jobs/", "/karriere/", "/career/"]);
    let job_listing = route_ends_with(
        &path,
        &[
            "/jobs",
            "/jobs/",
            "/karriere",
            "/karriere/",
            "/career",
            "/career/",
        ],
    );
    let event_path = contains_any(
        &path,
        &[
            "/event/",
            "/events/",
            "/veranstaltung/",
            "/veranstaltungen/",
        ],
    );
    let event_listing = route_ends_with(
        &path,
        &[
            "/events",
            "/events/",
            "/veranstaltungen",
            "/veranstaltungen/",
        ],
    );
    let service_path = contains_any(
        &path,
        &["/service/", "/services/", "/leistung/", "/leistungen/"],
    );
    let software_path = contains_any(&path, &["/software/", "/app/", "/saas/", "/plattform/"])
        || contains_any(
            &visible_heading,
            &["software", " saas", "web-app", "plattform"],
        );
    let faq_page = facts.faq_questions.len() >= 2
        || route_ends_with(&path, &["/faq", "/faq/", "/fragen", "/fragen/"]);
    let person_path = contains_any(&path, &["/person/", "/personen/", "/team/"])
        && !route_ends_with(&path, &["/team", "/team/", "/personen", "/personen/"]);
    let location_path = contains_any(&path, &["/standort/", "/standorte/", "/location/"])
        && !route_ends_with(&path, &["/standorte", "/standorte/"]);
    let editorial_review = page_intent == PageIntent::Editorial
        && (contains_any(&path, &["/test/", "/review/", "/vergleich/"])
            || contains_any(&visible_heading, &["test", "review", "vergleich"]));

    let (page_kind, confidence, expected, mut evidence): (
        SchemaPageKind,
        u8,
        &[&str],
        Vec<String>,
    ) = if faq_page {
        (
            SchemaPageKind::FaqPage,
            90,
            &["FAQPage"],
            vec!["Visible FAQ questions or a dedicated FAQ route were detected".to_string()],
        )
    } else if job_path && !job_listing {
        (
            SchemaPageKind::JobDetail,
            90,
            &["JobPosting"],
            vec!["URL path indicates a single job posting".to_string()],
        )
    } else if job_listing {
        (
            SchemaPageKind::JobListing,
            90,
            &["CollectionPage", "ItemList"],
            vec!["URL path indicates a job listing".to_string()],
        )
    } else if event_path && !event_listing {
        (
            SchemaPageKind::EventDetail,
            90,
            &["Event"],
            vec!["URL path indicates a single event".to_string()],
        )
    } else if event_listing {
        (
            SchemaPageKind::EventListing,
            90,
            &["CollectionPage", "ItemList"],
            vec!["URL path indicates an event listing".to_string()],
        )
    } else if editorial_review {
        (
            SchemaPageKind::EditorialProductReview,
            85,
            &["Article", "BlogPosting", "Product"],
            vec![
                "Visible editorial intent and review-specific route or heading were detected"
                    .to_string(),
            ],
        )
    } else if person_path {
        (
            SchemaPageKind::PersonProfile,
            85,
            &["ProfilePage", "Person"],
            vec!["URL path indicates a single person profile".to_string()],
        )
    } else if location_path {
        (
            SchemaPageKind::LocationDetail,
            85,
            &["LocalBusiness", "Organization"],
            vec!["URL path indicates a single business location".to_string()],
        )
    } else if software_path && matches!(page_intent, PageIntent::Marketing | PageIntent::LeadGen) {
        (
            SchemaPageKind::SoftwareDetail,
            80,
            &["SoftwareApplication", "WebApplication"],
            vec![
                "Visible page intent and route or heading indicate a software offering".to_string(),
            ],
        )
    } else if service_path && matches!(page_intent, PageIntent::Marketing | PageIntent::LeadGen) {
        (
            SchemaPageKind::ServiceDetail,
            80,
            &["Service", "ProfessionalService"],
            vec!["Visible page intent and URL path indicate a service detail page".to_string()],
        )
    } else {
        match page_intent {
            PageIntent::Shop if product_path => (
                SchemaPageKind::MerchantProductDetail,
                90,
                &["Product"],
                vec![
                    "Visible page intent indicates commerce".to_string(),
                    "URL path indicates a product-detail route".to_string(),
                ],
            ),
            PageIntent::Shop if collection_path => (
                SchemaPageKind::ProductCollection,
                90,
                &["CollectionPage", "ItemList"],
                vec![
                    "Visible page intent indicates commerce".to_string(),
                    "URL path indicates a product collection route".to_string(),
                ],
            ),
            PageIntent::Shop => (
                SchemaPageKind::Indeterminate,
                65,
                &[],
                vec![
                    "Visible page intent indicates commerce, but no specific route was established"
                        .to_string(),
                ],
            ),
            PageIntent::Editorial => (
                SchemaPageKind::EditorialContent,
                85,
                &["Article", "BlogPosting", "NewsArticle"],
                vec!["Visible page structure indicates editorial content".to_string()],
            ),
            PageIntent::Marketing => (
                SchemaPageKind::MarketingLandingPage,
                65,
                &["WebPage"],
                vec!["Visible page structure indicates a marketing landing page".to_string()],
            ),
            PageIntent::Corporate => (
                SchemaPageKind::CorporatePage,
                70,
                &["WebPage", "Organization", "LocalBusiness"],
                vec!["Visible page structure indicates corporate information".to_string()],
            ),
            PageIntent::Hub => (
                SchemaPageKind::HubPage,
                70,
                &["WebPage", "CollectionPage", "ItemList"],
                vec!["Visible page structure indicates a hub or portal".to_string()],
            ),
            PageIntent::LeadGen => (
                SchemaPageKind::LeadGenerationPage,
                65,
                &["WebPage", "Service", "ProfessionalService"],
                vec!["Visible page structure indicates lead generation".to_string()],
            ),
            PageIntent::Unknown => (
                SchemaPageKind::Indeterminate,
                0,
                &[],
                vec!["Visible page type could not be determined".to_string()],
            ),
        }
    };

    let expected_primary_types = expected.iter().map(|value| (*value).to_string()).collect();
    let detected_primary_types = structured_data
        .types
        .iter()
        .map(|schema_type| schema_type.as_str().to_string())
        .filter(|schema_type| !is_supporting_type(schema_type))
        .collect::<Vec<_>>();
    let has_expected = detected_primary_types
        .iter()
        .any(|detected| expected.contains(&detected.as_str()));

    let (fit_status, coverage_status) = if expected.is_empty() {
        (
            SchemaFitStatus::Indeterminate,
            SchemaCoverageStatus::ManualReview,
        )
    } else if has_expected {
        if confidence >= 80 {
            (SchemaFitStatus::Matched, SchemaCoverageStatus::Complete)
        } else {
            (
                SchemaFitStatus::Plausible,
                SchemaCoverageStatus::ManualReview,
            )
        }
    } else if confidence >= 80 {
        if matches!(
            page_kind,
            SchemaPageKind::ProductCollection
                | SchemaPageKind::JobListing
                | SchemaPageKind::EventListing
        ) && detected_primary_types
            .iter()
            .any(|schema_type| matches!(schema_type.as_str(), "Product" | "JobPosting" | "Event"))
        {
            let single_item_types = detected_primary_types
                .iter()
                .filter(|schema_type| {
                    matches!(schema_type.as_str(), "Product" | "JobPosting" | "Event")
                })
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            evidence.push(format!(
                "Single-item schema ({single_item_types}) was found on a collection route"
            ));
            (SchemaFitStatus::Mismatch, SchemaCoverageStatus::Opportunity)
        } else {
            (
                SchemaFitStatus::Indeterminate,
                SchemaCoverageStatus::Opportunity,
            )
        }
    } else {
        (
            SchemaFitStatus::Indeterminate,
            SchemaCoverageStatus::ManualReview,
        )
    };

    SchemaFitAssessment {
        page_kind,
        confidence,
        fit_status,
        coverage_status,
        expected_primary_types,
        detected_primary_types,
        evidence,
    }
}

fn url_path(url: &str) -> String {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    after_scheme
        .find('/')
        .map(|index| after_scheme[index..].to_lowercase())
        .unwrap_or_default()
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn route_ends_with(path: &str, endings: &[&str]) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    endings.iter().any(|ending| path.ends_with(ending))
}

fn is_supporting_type(schema_type: &str) -> bool {
    matches!(
        schema_type,
        "WebSite" | "BreadcrumbList" | "Person" | "Offer" | "Review" | "AggregateRating"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seo::schema::analyze_structured_data_payloads;

    fn structured_data(value: serde_json::Value) -> StructuredData {
        analyze_structured_data_payloads(&[value.to_string()], false, false)
    }

    #[test]
    fn merchant_product_route_expects_product_and_enables_merchant_context() {
        let data = structured_data(serde_json::json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "name": "Product",
            "offers": {"@type": "Offer", "price": "10", "priceCurrency": "EUR"}
        }));
        let fit = assess_schema_fit(
            "https://example.com/produkt/widget",
            PageIntent::Shop,
            &data,
        );

        assert_eq!(fit.page_kind, SchemaPageKind::MerchantProductDetail);
        assert_eq!(fit.fit_status, SchemaFitStatus::Matched);
        assert_eq!(
            fit.product_context(),
            ProductRuleContext::MerchantProductDetail
        );
    }

    #[test]
    fn generic_landing_page_never_requires_product() {
        let data = StructuredData::default();
        let fit = assess_schema_fit("https://example.com/angebot", PageIntent::Marketing, &data);

        assert_eq!(fit.page_kind, SchemaPageKind::MarketingLandingPage);
        assert_eq!(fit.coverage_status, SchemaCoverageStatus::ManualReview);
        assert_eq!(fit.expected_primary_types, vec!["WebPage"]);
    }

    #[test]
    fn product_markup_on_collection_route_is_a_fit_mismatch() {
        let data = structured_data(serde_json::json!({
            "@context": "https://schema.org",
            "@type": "Product",
            "name": "Teaser",
            "offers": {"@type": "Offer", "price": "10", "priceCurrency": "EUR"}
        }));
        let fit = assess_schema_fit(
            "https://example.com/kategorie/widgets",
            PageIntent::Shop,
            &data,
        );

        assert_eq!(fit.page_kind, SchemaPageKind::ProductCollection);
        assert_eq!(fit.fit_status, SchemaFitStatus::Mismatch);
    }

    #[test]
    fn single_item_schema_is_rejected_on_job_and_event_listings() {
        for (url, schema_type) in [
            ("https://example.com/jobs/", "JobPosting"),
            ("https://example.com/events/", "Event"),
        ] {
            let data = structured_data(serde_json::json!({
                "@context": "https://schema.org",
                "@type": schema_type,
                "name": "Teaser"
            }));
            let fit = assess_schema_fit(url, PageIntent::Corporate, &data);
            assert_eq!(fit.fit_status, SchemaFitStatus::Mismatch, "{url}");
        }
    }

    #[test]
    fn specific_routes_classify_supported_page_subtypes() {
        let empty = StructuredData::default();
        let cases = [
            (
                "https://example.com/leistungen/audit",
                PageIntent::Marketing,
                SchemaPageKind::ServiceDetail,
            ),
            (
                "https://example.com/software/platform",
                PageIntent::Marketing,
                SchemaPageKind::SoftwareDetail,
            ),
            (
                "https://example.com/jobs/engineer",
                PageIntent::Corporate,
                SchemaPageKind::JobDetail,
            ),
            (
                "https://example.com/jobs/",
                PageIntent::Corporate,
                SchemaPageKind::JobListing,
            ),
            (
                "https://example.com/events/conference",
                PageIntent::Marketing,
                SchemaPageKind::EventDetail,
            ),
            (
                "https://example.com/events/",
                PageIntent::Marketing,
                SchemaPageKind::EventListing,
            ),
            (
                "https://example.com/team/ada",
                PageIntent::Corporate,
                SchemaPageKind::PersonProfile,
            ),
            (
                "https://example.com/standorte/berlin",
                PageIntent::Corporate,
                SchemaPageKind::LocationDetail,
            ),
            (
                "https://example.com/test/widget",
                PageIntent::Editorial,
                SchemaPageKind::EditorialProductReview,
            ),
        ];

        for (url, intent, expected) in cases {
            assert_eq!(
                assess_schema_fit(url, intent, &empty).page_kind,
                expected,
                "{url}"
            );
        }

        let faq_facts = VisibleSchemaFacts {
            faq_questions: vec!["One?".into(), "Two?".into()],
            ..Default::default()
        };
        assert_eq!(
            assess_schema_fit_with_facts(
                "https://example.com/help",
                PageIntent::Corporate,
                &empty,
                &faq_facts,
            )
            .page_kind,
            SchemaPageKind::FaqPage
        );
    }

    #[test]
    fn english_fit_labels_have_no_german_characters() {
        for page_kind in [
            SchemaPageKind::MerchantProductDetail,
            SchemaPageKind::ProductCollection,
            SchemaPageKind::ServiceDetail,
            SchemaPageKind::SoftwareDetail,
            SchemaPageKind::JobDetail,
            SchemaPageKind::JobListing,
            SchemaPageKind::EventDetail,
            SchemaPageKind::EventListing,
            SchemaPageKind::FaqPage,
            SchemaPageKind::PersonProfile,
            SchemaPageKind::LocationDetail,
            SchemaPageKind::EditorialProductReview,
            SchemaPageKind::EditorialContent,
            SchemaPageKind::MarketingLandingPage,
            SchemaPageKind::CorporatePage,
            SchemaPageKind::HubPage,
            SchemaPageKind::LeadGenerationPage,
            SchemaPageKind::Indeterminate,
        ] {
            assert!(!page_kind
                .label(true)
                .contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
        }
    }
}
