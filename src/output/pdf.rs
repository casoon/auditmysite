//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

use renderreport::components::advanced::{Grid, KeyValueList, List, PageBreak, TableOfContents};
use renderreport::components::text::{Label, TextBlock};
use renderreport::prelude::*;
use renderreport::theme::{Theme, TokenValue};
use renderreport::Engine;

// Composite components
use renderreport::components::{
    BenchmarkRow, BenchmarkSummary, BenchmarkTable, ComparisonModule, MetricCard, ModuleComparison,
    ScoreCard, SeverityOverview, SummaryBox,
};

use crate::audit::{normalize, AuditReport, BatchReport};
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::report_builder::{build_batch_presentation, build_view_model};
use crate::output::report_model::*;
use crate::util::truncate_url;

/// Create engine with proper font configuration for German text
fn create_engine() -> anyhow::Result<Engine> {
    let mut engine = Engine::new()?;

    let mut theme = Theme::default_theme();
    theme
        .tokens
        .set("font.body", TokenValue::Font("Helvetica".into()));
    theme
        .tokens
        .set("font.heading", TokenValue::Font("Helvetica".into()));
    theme
        .tokens
        .set("font.mono", TokenValue::Font("Courier".into()));
    engine.set_default_theme(theme);

    Ok(engine)
}

/// Map our severity to renderreport severity
fn map_severity(severity: &crate::wcag::Severity) -> Severity {
    match severity {
        crate::wcag::Severity::Critical => Severity::Critical,
        crate::wcag::Severity::High => Severity::High,
        crate::wcag::Severity::Medium => Severity::Medium,
        crate::wcag::Severity::Low => Severity::Low,
    }
}

fn severity_label_i18n(severity: crate::wcag::Severity, i18n: &I18n) -> String {
    match severity {
        crate::wcag::Severity::Critical => i18n.t("severity-critical"),
        crate::wcag::Severity::High => i18n.t("severity-high"),
        crate::wcag::Severity::Medium => i18n.t("severity-medium"),
        crate::wcag::Severity::Low => i18n.t("severity-low"),
    }
}

fn priority_label_i18n(priority: Priority, i18n: &I18n) -> String {
    match priority {
        Priority::Critical => i18n.t("priority-critical"),
        Priority::High => i18n.t("priority-high"),
        Priority::Medium => i18n.t("priority-medium"),
        Priority::Low => i18n.t("priority-low"),
    }
}

fn role_label_i18n(role: Role, i18n: &I18n) -> String {
    match role {
        Role::Development => i18n.t("role-development"),
        Role::Editorial => i18n.t("role-editorial"),
        Role::DesignUx => i18n.t("role-designux"),
        Role::ProjectManagement => i18n.t("role-projectmanagement"),
    }
}

fn effort_label_i18n(effort: Effort, i18n: &I18n) -> String {
    match effort {
        Effort::Quick => i18n.t("effort-quick"),
        Effort::Medium => i18n.t("effort-medium"),
        Effort::Structural => i18n.t("effort-structural"),
    }
}

// ─── Single Report ──────────────────────────────────────────────────────────

