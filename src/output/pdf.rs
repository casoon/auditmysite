//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

use renderreport::components::advanced::{KeyValueList, List, PageBreak, TableOfContents};
use renderreport::components::text::{Label, Text};
use renderreport::prelude::*;
use renderreport::theme::{Theme, TokenValue};
use renderreport::Engine;

// Composite components
use renderreport::components::{
    ActionRoadmap, BenchmarkRow, BenchmarkSummary, BenchmarkTable, ComparisonModule, CoverPage,
    DashboardModule, HeroMetric, HeroSummary, ModuleComparison, ModuleDashboard, RoadmapColumn,
    RoadmapItem, SeverityOverview,
};

use crate::audit::{AuditReport, BatchReport};
use crate::cli::ReportLevel;
use crate::output::report_builder::{build_batch_presentation, build_view_model};
use crate::output::report_model::*;
use crate::util::truncate_url;

/// Create engine with proper font configuration for German text
fn create_engine() -> anyhow::Result<Engine> {
    let mut engine = Engine::new()?;

    let mut theme = Theme::default_theme();
    theme.tokens.set("font.body", TokenValue::Font("Helvetica Neue".into()));
    theme.tokens.set("font.heading", TokenValue::Font("Helvetica Neue".into()));
    theme.tokens.set("font.mono", TokenValue::Font("Menlo".into()));
    engine.set_default_theme(theme);

    Ok(engine)
}

/// Map our severity to renderreport severity
fn map_severity(severity: &crate::wcag::Severity) -> Severity {
    match severity {
        crate::wcag::Severity::Critical => Severity::Critical,
        crate::wcag::Severity::Serious => Severity::High,
        crate::wcag::Severity::Moderate => Severity::Medium,
        crate::wcag::Severity::Minor => Severity::Low,
    }
}

// ─── Single Report ──────────────────────────────────────────────────────────

