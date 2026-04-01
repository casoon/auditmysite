//! Report builder — transforms raw audit data into ViewModels
//!
//! This module takes raw AuditReport / BatchReport data and produces
//! structured ViewModels with grouped findings, aggregated statistics,
//! and pre-computed presentation data. The renderer does zero data transformation.

use std::collections::HashMap;

use crate::audit::normalized::NormalizedReport;
use crate::audit::BatchReport;
use crate::cli::ReportLevel;
use crate::output::explanations::get_explanation;
use crate::output::report_model::*;
use crate::seo::profile::PageType;
use crate::taxonomy::RuleLookup;
use crate::util::truncate_url;
use crate::wcag::Severity;

const NBSP: &str = "\u{00A0}";

// ─── Single Report ViewModel ────────────────────────────────────────────────

/// Build a complete ViewModel from a normalized report (single source of truth for score/grade/certificate)
pub fn build_view_model(normalized: &NormalizedReport, config: &ReportConfig) -> ReportViewModel {
    let priority_by_rule: HashMap<&str, f32> = normalized
        .findings
        .iter()
        .map(|f| (f.rule_id.as_str(), f.priority_score))
        .collect();

    // Convert NormalizedFindings → FindingGroups, filtering by report visibility
    let mut sorted_groups: Vec<FindingGroup> = normalized
        .findings
        .iter()
        .filter(|f| match config.level {
            ReportLevel::Executive => f.report_visibility.executive,
            ReportLevel::Standard => f.report_visibility.standard,
            ReportLevel::Technical => f.report_visibility.technical,
        })
        .map(finding_group_from_normalized)
        .collect();
    sorted_groups.sort_by(|a, b| {
        let pa = priority_by_rule
            .get(a.rule_id.as_str())
            .copied()
            .unwrap_or(0.0);
        let pb = priority_by_rule
            .get(b.rule_id.as_str())
            .copied()
            .unwrap_or(0.0);
        pb.partial_cmp(&pa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| impact_score(b).cmp(&impact_score(a)))
    });

    let score = normalized.score;
    let grade = normalized.grade.clone();
    let certificate = normalized.certificate.clone();
    let date = normalized.timestamp.format("%d.%m.%Y").to_string();
    let report_title = localized_report_title(&config.locale);
    let report_subtitle = localized_report_subtitle(&config.locale);
    let report_author = config
        .company_name
        .as_deref()
        .unwrap_or("AuditMySite")
        .to_string();
    let has_quality_modules = normalized.module_scores.len() > 1;

    let top_findings: Vec<FindingGroup> = sorted_groups.iter().take(5).cloned().collect();
    let positive_aspects = derive_positive_aspects_from_normalized(normalized);
    let action_plan = derive_action_plan(&sorted_groups);

    // Build module list
    let mut module_names: Vec<String> = vec!["Accessibility".into()];
    if normalized.raw_performance.is_some() {
        module_names.push("Performance".into());
    }
    if normalized.raw_seo.is_some() {
        module_names.push("SEO".into());
    }
    if normalized.raw_security.is_some() {
        module_names.push("Sicherheit".into());
    }
    if normalized.raw_mobile.is_some() {
        module_names.push("Mobile".into());
    }

    // Build severity block from normalized counts
    let severity = SeverityBlock {
        critical: normalized.severity_counts.critical as u32,
        high: normalized.severity_counts.high as u32,
        medium: normalized.severity_counts.medium as u32,
        low: normalized.severity_counts.low as u32,
        total: normalized.severity_counts.total as u32,
        has_issues: normalized.severity_counts.total > 0,
    };

    // Build modules block
    let modules = build_modules_block_from_normalized(normalized);

    // Build summary metrics
    let quick_win_count = action_plan.quick_wins.len();
    let critical_count =
        (normalized.severity_counts.critical + normalized.severity_counts.high) as u32;
    let total_violations = normalized.severity_counts.total as u32;
    let nodes_analyzed = normalized.nodes_analyzed;

    // Build actions block (pre-mapped for ActionRoadmap component)
    let actions = build_actions_block(&action_plan);

    // Build module details from raw data
    let module_details = build_module_details_from_normalized(normalized);
    let history = config
        .history_preview
        .as_ref()
        .map(build_history_trend_block);

    ReportViewModel {
        meta: MetaBlock {
            title: report_title.clone(),
            subtitle: normalized.url.clone(),
            date: date.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: report_author.clone(),
            report_level: config.level,
            score_label: format!("{}/100", score),
        },
        cover: CoverBlock {
            brand: report_author,
            title: report_title,
            domain: normalized.url.clone(),
            subtitle: report_subtitle.to_string(),
            date: date.clone(),
            score,
            grade: grade.clone(),
            certificate: certificate.clone(),
            total_issues: total_violations,
            critical_issues: critical_count,
            modules: module_names,
        },
        summary: SummaryBlock {
            score,
            grade: grade.clone(),
            certificate: certificate.clone(),
            domain: normalized.url.clone(),
            date: date.clone(),
            verdict: build_verdict_text(&normalized.url, score as f32),
            metrics: vec![
                MetricItem {
                    title: format!("Verstöße{NBSP}gesamt"),
                    value: total_violations.to_string(),
                    accent_color: None,
                },
                MetricItem {
                    title: "Kritisch".into(),
                    value: critical_count.to_string(),
                    accent_color: Some("#ef4444".into()),
                },
                MetricItem {
                    title: if has_quality_modules {
                        format!("Gesamtscore{NBSP}Website")
                    } else {
                        format!("Geprüfte{NBSP}Knoten")
                    },
                    value: if has_quality_modules {
                        format!("{}/100", normalized.overall_score)
                    } else {
                        nodes_analyzed.to_string()
                    },
                    accent_color: Some("#22c55e".into()),
                },
                MetricItem {
                    title: if has_quality_modules {
                        format!("Geprüfte{NBSP}Knoten")
                    } else {
                        "Quick Wins".into()
                    },
                    value: if has_quality_modules {
                        nodes_analyzed.to_string()
                    } else {
                        quick_win_count.to_string()
                    },
                    accent_color: Some("#2563eb".into()),
                },
                MetricItem {
                    title: if has_quality_modules {
                        "Quick Wins".into()
                    } else {
                        "WCAG-Level".into()
                    },
                    value: if has_quality_modules {
                        quick_win_count.to_string()
                    } else {
                        normalized.wcag_level.to_string()
                    },
                    accent_color: Some("#7c3aed".into()),
                },
            ],
            top_actions: top_findings
                .iter()
                .take(3)
                .map(|f| f.recommendation.clone())
                .collect(),
            positive_aspects: positive_aspects
                .iter()
                .map(|a| format!("{}: {}", a.area, a.description))
                .collect(),
        },
        history,
        methodology: build_methodology(normalized),
        modules,
        severity,
        findings: FindingsBlock {
            top_findings,
            all_findings: sorted_groups,
        },
        module_details,
        actions,
        appendix: build_appendix_block_from_normalized(normalized),
    }
}

