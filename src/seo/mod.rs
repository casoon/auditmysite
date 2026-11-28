//! SEO analysis module
//!
//! Provides meta tags validation, heading structure, social tags, and technical SEO checks.

pub(crate) mod canonical;
pub(crate) mod headings;
pub mod image_efficiency;
pub mod interpretation;
pub mod issue;
pub(crate) mod meta;
pub mod module;
pub mod page_health;
pub mod profile;
pub mod robots;
pub mod schema;
pub mod schema_fit;
pub mod schema_parity;
pub mod schema_rules;
pub mod serp;
pub(crate) mod social;
pub mod technical;
pub mod topics;

pub use module::SeoModule;

pub use headings::{
    analyze_heading_structure, long_heading_text, HeadingInfo, HeadingIssue, HeadingStructure,
    LongHeading,
};
pub use image_efficiency::{analyze_image_efficiency, ImageEfficiencyAnalysis, OversizedImage};
pub use issue::SeoIssue;
pub use meta::{
    extract_meta_tags, meta_issue_text, MetaIssueKind, MetaIssueValues, MetaTags, MetaValidation,
};
pub use page_health::{
    analyze_page_health, collect_issues, HtmlValidationIssue, HtmlValidationKind,
    PageHealthAnalysis, UrlCanonicalizationCheck, WwwConsolidation,
};
pub use profile::{build_content_profile, SeoContentProfile};
pub use robots::{audit_robots_txt, infer_robots_policy, BotClass, RobotsAudit, RobotsGroup};
pub use schema::{detect_structured_data, SchemaType, StructuredData};
pub use schema_fit::{
    assess_schema_fit, assess_schema_fit_with_facts, SchemaCoverageStatus, SchemaFitAssessment,
    SchemaFitStatus, SchemaPageKind,
};
pub use schema_parity::{ContentParityAssessment, ContentParityStatus, VisibleSchemaFacts};
pub use schema_rules::{
    SchemaFeature, SchemaFeatureAvailability, SchemaRequirementStatus, SchemaRuleAssessment,
};
pub use serp::{build_serp_analysis, SerpAnalysis, SerpSignal, SerpSignalStatus};
pub use social::{extract_social_tags, OpenGraph, SocialTags, TwitterCard};
pub use technical::{
    analyze_technical_seo, collect_technical_issues, is_optional_hint, TechnicalSeo,
};
pub use topics::{
    average_page_semantic_score, derive_domain_topics, derive_topic_overlap_pairs,
    extract_page_topics,
};

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Complete SEO analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeoAnalysis {
    /// Meta tags analysis
    pub meta: MetaTags,
    /// Meta validation issues
    pub meta_issues: Vec<MetaValidation>,
    /// Heading structure
    pub headings: HeadingStructure,
    /// Social media tags
    pub social: SocialTags,
    /// Technical SEO
    pub technical: TechnicalSeo,
    /// Structured data
    pub structured_data: StructuredData,
    /// Overall SEO score (0-100)
    pub score: u32,
    /// Content profile analysis
    pub content_profile: Option<SeoContentProfile>,
    /// robots.txt audit (informational, no score impact)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub robots: Option<RobotsAudit>,
    /// Page health analysis (HTTP probes, DOM checks, URL analysis)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_health: Option<PageHealthAnalysis>,
    /// SERP pass — aggregated search-result-page readiness signals
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serp: Option<SerpAnalysis>,
    /// Image efficiency analysis (format + oversizing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_efficiency: Option<ImageEfficiencyAnalysis>,
    /// Whether the page is meant to be indexed and is (#780).
    #[serde(default)]
    pub indexability: IndexabilityCheck,
    /// Page type estimated from the accessibility tree at collection; the
    /// page-to-schema fit is judged against it. It is the SEO module's own
    /// input (the experimental Journey indicator is not needed for it, #916)
    /// and not part of the JSON.
    #[serde(skip)]
    pub page_intent: Option<crate::page_flow::PageIntent>,
}