pub fn generate_pdf(report: &AuditReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let normalized = normalize(report);
    let vm = build_view_model(&normalized, config);
    let i18n = I18n::new(&config.locale)?;

    let mut builder = engine
        .report("wcag-audit")
        .metadata("date", &vm.meta.date)
        .metadata("version", &vm.meta.version)
        .metadata("author", &vm.meta.author)
        .metadata("score", &vm.meta.score_label);

    // ── Cover Page ───────────────────────────────────────────────────
    builder = builder
        .add_component(
            Label::new(&vm.cover.brand)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(Label::new(&vm.cover.title).with_size("28pt").bold())
        .add_component(
            Label::new(
                "Technischer Website-Check mit Fokus auf Accessibility, SEO und Performance",
            )
            .with_size("12pt")
            .with_color("#475569"),
        )
        .add_component(
            Label::new(&vm.cover.domain)
                .with_size("16pt")
                .with_color("#0f766e"),
        )
        .add_component(build_cover_meta(&vm.cover))
        .add_component(build_cover_score_row(&vm.cover))
        .add_component(
            TextBlock::new(&vm.summary.verdict)
                .with_size("11pt")
                .with_line_height("1.4em")
                .with_max_width("100%"),
        )
        .add_component(PageBreak::new());

    if vm.meta.report_level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    // ── 1. Executive Summary ────────────────────────────────────────
    builder = builder
        .add_component(Section::new("Executive Summary").with_level(1))
        .add_component(build_summary_overview(&vm.summary))
        .add_component(build_executive_highlights_table(&vm))
        .add_component(Callout::info(build_executive_recommendation(&vm)).with_title("Empfehlung"));

    // Executive level: compact view
    if vm.meta.report_level == ReportLevel::Executive {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Key Findings").with_level(1));

        for (idx, group) in vm.findings.top_findings.iter().take(3).enumerate() {
            if idx > 0 {
                builder = builder.add_component(PageBreak::new());
            }
            builder = render_key_finding_block(builder, group, &i18n);
        }

        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new(i18n.t("section-modules")).with_level(1))
            .add_component(build_module_dashboard(&vm.modules))
            .add_component(build_module_summary_table(&vm.modules));

        if vm.modules.dashboard.len() > 1 {
            builder = builder.add_component(build_module_comparison(&vm.modules));
        }

        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Maßnahmenplan").with_level(1))
            .add_component(TextBlock::new(&vm.actions.intro_text));
        builder = render_action_plan_tables(builder, &vm.actions);

        if let Some(ref history) = vm.history {
            builder = render_history_section(builder, history);
        }

        builder = render_methodology_section(builder, &vm.methodology, &i18n);

        let built_report = builder.build();
        return Ok(engine.render_pdf(&built_report)?);
    }

    builder = builder.add_component(PageBreak::new());

    // ── 2. Key Findings ─────────────────────────────────────────────
    builder = builder
        .add_component(Section::new("Key Findings").with_level(1))
        .add_component(TextBlock::new(
            "Diese Themen sollten Sie zuerst angehen. Jeder Block zeigt kurz Problem, Relevanz und die empfohlene nächste Maßnahme.",
        ));

    for (idx, group) in vm.findings.top_findings.iter().take(5).enumerate() {
        if idx > 0 {
            builder = builder.add_component(PageBreak::new());
        }
        builder = render_key_finding_block(builder, group, &i18n);
    }

    // ── 3. Quick Wins ───────────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Quick Wins").with_level(1))
        .add_component(TextBlock::new(
            "Diese Punkte sind mit vergleichsweise wenig Aufwand umsetzbar und verbessern die Seite schnell sichtbar.",
        ))
        .add_component(build_quick_wins_table(&vm.actions));

    // ── 4. Module Overview ──────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Modul-Übersicht").with_level(1))
        .add_component(build_module_dashboard(&vm.modules))
        .add_component(build_module_summary_table(&vm.modules));

    if let Some(ref overall_text) = vm.modules.overall_interpretation {
        builder =
            builder.add_component(Callout::info(overall_text).with_title("Gesamtscore einordnen"));
    }

    if vm.modules.dashboard.len() > 1 {
        builder = builder.add_component(build_module_comparison(&vm.modules));
    }

    // ── 5. Maßnahmenplan ────────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Maßnahmenplan").with_level(1))
        .add_component(TextBlock::new(&vm.actions.intro_text));
    builder = render_action_plan_tables(builder, &vm.actions);

    // ── 6. Historie ─────────────────────────────────────────────────
    if let Some(ref history) = vm.history {
        builder = render_history_section(builder, history);
    }

    // ── 7. Technischer Detailteil ───────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Technischer Detailteil").with_level(1))
        .add_component(TextBlock::new(
            "Ab hier folgt die technische Sicht für Entwicklung, Design und Redaktion. Die Abschnitte nennen betroffene Elemente, konkrete Maßnahmen und technische Hinweise.",
        ));

    builder = builder
        .add_component(Section::new("Problem-Details").with_level(2))
        .add_component(if !vm.severity.has_issues {
            Callout::success(i18n.t("callout-no-issues-body"))
                .with_title(i18n.t("callout-no-issues-title"))
        } else {
            Callout::info(
                "Die folgende Übersicht enthält die relevanten Problemgruppen mit technischer Einordnung.",
            )
            .with_title("Technische Übersicht")
        });

    if vm.severity.has_issues {
        builder = builder.add_component(SeverityOverview::new(
            vm.severity.critical,
            vm.severity.high,
            vm.severity.medium,
            vm.severity.low,
        ));
        for group in &vm.findings.all_findings {
            builder = render_finding_technical(builder, group, &i18n);
        }
    }

    // ── 8. Technische Metriken ──────────────────────────────────────
    if vm.module_details.has_any {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Technische Metriken").with_level(2))
            .add_component(TextBlock::new(
                "Zusätzliche technische Kennzahlen helfen bei Performance-, SEO-, Sicherheits- und Mobile-Themen. Die Werte sind als Arbeitsgrundlage für die Umsetzung gedacht.",
            ))
            .add_component(build_additional_analyses_overview(&vm.module_details))
            .add_component(build_analysis_focus_table());
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

    // ── 9. Anhang ───────────────────────────────────────────────────
    if vm.appendix.has_violations {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new(i18n.t("section-appendix")).with_level(1))
            .add_component(TextBlock::new(
                "Die folgende Tabelle enthält alle erkannten Verstöße mit \
                 technischen Details für die Umsetzung.",
            ))
            .add_component(TextBlock::new(&vm.appendix.score_methodology));

        if vm.meta.report_level == ReportLevel::Technical {
            for v in &vm.appendix.violations {
                let mut desc = v.message.clone();
                if let Some(ref fix) = v.fix_suggestion {
                    desc.push_str(&format!("\n\nEmpfohlener Fix: {}", fix));
                }
                desc.push_str(&format!(
                    "\n\nVorkommen: {} Elemente betroffen",
                    v.affected_elements.len()
                ));
                // Only show selectors that look like real CSS selectors (contain . # [ or >)
                let useful_selectors: Vec<&str> = v
                    .affected_elements
                    .iter()
                    .map(|e| e.selector.as_str())
                    .filter(|s| {
                        s.contains('.')
                            || s.contains('#')
                            || s.contains('[')
                            || s.contains('>')
                            || s.contains(' ')
                    })
                    .collect();
                if !useful_selectors.is_empty() {
                    desc.push_str(&format!("\nSelektoren: {}", useful_selectors.join(", ")));
                }
                builder = builder.add_component(Finding::new(
                    format!("{} — {}", v.rule, v.rule_name),
                    map_severity(&v.severity),
                    &desc,
                ));
            }
        } else {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Regel"),
                TableColumn::new("Schweregrad"),
                TableColumn::new("Beschreibung"),
                TableColumn::new("Vorkommen"),
            ])
            .with_title("Alle Verstöße (aggregiert nach Regel)");

            for v in &vm.appendix.violations {
                table = table.add_row(vec![
                    format!("{} — {}", v.rule, v.rule_name),
                    severity_label_i18n(v.severity, &i18n),
                    v.message.clone(),
                    format!("{} Elemente", v.affected_elements.len()),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    // ── 10. Methodik & Einschränkungen ──────────────────────────────
    builder = render_methodology_section(builder, &vm.methodology, &i18n);

    let built_report = builder.build();
    Ok(engine.render_pdf(&built_report)?)
}

// ─── Batch Report ───────────────────────────────────────────────────────────

pub fn generate_batch_pdf(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let i18n = I18n::new(&config.locale)?;
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
                Image::new(logo_path.to_string_lossy().to_string()).with_width("30%"),
            );
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
    )
    .with_issues(
        pres.portfolio_summary.total_violations as u32,
        (dist.critical + dist.high) as u32,
    );

    if let Some(b) = best {
        benchmark = benchmark.with_best(&truncate_url(&b.url, 35), b.score as u32);
    }
    if let Some(w) = worst {
        benchmark = benchmark.with_worst(&truncate_url(&w.url, 35), w.score as u32);
    }

    builder = builder
        .add_component(Section::new("Gesamtübersicht").with_level(1))
        .add_component(benchmark)
        .add_component(TextBlock::new(&pres.portfolio_summary.verdict_text))
        .add_component(PageBreak::new());

    // ── 2. URL-Ranking ──────────────────────────────────────────────
    let rows: Vec<BenchmarkRow> = pres
        .url_ranking
        .iter()
        .enumerate()
        .map(|(i, u)| {
            BenchmarkRow::new(
                (i + 1) as u32,
                &truncate_url(&u.url, 35),
                u.score as u32,
                u.score as u32,
                u.critical_violations as u32,
            )
        })
        .collect();

    builder = builder
        .add_component(Section::new("URL-Ranking").with_level(1))
        .add_component(TextBlock::new(
            "Übersicht aller geprüften URLs, sortiert nach Score. \
             URLs mit niedrigerem Score haben höheren Handlungsbedarf.",
        ))
        .add_component(BenchmarkTable::new(rows))
        .add_component(PageBreak::new());

    // ── 3. Top-Probleme ─────────────────────────────────────────────
    builder = builder
        .add_component(Section::new("Häufigste Probleme").with_level(1))
        .add_component(TextBlock::new(
            "Die folgenden Problemgruppen treten über mehrere URLs hinweg auf. \
             Durch Behebung dieser Probleme wird die größte Verbesserung erzielt, \
             da sie viele Seiten gleichzeitig betreffen.",
        ));

    if !pres.issue_frequency.is_empty() {
        let mut freq_table = AuditTable::new(vec![
            TableColumn::new("Problem"),
            TableColumn::new("WCAG"),
            TableColumn::new("Vorkommen"),
            TableColumn::new("Betr. URLs"),
            TableColumn::new("Priorität"),
        ])
        .with_title("Häufigste Verstöße");

        for issue in &pres.issue_frequency {
            freq_table = freq_table.add_row(vec![
                issue.problem.clone(),
                issue.wcag.clone(),
                issue.occurrences.to_string(),
                issue.affected_urls.to_string(),
                priority_label_i18n(issue.priority, &i18n),
            ]);
        }
        builder = builder.add_component(freq_table);
    }

    for group in &pres.top_issues {
        builder = render_finding_group(builder, group, &i18n);
    }

    // ── 4. Maßnahmenplan ────────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Maßnahmenplan").with_level(1))
        .add_component(TextBlock::new(
            "Die folgenden Maßnahmen sind nach Aufwand und Wirkung priorisiert. \
             Maßnahmen, die viele Seiten gleichzeitig verbessern, haben Vorrang.",
        ));
    builder = render_action_plan(builder, &pres.action_plan, &i18n);

    // ── 5. URL-Summaries ────────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Einzelergebnisse je URL").with_level(1))
        .add_component(TextBlock::new(
            "Kompakte Zusammenfassung der Ergebnisse pro geprüfter URL. \
             Detaillierte technische Daten finden sich im Anhang.",
        ));

    for detail in &pres.url_details {
        builder = builder
            .add_component(Section::new(truncate_url(&detail.url, 70)).with_level(2))
            .add_component(
                ScoreCard::new("Score", detail.score.round() as u32)
                    .with_description(format!("Note: {}", detail.grade))
                    .with_thresholds(70, 50),
            );

        if !detail.module_scores.is_empty() {
            let mut scores = SummaryBox::new("Modul-Scores");
            for (module, score) in &detail.module_scores {
                scores = scores.add_item(module, format!("{}/100", score));
            }
            builder = builder.add_component(scores);
        }

        if !detail.top_issues.is_empty() {
            let mut issues = List::new().with_title("Wichtigste Probleme");
            for issue in &detail.top_issues {
                issues = issues.add_item(issue);
            }
            builder = builder.add_component(issues);
        }
    }

    // ── 6. Anhang ───────────────────────────────────────────────────
    if !pres.appendix.per_url.is_empty() {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Anhang: Technische Details").with_level(1))
            .add_component(TextBlock::new(
                "Vollständige Auflistung aller erkannten Verstöße pro URL \
                 mit technischen Details für die Umsetzung.",
            ));

        for url_appendix in &pres.appendix.per_url {
            if url_appendix.violations.is_empty() {
                continue;
            }

            builder = builder
                .add_component(Section::new(truncate_url(&url_appendix.url, 70)).with_level(2));

            let mut table = AuditTable::new(vec![
                TableColumn::new("Regel"),
                TableColumn::new("Schweregrad"),
                TableColumn::new("Beschreibung"),
                TableColumn::new("Betr. Elemente"),
            ]);

            for v in &url_appendix.violations {
                let elements = v
                    .affected_elements
                    .iter()
                    .map(|e| e.selector.clone())
                    .collect::<Vec<_>>()
                    .join("; ");
                table = table.add_row(vec![
                    format!(
                        "{} — {} ({}×)",
                        v.rule,
                        v.rule_name,
                        v.affected_elements.len()
                    ),
                    severity_label_i18n(v.severity, &i18n),
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
    let comparison_modules: Vec<ComparisonModule> = modules
        .dashboard
        .iter()
        .map(|m| ComparisonModule::new(&m.name, m.score))
        .collect();
    ModuleComparison::new(comparison_modules)
}

fn build_summary_overview(summary: &SummaryBlock) -> Grid {
    let score_card = serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new("Accessibility-Score", summary.score)
            .with_description(format!("Grade {} — {}", summary.grade, summary.certificate))
            .with_thresholds(70, 50)
            .to_data()
    });
    let mut grid = Grid::new(2).add_item(score_card);

    for metric in summary.metrics.iter().take(3) {
        let mut card = MetricCard::new(&metric.title, &metric.value);
        if let Some(ref color) = metric.accent_color {
            card = card.with_accent_color(color);
        }
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": card.to_data()
        }));
    }

    grid
}

fn render_action_plan_tables(
    mut builder: renderreport::engine::ReportBuilder,
    actions: &ActionsBlock,
) -> renderreport::engine::ReportBuilder {
    for column in &actions.roadmap_columns {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Priorität"),
            TableColumn::new("Aufwand"),
        ])
        .with_title(&column.title);

        for item in &column.items {
            table = table.add_row(vec![
                item.action.clone(),
                item.execution_priority.clone(),
                item.effort.clone(),
            ]);
        }

        builder = builder.add_component(table);
    }

    builder
}

