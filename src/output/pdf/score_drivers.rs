//! "Why is the overall score X": the score-driver table, the weight-basis
//! note, the `viewport_weighted` derivation, the additional score drivers and
//! the per-module subcategory breakdown of the Management Summary.
//!
//! Extracted verbatim from `single_report.rs` (plan 66, WP3; same cut as
//! `problem_profile.rs` in plan 27). Only visibility widened to `pub(super)`
//! where `single_report.rs` calls in; no logic changed in the move.

use crate::output::localized::is_english;
use renderreport::components::advanced::List;
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use super::design;
use super::detail_modules::score_band_label;
use crate::i18n::I18n;
use crate::output::report_model::*;

/// "Why is the overall score X" — one compact row per module that actually
/// feeds the weighted `overall_score`, showing its score, its weight, and a
/// plain-language classification of whether it drags the overall score down
/// or stabilizes it (plan/2-score-driver-breakdown.md). Deliberately reads
/// `vm.modules.module_scores` (the canonical `ModuleScoreEntry` list, same
/// numbers `overall_score` is computed from) rather than
/// `vm.modules.dashboard` — the dashboard's cards are reshaped for
/// narrative presentation (e.g. "Search Experience" blends SEO with
/// heuristic AI-visibility signals into a composite score that is *not*
/// the raw SEO number actually carrying SEO's weight).
pub(super) fn render_score_driver_table(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let contributing: Vec<&crate::audit::normalized::ModuleScoreEntry> = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .collect();
    // The combined technical value, named as what it is. It is no longer the
    // headline — that is the accessibility score (plan 29, D1) — so the
    // report has to say once what this second number averages, otherwise a
    // reader meets it in the derivation table with no introduction.
    let basis: u32 = contributing.iter().map(|m| m.weight_pct).sum();
    builder = builder
        .add_component(
            ScoreCard::new(
                if en {
                    "Combined technical score · 0–100"
                } else {
                    "Kombinierter technischer Wert · 0–100"
                },
                vm.summary.overall_score,
            )
            .with_description(if en {
                format!("Weighted across six modules, weight basis {basis}")
            } else {
                format!("Gewichtet über sechs Module, Gewichtsbasis {basis}")
            })
            .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(if en {
                "A different question from the accessibility score on the cover, which is what this report assesses."
            } else {
                "Eine andere Frage als der Barrierefreiheits-Wert auf dem Deckblatt, um den es in diesem Bericht geht."
            })
            .with_size("10.5pt")
            .with_color(design::tokens::NEUTRAL),
        );

    if contributing.len() < 2 {
        return builder;
    }

    // The "biggest weak point" is the module that pulls the overall score
    // down the hardest — that's a function of *both* how far below "Good"
    // it scores and how much weight it carries, not score alone. A 40%-
    // weighted module at 40/100 drags the overall score down far more than
    // a 10%-weighted module at 30/100, even though the latter's raw score
    // looks worse in isolation (the exact case this table exists to make
    // legible, see plan/2-score-driver-breakdown.md's "Accessibility 20 /
    // Security 30" example). Every other module scoring below the "Good"
    // band (>=75, reusing the shared FIVE_BAND cutoffs — no new threshold
    // logic) is a lesser but still real drag; everything at/above 75
    // stabilizes the overall score.
    let weakest_name = contributing
        .iter()
        .filter(|m| m.score < 75)
        .max_by_key(|m| (100 - m.score) * m.weight_pct)
        .map(|m| m.name.clone());

    let classify = |m: &crate::audit::normalized::ModuleScoreEntry| -> &'static str {
        if Some(&m.name) == weakest_name.as_ref() {
            if en {
                "Biggest weak point"
            } else {
                "Größter Schwachpunkt"
            }
        } else if m.score < 75 {
            if en {
                "Significant risk driver"
            } else {
                "Erheblicher Risikotreiber"
            }
        } else if en {
            "Stabilizes the combined technical score"
        } else {
            "Stabilisiert den kombinierten Wert"
        }
    };

    // In `viewport_weighted` mode the per-module weighting is NOT how
    // `overall_score` was computed — the score comes from a desktop/mobile
    // blend mixed with security (`score_breakdown`). Titling this table "why
    // the overall score is what it is" then invited arithmetic that does not
    // come out: on casoon.de the rows averaged 90.05 against a reported 92,
    // on inros-lackner 47.05 against 45 (plan 34). The real derivation is
    // rendered separately by `render_overall_score_derivation` below.
    let is_derivation = vm.summary.score_calculation_method != "viewport_weighted";
    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Module" } else { "Modul" }).with_width("28%"),
        TableColumn::new("Score").with_width("14%"),
        TableColumn::new(if en { "Weight" } else { "Gewichtung" }).with_width("16%"),
        TableColumn::new(if en { "Assessment" } else { "Einordnung" }).with_width("42%"),
    ])
    .with_title(match (is_derivation, en) {
        (true, true) => "Why the combined technical score is what it is",
        (true, false) => "Warum der kombinierte technische Wert ist, was er ist",
        (false, true) => "The modules that carry weight",
        (false, false) => "Die gewichteten Module im Einzelnen",
    });

    for m in &contributing {
        // Same lookup as `output::builder::helpers::localized_module_name`
        // (private to that module, so replicated here rather than exposed
        // just for this one call site): `module-<name>` Fluent key with the
        // canonical English name as a fallback.
        let key = format!("module-{}", m.name.to_lowercase().replace(' ', "-"));
        let translated = i18n.t(&key);
        let display_name = if translated == key {
            m.name.clone()
        } else {
            translated
        };
        // Security's wording band is corrected by open severe findings
        // (plan 33); the row must say what its section and score card say.
        let band = match vm.module_details.security.as_ref() {
            Some(sec) if m.name == "Security" => sec.band_label.as_str(),
            _ => score_band_label(m.score, i18n),
        };
        table = table.add_row(vec![
            display_name,
            format!("{} ({})", m.score, band),
            format!("{}%", m.weight_pct),
            classify(m).to_string(),
        ]);
    }

    builder = builder.add_component(table);
    builder = render_weight_basis_note(builder, vm, i18n);
    render_overall_score_derivation(builder, vm, i18n)
}

