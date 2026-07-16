//! Central structured-data rule profiles.
//!
//! The rule layer is deliberately pure: extraction and JSON-LD normalization
//! stay in `schema`, while this module evaluates normalized nodes against
//! feature-specific property requirements. Stored assessments are canonical
//! English and carry their normative source and review date.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const RULESET_VERSION: &str = "2026-07-16";

const PRODUCT_SNIPPET_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/product-snippet";
const MERCHANT_LISTING_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/merchant-listing";
const ARTICLE_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/article";
const BREADCRUMB_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/breadcrumb";
const ORGANIZATION_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/organization";
const LOCAL_BUSINESS_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/local-business";
const FAQ_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/faqpage";
const EVENT_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/event";
const RECIPE_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/recipe";
const VIDEO_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/video";
const JOB_POSTING_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/job-posting";
const SOFTWARE_APP_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/software-app";
const PROFILE_PAGE_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/profile-page";
const CAROUSEL_SOURCE: &str =
    "https://developers.google.com/search/docs/appearance/structured-data/carousel";
const WEBPAGE_SOURCE: &str = "https://schema.org/WebPage";
const WEBSITE_SOURCE: &str = "https://schema.org/WebSite";
const PERSON_SOURCE: &str = "https://schema.org/Person";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFeature {
    ProductSnippet,
    MerchantListing,
    Article,
    Breadcrumb,
    Organization,
    LocalBusiness,
    Faq,
    Event,
    Recipe,
    Video,
    JobPosting,
    SoftwareApplication,
    ProfilePage,
    ItemList,
    WebPage,
    WebSite,
    Person,
}

