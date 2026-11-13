//! SERP pass — aggregates existing SEO signals into a search-result-page readiness check.
//!
//! No live search calls, no ML. Pure heuristics over already-collected data.

use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};
use web_checks::meta::LengthClass;

use crate::seo::SeoAnalysis;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerpSignalStatus {
    Ok,
    Warning,
    Fail,
}

impl SerpSignalStatus {
    /// The `seo-serp-status-*` message of `locales/{en,de}/seo-issues.ftl`.
    pub fn label(&self, en: bool) -> String {
        let key = match self {
            SerpSignalStatus::Ok => "seo-serp-status-ok",
            SerpSignalStatus::Warning => "seo-serp-status-warning",
            SerpSignalStatus::Fail => "seo-serp-status-fail",
        };
        crate::i18n::text(key, en)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerpSignal {
    pub category: String,
    pub label: String,
    pub status: SerpSignalStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerpAnalysis {
    pub signals: Vec<SerpSignal>,
    pub score: u32,
    pub pass_count: u32,
    pub warning_count: u32,
    pub fail_count: u32,
    /// Rich-result-related schema types detected; this is not an eligibility claim.
    pub rich_result_types: Vec<String>,
}

pub fn build_serp_analysis(seo: &SeoAnalysis, url: &str, locale: &str) -> SerpAnalysis {
    let en = locale == "en";
    let mut signals = Vec::new();

    // --- robots: noindex / nosnippet ---
    if let Some(robots_meta) = &seo.technical.robots_meta {
        let lower = robots_meta.to_ascii_lowercase();
        if lower.contains("noindex") {
            signals.push(sig(
                t("seo-serp-category-indexing", en),
                "robots noindex",
                SerpSignalStatus::Fail,
                arg_text("seo-serp-detail-noindex", "robots", robots_meta, en),
            ));
        }
        if lower.contains("nosnippet") {
            signals.push(sig(
                t("seo-serp-category-indexing", en),
                "robots nosnippet",
                SerpSignalStatus::Warning,
                t("seo-serp-detail-nosnippet", en),
            ));
        }
    }

    // --- Title ---
    match &seo.meta.title {
        None => signals.push(sig(
            t("seo-serp-category-title", en),
            t("seo-serp-label-title-present", en),
            SerpSignalStatus::Fail,
            t("seo-serp-detail-no-title", en),
        )),
        Some(title) => {
            let len = web_checks::meta::length(title);
            let status = if web_checks::meta::TITLE.classify(title) != LengthClass::Ok {
                SerpSignalStatus::Warning
            } else {
                SerpSignalStatus::Ok
            };
            signals.push(sig(
                t("seo-serp-category-title", en),
                t("seo-serp-label-title-length", en),
                status,
                count_text("seo-serp-detail-title-length", len, en),
            ));
            if len > 55 {
                signals.push(sig(
                    t("seo-serp-category-title", en),
                    t("seo-serp-label-truncation-risk", en),
                    SerpSignalStatus::Warning,
                    count_text("seo-serp-detail-title-truncated", len, en),
                ));
            }
        }
    }

    // --- Meta Description ---
    match &seo.meta.description {
        None => signals.push(sig(
            "Description",
            t("seo-serp-label-description-present", en),
            SerpSignalStatus::Fail,
            t("seo-serp-detail-no-description", en),
        )),
        Some(desc) => {
            let len = web_checks::meta::length(desc);
            let status = if web_checks::meta::DESCRIPTION.classify(desc) != LengthClass::Ok {
                SerpSignalStatus::Warning
            } else {
                SerpSignalStatus::Ok
            };
            signals.push(sig(
                "Description",
                t("seo-serp-label-description-length", en),
                status,
                count_text("seo-serp-detail-description-length", len, en),
            ));
        }
    }

    // --- Canonical ---
    signals.push(sig(
        "Canonical",
        t("seo-serp-label-canonical-tag", en),
        if seo.technical.has_canonical {
            SerpSignalStatus::Ok
        } else {
            SerpSignalStatus::Warning
        },
        if seo.technical.has_canonical {
            seo.technical.canonical_url.clone().unwrap_or_default()
        } else {
            t("seo-serp-detail-no-canonical", en)
        },
    ));

    if let Some(canon) = &seo.technical.canonical_url {
        let canon_clean = canon.trim_end_matches('/');
        let url_clean = url.trim_end_matches('/');
        if !canon_clean.is_empty() && !canon_clean.eq_ignore_ascii_case(url_clean) {
            signals.push(sig(
                "Canonical",
                t("seo-serp-label-canonical-self-reference", en),
                SerpSignalStatus::Warning,
                arg_text("seo-serp-detail-canonical-other-url", "url", canon, en),
            ));
        }
    }

    // --- Favicon ---
    signals.push(sig(
        t("seo-serp-category-appearance", en),
        "Favicon",
        if seo.technical.has_favicon {
            SerpSignalStatus::Ok
        } else {
            SerpSignalStatus::Warning
        },
        if seo.technical.has_favicon {
            t("seo-serp-detail-favicon-present", en)
        } else {
            t("seo-serp-detail-no-favicon", en)
        },
    ));

    // --- Hreflang x-default ---
    if seo.technical.has_hreflang {
        let has_x_default = seo.technical.hreflang.iter().any(|h| h.lang == "x-default");
        signals.push(sig(
            t("seo-serp-category-internationalisation", en),
            "hreflang x-default",
            if has_x_default {
                SerpSignalStatus::Ok
            } else {
                SerpSignalStatus::Warning
            },
            count_text(
                if has_x_default {
                    "seo-serp-detail-hreflang-x-default"
                } else {
                    "seo-serp-detail-hreflang-no-x-default"
                },
                seo.technical.hreflang.len(),
                en,
            ),
        ));
    }

    // --- Breadcrumb Schema ---
    {
        use crate::seo::{SchemaRequirementStatus, SchemaType};
        let has_breadcrumb = seo
            .structured_data
            .types
            .contains(&SchemaType::BreadcrumbList);
        // Type presence alone never promises a SERP path display: the same
        // report validates BreadcrumbList's required properties, and an
        // incomplete node is not eligible. Reporting "Google shows the path"
        // next to "required properties missing" contradicted itself.
        let incomplete: Vec<&str> = seo
            .structured_data
            .rule_assessments
            .iter()
            .filter(|a| {
                a.schema_type == "BreadcrumbList"
                    && a.requirement_status == SchemaRequirementStatus::MissingRequiredProperties
            })
            .flat_map(|a| a.missing_required.iter().map(String::as_str))
            .collect();

        let (status, detail) = match (has_breadcrumb, incomplete.is_empty()) {
            (true, true) => (
                SerpSignalStatus::Ok,
                t("seo-serp-detail-breadcrumb-complete", en),
            ),
            (true, false) => (
                SerpSignalStatus::Warning,
                arg_text(
                    "seo-serp-detail-breadcrumb-incomplete",
                    "list",
                    &incomplete.join(", "),
                    en,
                ),
            ),
            (false, _) => (
                SerpSignalStatus::Warning,
                t("seo-serp-detail-no-breadcrumb", en),
            ),
        };
        signals.push(sig(
            "Rich Result",
            t("seo-serp-label-breadcrumb-schema", en),
            status,
            detail,
        ));
    }

    // --- Score ---
    let pass_count = signals
        .iter()
        .filter(|s| s.status == SerpSignalStatus::Ok)
        .count() as u32;
    let warning_count = signals
        .iter()
        .filter(|s| s.status == SerpSignalStatus::Warning)
        .count() as u32;
    let fail_count = signals
        .iter()
        .filter(|s| s.status == SerpSignalStatus::Fail)
        .count() as u32;
    let total = signals.len() as u32;
    let score = (pass_count * 100 + warning_count * 50)
        .checked_div(total)
        .unwrap_or(100)
        .min(100);

    SerpAnalysis {
        signals,
        score,
        pass_count,
        warning_count,
        fail_count,
        rich_result_types: seo.structured_data.rich_snippets_potential.clone(),
    }
}

/// The `seo-serp-*` message `key` of `locales/{en,de}/seo-issues.ftl`.
fn t(key: &str, en: bool) -> String {
    crate::i18n::text(key, en)
}

/// The `seo-serp-*` message `key` with the number `$count`.
fn count_text(key: &str, count: usize, en: bool) -> String {
    crate::i18n::count_text(key, count, en)
}

/// The `seo-serp-*` message `key` with the text argument `name`.
fn arg_text(key: &str, name: &str, value: &str, en: bool) -> String {
    crate::i18n::text_args(key, &[(name, FluentValue::from(value.to_string()))], en)
}

fn sig(
    category: impl Into<String>,
    label: impl Into<String>,
    status: SerpSignalStatus,
    detail: impl Into<String>,
) -> SerpSignal {
    SerpSignal {
        category: category.into(),
        label: label.into(),
        status,
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seo::technical::HreflangTag;
    use crate::seo::SchemaType;

    fn has_german_chars(s: &str) -> bool {
        s.chars().any(|c| "äöüÄÖÜß".contains(c))
    }

    fn breadcrumb_assessment(
        status: crate::seo::SchemaRequirementStatus,
        missing: &[&str],
    ) -> crate::seo::SchemaRuleAssessment {
        crate::seo::SchemaRuleAssessment {
            node_index: 0,
            schema_type: "BreadcrumbList".to_string(),
            feature: crate::seo::SchemaFeature::Breadcrumb,
            availability: crate::seo::SchemaFeatureAvailability::General,
            requirement_status: status,
            missing_required: missing.iter().map(|s| s.to_string()).collect(),
            missing_recommended: Vec::new(),
            manual_review: Vec::new(),
            quality_score: None,
            source_url: String::new(),
            reviewed_at: String::new(),
        }
    }

    fn breadcrumb_signal(seo: &SeoAnalysis) -> SerpSignal {
        build_serp_analysis(seo, "https://example.com/", "de")
            .signals
            .into_iter()
            .find(|s| s.label == "Breadcrumb-Schema")
            .expect("breadcrumb signal is always emitted")
    }

    /// Regression: the signal only checked whether the *type* was present, so
    /// a BreadcrumbList missing `itemListElement` was reported as "Google
    /// shows the path" on one page while the schema table on another page of
    /// the same report said "required properties missing".
    #[test]
    fn incomplete_breadcrumb_schema_is_not_reported_as_eligible() {
        let mut seo = SeoAnalysis::default();
        seo.structured_data.types.push(SchemaType::BreadcrumbList);
        seo.structured_data
            .rule_assessments
            .push(breadcrumb_assessment(
                crate::seo::SchemaRequirementStatus::MissingRequiredProperties,
                &["itemListElement (minimum 2 entries)"],
            ));

        let signal = breadcrumb_signal(&seo);
        assert_eq!(signal.status, SerpSignalStatus::Warning);
        assert!(signal.detail.contains("itemListElement"), "{signal:?}");
    }

    #[test]
    fn complete_breadcrumb_schema_stays_eligible() {
        let mut seo = SeoAnalysis::default();
        seo.structured_data.types.push(SchemaType::BreadcrumbList);
        seo.structured_data
            .rule_assessments
            .push(breadcrumb_assessment(
                crate::seo::SchemaRequirementStatus::MeetsRequiredProperties,
                &[],
            ));

        assert_eq!(breadcrumb_signal(&seo).status, SerpSignalStatus::Ok);
    }

    #[test]
    fn test_serp_signals_have_no_german_chars_in_en() {
        // Populate technical + meta so every conditional branch that produces a
        // visible label/detail string is exercised.
        let mut seo = SeoAnalysis::default();
        seo.meta.title =
            Some("A title long enough to trip the truncation risk branch here".to_string());
        seo.meta.description = Some("short".to_string());
        seo.technical.robots_meta = Some("noindex, nosnippet".to_string());
        seo.technical.has_canonical = true;
        seo.technical.canonical_url = Some("https://example.com/other".to_string());
        seo.technical.has_favicon = false;
        seo.technical.has_hreflang = true;
        seo.technical.hreflang = vec![HreflangTag {
            lang: "de".to_string(),
            url: "https://example.com/de".to_string(),
        }];
        seo.structured_data.types = vec![SchemaType::Article];

        let analysis = build_serp_analysis(&seo, "https://example.com/", "en");
        for signal in &analysis.signals {
            assert!(
                !has_german_chars(&signal.category),
                "German chars in category: {}",
                signal.category
            );
            assert!(
                !has_german_chars(&signal.label),
                "German chars in label: {}",
                signal.label
            );
            assert!(
                !has_german_chars(&signal.detail),
                "German chars in detail: {}",
                signal.detail
            );
        }
    }
}