fn build_history_trend_block(preview: &ReportHistoryPreview) -> HistoryTrendBlock {
    let trend_interpretation = if preview.delta_accessibility > 0 {
        "Die Barrierefreiheit hat sich gegenüber dem letzten Lauf verbessert."
    } else if preview.delta_accessibility < 0 {
        "Die Barrierefreiheit ist gegenüber dem letzten Lauf leicht zurückgegangen."
    } else {
        "Die Barrierefreiheit ist gegenüber dem letzten Lauf stabil geblieben."
    };

    HistoryTrendBlock {
        previous_date: preview.previous_date.clone(),
        timeline_entries: preview.timeline_entries,
        summary: format!(
            "Vergleich zum letzten verfügbaren Lauf vom {}. {} Die Historie umfasst aktuell {} verwertbare Snapshots im Report-Ordner.",
            preview.previous_date, trend_interpretation, preview.timeline_entries
        ),
        metrics: vec![
            (
                "Accessibility-Delta".to_string(),
                format!("{:+}", preview.delta_accessibility),
            ),
            (
                "Gesamt-Delta".to_string(),
                format!("{:+}", preview.delta_overall),
            ),
            (
                "Issue-Delta".to_string(),
                format!("{:+}", preview.delta_total_issues),
            ),
            (
                "Kritisch+Hoch-Delta".to_string(),
                format!("{:+}", preview.delta_critical_issues),
            ),
            (
                "Vorher Accessibility".to_string(),
                preview.previous_accessibility_score.to_string(),
            ),
            (
                "Vorher Gesamt".to_string(),
                preview.previous_overall_score.to_string(),
            ),
        ],
        timeline_rows: preview
            .recent_entries
            .iter()
            .map(|entry| {
                (
                    entry.0.clone(),
                    entry.1.to_string(),
                    entry.2.to_string(),
                    entry.3.clone(),
                    entry.4.to_string(),
                )
            })
            .collect(),
        new_findings: preview.new_findings.clone(),
        resolved_findings: preview.resolved_findings.clone(),
    }
}

// ─── Block Builders ─────────────────────────────────────────────────────────

fn build_modules_block_from_normalized(normalized: &NormalizedReport) -> ModulesBlock {
    let a11y_score = normalized.score as f32;
    let mut dashboard = vec![ModuleScore {
        name: "Barrierefreiheit".into(),
        score: a11y_score.round() as u32,
        interpretation: interpret_score(a11y_score, "Barrierefreiheit"),
        key_lever: derive_accessibility_lever(normalized),
        good_threshold: 75,
        warn_threshold: 50,
    }];

    if let Some(ref p) = normalized.raw_performance {
        dashboard.push(ModuleScore {
            name: "Performance".into(),
            score: p.score.overall,
            interpretation: interpret_score(p.score.overall as f32, "Performance"),
            key_lever: derive_performance_lever(p),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref s) = normalized.raw_seo {
        dashboard.push(ModuleScore {
            name: "SEO".into(),
            score: s.score,
            interpretation: interpret_score(s.score as f32, "SEO"),
            key_lever: derive_seo_lever(s),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref s) = normalized.raw_security {
        dashboard.push(ModuleScore {
            name: "Sicherheit".into(),
            score: s.score,
            interpretation: interpret_score(s.score as f32, "Sicherheit"),
            key_lever: derive_security_lever(s),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref m) = normalized.raw_mobile {
        dashboard.push(ModuleScore {
            name: "Mobile".into(),
            score: m.score,
            interpretation: interpret_score(m.score as f32, "mobile Nutzbarkeit"),
            key_lever: derive_mobile_lever(m),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }

    let has_multiple = dashboard.len() > 1;
    let overall_score = if has_multiple {
        Some(normalized.overall_score)
    } else {
        None
    };
    let overall_interpretation = overall_score.map(|_| {
        "Gewichteter Durchschnitt aller aktiven Module. Accessibility fließt mit 40% ein, \
         Performance und SEO mit je 20%, Sicherheit und Mobile mit je 10%."
            .to_string()
    });

    ModulesBlock {
        dashboard,
        overall_score,
        overall_interpretation,
    }
}

fn build_actions_block(plan: &ActionPlan) -> ActionsBlock {
    let map_items = |items: &[ActionItem], effort: &str| -> Vec<RoadmapItemData> {
        items
            .iter()
            .map(|i| RoadmapItemData {
                action: i.action.clone(),
                role: i.role.label().to_string(),
                priority: i.priority.label().to_string(),
                execution_priority: i.execution_priority.label().to_string(),
                effort: effort.to_string(),
                benefit: i.benefit.clone(),
            })
            .collect()
    };

    let mut columns = Vec::new();
    if !plan.quick_wins.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Sofort (0-2 Wochen)".into(),
            accent_color: "#22c55e".into(),
            items: map_items(&plan.quick_wins, "Niedrig"),
        });
    }
    if !plan.medium_term.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Mittelfristig (2-6 Wochen)".into(),
            accent_color: "#f59e0b".into(),
            items: map_items(&plan.medium_term, "Mittel"),
        });
    }
    if !plan.structural.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Langfristig".into(),
            accent_color: "#2563eb".into(),
            items: map_items(&plan.structural, "Hoch"),
        });
    }

    ActionsBlock {
        roadmap_columns: columns,
        role_assignments: plan.role_assignments.clone(),
        intro_text: "Die Maßnahmen sind nach Zeithorizont, Aufwand und Wirkung geordnet. So lässt sich die Umsetzung schrittweise planen."
            .to_string(),
    }
}

fn build_appendix_block_from_normalized(normalized: &NormalizedReport) -> AppendixBlock {
    // Build appendix from normalized findings (already suppressed/filtered)
    let violations: Vec<AppendixViolation> = normalized
        .findings
        .iter()
        .map(|f| {
            let affected_elements: Vec<AffectedElement> = f
                .occurrences
                .iter()
                .map(|occ| AffectedElement {
                    selector: occ.selector.clone().unwrap_or_else(|| occ.node_id.clone()),
                    node_id: occ.node_id.clone(),
                })
                .collect();

            AppendixViolation {
                rule: f.wcag_criterion.clone(),
                rule_name: f.title.clone(),
                severity: f.severity,
                message: f.description.clone(),
                fix_suggestion: f.occurrences.first().and_then(|o| o.fix_suggestion.clone()),
                affected_elements,
            }
        })
        .collect();

    let has_violations = !violations.is_empty();

    AppendixBlock {
        violations,
        score_methodology: "Score-Berechnung: Basis 100 Punkte. Abzug auf Basis der Taxonomie-Regel-Definitionen \
            mit regelspezifischen Penalties und logarithmischer Skalierung für wiederholte Verstöße. \
            Korrektur bei supprimierten Regeln (z.B. 3.1.1 bei vorhandener Sprachangabe).".to_string(),
        has_violations,
    }
}

