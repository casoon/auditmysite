//! Pre-computed interpretation layer for normalized audit reports.
//!
//! Moves evaluation logic out of the output builder so that scoring verdicts
//! and technical-overview bullets are available in both DE and EN without
//! needing the builder to know about thresholds.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::audit::normalized::AuditContext;

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
    /// Three-row overall impact table (user experience, risk level, conversion
    /// effect) as pre-computed DE+EN label-value pairs.
    pub overall_impact: Vec<(LocalizedText, LocalizedText)>,
    /// Benchmark context sentence comparing this score to other sites.
    pub benchmark_context: LocalizedText,
    /// i18n key for the business-consequence sentence. The output layer calls
    /// `i18n.t(key)` so the audit layer never depends on FTL bundles.
    pub business_consequence_key: String,
    /// i18n key for the consequence text. Empty string means no consequence
    /// text (e.g. no violations found).
    pub consequence_key: String,
    /// i18n key for the verdict tier ("verdict-tier-excellent", …).
    /// Combine with URL and score in `i18n.t_args`.
    pub verdict_key: String,
    /// i18n key for the score-note callout, or `None` if no note applies.
    pub score_note_key: Option<String>,
    /// i18n key for the batch verdict tier, or empty string for single-page reports.
    pub batch_verdict_key: String,
}