/// Upper bound of the SEO score of a page that is meant to be indexed but
/// carries `noindex`: the lowest band. Nothing else on the page reaches
/// search users while the directive stands (#780).
pub const INDEXABILITY_BLOCKER_CAP: u32 = 39;

/// The page's indexability against what it is meant to have (#780). A
/// `noindex` without a declared or implied expectation stays undecided: an
/// intended noindex page is not an SEO defect.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexabilityCheck {
    pub expectation: crate::audit::finding_contract::IndexabilityExpectation,
    /// The robots meta tag carries `noindex`.
    pub noindex: bool,
    /// Meant to be indexed, but `noindex`: a technical blocker that caps the
    /// score at [`INDEXABILITY_BLOCKER_CAP`].
    pub blocked: bool,
}

impl IndexabilityCheck {
    pub fn assess(
        technical: &TechnicalSeo,
        expectation: crate::audit::finding_contract::IndexabilityExpectation,
    ) -> Self {
        let noindex = technical
            .robots_meta
            .as_deref()
            .is_some_and(|robots| robots.to_lowercase().contains("noindex"));
        Self {
            expectation,
            noindex,
            blocked: noindex && expectation.expected == Some(true),
        }
    }
}

/// Run complete SEO analysis.
///
/// All derived text is baked in canonical English so the stored struct (and the
/// JSON report) is language-independent. The PDF re-derives localized text from
/// the same pure builders at presentation time (#406).
pub async fn analyze_seo(
    page: &Page,
    url: &str,
    log: &crate::audit::collection::CollectionLog,
) -> Result<SeoAnalysis> {
    // Extract all SEO data in parallel where possible
    let meta = extract_meta_tags(page, log).await?;
    let mut meta_issues = meta.validate();
    let mut headings = analyze_heading_structure(page, log).await?;
    headings::check_h1_title_alignment(
        &mut headings,
        meta.title.as_deref(),
        meta.description.as_deref(),
    );
    let social = extract_social_tags(page, log).await?;
    meta_issues.extend(social.validate());
    let technical = analyze_technical_seo(page, url, log).await?;
    let structured_data = detect_structured_data(page, log).await?;

    // robots.txt — HTTP fetch, independent of browser
    let is_noindex = technical
        .robots_meta
        .as_deref()
        .map(|r| r.to_lowercase().contains("noindex"))
        .unwrap_or(false);
    let robots = audit_robots_txt(
        url,
        technical.canonical_url.as_deref(),
        is_noindex,
        "en",
        log,
    )
    .await;

    // Page health analysis — HTTP probes + DOM inspection
    let page_health = analyze_page_health(page, url, log).await.map(|mut ph| {
        // Copy in the declared <meta charset> so charset-consistency
        // checking (#543) stays a pure function of PageHealthAnalysis
        // without page_health.rs depending on the meta module.
        ph.declared_charset = meta.charset.clone();
        ph.issues = collect_issues(&ph, true);
        ph
    });

    // Image efficiency analysis
    let image_efficiency = analyze_image_efficiency(page, log).await;

    // Calculate score
    let score = calculate_seo_score(
        &meta,
        &meta_issues,
        &headings,
        &social,
        &technical,
        image_efficiency.as_ref(),
        page_health.as_ref(),
    );

    let mut analysis = SeoAnalysis {
        meta,
        meta_issues,
        headings,
        social,
        technical,
        structured_data,
        score,
        content_profile: None,
        robots,
        page_health,
        serp: None,
        image_efficiency,
        indexability: IndexabilityCheck::default(),
        page_intent: None,
    };

    // Build content profile from collected data (baked English; PDF re-derives)
    analysis.content_profile = Some(build_content_profile(&analysis, "en"));

    // SERP pass — pure aggregation, no additional CDP calls needed
    analysis.serp = Some(build_serp_analysis(&analysis, url, "en"));

    Ok(analysis)
}