fn build_methodology(normalized: &NormalizedReport) -> MethodologyBlock {
    let active_modules = normalized
        .module_scores
        .iter()
        .map(|m| m.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    MethodologyBlock {
        scope: format!(
            "Automatisierte Prüfung der Seite {} auf Barrierefreiheit nach WCAG 2.1 (Level {}). \
             Zusätzlich wurden Performance, SEO, Sicherheit und mobile Nutzbarkeit analysiert.",
            normalized.url, normalized.wcag_level
        ),
        method: "Die Prüfung erfolgte über den Chrome DevTools Protocol (CDP) und den \
                 nativen Accessibility Tree des Browsers. 21 WCAG-Regeln wurden automatisiert \
                 gegen den Seiteninhalt geprüft."
            .to_string(),
        limitations:
            "Automatisierte Tests können ca. 30–40% aller Barrierefreiheitsprobleme erkennen. \
                      Komplexe Aspekte wie korrekte Tab-Reihenfolge, sinnvolle Alt-Texte oder \
                      verständliche Sprache erfordern zusätzlich manuelle Prüfung."
                .to_string(),
        disclaimer: "Dieser Report stellt eine automatisierte technische Analyse dar. \
                     Er ersetzt keine vollständige Konformitätsbewertung nach WCAG 2.1. \
                     Für eine rechtsverbindliche Aussage zur Barrierefreiheit ist eine \
                     umfassende manuelle Prüfung durch Experten erforderlich."
            .to_string(),
        audit_facts: vec![
            (
                "Primärscore".to_string(),
                format!("Accessibility {} / 100", normalized.score),
            ),
            (
                "Gesamtscore".to_string(),
                format!(
                    "{} / 100 (gewichteter Mix aktiver Module)",
                    normalized.overall_score
                ),
            ),
            ("WCAG-Level".to_string(), normalized.wcag_level.to_string()),
            (
                "Geprüfte Knoten".to_string(),
                normalized.nodes_analyzed.to_string(),
            ),
            (
                "Laufzeit".to_string(),
                format!("{:.1} s", normalized.duration_ms as f64 / 1000.0),
            ),
            ("Aktive Module".to_string(), active_modules),
        ],
    }
}

fn build_module_details_from_normalized(normalized: &NormalizedReport) -> ModuleDetailsBlock {
    let performance = normalized.raw_performance.as_ref().map(|p| {
        let mut vitals = Vec::new();
        if let Some(ref lcp) = p.vitals.lcp {
            vitals.push((
                "Largest Contentful Paint (LCP)".to_string(),
                format!("{:.0}ms", lcp.value),
                lcp.rating.clone(),
            ));
        }
        if let Some(ref fcp) = p.vitals.fcp {
            vitals.push((
                "First Contentful Paint (FCP)".to_string(),
                format!("{:.0}ms", fcp.value),
                fcp.rating.clone(),
            ));
        }
        if let Some(ref cls) = p.vitals.cls {
            vitals.push((
                "Cumulative Layout Shift (CLS)".to_string(),
                format!("{:.3}", cls.value),
                cls.rating.clone(),
            ));
        }
        if let Some(ref ttfb) = p.vitals.ttfb {
            vitals.push((
                "Time to First Byte (TTFB)".to_string(),
                format!("{:.0}ms", ttfb.value),
                ttfb.rating.clone(),
            ));
        }
        if let Some(ref inp) = p.vitals.inp {
            vitals.push((
                "Interaction to Next Paint (INP)".to_string(),
                format!("{:.0}ms", inp.value),
                inp.rating.clone(),
            ));
        }
        if let Some(ref tbt) = p.vitals.tbt {
            vitals.push((
                "Total Blocking Time (TBT)".to_string(),
                format!("{:.0}ms", tbt.value),
                tbt.rating.clone(),
            ));
        }

        let mut additional = Vec::new();
        if let Some(nodes) = p.vitals.dom_nodes {
            additional.push(("DOM-Knoten".to_string(), nodes.to_string()));
        }
        if let Some(heap) = p.vitals.js_heap_size {
            additional.push((
                "JS Heap".to_string(),
                format!("{:.1} MB", heap as f64 / 1_048_576.0),
            ));
        }
        if let Some(load) = p.vitals.load_time {
            additional.push(("Ladezeit".to_string(), format!("{:.0}ms", load)));
        }
        if let Some(dcl) = p.vitals.dom_content_loaded {
            additional.push(("DOM Content Loaded".to_string(), format!("{:.0}ms", dcl)));
        }

        PerformancePresentation {
            score: p.score.overall,
            grade: p.score.grade.label().to_string(),
            interpretation: interpret_score(p.score.overall as f32, "Performance"),
            vitals,
            additional_metrics: additional,
        }
    });

    let seo = normalized.raw_seo.as_ref().map(|s| {
        let mut meta_tags = Vec::new();
        if let Some(ref title) = s.meta.title {
            meta_tags.push(("Titel".to_string(), title.clone()));
        }
        if let Some(ref desc) = s.meta.description {
            meta_tags.push(("Beschreibung".to_string(), desc.clone()));
        }
        if let Some(ref viewport) = s.meta.viewport {
            meta_tags.push(("Viewport".to_string(), viewport.clone()));
        }

        let meta_issues: Vec<(String, Severity, String)> = s
            .meta_issues
            .iter()
            .map(|i| (i.field.clone(), i.severity, i.message.clone()))
            .collect();

        let profile = s.content_profile.as_ref().map(|cp| {
            use crate::seo::profile::SchemaExtracted;

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
                            format!("{} Fragen", question_count)
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
                            if *has_search_action { " (Suche)" } else { "" }
                        ),
                        SchemaExtracted::BreadcrumbList { item_count } => {
                            format!("{} Ebenen", item_count)
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
                    let rating = match cat.score_pct {
                        90..=100 => "Sehr gut",
                        67..=89 => "Gut",
                        34..=66 => "Teilweise",
                        1..=33 => "Minimal",
                        _ => "Fehlt",
                    };
                    (
                        cat.name.clone(),
                        format!("{}%", cat.score_pct),
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
                page_type: cp.page_classification.primary_type.label().to_string(),
                page_attributes: cp.page_classification.attributes.clone(),
                content_depth_score: cp.page_classification.content_depth_score,
                structural_richness_score: cp.page_classification.structural_richness_score,
                media_text_balance_score: cp.page_classification.media_text_balance_score,
                intent_fit_score: cp.page_classification.intent_fit_score,
                page_profile_summary: summarize_page_profile(cp),
                optimization_note: page_profile_optimization_note(cp),
                schema_rows,
                schema_count: cp.schema_inventory.total_count,
                signal_rows,
                signal_overall_pct: cp.signal_strength.overall_pct,
                signal_details,
                maturity_level: cp.maturity.label().to_string(),
                maturity_description: cp.maturity.description().to_string(),
                maturity_techniques_used: cp.maturity_techniques,
                maturity_techniques_total: 13,
            }
        });

        SeoPresentation {
            score: s.score,
            interpretation: interpret_score(s.score as f32, "SEO"),
            meta_tags,
            meta_issues,
            heading_summary: format!(
                "{} H1-Überschrift(en), {} Überschriften gesamt, {} Probleme",
                s.headings.h1_count,
                s.headings.total_count,
                s.headings.issues.len()
            ),
            social_summary: format!(
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
            ),
            technical_summary: vec![
                ("HTTPS".to_string(), yes_no(s.technical.https)),
                ("Canonical".to_string(), yes_no(s.technical.has_canonical)),
                ("Sprachangabe".to_string(), yes_no(s.technical.has_lang)),
                ("Wortanzahl".to_string(), s.technical.word_count.to_string()),
            ],
            profile,
        }
    });

    let security = normalized.raw_security.as_ref().map(|sec| {
        let header_checks: Vec<(&str, &Option<String>)> = vec![
            (
                "Content-Security-Policy",
                &sec.headers.content_security_policy,
            ),
            (
                "Strict-Transport-Security",
                &sec.headers.strict_transport_security,
            ),
            (
                "X-Content-Type-Options",
                &sec.headers.x_content_type_options,
            ),
            ("X-Frame-Options", &sec.headers.x_frame_options),
            ("X-XSS-Protection", &sec.headers.x_xss_protection),
            ("Referrer-Policy", &sec.headers.referrer_policy),
            ("Permissions-Policy", &sec.headers.permissions_policy),
            (
                "Cross-Origin-Opener-Policy",
                &sec.headers.cross_origin_opener_policy,
            ),
            (
                "Cross-Origin-Resource-Policy",
                &sec.headers.cross_origin_resource_policy,
            ),
        ];

        SecurityPresentation {
            score: sec.score,
            grade: sec.grade.clone(),
            interpretation: interpret_score(sec.score as f32, "Sicherheit"),
            headers: header_checks
                .iter()
                .map(|(name, value)| {
                    let (status, val) = match value {
                        Some(v) => ("Vorhanden".to_string(), truncate_url(v, 50)),
                        None => ("Fehlt".to_string(), "—".to_string()),
                    };
                    (name.to_string(), status, val)
                })
                .collect(),
            ssl_info: vec![
                ("HTTPS".to_string(), yes_no(sec.ssl.https)),
                (
                    "Gültiges Zertifikat".to_string(),
                    yes_no(sec.ssl.valid_certificate),
                ),
                ("HSTS".to_string(), yes_no(sec.ssl.has_hsts)),
                (
                    "HSTS Max-Age".to_string(),
                    sec.ssl
                        .hsts_max_age
                        .map(|v| format!("{}s", v))
                        .unwrap_or_else(|| "—".to_string()),
                ),
                (
                    "Subdomains".to_string(),
                    yes_no(sec.ssl.hsts_include_subdomains),
                ),
                ("Preload".to_string(), yes_no(sec.ssl.hsts_preload)),
            ],
            issues: sec
                .issues
                .iter()
                .map(|i| (i.header.clone(), i.severity, i.message.clone()))
                .collect(),
            recommendations: sec.recommendations.clone(),
        }
    });

    let mobile = normalized.raw_mobile.as_ref().map(|m| MobilePresentation {
        score: m.score,
        interpretation: interpret_score(m.score as f32, "mobile Nutzbarkeit"),
        viewport: vec![
            ("Viewport-Tag".to_string(), yes_no(m.viewport.has_viewport)),
            (
                "device-width".to_string(),
                yes_no(m.viewport.uses_device_width),
            ),
            (
                "Initial Scale".to_string(),
                yes_no(m.viewport.has_initial_scale),
            ),
            ("Skalierbar".to_string(), yes_no(m.viewport.is_scalable)),
            (
                "Korrekt konfiguriert".to_string(),
                yes_no(m.viewport.is_properly_configured),
            ),
        ],
        touch_targets: vec![
            (
                "Gesamt".to_string(),
                m.touch_targets.total_targets.to_string(),
            ),
            (
                "Ausreichend (≥44px)".to_string(),
                m.touch_targets.adequate_targets.to_string(),
            ),
            (
                "Zu klein".to_string(),
                m.touch_targets.small_targets.to_string(),
            ),
            (
                "Zu eng beieinander".to_string(),
                m.touch_targets.crowded_targets.to_string(),
            ),
        ],
        font_analysis: vec![
            (
                "Basis-Schriftgröße".to_string(),
                format!("{:.0}px", m.font_sizes.base_font_size),
            ),
            (
                "Kleinste Schrift".to_string(),
                format!("{:.0}px", m.font_sizes.smallest_font_size),
            ),
            (
                "Lesbarer Text".to_string(),
                format!("{:.0}%", m.font_sizes.legible_percentage),
            ),
            (
                "Relative Einheiten".to_string(),
                yes_no(m.font_sizes.uses_relative_units),
            ),
        ],
        content_sizing: vec![
            (
                "Passt in Viewport".to_string(),
                yes_no(m.content_sizing.fits_viewport),
            ),
            (
                "Kein hor. Scrollen".to_string(),
                yes_no(!m.content_sizing.has_horizontal_scroll),
            ),
            (
                "Responsive Bilder".to_string(),
                yes_no(m.content_sizing.uses_responsive_images),
            ),
            (
                "Media Queries".to_string(),
                yes_no(m.content_sizing.uses_media_queries),
            ),
        ],
        issues: m
            .issues
            .iter()
            .map(|i| (i.category.clone(), i.severity, i.message.clone()))
            .collect(),
    });

    let has_any = performance.is_some() || seo.is_some() || security.is_some() || mobile.is_some();
    ModuleDetailsBlock {
        performance,
        seo,
        security,
        mobile,
        has_any,
    }
}

/// Convert a NormalizedFinding into a FindingGroup (with explanation enrichment)
fn finding_group_from_normalized(f: &crate::audit::normalized::NormalizedFinding) -> FindingGroup {
    let explanation = get_explanation(&f.wcag_criterion);

    let (
        title,
        customer_desc,
        user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        role,
        effort,
        execution_priority,
    ) = if let Some(expl) = explanation {
        (
            expl.customer_title.to_string(),
            expl.customer_description.to_string(),
            expl.user_impact.to_string(),
            derive_business_impact(expl.user_impact, f.dimension.as_str(), f.severity),
            expl.typical_cause.to_string(),
            expl.recommendation.to_string(),
            expl.technical_note.to_string(),
            expl.responsible_role,
            expl.effort_estimate,
            derive_execution_priority(f.severity, expl.effort_estimate, f.dimension.as_str()),
        )
    } else {
        (
            f.title.clone(),
            f.description.clone(),
            f.user_impact.clone(),
            derive_business_impact(&f.user_impact, f.dimension.as_str(), f.severity),
            "Automatisch erkanntes Problem.".to_string(),
            f.occurrences
                .first()
                .and_then(|o| o.fix_suggestion.clone())
                .unwrap_or_else(|| "Bitte prüfen und beheben.".to_string()),
            String::new(),
            Role::Development,
            Effort::Medium,
            derive_execution_priority(f.severity, Effort::Medium, f.dimension.as_str()),
        )
    };

    let examples = explanation.map(|e| e.examples()).unwrap_or_default();

    FindingGroup {
        title,
        rule_id: f.rule_id.clone(),
        wcag_criterion: f.wcag_criterion.clone(),
        wcag_level: f.wcag_level.clone(),
        dimension: Some(f.dimension.clone()),
        subcategory: Some(f.subcategory.clone()),
        issue_class: Some(f.issue_class.clone()),
        severity: f.severity,
        priority: severity_to_priority(f.severity),
        customer_description: customer_desc,
        user_impact: user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        occurrence_count: f.occurrence_count,
        affected_urls: Vec::new(),
        affected_elements: f.occurrence_count,
        responsible_role: role,
        effort,
        execution_priority,
        examples,
    }
}

fn derive_positive_aspects_from_normalized(normalized: &NormalizedReport) -> Vec<PositiveAspect> {
    let mut positives = Vec::new();
    let a11y_score = normalized.score as f32;

    if normalized.findings.is_empty() {
        positives.push(PositiveAspect {
            area: "Barrierefreiheit".into(),
            description: "Keine automatisch erkennbaren Verstöße gefunden.".into(),
        });
    } else if a11y_score >= 80.0 {
        positives.push(PositiveAspect {
            area: "Barrierefreiheit".into(),
            description: "Solide Grundqualität mit gezielt priorisierbaren Restpunkten.".into(),
        });
    }

    if let Some(ref perf) = normalized.raw_performance {
        if perf.score.overall >= 80 {
            positives.push(PositiveAspect {
                area: "Performance".into(),
                description: "Stabile Ladezeiten und insgesamt reaktionsschneller Seitenaufbau."
                    .into(),
            });
        }
    }
    if let Some(ref seo) = normalized.raw_seo {
        if seo.score >= 80 {
            positives.push(PositiveAspect {
                area: "SEO".into(),
                description: "Saubere Basis für Auffindbarkeit, Struktur und Meta-Daten.".into(),
            });
        }
    }
    if let Some(ref sec) = normalized.raw_security {
        if sec.score >= 80 {
            positives.push(PositiveAspect {
                area: "Sicherheit".into(),
                description: "Wichtige Sicherheitsmechanismen sind grundsätzlich vorhanden.".into(),
            });
        }
    }
    if let Some(ref mobile) = normalized.raw_mobile {
        if mobile.score >= 80 {
            positives.push(PositiveAspect {
                area: "Mobile".into(),
                description: "Die Seite ist auf kleinen Displays gut bedienbar und lesbar.".into(),
            });
        }
    }

    if positives.is_empty() {
        positives.push(PositiveAspect {
            area: "Grundstruktur".into(),
            description: "Die Seite ist grundsätzlich funktional und erreichbar.".into(),
        });
    }
    positives
}

// ─── Batch Report Builder (unchanged) ───────────────────────────────────────

/// Build a complete presentation model from a batch audit report
pub fn build_batch_presentation(batch: &BatchReport) -> BatchPresentation {
    let all_violations: Vec<_> = batch
        .reports
        .iter()
        .flat_map(|r| r.wcag_results.violations.iter().map(move |v| (v, &r.url)))
        .collect();

    let mut rule_groups: HashMap<String, GroupAccumulator> = HashMap::new();
    for (violation, url) in &all_violations {
        let entry = rule_groups
            .entry(violation.rule.clone())
            .or_insert_with(|| GroupAccumulator {
                rule: violation.rule.clone(),
                rule_name: violation.rule_name.clone(),
                severity: violation.severity,
                count: 0,
                urls: Vec::new(),
            });
        entry.count += 1;
        if !entry.urls.contains(url) {
            entry.urls.push((*url).clone());
        }
        if violation.severity > entry.severity {
            entry.severity = violation.severity;
        }
    }

    let mut top_issues: Vec<FindingGroup> = rule_groups
        .values()
        .map(build_finding_group_from_accumulator)
        .collect();
    top_issues.sort_by_key(|b| std::cmp::Reverse(impact_score(b)));

    let issue_frequency: Vec<IssueFrequency> = top_issues
        .iter()
        .map(|g| IssueFrequency {
            problem: g.title.clone(),
            wcag: g.wcag_criterion.clone(),
            occurrences: g.occurrence_count,
            affected_urls: g.affected_urls.len(),
            priority: g.priority,
        })
        .collect();

    let action_plan = derive_action_plan(&top_issues);

    let mut url_ranking: Vec<UrlSummary> = batch
        .reports
        .iter()
        .map(|r| {
            let critical_count = r
                .wcag_results
                .violations
                .iter()
                .filter(|v| matches!(v.severity, Severity::Critical | Severity::High))
                .count();
            UrlSummary {
                url: r.url.clone(),
                score: r.score,
                grade: r.grade.clone(),
                critical_violations: critical_count,
                total_violations: r.wcag_results.violations.len(),
                passed: r.passed(),
                priority: score_to_priority(r.score),
            }
        })
        .collect();
    url_ranking.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

    let url_details: Vec<CompactUrlSummary> = batch
        .reports
        .iter()
        .map(|r| {
            let per_url_groups = group_violations(&r.wcag_results.violations, &[]);
            let mut sorted = per_url_groups;
            sorted.sort_by_key(|b| std::cmp::Reverse(impact_score(b)));
            let top_issue_titles: Vec<String> =
                sorted.iter().take(3).map(|g| g.title.clone()).collect();

            let mut module_scores = vec![("Accessibility".to_string(), r.score.round() as u32)];
            if let Some(ref p) = r.performance {
                module_scores.push(("Performance".to_string(), p.score.overall));
            }
            if let Some(ref s) = r.seo {
                module_scores.push(("SEO".to_string(), s.score));
            }
            if let Some(ref s) = r.security {
                module_scores.push(("Security".to_string(), s.score));
            }
            if let Some(ref m) = r.mobile {
                module_scores.push(("Mobile".to_string(), m.score));
            }

            CompactUrlSummary {
                url: r.url.clone(),
                score: r.score,
                grade: r.grade.clone(),
                page_type: r
                    .seo
                    .as_ref()
                    .and_then(|seo| seo.content_profile.as_ref())
                    .map(|profile| profile.page_classification.primary_type.label().to_string()),
                page_attributes: r
                    .seo
                    .as_ref()
                    .and_then(|seo| seo.content_profile.as_ref())
                    .map(|profile| profile.page_classification.attributes.clone())
                    .unwrap_or_default(),
                top_issues: top_issue_titles,
                module_scores,
            }
        })
        .collect();

    let mut sorted_by_score: Vec<_> = batch.reports.iter().collect();
    sorted_by_score.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
    let worst_urls: Vec<(String, f32)> = sorted_by_score
        .iter()
        .take(3)
        .map(|r| (truncate_url(&r.url, 60), r.score))
        .collect();
    let best_urls: Vec<(String, f32)> = sorted_by_score
        .iter()
        .rev()
        .take(3)
        .map(|r| (truncate_url(&r.url, 60), r.score))
        .collect();

    let verdict_text = build_batch_verdict(batch);

    let severity_distribution = {
        let (mut critical, mut high, mut medium, mut low) = (0usize, 0usize, 0usize, 0usize);
        for (violation, _) in &all_violations {
            match violation.severity {
                Severity::Critical => critical += 1,
                Severity::High => high += 1,
                Severity::Medium => medium += 1,
                Severity::Low => low += 1,
            }
        }
        SeverityDistribution {
            critical,
            high,
            medium,
            low,
        }
    };

    let mut page_type_counts: HashMap<String, usize> = HashMap::new();
    let mut page_semantic_scores: Vec<(String, String, u32)> = Vec::new();
    let mut thin_pages = 0usize;
    let mut editorial_pages = 0usize;
    let mut marketing_pages = 0usize;
    for report in &batch.reports {
        if let Some(profile) = report
            .seo
            .as_ref()
            .and_then(|seo| seo.content_profile.as_ref())
        {
            let label = profile.page_classification.primary_type.label().to_string();
            *page_type_counts.entry(label).or_default() += 1;
            let semantic_score = average_page_semantic_score(&profile.page_classification);
            page_semantic_scores.push((
                report.url.clone(),
                profile.page_classification.primary_type.label().to_string(),
                semantic_score,
            ));
            match profile.page_classification.primary_type {
                PageType::ThinContent => thin_pages += 1,
                PageType::Editorial => editorial_pages += 1,
                PageType::MarketingLanding => marketing_pages += 1,
                _ => {}
            }
        }
    }
    let mut page_type_distribution: Vec<(String, usize, u32)> = page_type_counts
        .into_iter()
        .map(|(label, count)| {
            let pct = ((count as f64 / batch.summary.total_urls as f64) * 100.0).round() as u32;
            (label, count, pct)
        })
        .collect();
    page_type_distribution.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut distribution_insights = Vec::new();
    if thin_pages > 0 && (thin_pages as f64 / batch.summary.total_urls as f64) >= 0.2 {
        distribution_insights.push(
            "Hoher Anteil an Thin-Content-Seiten: Das kann Informationswert und SEO-Potenzial begrenzen."
                .to_string(),
        );
    }
    if editorial_pages == 0 {
        distribution_insights.push(
            "Editoriale Inhaltsseiten fehlen: Wissensaufbau und Suchintentionen werden kaum bedient."
                .to_string(),
        );
    }
    if marketing_pages > 0 && (marketing_pages as f64 / batch.summary.total_urls as f64) >= 0.5 {
        distribution_insights.push(
            "Marketing- und Landingpages dominieren: Mehr strukturierter Tiefeninhalt würde die Domain ausbalancieren."
                .to_string(),
        );
    }
    if distribution_insights.is_empty() && !page_type_distribution.is_empty() {
        distribution_insights.push(
            "Die Seitentypen sind insgesamt ausgewogen verteilt, ohne klar dominierende Schwachmuster."
                .to_string(),
        );
    }

    page_semantic_scores.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
    let strongest_content_pages = page_semantic_scores.iter().take(5).cloned().collect();
    let weakest_content_pages = page_semantic_scores.iter().rev().take(5).cloned().collect();

    BatchPresentation {
        cover: CoverData {
            title: "Web Accessibility Batch Audit Report".to_string(),
            url: format!("{} URLs geprüft", batch.summary.total_urls),
            date: chrono::Utc::now().format("%d.%m.%Y").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        portfolio_summary: PortfolioSummary {
            total_urls: batch.summary.total_urls,
            passed: batch.summary.passed,
            failed: batch.summary.failed,
            average_score: batch.summary.average_score,
            total_violations: batch.summary.total_violations,
            duration_ms: batch.total_duration_ms,
            verdict_text,
            worst_urls,
            best_urls,
            severity_distribution,
            page_type_distribution,
            distribution_insights,
            strongest_content_pages,
            weakest_content_pages,
        },
        top_issues: top_issues.into_iter().take(10).collect(),
        issue_frequency,
        action_plan,
        url_ranking,
        url_details,
        appendix: build_batch_appendix(batch),
    }
}

// ─── Internal helpers ───────────────────────────────────────────────────────

/// Look up taxonomy metadata for a rule by its WCAG ID
fn taxonomy_fields(wcag_id: &str) -> (Option<String>, Option<String>, Option<String>, String) {
    if let Some(rule) = RuleLookup::by_legacy_wcag_id(wcag_id) {
        (
            Some(rule.dimension.label().to_string()),
            Some(rule.subcategory.label().to_string()),
            Some(rule.issue_class.label().to_string()),
            rule.id.to_string(),
        )
    } else {
        (None, None, None, wcag_id.to_string())
    }
}

struct GroupAccumulator {
    rule: String,
    rule_name: String,
    severity: Severity,
    count: usize,
    urls: Vec<String>,
}

fn group_violations(
    violations: &[crate::wcag::Violation],
    _url_context: &[&str],
) -> Vec<FindingGroup> {
    let mut groups: HashMap<String, (Vec<&crate::wcag::Violation>, usize)> = HashMap::new();
    for v in violations {
        let entry = groups
            .entry(v.rule.clone())
            .or_insert_with(|| (Vec::new(), 0));
        entry.0.push(v);
        entry.1 += 1;
    }

    groups
        .into_iter()
        .map(|(rule_id, (violations, count))| {
            let first = violations[0];
            let explanation = get_explanation(&rule_id);
            let (dimension, subcategory, issue_class, mapped_rule_id) = taxonomy_fields(&rule_id);
            let dimension_label = dimension.as_deref().unwrap_or("Accessibility");

            let (
                title,
                customer_desc,
                user_impact_text,
                business_impact,
                typical_cause,
                recommendation,
                technical_note,
                role,
                effort,
                execution_priority,
            ) = if let Some(expl) = explanation {
                (
                    expl.customer_title.to_string(),
                    expl.customer_description.to_string(),
                    expl.user_impact.to_string(),
                    derive_business_impact(expl.user_impact, dimension_label, first.severity),
                    expl.typical_cause.to_string(),
                    expl.recommendation.to_string(),
                    expl.technical_note.to_string(),
                    expl.responsible_role,
                    expl.effort_estimate,
                    derive_execution_priority(
                        first.severity,
                        expl.effort_estimate,
                        dimension_label,
                    ),
                )
            } else {
                (
                    format!("{} — {}", first.rule, first.rule_name),
                    first.message.clone(),
                    "Nutzer mit Einschränkungen können betroffen sein.".to_string(),
                    derive_business_impact(
                        "Nutzer mit Einschränkungen können betroffen sein.",
                        dimension_label,
                        first.severity,
                    ),
                    "Automatisch erkanntes Problem.".to_string(),
                    first
                        .fix_suggestion
                        .clone()
                        .unwrap_or_else(|| "Bitte prüfen und beheben.".to_string()),
                    first.fix_suggestion.clone().unwrap_or_default(),
                    Role::Development,
                    Effort::Medium,
                    derive_execution_priority(first.severity, Effort::Medium, dimension_label),
                )
            };

            let examples = explanation.map(|e| e.examples()).unwrap_or_default();

            FindingGroup {
                title,
                rule_id: mapped_rule_id,
                wcag_criterion: rule_id,
                wcag_level: format!("{:?}", first.level),
                dimension,
                subcategory,
                issue_class,
                severity: first.severity,
                priority: severity_to_priority(first.severity),
                customer_description: customer_desc,
                user_impact: user_impact_text,
                business_impact,
                typical_cause,
                recommendation,
                technical_note,
                occurrence_count: count,
                affected_urls: Vec::new(),
                affected_elements: count,
                responsible_role: role,
                effort,
                execution_priority,
                examples,
            }
        })
        .collect()
}

fn build_finding_group_from_accumulator(acc: &GroupAccumulator) -> FindingGroup {
    let explanation = get_explanation(&acc.rule);
    let (dimension, subcategory, issue_class, mapped_rule_id) = taxonomy_fields(&acc.rule);
    let dimension_label = dimension.as_deref().unwrap_or("Accessibility");
    let (
        title,
        customer_desc,
        user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        role,
        effort,
        execution_priority,
    ) = if let Some(expl) = explanation {
        (
            expl.customer_title.to_string(),
            expl.customer_description.to_string(),
            expl.user_impact.to_string(),
            derive_business_impact(expl.user_impact, dimension_label, acc.severity),
            expl.typical_cause.to_string(),
            expl.recommendation.to_string(),
            expl.technical_note.to_string(),
            expl.responsible_role,
            expl.effort_estimate,
            derive_execution_priority(acc.severity, expl.effort_estimate, dimension_label),
        )
    } else {
        (
            format!("{} — {}", acc.rule, acc.rule_name),
            String::new(),
            String::new(),
            derive_business_impact("", dimension_label, acc.severity),
            String::new(),
            String::new(),
            String::new(),
            Role::Development,
            Effort::Medium,
            derive_execution_priority(acc.severity, Effort::Medium, dimension_label),
        )
    };
    let examples = explanation.map(|e| e.examples()).unwrap_or_default();

    FindingGroup {
        title,
        rule_id: mapped_rule_id,
        wcag_criterion: acc.rule.clone(),
        wcag_level: String::new(),
        dimension,
        subcategory,
        issue_class,
        severity: acc.severity,
        priority: severity_to_priority(acc.severity),
        customer_description: customer_desc,
        user_impact: user_impact_text,
        business_impact,
        typical_cause,
        recommendation,
        technical_note,
        occurrence_count: acc.count,
        affected_urls: acc.urls.clone(),
        affected_elements: acc.count,
        responsible_role: role,
        effort,
        execution_priority,
        examples,
    }
}

fn severity_to_priority(severity: Severity) -> Priority {
    match severity {
        Severity::Critical => Priority::Critical,
        Severity::High => Priority::High,
        Severity::Medium => Priority::Medium,
        Severity::Low => Priority::Low,
    }
}

fn score_to_priority(score: f32) -> Priority {
    if score < 50.0 {
        Priority::Critical
    } else if score < 70.0 {
        Priority::High
    } else if score < 85.0 {
        Priority::Medium
    } else {
        Priority::Low
    }
}

fn impact_score(group: &FindingGroup) -> u32 {
    let severity_weight = match group.severity {
        Severity::Critical => 4,
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
    };
    severity_weight * group.occurrence_count as u32
}

fn build_verdict_text(url: &str, score: f32) -> String {
    if score >= 90.0 {
        format!(
            "Die Website {} erreicht im Accessibility-Audit {:.0}/100 Punkte. \
                 Die technische Basis ist stark; verbleibende Barrieren sind gezielt und gut priorisierbar.",
            url, score
        )
    } else if score >= 70.0 {
        format!(
            "Die Website {} erreicht im Accessibility-Audit {:.0}/100 Punkte. \
                 Die Basis ist solide, es bestehen aber relevante Barrieren mit klarem Verbesserungshebel.",
            url, score
        )
    } else if score >= 50.0 {
        format!(
            "Die Website {} erreicht im Accessibility-Audit nur {:.0}/100 Punkte. \
                 Es bestehen deutliche Barrieren, die zeitnah priorisiert und behoben werden sollten.",
            url, score
        )
    } else {
        format!(
            "Die Website {} erreicht im Accessibility-Audit nur {:.0}/100 Punkte. \
                 Die Barrierefreiheit ist stark eingeschränkt; es besteht akuter Handlungsbedarf.",
            url, score
        )
    }
}

fn localized_report_title(locale: &str) -> String {
    match locale {
        "en" => "Accessibility Audit Report".to_string(),
        _ => "Barrierefreiheits-Prüfbericht".to_string(),
    }
}

fn localized_report_subtitle(locale: &str) -> &'static str {
    match locale {
        "en" => "Automated accessibility audit with optional website quality modules.",
        _ => "Automatisierter Accessibility-Report mit ergänzenden Qualitätsmodulen.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalize;
    use crate::audit::AuditReport;
    use crate::cli::WcagLevel;
    use crate::output::report_model::ReportConfig;
    use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};
    use crate::seo::SeoAnalysis;
    use crate::wcag::{Violation, WcagResults};

    #[test]
    fn test_view_model_uses_accessibility_score_as_primary_score() {
        let mut wcag = WcagResults::new();
        wcag.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::taxonomy::Severity::High,
            "Missing alt text",
            "n1",
        ));

        let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, wcag, 1500)
            .with_performance(crate::audit::PerformanceResults {
                vitals: WebVitals::default(),
                score: PerformanceScore {
                    overall: 60,
                    grade: PerformanceGrade::NeedsImprovement,
                    lcp_score: 15,
                    fcp_score: 15,
                    cls_score: 15,
                    interactivity_score: 15,
                },
            })
            .with_seo(SeoAnalysis::default());

        let normalized = normalize(&report);
        let vm = build_view_model(&normalized, &ReportConfig::default());

        assert_eq!(vm.summary.score, normalized.score);
        assert_eq!(vm.summary.grade, normalized.grade);
        assert_eq!(vm.summary.certificate, normalized.certificate);
        assert!(vm
            .summary
            .metrics
            .iter()
            .any(|m| m.title == format!("Gesamtscore{NBSP}Website")));
    }
}