impl SchemaFeature {
    pub fn label(self, en: bool) -> &'static str {
        match self {
            Self::ProductSnippet => "Product Snippet",
            Self::MerchantListing => "Merchant Listing",
            Self::Article => "Article",
            Self::Breadcrumb => {
                if en {
                    "Breadcrumb"
                } else {
                    "Breadcrumb-Navigation"
                }
            }
            Self::Organization => "Organization",
            Self::LocalBusiness => "LocalBusiness",
            Self::Faq => "FAQ",
            Self::Event => "Event",
            Self::Recipe => "Recipe",
            Self::Video => "Video",
            Self::JobPosting => "Job Posting",
            Self::SoftwareApplication => "Software Application",
            Self::ProfilePage => "Profile Page",
            Self::ItemList => "Item List",
            Self::WebPage => "WebPage",
            Self::WebSite => "WebSite",
            Self::Person => "Person",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::ProductSnippet => "product_snippet",
            Self::MerchantListing => "merchant_listing",
            Self::Article => "article",
            Self::Breadcrumb => "breadcrumb",
            Self::Organization => "organization",
            Self::LocalBusiness => "local_business",
            Self::Faq => "faq",
            Self::Event => "event",
            Self::Recipe => "recipe",
            Self::Video => "video",
            Self::JobPosting => "job_posting",
            Self::SoftwareApplication => "software_application",
            Self::ProfilePage => "profile_page",
            Self::ItemList => "item_list",
            Self::WebPage => "web_page",
            Self::WebSite => "web_site",
            Self::Person => "person",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFeatureAvailability {
    General,
    Limited,
    ContextDependent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaRequirementStatus {
    MeetsRequiredProperties,
    MissingRequiredProperties,
    RecommendationsOnly,
    NotEvaluated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaRuleAssessment {
    pub node_index: usize,
    pub schema_type: String,
    pub feature: SchemaFeature,
    pub availability: SchemaFeatureAvailability,
    pub requirement_status: SchemaRequirementStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_required: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_recommended: Vec<String>,
    /// Conditions or content-quality statements that cannot be established
    /// from the JSON-LD node alone. These are never counted as missing fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manual_review: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_score: Option<u32>,
    pub source_url: String,
    pub reviewed_at: String,
}

impl SchemaRuleAssessment {
    pub fn status_text(&self, en: bool) -> &'static str {
        match self.requirement_status {
            SchemaRequirementStatus::MeetsRequiredProperties => {
                if en {
                    "Required properties met"
                } else {
                    "Pflichtangaben erfüllt"
                }
            }
            SchemaRequirementStatus::MissingRequiredProperties => {
                if en {
                    "Required properties missing"
                } else {
                    "Pflichtangaben fehlen"
                }
            }
            SchemaRequirementStatus::RecommendationsOnly => {
                if en {
                    "No required properties"
                } else {
                    "Keine Pflichtangaben"
                }
            }
            SchemaRequirementStatus::NotEvaluated => {
                if en {
                    "Context not established"
                } else {
                    "Kontext nicht nachgewiesen"
                }
            }
        }
    }
}

pub fn manual_review_text(review: &str, en: bool) -> String {
    if en {
        return review.to_string();
    }
    match review {
        "Confirm that this page is a purchasable product detail page before applying merchant-listing requirements." =>
            "Prüfen, ob dies eine kaufbare Produktdetailseite ist, bevor Merchant-Listing-Anforderungen angewendet werden.".to_string(),
        "Confirm that the cancelled event retains its original startDate and location." =>
            "Prüfen, ob die abgesagte Veranstaltung ihr ursprüngliches startDate und ihren ursprünglichen Ort beibehält.".to_string(),
        "Confirm that the event is a single bookable public event and all marked-up details are visible on the page." =>
            "Prüfen, ob es sich um eine einzelne buchbare öffentliche Veranstaltung handelt und alle ausgezeichneten Angaben sichtbar sind.".to_string(),
        "Google recommends using cookTime and prepTime together; verify the missing duration." =>
            "Google empfiehlt cookTime und prepTime gemeinsam; die fehlende Dauer ist zu prüfen.".to_string(),
        "Confirm that the page describes preparation of one dish and that ingredients and instructions match the visible recipe." =>
            "Prüfen, ob die Seite die Zubereitung eines Gerichts beschreibt und Zutaten sowie Anleitung dem sichtbaren Rezept entsprechen.".to_string(),
        "Confirm that this is one currently open position with a visible application path; expired jobs must be removed or marked with a past validThrough date." =>
            "Prüfen, ob genau eine aktuell offene Stelle mit sichtbarem Bewerbungsweg vorliegt; abgelaufene Stellen müssen entfernt oder mit einem vergangenen validThrough-Datum gekennzeichnet werden.".to_string(),
        "Confirm that the rating or review is visible, genuine, and specifically about this application." =>
            "Prüfen, ob Bewertung oder Rezension sichtbar, authentisch und eindeutig dieser Anwendung zugeordnet ist.".to_string(),
        "Confirm that the page is primarily about one person or organization affiliated with the site." =>
            "Prüfen, ob die Seite hauptsächlich eine mit der Website verbundene Person oder Organisation beschreibt.".to_string(),
        "Confirm that each listed URL is unique and points to a canonical detail page represented by the visible list." =>
            "Prüfen, ob jede aufgeführte URL eindeutig ist und auf eine kanonische Detailseite verweist, die in der sichtbaren Liste enthalten ist.".to_string(),
        _ => review.to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProductRuleContext {
    MerchantProductDetail,
    EditorialProduct,
    #[default]
    Indeterminate,
}

pub fn assess_node(
    node_index: usize,
    schema_type: &str,
    content: &Value,
    product_context: ProductRuleContext,
) -> Vec<SchemaRuleAssessment> {
    match schema_type {
        "Product" => assess_product(node_index, content, product_context),
        "Article" | "BlogPosting" | "NewsArticle" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::Article,
            SchemaFeatureAvailability::General,
            &[],
            &[
                "author",
                "dateModified",
                "datePublished",
                "headline",
                "image",
            ],
            content,
            ARTICLE_SOURCE,
        )],
        "BreadcrumbList" => vec![assess_breadcrumb(node_index, content)],
        "Organization" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::Organization,
            SchemaFeatureAvailability::General,
            &[],
            &["name", "url", "logo", "address", "telephone", "sameAs"],
            content,
            ORGANIZATION_SOURCE,
        )],
        "LocalBusiness" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::LocalBusiness,
            SchemaFeatureAvailability::General,
            &["name", "address"],
            &[
                "telephone",
                "url",
                "openingHoursSpecification",
                "image",
                "priceRange",
                "geo",
            ],
            content,
            LOCAL_BUSINESS_SOURCE,
        )],
        "FAQPage" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::Faq,
            SchemaFeatureAvailability::Limited,
            &["mainEntity"],
            &[],
            content,
            FAQ_SOURCE,
        )],
        "Event" => vec![assess_event(node_index, content)],
        "Recipe" => vec![assess_recipe(node_index, content)],
        "VideoObject" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::Video,
            SchemaFeatureAvailability::General,
            &["name", "thumbnailUrl", "uploadDate"],
            &[
                "contentUrl | embedUrl",
                "description",
                "duration",
                "interactionStatistic",
                "regionsAllowed",
            ],
            content,
            VIDEO_SOURCE,
        )],
        "JobPosting" => vec![assess_job_posting(node_index, content)],
        "SoftwareApplication" | "WebApplication" | "MobileApplication" => {
            vec![assess_software_application(
                node_index,
                schema_type,
                content,
            )]
        }
        "ProfilePage" => vec![assess_profile_page(node_index, content)],
        "CollectionPage" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::ItemList,
            SchemaFeatureAvailability::ContextDependent,
            &[],
            &["name", "url", "mainEntity | hasPart"],
            content,
            CAROUSEL_SOURCE,
        )],
        "ItemList" => vec![assess_item_list(node_index, content)],
        "WebPage" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::WebPage,
            SchemaFeatureAvailability::ContextDependent,
            &[],
            &["name", "url", "description", "inLanguage", "isPartOf"],
            content,
            WEBPAGE_SOURCE,
        )],
        "WebSite" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::WebSite,
            SchemaFeatureAvailability::ContextDependent,
            &[],
            &["name", "url", "inLanguage", "publisher"],
            content,
            WEBSITE_SOURCE,
        )],
        "Person" => vec![assessment(
            node_index,
            schema_type,
            SchemaFeature::Person,
            SchemaFeatureAvailability::ContextDependent,
            &[],
            &["name", "url", "image", "jobTitle", "worksFor", "sameAs"],
            content,
            PERSON_SOURCE,
        )],
        _ => Vec::new(),
    }
}