pub fn generate_pdf(report: &AuditReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let vm = build_view_model(report, config);

    let mut builder = engine
        .report("wcag-audit")
        .title(&vm.meta.title)
        .subtitle(&vm.meta.subtitle)
        .metadata("date", &vm.meta.date)
        .metadata("version", &vm.meta.version)
        .metadata("author", &vm.meta.author)
        .metadata("score", &vm.meta.score_label);

    // ── Cover Page ───────────────────────────────────────────────────
    let cover = CoverPage::new(&vm.cover.title, &vm.cover.domain, vm.cover.score, &vm.cover.grade)
        .with_brand(&vm.cover.brand)
        .with_date(&vm.cover.date)
        .with_issues(vm.cover.total_issues, vm.cover.critical_issues)
        .with_modules(vm.cover.modules);

    builder = builder.add_component(cover);

    if vm.meta.report_level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    // ── 1. Kurzfazit (Hero Summary) ──────────────────────────────────
    let mut hero = HeroSummary::new(vm.summary.score, &vm.summary.grade, &vm.summary.domain)
        .with_date(&vm.summary.date)
        .with_verdict(&vm.summary.verdict)
        .with_top_actions(vm.summary.top_actions)
        .with_positive_aspects(vm.summary.positive_aspects)
        .with_thresholds(70, 50);

    for m in vm.summary.metrics {
        hero = hero.add_metric(HeroMetric {
            title: m.title,
            value: m.value,
            accent_color: m.accent_color,
        });
    }

    builder = builder
        .add_component(Section::new("Kurzfazit").with_level(1))
        .add_component(hero)
        .add_component(PageBreak::new());

    // ── 2. Methodik ──────────────────────────────────────────────────
    builder = builder
        .add_component(Section::new("Prüfumfang und Methodik").with_level(1))
        .add_component(Text::new(&vm.methodology.scope))
        .add_component(Text::new(&vm.methodology.method))
        .add_component(Callout::info(&vm.methodology.limitations).with_title("Grenzen automatisierter Tests"))
        .add_component(Callout::warning(&vm.methodology.disclaimer).with_title("Hinweis"));

    // Executive level: compact view
    if vm.meta.report_level == ReportLevel::Executive {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Bewertung nach Modulen").with_level(1));

        if vm.modules.dashboard.len() > 1 {
            builder = builder.add_component(build_module_comparison(&vm.modules));
        }

        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Maßnahmenplan").with_level(1))
            .add_component(Text::new(&vm.actions.intro_text))
            .add_component(build_action_roadmap(&vm.actions));

        let built_report = builder.build();
        return Ok(engine.render_pdf(&built_report)?);
    }

    builder = builder.add_component(PageBreak::new());

    // ── 3. Score Breakdown (Module Dashboard) ────────────────────────
    let dashboard_modules: Vec<DashboardModule> = vm.modules.dashboard.iter().map(|m| {
        DashboardModule {
            name: m.name.clone(),
            score: m.score,
            interpretation: m.interpretation.clone(),
            good_threshold: m.good_threshold,
            warn_threshold: m.warn_threshold,
        }
    }).collect();

    builder = builder
        .add_component(Section::new("Bewertung nach Modulen").with_level(1))
        .add_component(ModuleDashboard::new(dashboard_modules));

    if vm.modules.dashboard.len() > 1 {
        builder = builder.add_component(build_module_comparison(&vm.modules));
    }

    // ── 4. Findings ──────────────────────────────────────────────────
    match vm.meta.report_level {
        ReportLevel::Technical => {
            builder = builder
                .add_component(PageBreak::new())
                .add_component(Section::new("Übersicht der Probleme").with_level(1));

            if !vm.severity.has_issues {
                builder = builder.add_component(
                    Callout::success("Keine automatisch erkennbaren Barrierefreiheitsprobleme gefunden.")
                        .with_title("Ausgezeichnete Barrierefreiheit"));
            } else {
                builder = builder.add_component(SeverityOverview::new(
                    vm.severity.critical, vm.severity.serious, vm.severity.moderate, vm.severity.minor));
                for group in vm.findings.top_findings.iter().take(5) {
                    builder = render_finding_compact(builder, group);
                }
            }

            if vm.severity.has_issues {
                builder = builder
                    .add_component(PageBreak::new())
                    .add_component(Section::new("Technische Detailanalyse").with_level(1));
                for group in &vm.findings.all_findings {
                    builder = render_finding_technical(builder, group);
                }
            }
        }
        _ => {
            builder = builder
                .add_component(PageBreak::new())
                .add_component(Section::new("Erkannte Probleme").with_level(1));

            if !vm.severity.has_issues {
                builder = builder.add_component(
                    Callout::success("Keine automatisch erkennbaren Barrierefreiheitsprobleme gefunden.")
                        .with_title("Ausgezeichnete Barrierefreiheit"));
            } else {
                builder = builder.add_component(SeverityOverview::new(
                    vm.severity.critical, vm.severity.serious, vm.severity.moderate, vm.severity.minor));
                for group in &vm.findings.all_findings {
                    builder = render_finding_compact(builder, group);
                }
            }
        }
    }

    // ── 5. Module Details ────────────────────────────────────────────
    if vm.module_details.has_any {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Weitere Analysen").with_level(1))
            .add_component(Text::new(
                "Neben der Barrierefreiheit wurden weitere Aspekte der Website analysiert. \
                 Die folgenden Abschnitte zeigen die Ergebnisse der Performance-, SEO-, \
                 Sicherheits- und Mobile-Analyse im Detail."));
    }

    if let Some(ref perf) = vm.module_details.performance {
        builder = render_performance(builder, perf);
    }
    if let Some(ref seo) = vm.module_details.seo {
        builder = render_seo(builder, seo);
    }
    if let Some(ref sec) = vm.module_details.security {
        builder = render_security(builder, sec);
    }
    if let Some(ref mobile) = vm.module_details.mobile {
        builder = render_mobile(builder, mobile);
    }

    // ── 6. Maßnahmenplan ─────────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Maßnahmenplan").with_level(1))
        .add_component(Text::new(&vm.actions.intro_text))
        .add_component(build_action_roadmap(&vm.actions));

    // ── 7. Anhang ────────────────────────────────────────────────────
    if vm.appendix.has_violations {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Anhang: Technische Details").with_level(1))
            .add_component(Text::new("Die folgende Tabelle enthält alle erkannten Verstöße mit \
                 technischen Details für die Umsetzung."))
            .add_component(Text::new(&vm.appendix.score_methodology));

        if vm.meta.report_level == ReportLevel::Technical {
            for v in &vm.appendix.violations {
                let mut desc = v.message.clone();
                if let Some(ref fix) = v.fix_suggestion {
                    desc.push_str(&format!("\n\nEmpfohlener Fix: {}", fix));
                }
                desc.push_str(&format!("\n\nVorkommen: {} Elemente betroffen", v.affected_elements.len()));
                // Only show selectors that look like real CSS selectors (contain . # [ or >)
                let useful_selectors: Vec<&str> = v.affected_elements.iter()
                    .map(|e| e.selector.as_str())
                    .filter(|s| s.contains('.') || s.contains('#') || s.contains('[') || s.contains('>') || s.contains(' '))
                    .collect();
                if !useful_selectors.is_empty() {
                    desc.push_str(&format!("\nSelektoren: {}", useful_selectors.join(", ")));
                }
                builder = builder.add_component(
                    Finding::new(
                        &format!("{} — {}", v.rule, v.rule_name),
                        match v.severity.to_lowercase().as_str() {
                            "critical" => Severity::Critical,
                            "serious" => Severity::High,
                            "moderate" => Severity::Medium,
                            _ => Severity::Low,
                        },
                        &desc,
                    ));
            }
        } else {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Regel"),
                TableColumn::new("Schweregrad"),
                TableColumn::new("Beschreibung"),
                TableColumn::new("Vorkommen"),
            ]).with_title("Alle Verstöße (aggregiert nach Regel)");

            for v in &vm.appendix.violations {
                table = table.add_row(vec![
                    format!("{} — {}", v.rule, v.rule_name),
                    v.severity.clone(),
                    v.message.clone(),
                    format!("{} Elemente", v.affected_elements.len()),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    let built_report = builder.build();
    Ok(engine.render_pdf(&built_report)?)
}

// ─── Batch Report ───────────────────────────────────────────────────────────

pub fn generate_batch_pdf(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let pres = build_batch_presentation(batch);

    let author = config.company_name.as_deref().unwrap_or("AuditMySite");

    let mut builder = engine
        .report("wcag-batch-audit")
        .title(&pres.cover.title)
        .subtitle(&pres.cover.url)
        .metadata("date", &pres.cover.date)
        .metadata("version", &pres.cover.version)
        .metadata("author", author);

    if let Some(ref logo_path) = config.logo_path {
        if logo_path.exists() {
            builder = builder.add_component(
                Image::new(logo_path.to_string_lossy().to_string()).with_width("30%"));
        }
    }

    builder = builder.add_component(PageBreak::new());
    if config.level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    // ── 1. Portfolio Overview ────────────────────────────────────────
    let best = pres.url_ranking.last();
    let worst = pres.url_ranking.first();
    let dist = &pres.portfolio_summary.severity_distribution;

    let mut benchmark = BenchmarkSummary::new(
        pres.portfolio_summary.total_urls as u32,
        pres.portfolio_summary.average_score.round() as u32,
    ).with_issues(pres.portfolio_summary.total_violations as u32, (dist.critical + dist.serious) as u32);

    if let Some(b) = best { benchmark = benchmark.with_best(&truncate_url(&b.url, 35), b.score as u32); }
    if let Some(w) = worst { benchmark = benchmark.with_worst(&truncate_url(&w.url, 35), w.score as u32); }

    builder = builder
        .add_component(Section::new("Gesamtübersicht").with_level(1))
        .add_component(benchmark)
        .add_component(Text::new(&pres.portfolio_summary.verdict_text))
        .add_component(PageBreak::new());

    // ── 2. URL-Ranking ──────────────────────────────────────────────
    let rows: Vec<BenchmarkRow> = pres.url_ranking.iter().enumerate().map(|(i, u)| {
        BenchmarkRow::new((i + 1) as u32, &truncate_url(&u.url, 35), u.score as u32, u.score as u32, u.critical_violations as u32)
    }).collect();

    builder = builder
        .add_component(Section::new("URL-Ranking").with_level(1))
        .add_component(Text::new("Übersicht aller geprüften URLs, sortiert nach Score. \
             URLs mit niedrigerem Score haben höheren Handlungsbedarf."))
        .add_component(BenchmarkTable::new(rows))
        .add_component(PageBreak::new());

    // ── 3. Top-Probleme ─────────────────────────────────────────────
    builder = builder
        .add_component(Section::new("Häufigste Probleme").with_level(1))
        .add_component(Text::new("Die folgenden Problemgruppen treten über mehrere URLs hinweg auf. \
             Durch Behebung dieser Probleme wird die größte Verbesserung erzielt, \
             da sie viele Seiten gleichzeitig betreffen."));

    if !pres.issue_frequency.is_empty() {
        let mut freq_table = AuditTable::new(vec![
            TableColumn::new("Problem"), TableColumn::new("WCAG"),
            TableColumn::new("Vorkommen"), TableColumn::new("Betr. URLs"),
            TableColumn::new("Priorität"),
        ]).with_title("Häufigste Verstöße");

        for issue in &pres.issue_frequency {
            freq_table = freq_table.add_row(vec![
                issue.problem.clone(), issue.wcag.clone(),
                issue.occurrences.to_string(), issue.affected_urls.to_string(),
                issue.priority.label().to_string(),
            ]);
        }
        builder = builder.add_component(freq_table);
    }

    for group in &pres.top_issues {
        builder = render_finding_group(builder, group);
    }

    // ── 4. Maßnahmenplan ────────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Maßnahmenplan").with_level(1))
        .add_component(Text::new("Die folgenden Maßnahmen sind nach Aufwand und Wirkung priorisiert. \
             Maßnahmen, die viele Seiten gleichzeitig verbessern, haben Vorrang."));
    builder = render_action_plan(builder, &pres.action_plan);

    // ── 5. URL-Summaries ────────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Einzelergebnisse je URL").with_level(1))
        .add_component(Text::new("Kompakte Zusammenfassung der Ergebnisse pro geprüfter URL. \
             Detaillierte technische Daten finden sich im Anhang."));

    for detail in &pres.url_details {
        builder = builder
            .add_component(Section::new(&truncate_url(&detail.url, 70)).with_level(2))
            .add_component(ScoreCard::new("Score", detail.score.round() as u32)
                .with_description(&format!("Note: {}", detail.grade)).with_thresholds(70, 50));

        if !detail.module_scores.is_empty() {
            let mut scores = SummaryBox::new("Modul-Scores");
            for (module, score) in &detail.module_scores {
                scores = scores.add_item(module, &format!("{}/100", score));
            }
            builder = builder.add_component(scores);
        }

        if !detail.top_issues.is_empty() {
            let mut issues = List::new().with_title("Wichtigste Probleme");
            for issue in &detail.top_issues { issues = issues.add_item(issue); }
            builder = builder.add_component(issues);
        }
    }

    // ── 6. Anhang ───────────────────────────────────────────────────
    if !pres.appendix.per_url.is_empty() {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Anhang: Technische Details").with_level(1))
            .add_component(Text::new("Vollständige Auflistung aller erkannten Verstöße pro URL \
                 mit technischen Details für die Umsetzung."));

        for url_appendix in &pres.appendix.per_url {
            if url_appendix.violations.is_empty() { continue; }

            builder = builder.add_component(Section::new(&truncate_url(&url_appendix.url, 70)).with_level(2));

            let mut table = AuditTable::new(vec![
                TableColumn::new("Regel"), TableColumn::new("Schweregrad"),
                TableColumn::new("Beschreibung"), TableColumn::new("Betr. Elemente"),
            ]);

            for v in &url_appendix.violations {
                let elements = v.affected_elements.iter()
                    .map(|e| e.selector.clone())
                    .collect::<Vec<_>>()
                    .join("; ");
                table = table.add_row(vec![
                    format!("{} — {} ({}×)", v.rule, v.rule_name, v.affected_elements.len()),
                    v.severity.clone(),
                    v.message.clone(),
                    elements,
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    let built_report = builder.build();
    Ok(engine.render_pdf(&built_report)?)
}

// ─── Component mapping helpers ──────────────────────────────────────────────

fn build_module_comparison(modules: &ModulesBlock) -> ModuleComparison {
    let comparison_modules: Vec<ComparisonModule> = modules.dashboard.iter()
        .map(|m| ComparisonModule::new(&m.name, m.score))
        .collect();
    ModuleComparison::new(comparison_modules)
}

fn build_action_roadmap(actions: &ActionsBlock) -> ActionRoadmap {
    let columns: Vec<RoadmapColumn> = actions.roadmap_columns.iter().map(|c| {
        RoadmapColumn {
            title: c.title.clone(),
            accent_color: Some(c.accent_color.clone()),
            items: c.items.iter().map(|i| RoadmapItem {
                action: i.action.clone(),
                role: i.role.clone(),
                priority: i.priority.clone(),
                effort: Some(i.effort.clone()),
                benefit: i.benefit.clone(),
            }).collect(),
        }
    }).collect();
    ActionRoadmap::new(columns)
}

// ─── Finding renderers ──────────────────────────────────────────────────────

fn render_finding_compact(
    builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
) -> renderreport::engine::ReportBuilder {
    let finding = Finding::new(
        &format!("{} (WCAG {})", group.title, group.wcag_criterion),
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(&format!("Priorität: {} | {} Vorkommen", group.priority.label(), group.occurrence_count))
    .with_affected(&format!("{} Elemente betroffen", group.affected_elements));

    builder.add_component(finding)
}

fn render_finding_technical(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        Label::new(&format!("{} — WCAG {} ({})", group.title, group.wcag_criterion, group.wcag_level))
            .bold().with_size("14pt"));

    let finding = Finding::new(&group.title, map_severity(&group.severity), &group.customer_description)
        .with_recommendation(&group.recommendation)
        .with_category(&format!("Priorität: {} | Zuständig: {} | Aufwand: {}",
            group.priority.label(), group.responsible_role.label(), group.effort.label()))
        .with_affected(&format!("{} Vorkommen, {} Elemente", group.occurrence_count, group.affected_elements));

    builder = builder.add_component(finding);

    if !group.technical_note.is_empty() {
        builder = builder.add_component(Callout::info(&group.technical_note).with_title("Technischer Hinweis"));
    }

    for example in &group.examples {
        builder = builder
            .add_component(Callout::error(&example.bad).with_title("Falsch"))
            .add_component(Callout::success(&example.good).with_title("Richtig"));
        if let Some(ref dec) = example.decorative {
            builder = builder.add_component(Callout::info(dec).with_title("Dekorativ"));
        }
    }

    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title("Betroffene URLs");
        for url in &group.affected_urls { url_list = url_list.add_item(&truncate_url(url, 70)); }
        builder = builder.add_component(url_list);
    }

    builder
}

fn render_finding_group(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        Section::new(&format!("{} (WCAG {})", group.title, group.wcag_criterion)).with_level(2));

    if matches!(group.severity, crate::wcag::Severity::Critical | crate::wcag::Severity::Serious) {
        builder = builder.add_component(
            Callout::error(&group.customer_description)
                .with_title(&format!("{} — Priorität: {}", group.title, group.priority.label())));
    }

    let mut finding = Finding::new(&group.title, map_severity(&group.severity), &group.customer_description)
        .with_recommendation(&group.recommendation)
        .with_category(&format!("Priorität: {} | Zuständig: {} | Aufwand: {}",
            group.priority.label(), group.responsible_role.label(), group.effort.label()));

    if group.occurrence_count > 0 {
        finding = finding.with_affected(&format!("{} Vorkommen, {} Elemente betroffen{}",
            group.occurrence_count, group.affected_elements,
            if group.affected_urls.is_empty() { String::new() }
            else { format!(", {} URLs", group.affected_urls.len()) }));
    }

    builder = builder.add_component(finding);

    if !group.user_impact.is_empty() {
        builder = builder.add_component(Callout::info(&group.user_impact).with_title("Auswirkung auf Nutzer"));
    }
    if !group.typical_cause.is_empty() {
        builder = builder.add_component(Text::new(&format!("Typische Ursache: {}", group.typical_cause)));
    }
    if !group.technical_note.is_empty() {
        builder = builder.add_component(Text::new(&format!("Technischer Hinweis: {}", group.technical_note)));
    }

    if !group.examples.is_empty() {
        builder = builder.add_component(Section::new("Codebeispiel").with_level(3));
        for example in &group.examples {
            builder = builder
                .add_component(Callout::error(&example.bad).with_title("Falsch"))
                .add_component(Callout::success(&example.good).with_title("Richtig"));
            if let Some(ref dec) = example.decorative {
                builder = builder.add_component(Callout::info(dec).with_title("Dekorativ"));
            }
        }
    }

    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title("Betroffene URLs");
        for url in &group.affected_urls { url_list = url_list.add_item(&truncate_url(url, 70)); }
        builder = builder.add_component(url_list);
    } else if group.affected_urls.len() > 10 {
        builder = builder.add_component(Text::new(&format!(
            "Betrifft {} URLs (zu viele für Einzelauflistung — siehe Anhang).", group.affected_urls.len())));
    }

    builder
}

// ─── Module detail renderers ────────────────────────────────────────────────

fn render_performance(mut builder: renderreport::engine::ReportBuilder, perf: &PerformancePresentation) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(Section::new("Performance").with_level(2))
        .add_component(Text::new(&perf.interpretation))
        .add_component(ScoreCard::new("Performance Score", perf.score)
            .with_description(&format!("Grade: {}", perf.grade)).with_thresholds(75, 50));

    if !perf.vitals.is_empty() {
        let mut kv = KeyValueList::new().with_title("Core Web Vitals");
        for (name, value, rating) in &perf.vitals { kv = kv.add(name, &format!("{} — {}", value, rating)); }
        builder = builder.add_component(kv);
    }

    if !perf.additional_metrics.is_empty() {
        let mut metrics = SummaryBox::new("Weitere Metriken");
        for (k, v) in &perf.additional_metrics { metrics = metrics.add_item(k, v); }
        builder = builder.add_component(metrics);
    }
    builder
}

fn render_seo(mut builder: renderreport::engine::ReportBuilder, seo: &SeoPresentation) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("SEO-Analyse").with_level(2))
        .add_component(Text::new(&seo.interpretation))
        .add_component(ScoreCard::new("SEO Score", seo.score).with_thresholds(80, 50));

    if !seo.meta_tags.is_empty() {
        let mut kv = KeyValueList::new().with_title("Meta-Tags");
        for (k, v) in &seo.meta_tags { kv = kv.add(k, v); }
        builder = builder.add_component(kv);
    }

    if !seo.meta_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Feld"), TableColumn::new("Schweregrad"), TableColumn::new("Beschreibung"),
        ]).with_title("Meta-Tag Probleme");
        for (field, sev, msg) in &seo.meta_issues {
            table = table.add_row(vec![field.as_str(), sev.as_str(), msg.as_str()]);
        }
        builder = builder.add_component(table);
    }

    builder = builder
        .add_component(Text::new(&seo.heading_summary))
        .add_component(Text::new(&seo.social_summary));

    if !seo.technical_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title("Technisches SEO");
        for (k, v) in &seo.technical_summary { kv = kv.add(k, v); }
        builder = builder.add_component(kv);
    }
    builder
}