fn derive_action_plan(finding_groups: &[FindingGroup]) -> ActionPlan {
    let mut quick_wins = Vec::new();
    let mut medium_term = Vec::new();
    let mut structural = Vec::new();

    for group in finding_groups {
        let item = ActionItem {
            action: group.recommendation.clone(),
            benefit: group.business_impact.clone(),
            role: group.responsible_role,
            priority: group.priority,
            execution_priority: group.execution_priority,
            effort: group.effort,
        };
        match group.effort {
            Effort::Quick => quick_wins.push(item),
            Effort::Medium => medium_term.push(item),
            Effort::Structural => structural.push(item),
        }
    }

    quick_wins.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
    medium_term.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
    structural.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));

    let mut role_map: HashMap<Role, Vec<String>> = HashMap::new();
    for group in finding_groups {
        role_map
            .entry(group.responsible_role)
            .or_default()
            .push(group.title.clone());
    }
    role_map
        .entry(Role::ProjectManagement)
        .or_default()
        .extend([
            "Priorisierung der Maßnahmen".to_string(),
            "Qualitätssicherung und Testing".to_string(),
            "Verantwortlichkeiten festlegen".to_string(),
        ]);

    let role_assignments: Vec<RoleAssignment> = role_map
        .into_iter()
        .map(|(role, mut responsibilities)| {
            responsibilities.dedup();
            RoleAssignment {
                role,
                responsibilities,
            }
        })
        .collect();

    ActionPlan {
        quick_wins,
        medium_term,
        structural,
        role_assignments,
    }
}

