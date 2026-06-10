//! Cover page components for PDF reports.

use std::{env, fs, path::PathBuf};

use renderreport::components::advanced::Grid;
use renderreport::components::charts::{Gauge, GaugeThreshold};
use renderreport::components::{MetricCard, ScoreCard};
use renderreport::prelude::Image;
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

use super::design::{score_color, tokens};

pub(super) fn build_cover_score_row_gauges(
    _cover: &CoverBlock,
    summary: &SummaryBlock,
    modules: &[ModuleScore],
    i18n: &I18n,
) -> Grid {
    let en = i18n.locale() == "en";

    // Overall Score and modules in a grid of 4 columns
    let mut grid = Grid::new(4).with_item_min_height("130pt");

    // Gauge 1: Gesamtscore (Overall Score)
    let overall_label = if en { "Overall Score" } else { "Gesamtscore" };
    let mut overall_gauge = Gauge::new(overall_label, summary.overall_score as f64);
    overall_gauge.thresholds = vec![
        GaugeThreshold {
            value: 0.0,
            color: tokens::DANGER.to_string(),
        },
        GaugeThreshold {
            value: 50.0,
            color: tokens::WARN_DEEP.to_string(),
        },
        GaugeThreshold {
            value: 90.0,
            color: tokens::SUCCESS.to_string(),
        },
    ];
    grid = grid.add_item(serde_json::json!({
        "type": "gauge",
        "data": overall_gauge.to_data()
    }));

    // Gauges for each audited module
    for module in modules {
        let mut gauge = Gauge::new(&module.name, module.score as f64);
        gauge.thresholds = vec![
            GaugeThreshold {
                value: 0.0,
                color: tokens::DANGER.to_string(),
            },
            GaugeThreshold {
                value: 50.0,
                color: tokens::WARN_DEEP.to_string(),
            },
            GaugeThreshold {
                value: 90.0,
                color: tokens::SUCCESS.to_string(),
            },
        ];
        grid = grid.add_item(serde_json::json!({
            "type": "gauge",
            "data": gauge.to_data()
        }));
    }

    grid
}

pub(super) fn build_cover_score_row(
    cover: &CoverBlock,
    summary: &SummaryBlock,
    i18n: &I18n,
) -> Grid {
    let mut grid = Grid::new(3).with_item_min_height("142pt");
    let en = i18n.locale() == "en";

    // Card 1: Gesamtbewertung (MetricCard)
    let overall_title = if en { "Overall Score" } else { "Gesamtscore" };
    let overall_subtitle = format!("{} • {} / 100", summary.certificate, summary.overall_score);
    grid = grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new(overall_title, &summary.grade)
            .with_subtitle(overall_subtitle)
            .with_accent_color(certificate_accent_color(&summary.certificate))
            .with_height("100%")
            .to_data()
    }));

    // Card 2: Barrierefreiheit (MetricCard)
    let a11y_title = if en {
        "Accessibility"
    } else {
        "Barrierefreiheit"
    };
    let a11y_subtitle = format!("{} • {} / 100", cover.certificate, cover.score);
    grid = grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new(a11y_title, &cover.grade)
            .with_subtitle(a11y_subtitle)
            .with_accent_color(certificate_accent_color(&cover.certificate))
            .with_height("100%")
            .to_data()
    }));

    // Card 3: Risiko & Befunde (MetricCard)
    let status_title = if en {
        "Risk & Issues"
    } else {
        "Risiko & Befunde"
    };
    let status_subtitle = if en {
        format!(
            "{} total issues / {} critical/high",
            cover.total_issues, cover.critical_issues
        )
    } else {
        format!(
            "{} Befunde / {} kritisch/hoch",
            cover.total_issues, cover.critical_issues
        )
    };
    let risk_color = match cover.maturity_label.as_str() {
        "Kritisch" | "Critical" => tokens::DANGER,
        "Instabil" | "Unstable" | "Ausbaufähig" | "Eingeschränkt" | "Needs Improvement" => {
            tokens::WARN_DEEP
        }
        _ => tokens::SUCCESS,
    };
    grid.add_item(serde_json::json!({
        "type": "metric-card",
        "data": MetricCard::new(status_title, &cover.maturity_label)
            .with_subtitle(status_subtitle)
            .with_accent_color(risk_color)
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
        "SEHR GUT" | "EXCELLENT" => tokens::SUCCESS,
        "GUT" | "GOOD" => tokens::ACCENT_BRONZE,
        "SOLIDE" | "SOLID" => tokens::NEUTRAL,
        "AUSBAUFÄHIG" | "EINGESCHRÄNKT" | "NEEDS IMPROVEMENT" => "#9a3412",
        "UNGENÜGEND" | "NICHT BESTANDEN" | "FAILED" => tokens::DANGER,
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