pub fn inventory_fields(schema_type: &str) -> &'static [&'static str] {
    match schema_type {
        "Product" => &["name", "offers", "review", "aggregateRating", "image"],
        "Article" | "BlogPosting" | "NewsArticle" => &[
            "headline",
            "author",
            "datePublished",
            "dateModified",
            "image",
        ],
        "BreadcrumbList" => &["itemListElement"],
        "Organization" => &["name", "url", "logo", "address", "telephone", "sameAs"],
        "LocalBusiness" => &[
            "name",
            "address",
            "telephone",
            "openingHoursSpecification",
            "image",
            "priceRange",
            "geo",
            "url",
        ],
        "FAQPage" => &["mainEntity"],
        "Event" => &[
            "name",
            "startDate",
            "endDate",
            "location",
            "eventStatus",
            "image",
            "offers",
            "performer",
            "organizer",
        ],
        "Recipe" => &[
            "name",
            "image",
            "author",
            "datePublished",
            "description",
            "recipeIngredient",
            "recipeInstructions",
            "recipeYield",
            "prepTime",
            "cookTime",
            "totalTime",
            "nutrition",
        ],
        "VideoObject" => &[
            "name",
            "thumbnailUrl",
            "uploadDate",
            "contentUrl",
            "embedUrl",
            "description",
            "duration",
            "interactionStatistic",
        ],
        "JobPosting" => &[
            "title",
            "datePosted",
            "description",
            "hiringOrganization",
            "jobLocation",
            "jobLocationType",
            "applicantLocationRequirements",
            "validThrough",
            "baseSalary",
        ],
        "SoftwareApplication" | "WebApplication" | "MobileApplication" => &[
            "name",
            "offers",
            "aggregateRating",
            "review",
            "applicationCategory",
            "operatingSystem",
        ],
        "ProfilePage" => &["mainEntity", "dateCreated", "dateModified"],
        "CollectionPage" => &["name", "url", "mainEntity", "hasPart"],
        "ItemList" => &["name", "itemListElement", "numberOfItems"],
        "Person" => &["name", "url", "image", "jobTitle", "worksFor", "sameAs"],
        "WebSite" => &["name", "url", "potentialAction"],
        "WebPage" => &[
            "name",
            "description",
            "url",
            "image",
            "inLanguage",
            "author",
            "publisher",
        ],
        "ProfessionalService"
        | "Service"
        | "WebDesignCompany"
        | "LegalService"
        | "AccountingService"
        | "FinancialService"
        | "HVACBusiness"
        | "Dentist"
        | "Physician"
        | "Attorney" => &[
            "name",
            "description",
            "url",
            "telephone",
            "address",
            "serviceType",
            "areaServed",
            "priceRange",
        ],
        _ => &[],
    }
}

fn assess_product(
    node_index: usize,
    content: &Value,
    product_context: ProductRuleContext,
) -> Vec<SchemaRuleAssessment> {
    let mut product_snippet = assessment(
        node_index,
        "Product",
        SchemaFeature::ProductSnippet,
        SchemaFeatureAvailability::General,
        &["name"],
        &[],
        content,
        PRODUCT_SNIPPET_SOURCE,
    );
    if !any_present(content, &["review", "aggregateRating", "offers"]) {
        product_snippet
            .missing_required
            .push("review | aggregateRating | offers".to_string());
        product_snippet.requirement_status = SchemaRequirementStatus::MissingRequiredProperties;
    }

    if is_present_at(content, "offers") {
        validate_product_snippet_offers(content, &mut product_snippet);
    }

    let merchant_listing = if product_context == ProductRuleContext::MerchantProductDetail {
        assess_merchant_listing(node_index, content)
    } else {
        SchemaRuleAssessment {
            node_index,
            schema_type: "Product".to_string(),
            feature: SchemaFeature::MerchantListing,
            availability: SchemaFeatureAvailability::ContextDependent,
            requirement_status: SchemaRequirementStatus::NotEvaluated,
            missing_required: Vec::new(),
            missing_recommended: Vec::new(),
            manual_review: vec![
                "Confirm that this page is a purchasable product detail page before applying merchant-listing requirements."
                    .to_string(),
            ],
            quality_score: None,
            source_url: MERCHANT_LISTING_SOURCE.to_string(),
            reviewed_at: RULESET_VERSION.to_string(),
        }
    };

    vec![product_snippet, merchant_listing]
}

