use super::*;

pub(in crate::output::pdf) fn render_a11y_journey_findings(
    mut builder: renderreport::engine::ReportBuilder,
    findings: &[crate::audit::normalized::InteractiveFinding],
    journey: Option<&crate::audit::normalized::AccessibilityJourney>,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    builder = builder.add_component(PageBreak::new()).add_component(
        Section::new(if en {
            "Keyboard Accessibility Journey"
        } else {
            "Tastatur-Accessibility-Journey"
        })
        .with_level(2),
    );

    // Collapse identical journey findings (same journey, category, message and
    // severity) into a single row with an occurrence count. Repeated identical
    // entries otherwise read like a render-loop bug (#358). Distinct elements
    // carry distinct messages and are never merged.
    let mut deduped: Vec<(&crate::audit::normalized::InteractiveFinding, usize)> = Vec::new();
    for f in findings {
        if let Some(entry) = deduped.iter_mut().find(|(g, _)| {
            g.journey == f.journey
                && g.category == f.category
                && g.message == f.message
                && g.severity == f.severity
        }) {
            entry.1 += 1;
        } else {
            deduped.push((f, 1));
        }
    }

    let critical_count = deduped
        .iter()
        .filter(|(f, _)| f.severity == crate::taxonomy::Severity::Critical)
        .count();
    let high_count = deduped
        .iter()
        .filter(|(f, _)| f.severity == crate::taxonomy::Severity::High)
        .count();
    let overview = if critical_count > 0 {
        if en {
            format!(
                "{} critical and {} high issues detected during keyboard and state-transition tests.",
                critical_count, high_count
            )
        } else {
            format!(
                "{} kritische und {} hohe Befunde aus Tastatur- und Zustandswechsel-Tests.",
                critical_count, high_count
            )
        }
    } else if high_count > 0 {
        if en {
            format!("{} high-severity interactive issues found.", high_count)
        } else {
            format!(
                "{} interaktive Befunde mit hoher Schwere gefunden.",
                high_count
            )
        }
    } else if en {
        format!(
            "{} minor interactive issues — no critical or high barriers detected.",
            deduped.len()
        )
    } else {
        format!(
            "{} kleinere interaktive Befunde — keine kritischen oder hohen Barrieren erkannt.",
            deduped.len()
        )
    };
    builder = builder.add_component(if critical_count > 0 {
        Callout::warning(&overview)
    } else if high_count > 0 {
        Callout::info(&overview)
    } else {
        Callout::success(&overview)
    });

    let shown = deduped.len().min(10);
    for (finding, count) in &deduped[..shown] {
        let sev = map_severity(&finding.severity);
        let body = if let Some(ref fix) = finding.fix_suggestion {
            format!("{} — {}", finding.message, fix)
        } else {
            finding.message.clone()
        };
        let label = journey_category_label(&finding.category, i18n);
        let title = if *count > 1 {
            format!("{label} (×{count})")
        } else {
            label
        };
        builder = builder.add_component(Finding::new(&title, sev, &body));
    }
    if deduped.len() > 10 {
        let more = if en {
            format!(
                "{} additional interactive findings in the JSON report.",
                deduped.len() - 10
            )
        } else {
            format!(
                "{} weitere interaktive Befunde im JSON-Report.",
                deduped.len() - 10
            )
        };
        builder = builder.add_component(Callout::info(&more));
    }

    if let Some(journey_data) = journey {
        if !journey_data.traces.is_empty() {
            let mut kv = KeyValueList::new().with_title(if en {
                "Tested Journey Sequences"
            } else {
                "Geprüfte Journey-Sequenzen"
            });
            for trace in &journey_data.traces {
                let step_count = trace.steps.len();
                let summary = format!("{} {}", step_count, if en { "steps" } else { "Schritte" });
                kv = kv.add(&trace.journey, summary);
            }
            builder = builder.add_component(kv);
        }
    }

    let disclaimer = if en {
        "These tests check whether browser, DOM, focus, and accessibility tree provide a robust foundation for screen reader use. They do not simulate the exact output of NVDA, JAWS, or VoiceOver."
    } else {
        "Diese Tests prüfen, ob Browser, DOM, Fokus und Accessibility Tree eine robuste Grundlage für Screenreader-Nutzung liefern. Sie simulieren nicht den exakten Output von NVDA, JAWS oder VoiceOver."
    };
    builder = builder.add_component(Callout::info(disclaimer));

    builder
}

