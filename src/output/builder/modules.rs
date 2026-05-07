//! Module-level score and context derivation helpers (accessibility, performance, SEO, security, mobile).

use crate::audit::NormalizedReport;
use crate::wcag::Severity;

#[inline]
fn is_en(locale: &str) -> bool {
    locale == "en"
}

// ─── Accessibility ───────────────────────────────────────────────────────────

pub(super) fn derive_accessibility_lever(locale: &str, normalized: &NormalizedReport) -> String {
    let en = is_en(locale);
    if let Some(finding) = normalized
        .findings
        .iter()
        .max_by_key(|f| f.occurrence_count)
    {
        if en {
            format!("Biggest lever: {}", finding.title)
        } else {
            format!("Größter Hebel: {}", finding.title)
        }
    } else if en {
        "Biggest lever: maintain results and verify manually".to_string()
    } else {
        "Größter Hebel: Ergebnisse stabil halten und manuell nachprüfen".to_string()
    }
}

pub(super) fn derive_accessibility_context(locale: &str, normalized: &NormalizedReport) -> String {
    let en = is_en(locale);
    let high = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::High | Severity::Critical))
        .count();
    let total = normalized.findings.len();
    if total == 0 {
        return if en {
            "No automatically detected barriers in the current run.".to_string()
        } else {
            "Keine automatisch erkannten Barrieren im aktuellen Lauf.".to_string()
        };
    }
    if en {
        format!("{total} detected finding group(s), {high} high priority.")
    } else {
        format!(
            "{} erkannte Problemgruppe(n), davon {} mit hoher Priorität.",
            total, high
        )
    }
}

pub(super) fn derive_accessibility_card_context(
    locale: &str,
    normalized: &NormalizedReport,
) -> String {
    let en = is_en(locale);
    let high = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::High | Severity::Critical))
        .count();
    if high == 0 {
        if en {
            "No high-priority findings".to_string()
        } else {
            "Keine High-Priority-Funde".to_string()
        }
    } else if en {
        format!("{high} high-priority finding group(s)")
    } else {
        format!("{high} Problemgruppe(n) mit hoher Priorität")
    }
}

// ─── Performance ─────────────────────────────────────────────────────────────

pub(super) fn derive_performance_lever(
    locale: &str,
    perf: &crate::audit::PerformanceResults,
) -> String {
    let en = is_en(locale);
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1500 {
            return if en {
                format!("Biggest lever: reduce DOM size ({dom_nodes} nodes)")
            } else {
                format!("Größter Hebel: DOM-Größe reduzieren ({dom_nodes} Knoten)")
            };
        }
    }
    if let Some(load) = perf.vitals.load_time {
        if load > 2_500.0 {
            return if en {
                format!("Biggest lever: reduce load time ({load:.0} ms)")
            } else {
                format!("Größter Hebel: Ladezeit senken ({load:.0} ms)")
            };
        }
    }
    if en {
        "Biggest lever: keep optimizing the render path and asset size".to_string()
    } else {
        "Größter Hebel: Render-Pfad und Asset-Größe weiter optimieren".to_string()
    }
}

pub(super) fn derive_performance_context(
    locale: &str,
    perf: &crate::audit::PerformanceResults,
) -> String {
    let en = is_en(locale);
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
        .map(|n| {
            if en {
                format!("{n} DOM nodes")
            } else {
                format!("{n} DOM-Knoten")
            }
        })
        .unwrap_or_else(|| {
            if en {
                "DOM nodes n/a".to_string()
            } else {
                "DOM-Knoten n/a".to_string()
            }
        });
    format!("{fcp}, {ttfb}, {dom}.")
}

