//! Cover page components for PDF reports.

use std::{env, fs, path::PathBuf};

use renderreport::components::advanced::Grid;
use renderreport::components::{MetricCard, ScoreCard};
use renderreport::prelude::Image;
use renderreport::prelude::*;

use crate::i18n::I18n;

use super::design::tokens;

/// One-line "technical condition" phrase for the cover.
///
/// Derived from the score alone this contradicted the run's own verdict: on
/// inros-lackner-de (verdict `fail`, 5 legal flags, rating NICHT BESTANDEN)
/// the cover read "Ausbaufähiger technischer Zustand" (plan 34). A failing
/// run now says so on the cover, and a partial run says that it is
/// provisional (plan 44).
pub(super) fn cover_band_phrase(
    score: u32,
    verdict: crate::audit::verdict::Verdict,
    partial: bool,
    en: bool,
) -> String {
    use crate::audit::verdict::Verdict;
    let base = match verdict {
        Verdict::Fail => {
            if en {
                "Not passed — action required"
            } else {
                "Nicht bestanden — Handlungsbedarf"
            }
        }
        // Warn and Pass keep the score-derived phrase: the score band is the
        // more informative statement when nothing blocks.
        Verdict::Warn | Verdict::Pass => crate::registry::COVER_PHRASE.label(score as f32, en),
    };
    if partial {
        let suffix = if en {
            " · provisional"
        } else {
            " · vorläufig"
        };
        format!("{base}{suffix}")
    } else {
        base.to_string()
    }
}

pub(super) fn build_batch_cover_score_row(
    avg_score: u32,
    // The grade and certificate the presentation already derived. Recomputing
    // them here meant a third band table (`BATCH_GRADE`, 95/90/80/70/60)
    // disagreeing with the one the JSON reports above 95 (plan 29, D1).
    grade: &str,
    certificate: &str,
    total_urls: u32,
    total_violations: u32,
    badge_asset: Option<&str>,
    i18n: &I18n,
) -> anyhow::Result<Grid> {
    let mut grid = Grid::new(3).with_item_min_height("142pt");

    if let Some(asset_name) = badge_asset {
        grid = grid.add_item(serde_json::json!({
            "type": "image",
            "data": Image::new(asset_name).with_width("68%").to_data()
        }));
    } else {
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": MetricCard::new(i18n.t("cover-card-certificate"), grade)
                .with_subtitle(format!("{} • {} / 100", certificate_label_localized(certificate, i18n.locale()), avg_score))
                .with_accent_color(certificate_accent_color(certificate))
                .with_height("100%")
                .to_data()
        }));
    }

    grid = grid.add_item(serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new(i18n.t("cover-card-average"), avg_score)
            .with_description(i18n.t("batch-cover-overall-score-note"))
            .with_thresholds(70, 50)
            .with_height("100%")
            .to_data()
    }));

    Ok(grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new(i18n.t("cover-card-urls"), total_urls.to_string())
            .with_subtitle(format!("{} {}", total_violations, i18n.t("cover-card-violations-suffix")))
            .with_accent_color(tokens::DANGER)
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

/// Localize the canonical (German) certificate token for display. The token
/// stays German internally so badge/colour lookups remain locale-independent;
/// only the rendered label is translated (#449).
pub(super) fn certificate_label_localized(canonical: &str, locale: &str) -> String {
    if locale != "en" {
        return canonical.to_string();
    }
    match canonical {
        "SEHR GUT" => "EXCELLENT",
        "GUT" => "GOOD",
        "STABIL" => "STABLE",
        "AUSBAUFÄHIG" => "INADEQUATE",
        "UNGENÜGEND" => "FAILED",
        "EINGESCHRÄNKT" => "RESTRICTED",
        "NICHT BESTANDEN" => "NOT PASSED",
        other => other,
    }
    .to_string()
}

pub(super) fn certificate_badge_path(certificate: &str) -> anyhow::Result<String> {
    let (filename, svg) = match certificate {
        "SEHR GUT" => (
            "auditmysite-certificate-platinum.svg",
            include_str!("../../../assets/certificates/platinum.svg"),
        ),
        "GUT" => (
            "auditmysite-certificate-gold.svg",
            include_str!("../../../assets/certificates/gold.svg"),
        ),
        "STABIL" => (
            "auditmysite-certificate-silver.svg",
            include_str!("../../../assets/certificates/silver.svg"),
        ),
        "AUSBAUFÄHIG" | "EINGESCHRÄNKT" => (
            "auditmysite-certificate-bronze.svg",
            include_str!("../../../assets/certificates/bronze.svg"),
        ),
        "UNGENÜGEND" | "NICHT BESTANDEN" => (
            "auditmysite-certificate-failed.svg",
            include_str!("../../../assets/certificates/failed.svg"),
        ),
        _ => return Err(anyhow::anyhow!("unknown certificate badge: {certificate}")),
    };

    let path: PathBuf = env::temp_dir().join(filename);
    fs::write(&path, svg)?;

    Ok(path.to_string_lossy().to_string())
}

pub(super) fn certificate_accent_color(certificate: &str) -> &'static str {
    match certificate {
        "SEHR GUT" => tokens::SUCCESS,
        "GUT" => tokens::SUCCESS,
        "STABIL" => tokens::NEUTRAL,
        "AUSBAUFÄHIG" | "EINGESCHRÄNKT" => tokens::WARN_DEEP,
        "UNGENÜGEND" | "NICHT BESTANDEN" => tokens::DANGER,
        _ => tokens::INFO,
    }
}

pub(super) fn batch_certificate_label(score: u32) -> &'static str {
    crate::registry::CERTIFICATE.label(score as f32, false)
}