fn render_security(mut builder: renderreport::engine::ReportBuilder, sec: &SecurityPresentation) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Sicherheit").with_level(2))
        .add_component(Text::new(&sec.interpretation))
        .add_component(ScoreCard::new("Security Score", sec.score)
            .with_description(&format!("Grade: {}", sec.grade)).with_thresholds(70, 50));

    if !sec.headers.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Header"), TableColumn::new("Status"), TableColumn::new("Wert"),
        ]).with_title("Security Headers");
        for (name, status, val) in &sec.headers {
            table = table.add_row(vec![name.as_str(), status.as_str(), val.as_str()]);
        }
        builder = builder.add_component(table);
    }

    if !sec.ssl_info.is_empty() {
        let mut kv = KeyValueList::new().with_title("SSL/TLS");
        for (k, v) in &sec.ssl_info { kv = kv.add(k, v); }
        builder = builder.add_component(kv);
    }

    for (title, sev, msg) in &sec.issues {
        let severity = match sev.as_str() {
            "critical" => Severity::Critical, "high" => Severity::High,
            "medium" => Severity::Medium, _ => Severity::Low,
        };
        builder = builder.add_component(Finding::new(title, severity, msg));
    }

    if !sec.recommendations.is_empty() {
        let mut rec_list = List::new().with_title("Empfehlungen");
        for rec in &sec.recommendations { rec_list = rec_list.add_item(rec); }
        builder = builder.add_component(rec_list);
    }
    builder
}

