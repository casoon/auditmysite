//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

use std::{env, fs, path::PathBuf};

use renderreport::components::advanced::{
    FlowGroup, Grid, KeyValueList, List, PageBreak, TableOfContents,
};
use renderreport::components::text::{Label, TextBlock};
use renderreport::prelude::*;
use renderreport::theme::{Theme, TokenValue};
use renderreport::Engine;

// Composite components
use renderreport::components::{
    BenchmarkRow, BenchmarkTable, ComparisonModule, Component, MetricCard, ModuleComparison,
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
        .metadata("score", &vm.meta.score_label)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "");

    // ── Cover Page ───────────────────────────────────────────────────
    builder = builder
        .add_component(
            Label::new("Automatisierter Audit-Report")
                .with_size("11pt")
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
        .add_component(build_cover_meta(&vm.cover));

    let single_badge_asset = "/certificate-badge-single.svg";
    let single_badge_enabled = if let Ok(path) = certificate_badge_path(&vm.cover.certificate) {
        builder = builder.asset(single_badge_asset, path);
        true
    } else {
        false
    };

    builder = builder
        .add_component(build_cover_score_row(
            &vm.cover,
            single_badge_enabled.then_some(single_badge_asset),
        ))
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

    // ── 1. Hero / Ergebnisblock ─────────────────────────────────────
    // The "Empfehlung und nächste Schritte" callout is included inside the
    // soft_flow_group so it stays on the same page as the section header and
    // does not trigger a standalone page break.
    {
        let mut ergebnis_items = vec![
            component_json(Section::new("Ergebnisblock").with_level(1)),
            component_json(Callout::info(&vm.summary.executive_lead).with_title("Einordnung")),
            component_json(build_summary_overview(&vm.summary)),
        ];
        {
            let mut kv = KeyValueList::new().with_title("Gesamtauswirkung");
            kv = kv.add("Problemtyp", &vm.summary.problem_type);
            if !vm.summary.business_consequence.is_empty() {
                kv = kv.add("Konsequenz", &vm.summary.business_consequence);
            }
            for (label, value) in &vm.summary.overall_impact {
                kv = kv.add(label, value);
            }
            if !vm.summary.benchmark_context.is_empty() {
                kv = kv.add("Benchmark", &vm.summary.benchmark_context);
            }
            ergebnis_items.push(component_json(kv));
        }
        if !vm.summary.technical_overview.is_empty() {
            let mut list = List::new().with_title("Technische Gesamteinschätzung");
            for insight in &vm.summary.technical_overview {
                list = list.add_item(insight);
            }
            ergebnis_items.push(component_json(list));
        }
        ergebnis_items.push(component_json(build_hero_highlights_table(&vm, &i18n)));
        if !vm.summary.consequence.is_empty() {
            ergebnis_items.push(component_json(
                Callout::warning(&vm.summary.consequence).with_title("Ohne Maßnahmen"),
            ));
        }
        ergebnis_items.push(component_json(
            Callout::info(build_executive_recommendation(&vm))
                .with_title("Empfehlung und nächste Schritte"),
        ));
        builder = builder.add_component(soft_flow_group("360pt", ergebnis_items));
    }

    // Executive level: compact view
    if vm.meta.report_level == ReportLevel::Executive {
        // ── 2. Gesamtbewertung (Executive) ──────────────────────────
        builder = builder
            .add_component(PageBreak::new())
            .add_component(soft_flow_group(
                "280pt",
                vec![
                    component_json(Section::new("Gesamtbewertung").with_level(1)),
                    component_json(build_gesamtbewertung_grid(&vm.modules)),
                    component_json(build_module_summary_table(&vm.modules)),
                ],
            ));

        if vm.modules.dashboard.len() > 1 {
            builder = builder.add_component(build_module_comparison(&vm.modules));
        }

        // ── 3. Trend (Executive) ─────────────────────────────────────
        if let Some(ref history) = vm.history {
            builder = render_history_section(builder, history);
        }

        // ── 4. Was jetzt tun? (Executive) ────────────────────────────
        {
            let wjt_table = build_was_jetzt_tun_table(&vm);
            let mut wjt_items = vec![
                component_json(Section::new("Was jetzt tun?").with_level(1)),
                component_json(TextBlock::new(
                    "Die folgenden Maßnahmen haben die höchste Wirkung. Jede Maßnahme ist direkt umsetzbar — kein Abstract, keine langen Tabellen.",
                )),
            ];
            match wjt_table {
                WasJetztTunContent::Table(t) => wjt_items.push(component_json(t)),
                WasJetztTunContent::Empty(c) => wjt_items.push(component_json(c)),
            }
            builder = builder
                .add_component(PageBreak::new())
                .add_component(soft_flow_group("300pt", wjt_items));
        }

        // ── 5. Key Findings (Executive) ──────────────────────────────
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Key Findings").with_level(1));

        for (idx, group) in vm.findings.top_findings.iter().take(3).enumerate() {
            if idx > 0 {
                builder = builder.add_component(PageBreak::new());
            }
            builder = render_key_finding_block(builder, group, &i18n);
        }

        builder = render_methodology_section(builder, &vm.methodology, &i18n);

        let built_report = builder.build();
        return Ok(engine.render_pdf(&built_report)?);
    }

    builder = builder.add_component(PageBreak::new());

    // ── 2. Gesamtbewertung ──────────────────────────────────────────
    builder = builder.add_component(soft_flow_group(
        "320pt",
        vec![
            component_json(Section::new("Gesamtbewertung").with_level(1)),
            component_json(build_gesamtbewertung_grid(&vm.modules)),
            component_json(build_module_summary_table(&vm.modules)),
        ],
    ));

    if let Some(ref overall_text) = vm.modules.overall_interpretation {
        builder =
            builder.add_component(Callout::info(overall_text).with_title("Gesamtscore einordnen"));
    }

    if vm.modules.dashboard.len() > 1 {
        builder = builder.add_component(build_module_comparison(&vm.modules));
    }

    // ── 3. Entwicklung / Trend ──────────────────────────────────────
    if let Some(ref history) = vm.history {
        builder = render_history_section(builder, history);
    }

    // ── 4. Was jetzt tun? ───────────────────────────────────────────
    {
        let wjt_table = build_was_jetzt_tun_table(&vm);
        let mut wjt_items = vec![
            component_json(Section::new(&vm.actions.block_title).with_level(1)),
            component_json(TextBlock::new(&vm.actions.intro_text)),
        ];
        // Phase-Preview: visual overview — label → impact arrow, then action bullets
        if !vm.actions.phase_preview.is_empty() {
            for phase in &vm.actions.phase_preview {
                // Title: "Phase 1 – Sofort  ·  4 Maßnahmen"
                let count_label = if phase.item_count == 1 {
                    "1 Maßnahme".to_string()
                } else {
                    format!("{} Maßnahmen", phase.item_count)
                };
                let title = format!("{}  ·  {}", phase.phase_label, count_label);
                let mut phase_list = List::new().with_title(&title);
                // First item: impact arrow — what this phase achieves
                phase_list = phase_list.add_item(format!("→ {}", phase.description));
                // Action bullets
                for item in &phase.top_items {
                    phase_list = phase_list.add_item(item);
                }
                // Show remaining count if list was capped
                let shown = phase.top_items.len();
                if phase.item_count > shown {
                    phase_list = phase_list.add_item(format!(
                        "+ {} weitere im Detail unten",
                        phase.item_count - shown
                    ));
                }
                wjt_items.push(component_json(phase_list));
            }
        }
        match wjt_table {
            WasJetztTunContent::Table(t) => wjt_items.push(component_json(t)),
            WasJetztTunContent::Empty(c) => wjt_items.push(component_json(c)),
        }
        builder = builder
            .add_component(PageBreak::new())
            .add_component(soft_flow_group("340pt", wjt_items));
    }

    // ── 5. Modulübersicht ───────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(soft_flow_group(
            "280pt",
            vec![
                component_json(Section::new("Modulübersicht").with_level(1)),
                component_json(TextBlock::new(
                    "Jedes Modul beantwortet eine Frage: Was ist der aktuelle Stand, was ist die Bedeutung, und wo ist der größte Hebel?",
                )),
                component_json(build_module_detail_table(&vm.modules)),
            ],
        ));

    // ── 6. Key Findings ─────────────────────────────────────────────
    let findings_count = vm.findings.top_findings.len();
    let findings_title = if vm.summary.score >= 85 && findings_count <= 2 {
        match findings_count {
            0 => "Keine offenen Themen".to_string(),
            1 => "1 offenes Thema".to_string(),
            n => format!("{n} offene Themen"),
        }
    } else {
        "Key Findings".to_string()
    };
    let findings_intro = if vm.summary.score >= 85 && findings_count <= 2 {
        "Die Seite ist technisch stark. Die folgenden Punkte sind letzte Feinschliff-Hebel ohne strukturellen Druck."
    } else {
        "Diese Themen sollten zuerst angegangen werden. Jeder Block zeigt Problem, Impact und die empfohlene Maßnahme — technische Details folgen im nächsten Abschnitt."
    };
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(&findings_title).with_level(1))
        .add_component(TextBlock::new(findings_intro));

    for (idx, group) in vm.findings.top_findings.iter().take(5).enumerate() {
        if idx > 0 {
            builder = builder.add_component(PageBreak::new());
        }
        builder = render_key_finding_block(builder, group, &i18n);
    }

    // ── 7. Technischer Detailteil ───────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(soft_flow_group(
            "260pt",
            vec![
                component_json(Section::new("Technische Analyse und Umsetzung").with_level(1)),
                component_json(TextBlock::new(
                    "Ab hier folgt die technische Sicht für Entwicklung, Design und Redaktion. Die folgenden Abschnitte zeigen betroffene Elemente, konkrete Maßnahmen und technische Hinweise für die Umsetzung.",
                )),
                component_json(Section::new("Problem-Details").with_level(2)),
                component_json(if !vm.severity.has_issues {
                    Callout::success(i18n.t("callout-no-issues-body"))
                        .with_title(i18n.t("callout-no-issues-title"))
                } else {
                    Callout::info(
                        "Die folgende Übersicht enthält die relevanten Problemgruppen mit technischer Einordnung.",
                    )
                    .with_title("Technische Übersicht")
                }),
            ],
        ));

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
            .add_component(soft_flow_group(
                "220pt",
                vec![
                    component_json(Section::new("Technische Detailmetriken").with_level(2)),
                    component_json(TextBlock::new(
                        "Die folgenden Kennzahlen ergänzen die Modul-Übersicht um technische Detailwerte für Analyse und Umsetzung.",
                    )),
                    component_json(build_analysis_focus_table()),
                ],
            ));
    }

    if let Some(ref perf) = vm.module_details.performance {
        builder = render_performance(builder, perf);
    }

    // ── Performance Budget Violations ───────────────────────────────
    if !report.budget_violations.is_empty() {
        builder = render_budget_violations(builder, &report.budget_violations);
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
    if let Some(ref dm) = vm.module_details.dark_mode {
        builder = render_dark_mode(builder, dm);
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

    let author = extract_domain(&pres.cover.url);

    let mut builder = engine
        .report("wcag-batch-audit")
        .title(&pres.cover.title)
        .subtitle(&pres.cover.url)
        .metadata("date", &pres.cover.date)
        .metadata("version", &pres.cover.version)
        .metadata("author", &author)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "");

    if let Some(ref logo_path) = config.logo_path {
        if logo_path.exists() {
            builder = builder.add_component(
                Image::new(logo_path.to_string_lossy().to_string()).with_width("30%"),
            );
        }
    }

    let batch_score = pres.portfolio_summary.average_score.round() as u32;
    builder = builder
        .add_component(
            Label::new(&author)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(
            Label::new("Domainweiter Accessibility-Check")
                .with_size("24pt")
                .bold(),
        )
        .add_component(
            Label::new(&pres.cover.url)
                .with_size("14pt")
                .with_color("#475569"),
        );

    let batch_badge_asset = "/certificate-badge-batch.svg";
    let batch_badge_enabled =
        if let Ok(path) = certificate_badge_path(batch_certificate_label(batch_score)) {
            builder = builder.asset(batch_badge_asset, path);
            true
        } else {
            false
        };

    builder = builder
        .add_component(build_batch_cover_score_row(
            batch_score,
            pres.portfolio_summary.total_urls as u32,
            pres.portfolio_summary.total_violations as u32,
            batch_badge_enabled.then_some(batch_badge_asset),
        )?)
        .add_component(
            TextBlock::new(&pres.portfolio_summary.verdict_text)
                .with_size("11pt")
                .with_line_height("1.4em")
                .with_max_width("100%"),
        )
        .add_component(PageBreak::new());
    if config.level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    // ── 1. Portfolio Overview ────────────────────────────────────────
    let best = pres.url_ranking.last();
    let worst = pres.url_ranking.first();
    let dist = &pres.portfolio_summary.severity_distribution;

    builder = builder
        .add_component(soft_flow_group(
            "300pt",
            vec![
                component_json(Section::new("Gesamtübersicht").with_level(1)),
                component_json(build_batch_overview_grid(
                    pres.portfolio_summary.total_urls as u32,
                    pres.portfolio_summary.average_score.round() as u32,
                    pres.portfolio_summary.total_violations as u32,
                    (dist.critical + dist.high) as u32,
                    pres.portfolio_summary.crawl_links.as_ref().map(|links| {
                        (links.broken_internal_links.len() + links.broken_external_links.len())
                            as u32
                    }),
                )),
                component_json(build_batch_overview_comparison(best, worst)),
            ],
        ))
        .add_component(TextBlock::new(&pres.portfolio_summary.verdict_text));

    builder = builder.add_component(PageBreak::new());

    // ── 2. URL-Ranking ──────────────────────────────────────────────
    let rows: Vec<BenchmarkRow> = pres
        .url_ranking
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let mut row = BenchmarkRow::new(
                (i + 1) as u32,
                &truncate_url(&u.url, 35),
                u.score as u32,
                u.score as u32,
                u.critical_violations as u32,
            );
            if let Some(detail) = pres.url_details.iter().find(|detail| detail.url == u.url) {
                if let Some((_, score)) = detail
                    .module_scores
                    .iter()
                    .find(|(module, _)| module == "SEO")
                {
                    row = row.with_seo(*score);
                }
                if let Some((_, score)) = detail
                    .module_scores
                    .iter()
                    .find(|(module, _)| module == "Performance")
                {
                    row = row.with_performance(*score);
                }
                if let Some((_, score)) = detail
                    .module_scores
                    .iter()
                    .find(|(module, _)| module == "Security")
                {
                    row = row.with_security(*score);
                }
            }
            row
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

    for group in pres.top_issues.iter().take(3) {
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

    // ── 5a. Render Blocking (Batch) ─────────────────────────────────
    if !pres.portfolio_summary.render_blocking_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title("Render-Blocking-Übersicht (domainweit)");
        for (label, value) in &pres.portfolio_summary.render_blocking_summary {
            kv = kv.add(label, value);
        }
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Render-Blocking & Assets").with_level(1))
            .add_component(TextBlock::new(
                "Aggregierte Auswertung render-blockierender Ressourcen über alle geprüften Seiten. \
                 Blocking-Scripts und -CSS verzögern den First Contentful Paint. \
                 Third-Party-Traffic entsteht durch externe Fonts, Analytics und Widgets.",
            ))
            .add_component(kv);
    }

    // ── 5b. Performance Budgets (Batch) ─────────────────────────────
    if !pres.portfolio_summary.budget_summary.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Metrik"),
            TableColumn::new("Budget"),
            TableColumn::new("Betr. Seiten"),
            TableColumn::new("Severity"),
        ])
        .with_title("Performance-Budget-Verstöße (domainweit)");
        for (metric, budget, count, sev) in &pres.portfolio_summary.budget_summary {
            table = table.add_row(vec![
                metric.clone(),
                budget.clone(),
                count.to_string(),
                sev.clone(),
            ]);
        }
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Performance Budgets").with_level(1))
            .add_component(TextBlock::new(
                "Performance-Budgets definieren Obergrenzen für Ladezeiten, Asset-Größen und \
                 Drittanbieter-Traffic. Die folgende Tabelle zeigt, auf wie vielen Seiten welche \
                 Budgets überschritten wurden.",
            ))
            .add_component(table);
    }

    // ── 5. Technische URL-Matrix ───────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Technische URL-Matrix").with_level(1))
        .add_component(TextBlock::new(
            "Verdichtete Übersicht aller geprüften URLs mit Fokus auf technische Priorisierung. Jede Zeile zeigt Score, Problemintensität und den größten Hebel für die nächste Optimierungsrunde.",
        ));

    if let Some(ref crawl_links) = pres.portfolio_summary.crawl_links {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Interne Broken Links").with_level(1))
            .add_component(TextBlock::new(format!(
                "Für den Crawl ab {} wurden {} interne Linkziele geprüft. {} kaputte interne Verlinkungen wurden erkannt.",
                crawl_links.seed_url,
                crawl_links.checked_internal_links,
                crawl_links.broken_internal_links.len()
            )));

        if crawl_links.broken_internal_links.is_empty() {
            builder = builder.add_component(Callout::info(
                "Keine kaputten internen Links im geprüften Crawl-Set erkannt.",
            ));
        } else {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Quelle"),
                TableColumn::new("Ziel"),
                TableColumn::new("Status"),
                TableColumn::new("Typ"),
            ])
            .with_title("Kaputte interne Links");

            for row in &crawl_links.broken_internal_links {
                let severity_color = match row.severity.as_str() {
                    "high" => "#dc2626",
                    "medium" => "#ea580c",
                    _ => "#ca8a04",
                };
                let typ_label = if row.redirect_hops > 0 {
                    format!("→{} Hops", row.redirect_hops)
                } else {
                    "direkt".to_string()
                };
                table = table.add_row(vec![
                    truncate_url(&row.source_url, 30),
                    truncate_url(&row.target_url, 38),
                    format!("\x1b[{}m{}\x1b[0m", severity_color, row.status),
                    typ_label,
                ]);
            }

            builder = builder.add_component(table);
        }

        // External broken links
        if !crawl_links.broken_external_links.is_empty() {
            builder = builder
                .add_component(Section::new("Externe Broken Links").with_level(2))
                .add_component(TextBlock::new(format!(
                    "{} externe Linkziele geprüft. {} kaputte externe Verlinkungen erkannt.",
                    crawl_links.checked_external_links,
                    crawl_links.broken_external_links.len()
                )));

            let mut ext_table = AuditTable::new(vec![
                TableColumn::new("Quelle"),
                TableColumn::new("Ziel"),
                TableColumn::new("Status"),
                TableColumn::new("Typ"),
            ])
            .with_title("Kaputte externe Links");

            for row in &crawl_links.broken_external_links {
                let typ_label = if row.redirect_hops > 0 {
                    format!("→{} Hops", row.redirect_hops)
                } else {
                    "direkt".to_string()
                };
                ext_table = ext_table.add_row(vec![
                    truncate_url(&row.source_url, 30),
                    truncate_url(&row.target_url, 38),
                    row.status.clone(),
                    typ_label,
                ]);
            }

            builder = builder.add_component(ext_table);
        } else if crawl_links.checked_external_links > 0 {
            builder = builder
                .add_component(Section::new("Externe Links").with_level(2))
                .add_component(Callout::info(format!(
                    "{} externe Linkziele geprüft — keine kaputten externen Links erkannt.",
                    crawl_links.checked_external_links
                )));
        }

        // Redirect chains
        if !crawl_links.redirect_chains.is_empty() {
            builder = builder
                .add_component(Section::new("Redirect-Ketten").with_level(2))
                .add_component(TextBlock::new(format!(
                    "{} Links mit mehr als einem Redirect-Hop erkannt.",
                    crawl_links.redirect_chains.len()
                )));

            let mut chain_table = AuditTable::new(vec![
                TableColumn::new("Quelle"),
                TableColumn::new("Ziel"),
                TableColumn::new("Hops"),
                TableColumn::new("Final-URL"),
            ])
            .with_title("Redirect-Ketten (> 1 Hop)");

            for chain in &crawl_links.redirect_chains {
                chain_table = chain_table.add_row(vec![
                    truncate_url(&chain.source_url, 28),
                    truncate_url(&chain.target_url, 28),
                    chain.hops.to_string(),
                    truncate_url(&chain.final_url, 32),
                ]);
            }

            builder = builder.add_component(chain_table);
        }
    }

    let mut matrix = AuditTable::new(vec![
        TableColumn::new("URL"),
        TableColumn::new("Typ"),
        TableColumn::new("Score"),
        TableColumn::new("Krit.+Hoch"),
        TableColumn::new("Gesamt"),
        TableColumn::new("Größter Hebel"),
    ])
    .with_title("Technische Seitenübersicht");

    for detail in &pres.url_details {
        matrix = matrix.add_row(vec![
            truncate_url(&detail.url, 38),
            detail.page_type.clone().unwrap_or_else(|| "—".to_string()),
            format!("{}/100", detail.score.round() as u32),
            detail.critical_violations.to_string(),
            detail.total_violations.to_string(),
            truncate_url(&detail.biggest_lever, 46),
        ]);
    }
    builder = builder.add_component(matrix);

    if config.level != ReportLevel::Executive {
        let mut focus_table = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new("Seitentyp"),
            TableColumn::new("Merkmale"),
            TableColumn::new("Top-Probleme"),
        ])
        .with_title("Fokus auf problematische Seiten");

        for detail in pres.url_details.iter().take(10) {
            focus_table = focus_table.add_row(vec![
                truncate_url(&detail.url, 38),
                detail.page_type.clone().unwrap_or_else(|| "—".to_string()),
                if detail.page_attributes.is_empty() {
                    "—".to_string()
                } else {
                    truncate_url(&detail.page_attributes.join(", "), 40)
                },
                if detail.top_issues.is_empty() {
                    "—".to_string()
                } else {
                    truncate_url(&detail.top_issues.join(", "), 52)
                },
            ]);
        }
        builder = builder.add_component(focus_table);
    }

    // ── 6. Content- und Seitenprofil ───────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Content- und Seitenprofil").with_level(1))
        .add_component(TextBlock::new(
            "Die folgenden Auswertungen ordnen Inhalt, Seitentypen und thematische Schwerpunkte ein. Dieser Block ist bewusst nachgelagert und ergänzt die technische Priorisierung um SEO- und Content-Signale.",
        ));

    if !pres.portfolio_summary.page_type_distribution.is_empty() {
        let mut type_table = AuditTable::new(vec![
            TableColumn::new("Seitentyp"),
            TableColumn::new("Seiten"),
            TableColumn::new("Anteil"),
        ])
        .with_title("Seitentyp-Verteilung");

        for (label, count, pct) in &pres.portfolio_summary.page_type_distribution {
            type_table =
                type_table.add_row(vec![label.clone(), count.to_string(), format!("{pct}%")]);
        }

        builder = builder.add_component(type_table);
    }

    if !pres.portfolio_summary.distribution_insights.is_empty() {
        let mut insights = List::new().with_title("Content-Auffälligkeiten");
        for insight in &pres.portfolio_summary.distribution_insights {
            insights = insights.add_item(insight);
        }
        builder = builder.add_component(insights);
    }

    if !pres.portfolio_summary.top_topics.is_empty() {
        let mut topics =
            AuditTable::new(vec![TableColumn::new("Thema"), TableColumn::new("Seiten")])
                .with_title("Top-Themen der Domain");

        for (topic, count) in &pres.portfolio_summary.top_topics {
            topics = topics.add_row(vec![topic.clone(), count.to_string()]);
        }
        builder = builder.add_component(topics);
    }

    if !pres.portfolio_summary.overlap_pairs.is_empty() {
        let mut overlap = AuditTable::new(vec![
            TableColumn::new("Seite A"),
            TableColumn::new("Seite B"),
            TableColumn::new("Ähnlichkeit"),
        ])
        .with_title("Thematische Überschneidungen");

        for (left, right, score) in &pres.portfolio_summary.overlap_pairs {
            overlap = overlap.add_row(vec![
                truncate_url(left, 30),
                truncate_url(right, 30),
                format!("{score}%"),
            ]);
        }
        builder = builder.add_component(overlap);
    }

    if !pres.portfolio_summary.near_duplicates.is_empty() {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Near-Duplicate Content").with_level(1))
            .add_component(TextBlock::new(
                "Die folgenden Seitenpaare haben inhaltlich sehr ähnliche Texte (SimHash-Ähnlichkeit ≥ 80 %). \
                 Near-Duplicate-Content kann zu Keyword-Kannibalisierung und schlechteren Rankings führen. \
                 Empfehlung: Inhalte konsolidieren oder stärker differenzieren.",
            ));

        let mut table = AuditTable::new(vec![
            TableColumn::new("Seite A"),
            TableColumn::new("Seite B"),
            TableColumn::new("Ähnlichkeit"),
            TableColumn::new("Typ"),
        ])
        .with_title("Near-Duplicate-Paare");

        for (url_a, url_b, sim) in &pres.portfolio_summary.near_duplicates {
            let kind = if *sim >= 95 {
                "Duplikat"
            } else {
                "Near-Duplicate"
            };
            table = table.add_row(vec![
                truncate_url(url_a, 38),
                truncate_url(url_b, 38),
                format!("{sim} %"),
                kind.to_string(),
            ]);
        }

        builder = builder.add_component(table);
    }

    let mut content_matrix = AuditTable::new(vec![
        TableColumn::new("URL"),
        TableColumn::new("Typ"),
        TableColumn::new("Profil"),
        TableColumn::new("Themen"),
    ])
    .with_title("Content-Übersicht");

    for detail in &pres.url_details {
        content_matrix = content_matrix.add_row(vec![
            truncate_url(&detail.url, 38),
            detail.page_type.clone().unwrap_or_else(|| "—".to_string()),
            detail
                .page_semantic_score
                .map(|score| format!("{score}/100"))
                .unwrap_or_else(|| "—".to_string()),
            if detail.topic_terms.is_empty() {
                "—".to_string()
            } else {
                truncate_url(&detail.topic_terms.join(", "), 34)
            },
        ]);
    }
    builder = builder.add_component(content_matrix);

    if !pres.portfolio_summary.strongest_content_pages.is_empty() {
        let mut strengths = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new("Seitentyp"),
            TableColumn::new("Profil"),
        ])
        .with_title("Stärkste Seitenprofile");

        for (url, page_type, score) in &pres.portfolio_summary.strongest_content_pages {
            strengths = strengths.add_row(vec![
                truncate_url(url, 42),
                page_type.clone(),
                format!("{score} / 100"),
            ]);
        }
        builder = builder.add_component(strengths);
    }

    if !pres.portfolio_summary.weakest_content_pages.is_empty() {
        let mut weaknesses = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new("Seitentyp"),
            TableColumn::new("Profil"),
        ])
        .with_title("Schwächste Seitenprofile");

        for (url, page_type, score) in &pres.portfolio_summary.weakest_content_pages {
            weaknesses = weaknesses.add_row(vec![
                truncate_url(url, 42),
                page_type.clone(),
                format!("{score} / 100"),
            ]);
        }
        builder = builder.add_component(weaknesses);
    }

    // ── 7. Anhang ───────────────────────────────────────────────────
    if config.level == ReportLevel::Technical && !pres.appendix.per_url.is_empty() {
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

// ─── Comparison Report ───────────────────────────────────────────────────────

/// Generate a competitive comparison PDF report.
pub fn generate_comparison_pdf(
    comparison: &crate::audit::ComparisonReport,
    _config: &ReportConfig,
) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;

    let avg_score = if comparison.entries.is_empty() {
        0u32
    } else {
        (comparison
            .entries
            .iter()
            .map(|e| e.overall_score as u64)
            .sum::<u64>()
            / comparison.entries.len() as u64) as u32
    };

    let author = comparison
        .entries
        .first()
        .map(|e| extract_domain(&e.url))
        .unwrap_or_default();

    let mut builder = engine
        .report("wcag-comparison")
        .metadata("date", chrono::Local::now().format("%d.%m.%Y").to_string())
        .metadata("author", &author)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "");

    builder = builder
        .add_component(
            Label::new(&author)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(Label::new("Wettbewerbsvergleich").with_size("28pt").bold())
        .add_component(
            Label::new(format!(
                "Vergleich von {} Domains — Ø Score: {}/100",
                comparison.entries.len(),
                avg_score
            ))
            .with_size("12pt")
            .with_color("#475569"),
        )
        .add_component(PageBreak::new());

    // ── 1. Domain-Ranking ────────────────────────────────────────────
    builder = builder
        .add_component(Section::new("Domain-Ranking").with_level(1))
        .add_component(TextBlock::new(format!(
            "Vergleich von {} Domains anhand eines vollständigen Audits der jeweiligen Startseite. \
             Durchschnittlicher Score: {}/100.",
            comparison.entries.len(),
            avg_score,
        )));

    let rows: Vec<BenchmarkRow> = comparison
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mut row = BenchmarkRow::new(
                (i + 1) as u32,
                &e.domain,
                e.overall_score,
                e.accessibility_score,
                e.critical_violations as u32,
            );
            if let Some(s) = e.seo_score {
                row = row.with_seo(s);
            }
            if let Some(s) = e.performance_score {
                row = row.with_performance(s);
            }
            if let Some(s) = e.security_score {
                row = row.with_security(s);
            }
            row
        })
        .collect();

    builder = builder.add_component(BenchmarkTable::new(rows).with_title("Domain-Ranking"));

    // ── 2. Modul-Vergleich ───────────────────────────────────────────
    let has_module_data = comparison
        .entries
        .iter()
        .any(|e| e.seo_score.is_some() || e.performance_score.is_some());

    if has_module_data {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Modul-Vergleich").with_level(1));

        let comparison_modules: Vec<ComparisonModule> = comparison
            .entries
            .iter()
            .map(|e| ComparisonModule::new(&e.domain, e.overall_score))
            .collect();
        builder = builder.add_component(ModuleComparison::new(comparison_modules));
    }

    // ── 3. Top Findings je Domain ────────────────────────────────────
    let has_issues = comparison.entries.iter().any(|e| !e.top_issues.is_empty());
    if has_issues {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Wichtigste Findings je Domain").with_level(1));

        for entry in &comparison.entries {
            if entry.top_issues.is_empty() {
                continue;
            }
            builder = builder.add_component(
                Section::new(format!("{} — Top Findings", entry.domain)).with_level(2),
            );
            let mut list = List::new();
            for issue in &entry.top_issues {
                list = list.add_item(issue.clone());
            }
            builder = builder.add_component(list);
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
            .with_description(format!("Grade {} — {}", summary.grade, summary.maturity_label))
            .with_thresholds(70, 50)
            .with_height("100%")
            .to_data()
    });
    let mut grid = Grid::new(2)
        .with_item_min_height("132pt")
        .add_item(score_card);

    for metric in summary.metrics.iter().take(3) {
        let mut card = MetricCard::new(&metric.title, &metric.value).with_height("100%");
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

fn build_batch_overview_grid(
    total_urls: u32,
    average_score: u32,
    total_violations: u32,
    critical_and_high: u32,
    broken_internal_links: Option<u32>,
) -> Grid {
    let mut metrics = vec![
        (
            "Durchschnitt",
            format!("{average_score} / 100"),
            Some(score_quality_color(average_score)),
        ),
        ("Geprüfte Websites", total_urls.to_string(), Some("#0f766e")),
        (
            "Verstöße gesamt",
            total_violations.to_string(),
            Some("#b45309"),
        ),
        (
            "Kritisch + Hoch",
            critical_and_high.to_string(),
            Some("#dc2626"),
        ),
    ];

    if let Some(count) = broken_internal_links {
        metrics.push((
            "Broken Links",
            count.to_string(),
            Some(if count > 0 { "#dc2626" } else { "#0f766e" }),
        ));
    }

    let mut grid = Grid::new(2);
    for (title, value, accent) in metrics {
        let mut card = MetricCard::new(title, value);
        if let Some(color) = accent {
            card = card.with_accent_color(color);
        }
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": card.to_data()
        }));
    }

    grid
}