fn build_executive_highlights_table(vm: &ReportViewModel) -> AuditTable {
    let problems = if vm.findings.top_findings.is_empty() {
        vec!["Keine priorisierten Probleme im automatischen Test erkannt.".to_string()]
    } else {
        vm.findings
            .top_findings
            .iter()
            .take(3)
            .map(|finding| {
                format!(
                    "{}: {}",
                    finding.title,
                    simplify_for_summary(&finding.customer_description)
                )
            })
            .collect()
    };

    let strengths = if vm.summary.positive_aspects.is_empty() {
        vec!["Die Seite ist grundsätzlich erreichbar und technisch auswertbar.".to_string()]
    } else {
        vm.summary
            .positive_aspects
            .iter()
            .take(3)
            .cloned()
            .collect()
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new("Wichtigste Probleme"),
        TableColumn::new("Wichtigste Stärken"),
    ])
    .with_title("Kernaussagen");

    let max_rows = problems.len().max(strengths.len());
    for idx in 0..max_rows {
        table = table.add_row(vec![
            problems.get(idx).cloned().unwrap_or_default(),
            strengths.get(idx).cloned().unwrap_or_default(),
        ]);
    }

    table
}

fn build_executive_recommendation(vm: &ReportViewModel) -> String {
    let lead = if let Some(first) = vm.findings.top_findings.first() {
        first.recommendation.clone()
    } else {
        "Die aktuelle Qualität sichern und die automatischen Prüfungen regelmäßig wiederholen."
            .to_string()
    };

    format!(
        "Zuerst die {} kritischsten Themen beheben, danach die Quick Wins umsetzen. Startpunkt: {}",
        vm.findings.top_findings.len().min(3),
        simplify_for_summary(&lead)
    )
}