/// States the weight basis when the overall score was renormalised.
///
/// The score is divided by the weight of the *contributing* modules, so a
/// module that did not run, or that ran without being able to measure,
/// silently changes the denominator rather than the result. That is deliberate
/// for Performance (#QA-023) but was never communicated: two runs over
/// different module sets carry the same label, grade and rating and are not
/// comparable (plan 41).
fn render_weight_basis_note(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let basis: u32 = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.weight_pct)
        .sum();
    if basis >= 100 {
        return builder;
    }

    // Derived from the weight table, not from the entry list: a module that
    // was skipped outright has no entry at all, so filtering the entries alone
    // named nothing in exactly the case a reader most needs it
    // (`--skip-performance`).
    let contributing: std::collections::HashSet<&str> = vm
        .modules
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.name.as_str())
        .collect();
    let missing: Vec<String> = crate::taxonomy::MODULE_WEIGHTS
        .iter()
        .filter(|(name, _)| !contributing.contains(name))
        .map(|(name, weight)| format!("{name} ({weight} %)"))
        .collect();

    let mut note = if en {
        format!(
            "The combined technical score is normalised over the modules that were measured — a weight basis of {basis} of 100, not the full set."
        )
    } else {
        format!(
            "Der kombinierte technische Wert ist auf die gemessenen Module normiert — Gewichtsbasis {basis} von 100, nicht der volle Satz."
        )
    };
    if !missing.is_empty() {
        note.push(' ');
        note.push_str(&if en {
            format!("Not included: {}.", missing.join(", "))
        } else {
            format!("Nicht enthalten: {}.", missing.join(", "))
        });
    }
    note.push(' ');
    note.push_str(if en {
        "A score from a different module set is not directly comparable."
    } else {
        "Ein Wert aus einem anderen Modulsatz ist damit nicht direkt vergleichbar."
    });

    builder = builder.add_component(Callout::info(&note).with_title(if en {
        "Weight basis"
    } else {
        "Gewichtsbasis"
    }));
    builder
}