fn build_batch_overview_comparison(
    best: Option<&UrlSummary>,
    worst: Option<&UrlSummary>,
) -> AuditTable {
    let mut table = AuditTable::new(vec![
        TableColumn::new("Vergleich"),
        TableColumn::new("URL"),
        TableColumn::new("Score"),
    ])
    .with_title("Spannweite im aktuellen Batch");

    if let Some(best) = best {
        table = table.add_row(vec![
            "Stärkste Seite".to_string(),
            truncate_url(&best.url, 48),
            format!("{} / 100", best.score.round() as u32),
        ]);
    }

    if let Some(worst) = worst {
        table = table.add_row(vec![
            "Schwächste Seite".to_string(),
            truncate_url(&worst.url, 48),
            format!("{} / 100", worst.score.round() as u32),
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
    let focus_count = vm.findings.top_findings.len().min(3);
    let focus_text = match focus_count {
        0 => "Zuerst die wichtigsten Themen priorisieren".to_string(),
        1 => "Zuerst das kritischste Thema beheben".to_string(),
        n => format!("Zuerst die {n} kritischsten Themen beheben"),
    };

    let recommendation = format!(
        "{focus_text}, danach die Quick Wins umsetzen. Startpunkt: {}",
        simplify_for_summary(&lead)
    );

    if let Some(score_note) = vm.summary.score_note.as_deref() {
        format!("{recommendation}\n\nHinweis zum Score: {score_note}")
    } else {
        recommendation
    }
}

/// Build the hero block highlights table: top 3 problems + top 3 next steps
fn build_hero_highlights_table(vm: &ReportViewModel, i18n: &I18n) -> AuditTable {
    let problems: Vec<String> = if vm.findings.top_findings.is_empty() {
        vec!["Keine priorisierten Probleme im automatischen Test erkannt.".to_string()]
    } else {
        vm.findings
            .top_findings
            .iter()
            .take(3)
            .map(|f| {
                format!(
                    "[{}] {}: {}",
                    severity_label_i18n(f.severity, i18n),
                    f.title,
                    simplify_for_summary(&f.customer_description)
                )
            })
            .collect()
    };

    let next_steps: Vec<String> = if vm.summary.top_actions.is_empty() {
        vec!["Qualität sichern und regelmäßige Audits einplanen.".to_string()]
    } else {
        vm.summary
            .top_actions
            .iter()
            .take(3)
            .map(|a| simplify_for_summary(a))
            .collect()
    };

    let problems_label = match vm.findings.top_findings.len() {
        0 => "Offene Themen".to_string(),
        1 => "1 offenes Thema".to_string(),
        n if vm.summary.score >= 85 => format!("{n} offene Themen"),
        _ => "Top 3 Probleme".to_string(),
    };
    let mut table = AuditTable::new(vec![
        TableColumn::new(&problems_label),
        TableColumn::new("Nächste Schritte"),
    ])
    .with_title("Sofortübersicht");

    let max_rows = problems.len().max(next_steps.len());
    for idx in 0..max_rows {
        table = table.add_row(vec![
            problems.get(idx).cloned().unwrap_or_default(),
            next_steps.get(idx).cloned().unwrap_or_default(),
        ]);
    }

    table
}

/// Build a 3-group overview grid: UX/Accessibility, Technik/Sicherheit, Sichtbarkeit/SEO
fn build_gesamtbewertung_grid(modules: &ModulesBlock) -> Grid {
    // Group modules thematically
    let a11y = modules
        .dashboard
        .iter()
        .find(|m| m.name == "Barrierefreiheit");
    let perf = modules.dashboard.iter().find(|m| m.name == "Performance");
    let seo = modules.dashboard.iter().find(|m| m.name == "SEO");
    let sec = modules.dashboard.iter().find(|m| m.name == "Sicherheit");
    let mob = modules.dashboard.iter().find(|m| m.name == "Mobile");

    // UX/Accessibility: average of a11y + mobile
    let ux_score = match (a11y, mob) {
        (Some(a), Some(m)) => (a.score + m.score) / 2,
        (Some(a), None) => a.score,
        (None, Some(m)) => m.score,
        (None, None) => 0,
    };
    let ux_label = match ux_score {
        85..=100 => "Stark",
        70..=84 => "Solide",
        50..=69 => "Ausbaufähig",
        _ => "Kritisch",
    };

    // Technik/Sicherheit: average of performance + security
    let tech_score = match (perf, sec) {
        (Some(p), Some(s)) => (p.score + s.score) / 2,
        (Some(p), None) => p.score,
        (None, Some(s)) => s.score,
        (None, None) => 0,
    };
    let tech_label = match tech_score {
        85..=100 => "Stark",
        70..=84 => "Solide",
        50..=69 => "Ausbaufähig",
        _ => "Kritisch",
    };

    // Sichtbarkeit/SEO
    let seo_score = seo.map(|s| s.score).unwrap_or(0);
    let seo_label = match seo_score {
        85..=100 => "Stark",
        70..=84 => "Solide",
        50..=69 => "Ausbaufähig",
        _ => "Kritisch",
    };

    let mut grid = Grid::new(3);

    // UX / Accessibility card
    let ux_accent = score_quality_color(ux_score);
    let ux_subtitle = if let Some(a) = a11y {
        format!(
            "{} · {}",
            ux_label,
            a.key_lever.split('.').next().unwrap_or("").trim()
        )
    } else {
        ux_label.to_string()
    };
    grid = grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("UX / Barrierefreiheit", format!("{ux_score} / 100"))
            .with_subtitle(ux_subtitle)
            .with_accent_color(ux_accent)
            .to_data()
    }));

    // Technik / Sicherheit card
    let tech_accent = score_quality_color(tech_score);
    let tech_subtitle = if let Some(p) = perf {
        format!(
            "{} · {}",
            tech_label,
            p.key_lever.split('.').next().unwrap_or("").trim()
        )
    } else {
        tech_label.to_string()
    };
    grid = grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("Technik / Sicherheit", format!("{tech_score} / 100"))
            .with_subtitle(tech_subtitle)
            .with_accent_color(tech_accent)
            .to_data()
    }));

    // Sichtbarkeit / SEO card
    let seo_accent = score_quality_color(seo_score);
    let seo_subtitle = if let Some(s) = seo {
        format!(
            "{} · {}",
            seo_label,
            s.key_lever.split('.').next().unwrap_or("").trim()
        )
    } else {
        seo_label.to_string()
    };
    grid = grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("Sichtbarkeit / SEO", format!("{seo_score} / 100"))
            .with_subtitle(seo_subtitle)
            .with_accent_color(seo_accent)
            .to_data()
    }));

    grid
}

