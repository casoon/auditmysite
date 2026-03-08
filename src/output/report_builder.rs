//! Report builder — transforms raw audit data into ViewModels
//!
//! This module takes raw AuditReport / BatchReport data and produces
//! structured ViewModels with grouped findings, aggregated statistics,
//! and pre-computed presentation data. The renderer does zero data transformation.

use std::collections::HashMap;

use crate::audit::{AccessibilityScorer, AuditReport, BatchReport};
use crate::output::explanations::get_explanation;
use crate::output::report_model::*;
use crate::util::truncate_url;
use crate::wcag::Severity;

// ─── Single Report ViewModel ────────────────────────────────────────────────

/// Build a complete ViewModel from a single audit report
pub fn build_view_model(report: &AuditReport, config: &ReportConfig) -> ReportViewModel {
    let finding_groups = group_violations(&report.wcag_results.violations, &[]);
    let mut sorted_groups = finding_groups;
    sorted_groups.sort_by(|a, b| impact_score(b).cmp(&impact_score(a)));

    // Normalize: If SEO detected a valid lang attribute in the DOM,
    // suppress the WCAG 3.1.1 violation (AXTree detection is unreliable for lang)
    let suppress_lang = report.seo.as_ref().map_or(false, |s| s.technical.has_lang);
    let had_311 = sorted_groups.iter().any(|g| g.wcag_criterion == "3.1.1");
    if suppress_lang {
        sorted_groups.retain(|g| g.wcag_criterion != "3.1.1");
    }

    // Correct score: if 3.1.1 was suppressed, reverse its scoring penalties
    // Scoring: -2.5 per serious violation + -10 specific penalty = -12.5
    let mut corrected_score = report.score;
    if suppress_lang && had_311 {
        corrected_score += 12.5; // Reverse: 2.5 (serious base) + 10.0 (specific penalty)
        corrected_score = corrected_score.clamp(0.0, 100.0);
    }
    let score = corrected_score.round() as u32;
    let grade = if suppress_lang && had_311 {
        AccessibilityScorer::calculate_grade(corrected_score).to_string()
    } else {
        report.grade.clone()
    };
    let date = report.timestamp.format("%d.%m.%Y").to_string();

    let top_findings: Vec<FindingGroup> = sorted_groups.iter().take(5).cloned().collect();
    let positive_aspects = derive_positive_aspects(report, corrected_score);
    let action_plan = derive_action_plan(&sorted_groups);

    // Build module list
    let mut module_names: Vec<String> = vec!["Accessibility".into()];
    if report.performance.is_some() { module_names.push("Performance".into()); }
    if report.seo.is_some() { module_names.push("SEO".into()); }
    if report.security.is_some() { module_names.push("Sicherheit".into()); }
    if report.mobile.is_some() { module_names.push("Mobile".into()); }

    // Build severity block (counts individual violations, not rule groups)
    let severity = build_severity_block(&sorted_groups);

    // Build modules block
    let modules = build_modules_block(report, corrected_score);

    // Build summary metrics
    let best_module = find_best_module_name(report, corrected_score);
    let quick_win_count = action_plan.quick_wins.len();
    let critical_count = sorted_groups.iter()
        .filter(|g| matches!(g.severity, Severity::Critical | Severity::Serious))
        .map(|g| g.occurrence_count)
        .sum::<usize>();
    let total_violations: usize = sorted_groups.iter().map(|g| g.occurrence_count).sum();


    // Build actions block (pre-mapped for ActionRoadmap component)
    let actions = build_actions_block(&action_plan);

    // Build module details
    let module_details = build_module_details(report);

    let author = config.company_name.as_deref().unwrap_or("AuditMySite").to_string();

    ReportViewModel {
        meta: MetaBlock {
            title: "Web Accessibility Audit Report".to_string(),
            subtitle: report.url.clone(),
            date: date.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            author,
            report_level: config.level,
            score_label: format!("{}/100", score),
        },
        cover: CoverBlock {
            brand: config.company_name.as_deref().unwrap_or("AuditMySite").to_string(),
            title: "Web Accessibility Audit Report".to_string(),
            domain: report.url.clone(),
            subtitle: "Automatisierte Analyse zu Accessibility, Performance, SEO, Sicherheit und Mobile.".to_string(),
            date: date.clone(),
            score,
            grade: grade.clone(),
            total_issues: total_violations as u32,
            critical_issues: critical_count as u32,
            modules: module_names,
        },
        summary: SummaryBlock {
            score,
            grade: grade.clone(),
            domain: report.url.clone(),
            date: date.clone(),
            verdict: build_verdict_text(&report.url, corrected_score),
            metrics: vec![
                MetricItem {
                    title: "Verstöße gesamt".into(),
                    value: total_violations.to_string(),
                    accent_color: None,
                },
                MetricItem {
                    title: "Kritisch".into(),
                    value: critical_count.to_string(),
                    accent_color: Some("#ef4444".into()),
                },
                MetricItem {
                    title: "Stärkstes Modul".into(),
                    value: best_module,
                    accent_color: Some("#22c55e".into()),
                },
                MetricItem {
                    title: "Quick Wins".into(),
                    value: quick_win_count.to_string(),
                    accent_color: Some("#2563eb".into()),
                },
            ],
            top_actions: top_findings.iter().take(3).map(|f| f.recommendation.clone()).collect(),
            positive_aspects: positive_aspects.iter()
                .map(|a| format!("{}: {}", a.area, a.description))
                .collect(),
        },
        methodology: build_methodology(&report.url),
        modules,
        severity,
        findings: FindingsBlock {
            top_findings,
            all_findings: sorted_groups,
        },
        module_details,
        actions,
        appendix: build_appendix_block(report, suppress_lang),
    }
}