fn build_module_summary_table(modules: &ModulesBlock) -> AuditTable {
    let mut table = AuditTable::new(vec![
        TableColumn::new("Modul"),
        TableColumn::new("Score"),
        TableColumn::new("Größter Hebel"),
    ])
    .with_title("Kurzbewertung je Modul");

    for module in &modules.dashboard {
        table = table.add_row(vec![
            module.name.clone(),
            format!("{}/100", module.score),
            simplify_for_summary(&module.key_lever),
        ]);
    }

    table
}

fn build_quick_wins_table(actions: &ActionsBlock) -> AuditTable {
    let quick_wins: Vec<&RoadmapItemData> = actions
        .roadmap_columns
        .iter()
        .filter(|column| column.title.contains("Sofort") || column.title.contains("Quick"))
        .flat_map(|column| column.items.iter())
        .filter(|item| matches!(item.priority.as_str(), "Kritisch" | "Hoch"))
        .take(5)
        .collect();

    let selected = if quick_wins.is_empty() {
        actions
            .roadmap_columns
            .iter()
            .flat_map(|column| column.items.iter())
            .take(5)
            .collect()
    } else {
        quick_wins
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new("Titel"),
        TableColumn::new("Was tun"),
        TableColumn::new("Warum lohnt sich das"),
    ]);

    for item in selected {
        table = table.add_row(vec![
            quick_win_title(item),
            item.action.clone(),
            simplify_for_summary(&item.execution_priority),
        ]);
    }

    table
}

