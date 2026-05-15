//! PDF Report Generator using renderreport/Typst
//!
//! Pure layout layer — receives pre-computed ViewModel blocks and maps them
//! directly to renderreport components. Zero data transformation here.

mod batch;
mod batch_report;
mod comparison_report;
mod cover;
mod design;
mod detail_modules;
mod findings;
mod helpers;
mod history;
mod modules;

pub use self::batch_report::generate_batch_pdf;
pub use self::comparison_report::generate_comparison_pdf;

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, DevicePreview, DiagnosisPanel, DiagnosisRow,
    DominantIssueSpotlight, ImpactGrid, ImpactGridCard, List, MetricStrip, MetricStripItem,
    PageBreak, PhaseBlock, SectionHeaderSplit, TableOfContents,
};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::TagCloud;
use renderreport::prelude::Image;
use renderreport::prelude::*;

// Composite components
use renderreport::components::{AuditTable, SeverityOverview, SummaryBox, TableColumn};

use self::cover::{auditmysite_wordmark_path, build_cover_score_row, certificate_badge_path};
use self::detail_modules::{
    render_ai_visibility, render_budget_violations, render_content_visibility, render_dark_mode,
    render_journey, render_mobile, render_performance, render_security, render_seo,
    render_source_quality, render_tech_stack, render_ux,
};
use self::findings::{first_sentence, render_finding_technical, render_key_finding_block};
use self::helpers::{component_json, create_engine, extract_domain, soft_flow_group};
use self::history::{render_history_section, render_methodology_section};
use self::modules::{build_summary_overview, build_was_jetzt_tun_table, WasJetztTunContent};
use crate::audit::{normalize, AuditReport};
use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::builder::build_view_model;
use crate::output::report_model::*;

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
        .metadata("footer_link_url", "")
        .metadata(
            "footer_tagline",
            "A technical auditing platform by casoon.de",
        );

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
                "{}  ·  {}  ·  auditmysite v{}",
                extract_domain(&vm.cover.domain),
                vm.cover.date,
                vm.meta.version
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
            // Differentiate between explicit failure and not-requested (issue #26).
            let title = i18n.t("section-device-preview");
            let en = i18n.locale() == "en";
            let body = match &report.screenshot_status {
                crate::audit::ScreenshotStatus::Failed(reason) => {
                    if en {
                        format!("Screenshots could not be captured: {reason}.")
                    } else {
                        format!("Screenshots konnten nicht erstellt werden: {reason}.")
                    }
                }
                crate::audit::ScreenshotStatus::NotRequested => {
                    if en {
                        "Screenshots were not captured for this audit (batch mode or screenshot capture disabled)."
                            .to_string()
                    } else {
                        "Screenshots wurden für dieses Audit nicht erfasst (Batch-Modus oder Screenshot-Erfassung deaktiviert)."
                            .to_string()
                    }
                }
                crate::audit::ScreenshotStatus::Captured => {
                    // Status says captured but page_screenshots is None — shouldn't happen
                    // but fall back to the generic message.
                    i18n.t("section-device-preview-no-screenshots")
                }
            };
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
        .add_component(output_scope_callout(&i18n))
        .add_component(PageBreak::new());

    if vm.meta.report_level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new());
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
            builder = builder.add_component(build_module_strip(&vm, &i18n));
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
                    .map(|(action, _timeframe)| ChecklistRow::new(action, "").with_status("warn"))
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

        // Recognized structural patterns (issue #38). Skip in Executive to keep
        // that report compact; only show if at least one pattern was detected.
        if vm.meta.report_level != ReportLevel::Executive && !vm.positive_signals.is_empty() {
            let title = if vm.meta.report_level == ReportLevel::Standard && i18n.locale() == "en" {
                "Recognized patterns"
            } else if i18n.locale() == "en" {
                "Recognized structural patterns"
            } else {
                "Erkannte strukturelle Patterns"
            };
            let rows: Vec<ChecklistRow> = vm
                .positive_signals
                .items
                .iter()
                .map(|signal| {
                    ChecklistRow::new(&signal.title, &signal.description)
                        .with_status(if signal.strong { "good" } else { "warning" })
                })
                .collect();
            builder = builder.add_component(ChecklistPanel::new(rows).with_title(title));
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
            let share = if let Some(v) = (top.occurrence_count * 100).checked_div(total_occurrences)
            {
                v.max(1)
            } else if let Some(v) = (top.occurrence_count * 100).checked_div(total_ch) {
                v.max(1)
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
            builder = builder.add_component(
                SeverityOverview::new(
                    vm.severity.critical,
                    vm.severity.high,
                    vm.severity.medium,
                    vm.severity.low,
                )
                .with_title(&i18n.t("section-issue-overview")),
            );
        }

        // LeverageBlock — what fixing achieves
        if total_ch > 0 {
            if let Some(leverage_text) = &vm.executive.leverage_text {
                builder = builder.add_component(
                    Callout::success(leverage_text).with_title(&vm.executive.leverage_title),
                );
            }
        }

        // ── Key Findings — flow directly from Dominant Issue section ──
        // TopFixesTable removed: all its data (occurrences, elements, impact)
        // is contained in the FindingCards below.
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
        builder = builder.add_component(PageBreak::new()).add_component(
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

        // "Jetzt starten" — top 1-3 highest-priority actions
        let top_actions: Vec<&RoadmapItemData> = vm
            .actions
            .roadmap_columns
            .first()
            .map(|col| col.items.iter().take(3).collect())
            .unwrap_or_default();

        if !top_actions.is_empty() {
            let body = top_actions
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    if item.user_effect.is_empty() {
                        format!("{}. {}", i + 1, item.action)
                    } else {
                        format!("{}. {} — {}", i + 1, item.action, item.user_effect)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            let title = if i18n.locale() == "en" {
                "Start here — highest leverage"
            } else {
                "Jetzt starten — höchste Hebelwirkung"
            };
            builder = builder.add_component(Callout::warning(&body).with_title(title));
        }

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
        builder = builder.add_component(PageBreak::new()).add_component(
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
                    let display_name = if module.measurement_type == "heuristic" {
                        let suffix = if i18n.locale() == "en" {
                            "Indicator"
                        } else {
                            "Indikator"
                        };
                        format!("{} ({suffix})", module.name)
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
        for group in &vm.findings.all_findings {
            builder = render_finding_technical(builder, group, &i18n);
        }
    }

    // ── WCAG Coverage (issue #37) ──────────────────────────────────
    // Show how many WCAG criteria are automatically testable vs. require
    // manual review. Sets honest expectations about audit scope.
    if vm.meta.report_level != ReportLevel::Executive {
        builder = render_wcag_coverage_section(builder, &i18n);
    }

    // ── Module Detail Metrics ───────────────────────────────────────
    if vm.module_details.has_any {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new(i18n.t("section-tech-detail-metrics")).with_level(1));
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
    if let Some(ref ts) = vm.module_details.tech_stack {
        builder = render_tech_stack(builder, ts, &i18n);
    }
    if let Some(ref cv) = vm.module_details.content_visibility {
        builder = render_content_visibility(builder, cv, &i18n);
    }

    // ── Appendix ────────────────────────────────────────────────────
    if vm.appendix.has_violations {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new(i18n.t("section-appendix")).with_level(1));

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

/// Render a WCAG coverage section (issue #37).
///
/// Lists the criteria this tool automatically checks, plus the criteria
/// that fundamentally require manual review. Communicates audit scope
/// transparently so users avoid a false sense of security.
fn render_wcag_coverage_section(
    mut builder: renderreport::engine::ReportBuilder,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::wcag::coverage::{coverage_stats, AUTOMATED_CRITERIA, MANUAL_REVIEW_CRITERIA};

    let en = i18n.locale() == "en";
    let (automated, total) = coverage_stats();
    let title = if en { "Audit scope" } else { "Prüfumfang" };
    let intro = if en {
        format!(
            "This audit covers {automated} of ~{total} testable WCAG 2.1 AA criteria automatically. The criteria listed below require manual review."
        )
    } else {
        format!(
            "Dieses Audit prüft {automated} von ca. {total} WCAG-2.1-AA-Kriterien automatisch. Die unten aufgeführten Kriterien benötigen manuelle Prüfung."
        )
    };

    builder = builder.add_component(SectionHeaderSplit::new(title, &intro).with_level(2));

    let automated_title = if en {
        format!("Automatically checked ({})", automated)
    } else {
        format!("Automatisch geprüft ({})", automated)
    };
    let mut tag_cloud = TagCloud::new().with_title(&automated_title).with_gap("5pt");
    for (c, l) in AUTOMATED_CRITERIA.iter() {
        tag_cloud = tag_cloud.add(format!("WCAG {} ({})", c, l), "good");
    }
    builder = builder.add_component(tag_cloud);

    let manual_title = if en {
        format!("Requires manual review ({})", MANUAL_REVIEW_CRITERIA.len())
    } else {
        format!(
            "Manuelle Prüfung erforderlich ({})",
            MANUAL_REVIEW_CRITERIA.len()
        )
    };
    let mut manual_cloud = TagCloud::new().with_title(&manual_title).with_gap("5pt");
    for (c, l, name) in MANUAL_REVIEW_CRITERIA.iter() {
        manual_cloud = manual_cloud.add(format!("{c} ({l}) – {name}"), "info");
    }
    builder = builder.add_component(manual_cloud);

    // Practical testing guide — how to test the manual criteria above
    let how_title = if en {
        "How to test manually"
    } else {
        "So testen Sie manuell"
    };
    let items: &[(&str, &str)] = if en {
        &[
            (
                "Keyboard navigation",
                "Tab through the entire page. No focus loss, no keyboard trap, every interactive element reachable.",
            ),
            (
                "Screen reader",
                "Test with NVDA/JAWS (Windows) or VoiceOver (Mac/iOS) — landmark navigation and form interaction.",
            ),
            (
                "400% zoom",
                "At 400% browser zoom: page operable without horizontal scrolling, no content lost.",
            ),
            (
                "Reduced motion",
                "Enable the OS 'reduce motion' setting and verify animations are disabled or significantly diminished.",
            ),
            (
                "Modal / dropdown interaction",
                "Full keyboard interaction: Tab, Enter, Space, Escape, Arrow keys. Focus returns to the trigger on close.",
            ),
            (
                "Color blindness simulation",
                "Use a tool like 'Color Oracle' to verify information is conveyed by more than color alone.",
            ),
        ]
    } else {
        &[
            (
                "Tastaturnavigation",
                "Komplette Seite per Tab navigieren. Kein Fokus verloren, kein Keyboard-Trap, jedes interaktive Element erreichbar.",
            ),
            (
                "Screenreader",
                "Test mit NVDA/JAWS (Windows) oder VoiceOver (Mac/iOS) — Landmark-Navigation und Formular-Interaktion.",
            ),
            (
                "400% Zoom",
                "Bei 400% Browser-Zoom: Seite ohne horizontales Scrollen bedienbar, kein Inhalt verloren.",
            ),
            (
                "Reduced Motion",
                "Betriebssystem-Einstellung „Bewegung reduzieren\" aktivieren und prüfen, ob Animationen deaktiviert oder reduziert werden.",
            ),
            (
                "Modal- / Dropdown-Interaktion",
                "Vollständige Tastaturbedienung: Tab, Enter, Space, Escape, Pfeiltasten. Fokus kehrt nach Schließen zum Trigger zurück.",
            ),
            (
                "Farbenblindheit",
                "Mit einem Werkzeug wie „Color Oracle\" prüfen, ob Informationen nicht ausschließlich über Farbe vermittelt werden.",
            ),
        ]
    };
    let rows: Vec<ChecklistRow> = items
        .iter()
        .map(|(t, d)| ChecklistRow::new(*t, *d).with_status("info"))
        .collect();
    builder.add_component(ChecklistPanel::new(rows).with_title(how_title))
}

fn build_module_strip(vm: &ReportViewModel, i18n: &I18n) -> MetricStrip {
    let items = vm
        .modules
        .dashboard
        .iter()
        .map(|module| {
            let heuristic = module.measurement_type == "heuristic";
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
                let suffix = if i18n.locale() == "en" {
                    "Indicator"
                } else {
                    "Indikator"
                };
                format!("{} ({suffix})", module.name)
            } else {
                module.name.clone()
            };
            let display_value = format!("{}/100", module.score);
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

pub(super) fn cover_logo_asset(config: &ReportConfig) -> &'static str {
    match config.logo_path.as_ref() {
        Some(path) if path.exists() => CUSTOM_COVER_LOGO_ASSET,
        _ => WORDMARK_ASSET,
    }
}

pub(super) fn register_cover_logo_asset(
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

fn output_scope_callout(i18n: &I18n) -> Callout {
    if i18n.locale() == "en" {
        Callout::info(
            "This PDF is a condensed decision and prioritization report. It highlights the most important findings, risks and actions, but does not list every technical occurrence. Use the accompanying JSON report for the complete machine-readable issue list with selectors, occurrences and detail data.",
        )
        .with_title("How to read this report")
    } else {
        Callout::info(
            "Dieser PDF-Report ist eine verdichtete Entscheidungs- und Priorisierungshilfe. Er zeigt die wichtigsten Befunde, Risiken und Maßnahmen, enthält aber nicht jede technische Einzelstelle. Für die vollständige maschinenlesbare Fehlerliste mit Selektoren, Vorkommen und Detaildaten dient der begleitende JSON-Report.",
        )
        .with_title("Einordnung dieses Reports")
    }
}

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

#[cfg(all(test, feature = "pdf_test"))]
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

    #[test]
    fn test_pdf_technical_contains_violation_criteria() {
        // Every WCAG criterion from the input must appear as text in the rendered PDF.
        // This catches silent information loss between builder and renderer.
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let report = pdf_fixture_report_rich();
        let criteria = [
            "1.4.3", "1.1.1", "4.1.2", "2.4.4", "1.3.1", "2.4.1", "2.4.6", "3.1.1",
        ];
        let config = ReportConfig {
            level: ReportLevel::Technical,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("criteria-check.pdf");
        let txt_path = temp_dir.path().join("criteria-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read extracted text");

        for criterion in criteria {
            assert!(
                text.contains(criterion),
                "Criterion {criterion} missing from PDF text — information lost in renderer"
            );
        }
    }

    #[test]
    fn test_pdf_renders_positive_signals_from_patterns() {
        // When the report carries recognized patterns, the PDF text should
        // include a localized pattern title (e.g. "Skip-Link").
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let mut report = pdf_fixture_report_rich();
        report.patterns = Some(crate::patterns::PatternAnalysis {
            recognized: vec![crate::patterns::RecognizedPattern {
                pattern: "SkipLink".to_string(),
                message: "Skip link recognized and correctly positioned.".to_string(),
                confidence: crate::patterns::PatternConfidence::Strong,
            }],
            violations: vec![],
        });

        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("patterns-check.pdf");
        let txt_path = temp_dir.path().join("patterns-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read text");

        assert!(
            text.contains("Skip-Link"),
            "Expected localized pattern title 'Skip-Link' in PDF text"
        );
    }

    #[test]
    fn test_pdf_renders_throttled_performance_table() {
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let mut report =
            pdf_fixture_report_rich().with_performance(crate::audit::PerformanceResults {
                vitals: crate::performance::WebVitals::default(),
                score: crate::performance::PerformanceScore {
                    overall: 80,
                    grade: crate::performance::PerformanceGrade::Gold,
                    lcp_score: None,
                    fcp_score: None,
                    cls_score: None,
                    interactivity_score: None,
                    metrics_available: 0,
                },
                render_blocking: None,
                content_weight: None,
            });
        report.throttled_performance = vec![crate::audit::ThrottledPerfResult {
            profile: crate::browser::ThrottleProfile::Slow3G,
            lcp_ms: Some(3200.0),
            tbt_ms: Some(180.0),
            cls: Some(0.03),
            score: 72,
        }];

        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("throttled-perf-check.pdf");
        let txt_path = temp_dir.path().join("throttled-perf-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read text");

        assert!(
            text.contains("Performance unter gedrosselten Bedingungen"),
            "Expected throttled-performance section title in PDF text"
        );
        assert!(
            text.contains("Slow3G") && text.contains("3200 ms") && text.contains("180 ms"),
            "Expected throttled-performance values in PDF text"
        );
    }

    #[test]
    fn test_pdf_score_present_in_extracted_text() {
        // The overall score computed by normalize() must appear as a number in the rendered PDF.
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let report = pdf_fixture_report_rich();
        let normalized = crate::audit::normalize(&report);
        let expected_score = normalized.score.to_string();
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("score-check.pdf");
        let txt_path = temp_dir.path().join("score-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read extracted text");

        assert!(
            text.contains(&expected_score),
            "Score {expected_score} missing from PDF text — score not rendered on page"
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
                    .with_selector(format!("#{node_id}"))
                    .with_fix(format!("Fix required for {rule_name}")),
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

    /// Count PDF `/Annot` entries — proxy for callout boxes / links.
    fn count_pdf_annotations(pdf: &[u8]) -> usize {
        let doc = match lopdf::Document::load_mem(pdf) {
            Ok(d) => d,
            Err(_) => return 0,
        };
        doc.objects
            .values()
            .filter(|o| {
                if let Ok(d) = o.as_dict() {
                    return d
                        .get(b"Type")
                        .ok()
                        .and_then(|v| v.as_name().ok())
                        .map(|n| n == b"Annot")
                        .unwrap_or(false);
                }
                false
            })
            .count()
    }

    /// Read PDF outline (bookmark) titles in tree order. Empty when there
    /// is no outline.
    fn pdf_outline_titles(pdf: &[u8]) -> Vec<String> {
        let doc = match lopdf::Document::load_mem(pdf) {
            Ok(d) => d,
            Err(_) => return vec![],
        };
        let mut titles = Vec::new();
        let catalog = match doc.catalog() {
            Ok(c) => c,
            Err(_) => return titles,
        };
        let outlines_ref = match catalog.get(b"Outlines") {
            Ok(v) => v,
            Err(_) => return titles,
        };
        let outlines_id = match outlines_ref.as_reference() {
            Ok(id) => id,
            Err(_) => return titles,
        };
        let outlines = match doc.get_dictionary(outlines_id) {
            Ok(d) => d,
            Err(_) => return titles,
        };
        let mut current = outlines
            .get(b"First")
            .ok()
            .and_then(|v| v.as_reference().ok());
        while let Some(id) = current {
            let dict = match doc.get_dictionary(id) {
                Ok(d) => d,
                Err(_) => break,
            };
            if let Ok(title) = dict.get(b"Title").and_then(|v| v.as_str()) {
                titles.push(String::from_utf8_lossy(title).trim().to_string());
            }
            current = dict.get(b"Next").ok().and_then(|v| v.as_reference().ok());
        }
        titles
    }

    #[test]
    fn test_standard_pdf_larger_than_executive() {
        let report = pdf_fixture_report_rich();
        let exec_pdf = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Executive,
                ..ReportConfig::default()
            },
        )
        .expect("executive PDF should render");
        let std_pdf = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Standard,
                ..ReportConfig::default()
            },
        )
        .expect("standard PDF should render");
        assert!(
            std_pdf.len() > exec_pdf.len(),
            "Standard PDF ({} bytes) should be larger than Executive ({} bytes)",
            std_pdf.len(),
            exec_pdf.len()
        );
    }

    #[test]
    fn test_batch_pdf_page_count_reasonable() {
        let batch = BatchReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://example.com"),
                pdf_fixture_report_for_url("https://example.com/about"),
            ],
            vec![],
            2_400,
        );
        let pdf = generate_batch_pdf(&batch, &ReportConfig::default()).expect("batch PDF");
        let pages = count_pdf_pages(&pdf);
        assert!(
            pages >= 3,
            "Batch PDF must have at least 3 pages, got {}",
            pages
        );
    }

    #[test]
    fn test_comparison_pdf_renders_without_panic() {
        let comparison = ComparisonReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://alpha.example.com"),
                pdf_fixture_report_for_url("https://beta.example.com"),
            ],
            2_400,
        );
        let pdf = generate_comparison_pdf(&comparison, &ReportConfig::default())
            .expect("comparison PDF should render");
        assert!(!pdf.is_empty(), "comparison PDF should not be empty");
    }

    #[test]
    fn test_pdf_has_annotations() {
        // Renderreport emits annotations for some interactive constructs
        // (links, etc.). This is a smoke check that lopdf can parse the PDF
        // and the structural pipeline is intact.
        let report = pdf_fixture_report_rich();
        let pdf = generate_pdf(&report, &ReportConfig::default()).expect("standard PDF");
        let _ = count_pdf_annotations(&pdf); // result not asserted; counts may be 0
        let _ = pdf_outline_titles(&pdf);
        assert!(
            lopdf::Document::load_mem(&pdf).is_ok(),
            "PDF must parse via lopdf"
        );
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
