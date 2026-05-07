//! Cover page components for PDF reports.

use std::{env, fs, path::PathBuf};

use renderreport::components::advanced::Grid;
use renderreport::components::MetricCard;
use renderreport::prelude::Image;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

use super::design::{score_color, tokens};

pub(super) fn build_cover_score_row(
    cover: &CoverBlock,
    badge_asset: Option<&str>,
    i18n: &I18n,
) -> Grid {
    let mut grid = Grid::new(3).with_item_min_height("142pt");

    if let Some(asset_name) = badge_asset {
        grid = grid.add_item(serde_json::json!({
            "type": "image",
            "data": Image::new(asset_name).with_width("68%").to_data()
        }));
    } else {
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": MetricCard::new(i18n.t("cover-card-certificate"), &cover.grade)
                .with_subtitle(format!("{} • {} / 100", cover.certificate, cover.score))
                .with_accent_color(certificate_accent_color(&cover.certificate))
                .with_height("100%")
                .to_data()
        }));
    }

    grid = grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new(i18n.t("cover-card-accessibility"), format!("{}/100", cover.score))
            .with_subtitle(cover.maturity_label.clone())
            .with_accent_color(score_color(cover.score))
            .with_height("100%")
            .to_data()
    }));

    grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new(i18n.t("cover-card-issues"), cover.total_issues.to_string())
            .with_subtitle(format!("{} {}", cover.critical_issues, i18n.t("cover-card-critical-high-suffix")))
            .with_accent_color(tokens::DANGER)
            .with_height("100%")
            .to_data()
    }))
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
                .with_subtitle(format!("{} • {} / 100", batch_certificate_label(avg_score), avg_score))
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
        "SOLIDE" => (
            "auditmysite-certificate-silver.svg",
            include_str!("../../../assets/certificates/silver.svg"),
        ),
        "AUSBAUFÄHIG" => (
            "auditmysite-certificate-bronze.svg",
            include_str!("../../../assets/certificates/bronze.svg"),
        ),
        "UNGENÜGEND" => (
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
        "GUT" => tokens::ACCENT_BRONZE,
        "SOLIDE" => tokens::NEUTRAL,
        "AUSBAUFÄHIG" => "#9a3412",
        "UNGENÜGEND" => tokens::DANGER,
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
        95..=100 => "SEHR GUT",
        85..=94 => "GUT",
        75..=84 => "SOLIDE",
        65..=74 => "AUSBAUFÄHIG",
        _ => "UNGENÜGEND",
    }
}
