//! Miscellaneous helper utilities used across builder submodules.

use std::collections::HashSet;

use crate::audit::{BatchReport, NormalizedReport};
use crate::i18n::I18n;
use crate::output::report_model::{
    AffectedElement, AppendixViolation, BatchAppendixData, UrlAppendix,
};
use crate::util::truncate_url;

pub(super) fn build_technical_overview(locale: &str, normalized: &NormalizedReport) -> Vec<String> {
    let mut insights = Vec::new();

    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let total = normalized.severity_counts.total;
    let rule_count = normalized.findings.len();
    let en = locale == "en";

    // 1. Accessibility-Systematik — always present
    let a11y = if total == 0 {
        if en {
            "Accessibility pattern: No violations — fully conformant baseline".to_string()
        } else {
            "Accessibility-Systematik: Keine Verstöße — Basis vollständig konform".to_string()
        }
    } else if critical >= 5 && total > 30 {
        if en {
            format!("Accessibility pattern: Systematic patterns ({rule_count} rule types, {total} instances) — process problem, not a one-off")
        } else {
            format!("Accessibility-Systematik: Systematische Muster ({rule_count} Regeltypen, {total} Instanzen) — Prozess-Problem, kein Einzelfall")
        }
    } else if critical >= 3 || (critical >= 2 && rule_count >= 5) {
        if en {
            format!("Accessibility pattern: Multiple critical blockers ({critical} critical, {high} high) — direct screen-reader barriers")
        } else {
            format!("Accessibility-Systematik: Mehrere kritische Blockaden ({critical} kritisch, {high} hoch) — direkte Screenreader-Barrieren")
        }
    } else if total > 10 {
        if en {
            format!("Accessibility pattern: Distributed across {rule_count} rule types — no pattern, fixable individually")
        } else {
            format!("Accessibility-Systematik: Verteilt über {rule_count} Regeltypen — kein Muster, einzeln behebbar")
        }
    } else if en {
        format!("Accessibility pattern: {total} violations across {rule_count} areas — focused and fixable")
    } else {
        format!("Accessibility-Systematik: {total} Verstöße in {rule_count} Bereichen — konzentriert und gezielt behebbar")
    };
    insights.push(a11y);

    // 2. SEO-Level — always present
    let seo = if let Some(ref s) = normalized.raw_seo {
        if s.score >= 85 {
            if en {
                format!(
                    "SEO level: {} pts — technical ranking prerequisites met",
                    s.score
                )
            } else {
                format!(
                    "SEO-Level: {} Pkt — technische Ranking-Voraussetzungen erfüllt",
                    s.score
                )
            }
        } else if s.score >= 65 {
            if en {
                format!(
                    "SEO level: {} pts — base in place, targeted optimizations possible",
                    s.score
                )
            } else {
                format!(
                    "SEO-Level: {} Pkt — Basis vorhanden, gezielte Optimierungen möglich",
                    s.score
                )
            }
        } else if s.score >= 45 {
            if en {
                format!(
                    "SEO level: {} pts — relevant signals missing, visibility limited",
                    s.score
                )
            } else {
                format!(
                    "SEO-Level: {} Pkt — relevante Signale fehlen, Sichtbarkeit eingeschränkt",
                    s.score
                )
            }
        } else if en {
            format!(
                "SEO level: {} pts — structural base missing, ranking potential is severely limited",
                s.score
            )
        } else {
            format!(
                "SEO-Level: {} Pkt — strukturelle Basis fehlt, Ranking-Potenzial deutlich eingeschränkt",
                s.score
            )
        }
    } else if en {
        "SEO level: Not audited (use --full for full analysis)".to_string()
    } else {
        "SEO-Level: Nicht geprüft (--full für vollständige Analyse)".to_string()
    };
    insights.push(seo);

    // 3. Security-Level — always present
    let sec = if let Some(ref s) = normalized.raw_security {
        if s.score >= 80 {
            if en {
                format!(
                    "Security level: {} pts — HTTP security headers fully set",
                    s.score
                )
            } else {
                format!(
                    "Security-Level: {} Pkt — HTTP-Security-Header vollständig gesetzt",
                    s.score
                )
            }
        } else if s.score >= 55 {
            if en {
                format!("Security level: {} pts — basic protection in place, individual headers missing", s.score)
            } else {
                format!(
                    "Security-Level: {} Pkt — Grundschutz vorhanden, einzelne Header fehlen",
                    s.score
                )
            }
        } else if s.score >= 30 {
            if en {
                format!(
                    "Security level: {} pts — multiple critical security headers missing",
                    s.score
                )
            } else {
                format!(
                    "Security-Level: {} Pkt — mehrere kritische Security-Header fehlen",
                    s.score
                )
            }
        } else if en {
            format!("Security level: {} pts — security headers almost entirely missing — high risk, quick to fix", s.score)
        } else {
            format!("Security-Level: {} Pkt — Security-Header fehlen fast vollständig — hohes Risiko, schnell behebbar", s.score)
        }
    } else if en {
        "Security level: Not audited (use --full for full analysis)".to_string()
    } else {
        "Security-Level: Nicht geprüft (--full für vollständige Analyse)".to_string()
    };
    insights.push(sec);

    // 4. Tech-Komplexität — DOM + performance combined
    let dom = normalized.nodes_analyzed;
    let perf_score = normalized.raw_performance.as_ref().map(|p| p.score.overall);
    let tech = if en {
        match (dom, perf_score) {
            (d, Some(p)) if d > 2000 && p < 60 => format!(
                "Tech complexity: High — {d} DOM nodes, performance {p} pts — refactoring recommended"
            ),
            (d, Some(p)) if d > 2000 => format!(
                "Tech complexity: Medium-high — {d} DOM nodes (performance {p} pts stable)"
            ),
            (d, Some(p)) if p < 60 => format!(
                "Tech complexity: Performance critical ({p} pts) — {d} DOM nodes analyzed"
            ),
            (d, Some(p)) if p < 80 => format!(
                "Tech complexity: Low — {d} DOM nodes, performance can be optimized ({p} pts)"
            ),
            (d, Some(p)) => format!(
                "Tech complexity: Low — {d} DOM nodes, performance {p} pts — technical baseline stable"
            ),
            (d, None) if d > 2000 => format!(
                "Tech complexity: High — {d} DOM nodes (performance not audited)"
            ),
            (d, None) => format!(
                "Tech complexity: {d} DOM nodes analyzed (performance not audited, use --full)"
            ),
        }
    } else {
        match (dom, perf_score) {
            (d, Some(p)) if d > 2000 && p < 60 => format!(
                "Tech-Komplexität: Hoch — {d} DOM-Knoten, Performance {p} Pkt — Refactoring empfohlen"
            ),
            (d, Some(p)) if d > 2000 => format!(
                "Tech-Komplexität: Mittel-hoch — {d} DOM-Knoten (Performance {p} Pkt stabil)"
            ),
            (d, Some(p)) if p < 60 => format!(
                "Tech-Komplexität: Performance kritisch ({p} Pkt) — {d} DOM-Knoten analysiert"
            ),
            (d, Some(p)) if p < 80 => format!(
                "Tech-Komplexität: Gering — {d} DOM-Knoten, Performance optimierbar ({p} Pkt)"
            ),
            (d, Some(p)) => format!(
                "Tech-Komplexität: Gering — {d} DOM-Knoten, Performance {p} Pkt — technische Basis stabil"
            ),
            (d, None) if d > 2000 => format!(
                "Tech-Komplexität: Hoch — {d} DOM-Knoten (Performance nicht geprüft)"
            ),
            (d, None) => format!(
                "Tech-Komplexität: {d} DOM-Knoten analysiert (Performance nicht geprüft, --full)"
            ),
        }
    };
    insights.push(tech);

    insights
}

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

