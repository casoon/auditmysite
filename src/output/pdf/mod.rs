//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

mod batch;
mod cover;
mod design;
mod detail_modules;
mod findings;
mod helpers;
mod history;
mod modules;

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DevicePreview, DiagnosisPanel, DiagnosisRow,
    DominantIssueSpotlight, ImpactGrid, ImpactGridCard, KeyValueList, List, MetricStrip,
    MetricStripItem, PageBreak, PhaseBlock, SectionHeaderSplit, TableOfContents,
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

const WORDMARK_ASSET: &str = "/auditmysite-wordmark.svg";
const CUSTOM_COVER_LOGO_ASSET: &str = "/cover-logo-custom";

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

    let mut builder = engine
        .report("wcag-audit")
        .metadata("date", &vm.meta.date)
        .metadata("version", &vm.meta.version)
        .metadata("author", &vm.meta.author)
        .metadata("score", &vm.meta.score_label)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "");

    let cover_logo_asset = cover_logo_asset(config);
    builder = register_cover_logo_asset(builder, config, cover_logo_asset);

    // ── Cover Page ───────────────────────────────────────────────────
    builder = builder
        .add_component(Image::new(cover_logo_asset).with_width("120pt"))
        .add_component(
            Label::new(&vm.executive.cover_eyebrow)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(Label::new(&vm.cover.title).with_size("22pt").bold())
        .add_component(
            Label::new(format!(
                "{}  ·  {}",
                extract_domain(&vm.cover.domain),
                vm.cover.date
            ))
            .with_size("11pt")
            .with_color("#0f766e"),
        )
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

    // ── Device Preview (desktop + mobile screenshots) — skip for executive
    if vm.meta.report_level != ReportLevel::Executive {
        if let Some(ref shots) = report.page_screenshots {
            let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
            let desktop_path = std::env::temp_dir().join(format!("ams-desktop-{}.png", ts));
            let mobile_path = std::env::temp_dir().join(format!("ams-mobile-{}.png", ts));
            if std::fs::write(&desktop_path, &shots.desktop).is_ok()
                && std::fs::write(&mobile_path, &shots.mobile).is_ok()
            {
                let desktop_key = "/page-screenshot-desktop.png";
                let mobile_key = "/page-screenshot-mobile.png";
                builder = builder
                    .asset(desktop_key, &desktop_path)
                    .asset(mobile_key, &mobile_path)
                    .add_component(DevicePreview::new(desktop_key, mobile_key));
            }
        } else {
            let title = i18n.t("section-device-preview");
            let body = i18n.t("section-device-preview-no-screenshots");
            builder = builder.add_component(Callout::info(&body).with_title(&title));
        }
    }

    builder = builder.add_component(build_cover_score_row(
        &vm.cover,
        single_badge_enabled.then_some(single_badge_asset),
        &i18n,
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
            "Kritisch" | "Critical" => {
                Callout::error(&vm.summary.risk_summary).with_title(&vm.executive.risk_title)
            }
            "Hoch" | "High" => {
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
                        if i18n.locale() == "en" {
                            "Classification, risk and the most important business consequences at a glance."
                        } else {
                            "Einordnung, Risiko und die geschäftlich wichtigsten Konsequenzen auf einen Blick."
                        },
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
                        MetricStripItem::new(i18n.t("metric-score"), vm.summary.score.to_string())
                            .with_accent("#0f766e"),
                        MetricStripItem::new(
                            i18n.t("metric-issues-detected"),
                            vm.severity.total.to_string(),
                        ),
                        MetricStripItem::new(
                            i18n.t("metric-critical-high"),
                            format!("{}", vm.severity.critical + vm.severity.high),
                        )
                        .with_status("bad")
                        .with_accent("#dc2626"),
                        MetricStripItem::new(i18n.t("metric-risk"), &vm.summary.risk_level)
                            .with_status(risk_status(&vm.summary.risk_level)),
                        MetricStripItem::new(i18n.t("metric-certificate"), &vm.summary.certificate)
                            .with_accent("#7c3aed"),
                    ])
                    .compact(),
                ),
            ],
        ));

        if vm.meta.report_level != ReportLevel::Executive {
            builder = builder.add_component(build_module_strip(&vm));
            builder = builder.add_component(build_raw_audit_snapshot(&vm, &i18n));
        }

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
            let en = i18n.locale() == "en";
            let (user_key, business_key, risk_key) = if en {
                ("User", "Business", "Risk")
            } else {
                ("Nutzer", "Business", "Risiko")
            };
            let (usability_label, compliance_label, business_eff_label) = if en {
                ("Usability", "Compliance", "Business effect")
            } else {
                ("Nutzbarkeit", "Compliance", "Geschäftswirkung")
            };
            let user = impact_row(&vm.executive.impact_rows, user_key);
            let business = impact_row(&vm.executive.impact_rows, business_key);
            let risk = impact_row(&vm.executive.impact_rows, risk_key);
            builder = builder.add_component(
                ImpactGrid::new(
                    ImpactGridCard::new(user_key, usability_label, user).with_status("warn"),
                    ImpactGridCard::new(risk_key, compliance_label, risk).with_status("bad"),
                    ImpactGridCard::new(business_key, business_eff_label, business)
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

        if vm.meta.report_level != ReportLevel::Executive && !vm.summary.positive_aspects.is_empty()
        {
            let strength_label = i18n.t("label-strength");
            let rows: Vec<ChecklistRow> = vm
                .summary
                .positive_aspects
                .iter()
                .take(3)
                .enumerate()
                .map(|(i, item)| {
                    ChecklistRow::new(format!("{} {}", strength_label, i + 1), item)
                        .with_status("good")
                })
                .collect();
            builder = builder
                .add_component(ChecklistPanel::new(rows).with_title(i18n.t("panel-strengths")));
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

        // Severity distribution — visual score-weight overview
        if vm.severity.has_issues {
            builder = builder.add_component(SeverityOverview::new(
                vm.severity.critical,
                vm.severity.high,
                vm.severity.medium,
                vm.severity.low,
            ));
        }

        // TopFixesTable
        if let Some(table) = build_top_hebel_table(&vm.findings, total_ch) {
            builder = builder.add_component(table.with_title(i18n.t("section-top-issues")));
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
        let findings_limit = if vm.meta.report_level == ReportLevel::Executive {
            3
        } else {
            5
        };
        for group in vm.findings.top_findings.iter().take(findings_limit) {
            builder = render_key_finding_block(
                builder,
                group,
                &i18n,
                vm.meta.report_level != ReportLevel::Executive,
            );
        }
    }

    if vm.meta.report_level != ReportLevel::Executive {
        if let Some(ref history) = vm.history {
            builder = render_history_section(builder, history, &i18n);
        }
    }

    // Executive level stops here — slim methodology (limitations only)
    if vm.meta.report_level == ReportLevel::Executive {
        builder = builder
            .add_component(
                Callout::info(&vm.methodology.limitations)
                    .with_title(i18n.t("callout-limitations-title")),
            )
            .add_component(
                Callout::warning(&vm.methodology.disclaimer)
                    .with_title(i18n.t("callout-note-title")),
            );
        let built_report = builder.build();
        let pdf_bytes = engine.render_pdf(&built_report)?;
        if let Some(ref _shots) = report.page_screenshots {
            let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
            let _ =
                std::fs::remove_file(std::env::temp_dir().join(format!("ams-desktop-{}.png", ts)));
            let _ =
                std::fs::remove_file(std::env::temp_dir().join(format!("ams-mobile-{}.png", ts)));
        }
        return Ok(pdf_bytes);
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
            .with_eyebrow(if i18n.locale() == "en" {
                "IMPLEMENTATION PLAN"
            } else {
                "UMSETZUNGSPLAN"
            })
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
            builder = builder
                .add_component(ChecklistPanel::new(rows).with_title(i18n.t("panel-quick-actions")));
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
                    let display_name = if matches!(
                        module.name.as_str(),
                        "UX" | "Journey" | "AI Visibility" | "KI-Sichtbarkeit"
                    ) {
                        format!("{} (~)", module.name)
                    } else {
                        module.name.clone()
                    };
                    DiagnosisRow::new(&display_name, format!("{}/100", module.score))
                        .with_status(status)
                })
                .collect();
            builder = builder.add_component(
                DiagnosisPanel::new(diag_rows).with_title(i18n.t("panel-modules-overview")),
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // SECTION 5b — SYSTEM DIAGNOSIS
    // Goal: pattern analysis, clusters, systematic vs. isolated
    // ─────────────────────────────────────────────────────────────────
    if vm.severity.has_issues {
        builder = render_diagnosis_section(builder, &vm.diagnosis, &i18n);
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
        builder = builder
            .add_component(Section::new(i18n.t("section-tech-detail-metrics")).with_level(2));
    }

    if let Some(ref perf) = vm.module_details.performance {
        builder = render_performance(builder, perf, &i18n);
    }
    if !report.budget_violations.is_empty() {
        builder = render_budget_violations(builder, &report.budget_violations, &i18n);
    }
    if let Some(ref seo) = vm.module_details.seo {
        builder = render_seo(builder, seo, &i18n);
    }
    if let Some(ref sec) = vm.module_details.security {
        builder = render_security(builder, sec, &i18n);
    }
    if let Some(ref mobile) = vm.module_details.mobile {
        builder = render_mobile(builder, mobile, &i18n);
    }
    if let Some(ref ux) = vm.module_details.ux {
        builder = render_ux(builder, ux, &i18n);
    }
    if let Some(ref journey) = vm.module_details.journey {
        builder = render_journey(builder, journey, &i18n);
    }
    if let Some(ref dm) = vm.module_details.dark_mode {
        builder = render_dark_mode(builder, dm, &i18n);
    }
    if let Some(ref sq) = vm.module_details.source_quality {
        builder = render_source_quality(builder, sq, &i18n);
    }
    if let Some(ref av) = vm.module_details.ai_visibility {
        builder = render_ai_visibility(builder, av, &i18n);
    }

    // ── Appendix ────────────────────────────────────────────────────
    if vm.appendix.has_violations {
        builder = builder.add_component(Section::new(i18n.t("section-appendix")).with_level(1));

        builder = builder.add_component(build_cli_snapshot_table(&vm, &i18n));

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
                ChecklistPanel::new(rows).with_title(i18n.t("section-all-violations")),
            );
        }
    }

    // ── Empfohlene nächste Schritte ───────────────────────────────
    builder = render_next_steps_single(builder, &vm);

    // ── Methodology ─────────────────────────────────────────────────
    builder = render_methodology_section(builder, &vm.methodology, &i18n);

    let built_report = builder.build();
    let pdf_bytes = engine.render_pdf(&built_report)?;

    // Clean up screenshot temp files
    if let Some(ref _shots) = report.page_screenshots {
        let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
        let _ = std::fs::remove_file(std::env::temp_dir().join(format!("ams-desktop-{}.png", ts)));
        let _ = std::fs::remove_file(std::env::temp_dir().join(format!("ams-mobile-{}.png", ts)));
    }

    Ok(pdf_bytes)
}

