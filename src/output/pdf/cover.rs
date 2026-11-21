//! Cover page components for PDF reports.

use std::{env, fs, path::PathBuf};

use renderreport::components::advanced::Grid;
use renderreport::components::{MetricCard, ScoreCard};
use renderreport::prelude::*;

use crate::audit::interpretation::ScoreBand;
use crate::i18n::I18n;

use super::design::tokens;

/// One-line "technical condition" phrase for the cover.
///
/// Derived from the score alone this contradicted the run's own verdict: on
/// a corporate site (verdict `fail`, 5 legal flags)
/// the cover read "Ausbaufähiger technischer Zustand". A failing
/// run now says so on the cover, and a partial run says that it is
/// provisional.
pub(super) fn cover_band_phrase(
    score: u32,
    verdict: crate::audit::verdict::Verdict,
    partial: bool,
    en: bool,
) -> String {
    use crate::audit::verdict::Verdict;
    let base = match verdict {
        Verdict::Fail => crate::i18n::text("pdf-sections-cover-fail", en),
        // Warn and Pass keep the score-derived phrase: the score band is the
        // more informative statement when nothing blocks.
        Verdict::Warn | Verdict::Pass => crate::registry::COVER_PHRASE
            .label(score as f32, en)
            .to_string(),
    };
    if partial {
        format!(
            "{base} · {}",
            crate::i18n::text("pdf-sections-cover-provisional", en)
        )
    } else {
        base
    }
}

pub(super) fn build_batch_cover_score_row(
    avg_score: u32,
    // The grade the presentation already derived. Recomputing it here meant a
    // third band table (`BATCH_GRADE`, 95/90/80/70/60) disagreeing with the
    // one the JSON reports above 95.
    grade: &str,
    // The batch decision: a failing batch says so under its grade, as the
    // single cover does (#1039).
    verdict: crate::audit::verdict::Verdict,
    partial: bool,
    total_urls: u32,
    passed_urls: u32,
    i18n: &I18n,
) -> anyhow::Result<Grid> {
    let en = i18n.locale() == "en";
    let mut grid = Grid::new(3)
        .with_item_min_height("142pt")
        .add_item(serde_json::json!({
            "type": "metric-card",
            "data": MetricCard::new(i18n.t("cover-card-grade"), grade)
                .with_subtitle(cover_band_phrase(avg_score, verdict, partial, en))
                .with_accent_color(score_accent_color(avg_score))
                .with_height("100%")
                .to_data()
        }));

    grid = grid.add_item(serde_json::json!({
        "type": "score-card",
        // The note sits below the row (overview.rs): the grid item has a
        // fixed height, and in the card it ran into the paragraph below.
        "data": ScoreCard::new(i18n.t("cover-card-average"), avg_score)
            .with_thresholds(70, 50)
            .with_height("100%")
            .to_data()
    }));

    Ok(grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new(i18n.t("cover-card-urls"), total_urls.to_string())
            // The card counts URLs, so it says how many of them passed; the
            // WCAG occurrences have their own cards (#1039).
            .with_subtitle(i18n.t_args(
                "cover-card-urls-passed",
                &[
                    ("passed", fluent_bundle::FluentValue::from(passed_urls as i64)),
                    ("total", fluent_bundle::FluentValue::from(total_urls as i64)),
                ],
            ))
            .with_accent_color(if passed_urls >= total_urls {
                tokens::SUCCESS
            } else if passed_urls == 0 {
                tokens::DANGER
            } else {
                tokens::WARN_DEEP
            })
            .with_height("100%")
            .to_data()
    })))
}

pub(super) fn auditmysite_wordmark_path() -> anyhow::Result<String> {
    let path: PathBuf = env::temp_dir().join("auditmysite-wordmark.svg");
    fs::write(
        &path,
        include_str!("../../../assets/brand/auditmysite-wordmark.svg"),
    )?;
    Ok(path.to_string_lossy().to_string())
}

/// Accent colour of the grade card: the score band (90/75/60/40) as colour.
pub(super) fn score_accent_color(score: u32) -> &'static str {
    match ScoreBand::from_score(score as f32) {
        ScoreBand::Excellent | ScoreBand::Good => tokens::SUCCESS,
        ScoreBand::NeedsImprovement => tokens::NEUTRAL,
        ScoreBand::Weak => tokens::WARN_DEEP,
        ScoreBand::Critical => tokens::DANGER,
    }
}