/// Recomputes the SEO score after the heading issues were replaced by the
/// shared findings (#724) and the page's indexability expectation is known
/// (#780).
pub(crate) fn rescore(
    seo: &mut SeoAnalysis,
    expectation: crate::audit::finding_contract::IndexabilityExpectation,
) {
    seo.indexability = IndexabilityCheck::assess(&seo.technical, expectation);
    let score = calculate_seo_score(
        &seo.meta,
        &seo.meta_issues,
        &seo.headings,
        &seo.social,
        &seo.technical,
        seo.image_efficiency.as_ref(),
        seo.page_health.as_ref(),
    );
    seo.score = if seo.indexability.blocked {
        score.min(INDEXABILITY_BLOCKER_CAP)
    } else {
        score
    };
}

/// Upper bound of the total page-health deduction, so many findings do not
/// collapse the score (same bound as the meta block).
const PAGE_HEALTH_PENALTY_CAP: u32 = 20;

/// Points one page-health finding costs in the SEO score (#1227): high 8,
/// medium 4, critical 12; low findings cost nothing. Zero as well when another
/// score already owns the cause: the DOM size is charged by Performance, and
/// oversized or legacy-format images by the image-efficiency deduction
/// (`image_efficiency_charged`), so no cause is charged twice.
pub(crate) fn page_health_charge(issue: &SeoIssue, image_efficiency_charged: bool) -> u32 {
    let owned_elsewhere = match issue.issue_type.as_str() {
        "excessive_dom" | "large_dom" => true,
        "oversized_images" | "images_without_modern_format" => image_efficiency_charged,
        _ => false,
    };
    if owned_elsewhere {
        return 0;
    }
    match issue.severity {
        crate::taxonomy::Severity::Critical => 12,
        crate::taxonomy::Severity::High => 8,
        crate::taxonomy::Severity::Medium => 4,
        crate::taxonomy::Severity::Low => 0,
    }
}

/// Total page-health deduction: each issue type once, capped at
/// [`PAGE_HEALTH_PENALTY_CAP`].
fn page_health_penalty(issues: &[SeoIssue], image_efficiency_charged: bool) -> u32 {
    let mut seen = std::collections::BTreeSet::new();
    issues
        .iter()
        .filter(|issue| seen.insert(issue.issue_type.as_str()))
        .map(|issue| page_health_charge(issue, image_efficiency_charged))
        .sum::<u32>()
        .min(PAGE_HEALTH_PENALTY_CAP)
}

