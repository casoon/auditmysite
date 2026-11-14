//! Text export of the builder helpers, the batch builder and the display
//! texts, slice 21 of #936: every text of `helpers.rs` (yes/no, report
//! title and subtitle, security tier and kind labels), `batch.rs`
//! (distribution insights, the lever fallback, the render-blocking summary
//! labels, the cover title and URL line) and `display/mod.rs` (audited mode
//! and offered modes), across an enumerated input space in both languages,
//! as records. They are part of `reports/builder-detail-texts.json`
//! (gitignored; the path can be set with `BUILDER_DETAIL_TEXT_EXPORT`),
//! which `output::pdf::detail_modules::module_text_export` writes together
//! with the PDF detail modules. Run it before and after a change of how the
//! texts are stored and compare the two files byte for byte.

use super::super::helpers::{
    localized_report_subtitle, localized_report_title, security_header_tier_label,
    security_issue_kind_label, yes_no,
};
use super::*;
use crate::audit::{AuditReport, BatchReport};
use crate::display::{AuditedDisplayMode, DisplayMode, DisplayModesInfo};
use crate::model::WcagLevel;
use crate::performance::{
    BlockingResource, PerformanceGrade, PerformanceScore, RenderBlockingAnalysis, WebVitals,
};
use crate::wcag::WcagResults;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// The locales the helpers take as a string: both languages and one the
/// helpers treat as the default.
const LOCALES: [&str; 3] = ["en", "de", "fr"];

fn push(records: &mut Vec<Value>, name: &str, input: String, lang: &str, output: Value) {
    records.push(json!([name, input, lang, output]));
}

fn blocking(url: &str, kind: &str) -> BlockingResource {
    BlockingResource {
        url: url.to_string(),
        transfer_bytes: 1_000,
        kind: kind.to_string(),
    }
}

/// A page with a render-blocking analysis (`scripts` + `css` resources and
/// `third_party_bytes` of third-party traffic).
fn page(url: &str, scripts: usize, css: usize, third_party_bytes: u64) -> AuditReport {
    let mut report = AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100);
    let score = PerformanceScore {
        overall: 80,
        grade: PerformanceGrade::Gold,
        lcp_score: None,
        fcp_score: None,
        cls_score: None,
        interactivity_score: None,
        si_score: None,
        metrics_available: 0,
        size_penalty: None,
        js_penalty: None,
        request_penalty: None,
        dom_penalty: None,
        is_capped: None,
        partial: false,
    };
    let mut performance = crate::audit::PerformanceResults::new(WebVitals::default(), score);
    performance.render_blocking = Some(RenderBlockingAnalysis {
        blocking_scripts: (0..scripts)
            .map(|i| blocking(&format!("https://example.com/{i}.js"), "script"))
            .collect(),
        blocking_css: (0..css)
            .map(|i| blocking(&format!("https://example.com/{i}.css"), "css"))
            .collect(),
        blocking_transfer_bytes: 0,
        first_party_bytes: 0,
        third_party_bytes,
        third_party_origin_count: 0,
        suggestions: vec![],
    });
    report.performance = Some(performance);
    report
}

fn export_helpers(records: &mut Vec<Value>) {
    for locale in LOCALES {
        for val in [true, false] {
            push(
                records,
                "yes_no",
                format!("{val}"),
                locale,
                json!(yes_no(locale, val)),
            );
        }
        push(
            records,
            "localized_report_title",
            String::new(),
            locale,
            json!(localized_report_title(locale)),
        );
        push(
            records,
            "localized_report_subtitle",
            String::new(),
            locale,
            json!(localized_report_subtitle(locale)),
        );
        for tier in [
            crate::security::HeaderTier::Baseline,
            crate::security::HeaderTier::ArchitectureDependent,
            crate::security::HeaderTier::ContextDependent,
            crate::security::HeaderTier::NotAssessable,
        ] {
            push(
                records,
                "security_header_tier_label",
                format!("{tier:?}"),
                locale,
                json!(security_header_tier_label(locale, tier)),
            );
        }
        for issue_type in [
            "missing_header",
            "missing_https",
            "weak_header",
            "misconfigured",
            "",
        ] {
            push(
                records,
                "security_issue_kind_label",
                format!("{issue_type:?}"),
                locale,
                json!(security_issue_kind_label(locale, issue_type)),
            );
        }
    }
}

