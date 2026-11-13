use super::*;

#[cfg(test)]
mod text_export;
#[cfg(test)]
mod text_guard;

pub(in crate::output::pdf) fn render_a11y_journey_findings(
    mut builder: renderreport::engine::ReportBuilder,
    findings: &[crate::audit::normalized::InteractiveFinding],
    journey: Option<&crate::audit::normalized::AccessibilityJourney>,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);

    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("a11y-details-journey-title")).with_level(2));

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
                && g.uncertainty == f.uncertainty
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
        i18n.t_args(
            "a11y-details-journey-overview-critical",
            &[
                ("critical", critical_count as i64),
                ("high", high_count as i64),
            ],
        )
    } else if high_count > 0 {
        i18n.count("a11y-details-journey-overview-high", high_count)
    } else {
        i18n.count("a11y-details-journey-overview-minor", deduped.len())
    };
    // Never a success callout: this section only renders when there *are*
    // interactive findings, so green read as "all clear" directly above the
    // findings it was summarizing.
    builder = builder.add_component(if critical_count > 0 {
        Callout::warning(&overview)
    } else {
        Callout::info(&overview)
    });

    let shown = deduped.len().min(10);
    for (finding, count) in &deduped[..shown] {
        let sev = map_severity(&finding.severity);
        // Re-derive message/fix_suggestion in the run language. The stored
        // `finding.message`/`finding.fix_suggestion` are baked in canonical
        // English (#406); the PDF localizes them from `kind` + `values`.
        let (message, fix_suggestion) =
            crate::audit::normalized::interactive_finding_text(finding.kind, &finding.values, en);
        let mut body = if let Some(fix) = fix_suggestion {
            format!("{} — {}", message, fix)
        } else {
            message
        };
        if let Some(uncertainty) = &finding.uncertainty {
            body.push(' ');
            body.push_str(&crate::audit::normalized::finding_uncertainty_text(
                uncertainty,
                en,
            ));
        }
        let label = journey_category_label(&finding.category, i18n);
        let title = if *count > 1 {
            format!("{label} (×{count})")
        } else {
            label
        };
        builder = builder.add_component(Finding::new(&title, sev, &body));
    }
    if deduped.len() > 10 {
        let more = i18n.count("a11y-details-journey-more", deduped.len() - 10);
        builder = builder.add_component(Callout::info(&more));
    }

    if let Some(journey_data) = journey {
        if !journey_data.traces.is_empty() {
            let mut kv = KeyValueList::new().with_title(i18n.t("a11y-details-journey-sequences"));
            for trace in &journey_data.traces {
                let step_count = trace.steps.len();
                let summary = i18n.count("a11y-details-journey-steps", step_count);
                kv = kv.add(journey_label(&trace.journey, en), summary);
            }
            builder = builder.add_component(kv);
        }
    }

    builder = builder.add_component(Callout::info(i18n.t("a11y-details-journey-disclaimer")));

    builder
}

/// Number of confirmed WCAG findings that carry BFSG/EAA relevance, read from
/// the finding's own `bfsg_relevance` -- the value its card prints. Findings
/// still awaiting a manual check are no confirmed violation.
pub(in crate::output::pdf) fn bfsg_relevant_wcag_findings(
    groups: &[crate::output::report_model::FindingGroup],
) -> usize {
    groups
        .iter()
        .filter(|g| {
            crate::audit::normalized::is_bfsg_relevant_open(
                &g.finding.bfsg_relevance,
                g.finding.occurrence_count,
                &g.finding.verification,
            )
        })
        .count()
}

