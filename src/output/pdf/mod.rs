//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

mod batch;
mod cover;
mod detail_modules;
mod findings;
mod helpers;
mod history;
mod modules;

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DiagnosisPanel, DiagnosisRow, Divider, DominantIssueSpotlight,
    KeyValueList, List, MetricStrip, MetricStripItem, PageBreak, PhaseBlock, RecommendationCard,
    SectionHeaderSplit, TableOfContents,
};
use renderreport::components::text::{Label, TextBlock};
use renderreport::prelude::Image;
use renderreport::prelude::*;

// Composite components
use renderreport::components::{
    AuditTable, BenchmarkRow, BenchmarkTable, ComparisonModule, ModuleComparison, SeverityOverview,
    TableColumn,
};

use crate::audit::{normalize, AuditReport, BatchReport};
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::builder::{build_batch_presentation, build_view_model};
use crate::output::report_model::*;
use crate::util::truncate_url;

use self::batch::{build_batch_benchmark_summary, build_batch_overview_grid, render_action_plan};
use self::cover::{
    auditmysite_wordmark_path, batch_certificate_label, build_batch_cover_score_row,
    build_cover_meta, build_cover_score_row, certificate_badge_path,
};
use self::detail_modules::{
    render_budget_violations, render_dark_mode, render_mobile, render_performance, render_security,
    render_seo,
};
use self::findings::{
    build_analysis_focus_table, render_finding_group, render_finding_technical,
    render_key_finding_block,
};
use self::helpers::{
    component_json, create_engine, extract_domain, severity_label_i18n, soft_flow_group,
};
use self::history::{render_history_section, render_methodology_section};
use self::modules::{
    build_module_cards_grid, build_module_radar_chart, build_overall_score_card,
    build_summary_overview, build_top_hebel_table, build_was_jetzt_tun_table, WasJetztTunContent,
};

// ─── Single Report ──────────────────────────────────────────────────────────

