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
    ChecklistPanel, ChecklistRow, DiagnosisPanel, DiagnosisRow, DominantIssueSpotlight, ImpactGrid,
    ImpactGridCard, KeyValueList, List, MetricStrip, MetricStripItem, PageBreak, PhaseBlock,
    SectionHeaderSplit, TableOfContents,
};
use renderreport::components::text::{Label, TextBlock};
use renderreport::prelude::Image;
use renderreport::prelude::*;

// Composite components
use renderreport::components::{
    AuditTable, BenchmarkRow, BenchmarkTable, ComparisonModule, ModuleComparison, SeverityOverview,
    SummaryBox, TableColumn,
};

use crate::audit::{normalize, AuditReport, BatchReport};
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::builder::{build_batch_presentation, build_view_model};
use crate::output::report_model::*;
use crate::util::truncate_url;

use self::batch::build_batch_overview_grid;
use self::cover::{
    auditmysite_wordmark_path, batch_certificate_label, build_batch_cover_score_row,
    build_cover_score_row, certificate_badge_path,
};
use self::detail_modules::{
    render_ai_visibility, render_budget_violations, render_dark_mode, render_journey,
    render_mobile, render_performance, render_security, render_seo, render_source_quality,
    render_ux,
};
use self::findings::{first_sentence, render_finding_technical, render_key_finding_block};
use self::helpers::{
    component_json, create_engine, extract_domain, priority_label_i18n, role_label_i18n,
    severity_label_i18n, soft_flow_group,
};
use self::history::{render_history_section, render_methodology_section};
use self::modules::{
    build_summary_overview, build_top_hebel_table, build_was_jetzt_tun_table, WasJetztTunContent,
};

