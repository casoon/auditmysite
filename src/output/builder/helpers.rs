//! Miscellaneous helper utilities used across builder submodules.

use std::collections::HashSet;

use crate::audit::{BatchReport, NormalizedReport};
use crate::i18n::I18n;
use crate::output::report_model::{
    AffectedElement, AppendixViolation, BatchAppendixData, UrlAppendix,
};
use crate::util::truncate_url;

pub(super) fn build_overall_impact(
    locale: &str,
    normalized: &NormalizedReport,
) -> Vec<(String, String)> {
    let score = normalized.score;
    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let urgent = critical + high;
    let en = locale == "en";

    let user_rating = if en {
        if score >= 90 && urgent == 0 {
            "Excellent — no relevant barriers"
        } else if score >= 75 {
            "Good — individual barriers for assistive technologies"
        } else if score >= 50 {
            "Limited — noticeable barriers for screen-reader users"
        } else {
            "Heavily limited — essential content not accessible"
        }
    } else if score >= 90 && urgent == 0 {
        "Sehr gut — keine relevanten Barrieren"
    } else if score >= 75 {
        "Gut — einzelne Barrieren für Hilfstechnologien"
    } else if score >= 50 {
        "Eingeschränkt — spürbare Barrieren für Screenreader-Nutzer"
    } else {
        "Stark eingeschränkt — wesentliche Inhalte nicht zugänglich"
    };

    let risk_level = if en {
        if critical >= 2 {
            "High — acute BITV/WCAG violation risk"
        } else if critical >= 1 || urgent >= 3 {
            "Medium — critical topics present"
        } else if score < 70 {
            "Medium — cumulative backlog"
        } else {
            "Low"
        }
    } else if critical >= 2 {
        "Hoch — BITV/WCAG-Verstoßrisiko akut"
    } else if critical >= 1 || urgent >= 3 {
        "Mittel — kritische Themen vorhanden"
    } else if score < 70 {
        "Mittel — kumulierter Nachholbedarf"
    } else {
        "Niedrig"
    };

    let conversion = if en {
        if score < 50 {
            "Likely negative"
        } else if score < 75 {
            "Possibly negative (navigation, forms)"
        } else {
            "Low — good usability"
        }
    } else if score < 50 {
        "Hoch wahrscheinlich negativ"
    } else if score < 75 {
        "Möglicherweise negativ (Navigation, Formulare)"
    } else {
        "Gering — gute Nutzbarkeit"
    };

    let (user_label, risk_label, conv_label) = if en {
        ("User experience", "Risk level", "Conversion effect")
    } else {
        ("Nutzererlebnis", "Risiko-Level", "Conversion-Effekt")
    };

    vec![
        (user_label.to_string(), user_rating.to_string()),
        (risk_label.to_string(), risk_level.to_string()),
        (conv_label.to_string(), conversion.to_string()),
    ]
}

pub(super) fn build_benchmark_context(locale: &str, score: f32) -> String {
    let en = locale == "en";
    if score >= 95.0 {
        if en {
            "Top 5% — exceptional level. No structural pressure to act.".to_string()
        } else {
            "Top 5% — Ausnahmeniveau. Kein struktureller Handlungsdruck.".to_string()
        }
    } else if score >= 90.0 {
        if en {
            "Top 15% — clearly above the majority. Polish is enough.".to_string()
        } else {
            "Top 15% — Deutlich besser als die Mehrheit. Feinschliff genügt.".to_string()
        }
    } else if score >= 80.0 {
        if en {
            "Upper third — good standing, individual optimizations pay off.".to_string()
        } else {
            "Oberes Drittel — Guter Stand, einzelne Optimierungen lohnen sich.".to_string()
        }
    } else if score >= 70.0 {
        if en {
            "Middle pack — improvement potential, no acute emergency.".to_string()
        } else {
            "Mittleres Feld — Verbesserungspotenzial vorhanden, kein akuter Notfall.".to_string()
        }
    } else if score >= 55.0 {
        if en {
            "Lower middle — clear gap to comparable websites.".to_string()
        } else {
            "Unteres Mittelfeld — Deutlicher Rückstand gegenüber vergleichbaren Websites."
                .to_string()
        }
    } else if score >= 40.0 {
        if en {
            "Lower third — significant gap, structural deficits common.".to_string()
        } else {
            "Unteres Drittel — Erheblicher Rückstand, strukturelle Defizite häufig.".to_string()
        }
    } else if en {
        "Critical — among the weakest audited sites. Immediate action required.".to_string()
    } else {
        "Kritisch — Zu den schwächsten geprüften Seiten. Sofortiger Handlungsbedarf.".to_string()
    }
}