pub(super) fn build_trend_label(
    locale: &str,
    delta_accessibility: i32,
    delta_issues: i32,
) -> String {
    let en = locale == "en";
    if delta_accessibility >= 10 || (delta_accessibility >= 5 && delta_issues <= -5) {
        if en {
            "Significantly improved".to_string()
        } else {
            "Deutlich verbessert".to_string()
        }
    } else if delta_accessibility > 0 || delta_issues < 0 {
        if en {
            "Improved".to_string()
        } else {
            "Verbessert".to_string()
        }
    } else if delta_accessibility == 0 && delta_issues == 0 {
        if en {
            "Stable".to_string()
        } else {
            "Stabil".to_string()
        }
    } else if delta_accessibility >= -5 && delta_issues <= 5 {
        if en {
            "Slightly regressed".to_string()
        } else {
            "Leicht zurückgegangen".to_string()
        }
    } else if en {
        "Significantly regressed".to_string()
    } else {
        "Deutlich verschlechtert".to_string()
    }
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
pub(super) fn localized_module_name(name: &str, en: bool) -> String {
    if en {
        return name.to_string();
    }
    match name {
        "Accessibility" => "Barrierefreiheit",
        "Security" => "Sicherheit",
        other => other,
    }
    .to_string()
}

/// Module identity for `interpret_score`. Each variant carries its own,
/// per-module wording so the dashboard does not read as one shared template.
#[derive(Clone, Copy)]
pub(super) enum InterpretArea {
    Accessibility,
    Performance,
    Security,
    Mobile,
    Ux,
    Journey,
}

#[derive(Clone, Copy)]
enum ScoreBand {
    Excellent,
    Good,
    NeedsImprovement,
    Weak,
    Critical,
}

fn score_band(score: f32) -> ScoreBand {
    match score.round() as i64 {
        s if s >= 90 => ScoreBand::Excellent,
        s if s >= 75 => ScoreBand::Good,
        s if s >= 60 => ScoreBand::NeedsImprovement,
        s if s >= 40 => ScoreBand::Weak,
        _ => ScoreBand::Critical,
    }
}

/// Localized, module-specific score interpretation. Wording follows the
/// "Report Wording Style" rules in CLAUDE.md: describe Zustand + Auswirkung,
/// no filler phrases ("auf einem hohen Niveau"), no school grades
/// ("Befriedigend"), and real English in the `en` locale.
pub(super) fn interpret_score(area: InterpretArea, score: f32, locale: &str) -> String {
    use InterpretArea::*;
    use ScoreBand::*;

    let (de, en): (&str, &str) = match (area, score_band(score)) {
        (Accessibility, Excellent) => (
            "Sehr gut — die Barrierefreiheit ist technisch sauber umgesetzt und weist nur geringe Einschränkungen auf.",
            "Excellent — accessibility is implemented cleanly, with only minor limitations.",
        ),
        (Accessibility, Good) => (
            "Gut — die Barrierefreiheit ist insgesamt solide, kleinere Optimierungen sind sinnvoll.",
            "Good — accessibility is sound overall; minor improvements are worthwhile.",
        ),
        (Accessibility, NeedsImprovement) => (
            "Verbesserungswürdig — einzelne Barrieren können die Nutzung einschränken.",
            "Needs improvement — individual barriers can restrict use.",
        ),
        (Accessibility, Weak) => (
            "Ausbaufähig — relevante Barrieren beeinträchtigen Nutzbarkeit und Zugänglichkeit.",
            "Inadequate — significant barriers impair usability and accessibility.",
        ),
        (Accessibility, Critical) => (
            "Kritisch — wesentliche Anforderungen an die Barrierefreiheit werden nicht erfüllt.",
            "Critical — essential accessibility requirements are not met.",
        ),

        (Performance, Excellent) => (
            "Sehr gut — die Seite reagiert schnell und bietet eine flüssige Nutzererfahrung.",
            "Excellent — the page responds quickly and feels smooth to use.",
        ),
        (Performance, Good) => (
            "Gut — die Performance ist stabil, vereinzelt bestehen Optimierungsmöglichkeiten.",
            "Good — performance is stable, with occasional room for optimization.",
        ),
        (Performance, NeedsImprovement) => (
            "Verbesserungswürdig — Ladezeiten und Reaktionsverhalten sind stellenweise uneinheitlich.",
            "Needs improvement — load times and responsiveness are inconsistent in places.",
        ),
        (Performance, Weak) => (
            "Ausbaufähig — Performance-Probleme können Nutzung und Conversion beeinträchtigen.",
            "Inadequate — performance issues can impair use and conversion.",
        ),
        (Performance, Critical) => (
            "Kritisch — deutliche Performance-Probleme beeinträchtigen die Nutzererfahrung erheblich.",
            "Critical — significant performance problems severely impair the user experience.",
        ),

        (Security, Excellent) => (
            "Sehr gut — keine wesentlichen Sicherheitsauffälligkeiten im geprüften Umfang erkannt.",
            "Excellent — no significant security issues found within the scope checked.",
        ),
        (Security, Good) => (
            "Gut — grundlegende Sicherheitsmechanismen sind vorhanden, kleinere Schwächen wurden erkannt.",
            "Good — basic security mechanisms are in place; minor weaknesses were identified.",
        ),
        (Security, NeedsImprovement) => (
            "Verbesserungswürdig — einzelne Sicherheitsaspekte sollten überprüft und abgesichert werden.",
            "Needs improvement — individual security aspects should be reviewed and hardened.",
        ),
        (Security, Weak) => (
            "Ausbaufähig — relevante Sicherheitsauffälligkeiten oder Fehlkonfigurationen wurden erkannt.",
            "Inadequate — relevant security issues or misconfigurations were found.",
        ),
        (Security, Critical) => (
            "Kritisch — es bestehen erhebliche Sicherheitsrisiken mit unmittelbarem Handlungsbedarf.",
            "Critical — significant security risks exist that require immediate action.",
        ),

        (Mobile, Excellent) => (
            "Sehr gut — die Nutzung auf Mobilgeräten funktioniert zuverlässig und ohne erkennbare Einschränkungen.",
            "Excellent — the site works reliably on mobile devices, with no noticeable limitations.",
        ),
        (Mobile, Good) => (
            "Gut — die Nutzung auf Mobilgeräten funktioniert insgesamt zuverlässig.",
            "Good — the site works reliably on mobile devices overall.",
        ),
        (Mobile, NeedsImprovement) => (
            "Verbesserungswürdig — auf Mobilgeräten treten stellenweise Bedien- und Darstellungsprobleme auf.",
            "Needs improvement — layout and usability issues appear in places on mobile devices.",
        ),
        (Mobile, Weak) => (
            "Ausbaufähig — Darstellung und Bedienung auf Mobilgeräten sind spürbar eingeschränkt.",
            "Inadequate — layout and usability on mobile devices are noticeably impaired.",
        ),
        (Mobile, Critical) => (
            "Kritisch — die Seite ist auf Mobilgeräten kaum zuverlässig nutzbar.",
            "Critical — the site is barely usable on mobile devices.",
        ),

        (Ux, Excellent) => (
            "Sehr gut — die Bedienung ist klar und führt Nutzer sicher durch die Seite.",
            "Excellent — the interface is clear and guides users confidently through the page.",
        ),
        (Ux, Good) => (
            "Gut — die Nutzerführung ist verständlich, einzelne Abläufe lassen sich straffen.",
            "Good — user guidance is clear; individual flows can be tightened.",
        ),
        (Ux, NeedsImprovement) => (
            "Verbesserungswürdig — Nutzerführung und Interaktion wirken stellenweise unnötig komplex.",
            "Needs improvement — user guidance and interaction feel needlessly complex in places.",
        ),
        (Ux, Weak) => (
            "Ausbaufähig — Reibungspunkte erschweren eine klare Nutzerführung.",
            "Inadequate — friction points get in the way of clear user guidance.",
        ),
        (Ux, Critical) => (
            "Kritisch — die Bedienung ist unübersichtlich und behindert die Zielerreichung.",
            "Critical — the interface is confusing and prevents users from reaching their goal.",
        ),

        (Journey, Excellent) => (
            "Sehr gut — die wichtigsten Nutzerpfade sind durchgängig und nachvollziehbar.",
            "Excellent — the key user paths are consistent and easy to follow.",
        ),
        (Journey, Good) => (
            "Gut — die zentralen Nutzerpfade funktionieren, einzelne Schritte lassen sich verbessern.",
            "Good — the core user paths work; individual steps can be improved.",
        ),
        (Journey, NeedsImprovement) => (
            "Verbesserungswürdig — einzelne Schritte der Nutzerführung sind umständlich oder unklar.",
            "Needs improvement — individual steps in the user flow are cumbersome or unclear.",
        ),
        (Journey, Weak) => (
            "Ausbaufähig — Brüche in der Nutzerführung erschweren das Erreichen zentraler Ziele.",
            "Inadequate — breaks in the user flow make it harder to reach key goals.",
        ),
        (Journey, Critical) => (
            "Kritisch — wichtige Schritte der Nutzerführung sind unnötig kompliziert oder unterbrochen.",
            "Critical — important steps in the user flow are needlessly complicated or broken.",
        ),
    };

    if locale == "en" {
        en.to_string()
    } else {
        de.to_string()
    }
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