fn derive_accessibility_lever(normalized: &NormalizedReport) -> String {
    if let Some(finding) = normalized
        .findings
        .iter()
        .max_by_key(|f| f.occurrence_count)
    {
        format!("Größter Hebel: {}", finding.title)
    } else {
        "Größter Hebel: Ergebnisse stabil halten und manuell nachprüfen".to_string()
    }
}

fn derive_performance_lever(perf: &crate::audit::PerformanceResults) -> String {
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1500 {
            return format!("Größter Hebel: DOM-Größe reduzieren ({dom_nodes} Knoten)");
        }
    }
    if let Some(load) = perf.vitals.load_time {
        if load > 2_500.0 {
            return format!("Größter Hebel: Ladezeit senken ({load:.0} ms)");
        }
    }
    "Größter Hebel: Render-Pfad und Asset-Größe weiter optimieren".to_string()
}

fn derive_seo_lever(seo: &crate::seo::SeoAnalysis) -> String {
    if !seo.meta_issues.is_empty() {
        return format!(
            "Größter Hebel: Meta-Daten bereinigen ({} offene Punkte)",
            seo.meta_issues.len()
        );
    }
    if seo.social.completeness < 80 {
        return "Größter Hebel: Social-Meta-Daten vervollständigen".to_string();
    }
    "Größter Hebel: Struktur- und Inhalts-Signale weiter schärfen".to_string()
}