pub(super) fn build_business_consequence(i18n: &I18n, normalized: &NormalizedReport) -> String {
    let critical = normalized.severity_counts.critical;
    let total = normalized.severity_counts.total;
    let score = normalized.score;

    if total == 0 {
        return i18n.t("business-consequence-clean");
    }

    let has_weak_seo = normalized.raw_seo.as_ref().is_some_and(|s| s.score < 65);
    let has_heading_issues = normalized.findings.iter().any(|f| {
        f.rule_id.to_lowercase().contains("heading")
            || f.title.to_lowercase().contains("überschrift")
    });

    let key = if score < 50 || (critical >= 5 && total > 30) {
        "business-consequence-severe"
    } else if has_weak_seo && has_heading_issues {
        "business-consequence-seo-headings"
    } else if critical >= 2 {
        "business-consequence-screenreader"
    } else {
        "business-consequence-default"
    };
    i18n.t(key)
}

pub(super) fn build_consequence_text(i18n: &I18n, normalized: &NormalizedReport) -> String {
    let critical = normalized.severity_counts.critical;
    let total = normalized.severity_counts.total;
    let score = normalized.score;

    if total == 0 {
        return String::new();
    }

    let weak_module_count = [
        normalized
            .raw_security
            .as_ref()
            .is_some_and(|s| s.score < 60),
        normalized.raw_seo.as_ref().is_some_and(|s| s.score < 60),
        normalized
            .raw_performance
            .as_ref()
            .is_some_and(|p| p.score.overall < 70),
        normalized.raw_mobile.as_ref().is_some_and(|m| m.score < 65),
    ]
    .iter()
    .filter(|&&v| v)
    .count();

    let key = if score < 50 || (critical >= 5 && total > 30) {
        "consequence-severe"
    } else if critical >= 3 || weak_module_count >= 3 {
        "consequence-many-weak-modules"
    } else if score >= 85 {
        "consequence-stable"
    } else {
        "consequence-default"
    };
    i18n.t(key)
}

pub(super) fn localized_report_title(locale: &str) -> String {
    match locale {
        "en" => "Accessibility Audit Report".to_string(),
        _ => "Barrierefreiheits-Prüfbericht".to_string(),
    }
}

pub(super) fn localized_report_subtitle(locale: &str) -> &'static str {
    match locale {
        "en" => "Automated accessibility audit with optional website quality modules.",
        _ => "Automatisierter Accessibility-Report mit ergänzenden Qualitätsmodulen.",
    }
}

pub(super) fn build_verdict_text(i18n: &I18n, url: &str, score: f32) -> String {
    let key = if score >= 90.0 {
        "verdict-tier-excellent"
    } else if score >= 70.0 {
        "verdict-tier-solid"
    } else if score >= 50.0 {
        "verdict-tier-deficient"
    } else {
        "verdict-tier-critical"
    };
    i18n.t_args(
        key,
        &[("url", url.to_string()), ("score", format!("{:.0}", score))],
    )
}

pub(super) fn build_score_note(i18n: &I18n, normalized: &NormalizedReport) -> Option<String> {
    let critical_topics = normalized.severity_counts.critical + normalized.severity_counts.high;
    if normalized.score >= 90 && critical_topics > 0 {
        Some(i18n.t("score-note-high-with-critical"))
    } else {
        None
    }
}

pub(super) fn build_batch_verdict(i18n: &I18n, total_urls: usize, overall_score: u32) -> String {
    let key = if overall_score >= 90 {
        "verdict-batch-excellent"
    } else if overall_score >= 70 {
        "verdict-batch-solid"
    } else if overall_score >= 50 {
        "verdict-batch-deficient"
    } else {
        "verdict-batch-critical"
    };
    i18n.t_args(
        key,
        &[
            ("total_urls", total_urls.to_string()),
            ("score", overall_score.to_string()),
        ],
    )
}

pub(super) fn build_batch_appendix(batch: &BatchReport) -> BatchAppendixData {
    BatchAppendixData {
        per_url: batch
            .reports
            .iter()
            .map(|r| {
                let normalized = crate::audit::normalize(r);

                UrlAppendix {
                    url: r.url.clone(),
                    violations: normalized
                        .findings
                        .iter()
                        .map(|finding| AppendixViolation {
                            rule: finding.rule_id.clone(),
                            rule_name: finding.title.clone(),
                            severity: finding.severity,
                            message: finding.description.clone(),
                            fix_suggestion: finding
                                .occurrences
                                .iter()
                                .find_map(|occ| occ.fix_suggestion.clone()),
                            affected_elements: finding
                                .occurrences
                                .iter()
                                .map(|occ| AffectedElement {
                                    selector: occ
                                        .selector
                                        .clone()
                                        .unwrap_or_else(|| occ.node_id.clone()),
                                    node_id: occ.node_id.clone(),
                                })
                                .collect(),
                        })
                        .collect(),
                }
            })
            .collect(),
    }
}

