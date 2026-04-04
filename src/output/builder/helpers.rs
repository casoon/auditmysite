//! Miscellaneous helper utilities used across builder submodules.

use std::collections::HashSet;

use crate::audit::{BatchReport, NormalizedReport};
use crate::output::report_model::{
    AffectedElement, AppendixViolation, BatchAppendixData, UrlAppendix,
};
use crate::util::truncate_url;

pub(super) fn build_technical_overview(normalized: &NormalizedReport) -> Vec<String> {
    let mut insights = Vec::new();

    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let total = normalized.severity_counts.total;
    let rule_count = normalized.findings.len();

    // 1. Accessibility-Systematik — always present
    let a11y = if total == 0 {
        "Accessibility-Systematik: Keine Verstöße — Basis vollständig konform".to_string()
    } else if critical >= 5 && total > 30 {
        format!("Accessibility-Systematik: Systematische Muster ({rule_count} Regeltypen, {total} Instanzen) — Prozess-Problem, kein Einzelfall")
    } else if critical >= 3 || (critical >= 2 && rule_count >= 5) {
        format!("Accessibility-Systematik: Mehrere kritische Blockaden ({critical} kritisch, {high} hoch) — direkte Screenreader-Barrieren")
    } else if total > 10 {
        format!("Accessibility-Systematik: Verteilt über {rule_count} Regeltypen — kein Muster, einzeln behebbar")
    } else {
        format!("Accessibility-Systematik: {total} Verstöße in {rule_count} Bereichen — konzentriert und gezielt behebbar")
    };
    insights.push(a11y);

    // 2. SEO-Level — always present
    let seo = if let Some(ref s) = normalized.raw_seo {
        if s.score >= 85 {
            format!(
                "SEO-Level: {} Pkt — technische Ranking-Voraussetzungen erfüllt",
                s.score
            )
        } else if s.score >= 65 {
            format!(
                "SEO-Level: {} Pkt — Basis vorhanden, gezielte Optimierungen möglich",
                s.score
            )
        } else if s.score >= 45 {
            format!(
                "SEO-Level: {} Pkt — relevante Signale fehlen, Sichtbarkeit eingeschränkt",
                s.score
            )
        } else {
            format!(
                "SEO-Level: {} Pkt — strukturelle Basis fehlt, Ranking praktisch ausgeschlossen",
                s.score
            )
        }
    } else {
        "SEO-Level: Nicht geprüft (--full für vollständige Analyse)".to_string()
    };
    insights.push(seo);

    // 3. Security-Level — always present
    let sec = if let Some(ref s) = normalized.raw_security {
        if s.score >= 80 {
            format!(
                "Security-Level: {} Pkt — HTTP-Security-Header vollständig gesetzt",
                s.score
            )
        } else if s.score >= 55 {
            format!(
                "Security-Level: {} Pkt — Grundschutz vorhanden, einzelne Header fehlen",
                s.score
            )
        } else if s.score >= 30 {
            format!(
                "Security-Level: {} Pkt — mehrere kritische Security-Header fehlen",
                s.score
            )
        } else {
            format!("Security-Level: {} Pkt — Security-Header fehlen fast vollständig — hohes Risiko, schnell behebbar", s.score)
        }
    } else {
        "Security-Level: Nicht geprüft (--full für vollständige Analyse)".to_string()
    };
    insights.push(sec);

    // 4. Tech-Komplexität — DOM + performance combined
    let dom = normalized.nodes_analyzed;
    let perf_score = normalized.raw_performance.as_ref().map(|p| p.score.overall);
    let tech = match (dom, perf_score) {
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
    };
    insights.push(tech);

    insights
}

pub(super) fn build_overall_impact(normalized: &NormalizedReport) -> Vec<(String, String)> {
    let score = normalized.score;
    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let urgent = critical + high;

    let user_rating = if score >= 90 && urgent == 0 {
        "Sehr gut — keine relevanten Barrieren"
    } else if score >= 75 {
        "Gut — einzelne Barrieren für Hilfstechnologien"
    } else if score >= 50 {
        "Eingeschränkt — spürbare Barrieren für Screenreader-Nutzer"
    } else {
        "Stark eingeschränkt — wesentliche Inhalte nicht zugänglich"
    };

    let risk_level = if critical >= 2 {
        "Hoch — BITV/WCAG-Verstoßrisiko akut"
    } else if critical >= 1 || urgent >= 3 {
        "Mittel — kritische Themen vorhanden"
    } else if score < 70 {
        "Mittel — kumulierter Nachholbedarf"
    } else {
        "Niedrig"
    };

    let conversion = if score < 50 {
        "Hoch wahrscheinlich negativ"
    } else if score < 75 {
        "Möglicherweise negativ (Navigation, Formulare)"
    } else {
        "Gering — gute Nutzbarkeit"
    };

    vec![
        ("Nutzererlebnis".to_string(), user_rating.to_string()),
        ("Risiko-Level".to_string(), risk_level.to_string()),
        ("Conversion-Effekt".to_string(), conversion.to_string()),
    ]
}