fn derive_security_lever(sec: &crate::security::SecurityAnalysis) -> String {
    let missing_headers = sec.headers.content_security_policy.is_none() as usize
        + sec.headers.strict_transport_security.is_none() as usize
        + sec.headers.permissions_policy.is_none() as usize
        + sec.headers.referrer_policy.is_none() as usize;
    if missing_headers > 0 {
        return format!(
            "Größter Hebel: fehlende Security-Header ergänzen ({missing_headers} Kernheader)"
        );
    }
    "Größter Hebel: Header-Regeln und TLS-Setup weiter härten".to_string()
}

fn derive_mobile_lever(mobile: &crate::mobile::MobileFriendliness) -> String {
    if mobile.touch_targets.small_targets > 0 {
        return format!(
            "Größter Hebel: Touch Targets vergrößern ({} zu klein)",
            mobile.touch_targets.small_targets
        );
    }
    if mobile.touch_targets.crowded_targets > 0 {
        return format!(
            "Größter Hebel: Abstände mobiler Bedienelemente erhöhen ({})",
            mobile.touch_targets.crowded_targets
        );
    }
    "Größter Hebel: mobile Lesbarkeit und Touch-Flows weiter optimieren".to_string()
}

fn derive_business_impact(user_impact: &str, dimension: &str, severity: Severity) -> String {
    match dimension {
        "SEO" => {
            "Kann Auffindbarkeit, Klickrate und organischen Traffic spürbar schwächen.".to_string()
        }
        "Security" => {
            "Kann Vertrauen senken und das technische Risiko für Angriffe erhöhen.".to_string()
        }
        "Performance" => {
            "Kann zu Absprüngen, geringerer Interaktion und schwächerer Conversion führen."
                .to_string()
        }
        "Mobile" => {
            "Kann Nutzung auf Smartphones erschweren und mobile Abschlüsse kosten.".to_string()
        }
        _ => match severity {
            Severity::Critical | Severity::High => {
                "Kann Nutzer ausschließen und zugleich rechtliches Risiko erhöhen.".to_string()
            }
            _ if user_impact.contains("Sprachsteuerung") => {
                "Kann Nutzungshürden erhöhen und Interaktionen mit zentralen Elementen verhindern."
                    .to_string()
            }
            _ => "Kann Nutzung, Conversion und Wahrnehmung der Website verschlechtern.".to_string(),
        },
    }
}