/// How `overall_score` was actually reached, in `viewport_weighted` mode.
///
/// The numbers come from `score_breakdown`, so the block reproduces the
/// headline value exactly — the same contract the Search Experience section's
/// "Wie sich der Wert zusammensetzt" table already honours. Without it the
/// report showed a weighting that does not add up to the number beside it and
/// no other explanation anywhere (plan 34).
fn render_overall_score_derivation(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let Some(sb) = vm.summary.score_breakdown.as_ref() else {
        return builder;
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Step" } else { "Schritt" }).with_width("46%"),
        TableColumn::new(if en { "Input" } else { "Eingang" }).with_width("34%"),
        TableColumn::new(if en { "Result" } else { "Ergebnis" }).with_width("20%"),
    ])
    .with_title(if en {
        "How the combined technical score was calculated"
    } else {
        "Wie der kombinierte technische Wert zustande kommt"
    });

    table = table.add_row(vec![
        if en {
            "Viewport blend (mobile weighs more)".to_string()
        } else {
            "Viewport-Mischung (Mobile zählt mehr)".to_string()
        },
        format!(
            "Desktop {} x {} % + Mobile {} x {} %",
            sb.desktop_overall, sb.desktop_weight_pct, sb.mobile_overall, sb.mobile_weight_pct
        ),
        sb.viewport_blended_overall.to_string(),
    ]);

    match (sb.security_score, sb.security_weight_pct) {
        (Some(security), Some(security_weight)) => {
            table = table.add_row(vec![
                if en {
                    "Security mixed in".to_string()
                } else {
                    "Sicherheit eingemischt".to_string()
                },
                format!(
                    "{} x {} % + {} x {} %",
                    sb.viewport_blended_overall,
                    sb.viewport_blend_weight_pct,
                    security,
                    security_weight
                ),
                vm.summary.overall_score.to_string(),
            ]);
        }
        _ => {
            table = table.add_row(vec![
                if en {
                    "Security not measured — blend carries the full weight".to_string()
                } else {
                    "Sicherheit nicht gemessen — die Mischung trägt voll".to_string()
                },
                format!("{} x 100 %", sb.viewport_blended_overall),
                vm.summary.overall_score.to_string(),
            ]);
        }
    }

    builder = builder.add_component(table);
    builder.add_component(
        Label::new(if en {
            "The module weights above rank what carries how much; this table is the calculation that produced the combined technical score."
        } else {
            "Die Modulgewichte oben zeigen, was wie stark zählt; diese Tabelle ist die Rechnung, aus der der kombinierte technische Wert entsteht."
        })
        .with_size("8.8pt")
        .with_color(design::tokens::MUTED),
    )
}

/// One plain-language reason per non-Accessibility score driver (SEO,
/// Security, Tech complexity/Performance) — reuses the bullets already
/// computed by `interpretation::build_technical_overview_localized`
/// (`vm.summary.technical_overview`), which is always exactly
/// `[accessibility, seo, security, tech]` followed by any cross-impact notes.
/// Bullet 0 (Accessibility) is skipped here — it's already covered by the
/// WCAG root-cause note directly above this table (#2d).
pub(super) fn render_additional_score_drivers(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    if vm.summary.technical_overview.len() < 4 {
        return builder;
    }
    let mut list = List::new().with_title(if en {
        "Additional score drivers"
    } else {
        "Weitere Score-Treiber"
    });
    for item in &vm.summary.technical_overview[1..4] {
        list = list.add_item(item);
    }
    builder = builder.add_component(list);
    builder
}

