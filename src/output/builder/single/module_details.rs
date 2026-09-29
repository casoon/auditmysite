//! Per-module detail presentations for the single-page report, one file per
//! module (mirrors `output/pdf/detail_modules/`).

use crate::audit::normalized::{AuditContext, NormalizedReport};
use crate::i18n::I18n;
use crate::output::report_model::ModuleDetailsBlock;
use crate::output::search_experience::build_search_experience;

mod ai_transparency;
mod commerce;
mod dark_mode;
mod design_quality;
mod experience;
mod html_conform;
mod performance;
mod platform;
mod seo;

use ai_transparency::build_ai_transparency_details;
use commerce::build_commerce_details;
use dark_mode::build_dark_mode_details;
use design_quality::build_design_quality_details;
use experience::{build_journey_details, build_ux_details};
use html_conform::build_html_conform_details;
use performance::build_performance_details;
use platform::{build_mobile_details, build_security_details};
use seo::build_seo_details;

/// Read the pre-computed interpretation for a module. Falls back to an empty
/// string if the interpretation layer was not populated (should not happen
/// after a full normalize() call).
fn module_interpretation(normalized: &NormalizedReport, module: &str, locale: &str) -> String {
    normalized
        .interpretation
        .as_ref()
        .and_then(|i| i.per_module.get(module))
        .map(|t| t.for_locale(locale).to_string())
        .unwrap_or_default()
}

pub(super) fn build_module_details_from_normalized(
    i18n: &I18n,
    normalized: &AuditContext<'_>,
) -> ModuleDetailsBlock {
    let performance = build_performance_details(normalized, i18n);
    let search_experience = build_search_experience(normalized, i18n);
    let seo = build_seo_details(normalized, i18n);
    let security = build_security_details(normalized, i18n);
    let html_conform = build_html_conform_details(normalized, i18n);
    let commerce = build_commerce_details(normalized, i18n);
    let mobile = build_mobile_details(normalized, i18n);
    let dark_mode = build_dark_mode_details(normalized, i18n);
    let design_quality = build_design_quality_details(normalized, i18n);
    let ai_transparency = build_ai_transparency_details(normalized, i18n);
    let ux = build_ux_details(normalized, i18n);
    let journey = build_journey_details(normalized, i18n);

    let network_dns = normalized.raw_network_dns.cloned();

    let source_quality = normalized.raw_source_quality.cloned();
    let ai_visibility = normalized.raw_ai_visibility.cloned();
    let tech_stack = normalized.raw_tech_stack.cloned();
    let content_visibility = normalized.raw_content_visibility.cloned();
    let best_practices = normalized.raw_best_practices.cloned();
    let patterns = normalized.raw_patterns.cloned();

    let has_any = performance.is_some()
        || search_experience.is_some()
        || seo.is_some()
        || security.is_some()
        || html_conform.is_some()
        || commerce.is_some()
        || mobile.is_some()
        || ux.is_some()
        || journey.is_some()
        || dark_mode.is_some()
        || design_quality.is_some()
        || ai_transparency.is_some()
        || network_dns.is_some()
        || source_quality.is_some()
        || ai_visibility.is_some()
        || tech_stack.is_some()
        || content_visibility.is_some()
        || best_practices.is_some()
        || patterns.is_some();

    ModuleDetailsBlock {
        search_experience,
        performance,
        seo,
        security,
        html_conform,
        commerce,
        mobile,
        ux,
        journey,
        dark_mode,
        design_quality,
        ai_transparency,
        network_dns,
        source_quality,
        ai_visibility,
        tech_stack,
        content_visibility,
        best_practices,
        patterns,
        has_any,
    }
}

/// Static set of module keys covered by [`ModuleDetailsBlock`].
///
/// Every optional field in `ModuleDetailsBlock` that carries module data must
/// appear here. The parity test compares this against `active_modules()` to
/// detect future coverage gaps.
#[cfg(test)]
pub(super) fn pdf_rendered_modules() -> std::collections::BTreeSet<&'static str> {
    [
        "performance",
        "seo",
        "security",
        "html_conform",
        "commerce",
        "mobile",
        "ux",
        "journey",
        "dark_mode",
        "design_quality",
        "ai_transparency",
        "network_dns",
        "source_quality",
        "ai_visibility",
        "tech_stack",
        "content_visibility",
        "best_practices",
        "patterns",
    ]
    .into_iter()
    .collect()
}

pub(super) fn normalized_module_score(
    normalized: &NormalizedReport,
    module_name: &str,
) -> Option<u32> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.score)
}

pub(super) fn normalized_module_grade(
    normalized: &NormalizedReport,
    module_name: &str,
) -> Option<String> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .and_then(|m| m.grade.clone())
}