fn build_module_strip(vm: &ReportViewModel) -> MetricStrip {
    const HEURISTIC: &[&str] = &["UX", "Journey", "AI Visibility", "KI-Sichtbarkeit"];
    let items = vm
        .modules
        .dashboard
        .iter()
        .take(6)
        .map(|module| {
            let heuristic = HEURISTIC.contains(&module.name.as_str());
            let status = if module.score >= 85 {
                "good"
            } else if module.score >= 70 {
                "info"
            } else if module.score >= 50 {
                "warn"
            } else {
                "bad"
            };
            let display_name = if heuristic {
                format!("{} (~)", module.name)
            } else {
                module.name.clone()
            };
            let display_value = if heuristic {
                format!("~{}/100", module.score)
            } else {
                format!("{}/100", module.score)
            };
            MetricStripItem::new(display_name, display_value)
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
        .unwrap_or("")
}

fn build_cli_snapshot_table(vm: &ReportViewModel, i18n: &I18n) -> AuditTable {
    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("audit-data-area")).with_width("22%"),
        TableColumn::new(i18n.t("audit-data-signal")).with_width("28%"),
        TableColumn::new(i18n.t("audit-data-value")).with_width("50%"),
    ])
    .with_title(i18n.t("audit-data-title"));

    let row_audit = i18n.t("audit-data-row-audit");
    let row_module = i18n.t("audit-data-row-module");
    let row_finding = i18n.t("audit-data-row-finding");

    for (label, value) in &vm.methodology.audit_facts {
        table = table.add_row(vec![row_audit.clone(), label.clone(), value.clone()]);
    }

    for module in &vm.modules.dashboard {
        table = table.add_row(vec![
            row_module.clone(),
            module.name.clone(),
            format!(
                "{} / 100 — {}. {}",
                module.score, module.interpretation, module.card_context
            ),
        ]);
    }

    let occurrences_word = if i18n.locale() == "en" {
        "occurrences"
    } else {
        "Vorkommen"
    };
    for finding in vm.findings.top_findings.iter().take(6) {
        table = table.add_row(vec![
            row_finding.clone(),
            format!("{} ({})", finding.rule_id, finding.wcag_criterion),
            format!(
                "{} {} — {}",
                finding.occurrence_count,
                occurrences_word,
                first_sentence(&finding.user_impact)
            ),
        ]);
    }

    table
}

fn build_raw_audit_snapshot(vm: &ReportViewModel, i18n: &I18n) -> SummaryBox {
    let lookup = |de: &str, en_label: &str| -> String {
        vm.methodology
            .audit_facts
            .iter()
            .find(|(label, _)| label == de || label == en_label)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    };
    SummaryBox::new(i18n.t("scope-box-title"))
        .add_item(i18n.t("scope-box-wcag-level"), {
            let v = lookup("WCAG-Level", "WCAG level");
            if v.is_empty() {
                "n/a".to_string()
            } else {
                v
            }
        })
        .add_item(i18n.t("scope-box-checked-nodes"), {
            let v = lookup("Geprüfte Knoten", "Checked nodes");
            if v.is_empty() {
                "n/a".to_string()
            } else {
                v
            }
        })
        .add_item(i18n.t("scope-box-runtime"), {
            let v = lookup("Laufzeit", "Runtime");
            if v.is_empty() {
                "n/a".to_string()
            } else {
                v
            }
        })
        .add_item(
            i18n.t("scope-box-findings-total"),
            vm.severity.total.to_string(),
        )
        .add_item(
            i18n.t("scope-box-critical-high"),
            format!("{}", vm.severity.critical + vm.severity.high),
        )
        .add_item(i18n.t("scope-box-audit-notes"), {
            let v = lookup("Audit-Hinweise", "Audit notes");
            if v.is_empty() {
                "0".to_string()
            } else {
                v
            }
        })
}

fn cover_logo_asset(config: &ReportConfig) -> &'static str {
    match config.logo_path.as_ref() {
        Some(path) if path.exists() => CUSTOM_COVER_LOGO_ASSET,
        _ => WORDMARK_ASSET,
    }
}

fn register_cover_logo_asset(
    mut builder: renderreport::engine::ReportBuilder,
    config: &ReportConfig,
    cover_logo_asset: &'static str,
) -> renderreport::engine::ReportBuilder {
    if cover_logo_asset == CUSTOM_COVER_LOGO_ASSET {
        if let Some(ref logo_path) = config.logo_path {
            return builder.asset(CUSTOM_COVER_LOGO_ASSET, logo_path);
        }
    }

    if let Ok(path) = auditmysite_wordmark_path() {
        builder = builder.asset(WORDMARK_ASSET, path);
    }
    builder
}

use self::design::{module_score_color, risk_status};

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

// ─── Diagnosis Section ───────────────────────────────────────────────────────