fn assess_event(node_index: usize, content: &Value) -> SchemaRuleAssessment {
    let mut result = assessment(
        node_index,
        "Event",
        SchemaFeature::Event,
        SchemaFeatureAvailability::General,
        &["location", "name", "startDate"],
        &[
            "description",
            "endDate",
            "eventStatus",
            "image",
            "offers",
            "organizer",
            "performer",
        ],
        content,
        EVENT_SOURCE,
    );

    if is_value_ending_with(content, "eventStatus", "EventRescheduled")
        && !is_present_at(content, "previousStartDate")
    {
        result
            .missing_required
            .push("previousStartDate (when eventStatus is EventRescheduled)".to_string());
    }
    if is_value_ending_with(content, "eventStatus", "EventCancelled") {
        result.manual_review.push(
            "Confirm that the cancelled event retains its original startDate and location."
                .to_string(),
        );
    }
    result.manual_review.push(
        "Confirm that the event is a single bookable public event and all marked-up details are visible on the page."
            .to_string(),
    );
    finalize_required_status(&mut result);
    result
}

fn assess_recipe(node_index: usize, content: &Value) -> SchemaRuleAssessment {
    let mut result = assessment(
        node_index,
        "Recipe",
        SchemaFeature::Recipe,
        SchemaFeatureAvailability::General,
        &["image", "name"],
        &[
            "author",
            "datePublished",
            "description",
            "recipeIngredient",
            "recipeInstructions",
            "recipeYield",
            "totalTime",
        ],
        content,
        RECIPE_SOURCE,
    );

    if is_present_at(content, "nutrition.calories") && !is_present_at(content, "recipeYield") {
        result
            .missing_required
            .push("recipeYield (when nutrition.calories is present)".to_string());
    }
    if is_present_at(content, "cookTime") != is_present_at(content, "prepTime") {
        result.manual_review.push(
            "Google recommends using cookTime and prepTime together; verify the missing duration."
                .to_string(),
        );
    }
    result.manual_review.push(
        "Confirm that the page describes preparation of one dish and that ingredients and instructions match the visible recipe."
            .to_string(),
    );
    finalize_required_status(&mut result);
    result
}

fn assess_job_posting(node_index: usize, content: &Value) -> SchemaRuleAssessment {
    let mut result = assessment(
        node_index,
        "JobPosting",
        SchemaFeature::JobPosting,
        SchemaFeatureAvailability::General,
        &["datePosted", "description", "hiringOrganization", "title"],
        &["baseSalary", "employmentType", "identifier", "validThrough"],
        content,
        JOB_POSTING_SOURCE,
    );

    let remote = is_value_ending_with(content, "jobLocationType", "TELECOMMUTE");
    if remote {
        if !is_present_at(content, "applicantLocationRequirements") {
            result
                .missing_required
                .push("applicantLocationRequirements (for a fully remote job)".to_string());
        }
    } else if !is_present_at(content, "jobLocation") {
        result
            .missing_required
            .push("jobLocation | remote-job properties".to_string());
    }
    result.manual_review.push(
        "Confirm that this is one currently open position with a visible application path; expired jobs must be removed or marked with a past validThrough date."
            .to_string(),
    );
    finalize_required_status(&mut result);
    result
}

fn assess_software_application(
    node_index: usize,
    schema_type: &str,
    content: &Value,
) -> SchemaRuleAssessment {
    let mut result = assessment(
        node_index,
        schema_type,
        SchemaFeature::SoftwareApplication,
        SchemaFeatureAvailability::General,
        &["name", "offers.price", "aggregateRating | review"],
        &["applicationCategory", "operatingSystem"],
        content,
        SOFTWARE_APP_SOURCE,
    );

    if number_at(content, "offers.price").is_some_and(|price| price > 0.0)
        && !is_present_at(content, "offers.priceCurrency")
    {
        result
            .missing_recommended
            .push("offers.priceCurrency (when price is greater than 0)".to_string());
    }
    result.manual_review.push(
        "Confirm that the rating or review is visible, genuine, and specifically about this application."
            .to_string(),
    );
    result
}

fn assess_profile_page(node_index: usize, content: &Value) -> SchemaRuleAssessment {
    let mut result = assessment(
        node_index,
        "ProfilePage",
        SchemaFeature::ProfilePage,
        SchemaFeatureAvailability::General,
        &["mainEntity", "mainEntity.name"],
        &["dateCreated", "dateModified"],
        content,
        PROFILE_PAGE_SOURCE,
    );
    if is_present_at(content, "mainEntity")
        && !any_value_ending_with(content, "mainEntity.@type", &["Person", "Organization"])
    {
        result
            .missing_required
            .push("mainEntity.@type = Person | Organization".to_string());
    }
    result.manual_review.push(
        "Confirm that the page is primarily about one person or organization affiliated with the site."
            .to_string(),
    );
    finalize_required_status(&mut result);
    result
}