// ─── Block Builders ─────────────────────────────────────────────────────────

fn build_severity_block(groups: &[FindingGroup]) -> SeverityBlock {
    // Count individual violations (occurrence_count), not rule groups
    let critical = groups.iter()
        .filter(|g| matches!(g.severity, Severity::Critical))
        .map(|g| g.occurrence_count as u32).sum();
    let serious = groups.iter()
        .filter(|g| matches!(g.severity, Severity::Serious))
        .map(|g| g.occurrence_count as u32).sum();
    let moderate = groups.iter()
        .filter(|g| matches!(g.severity, Severity::Moderate))
        .map(|g| g.occurrence_count as u32).sum();
    let minor = groups.iter()
        .filter(|g| matches!(g.severity, Severity::Minor))
        .map(|g| g.occurrence_count as u32).sum();
    let total = critical + serious + moderate + minor;

    SeverityBlock {
        critical,
        serious,
        moderate,
        minor,
        total,
        has_issues: total > 0,
    }
}

fn build_modules_block(report: &AuditReport, a11y_score: f32) -> ModulesBlock {
    let mut dashboard = vec![ModuleScore {
        name: "Barrierefreiheit".into(),
        score: a11y_score.round() as u32,
        interpretation: interpret_score(a11y_score, "Barrierefreiheit"),
        good_threshold: 75,
        warn_threshold: 50,
    }];

    if let Some(ref p) = report.performance {
        dashboard.push(ModuleScore {
            name: "Performance".into(),
            score: p.score.overall,
            interpretation: interpret_score(p.score.overall as f32, "Performance"),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref s) = report.seo {
        dashboard.push(ModuleScore {
            name: "SEO".into(),
            score: s.score,
            interpretation: interpret_score(s.score as f32, "SEO"),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref s) = report.security {
        dashboard.push(ModuleScore {
            name: "Sicherheit".into(),
            score: s.score,
            interpretation: interpret_score(s.score as f32, "Sicherheit"),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }
    if let Some(ref m) = report.mobile {
        dashboard.push(ModuleScore {
            name: "Mobile".into(),
            score: m.score,
            interpretation: interpret_score(m.score as f32, "mobile Nutzbarkeit"),
            good_threshold: 75,
            warn_threshold: 50,
        });
    }

    let has_multiple = dashboard.len() > 1;
    let overall_score = if has_multiple { Some(report.overall_score()) } else { None };
    let overall_interpretation = overall_score.map(|_| {
        "Gewichteter Durchschnitt aller aktiven Module. Accessibility fließt mit 40% ein, \
         Performance und SEO mit je 20%, Sicherheit und Mobile mit je 10%.".to_string()
    });

    ModulesBlock { dashboard, overall_score, overall_interpretation }
}

fn build_actions_block(plan: &ActionPlan) -> ActionsBlock {
    let map_items = |items: &[ActionItem], effort: &str| -> Vec<RoadmapItemData> {
        items.iter().map(|i| RoadmapItemData {
            action: i.action.clone(),
            role: i.role.label().to_string(),
            priority: i.priority.label().to_string(),
            effort: effort.to_string(),
            benefit: i.benefit.clone(),
        }).collect()
    };

    let mut columns = Vec::new();
    if !plan.quick_wins.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Quick Wins".into(),
            accent_color: "#22c55e".into(),
            items: map_items(&plan.quick_wins, "Niedrig"),
        });
    }
    if !plan.medium_term.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Mittelfristig".into(),
            accent_color: "#f59e0b".into(),
            items: map_items(&plan.medium_term, "Mittel"),
        });
    }
    if !plan.structural.is_empty() {
        columns.push(RoadmapColumnData {
            title: "Strukturell".into(),
            accent_color: "#2563eb".into(),
            items: map_items(&plan.structural, "Hoch"),
        });
    }

    ActionsBlock {
        roadmap_columns: columns,
        role_assignments: plan.role_assignments.clone(),
        intro_text: "Auf Basis der identifizierten Probleme empfehlen wir die folgenden Maßnahmen, \
                     gegliedert nach Aufwand und Wirkung.".to_string(),
    }
}