/// The reading-order audit alone only covers its structural criteria. A
/// BFSG-relevant WCAG violation elsewhere in the report must never sit next to
/// a "no violations" statement (#902-5): it lifts the verdict to
/// `ViolationsFound`; the stricter state of the two wins.
///
/// Reading-order notes alone (Low/Medium heuristics) are no violation: without
/// a BFSG-relevant WCAG finding and without a high-severity screen-reader
/// blocker (the same bar `legal_flags` uses, #484), a `ViolationsFound` from the
/// reading-order audit is shown as "needs review" (#952-2).
pub(in crate::output::pdf) fn effective_bfsg_verdict(
    sr_verdict: crate::screen_reader::BfsgVerdict,
    bfsg_relevant_wcag_findings: usize,
    sr_high_issues: usize,
) -> crate::screen_reader::BfsgVerdict {
    use crate::screen_reader::BfsgVerdict;
    if bfsg_relevant_wcag_findings > 0 {
        BfsgVerdict::ViolationsFound
    } else if sr_verdict == BfsgVerdict::ViolationsFound && sr_high_issues == 0 {
        BfsgVerdict::NeedsReview
    } else {
        sr_verdict
    }
}

/// Screen-reader reading-order audit section (#411). Renders the quality scores,
/// the BFSG verdict and the detected issues. The full reading sequence stays in
/// the sidecar JSON; this section surfaces the actionable summary in the report.
pub(in crate::output::pdf) fn render_screen_reader_section(
    mut builder: renderreport::engine::ReportBuilder,
    sr: &crate::screen_reader::SrAuditReport,
    patterns: Option<&crate::patterns::PatternAnalysis>,
    bfsg_relevant_wcag_findings: usize,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);

    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("a11y-details-sr-title")).with_level(2));

    builder = builder.add_component(
        Label::new(i18n.t("a11y-details-sr-intro"))
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    );

    let s = &sr.summary;
    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new(
                i18n.t("a11y-details-sr-heading-quality"),
                format!("{} / 100", s.heading_quality_score),
            )
            .with_accent(crate::output::pdf::design::score_color(
                s.heading_quality_score as u8,
            )),
            MetricStripItem::new(
                i18n.t("a11y-details-sr-landmark-quality"),
                format!("{} / 100", s.landmark_quality_score),
            )
            .with_accent(crate::output::pdf::design::score_color(
                s.landmark_quality_score as u8,
            )),
            MetricStripItem::new(
                i18n.t("a11y-details-sr-name-quality"),
                format!("{} / 100", s.name_quality_score),
            )
            .with_accent(crate::output::pdf::design::score_color(
                s.name_quality_score as u8,
            )),
            MetricStripItem::new(
                i18n.t("a11y-details-sr-announced-nodes"),
                s.total_announced_nodes.to_string(),
            ),
            MetricStripItem::new(i18n.t("a11y-details-sr-tab-stops"), s.tab_stops.to_string()),
        ])
        .compact(),
    );
    builder = builder.add_component(
        Label::new(i18n.t("a11y-details-sr-score-note"))
            .with_size("9pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    );

    let bfsg_note = bfsg_callout(
        effective_bfsg_verdict(
            sr.bfsg_compliance.verdict,
            bfsg_relevant_wcag_findings,
            sr.issues
                .iter()
                .filter(|i| !i.review && i.severity.eq_ignore_ascii_case("high"))
                .count(),
        ),
        en,
    );
    builder = builder.add_component(bfsg_note);

    // Re-derive issues in the run language. The stored `sr.issues` are baked in
    // canonical English (#406); the PDF localizes them by recomputing from the
    // reading sequence and navigation views.
    let items: Vec<crate::screen_reader::ReadingItem> =
        sr.reading_sequence.iter().map(|a| a.item.clone()).collect();
    let has_disclosure_menu_pattern =
        patterns.is_some_and(|patterns| patterns.has_recognized("DisclosureMenu"));
    let mut localized_issues = crate::screen_reader::analyze_reading_sequence(
        &items,
        &sr.navigation_views,
        i18n.locale(),
        is_english(i18n),
        has_disclosure_menu_pattern,
    );
    // Same audit exclusions as the stored issues (#703).
    crate::screen_reader::drop_excluded_nodes(&mut localized_issues, &sr.excluded_node_ids);

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

        let sev_label = |sev: &str| {
            i18n.t(match sev {
                "high" => "severity-high",
                "medium" => "severity-medium",
                _ => "severity-low",
            })
        };

        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("label-issue")).with_width("75%"),
            TableColumn::new(i18n.t("label-severity")).with_width("25%"),
        ])
        .with_title(i18n.t("a11y-details-sr-findings-title"));
        for (issue, count) in deduped {
            let mut msg = if count > 1 {
                format!("{} ({}×)", issue.message, count)
            } else {
                issue.message.clone()
            };
            if let Some(instruction) = issue
                .wcag_criterion
                .as_deref()
                .and_then(|c| manual_recheck_instruction(c, en))
            {
                msg.push(' ');
                msg.push_str(instruction);
            }
            table = table.add_row(vec![msg, sev_label(&issue.severity)]);
        }
        builder = builder.add_component(table);
    }

    builder
}