impl Interpretation {
    pub fn from_context(ctx: &AuditContext<'_>) -> Self {
        let technical_overview = build_technical_overview_localized(ctx);
        let per_module = build_per_module_localized(ctx);
        let overall_score_band = ScoreBand::from_score(ctx.normalized.score as f32);
        let overall_impact = build_overall_impact_localized(ctx);
        let benchmark_context =
            build_benchmark_context_localized(ctx.normalized.overall_score as f32);
        let business_consequence_key = pick_business_consequence_key(ctx);
        let consequence_key = pick_consequence_key(ctx);
        let verdict_key = pick_verdict_key(
            ctx.normalized.overall_score as f32,
            ctx.normalized.risk.legal_flags,
        );
        let score_note_key = pick_score_note_key(ctx);
        let batch_verdict_key = String::new();

        Self {
            technical_overview,
            per_module,
            overall_score_band,
            overall_impact,
            benchmark_context,
            business_consequence_key,
            consequence_key,
            verdict_key,
            score_note_key,
            batch_verdict_key,
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
    HtmlConform,
}

/// Localized, module-specific score interpretation. Returns both DE and EN.
///
/// Wording follows the "Report Wording Style" rules in CLAUDE.md.
pub fn interpret_score_localized(area: InterpretArea, score: f32) -> LocalizedText {
    interpret_band_localized(area, ScoreBand::from_score(score))
}

/// Security-specific override of `interpret_score_localized` (#578).
///
/// The generic band text above is shared across six modules and keyed on the
/// numeric score alone — for Security that's a real problem: several
/// Low-severity/context-dependent header misses (e.g. missing COOP/CORP,
/// which most standard sites don't need) can stack up and push the
/// aggregate score into the `Critical` band even though no individual issue
/// is actually severe. The `Critical` band's "significant security risks...
/// immediate action" wording should only fire when a genuinely severe issue
/// is present — not merely because several minor items stacked up.
///
/// Rather than threading the issues list into the shared `interpret_score_localized`
/// (which would change behavior for five unrelated modules), this override
/// lives here and only ever downgrades the *text* band actually shown — the
/// numeric score/grade elsewhere in the report is untouched.
pub fn interpret_security_score_localized(
    score: f32,
    issues: &[crate::security::SecurityIssue],
) -> LocalizedText {
    let band = ScoreBand::from_score(score);
    let has_severe_issue = issues.iter().any(|i| {
        matches!(
            i.severity,
            crate::taxonomy::Severity::Critical | crate::taxonomy::Severity::High
        )
    });
    let effective_band = if band == ScoreBand::Critical && !has_severe_issue {
        ScoreBand::Weak
    } else {
        band
    };
    interpret_band_localized(InterpretArea::Security, effective_band)
}

fn interpret_band_localized(area: InterpretArea, band: ScoreBand) -> LocalizedText {
    use InterpretArea::*;
    use ScoreBand::*;

    let (de, en): (&str, &str) = match (area, band) {
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

        (HtmlConform, Excellent) => (
            "Sehr gut — das ausgelieferte HTML ist technisch sauber und spezifikationskonform, Browser und assistive Technologien können es zuverlässig verarbeiten.",
            "Excellent — the rendered HTML is technically clean and spec-conformant; browsers and assistive technologies can parse it reliably.",
        ),
        (HtmlConform, Good) => (
            "Gut — das HTML ist überwiegend konform, einzelne Abweichungen von der Spezifikation sind vorhanden.",
            "Good — the HTML is largely conformant, with a few deviations from the specification.",
        ),
        (HtmlConform, NeedsImprovement) => (
            "Verbesserungswürdig — mehrere Spezifikationsverstöße im HTML können die Verarbeitung durch Browser oder assistive Technologien beeinträchtigen.",
            "Needs improvement — several HTML specification violations can affect processing by browsers or assistive technologies.",
        ),
        (HtmlConform, Weak) => (
            "Ausbaufähig — verbreitete Spezifikationsverstöße erschweren eine zuverlässige Verarbeitung des Markups.",
            "Inadequate — widespread specification violations make reliable markup processing harder.",
        ),
        (HtmlConform, Critical) => (
            "Kritisch — das HTML weicht erheblich von der Spezifikation ab, Rendering und Barrierefreiheit sind unmittelbar betroffen.",
            "Critical — the HTML deviates significantly from the specification, directly affecting rendering and accessibility.",
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

fn build_technical_overview_localized(normalized: &AuditContext<'_>) -> Vec<LocalizedText> {
    let mut bullets = Vec::new();

    let critical = normalized.normalized.severity_counts.critical;
    let high = normalized.normalized.severity_counts.high;
    let total = normalized.normalized.severity_counts.total;
    let rule_count = normalized.normalized.findings.len();

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
    let seo = if let Some(s) = normalized.raw_seo {
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
    let sec = if let Some(s) = normalized.raw_security {
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
    let dom = normalized.normalized.nodes_analyzed;
    let perf_score = normalized.raw_performance.map(|p| p.score.overall);
    // The branch thresholds below are calibrated against `nodes_analyzed`
    // (the WCAG engine's own checked-node count), not the page's real DOM
    // element count — those are different metrics at very different scales
    // (e.g. ~4,000 checked nodes vs. ~20,000 real DOM nodes on the same
    // page). `dom_display` prefers the real DOM count from the Performance
    // module's own vitals when available, so this sentence doesn't state a
    // "DOM-Knoten"/"DOM nodes" figure that contradicts the Performance
    // section's own DOM node count a few paragraphs later in the same
    // report (plan/30-pdf-numbers-not-traceable-to-json.md). Falls back to
    // `nodes_analyzed` only when no Performance data is available at all.
    let dom_display = normalized
        .raw_performance
        .and_then(|p| p.vitals.dom_nodes)
        .map(|n| n as usize)
        .unwrap_or(dom);
    let tech = match (dom, perf_score) {
        (d, Some(p)) if d > 2000 && p < 60 => LocalizedText {
            de: format!("Tech-Komplexität: Hoch — {dom_display} DOM-Knoten, Performance {p} Pkt — Refactoring empfohlen"),
            en: format!("Tech complexity: High — {dom_display} DOM nodes, performance {p} pts — refactoring recommended"),
        },
        // p is 60-74 here ("Verbesserungswürdig"/"Needs improvement" band,
        // same < 75 "Good" cutoff `render_score_driver_table` uses to call a
        // module a "Risikotreiber") — calling that "stabil"/"stable" a few
        // lines below a table that labels the same score a risk driver was
        // self-contradictory (feedback 2026-09-07). Only p >= 75 earns
        // "stabil" below.
        (d, Some(p)) if d > 2000 && p < 75 => LocalizedText {
            de: format!("Tech-Komplexität: Mittel-hoch — {dom_display} DOM-Knoten belasten eine nur mittelmäßige Performance ({p} Pkt) — Optimierungspotenzial vorhanden"),
            en: format!("Tech complexity: Medium-high — {dom_display} DOM nodes add strain to only middling performance ({p} pts) — optimization potential"),
        },
        (d, Some(p)) if d > 2000 => LocalizedText {
            de: format!("Tech-Komplexität: Mittel-hoch — {dom_display} DOM-Knoten (Performance {p} Pkt stabil)"),
            en: format!("Tech complexity: Medium-high — {dom_display} DOM nodes (performance {p} pts stable)"),
        },
        (_d, Some(p)) if p < 60 => LocalizedText {
            de: format!("Tech-Komplexität: Performance kritisch ({p} Pkt) — {dom_display} DOM-Knoten analysiert"),
            en: format!("Tech complexity: Performance critical ({p} pts) — {dom_display} DOM nodes analyzed"),
        },
        (_d, Some(p)) if p < 80 => LocalizedText {
            de: format!("Tech-Komplexität: Gering — {dom_display} DOM-Knoten, Performance optimierbar ({p} Pkt)"),
            en: format!("Tech complexity: Low — {dom_display} DOM nodes, performance can be optimized ({p} pts)"),
        },
        (_d, Some(p)) => LocalizedText {
            de: format!("Tech-Komplexität: Gering — {dom_display} DOM-Knoten, Performance {p} Pkt — technische Basis stabil"),
            en: format!("Tech complexity: Low — {dom_display} DOM nodes, performance {p} pts — technical baseline stable"),
        },
        (d, None) if d > 2000 => LocalizedText {
            de: format!("Tech-Komplexität: Hoch — {d} geprüfte Knoten (Performance nicht geprüft)"),
            en: format!("Tech complexity: High — {d} nodes checked (performance not audited)"),
        },
        (d, None) => LocalizedText {
            de: format!("Tech-Komplexität: {d} geprüfte Knoten (Performance nicht geprüft, --full)"),
            en: format!("Tech complexity: {d} nodes checked (performance not audited, use --full)"),
        },
    };
    bullets.push(tech);

    bullets
}

fn build_per_module_localized(normalized: &AuditContext<'_>) -> HashMap<String, LocalizedText> {
    let mut map = HashMap::new();

    map.insert(
        "accessibility".to_string(),
        interpret_score_localized(
            InterpretArea::Accessibility,
            normalized.normalized.score as f32,
        ),
    );

    if let Some(p) = normalized.raw_performance {
        map.insert(
            "performance".to_string(),
            interpret_score_localized(InterpretArea::Performance, p.score.overall as f32),
        );
    }

    if let Some(s) = normalized.raw_security {
        map.insert(
            "security".to_string(),
            interpret_security_score_localized(s.score as f32, &s.issues),
        );
    }

    if let Some(hc) = normalized.raw_html_conform {
        if hc.checked {
            map.insert(
                "html_conform".to_string(),
                interpret_score_localized(InterpretArea::HtmlConform, hc.score as f32),
            );
        }
    }

    if let Some(m) = normalized.raw_mobile {
        map.insert(
            "mobile".to_string(),
            interpret_score_localized(InterpretArea::Mobile, m.score as f32),
        );
    }

    if let Some(ux) = normalized.raw_ux {
        map.insert(
            "ux".to_string(),
            interpret_score_localized(InterpretArea::Ux, ux.score as f32),
        );
    }

    if let Some(j) = normalized.raw_journey {
        map.insert(
            "journey".to_string(),
            interpret_score_localized(InterpretArea::Journey, j.score as f32),
        );
    }

    map
}

fn build_overall_impact_localized(
    normalized: &AuditContext<'_>,
) -> Vec<(LocalizedText, LocalizedText)> {
    let score = normalized.normalized.score;
    let critical = normalized.normalized.severity_counts.critical;
    let high = normalized.normalized.severity_counts.high;
    let urgent = critical + high;

    let user_rating = LocalizedText {
        de: if score >= 90 && urgent == 0 {
            "Sehr gut — keine relevanten Barrieren".to_string()
        } else if score >= 75 {
            "Gut — einzelne Barrieren für Hilfstechnologien".to_string()
        } else if score >= 50 {
            "Eingeschränkt — spürbare Barrieren für Screenreader-Nutzer".to_string()
        } else {
            "Stark eingeschränkt — wesentliche Inhalte nicht zugänglich".to_string()
        },
        en: if score >= 90 && urgent == 0 {
            "Excellent — no relevant barriers".to_string()
        } else if score >= 75 {
            "Good — individual barriers for assistive technologies".to_string()
        } else if score >= 50 {
            "Limited — noticeable barriers for screen-reader users".to_string()
        } else {
            "Heavily limited — essential content not accessible".to_string()
        },
    };

    let risk_level = LocalizedText {
        de: if critical >= 2 {
            "Hoch — BITV/WCAG-Verstoßrisiko akut".to_string()
        } else if critical >= 1 || urgent >= 3 {
            "Mittel — kritische Themen vorhanden".to_string()
        } else if score < 70 {
            "Mittel — kumulierter Nachholbedarf".to_string()
        } else {
            "Niedrig".to_string()
        },
        en: if critical >= 2 {
            "High — acute BITV/WCAG violation risk".to_string()
        } else if critical >= 1 || urgent >= 3 {
            "Medium — critical topics present".to_string()
        } else if score < 70 {
            "Medium — cumulative backlog".to_string()
        } else {
            "Low".to_string()
        },
    };

    let conversion = LocalizedText {
        de: if score < 50 {
            "Hoch wahrscheinlich negativ".to_string()
        } else if score < 75 {
            "Möglicherweise negativ (Navigation, Formulare)".to_string()
        } else {
            "Gering — gute Nutzbarkeit".to_string()
        },
        en: if score < 50 {
            "Likely negative".to_string()
        } else if score < 75 {
            "Possibly negative (navigation, forms)".to_string()
        } else {
            "Low — good usability".to_string()
        },
    };

    vec![
        (
            LocalizedText {
                de: "Nutzererlebnis".to_string(),
                en: "User experience".to_string(),
            },
            user_rating,
        ),
        (
            LocalizedText {
                de: "Risiko-Level".to_string(),
                en: "Risk level".to_string(),
            },
            risk_level,
        ),
        (
            LocalizedText {
                de: "Conversion-Effekt".to_string(),
                en: "Conversion effect".to_string(),
            },
            conversion,
        ),
    ]
}

fn build_benchmark_context_localized(score: f32) -> LocalizedText {
    LocalizedText {
        de: if score >= 95.0 {
            "Top 5% — Ausnahmeniveau. Kein struktureller Handlungsdruck.".to_string()
        } else if score >= 90.0 {
            "Top 15% — Deutlich besser als die Mehrheit. Feinschliff genügt.".to_string()
        } else if score >= 80.0 {
            "Oberes Drittel — Guter Stand, einzelne Optimierungen lohnen sich.".to_string()
        } else if score >= 70.0 {
            "Mittleres Feld — Verbesserungspotenzial vorhanden, kein akuter Notfall.".to_string()
        } else if score >= 55.0 {
            "Unteres Mittelfeld — Deutlicher Rückstand gegenüber vergleichbaren Websites."
                .to_string()
        } else if score >= 40.0 {
            "Unteres Drittel — Erheblicher Rückstand, strukturelle Defizite häufig.".to_string()
        } else {
            "Kritisch — Zu den schwächsten geprüften Seiten. Sofortiger Handlungsbedarf."
                .to_string()
        },
        en: if score >= 95.0 {
            "Top 5% — exceptional level. No structural pressure to act.".to_string()
        } else if score >= 90.0 {
            "Top 15% — clearly above the majority. Polish is enough.".to_string()
        } else if score >= 80.0 {
            "Upper third — good standing, individual optimizations pay off.".to_string()
        } else if score >= 70.0 {
            "Middle pack — improvement potential, no acute emergency.".to_string()
        } else if score >= 55.0 {
            "Lower middle — clear gap to comparable websites.".to_string()
        } else if score >= 40.0 {
            "Lower third — significant gap, structural deficits common.".to_string()
        } else {
            "Critical — among the weakest audited sites. Immediate action required.".to_string()
        },
    }
}

fn pick_business_consequence_key(normalized: &AuditContext<'_>) -> String {
    let critical = normalized.normalized.severity_counts.critical;
    let total = normalized.normalized.severity_counts.total;
    let score = normalized.normalized.score;

    if total == 0 {
        return "business-consequence-clean".to_string();
    }

    let has_weak_seo = normalized.raw_seo.is_some_and(|s| s.score < 65);
    let has_heading_issues = normalized.normalized.findings.iter().any(|f| {
        f.rule_id.to_lowercase().contains("heading")
            || f.title.to_lowercase().contains("überschrift")
    });

    if score < 50 || (critical >= 5 && total > 30) {
        "business-consequence-severe"
    } else if has_weak_seo && has_heading_issues {
        "business-consequence-seo-headings"
    } else if critical >= 2 {
        "business-consequence-screenreader"
    } else {
        "business-consequence-default"
    }
    .to_string()
}

fn pick_consequence_key(normalized: &AuditContext<'_>) -> String {
    let critical = normalized.normalized.severity_counts.critical;
    let total = normalized.normalized.severity_counts.total;
    let score = normalized.normalized.score;

    if total == 0 {
        return String::new();
    }

    let weak_module_count = [
        normalized.raw_security.is_some_and(|s| s.score < 60),
        normalized.raw_seo.is_some_and(|s| s.score < 60),
        normalized
            .raw_performance
            .is_some_and(|p| p.score.overall < 70),
        normalized.raw_mobile.is_some_and(|m| m.score < 65),
    ]
    .iter()
    .filter(|&&v| v)
    .count();

    if score < 50 || (critical >= 5 && total > 30) {
        "consequence-severe"
    } else if critical >= 3 || weak_module_count >= 3 {
        "consequence-many-weak-modules"
    } else if score >= 85 {
        "consequence-stable"
    } else {
        "consequence-default"
    }
    .to_string()
}

fn pick_verdict_key(score: f32, legal_flags: usize) -> String {
    // A page with legal-relevant Level-A findings must never receive the
    // reassuring "solid" verdict, regardless of an otherwise passable score (#356).
    if legal_flags > 0 {
        return if score < 50.0 {
            "verdict-tier-critical"
        } else {
            "verdict-tier-deficient"
        }
        .to_string();
    }
    if score >= 90.0 {
        "verdict-tier-excellent"
    } else if score >= 70.0 {
        "verdict-tier-solid"
    } else if score >= 50.0 {
        "verdict-tier-deficient"
    } else {
        "verdict-tier-critical"
    }
    .to_string()
}

fn pick_score_note_key(normalized: &AuditContext<'_>) -> Option<String> {
    let critical_topics =
        normalized.normalized.severity_counts.critical + normalized.normalized.severity_counts.high;
    if normalized.normalized.score >= 90 && critical_topics > 0 {
        Some("score-note-high-with-critical".to_string())
    } else if normalized.normalized.score == 100 {
        Some("score-note-perfect-automated-scope".to_string())
    } else {
        None
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecurityIssue;
    use crate::taxonomy::Severity;

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
        let areas = [
            Accessibility,
            Performance,
            Security,
            Mobile,
            Ux,
            Journey,
            HtmlConform,
        ];
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
            InterpretArea::HtmlConform,
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
    fn tech_complexity_bullet_uses_real_dom_count_not_wcag_nodes_analyzed() {
        // plan/30-pdf-numbers-not-traceable-to-json.md: the WCAG engine's own
        // checked-node count (`nodes_analyzed`, tiny by comparison) must not
        // be printed under the "DOM-Knoten"/"DOM nodes" label when the
        // Performance module's real DOM element count is available —
        // otherwise the same report states two contradictory "DOM node"
        // figures for the same page.
        use crate::audit::PerformanceResults;
        use crate::performance::{calculate_performance_score, WebVitals};
        use crate::wcag::WcagResults;
        use crate::WcagLevel;

        let mut results = WcagResults::new();
        results.nodes_checked = 3974; // small WCAG-checked-node count

        let mut report = crate::audit::AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1_000,
        );

        let vitals = WebVitals {
            dom_nodes: Some(20368), // much larger real DOM element count
            ..Default::default()
        };
        let score = calculate_performance_score(&vitals, None);
        report.performance = Some(PerformanceResults {
            vitals,
            score,
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
            measurement_warnings: Vec::new(),
        });

        let ctx = crate::audit::normalized::normalize(&report);
        let bullets = build_technical_overview_localized(&ctx);
        let tech_bullet = bullets
            .iter()
            .find(|b| b.de.starts_with("Tech-Komplexität"))
            .expect("technical overview must include a tech-complexity bullet");

        assert!(
            tech_bullet.de.contains("20368"),
            "tech-complexity bullet must state the real DOM element count, got: {}",
            tech_bullet.de
        );
        assert!(
            !tech_bullet.de.contains("3974"),
            "tech-complexity bullet must not state the WCAG checked-node count as if it were the DOM count, got: {}",
            tech_bullet.de
        );
    }

    fn low_severity_context_issue(
        header: &str,
        tier: crate::security::HeaderTier,
    ) -> SecurityIssue {
        SecurityIssue {
            header: header.to_string(),
            issue_type: "missing_header".to_string(),
            message: format!("{header} is not set."),
            severity: Severity::Low,
            tier,
            values: Default::default(),
        }
    }

    /// "Standardmarketingseite" scenario (#578 acceptance criteria): several
    /// Low-severity/context-dependent header misses (Referrer-Policy,
    /// Permissions-Policy, COOP, CORP) can push the numeric score below the
    /// Critical threshold on their own, even though none of them is an
    /// actually severe issue. The report must not claim "significant
    /// security risks... immediate action" in that case.
    #[test]
    fn interpret_security_score_stacked_low_severity_misses_does_not_read_as_critical() {
        use crate::security::HeaderTier;

        let issues = vec![
            low_severity_context_issue("Referrer-Policy", HeaderTier::ArchitectureDependent),
            low_severity_context_issue("Permissions-Policy", HeaderTier::ArchitectureDependent),
            low_severity_context_issue("Cross-Origin-Opener-Policy", HeaderTier::ContextDependent),
            low_severity_context_issue(
                "Cross-Origin-Resource-Policy",
                HeaderTier::ContextDependent,
            ),
        ];
        // Numeric band alone would be Critical at this score.
        assert_eq!(ScoreBand::from_score(20.0), ScoreBand::Critical);

        let text = interpret_security_score_localized(20.0, &issues);
        assert!(
            !text.de.starts_with("Kritisch"),
            "stacked low-severity misses must not read as Kritisch: {}",
            text.de
        );
        assert!(
            !text.en.starts_with("Critical"),
            "stacked low-severity misses must not read as Critical: {}",
            text.en
        );
        assert!(text.de.starts_with("Ausbaufähig"), "got: {}", text.de);
        assert!(text.en.starts_with("Inadequate"), "got: {}", text.en);
    }

    /// A genuinely severe finding (missing HTTPS, broken CSP, missing
    /// clickjacking protection, ...) must keep the Critical wording
    /// regardless of how many other low-severity items are also present —
    /// CSP/HSTS/clickjacking protection stay clearly prioritized (#578).
    #[test]
    fn interpret_security_score_real_critical_issue_still_reads_as_critical() {
        use crate::security::HeaderTier;

        let issues = vec![
            SecurityIssue {
                header: "HTTPS".to_string(),
                issue_type: "missing_https".to_string(),
                message: "Site is not served over HTTPS".to_string(),
                severity: Severity::Critical,
                tier: HeaderTier::Baseline,
                values: Default::default(),
            },
            low_severity_context_issue("Cross-Origin-Opener-Policy", HeaderTier::ContextDependent),
        ];

        let text = interpret_security_score_localized(20.0, &issues);
        assert!(text.de.starts_with("Kritisch"), "got: {}", text.de);
        assert!(text.en.starts_with("Critical"), "got: {}", text.en);
    }

    /// The override only ever downgrades the Critical band — non-Critical
    /// bands must behave exactly like the generic `interpret_score_localized`.
    #[test]
    fn interpret_security_score_non_critical_band_is_unaffected() {
        use crate::security::HeaderTier;

        let issues = vec![low_severity_context_issue(
            "Cross-Origin-Opener-Policy",
            HeaderTier::ContextDependent,
        )];
        let text = interpret_security_score_localized(80.0, &issues);
        let baseline = interpret_score_localized(InterpretArea::Security, 80.0);
        assert_eq!(text.de, baseline.de);
        assert_eq!(text.en, baseline.en);
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