fn build_appendix_block(report: &AuditReport, suppress_lang: bool) -> AppendixBlock {
    // Aggregate violations by rule ID, filtering suppressed rules
    let mut rule_map: std::collections::HashMap<String, AppendixViolation> = std::collections::HashMap::new();
    let mut rule_order: Vec<String> = Vec::new();

    for v in report.wcag_results.violations.iter()
        .filter(|v| !(suppress_lang && v.rule == "3.1.1"))
    {
        let element = AffectedElement {
            selector: v.selector.clone().unwrap_or_else(|| v.node_id.clone()),
            node_id: v.node_id.clone(),
        };

        if let Some(existing) = rule_map.get_mut(&v.rule) {
            existing.affected_elements.push(element);
        } else {
            rule_order.push(v.rule.clone());
            rule_map.insert(v.rule.clone(), AppendixViolation {
                rule: v.rule.clone(),
                rule_name: v.rule_name.clone(),
                severity: capitalize_severity(&v.severity),
                message: v.message.clone(),
                fix_suggestion: v.fix_suggestion.clone(),
                affected_elements: vec![element],
            });
        }
    }

    let violations: Vec<AppendixViolation> = rule_order.into_iter()
        .filter_map(|rule| rule_map.remove(&rule))
        .collect();

    let has_violations = !violations.is_empty();

    AppendixBlock {
        violations,
        score_methodology: "Score-Berechnung: Basis 100 Punkte. Abzug von 2,5 Punkten pro \
            kritischem/schwerem Verstoß und 1 Punkt pro mäßigem Verstoß. \
            Zusätzliche Abzüge für besonders impactstarke Regeln (z. B. fehlende Sprache: -10, \
            fehlende Überschriften: -20, fehlende Alt-Texte: -3, fehlende Labels: -5, \
            Kontrastprobleme: -5).".to_string(),
        has_violations,
    }
}

fn build_methodology(url: &str) -> MethodologyBlock {
    MethodologyBlock {
        scope: format!(
            "Automatisierte Prüfung der Seite {} auf Barrierefreiheit nach WCAG 2.1 (Level AA). \
             Zusätzlich wurden Performance, SEO, Sicherheit und mobile Nutzbarkeit analysiert.", url
        ),
        method: "Die Prüfung erfolgte über den Chrome DevTools Protocol (CDP) und den \
                 nativen Accessibility Tree des Browsers. 21 WCAG-Regeln wurden automatisiert \
                 gegen den Seiteninhalt geprüft.".to_string(),
        limitations: "Automatisierte Tests können ca. 30–40% aller Barrierefreiheitsprobleme erkennen. \
                      Komplexe Aspekte wie korrekte Tab-Reihenfolge, sinnvolle Alt-Texte oder \
                      verständliche Sprache erfordern zusätzlich manuelle Prüfung.".to_string(),
        disclaimer: "Dieser Report stellt eine automatisierte technische Analyse dar. \
                     Er ersetzt keine vollständige Konformitätsbewertung nach WCAG 2.1. \
                     Für eine rechtsverbindliche Aussage zur Barrierefreiheit ist eine \
                     umfassende manuelle Prüfung durch Experten erforderlich.".to_string(),
    }
}

