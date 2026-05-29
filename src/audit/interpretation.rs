//! Pre-computed interpretation layer for normalized audit reports.
//!
//! Moves evaluation logic out of the output builder so that scoring verdicts
//! and technical-overview bullets are available in both DE and EN without
//! needing the builder to know about thresholds.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::audit::normalized::NormalizedReport;

// ── LocalizedText ─────────────────────────────────────────────────────────────

/// A piece of text pre-computed in both German and English.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedText {
    pub de: String,
    pub en: String,
}

impl LocalizedText {
    pub fn for_locale<'a>(&'a self, locale: &str) -> &'a str {
        if locale == "en" {
            &self.en
        } else {
            &self.de
        }
    }
}

// ── ScoreBand ─────────────────────────────────────────────────────────────────

/// Five-level quality band derived from a numeric score.
///
/// Matches the label prefixes in CLAUDE.md: Sehr gut / Gut / Verbesserungswürdig /
/// Ausbaufähig / Kritisch (EN: Excellent / Good / Needs improvement / Inadequate /
/// Critical).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreBand {
    Excellent,
    Good,
    NeedsImprovement,
    Weak,
    Critical,
}

impl ScoreBand {
    pub fn from_score(score: f32) -> Self {
        match score.round() as i64 {
            s if s >= 90 => Self::Excellent,
            s if s >= 75 => Self::Good,
            s if s >= 60 => Self::NeedsImprovement,
            s if s >= 40 => Self::Weak,
            _ => Self::Critical,
        }
    }
}

// ── Interpretation ────────────────────────────────────────────────────────────

/// Pre-computed interpretation for one normalized report.
///
/// Computed once during normalization and stored on the `NormalizedReport` so
/// that every output formatter can read values rather than recalculating them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interpretation {
    /// Technical-overview bullets (a11y pattern, SEO level, security level,
    /// tech complexity) as pre-computed DE+EN pairs.
    pub technical_overview: Vec<LocalizedText>,
    /// Per-module score interpretation keyed by module name (lowercase,
    /// e.g. `"accessibility"`, `"performance"`).
    pub per_module: HashMap<String, LocalizedText>,
    /// Overall score band for the accessibility score.
    pub overall_score_band: ScoreBand,
}

impl Interpretation {
    pub fn from_report(report: &NormalizedReport) -> Self {
        let technical_overview = build_technical_overview_localized(report);
        let per_module = build_per_module_localized(report);
        let overall_score_band = ScoreBand::from_score(report.score as f32);

        Self {
            technical_overview,
            per_module,
            overall_score_band,
        }
    }
}

// ── Module areas ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum InterpretArea {
    Accessibility,
    Performance,
    Security,
    Mobile,
    Ux,
    Journey,
}