/// Human label for an internal journey id such as `tab_walk` or
/// `disclosure_0`. The ids are analysis-internal and were previously printed
/// verbatim into the customer-facing PDF. Unknown ids fall back to the raw id
/// rather than inventing a name for them.
pub(super) fn journey_label(journey: &str, en: bool) -> String {
    let (base, index) = match journey.rsplit_once('_') {
        Some((base, tail)) if tail.chars().all(|c| c.is_ascii_digit()) && !tail.is_empty() => (
            base,
            tail.parse::<usize>()
                .ok()
                .map(|index| index.saturating_add(1)),
        ),
        _ => (journey, None),
    };

    let key = match base {
        "tab_walk" => Some("a11y-details-journey-label-tab-walk"),
        "spa_navigation" => Some("a11y-details-journey-label-spa-navigation"),
        "skip_link" => Some("a11y-details-journey-label-skip-link"),
        "disclosure" => Some("a11y-details-journey-label-disclosure"),
        "menu" => Some("a11y-details-journey-label-menu"),
        "modal" => Some("a11y-details-journey-label-modal"),
        "tabs" => Some("a11y-details-journey-label-tabs"),
        "form_error" => Some("a11y-details-journey-label-form-error"),
        "add_to_cart" => Some("a11y-details-journey-label-add-to-cart"),
        "quantity_stepper" => Some("a11y-details-journey-label-quantity-stepper"),
        _ => None,
    };

    match (key.map(|key| crate::i18n::text(key, en)), index) {
        (Some(label), Some(index)) => format!("{label} {index}"),
        (Some(label), None) => label,
        (None, _) => journey.to_string(),
    }
}

/// Reader-facing BFSG callout. The single place that words the verdict.
pub(in crate::output::pdf) fn bfsg_callout(
    verdict: crate::screen_reader::BfsgVerdict,
    en: bool,
) -> Callout {
    use crate::screen_reader::BfsgVerdict;
    // Deliberately never phrased as a conformance verdict: this check compares
    // technical criteria, it cannot establish whether a service falls under
    // the BFSG at all or whether every legal requirement is met. "No
    // violations" would read as a legal clearance the audit cannot give.
    match verdict {
        BfsgVerdict::NoViolationsFound => {
            Callout::info(crate::i18n::text("a11y-details-bfsg-no-violations", en))
        }
        BfsgVerdict::Incomplete => {
            Callout::warning(crate::i18n::text("a11y-details-bfsg-incomplete", en))
        }
        BfsgVerdict::ViolationsFound => {
            Callout::warning(crate::i18n::text("a11y-details-bfsg-violations", en))
        }
        BfsgVerdict::NeedsReview => {
            Callout::info(crate::i18n::text("a11y-details-bfsg-needs-review", en))
        }
        BfsgVerdict::NotEvaluated => {
            Callout::info(crate::i18n::text("a11y-details-bfsg-not-evaluated", en))
        }
    }
}