fn build_module_details(report: &AuditReport) -> ModuleDetailsBlock {
    let performance = report.performance.as_ref().map(|p| {
        let mut vitals = Vec::new();
        if let Some(ref lcp) = p.vitals.lcp {
            vitals.push(("Largest Contentful Paint (LCP)".to_string(), format!("{:.0}ms", lcp.value), lcp.rating.clone()));
        }
        if let Some(ref fcp) = p.vitals.fcp {
            vitals.push(("First Contentful Paint (FCP)".to_string(), format!("{:.0}ms", fcp.value), fcp.rating.clone()));
        }
        if let Some(ref cls) = p.vitals.cls {
            vitals.push(("Cumulative Layout Shift (CLS)".to_string(), format!("{:.3}", cls.value), cls.rating.clone()));
        }
        if let Some(ref ttfb) = p.vitals.ttfb {
            vitals.push(("Time to First Byte (TTFB)".to_string(), format!("{:.0}ms", ttfb.value), ttfb.rating.clone()));
        }
        if let Some(ref inp) = p.vitals.inp {
            vitals.push(("Interaction to Next Paint (INP)".to_string(), format!("{:.0}ms", inp.value), inp.rating.clone()));
        }
        if let Some(ref tbt) = p.vitals.tbt {
            vitals.push(("Total Blocking Time (TBT)".to_string(), format!("{:.0}ms", tbt.value), tbt.rating.clone()));
        }

        let mut additional = Vec::new();
        if let Some(nodes) = p.vitals.dom_nodes {
            additional.push(("DOM-Knoten".to_string(), nodes.to_string()));
        }
        if let Some(heap) = p.vitals.js_heap_size {
            additional.push(("JS Heap".to_string(), format!("{:.1} MB", heap as f64 / 1_048_576.0)));
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

    let seo = report.seo.as_ref().map(|s| {
        let mut meta_tags = Vec::new();
        if let Some(ref title) = s.meta.title { meta_tags.push(("Titel".to_string(), title.clone())); }
        if let Some(ref desc) = s.meta.description { meta_tags.push(("Beschreibung".to_string(), desc.clone())); }
        if let Some(ref viewport) = s.meta.viewport { meta_tags.push(("Viewport".to_string(), viewport.clone())); }

        let meta_issues: Vec<(String, String, String)> = s.meta_issues.iter()
            .map(|i| (i.field.clone(), i.severity.clone(), i.message.clone())).collect();

        SeoPresentation {
            score: s.score,
            interpretation: interpret_score(s.score as f32, "SEO"),
            meta_tags,
            meta_issues,
            heading_summary: format!("{} H1-Überschrift(en), {} Überschriften gesamt, {} Probleme",
                s.headings.h1_count, s.headings.total_count, s.headings.issues.len()),
            social_summary: format!("Open Graph: {}, Twitter Card: {}, Vollständigkeit: {}%",
                if s.social.open_graph.is_some() { "vorhanden" } else { "fehlt" },
                if s.social.twitter_card.is_some() { "vorhanden" } else { "fehlt" },
                s.social.completeness),
            technical_summary: vec![
                ("HTTPS".to_string(), yes_no(s.technical.https)),
                ("Canonical".to_string(), yes_no(s.technical.has_canonical)),
                ("Sprachangabe".to_string(), yes_no(s.technical.has_lang)),
                ("Wortanzahl".to_string(), s.technical.word_count.to_string()),
            ],
        }
    });

    let security = report.security.as_ref().map(|sec| {
        let header_checks: Vec<(&str, &Option<String>)> = vec![
            ("Content-Security-Policy", &sec.headers.content_security_policy),
            ("Strict-Transport-Security", &sec.headers.strict_transport_security),
            ("X-Content-Type-Options", &sec.headers.x_content_type_options),
            ("X-Frame-Options", &sec.headers.x_frame_options),
            ("X-XSS-Protection", &sec.headers.x_xss_protection),
            ("Referrer-Policy", &sec.headers.referrer_policy),
            ("Permissions-Policy", &sec.headers.permissions_policy),
            ("Cross-Origin-Opener-Policy", &sec.headers.cross_origin_opener_policy),
            ("Cross-Origin-Resource-Policy", &sec.headers.cross_origin_resource_policy),
        ];

        SecurityPresentation {
            score: sec.score,
            grade: sec.grade.clone(),
            interpretation: interpret_score(sec.score as f32, "Sicherheit"),
            headers: header_checks.iter().map(|(name, value)| {
                let (status, val) = match value {
                    Some(v) => ("Vorhanden".to_string(), truncate_url(v, 50)),
                    None => ("Fehlt".to_string(), "—".to_string()),
                };
                (name.to_string(), status, val)
            }).collect(),
            ssl_info: vec![
                ("HTTPS".to_string(), yes_no(sec.ssl.https)),
                ("Gültiges Zertifikat".to_string(), yes_no(sec.ssl.valid_certificate)),
                ("HSTS".to_string(), yes_no(sec.ssl.has_hsts)),
                ("HSTS Max-Age".to_string(), sec.ssl.hsts_max_age.map(|v| format!("{}s", v)).unwrap_or_else(|| "—".to_string())),
                ("Subdomains".to_string(), yes_no(sec.ssl.hsts_include_subdomains)),
                ("Preload".to_string(), yes_no(sec.ssl.hsts_preload)),
            ],
            issues: sec.issues.iter().map(|i| (i.header.clone(), i.severity.clone(), i.message.clone())).collect(),
            recommendations: sec.recommendations.clone(),
        }
    });

    let mobile = report.mobile.as_ref().map(|m| {
        MobilePresentation {
            score: m.score,
            interpretation: interpret_score(m.score as f32, "mobile Nutzbarkeit"),
            viewport: vec![
                ("Viewport-Tag".to_string(), yes_no(m.viewport.has_viewport)),
                ("device-width".to_string(), yes_no(m.viewport.uses_device_width)),
                ("Initial Scale".to_string(), yes_no(m.viewport.has_initial_scale)),
                ("Skalierbar".to_string(), yes_no(m.viewport.is_scalable)),
                ("Korrekt konfiguriert".to_string(), yes_no(m.viewport.is_properly_configured)),
            ],
            touch_targets: vec![
                ("Gesamt".to_string(), m.touch_targets.total_targets.to_string()),
                ("Ausreichend (≥44px)".to_string(), m.touch_targets.adequate_targets.to_string()),
                ("Zu klein".to_string(), m.touch_targets.small_targets.to_string()),
                ("Zu eng beieinander".to_string(), m.touch_targets.crowded_targets.to_string()),
            ],
            font_analysis: vec![
                ("Basis-Schriftgröße".to_string(), format!("{:.0}px", m.font_sizes.base_font_size)),
                ("Kleinste Schrift".to_string(), format!("{:.0}px", m.font_sizes.smallest_font_size)),
                ("Lesbarer Text".to_string(), format!("{:.0}%", m.font_sizes.legible_percentage)),
                ("Relative Einheiten".to_string(), yes_no(m.font_sizes.uses_relative_units)),
            ],
            content_sizing: vec![
                ("Passt in Viewport".to_string(), yes_no(m.content_sizing.fits_viewport)),
                ("Kein hor. Scrollen".to_string(), yes_no(!m.content_sizing.has_horizontal_scroll)),
                ("Responsive Bilder".to_string(), yes_no(m.content_sizing.uses_responsive_images)),
                ("Media Queries".to_string(), yes_no(m.content_sizing.uses_media_queries)),
            ],
            issues: m.issues.iter().map(|i| (i.category.clone(), i.severity.clone(), i.message.clone())).collect(),
        }
    });

    let has_any = performance.is_some() || seo.is_some() || security.is_some() || mobile.is_some();
    ModuleDetailsBlock { performance, seo, security, mobile, has_any }
}

// ─── Batch Report Builder (unchanged) ───────────────────────────────────────

/// Build a complete presentation model from a batch audit report
pub fn build_batch_presentation(batch: &BatchReport) -> BatchPresentation {
    let all_violations: Vec<_> = batch.reports.iter()
        .flat_map(|r| r.wcag_results.violations.iter().map(move |v| (v, &r.url)))
        .collect();

    let mut rule_groups: HashMap<String, GroupAccumulator> = HashMap::new();
    for (violation, url) in &all_violations {
        let entry = rule_groups.entry(violation.rule.clone()).or_insert_with(|| GroupAccumulator {
            rule: violation.rule.clone(),
            rule_name: violation.rule_name.clone(),
            severity: violation.severity,
            count: 0,
            urls: Vec::new(),
        });
        entry.count += 1;
        if !entry.urls.contains(url) { entry.urls.push((*url).clone()); }
        if violation.severity > entry.severity { entry.severity = violation.severity; }
    }

    let mut top_issues: Vec<FindingGroup> = rule_groups.values()
        .map(|acc| build_finding_group_from_accumulator(acc)).collect();
    top_issues.sort_by(|a, b| impact_score(b).cmp(&impact_score(a)));

    let issue_frequency: Vec<IssueFrequency> = top_issues.iter().map(|g| IssueFrequency {
        problem: g.title.clone(), wcag: g.wcag_criterion.clone(),
        occurrences: g.occurrence_count, affected_urls: g.affected_urls.len(), priority: g.priority,
    }).collect();

    let action_plan = derive_action_plan(&top_issues);

    let mut url_ranking: Vec<UrlSummary> = batch.reports.iter().map(|r| {
        let critical_count = r.wcag_results.violations.iter()
            .filter(|v| matches!(v.severity, Severity::Critical | Severity::Serious)).count();
        UrlSummary {
            url: r.url.clone(), score: r.score, grade: r.grade.clone(),
            critical_violations: critical_count, total_violations: r.wcag_results.violations.len(),
            passed: r.passed(), priority: score_to_priority(r.score),
        }
    }).collect();
    url_ranking.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

    let url_details: Vec<CompactUrlSummary> = batch.reports.iter().map(|r| {
        let per_url_groups = group_violations(&r.wcag_results.violations, &[]);
        let mut sorted = per_url_groups;
        sorted.sort_by(|a, b| impact_score(b).cmp(&impact_score(a)));
        let top_issue_titles: Vec<String> = sorted.iter().take(3).map(|g| g.title.clone()).collect();

        let mut module_scores = vec![("Accessibility".to_string(), r.score.round() as u32)];
        if let Some(ref p) = r.performance { module_scores.push(("Performance".to_string(), p.score.overall)); }
        if let Some(ref s) = r.seo { module_scores.push(("SEO".to_string(), s.score)); }
        if let Some(ref s) = r.security { module_scores.push(("Security".to_string(), s.score)); }
        if let Some(ref m) = r.mobile { module_scores.push(("Mobile".to_string(), m.score)); }

        CompactUrlSummary { url: r.url.clone(), score: r.score, grade: r.grade.clone(), top_issues: top_issue_titles, module_scores }
    }).collect();

    let mut sorted_by_score: Vec<_> = batch.reports.iter().collect();
    sorted_by_score.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
    let worst_urls: Vec<(String, f32)> = sorted_by_score.iter().take(3).map(|r| (truncate_url(&r.url, 60), r.score)).collect();
    let best_urls: Vec<(String, f32)> = sorted_by_score.iter().rev().take(3).map(|r| (truncate_url(&r.url, 60), r.score)).collect();

    let verdict_text = build_batch_verdict(batch);

    let severity_distribution = {
        let (mut critical, mut serious, mut moderate, mut minor) = (0usize, 0usize, 0usize, 0usize);
        for (violation, _) in &all_violations {
            match violation.severity {
                Severity::Critical => critical += 1, Severity::Serious => serious += 1,
                Severity::Moderate => moderate += 1, Severity::Minor => minor += 1,
            }
        }
        SeverityDistribution { critical, serious, moderate, minor }
    };

    BatchPresentation {
        cover: CoverData {
            title: "Web Accessibility Batch Audit Report".to_string(),
            url: format!("{} URLs geprüft", batch.summary.total_urls),
            date: chrono::Utc::now().format("%d.%m.%Y").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        portfolio_summary: PortfolioSummary {
            total_urls: batch.summary.total_urls, passed: batch.summary.passed,
            failed: batch.summary.failed, average_score: batch.summary.average_score,
            total_violations: batch.summary.total_violations, duration_ms: batch.total_duration_ms,
            verdict_text, worst_urls, best_urls, severity_distribution,
        },
        top_issues: top_issues.into_iter().take(10).collect(),
        issue_frequency, action_plan, url_ranking, url_details,
        appendix: build_batch_appendix(batch),
    }
}

// ─── Internal helpers ───────────────────────────────────────────────────────

struct GroupAccumulator {
    rule: String,
    rule_name: String,
    severity: Severity,
    count: usize,
    urls: Vec<String>,
}

fn capitalize_severity(severity: &Severity) -> String {
    match severity {
        Severity::Critical => "Critical".to_string(),
        Severity::Serious => "Serious".to_string(),
        Severity::Moderate => "Moderate".to_string(),
        Severity::Minor => "Minor".to_string(),
    }
}

fn group_violations(violations: &[crate::wcag::Violation], _url_context: &[&str]) -> Vec<FindingGroup> {
    let mut groups: HashMap<String, (Vec<&crate::wcag::Violation>, usize)> = HashMap::new();
    for v in violations {
        let entry = groups.entry(v.rule.clone()).or_insert_with(|| (Vec::new(), 0));
        entry.0.push(v);
        entry.1 += 1;
    }

    groups.into_iter().map(|(rule_id, (violations, count))| {
        let first = violations[0];
        let explanation = get_explanation(&rule_id);

        let (title, customer_desc, user_impact_text, typical_cause, recommendation, technical_note, role, effort) =
            if let Some(expl) = explanation {
                (expl.customer_title.to_string(), expl.customer_description.to_string(),
                 expl.user_impact.to_string(), expl.typical_cause.to_string(),
                 expl.recommendation.to_string(), expl.technical_note.to_string(),
                 expl.responsible_role, expl.effort_estimate)
            } else {
                (format!("{} — {}", first.rule, first.rule_name), first.message.clone(),
                 "Nutzer mit Einschränkungen können betroffen sein.".to_string(),
                 "Automatisch erkanntes Problem.".to_string(),
                 first.fix_suggestion.clone().unwrap_or_else(|| "Bitte prüfen und beheben.".to_string()),
                 first.fix_suggestion.clone().unwrap_or_default(), Role::Development, Effort::Medium)
            };

        let examples = explanation.map(|e| e.examples()).unwrap_or_default();

        FindingGroup {
            title, wcag_criterion: rule_id, wcag_level: format!("{:?}", first.level),
            severity: first.severity, priority: severity_to_priority(first.severity),
            customer_description: customer_desc, user_impact: user_impact_text, typical_cause,
            recommendation, technical_note, occurrence_count: count,
            affected_urls: Vec::new(), affected_elements: count,
            responsible_role: role, effort, examples,
        }
    }).collect()
}

fn build_finding_group_from_accumulator(acc: &GroupAccumulator) -> FindingGroup {
    let explanation = get_explanation(&acc.rule);
    let (title, customer_desc, user_impact_text, typical_cause, recommendation, technical_note, role, effort) =
        if let Some(expl) = explanation {
            (expl.customer_title.to_string(), expl.customer_description.to_string(),
             expl.user_impact.to_string(), expl.typical_cause.to_string(),
             expl.recommendation.to_string(), expl.technical_note.to_string(),
             expl.responsible_role, expl.effort_estimate)
        } else {
            (format!("{} — {}", acc.rule, acc.rule_name), String::new(), String::new(),
             String::new(), String::new(), String::new(), Role::Development, Effort::Medium)
        };
    let examples = explanation.map(|e| e.examples()).unwrap_or_default();

    FindingGroup {
        title, wcag_criterion: acc.rule.clone(), wcag_level: String::new(),
        severity: acc.severity, priority: severity_to_priority(acc.severity),
        customer_description: customer_desc, user_impact: user_impact_text, typical_cause,
        recommendation, technical_note, occurrence_count: acc.count,
        affected_urls: acc.urls.clone(), affected_elements: acc.count,
        responsible_role: role, effort, examples,
    }
}

fn severity_to_priority(severity: Severity) -> Priority {
    match severity {
        Severity::Critical => Priority::Critical,
        Severity::Serious => Priority::High,
        Severity::Moderate => Priority::Medium,
        Severity::Minor => Priority::Low,
    }
}

fn score_to_priority(score: f32) -> Priority {
    if score < 50.0 { Priority::Critical }
    else if score < 70.0 { Priority::High }
    else if score < 85.0 { Priority::Medium }
    else { Priority::Low }
}

fn impact_score(group: &FindingGroup) -> u32 {
    let severity_weight = match group.severity {
        Severity::Critical => 4, Severity::Serious => 3,
        Severity::Moderate => 2, Severity::Minor => 1,
    };
    severity_weight * group.occurrence_count as u32
}

fn build_verdict_text(url: &str, score: f32) -> String {
    if score >= 90.0 {
        format!("Die Website {} erreicht mit {:.0}/100 Punkten ein sehr gutes Ergebnis. \
                 Die Barrierefreiheit ist weitgehend gewährleistet.", url, score)
    } else if score >= 70.0 {
        format!("Die Website {} erreicht {:.0}/100 Punkte — eine solide Basis, \
                 aber mit relevanten Barrieren, die behoben werden sollten.", url, score)
    } else if score >= 50.0 {
        format!("Die Website {} erreicht nur {:.0}/100 Punkte. \
                 Es bestehen erhebliche Barrierefreiheitsprobleme, die zeitnah behoben werden müssen.", url, score)
    } else {
        format!("Die Website {} erreicht nur {:.0}/100 Punkte. \
                 Die Barrierefreiheit ist stark eingeschränkt — dringender Handlungsbedarf.", url, score)
    }
}

fn derive_positive_aspects(report: &AuditReport, a11y_score: f32) -> Vec<PositiveAspect> {
    let mut positives = Vec::new();

    if report.wcag_results.violations.is_empty() {
        positives.push(PositiveAspect { area: "Barrierefreiheit".into(),
            description: "Keine automatisch erkennbaren Verstöße gefunden.".into() });
    } else if a11y_score >= 80.0 {
        positives.push(PositiveAspect { area: "Barrierefreiheit".into(),
            description: format!("Guter Score von {:.0}/100 — die Basis stimmt.", a11y_score) });
    }

    if let Some(ref perf) = report.performance {
        if perf.score.overall >= 80 {
            positives.push(PositiveAspect { area: "Performance".into(),
                description: format!("Gute Ladezeiten mit {}/100 Punkten.", perf.score.overall) });
        }
    }
    if let Some(ref seo) = report.seo {
        if seo.score >= 80 {
            positives.push(PositiveAspect { area: "SEO".into(),
                description: format!("Solide SEO-Basis mit {}/100 Punkten.", seo.score) });
        }
    }
    if let Some(ref sec) = report.security {
        if sec.score >= 80 {
            positives.push(PositiveAspect { area: "Sicherheit".into(),
                description: format!("Gute Security-Konfiguration mit {}/100 Punkten (Grade {}).", sec.score, sec.grade) });
        }
    }
    if let Some(ref mobile) = report.mobile {
        if mobile.score >= 80 {
            positives.push(PositiveAspect { area: "Mobile".into(),
                description: format!("Gute mobile Nutzbarkeit mit {}/100 Punkten.", mobile.score) });
        }
    }

    if positives.is_empty() {
        positives.push(PositiveAspect { area: "Grundstruktur".into(),
            description: "Die Seite ist grundsätzlich funktional und erreichbar.".into() });
    }
    positives
}

fn derive_action_plan(finding_groups: &[FindingGroup]) -> ActionPlan {
    let mut quick_wins = Vec::new();
    let mut medium_term = Vec::new();
    let mut structural = Vec::new();

    for group in finding_groups {
        let item = ActionItem {
            action: group.recommendation.clone(),
            benefit: group.user_impact.clone(),
            role: group.responsible_role,
            priority: group.priority,
        };
        match group.effort {
            Effort::Quick => quick_wins.push(item),
            Effort::Medium => medium_term.push(item),
            Effort::Structural => structural.push(item),
        }
    }

    quick_wins.sort_by(|a, b| b.priority.cmp(&a.priority));
    medium_term.sort_by(|a, b| b.priority.cmp(&a.priority));
    structural.sort_by(|a, b| b.priority.cmp(&a.priority));

    let mut role_map: HashMap<Role, Vec<String>> = HashMap::new();
    for group in finding_groups {
        role_map.entry(group.responsible_role).or_default().push(group.title.clone());
    }
    role_map.entry(Role::ProjectManagement).or_default().extend([
        "Priorisierung der Maßnahmen".to_string(),
        "Qualitätssicherung und Testing".to_string(),
        "Verantwortlichkeiten festlegen".to_string(),
    ]);

    let role_assignments: Vec<RoleAssignment> = role_map.into_iter().map(|(role, mut responsibilities)| {
        responsibilities.dedup();
        RoleAssignment { role, responsibilities }
    }).collect();

    ActionPlan { quick_wins, medium_term, structural, role_assignments }
}

fn find_best_module_name(report: &AuditReport, a11y_score: f32) -> String {
    let mut best = ("Barrierefreiheit", a11y_score.round() as u32);
    if let Some(ref p) = report.performance { if p.score.overall > best.1 { best = ("Performance", p.score.overall); } }
    if let Some(ref s) = report.seo { if s.score > best.1 { best = ("SEO", s.score); } }
    if let Some(ref s) = report.security { if s.score > best.1 { best = ("Sicherheit", s.score); } }
    if let Some(ref m) = report.mobile { if m.score > best.1 { best = ("Mobile", m.score); } }
    best.0.to_string()
}

fn grade_label(score: u32) -> &'static str {
    match score {
        90..=100 => "Sehr gut", 75..=89 => "Gut", 60..=74 => "Befriedigend",
        40..=59 => "Ausbaufähig", _ => "Kritisch",
    }
}

fn interpret_score(score: f32, area: &str) -> String {
    let grade = grade_label(score.round() as u32);
    match grade {
        "Sehr gut" => format!("{} — die {} ist auf einem hohen Niveau.", grade, area),
        "Gut" => format!("{} — die {} ist solide, einzelne Verbesserungen sind möglich.", grade, area),
        "Befriedigend" => format!("{} — die {} weist einzelne Schwächen auf.", grade, area),
        "Ausbaufähig" => format!("{} — die {} weist relevante Schwächen auf.", grade, area),
        _ => format!("{} — die {} hat erhebliche Mängel, die behoben werden sollten.", grade, area),
    }
}

fn build_batch_verdict(batch: &BatchReport) -> String {
    let avg = batch.summary.average_score;
    if avg >= 90.0 {
        format!("Über {} geprüfte URLs hinweg erreicht die Website einen durchschnittlichen \
                 Accessibility-Score von {:.0}/100 — ein sehr gutes Ergebnis.", batch.summary.total_urls, avg)
    } else if avg >= 70.0 {
        format!("Im Durchschnitt erreichen die {} geprüften URLs {:.0}/100 Punkte. \
                 Die Basis ist solide, es bestehen aber wiederkehrende Barrieren.", batch.summary.total_urls, avg)
    } else if avg >= 50.0 {
        format!("Die {} geprüften URLs erreichen im Schnitt nur {:.0}/100 Punkte. \
                 Es bestehen erhebliche systematische Barrierefreiheitsprobleme.", batch.summary.total_urls, avg)
    } else {
        format!("Die {} geprüften URLs erreichen im Schnitt nur {:.0}/100 Punkte. \
                 Die Barrierefreiheit ist stark eingeschränkt — dringender Handlungsbedarf.", batch.summary.total_urls, avg)
    }
}

fn build_batch_appendix(batch: &BatchReport) -> BatchAppendixData {
    BatchAppendixData {
        per_url: batch.reports.iter().map(|r| {
            // Aggregate violations by rule for each URL
            let mut rule_map: std::collections::HashMap<String, AppendixViolation> = std::collections::HashMap::new();
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
                    rule_map.insert(v.rule.clone(), AppendixViolation {
                        rule: v.rule.clone(),
                        rule_name: v.rule_name.clone(),
                        severity: capitalize_severity(&v.severity),
                        message: v.message.clone(),
                        fix_suggestion: v.fix_suggestion.clone(),
                        affected_elements: vec![element],
                    });
                }
            }

            UrlAppendix {
                url: r.url.clone(),
                violations: rule_order.into_iter()
                    .filter_map(|rule| rule_map.remove(&rule))
                    .collect(),
            }
        }).collect(),
    }
}

fn yes_no(val: bool) -> String {
    if val { "Ja".to_string() } else { "Nein".to_string() }
}

// ─── Clone implementations ──────────────────────────────────────────────────

impl Clone for FindingGroup {
    fn clone(&self) -> Self {
        FindingGroup {
            title: self.title.clone(), wcag_criterion: self.wcag_criterion.clone(),
            wcag_level: self.wcag_level.clone(), severity: self.severity, priority: self.priority,
            customer_description: self.customer_description.clone(),
            user_impact: self.user_impact.clone(), typical_cause: self.typical_cause.clone(),
            recommendation: self.recommendation.clone(), technical_note: self.technical_note.clone(),
            occurrence_count: self.occurrence_count, affected_urls: self.affected_urls.clone(),
            affected_elements: self.affected_elements, responsible_role: self.responsible_role,
            effort: self.effort, examples: self.examples.clone(),
        }
    }
}

impl Clone for ExampleBlock {
    fn clone(&self) -> Self {
        ExampleBlock { bad: self.bad.clone(), good: self.good.clone(), decorative: self.decorative.clone() }
    }
}

impl Clone for RoleAssignment {
    fn clone(&self) -> Self {
        RoleAssignment { role: self.role, responsibilities: self.responsibilities.clone() }
    }
}