fn derive_execution_priority(
    severity: Severity,
    effort: Effort,
    dimension: &str,
) -> ExecutionPriority {
    match (severity, effort, dimension) {
        (Severity::Critical, _, _) => ExecutionPriority::Immediate,
        (Severity::High, _, "Accessibility") => ExecutionPriority::Immediate,
        (Severity::High, Effort::Quick, _) => ExecutionPriority::Important,
        (Severity::High, _, _) => ExecutionPriority::Important,
        (Severity::Medium, Effort::Quick, _) => ExecutionPriority::Important,
        _ => ExecutionPriority::Optional,
    }
}

fn average_page_semantic_score(classification: &crate::seo::profile::PageClassification) -> u32 {
    let total = classification.content_depth_score
        + classification.structural_richness_score
        + classification.media_text_balance_score
        + classification.intent_fit_score;
    total / 4
}

fn summarize_page_profile(profile: &crate::seo::profile::SeoContentProfile) -> String {
    let classification = &profile.page_classification;
    let avg = average_page_semantic_score(classification);
    let quality = match avg {
        85..=100 => "sehr stimmig aufgebaut",
        70..=84 => "inhaltlich solide aufgestellt",
        50..=69 => "nur teilweise klar strukturiert",
        _ => "aktuell inhaltlich und strukturell schwach ausgeprägt",
    };

    let mut traits = classification.attributes.clone();
    if traits.is_empty() {
        traits.push("ohne klare Zusatzmerkmale".to_string());
    }

    format!(
        "Die Seite wirkt wie „{}“ und ist {}. Auffällig sind {}.",
        classification.primary_type.label(),
        quality,
        traits.join(", ")
    )
}

