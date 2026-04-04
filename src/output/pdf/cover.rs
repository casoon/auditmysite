//! Cover page components for PDF reports.

use std::{env, fs, path::PathBuf};

use renderreport::components::advanced::Grid;
use renderreport::components::MetricCard;
use renderreport::prelude::Image;
use renderreport::prelude::*;

use crate::output::report_model::*;

pub(super) fn build_cover_meta(cover: &CoverBlock, version: &str) -> SummaryBox {
    SummaryBox::new("Audit-Rahmen")
        .add_item("Domain", &cover.brand)
        .add_item("Prüfdatum", &cover.date)
        .add_item("Ziel", &cover.domain)
        .add_item("Zertifikat", &cover.certificate)
        .add_item("Aktive Module", cover.modules.join(", "))
        .add_item("Tool-Version", format!("auditmysite v{}", version))
}

pub(super) fn build_cover_score_row(cover: &CoverBlock, badge_asset: Option<&str>) -> Grid {
    let mut grid = Grid::new(3).with_item_min_height("142pt");

    if let Some(asset_name) = badge_asset {
        grid = grid.add_item(serde_json::json!({
            "type": "image",
            "data": Image::new(asset_name).with_width("68%").to_data()
        }));
    } else {
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": MetricCard::new("Zertifikat", &cover.grade)
                .with_subtitle(format!("{} • {} / 100", cover.certificate, cover.score))
                .with_accent_color(certificate_accent_color(&cover.certificate))
                .with_height("100%")
                .to_data()
        }));
    }

    let score_color = if cover.score >= 70 { "#22c55e" } else if cover.score >= 50 { "#f59e0b" } else { "#ef4444" };
    grid = grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("Accessibility", format!("{}/100", cover.score))
            .with_subtitle(cover.maturity_label.clone())
            .with_accent_color(score_color)
            .with_height("100%")
            .to_data()
    }));

    grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("Issues", cover.total_issues.to_string())
            .with_subtitle(format!("{} kritisch/hoch", cover.critical_issues))
            .with_accent_color("#dc2626")
            .with_height("100%")
            .to_data()
    }))
}

pub(super) fn build_batch_cover_score_row(
    avg_score: u32,
    total_urls: u32,
    total_violations: u32,
    badge_asset: Option<&str>,
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
            "data": MetricCard::new("Zertifikat", batch_grade_label(avg_score))
                .with_subtitle(format!("{} • {} / 100", batch_certificate_label(avg_score), avg_score))
                .with_accent_color(certificate_accent_color(batch_certificate_label(avg_score)))
                .with_height("100%")
                .to_data()
        }));
    }

    grid = grid.add_item(serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new("Durchschnitt", avg_score)
            .with_thresholds(70, 50)
            .with_height("100%")
            .to_data()
    }));

    Ok(grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new("URLs", total_urls.to_string())
            .with_subtitle(format!("{} Verstöße", total_violations))
            .with_accent_color("#dc2626")
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
        "PLATINUM" => (
            "auditmysite-certificate-platinum.svg",
            include_str!("../../../assets/certificates/platinum.svg"),
        ),
        "GOLD" => (
            "auditmysite-certificate-gold.svg",
            include_str!("../../../assets/certificates/gold.svg"),
        ),
        "SILVER" => (
            "auditmysite-certificate-silver.svg",
            include_str!("../../../assets/certificates/silver.svg"),
        ),
        "BRONZE" => (
            "auditmysite-certificate-bronze.svg",
            include_str!("../../../assets/certificates/bronze.svg"),
        ),
        "FAILED" => (
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
        "PLATINUM" => "#0f766e",
        "GOLD" => "#b45309",
        "SILVER" => "#475569",
        "BRONZE" => "#9a3412",
        "FAILED" => "#dc2626",
        _ => "#2563eb",
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
        95..=100 => "PLATINUM",
        85..=94 => "GOLD",
        75..=84 => "SILVER",
        65..=74 => "BRONZE",
        _ => "FAILED",
    }
}