// ─── Single Report ──────────────────────────────────────────────────────────
//
// 6-page structure:
//   Page 1 — Hero / Entry (pitch: status, scores, impact, consequences)
//   Page 2 — Dominant Issue (focus: biggest problem, top fixes, leverage)
//   Page 3 — Key Findings (bridge: Problem/Impact/Ursache/Fix cards)
//   Page 4 — Action Plan (decide: quick wins, action table, execution note)
//   Page 5 — Tech Entry (transition: intro + severity overview)
//   Page 6+ — Tech Details (implement: WCAG details, code examples, modules)

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
        .add_component(Image::new(wordmark_asset).with_width("120pt"))
        .add_component(
            Label::new(&vm.executive.cover_eyebrow)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(Label::new(&vm.cover.title).with_size("28pt").bold())
        .add_component(
            Label::new(&vm.executive.cover_kicker)
                .with_size("12pt")
                .with_color("#475569"),
        );

    let single_badge_asset = "/certificate-badge-single.svg";
    let single_badge_enabled = if let Ok(path) = certificate_badge_path(&vm.cover.certificate) {
        builder = builder.asset(single_badge_asset, path);
        true
    } else {
        false
    };

    builder = builder.add_component(build_cover_score_row(
        &vm.cover,
        single_badge_enabled.then_some(single_badge_asset),
    ));

    // HeroAssessment: concise pattern-based headline statement (D)
    if !vm.summary.executive_lead.is_empty() {
        builder = builder.add_component(
            Label::new(&vm.summary.executive_lead)
                .with_size("13pt")
                .bold()
                .with_color("#0f4c42"),
        );
    }

    builder = builder
        .add_component(
            TextBlock::new(&vm.summary.verdict)
                .with_size("11pt")
                .with_line_height("1.4em")
                .with_max_width("100%"),
        )
        .add_component(build_cover_fact_strip(&vm))
        .add_component(PageBreak::new());

    if vm.meta.report_level != ReportLevel::Executive {
        builder = builder
            .add_component(TableOfContents::new())
            .add_component(PageBreak::new());
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 1 — STATUS & BEWERTUNG
    // Goal: In 10 Sekunden verstehen: Wie schlimm? Warum relevant? Was tun?
    // ─────────────────────────────────────────────────────────────────
    {
        let risk_callout = match vm.summary.risk_level.as_str() {
            "Kritisch" => {
                Callout::error(&vm.summary.risk_summary).with_title(&vm.executive.risk_title)
            }
            "Hoch" => {
                Callout::warning(&vm.summary.risk_summary).with_title(&vm.executive.risk_title)
            }
            _ => Callout::info(&vm.summary.risk_summary).with_title(&vm.executive.risk_title),
        };

        builder = builder.add_component(soft_flow_group(
            "180pt",
            vec![
                component_json(
                    SectionHeaderSplit::new(
                        &vm.executive.status_title,
                        "Einordnung, Risiko und die geschäftlich wichtigsten Konsequenzen auf einen Blick.",
                    )
                    .with_eyebrow("EXECUTIVE SNAPSHOT")
                    .with_level(1)
                ),
                component_json(risk_callout),
            ],
        ));

        // ScoreBlock — scores + issue counts (sekundär)
        builder = builder.add_component(soft_flow_group(
            "200pt",
            vec![
                component_json(build_summary_overview(&vm.summary)),
                component_json(
                    MetricStrip::new(vec![
                        MetricStripItem::new("Gesamtscore", vm.summary.score.to_string())
                            .with_accent("#0f766e"),
                        MetricStripItem::new("Probleme erkannt", vm.severity.total.to_string()),
                        MetricStripItem::new(
                            "Kritisch / Hoch",
                            format!("{}", vm.severity.critical + vm.severity.high),
                        )
                        .with_status("bad")
                        .with_accent("#dc2626"),
                        MetricStripItem::new("Risiko", &vm.summary.risk_level)
                            .with_status(risk_status(&vm.summary.risk_level)),
                        MetricStripItem::new("Zertifikat", &vm.summary.certificate)
                            .with_accent("#7c3aed"),
                    ])
                    .compact(),
                ),
            ],
        ));

        builder = builder.add_component(build_module_strip(&vm));
        builder = builder.add_component(build_raw_audit_snapshot(&vm));

        // Block 2: Kernaussagen — max 3 Punkte, direkt aus Daten
        {
            let mut kp_list = List::new().with_title(&vm.executive.key_points_title);
            for point in &vm.executive.key_points {
                kp_list = kp_list.add_item(point);
            }
            builder = builder.add_component(kp_list);
        }

        // Auswirkungen — Nutzer / Business / Risiko
        {
            let user = impact_row(&vm.executive.impact_rows, "Nutzer");
            let business = impact_row(&vm.executive.impact_rows, "Business");
            let risk = impact_row(&vm.executive.impact_rows, "Risiko");
            builder = builder.add_component(
                ImpactGrid::new(
                    ImpactGridCard::new("Nutzer", "Nutzbarkeit", user).with_status("warn"),
                    ImpactGridCard::new("Risiko", "Compliance", risk).with_status("bad"),
                    ImpactGridCard::new("Business", "Geschäftswirkung", business)
                        .with_status("info"),
                )
                .with_title(&vm.executive.impact_title),
            );
        }

        // Block 3: Handlungsempfehlung — sehr konkret
        {
            if !vm.executive.quick_actions.is_empty() {
                let rows: Vec<ChecklistRow> = vm
                    .executive
                    .quick_actions
                    .iter()
                    .map(|(action, timeframe)| {
                        ChecklistRow::new(timeframe, action).with_status("warn")
                    })
                    .collect();
                builder = builder.add_component(
                    ChecklistPanel::new(rows).with_title(&vm.executive.quick_actions_title),
                );
            }
        }

        if !vm.summary.positive_aspects.is_empty() {
            let rows: Vec<ChecklistRow> = vm
                .summary
                .positive_aspects
                .iter()
                .take(3)
                .enumerate()
                .map(|(i, item)| {
                    ChecklistRow::new(format!("Stärke {}", i + 1), item).with_status("good")
                })
                .collect();
            builder = builder
                .add_component(ChecklistPanel::new(rows).with_title("Was bereits stark ist"));
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 2 — DOMINANT ISSUE
    // Goal: focus — one problem, leverage, top fixes table
    // ─────────────────────────────────────────────────────────────────
    {
        let total_ch = (vm.severity.critical + vm.severity.high) as usize;

        // DominantIssueHero — percentage dominant, minimal text
        if let Some(top) = vm.findings.top_findings.first() {
            // Share = this finding's occurrences / total occurrences across all findings.
            // Fallback to total_ch if no occurrence data; never show 0% for the only finding.
            let total_occurrences: usize = vm
                .findings
                .top_findings
                .iter()
                .map(|f| f.occurrence_count)
                .sum();
            let share = if total_occurrences > 0 {
                (top.occurrence_count * 100 / total_occurrences).max(1)
            } else if total_ch > 0 {
                (top.occurrence_count * 100 / total_ch).max(1)
            } else {
                100
            };
            let spotlight = DominantIssueSpotlight::new(
                &top.title,
                format!("{:?}", top.severity).to_lowercase(),
                &vm.executive.spotlight_body,
                &vm.executive.spotlight_impact,
                &vm.executive.spotlight_recommendation,
            )
            .with_eyebrow(&vm.executive.spotlight_eyebrow)
            .with_affected_count(share as u32);
            builder = builder.add_component(spotlight);
        }

        // TopFixesTable
        if let Some(table) = build_top_hebel_table(&vm.findings, total_ch) {
            builder =
                builder.add_component(table.with_title("Die wichtigsten Probleme im Überblick"));
        }

        // LeverageBlock — what fixing achieves
        if total_ch > 0 {
            if let Some(leverage_text) = &vm.executive.leverage_text {
                builder = builder.add_component(
                    Callout::success(leverage_text).with_title(&vm.executive.leverage_title),
                );
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 3 — KEY FINDINGS
    // Goal: understand — compact cards, no tech detail here
    // ─────────────────────────────────────────────────────────────────
    {
        builder = builder.add_component(
            SectionHeaderSplit::new(&vm.executive.findings_title, &vm.executive.findings_intro)
                .with_level(1),
        );

        // FindingCards — compact: Problem/Impact/Ursache/Fix
        for group in vm.findings.top_findings.iter().take(5) {
            builder = render_key_finding_block(builder, group, &i18n);
        }
    }

    if let Some(ref history) = vm.history {
        builder = render_history_section(builder, history);
    }

    // Executive level stops here
    if vm.meta.report_level == ReportLevel::Executive {
        builder = render_methodology_section(builder, &vm.methodology, &i18n);
        let built_report = builder.build();
        return Ok(engine.render_pdf(&built_report)?);
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 4 — ACTION PLAN
    // Goal: decide — quick wins, prioritized actions, execution note
    // ─────────────────────────────────────────────────────────────────
    {
        builder = builder.add_component(
            SectionHeaderSplit::new(
                &vm.executive.action_plan_title,
                &vm.executive.action_plan_intro,
            )
            .with_eyebrow("UMSETZUNGSPLAN")
            .with_level(1),
        );

        // Empfohlene Vorgehensweise
        builder = builder.add_component(
            Callout::info(&vm.executive.action_plan_callout_body)
                .with_title(&vm.executive.action_plan_callout_title),
        );

        if !vm.actions.phase_preview.is_empty() {
            for (idx, phase) in vm.actions.phase_preview.iter().enumerate() {
                builder = builder.add_component(
                    PhaseBlock::new((idx + 1) as u8, &phase.phase_label, &phase.description)
                        .with_items(phase.top_items.clone())
                        .with_total(phase.item_count)
                        .with_color(&phase.accent_color),
                );
            }
        }

        // QuickWins — immediate actions
        let quick_items: Vec<&RoadmapItemData> = vm
            .actions
            .roadmap_columns
            .iter()
            .flat_map(|col| col.items.iter())
            .filter(|i| {
                i.execution_priority.contains("Direkt") || i.execution_priority.contains("Sofort")
            })
            .take(5)
            .collect();

        if !quick_items.is_empty() {
            let rows: Vec<ChecklistRow> = quick_items
                .iter()
                .map(|item| ChecklistRow::new(&item.action, &item.user_effect).with_status("warn"))
                .collect();
            builder =
                builder.add_component(ChecklistPanel::new(rows).with_title("Sofort umsetzbar"));
        }

        // ActionTable — full prioritized table
        let wjt_table = build_was_jetzt_tun_table(&vm);
        match wjt_table {
            WasJetztTunContent::Table(t) => builder = builder.add_component(t),
            WasJetztTunContent::Empty(c) => builder = builder.add_component(c),
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 5 — TECH ENTRY
    // Goal: transition — intro for dev/design/content, severity overview
    // ─────────────────────────────────────────────────────────────────
    {
        builder = builder.add_component(
            SectionHeaderSplit::new(&vm.executive.technical_title, &vm.executive.technical_intro)
                .with_eyebrow("TECHNICAL HANDOFF")
                .with_level(1),
        );

        // Module health diagnosis
        if !vm.modules.dashboard.is_empty() {
            let diag_rows: Vec<DiagnosisRow> = vm
                .modules
                .dashboard
                .iter()
                .map(|module| {
                    let status = if module.score >= 80 {
                        "good"
                    } else if module.score >= 50 {
                        "warn"
                    } else {
                        "bad"
                    };
                    DiagnosisRow::new(&module.name, format!("{}/100", module.score))
                        .with_status(status)
                })
                .collect();
            builder =
                builder.add_component(DiagnosisPanel::new(diag_rows).with_title("Modulübersicht"));
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 6+ — TECH DETAILS
    // Goal: implement — WCAG details, code examples, module metrics
    // ─────────────────────────────────────────────────────────────────
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

    // ── Module Detail Metrics ───────────────────────────────────────
    if vm.module_details.has_any {
        builder = builder.add_component(Section::new("Technische Detailmetriken").with_level(2));
    }

    if let Some(ref perf) = vm.module_details.performance {
        builder = render_performance(builder, perf);
    }
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
    if let Some(ref ux) = vm.module_details.ux {
        builder = render_ux(builder, ux);
    }
    if let Some(ref journey) = vm.module_details.journey {
        builder = render_journey(builder, journey);
    }
    if let Some(ref dm) = vm.module_details.dark_mode {
        builder = render_dark_mode(builder, dm);
    }
    if let Some(ref sq) = vm.module_details.source_quality {
        builder = render_source_quality(builder, sq);
    }
    if let Some(ref av) = vm.module_details.ai_visibility {
        builder = render_ai_visibility(builder, av);
    }

    // ── Appendix ────────────────────────────────────────────────────
    if vm.appendix.has_violations {
        builder = builder.add_component(Section::new(i18n.t("section-appendix")).with_level(1));

        builder = builder.add_component(build_cli_snapshot_table(&vm));

        if vm.meta.report_level == ReportLevel::Technical {
            for v in &vm.appendix.violations {
                let mut desc = v.message.clone();
                if let Some(ref fix) = v.fix_suggestion {
                    desc.push_str(&format!("\n\nFix: {}", fix));
                }
                desc.push_str(&format!(
                    "\n\n{} Elemente betroffen",
                    v.affected_elements.len()
                ));
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

    // ── Empfohlene nächste Schritte ───────────────────────────────
    builder = render_next_steps_single(builder, &vm);

    // ── Methodology ─────────────────────────────────────────────────
    builder = render_methodology_section(builder, &vm.methodology, &i18n);

    let built_report = builder.build();
    Ok(engine.render_pdf(&built_report)?)
}

fn build_cover_fact_strip(vm: &ReportViewModel) -> MetricStrip {
    MetricStrip::new(vec![
        MetricStripItem::new("Domain", extract_domain(&vm.cover.domain)).with_accent("#0f766e"),
        MetricStripItem::new("Report-Level", format!("{:?}", vm.meta.report_level))
            .with_accent("#475569"),
        MetricStripItem::new("Module", vm.cover.modules.len().to_string()).with_accent("#7c3aed"),
        MetricStripItem::new("Prüfdatum", &vm.cover.date).with_accent("#b45309"),
    ])
}

fn build_module_strip(vm: &ReportViewModel) -> MetricStrip {
    let items = vm
        .modules
        .dashboard
        .iter()
        .take(6)
        .map(|module| {
            let status = if module.score >= 85 {
                "good"
            } else if module.score >= 70 {
                "info"
            } else if module.score >= 50 {
                "warn"
            } else {
                "bad"
            };
            MetricStripItem::new(&module.name, format!("{}/100", module.score))
                .with_status(status)
                .with_accent(module_score_color(module.score))
        })
        .collect();
    MetricStrip::new(items).compact()
}

fn impact_row<'a>(rows: &'a [(String, String)], label: &str) -> &'a str {
    rows.iter()
        .find(|(key, _)| key == label)
        .map(|(_, value)| value.as_str())
        .unwrap_or("Keine Details verfügbar.")
}

fn build_cli_snapshot_table(vm: &ReportViewModel) -> AuditTable {
    let mut table = AuditTable::new(vec![
        TableColumn::new("Bereich").with_width("22%"),
        TableColumn::new("Signal").with_width("28%"),
        TableColumn::new("Wert").with_width("50%"),
    ])
    .with_title("Technical Snapshot (CLI parity)");

    for (label, value) in &vm.methodology.audit_facts {
        table = table.add_row(vec!["Audit".to_string(), label.clone(), value.clone()]);
    }

    for module in &vm.modules.dashboard {
        table = table.add_row(vec![
            "Modul".to_string(),
            module.name.clone(),
            format!(
                "{} / 100 — {}. {}",
                module.score, module.interpretation, module.card_context
            ),
        ]);
    }

    for finding in vm.findings.top_findings.iter().take(6) {
        table = table.add_row(vec![
            "Finding".to_string(),
            format!("{} ({})", finding.rule_id, finding.wcag_criterion),
            format!(
                "{} Vorkommen — {}",
                finding.occurrence_count,
                first_sentence(&finding.user_impact)
            ),
        ]);
    }

    table
}

fn build_raw_audit_snapshot(vm: &ReportViewModel) -> SummaryBox {
    SummaryBox::new("Raw Audit Snapshot")
        .add_item(
            "WCAG-Level",
            vm.methodology
                .audit_facts
                .iter()
                .find(|(label, _)| label == "WCAG-Level")
                .map(|(_, value)| value.as_str())
                .unwrap_or("n/a"),
        )
        .add_item(
            "Geprüfte Knoten",
            vm.methodology
                .audit_facts
                .iter()
                .find(|(label, _)| label == "Geprüfte Knoten")
                .map(|(_, value)| value.as_str())
                .unwrap_or("n/a"),
        )
        .add_item(
            "Laufzeit",
            vm.methodology
                .audit_facts
                .iter()
                .find(|(label, _)| label == "Laufzeit")
                .map(|(_, value)| value.as_str())
                .unwrap_or("n/a"),
        )
        .add_item("Findings gesamt", vm.severity.total.to_string())
        .add_item(
            "Kritisch / Hoch",
            format!("{}", vm.severity.critical + vm.severity.high),
        )
        .add_item(
            "Audit-Hinweise",
            vm.methodology
                .audit_facts
                .iter()
                .find(|(label, _)| label == "Audit-Hinweise")
                .map(|(_, value)| value.as_str())
                .unwrap_or("0"),
        )
}

fn module_score_color(score: u32) -> &'static str {
    if score >= 85 {
        "#0f766e"
    } else if score >= 70 {
        "#2563eb"
    } else if score >= 50 {
        "#d97706"
    } else {
        "#dc2626"
    }
}

fn risk_status(level: &str) -> &'static str {
    match level {
        "Kritisch" => "bad",
        "Hoch" => "warn",
        "Mittel" => "info",
        _ => "good",
    }
}

/// Closing section: recommended next steps
fn render_next_steps_single(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            &vm.executive.next_steps_title,
            &vm.executive.next_steps_intro,
        )
        .with_level(1),
    );

    let mut steps: Vec<String> = Vec::new();

    // Highest priority from quick wins
    for col in &vm.actions.roadmap_columns {
        for item in &col.items {
            if steps.len() >= 3 {
                break;
            }
            steps.push(item.action.clone());
        }
    }

    // Fallback from findings
    if steps.is_empty() {
        for group in vm.findings.top_findings.iter().take(3) {
            steps.push(first_sentence(&group.recommendation).to_string());
        }
    }

    if !steps.is_empty() {
        let mut list = List::new();
        for action in &steps {
            list = list.add_item(action);
        }
        builder = builder.add_component(list);
    }

    builder = builder.add_component(
        Callout::info(&vm.executive.next_steps_callout_body)
            .with_title(&vm.executive.next_steps_callout_title),
    );

    builder
}

// ─── Helper: Business Relevance ─────────────────────────────────────────────

/// Map page type + URL to business relevance (hoch/mittel/niedrig)
fn format_word_count(n: u32) -> String {
    if n >= 1_000 {
        format!("{}.{:03}", n / 1_000, n % 1_000)
    } else {
        n.to_string()
    }
}

fn business_relevance(page_type: Option<&str>, url: &str) -> &'static str {
    // URL-based heuristics first
    let path = url.to_lowercase();
    if path.contains("impressum") || path.contains("datenschutz") || path.contains("agb") {
        return "niedrig";
    }
    if path.ends_with('/') && path.matches('/').count() <= 3 {
        return "hoch"; // homepage or top-level pages
    }

    // Page type based
    match page_type {
        Some("Marketing / Landing Page") => "hoch",
        Some("Transaktional / Utility") => "hoch",
        Some("Editorial / Artikel") => "mittel",
        Some("Strukturierter Wissensinhalt") => "mittel",
        Some("Navigations- / Hub-Seite") => "mittel",
        Some("Medienorientierte Seite") => "mittel",
        Some("Thin / Minimal Content") => "niedrig",
        _ => "mittel",
    }
}

// ─── Helper: Batch Report Assessment & Key Points ──────────────────────────

/// Clear batch assessment — no score, just interpretation
fn build_batch_assessment(
    summary: &crate::output::report_model::PortfolioSummary,
    dist: &SeverityDistribution,
) -> String {
    if dist.critical > 0 && summary.average_overall_score < 50 {
        "Kritische Barrieren — nicht WCAG-konform".to_string()
    } else if dist.critical > 0 {
        "Technisch solide, aber rechtlich riskant".to_string()
    } else if dist.high > 0 {
        "Gute Basis, aber nicht barrierefrei".to_string()
    } else if summary.average_overall_score >= 85 {
        "Weitgehend barrierefrei — Feinschliff".to_string()
    } else {
        "Solide Grundlage mit Optimierungspotenzial".to_string()
    }
}

/// 3 key takeaways for batch report
fn build_batch_key_points(pres: &BatchPresentation, dist: &SeverityDistribution) -> Vec<String> {
    let mut points = Vec::with_capacity(3);

    // Point 1: Critical/high count across all URLs
    let ch = dist.critical + dist.high;
    if ch > 0 {
        points.push(format!(
            "{} kritische/hohe Verstöße über {} URLs hinweg",
            ch, pres.portfolio_summary.total_urls
        ));
    }

    // Point 2: Dominant/recurring issue
    if let Some(top) = pres.top_issues.first() {
        points.push(format!(
            "Hauptproblem: {} ({} Vorkommen auf {} URLs)",
            top.title,
            top.occurrence_count,
            top.affected_urls.len()
        ));
    }

    // Point 3: Legal status
    if dist.critical > 0 {
        points.push(
            "WCAG-Level-A-Verstöße automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig".to_string(),
        );
    } else if dist.high > 0 {
        points.push(
            "Keine Level-A-Verstöße, aber strukturelle Schwächen auf mehreren Seiten".to_string(),
        );
    } else {
        points.push(
            "Keine automatisiert erkennbaren kritischen Barrieren — manuelle Prüfung empfohlen."
                .to_string(),
        );
    }

    points
}

/// Concrete quick actions for batch report
fn build_batch_quick_actions(pres: &BatchPresentation) -> Vec<(String, &'static str)> {
    let mut actions: Vec<(String, &str)> = Vec::new();

    for item in &pres.action_plan.quick_wins {
        if actions.len() >= 3 {
            break;
        }
        let timeframe = match item.effort {
            Effort::Quick => "1–2 Tage",
            Effort::Medium => "3–5 Tage",
            Effort::Structural => "1–2 Wochen",
        };
        let scope = if item.action.contains("alle") || item.action.contains("global") {
            " (global)"
        } else {
            ""
        };
        actions.push((format!("{}{}", item.action, scope), timeframe));
    }

    // Fallback from top issues
    if actions.is_empty() {
        for group in pres.top_issues.iter().take(3) {
            let timeframe = match group.effort {
                Effort::Quick => "1–2 Tage",
                Effort::Medium => "3–5 Tage",
                Effort::Structural => "1–2 Wochen",
            };
            actions.push((first_sentence(&group.recommendation).to_string(), timeframe));
        }
    }

    actions
}

/// Enhanced action plan with effort + scope columns
fn render_batch_action_plan_enhanced(
    mut builder: renderreport::engine::ReportBuilder,
    plan: &ActionPlan,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let render_section = |mut b: renderreport::engine::ReportBuilder,
                          title: &str,
                          items: &[crate::output::report_model::ActionItem],
                          i18n: &I18n|
     -> renderreport::engine::ReportBuilder {
        if items.is_empty() {
            return b;
        }
        b = b.add_component(Section::new(title).with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Aufwand"),
            TableColumn::new("Scope"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ]);
        for item in items {
            let scope = if item.action.contains("alle")
                || item.action.contains("global")
                || item.action.contains("Designsystem")
                || item.action.contains("seitenübergreifend")
            {
                "global"
            } else if item.action.contains("Content")
                || item.action.contains("Text")
                || item.action.contains("Bild")
            {
                "Content"
            } else {
                "Komponente"
            };
            table = table.add_row(vec![
                item.action.clone(),
                item.effort.label().to_string(),
                scope.to_string(),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
        }
        b.add_component(table)
    };

    builder = render_section(builder, "Quick Wins", &plan.quick_wins, i18n);
    builder = render_section(builder, "Mittelfristige Maßnahmen", &plan.medium_term, i18n);
    builder = render_section(builder, "Strukturelle Maßnahmen", &plan.structural, i18n);
    builder
}

/// Closing section for batch report
fn render_next_steps_batch(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            "Empfohlene nächste Schritte",
            "Konkrete Handlungsempfehlung für die Umsetzung.",
        )
        .with_level(1),
    );

    let mut steps: Vec<(String, &str, &str)> = Vec::new();

    // From quick wins
    for item in &pres.action_plan.quick_wins {
        if steps.len() >= 3 {
            break;
        }
        let timeframe = match item.effort {
            Effort::Quick => "Woche 1",
            Effort::Medium => "Woche 2–3",
            Effort::Structural => "Monat 1–2",
        };
        let scope = if item.action.contains("alle")
            || item.action.contains("Designsystem")
            || item.action.contains("global")
        {
            "global"
        } else {
            "komponentenbasiert"
        };
        steps.push((item.action.clone(), timeframe, scope));
    }

    // Fallback from medium_term
    if steps.len() < 3 {
        for item in &pres.action_plan.medium_term {
            if steps.len() >= 3 {
                break;
            }
            steps.push((item.action.clone(), "Monat 1", "komponentenbasiert"));
        }
    }

    if !steps.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Priorität"),
            TableColumn::new("Maßnahme"),
            TableColumn::new("Zeitrahmen"),
            TableColumn::new("Scope"),
        ]);
        for (i, (action, timeframe, scope)) in steps.iter().enumerate() {
            table = table.add_row(vec![
                format!("{}", i + 1),
                action.clone(),
                timeframe.to_string(),
                scope.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder = builder.add_component(
        Callout::info(
            "Für eine vollständige WCAG-Konformitätsprüfung empfehlen wir ergänzend einen manuellen Audit mit assistiven Technologien (Screenreader, Tastaturnavigation). Dieser automatisierte Audit deckt ca. 30–40% der WCAG-Kriterien ab.",
        )
        .with_title("Weiteres Vorgehen"),
    );

    builder
}

// ─── Batch Report ───────────────────────────────────────────────────────────

pub fn generate_batch_pdf(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let i18n = I18n::new(&config.locale)?;
    let pres = build_batch_presentation(batch);
    let wordmark_asset = "/auditmysite-wordmark.svg";

    let domain = &pres.portfolio_summary.domain;
    let overall_score = pres.portfolio_summary.average_overall_score;

    let mut builder = engine
        .report("wcag-batch-audit")
        .title(&pres.cover.title)
        .subtitle(&pres.cover.url)
        .metadata("date", &pres.cover.date)
        .metadata("version", &pres.cover.version)
        .metadata("author", domain)
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

    // ── Cover Page with Audit-Rahmen ────────────────────────────────
    builder = builder
        .add_component(
            Label::new("Automatisierter Batch-Audit-Report")
                .with_size("11pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(
            Label::new("Barrierefreiheits-Prüfbericht")
                .with_size("28pt")
                .bold(),
        )
        .add_component(
            Label::new(
                "Domainweiter Website-Check mit Fokus auf Accessibility, SEO und Performance",
            )
            .with_size("12pt")
            .with_color("#475569"),
        );

    // Audit-Rahmen box (matching single report style)
    {
        let modules_str = pres.portfolio_summary.active_modules.join(", ");

        let mut cover_meta = KeyValueList::new().with_title("Audit-Rahmen");
        cover_meta = cover_meta
            .add("Domain", domain)
            .add("Prüfdatum", &pres.cover.date)
            .add(
                "Geprüfte URLs",
                format!("{}", pres.portfolio_summary.total_urls),
            )
            .add("Zertifikat", &pres.portfolio_summary.certificate)
            .add("Aktive Module", &modules_str)
            .add(
                "Tool-Version",
                format!("auditmysite v{}", pres.cover.version),
            );
        builder = builder.add_component(cover_meta);
    }

    let batch_badge_asset = "/certificate-badge-batch.svg";
    let batch_badge_enabled =
        if let Ok(path) = certificate_badge_path(batch_certificate_label(overall_score)) {
            builder = builder.asset(batch_badge_asset, path);
            true
        } else {
            false
        };

    builder = builder
        .add_component(build_batch_cover_score_row(
            overall_score,
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

    let dist = &pres.portfolio_summary.severity_distribution;

    // ── 1. Status der Website ───────────────────────────────────────
    {
        // Block 1: Bewertung — klare Einordnung, Risiko primär
        let assessment = build_batch_assessment(&pres.portfolio_summary, dist);
        let risk_action = match pres.portfolio_summary.risk_level.as_str() {
            "Kritisch" => "Sofort handeln",
            "Hoch" => "Zeitnah beheben",
            "Mittel" => "Gezielt verbessern",
            _ => "Niveau halten",
        };
        let risk_title = format!("{}  —  {}", assessment, risk_action);
        let callout = match pres.portfolio_summary.risk_level.as_str() {
            "Kritisch" | "Hoch" => {
                Callout::warning(&pres.portfolio_summary.risk_summary).with_title(&risk_title)
            }
            "Mittel" => Callout::info(&pres.portfolio_summary.risk_summary).with_title(&risk_title),
            _ => Callout::success(&pres.portfolio_summary.risk_summary).with_title(&risk_title),
        };
        builder = builder
            .add_component(Section::new("Status der Website").with_level(1))
            .add_component(callout);
    }

    // Score overview cards (sekundär)
    builder = builder.add_component(build_batch_overview_grid(
        pres.portfolio_summary.total_urls as u32,
        overall_score,
        pres.portfolio_summary.total_violations as u32,
        (dist.critical + dist.high) as u32,
        pres.portfolio_summary.crawl_links.as_ref().map(|links| {
            (links.broken_internal_links.len() + links.broken_external_links.len()) as u32
        }),
    ));

    // Block 2: Kernaussagen (max 3 Punkte)
    {
        let key_points = build_batch_key_points(&pres, dist);
        let mut kp_list = List::new().with_title("Kernaussagen");
        for point in &key_points {
            kp_list = kp_list.add_item(point);
        }
        builder = builder.add_component(kp_list);
    }

    // Auswirkungen
    {
        let a11y_avg = pres.portfolio_summary.average_score.round() as u32;
        let user_impact = if a11y_avg < 50 {
            "Kritisch — zentrale Funktionen sind für Nutzer mit Einschränkungen nicht erreichbar"
        } else if a11y_avg < 70 {
            "Eingeschränkt — strukturelle Probleme behindern Nutzer mit Hilfstechnologien"
        } else if a11y_avg < 85 {
            "Gut — einzelne Barrieren für Hilfstechnologien auf mehreren Seiten"
        } else {
            "Sehr gut — Hilfstechnologien werden weitgehend unterstützt"
        };
        let business_impact = if dist.critical > 0 {
            "Weite Teile der Website sind für bestimmte Nutzergruppen nicht oder kaum nutzbar."
        } else if dist.high > 0 {
            "Einzelne Funktionsbereiche sind für Nutzer mit Einschränkungen problematisch."
        } else {
            "Geringe Auswirkung — Nutzer können die Website grundsätzlich verwenden."
        };
        let legal_impact = if dist.critical > 0 {
            "WCAG-Level-A-Verstöße automatisiert erkannt — für belastbare BFSG-Einordnung ist manuelle Prüfung nötig."
        } else {
            "Automatisiert keine kritischen Verstöße erkannt — manuelle Prüfung für vollständige Einordnung empfohlen."
        };

        let mut impact_kv = KeyValueList::new().with_title("Auswirkungen");
        impact_kv = impact_kv
            .add("Nutzer", user_impact)
            .add("Business", business_impact)
            .add("Risiko", legal_impact);
        builder = builder.add_component(impact_kv);
    }

    // Block 3: Handlungsempfehlung
    {
        let actions = build_batch_quick_actions(&pres);
        if !actions.is_empty() {
            let rows: Vec<ChecklistRow> = actions
                .iter()
                .map(|(action, timeframe)| {
                    ChecklistRow::new(*timeframe, action).with_status("warn")
                })
                .collect();
            builder = builder
                .add_component(ChecklistPanel::new(rows).with_title("Empfohlene Sofortmaßnahmen"));
        }
    }

    // Module overview
    if !pres.portfolio_summary.module_averages.is_empty() {
        let mut module_kv = KeyValueList::new().with_title("Modulübersicht (Ø über alle URLs)");
        for (name, score) in &pres.portfolio_summary.module_averages {
            module_kv = module_kv.add(name, format!("{}/100", score));
        }
        builder = builder.add_component(module_kv);
    }

    // ── 2. URL-Ranking ──────────────────────────────────────────────
    let rows: Vec<BenchmarkRow> = pres
        .url_ranking
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let mut row = BenchmarkRow::new(
                (i + 1) as u32,
                &truncate_url(&u.url, 35),
                u.overall_score,
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
        .add_component(BenchmarkTable::new(rows));

    // ── 3. Top-Probleme (vereinheitlicht) ─────────────────────────
    builder = builder.add_component(
        SectionHeaderSplit::new(
            "Häufigste Probleme",
            "Die folgenden Problemgruppen treten über mehrere URLs hinweg auf. \
             Durch Behebung dieser Probleme wird die größte Verbesserung erzielt, \
             da sie viele Seiten gleichzeitig betreffen.",
        )
        .with_level(1),
    );

    // Übersichtstabelle mit Aufwand
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

    // Unified problem blocks — 1 Problem = 1 kompakter Block
    // (keine doppelten Cards + Details mehr)
    for group in pres.top_issues.iter().take(5) {
        let scope = if group.affected_urls.len() >= pres.portfolio_summary.total_urls {
            "global (alle Seiten)"
        } else {
            "einzelne Seiten"
        };
        let effort_label = group.effort.label();
        let meta_line = format!(
            "{} Vorkommen · {} betroffene URLs · Aufwand: {} · Scope: {}",
            group.occurrence_count,
            group.affected_urls.len(),
            effort_label,
            scope
        );

        let mut kv = KeyValueList::new().with_title(&group.title);
        kv = kv
            .add("Problem", &group.customer_description)
            .add("Impact (Nutzer)", &group.user_impact)
            .add("Impact (Business)", &group.business_impact)
            .add("Ursache", &group.typical_cause)
            .add("Fix", &group.recommendation)
            .add("Meta", meta_line);
        builder = builder.add_component(kv);
    }

    // ── 4. Maßnahmenplan (mit Aufwand + Scope) ─────────────────────
    builder = builder.add_component(
        SectionHeaderSplit::new(
            "Maßnahmenplan",
            "Die folgenden Maßnahmen sind nach Aufwand und Wirkung priorisiert. \
             Maßnahmen, die viele Seiten gleichzeitig verbessern, haben Vorrang.",
        )
        .with_level(1),
    );
    builder = render_batch_action_plan_enhanced(builder, &pres.action_plan, &i18n);

    // ── 5a. Render Blocking (Batch) ─────────────────────────────────
    if !pres.portfolio_summary.render_blocking_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title("Render-Blocking-Übersicht (domainweit)");
        for (label, value) in &pres.portfolio_summary.render_blocking_summary {
            kv = kv.add(label, value);
        }
        builder = builder
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
        .add_component(SectionHeaderSplit::new("Technische URL-Matrix", "Verdichtete Übersicht aller geprüften URLs mit Fokus auf technische Priorisierung. Jede Zeile zeigt Score, Problemintensität und den größten Hebel für die nächste Optimierungsrunde.").with_level(1));

    if let Some(ref crawl_links) = pres.portfolio_summary.crawl_links {
        builder = builder
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
        TableColumn::new("#").with_width("4%"),
        TableColumn::new("Seite").with_width("26%"),
        TableColumn::new("Titel").with_width("28%"),
        TableColumn::new("Links zu").with_width("10%"),
        TableColumn::new("Links von").with_width("10%"),
        TableColumn::new("Wörter").with_width("10%"),
        TableColumn::new("Score").with_width("12%"),
    ])
    .with_title("Seiten-Übersicht");

    for row in &pres.url_matrix {
        let score_str = pres
            .url_details
            .iter()
            .find(|d| d.url == row.url)
            .map(|d| format!("{}/100", d.score.round() as u32))
            .unwrap_or_else(|| "—".to_string());
        matrix = matrix.add_row(vec![
            row.rank.to_string(),
            truncate_url(&row.url, 34),
            row.title
                .as_deref()
                .map(|t| truncate_url(t, 36))
                .unwrap_or_else(|| "—".to_string()),
            row.inbound_links.to_string(),
            row.outbound_links.to_string(),
            format_word_count(row.word_count),
            score_str,
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

    // ── 6. Content & SEO — integriert mit Business-Impact ─────────
    builder = builder
        .add_component(SectionHeaderSplit::new(
            "Content & SEO-Potenzial",
            "Content-Stärken und -Schwächen mit direktem Bezug zu Rankings, Sichtbarkeit und Conversion. \
             Jede Auffälligkeit ist an eine konkrete Handlung geknüpft.",
        ).with_level(1));

    // Schwache Seiten zuerst — mit Business-Impact
    if !pres.portfolio_summary.weakest_content_pages.is_empty() {
        let mut issues_kv = KeyValueList::new().with_title("Content-Probleme mit Handlungsbedarf");
        for (url, page_type, score) in &pres.portfolio_summary.weakest_content_pages {
            let relevance = business_relevance(Some(page_type.as_str()), url);
            let impact = if relevance == "hoch" {
                "Rankingverlust + geringere Conversion wahrscheinlich"
            } else if *score < 30 {
                "Schwache organische Sichtbarkeit"
            } else {
                "Optimierungspotenzial für SEO"
            };
            issues_kv = issues_kv.add(
                format!("{} (Profil: {}/100)", truncate_url(url, 35), score),
                format!(
                    "{} — {} → +300–800 Wörter strukturierter Inhalt empfohlen",
                    page_type, impact
                ),
            );
        }
        builder = builder.add_component(issues_kv);
    }

    // Content-Auffälligkeiten als Business-Relevanz
    if !pres.portfolio_summary.distribution_insights.is_empty() {
        let rows: Vec<ChecklistRow> = pres
            .portfolio_summary
            .distribution_insights
            .iter()
            .map(|insight| {
                let impact = if insight.contains("Thin") || insight.contains("dünn") {
                    format!("{} → schwächere Rankings, geringere Verweildauer", insight)
                } else if insight.contains("Duplikat") || insight.contains("duplicate") {
                    format!(
                        "{} → Keyword-Kannibalisierung, Split der Ranking-Signale",
                        insight
                    )
                } else {
                    insight.clone()
                };
                ChecklistRow::new("Handlungsbedarf", &impact).with_status("warn")
            })
            .collect();
        builder = builder.add_component(
            ChecklistPanel::new(rows).with_title("Content-Auffälligkeiten → Business-Impact"),
        );
    }

    // Near-duplicates mit Business-Kontext
    if !pres.portfolio_summary.near_duplicates.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Seite A"),
            TableColumn::new("Seite B"),
            TableColumn::new("Ähnlichkeit"),
            TableColumn::new("Risiko"),
        ])
        .with_title("Near-Duplicate-Content → Keyword-Kannibalisierung");

        for (url_a, url_b, sim) in &pres.portfolio_summary.near_duplicates {
            let risk = if *sim >= 95 {
                "Hoch — konsolidieren"
            } else {
                "Mittel — differenzieren"
            };
            table = table.add_row(vec![
                truncate_url(url_a, 35),
                truncate_url(url_b, 35),
                format!("{sim}%"),
                risk.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    // Seitentyp-Verteilung (kompakt)
    if !pres.portfolio_summary.page_type_distribution.is_empty() {
        let mut type_table = AuditTable::new(vec![
            TableColumn::new("Seitentyp"),
            TableColumn::new("Seiten"),
            TableColumn::new("Anteil"),
            TableColumn::new("Relevanz"),
        ])
        .with_title("Seitentyp-Verteilung");

        for (label, count, pct) in &pres.portfolio_summary.page_type_distribution {
            let relevance = match label.as_str() {
                "Marketing / Landing Page" | "Transaktional / Utility" => "hoch",
                "Editorial / Artikel" | "Strukturierter Wissensinhalt" => "mittel",
                "Thin / Minimal Content" => "niedrig",
                _ => "mittel",
            };
            type_table = type_table.add_row(vec![
                label.clone(),
                count.to_string(),
                format!("{pct}%"),
                relevance.to_string(),
            ]);
        }
        builder = builder.add_component(type_table);
    }

    // Schema-Typ-Verteilung
    if !pres.portfolio_summary.schema_distribution.is_empty() {
        let total = pres.portfolio_summary.total_urls;
        let without = pres.portfolio_summary.pages_without_schema;
        let summary = if without == 0 {
            format!("Alle {} Seiten haben strukturierte Daten.", total)
        } else {
            format!(
                "{} von {} Seiten ohne strukturierte Daten.",
                without, total
            )
        };
        builder = builder.add_component(
            Callout::info(&summary).with_title("Strukturierte Daten (Schema.org)"),
        );
        let mut schema_table = AuditTable::new(vec![
            TableColumn::new("Schema-Typ").with_width("55%"),
            TableColumn::new("Seiten").with_width("20%"),
            TableColumn::new("Anteil").with_width("25%"),
        ])
        .with_title("Schema-Typ-Verteilung");
        for (schema_type, count) in &pres.portfolio_summary.schema_distribution {
            let pct = (*count * 100).checked_div(total).unwrap_or(0);
            schema_table = schema_table.add_row(vec![
                schema_type.clone(),
                count.to_string(),
                format!("{pct}%"),
            ]);
        }
        builder = builder.add_component(schema_table);
    }

    // Stärkste Seiten (kurz)
    if !pres.portfolio_summary.strongest_content_pages.is_empty() {
        let mut strengths = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new("Seitentyp"),
            TableColumn::new("Profil"),
        ])
        .with_title("Stärkste Content-Seiten");

        for (url, page_type, score) in &pres.portfolio_summary.strongest_content_pages {
            strengths = strengths.add_row(vec![
                truncate_url(url, 42),
                page_type.clone(),
                format!("{score}/100"),
            ]);
        }
        builder = builder.add_component(strengths);
    }

    // ── Empfohlene nächste Schritte ───────────────────────────────
    builder = render_next_steps_batch(builder, &pres);

    // ── 7. Anhang ───────────────────────────────────────────────────
    if config.level == ReportLevel::Technical && !pres.appendix.per_url.is_empty() {
        builder = builder.add_component(
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
        builder = builder.add_component(Section::new("Modul-Vergleich").with_level(1));

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
        builder =
            builder.add_component(Section::new("Wichtigste Findings je Domain").with_level(1));

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