/// Build interpretive module detail table: Status + Bedeutung + größter Hebel
fn build_module_detail_table(modules: &ModulesBlock) -> AuditTable {
    let mut table = AuditTable::new(vec![
        TableColumn::new("Modul"),
        TableColumn::new("Score"),
        TableColumn::new("Status"),
        TableColumn::new("Bedeutung"),
        TableColumn::new("Größter Hebel"),
    ])
    .with_title("Modulübersicht — Status und Interpretation");

    for module in &modules.dashboard {
        let status = if module.score >= module.good_threshold {
            "Gut"
        } else if module.score >= module.warn_threshold {
            "Solide"
        } else {
            "Handlungsbedarf"
        };

        let bedeutung = module_bedeutung(&module.name);

        table = table.add_row(vec![
            module.name.clone(),
            format!("{}/100", module.score),
            status.to_string(),
            bedeutung.to_string(),
            simplify_for_summary(&module.key_lever),
        ]);
    }

    table
}

fn module_bedeutung(name: &str) -> &'static str {
    match name {
        "Barrierefreiheit" => "Nutzbarkeit für alle — rechtliche Grundlage, UX-Qualität",
        "Performance" => "Ladezeit und Reaktionsverhalten — direkt spürbar für Nutzer",
        "SEO" => "Auffindbarkeit in Suchmaschinen — Reichweite und organischer Traffic",
        "Sicherheit" => "Schutz vor Angriffen und Datenlecks — Vertrauen und Compliance",
        "Mobile" => "Bedienbarkeit auf kleinen Bildschirmen — Mehrheit der Nutzer",
        _ => "Qualitätsmerkmal der Website",
    }
}