fn export_distribution(records: &mut Vec<Value>, lang: &str) {
    let en = lang == "en";
    for kind in [
        DistributionInsightKind::HighThinContentShare,
        DistributionInsightKind::NoEditorialContent,
        DistributionInsightKind::MarketingDominated,
    ] {
        let name = match kind {
            DistributionInsightKind::HighThinContentShare => "HighThinContentShare",
            DistributionInsightKind::NoEditorialContent => "NoEditorialContent",
            DistributionInsightKind::MarketingDominated => "MarketingDominated",
        };
        push(
            records,
            "distribution_insight_text",
            name.to_string(),
            lang,
            json!(distribution_insight_text(kind, en)),
        );
    }
}

fn export_presentation(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let clean = |url: &str| AuditReport::new(url.into(), WcagLevel::AA, WcagResults::new(), 100);
    let scenarios: [(&str, Vec<AuditReport>); 4] = [
        (
            "clean pages without module data",
            vec![
                clean("https://example.com/a"),
                clean("https://example.com/b"),
            ],
        ),
        (
            "one page, nothing blocking",
            vec![page("https://example.com/a", 0, 0, 0)],
        ),
        (
            "two pages with blocking resources",
            vec![
                page("https://example.com/a", 2, 1, 2_048),
                page("https://example.com/b", 0, 0, 1_234_567),
            ],
        ),
        (
            "three pages, a large third-party share",
            vec![
                page("https://example.com/a", 1, 0, 0),
                page("https://example.com/b", 1, 1, 10_000_000),
                page("https://example.com/c", 0, 0, 0),
            ],
        ),
    ];
    for (name, reports) in scenarios {
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let normalized: Vec<_> = batch
            .reports
            .iter()
            .map(|r| {
                crate::audit::validation::normalize_checked(r)
                    .expect("fixture report passes the report contract")
                    .normalized
            })
            .collect();
        let pres = build_batch_presentation_with_normalized(&batch, i18n, &normalized);
        push(
            records,
            "build_batch_presentation",
            name.to_string(),
            lang,
            json!({
                "cover_title": pres.cover.title,
                "cover_url": pres.cover.url,
                "render_blocking_summary": pres.portfolio_summary.render_blocking_summary,
                "distribution_insights": pres.portfolio_summary.distribution_insights,
                "biggest_levers": pres
                    .url_details
                    .iter()
                    .map(|d| d.biggest_lever.clone())
                    .collect::<Vec<_>>(),
            }),
        );
    }
}

fn export_display(records: &mut Vec<Value>, lang: &str) {
    let en = lang == "en";
    for mode in [
        AuditedDisplayMode::SiteDefault,
        AuditedDisplayMode::Visual,
        AuditedDisplayMode::Calm,
        AuditedDisplayMode::Text,
    ] {
        push(
            records,
            "audited_mode_text",
            format!("{mode:?}"),
            lang,
            json!(crate::display::audited_mode_text(mode, en)),
        );
    }
    let offering = |modes: Vec<DisplayMode>, active: Option<&str>| DisplayModesInfo {
        offers_display_modes: true,
        offered_modes: modes,
        active_mode: active.map(str::to_string),
        ..DisplayModesInfo::default()
    };
    let infos = [
        ("no info", None),
        ("not offering", Some(DisplayModesInfo::default())),
        (
            "offering, active",
            Some(offering(DisplayMode::ALL.to_vec(), Some("calm"))),
        ),
        (
            "offering, no active mode",
            Some(offering(vec![DisplayMode::Visual], None)),
        ),
        (
            "offering, nothing listed",
            Some(offering(vec![], Some("text"))),
        ),
    ];
    for (name, info) in infos {
        push(
            records,
            "offered_modes_text",
            name.to_string(),
            lang,
            json!(crate::display::offered_modes_text(info.as_ref(), en)),
        );
    }
}

/// Every record of the export, in both languages.
pub(crate) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    export_helpers(&mut records);
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_distribution(&mut records, lang);
        export_presentation(&mut records, lang, &i18n);
        export_display(&mut records, lang);
    }
    records
}