/// Localized, module-specific score interpretation. Returns both DE and EN.
///
/// Wording follows the "Report Wording Style" rules in CLAUDE.md.
pub fn interpret_score_localized(area: InterpretArea, score: f32) -> LocalizedText {
    use InterpretArea::*;
    use ScoreBand::*;

    let (de, en): (&str, &str) = match (area, ScoreBand::from_score(score)) {
        (Accessibility, Excellent) => (
            "Sehr gut — die Barrierefreiheit ist technisch sauber umgesetzt und weist nur geringe Einschränkungen auf.",
            "Excellent — accessibility is implemented cleanly, with only minor limitations.",
        ),
        (Accessibility, Good) => (
            "Gut — die Barrierefreiheit ist insgesamt stabil und konsistent, kleinere Optimierungen sind sinnvoll.",
            "Good — accessibility is sound overall; minor improvements are worthwhile.",
        ),
        (Accessibility, NeedsImprovement) => (
            "Verbesserungswürdig — einzelne Barrieren können die Nutzung einschränken.",
            "Needs improvement — individual barriers can restrict use.",
        ),
        (Accessibility, Weak) => (
            "Ausbaufähig — relevante Barrieren beeinträchtigen Nutzbarkeit und Zugänglichkeit.",
            "Inadequate — significant barriers impair usability and accessibility.",
        ),
        (Accessibility, Critical) => (
            "Kritisch — wesentliche Anforderungen an die Barrierefreiheit werden nicht erfüllt.",
            "Critical — essential accessibility requirements are not met.",
        ),

        (Performance, Excellent) => (
            "Sehr gut — die Seite reagiert schnell und bietet eine flüssige Nutzererfahrung.",
            "Excellent — the page responds quickly and feels smooth to use.",
        ),
        (Performance, Good) => (
            "Gut — die Performance ist stabil, vereinzelt bestehen Optimierungsmöglichkeiten.",
            "Good — performance is stable, with occasional room for optimization.",
        ),
        (Performance, NeedsImprovement) => (
            "Verbesserungswürdig — Ladezeiten und Reaktionsverhalten sind stellenweise uneinheitlich.",
            "Needs improvement — load times and responsiveness are inconsistent in places.",
        ),
        (Performance, Weak) => (
            "Ausbaufähig — Performance-Probleme können Nutzung und Conversion beeinträchtigen.",
            "Inadequate — performance issues can impair use and conversion.",
        ),
        (Performance, Critical) => (
            "Kritisch — deutliche Performance-Probleme beeinträchtigen die Nutzererfahrung erheblich.",
            "Critical — significant performance problems severely impair the user experience.",
        ),

        (Security, Excellent) => (
            "Sehr gut — keine wesentlichen Sicherheitsauffälligkeiten im geprüften Umfang erkannt.",
            "Excellent — no significant security issues found within the scope checked.",
        ),
        (Security, Good) => (
            "Gut — grundlegende Sicherheitsmechanismen sind vorhanden, kleinere Optimierungspotenziale wurden erkannt.",
            "Good — basic security mechanisms are in place; minor weaknesses were identified.",
        ),
        (Security, NeedsImprovement) => (
            "Verbesserungswürdig — einzelne Sicherheitsaspekte sollten überprüft und abgesichert werden.",
            "Needs improvement — individual security aspects should be reviewed and hardened.",
        ),
        (Security, Weak) => (
            "Ausbaufähig — relevante Sicherheitsauffälligkeiten oder Fehlkonfigurationen wurden erkannt.",
            "Inadequate — relevant security issues or misconfigurations were found.",
        ),
        (Security, Critical) => (
            "Kritisch — es bestehen erhebliche Sicherheitsrisiken mit unmittelbarem Handlungsbedarf.",
            "Critical — significant security risks exist that require immediate action.",
        ),

        (Mobile, Excellent) => (
            "Sehr gut — die Nutzung auf Mobilgeräten funktioniert zuverlässig und ohne erkennbare Einschränkungen.",
            "Excellent — the site works reliably on mobile devices, with no noticeable limitations.",
        ),
        (Mobile, Good) => (
            "Gut — die Nutzung auf Mobilgeräten funktioniert insgesamt zuverlässig.",
            "Good — the site works reliably on mobile devices overall.",
        ),
        (Mobile, NeedsImprovement) => (
            "Verbesserungswürdig — auf Mobilgeräten treten stellenweise Bedien- und Darstellungsprobleme auf.",
            "Needs improvement — layout and usability issues appear in places on mobile devices.",
        ),
        (Mobile, Weak) => (
            "Ausbaufähig — Darstellung und Bedienung auf Mobilgeräten sind spürbar eingeschränkt.",
            "Inadequate — layout and usability on mobile devices are noticeably impaired.",
        ),
        (Mobile, Critical) => (
            "Kritisch — die Seite ist auf Mobilgeräten kaum zuverlässig nutzbar.",
            "Critical — the site is barely usable on mobile devices.",
        ),

        (Ux, Excellent) => (
            "Sehr gut — die Bedienung ist klar und führt Nutzer sicher durch die Seite.",
            "Excellent — the interface is clear and guides users confidently through the page.",
        ),
        (Ux, Good) => (
            "Gut — die Nutzerführung ist verständlich, einzelne Abläufe lassen sich straffen.",
            "Good — user guidance is clear; individual flows can be tightened.",
        ),
        (Ux, NeedsImprovement) => (
            "Verbesserungswürdig — Nutzerführung und Interaktion wirken stellenweise unnötig komplex.",
            "Needs improvement — user guidance and interaction feel needlessly complex in places.",
        ),
        (Ux, Weak) => (
            "Ausbaufähig — Reibungspunkte erschweren eine klare Nutzerführung.",
            "Inadequate — friction points get in the way of clear user guidance.",
        ),
        (Ux, Critical) => (
            "Kritisch — die Bedienung ist unübersichtlich und behindert die Zielerreichung.",
            "Critical — the interface is confusing and prevents users from reaching their goal.",
        ),

        (Journey, Excellent) => (
            "Sehr gut — die wichtigsten Nutzerpfade sind durchgängig und nachvollziehbar.",
            "Excellent — the key user paths are consistent and easy to follow.",
        ),
        (Journey, Good) => (
            "Gut — die zentralen Nutzerpfade funktionieren, einzelne Schritte lassen sich verbessern.",
            "Good — the core user paths work; individual steps can be improved.",
        ),
        (Journey, NeedsImprovement) => (
            "Verbesserungswürdig — einzelne Schritte der Nutzerführung sind umständlich oder unklar.",
            "Needs improvement — individual steps in the user flow are cumbersome or unclear.",
        ),
        (Journey, Weak) => (
            "Ausbaufähig — Brüche in der Nutzerführung erschweren das Erreichen zentraler Ziele.",
            "Inadequate — breaks in the user flow make it harder to reach key goals.",
        ),
        (Journey, Critical) => (
            "Kritisch — wichtige Schritte der Nutzerführung sind unnötig kompliziert oder unterbrochen.",
            "Critical — important steps in the user flow are needlessly complicated or broken.",
        ),
    };

    LocalizedText {
        de: de.to_string(),
        en: en.to_string(),
    }
}

