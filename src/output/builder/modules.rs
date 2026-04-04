//! Module-level score and context derivation helpers (accessibility, performance, SEO, security, mobile).

use crate::audit::NormalizedReport;
use crate::wcag::Severity;

// ─── Accessibility ───────────────────────────────────────────────────────────

pub(super) fn derive_accessibility_lever(normalized: &NormalizedReport) -> String {
    if let Some(finding) = normalized
        .findings
        .iter()
        .max_by_key(|f| f.occurrence_count)
    {
        format!("Größter Hebel: {}", finding.title)
    } else {
        "Größter Hebel: Ergebnisse stabil halten und manuell nachprüfen".to_string()
    }
}

pub(super) fn derive_accessibility_context(normalized: &NormalizedReport) -> String {
    let high = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::High | Severity::Critical))
        .count();
    let total = normalized.findings.len();
    if total == 0 {
        return "Keine automatisch erkannten Barrieren im aktuellen Lauf.".to_string();
    }
    format!(
        "{} erkannte Problemgruppe(n), davon {} mit hoher Priorität.",
        total, high
    )
}

pub(super) fn derive_accessibility_card_context(normalized: &NormalizedReport) -> String {
    let high = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::High | Severity::Critical))
        .count();
    if high == 0 {
        "Keine High-Priority-Funde".to_string()
    } else {
        format!("{high} Problemgruppe(n) mit hoher Priorität")
    }
}

// ─── Performance ─────────────────────────────────────────────────────────────

pub(super) fn derive_performance_lever(perf: &crate::audit::PerformanceResults) -> String {
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1500 {
            return format!("Größter Hebel: DOM-Größe reduzieren ({dom_nodes} Knoten)");
        }
    }
    if let Some(load) = perf.vitals.load_time {
        if load > 2_500.0 {
            return format!("Größter Hebel: Ladezeit senken ({load:.0} ms)");
        }
    }
    "Größter Hebel: Render-Pfad und Asset-Größe weiter optimieren".to_string()
}

pub(super) fn derive_performance_context(perf: &crate::audit::PerformanceResults) -> String {
    let fcp = perf
        .vitals
        .fcp
        .as_ref()
        .map(|v| format!("FCP {:.0} ms", v.value))
        .unwrap_or_else(|| "FCP n/a".to_string());
    let ttfb = perf
        .vitals
        .ttfb
        .as_ref()
        .map(|v| format!("TTFB {:.0} ms", v.value))
        .unwrap_or_else(|| "TTFB n/a".to_string());
    let dom = perf
        .vitals
        .dom_nodes
        .map(|n| format!("{n} DOM-Knoten"))
        .unwrap_or_else(|| "DOM-Knoten n/a".to_string());
    format!("{fcp}, {ttfb}, {dom}.")
}

pub(super) fn derive_performance_card_context(perf: &crate::audit::PerformanceResults) -> String {
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        return format!("{dom_nodes} DOM-Knoten");
    }
    if let Some(load) = perf.vitals.load_time {
        return format!("Ladezeit {:.0} ms", load);
    }
    "Render-Pfad weiter optimieren".to_string()
}

pub(super) fn derive_performance_recommendations(
    perf: &crate::audit::PerformanceResults,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if let Some(lcp) = &perf.vitals.lcp {
        if lcp.value > 2500.0 {
            recommendations.push(
                "Größtes sichtbares Element schneller laden: Hero-Bilder optimieren, priorisieren und kritische Styles früher ausliefern."
                    .to_string(),
            );
        }
    }

    if let Some(fcp) = &perf.vitals.fcp {
        if fcp.value > 1800.0 {
            recommendations.push(
                "Ersten sichtbaren Inhalt früher ausliefern: render-blockierende CSS- und JavaScript-Dateien reduzieren."
                    .to_string(),
            );
        }
    }

    if let Some(interactivity) = perf.vitals.inp.as_ref().or(perf.vitals.tbt.as_ref()) {
        if interactivity.value > 200.0 {
            recommendations.push(
                "Haupt-Thread entlasten: große JavaScript-Aufgaben aufteilen und nicht benötigte Skripte später laden."
                    .to_string(),
            );
        }
    }

    if let Some(cls) = &perf.vitals.cls {
        if cls.value > 0.1 {
            recommendations.push(
                "Layout-Verschiebungen vermeiden: Medien, Banner und dynamische Inhalte mit festen Platzhaltern reservieren."
                    .to_string(),
            );
        }
    }

    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1200 {
            recommendations.push(
                "DOM-Struktur verschlanken: große Komponenten, tiefe Container-Hierarchien und wiederholte Markup-Blöcke reduzieren."
                    .to_string(),
            );
        }
    }

    if let Some(load_time) = perf.vitals.load_time {
        if load_time > 3000.0 {
            recommendations.push(
                "Gesamte Ladezeit senken: große Assets komprimieren, Caching schärfen und Drittanbieter-Skripte prüfen."
                    .to_string(),
            );
        }
    }

    if recommendations.is_empty() {
        recommendations.push(
            "Die Kernmetriken sind stabil. Nächster Hebel: Seitengröße und Drittanbieter-Skripte regelmäßig überwachen, damit das Niveau gehalten wird."
                .to_string(),
        );
    }

    recommendations.truncate(3);
    recommendations
}

