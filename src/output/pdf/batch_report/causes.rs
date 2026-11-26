//! Shared and page-specific causes, and the next step of the batch PDF
//! (#1212): one compact card per finding type with the largest effect and the
//! concrete recommendation of a single report in place of a generic list of
//! next steps.

use crate::output::localized::is_english;
use fluent_bundle::FluentValue;
use renderreport::components::advanced::{KeyValueList, SectionHeaderSplit, SoftPageBreak};
use renderreport::components::text::TextBlock;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

use super::super::findings::report_code_label;
use super::super::helpers::{effort_short_label_i18n, priority_label_i18n, role_label_i18n};
use super::actions::batch_reach_label;
use super::compare::{page_label, url_path};
use super::management::{batch_cause_text, batch_score_effect};

/// How many finding types get a card; the rest is in the JSON report.
const CAUSE_CARDS: usize = 5;

/// The share of a page a section opener (heading, intro and the first block)
/// needs; with less left the section starts on a new page.
pub(super) const SECTION_OPENER_ROOM: f64 = 0.30;

/// How far a finding type reaches: its occurrences and the pages, a
/// single-page type with the path of its page.
fn reach_text(group: &FindingGroup, total_urls: usize, i18n: &I18n) -> String {
    let urls = group.affected_urls.len();
    let reach = batch_reach_label(urls, total_urls, i18n);
    let reach = match group
        .affected_urls
        .first()
        .filter(|_| urls == 1 && total_urls > 1)
    {
        Some(url) => i18n.t_args(
            "batch-causes-reach-page",
            &[
                ("reach", FluentValue::from(reach)),
                ("page", FluentValue::from(url_path(url))),
            ],
        ),
        None => reach,
    };
    format!(
        "{} · {reach}",
        i18n.count("batch-meta-occurrences", group.finding.occurrence_count)
    )
}

/// One card: reach, cause and how sure the report is about it, the measure,
/// priority with effort and owner and the score effect; the finding id is in
/// the title. What the problem is and whom it affects is the title and the
/// single report.
fn cause_card(group: &FindingGroup, pres: &BatchPresentation, i18n: &I18n) -> KeyValueList {
    // The finding id, which links the card to the JSON report, stands in the
    // title: a row of its own cost a line per card.
    let title = format!("{} · {}", group.title, group.finding.id);
    let mut kv = KeyValueList::new().with_title(&title).add(
        i18n.t("pdf-analysis-card-key-reach"),
        reach_text(group, pres.portfolio_summary.total_urls, i18n),
    );
    kv = kv.add(
        i18n.t("pdf-analysis-card-key-cause"),
        batch_cause_text(group, pres, i18n),
    );
    // Contrast measured under opacity: name the cause, not only the ratio
    // (#1112). The provider is named once on the cross-page card, not once per
    // URL (#1109).
    if let Some(opacity) =
        super::super::findings::opacity_pattern_text(&group.finding.occurrences, is_english(i18n))
    {
        kv = kv.add(i18n.t("pdf-findings-opacity"), opacity);
    }
    if let Some(origin) = super::super::findings::origin_text(&group.finding, i18n) {
        kv = kv.add(i18n.t("pdf-findings-origin"), origin);
    }
    // The measure; a rule without one is told by its problem line.
    if !group.recommendation.trim().is_empty() {
        kv = kv.add(i18n.t("pdf-analysis-card-key-fix"), &group.recommendation);
    } else if !group.customer_description.trim().is_empty() {
        kv = kv
            .add(
                i18n.t("pdf-analysis-card-key-problem"),
                &group.customer_description,
            )
            .add(
                i18n.t("pdf-analysis-card-key-evidence"),
                report_code_label(&group.finding.verification, i18n),
            );
    }
    kv = kv.add(
        i18n.t("pdf-analysis-card-key-priority"),
        i18n.t_args(
            "pdf-analysis-card-priority-effort",
            &[
                ("priority", priority_label_i18n(group.priority, i18n)),
                ("effort", effort_short_label_i18n(group.effort, i18n)),
                ("role", role_label_i18n(group.responsible_role, i18n)),
            ],
        ),
    );
    if let Some(effect) = batch_score_effect(group, i18n) {
        kv = kv.add(i18n.t("pdf-analysis-card-key-score-effect"), effect);
    }
    kv
}

pub(super) fn render_batch_causes(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // The heading and its intro never end a page with the first card on the
    // next: with less than a third of the page left the section starts a new one.
    builder = builder.add_component(SoftPageBreak::needs(SECTION_OPENER_ROOM));
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-causes-title"), i18n.t("batch-causes-intro"))
            .with_level(1),
    );

    let carded: Vec<&FindingGroup> = pres.top_issues.iter().take(CAUSE_CARDS).collect();
    for group in &carded {
        builder = builder.add_component(cause_card(group, pres, i18n));
    }

    // The finding types beyond the cards are in the JSON report: with their
    // ids, reach and priority they are a list, not an overview.
    let not_shown = pres.finding_spread.types.saturating_sub(carded.len());
    if not_shown > 0 {
        builder = builder.add_component(TextBlock::new(
            i18n.count("batch-causes-more-json", not_shown),
        ));
    }

    builder
}

/// The next step: the single report of the recommended page and how to
/// create it. Without a recommended page there is no step to name.
pub(super) fn render_batch_next_step(
    builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let Some(pick) = &pres.single_report_pick else {
        return builder;
    };
    let mut body = i18n.t_args(
        "batch-next-body",
        &[
            ("page", FluentValue::from(page_label(pres, &pick.url))),
            (
                "command",
                FluentValue::from(format!("auditmysite {} -f pdf", pick.url)),
            ),
        ],
    );
    if let Some(lead) = super::glance::pick_lead_text(pick, pres, i18n) {
        body.push(' ');
        body.push_str(&lead);
    }
    builder.add_component(Callout::info(body).with_title(i18n.t("batch-next-title")))
}
