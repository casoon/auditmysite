//! Cover page components for PDF reports.

use std::{env, fs, path::PathBuf};

use renderreport::components::advanced::Grid;
use renderreport::components::{MetricCard, ScoreCard};
use renderreport::prelude::Image;
use renderreport::prelude::*;

use crate::i18n::I18n;

use super::design::tokens;

/// One-line "technical condition" phrase for the dominant cover score, aligned
/// with the grade bands (Sehr gut ≥ 90 … Kritisch < 40). Localized de/en.
pub(super) fn cover_band_phrase(score: u32, en: bool) -> &'static str {
    match (score, en) {
        (90..=u32::MAX, true) => "Excellent technical condition",
        (75..=89, true) => "Good technical condition",
        (60..=74, true) => "Technical condition needs improvement",
        (40..=59, true) => "Inadequate technical condition",
        (_, true) => "Critical technical condition",
        (90..=u32::MAX, false) => "Sehr guter technischer Zustand",
        (75..=89, false) => "Guter technischer Zustand",
        (60..=74, false) => "Verbesserungswürdiger Zustand",
        (40..=59, false) => "Ausbaufähiger technischer Zustand",
        (_, false) => "Kritischer technischer Zustand",
    }
}

pub(super) fn build_batch_cover_score_row(
    avg_score: u32,
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
            "data": MetricCard::new(i18n.t("cover-card-certificate"), batch_grade_label(avg_score))
                .with_subtitle(format!("{} • {} / 100", certificate_label_localized(batch_certificate_label(avg_score), i18n.locale()), avg_score))
                .with_accent_color(certificate_accent_color(batch_certificate_label(avg_score)))
                .with_height("100%")
                .to_data()
        }));
    }

    grid = grid.add_item(serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new(i18n.t("cover-card-average"), avg_score)
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

pub(super) fn batch_grade_label(score: u32) -> &'static str {
    match score {
        95..=100 => "A+",
        90..=94 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
}

pub(super) fn batch_certificate_label(score: u32) -> &'static str {
    match score {
        90..=100 => "SEHR GUT",
        75..=89 => "GUT",
        60..=74 => "STABIL",
        40..=59 => "AUSBAUFÄHIG",
        _ => "UNGENÜGEND",
    }
}