// ── Private builders ──────────────────────────────────────────────────────────

fn build_technical_overview_localized(normalized: &NormalizedReport) -> Vec<LocalizedText> {
    let mut bullets = Vec::new();

    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let total = normalized.severity_counts.total;
    let rule_count = normalized.findings.len();

    // 1. Accessibility pattern
    let a11y = if total == 0 {
        LocalizedText {
            de: "Accessibility-Systematik: Keine Verstöße — Basis vollständig konform".to_string(),
            en: "Accessibility pattern: No violations — fully conformant baseline".to_string(),
        }
    } else if critical >= 5 && total > 30 {
        LocalizedText {
            de: format!("Accessibility-Systematik: Systematische Muster ({rule_count} Regeltypen, {total} Instanzen) — Prozess-Problem, kein Einzelfall"),
            en: format!("Accessibility pattern: Systematic patterns ({rule_count} rule types, {total} instances) — process problem, not a one-off"),
        }
    } else if critical >= 3 || (critical >= 2 && rule_count >= 5) {
        LocalizedText {
            de: format!("Accessibility-Systematik: Mehrere kritische Blockaden ({critical} kritisch, {high} hoch) — direkte Screenreader-Barrieren"),
            en: format!("Accessibility pattern: Multiple critical blockers ({critical} critical, {high} high) — direct screen-reader barriers"),
        }
    } else if total > 10 {
        LocalizedText {
            de: format!("Accessibility-Systematik: Verteilt über {rule_count} Regeltypen — kein Muster, einzeln behebbar"),
            en: format!("Accessibility pattern: Distributed across {rule_count} rule types — no pattern, fixable individually"),
        }
    } else {
        LocalizedText {
            de: format!("Accessibility-Systematik: {total} Verstöße in {rule_count} Bereichen — konzentriert und gezielt behebbar"),
            en: format!("Accessibility pattern: {total} violations across {rule_count} areas — focused and fixable"),
        }
    };
    bullets.push(a11y);

    // 2. SEO level
    let seo = if let Some(ref s) = normalized.raw_seo {
        if s.score >= 85 {
            LocalizedText {
                de: format!(
                    "SEO-Level: {} Pkt — technische Ranking-Voraussetzungen erfüllt",
                    s.score
                ),
                en: format!(
                    "SEO level: {} pts — technical ranking prerequisites met",
                    s.score
                ),
            }
        } else if s.score >= 65 {
            LocalizedText {
                de: format!(
                    "SEO-Level: {} Pkt — Basis vorhanden, gezielte Optimierungen möglich",
                    s.score
                ),
                en: format!(
                    "SEO level: {} pts — base in place, targeted optimizations possible",
                    s.score
                ),
            }
        } else if s.score >= 45 {
            LocalizedText {
                de: format!(
                    "SEO-Level: {} Pkt — relevante Signale fehlen, Sichtbarkeit eingeschränkt",
                    s.score
                ),
                en: format!(
                    "SEO level: {} pts — relevant signals missing, visibility limited",
                    s.score
                ),
            }
        } else {
            LocalizedText {
                de: format!("SEO-Level: {} Pkt — strukturelle Basis fehlt, Ranking-Potenzial deutlich eingeschränkt", s.score),
                en: format!("SEO level: {} pts — structural base missing, ranking potential is severely limited", s.score),
            }
        }
    } else {
        LocalizedText {
            de: "SEO-Level: Nicht geprüft (--full für vollständige Analyse)".to_string(),
            en: "SEO level: Not audited (use --full for full analysis)".to_string(),
        }
    };
    bullets.push(seo);

    // 3. Security level
    let sec = if let Some(ref s) = normalized.raw_security {
        if s.score >= 80 {
            LocalizedText {
                de: format!(
                    "Security-Level: {} Pkt — HTTP-Security-Header vollständig gesetzt",
                    s.score
                ),
                en: format!(
                    "Security level: {} pts — HTTP security headers fully set",
                    s.score
                ),
            }
        } else if s.score >= 55 {
            LocalizedText {
                de: format!("Security-Level: {} Pkt — Grundschutz vorhanden, einzelne Header fehlen", s.score),
                en: format!("Security level: {} pts — basic protection in place, individual headers missing", s.score),
            }
        } else if s.score >= 30 {
            LocalizedText {
                de: format!(
                    "Security-Level: {} Pkt — mehrere kritische Security-Header fehlen",
                    s.score
                ),
                en: format!(
                    "Security level: {} pts — multiple critical security headers missing",
                    s.score
                ),
            }
        } else {
            LocalizedText {
                de: format!("Security-Level: {} Pkt — Security-Header fehlen fast vollständig — hohes Risiko, schnell behebbar", s.score),
                en: format!("Security level: {} pts — security headers almost entirely missing — high risk, quick to fix", s.score),
            }
        }
    } else {
        LocalizedText {
            de: "Security-Level: Nicht geprüft (--full für vollständige Analyse)".to_string(),
            en: "Security level: Not audited (use --full for full analysis)".to_string(),
        }
    };
    bullets.push(sec);

    // 4. Tech complexity (DOM + performance)
    let dom = normalized.nodes_analyzed;
    let perf_score = normalized.raw_performance.as_ref().map(|p| p.score.overall);
    let tech = match (dom, perf_score) {
        (d, Some(p)) if d > 2000 && p < 60 => LocalizedText {
            de: format!("Tech-Komplexität: Hoch — {d} DOM-Knoten, Performance {p} Pkt — Refactoring empfohlen"),
            en: format!("Tech complexity: High — {d} DOM nodes, performance {p} pts — refactoring recommended"),
        },
        (d, Some(p)) if d > 2000 => LocalizedText {
            de: format!("Tech-Komplexität: Mittel-hoch — {d} DOM-Knoten (Performance {p} Pkt stabil)"),
            en: format!("Tech complexity: Medium-high — {d} DOM nodes (performance {p} pts stable)"),
        },
        (d, Some(p)) if p < 60 => LocalizedText {
            de: format!("Tech-Komplexität: Performance kritisch ({p} Pkt) — {d} DOM-Knoten analysiert"),
            en: format!("Tech complexity: Performance critical ({p} pts) — {d} DOM nodes analyzed"),
        },
        (d, Some(p)) if p < 80 => LocalizedText {
            de: format!("Tech-Komplexität: Gering — {d} DOM-Knoten, Performance optimierbar ({p} Pkt)"),
            en: format!("Tech complexity: Low — {d} DOM nodes, performance can be optimized ({p} pts)"),
        },
        (d, Some(p)) => LocalizedText {
            de: format!("Tech-Komplexität: Gering — {d} DOM-Knoten, Performance {p} Pkt — technische Basis stabil"),
            en: format!("Tech complexity: Low — {d} DOM nodes, performance {p} pts — technical baseline stable"),
        },
        (d, None) if d > 2000 => LocalizedText {
            de: format!("Tech-Komplexität: Hoch — {d} DOM-Knoten (Performance nicht geprüft)"),
            en: format!("Tech complexity: High — {d} DOM nodes (performance not audited)"),
        },
        (d, None) => LocalizedText {
            de: format!("Tech-Komplexität: {d} DOM-Knoten analysiert (Performance nicht geprüft, --full)"),
            en: format!("Tech complexity: {d} DOM nodes analyzed (performance not audited, use --full)"),
        },
    };
    bullets.push(tech);

    bullets
}