pub(super) fn derive_performance_card_context(
    locale: &str,
    perf: &crate::audit::PerformanceResults,
) -> String {
    let en = is_en(locale);
    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        return if en {
            format!("{dom_nodes} DOM nodes")
        } else {
            format!("{dom_nodes} DOM-Knoten")
        };
    }
    if let Some(load) = perf.vitals.load_time {
        return if en {
            format!("Load time {:.0} ms", load)
        } else {
            format!("Ladezeit {:.0} ms", load)
        };
    }
    if en {
        "Continue optimizing the render path".to_string()
    } else {
        "Render-Pfad weiter optimieren".to_string()
    }
}

pub(super) fn derive_performance_recommendations(
    locale: &str,
    perf: &crate::audit::PerformanceResults,
) -> Vec<String> {
    let en = is_en(locale);
    let mut recommendations = Vec::new();

    if let Some(lcp) = &perf.vitals.lcp {
        if lcp.value > 2500.0 {
            recommendations.push(if en {
                "Load the largest visible element faster: optimize and prioritize hero images and ship critical styles earlier.".to_string()
            } else {
                "Größtes sichtbares Element schneller laden: Hero-Bilder optimieren, priorisieren und kritische Styles früher ausliefern.".to_string()
            });
        }
    }

    if let Some(fcp) = &perf.vitals.fcp {
        if fcp.value > 1800.0 {
            recommendations.push(if en {
                "Deliver first visible content earlier: reduce render-blocking CSS and JavaScript files.".to_string()
            } else {
                "Ersten sichtbaren Inhalt früher ausliefern: render-blockierende CSS- und JavaScript-Dateien reduzieren.".to_string()
            });
        }
    }

    if let Some(interactivity) = perf.vitals.inp.as_ref().or(perf.vitals.tbt.as_ref()) {
        if interactivity.value > 200.0 {
            recommendations.push(if en {
                "Free the main thread: split large JavaScript tasks and load non-essential scripts later.".to_string()
            } else {
                "Haupt-Thread entlasten: große JavaScript-Aufgaben aufteilen und nicht benötigte Skripte später laden.".to_string()
            });
        }
    }

    if let Some(cls) = &perf.vitals.cls {
        if cls.value > 0.1 {
            recommendations.push(if en {
                "Avoid layout shifts: reserve space for media, banners and dynamic content with fixed placeholders.".to_string()
            } else {
                "Layout-Verschiebungen vermeiden: Medien, Banner und dynamische Inhalte mit festen Platzhaltern reservieren.".to_string()
            });
        }
    }

    if let Some(dom_nodes) = perf.vitals.dom_nodes {
        if dom_nodes > 1200 {
            recommendations.push(if en {
                "Slim down the DOM: reduce large components, deep container hierarchies and repeated markup blocks.".to_string()
            } else {
                "DOM-Struktur verschlanken: große Komponenten, tiefe Container-Hierarchien und wiederholte Markup-Blöcke reduzieren.".to_string()
            });
        }
    }

    if let Some(load_time) = perf.vitals.load_time {
        if load_time > 3000.0 {
            recommendations.push(if en {
                "Reduce overall load time: compress large assets, sharpen caching and review third-party scripts.".to_string()
            } else {
                "Gesamte Ladezeit senken: große Assets komprimieren, Caching schärfen und Drittanbieter-Skripte prüfen.".to_string()
            });
        }
    }

    if recommendations.is_empty() {
        recommendations.push(if en {
            "Core metrics are stable. Next lever: regularly monitor page size and third-party scripts to keep the level.".to_string()
        } else {
            "Die Kernmetriken sind stabil. Nächster Hebel: Seitengröße und Drittanbieter-Skripte regelmäßig überwachen, damit das Niveau gehalten wird.".to_string()
        });
    }

    recommendations.truncate(3);
    recommendations
}

// ─── SEO ─────────────────────────────────────────────────────────────────────