fn calculate_seo_score(
    meta: &MetaTags,
    meta_issues: &[MetaValidation],
    headings: &HeadingStructure,
    social: &SocialTags,
    technical: &TechnicalSeo,
    image_efficiency: Option<&ImageEfficiencyAnalysis>,
    page_health: Option<&PageHealthAnalysis>,
) -> u32 {
    let mut score = 100u32;

    // Meta tags — cap total penalty at 20 to avoid collapse from many minor issues.
    // A missing lang or canonical is charged once, in the technical section
    // below, and is listed once, by the technical checks. A missing viewport is
    // charged by Mobile and Accessibility, not here as well.
    let meta_penalty: u32 = meta_issues
        .iter()
        .filter(|issue| issue.field != "viewport")
        // An overlong description is an advisory: Google truncates snippets
        // dynamically and sets no fixed maximum.
        .filter(|issue| issue.kind != Some(crate::seo::meta::MetaIssueKind::DescriptionTooLong))
        .map(|issue| match issue.severity {
            crate::taxonomy::Severity::Critical => 15,
            crate::taxonomy::Severity::High => 10,
            crate::taxonomy::Severity::Medium => 5,
            crate::taxonomy::Severity::Low => 2,
        })
        .sum::<u32>()
        .min(20);
    score = score.saturating_sub(meta_penalty);

    // Heading structure
    if headings.h1_count == 0 {
        score = score.saturating_sub(15);
    } else if headings.h1_count > 1 {
        score = score.saturating_sub(5);
    }
    if !headings.issues.is_empty() {
        // Cap at 15 — reflects severity, not the count of repeated instances
        let heading_penalty = (headings.issues.len() as u32 * 3).min(15);
        score = score.saturating_sub(heading_penalty);
    }

    // Social tags
    if social.open_graph.is_none() {
        score = score.saturating_sub(5);
    }
    if social.twitter_card.is_none() {
        score = score.saturating_sub(5);
    }

    // Technical SEO
    if !technical.https {
        score = score.saturating_sub(10);
    }
    if !technical.has_canonical {
        score = score.saturating_sub(5);
    }
    if !technical.has_lang {
        score = score.saturating_sub(3);
    }

    // Image efficiency penalty
    if let Some(ie) = image_efficiency {
        let ie_penalty = (100 - ie.score).min(15);
        score = score.saturating_sub(ie_penalty);
    }

    // Page-health findings (#1227): canonical variants, redirects, soft 404,
    // hreflang, charset, render-blocking scripts and the like. Charged by
    // severity, once per issue type, capped like the meta block.
    if let Some(health) = page_health {
        score = score.saturating_sub(page_health_penalty(
            &health.issues,
            image_efficiency.is_some(),
        ));
    }

    // Baseline-signal floor (#486): a page with the core meta/technical SEO
    // signals in place must not collapse from a couple of structural heading
    // issues. Each present baseline signal raises the floor by 10 points, so
    // well-configured pages stay in a credible range even with a missing H1 —
    // a high-profile site with complete meta tags, canonical, lang and Open
    // Graph no longer scores 44 from two heading findings.
    let baseline_signals = [
        meta.title.as_ref().is_some_and(|t| !t.trim().is_empty()),
        meta.description
            .as_ref()
            .is_some_and(|d| !d.trim().is_empty()),
        technical.https,
        technical.has_canonical,
        technical.has_lang,
        social.open_graph.is_some(),
    ];
    let floor = baseline_signals.iter().filter(|present| **present).count() as u32 * 10;

    score.max(floor).min(100)
}

#[cfg(test)]
pub(crate) mod issue_texts_export;

#[cfg(test)]
mod issue_texts_guard;

#[cfg(test)]
pub(crate) mod label_texts_export;

#[cfg(test)]
pub(crate) mod profile_texts_export;

#[cfg(test)]
pub(crate) mod schema_rules_texts_export;

#[cfg(test)]
pub(crate) mod serp_texts_export;

#[cfg(test)]
mod score_tests {
    use super::*;

    fn issue(field: &str, severity: crate::taxonomy::Severity) -> MetaValidation {
        MetaValidation {
            field: field.to_string(),
            message: String::new(),
            severity,
            suggestion: None,
            kind: None,
            values: Default::default(),
        }
    }

    /// A missing lang or canonical is listed once, by the technical checks,
    /// where the score charges it; the meta list does not repeat it.
    #[test]
    fn missing_lang_and_canonical_are_listed_and_charged_once() {
        let meta = MetaTags::default();
        assert!(!meta
            .validate()
            .iter()
            .any(|issue| matches!(issue.field.as_str(), "lang" | "canonical")));
        let technical = TechnicalSeo {
            https: true,
            has_canonical: false,
            has_lang: false,
            ..Default::default()
        };
        let social = SocialTags {
            open_graph: Some(Default::default()),
            twitter_card: Some(Default::default()),
            ..Default::default()
        };
        let headings = HeadingStructure {
            h1_count: 1,
            ..Default::default()
        };
        let score = calculate_seo_score(&meta, &[], &headings, &social, &technical, None, None);
        assert_eq!(score, 100 - 5 - 3);
    }

    /// A missing viewport is charged by Mobile and Accessibility;
    /// SEO lists it without a deduction.
    #[test]
    fn missing_viewport_costs_seo_nothing() {
        let technical = TechnicalSeo {
            https: true,
            has_canonical: true,
            has_lang: true,
            ..Default::default()
        };
        let social = SocialTags {
            open_graph: Some(Default::default()),
            twitter_card: Some(Default::default()),
            ..Default::default()
        };
        let headings = HeadingStructure {
            h1_count: 1,
            ..Default::default()
        };
        let meta = MetaTags::default();
        assert_eq!(
            calculate_seo_score(
                &meta,
                &[issue("viewport", crate::taxonomy::Severity::High)],
                &headings,
                &social,
                &technical,
                None,
                None,
            ),
            100
        );
    }

