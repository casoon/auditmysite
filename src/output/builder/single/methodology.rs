use super::super::helpers::localized_module_name;
use crate::audit::normalized::NormalizedReport;
use crate::output::report_model::{
    AffectedElement, AppendixBlock, AppendixViolation, CapabilitySignal, MethodologyBlock,
};

pub(super) fn build_appendix_block_from_normalized(normalized: &NormalizedReport) -> AppendixBlock {
    let violations: Vec<AppendixViolation> = normalized
        .findings
        .iter()
        .map(|f| {
            let affected_elements: Vec<AffectedElement> = f
                .occurrences
                .iter()
                .map(|occ| AffectedElement {
                    selector: occ.selector.clone().unwrap_or_else(|| occ.node_id.clone()),
                    node_id: occ.node_id.clone(),
                })
                .collect();

            AppendixViolation {
                rule: f.wcag_criterion.clone(),
                rule_name: f.title.clone(),
                severity: f.severity,
                message: f.description.clone(),
                fix_suggestion: f.occurrences.first().and_then(|o| o.fix_suggestion.clone()),
                affected_elements,
            }
        })
        .collect();

    let has_violations = !violations.is_empty();

    AppendixBlock {
        violations,
        score_methodology: "Score-Berechnung: Basis 100 Punkte. Abzug auf Basis der Taxonomie-Regel-Definitionen \
            mit regelspezifischen Penalties und logarithmischer Skalierung für wiederholte Verstöße. \
            Konfligierende Signale werden als Hinweis markiert, verändern den Score jedoch nicht nachträglich."
            .to_string(),
        has_violations,
    }
}

