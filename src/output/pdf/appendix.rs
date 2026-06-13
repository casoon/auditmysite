//! Appendix and snapshot helper components for PDF reports.

use renderreport::components::{AuditTable, TableColumn};

use super::cover::auditmysite_wordmark_path;
use super::{CUSTOM_COVER_LOGO_ASSET, WORDMARK_ASSET};
use crate::i18n::I18n;
use crate::output::report_model::*;

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
