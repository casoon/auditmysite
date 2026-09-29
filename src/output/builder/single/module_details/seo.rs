use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
use crate::output::report_model::{
    ImageEfficiencyPresentation, OversizedImageRow, RobotsPresentation, SeoPresentation,
    SeoProfilePresentation, SignalDetails,
};
use crate::taxonomy::SeverityExt;

use super::super::super::helpers::{truncate_list, truncate_url_list, yes_no};
use super::super::super::modules::build_tracking_summary_text;
use super::super::serp::{build_page_health_presentation, build_serp_presentation};
use super::normalized_module_score;
use crate::seo::interpretation::{
    page_profile_optimization_note_text, page_profile_summary_text, seo_interpretation_text,
};
use crate::seo::{ImageEfficiencyAnalysis, RobotsAudit, SeoAnalysis};
use crate::util::truncate_url;

pub(super) fn build_seo_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<SeoPresentation> {
    let locale = i18n.locale();
    let en = locale == "en";
    normalized.raw_seo.map(|s| {
        let seo_score = normalized_module_score(&normalized.normalized, "SEO").unwrap_or(s.score);
        let mut meta_tags = Vec::new();
        if let Some(ref title) = s.meta.title {
            meta_tags.push((
                if en { "Title" } else { "Titel" }.to_string(),
                title.clone(),
            ));
        }
        if let Some(ref desc) = s.meta.description {
            meta_tags.push((
                if en { "Description" } else { "Beschreibung" }.to_string(),
                desc.clone(),
            ));
        }
        if let Some(ref viewport) = s.meta.viewport {
            meta_tags.push(("Viewport".to_string(), viewport.clone()));
        }

        let meta_issues: Vec<(String, crate::wcag::Severity, String)> = s
            .meta_issues
            .iter()
            .map(|i| (i.field.clone(), i.severity, i.message.clone()))
            .collect();

        // Re-derive the content profile in the report locale. `s.content_profile`
        // is stored in canonical English (for JSON); the PDF localizes here (#406).
        let localized_profile = crate::seo::build_content_profile(s, locale);
        let profile = Some(&localized_profile).map(|cp| {
            use crate::seo::profile::SchemaExtracted;

            let structural_issues: Vec<_> = s
                .structured_data
                .schema_issues
                .iter()
                .filter(|issue| issue.issue_type.starts_with("jsonld_"))
                .collect();
            let valid_schema_nodes = s
                .structured_data
                .json_ld
                .iter()
                .filter(|schema| schema.is_valid && !schema.schema_types.is_empty())
                .count();
            let schema_status_has_json_ld = !s.structured_data.json_ld.is_empty();
            let schema_status_summary = if !structural_issues.is_empty() {
                i18n.t_args(
                    "pdf-seo-schema-status-issues",
                    &[
                        ("nodes", valid_schema_nodes as i64),
                        ("issues", structural_issues.len() as i64),
                    ],
                )
            } else if schema_status_has_json_ld {
                i18n.t_args(
                    "pdf-seo-schema-status-ok",
                    &[("nodes", valid_schema_nodes as i64)],
                )
            } else if s.structured_data.has_structured_data {
                i18n.t("pdf-seo-schema-status-non-jsonld")
            } else {
                i18n.t("pdf-seo-schema-status-none")
            };

            let mut grouped_structural_issues: std::collections::BTreeMap<
                &str,
                (&crate::seo::schema::SchemaIssue, usize),
            > = std::collections::BTreeMap::new();
            for issue in &structural_issues {
                grouped_structural_issues
                    .entry(issue.issue_type.as_str())
                    .and_modify(|(_, count)| *count += 1)
                    .or_insert((issue, 1));
            }
            let schema_status_rows = grouped_structural_issues
                .into_values()
                .map(|(issue, count)| {
                    let label = crate::seo::schema::schema_issue_label(issue, en);
                    (
                        if count > 1 {
                            format!("{label} ({count}×)")
                        } else {
                            label.to_string()
                        },
                        crate::seo::schema::schema_issue_text(issue, en),
                    )
                })
                .collect();

            let (
                schema_fit_summary,
                schema_fit_is_success,
                schema_fit_is_warning,
                schema_fit_facts,
            ) = if let Some(fit) = &s.structured_data.fit_assessment {
                use crate::seo::schema_fit::{SchemaCoverageStatus, SchemaFitStatus};

                let expected = if fit.expected_primary_types.is_empty() {
                    i18n.t("pdf-seo-schema-fit-none")
                } else {
                    fit.expected_primary_types.join(", ")
                };
                let detected = if fit.detected_primary_types.is_empty() {
                    i18n.t("pdf-seo-schema-fit-none")
                } else {
                    fit.detected_primary_types.join(", ")
                };
                (
                    Some(fit.fit_text(en).to_string()),
                    fit.fit_status == SchemaFitStatus::Matched,
                    fit.fit_status == SchemaFitStatus::Mismatch
                        || fit.coverage_status == SchemaCoverageStatus::Opportunity,
                    vec![
                        (
                            i18n.t("pdf-seo-schema-fit-page-type"),
                            fit.page_kind.label(en).to_string(),
                        ),
                        (
                            i18n.t("pdf-seo-schema-fit-confidence"),
                            format!("{}%", fit.confidence),
                        ),
                        (i18n.t("pdf-seo-schema-fit-expected"), expected),
                        (i18n.t("pdf-seo-schema-fit-detected"), detected),
                    ],
                )
            } else {
                (None, false, false, Vec::new())
            };

            let schema_rule_rows = s
                .structured_data
                .rule_assessments
                .iter()
                .map(|assessment| {
                    use crate::seo::schema_rules::SchemaFeatureAvailability;

                    let status = match assessment.availability {
                        SchemaFeatureAvailability::Limited => format!(
                            "{} - {}",
                            crate::seo::schema_rules::status_text(assessment, en),
                            i18n.t("pdf-seo-schema-rule-limited")
                        ),
                        SchemaFeatureAvailability::ContextDependent => format!(
                            "{} - {}",
                            crate::seo::schema_rules::status_text(assessment, en),
                            i18n.t("pdf-seo-schema-rule-context-dependent")
                        ),
                        SchemaFeatureAvailability::General => {
                            crate::seo::schema_rules::status_text(assessment, en).to_string()
                        }
                    };
                    let detail = if !assessment.missing_required.is_empty() {
                        i18n.t_args(
                            "pdf-seo-schema-rule-missing-required",
                            &[("fields", assessment.missing_required.join(", "))],
                        )
                    } else if !assessment.missing_recommended.is_empty() {
                        i18n.t_args(
                            "pdf-seo-schema-rule-missing-recommended",
                            &[("fields", assessment.missing_recommended.join(", "))],
                        )
                    } else if assessment.requirement_status
                        == crate::seo::schema_rules::SchemaRequirementStatus::NotEvaluated
                    {
                        i18n.t("pdf-seo-schema-rule-not-evaluated")
                    } else {
                        i18n.t("pdf-seo-schema-rule-complete")
                    };
                    (
                        format!(
                            "{} - {}",
                            assessment.schema_type,
                            crate::seo::schema_rules::feature_label(assessment.feature, en)
                        ),
                        status,
                        detail,
                    )
                })
                .collect();

            let schema_manual_review_rows = s
                .structured_data
                .rule_assessments
                .iter()
                .flat_map(|assessment| {
                    assessment.manual_review.iter().map(|review| {
                        (
                            format!(
                                "{} - {}",
                                assessment.schema_type,
                                crate::seo::schema_rules::feature_label(assessment.feature, en)
                            ),
                            crate::seo::schema_rules::manual_review_text(*review, en).to_string(),
                        )
                    })
                })
                .collect();

            let schema_parity_rows = s
                .structured_data
                .content_parity
                .iter()
                .map(|assessment| {
                    (
                        format!("{} - {}", assessment.schema_type, assessment.property),
                        assessment.status_text(en).to_string(),
                        assessment.evidence_text(en),
                    )
                })
                .collect();

            let schema_rows: Vec<(String, String, String)> = cp
                .schema_inventory
                .schemas
                .iter()
                .map(|sd| {
                    let detail = match &sd.extracted {
                        SchemaExtracted::Organization { name, .. } => {
                            name.clone().unwrap_or_default()
                        }
                        SchemaExtracted::LocalBusiness { name, address, .. } => format!(
                            "{}{}",
                            name.as_deref().unwrap_or(""),
                            address
                                .as_ref()
                                .map(|a| format!(", {}", a))
                                .unwrap_or_default()
                        ),
                        SchemaExtracted::Article {
                            headline, author, ..
                        } => format!(
                            "{}{}",
                            headline.as_deref().unwrap_or(""),
                            author
                                .as_ref()
                                .map(|a| format!(" ({})", a))
                                .unwrap_or_default()
                        ),
                        SchemaExtracted::FAQPage { question_count, .. } => {
                            if en {
                                format!("{} questions", question_count)
                            } else {
                                format!("{} Fragen", question_count)
                            }
                        }
                        SchemaExtracted::Product {
                            name,
                            price,
                            currency,
                            ..
                        } => format!(
                            "{}{}",
                            name.as_deref().unwrap_or(""),
                            price
                                .as_ref()
                                .map(|p| format!(" — {} {}", p, currency.as_deref().unwrap_or("")))
                                .unwrap_or_default()
                        ),
                        SchemaExtracted::WebSite {
                            name,
                            has_search_action,
                            ..
                        } => format!(
                            "{}{}",
                            name.as_deref().unwrap_or(""),
                            if *has_search_action {
                                if en {
                                    " (search)"
                                } else {
                                    " (Suche)"
                                }
                            } else {
                                ""
                            }
                        ),
                        SchemaExtracted::WebPage {
                            name,
                            author,
                            in_language,
                            ..
                        } => format!(
                            "{}{}{}",
                            name.as_deref().unwrap_or(""),
                            author
                                .as_ref()
                                .map(|a| format!(" ({})", a))
                                .unwrap_or_default(),
                            in_language
                                .as_ref()
                                .map(|lang| format!(" · {}", lang))
                                .unwrap_or_default()
                        ),
                        SchemaExtracted::Service {
                            name,
                            address,
                            area_served_count,
                            ..
                        } => format!(
                            "{}{}{}",
                            name.as_deref().unwrap_or(""),
                            address
                                .as_ref()
                                .map(|a| format!(" — {}", a))
                                .unwrap_or_default(),
                            if *area_served_count > 0 {
                                if en {
                                    format!(" · {} regions", area_served_count)
                                } else {
                                    format!(" · {} Regionen", area_served_count)
                                }
                            } else {
                                String::new()
                            }
                        ),
                        SchemaExtracted::BreadcrumbList { item_count } => {
                            if en {
                                format!("{} levels", item_count)
                            } else {
                                format!("{} Ebenen", item_count)
                            }
                        }
                        SchemaExtracted::Generic { key_fields } => key_fields
                            .first()
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .unwrap_or_default(),
                    };
                    (
                        sd.schema_type.clone(),
                        format!("{}%", sd.completeness_pct),
                        detail,
                    )
                })
                .collect();

            let signal_rows: Vec<(String, String, String)> = cp
                .signal_strength
                .categories
                .iter()
                .map(|cat| {
                    let rating = match (cat.pct(), en) {
                        (90..=100, true) => "Excellent",
                        (67..=89, true) => "Good",
                        (34..=66, true) => "Partial",
                        (1..=33, true) => "Minimal",
                        (_, true) => "Missing",
                        (90..=100, false) => "Sehr gut",
                        (67..=89, false) => "Gut",
                        (34..=66, false) => "Teilweise",
                        (1..=33, false) => "Minimal",
                        (_, false) => "Fehlt",
                    };
                    (
                        cat.name.clone(),
                        // Counts, not a percentage: the denominator is the
                        // point (plan 29, D4).
                        format!("{} / {}", cat.passed, cat.total),
                        rating.to_string(),
                    )
                })
                .collect();

            let signal_details: SignalDetails = cp
                .signal_strength
                .categories
                .iter()
                .map(|cat| {
                    let checks = cat
                        .checks
                        .iter()
                        .map(|c| {
                            (
                                c.label.clone(),
                                c.passed,
                                c.detail.clone().unwrap_or_default(),
                            )
                        })
                        .collect();
                    (cat.name.clone(), checks)
                })
                .collect();

            SeoProfilePresentation {
                identity_summary: cp.content_identity.summary.clone(),
                site_name: cp
                    .content_identity
                    .site_name
                    .clone()
                    .unwrap_or_else(|| "—".to_string()),
                content_type: cp.content_identity.content_type.clone(),
                language: cp
                    .content_identity
                    .language
                    .clone()
                    .unwrap_or_else(|| "—".to_string()),
                category_hints: cp.content_identity.category_hints.clone(),
                identity_facts: vec![
                    (
                        if en { "Page title" } else { "Seitentitel" }.to_string(),
                        cp.content_identity
                            .site_name
                            .clone()
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        if en { "Content type" } else { "Inhaltstyp" }.to_string(),
                        cp.content_identity.content_type.clone(),
                    ),
                    (
                        if en { "Language" } else { "Sprache" }.to_string(),
                        cp.content_identity
                            .language
                            .clone()
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        if en { "Topic hints" } else { "Themenhinweise" }.to_string(),
                        if cp.content_identity.category_hints.is_empty() {
                            if en {
                                "No clear topic hints detected".to_string()
                            } else {
                                "Keine klaren Themenhinweise erkannt".to_string()
                            }
                        } else {
                            cp.content_identity.category_hints.join(", ")
                        },
                    ),
                ],
                page_type: cp.page_classification.primary_type.label(en).to_string(),
                page_attributes: cp.page_classification.attributes.clone(),
                content_depth_score: cp.page_classification.content_depth_score,
                structural_richness_score: cp.page_classification.structural_richness_score,
                media_text_balance_score: cp.page_classification.media_text_balance_score,
                intent_fit_score: cp.page_classification.intent_fit_score,
                page_profile_summary: page_profile_summary_text(cp, en),
                optimization_note: page_profile_optimization_note_text(cp, en),
                page_profile_facts: vec![
                    (
                        if en { "Page type" } else { "Seitentyp" }.to_string(),
                        cp.page_classification.primary_type.label(en).to_string(),
                    ),
                    (
                        if en { "Characteristics" } else { "Merkmale" }.to_string(),
                        if cp.page_classification.attributes.is_empty() {
                            if en {
                                "No defining characteristics detected".to_string()
                            } else {
                                "Keine prägenden Merkmale erkannt".to_string()
                            }
                        } else {
                            format!("{}.", cp.page_classification.attributes.join(", "))
                        },
                    ),
                    (
                        if en { "Classification" } else { "Einordnung" }.to_string(),
                        page_profile_summary_text(cp, en),
                    ),
                    (
                        if en { "Recommendation" } else { "Empfehlung" }.to_string(),
                        page_profile_optimization_note_text(cp, en),
                    ),
                ],
                schema_rows,
                schema_count: cp.schema_inventory.total_count,
                schema_status_summary,
                schema_status_has_errors: !structural_issues.is_empty(),
                schema_status_has_json_ld,
                schema_status_rows,
                schema_fit_summary,
                schema_fit_is_success,
                schema_fit_is_warning,
                schema_fit_facts,
                schema_rule_rows,
                schema_manual_review_rows,
                schema_parity_rows,
                signal_rows,
                signal_passed: cp.signal_strength.totals().0,
                signal_total: cp.signal_strength.totals().1,
                signal_details,
                maturity_level: cp.maturity.label(en).to_string(),
                maturity_description: cp.maturity.description(en).to_string(),
                maturity_techniques_used: cp.maturity_techniques,
                maturity_techniques_total: 13,
            }
        });

        SeoPresentation {
            score: seo_score,
            interpretation: seo_interpretation_text(s, en),
            meta_tags,
            meta_issues,
            heading_summary: heading_summary(s, en),
            social_summary: social_summary(s, en),
            technical_summary: technical_summary_rows(s, locale, en),
            tracking_summary: tracking_summary_rows(normalized, s, en),
            tracking_summary_text: build_tracking_summary_text(i18n, &s.technical),
            profile,
            page_health: s
                .page_health
                .as_ref()
                .map(|p| build_page_health_presentation(locale, p)),
            // Re-derive SERP signals in the report locale; `s.serp` is canonical
            // English for JSON, the PDF localizes here (#406).
            serp: s.serp.as_ref().map(|_| {
                let localized =
                    crate::seo::build_serp_analysis(s, &normalized.normalized.url, locale);
                build_serp_presentation(locale, &localized)
            }),
            robots: s.robots.as_ref().map(|r| build_robots_presentation(r, en)),
            image_efficiency: s
                .image_efficiency
                .as_ref()
                .filter(|ie| ie.total_images > 0)
                .map(build_image_efficiency_presentation),
            // Re-derive technical issues in the report locale; `s.technical.issues`
            // is canonical English for JSON, the PDF localizes here (#406).
            technical_issues: technical_issue_rows(s, en),
        }
    })
}