    fn health_issue(issue_type: &str, severity: crate::taxonomy::Severity) -> SeoIssue {
        SeoIssue {
            issue_type: issue_type.to_string(),
            message: String::new(),
            severity,
        }
    }

    /// Score of a page with every other signal in place, so only the
    /// page-health issues move it.
    fn score_with_health(
        issues: Vec<SeoIssue>,
        image_efficiency: Option<&ImageEfficiencyAnalysis>,
    ) -> u32 {
        let technical = TechnicalSeo {
            https: true,
            has_canonical: true,
            has_lang: true,
            ..Default::default()
        };
        let social = SocialTags {
            open_graph: Some(Default::default()),
            twitter_card: Some(Default::default()),
            ..Default::default()
        };
        let headings = HeadingStructure {
            h1_count: 1,
            ..Default::default()
        };
        let health = PageHealthAnalysis {
            issues,
            ..Default::default()
        };
        calculate_seo_score(
            &MetaTags::default(),
            &[],
            &headings,
            &social,
            &technical,
            image_efficiency,
            Some(&health),
        )
    }

    /// A score of 100 needs no open medium-or-higher page-health finding;
    /// exactly one medium finding already costs points (#1227).
    #[test]
    fn one_medium_page_health_issue_is_below_100() {
        use crate::taxonomy::Severity::*;
        assert_eq!(score_with_health(vec![], None), 100);
        assert_eq!(
            score_with_health(vec![health_issue("redirect", Medium)], None),
            96
        );
        assert_eq!(
            score_with_health(vec![health_issue("soft_404", High)], None),
            92
        );
    }

    /// Low findings are listed but cost nothing.
    #[test]
    fn low_page_health_issues_cost_nothing() {
        use crate::taxonomy::Severity::*;
        let issues = vec![
            health_issue("missing_preconnect", Low),
            health_issue("url_too_long", Low),
        ];
        assert_eq!(score_with_health(issues, None), 100);
    }

    /// Many findings stay at the cap, and one issue type is charged once.
    #[test]
    fn page_health_deduction_is_capped_and_charged_once_per_type() {
        use crate::taxonomy::Severity::*;
        let many: Vec<SeoIssue> = [
            "redirect",
            "www_not_consolidated",
            "trailing_slash_inconsistent",
            "hreflang_invalid",
            "jsonld_invalid",
            "charset_mismatch",
            "sync_head_scripts",
        ]
        .iter()
        .map(|t| health_issue(t, Medium))
        .collect();
        assert_eq!(score_with_health(many, None), 100 - PAGE_HEALTH_PENALTY_CAP);
        let twice = vec![
            health_issue("redirect", Medium),
            health_issue("redirect", Medium),
        ];
        assert_eq!(score_with_health(twice, None), 96);
    }

    /// A cause another score owns is not charged again: DOM size (Performance)
    /// and, when the image-efficiency deduction runs, oversized and
    /// legacy-format images.
    #[test]
    fn causes_owned_by_another_score_are_not_charged_here() {
        use crate::taxonomy::Severity::*;
        assert_eq!(
            score_with_health(
                vec![
                    health_issue("excessive_dom", High),
                    health_issue("large_dom", Medium)
                ],
                None
            ),
            100
        );
        let images = vec![
            health_issue("oversized_images", Medium),
            health_issue("images_without_modern_format", Medium),
        ];
        assert_eq!(score_with_health(images.clone(), None), 92);
        let ie = ImageEfficiencyAnalysis {
            total_images: 0,
            modern_format_count: 0,
            legacy_format_count: 0,
            modern_format_pct: 100.0,
            oversized_images: Vec::new(),
            legacy_format_urls: Vec::new(),
            score: 100,
        };
        assert_eq!(score_with_health(images, Some(&ie)), 100);
    }
}