pub(super) fn build_trend_label(delta_accessibility: i32, delta_issues: i32) -> String {
    if delta_accessibility >= 10 || (delta_accessibility >= 5 && delta_issues <= -5) {
        "Deutlich verbessert".to_string()
    } else if delta_accessibility > 0 || delta_issues < 0 {
        "Verbessert".to_string()
    } else if delta_accessibility == 0 && delta_issues == 0 {
        "Stabil".to_string()
    } else if delta_accessibility >= -5 && delta_issues <= 5 {
        "Leicht zurückgegangen".to_string()
    } else {
        "Deutlich verschlechtert".to_string()
    }
}

pub(super) fn build_benchmark_context(score: f32) -> String {
    if score >= 95.0 {
        "Top 5% — Ausnahmeniveau. Kein struktureller Handlungsdruck.".to_string()
    } else if score >= 90.0 {
        "Top 15% — Deutlich besser als die Mehrheit. Feinschliff genügt.".to_string()
    } else if score >= 80.0 {
        "Oberes Drittel — Guter Stand, einzelne Optimierungen lohnen sich.".to_string()
    } else if score >= 70.0 {
        "Mittleres Feld — Verbesserungspotenzial vorhanden, kein akuter Notfall.".to_string()
    } else if score >= 55.0 {
        "Unteres Mittelfeld — Deutlicher Rückstand gegenüber vergleichbaren Websites.".to_string()
    } else if score >= 40.0 {
        "Unteres Drittel — Erheblicher Rückstand, strukturelle Defizite häufig.".to_string()
    } else {
        "Kritisch — Zu den schwächsten geprüften Seiten. Sofortiger Handlungsbedarf.".to_string()
    }
}

pub(super) fn build_business_consequence(normalized: &NormalizedReport) -> String {
    let critical = normalized.severity_counts.critical;
    let total = normalized.severity_counts.total;
    let score = normalized.score;

    if total == 0 {
        return "Keine bekannten Barrieren — gutes Fundament für alle Nutzergruppen.".to_string();
    }

    let has_weak_seo = normalized.raw_seo.as_ref().is_some_and(|s| s.score < 65);
    let has_heading_issues = normalized.findings.iter().any(|f| {
        f.rule_id.to_lowercase().contains("heading")
            || f.title.to_lowercase().contains("überschrift")
    });

    if score < 50 || (critical >= 5 && total > 30) {
        "Weite Teile der Seite sind für bestimmte Nutzergruppen nicht oder kaum nutzbar."
            .to_string()
    } else if has_weak_seo && has_heading_issues {
        "Seite wird schlechter gefunden und ist für Teile der Nutzer strukturell nicht zugänglich."
            .to_string()
    } else if critical >= 2 {
        "Einzelne Kernfunktionen sind für Screenreader-Nutzer blockiert oder fehleranfällig."
            .to_string()
    } else {
        "Nutzbarkeit ist gegeben — gezielte Verbesserungen erhöhen Qualität und Reichweite."
            .to_string()
    }
}