enum WasJetztTunContent {
    Table(AuditTable),
    Empty(Callout),
}

/// Build the "Was jetzt tun?" task table (max 5 actions)
fn build_was_jetzt_tun_table(vm: &ReportViewModel) -> WasJetztTunContent {
    // Collect top items from action roadmap, prioritize by execution priority
    let all_items: Vec<&RoadmapItemData> = vm
        .actions
        .roadmap_columns
        .iter()
        .flat_map(|col| col.items.iter())
        .collect();

    // Sort: Sofort beheben / Direkt angehen first
    let mut sorted: Vec<&RoadmapItemData> = all_items;
    sorted.sort_by_key(|i| {
        let ep = i.execution_priority.as_str();
        if ep.contains("Direkt") || ep.contains("Sofort") {
            0u8
        } else if ep.contains("Nächstes") || ep.contains("Wichtig") {
            1
        } else {
            2
        }
    });

    let selected: Vec<&RoadmapItemData> = sorted.into_iter().take(5).collect();

    if selected.is_empty() {
        return WasJetztTunContent::Empty(
            Callout::success(
                "Keine priorisierten Maßnahmen identifiziert — Qualität sichern und regelmäßige Audits einplanen.",
            )
            .with_title("Aktuell keine offenen Maßnahmen"),
        );
    }

    let mut table = AuditTable::new(vec![
        TableColumn::new("Maßnahme"),
        TableColumn::new("Nutzer-Effekt"),
        TableColumn::new("Risiko"),
        TableColumn::new("Conversion"),
        TableColumn::new("Rolle / Aufwand"),
    ])
    .with_title("Maßnahmenplan (Top 5) — Executive View");

    for item in selected {
        table = table.add_row(vec![
            item.action.clone(),
            item.user_effect.clone(),
            item.risk_effect.clone(),
            item.conversion_effect.clone(),
            format!("{} / {}", item.role, item.effort),
        ]);
    }

    WasJetztTunContent::Table(table)
}

