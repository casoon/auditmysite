//! The page comparison of the batch PDF (#1212): the audited pages with title
//! and path (the domain is the same on every row), the accessibility score,
//! the findings and the reason a page deserves a single report; the module
//! averages below. The scores of every module per page are in the technical
//! part.

use renderreport::components::advanced::SectionHeaderSplit;
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, TableColumn};

use crate::i18n::I18n;
use crate::output::report_model::*;

use super::portfolio::leading_items;

/// How many of the lowest and the highest scoring pages the comparison shows
/// once there are more pages than both edges fill.
const COMPARE_EDGE: usize = 5;

/// The path of a page URL (with its query), the part that tells the pages of
/// one domain apart; long paths keep their last segment.
pub(super) fn url_path(url: &str) -> String {
    let bare = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let path = bare.find('/').map_or("/", |at| &bare[at..]);
    if path.chars().count() <= 46 {
        return path.to_string();
    }
    let shortened = crate::util::short_url(&format!("h{path}"), 46);
    shortened
        .strip_prefix('h')
        .unwrap_or(&shortened)
        .to_string()
}

/// The suffix every titled page ends with ("… - Company"): the site name, not
/// what tells the pages apart.
fn common_title_suffix(titles: &[&str]) -> Option<String> {
    if titles.len() < 2 {
        return None;
    }
    [" - ", " | ", " – ", " — ", " · "].iter().find_map(|sep| {
        let suffixes: Vec<&str> = titles
            .iter()
            .filter_map(|title| title.rfind(sep).map(|at| &title[at..]))
            .collect();
        (suffixes.len() == titles.len() && suffixes.iter().all(|s| *s == suffixes[0]))
            .then(|| suffixes[0].to_string())
    })
}

/// The title of an audited page without the suffix all pages share; `None`
/// when the page has no title.
fn page_title(pres: &BatchPresentation, url: &str) -> Option<String> {
    let titles: Vec<&str> = pres
        .url_matrix
        .iter()
        .filter_map(|row| row.title.as_deref())
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .collect();
    let suffix = common_title_suffix(&titles);
    let title = pres
        .url_matrix
        .iter()
        .find(|row| row.url == url)?
        .title
        .as_deref()?
        .trim();
    let title = suffix
        .as_deref()
        .and_then(|suffix| title.strip_suffix(suffix))
        .unwrap_or(title);
    (!title.is_empty()).then(|| crate::util::truncate_words(title, 56))
}

/// "Title (/path)" of a page, the path alone when the page has no title.
pub(super) fn page_label(pres: &BatchPresentation, url: &str) -> String {
    let path = url_path(url);
    match page_title(pres, url) {
        Some(title) => format!("{title} ({path})"),
        None => path,
    }
}

/// Why a page deserves a single report: its finding types and what only
/// this page shows.
fn focus_text(focus: &PageFocus, i18n: &I18n) -> String {
    if focus.types == 0 {
        return i18n.t("batch-compare-focus-none");
    }
    let mut text = i18n.t_args(
        "batch-compare-focus",
        &[
            ("types", focus.types as i64),
            ("severe", focus.severe_types as i64),
        ],
    );
    if !focus.only_here.is_empty() {
        let titles: Vec<String> = focus.only_here.iter().map(|t| t.title.clone()).collect();
        text.push_str(&i18n.t_args(
            "batch-compare-focus-only",
            &[("list", leading_items(&titles, 2, i18n))],
        ));
    }
    text
}

/// The rows of the comparison: every page, or the lowest and the highest
/// scoring ones when there are more than both edges fill.
pub(super) fn compared_pages(pres: &BatchPresentation) -> (Vec<usize>, usize) {
    let total = pres.url_ranking.len();
    if total <= 2 * COMPARE_EDGE {
        return ((0..total).collect(), 0);
    }
    let shown: Vec<usize> = (0..COMPARE_EDGE)
        .chain(total - COMPARE_EDGE..total)
        .collect();
    let left_out = total - shown.len();
    (shown, left_out)
}

pub(super) fn render_batch_page_comparison(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-compare-title"), i18n.t("batch-compare-intro"))
            .with_level(1),
    );

    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("batch-compare-col-rank")).with_width("6%"),
        TableColumn::new(i18n.t("batch-compare-col-page")).with_width("32%"),
        TableColumn::new(i18n.t("batch-compare-col-accessibility")).with_width("15%"),
        TableColumn::new(i18n.t("batch-compare-col-severe")).with_width("15%"),
        TableColumn::new(i18n.t("batch-compare-col-focus")).with_width("32%"),
    ]);
    let (shown, left_out) = compared_pages(pres);
    let picked = pres
        .single_report_pick
        .as_ref()
        .map(|pick| pick.url.as_str());
    for i in shown {
        let page = &pres.url_ranking[i];
        let focus = pres
            .page_focus
            .iter()
            .find(|focus| focus.url == page.url)
            .map(|focus| focus_text(focus, i18n))
            .unwrap_or_default();
        let focus = if picked == Some(page.url.as_str()) {
            i18n.t_args("batch-compare-pick", &[("text", focus)])
        } else {
            focus
        };
        table = table.add_row(vec![
            (i + 1).to_string(),
            page_label(pres, &page.url),
            format!("{}", page.score.round() as u32),
            page.critical_violations.to_string(),
            focus,
        ]);
    }
    builder = builder.add_component(table);
    if left_out > 0 {
        builder = builder.add_component(TextBlock::new(i18n.count("batch-compare-more", left_out)));
    }

    builder
}