fn build_per_module_localized(normalized: &NormalizedReport) -> HashMap<String, LocalizedText> {
    let mut map = HashMap::new();

    map.insert(
        "accessibility".to_string(),
        interpret_score_localized(InterpretArea::Accessibility, normalized.score as f32),
    );

    if let Some(ref p) = normalized.raw_performance {
        map.insert(
            "performance".to_string(),
            interpret_score_localized(InterpretArea::Performance, p.score.overall as f32),
        );
    }

    if let Some(ref s) = normalized.raw_security {
        map.insert(
            "security".to_string(),
            interpret_score_localized(InterpretArea::Security, s.score as f32),
        );
    }

    if let Some(ref m) = normalized.raw_mobile {
        map.insert(
            "mobile".to_string(),
            interpret_score_localized(InterpretArea::Mobile, m.score as f32),
        );
    }

    if let Some(ref ux) = normalized.raw_ux {
        map.insert(
            "ux".to_string(),
            interpret_score_localized(InterpretArea::Ux, ux.score as f32),
        );
    }

    if let Some(ref j) = normalized.raw_journey {
        map.insert(
            "journey".to_string(),
            interpret_score_localized(InterpretArea::Journey, j.score as f32),
        );
    }

    map
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_band_boundaries() {
        assert_eq!(ScoreBand::from_score(95.0), ScoreBand::Excellent);
        assert_eq!(ScoreBand::from_score(90.0), ScoreBand::Excellent);
        assert_eq!(ScoreBand::from_score(89.0), ScoreBand::Good);
        assert_eq!(ScoreBand::from_score(75.0), ScoreBand::Good);
        assert_eq!(ScoreBand::from_score(74.0), ScoreBand::NeedsImprovement);
        assert_eq!(ScoreBand::from_score(60.0), ScoreBand::NeedsImprovement);
        assert_eq!(ScoreBand::from_score(59.0), ScoreBand::Weak);
        assert_eq!(ScoreBand::from_score(40.0), ScoreBand::Weak);
        assert_eq!(ScoreBand::from_score(39.0), ScoreBand::Critical);
        assert_eq!(ScoreBand::from_score(0.0), ScoreBand::Critical);
    }

    #[test]
    fn interpret_score_all_areas_all_bands() {
        use InterpretArea::*;
        let areas = [Accessibility, Performance, Security, Mobile, Ux, Journey];
        let scores: &[f32] = &[95.0, 80.0, 65.0, 50.0, 20.0];

        for area in areas {
            for &score in scores {
                let text = interpret_score_localized(area, score);
                assert!(
                    !text.de.is_empty(),
                    "DE text must not be empty for score {score}"
                );
                assert!(
                    !text.en.is_empty(),
                    "EN text must not be empty for score {score}"
                );
                // DE and EN must differ (real translations, not the same string)
                assert_ne!(text.de, text.en, "DE and EN must differ for score {score}");
            }
        }
    }

    #[test]
    fn interpret_score_label_prefix_contract() {
        // Wording rules: no "Befriedigend"; correct label prefix per band
        let areas = [
            InterpretArea::Accessibility,
            InterpretArea::Performance,
            InterpretArea::Security,
            InterpretArea::Mobile,
            InterpretArea::Ux,
            InterpretArea::Journey,
        ];

        let band_prefixes: &[(f32, &str, &str)] = &[
            (95.0, "Sehr gut", "Excellent"),
            (80.0, "Gut", "Good"),
            (65.0, "Verbesserungswürdig", "Needs improvement"),
            (50.0, "Ausbaufähig", "Inadequate"),
            (20.0, "Kritisch", "Critical"),
        ];

        for area in areas {
            for &(score, de_prefix, en_prefix) in band_prefixes {
                let text = interpret_score_localized(area, score);
                assert!(
                    text.de.starts_with(de_prefix),
                    "DE for score {score} must start with \"{de_prefix}\", got: \"{}\"",
                    text.de
                );
                assert!(
                    text.en.starts_with(en_prefix),
                    "EN for score {score} must start with \"{en_prefix}\", got: \"{}\"",
                    text.en
                );
                assert!(
                    !text.de.contains("Befriedigend"),
                    "Forbidden word \"Befriedigend\" in DE text for score {score}"
                );
            }
        }
    }

    #[test]
    fn technical_overview_has_four_bullets() {
        use crate::WcagLevel;

        let report = crate::audit::report::AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            100,
        );
        let normalized = crate::audit::normalized::normalize(&report);

        let bullets = build_technical_overview_localized(&normalized);
        assert_eq!(bullets.len(), 4, "always exactly 4 overview bullets");
        for bullet in &bullets {
            assert!(!bullet.de.is_empty());
            assert!(!bullet.en.is_empty());
        }
    }

    #[test]
    fn technical_overview_zero_violations_message() {
        use crate::WcagLevel;

        let report = crate::audit::report::AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            100,
        );
        let normalized = crate::audit::normalized::normalize(&report);
        let bullets = build_technical_overview_localized(&normalized);

        assert!(
            bullets[0].de.contains("Keine Verstöße"),
            "zero-violation DE message expected"
        );
        assert!(
            bullets[0].en.contains("No violations"),
            "zero-violation EN message expected"
        );
    }
}