pub(super) fn derive_seo_lever(locale: &str, seo: &crate::seo::SeoAnalysis) -> String {
    let en = is_en(locale);
    if !seo.meta_issues.is_empty() {
        return if en {
            format!(
                "Biggest lever: clean up meta data ({} open issues)",
                seo.meta_issues.len()
            )
        } else {
            format!(
                "Größter Hebel: Meta-Daten bereinigen ({} offene Punkte)",
                seo.meta_issues.len()
            )
        };
    }
    if seo.social.completeness < 80 {
        return if en {
            "Biggest lever: complete social meta data".to_string()
        } else {
            "Größter Hebel: Social-Meta-Daten vervollständigen".to_string()
        };
    }
    if en {
        "Biggest lever: keep sharpening structure and content signals".to_string()
    } else {
        "Größter Hebel: Struktur- und Inhalts-Signale weiter schärfen".to_string()
    }
}

pub(super) fn derive_seo_context(locale: &str, seo: &crate::seo::SeoAnalysis) -> String {
    let en = is_en(locale);
    let meta_issues = seo.meta_issues.len();
    let schema_count = seo.structured_data.json_ld.len();
    let h1 = seo.headings.h1_count;
    if en {
        format!(
            "{meta_issues} meta issues, {h1} H1, {schema_count} structured data items detected."
        )
    } else {
        format!(
            "{} Meta-Probleme, {} H1, {} strukturierte Daten erkannt.",
            meta_issues, h1, schema_count
        )
    }
}

pub(super) fn derive_seo_card_context(locale: &str, seo: &crate::seo::SeoAnalysis) -> String {
    let en = is_en(locale);
    if !seo.meta_issues.is_empty() {
        return if en {
            format!("{} meta issues open", seo.meta_issues.len())
        } else {
            format!("{} Meta-Probleme offen", seo.meta_issues.len())
        };
    }
    if en {
        format!(
            "{} structured data items detected",
            seo.structured_data.json_ld.len()
        )
    } else {
        format!(
            "{} strukturierte Daten erkannt",
            seo.structured_data.json_ld.len()
        )
    }
}

// ─── Security ────────────────────────────────────────────────────────────────

pub(super) fn derive_security_lever(
    locale: &str,
    sec: &crate::security::SecurityAnalysis,
) -> String {
    let en = is_en(locale);
    let missing_headers = sec.headers.content_security_policy.is_none() as usize
        + sec.headers.strict_transport_security.is_none() as usize
        + sec.headers.permissions_policy.is_none() as usize
        + sec.headers.referrer_policy.is_none() as usize;
    if missing_headers > 0 {
        return if en {
            format!("Biggest lever: add missing security headers ({missing_headers} core headers)")
        } else {
            format!(
                "Größter Hebel: fehlende Security-Header ergänzen ({missing_headers} Kernheader)"
            )
        };
    }
    if en {
        "Biggest lever: keep hardening header rules and TLS setup".to_string()
    } else {
        "Größter Hebel: Header-Regeln und TLS-Setup weiter härten".to_string()
    }
}

pub(super) fn derive_security_context(
    locale: &str,
    sec: &crate::security::SecurityAnalysis,
) -> String {
    let en = is_en(locale);
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
    if en {
        format!(
            "{present_headers} of 9 core headers present, HTTPS {}.",
            if sec.ssl.https { "active" } else { "missing" }
        )
    } else {
        format!(
            "{} von 9 Kern-Headern vorhanden, HTTPS {}.",
            present_headers,
            if sec.ssl.https { "aktiv" } else { "fehlt" }
        )
    }
}

pub(super) fn derive_security_card_context(
    locale: &str,
    sec: &crate::security::SecurityAnalysis,
) -> String {
    let en = is_en(locale);
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
    if en {
        format!("{present_headers} of 9 core headers present")
    } else {
        format!("{present_headers} von 9 Kern-Headern vorhanden")
    }
}