fn render_mobile(mut builder: renderreport::engine::ReportBuilder, mobile: &MobilePresentation) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Mobile Nutzbarkeit").with_level(2))
        .add_component(Text::new(&mobile.interpretation))
        .add_component(ScoreCard::new("Mobile Score", mobile.score).with_thresholds(80, 50));

    if !mobile.viewport.is_empty() {
        let mut kv = KeyValueList::new().with_title("Viewport-Konfiguration");
        for (k, v) in &mobile.viewport { kv = kv.add(k, v); }
        builder = builder.add_component(kv);
    }
    if !mobile.touch_targets.is_empty() {
        let mut box_ = SummaryBox::new("Touch Targets");
        for (k, v) in &mobile.touch_targets { box_ = box_.add_item(k, v); }
        builder = builder.add_component(box_);
    }
    if !mobile.font_analysis.is_empty() {
        let mut kv = KeyValueList::new().with_title("Schriftanalyse");
        for (k, v) in &mobile.font_analysis { kv = kv.add(k, v); }
        builder = builder.add_component(kv);
    }
    if !mobile.content_sizing.is_empty() {
        let mut box_ = SummaryBox::new("Content Sizing");
        for (k, v) in &mobile.content_sizing { box_ = box_.add_item(k, v); }
        builder = builder.add_component(box_);
    }

    for (cat, sev, msg) in &mobile.issues {
        let severity = match sev.as_str() {
            "critical" => Severity::Critical, "high" => Severity::High,
            "medium" => Severity::Medium, _ => Severity::Low,
        };
        builder = builder.add_component(Finding::new(cat, severity, msg));
    }
    builder
}

