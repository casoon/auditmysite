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
    pub fn label(&self, en: bool) -> &'static str {
        match self {
            SerpSignalStatus::Ok => "OK",
            SerpSignalStatus::Warning => {
                if en {
                    "Warning"
                } else {
                    "Warnung"
                }
            }
            SerpSignalStatus::Fail => {
                if en {
                    "Error"
                } else {
                    "Fehler"
                }
            }
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
    /// Rich-result-related schema types detected; this is not an eligibility claim.
    pub rich_result_types: Vec<String>,
}

pub fn build_serp_analysis(seo: &SeoAnalysis, url: &str, locale: &str) -> SerpAnalysis {
    let en = locale == "en";
    let mut signals = Vec::new();

    // --- robots: noindex / nosnippet ---
    if let Some(robots_meta) = &seo.technical.robots_meta {
        let lower = robots_meta.to_ascii_lowercase();
        if lower.contains("noindex") {
            signals.push(sig(
                if en { "Indexing" } else { "Indexierung" },
                "robots noindex",
                SerpSignalStatus::Fail,
                if en {
                    format!(
                        "Page is marked as noindex and is not indexed ({})",
                        robots_meta
                    )
                } else {
                    format!(
                        "Seite ist als noindex markiert und wird nicht indexiert ({})",
                        robots_meta
                    )
                },
            ));
        }
        if lower.contains("nosnippet") {
            signals.push(sig(
                if en { "Indexing" } else { "Indexierung" },
                "robots nosnippet",
                SerpSignalStatus::Warning,
                if en {
                    "nosnippet prevents the meta description from appearing in the SERP entry."
                } else {
                    "nosnippet verhindert die Meta-Description im SERP-Eintrag."
                },
            ));
        }
    }

    // --- Title ---
    match &seo.meta.title {
        None => signals.push(sig(
            if en { "Title" } else { "Titel" },
            if en {
                "Title present"
            } else {
                "Titel vorhanden"
            },
            SerpSignalStatus::Fail,
            if en {
                "No <title> tag found — required field for Google listings."
            } else {
                "Kein <title>-Tag gefunden — Pflichtfeld für Google-Listeneinträge."
            },
        )),
        Some(title) => {
            let len = title.len();
            let status = if !(30..=60).contains(&len) {
                SerpSignalStatus::Warning
            } else {
                SerpSignalStatus::Ok
            };
            signals.push(sig(
                if en { "Title" } else { "Titel" },
                if en { "Title length" } else { "Titellänge" },
                status,
                if en {
                    format!("{} characters (recommended 30–60)", len)
                } else {
                    format!("{} Zeichen (empfohlen 30–60)", len)
                },
            ));
            if len > 55 {
                signals.push(sig(
                    if en { "Title" } else { "Titel" },
                    if en {
                        "Truncation risk"
                    } else {
                        "Abschneidungsrisiko"
                    },
                    SerpSignalStatus::Warning,
                    if en {
                        format!(
                            "{} characters — title gets truncated in narrow SERPs (about 55 characters is safe).",
                            len
                        )
                    } else {
                        format!(
                            "{} Zeichen — Titel wird in schmalen SERPs abgeschnitten (ca. 55 Zeichen sicher).",
                            len
                        )
                    },
                ));
            }
        }
    }

    // --- Meta Description ---
    match &seo.meta.description {
        None => signals.push(sig(
            "Description",
            if en {
                "Description present"
            } else {
                "Description vorhanden"
            },
            SerpSignalStatus::Fail,
            if en {
                "No meta description — Google shows uncontrolled page text."
            } else {
                "Keine Meta-Description — Google zeigt unkontrollierten Seitentext."
            },
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
                if en {
                    "Description length"
                } else {
                    "Description-Länge"
                },
                status,
                if en {
                    format!("{} characters (recommended 120–160)", len)
                } else {
                    format!("{} Zeichen (empfohlen 120–160)", len)
                },
            ));
        }
    }

    // --- Canonical ---
    signals.push(sig(
        "Canonical",
        if en { "Canonical tag" } else { "Canonical-Tag" },
        if seo.technical.has_canonical {
            SerpSignalStatus::Ok
        } else {
            SerpSignalStatus::Warning
        },
        if seo.technical.has_canonical {
            seo.technical.canonical_url.clone().unwrap_or_default()
        } else if en {
            "No canonical tag — Google picks the canonical URL itself.".to_string()
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
                if en {
                    "Canonical self-reference"
                } else {
                    "Canonical-Selbstreferenz"
                },
                SerpSignalStatus::Warning,
                if en {
                    format!("Canonical points to a different URL: {}", canon)
                } else {
                    format!("Canonical zeigt auf eine andere URL: {}", canon)
                },
            ));
        }
    }

    // --- Favicon ---
    signals.push(sig(
        if en { "Appearance" } else { "Erscheinungsbild" },
        "Favicon",
        if seo.technical.has_favicon {
            SerpSignalStatus::Ok
        } else {
            SerpSignalStatus::Warning
        },
        if seo.technical.has_favicon {
            if en {
                "Favicon present (appears in mobile SERPs and browser tabs)."
            } else {
                "Favicon vorhanden (erscheint in mobilen SERPs und Browser-Tabs)."
            }
        } else if en {
            "No favicon found — Google shows a generic icon in mobile SERPs."
        } else {
            "Kein Favicon gefunden — Google zeigt in mobilen SERPs ein generisches Icon."
        },
    ));

    // --- Hreflang x-default ---
    if seo.technical.has_hreflang {
        let has_x_default = seo.technical.hreflang.iter().any(|h| h.lang == "x-default");
        signals.push(sig(
            if en {
                "Internationalisation"
            } else {
                "Internationalisierung"
            },
            "hreflang x-default",
            if has_x_default {
                SerpSignalStatus::Ok
            } else {
                SerpSignalStatus::Warning
            },
            if has_x_default {
                if en {
                    format!(
                        "{} hreflang tags incl. x-default",
                        seo.technical.hreflang.len()
                    )
                } else {
                    format!(
                        "{} hreflang-Tags inkl. x-default",
                        seo.technical.hreflang.len()
                    )
                }
            } else if en {
                format!(
                    "{} hreflang tags, no x-default — Google recommendation for multilingual pages.",
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
        use crate::seo::{SchemaRequirementStatus, SchemaType};
        let has_breadcrumb = seo
            .structured_data
            .types
            .contains(&SchemaType::BreadcrumbList);
        // Type presence alone never promises a SERP path display: the same
        // report validates BreadcrumbList's required properties, and an
        // incomplete node is not eligible. Reporting "Google shows the path"
        // next to "required properties missing" contradicted itself.
        let incomplete: Vec<&str> = seo
            .structured_data
            .rule_assessments
            .iter()
            .filter(|a| {
                a.schema_type == "BreadcrumbList"
                    && a.requirement_status == SchemaRequirementStatus::MissingRequiredProperties
            })
            .flat_map(|a| a.missing_required.iter().map(String::as_str))
            .collect();

        let (status, detail) = match (has_breadcrumb, incomplete.is_empty()) {
            (true, true) => (
                SerpSignalStatus::Ok,
                if en {
                    "BreadcrumbList detected, required properties met — eligible for a path display.".to_string()
                } else {
                    "BreadcrumbList erkannt, Pflichtangaben erfüllt — für eine Pfadangabe grundsätzlich geeignet.".to_string()
                },
            ),
            (true, false) => (
                SerpSignalStatus::Warning,
                if en {
                    format!(
                        "BreadcrumbList detected but incomplete ({}) — not eligible for a path display.",
                        incomplete.join(", ")
                    )
                } else {
                    format!(
                        "BreadcrumbList erkannt, aber unvollständig ({}) — für eine Pfadangabe nicht geeignet.",
                        incomplete.join(", ")
                    )
                },
            ),
            (false, _) => (
                SerpSignalStatus::Warning,
                if en {
                    "No BreadcrumbList schema — path display in the SERP entry not possible."
                        .to_string()
                } else {
                    "Kein BreadcrumbList-Schema — Pfadangabe im SERP-Eintrag nicht möglich."
                        .to_string()
                },
            ),
        };
        signals.push(sig(
            "Rich Result",
            if en {
                "Breadcrumb schema"
            } else {
                "Breadcrumb-Schema"
            },
            status,
            detail,
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
    let score = (pass_count * 100 + warning_count * 50)
        .checked_div(total)
        .unwrap_or(100)
        .min(100);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seo::technical::HreflangTag;
    use crate::seo::SchemaType;

    fn has_german_chars(s: &str) -> bool {
        s.chars().any(|c| "äöüÄÖÜß".contains(c))
    }

    fn breadcrumb_assessment(
        status: crate::seo::SchemaRequirementStatus,
        missing: &[&str],
    ) -> crate::seo::SchemaRuleAssessment {
        crate::seo::SchemaRuleAssessment {
            node_index: 0,
            schema_type: "BreadcrumbList".to_string(),
            feature: crate::seo::SchemaFeature::Breadcrumb,
            availability: crate::seo::SchemaFeatureAvailability::General,
            requirement_status: status,
            missing_required: missing.iter().map(|s| s.to_string()).collect(),
            missing_recommended: Vec::new(),
            manual_review: Vec::new(),
            quality_score: None,
            source_url: String::new(),
            reviewed_at: String::new(),
        }
    }

    fn breadcrumb_signal(seo: &SeoAnalysis) -> SerpSignal {
        build_serp_analysis(seo, "https://example.com/", "de")
            .signals
            .into_iter()
            .find(|s| s.label == "Breadcrumb-Schema")
            .expect("breadcrumb signal is always emitted")
    }

    /// Regression: the signal only checked whether the *type* was present, so
    /// a BreadcrumbList missing `itemListElement` was reported as "Google
    /// shows the path" on one page while the schema table on another page of
    /// the same report said "required properties missing".
    #[test]
    fn incomplete_breadcrumb_schema_is_not_reported_as_eligible() {
        let mut seo = SeoAnalysis::default();
        seo.structured_data.types.push(SchemaType::BreadcrumbList);
        seo.structured_data
            .rule_assessments
            .push(breadcrumb_assessment(
                crate::seo::SchemaRequirementStatus::MissingRequiredProperties,
                &["itemListElement (minimum 2 entries)"],
            ));

        let signal = breadcrumb_signal(&seo);
        assert_eq!(signal.status, SerpSignalStatus::Warning);
        assert!(signal.detail.contains("itemListElement"), "{signal:?}");
    }

    #[test]
    fn complete_breadcrumb_schema_stays_eligible() {
        let mut seo = SeoAnalysis::default();
        seo.structured_data.types.push(SchemaType::BreadcrumbList);
        seo.structured_data
            .rule_assessments
            .push(breadcrumb_assessment(
                crate::seo::SchemaRequirementStatus::MeetsRequiredProperties,
                &[],
            ));

        assert_eq!(breadcrumb_signal(&seo).status, SerpSignalStatus::Ok);
    }

    #[test]
    fn test_serp_signals_have_no_german_chars_in_en() {
        // Populate technical + meta so every conditional branch that produces a
        // visible label/detail string is exercised.
        let mut seo = SeoAnalysis::default();
        seo.meta.title =
            Some("A title long enough to trip the truncation risk branch here".to_string());
        seo.meta.description = Some("short".to_string());
        seo.technical.robots_meta = Some("noindex, nosnippet".to_string());
        seo.technical.has_canonical = true;
        seo.technical.canonical_url = Some("https://example.com/other".to_string());
        seo.technical.has_favicon = false;
        seo.technical.has_hreflang = true;
        seo.technical.hreflang = vec![HreflangTag {
            lang: "de".to_string(),
            url: "https://example.com/de".to_string(),
        }];
        seo.structured_data.types = vec![SchemaType::Article];

        let analysis = build_serp_analysis(&seo, "https://example.com/", "en");
        for signal in &analysis.signals {
            assert!(
                !has_german_chars(&signal.category),
                "German chars in category: {}",
                signal.category
            );
            assert!(
                !has_german_chars(&signal.label),
                "German chars in label: {}",
                signal.label
            );
            assert!(
                !has_german_chars(&signal.detail),
                "German chars in detail: {}",
                signal.detail
            );
        }
    }
}