pub(super) fn build_methodology(
    i18n: &crate::i18n::I18n,
    normalized: &NormalizedReport,
) -> MethodologyBlock {
    let locale = i18n.locale();
    let en = locale == "en";
    let active_modules = normalized
        .module_scores
        .iter()
        .map(|m| m.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    let scope = if en {
        format!(
            "Automated audit of {} for accessibility per WCAG 2.1 (level {}). \
             Performance, SEO, security and mobile usability were also analyzed.",
            normalized.url, normalized.wcag_level
        )
    } else {
        format!(
            "Automatisierte Prüfung der Seite {} auf Barrierefreiheit nach WCAG 2.1 (Level {}). \
             Zusätzlich wurden Performance, SEO, Sicherheit und mobile Nutzbarkeit analysiert.",
            normalized.url, normalized.wcag_level
        )
    };
    let method = if en {
        "The audit was performed via the Chrome DevTools Protocol (CDP) and the browser's native \
         accessibility tree. 21 WCAG rules were checked automatically against the page content."
            .to_string()
    } else {
        "Die Prüfung erfolgte über den Chrome DevTools Protocol (CDP) und den \
         nativen Accessibility Tree des Browsers. 21 WCAG-Regeln wurden automatisiert \
         gegen den Seiteninhalt geprüft."
            .to_string()
    };
    let limitations = if en {
        "Automated tests can detect about 30–40% of all accessibility issues. Complex aspects \
         such as correct tab order, meaningful alt texts, or understandable language additionally \
         require manual review."
            .to_string()
    } else {
        "Automatisierte Tests können ca. 30–40% aller Barrierefreiheitsprobleme erkennen. \
         Komplexe Aspekte wie korrekte Tab-Reihenfolge, sinnvolle Alt-Texte oder \
         verständliche Sprache erfordern zusätzlich manuelle Prüfung."
            .to_string()
    };
    let disclaimer = if en {
        "This report represents an automated technical analysis. It does not replace a complete \
         WCAG 2.1 conformance assessment. A legally defensible accessibility statement requires a \
         comprehensive manual audit by experts."
            .to_string()
    } else {
        "Dieser Report stellt eine automatisierte technische Analyse dar. \
         Er ersetzt keine vollständige Konformitätsbewertung nach WCAG 2.1. \
         Für eine rechtsverbindliche Aussage zur Barrierefreiheit ist eine \
         umfassende manuelle Prüfung durch Experten erforderlich."
            .to_string()
    };

    let key = |de: &str, en_label: &str| -> String {
        if en {
            en_label.to_string()
        } else {
            de.to_string()
        }
    };

    let preview_value = if normalized.has_screenshots {
        if en {
            "Desktop and mobile captured".to_string()
        } else {
            "Desktop und Mobile erfasst".to_string()
        }
    } else if en {
        "Not captured".to_string()
    } else {
        "Nicht erfasst".to_string()
    };

    let total_score_value = {
        let contributing: Vec<_> = normalized
            .module_scores
            .iter()
            .filter(|m| m.contributes_to_overall)
            .collect();
        let total_raw: u32 = contributing.iter().map(|m| m.weight_pct).sum();
        let weights: Vec<String> = contributing
            .iter()
            .map(|m| {
                let pct = (m.weight_pct * 100 + total_raw / 2)
                    .checked_div(total_raw)
                    .unwrap_or(0);
                let name = localized_module_name(&m.name, i18n);
                if en {
                    format!("{name} {pct}%")
                } else {
                    format!("{name} {pct} %")
                }
            })
            .collect();
        let weights_label = if weights.is_empty() {
            if en {
                "Accessibility 100%".to_string()
            } else {
                "Barrierefreiheit 100 %".to_string()
            }
        } else {
            weights.join(", ")
        };
        let indicator_names: Vec<String> = normalized
            .module_scores
            .iter()
            .filter(|m| !m.contributes_to_overall || m.measurement_type == "heuristic")
            .map(|m| localized_module_name(&m.name, i18n))
            .collect();
        let indicator_note = if indicator_names.is_empty() {
            String::new()
        } else if en {
            format!(
                " Indicator modules shown separately: {}.",
                indicator_names.join(", ")
            )
        } else {
            format!(
                " Separat ausgewiesene Indikator-Module: {}.",
                indicator_names.join(", ")
            )
        };
        if en {
            format!(
                "{} / 100 — contributing weights: {}{}",
                normalized.overall_score, weights_label, indicator_note
            )
        } else {
            format!(
                "{} / 100 — beitragende Gewichtung: {}{}",
                normalized.overall_score, weights_label, indicator_note
            )
        }
    };

    let runtime_unit = "s";

    MethodologyBlock {
        scope,
        method,
        limitations,
        disclaimer,
        audit_facts: vec![
            (
                key("Primärscore", "Primary score"),
                format!("Accessibility {} / 100", normalized.score),
            ),
            (key("Gesamtscore", "Overall score"), total_score_value),
            (
                key("WCAG-Level", "WCAG level"),
                normalized.wcag_level.to_string(),
            ),
            (
                key("Geprüfte Knoten", "Checked nodes"),
                normalized.nodes_analyzed.to_string(),
            ),
            (
                key("Laufzeit", "Runtime"),
                format!(
                    "{:.1} {}",
                    normalized.duration_ms as f64 / 1000.0,
                    runtime_unit
                ),
            ),
            (key("Aktive Module", "Active modules"), active_modules),
            (
                key("Audit-Hinweise", "Audit notes"),
                normalized.audit_flags.len().to_string(),
            ),
            (key("Vorschau", "Preview"), preview_value),
        ],
        confidence_summary: build_confidence_summary(locale, normalized),
        capabilities: build_capability_matrix(locale, normalized),
    }
}

fn build_confidence_summary(locale: &str, normalized: &NormalizedReport) -> Vec<(String, String)> {
    let en = locale == "en";
    let base_confidence = if normalized.nodes_analyzed >= 2_000 {
        if en {
            "High"
        } else {
            "Hoch"
        }
    } else if normalized.nodes_analyzed >= 500 {
        if en {
            "Solid"
        } else {
            "Solide"
        }
    } else if en {
        "Limited"
    } else {
        "Begrenzt"
    };
    let caveat_level = if normalized.audit_flags.is_empty() {
        if en {
            "No automatically detected conflict signals"
        } else {
            "Keine automatisiert erkannten Konfliktsignale"
        }
    } else if normalized.audit_flags.len() == 1 {
        if en {
            "1 caveat signal"
        } else {
            "1 Hinweissignal"
        }
    } else if en {
        "Multiple caveat signals"
    } else {
        "Mehrere Hinweissignale"
    };
    let module_coverage = if normalized.module_scores.len() >= 5 {
        if en {
            "Broad"
        } else {
            "Breit"
        }
    } else if normalized.module_scores.len() >= 3 {
        if en {
            "Extended"
        } else {
            "Erweitert"
        }
    } else if en {
        "Core checks"
    } else {
        "Kern-Checks"
    };

    let (label_trust, label_coverage, label_signals, label_manual, val_manual) = if en {
        (
            "Audit confidence",
            "Module coverage",
            "Conflict signals",
            "Manual review needed",
            "Yes, for semantic quality and usage context",
        )
    } else {
        (
            "Audit-Vertrauen",
            "Modul-Abdeckung",
            "Konfliktsignale",
            "Manuelle Prüfung nötig",
            "Ja, für semantische Qualität und Nutzungskontext",
        )
    };

    vec![
        (label_trust.to_string(), base_confidence.to_string()),
        (label_coverage.to_string(), module_coverage.to_string()),
        (label_signals.to_string(), caveat_level.to_string()),
        (label_manual.to_string(), val_manual.to_string()),
    ]
}

fn build_capability_matrix(locale: &str, normalized: &NormalizedReport) -> Vec<CapabilitySignal> {
    let en = locale == "en";
    let confidence_high = if en { "High" } else { "Hoch" };
    let confidence_solid = if en { "Solid" } else { "Solide" };
    let confidence_off = if en { "Not active" } else { "Nicht aktiv" };

    let mut capabilities = vec![
        CapabilitySignal {
            signal: if en {
                "WCAG rules & occurrences".into()
            } else {
                "WCAG-Regeln & Vorkommen".into()
            },
            source: if en {
                "Accessibility tree + rule engine".into()
            } else {
                "Accessibility Tree + Regelengine".into()
            },
            confidence: if normalized.nodes_analyzed >= 500 {
                confidence_high.to_string()
            } else {
                confidence_solid.to_string()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into(), "Studio".into()],
            note: if en {
                "Primary audit truth for automatically detectable violations.".to_string()
            } else {
                "Primäre Audit-Wahrheit für automatisiert erkennbare Verstöße.".to_string()
            },
        },
        CapabilitySignal {
            signal: if en {
                "Web vitals & loading indicators".into()
            } else {
                "Web Vitals & Ladeindikatoren".into()
            },
            source: if en {
                "Performance module".into()
            } else {
                "Performance-Modul".into()
            },
            confidence: if normalized.raw_performance.is_some() {
                confidence_high.to_string()
            } else {
                confidence_off.to_string()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: if en {
                "FCP, CLS and TTFB are reflected in facts and module sections.".to_string()
            } else {
                "FCP, CLS und TTFB werden in Facts und Modulkapiteln gespiegelt.".to_string()
            },
        },
        CapabilitySignal {
            signal: if en {
                "SEO structure & schema".into()
            } else {
                "SEO-Struktur & Schema".into()
            },
            source: if en {
                "SEO module".into()
            } else {
                "SEO-Modul".into()
            },
            confidence: if normalized.raw_seo.is_some() {
                confidence_solid.to_string()
            } else {
                confidence_off.to_string()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: if en {
                "Meta, heading and schema signals are condensed into a report-ready form."
                    .to_string()
            } else {
                "Meta-, Heading- und Schema-Signale sind reportfähig verdichtet.".to_string()
            },
        },
        CapabilitySignal {
            signal: if en {
                "Security headers & HTTPS".into()
            } else {
                "Security Header & HTTPS".into()
            },
            source: if en {
                "Security module".into()
            } else {
                "Security-Modul".into()
            },
            confidence: if normalized.raw_security.is_some() {
                confidence_high.to_string()
            } else {
                confidence_off.to_string()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into()],
            note: if en {
                "Header presence and HTTPS status remain visible as raw signal.".to_string()
            } else {
                "Header-Präsenz und HTTPS-Status bleiben als Rohsignal sichtbar.".to_string()
            },
        },
        CapabilitySignal {
            signal: if en {
                "Mobile, UX, journey".into()
            } else {
                "Mobile, UX, Journey".into()
            },
            source: if en {
                "Heuristic modules".into()
            } else {
                "Heuristik-Module".into()
            },
            confidence: if en {
                "Indicator-based".into()
            } else {
                "Hinweisbasiert".into()
            },
            surfaces: vec!["CLI".into(), "JSON".into(), "PDF".into(), "Studio".into()],
            note: if en {
                "Useful for prioritization, not the sole UX truth.".to_string()
            } else {
                "Zur Priorisierung geeignet, nicht als alleinige UX-Gesamtwahrheit.".to_string()
            },
        },
    ];

    if !normalized.audit_flags.is_empty() {
        capabilities.push(CapabilitySignal {
            signal: if en {
                "Audit conflict signals".into()
            } else {
                "Audit-Konfliktsignale".into()
            },
            source: if en {
                "Normalization / cross-checks".into()
            } else {
                "Normalisierung / Cross-Checks".into()
            },
            confidence: if en {
                "Explicitly flagged".into()
            } else {
                "Explizit markiert".into()
            },
            surfaces: vec!["JSON".into(), "PDF".into()],
            note: if en {
                format!(
                    "{} conflict signal(s) are surfaced openly rather than hidden in the score.",
                    normalized.audit_flags.len()
                )
            } else {
                format!(
                    "{} Konfliktsignal(e) werden offen ausgewiesen statt im Score versteckt.",
                    normalized.audit_flags.len()
                )
            },
        });
    }

    capabilities
}