pub fn generate_pdf(report: &AuditReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let normalized = normalize(report);
    let vm = build_view_model(&normalized, config);
    let i18n = I18n::new(&config.locale)?;
    let wordmark_asset = "/auditmysite-wordmark.svg";

    let mut builder = engine
        .report("wcag-audit")
        .metadata("date", &vm.meta.date)
        .metadata("version", &vm.meta.version)
        .metadata("author", &vm.meta.author)
        .metadata("score", &vm.meta.score_label)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "");

    if let Ok(path) = auditmysite_wordmark_path() {
        builder = builder.asset(wordmark_asset, path);
    }

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
        .add_component(build_cover_meta(&vm.cover, &vm.meta.version));

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
    // Split into two flow groups so the ImpactGrid never breaks across pages.
    // Group A: Section + Hero + KPI-Strip
    // Group B: Impact triad + Text summary
    {
        let hero_items = vec![
            component_json(Section::new("Ergebnisblock").with_level(1)),
            // ROW 1: HERO — score card + 3 metric cards
            component_json(build_summary_overview(&vm.summary)),
            component_json(Divider::new()),
            // ROW 2: KPI STRIP — counts
            component_json(MetricStrip::new(vec![
                MetricStripItem::new("Issues gesamt", vm.severity.total.to_string()),
                MetricStripItem::new("Kritisch", vm.severity.critical.to_string())
                    .with_status("bad"),
                MetricStripItem::new("Hoch", vm.severity.high.to_string()).with_status("warn"),
                MetricStripItem::new("Problemgruppen", vm.findings.top_findings.len().to_string()),
            ])),
            component_json(Divider::new()),
        ];
        builder = builder.add_component(soft_flow_group("240pt", hero_items));

        // ROW 3: IMPACT — Nutzer | Business | Risiko (as KeyValueList for full content)
        let mut impact_items = Vec::new();
        {
            let mut impact_kv = KeyValueList::new().with_title("Auswirkungen");

            let nutzer_text = if !vm.summary.overall_impact.is_empty() {
                vm.summary.overall_impact[0].1.clone()
            } else if !vm.summary.business_consequence.is_empty() {
                vm.summary.business_consequence.clone()
            } else {
                vm.summary.executive_lead.clone()
            };
            impact_kv = impact_kv.add("Nutzer erleben", nutzer_text);

            let business_text = if !vm.summary.business_consequence.is_empty() {
                vm.summary.business_consequence.clone()
            } else {
                vm.summary.problem_type.clone()
            };
            impact_kv = impact_kv.add("Business-Auswirkung", business_text);

            impact_kv = impact_kv.add(
                "Risiko",
                "WCAG-Verstöße können rechtliche Relevanz haben (BFSG).",
            );

            impact_items.push(component_json(impact_kv));
            impact_items.push(component_json(Divider::new()));
        }
        // ROW 4: TEXT — Einordnung · Problemtyp · Benchmark
        {
            let mut kv = KeyValueList::new();
            kv = kv
                .add("Einordnung", vm.summary.executive_lead.clone())
                .add("Problemtyp", vm.summary.problem_type.clone())
                .add("Benchmark", vm.summary.benchmark_context.clone());
            impact_items.push(component_json(kv));
        }
        builder = builder.add_component(soft_flow_group("200pt", impact_items));
    }

    // ── Top-Hebel-Block ──────────────────────────────────────────────
    {
        let total_ch = (vm.severity.critical + vm.severity.high) as usize;
        if let Some(table) = build_top_hebel_table(&vm.findings, total_ch) {
            let mut items = vec![
                component_json(SectionHeaderSplit::new(
                    "Top-Hebel",
                    "Die drei häufigsten Probleme und ihr Anteil an den kritischen Findings — wenn du nur eine Sache angehst, dann diese.",
                ).with_level(1)),
                component_json(table),
            ];
            if let Some(ref note) = vm.summary.dominant_issue_note {
                if let Some(top) = vm.findings.top_findings.first() {
                    let spotlight = DominantIssueSpotlight::new(
                        &top.title,
                        format!("{:?}", top.severity).to_lowercase(),
                        note,
                        &top.user_impact,
                        &top.recommendation,
                    )
                    .with_eyebrow("Dominierendes Problem")
                    .with_affected_count(top.affected_elements as u32);
                    items.push(component_json(spotlight));
                } else {
                    items.push(component_json(
                        Callout::warning(note).with_title("Dominierendes Problem"),
                    ));
                }
            }
            builder = builder.add_component(soft_flow_group("280pt", items));
        }
    }

    // Executive level: compact view
    if vm.meta.report_level == ReportLevel::Executive {
        // ── 2. Gesamtbewertung (Executive) ──────────────────────────
        {
            let mut items = vec![component_json(
                Section::new("Gesamtbewertung").with_level(1),
            )];
            if let Some(overall) = vm.modules.overall_score {
                items.push(component_json(build_overall_score_card(overall)));
                items.push(component_json(Divider::new()));
            }
            items.push(component_json(build_module_cards_grid(&vm.modules)));
            if vm.modules.dashboard.len() >= 2 {
                items.push(component_json(build_module_radar_chart(&vm.modules)));
            }
            if let Some(ref overall_text) = vm.modules.overall_interpretation {
                items.push(component_json(
                    Callout::info(overall_text).with_title("Gesamtscore einordnen"),
                ));
            }
            builder = builder
                .add_component(PageBreak::new())
                .add_component(soft_flow_group("280pt", items));
        }

        // ── 3. Trend (Executive) ─────────────────────────────────────
        if let Some(ref history) = vm.history {
            builder = render_history_section(builder, history);
        }

        // ── 4. Was jetzt tun? (Executive) ────────────────────────────
        {
            let wjt_table = build_was_jetzt_tun_table(&vm);
            let mut wjt_items = vec![
                component_json(SectionHeaderSplit::new(
                    "Was jetzt tun?",
                    "Die folgenden Maßnahmen haben die höchste Wirkung. Jede Maßnahme ist direkt umsetzbar — kein Abstract, keine langen Tabellen.",
                ).with_level(1)),
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
    {
        let mut items = vec![component_json(
            Section::new("Gesamtbewertung").with_level(1),
        )];
        if let Some(overall) = vm.modules.overall_score {
            items.push(component_json(build_overall_score_card(overall)));
            items.push(component_json(Divider::new()));
        }
        items.push(component_json(build_module_cards_grid(&vm.modules)));
        if vm.modules.dashboard.len() >= 2 {
            items.push(component_json(build_module_radar_chart(&vm.modules)));
        }
        if let Some(ref overall_text) = vm.modules.overall_interpretation {
            items.push(component_json(
                Callout::info(overall_text).with_title("Gesamtscore einordnen"),
            ));
        }
        builder = builder.add_component(soft_flow_group("320pt", items));
    }

    // ── 3. Entwicklung / Trend ──────────────────────────────────────
    if let Some(ref history) = vm.history {
        builder = render_history_section(builder, history);
    }

    // ── 4. Was jetzt tun? ───────────────────────────────────────────
    {
        let wjt_table = build_was_jetzt_tun_table(&vm);
        let mut wjt_items = vec![component_json(
            SectionHeaderSplit::new(&vm.actions.block_title, &vm.actions.intro_text).with_level(1),
        )];
        // Phase-Preview: visual overview using PhaseBlock (FIX 5: priority grouping)
        if !vm.actions.phase_preview.is_empty() {
            let phase_count = vm.actions.phase_preview.len();
            for (i, phase) in vm.actions.phase_preview.iter().enumerate() {
                let mut block =
                    PhaseBlock::new((i + 1) as u8, &phase.phase_label, &phase.description)
                        .with_items(phase.top_items.clone())
                        .with_total(phase.item_count);
                if !phase.accent_color.is_empty() {
                    block = block.with_color(&phase.accent_color);
                }
                wjt_items.push(component_json(block));
                if i + 1 < phase_count {
                    wjt_items.push(component_json(Divider::new()));
                }
            }
            wjt_items.push(component_json(Divider::new()));
        }
        match wjt_table {
            WasJetztTunContent::Table(t) => wjt_items.push(component_json(t)),
            WasJetztTunContent::Empty(c) => wjt_items.push(component_json(c)),
        }
        builder = builder
            .add_component(PageBreak::new())
            .add_component(soft_flow_group("340pt", wjt_items));
    }

    // ── 5. Key Findings ─────────────────────────────────────────────
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
        .add_component(SectionHeaderSplit::new(&findings_title, findings_intro).with_level(1));

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
                component_json(SectionHeaderSplit::new(
                    "Technische Analyse und Umsetzung",
                    "Ab hier folgt die technische Sicht für Entwicklung, Design und Redaktion. Die folgenden Abschnitte zeigen betroffene Elemente, konkrete Maßnahmen und technische Hinweise für die Umsetzung.",
                ).with_level(1)),
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
        // Module health diagnosis
        let mut diag_rows = Vec::new();
        for module in &vm.modules.dashboard {
            let status = if module.score >= 80 {
                "good"
            } else if module.score >= 50 {
                "warn"
            } else {
                "bad"
            };
            diag_rows.push(
                DiagnosisRow::new(&module.name, format!("{}/100", module.score))
                    .with_status(status),
            );
        }
        if !diag_rows.is_empty() {
            builder =
                builder.add_component(DiagnosisPanel::new(diag_rows).with_title("Modulübersicht"));
        }

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
                    component_json(SectionHeaderSplit::new(
                        "Technische Detailmetriken",
                        "Die folgenden Kennzahlen ergänzen die Modul-Übersicht um technische Detailwerte für Analyse und Umsetzung.",
                    ).with_level(2)),
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
                    self::helpers::map_severity(&v.severity),
                    &desc,
                ));
            }
        } else {
            let rows: Vec<ChecklistRow> = vm
                .appendix
                .violations
                .iter()
                .map(|v| {
                    let status = match v.severity {
                        crate::wcag::Severity::Critical => "bad",
                        crate::wcag::Severity::High => "warn",
                        _ => "neutral",
                    };
                    ChecklistRow::new(format!("{} — {}", v.rule, v.rule_name), v.message.clone())
                        .with_status(status)
                })
                .collect();
            builder = builder.add_component(
                ChecklistPanel::new(rows).with_title("Alle Verstöße (aggregiert nach Regel)"),
            );
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
    let wordmark_asset = "/auditmysite-wordmark.svg";

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

    if let Ok(path) = auditmysite_wordmark_path() {
        builder = builder.asset(wordmark_asset, path);
    }

    if let Some(ref logo_path) = config.logo_path {
        if logo_path.exists() {
            builder = builder.add_component(
                Image::new(logo_path.to_string_lossy().to_string()).with_width("30%"),
            );
        }
    }

    let batch_score = pres.portfolio_summary.average_score.round() as u32;
    builder = builder
        .add_component(Image::new(wordmark_asset).with_width("22%"))
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
        .add_component(
            Label::new(format!("auditmysite v{}", pres.cover.version))
                .with_size("9pt")
                .with_color("#94a3b8"),
        )
        .add_component(PageBreak::new());
    if config.level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    // ── 1. Portfolio Overview ────────────────────────────────────────
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
                component_json(build_batch_benchmark_summary(&pres)),
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
        .add_component(
            SectionHeaderSplit::new(
                "URL-Ranking",
                "Übersicht aller geprüften URLs, sortiert nach Score. \
             URLs mit niedrigerem Score haben höheren Handlungsbedarf.",
            )
            .with_level(1),
        )
        .add_component(BenchmarkTable::new(rows))
        .add_component(PageBreak::new());

    // ── 3. Top-Probleme ─────────────────────────────────────────────
    builder = builder.add_component(
        SectionHeaderSplit::new(
            "Häufigste Probleme",
            "Die folgenden Problemgruppen treten über mehrere URLs hinweg auf. \
             Durch Behebung dieser Probleme wird die größte Verbesserung erzielt, \
             da sie viele Seiten gleichzeitig betreffen.",
        )
        .with_level(1),
    );

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
                self::helpers::priority_label_i18n(issue.priority, &i18n),
            ]);
        }
        builder = builder.add_component(freq_table);
    }

    // Quick-reference cards before detailed findings
    for group in pres.top_issues.iter().take(3) {
        let mut card = RecommendationCard::new(&group.title, &group.customer_description);
        if !group.severity.to_string().is_empty() {
            card = card.with_priority(format!("{:?}", group.severity).to_lowercase());
        }
        if !group.recommendation.is_empty() {
            card = card.with_impact(&group.recommendation);
        }
        builder = builder.add_component(card);
    }
    builder = builder.add_component(PageBreak::new());

    for group in pres.top_issues.iter().take(3) {
        builder = render_finding_group(builder, group, &i18n);
    }

    // ── 4. Maßnahmenplan ────────────────────────────────────────────
    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(
            "Maßnahmenplan",
            "Die folgenden Maßnahmen sind nach Aufwand und Wirkung priorisiert. \
             Maßnahmen, die viele Seiten gleichzeitig verbessern, haben Vorrang.",
        )
        .with_level(1),
    );
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
        .add_component(SectionHeaderSplit::new("Technische URL-Matrix", "Verdichtete Übersicht aller geprüften URLs mit Fokus auf technische Priorisierung. Jede Zeile zeigt Score, Problemintensität und den größten Hebel für die nächste Optimierungsrunde.").with_level(1));

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
        .add_component(SectionHeaderSplit::new("Content- und Seitenprofil", "Die folgenden Auswertungen ordnen Inhalt, Seitentypen und thematische Schwerpunkte ein. Dieser Block ist bewusst nachgelagert und ergänzt die technische Priorisierung um SEO- und Content-Signale.").with_level(1));

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
        let rows: Vec<ChecklistRow> = pres
            .portfolio_summary
            .distribution_insights
            .iter()
            .map(|insight| ChecklistRow::new("Auffälligkeit", insight.as_str()))
            .collect();
        builder =
            builder.add_component(ChecklistPanel::new(rows).with_title("Content-Auffälligkeiten"));
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
            .add_component(SectionHeaderSplit::new("Near-Duplicate Content", "Die folgenden Seitenpaare haben inhaltlich sehr ähnliche Texte (SimHash-Ähnlichkeit ≥ 80 %). \
                 Near-Duplicate-Content kann zu Keyword-Kannibalisierung und schlechteren Rankings führen. \
                 Empfehlung: Inhalte konsolidieren oder stärker differenzieren.").with_level(1));

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
        builder = builder.add_component(PageBreak::new()).add_component(
            SectionHeaderSplit::new(
                "Anhang: Technische Details",
                "Vollständige Auflistung aller erkannten Verstöße pro URL \
                 mit technischen Details für die Umsetzung.",
            )
            .with_level(1),
        );

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
    let wordmark_asset = "/auditmysite-wordmark.svg";

    let mut builder = engine
        .report("wcag-comparison")
        .metadata("date", chrono::Local::now().format("%d.%m.%Y").to_string())
        .metadata("author", &author)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "");

    if let Ok(path) = auditmysite_wordmark_path() {
        builder = builder.asset(wordmark_asset, path);
    }

    builder = builder
        .add_component(Image::new(wordmark_asset).with_width("22%"))
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
        .add_component(
            Label::new(format!("auditmysite v{}", env!("CARGO_PKG_VERSION")))
                .with_size("9pt")
                .with_color("#94a3b8"),
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