/// Screen-reader reading-order audit section (#411). Renders the quality scores,
/// the BFSG verdict and the detected issues. The full reading sequence stays in
/// the sidecar JSON; this section surfaces the actionable summary in the report.
pub(in crate::output::pdf) fn render_screen_reader_section(
    mut builder: renderreport::engine::ReportBuilder,
    sr: &crate::screen_reader::SrAuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::screen_reader::BfsgVerdict;
    let en = i18n.locale() == "en";

    builder = builder.add_component(PageBreak::new()).add_component(
        Section::new(if en {
            "Screen Reader Reading Order"
        } else {
            "Screenreader-Lesereihenfolge"
        })
        .with_level(2),
    );

    let intro = if en {
        "How the page is announced to screen reader users: reading order, landmark and heading quality, and accessible names. These are structural checks on the accessibility tree, not a simulation of NVDA, JAWS or VoiceOver."
    } else {
        "Wie die Seite Screenreader-Nutzern angekündigt wird: Lesereihenfolge, Landmark- und Heading-Qualität sowie zugängliche Namen. Strukturelle Prüfungen am Accessibility Tree, keine Simulation von NVDA, JAWS oder VoiceOver."
    };
    builder = builder.add_component(Label::new(intro).with_size("10.5pt").with_color("#475569"));

    let s = &sr.summary;
    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new(
                if en {
                    "Heading quality"
                } else {
                    "Heading-Qualität"
                },
                format!("{}/100", s.heading_quality_score),
            )
            .with_accent("#0f766e"),
            MetricStripItem::new(
                if en {
                    "Landmark quality"
                } else {
                    "Landmark-Qualität"
                },
                format!("{}/100", s.landmark_quality_score),
            )
            .with_accent("#2563eb"),
            MetricStripItem::new(
                if en {
                    "Name quality"
                } else {
                    "Namens-Qualität"
                },
                format!("{}/100", s.name_quality_score),
            )
            .with_accent("#7c3aed"),
            MetricStripItem::new(
                if en {
                    "Announced nodes"
                } else {
                    "Angekündigte Knoten"
                },
                s.total_announced_nodes.to_string(),
            ),
            MetricStripItem::new(
                if en { "Tab stops" } else { "Tab-Stopps" },
                s.tab_stops.to_string(),
            ),
        ])
        .compact(),
    );

    let bfsg_note = match sr.bfsg_compliance.verdict {
        BfsgVerdict::Compliant => Callout::success(if en {
            "BFSG check: no violations in the evaluated scope."
        } else {
            "BFSG-Prüfung: keine Verstöße im geprüften Umfang."
        }),
        BfsgVerdict::NonCompliant => Callout::warning(if en {
            "BFSG check: violations detected in the evaluated scope (see issues below)."
        } else {
            "BFSG-Prüfung: Verstöße im geprüften Umfang festgestellt (siehe Befunde unten)."
        }),
        BfsgVerdict::NotEvaluated => Callout::info(if en {
            "BFSG conformance was not evaluated for this page."
        } else {
            "BFSG-Konformität wurde für diese Seite nicht bewertet."
        }),
    };
    builder = builder.add_component(bfsg_note);

    // Re-derive issues in the run language. The stored `sr.issues` are baked in
    // canonical English (#406); the PDF localizes them by recomputing from the
    // reading sequence and navigation views.
    let items: Vec<crate::screen_reader::ReadingItem> =
        sr.reading_sequence.iter().map(|a| a.item.clone()).collect();
    let localized_issues = crate::screen_reader::analyze_reading_sequence(
        &items,
        &sr.navigation_views,
        i18n.locale(),
        i18n.locale() == "en",
    );

    if !localized_issues.is_empty() {
        // Collapse identical messages into one row with an occurrence count.
        let mut deduped: Vec<(&crate::screen_reader::SrAuditIssue, usize)> = Vec::new();
        for issue in &localized_issues {
            if let Some(entry) = deduped
                .iter_mut()
                .find(|(g, _)| g.message == issue.message && g.severity == issue.severity)
            {
                entry.1 += 1;
            } else {
                deduped.push((issue, 1));
            }
        }

        let sev_label = |sev: &str| -> &'static str {
            match (sev, en) {
                ("high", true) => "High",
                ("high", false) => "Hoch",
                ("medium", true) => "Medium",
                ("medium", false) => "Mittel",
                (_, true) => "Low",
                (_, false) => "Niedrig",
            }
        };

        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("label-issue")).with_width("75%"),
            TableColumn::new(i18n.t("label-severity")).with_width("25%"),
        ])
        .with_title(if en {
            "Screen reader findings"
        } else {
            "Screenreader-Befunde"
        });
        for (issue, count) in deduped {
            let msg = if count > 1 {
                format!("{} ({}×)", issue.message, count)
            } else {
                issue.message.clone()
            };
            table = table.add_row(vec![msg, sev_label(&issue.severity).to_string()]);
        }
        builder = builder.add_component(table);
    }

    builder
}