fn page_profile_optimization_note(profile: &crate::seo::profile::SeoContentProfile) -> String {
    let classification = &profile.page_classification;
    if classification.content_depth_score < 45 {
        return "Mehr inhaltliche Tiefe und klar gegliederte Abschnitte würden den Nutzwert erhöhen."
            .to_string();
    }
    if classification.structural_richness_score < 55 {
        return "Mehr Zwischenüberschriften und eine klarere Inhaltsstruktur würden die Seite besser scannbar machen."
            .to_string();
    }
    if classification.media_text_balance_score < 55 {
        return "Das Verhältnis aus Text und visuellen Elementen ist unausgewogen; mehr erklärender Kontext würde helfen."
            .to_string();
    }
    if classification.intent_fit_score < 65 {
        return "Die Seite bedient ihren Seitentyp noch nicht sauber; Aufbau und Inhalte sollten stärker auf das eigentliche Nutzerziel einzahlen."
            .to_string();
    }
    "Die Seite passt insgesamt gut zu ihrem Seitentyp. Der größte Hebel liegt in weiterer inhaltlicher Schärfung statt in Grundsatzumbauten."
        .to_string()
}

fn grade_label(score: u32) -> &'static str {
    match score {
        90..=100 => "Sehr gut",
        75..=89 => "Gut",
        60..=74 => "Befriedigend",
        40..=59 => "Ausbaufähig",
        _ => "Kritisch",
    }
}

fn interpret_score(score: f32, area: &str) -> String {
    let grade = grade_label(score.round() as u32);
    match grade {
        "Sehr gut" => format!("{} — die {} ist auf einem hohen Niveau.", grade, area),
        "Gut" => format!(
            "{} — die {} ist solide, einzelne Verbesserungen sind möglich.",
            grade, area
        ),
        "Befriedigend" => format!("{} — die {} weist einzelne Schwächen auf.", grade, area),
        "Ausbaufähig" => format!("{} — die {} weist relevante Schwächen auf.", grade, area),
        _ => format!(
            "{} — die {} hat erhebliche Mängel, die behoben werden sollten.",
            grade, area
        ),
    }
}

fn build_batch_verdict(batch: &BatchReport) -> String {
    let avg = batch.summary.average_score;
    if avg >= 90.0 {
        format!(
            "Über {} geprüfte URLs hinweg erreicht die Website einen durchschnittlichen \
                 Accessibility-Score von {:.0}/100 — ein sehr gutes Ergebnis.",
            batch.summary.total_urls, avg
        )
    } else if avg >= 70.0 {
        format!(
            "Im Durchschnitt erreichen die {} geprüften URLs {:.0}/100 Punkte. \
                 Die Basis ist solide, es bestehen aber wiederkehrende Barrieren.",
            batch.summary.total_urls, avg
        )
    } else if avg >= 50.0 {
        format!(
            "Die {} geprüften URLs erreichen im Schnitt nur {:.0}/100 Punkte. \
                 Es bestehen erhebliche systematische Barrierefreiheitsprobleme.",
            batch.summary.total_urls, avg
        )
    } else {
        format!(
            "Die {} geprüften URLs erreichen im Schnitt nur {:.0}/100 Punkte. \
                 Die Barrierefreiheit ist stark eingeschränkt — dringender Handlungsbedarf.",
            batch.summary.total_urls, avg
        )
    }
}

fn build_batch_appendix(batch: &BatchReport) -> BatchAppendixData {
    BatchAppendixData {
        per_url: batch
            .reports
            .iter()
            .map(|r| {
                // Aggregate violations by rule for each URL
                let mut rule_map: std::collections::HashMap<String, AppendixViolation> =
                    std::collections::HashMap::new();
                let mut rule_order: Vec<String> = Vec::new();

                for v in &r.wcag_results.violations {
                    let element = AffectedElement {
                        selector: v.selector.clone().unwrap_or_else(|| v.node_id.clone()),
                        node_id: v.node_id.clone(),
                    };

                    if let Some(existing) = rule_map.get_mut(&v.rule) {
                        existing.affected_elements.push(element);
                    } else {
                        rule_order.push(v.rule.clone());
                        rule_map.insert(
                            v.rule.clone(),
                            AppendixViolation {
                                rule: v.rule.clone(),
                                rule_name: v.rule_name.clone(),
                                severity: v.severity,
                                message: v.message.clone(),
                                fix_suggestion: v.fix_suggestion.clone(),
                                affected_elements: vec![element],
                            },
                        );
                    }
                }

                UrlAppendix {
                    url: r.url.clone(),
                    violations: rule_order
                        .into_iter()
                        .filter_map(|rule| rule_map.remove(&rule))
                        .collect(),
                }
            })
            .collect(),
    }
}

fn yes_no(val: bool) -> String {
    if val {
        "Ja".to_string()
    } else {
        "Nein".to_string()
    }
}

// ─── Clone implementations ──────────────────────────────────────────────────

impl Clone for FindingGroup {
    fn clone(&self) -> Self {
        FindingGroup {
            title: self.title.clone(),
            rule_id: self.rule_id.clone(),
            wcag_criterion: self.wcag_criterion.clone(),
            wcag_level: self.wcag_level.clone(),
            dimension: self.dimension.clone(),
            subcategory: self.subcategory.clone(),
            issue_class: self.issue_class.clone(),
            severity: self.severity,
            priority: self.priority,
            customer_description: self.customer_description.clone(),
            user_impact: self.user_impact.clone(),
            business_impact: self.business_impact.clone(),
            typical_cause: self.typical_cause.clone(),
            recommendation: self.recommendation.clone(),
            technical_note: self.technical_note.clone(),
            occurrence_count: self.occurrence_count,
            affected_urls: self.affected_urls.clone(),
            affected_elements: self.affected_elements,
            responsible_role: self.responsible_role,
            effort: self.effort,
            execution_priority: self.execution_priority,
            examples: self.examples.clone(),
        }
    }
}

impl Clone for ExampleBlock {
    fn clone(&self) -> Self {
        ExampleBlock {
            bad: self.bad.clone(),
            good: self.good.clone(),
            decorative: self.decorative.clone(),
        }
    }
}

impl Clone for RoleAssignment {
    fn clone(&self) -> Self {
        RoleAssignment {
            role: self.role,
            responsibilities: self.responsibilities.clone(),
        }
    }
}
