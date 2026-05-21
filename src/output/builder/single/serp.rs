use crate::output::report_model::{PageHealthPresentation, SerpPresentation};

use super::super::helpers::yes_no;

pub(super) fn build_serp_presentation(s: &crate::seo::SerpAnalysis) -> SerpPresentation {
    let signals = s
        .signals
        .iter()
        .map(|sig| {
            (
                sig.category.clone(),
                sig.label.clone(),
                sig.status.label().to_string(),
                sig.detail.clone(),
            )
        })
        .collect();
    SerpPresentation {
        score: s.score,
        pass_count: s.pass_count,
        warning_count: s.warning_count,
        fail_count: s.fail_count,
        signals,
        rich_result_types: s.rich_result_types.clone(),
    }
}

pub(super) fn build_page_health_presentation(
    locale: &str,
    ph: &crate::seo::PageHealthAnalysis,
) -> PageHealthPresentation {
    let issues: Vec<(String, String, String)> = ph
        .issues
        .iter()
        .map(|i| (i.issue_type.clone(), i.message.clone(), i.severity.clone()))
        .collect();

    let mut url_info: Vec<(String, String)> = vec![
        (
            "URL-Länge".to_string(),
            format!("{} Zeichen", ph.url_length),
        ),
        ("Pfadtiefe".to_string(), ph.url_path_depth.to_string()),
        (
            "Query-Parameter".to_string(),
            yes_no(locale, ph.url_has_query_params),
        ),
        (
            "Eigene Weiterleitung".to_string(),
            yes_no(locale, ph.own_redirect_detected),
        ),
    ];
    if let Some(ref final_url) = ph.own_final_url {
        url_info.push(("Ziel-URL".to_string(), final_url.clone()));
    }

    let html_issues: Vec<(String, u32, String, String)> = ph
        .html_issues
        .iter()
        .map(|i| {
            (
                i.check.clone(),
                i.count,
                i.severity.clone(),
                i.detail.clone(),
            )
        })
        .collect();

    let html_validator = Some((
        match ph.html_validator_status.as_str() {
            "executed" => "Ausgeführt".to_string(),
            "failed" => "Fehlgeschlagen".to_string(),
            _ => "Übersprungen".to_string(),
        },
        ph.html_validator_detail
            .clone()
            .unwrap_or_else(|| "Keine Zusatzinformationen verfügbar".to_string()),
    ));

    let www_status = ph.www_consolidation.as_ref().map(|w| {
        let www_label = w
            .www_status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string());
        let non_www_label = w
            .non_www_status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string());
        (www_label, non_www_label, w.is_consolidated)
    });

    let soft_404 = ph.soft_404_status.map(|s| (s, ph.is_soft_404));

    let has_any_issue = !issues.is_empty() || !html_issues.is_empty();

    PageHealthPresentation {
        issues,
        url_info,
        html_issues,
        html_validator,
        www_status,
        soft_404,
        has_any_issue,
    }
}