fn build_module_summary_table(modules: &ModulesBlock) -> AuditTable {
    let mut table = AuditTable::new(vec![
        TableColumn::new("Modul"),
        TableColumn::new("Score"),
        TableColumn::new("Einordnung"),
        TableColumn::new("Wodurch der Wert entsteht"),
        TableColumn::new("Größter Hebel"),
    ])
    .with_title("Kurzbewertung je Modul");

    for module in &modules.dashboard {
        table = table.add_row(vec![
            module.name.clone(),
            format!("{}/100", module.score),
            module.interpretation.clone(),
            simplify_for_summary(&module.score_context),
            simplify_for_summary(&module.key_lever),
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

    let trend_color = match history.trend_label.as_str() {
        "Deutlich verbessert" | "Verbessert" => "#22c55e",
        "Stabil" => "#2563eb",
        _ => "#ef4444",
    };

    builder = builder.add_component(soft_flow_group(
        "240pt",
        vec![
            component_json(Section::new("Historie und Trend").with_level(1)),
            component_json(
                Label::new(&history.trend_label)
                    .with_size("13pt")
                    .bold()
                    .with_color(trend_color),
            ),
            component_json(TextBlock::new(&history.summary)),
            component_json(kv),
        ],
    ));

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
        .add_item("Domain", &cover.brand)
        .add_item("Prüfdatum", &cover.date)
        .add_item("Ziel", &cover.domain)
        .add_item("Zertifikat", &cover.certificate)
        .add_item("Aktive Module", cover.modules.join(", "))
}

fn component_json<C: Component>(component: C) -> serde_json::Value {
    serde_json::json!({
        "type": component.component_id(),
        "data": component.to_data()
    })
}

fn soft_flow_group(threshold: &str, items: Vec<serde_json::Value>) -> FlowGroup {
    let mut group = FlowGroup::new().with_spacing("12pt");
    for item in items {
        group = group.add_item(item);
    }
    group.with_keep_together_if_under(threshold)
}

fn build_cover_score_row(cover: &CoverBlock, badge_asset: Option<&str>) -> Grid {
    let mut grid = Grid::new(3).with_item_min_height("142pt");

    if let Some(asset_name) = badge_asset {
        grid = grid.add_item(serde_json::json!({
            "type": "image",
            "data": Image::new(asset_name).with_width("68%").to_data()
        }));
    } else {
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": MetricCard::new("Zertifikat", &cover.grade)
                .with_subtitle(format!("{} • {} / 100", cover.certificate, cover.score))
                .with_accent_color(certificate_accent_color(&cover.certificate))
                .with_height("100%")
                .to_data()
        }));
    }

    grid = grid.add_item(serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new("Accessibility", cover.score)
            .with_description(&cover.maturity_label)
            .with_thresholds(70, 50)
            .with_height("100%")
            .to_data()
    }));

    grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("Issues", cover.total_issues.to_string())
            .with_subtitle(format!("{} kritisch", cover.critical_issues))
            .with_accent_color("#dc2626")
            .with_height("100%")
            .to_data()
    }))
}

