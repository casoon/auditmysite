//! Appendix and snapshot helper components for PDF reports.

use renderreport::components::advanced::MetricStrip;
use renderreport::components::advanced::MetricStripItem;
use renderreport::components::{AuditTable, SummaryBox, TableColumn};

use super::cover::auditmysite_wordmark_path;
use super::design::module_score_color;
use super::{CUSTOM_COVER_LOGO_ASSET, WORDMARK_ASSET};
use crate::i18n::I18n;
use crate::output::report_model::*;

// Re-export risk_status so mod.rs can import it from appendix.
pub(super) use super::design::risk_status;

pub(super) fn build_module_strip(vm: &ReportViewModel, i18n: &I18n) -> MetricStrip {
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

pub(super) fn impact_row<'a>(rows: &'a [(String, String)], label: &str) -> &'a str {
    rows.iter()
        .find(|(key, _)| key == label)
        .map(|(_, value)| value.as_str())
        .unwrap_or("")
}

pub(super) fn build_cli_snapshot_table(vm: &ReportViewModel, i18n: &I18n) -> AuditTable {
    use super::findings::first_sentence;

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

pub(super) fn build_raw_audit_snapshot(vm: &ReportViewModel, i18n: &I18n) -> SummaryBox {
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