/// Render action plan for batch reports (using AuditTable)
fn render_action_plan(
    mut builder: renderreport::engine::ReportBuilder,
    plan: &ActionPlan,
) -> renderreport::engine::ReportBuilder {
    if !plan.quick_wins.is_empty() {
        builder = builder.add_component(Section::new("Quick Wins").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"), TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"), TableColumn::new("Priorität"),
        ]);
        for item in &plan.quick_wins {
            table = table.add_row(vec![item.action.clone(), item.benefit.clone(),
                item.role.label().to_string(), item.priority.label().to_string()]);
        }
        builder = builder.add_component(table);
    }
    if !plan.medium_term.is_empty() {
        builder = builder.add_component(Section::new("Mittelfristige Maßnahmen").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"), TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"), TableColumn::new("Priorität"),
        ]);
        for item in &plan.medium_term {
            table = table.add_row(vec![item.action.clone(), item.benefit.clone(),
                item.role.label().to_string(), item.priority.label().to_string()]);
        }
        builder = builder.add_component(table);
    }
    if !plan.structural.is_empty() {
        builder = builder.add_component(Section::new("Strukturelle Maßnahmen").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"), TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"), TableColumn::new("Priorität"),
        ]);
        for item in &plan.structural {
            table = table.add_row(vec![item.action.clone(), item.benefit.clone(),
                item.role.label().to_string(), item.priority.label().to_string()]);
        }
        builder = builder.add_component(table);
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_url() {
        assert_eq!(truncate_url("https://example.com/very/long/path/that/exceeds/limit", 30), "https://example.com/very/lo...");
        assert_eq!(truncate_url("https://example.com", 30), "https://example.com");
    }
}