fn assess_item_list(node_index: usize, content: &Value) -> SchemaRuleAssessment {
    let mut result = assessment(
        node_index,
        "ItemList",
        SchemaFeature::ItemList,
        SchemaFeatureAvailability::ContextDependent,
        &["itemListElement"],
        &["name", "numberOfItems"],
        content,
        CAROUSEL_SOURCE,
    );
    for (index, item) in values_at(content, "itemListElement").iter().enumerate() {
        for property in ["position", "url"] {
            if !is_present_at(item, property) {
                result
                    .missing_required
                    .push(format!("itemListElement[{index}].{property}"));
            }
        }
    }
    result.manual_review.push(
        "Confirm that each listed URL is unique and points to a canonical detail page represented by the visible list."
            .to_string(),
    );
    finalize_required_status(&mut result);
    result
}

fn assess_merchant_listing(node_index: usize, content: &Value) -> SchemaRuleAssessment {
    let mut result = assessment(
        node_index,
        "Product",
        SchemaFeature::MerchantListing,
        SchemaFeatureAvailability::ContextDependent,
        &["name", "image", "offers"],
        &["description", "sku", "brand.name", "offers.availability"],
        content,
        MERCHANT_LISTING_SOURCE,
    );

    let offers = values_at(content, "offers");
    if !offers.is_empty() {
        for (index, offer) in offers.iter().enumerate() {
            let prefix = if offers.len() == 1 {
                "offers".to_string()
            } else {
                format!("offers[{index}]")
            };
            let offer_types = string_values(offer.get("@type"));
            if !offer_types.contains(&"Offer") {
                result
                    .missing_required
                    .push(format!("{prefix}.@type = Offer"));
            }
            if !any_present(offer, &["price", "priceSpecification.price"]) {
                result.missing_required.push(format!(
                    "{prefix}.price | {prefix}.priceSpecification.price"
                ));
            }
            if !any_present(
                offer,
                &["priceCurrency", "priceSpecification.priceCurrency"],
            ) {
                result.missing_required.push(format!(
                    "{prefix}.priceCurrency | {prefix}.priceSpecification.priceCurrency"
                ));
            }
        }
    }
    finalize_required_status(&mut result);
    result
}

fn validate_product_snippet_offers(content: &Value, result: &mut SchemaRuleAssessment) {
    for (index, offer) in values_at(content, "offers").iter().enumerate() {
        let prefix = if values_at(content, "offers").len() == 1 {
            "offers".to_string()
        } else {
            format!("offers[{index}]")
        };
        let offer_types = string_values(offer.get("@type"));
        if offer_types.contains(&"AggregateOffer") {
            for property in ["lowPrice", "priceCurrency"] {
                if !is_present_at(offer, property) {
                    result.missing_required.push(format!("{prefix}.{property}"));
                }
            }
        } else if !any_present(offer, &["price", "priceSpecification.price"]) {
            result.missing_required.push(format!(
                "{prefix}.price | {prefix}.priceSpecification.price"
            ));
        }
    }
    finalize_required_status(result);
}

fn assess_breadcrumb(node_index: usize, content: &Value) -> SchemaRuleAssessment {
    let mut result = assessment(
        node_index,
        "BreadcrumbList",
        SchemaFeature::Breadcrumb,
        SchemaFeatureAvailability::General,
        &["itemListElement"],
        &[],
        content,
        BREADCRUMB_SOURCE,
    );

    let items = content
        .get("itemListElement")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !items.is_empty() && items.len() < 2 {
        result
            .missing_required
            .push("itemListElement (minimum 2 entries)".to_string());
    }
    for (index, item) in items.iter().enumerate() {
        for property in ["name", "position"] {
            if !is_present_at(item, property) {
                result
                    .missing_required
                    .push(format!("itemListElement[{index}].{property}"));
            }
        }
        if index + 1 < items.len() && !is_present_at(item, "item") {
            result
                .missing_required
                .push(format!("itemListElement[{index}].item"));
        }
    }
    finalize_required_status(&mut result);
    result
}