fn render_diagnosis_section(
    mut builder: renderreport::engine::ReportBuilder,
    diagnosis: &DiagnosisBlock,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    builder = builder.add_component(Section::new(&diagnosis.section_title).with_level(2));

    // Pattern overview — label + description
    let pattern_intro = format!(
        "{}: {}",
        diagnosis.pattern_label, diagnosis.pattern_description
    );
    let callout = if diagnosis.is_systematic {
        Callout::warning(&pattern_intro).with_title(&diagnosis.pattern_label)
    } else {
        Callout::info(&pattern_intro).with_title(&diagnosis.pattern_label)
    };
    builder = builder.add_component(callout);

    // Dominant issue spotlight
    if let Some(ref dominant) = diagnosis.dominant_issue {
        let spotlight_text = if en {
            format!(
                "\"{}\" accounts for the majority of critical/high findings.",
                dominant
            )
        } else {
            format!(
                "\"{}\" verursacht den Großteil der kritischen/hohen Findings.",
                dominant
            )
        };
        builder = builder.add_component(Callout::warning(&spotlight_text));
    }

    // Category breakdown table
    if !diagnosis.category_breakdown.is_empty() {
        let col_dim = i18n.t("diagnosis-col-category");
        let col_count = i18n.t("diagnosis-col-findings");
        let col_sev = i18n.t("diagnosis-col-worst-severity");
        let table_title = i18n.t("diagnosis-table-categories");
        let mut table = AuditTable::new(vec![
            TableColumn::new(col_dim),
            TableColumn::new(col_count).with_width("15%"),
            TableColumn::new(col_sev).with_width("25%"),
        ])
        .with_title(table_title);
        for (dim, count, sev_label) in &diagnosis.category_breakdown {
            table = table.add_row(vec![dim.clone(), count.to_string(), sev_label.clone()]);
        }
        builder = builder.add_component(table);
    }

    // Thematic clusters
    if !diagnosis.clusters.is_empty() {
        let clusters_title = i18n.t("diagnosis-table-clusters");
        let col_cluster = "Cluster";
        let col_findings = i18n.t("diagnosis-col-findings");
        let col_occ = i18n.t("diagnosis-col-occurrences");
        let col_sev = i18n.t("diagnosis-col-max-severity");
        let mut table = AuditTable::new(vec![
            TableColumn::new(col_cluster),
            TableColumn::new(col_findings).with_width("12%"),
            TableColumn::new(col_occ).with_width("14%"),
            TableColumn::new(col_sev).with_width("18%"),
        ])
        .with_title(clusters_title);
        for cluster in &diagnosis.clusters {
            table = table.add_row(vec![
                cluster.label.clone(),
                cluster.finding_count.to_string(),
                cluster.occurrence_total.to_string(),
                cluster.severity_label.clone(),
            ]);
        }
        builder = builder.add_component(table);
    }

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

fn business_relevance(page_type: Option<&str>, url: &str, locale: &str) -> &'static str {
    let en = locale == "en";
    let high = if en { "high" } else { "hoch" };
    let medium = if en { "medium" } else { "mittel" };
    let low = if en { "low" } else { "niedrig" };

    // URL-based heuristics first
    let path = url.to_lowercase();
    if path.contains("impressum")
        || path.contains("datenschutz")
        || path.contains("agb")
        || path.contains("imprint")
        || path.contains("privacy")
        || path.contains("terms")
    {
        return low;
    }
    if path.ends_with('/') && path.matches('/').count() <= 3 {
        return high; // homepage or top-level pages
    }

    // Page type based
    match page_type {
        Some("Marketing / Landing Page") => high,
        Some("Transaktional / Utility") | Some("Transactional / Utility") => high,
        Some("Editorial / Artikel") | Some("Editorial / Article") => medium,
        Some("Strukturierter Wissensinhalt") | Some("Structured knowledge content") => medium,
        Some("Navigations- / Hub-Seite") | Some("Navigation / Hub page") => medium,
        Some("Medienorientierte Seite") | Some("Media-oriented page") => medium,
        Some("Thin / Minimal Content") => low,
        _ => medium,
    }
}

// ─── Helper: Batch Report Assessment & Key Points ──────────────────────────

/// Clear batch assessment — no score, just interpretation
fn build_batch_assessment(
    summary: &crate::output::report_model::PortfolioSummary,
    dist: &SeverityDistribution,
    i18n: &I18n,
) -> String {
    let en = i18n.locale() == "en";
    if dist.critical > 0 && summary.average_overall_score < 50 {
        if en {
            "Critical barriers — not WCAG conformant".to_string()
        } else {
            "Kritische Barrieren — nicht WCAG-konform".to_string()
        }
    } else if dist.critical > 0 {
        if en {
            "Technically solid, but legally risky".to_string()
        } else {
            "Technisch solide, aber rechtlich riskant".to_string()
        }
    } else if dist.high > 0 {
        if en {
            "Good foundation, but not accessible".to_string()
        } else {
            "Gute Basis, aber nicht barrierefrei".to_string()
        }
    } else if summary.average_overall_score >= 85 {
        if en {
            "Largely accessible — polish".to_string()
        } else {
            "Weitgehend barrierefrei — Feinschliff".to_string()
        }
    } else if en {
        "Solid foundation with room to optimize".to_string()
    } else {
        "Solide Grundlage mit Optimierungspotenzial".to_string()
    }
}