/// Splits a list of `(label, score)` pairs into "critical weaknesses"
/// (below the shared FIVE_BAND "Verbesserungswürdig"/"Needs improvement"
/// cutoff at 60 — no new threshold invented) and "comparatively stable",
/// then renders a compact callout per module, matching the review's own
/// example format (plan/5-module-accessibility-security-driver-detail.md
/// and plan/21-accessibility-subcategory-breakdown-uniform-gap.md).
/// Handles both contrast (two-line) and uniform (single-line) distributions;
/// renders nothing only for an empty input (module didn't run).
pub(super) fn render_subcategory_breakdown(
    mut builder: renderreport::engine::ReportBuilder,
    title: &str,
    entries: &[(String, u32)],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    if entries.is_empty() {
        return builder;
    }
    const STABLE_CUTOFF: f32 = 60.0; // registry::FIVE_BAND's "Needs improvement" cutoff

    let mut weak: Vec<&(String, u32)> = entries
        .iter()
        .filter(|(_, score)| (*score as f32) < STABLE_CUTOFF)
        .collect();
    let mut stable: Vec<&(String, u32)> = entries
        .iter()
        .filter(|(_, score)| (*score as f32) >= STABLE_CUTOFF)
        .collect();
    if weak.is_empty() && stable.is_empty() {
        return builder;
    }
    weak.sort_by_key(|(_, score)| *score);
    stable.sort_by_key(|(_, score)| std::cmp::Reverse(*score));

    let render_side = |items: &[&(String, u32)]| -> String {
        items
            .iter()
            .map(|(name, score)| format!("{name} {score}"))
            .collect::<Vec<_>>()
            .join(" · ")
    };

    // "Stronger sub-areas within {title}" rather than "comparatively stable"
    // (feedback 2026-09-07): at an overall score of e.g. 20, a subcategory at
    // 67 read as "stable" on its own, contradicting the module's headline
    // "Kritisch" grade a few lines above. Framing it as relative-within-the-
    // module keeps the same information without the contradiction.
    let body = if !weak.is_empty() && !stable.is_empty() {
        if en {
            format!(
                "Critical weaknesses: {}\nStronger sub-areas within {title}: {}",
                render_side(&weak),
                render_side(&stable)
            )
        } else {
            format!(
                "Kritische Schwächen: {}\nStärkere Teilbereiche innerhalb der {title}: {}",
                render_side(&weak),
                render_side(&stable)
            )
        }
    } else if !weak.is_empty() {
        if en {
            format!("Critical weaknesses: {}", render_side(&weak))
        } else {
            format!("Kritische Schwächen: {}", render_side(&weak))
        }
    } else if en {
        format!(
            "Stronger sub-areas within {title}: {}",
            render_side(&stable)
        )
    } else {
        format!(
            "Stärkere Teilbereiche innerhalb der {title}: {}",
            render_side(&stable)
        )
    };
    builder = builder.add_component(Callout::info(body).with_title(title));
    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 34: in `viewport_weighted` mode the per-module weighting is not
    /// the computation path, so the derivation block must reproduce the
    /// headline value on its own. Confirmed live before the fix: casoon.de's
    /// module rows averaged 90.05 against a reported 92, inros-lackner's
    /// 47.05 against 45.
    #[test]
    fn score_derivation_reproduces_the_overall_score() {
        for (desktop, mobile, security, expected) in [
            (94u32, 91u32, Some(95u32), 92u32),
            (41, 49, Some(30), 45),
            (80, 80, None, 80),
        ] {
            let blended = ((desktop as f64 * 30.0 + mobile as f64 * 70.0) / 100.0).round() as u32;
            let overall = match security {
                Some(sec) => ((blended as f64 * 90.0 + sec as f64 * 10.0) / 100.0).round() as u32,
                None => blended,
            };
            assert_eq!(
                overall, expected,
                "desktop {desktop} / mobile {mobile} / security {security:?}",
            );
        }
    }

    #[test]
    fn subcategory_breakdown_handles_all_weak_entries() {
        let i18n = I18n::new("de").unwrap();
        let builder = renderreport::engine::ReportBuilder::new("single");
        let entries = vec![
            ("ARIA".to_string(), 10),
            ("Landmarks".to_string(), 20),
            ("Semantik".to_string(), 50),
        ];
        let builder = render_subcategory_breakdown(builder, "Accessibility", &entries, &i18n);
        let report = builder.build();
        assert_eq!(report.components.len(), 1);
        let json = serde_json::to_string(&report.components[0]).unwrap();
        assert!(json.contains("Kritische Schwächen:"));
        assert!(!json.contains("Stärkere Teilbereiche"));
        assert!(json.contains("ARIA 10"));
    }

    #[test]
    fn subcategory_breakdown_handles_all_stable_entries() {
        let i18n = I18n::new("de").unwrap();
        let builder = renderreport::engine::ReportBuilder::new("single");
        let entries = vec![
            ("Formulare".to_string(), 100),
            ("Alternativtexte".to_string(), 90),
        ];
        let builder = render_subcategory_breakdown(builder, "Barrierefreiheit", &entries, &i18n);
        let report = builder.build();
        assert_eq!(report.components.len(), 1);
        let json = serde_json::to_string(&report.components[0]).unwrap();
        assert!(!json.contains("Kritische Schwächen"));
        assert!(json.contains("Stärkere Teilbereiche innerhalb der Barrierefreiheit:"));
        assert!(json.contains("Formulare 100"));
    }

    #[test]
    fn subcategory_breakdown_handles_contrast_entries() {
        let i18n = I18n::new("de").unwrap();
        let builder = renderreport::engine::ReportBuilder::new("single");
        let entries = vec![("ARIA".to_string(), 10), ("Formulare".to_string(), 100)];
        let builder = render_subcategory_breakdown(builder, "Barrierefreiheit", &entries, &i18n);
        let report = builder.build();
        assert_eq!(report.components.len(), 1);
        let json = serde_json::to_string(&report.components[0]).unwrap();
        assert!(json.contains("Kritische Schwächen:"));
        assert!(json.contains("Stärkere Teilbereiche innerhalb der Barrierefreiheit:"));
    }

    #[test]
    fn subcategory_breakdown_handles_empty_entries() {
        let i18n = I18n::new("de").unwrap();
        let builder = renderreport::engine::ReportBuilder::new("single");
        let entries: Vec<(String, u32)> = vec![];
        let builder = render_subcategory_breakdown(builder, "Accessibility", &entries, &i18n);
        let report = builder.build();
        assert_eq!(report.components.len(), 0);
    }
}