#[allow(clippy::too_many_arguments)]
fn assessment(
    node_index: usize,
    schema_type: &str,
    feature: SchemaFeature,
    availability: SchemaFeatureAvailability,
    required: &[&str],
    recommended: &[&str],
    content: &Value,
    source_url: &str,
) -> SchemaRuleAssessment {
    let missing_required = required
        .iter()
        .filter(|property| !is_requirement_present(content, property))
        .map(|property| (*property).to_string())
        .collect::<Vec<_>>();
    let missing_recommended = recommended
        .iter()
        .filter(|property| !is_requirement_present(content, property))
        .map(|property| (*property).to_string())
        .collect::<Vec<_>>();
    let requirement_status = if required.is_empty() {
        SchemaRequirementStatus::RecommendationsOnly
    } else if missing_required.is_empty() {
        SchemaRequirementStatus::MeetsRequiredProperties
    } else {
        SchemaRequirementStatus::MissingRequiredProperties
    };
    let quality_score = if recommended.is_empty() {
        None
    } else {
        Some(((recommended.len() - missing_recommended.len()) * 100 / recommended.len()) as u32)
    };

    SchemaRuleAssessment {
        node_index,
        schema_type: schema_type.to_string(),
        feature,
        availability,
        requirement_status,
        missing_required,
        missing_recommended,
        manual_review: Vec::new(),
        quality_score,
        source_url: source_url.to_string(),
        reviewed_at: RULESET_VERSION.to_string(),
    }
}

fn finalize_required_status(result: &mut SchemaRuleAssessment) {
    result.requirement_status = if result.missing_required.is_empty() {
        SchemaRequirementStatus::MeetsRequiredProperties
    } else {
        SchemaRequirementStatus::MissingRequiredProperties
    };
}

fn any_present(value: &Value, paths: &[&str]) -> bool {
    paths.iter().any(|path| is_present_at(value, path))
}

fn is_requirement_present(value: &Value, requirement: &str) -> bool {
    requirement
        .split('|')
        .map(str::trim)
        .any(|path| is_present_at(value, path))
}

fn is_value_ending_with(value: &Value, path: &str, suffix: &str) -> bool {
    values_at_path(value, path)
        .iter()
        .filter_map(|value| value.as_str())
        .any(|value| value.trim_end_matches('/').ends_with(suffix))
}

fn any_value_ending_with(value: &Value, path: &str, suffixes: &[&str]) -> bool {
    suffixes
        .iter()
        .any(|suffix| is_value_ending_with(value, path, suffix))
}

fn values_at_path<'a>(value: &'a Value, path: &str) -> Vec<&'a Value> {
    fn collect<'a>(value: &'a Value, parts: &[&str], output: &mut Vec<&'a Value>) {
        if parts.is_empty() {
            match value {
                Value::Array(values) => output.extend(values),
                _ => output.push(value),
            }
            return;
        }
        match value {
            Value::Array(values) => {
                for value in values {
                    collect(value, parts, output);
                }
            }
            Value::Object(values) => {
                if let Some(value) = values.get(parts[0]) {
                    collect(value, &parts[1..], output);
                }
            }
            _ => {}
        }
    }

    let mut output = Vec::new();
    collect(value, &path.split('.').collect::<Vec<_>>(), &mut output);
    output
}

fn number_at(value: &Value, path: &str) -> Option<f64> {
    values_at_path(value, path).into_iter().find_map(|value| {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
    })
}

fn is_present_at(value: &Value, path: &str) -> bool {
    is_present_parts(value, &path.split('.').collect::<Vec<_>>())
}

fn is_present_parts(value: &Value, parts: &[&str]) -> bool {
    if parts.is_empty() {
        return is_non_empty(value);
    }
    match value {
        Value::Array(values) => values.iter().any(|value| is_present_parts(value, parts)),
        Value::Object(values) => values
            .get(parts[0])
            .is_some_and(|value| is_present_parts(value, &parts[1..])),
        _ => false,
    }
}

fn is_non_empty(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        _ => true,
    }
}

fn values_at<'a>(value: &'a Value, key: &str) -> Vec<&'a Value> {
    match value.get(key) {
        Some(Value::Array(values)) => values.iter().collect(),
        Some(value) if is_non_empty(value) => vec![value],
        _ => Vec::new(),
    }
}