pub(super) fn build_consequence_text(normalized: &NormalizedReport) -> String {
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

    if score < 50 || (critical >= 5 && total > 30) {
        "Neue Inhalte und Funktionen erben die bestehenden Fehler — Korrekturaufwand wächst mit jeder Erweiterung.".to_string()
    } else if critical >= 3 || weak_module_count >= 3 {
        "Aufwand für spätere Korrekturen steigt deutlich — besonders bei Relaunch oder größerem Content-Ausbau.".to_string()
    } else if score >= 85 {
        "Kein akuter Handlungsdruck. Regelmäßige Checks sichern das Niveau nach Updates und Erweiterungen.".to_string()
    } else {
        "Ohne Korrektur bleibt die Seite hinter erreichbarem Standard — Verbesserungspotenzial wird nicht genutzt.".to_string()
    }
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

pub(super) fn build_verdict_text(url: &str, score: f32) -> String {
    if score >= 90.0 {
        format!(
            "{url} erreicht {score:.0}/100 im Accessibility-Audit. \
             Die verbleibenden Findings sind letzte Optimierungshebel — kein strukturelles Problem, sondern Feinschliff.",
        )
    } else if score >= 70.0 {
        format!(
            "{url} erreicht {score:.0}/100 im Accessibility-Audit. \
             Die Basis ist solide — klarer Verbesserungshebel mit überschaubarem Aufwand.",
        )
    } else if score >= 50.0 {
        format!(
            "{url} erreicht {score:.0}/100 im Accessibility-Audit. \
             Es bestehen deutliche Barrieren — nicht nur Detailprobleme, sondern struktureller Nachholbedarf.",
        )
    } else {
        format!(
            "{url} erreicht nur {score:.0}/100 im Accessibility-Audit. \
             Akuter Handlungsbedarf: Wesentliche Inhalte und Funktionen sind für einen Teil der Nutzer nicht zugänglich.",
        )
    }
}

pub(super) fn build_score_note(normalized: &NormalizedReport) -> Option<String> {
    let critical_topics = normalized.severity_counts.critical + normalized.severity_counts.high;
    if normalized.score >= 90 && critical_topics > 0 {
        Some(
            "Der Score berücksichtigt Gewichtung und Häufigkeit. Einzelne kritische Themen können trotz hoher Gesamtbewertung bestehen."
                .to_string(),
        )
    } else {
        None
    }
}

pub(super) fn build_batch_verdict(batch: &BatchReport) -> String {
    let avg = batch.summary.average_score;
    if avg >= 90.0 {
        format!(
            "Über {} geprüfte URLs hinweg erreicht die Website einen durchschnittlichen \
                 Accessibility-Score von {:.0}/100 — ein sehr gutes Ergebnis.",
            batch.summary.total_urls, avg
        )
    } else if avg >= 70.0 {
        format!(
            "Im Durchschnitt erreichen die {} geprüften URLs {:.0}/100 Punkte. \
                 Die Basis ist solide, es bestehen aber wiederkehrende Barrieren.",
            batch.summary.total_urls, avg
        )
    } else if avg >= 50.0 {
        format!(
            "Die {} geprüften URLs erreichen im Schnitt nur {:.0}/100 Punkte. \
                 Es bestehen erhebliche systematische Barrierefreiheitsprobleme.",
            batch.summary.total_urls, avg
        )
    } else {
        format!(
            "Die {} geprüften URLs erreichen im Schnitt nur {:.0}/100 Punkte. \
                 Die Barrierefreiheit ist stark eingeschränkt — dringender Handlungsbedarf.",
            batch.summary.total_urls, avg
        )
    }
}

pub(super) fn build_batch_appendix(batch: &BatchReport) -> BatchAppendixData {
    BatchAppendixData {
        per_url: batch
            .reports
            .iter()
            .map(|r| {
                let mut rule_map: std::collections::HashMap<String, AppendixViolation> =
                    std::collections::HashMap::new();
                let mut rule_order: Vec<String> = Vec::new();

                for v in &r.wcag_results.violations {
                    let element = AffectedElement {
                        selector: v.selector.clone().unwrap_or_else(|| v.node_id.clone()),
                        node_id: v.node_id.clone(),
                    };

                    if let Some(existing) = rule_map.get_mut(&v.rule) {
                        existing.affected_elements.push(element);
                    } else {
                        rule_order.push(v.rule.clone());
                        rule_map.insert(
                            v.rule.clone(),
                            AppendixViolation {
                                rule: v.rule.clone(),
                                rule_name: v.rule_name.clone(),
                                severity: v.severity,
                                message: v.message.clone(),
                                fix_suggestion: v.fix_suggestion.clone(),
                                affected_elements: vec![element],
                            },
                        );
                    }
                }

                UrlAppendix {
                    url: r.url.clone(),
                    violations: rule_order
                        .into_iter()
                        .filter_map(|rule| rule_map.remove(&rule))
                        .collect(),
                }
            })
            .collect(),
    }
}

pub(super) fn yes_no(val: bool) -> String {
    if val {
        "Ja".to_string()
    } else {
        "Nein".to_string()
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

pub(super) fn grade_label(score: u32) -> &'static str {
    match score {
        90..=100 => "Sehr gut",
        75..=89 => "Gut",
        60..=74 => "Befriedigend",
        40..=59 => "Ausbaufähig",
        _ => "Kritisch",
    }
}

pub(super) fn interpret_score(score: f32, area: &str) -> String {
    let grade = grade_label(score.round() as u32);
    match grade {
        "Sehr gut" => format!("{} — die {} ist auf einem hohen Niveau.", grade, area),
        "Gut" => format!(
            "{} — die {} ist solide, einzelne Verbesserungen sind möglich.",
            grade, area
        ),
        "Befriedigend" => format!("{} — die {} weist einzelne Schwächen auf.", grade, area),
        "Ausbaufähig" => format!("{} — die {} weist relevante Schwächen auf.", grade, area),
        _ => format!(
            "{} — die {} hat erhebliche Mängel, die behoben werden sollten.",
            grade, area
        ),
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