// ─── SEO ─────────────────────────────────────────────────────────────────────

pub(super) fn derive_seo_lever(seo: &crate::seo::SeoAnalysis) -> String {
    if !seo.meta_issues.is_empty() {
        return format!(
            "Größter Hebel: Meta-Daten bereinigen ({} offene Punkte)",
            seo.meta_issues.len()
        );
    }
    if seo.social.completeness < 80 {
        return "Größter Hebel: Social-Meta-Daten vervollständigen".to_string();
    }
    "Größter Hebel: Struktur- und Inhalts-Signale weiter schärfen".to_string()
}

pub(super) fn derive_seo_context(seo: &crate::seo::SeoAnalysis) -> String {
    let meta_issues = seo.meta_issues.len();
    let schema_count = seo.structured_data.json_ld.len();
    let h1 = seo.headings.h1_count;
    format!(
        "{} Meta-Probleme, {} H1, {} strukturierte Daten erkannt.",
        meta_issues, h1, schema_count
    )
}

pub(super) fn derive_seo_card_context(seo: &crate::seo::SeoAnalysis) -> String {
    if !seo.meta_issues.is_empty() {
        return format!("{} Meta-Probleme offen", seo.meta_issues.len());
    }
    format!(
        "{} strukturierte Daten erkannt",
        seo.structured_data.json_ld.len()
    )
}

// ─── Security ────────────────────────────────────────────────────────────────

pub(super) fn derive_security_lever(sec: &crate::security::SecurityAnalysis) -> String {
    let missing_headers = sec.headers.content_security_policy.is_none() as usize
        + sec.headers.strict_transport_security.is_none() as usize
        + sec.headers.permissions_policy.is_none() as usize
        + sec.headers.referrer_policy.is_none() as usize;
    if missing_headers > 0 {
        return format!(
            "Größter Hebel: fehlende Security-Header ergänzen ({missing_headers} Kernheader)"
        );
    }
    "Größter Hebel: Header-Regeln und TLS-Setup weiter härten".to_string()
}

pub(super) fn derive_security_context(sec: &crate::security::SecurityAnalysis) -> String {
    let present_headers = [
        sec.headers.content_security_policy.is_some(),
        sec.headers.strict_transport_security.is_some(),
        sec.headers.x_content_type_options.is_some(),
        sec.headers.x_frame_options.is_some(),
        sec.headers.x_xss_protection.is_some(),
        sec.headers.referrer_policy.is_some(),
        sec.headers.permissions_policy.is_some(),
        sec.headers.cross_origin_opener_policy.is_some(),
        sec.headers.cross_origin_resource_policy.is_some(),
    ]
    .into_iter()
    .filter(|p| *p)
    .count();
    format!(
        "{} von 9 Kern-Headern vorhanden, HTTPS {}.",
        present_headers,
        if sec.ssl.https { "aktiv" } else { "fehlt" }
    )
}

pub(super) fn derive_security_card_context(sec: &crate::security::SecurityAnalysis) -> String {
    let present_headers = [
        sec.headers.content_security_policy.is_some(),
        sec.headers.strict_transport_security.is_some(),
        sec.headers.x_content_type_options.is_some(),
        sec.headers.x_frame_options.is_some(),
        sec.headers.x_xss_protection.is_some(),
        sec.headers.referrer_policy.is_some(),
        sec.headers.permissions_policy.is_some(),
        sec.headers.cross_origin_opener_policy.is_some(),
        sec.headers.cross_origin_resource_policy.is_some(),
    ]
    .into_iter()
    .filter(|p| *p)
    .count();
    format!("{present_headers} von 9 Kern-Headern vorhanden")
}