fn build_batch_cover_score_row(
    avg_score: u32,
    total_urls: u32,
    total_violations: u32,
    badge_asset: Option<&str>,
) -> anyhow::Result<Grid> {
    let mut grid = Grid::new(3).with_item_min_height("142pt");

    if let Some(asset_name) = badge_asset {
        grid = grid.add_item(serde_json::json!({
            "type": "image",
            "data": Image::new(asset_name).with_width("68%").to_data()
        }));
    } else {
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": MetricCard::new("Zertifikat", batch_grade_label(avg_score))
                .with_subtitle(format!("{} • {} / 100", batch_certificate_label(avg_score), avg_score))
                .with_accent_color(certificate_accent_color(batch_certificate_label(avg_score)))
                .with_height("100%")
                .to_data()
        }));
    }

    grid = grid.add_item(serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new("Durchschnitt", avg_score)
            .with_thresholds(70, 50)
            .with_height("100%")
            .to_data()
    }));

    Ok(grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("URLs", total_urls.to_string())
            .with_subtitle(format!("{} Verstöße", total_violations))
            .with_accent_color("#dc2626")
            .with_height("100%")
            .to_data()
    })))
}

fn certificate_accent_color(certificate: &str) -> &'static str {
    match certificate {
        "PLATINUM" => "#0f766e",
        "GOLD" => "#b45309",
        "SILVER" => "#475569",
        "BRONZE" => "#9a3412",
        "FAILED" => "#dc2626",
        _ => "#2563eb",
    }
}