fn render_history_section(
    mut builder: renderreport::engine::ReportBuilder,
    history: &HistoryTrendBlock,
) -> renderreport::engine::ReportBuilder {
    let mut kv = KeyValueList::new().with_title("Trend zum letzten Lauf");
    for (key, value) in &history.metrics {
        kv = kv.add(key, value);
    }

    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Historie und Trend").with_level(1))
        .add_component(TextBlock::new(&history.summary))
        .add_component(kv);

    if !history.timeline_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Datum"),
            TableColumn::new("Accessibility"),
            TableColumn::new("Gesamt"),
            TableColumn::new("Note"),
            TableColumn::new("Issues"),
        ])
        .with_title("Verlauf der letzten Läufe");

        for row in &history.timeline_rows {
            table = table.add_row(vec![
                row.0.clone(),
                row.1.clone(),
                row.2.clone(),
                row.3.clone(),
                row.4.clone(),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !history.new_findings.is_empty() {
        let mut list = List::new().with_title("Neue Themen seit dem letzten Lauf");
        for finding in &history.new_findings {
            list = list.add_item(finding);
        }
        builder = builder.add_component(list);
    }

    if !history.resolved_findings.is_empty() {
        let mut list = List::new().with_title("Behobene Themen seit dem letzten Lauf");
        for finding in &history.resolved_findings {
            list = list.add_item(finding);
        }
        builder = builder.add_component(list);
    }

    builder
}

fn render_methodology_section(
    mut builder: renderreport::engine::ReportBuilder,
    methodology: &MethodologyBlock,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let mut table = AuditTable::new(vec![TableColumn::new("Aspekt"), TableColumn::new("Wert")])
        .with_title("Audit-Kontext");

    for (key, value) in &methodology.audit_facts {
        table = table.add_row(vec![key.clone(), value.clone()]);
    }

    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Methodik & Einschränkungen").with_level(1))
        .add_component(TextBlock::new(&methodology.scope))
        .add_component(TextBlock::new(&methodology.method))
        .add_component(table)
        .add_component(
            Callout::info(&methodology.limitations).with_title(i18n.t("callout-limitations-title")),
        )
        .add_component(
            Callout::warning(&methodology.disclaimer).with_title(i18n.t("callout-note-title")),
        )
        .add_component(TextBlock::new(i18n.t("certificate-thresholds")));

    builder
}

fn build_cover_meta(cover: &CoverBlock) -> SummaryBox {
    SummaryBox::new("Audit-Rahmen")
        .add_item("Kunde", &cover.brand)
        .add_item("Prüfdatum", &cover.date)
        .add_item("Ziel", &cover.domain)
        .add_item("Zertifikat", &cover.certificate)
        .add_item("Aktive Module", cover.modules.join(", "))
}

fn build_cover_score_row(cover: &CoverBlock) -> Grid {
    let mut grid = Grid::new(3);

    grid = grid.add_item(serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new("Accessibility", cover.score)
            .with_description("Primärscore")
            .with_thresholds(70, 50)
            .to_data()
    }));

    grid = grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("Grade", &cover.grade)
            .with_subtitle(format!("Zertifikat: {}", cover.certificate))
            .with_accent_color("#0f766e")
            .to_data()
    }));

    grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("Issues", cover.total_issues.to_string())
            .with_subtitle(format!("{} kritisch", cover.critical_issues))
            .with_accent_color("#dc2626")
            .to_data()
    }))
}

fn build_module_dashboard(modules: &ModulesBlock) -> Grid {
    let columns = match modules.dashboard.len() {
        0 | 1 => 1,
        2 => 2,
        _ => 3,
    };

    let mut grid = Grid::new(columns);

    for module in &modules.dashboard {
        let status = if module.score >= module.good_threshold {
            "Sehr gut"
        } else if module.score >= module.warn_threshold {
            "Solide"
        } else {
            "Ausbaufähig"
        };

        let accent = if module.score >= module.good_threshold {
            "#16a34a"
        } else if module.score >= module.warn_threshold {
            "#d97706"
        } else {
            "#dc2626"
        };

        let card = MetricCard::new(&module.name, format!("{} / 100", module.score))
            .with_subtitle(status)
            .with_accent_color(accent);

        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": card.to_data()
        }));
    }

    grid
}

fn build_additional_analyses_overview(details: &ModuleDetailsBlock) -> Grid {
    let mut grid = Grid::new(2);

    if let Some(ref perf) = details.performance {
        grid = grid.add_item(metric_card_json(
            "Performance",
            perf.score,
            "Ladezeit, Interaktivität und technische Effizienz",
        ));
    }

    if let Some(ref seo) = details.seo {
        grid = grid.add_item(metric_card_json(
            "SEO",
            seo.score,
            "Meta-Daten, Struktur und Suchmaschinen-Signale",
        ));
    }

    if let Some(ref sec) = details.security {
        grid = grid.add_item(metric_card_json(
            "Sicherheit",
            sec.score,
            "Header, HTTPS-Konfiguration und Schutzmechanismen",
        ));
    }

    if let Some(ref mobile) = details.mobile {
        grid = grid.add_item(metric_card_json(
            "Mobile",
            mobile.score,
            "Viewport, Touch Targets und Lesbarkeit auf kleinen Displays",
        ));
    }

    grid
}

fn build_analysis_focus_table() -> AuditTable {
    AuditTable::new(vec![TableColumn::new("Modul"), TableColumn::new("Fokus")])
        .with_title("Analysefokus")
        .add_row(vec![
            "Performance".to_string(),
            "Nutzerwahrnehmung, Ladezeit und Reaktionsverhalten".to_string(),
        ])
        .add_row(vec![
            "SEO".to_string(),
            "Indexierbarkeit, Struktur und inhaltliche Signale".to_string(),
        ])
        .add_row(vec![
            "Sicherheit".to_string(),
            "HTTP-Header, TLS-Setup und fehlende Schutzmechanismen".to_string(),
        ])
        .add_row(vec![
            "Mobile".to_string(),
            "Bedienbarkeit, Responsiveness und Lesbarkeit".to_string(),
        ])
}

fn metric_card_json(title: &str, score: u32, subtitle: &str) -> serde_json::Value {
    let (status, accent) = score_status(score, 80, 50);
    serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new(title, format!("{} / 100", score))
            .with_subtitle(format!("{} · {}", status, subtitle))
            .with_accent_color(accent)
            .to_data()
    })
}

fn score_status(
    score: u32,
    good_threshold: u32,
    warn_threshold: u32,
) -> (&'static str, &'static str) {
    if score >= good_threshold {
        ("Sehr gut", "#16a34a")
    } else if score >= warn_threshold {
        ("Solide", "#d97706")
    } else {
        ("Ausbaufähig", "#dc2626")
    }
}

// ─── Finding renderers ──────────────────────────────────────────────────────

