use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
use crate::output::report_model::{
    ImageEfficiencyPresentation, OversizedImageRow, RobotsPresentation, SeoPresentation,
    SeoProfilePresentation, SignalDetails,
};
use crate::taxonomy::SeverityExt;
use fluent_bundle::FluentValue;

use super::super::super::helpers::{truncate_list, truncate_url_list, yes_no};
use super::super::super::modules::build_tracking_summary_text;
use super::super::serp::{build_page_health_presentation, build_serp_presentation};
use super::normalized_module_score;
use crate::seo::interpretation::{
    page_profile_optimization_note_text, page_profile_summary_text, seo_interpretation_text,
};
use crate::seo::{ImageEfficiencyAnalysis, RobotsAudit, SeoAnalysis};
use crate::util::truncate_url;

#[cfg(test)]
mod text_export;
#[cfg(test)]
mod text_guard;

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
            meta_tags.push((i18n.t("seo-details-meta-title"), title.clone()));
        }
        if let Some(ref desc) = s.meta.description {
            meta_tags.push((i18n.t("pdf-seo-meta-description"), desc.clone()));
        }
        if let Some(ref viewport) = s.meta.viewport {
            meta_tags.push(("Viewport".to_string(), viewport.clone()));
        }

        let meta_issues: Vec<(String, crate::wcag::Severity, String)> = s
            .meta_issues
            .iter()
            .map(|i| {
                // A missing lang or page title is also an accessibility
                // finding (WCAG 3.1.1 / 2.4.2) and costs points there too —
                // two perspectives, stated so the overlap is visible (plan 105).
                let base = i.localized_message(en);
                let criterion = match (i.field.as_str(), i.severity) {
                    ("lang", _) => Some("3.1.1"),
                    ("title", crate::wcag::Severity::High) => Some("2.4.2"),
                    _ => None,
                };
                let message = match criterion {
                    Some(c) => i18n.t_args(
                        "seo-details-meta-also-accessibility",
                        &[("message", base), ("criterion", c.to_string())],
                    ),
                    // Charged by Mobile and Accessibility, listed here only.
                    None if i.field == "viewport" => {
                        i18n.t_args("seo-details-meta-viewport-elsewhere", &[("message", base)])
                    }
                    None => base,
                };
                (i.field.clone(), i.severity, message)
            })
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
                            label
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
                    Some(fit.fit_text(en)),
                    fit.fit_status == SchemaFitStatus::Matched,
                    fit.fit_status == SchemaFitStatus::Mismatch
                        || fit.coverage_status == SchemaCoverageStatus::Opportunity,
                    vec![
                        (
                            i18n.t("pdf-seo-schema-fit-page-type"),
                            fit.page_kind.label(en),
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
                        assessment.status_text(en),
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
                        SchemaExtracted::FAQPage { question_count, .. } => i18n.t_args(
                            "seo-details-schema-questions",
                            &[("count", *question_count as i64)],
                        ),
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
                        } => {
                            let name = name.clone().unwrap_or_default();
                            if *has_search_action {
                                i18n.t_args("seo-details-schema-with-search", &[("name", name)])
                            } else {
                                name
                            }
                        }
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
                        } => {
                            let base = format!(
                                "{}{}",
                                name.as_deref().unwrap_or(""),
                                address
                                    .as_ref()
                                    .map(|a| format!(" — {}", a))
                                    .unwrap_or_default()
                            );
                            if *area_served_count > 0 {
                                i18n.t_args(
                                    "seo-details-schema-regions",
                                    &[
                                        ("base", FluentValue::from(base)),
                                        ("count", FluentValue::from(*area_served_count as i64)),
                                    ],
                                )
                            } else {
                                base
                            }
                        }
                        SchemaExtracted::BreadcrumbList { item_count } => i18n.t_args(
                            "seo-details-schema-levels",
                            &[("count", *item_count as i64)],
                        ),
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
                    let rating = match cat.pct() {
                        90..=100 => "seo-details-signal-excellent",
                        67..=89 => "seo-details-signal-good",
                        34..=66 => "seo-details-signal-partial",
                        1..=33 => "seo-details-signal-minimal",
                        _ => "seo-details-signal-missing",
                    };
                    (
                        cat.name.clone(),
                        // Counts, not a percentage: the denominator is the
                        // point (plan 29, D4).
                        format!("{} / {}", cat.passed, cat.total),
                        i18n.t(rating),
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
                        i18n.t("seo-details-identity-page-title"),
                        cp.content_identity
                            .site_name
                            .clone()
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        i18n.t("pdf-seo-profile-content-type"),
                        cp.content_identity.content_type.clone(),
                    ),
                    (
                        i18n.t("pdf-seo-profile-language"),
                        cp.content_identity
                            .language
                            .clone()
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        i18n.t("seo-details-identity-topic-hints"),
                        if cp.content_identity.category_hints.is_empty() {
                            i18n.t("seo-details-identity-no-topic-hints")
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
                        i18n.t("seo-details-profile-page-type"),
                        cp.page_classification.primary_type.label(en).to_string(),
                    ),
                    (
                        i18n.t("seo-details-profile-characteristics"),
                        if cp.page_classification.attributes.is_empty() {
                            i18n.t("seo-details-profile-no-characteristics")
                        } else {
                            format!("{}.", cp.page_classification.attributes.join(", "))
                        },
                    ),
                    (
                        i18n.t("seo-details-profile-classification"),
                        page_profile_summary_text(cp, en),
                    ),
                    (
                        i18n.t("seo-details-profile-recommendation"),
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
            heading_summary: heading_summary(s, i18n),
            social_summary: social_summary(s, i18n),
            technical_summary: technical_summary_rows(s, i18n),
            tracking_summary: tracking_summary_rows(normalized, s, i18n),
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

fn heading_summary(s: &SeoAnalysis, i18n: &I18n) -> String {
    i18n.t_args(
        "seo-details-heading-summary",
        &[
            ("h1", s.headings.h1_count as i64),
            ("total", s.headings.total_count as i64),
            ("issues", s.headings.issues.len() as i64),
        ],
    )
}

fn social_summary(s: &SeoAnalysis, i18n: &I18n) -> String {
    let presence = |present: bool| {
        i18n.t(if present {
            "seo-profile-detail-present"
        } else {
            "seo-profile-detail-missing"
        })
    };
    i18n.t_args(
        "seo-details-social-summary",
        &[
            (
                "og",
                FluentValue::from(presence(s.social.open_graph.is_some())),
            ),
            (
                "twitter",
                FluentValue::from(presence(s.social.twitter_card.is_some())),
            ),
            (
                "completeness",
                FluentValue::from(i64::from(s.social.completeness)),
            ),
        ],
    )
}

fn technical_summary_rows(s: &SeoAnalysis, i18n: &I18n) -> Vec<(String, String)> {
    let locale = i18n.locale();
    vec![
        ("HTTPS".to_string(), yes_no(locale, s.technical.https)),
        (
            "Canonical".to_string(),
            yes_no(locale, s.technical.has_canonical),
        ),
        (
            i18n.t("seo-details-language-tag"),
            yes_no(locale, s.technical.has_lang),
        ),
        (
            i18n.t("seo-details-word-count"),
            s.technical.word_count.to_string(),
        ),
        (
            i18n.t("seo-details-internal-links"),
            s.technical.internal_links.to_string(),
        ),
        (
            i18n.t("seo-details-external-links"),
            s.technical.external_links.to_string(),
        ),
        (
            i18n.t("seo-details-dofollow-links"),
            s.technical.dofollow_links.to_string(),
        ),
        (
            i18n.t("seo-details-nofollow-links"),
            s.technical.nofollow_links.to_string(),
        ),
        (
            "AMP".to_string(),
            if !s.technical.amp.detected {
                i18n.t("seo-details-not-detected")
            } else if s.technical.amp.issues.is_empty() {
                i18n.t("seo-details-amp-complete")
            } else {
                i18n.t_args(
                    "seo-details-amp-issues",
                    &[("count", s.technical.amp.issues.len() as i64)],
                )
            },
        ),
    ]
}

fn tracking_summary_rows(
    normalized: &AuditContext<'_>,
    s: &SeoAnalysis,
    i18n: &I18n,
) -> Vec<(String, String)> {
    let categories = |summary: String| {
        format!(
            "; {}",
            i18n.t_args("seo-details-categories", &[("summary", summary)])
        )
    };
    vec![
        (
            i18n.t("seo-details-google-fonts"),
            if s.technical.uses_remote_google_fonts {
                i18n.t_args(
                    "seo-details-google-fonts-used",
                    &[(
                        "sources",
                        truncate_url_list(&s.technical.google_fonts_sources, 2, 48),
                    )],
                )
            } else {
                i18n.t("no")
            },
        ),
        (
            i18n.t("seo-details-tracking-cookies"),
            if s.technical.tracking_cookies.is_empty() {
                i18n.t("seo-details-none-detected")
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
            i18n.t("seo-details-cookie-inventory"),
            match s.technical.cookie_inventory.as_deref() {
                None => i18n.t("seo-details-cookie-inventory-not-collected"),
                Some([]) => i18n.t("seo-details-no-cookies"),
                Some(cookie_inventory) => {
                    let risky = cookie_inventory
                        .iter()
                        .filter(|cookie| !cookie.secure || cookie.same_site.is_none())
                        .count();
                    let category_suffix = tracking_category_summary(
                        cookie_inventory
                            .iter()
                            .filter_map(|cookie| cookie.category.as_deref()),
                    )
                    .map(categories)
                    .unwrap_or_default();
                    let summary = i18n.count("seo-cookie-inventory", cookie_inventory.len());
                    let risky_text = i18n.count("seo-cookie-inventory-risky", risky);
                    format!("{summary}, {risky_text}{category_suffix}")
                }
            },
        ),
        (
            "Browser Storage".to_string(),
            if s.technical.storage_items.is_empty() {
                i18n.t("seo-details-no-storage-keys")
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
                    .map(categories)
                    .unwrap_or_default()
                )
            },
        ),
        (
            i18n.t("seo-details-tracking-signals"),
            if s.technical.tracking_signals.is_empty() {
                i18n.t("seo-details-none-detected")
            } else {
                truncate_list(&s.technical.tracking_signals, 3)
            },
        ),
        (
            i18n.t("seo-details-consent-cookies"),
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
                    .map(categories)
                    .unwrap_or_default();
                    let summary = i18n.t_args(
                        "seo-details-consent-summary",
                        &[
                            ("before", snapshot.before_interaction.len() as i64),
                            ("after", snapshot.after_interaction.len() as i64),
                            ("added", snapshot.added_after_interaction.len() as i64),
                        ],
                    );
                    format!("{summary}{category_suffix}")
                })
                .unwrap_or_else(|| i18n.t("seo-details-not-collected")),
        ),
        (
            "Zaraz".to_string(),
            if s.technical.zaraz.detected {
                i18n.t_args(
                    "seo-details-zaraz-detected",
                    &[("signals", truncate_list(&s.technical.zaraz.signals, 2))],
                )
            } else {
                i18n.t("seo-details-not-detected")
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
                g.bot_class.label(en),
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