pub(super) fn derive_security_recommendations(
    sec: &crate::security::SecurityAnalysis,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !sec.ssl.https {
        recommendations.push(
            "HTTPS durchgängig erzwingen und ein gültiges TLS-Zertifikat für alle Varianten der Domain sicherstellen."
                .to_string(),
        );
    }

    if sec.headers.content_security_policy.is_none() {
        recommendations.push(
            "Content-Security-Policy ergänzen und nur die tatsächlich benötigten Skript-, Style- und Medienquellen erlauben."
                .to_string(),
        );
    }

    if sec.headers.strict_transport_security.is_none() && sec.ssl.https {
        recommendations.push(
            "HSTS ergänzen, damit Browser die Seite dauerhaft nur noch per HTTPS laden."
                .to_string(),
        );
    }

    if sec.headers.cross_origin_opener_policy.is_none() {
        recommendations.push(
            "Cross-Origin-Opener-Policy prüfen und setzen, um die Isolation des Browser-Kontexts für moderne Webfunktionen zu stärken."
                .to_string(),
        );
    }

    if sec.headers.cross_origin_resource_policy.is_none() {
        recommendations.push(
            "Cross-Origin-Resource-Policy ergänzen, damit eingebundene Ressourcen nicht unnötig von fremden Origins mitgenutzt werden können."
                .to_string(),
        );
    }

    if sec.headers.permissions_policy.is_none() {
        recommendations.push(
            "Permissions-Policy definieren und nur die Browser-Funktionen freigeben, die auf der Seite wirklich benötigt werden."
                .to_string(),
        );
    }

    if sec.headers.referrer_policy.is_none() {
        recommendations.push(
            "Referrer-Policy setzen, damit bei Weiterleitungen und externen Aufrufen nicht mehr Informationen als nötig übergeben werden."
                .to_string(),
        );
    }

    if recommendations.is_empty() {
        recommendations.push(
            "Die grundlegenden Security-Header sind sauber gesetzt. Nächster Schritt: Richtlinien regelmäßig prüfen und an neue Skript- oder Integrationsquellen anpassen."
                .to_string(),
        );
    }

    recommendations.truncate(4);
    recommendations
}

// ─── Mobile ──────────────────────────────────────────────────────────────────

pub(super) fn derive_mobile_lever(mobile: &crate::mobile::MobileFriendliness) -> String {
    if mobile.touch_targets.small_targets > 0 {
        return format!(
            "Größter Hebel: Touch Targets vergrößern ({} zu klein)",
            mobile.touch_targets.small_targets
        );
    }
    if mobile.touch_targets.crowded_targets > 0 {
        return format!(
            "Größter Hebel: Abstände mobiler Bedienelemente erhöhen ({})",
            mobile.touch_targets.crowded_targets
        );
    }
    "Größter Hebel: mobile Lesbarkeit und Touch-Flows weiter optimieren".to_string()
}

pub(super) fn derive_mobile_context(mobile: &crate::mobile::MobileFriendliness) -> String {
    format!(
        "Viewport {}, {} zu kleine Touch Targets, {} zu enge Abstände.",
        if mobile.viewport.is_properly_configured {
            "korrekt gesetzt"
        } else {
            "nicht sauber konfiguriert"
        },
        mobile.touch_targets.small_targets,
        mobile.touch_targets.crowded_targets
    )
}

pub(super) fn derive_mobile_card_context(mobile: &crate::mobile::MobileFriendliness) -> String {
    if mobile.touch_targets.small_targets > 0 {
        format!(
            "{} zu kleine Touch Targets",
            mobile.touch_targets.small_targets
        )
    } else if mobile.touch_targets.crowded_targets > 0 {
        format!("{} zu enge Abstände", mobile.touch_targets.crowded_targets)
    } else if mobile.viewport.is_properly_configured {
        "Viewport korrekt gesetzt".to_string()
    } else {
        "Viewport prüfen".to_string()
    }
}

// ─── Tracking ────────────────────────────────────────────────────────────────

pub(super) fn build_tracking_summary_text(
    technical: &crate::seo::technical::TechnicalSeo,
) -> String {
    if technical.zaraz.detected {
        if technical.tracking_cookies.is_empty() && technical.tracking_signals.is_empty() {
            return "Zaraz ist auf der Seite erkennbar. Zusätzlich wurden im Lauf keine weiteren Tracking-Cookies oder externen Tracking-Signale festgestellt.".to_string();
        }
        return "Auf der Seite sind Tracking- oder Consent-nahe Signale erkennbar. Prüfen Sie insbesondere externe Einbindungen, Cookie-Setzung und den tatsächlichen Auslösezeitpunkt nach Einwilligung.".to_string();
    }

    if technical.uses_remote_google_fonts {
        return "Es werden extern gehostete Google Fonts geladen. Das ist datenschutz- und performance-relevant und sollte bewusst geprüft werden.".to_string();
    }

    if !technical.tracking_cookies.is_empty() || !technical.tracking_signals.is_empty() {
        return "Es wurden Tracking-Signale erkannt. Prüfen Sie Einwilligung, Auslösezeitpunkt und die Herkunft der eingebundenen Dienste.".to_string();
    }

    "Im aktuellen Lauf wurden keine externen Google Fonts, keine Tracking-Cookies und keine weiteren Tracking-Signale erkannt.".to_string()
}