fn render_key_finding_block(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let category = group
        .dimension
        .as_deref()
        .or(group.issue_class.as_deref())
        .unwrap_or("Barrierefreiheit");

    let details = AuditTable::new(vec![
        TableColumn::new("Aspekt"),
        TableColumn::new("Einordnung"),
    ])
    .add_row(vec![
        "Problem".to_string(),
        simplify_for_summary(&group.customer_description),
    ])
    .add_row(vec![
        "Warum relevant".to_string(),
        simplify_for_summary(&group.user_impact),
    ])
    .add_row(vec![
        "Business-Auswirkung".to_string(),
        simplify_for_summary(&group.business_impact),
    ])
    .add_row(vec![
        "Maßnahme".to_string(),
        simplify_for_summary(&group.recommendation),
    ])
    .add_row(vec![
        "Priorität".to_string(),
        execution_priority_label(group.execution_priority).to_string(),
    ])
    .add_row(vec![
        "Aufwand".to_string(),
        effort_label_i18n(group.effort, i18n),
    ])
    .add_row(vec!["Impact".to_string(), impact_label(group).to_string()])
    .add_row(vec![
        "Rolle".to_string(),
        role_label_i18n(group.responsible_role, i18n),
    ]);

    builder = builder
        .add_component(
            Label::new(format!(
                "[{} | {}] {}",
                severity_label_i18n(group.severity, i18n),
                category,
                group.title
            ))
            .bold()
            .with_size("13pt"),
        )
        .add_component(details);

    builder
}

fn render_finding_technical(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let header = if !group.wcag_criterion.is_empty() {
        format!(
            "{} — WCAG {} ({})",
            group.title, group.wcag_criterion, group.wcag_level
        )
    } else {
        format!("{} — {}", group.title, group.rule_id)
    };
    builder = builder.add_component(Label::new(&header).bold().with_size("14pt"));

    let mut category_parts = vec![
        format!(
            "{}: {}",
            i18n.t("label-priority"),
            priority_label_i18n(group.priority, i18n)
        ),
        format!(
            "{}: {}",
            i18n.t("label-owner"),
            role_label_i18n(group.responsible_role, i18n)
        ),
        format!(
            "{}: {}",
            i18n.t("label-effort"),
            effort_label_i18n(group.effort, i18n)
        ),
    ];
    if let Some(ref dim) = group.dimension {
        category_parts.push(format!("{}: {}", i18n.t("label-module"), dim));
    }
    if let Some(ref cls) = group.issue_class {
        category_parts.push(format!("{}: {}", i18n.t("label-type"), cls));
    }

    let finding = Finding::new(
        &group.title,
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(category_parts.join(" | "))
    .with_affected(format!(
        "{} Vorkommen, {} Elemente",
        group.occurrence_count, group.affected_elements
    ));

    builder = builder.add_component(finding);

    let mut details = KeyValueList::new().with_title("Technische Einordnung");
    details = details
        .add(
            "WCAG-Regel",
            if group.wcag_criterion.is_empty() {
                "—".to_string()
            } else {
                group.wcag_criterion.clone()
            },
        )
        .add("Schweregrad", severity_label_i18n(group.severity, i18n))
        .add(
            "Betroffene Elemente",
            format!("{}", group.affected_elements),
        )
        .add(
            "Technische Empfehlung",
            simplify_for_summary(&group.recommendation),
        )
        .add(
            "Business-Auswirkung",
            simplify_for_summary(&group.business_impact),
        )
        .add(
            "Umsetzungspriorität",
            execution_priority_label(group.execution_priority).to_string(),
        );
    builder = builder.add_component(details);

    if !group.technical_note.is_empty() {
        builder = builder.add_component(
            Callout::info(&group.technical_note).with_title(i18n.t("label-tech-note")),
        );
    }

    for example in &group.examples {
        builder = builder
            .add_component(Callout::error(&example.bad).with_title(i18n.t("label-wrong")))
            .add_component(Callout::success(&example.good).with_title(i18n.t("label-right")));
        if let Some(ref dec) = example.decorative {
            builder =
                builder.add_component(Callout::info(dec).with_title(i18n.t("label-decorative")));
        }
    }

    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title(i18n.t("label-affected-urls"));
        for url in &group.affected_urls {
            url_list = url_list.add_item(truncate_url(url, 70));
        }
        builder = builder.add_component(url_list);
    }

    builder
}

fn simplify_for_summary(text: &str) -> String {
    let single_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = single_line.trim();
    if trimmed.ends_with('.') {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
}

fn impact_label(group: &FindingGroup) -> &'static str {
    match (group.priority, group.severity) {
        (Priority::Critical, _) | (_, crate::wcag::Severity::Critical) => "Hoch",
        (Priority::High, _) | (_, crate::wcag::Severity::High) => "Hoch",
        (Priority::Medium, _) | (_, crate::wcag::Severity::Medium) => "Mittel",
        _ => "Niedrig",
    }
}

fn execution_priority_label(priority: ExecutionPriority) -> &'static str {
    match priority {
        ExecutionPriority::Immediate => "Sofort beheben",
        ExecutionPriority::Important => "Wichtig",
        ExecutionPriority::Optional => "Optional",
    }
}

fn quick_win_title(item: &RoadmapItemData) -> String {
    let title = item
        .action
        .split(['.', ':'])
        .next()
        .unwrap_or(item.action.as_str())
        .trim();
    let shortened: String = title.chars().take(48).collect();
    if title.chars().count() > 48 {
        format!("{shortened}…")
    } else {
        shortened
    }
}