/// 3 key takeaways for batch report
fn build_batch_key_points(
    pres: &BatchPresentation,
    dist: &SeverityDistribution,
    i18n: &I18n,
) -> Vec<String> {
    let en = i18n.locale() == "en";
    let mut points = Vec::with_capacity(3);

    // Point 1: Critical/high count across all URLs
    let ch = dist.critical + dist.high;
    if ch > 0 {
        if en {
            points.push(format!(
                "{} critical/high violations across {} URLs",
                ch, pres.portfolio_summary.total_urls
            ));
        } else {
            points.push(format!(
                "{} kritische/hohe Verstöße über {} URLs hinweg",
                ch, pres.portfolio_summary.total_urls
            ));
        }
    }

    // Point 2: Dominant/recurring issue
    if let Some(top) = pres.top_issues.first() {
        if en {
            points.push(format!(
                "Main issue: {} ({} occurrences on {} URLs)",
                top.title,
                top.occurrence_count,
                top.affected_urls.len()
            ));
        } else {
            points.push(format!(
                "Hauptproblem: {} ({} Vorkommen auf {} URLs)",
                top.title,
                top.occurrence_count,
                top.affected_urls.len()
            ));
        }
    }

    // Point 3: Legal status
    if dist.critical > 0 {
        if en {
            points.push(
                "WCAG Level A violations detected automatically — manual review needed for a defensible BFSG classification".to_string(),
            );
        } else {
            points.push(
                "WCAG-Level-A-Verstöße automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig".to_string(),
            );
        }
    } else if dist.high > 0 {
        if en {
            points.push(
                "No Level A violations, but structural weaknesses on multiple pages".to_string(),
            );
        } else {
            points.push(
                "Keine Level-A-Verstöße, aber strukturelle Schwächen auf mehreren Seiten"
                    .to_string(),
            );
        }
    } else if en {
        points.push(
            "No automatically detectable critical barriers — manual review recommended."
                .to_string(),
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
fn build_batch_quick_actions(pres: &BatchPresentation, i18n: &I18n) -> Vec<(String, String)> {
    let en = i18n.locale() == "en";
    let timeframe_label = |effort: Effort| -> &'static str {
        match (effort, en) {
            (Effort::Quick, true) => "1–2 days",
            (Effort::Quick, false) => "1–2 Tage",
            (Effort::Medium, true) => "3–5 days",
            (Effort::Medium, false) => "3–5 Tage",
            (Effort::Structural, true) => "1–2 weeks",
            (Effort::Structural, false) => "1–2 Wochen",
        }
    };

    let mut actions: Vec<(String, String)> = Vec::new();

    for item in &pres.action_plan.quick_wins {
        if actions.len() >= 3 {
            break;
        }
        let action_lower = item.action.to_lowercase();
        let scope = if action_lower.contains("alle") || action_lower.contains("global") {
            " (global)"
        } else {
            ""
        };
        actions.push((
            format!("{}{}", item.action, scope),
            timeframe_label(item.effort).to_string(),
        ));
    }

    // Fallback from top issues
    if actions.is_empty() {
        for group in pres.top_issues.iter().take(3) {
            actions.push((
                first_sentence(&group.recommendation).to_string(),
                timeframe_label(group.effort).to_string(),
            ));
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
    let en = i18n.locale() == "en";
    let effort_col = if en { "Effort" } else { "Aufwand" };
    let role_col = if en { "Role" } else { "Rolle" };
    let scope_global = "global";
    let scope_content = "Content";
    let scope_component = if en { "Component" } else { "Komponente" };

    let render_section = |mut b: renderreport::engine::ReportBuilder,
                          title: String,
                          items: &[crate::output::report_model::ActionItem],
                          i18n: &I18n|
     -> renderreport::engine::ReportBuilder {
        if items.is_empty() {
            return b;
        }
        b = b.add_component(Section::new(title).with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("column-action")),
            TableColumn::new(effort_col),
            TableColumn::new("Scope"),
            TableColumn::new(role_col),
            TableColumn::new(i18n.t("column-priority")),
        ]);
        for item in items {
            let action_lower = item.action.to_lowercase();
            let scope = if action_lower.contains("alle")
                || action_lower.contains("global")
                || action_lower.contains("designsystem")
                || action_lower.contains("design system")
                || action_lower.contains("seitenübergreifend")
                || action_lower.contains("site-wide")
            {
                scope_global
            } else if action_lower.contains("content")
                || action_lower.contains("text")
                || action_lower.contains("bild")
                || action_lower.contains("image")
            {
                scope_content
            } else {
                scope_component
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

    builder = render_section(
        builder,
        i18n.t("section-quick-wins"),
        &plan.quick_wins,
        i18n,
    );
    builder = render_section(
        builder,
        i18n.t("section-medium-actions"),
        &plan.medium_term,
        i18n,
    );
    builder = render_section(
        builder,
        i18n.t("section-structural-actions"),
        &plan.structural,
        i18n,
    );
    builder
}

/// Closing section for batch report
fn render_next_steps_batch(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let intro = if en {
        "Concrete recommendation for implementation."
    } else {
        "Konkrete Handlungsempfehlung für die Umsetzung."
    };
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("section-next-steps-recommended"), intro).with_level(1),
    );

    let mut steps: Vec<(String, &str, &str)> = Vec::new();

    let timeframe_quick = if en { "Week 1" } else { "Woche 1" };
    let timeframe_medium = if en { "Week 2–3" } else { "Woche 2–3" };
    let timeframe_structural = if en { "Month 1–2" } else { "Monat 1–2" };
    let timeframe_medium_alt = if en { "Month 1" } else { "Monat 1" };
    let scope_global = "global";
    let scope_component = if en {
        "component-based"
    } else {
        "komponentenbasiert"
    };

    // From quick wins
    for item in &pres.action_plan.quick_wins {
        if steps.len() >= 3 {
            break;
        }
        let timeframe = match item.effort {
            Effort::Quick => timeframe_quick,
            Effort::Medium => timeframe_medium,
            Effort::Structural => timeframe_structural,
        };
        let action_lower = item.action.to_lowercase();
        let scope = if action_lower.contains("alle")
            || action_lower.contains("designsystem")
            || action_lower.contains("design system")
            || action_lower.contains("global")
        {
            scope_global
        } else {
            scope_component
        };
        steps.push((item.action.clone(), timeframe, scope));
    }

    // Fallback from medium_term
    if steps.len() < 3 {
        for item in &pres.action_plan.medium_term {
            if steps.len() >= 3 {
                break;
            }
            steps.push((item.action.clone(), timeframe_medium_alt, scope_component));
        }
    }

    if !steps.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("column-priority")),
            TableColumn::new(i18n.t("column-action")),
            TableColumn::new(i18n.t("column-timeframe")),
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

    let callout_body = if en {
        "For a complete WCAG conformance check we additionally recommend a manual audit with assistive technologies (screen reader, keyboard navigation). This automated audit covers about 30–40% of WCAG criteria."
    } else {
        "Für eine vollständige WCAG-Konformitätsprüfung empfehlen wir ergänzend einen manuellen Audit mit assistiven Technologien (Screenreader, Tastaturnavigation). Dieser automatisierte Audit deckt ca. 30–40% der WCAG-Kriterien ab."
    };
    builder = builder
        .add_component(Callout::info(callout_body).with_title(i18n.t("section-next-steps-block")));

    builder
}

// ─── Batch Report ───────────────────────────────────────────────────────────

pub fn generate_batch_pdf(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let i18n = I18n::new(&config.locale)?;
    let pres = build_batch_presentation(batch);

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

    let cover_logo_asset = cover_logo_asset(config);
    builder = register_cover_logo_asset(builder, config, cover_logo_asset);

    // ── Cover Page with Audit-Rahmen ────────────────────────────────
    builder = builder
        .add_component(Image::new(cover_logo_asset).with_width("120pt"))
        .add_component(
            Label::new(i18n.t("batch-cover-eyebrow"))
                .with_size("11pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(
            Label::new(i18n.t("batch-cover-title"))
                .with_size("28pt")
                .bold(),
        )
        .add_component(
            Label::new(i18n.t("batch-cover-kicker"))
                .with_size("12pt")
                .with_color("#475569"),
        );

    // Audit-Rahmen box (matching single report style)
    {
        let modules_str = pres.portfolio_summary.active_modules.join(", ");

        let mut cover_meta = KeyValueList::new().with_title(i18n.t("batch-cover-frame-title"));
        cover_meta = cover_meta
            .add(i18n.t("batch-cover-frame-domain"), domain)
            .add(i18n.t("batch-cover-frame-date"), &pres.cover.date)
            .add(
                i18n.t("batch-cover-frame-urls"),
                format!("{}", pres.portfolio_summary.total_urls),
            )
            .add(
                i18n.t("batch-cover-frame-certificate"),
                &pres.portfolio_summary.certificate,
            )
            .add(i18n.t("batch-cover-frame-modules"), &modules_str)
            .add(
                i18n.t("batch-cover-frame-version"),
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
            &i18n,
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
        let assessment = build_batch_assessment(&pres.portfolio_summary, dist, &i18n);
        let en_status = i18n.locale() == "en";
        let risk_action = match (pres.portfolio_summary.risk_level.as_str(), en_status) {
            ("Kritisch" | "Critical", true) => "Act immediately",
            ("Kritisch" | "Critical", false) => "Sofort handeln",
            ("Hoch" | "High", true) => "Fix soon",
            ("Hoch" | "High", false) => "Zeitnah beheben",
            ("Mittel" | "Medium", true) => "Improve deliberately",
            ("Mittel" | "Medium", false) => "Gezielt verbessern",
            (_, true) => "Maintain level",
            (_, false) => "Niveau halten",
        };
        let risk_title = format!("{}  —  {}", assessment, risk_action);
        let callout = match pres.portfolio_summary.risk_level.as_str() {
            "Kritisch" | "Hoch" | "Critical" | "High" => {
                Callout::warning(&pres.portfolio_summary.risk_summary).with_title(&risk_title)
            }
            "Mittel" | "Medium" => {
                Callout::info(&pres.portfolio_summary.risk_summary).with_title(&risk_title)
            }
            _ => Callout::success(&pres.portfolio_summary.risk_summary).with_title(&risk_title),
        };
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-status")).with_level(1))
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
        let key_points = build_batch_key_points(&pres, dist, &i18n);
        let mut kp_list = List::new().with_title(i18n.t("narrative-key-points-title"));
        for point in &key_points {
            kp_list = kp_list.add_item(point);
        }
        builder = builder.add_component(kp_list);
    }

    // Auswirkungen
    {
        let en = i18n.locale() == "en";
        let a11y_avg = pres.portfolio_summary.average_score.round() as u32;
        let user_impact = match (a11y_avg, en) {
            (s, true) if s < 50 => "Critical — core functions are unreachable for users with disabilities",
            (s, false) if s < 50 => "Kritisch — zentrale Funktionen sind für Nutzer mit Einschränkungen nicht erreichbar",
            (s, true) if s < 70 => "Limited — structural issues impede users with assistive technologies",
            (s, false) if s < 70 => "Eingeschränkt — strukturelle Probleme behindern Nutzer mit Hilfstechnologien",
            (s, true) if s < 85 => "Good — individual barriers for assistive technologies on several pages",
            (s, false) if s < 85 => "Gut — einzelne Barrieren für Hilfstechnologien auf mehreren Seiten",
            (_, true) => "Very good — assistive technologies are largely supported",
            (_, false) => "Sehr gut — Hilfstechnologien werden weitgehend unterstützt",
        };
        let business_impact = if dist.critical > 0 {
            if en {
                "Large parts of the website are unusable or barely usable for certain user groups."
            } else {
                "Weite Teile der Website sind für bestimmte Nutzergruppen nicht oder kaum nutzbar."
            }
        } else if dist.high > 0 {
            if en {
                "Individual functional areas are problematic for users with disabilities."
            } else {
                "Einzelne Funktionsbereiche sind für Nutzer mit Einschränkungen problematisch."
            }
        } else if en {
            "Low impact — users can fundamentally use the website."
        } else {
            "Geringe Auswirkung — Nutzer können die Website grundsätzlich verwenden."
        };
        let legal_impact = if dist.critical > 0 {
            if en {
                "WCAG Level A violations detected automatically — manual review required for a defensible BFSG classification."
            } else {
                "WCAG-Level-A-Verstöße automatisiert erkannt — für belastbare BFSG-Einordnung ist manuelle Prüfung nötig."
            }
        } else if en {
            "No critical violations detected automatically — manual review recommended for full classification."
        } else {
            "Automatisiert keine kritischen Verstöße erkannt — manuelle Prüfung für vollständige Einordnung empfohlen."
        };

        let (user_label, business_label, risk_label) = if en {
            ("User", "Business", "Risk")
        } else {
            ("Nutzer", "Business", "Risiko")
        };
        let mut impact_kv = KeyValueList::new().with_title(i18n.t("narrative-impact-title"));
        impact_kv = impact_kv
            .add(user_label, user_impact)
            .add(business_label, business_impact)
            .add(risk_label, legal_impact);
        builder = builder.add_component(impact_kv);
    }

    // Block 3: Handlungsempfehlung
    {
        let actions = build_batch_quick_actions(&pres, &i18n);
        if !actions.is_empty() {
            let rows: Vec<ChecklistRow> = actions
                .iter()
                .map(|(action, timeframe)| {
                    ChecklistRow::new(timeframe.clone(), action).with_status("warn")
                })
                .collect();
            builder = builder.add_component(
                ChecklistPanel::new(rows).with_title(i18n.t("narrative-quick-actions-title")),
            );
        }
    }

    // Module overview
    if !pres.portfolio_summary.module_averages.is_empty() {
        let mut module_kv = KeyValueList::new().with_title(i18n.t("batch-panel-module-averages"));
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
    let en_top = i18n.locale() == "en";
    let top_intro = if en_top {
        "The following problem groups occur across multiple URLs. \
         Fixing them yields the largest improvement because they affect many pages simultaneously."
    } else {
        "Die folgenden Problemgruppen treten über mehrere URLs hinweg auf. \
         Durch Behebung dieser Probleme wird die größte Verbesserung erzielt, \
         da sie viele Seiten gleichzeitig betreffen."
    };
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-section-most-frequent"), top_intro).with_level(1),
    );

    // Übersichtstabelle mit Aufwand
    if !pres.issue_frequency.is_empty() {
        let affected_col = if en_top {
            "Affected URLs"
        } else {
            "Betr. URLs"
        };
        let mut freq_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-problem")),
            TableColumn::new("WCAG"),
            TableColumn::new(i18n.t("batch-col-occurrences")),
            TableColumn::new(affected_col),
            TableColumn::new(i18n.t("batch-col-priority")),
        ])
        .with_title(i18n.t("batch-section-most-frequent-violations"));

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
    let scope_global_word = if en_top {
        "global (all pages)"
    } else {
        "global (alle Seiten)"
    };
    let scope_individual = if en_top {
        "individual pages"
    } else {
        "einzelne Seiten"
    };
    let occurrences_word_top = if en_top { "occurrences" } else { "Vorkommen" };
    let affected_urls_word = if en_top {
        "affected URLs"
    } else {
        "betroffene URLs"
    };
    let effort_word = if en_top { "Effort" } else { "Aufwand" };
    let scope_word = "Scope";
    let impact_user_label = if en_top {
        "Impact (user)"
    } else {
        "Impact (Nutzer)"
    };
    let impact_business_label = if en_top {
        "Impact (business)"
    } else {
        "Impact (Business)"
    };
    let fix_label = "Fix";
    let meta_label = "Meta";

    for group in pres.top_issues.iter().take(5) {
        let scope = if group.affected_urls.len() >= pres.portfolio_summary.total_urls {
            scope_global_word
        } else {
            scope_individual
        };
        let effort_label = group.effort.label();
        let meta_line = format!(
            "{} {} · {} {} · {}: {} · {}: {}",
            group.occurrence_count,
            occurrences_word_top,
            group.affected_urls.len(),
            affected_urls_word,
            effort_word,
            effort_label,
            scope_word,
            scope
        );

        let mut kv = KeyValueList::new().with_title(&group.title);
        kv = kv
            .add(
                i18n.t("findings-card-key-problem"),
                &group.customer_description,
            )
            .add(impact_user_label, &group.user_impact)
            .add(impact_business_label, &group.business_impact)
            .add(i18n.t("findings-card-key-cause"), &group.typical_cause)
            .add(fix_label, &group.recommendation)
            .add(meta_label, meta_line);
        builder = builder.add_component(kv);
    }

    // ── 4. Maßnahmenplan (mit Aufwand + Scope) ─────────────────────
    let action_intro = if en_top {
        "The following actions are prioritized by effort and impact. \
         Actions that improve many pages simultaneously take precedence."
    } else {
        "Die folgenden Maßnahmen sind nach Aufwand und Wirkung priorisiert. \
         Maßnahmen, die viele Seiten gleichzeitig verbessern, haben Vorrang."
    };
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-action-plan-title"), action_intro).with_level(1),
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
    let en_pdf = i18n.locale() == "en";
    if !pres.portfolio_summary.budget_summary.is_empty() {
        let pages_col = if en_pdf {
            "Affected pages"
        } else {
            "Betr. Seiten"
        };
        let budget_table_title = if en_pdf {
            "Performance budget violations (domain-wide)"
        } else {
            "Performance-Budget-Verstöße (domainweit)"
        };
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-metric")),
            TableColumn::new(i18n.t("batch-col-budget")),
            TableColumn::new(pages_col),
            TableColumn::new(i18n.t("batch-col-severity")),
        ])
        .with_title(budget_table_title);
        for (metric, budget, count, sev) in &pres.portfolio_summary.budget_summary {
            table = table.add_row(vec![
                metric.clone(),
                budget.clone(),
                count.to_string(),
                sev.clone(),
            ]);
        }
        let budgets_intro = if en_pdf {
            "Performance budgets define limits for load times, asset sizes and third-party \
             traffic. The following table shows on how many pages each budget was exceeded."
        } else {
            "Performance-Budgets definieren Obergrenzen für Ladezeiten, Asset-Größen und \
             Drittanbieter-Traffic. Die folgende Tabelle zeigt, auf wie vielen Seiten welche \
             Budgets überschritten wurden."
        };
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-performance-budgets")).with_level(1))
            .add_component(TextBlock::new(budgets_intro))
            .add_component(table);
    }

    // ── 5. Technische URL-Matrix ───────────────────────────────────
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-section-tech-url-matrix"),
            i18n.t("batch-section-tech-url-matrix-intro"),
        )
        .with_level(1),
    );

    if let Some(ref crawl_links) = pres.portfolio_summary.crawl_links {
        let target_col = if en_pdf { "Target" } else { "Ziel" };
        let type_col = if en_pdf { "Type" } else { "Typ" };
        let direct_label = if en_pdf { "direct" } else { "direkt" };
        let hops_label = if en_pdf { "hops" } else { "Hops" };
        let internal_intro = if en_pdf {
            format!(
                "For the crawl starting at {} we checked {} internal link targets. {} broken internal links detected.",
                crawl_links.seed_url,
                crawl_links.checked_internal_links,
                crawl_links.broken_internal_links.len()
            )
        } else {
            format!(
                "Für den Crawl ab {} wurden {} interne Linkziele geprüft. {} kaputte interne Verlinkungen wurden erkannt.",
                crawl_links.seed_url,
                crawl_links.checked_internal_links,
                crawl_links.broken_internal_links.len()
            )
        };
        builder = builder
            .add_component(
                Section::new(i18n.t("batch-section-broken-links-internal")).with_level(1),
            )
            .add_component(TextBlock::new(internal_intro));

        if crawl_links.broken_internal_links.is_empty() {
            builder = builder.add_component(Callout::info(if en_pdf {
                "No broken internal links detected in the audited crawl set."
            } else {
                "Keine kaputten internen Links im geprüften Crawl-Set erkannt."
            }));
        } else {
            let mut table = AuditTable::new(vec![
                TableColumn::new(i18n.t("batch-col-source")),
                TableColumn::new(target_col),
                TableColumn::new(i18n.t("batch-col-status-code")),
                TableColumn::new(type_col),
            ])
            .with_title(i18n.t("batch-table-broken-internal"));

            for row in &crawl_links.broken_internal_links {
                let severity_color = match row.severity.as_str() {
                    "high" => "#dc2626",
                    "medium" => "#ea580c",
                    _ => "#ca8a04",
                };
                let typ_label = if row.redirect_hops > 0 {
                    format!("→{} {}", row.redirect_hops, hops_label)
                } else {
                    direct_label.to_string()
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
            let ext_intro = if en_pdf {
                format!(
                    "{} external link targets checked. {} broken external links detected.",
                    crawl_links.checked_external_links,
                    crawl_links.broken_external_links.len()
                )
            } else {
                format!(
                    "{} externe Linkziele geprüft. {} kaputte externe Verlinkungen erkannt.",
                    crawl_links.checked_external_links,
                    crawl_links.broken_external_links.len()
                )
            };
            builder = builder
                .add_component(
                    Section::new(i18n.t("batch-section-broken-links-external")).with_level(2),
                )
                .add_component(TextBlock::new(ext_intro));

            let mut ext_table = AuditTable::new(vec![
                TableColumn::new(i18n.t("batch-col-source")),
                TableColumn::new(target_col),
                TableColumn::new(i18n.t("batch-col-status-code")),
                TableColumn::new(type_col),
            ])
            .with_title(i18n.t("batch-table-broken-external"));

            for row in &crawl_links.broken_external_links {
                let typ_label = if row.redirect_hops > 0 {
                    format!("→{} {}", row.redirect_hops, hops_label)
                } else {
                    direct_label.to_string()
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
            let ext_clean_msg = if en_pdf {
                format!(
                    "{} external link targets checked — no broken external links detected.",
                    crawl_links.checked_external_links
                )
            } else {
                format!(
                    "{} externe Linkziele geprüft — keine kaputten externen Links erkannt.",
                    crawl_links.checked_external_links
                )
            };
            builder = builder
                .add_component(Section::new(i18n.t("batch-section-external-links")).with_level(2))
                .add_component(Callout::info(ext_clean_msg));
        }

        // Redirect chains
        if !crawl_links.redirect_chains.is_empty() {
            let chain_intro = if en_pdf {
                format!(
                    "{} links with more than one redirect hop detected.",
                    crawl_links.redirect_chains.len()
                )
            } else {
                format!(
                    "{} Links mit mehr als einem Redirect-Hop erkannt.",
                    crawl_links.redirect_chains.len()
                )
            };
            builder = builder
                .add_component(Section::new(i18n.t("batch-section-redirect-chains")).with_level(2))
                .add_component(TextBlock::new(chain_intro));

            let mut chain_table = AuditTable::new(vec![
                TableColumn::new(i18n.t("batch-col-source")),
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

    let page_col = if en_pdf { "Page" } else { "Seite" };
    let title_col = if en_pdf { "Title" } else { "Titel" };
    let mut matrix = AuditTable::new(vec![
        TableColumn::new("#").with_width("4%"),
        TableColumn::new(page_col).with_width("26%"),
        TableColumn::new(title_col).with_width("28%"),
        TableColumn::new(i18n.t("batch-col-links-to")).with_width("10%"),
        TableColumn::new(i18n.t("batch-col-links-from")).with_width("10%"),
        TableColumn::new(i18n.t("batch-col-words")).with_width("10%"),
        TableColumn::new("Score").with_width("12%"),
    ])
    .with_title(i18n.t("batch-table-pages-overview"));

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
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-attributes")),
            TableColumn::new(i18n.t("batch-col-top-issues")),
        ])
        .with_title(i18n.t("batch-table-focus-pages"));

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
    let en_batch = i18n.locale() == "en";
    let (content_seo_title, content_seo_intro) = if en_batch {
        (
            "Content & SEO potential",
            "Content strengths and weaknesses with direct relevance to rankings, visibility and conversion. \
             Each finding is tied to a concrete action.",
        )
    } else {
        (
            "Content & SEO-Potenzial",
            "Content-Stärken und -Schwächen mit direktem Bezug zu Rankings, Sichtbarkeit und Conversion. \
             Jede Auffälligkeit ist an eine konkrete Handlung geknüpft.",
        )
    };
    builder = builder
        .add_component(SectionHeaderSplit::new(content_seo_title, content_seo_intro).with_level(1));

    // Schwache Seiten zuerst — mit Business-Impact
    if !pres.portfolio_summary.weakest_content_pages.is_empty() {
        let issues_title = if en_batch {
            "Content issues needing action"
        } else {
            "Content-Probleme mit Handlungsbedarf"
        };
        let high_marker = if en_batch { "high" } else { "hoch" };
        let mut issues_kv = KeyValueList::new().with_title(issues_title);
        for (url, page_type, score) in &pres.portfolio_summary.weakest_content_pages {
            let relevance = business_relevance(Some(page_type.as_str()), url, i18n.locale());
            let impact = if relevance == high_marker {
                if en_batch {
                    "Likely ranking loss + lower conversion"
                } else {
                    "Rankingverlust + geringere Conversion wahrscheinlich"
                }
            } else if *score < 30 {
                if en_batch {
                    "Weak organic visibility"
                } else {
                    "Schwache organische Sichtbarkeit"
                }
            } else if en_batch {
                "Optimization potential for SEO"
            } else {
                "Optimierungspotenzial für SEO"
            };
            let value = if en_batch {
                format!(
                    "{} — {} → +300–800 words of structured content recommended",
                    page_type, impact
                )
            } else {
                format!(
                    "{} — {} → +300–800 Wörter strukturierter Inhalt empfohlen",
                    page_type, impact
                )
            };
            let key_str = if en_batch {
                format!("{} (profile: {}/100)", truncate_url(url, 35), score)
            } else {
                format!("{} (Profil: {}/100)", truncate_url(url, 35), score)
            };
            issues_kv = issues_kv.add(key_str, value);
        }
        builder = builder.add_component(issues_kv);
    }

    // Content-Auffälligkeiten als Business-Relevanz
    if !pres.portfolio_summary.distribution_insights.is_empty() {
        let action_label = if en_batch {
            "Action needed"
        } else {
            "Handlungsbedarf"
        };
        let panel_title = if en_batch {
            "Content patterns → business impact"
        } else {
            "Content-Auffälligkeiten → Business-Impact"
        };
        let rows: Vec<ChecklistRow> = pres
            .portfolio_summary
            .distribution_insights
            .iter()
            .map(|insight| {
                let impact = if insight.contains("Thin") || insight.contains("dünn") {
                    if en_batch {
                        format!("{} → weaker rankings, lower dwell time", insight)
                    } else {
                        format!("{} → schwächere Rankings, geringere Verweildauer", insight)
                    }
                } else if insight.contains("Duplikat") || insight.contains("duplicate") {
                    if en_batch {
                        format!(
                            "{} → keyword cannibalization, split ranking signals",
                            insight
                        )
                    } else {
                        format!(
                            "{} → Keyword-Kannibalisierung, Split der Ranking-Signale",
                            insight
                        )
                    }
                } else {
                    insight.clone()
                };
                ChecklistRow::new(action_label, &impact).with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(panel_title));
    }

    // Near-duplicates mit Business-Kontext
    if !pres.portfolio_summary.near_duplicates.is_empty() {
        let near_dup_title = if en_batch {
            "Near-duplicate content → keyword cannibalization"
        } else {
            "Near-Duplicate-Content → Keyword-Kannibalisierung"
        };
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-page-a")),
            TableColumn::new(i18n.t("batch-col-page-b")),
            TableColumn::new(i18n.t("batch-col-similarity")),
            TableColumn::new(i18n.t("batch-col-risk")),
        ])
        .with_title(near_dup_title);

        for (url_a, url_b, sim) in &pres.portfolio_summary.near_duplicates {
            let risk = if *sim >= 95 {
                if en_batch {
                    "High — consolidate"
                } else {
                    "Hoch — konsolidieren"
                }
            } else if en_batch {
                "Medium — differentiate"
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
        let high_label = if en_batch { "high" } else { "hoch" };
        let medium_label = if en_batch { "medium" } else { "mittel" };
        let low_label = if en_batch { "low" } else { "niedrig" };
        let mut type_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-pages-list")),
            TableColumn::new(i18n.t("batch-col-share")),
            TableColumn::new(i18n.t("batch-col-relevance")),
        ])
        .with_title(i18n.t("batch-table-page-type-distribution"));

        for (label, count, pct) in &pres.portfolio_summary.page_type_distribution {
            let relevance = match label.as_str() {
                "Marketing / Landing Page"
                | "Transaktional / Utility"
                | "Transactional / Utility" => high_label,
                "Editorial / Artikel"
                | "Editorial / Article"
                | "Strukturierter Wissensinhalt"
                | "Structured knowledge content" => medium_label,
                "Thin / Minimal Content" => low_label,
                _ => medium_label,
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
            if en_batch {
                format!("All {} pages have structured data.", total)
            } else {
                format!("Alle {} Seiten haben strukturierte Daten.", total)
            }
        } else if en_batch {
            format!("{} of {} pages without structured data.", without, total)
        } else {
            format!("{} von {} Seiten ohne strukturierte Daten.", without, total)
        };
        let schema_callout_title = if en_batch {
            "Structured data (Schema.org)"
        } else {
            "Strukturierte Daten (Schema.org)"
        };
        builder = builder.add_component(Callout::info(&summary).with_title(schema_callout_title));
        let mut schema_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-schema-type")).with_width("55%"),
            TableColumn::new(i18n.t("batch-col-pages-list")).with_width("20%"),
            TableColumn::new(i18n.t("batch-col-share")).with_width("25%"),
        ])
        .with_title(i18n.t("batch-table-schema-distribution"));
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
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-profile")),
        ])
        .with_title(i18n.t("batch-table-top-pages"));

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
    builder = render_next_steps_batch(builder, &pres, &i18n);

    // ── 7. Anhang ───────────────────────────────────────────────────
    if config.level == ReportLevel::Technical && !pres.appendix.per_url.is_empty() {
        let appendix_intro = if en_pdf {
            "Complete listing of all detected violations per URL with technical details for implementation."
        } else {
            "Vollständige Auflistung aller erkannten Verstöße pro URL \
             mit technischen Details für die Umsetzung."
        };
        builder = builder.add_component(
            SectionHeaderSplit::new(i18n.t("section-appendix"), appendix_intro).with_level(1),
        );

        let rule_col = if en_pdf { "Rule" } else { "Regel" };
        let elements_col = if en_pdf {
            "Affected elements"
        } else {
            "Betr. Elemente"
        };
        for url_appendix in &pres.appendix.per_url {
            if url_appendix.violations.is_empty() {
                continue;
            }

            builder = builder
                .add_component(Section::new(truncate_url(&url_appendix.url, 70)).with_level(2));

            let mut table = AuditTable::new(vec![
                TableColumn::new(rule_col),
                TableColumn::new(i18n.t("batch-col-severity")),
                TableColumn::new(i18n.t("batch-col-description")),
                TableColumn::new(elements_col),
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
    config: &ReportConfig,
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

    let i18n = I18n::new(&config.locale)?;
    let en_cmp = i18n.locale() == "en";
    let date_format = if en_cmp { "%Y-%m-%d" } else { "%d.%m.%Y" };
    let mut builder = engine
        .report("wcag-comparison")
        .metadata("date", chrono::Local::now().format(date_format).to_string())
        .metadata("author", &author)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "");

    let cover_logo_asset = cover_logo_asset(config);
    builder = register_cover_logo_asset(builder, config, cover_logo_asset);

    let comparison_subline = if en_cmp {
        format!(
            "Comparison of {} domains — avg score: {}/100",
            comparison.entries.len(),
            avg_score
        )
    } else {
        format!(
            "Vergleich von {} Domains — Ø Score: {}/100",
            comparison.entries.len(),
            avg_score
        )
    };

    builder = builder
        .add_component(Image::new(cover_logo_asset).with_width("22%"))
        .add_component(
            Label::new(&author)
                .with_size("10pt")
                .bold()
                .with_color("#0f766e"),
        )
        .add_component(
            Label::new(i18n.t("comparison-cover-title"))
                .with_size("28pt")
                .bold(),
        )
        .add_component(
            Label::new(comparison_subline)
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
    let ranking_intro = if en_cmp {
        format!(
            "Comparison of {} domains based on a full audit of each home page. \
             Average score: {}/100.",
            comparison.entries.len(),
            avg_score,
        )
    } else {
        format!(
            "Vergleich von {} Domains anhand eines vollständigen Audits der jeweiligen Startseite. \
             Durchschnittlicher Score: {}/100.",
            comparison.entries.len(),
            avg_score,
        )
    };
    builder = builder
        .add_component(Section::new(i18n.t("comparison-domain-ranking")).with_level(1))
        .add_component(TextBlock::new(ranking_intro));

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

    builder = builder
        .add_component(BenchmarkTable::new(rows).with_title(&i18n.t("comparison-domain-ranking")));

    // ── 2. Modul-Vergleich ───────────────────────────────────────────
    let has_module_data = comparison
        .entries
        .iter()
        .any(|e| e.seo_score.is_some() || e.performance_score.is_some());

    if has_module_data {
        builder = builder
            .add_component(Section::new(i18n.t("comparison-module-comparison")).with_level(1));

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
        builder = builder.add_component(
            Section::new(i18n.t("comparison-top-findings-per-domain")).with_level(1),
        );

        let top_findings_label = if en_cmp {
            "Top findings"
        } else {
            "Top Findings"
        };
        for entry in &comparison.entries {
            if entry.top_issues.is_empty() {
                continue;
            }
            builder = builder.add_component(
                Section::new(format!("{} — {}", entry.domain, top_findings_label)).with_level(2),
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
    use crate::audit::{AuditReport, BatchReport, ComparisonReport};
    use crate::cli::{ReportLevel, WcagLevel};
    use crate::wcag::{Severity, Violation, WcagResults};
    use std::path::PathBuf;
    use std::process::Command;

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

    #[test]
    fn test_single_pdf_smoke_renders_valid_pdf() {
        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };

        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        assert_pdf_smoke(&pdf, 20_000);
    }

    #[test]
    fn test_single_pdf_smoke_renders_all_report_levels() {
        for level in [
            ReportLevel::Executive,
            ReportLevel::Standard,
            ReportLevel::Technical,
        ] {
            let report = pdf_fixture_report();
            let config = ReportConfig {
                level,
                ..ReportConfig::default()
            };

            let pdf = generate_pdf(&report, &config).expect("PDF should render");
            assert_pdf_smoke(&pdf, 15_000);
        }
    }

    #[test]
    fn test_batch_pdf_smoke_renders_valid_pdf() {
        let batch = BatchReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://example.com"),
                pdf_fixture_report_for_url("https://example.com/about"),
            ],
            vec![],
            2_400,
        );

        let pdf = generate_batch_pdf(&batch, &ReportConfig::default()).expect("PDF should render");
        assert_pdf_smoke(&pdf, 20_000);
    }

    #[test]
    fn test_comparison_pdf_smoke_renders_valid_pdf() {
        let comparison = ComparisonReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://alpha.example.com"),
                pdf_fixture_report_for_url("https://beta.example.com"),
            ],
            2_400,
        );

        let pdf = generate_comparison_pdf(&comparison, &ReportConfig::default())
            .expect("PDF should render");
        assert_pdf_smoke(&pdf, 12_000);
    }

    #[test]
    fn test_cover_logo_asset_prefers_existing_custom_logo() {
        let logo = tempfile::NamedTempFile::new().expect("custom logo fixture should be writable");
        let config = ReportConfig {
            logo_path: Some(logo.path().to_path_buf()),
            ..ReportConfig::default()
        };

        assert_eq!(cover_logo_asset(&config), CUSTOM_COVER_LOGO_ASSET);
    }

    #[test]
    fn test_cover_logo_asset_falls_back_for_missing_custom_logo() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let missing_logo = temp_dir.path().join("missing-logo.svg");
        let config = ReportConfig {
            logo_path: Some(missing_logo),
            ..ReportConfig::default()
        };

        assert_eq!(cover_logo_asset(&config), WORDMARK_ASSET);
    }

    #[test]
    fn test_single_pdf_renders_in_english_locale() {
        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Standard,
            locale: "en".to_string(),
            ..ReportConfig::default()
        };

        let pdf = generate_pdf(&report, &config).expect("English PDF should render");
        assert_pdf_smoke(&pdf, 15_000);
    }

    #[test]
    fn test_pdf_german_and_english_outputs_differ() {
        // Locale-aware narrative must produce different PDF bytes.
        let report = pdf_fixture_report();
        let de = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Standard,
                locale: "de".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("German PDF should render");
        let en = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Standard,
                locale: "en".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("English PDF should render");
        assert_ne!(de, en, "German and English PDFs should differ in content");
    }

    #[test]
    fn test_pdf_with_custom_logo_differs_from_default() {
        // A custom logo asset registered on the cover must change PDF bytes.
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let logo_path = temp_dir.path().join("custom-logo.svg");
        // Minimal valid SVG so Typst can decode it.
        std::fs::write(
            &logo_path,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="32" viewBox="0 0 120 32"><rect width="120" height="32" fill="#ff00ff"/></svg>"##,
        )
        .expect("write logo");

        let report = pdf_fixture_report();
        let default_pdf =
            generate_pdf(&report, &ReportConfig::default()).expect("default PDF should render");
        let custom_pdf = generate_pdf(
            &report,
            &ReportConfig {
                logo_path: Some(logo_path),
                ..ReportConfig::default()
            },
        )
        .expect("custom logo PDF should render");

        assert_ne!(
            default_pdf, custom_pdf,
            "PDF with custom logo should differ from default cover"
        );
    }

    #[test]
    fn test_single_pdf_technical_renders_multiple_pages_when_pdftoppm_is_available() {
        let Some(pdftoppm) = find_executable("pdftoppm") else {
            return;
        };

        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Technical,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("auditmysite-pages.pdf");
        let png_prefix = temp_dir.path().join("auditmysite-pages");
        std::fs::write(&pdf_path, pdf).expect("PDF fixture should be writable");

        let status = Command::new(pdftoppm)
            .arg("-png")
            .arg("-r")
            .arg("72")
            .arg(&pdf_path)
            .arg(&png_prefix)
            .status()
            .expect("pdftoppm should run");
        assert!(status.success(), "pdftoppm failed with {status}");

        let mut produced_pages = 0;
        for entry in std::fs::read_dir(temp_dir.path()).expect("temp dir should be readable") {
            let entry = entry.expect("dir entry should be readable");
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("auditmysite-pages-") && name_str.ends_with(".png") {
                produced_pages += 1;
            }
        }

        assert!(
            produced_pages >= 3,
            "Technical report should render at least 3 pages, got {produced_pages}"
        );
    }

    #[test]
    fn test_single_pdf_first_page_can_be_rasterized_when_pdftoppm_is_available() {
        let Some(pdftoppm) = find_executable("pdftoppm") else {
            return;
        };

        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Executive,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("auditmysite-smoke.pdf");
        let png_prefix = temp_dir.path().join("auditmysite-smoke-page");
        std::fs::write(&pdf_path, pdf).expect("PDF fixture should be writable");

        let status = Command::new(pdftoppm)
            .arg("-png")
            .arg("-f")
            .arg("1")
            .arg("-singlefile")
            .arg(&pdf_path)
            .arg(&png_prefix)
            .status()
            .expect("pdftoppm should run");

        assert!(status.success(), "pdftoppm failed with {status}");

        let png_path = png_prefix.with_extension("png");
        let png = std::fs::read(&png_path).expect("first page PNG should exist");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "PNG header missing");
        assert!(
            png.len() > 10_000,
            "PNG too small to represent a rendered report page: {} bytes",
            png.len()
        );
    }

    fn assert_pdf_smoke(pdf: &[u8], min_size: usize) {
        assert!(pdf.starts_with(b"%PDF-"), "PDF header missing");
        assert!(
            pdf.windows(5).any(|window| window == b"%%EOF"),
            "PDF EOF marker missing"
        );
        assert!(
            pdf.len() > min_size,
            "PDF too small to contain the expected report layout: {} bytes",
            pdf.len()
        );
    }

    fn pdf_fixture_report() -> AuditReport {
        pdf_fixture_report_for_url("https://example.com")
    }

    fn pdf_fixture_report_for_url(url: &str) -> AuditReport {
        let mut results = WcagResults::new();
        results.nodes_checked = 42;
        results.passes = 8;
        results.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::High,
                "Image missing alternative text",
                "node-hero-image",
            )
            .with_selector("img.hero")
            .with_html_snippet("<img class=\"hero\" src=\"hero.jpg\">")
            .with_fix("Add a meaningful alt attribute"),
        );

        AuditReport::new(url.to_string(), WcagLevel::AA, results, 1_200)
    }

    /// Richer fixture with multiple violations across severities — closer to a real-world report.
    fn pdf_fixture_report_rich() -> AuditReport {
        let mut results = WcagResults::new();
        results.nodes_checked = 320;
        results.passes = 48;

        let violations = [
            (
                "1.4.3",
                "Contrast (Minimum)",
                WcagLevel::AA,
                Severity::Critical,
                "Text has insufficient color contrast ratio",
                "node-body-text",
            ),
            (
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::Critical,
                "Image missing alternative text on hero banner",
                "node-hero-1",
            ),
            (
                "4.1.2",
                "Name, Role, Value",
                WcagLevel::A,
                Severity::High,
                "Button has no accessible name",
                "node-cta-btn",
            ),
            (
                "2.4.4",
                "Link Purpose",
                WcagLevel::A,
                Severity::High,
                "Link text is not descriptive enough",
                "node-read-more",
            ),
            (
                "1.3.1",
                "Info and Relationships",
                WcagLevel::A,
                Severity::High,
                "Form field missing label",
                "node-email-input",
            ),
            (
                "2.4.1",
                "Bypass Blocks",
                WcagLevel::A,
                Severity::Medium,
                "Skip navigation link missing",
                "node-skip",
            ),
            (
                "2.4.6",
                "Headings and Labels",
                WcagLevel::AA,
                Severity::Medium,
                "Heading hierarchy skips levels",
                "node-h3",
            ),
            (
                "3.1.1",
                "Language of Page",
                WcagLevel::A,
                Severity::Low,
                "HTML lang attribute not set",
                "node-html",
            ),
        ];

        for (criterion, rule_name, level, severity, msg, node_id) in violations {
            results.add_violation(
                Violation::new(criterion, rule_name, level, severity, msg, node_id)
                    .with_selector(&format!("#{node_id}"))
                    .with_fix(&format!("Fix required for {rule_name}")),
            );
        }

        AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            3_800,
        )
    }

    fn find_executable(name: &str) -> Option<PathBuf> {
        let paths = std::env::var_os("PATH")?;
        std::env::split_paths(&paths)
            .map(|path| path.join(name))
            .find(|path| path.is_file())
    }

    /// Count PDF pages by scanning for `/Type /Page` objects (not `/Type /Pages`).
    fn count_pdf_pages(pdf: &[u8]) -> usize {
        let needle = b"/Type /Page";
        let mut count = 0;
        let mut i = 0;
        while i + needle.len() <= pdf.len() {
            if pdf[i..i + needle.len()] == *needle {
                // Exclude /Type /Pages (the catalogue node)
                if pdf.get(i + needle.len()).copied() != Some(b's') {
                    count += 1;
                }
            }
            i += 1;
        }
        count
    }

    #[test]
    fn test_executive_pdf_page_count_within_target() {
        // Use the richer fixture (8 violations across severities) to validate
        // that executive stays compact even with a realistic finding load.
        let report = pdf_fixture_report_rich();
        let config = ReportConfig {
            level: ReportLevel::Executive,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("Executive PDF should render");
        let pages = count_pdf_pages(&pdf);
        assert!(
            pages <= 8,
            "Executive PDF must be ≤ 8 pages per target, got {} pages",
            pages
        );
    }

    #[test]
    fn test_standard_pdf_page_count_reasonable() {
        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("Standard PDF should render");
        let pages = count_pdf_pages(&pdf);
        assert!(
            pages >= 3,
            "Standard PDF must have at least 3 pages, got {}",
            pages
        );
        assert!(
            pages <= 35,
            "Standard PDF must not exceed 35 pages, got {} pages",
            pages
        );
    }
}