fn heading_summary(s: &SeoAnalysis, en: bool) -> String {
    if en {
        format!(
            "{} H1 heading(s), {} headings total, {} issues",
            s.headings.h1_count,
            s.headings.total_count,
            s.headings.issues.len()
        )
    } else {
        format!(
            "{} H1-Überschrift(en), {} Überschriften gesamt, {} Probleme",
            s.headings.h1_count,
            s.headings.total_count,
            s.headings.issues.len()
        )
    }
}

fn social_summary(s: &SeoAnalysis, en: bool) -> String {
    if en {
        format!(
            "Open Graph: {}, Twitter Card: {}, Completeness: {}%",
            if s.social.open_graph.is_some() {
                "present"
            } else {
                "missing"
            },
            if s.social.twitter_card.is_some() {
                "present"
            } else {
                "missing"
            },
            s.social.completeness
        )
    } else {
        format!(
            "Open Graph: {}, Twitter Card: {}, Vollständigkeit: {}%",
            if s.social.open_graph.is_some() {
                "vorhanden"
            } else {
                "fehlt"
            },
            if s.social.twitter_card.is_some() {
                "vorhanden"
            } else {
                "fehlt"
            },
            s.social.completeness
        )
    }
}

fn technical_summary_rows(s: &SeoAnalysis, locale: &str, en: bool) -> Vec<(String, String)> {
    vec![
        ("HTTPS".to_string(), yes_no(locale, s.technical.https)),
        (
            "Canonical".to_string(),
            yes_no(locale, s.technical.has_canonical),
        ),
        (
            if en { "Language tag" } else { "Sprachangabe" }.to_string(),
            yes_no(locale, s.technical.has_lang),
        ),
        (
            if en { "Word count" } else { "Wortanzahl" }.to_string(),
            s.technical.word_count.to_string(),
        ),
        (
            if en {
                "Internal links"
            } else {
                "Interne Links"
            }
            .to_string(),
            s.technical.internal_links.to_string(),
        ),
        (
            if en {
                "External links"
            } else {
                "Externe Links"
            }
            .to_string(),
            s.technical.external_links.to_string(),
        ),
        (
            if en {
                "Dofollow links"
            } else {
                "Dofollow-Links"
            }
            .to_string(),
            s.technical.dofollow_links.to_string(),
        ),
        (
            if en {
                "Nofollow links"
            } else {
                "Nofollow-Links"
            }
            .to_string(),
            s.technical.nofollow_links.to_string(),
        ),
        (
            "AMP".to_string(),
            if !s.technical.amp.detected {
                if en {
                    "Not detected".to_string()
                } else {
                    "Nicht erkannt".to_string()
                }
            } else if s.technical.amp.issues.is_empty() {
                if en {
                    "Detected, basic signals complete".to_string()
                } else {
                    "Erkannt, Basis-Signale vollständig".to_string()
                }
            } else if en {
                format!("Detected, {} basic issue(s)", s.technical.amp.issues.len())
            } else {
                format!("Erkannt, {} Basisproblem(e)", s.technical.amp.issues.len())
            },
        ),
    ]
}

