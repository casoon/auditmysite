//! SERP pass — aggregates existing SEO signals into a search-result-page readiness check.
//!
//! No live search calls, no ML. Pure heuristics over already-collected data.

use serde::{Deserialize, Serialize};

use crate::seo::SeoAnalysis;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerpSignalStatus {
    Ok,
    Warning,
    Fail,
}

impl SerpSignalStatus {
    pub fn label(&self) -> &'static str {
        match self {
            SerpSignalStatus::Ok => "OK",
            SerpSignalStatus::Warning => "Warnung",
            SerpSignalStatus::Fail => "Fehler",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerpSignal {
    pub category: String,
    pub label: String,
    pub status: SerpSignalStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerpAnalysis {
    pub signals: Vec<SerpSignal>,
    pub score: u32,
    pub pass_count: u32,
    pub warning_count: u32,
    pub fail_count: u32,
    /// Rich result types eligible based on detected schema
    pub rich_result_types: Vec<String>,
}

pub fn build_serp_analysis(seo: &SeoAnalysis, url: &str) -> SerpAnalysis {
    let mut signals = Vec::new();

    // --- robots: noindex / nosnippet ---
    if let Some(robots_meta) = &seo.technical.robots_meta {
        let lower = robots_meta.to_ascii_lowercase();
        if lower.contains("noindex") {
            signals.push(sig(
                "Indexierung",
                "robots noindex",
                SerpSignalStatus::Fail,
                format!(
                    "Seite ist als noindex markiert und wird nicht indexiert ({})",
                    robots_meta
                ),
            ));
        }
        if lower.contains("nosnippet") {
            signals.push(sig(
                "Indexierung",
                "robots nosnippet",
                SerpSignalStatus::Warning,
                "nosnippet verhindert die Meta-Description im SERP-Eintrag.",
            ));
        }
    }

    // --- Title ---
    match &seo.meta.title {
        None => signals.push(sig(
            "Titel",
            "Titel vorhanden",
            SerpSignalStatus::Fail,
            "Kein <title>-Tag gefunden — Pflichtfeld für Google-Listeneinträge.",
        )),
        Some(title) => {
            let len = title.len();
            let status = if len < 30 || len > 60 {
                SerpSignalStatus::Warning
            } else {
                SerpSignalStatus::Ok
            };
            signals.push(sig(
                "Titel",
                "Titellänge",
                status,
                format!("{} Zeichen (empfohlen 30–60)", len),
            ));
            if len > 55 {
                signals.push(sig(
                    "Titel",
                    "Abschneidungsrisiko",
                    SerpSignalStatus::Warning,
                    format!(
                        "{} Zeichen — Titel wird in schmalen SERPs abgeschnitten (ca. 55 Zeichen sicher).",
                        len
                    ),
                ));
            }
        }
    }

    // --- Meta Description ---
    match &seo.meta.description {
        None => signals.push(sig(
            "Description",
            "Description vorhanden",
            SerpSignalStatus::Fail,
            "Keine Meta-Description — Google zeigt unkontrollierten Seitentext.",
        )),
        Some(desc) => {
            let len = desc.len();
            let status = if !(120..=160).contains(&len) {
                SerpSignalStatus::Warning
            } else {
                SerpSignalStatus::Ok
            };
            signals.push(sig(
                "Description",
                "Description-Länge",
                status,
                format!("{} Zeichen (empfohlen 120–160)", len),
            ));
        }
    }

    // --- Canonical ---
    signals.push(sig(
        "Canonical",
        "Canonical-Tag",
        if seo.technical.has_canonical {
            SerpSignalStatus::Ok
        } else {
            SerpSignalStatus::Warning
        },
        if seo.technical.has_canonical {
            seo.technical.canonical_url.clone().unwrap_or_default()
        } else {
            "Kein Canonical-Tag — Google wählt selbst die kanonische URL.".to_string()
        },
    ));

    if let Some(canon) = &seo.technical.canonical_url {
        let canon_clean = canon.trim_end_matches('/');
        let url_clean = url.trim_end_matches('/');
        if !canon_clean.is_empty() && !canon_clean.eq_ignore_ascii_case(url_clean) {
            signals.push(sig(
                "Canonical",
                "Canonical-Selbstreferenz",
                SerpSignalStatus::Warning,
                format!("Canonical zeigt auf eine andere URL: {}", canon),
            ));
        }
    }

    // --- Favicon ---
    signals.push(sig(
        "Erscheinungsbild",
        "Favicon",
        if seo.technical.has_favicon {
            SerpSignalStatus::Ok
        } else {
            SerpSignalStatus::Warning
        },
        if seo.technical.has_favicon {
            "Favicon vorhanden (erscheint in mobilen SERPs und Browser-Tabs)."
        } else {
            "Kein Favicon gefunden — Google zeigt in mobilen SERPs ein generisches Icon."
        },
    ));

    // --- Hreflang x-default ---
    if seo.technical.has_hreflang {
        let has_x_default = seo.technical.hreflang.iter().any(|h| h.lang == "x-default");
        signals.push(sig(
            "Internationalisierung",
            "hreflang x-default",
            if has_x_default {
                SerpSignalStatus::Ok
            } else {
                SerpSignalStatus::Warning
            },
            if has_x_default {
                format!(
                    "{} hreflang-Tags inkl. x-default",
                    seo.technical.hreflang.len()
                )
            } else {
                format!(
                    "{} hreflang-Tags, kein x-default — Google-Empfehlung für mehrsprachige Seiten.",
                    seo.technical.hreflang.len()
                )
            },
        ));
    }

    // --- Breadcrumb Schema ---
    {
        use crate::seo::SchemaType;
        let has_breadcrumb = seo
            .structured_data
            .types
            .contains(&SchemaType::BreadcrumbList);
        signals.push(sig(
            "Rich Result",
            "Breadcrumb-Schema",
            if has_breadcrumb {
                SerpSignalStatus::Ok
            } else {
                SerpSignalStatus::Warning
            },
            if has_breadcrumb {
                "BreadcrumbList erkannt — Google zeigt Pfadangabe im Listeneintrag."
            } else {
                "Kein BreadcrumbList-Schema — Pfadangabe im SERP-Eintrag nicht möglich."
            },
        ));
    }

    // --- Score ---
    let pass_count = signals
        .iter()
        .filter(|s| s.status == SerpSignalStatus::Ok)
        .count() as u32;
    let warning_count = signals
        .iter()
        .filter(|s| s.status == SerpSignalStatus::Warning)
        .count() as u32;
    let fail_count = signals
        .iter()
        .filter(|s| s.status == SerpSignalStatus::Fail)
        .count() as u32;
    let total = signals.len() as u32;
    let score = if total == 0 {
        100
    } else {
        ((pass_count * 100 + warning_count * 50) / total).min(100)
    };

    SerpAnalysis {
        signals,
        score,
        pass_count,
        warning_count,
        fail_count,
        rich_result_types: seo.structured_data.rich_snippets_potential.clone(),
    }
}

fn sig(
    category: impl Into<String>,
    label: impl Into<String>,
    status: SerpSignalStatus,
    detail: impl Into<String>,
) -> SerpSignal {
    SerpSignal {
        category: category.into(),
        label: label.into(),
        status,
        detail: detail.into(),
    }
}