fn batch_grade_label(score: u32) -> &'static str {
    match score {
        95..=100 => "A+",
        90..=94 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
}

fn batch_certificate_label(score: u32) -> &'static str {
    match score {
        95..=100 => "PLATINUM",
        85..=94 => "GOLD",
        75..=84 => "SILVER",
        65..=74 => "BRONZE",
        _ => "FAILED",
    }
}

fn certificate_badge_path(certificate: &str) -> anyhow::Result<String> {
    let (filename, svg) = match certificate {
        "PLATINUM" => (
            "auditmysite-certificate-platinum.svg",
            include_str!("../../assets/certificates/platinum.svg"),
        ),
        "GOLD" => (
            "auditmysite-certificate-gold.svg",
            include_str!("../../assets/certificates/gold.svg"),
        ),
        "SILVER" => (
            "auditmysite-certificate-silver.svg",
            include_str!("../../assets/certificates/silver.svg"),
        ),
        "BRONZE" => (
            "auditmysite-certificate-bronze.svg",
            include_str!("../../assets/certificates/bronze.svg"),
        ),
        "FAILED" => (
            "auditmysite-certificate-failed.svg",
            include_str!("../../assets/certificates/failed.svg"),
        ),
        _ => return Err(anyhow::anyhow!("unknown certificate badge: {certificate}")),
    };

    let path: PathBuf = env::temp_dir().join(filename);
    fs::write(&path, svg)?;

    Ok(path.to_string_lossy().to_string())
}

fn score_quality_label(score: u32) -> &'static str {
    match score {
        85..=100 => "Stark",
        70..=84 => "Solide",
        50..=69 => "Uneinheitlich",
        _ => "Schwach",
    }
}