fn string_values(value: Option<&Value>) -> Vec<&str> {
    match value {
        Some(Value::String(value)) => vec![value.as_str()],
        Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_has_recommendations_but_no_required_properties() {
        let result = assess_node(
            0,
            "Article",
            &serde_json::json!({"@type": "Article", "headline": "Example"}),
            ProductRuleContext::Indeterminate,
        );

        assert_eq!(
            result[0].requirement_status,
            SchemaRequirementStatus::RecommendationsOnly
        );
        assert!(result[0].missing_required.is_empty());
        assert!(result[0]
            .missing_recommended
            .contains(&"author".to_string()));
    }

    #[test]
    fn product_snippet_accepts_review_instead_of_offer() {
        let result = assess_node(
            0,
            "Product",
            &serde_json::json!({
                "@type": "Product",
                "name": "Reviewed product",
                "review": {"@type": "Review"}
            }),
            ProductRuleContext::EditorialProduct,
        );

        assert_eq!(
            result[0].requirement_status,
            SchemaRequirementStatus::MeetsRequiredProperties
        );
        assert_eq!(
            result[1].requirement_status,
            SchemaRequirementStatus::NotEvaluated
        );
    }

    #[test]
    fn merchant_listing_validates_nested_offer_price_and_currency() {
        let result = assess_node(
            0,
            "Product",
            &serde_json::json!({
                "@type": "Product",
                "name": "Buyable product",
                "image": "https://example.com/product.jpg",
                "offers": {"@type": "Offer", "price": "19.99"}
            }),
            ProductRuleContext::MerchantProductDetail,
        );

        assert_eq!(
            result[1].requirement_status,
            SchemaRequirementStatus::MissingRequiredProperties
        );
        assert!(result[1].missing_required.contains(
            &"offers.priceCurrency | offers.priceSpecification.priceCurrency".to_string()
        ));
    }

    #[test]
    fn breadcrumb_requires_two_complete_items_but_last_item_may_omit_url() {
        let valid = serde_json::json!({
            "@type": "BreadcrumbList",
            "itemListElement": [
                {"@type": "ListItem", "position": 1, "name": "Home", "item": "https://example.com"},
                {"@type": "ListItem", "position": 2, "name": "Current"}
            ]
        });
        let result = assess_node(
            0,
            "BreadcrumbList",
            &valid,
            ProductRuleContext::Indeterminate,
        );
        assert_eq!(
            result[0].requirement_status,
            SchemaRequirementStatus::MeetsRequiredProperties
        );

        let incomplete = serde_json::json!({
            "@type": "BreadcrumbList",
            "itemListElement": [{"@type": "ListItem", "position": 1, "name": "Only"}]
        });
        let result = assess_node(
            0,
            "BreadcrumbList",
            &incomplete,
            ProductRuleContext::Indeterminate,
        );
        assert!(result[0]
            .missing_required
            .contains(&"itemListElement (minimum 2 entries)".to_string()));
    }

    #[test]
    fn english_rule_labels_have_no_german_characters() {
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
            assert!(!feature
                .label(true)
                .contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
        }
    }

    #[test]
    fn new_rich_result_profiles_distinguish_minimal_complete_and_conditional_data() {
        let cases = [
            (
                "Event",
                serde_json::json!({"name":"Conference","startDate":"2026-09-01","location":{"@type":"Place","name":"Hall","address":"Main Street 1"}}),
                serde_json::json!({"name":"Conference","startDate":"2026-09-01","endDate":"2026-09-02","location":{"@type":"Place","name":"Hall","address":"Main Street 1"},"description":"Annual conference","eventStatus":"https://schema.org/EventScheduled","image":"https://example.com/event.jpg","offers":{"@type":"Offer","price":0},"organizer":{"@type":"Organization","name":"Example"},"performer":{"@type":"Person","name":"Speaker"}}),
            ),
            (
                "Recipe",
                serde_json::json!({"name":"Soup","image":"https://example.com/soup.jpg"}),
                serde_json::json!({"name":"Soup","image":"https://example.com/soup.jpg","author":{"name":"Cook"},"datePublished":"2026-01-01","description":"A soup","recipeIngredient":["water"],"recipeInstructions":["Cook"],"recipeYield":"2 servings","totalTime":"PT30M"}),
            ),
            (
                "VideoObject",
                serde_json::json!({"name":"Demo","thumbnailUrl":"https://example.com/thumb.jpg","uploadDate":"2026-01-01"}),
                serde_json::json!({"name":"Demo","thumbnailUrl":"https://example.com/thumb.jpg","uploadDate":"2026-01-01","contentUrl":"https://example.com/video.mp4","description":"Demo video","duration":"PT1M","interactionStatistic":{"userInteractionCount":1},"regionsAllowed":"DE"}),
            ),
            (
                "JobPosting",
                serde_json::json!({"title":"Engineer","datePosted":"2026-01-01","description":"<p>Role</p>","hiringOrganization":{"name":"Example"},"jobLocation":{"address":{"addressCountry":"DE"}}}),
                serde_json::json!({"title":"Engineer","datePosted":"2026-01-01","description":"<p>Role</p>","hiringOrganization":{"name":"Example"},"jobLocation":{"address":{"addressCountry":"DE"}},"baseSalary":{"value":50000},"employmentType":"FULL_TIME","identifier":"job-1","validThrough":"2026-12-31"}),
            ),
            (
                "SoftwareApplication",
                serde_json::json!({"name":"App","offers":{"price":0},"review":{"reviewRating":{"ratingValue":5}}}),
                serde_json::json!({"name":"App","offers":{"price":0},"review":{"reviewRating":{"ratingValue":5}},"applicationCategory":"BusinessApplication","operatingSystem":"Web"}),
            ),
            (
                "ProfilePage",
                serde_json::json!({"mainEntity":{"@type":"Person","name":"Ada"}}),
                serde_json::json!({"mainEntity":{"@type":"Person","name":"Ada"},"dateCreated":"2026-01-01","dateModified":"2026-02-01"}),
            ),
            (
                "ItemList",
                serde_json::json!({"itemListElement":[{"position":1,"url":"https://example.com/1"}]}),
                serde_json::json!({"name":"Items","numberOfItems":1,"itemListElement":[{"position":1,"url":"https://example.com/1"}]}),
            ),
        ];

        for (schema_type, minimal, complete) in cases {
            let minimal = assess_node(0, schema_type, &minimal, ProductRuleContext::Indeterminate);
            assert_eq!(
                minimal[0].requirement_status,
                SchemaRequirementStatus::MeetsRequiredProperties,
                "minimal {schema_type}"
            );
            let complete =
                assess_node(0, schema_type, &complete, ProductRuleContext::Indeterminate);
            assert!(
                complete[0].missing_required.is_empty(),
                "complete {schema_type}"
            );
            assert!(
                complete[0].missing_recommended.is_empty(),
                "complete {schema_type}: {:?}",
                complete[0].missing_recommended
            );
        }

        let rescheduled = assess_node(
            0,
            "Event",
            &serde_json::json!({"name":"Conference","startDate":"2026-09-01","location":{"name":"Hall"},"eventStatus":"https://schema.org/EventRescheduled"}),
            ProductRuleContext::Indeterminate,
        );
        assert!(rescheduled[0]
            .missing_required
            .iter()
            .any(|property| property.starts_with("previousStartDate")));

        let remote = assess_node(
            0,
            "JobPosting",
            &serde_json::json!({"title":"Engineer","datePosted":"2026-01-01","description":"Role","hiringOrganization":{"name":"Example"},"jobLocationType":"TELECOMMUTE"}),
            ProductRuleContext::Indeterminate,
        );
        assert!(remote[0]
            .missing_required
            .iter()
            .any(|property| property.starts_with("applicantLocationRequirements")));
    }

    #[test]
    fn unknown_schema_types_are_inventory_only() {
        assert!(assess_node(
            0,
            "FutureSchemaType",
            &serde_json::json!({"name":"Example"}),
            ProductRuleContext::Indeterminate,
        )
        .is_empty());
        assert!(inventory_fields("FutureSchemaType").is_empty());
    }

    #[test]
    fn general_schema_profiles_distinguish_sparse_and_complete_data() {
        let cases = [
            (
                "CollectionPage",
                serde_json::json!({}),
                serde_json::json!({
                    "name": "Jobs",
                    "url": "https://example.com/jobs",
                    "mainEntity": {"@type": "ItemList"}
                }),
            ),
            (
                "WebPage",
                serde_json::json!({}),
                serde_json::json!({
                    "name": "About",
                    "url": "https://example.com/about",
                    "description": "About Example",
                    "inLanguage": "en",
                    "isPartOf": {"@type": "WebSite"}
                }),
            ),
            (
                "WebSite",
                serde_json::json!({}),
                serde_json::json!({
                    "name": "Example",
                    "url": "https://example.com",
                    "inLanguage": "en",
                    "publisher": {"@type": "Organization", "name": "Example"}
                }),
            ),
            (
                "Person",
                serde_json::json!({}),
                serde_json::json!({
                    "name": "Ada Example",
                    "url": "https://example.com/team/ada",
                    "image": "https://example.com/ada.jpg",
                    "jobTitle": "Engineer",
                    "worksFor": {"@type": "Organization", "name": "Example"},
                    "sameAs": ["https://social.example/ada"]
                }),
            ),
        ];

        for (schema_type, sparse, complete) in cases {
            let sparse = assess_node(0, schema_type, &sparse, ProductRuleContext::Indeterminate);
            assert_eq!(
                sparse[0].requirement_status,
                SchemaRequirementStatus::RecommendationsOnly,
                "sparse {schema_type}"
            );
            assert!(!sparse[0].missing_recommended.is_empty(), "{schema_type}");

            let complete =
                assess_node(0, schema_type, &complete, ProductRuleContext::Indeterminate);
            assert!(
                complete[0].missing_recommended.is_empty(),
                "complete {schema_type}: {:?}",
                complete[0].missing_recommended
            );
        }
    }

    #[test]
    fn paid_software_offer_requires_currency_as_conditional_guidance() {
        let assessment = assess_node(
            0,
            "WebApplication",
            &serde_json::json!({
                "name": "Paid app",
                "offers": {"price": 19},
                "review": {"reviewRating": {"ratingValue": 5}}
            }),
            ProductRuleContext::Indeterminate,
        );

        assert!(assessment[0]
            .missing_recommended
            .iter()
            .any(|property| property.contains("priceCurrency")));
    }
}