fn tracking_summary_rows(
    normalized: &AuditContext<'_>,
    s: &SeoAnalysis,
    en: bool,
) -> Vec<(String, String)> {
    vec![
        (
            if en {
                "Google Fonts (external)"
            } else {
                "Google Fonts (extern)"
            }
            .to_string(),
            if s.technical.uses_remote_google_fonts {
                format!(
                    "{} ({})",
                    if en { "Yes" } else { "Ja" },
                    truncate_url_list(&s.technical.google_fonts_sources, 2, 48)
                )
            } else if en {
                "No".to_string()
            } else {
                "Nein".to_string()
            },
        ),
        (
            if en {
                "Tracking cookies"
            } else {
                "Tracking-Cookies"
            }
            .to_string(),
            if s.technical.tracking_cookies.is_empty() {
                if en {
                    "None detected".to_string()
                } else {
                    "Keine erkannt".to_string()
                }
            } else {
                format!(
                    "{} ({})",
                    s.technical.tracking_cookies.len(),
                    s.technical
                        .tracking_cookies
                        .iter()
                        .map(|c| c.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
        ),
        (
            if en {
                "Cookie inventory"
            } else {
                "Cookie-Inventar"
            }
            .to_string(),
            if s.technical.cookie_inventory.is_empty() {
                if en {
                    "No cookies detected".to_string()
                } else {
                    "Keine Cookies erkannt".to_string()
                }
            } else {
                let risky = s
                    .technical
                    .cookie_inventory
                    .iter()
                    .filter(|cookie| !cookie.secure || cookie.same_site.is_none())
                    .count();
                let category_suffix = tracking_category_summary(
                    s.technical
                        .cookie_inventory
                        .iter()
                        .filter_map(|cookie| cookie.category.as_deref()),
                )
                .map(|summary| {
                    if en {
                        format!("; categories: {summary}")
                    } else {
                        format!("; Kategorien: {summary}")
                    }
                })
                .unwrap_or_default();
                if en {
                    format!(
                        "{} cookies, {} without Secure/SameSite signal{}",
                        s.technical.cookie_inventory.len(),
                        risky,
                        category_suffix
                    )
                } else {
                    format!(
                        "{} Cookies, {} ohne Secure/SameSite-Signal{}",
                        s.technical.cookie_inventory.len(),
                        risky,
                        category_suffix
                    )
                }
            },
        ),
        (
            "Browser Storage".to_string(),
            if s.technical.storage_items.is_empty() {
                if en {
                    "No keys detected".to_string()
                } else {
                    "Keine Keys erkannt".to_string()
                }
            } else {
                format!(
                    "{} ({}){}",
                    s.technical.storage_items.len(),
                    s.technical
                        .storage_items
                        .iter()
                        .map(|item| format!("{}:{}", item.area, item.key))
                        .collect::<Vec<_>>()
                        .join(", "),
                    tracking_category_summary(
                        s.technical
                            .storage_items
                            .iter()
                            .filter_map(|item| item.category.as_deref()),
                    )
                    .map(|summary| {
                        if en {
                            format!("; categories: {summary}")
                        } else {
                            format!("; Kategorien: {summary}")
                        }
                    })
                    .unwrap_or_default()
                )
            },
        ),
        (
            if en {
                "Tracking signals"
            } else {
                "Tracking-Signale"
            }
            .to_string(),
            if s.technical.tracking_signals.is_empty() {
                if en {
                    "None detected".to_string()
                } else {
                    "Keine erkannt".to_string()
                }
            } else {
                truncate_list(&s.technical.tracking_signals, 3)
            },
        ),
        (
            if en {
                "Consent cookies"
            } else {
                "Consent-Cookies"
            }
            .to_string(),
            normalized
                .normalized
                .consent_privacy
                .as_ref()
                .map(|snapshot| {
                    let category_suffix = tracking_category_summary(
                        snapshot
                            .added_after_interaction
                            .iter()
                            .filter_map(|cookie| cookie.category.as_deref()),
                    )
                    .map(|summary| {
                        if en {
                            format!("; categories: {summary}")
                        } else {
                            format!("; Kategorien: {summary}")
                        }
                    })
                    .unwrap_or_default();
                    if en {
                        format!(
                            "{} before interaction, {} after interaction, {} new{}",
                            snapshot.before_interaction.len(),
                            snapshot.after_interaction.len(),
                            snapshot.added_after_interaction.len(),
                            category_suffix
                        )
                    } else {
                        format!(
                            "{} vor Interaktion, {} nach Interaktion, {} neu{}",
                            snapshot.before_interaction.len(),
                            snapshot.after_interaction.len(),
                            snapshot.added_after_interaction.len(),
                            category_suffix
                        )
                    }
                })
                .unwrap_or_else(|| {
                    if en {
                        "Not collected".to_string()
                    } else {
                        "Nicht erhoben".to_string()
                    }
                }),
        ),
        (
            "Zaraz".to_string(),
            if s.technical.zaraz.detected {
                if en {
                    format!(
                        "Detected ({})",
                        truncate_list(&s.technical.zaraz.signals, 2)
                    )
                } else {
                    format!("Erkannt ({})", truncate_list(&s.technical.zaraz.signals, 2))
                }
            } else if en {
                "Not detected".to_string()
            } else {
                "Nicht erkannt".to_string()
            },
        ),
    ]
}

fn build_robots_presentation(r: &RobotsAudit, en: bool) -> RobotsPresentation {
    use crate::seo::BotClass;
    let bot_rows: Vec<(String, String, usize, usize, bool)> = r
        .groups
        .iter()
        .map(|g| {
            let fully_blocked = g.disallows.iter().any(|d| d == "/");
            (
                g.user_agent.clone(),
                g.bot_class.label(en).to_string(),
                g.allows.len(),
                g.disallows.len(),
                fully_blocked,
            )
        })
        .collect();

    let blocked_ai_bots: Vec<String> = r
        .groups
        .iter()
        .filter(|g| {
            matches!(
                g.bot_class,
                BotClass::AiTraining
                    | BotClass::AiCitation
                    | BotClass::AiMixed
                    | BotClass::UnknownAi
            ) && g.disallows.iter().any(|d| d == "/")
        })
        .map(|g| g.user_agent.clone())
        .collect();

    RobotsPresentation {
        error: r.error.clone(),
        has_wildcard_disallow_all: r.has_wildcard_disallow_all,
        blocks_ai_crawlers: r.blocks_ai_crawlers,
        blocks_ai_citation: r.blocks_ai_citation,
        // Re-derive the policy label in the report locale; the stored
        // value is canonical English for JSON (#406).
        inferred_policy: crate::seo::infer_robots_policy(r, en),
        sitemaps: r.sitemaps.clone(),
        crawl_delays: r.crawl_delays.clone(),
        bot_rows,
        blocked_ai_bots,
        noindex_in_sitemap: r.noindex_in_sitemap,
    }
}

fn build_image_efficiency_presentation(
    ie: &ImageEfficiencyAnalysis,
) -> ImageEfficiencyPresentation {
    ImageEfficiencyPresentation {
        total_images: ie.total_images,
        modern_format_pct: ie.modern_format_pct,
        legacy_count: ie.legacy_format_count,
        oversized: ie
            .oversized_images
            .iter()
            .take(5)
            .map(|o| OversizedImageRow {
                src: truncate_url(&o.src, 60),
                natural: format!("{}×{}", o.natural_width, o.natural_height),
                display: format!("{}×{}", o.display_width, o.display_height),
            })
            .collect(),
    }
}

fn technical_issue_rows(s: &SeoAnalysis, en: bool) -> Vec<(String, String, String)> {
    crate::seo::collect_technical_issues(&s.technical, en)
        .iter()
        .map(|i| {
            (
                i.issue_type.clone(),
                i.message.clone(),
                if en {
                    i.severity.label_en().to_string()
                } else {
                    i.severity.label().to_string()
                },
            )
        })
        .collect()
}

fn tracking_category_summary<'a>(categories: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut counts = std::collections::BTreeMap::new();
    for category in categories {
        *counts.entry(category).or_insert(0usize) += 1;
    }
    if counts.is_empty() {
        return None;
    }
    Some(
        counts
            .into_iter()
            .map(|(category, count)| format!("{category} {count}"))
            .collect::<Vec<_>>()
            .join(", "),
    )
}