fn score_quality_color(score: u32) -> &'static str {
    match score {
        85..=100 => "#16a34a",
        70..=84 => "#0f766e",
        50..=69 => "#d97706",
        _ => "#dc2626",
    }
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
            "Fundstellen",
            if group.location_hints.is_empty() {
                "Keine genaue Lokalisierung verfügbar".to_string()
            } else {
                group.location_hints.join(", ")
            },
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

    if !group.location_hints.is_empty() {
        let mut locations = List::new().with_title("Betroffene Fundstellen");
        for hint in &group.location_hints {
            locations = locations.add_item(hint);
        }
        builder = builder.add_component(locations);
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
    let normalized = match trimmed {
        "Sofort beheben" | "Sofort beheben." => "Direkt angehen",
        "Wichtig" | "Wichtig." => "Als Nächstes einplanen",
        "Optional" | "Optional." => "Bei der nächsten Optimierungsrunde mitnehmen",
        _ => trimmed,
    };
    if normalized.ends_with('.') {
        normalized.to_string()
    } else {
        format!("{normalized}.")
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
        ExecutionPriority::Immediate => "Direkt angehen",
        ExecutionPriority::Important => "Als Nächstes einplanen",
        ExecutionPriority::Optional => "Bei der nächsten Optimierungsrunde mitnehmen",
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

fn render_budget_violations(
    mut builder: renderreport::engine::ReportBuilder,
    violations: &[crate::audit::BudgetViolation],
) -> renderreport::engine::ReportBuilder {
    use crate::audit::BudgetSeverity;

    builder = builder.add_component(Section::new("Performance-Budget-Verletzungen").with_level(2));

    let error_count = violations
        .iter()
        .filter(|v| v.severity == BudgetSeverity::Error)
        .count();
    let warning_count = violations
        .iter()
        .filter(|v| v.severity == BudgetSeverity::Warning)
        .count();

    let summary_text = format!(
        "{} Budget-Verletzung{} erkannt: {} kritisch (>50% überschritten), {} Warnung{}.",
        violations.len(),
        if violations.len() == 1 { "" } else { "en" },
        error_count,
        warning_count,
        if warning_count == 1 { "" } else { "en" },
    );

    builder = if error_count > 0 {
        builder.add_component(Callout::warning(&summary_text).with_title("Budget überschritten"))
    } else {
        builder.add_component(Callout::info(&summary_text).with_title("Budget-Hinweise"))
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new("Metrik"),
        TableColumn::new("Budget"),
        TableColumn::new("Ist-Wert"),
        TableColumn::new("Überschreitung"),
        TableColumn::new("Schweregrad"),
    ])
    .with_title("Budget-Details");

    for v in violations {
        table = table.add_row(vec![
            v.metric.clone(),
            v.budget_label.clone(),
            v.actual_label.clone(),
            format!("+{:.0}%", v.exceeded_by_pct),
            v.severity.label().to_string(),
        ]);
    }

    builder = builder.add_component(table);
    builder
}

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

    if !perf.recommendations.is_empty() {
        let mut rec_list = List::new().with_title("Verbesserungsvorschläge");
        for recommendation in &perf.recommendations {
            rec_list = rec_list.add_item(recommendation);
        }
        builder = builder.add_component(rec_list);
    }

    if perf.has_render_blocking {
        builder =
            builder.add_component(Section::new("Render Blocking & Asset-Größen").with_level(3));

        if !perf.render_blocking_metrics.is_empty() {
            let mut kv = KeyValueList::new().with_title("Render-Blocking Analyse");
            for (k, v) in &perf.render_blocking_metrics {
                kv = kv.add(k, v);
            }
            builder = builder.add_component(kv);
        }

        if !perf.render_blocking_suggestions.is_empty() {
            let mut suggestions = List::new().with_title("Empfehlungen");
            for s in &perf.render_blocking_suggestions {
                suggestions = suggestions.add_item(s);
            }
            builder = builder.add_component(suggestions);
        }
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
        let mut table = AuditTable::new(vec![
            TableColumn::new("Feld").with_width("24%"),
            TableColumn::new("Wert").with_width("76%"),
        ])
        .with_title("Meta-Tags");
        for (k, v) in &seo.meta_tags {
            table = table.add_row(vec![k.clone(), v.clone()]);
        }
        builder = builder.add_component(table);
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

    if !seo.tracking_summary.is_empty() {
        let mut tracking_table = AuditTable::new(vec![
            TableColumn::new("Signal").with_width("32%"),
            TableColumn::new("Status").with_width("68%"),
        ])
        .with_title("Tracking und externe Dienste");
        for (k, v) in &seo.tracking_summary {
            tracking_table = tracking_table.add_row(vec![k.clone(), v.clone()]);
        }
        builder = builder
            .add_component(Callout::info(&seo.tracking_summary_text).with_title("Einordnung"))
            .add_component(tracking_table);
    }

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
    builder = builder.add_component(Section::new("SEO-Inhaltsprofil").with_level(3));

    let mut identity_table = AuditTable::new(vec![
        TableColumn::new("Aspekt").with_width("24%"),
        TableColumn::new("Wert").with_width("76%"),
    ])
    .with_title("Inhaltsprofil");
    for (key, value) in &profile.identity_facts {
        identity_table = identity_table.add_row(vec![key.clone(), value.clone()]);
    }

    let mut page_profile_table = AuditTable::new(vec![
        TableColumn::new("Aspekt").with_width("24%"),
        TableColumn::new("Wert").with_width("76%"),
    ])
    .with_title("Seitenprofil");
    for (key, value) in &profile.page_profile_facts {
        page_profile_table = page_profile_table.add_row(vec![key.clone(), value.clone()]);
    }

    builder = builder
        .add_component(Callout::info(&profile.identity_summary).with_title("Einordnung"))
        .add_component(identity_table)
        .add_component(page_profile_table);

    let mut score_grid = Grid::new(2);
    for (title, score, subtitle, accent) in [
        (
            "Content-Tiefe",
            profile.content_depth_score,
            score_quality_label(profile.content_depth_score),
            score_quality_color(profile.content_depth_score),
        ),
        (
            "Strukturqualität",
            profile.structural_richness_score,
            score_quality_label(profile.structural_richness_score),
            score_quality_color(profile.structural_richness_score),
        ),
        (
            "Medienbalance",
            profile.media_text_balance_score,
            score_quality_label(profile.media_text_balance_score),
            score_quality_color(profile.media_text_balance_score),
        ),
        (
            "Intent-Fit",
            profile.intent_fit_score,
            score_quality_label(profile.intent_fit_score),
            score_quality_color(profile.intent_fit_score),
        ),
    ] {
        score_grid = score_grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": MetricCard::new(title, format!("{} / 100", score))
                .with_subtitle(subtitle)
                .with_accent_color(accent)
                .to_data()
        }));
    }
    builder = builder.add_component(score_grid);

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
        let mut rec_list = List::new().with_title("Verbesserungsvorschläge");
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

fn render_dark_mode(
    mut builder: renderreport::engine::ReportBuilder,
    dm: &DarkModePresentation,
) -> renderreport::engine::ReportBuilder {
    let support_label = if dm.supported {
        "Unterstützt"
    } else {
        "Nicht unterstützt"
    };
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Dark Mode").with_level(2))
        .add_component(ScoreCard::new("Dark Mode Score", dm.score).with_thresholds(80, 50));

    let mut kv = KeyValueList::new().with_title("Dark Mode Übersicht");
    kv = kv.add("Unterstützung", support_label);
    if !dm.detection_methods.is_empty() {
        kv = kv.add("Implementierungsmethoden", dm.detection_methods.join(", "));
    }
    kv = kv.add(
        "color-scheme CSS",
        if dm.color_scheme_css { "Ja" } else { "Nein" },
    );
    if let Some(ref meta) = dm.meta_color_scheme {
        kv = kv.add("<meta name=\"color-scheme\">", meta.as_str());
    }
    if dm.css_custom_properties > 0 {
        kv = kv.add(
            "CSS Custom Properties (Farben)",
            dm.css_custom_properties.to_string(),
        );
    }
    if dm.supported {
        kv = kv.add(
            "Kontrast-Violations im Dark Mode",
            dm.dark_contrast_violations.to_string(),
        );
        if dm.dark_only_violations > 0 {
            kv = kv.add(
                "Nur-Dark-Mode-Probleme",
                format!("{} (nicht im Light Mode)", dm.dark_only_violations),
            );
        }
        if dm.light_only_violations > 0 {
            kv = kv.add(
                "Im Dark Mode behoben",
                format!(
                    "{} Light-Mode-Probleme verschwinden im Dark Mode",
                    dm.light_only_violations
                ),
            );
        }
    }
    builder = builder.add_component(kv);

    if !dm.issues.is_empty() {
        for (severity, description) in &dm.issues {
            builder = builder.add_component(match severity.as_str() {
                "high" => Callout::warning(description).with_title("Dark Mode Problem"),
                _ => Callout::info(description).with_title("Dark Mode Hinweis"),
            });
        }
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

fn extract_domain(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = without_scheme.split('/').next().unwrap_or(without_scheme);
    host.trim_start_matches("www.").to_string()
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