fn render_finding_group(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let section_title = if !group.wcag_criterion.is_empty() {
        format!("{} (WCAG {})", group.title, group.wcag_criterion)
    } else if let Some(ref dim) = group.dimension {
        format!("{} ({})", group.title, dim)
    } else {
        group.title.clone()
    };
    builder = builder.add_component(Section::new(&section_title).with_level(2));

    if matches!(
        group.severity,
        crate::wcag::Severity::Critical | crate::wcag::Severity::High
    ) {
        builder = builder.add_component(Callout::error(&group.customer_description).with_title(
            format!(
                "{} — {}: {}",
                group.title,
                i18n.t("label-priority"),
                priority_label_i18n(group.priority, i18n)
            ),
        ));
    }

    let mut finding = Finding::new(
        &group.title,
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(format!(
        "{}: {} | {}: {} | {}: {}",
        i18n.t("label-priority"),
        priority_label_i18n(group.priority, i18n),
        i18n.t("label-owner"),
        role_label_i18n(group.responsible_role, i18n),
        i18n.t("label-effort"),
        effort_label_i18n(group.effort, i18n)
    ));

    if group.occurrence_count > 0 {
        finding = finding.with_affected(format!(
            "{} Vorkommen, {} Elemente betroffen{}",
            group.occurrence_count,
            group.affected_elements,
            if group.affected_urls.is_empty() {
                String::new()
            } else {
                format!(", {} URLs", group.affected_urls.len())
            }
        ));
    }

    builder = builder.add_component(finding);

    if !group.user_impact.is_empty() {
        builder = builder.add_component(
            Callout::info(&group.user_impact).with_title(i18n.t("label-user-impact")),
        );
    }
    if !group.typical_cause.is_empty() {
        builder = builder.add_component(TextBlock::new(format!(
            "{}: {}",
            i18n.t("label-typical-cause"),
            group.typical_cause
        )));
    }
    if !group.technical_note.is_empty() {
        builder = builder.add_component(TextBlock::new(format!(
            "{}: {}",
            i18n.t("label-tech-note"),
            group.technical_note
        )));
    }

    if !group.examples.is_empty() {
        builder = builder.add_component(Section::new(i18n.t("label-code-example")).with_level(3));
        for example in &group.examples {
            builder = builder
                .add_component(Callout::error(&example.bad).with_title(i18n.t("label-wrong")))
                .add_component(Callout::success(&example.good).with_title(i18n.t("label-right")));
            if let Some(ref dec) = example.decorative {
                builder = builder
                    .add_component(Callout::info(dec).with_title(i18n.t("label-decorative")));
            }
        }
    }

    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title(i18n.t("label-affected-urls"));
        for url in &group.affected_urls {
            url_list = url_list.add_item(truncate_url(url, 70));
        }
        builder = builder.add_component(url_list);
    } else if group.affected_urls.len() > 10 {
        builder = builder.add_component(TextBlock::new(format!(
            "Betrifft {} URLs (zu viele für Einzelauflistung — siehe Anhang).",
            group.affected_urls.len()
        )));
    }

    builder
}

// ─── Module detail renderers ────────────────────────────────────────────────

fn render_performance(
    mut builder: renderreport::engine::ReportBuilder,
    perf: &PerformancePresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(Section::new("Performance").with_level(2))
        .add_component(TextBlock::new(&perf.interpretation))
        .add_component(
            ScoreCard::new("Performance Score", perf.score)
                .with_description(format!("Grade: {}", perf.grade))
                .with_thresholds(75, 50),
        );

    if !perf.vitals.is_empty() {
        let mut kv = KeyValueList::new().with_title("Core Web Vitals");
        for (name, value, rating) in &perf.vitals {
            kv = kv.add(name, format!("{} — {}", value, rating));
        }
        builder = builder.add_component(kv);
    }

    if !perf.additional_metrics.is_empty() {
        let mut metrics = SummaryBox::new("Weitere Metriken");
        for (k, v) in &perf.additional_metrics {
            metrics = metrics.add_item(k, v);
        }
        builder = builder.add_component(metrics);
    }
    builder
}

fn render_seo(
    mut builder: renderreport::engine::ReportBuilder,
    seo: &SeoPresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("SEO-Analyse").with_level(2))
        .add_component(TextBlock::new(&seo.interpretation))
        .add_component(ScoreCard::new("SEO Score", seo.score).with_thresholds(80, 50));

    if !seo.meta_tags.is_empty() {
        let mut kv = KeyValueList::new().with_title("Meta-Tags");
        for (k, v) in &seo.meta_tags {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    if !seo.meta_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Feld"),
            TableColumn::new("Schweregrad"),
            TableColumn::new("Beschreibung"),
        ])
        .with_title("Meta-Tag Probleme");
        for (field, sev, msg) in &seo.meta_issues {
            table = table.add_row(vec![field.as_str(), sev.label(), msg.as_str()]);
        }
        builder = builder.add_component(table);
    }

    builder = builder
        .add_component(TextBlock::new(&seo.heading_summary))
        .add_component(TextBlock::new(&seo.social_summary));

    if !seo.technical_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title("Technisches SEO");
        for (k, v) in &seo.technical_summary {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    // SEO Content Profile
    if let Some(profile) = &seo.profile {
        builder = render_seo_profile(builder, profile);
    }

    builder
}

