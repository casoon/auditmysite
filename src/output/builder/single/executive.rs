use crate::audit::normalized::{AuditContext, NormalizedReport};
use crate::i18n::I18n;
use crate::output::report_model::{
    ExecutiveNarrativeBlock, FindingGroup, PositiveSignal, PositiveSignalsBlock, SeverityBlock,
};

/// Translates pattern names into localized titles; the message stays as the
/// description (already human-readable from the detector).
pub(super) fn build_positive_signals(
    locale: &str,
    normalized: &AuditContext<'_>,
) -> PositiveSignalsBlock {
    let en = locale == "en";
    let mut items: Vec<PositiveSignal> = normalized
        .raw_patterns
        .map(|p| {
            p.recognized
                .iter()
                .map(|r| {
                    let title = match (en, r.pattern.as_str()) {
                        (true, "MainNavigation") => "Semantic main navigation",
                        (false, "MainNavigation") => "Semantische Hauptnavigation",
                        (true, "DisclosureMenu") => "Disclosure menu",
                        (false, "DisclosureMenu") => "Disclosure-Menü",
                        (true, "ModalDialog") => "Modal dialog",
                        (false, "ModalDialog") => "Modaler Dialog",
                        (true, "TabList") => "Tab list",
                        (false, "TabList") => "Tab-Liste",
                        (true, "SkipLink") => "Skip link",
                        (false, "SkipLink") => "Skip-Link",
                        (true, "Accordion") => "Accordion",
                        (false, "Accordion") => "Accordion",
                        (_, other) => other,
                    };
                    PositiveSignal {
                        title: title.to_string(),
                        description: r.message.clone(),
                        strong: matches!(r.confidence, crate::patterns::PatternConfidence::Strong),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    for pos in &normalized.raw_wcag.positives {
        let title = if en { "WCAG signal" } else { "WCAG-Signal" };
        items.push(PositiveSignal {
            title: title.to_string(),
            description: pos.message.clone(),
            strong: false,
        });
    }

    PositiveSignalsBlock { items }
}

pub(super) fn build_executive_narrative(
    i18n: &I18n,
    normalized: &NormalizedReport,
    audit_summary: &crate::audit::summary::AuditSummary,
    severity: &SeverityBlock,
    top_findings: &[FindingGroup],
) -> ExecutiveNarrativeBlock {
    let dominant_share = audit_summary
        .dominant_issue
        .as_ref()
        .map(|d| d.share_pct.round() as usize);
    let key_points = build_single_key_points_text(
        i18n.locale(),
        severity,
        top_findings,
        normalized,
        dominant_share,
    );

    ExecutiveNarrativeBlock {
        cover_eyebrow: i18n.t("narrative-cover-eyebrow"),
        cover_kicker: i18n.t("narrative-cover-kicker"),
        key_points,
        next_steps_callout_body: i18n.t("narrative-next-steps-callout-body"),
    }
}

fn build_single_key_points_text(
    locale: &str,
    severity: &SeverityBlock,
    top_findings: &[FindingGroup],
    normalized: &NormalizedReport,
    dominant_share: Option<usize>,
) -> Vec<String> {
    let en = locale == "en";
    let mut points = Vec::with_capacity(3);
    if severity.critical > 0 {
        if en {
            points.push("Multiple critical WCAG barriers block assistive technology users — screen readers and keyboard navigation are affected.".to_string());
        } else {
            points.push("Mehrere kritische WCAG-Barrieren blockieren Nutzer von Hilfstechnologien — Screenreader und Tastaturnavigation sind betroffen.".to_string());
        }
    } else if severity.high > 0 {
        if en {
            points.push("Significant accessibility gaps reduce usability for users with disabilities — no complete access barriers, but relevant friction points.".to_string());
        } else {
            points.push("Deutliche Barrierefreiheitslücken erschweren die Nutzung für Menschen mit Behinderungen — keine vollständigen Zugangssperren, aber relevante Reibungspunkte.".to_string());
        }
    }

    if let Some(top) = top_findings.first() {
        let total_ch = (severity.critical + severity.high) as usize;
        // Prefer the canonical dominant-issue share so this line and the
        // leverage text never disagree on the same percentage (#360).
        let share = dominant_share.unwrap_or_else(|| {
            (top.occurrence_count * 100)
                .checked_div(total_ch)
                .unwrap_or(0)
        });
        if share >= 30 {
            if en {
                points.push(format!(
                    "Main issue: {} ({}% of all critical/high findings)",
                    top.title,
                    share.min(99)
                ));
            } else {
                points.push(format!(
                    "Hauptproblem: {} ({}% aller kritischen/hohen Befunde)",
                    top.title,
                    share.min(99)
                ));
            }
        } else if en {
            points.push(format!("Most frequent issue: {}", top.title));
        } else {
            points.push(format!("Häufigstes Problem: {}", top.title));
        }
    }

    let score_driver_note = build_score_driver_note(locale, normalized);
    if !score_driver_note.is_empty() {
        points.push(score_driver_note);
    }

    if normalized.risk.legal_flags > 0 {
        if en {
            points.push(
                "High or critical WCAG Level A findings detected automatically — manual review needed for a defensible BFSG classification".to_string(),
            );
        } else {
            points.push(
                "Hohe oder kritische WCAG-Level-A-Befunde automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig".to_string(),
            );
        }
    } else if severity.high > 0 {
        if en {
            points.push("No Level A violations, but structural weaknesses".to_string());
        } else {
            points.push(
                "Keine Level-A-Verstöße, aber strukturelle Optimierungspotenziale".to_string(),
            );
        }
    } else if !normalized.audit_flags.is_empty() {
        if en {
            points.push(
                "Audit notes present — individual signals should be verified manually.".to_string(),
            );
        } else {
            points.push(
                "Audit-Hinweise vorhanden — einzelne Signale sollten fachlich gegengeprüft werden."
                    .to_string(),
            );
        }
    } else if en {
        points.push(
            "No automatically detectable critical barriers — manual review recommended."
                .to_string(),
        );
    } else {
        points.push(
            "Keine automatisiert erkennbaren kritischen Barrieren — manuelle Prüfung empfohlen."
                .to_string(),
        );
    }

    let (automated, total) = crate::wcag::coverage::coverage_stats();
    if en {
        points.push(format!(
            "WCAG scope: {automated} of about {total} WCAG 2.1 AA criteria are checked automatically; manual review remains required for context-dependent criteria."
        ));
    } else {
        points.push(format!(
            "WCAG-Prüfumfang: {automated} von ca. {total} WCAG-2.1-AA-Kriterien werden automatisch geprüft; kontextabhängige Kriterien bleiben manuell zu prüfen."
        ));
    }

    if let Some(flag) = normalized
        .audit_flags
        .iter()
        .find(|f| f.kind == "viewport_gap")
    {
        points.push(flag.message.clone());
    }

    points
}

fn build_score_driver_note(locale: &str, normalized: &NormalizedReport) -> String {
    let en = locale == "en";
    let mut drivers: std::collections::BTreeMap<&'static str, u32> =
        std::collections::BTreeMap::new();
    for finding in &normalized.findings {
        let severity_weight = match finding.severity {
            crate::wcag::Severity::Critical => 20,
            crate::wcag::Severity::High => 14,
            crate::wcag::Severity::Medium => 8,
            crate::wcag::Severity::Low => 4,
        };
        *drivers
            .entry(score_area_for_key_point(finding))
            .or_default() += severity_weight * finding.occurrence_count as u32;
    }
    let mut drivers: Vec<_> = drivers.into_iter().filter(|(_, loss)| *loss > 0).collect();
    drivers.sort_by_key(|(_, loss)| std::cmp::Reverse(*loss));
    let labels: Vec<_> = drivers.into_iter().take(3).map(|(area, _)| area).collect();
    if labels.is_empty() {
        if en {
            "Score matrix: no negative driver detected in the weighted accessibility topics."
                .to_string()
        } else {
            "Score-Matrix: kein negativer Haupttreiber in den gewichteten Accessibility-Themen erkannt."
                .to_string()
        }
    } else if en {
        format!(
            "Score matrix: strongest negative drivers are {}; weighting includes semantics, forms, keyboard, focus, images, ARIA, headings and landmarks.",
            labels.join(", ")
        )
    } else {
        format!(
            "Score-Matrix: stärkste negative Treiber sind {}; gewichtet werden Semantik, Formulare, Tastatur, Fokus, Bilder, ARIA, Überschriften und Landmarks.",
            labels.join(", ")
        )
    }
}

fn score_area_for_key_point(finding: &crate::audit::normalized::NormalizedFinding) -> &'static str {
    // Use the German subcategory label and German taxonomy title so the mixed
    // DE/EN token matching below keeps the exact behavior it had before the
    // stored JSON title became canonical English (#406).
    let title_de = crate::taxonomy::RuleLookup::by_id(&finding.rule_id)
        .map(|r| r.title)
        .unwrap_or(finding.title.as_str());
    let key = format!(
        "{} {} {} {}",
        finding.rule_id.to_ascii_lowercase(),
        finding.subcategory_kind.label(false).to_ascii_lowercase(),
        title_de.to_ascii_lowercase(),
        finding.description.to_ascii_lowercase()
    );
    if key.contains("form") || key.contains("label") || key.contains("input") {
        "Forms"
    } else if key.contains("keyboard") || key.contains("tastatur") {
        "Keyboard"
    } else if key.contains("focus") || key.contains("fokus") {
        "Focus management"
    } else if key.contains("alt") || key.contains("image") || key.contains("bild") {
        "Images / alternative text"
    } else if key.contains("aria") || key.contains("role") {
        "ARIA"
    } else if key.contains("heading") || key.contains("überschrift") || key.contains("h1") {
        "Heading structure"
    } else if key.contains("landmark") || key.contains("main") || key.contains("navigation") {
        "Landmarks / page structure"
    } else {
        "Semantics"
    }
}