pub(super) fn derive_security_recommendations(
    locale: &str,
    sec: &crate::security::SecurityAnalysis,
) -> Vec<String> {
    let en = is_en(locale);
    let mut recommendations = Vec::new();

    if !sec.ssl.https {
        recommendations.push(if en {
            "Enforce HTTPS everywhere and ensure a valid TLS certificate for every variant of the domain.".to_string()
        } else {
            "HTTPS durchgängig erzwingen und ein gültiges TLS-Zertifikat für alle Varianten der Domain sicherstellen.".to_string()
        });
    }

    if sec.headers.content_security_policy.is_none() {
        recommendations.push(if en {
            "Add a Content Security Policy and only allow the script, style and media sources actually needed.".to_string()
        } else {
            "Content-Security-Policy ergänzen und nur die tatsächlich benötigten Skript-, Style- und Medienquellen erlauben.".to_string()
        });
    }

    if sec.headers.strict_transport_security.is_none() && sec.ssl.https {
        recommendations.push(if en {
            "Add HSTS so browsers permanently load the site only via HTTPS.".to_string()
        } else {
            "HSTS ergänzen, damit Browser die Seite dauerhaft nur noch per HTTPS laden.".to_string()
        });
    }

    if sec.headers.cross_origin_opener_policy.is_none() {
        recommendations.push(if en {
            "Review and set Cross-Origin-Opener-Policy to strengthen browser context isolation for modern web features.".to_string()
        } else {
            "Cross-Origin-Opener-Policy prüfen und setzen, um die Isolation des Browser-Kontexts für moderne Webfunktionen zu stärken.".to_string()
        });
    }

    if sec.headers.cross_origin_resource_policy.is_none() {
        recommendations.push(if en {
            "Add Cross-Origin-Resource-Policy so embedded resources cannot be reused by unrelated origins unnecessarily.".to_string()
        } else {
            "Cross-Origin-Resource-Policy ergänzen, damit eingebundene Ressourcen nicht unnötig von fremden Origins mitgenutzt werden können.".to_string()
        });
    }

    if sec.headers.permissions_policy.is_none() {
        recommendations.push(if en {
            "Define a Permissions-Policy and only enable browser features actually used by the page.".to_string()
        } else {
            "Permissions-Policy definieren und nur die Browser-Funktionen freigeben, die auf der Seite wirklich benötigt werden.".to_string()
        });
    }

    if sec.headers.referrer_policy.is_none() {
        recommendations.push(if en {
            "Set a Referrer-Policy so redirects and external requests don't expose more information than necessary.".to_string()
        } else {
            "Referrer-Policy setzen, damit bei Weiterleitungen und externen Aufrufen nicht mehr Informationen als nötig übergeben werden.".to_string()
        });
    }

    if recommendations.is_empty() {
        recommendations.push(if en {
            "Core security headers are clean. Next step: review policies regularly and adapt them to new script and integration sources.".to_string()
        } else {
            "Die grundlegenden Security-Header sind sauber gesetzt. Nächster Schritt: Richtlinien regelmäßig prüfen und an neue Skript- oder Integrationsquellen anpassen.".to_string()
        });
    }

    recommendations.truncate(4);
    recommendations
}

// ─── Mobile ──────────────────────────────────────────────────────────────────

pub(super) fn derive_mobile_lever(
    locale: &str,
    mobile: &crate::mobile::MobileFriendliness,
) -> String {
    let en = is_en(locale);
    if mobile.touch_targets.small_targets > 0 {
        return if en {
            format!(
                "Biggest lever: enlarge touch targets ({} too small)",
                mobile.touch_targets.small_targets
            )
        } else {
            format!(
                "Größter Hebel: Touch Targets vergrößern ({} zu klein)",
                mobile.touch_targets.small_targets
            )
        };
    }
    if mobile.touch_targets.crowded_targets > 0 {
        return if en {
            format!(
                "Biggest lever: increase spacing of mobile controls ({})",
                mobile.touch_targets.crowded_targets
            )
        } else {
            format!(
                "Größter Hebel: Abstände mobiler Bedienelemente erhöhen ({})",
                mobile.touch_targets.crowded_targets
            )
        };
    }
    if en {
        "Biggest lever: keep optimizing mobile readability and touch flows".to_string()
    } else {
        "Größter Hebel: mobile Lesbarkeit und Touch-Flows weiter optimieren".to_string()
    }
}