fn render_seo_profile(
    mut builder: renderreport::engine::ReportBuilder,
    profile: &SeoProfilePresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(Section::new("SEO-Inhaltsprofil").with_level(3))
        .add_component(Callout::info(&profile.identity_summary).with_title("Inhaltsprofil"));

    // Content Identity
    let mut identity = KeyValueList::new().with_title("Website-Identität");
    identity = identity.add("Website", &profile.site_name);
    identity = identity.add("Inhaltstyp", &profile.content_type);
    identity = identity.add("Sprache", &profile.language);
    if !profile.category_hints.is_empty() {
        identity = identity.add("Schema-Typen", profile.category_hints.join(", "));
    }
    builder = builder.add_component(identity);

    // Schema Inventory
    if !profile.schema_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Schema-Typ"),
            TableColumn::new("Vollständigkeit"),
            TableColumn::new("Details"),
        ])
        .with_title(format!(
            "Strukturierte Daten ({} Schemas)",
            profile.schema_count
        ));
        for (typ, completeness, details) in &profile.schema_rows {
            table = table.add_row(vec![typ.as_str(), completeness.as_str(), details.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // Signal Strength Overview
    if !profile.signal_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Kategorie"),
            TableColumn::new("Bewertung"),
            TableColumn::new("Einstufung"),
        ])
        .with_title(format!(
            "SEO-Signalstärke (Gesamt: {}%)",
            profile.signal_overall_pct
        ));
        for (cat, score, rating) in &profile.signal_rows {
            table = table.add_row(vec![cat.as_str(), score.as_str(), rating.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // Signal Details per category
    for (cat_name, checks) in &profile.signal_details {
        let mut detail_table = AuditTable::new(vec![
            TableColumn::new("Prüfung"),
            TableColumn::new("Status"),
            TableColumn::new("Detail"),
        ])
        .with_title(cat_name);
        for (label, passed, detail) in checks {
            let status = if *passed { "✓" } else { "✗" };
            detail_table = detail_table.add_row(vec![label.as_str(), status, detail.as_str()]);
        }
        builder = builder.add_component(detail_table);
    }

    // Maturity Rating
    let mut maturity = SummaryBox::new("SEO-Reifegrad");
    maturity = maturity.add_item("Level", &profile.maturity_level);
    maturity = maturity.add_item("Bewertung", &profile.maturity_description);
    maturity = maturity.add_item(
        "Techniken",
        format!(
            "{} von {} erkannt",
            profile.maturity_techniques_used, profile.maturity_techniques_total
        ),
    );
    builder = builder.add_component(maturity);

    builder
}

fn render_security(
    mut builder: renderreport::engine::ReportBuilder,
    sec: &SecurityPresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Sicherheit").with_level(2))
        .add_component(TextBlock::new(&sec.interpretation))
        .add_component(
            ScoreCard::new("Security Score", sec.score)
                .with_description(format!("Grade: {}", sec.grade))
                .with_thresholds(70, 50),
        );

    if !sec.headers.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Header"),
            TableColumn::new("Status"),
            TableColumn::new("Wert"),
        ])
        .with_title("Security Headers");
        for (name, status, val) in &sec.headers {
            table = table.add_row(vec![name.as_str(), status.as_str(), val.as_str()]);
        }
        builder = builder.add_component(table);
    }

    if !sec.ssl_info.is_empty() {
        let mut kv = KeyValueList::new().with_title("SSL/TLS");
        for (k, v) in &sec.ssl_info {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    for (title, sev, msg) in &sec.issues {
        builder = builder.add_component(Finding::new(title, map_severity(sev), msg));
    }

    if !sec.recommendations.is_empty() {
        let mut rec_list = List::new().with_title("Empfehlungen");
        for rec in &sec.recommendations {
            rec_list = rec_list.add_item(rec);
        }
        builder = builder.add_component(rec_list);
    }
    builder
}

fn render_mobile(
    mut builder: renderreport::engine::ReportBuilder,
    mobile: &MobilePresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Mobile Nutzbarkeit").with_level(2))
        .add_component(TextBlock::new(&mobile.interpretation))
        .add_component(ScoreCard::new("Mobile Score", mobile.score).with_thresholds(80, 50));

    if !mobile.viewport.is_empty() {
        let mut kv = KeyValueList::new().with_title("Viewport-Konfiguration");
        for (k, v) in &mobile.viewport {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.touch_targets.is_empty() {
        let mut box_ = SummaryBox::new("Touch Targets");
        for (k, v) in &mobile.touch_targets {
            box_ = box_.add_item(k, v);
        }
        builder = builder.add_component(box_);
    }
    if !mobile.font_analysis.is_empty() {
        let mut kv = KeyValueList::new().with_title("Schriftanalyse");
        for (k, v) in &mobile.font_analysis {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.content_sizing.is_empty() {
        let mut box_ = SummaryBox::new("Content Sizing");
        for (k, v) in &mobile.content_sizing {
            box_ = box_.add_item(k, v);
        }
        builder = builder.add_component(box_);
    }

    for (cat, sev, msg) in &mobile.issues {
        builder = builder.add_component(Finding::new(cat, map_severity(sev), msg));
    }
    builder
}

/// Render action plan for batch reports (using AuditTable)
fn render_action_plan(
    mut builder: renderreport::engine::ReportBuilder,
    plan: &ActionPlan,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !plan.quick_wins.is_empty() {
        builder = builder.add_component(Section::new("Quick Wins").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ]);
        for item in &plan.quick_wins {
            table = table.add_row(vec![
                item.action.clone(),
                item.benefit.clone(),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
        }
        builder = builder.add_component(table);
    }
    if !plan.medium_term.is_empty() {
        builder = builder.add_component(Section::new("Mittelfristige Maßnahmen").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ]);
        for item in &plan.medium_term {
            table = table.add_row(vec![
                item.action.clone(),
                item.benefit.clone(),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
        }
        builder = builder.add_component(table);
    }
    if !plan.structural.is_empty() {
        builder = builder.add_component(Section::new("Strukturelle Maßnahmen").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ]);
        for item in &plan.structural {
            table = table.add_row(vec![
                item.action.clone(),
                item.benefit.clone(),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
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
        assert_eq!(
            truncate_url("https://example.com/very/long/path/that/exceeds/limit", 30),
            "https://example.com/very/lo..."
        );
        assert_eq!(
            truncate_url("https://example.com", 30),
            "https://example.com"
        );
    }
}