pub(super) fn yes_no(locale: &str, val: bool) -> String {
    match (locale, val) {
        ("en", true) => "Yes".to_string(),
        ("en", false) => "No".to_string(),
        (_, true) => "Ja".to_string(),
        (_, false) => "Nein".to_string(),
    }
}

pub(super) fn truncate_list(items: &[String], limit: usize) -> String {
    let mut values: Vec<String> = items
        .iter()
        .filter(|item| !item.trim().is_empty())
        .cloned()
        .collect();
    values.sort();
    values.dedup();

    let shown: Vec<String> = values.iter().take(limit).cloned().collect();
    if values.len() > limit {
        format!("{} +{}", shown.join(", "), values.len() - limit)
    } else {
        shown.join(", ")
    }
}

pub(super) fn truncate_url_list(items: &[String], limit: usize, max_len: usize) -> String {
    let shortened: Vec<String> = items
        .iter()
        .map(|item| truncate_url(item, max_len))
        .collect();
    truncate_list(&shortened, limit)
}

pub(crate) fn extract_domain(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = without_scheme.split('/').next().unwrap_or(without_scheme);
    host.trim_start_matches("www.").to_string()
}

/// German display name for a module identifier used in prose (overall-score
/// weights, indicator notes). English keeps the canonical identifier.
pub(super) fn localized_module_name(name: &str, i18n: &I18n) -> String {
    let key = format!("module-{}", name.to_lowercase());
    let translated = i18n.t(&key);
    if translated == key {
        // Fallback to name if not found in Fluent
        name.to_string()
    } else {
        translated
    }
}

/// Module identity for `interpret_score`. Re-exported from `audit::interpretation`
/// so existing builder call sites need no import changes.
pub(super) use crate::audit::interpretation::InterpretArea;

/// Localized, module-specific score interpretation.
/// Delegates to `audit::interpretation::interpret_score_localized` and selects
/// the requested locale so existing call sites need no changes.
pub(super) fn interpret_score(area: InterpretArea, score: f32, locale: &str) -> String {
    let text = crate::audit::interpretation::interpret_score_localized(area, score);
    text.for_locale(locale).to_string()
}

pub(super) fn normalize_topic_token(token: &str) -> String {
    token
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
        .replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ü", "ue")
        .replace("ß", "ss")
}

pub(super) fn german_stopwords() -> HashSet<&'static str> {
    [
        "2026",
        "aber",
        "allem",
        "alle",
        "auch",
        "auf",
        "aus",
        "autor",
        "bei",
        "bereits",
        "bietet",
        "bild",
        "bilder",
        "casoon",
        "checker",
        "cloud",
        "content",
        "damit",
        "dass",
        "deine",
        "diese",
        "dieser",
        "drei",
        "durch",
        "eine",
        "einem",
        "einen",
        "einer",
        "eines",
        "einfach",
        "entwickelt",
        "entwicklung",
        "erfahren",
        "fuer",
        "für",
        "gmbh",
        "heute",
        "hier",
        "ihre",
        "ihren",
        "ihrer",
        "ihres",
        "inklusive",
        "inhalt",
        "jetzt",
        "keine",
        "kunden",
        "launch",
        "lesen",
        "mehr",
        "moderne",
        "klare",
        "oder",
        "page",
        "pages",
        "projekt",
        "projekten",
        "recht",
        "rund",
        "seite",
        "seiten",
        "seine",
        "seiner",
        "sich",
        "sind",
        "site",
        "statt",
        "systeme",
        "technik",
        "themen",
        "thema",
        "über",
        "und",
        "unsere",
        "unserer",
        "unsers",
        "unter",
        "transparent",
        "viele",
        "vom",
        "von",
        "web",
        "websites",
        "webentwicklung",
        "website",
        "weiter",
        "werden",
        "wird",
        "wenig",
        "willkommen",
        "zeigen",
        "ziel",
        "with",
        "your",
        "about",
        "into",
        "that",
        "this",
        "from",
        "haben",
        "sowie",
        "digitale",
    ]
    .into_iter()
    .collect()
}