pub(super) fn derive_mobile_context(
    locale: &str,
    mobile: &crate::mobile::MobileFriendliness,
) -> String {
    let en = is_en(locale);
    if en {
        format!(
            "Viewport {}, {} too-small touch targets, {} crowded gaps.",
            if mobile.viewport.is_properly_configured {
                "properly set"
            } else {
                "not properly configured"
            },
            mobile.touch_targets.small_targets,
            mobile.touch_targets.crowded_targets
        )
    } else {
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
}

pub(super) fn derive_mobile_card_context(
    locale: &str,
    mobile: &crate::mobile::MobileFriendliness,
) -> String {
    let en = is_en(locale);
    if mobile.touch_targets.small_targets > 0 {
        if en {
            format!(
                "{} too-small touch targets",
                mobile.touch_targets.small_targets
            )
        } else {
            format!(
                "{} zu kleine Touch Targets",
                mobile.touch_targets.small_targets
            )
        }
    } else if mobile.touch_targets.crowded_targets > 0 {
        if en {
            format!("{} crowded gaps", mobile.touch_targets.crowded_targets)
        } else {
            format!("{} zu enge Abstände", mobile.touch_targets.crowded_targets)
        }
    } else if mobile.viewport.is_properly_configured {
        if en {
            "Viewport properly set".to_string()
        } else {
            "Viewport korrekt gesetzt".to_string()
        }
    } else if en {
        "Review viewport".to_string()
    } else {
        "Viewport prüfen".to_string()
    }
}

// ─── Tracking ────────────────────────────────────────────────────────────────

pub(super) fn build_tracking_summary_text(
    locale: &str,
    technical: &crate::seo::technical::TechnicalSeo,
) -> String {
    let en = is_en(locale);
    if technical.zaraz.detected {
        if technical.tracking_cookies.is_empty() && technical.tracking_signals.is_empty() {
            return if en {
                "Zaraz is detectable on the page. No additional tracking cookies or external tracking signals were observed during the run.".to_string()
            } else {
                "Zaraz ist auf der Seite erkennbar. Zusätzlich wurden im Lauf keine weiteren Tracking-Cookies oder externen Tracking-Signale festgestellt.".to_string()
            };
        }
        return if en {
            "Tracking or consent-related signals are detectable on the page. Inspect external embeds, cookie behavior and the actual trigger time after consent.".to_string()
        } else {
            "Auf der Seite sind Tracking- oder Consent-nahe Signale erkennbar. Prüfen Sie insbesondere externe Einbindungen, Cookie-Setzung und den tatsächlichen Auslösezeitpunkt nach Einwilligung.".to_string()
        };
    }

    if technical.uses_remote_google_fonts {
        return if en {
            "Externally hosted Google Fonts are loaded. This is privacy- and performance-relevant and should be reviewed deliberately.".to_string()
        } else {
            "Es werden extern gehostete Google Fonts geladen. Das ist datenschutz- und performance-relevant und sollte bewusst geprüft werden.".to_string()
        };
    }

    if !technical.tracking_cookies.is_empty() || !technical.tracking_signals.is_empty() {
        return if en {
            "Tracking signals were detected. Verify consent, trigger time and the origin of embedded services.".to_string()
        } else {
            "Es wurden Tracking-Signale erkannt. Prüfen Sie Einwilligung, Auslösezeitpunkt und die Herkunft der eingebundenen Dienste.".to_string()
        };
    }

    if en {
        "No external Google Fonts, tracking cookies or other tracking signals were detected in the current run.".to_string()
    } else {
        "Im aktuellen Lauf wurden keine externen Google Fonts, keine Tracking-Cookies und keine weiteren Tracking-Signale erkannt.".to_string()
    }
}
