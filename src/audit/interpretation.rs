//! Pre-computed interpretation layer for normalized audit reports.
//!
//! Moves evaluation logic out of the output builder so that scoring verdicts
//! and technical-overview bullets are available in both DE and EN without
//! needing the builder to know about thresholds.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::audit::normalized::AuditContext;
use crate::audit::{AuditQualityStatus, ExecutionStatus};
use crate::i18n::{text, text_args};
use crate::wcag::coverage::{ClaimLimits, ManualReview};
use fluent_bundle::FluentValue;

// ── LocalizedText ─────────────────────────────────────────────────────────────

/// A piece of text pre-computed in both German and English.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LocalizedText {
    pub de: String,
    pub en: String,
}

/// A text argument: `value` as the plain text the message prints.
fn arg(value: impl ToString) -> FluentValue<'static> {
    FluentValue::from(value.to_string())
}

/// The message `key` in both languages.
fn localized(key: &str) -> LocalizedText {
    LocalizedText {
        de: text(key, false),
        en: text(key, true),
    }
}

/// The message `key` with the same arguments in both languages.
fn localized_args(key: &str, args: &[(&str, FluentValue<'_>)]) -> LocalizedText {
    LocalizedText {
        de: text_args(key, args, false),
        en: text_args(key, args, true),
    }
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
#[non_exhaustive]
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

// ── AccessibilityClaim ────────────────────────────────────────────────────────

/// What the automated accessibility run found, bounded by what it covered.
/// Never a conformance state: "no automated violations" is its own kind (#782).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AccessibilityClaimKind {
    /// The accessibility module did not run or failed.
    NotMeasured,
    /// The run was partial or insufficient and found nothing — no evidence
    /// either way.
    IncompleteRun,
    ViolationsFound,
    /// A complete run found no violation on the automatically testable
    /// criteria. Says nothing about the criteria that need a person.
    NoAutomatedViolationsFound,
}

/// Machine-readable accessibility statement: result, scope and what is left
/// to a manual review (#782).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccessibilityClaim {
    pub kind: AccessibilityClaimKind,
    /// Run coverage the kind is bounded by (`execution.quality.status`).
    pub coverage: AuditQualityStatus,
    pub wcag_version: String,
    /// WCAG A/AA criteria the automated rules can fail.
    pub automated_criteria: usize,
    pub total_criteria: usize,
    /// A/AA criteria only a person can judge.
    pub manual_review_criteria: Vec<String>,
    #[serde(flatten)]
    pub limits: ClaimLimits,
}

impl AccessibilityClaim {
    pub fn from_context(ctx: &AuditContext<'_>) -> Self {
        let normalized = &ctx.normalized;
        let run = normalized
            .execution
            .module_runs
            .iter()
            .find(|run| run.module == "accessibility")
            .map(|run| run.status);
        let coverage = normalized.execution.quality.status;
        let kind = if normalized.severity_counts.total > 0 {
            AccessibilityClaimKind::ViolationsFound
        } else if matches!(
            run,
            Some(
                ExecutionStatus::Failed
                    | ExecutionStatus::Skipped
                    | ExecutionStatus::NotApplicable
                    | ExecutionStatus::NotRequested
            )
        ) {
            AccessibilityClaimKind::NotMeasured
        } else if coverage != AuditQualityStatus::Complete || run == Some(ExecutionStatus::Partial)
        {
            AccessibilityClaimKind::IncompleteRun
        } else {
            AccessibilityClaimKind::NoAutomatedViolationsFound
        };
        let mut remaining_manual_review = vec![ManualReview::CriteriaOutsideAutomatedScope];
        if matches!(
            kind,
            AccessibilityClaimKind::NotMeasured | AccessibilityClaimKind::IncompleteRun
        ) || coverage != AuditQualityStatus::Complete
        {
            remaining_manual_review.push(ManualReview::RepeatIncompleteRun);
        }
        let (automated_criteria, total_criteria) = crate::wcag::coverage::coverage_stats();
        Self {
            kind,
            coverage,
            wcag_version: crate::wcag::coverage::WCAG_VERSION.to_string(),
            automated_criteria,
            total_criteria,
            manual_review_criteria: crate::wcag::coverage::manual_review_criteria()
                .iter()
                .map(|(id, _, _)| id.to_string())
                .collect(),
            limits: ClaimLimits::with_manual_review(remaining_manual_review),
        }
    }
}

/// The technical-overview bullet for the accessibility claim — the only text
/// source for it, so the stored statement never outgrows `claim.kind`.
pub fn accessibility_claim_text(
    claim: &AccessibilityClaim,
    severity: &crate::audit::normalized::SeverityCounts,
    rule_count: usize,
    en: bool,
) -> String {
    let critical = severity.critical;
    let high = severity.high;
    let total = severity.total;
    let automated = claim.automated_criteria;
    let all = claim.total_criteria;
    let version = &claim.wcag_version;
    match claim.kind {
        AccessibilityClaimKind::NotMeasured => text("audit-interpretation-claim-not-measured", en),
        AccessibilityClaimKind::IncompleteRun => text("audit-interpretation-claim-incomplete", en),
        AccessibilityClaimKind::NoAutomatedViolationsFound => text_args(
            "audit-interpretation-claim-clean",
            &[
                ("automated", arg(automated)),
                ("all", arg(all)),
                ("version", arg(version)),
            ],
            en,
        ),
        AccessibilityClaimKind::ViolationsFound => {
            if critical >= 5 && total > 30 {
                text_args(
                    "audit-interpretation-claim-systematic",
                    &[("rules", arg(rule_count)), ("total", arg(total))],
                    en,
                )
            } else if critical >= 3 || (critical >= 2 && rule_count >= 5) {
                text_args(
                    "audit-interpretation-claim-blockers",
                    &[("critical", arg(critical)), ("high", arg(high))],
                    en,
                )
            } else if total > 10 {
                text_args(
                    "audit-interpretation-claim-distributed",
                    &[("rules", arg(rule_count))],
                    en,
                )
            } else {
                text_args(
                    "audit-interpretation-claim-focused",
                    &[("total", arg(total)), ("rules", arg(rule_count))],
                    en,
                )
            }
        }
    }
}

// ── Interpretation ────────────────────────────────────────────────────────────

/// Pre-computed interpretation for one normalized report.
///
/// Computed once during normalization and stored on the `NormalizedReport` so
/// that every output formatter can read values rather than recalculating them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Interpretation {
    /// Scoped accessibility statement; the first technical-overview bullet
    /// is derived from it.
    pub accessibility_claim: AccessibilityClaim,
    /// Technical-overview bullets (a11y pattern, SEO level, security level,
    /// tech complexity) as pre-computed DE+EN pairs.
    pub technical_overview: Vec<LocalizedText>,
    /// Per-module score interpretation keyed by module name (lowercase,
    /// e.g. `"accessibility"`, `"performance"`), in key order so that a
    /// serialized report does not vary between runs.
    pub per_module: BTreeMap<String, LocalizedText>,
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
        let accessibility_claim = AccessibilityClaim::from_context(ctx);
        let technical_overview = build_technical_overview_localized(ctx, &accessibility_claim);
        let per_module = build_per_module_localized(ctx);
        let overall_score_band = ScoreBand::from_score(ctx.normalized.score as f32);
        let overall_impact = build_overall_impact_localized(ctx);
        let benchmark_context =
            build_benchmark_context_localized(ctx.normalized.site_quality_score as f32);
        let business_consequence_key = pick_business_consequence_key(ctx, &accessibility_claim);
        let consequence_key = pick_consequence_key(ctx);
        // The sentence names the accessibility score, so that score picks the
        // tier — not the weighted technical value (#896).
        let verdict_key = pick_verdict_key(
            ctx.normalized.score,
            ctx.normalized.risk.severe_legal_flags,
            &ctx.normalized.severity_counts,
        );
        let score_note_key = pick_score_note_key(ctx);
        let batch_verdict_key = String::new();

        Self {
            accessibility_claim,
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
/// The worse (lower-rated) of two bands. Written out rather than relying on
/// `Ord` over the enum's declaration order, which would make the direction of
/// the comparison invisible at the call site.
fn worse_of(a: ScoreBand, b: ScoreBand) -> ScoreBand {
    let rank = |band: ScoreBand| match band {
        ScoreBand::Excellent => 0u8,
        ScoreBand::Good => 1,
        ScoreBand::NeedsImprovement => 2,
        ScoreBand::Weak => 3,
        ScoreBand::Critical => 4,
    };
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

pub fn interpret_security_score_localized(
    score: f32,
    issues: &[crate::security::SecurityIssue],
) -> LocalizedText {
    interpret_band_localized(InterpretArea::Security, security_text_band(score, issues))
}

/// Mobile wording follows the listed issues, like Security's (#976). Small
/// touch targets and a locked zoom stay listed but no longer lower the Mobile
/// score (#921), so the score alone can sit in the top band while the page
/// lists dozens of them; "works reliably without noticeable limitations" must
/// not be printed above that list. Any listed issue caps the wording at
/// `Good`; the number and grade are untouched.
pub fn mobile_text_band(score: f32, issues: &[crate::mobile::MobileIssue]) -> ScoreBand {
    let band = ScoreBand::from_score(score);
    if issues.is_empty() {
        band
    } else {
        worse_of(band, ScoreBand::Good)
    }
}

pub fn interpret_mobile_score_localized(
    score: f32,
    issues: &[crate::mobile::MobileIssue],
) -> LocalizedText {
    interpret_band_localized(InterpretArea::Mobile, mobile_text_band(score, issues))
}

impl ScoreBand {
    /// A score that falls inside this band, for looking the band's label up in
    /// the shared `FIVE_BAND` vocabulary instead of duplicating the labels.
    pub fn representative_score(self) -> f32 {
        match self {
            ScoreBand::Excellent => 95.0,
            ScoreBand::Good => 80.0,
            ScoreBand::NeedsImprovement => 65.0,
            ScoreBand::Weak => 45.0,
            ScoreBand::Critical => 20.0,
        }
    }
}

/// The band the security *wording* uses, which is not always the band the
/// numeric score falls into.
///
/// Exposed so the section's score card can be labelled with the same band as
/// its takeaway. Labelling the card straight from the score produced "95 —
/// Sehr gut" directly above a takeaway reading "Verbesserungswürdig": two
/// labels from one vocabulary for one module.
pub fn security_text_band(score: f32, issues: &[crate::security::SecurityIssue]) -> ScoreBand {
    let band = ScoreBand::from_score(score);
    let has_severe_issue = issues.iter().any(|i| {
        matches!(
            i.severity,
            crate::taxonomy::Severity::Critical | crate::taxonomy::Severity::High
        )
    });
    // The same correction has to work in both directions. Downward: a stack of
    // minor header misses must not trigger the Critical band's "immediate
    // action" wording. Upward: the Excellent band's text claims "no significant
    // security issues found", and the Good band's claims only "minor
    // weaknesses" — neither may be printed above a list of open findings.
    //
    // Confirmed live on casoon.de (2026-09-19): security score 95, three open
    // CSP issues including a High ("CSP allows unsafe-inline scripts without
    // nonce/hash protection"), and the section still read "Sehr gut — keine
    // wesentlichen Sicherheitsauffälligkeiten im geprüften Umfang erkannt."
    // The score is generous by design — `calculate_security_score` caps the
    // combined CSP-quality penalty at the missing-CSP penalty — so the
    // correction belongs to the text, not the number.
    if band == ScoreBand::Critical && !has_severe_issue {
        ScoreBand::Weak
    } else if has_severe_issue {
        worse_of(band, ScoreBand::NeedsImprovement)
    } else if !issues.is_empty() {
        worse_of(band, ScoreBand::Good)
    } else {
        band
    }
}

fn interpret_band_localized(area: InterpretArea, band: ScoreBand) -> LocalizedText {
    use InterpretArea::*;
    use ScoreBand::*;

    localized(match (area, band) {
        (Accessibility, Excellent) => "audit-interpretation-score-accessibility-excellent",
        (Accessibility, Good) => "audit-interpretation-score-accessibility-good",
        (Accessibility, NeedsImprovement) => {
            "audit-interpretation-score-accessibility-needs-improvement"
        }
        (Accessibility, Weak) => "audit-interpretation-score-accessibility-weak",
        (Accessibility, Critical) => "audit-interpretation-score-accessibility-critical",
        (Performance, Excellent) => "audit-interpretation-score-performance-excellent",
        (Performance, Good) => "audit-interpretation-score-performance-good",
        (Performance, NeedsImprovement) => {
            "audit-interpretation-score-performance-needs-improvement"
        }
        (Performance, Weak) => "audit-interpretation-score-performance-weak",
        (Performance, Critical) => "audit-interpretation-score-performance-critical",
        (Security, Excellent) => "audit-interpretation-score-security-excellent",
        (Security, Good) => "audit-interpretation-score-security-good",
        (Security, NeedsImprovement) => "audit-interpretation-score-security-needs-improvement",
        (Security, Weak) => "audit-interpretation-score-security-weak",
        (Security, Critical) => "audit-interpretation-score-security-critical",
        (HtmlConform, Excellent) => "audit-interpretation-score-html-conform-excellent",
        (HtmlConform, Good) => "audit-interpretation-score-html-conform-good",
        (HtmlConform, NeedsImprovement) => {
            "audit-interpretation-score-html-conform-needs-improvement"
        }
        (HtmlConform, Weak) => "audit-interpretation-score-html-conform-weak",
        (HtmlConform, Critical) => "audit-interpretation-score-html-conform-critical",
        (Mobile, Excellent) => "audit-interpretation-score-mobile-excellent",
        (Mobile, Good) => "audit-interpretation-score-mobile-good",
        (Mobile, NeedsImprovement) => "audit-interpretation-score-mobile-needs-improvement",
        (Mobile, Weak) => "audit-interpretation-score-mobile-weak",
        (Mobile, Critical) => "audit-interpretation-score-mobile-critical",
        (Ux, Excellent) => "audit-interpretation-score-ux-excellent",
        (Ux, Good) => "audit-interpretation-score-ux-good",
        (Ux, NeedsImprovement) => "audit-interpretation-score-ux-needs-improvement",
        (Ux, Weak) => "audit-interpretation-score-ux-weak",
        (Ux, Critical) => "audit-interpretation-score-ux-critical",
        (Journey, Excellent) => "audit-interpretation-score-journey-excellent",
        (Journey, Good) => "audit-interpretation-score-journey-good",
        (Journey, NeedsImprovement) => "audit-interpretation-score-journey-needs-improvement",
        (Journey, Weak) => "audit-interpretation-score-journey-weak",
        (Journey, Critical) => "audit-interpretation-score-journey-critical",
    })
}

// ── Private builders ──────────────────────────────────────────────────────────

fn build_technical_overview_localized(
    normalized: &AuditContext<'_>,
    claim: &AccessibilityClaim,
) -> Vec<LocalizedText> {
    let mut bullets = Vec::new();

    let severity = &normalized.normalized.severity_counts;
    let rule_count = normalized.normalized.findings.len();

    // 1. Accessibility pattern
    let a11y = LocalizedText {
        de: accessibility_claim_text(claim, severity, rule_count, false),
        en: accessibility_claim_text(claim, severity, rule_count, true),
    };
    bullets.push(a11y);

    // 2. SEO level
    let seo = if let Some(s) = normalized.raw_seo {
        let key = if s.score >= 85 {
            "audit-interpretation-overview-seo-excellent"
        } else if s.score >= 65 {
            "audit-interpretation-overview-seo-good"
        } else if s.score >= 45 {
            "audit-interpretation-overview-seo-partial"
        } else {
            "audit-interpretation-overview-seo-weak"
        };
        localized_args(key, &[("score", arg(s.score))])
    } else {
        localized("audit-interpretation-overview-seo-none")
    };
    bullets.push(seo);

    // 3. Security level
    let sec = if let Some(s) = normalized.raw_security {
        let (present, total) = s.headers.core_presence();
        if s.score >= 80 {
            // "Fully set" is a claim about the headers, so it comes from the
            // headers and open findings, not from the score alone (#902-3).
            if present == total && s.issues.is_empty() {
                localized_args(
                    "audit-interpretation-overview-security-full",
                    &[("score", arg(s.score))],
                )
            } else {
                localized_args(
                    "audit-interpretation-overview-security-core",
                    &[
                        ("score", arg(s.score)),
                        ("present", arg(present)),
                        ("total", arg(total)),
                    ],
                )
            }
        } else {
            let key = if s.score >= 55 {
                "audit-interpretation-overview-security-basic"
            } else if s.score >= 30 {
                "audit-interpretation-overview-security-several"
            } else {
                "audit-interpretation-overview-security-almost"
            };
            localized_args(key, &[("score", arg(s.score))])
        }
    } else {
        localized("audit-interpretation-overview-security-none")
    };
    bullets.push(sec);

    // 4. Tech complexity (DOM + performance)
    let dom = normalized.normalized.nodes_analyzed;
    let perf_score = normalized.raw_performance.map(|p| p.score.overall);
    // `dom_display` prefers the real DOM count from the Performance
    // module's own vitals when available, so this sentence doesn't state a
    // "DOM-Knoten"/"DOM nodes" figure that contradicts the Performance
    // section's own DOM node count a few paragraphs later in the same
    // report. Falls back to
    // `nodes_analyzed` only when no Performance data is available at all.
    let dom_display = normalized
        .raw_performance
        .and_then(|p| p.vitals.dom_nodes)
        .map(|n| n as usize)
        .unwrap_or(dom);
    // German text groups thousands ("7.594"); the canonical English fields
    // keep the plain digits so the JSON stays unchanged.
    let dom_de = crate::i18n::number::integer(dom_display as i64, false);
    // The class follows the figure the sentence prints (#902-6): classifying
    // on `nodes_analyzed` printed "Gering — 7347 DOM-Knoten" for a page the
    // Performance module itself rates as far above its DOM guideline.
    let with_dom = |key: &str, perf: u32| LocalizedText {
        de: text_args(key, &[("dom", arg(&dom_de)), ("perf", arg(perf))], false),
        en: text_args(key, &[("dom", arg(dom_display)), ("perf", arg(perf))], true),
    };
    let tech = match (dom_display, perf_score) {
        (d, Some(p)) if d > 2000 && p < 60 => {
            with_dom("audit-interpretation-overview-tech-high", p)
        }
        // p is 60-74 here ("Verbesserungswürdig"/"Needs improvement" band,
        // same < 75 "Good" cutoff `render_score_driver_table` uses to call a
        // module a "Risikotreiber") — calling that "stabil"/"stable" a few
        // lines below a table that labels the same score a risk driver was
        // self-contradictory (feedback 2026-09-07). Only p >= 75 earns
        // "stabil" below.
        (d, Some(p)) if d > 2000 && p < 75 => {
            with_dom("audit-interpretation-overview-tech-strain", p)
        }
        // "stabil"/"stable" in parentheses read as reassurance about exactly
        // the thing the action list prioritises ("DOM-Struktur verschlanken")
        // two blocks further up. Both facts, neither arguing
        // with the other.
        (d, Some(p)) if d > 2000 => with_dom("audit-interpretation-overview-tech-holding", p),
        (_d, Some(p)) if p < 60 => with_dom("audit-interpretation-overview-tech-perf-critical", p),
        (_d, Some(p)) if p < 80 => {
            with_dom("audit-interpretation-overview-tech-low-optimizable", p)
        }
        (_d, Some(p)) => with_dom("audit-interpretation-overview-tech-low-stable", p),
        (d, None) if d > 2000 => localized_args(
            "audit-interpretation-overview-tech-unmeasured-high",
            &[("nodes", arg(d))],
        ),
        (d, None) => localized_args(
            "audit-interpretation-overview-tech-unmeasured",
            &[("nodes", arg(d))],
        ),
    };
    bullets.push(tech);

    bullets
}

fn build_per_module_localized(normalized: &AuditContext<'_>) -> BTreeMap<String, LocalizedText> {
    let mut map = BTreeMap::new();

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
            interpret_mobile_score_localized(m.score as f32, &m.issues),
        );
    }

    #[cfg(feature = "experimental")]
    if let Some(ux) = normalized.raw_ux {
        map.insert(
            "ux".to_string(),
            interpret_score_localized(InterpretArea::Ux, ux.score as f32),
        );
    }

    #[cfg(feature = "experimental")]
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

    let user_rating = localized(if score >= 90 && urgent == 0 {
        "audit-interpretation-impact-user-excellent"
    } else if score >= 75 {
        "audit-interpretation-impact-user-good"
    } else if score >= 50 {
        "audit-interpretation-impact-user-limited"
    } else {
        "audit-interpretation-impact-user-heavy"
    });

    let risk_level = localized(if critical >= 2 {
        "audit-interpretation-impact-risk-high"
    } else if critical >= 1 || urgent >= 3 {
        "audit-interpretation-impact-risk-critical-topics"
    } else if score < 70 {
        "audit-interpretation-impact-risk-backlog"
    } else {
        "audit-interpretation-impact-risk-low"
    });

    let conversion = localized(if score < 50 {
        "audit-interpretation-impact-conversion-negative"
    } else if score < 75 {
        "audit-interpretation-impact-conversion-possible"
    } else {
        "audit-interpretation-impact-conversion-low"
    });

    vec![
        (
            localized("audit-interpretation-impact-label-user"),
            user_rating,
        ),
        (
            localized("audit-interpretation-impact-label-risk"),
            risk_level,
        ),
        (
            localized("audit-interpretation-impact-label-conversion"),
            conversion,
        ),
    ]
}

fn build_benchmark_context_localized(score: f32) -> LocalizedText {
    localized(if score >= 95.0 {
        "audit-interpretation-benchmark-top5"
    } else if score >= 90.0 {
        "audit-interpretation-benchmark-top15"
    } else if score >= 80.0 {
        "audit-interpretation-benchmark-upper-third"
    } else if score >= 70.0 {
        "audit-interpretation-benchmark-middle"
    } else if score >= 55.0 {
        "audit-interpretation-benchmark-lower-middle"
    } else if score >= 40.0 {
        "audit-interpretation-benchmark-lower-third"
    } else {
        "audit-interpretation-benchmark-critical"
    })
}

fn pick_business_consequence_key(
    normalized: &AuditContext<'_>,
    claim: &AccessibilityClaim,
) -> String {
    let critical = normalized.normalized.severity_counts.critical;
    let total = normalized.normalized.severity_counts.total;
    let score = normalized.normalized.score;

    match claim.kind {
        AccessibilityClaimKind::NoAutomatedViolationsFound => {
            return "business-consequence-clean".to_string();
        }
        AccessibilityClaimKind::NotMeasured | AccessibilityClaimKind::IncompleteRun => {
            return "business-consequence-unverified".to_string();
        }
        AccessibilityClaimKind::ViolationsFound => {}
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

/// The verdict tier of an accessibility score, banded like [`ScoreBand`]
/// (90/75/60/40), so the cover sentence and the score label beside it can
/// never name different tiers (#896). `prefix` is `verdict-tier` (single
/// audit) or `verdict-batch`.
pub fn verdict_tier_key(prefix: &str, score: u32) -> String {
    let tier = match ScoreBand::from_score(score as f32) {
        ScoreBand::Excellent => "excellent",
        ScoreBand::Good => "solid",
        ScoreBand::NeedsImprovement => "stable",
        ScoreBand::Weak => "deficient",
        ScoreBand::Critical => "critical",
    };
    format!("{prefix}-{tier}")
}

/// The cover sentence of the top tier says "no structural problem, the rest is
/// polish". That is only true while nothing above medium is open, so the
/// wording follows the highest open severity (#899).
fn top_tier_key(key: String, counts: &crate::audit::normalized::SeverityCounts) -> String {
    if key != "verdict-tier-excellent" {
        return key;
    }
    let suffix = if counts.critical > 0 {
        "critical"
    } else if counts.high > 0 {
        "high"
    } else if counts.medium > 0 {
        "medium"
    } else {
        return key;
    };
    format!("{key}-{suffix}")
}

fn pick_verdict_key(
    score: u32,
    severe_legal_flags: usize,
    counts: &crate::audit::normalized::SeverityCounts,
) -> String {
    match legal_gate("verdict-tier", score, severe_legal_flags) {
        Some(key) => key,
        None => top_tier_key(verdict_tier_key("verdict-tier", score), counts),
    }
}

/// The batch cover sentence: the band of the average accessibility score,
/// under the same legal gate as a single page (#356) — a batch that fails on
/// legal flags must not read "a very good result" (#1039).
pub fn batch_verdict_key(average: u32, severe_legal_flags: usize) -> String {
    legal_gate("verdict-batch", average, severe_legal_flags)
        .unwrap_or_else(|| verdict_tier_key("verdict-batch", average))
}

/// From this many rules with a High/Critical legal flag the findings are
/// systemic, not a single defect: only then may a passable score read as
/// "structural backlog" (#1131).
const SYSTEMIC_SEVERE_LEGAL_FLAGS: usize = 3;

/// Legal flags of High/Critical severity never receive a reassuring tier,
/// regardless of an otherwise passable score (#356). The gate caps the tier:
/// at `violations` ("WCAG violations were found"), and only a systemic
/// concentration of flags (several rules) lowers it to `deficient`
/// ("structural backlog"); one confirmed violation never does (#1131). A
/// score that already reads `deficient` or `critical` stays. `None` when the
/// gate does not apply.
fn legal_gate(prefix: &str, score: u32, severe_legal_flags: usize) -> Option<String> {
    let key = verdict_tier_key(prefix, score);
    let deficient = format!("{prefix}-deficient");
    let critical = format!("{prefix}-critical");
    if severe_legal_flags == 0 || key == deficient || key == critical {
        return None;
    }
    Some(if severe_legal_flags >= SYSTEMIC_SEVERE_LEGAL_FLAGS {
        deficient
    } else {
        format!("{prefix}-violations")
    })
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

    /// #896: the cover sentence and the score label name the same tier,
    /// at every band edge, for single and batch.
    fn mobile_issue(issue_type: &str) -> crate::mobile::MobileIssue {
        crate::mobile::MobileIssue {
            category: "Touch Targets".into(),
            issue_type: issue_type.into(),
            message: String::new(),
            severity: crate::taxonomy::Severity::Medium,
            impact: String::new(),
            values: Default::default(),
        }
    }

    /// #976: a Mobile score of 100 with listed issues (37 small targets stay
    /// listed but are not charged) must not read "without noticeable
    /// limitations"; without issues the band is the score's.
    #[test]
    fn mobile_wording_is_capped_while_issues_are_listed() {
        assert_eq!(mobile_text_band(100.0, &[]), ScoreBand::Excellent);
        let issues = [mobile_issue("small_targets")];
        assert_eq!(mobile_text_band(100.0, &issues), ScoreBand::Good);
        // A lower score is never lifted.
        assert_eq!(mobile_text_band(50.0, &issues), ScoreBand::Weak);

        let clean = interpret_mobile_score_localized(100.0, &[]);
        assert!(
            clean.de.contains("ohne erkennbare Einschränkungen"),
            "{}",
            clean.de
        );
        for text in [
            interpret_mobile_score_localized(100.0, &issues),
            interpret_mobile_score_localized(95.0, &issues),
        ] {
            assert!(
                !text.de.contains("ohne erkennbare Einschränkungen"),
                "{}",
                text.de
            );
            assert!(
                !text.en.contains("no noticeable limitations"),
                "{}",
                text.en
            );
        }
    }

    #[test]
    fn verdict_tier_follows_the_score_bands() {
        for score in 0..=100u32 {
            let expected = match ScoreBand::from_score(score as f32) {
                ScoreBand::Excellent => "excellent",
                ScoreBand::Good => "solid",
                ScoreBand::NeedsImprovement => "stable",
                ScoreBand::Weak => "deficient",
                ScoreBand::Critical => "critical",
            };
            assert_eq!(
                verdict_tier_key("verdict-tier", score),
                format!("verdict-tier-{expected}"),
                "score {score}"
            );
            assert_eq!(
                verdict_tier_key("verdict-batch", score),
                format!("verdict-batch-{expected}")
            );
        }
        // Edges of the score bands (90/75/60/40) are the edges of the tiers.
        assert_eq!(
            verdict_tier_key("verdict-tier", 93),
            "verdict-tier-excellent"
        );
        assert_eq!(verdict_tier_key("verdict-tier", 74), "verdict-tier-stable");
        // A legal finding never keeps a reassuring tier.
        let none = crate::audit::normalized::SeverityCounts::default();
        // One confirmed violation caps the tier at "violations found"; it is
        // no "structural backlog" (#1131). Several rules are systemic.
        assert_eq!(pick_verdict_key(93, 1, &none), "verdict-tier-violations");
        assert_eq!(pick_verdict_key(70, 2, &none), "verdict-tier-violations");
        assert_eq!(pick_verdict_key(93, 3, &none), "verdict-tier-deficient");
        assert_eq!(pick_verdict_key(50, 1, &none), "verdict-tier-deficient");
        assert_eq!(pick_verdict_key(30, 1, &none), "verdict-tier-critical");
        assert_eq!(pick_verdict_key(93, 0, &none), "verdict-tier-excellent");
    }

    /// #899: the top tier never says "no structural problem" while a high or
    /// critical finding is open; the wording follows the highest open severity.
    #[test]
    fn top_tier_wording_follows_highest_open_severity() {
        use crate::audit::normalized::SeverityCounts;
        let c = |critical, high, medium, low| SeverityCounts {
            critical,
            high,
            medium,
            low,
            total: critical + high + medium + low,
        };
        let key = |counts| pick_verdict_key(93, 0, &counts);
        assert_eq!(key(c(0, 0, 0, 4)), "verdict-tier-excellent");
        assert_eq!(key(c(0, 0, 2, 4)), "verdict-tier-excellent-medium");
        assert_eq!(key(c(0, 1, 2, 4)), "verdict-tier-excellent-high");
        assert_eq!(key(c(1, 1, 2, 4)), "verdict-tier-excellent-critical");
        // Lower tiers keep their own wording.
        assert_eq!(
            pick_verdict_key(80, 0, &c(1, 1, 0, 0)),
            "verdict-tier-solid"
        );

        for locale in ["de", "en"] {
            let i18n = crate::i18n::I18n::new(locale).unwrap();
            let args = [("url", "u".to_string()), ("score", "93".to_string())];
            for suffix in ["", "-medium", "-high", "-critical"] {
                let key = format!("verdict-tier-excellent{suffix}");
                let text = i18n.t_args(&key, &args);
                assert_ne!(text, key, "{locale}: {key} missing");
                if locale == "en" {
                    assert!(!text.contains(['ä', 'ö', 'ü', 'ß']), "{text}");
                }
                let claims_none = text.contains("kein strukturelles Problem")
                    || text.contains("no structural problem");
                let polish = text.contains("Feinschliff") || text.contains("polish");
                assert_eq!(claims_none && polish, suffix.is_empty(), "{locale}: {text}");
            }
        }
    }

    /// #896: every tier exists in both locales and the English text carries no
    /// German, the German none of the English tier words.
    #[test]
    fn verdict_tier_texts_exist_in_both_locales_and_say_the_tier() {
        for locale in ["de", "en"] {
            let i18n = crate::i18n::I18n::new(locale).unwrap();
            for prefix in ["verdict-tier", "verdict-batch"] {
                for tier in [
                    "excellent",
                    "solid",
                    "stable",
                    "violations",
                    "deficient",
                    "critical",
                ] {
                    let key = format!("{prefix}-{tier}");
                    let text = i18n.t_args(
                        &key,
                        &[
                            ("url", "https://example.com".to_string()),
                            ("score", "80".to_string()),
                            ("total_urls", "3".to_string()),
                        ],
                    );
                    assert_ne!(text, key, "{locale}: {key} is missing");
                    if locale == "en" {
                        assert!(!text.contains(['ä', 'ö', 'ü', 'ß']), "{text}");
                    }
                }
            }
            // "stable" is the lower tier: only it may call the foundation stable.
            let solid = i18n.t_args(
                "verdict-tier-solid",
                &[("url", "u".to_string()), ("score", "80".to_string())],
            );
            assert!(
                !solid.contains("stabil") && !solid.contains("stable"),
                "{solid}"
            );
        }
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
        // The WCAG engine's own
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
            runs: 1,
            stability: Default::default(),
        });

        let ctx = crate::audit::normalized::normalize(&report);
        let bullets =
            build_technical_overview_localized(&ctx, &AccessibilityClaim::from_context(&ctx));
        let tech_bullet = bullets
            .iter()
            .find(|b| b.de.starts_with("Tech-Komplexität"))
            .expect("technical overview must include a tech-complexity bullet");

        assert!(
            tech_bullet.de.contains("20.368"),
            "tech-complexity bullet must state the real DOM element count, got: {}",
            tech_bullet.de
        );
        assert!(
            !tech_bullet.de.contains("3974"),
            "tech-complexity bullet must not state the WCAG checked-node count as if it were the DOM count, got: {}",
            tech_bullet.de
        );
    }

    /// #902-6 and #902-3: the tech-complexity class follows the DOM figure the
    /// sentence prints, and "headers fully set" needs the headers and the
    /// absence of open findings, not just a high score.
    #[test]
    fn technical_overview_claims_follow_the_printed_facts() {
        use crate::audit::PerformanceResults;
        use crate::performance::{calculate_performance_score, WebVitals};
        use crate::security::{SecurityAnalysis, SecurityHeaders, SslInfo};
        use crate::wcag::WcagResults;
        use crate::WcagLevel;

        let mut results = WcagResults::new();
        results.nodes_checked = 500; // far below the real DOM count
        let mut report = crate::audit::AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1_000,
        );
        let vitals = WebVitals {
            dom_nodes: Some(7347),
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
            runs: 1,
            stability: Default::default(),
        });
        report.security = Some(SecurityAnalysis {
            score: 95,
            grade: "A".to_string(),
            headers: SecurityHeaders::default(),
            ssl: SslInfo::default(),
            issues: vec![severe_csp_issue()],
            protection: Default::default(),
            sourcemap_leaks: Default::default(),
            fetch: Default::default(),
            recommendations: vec![],
        });

        let ctx = crate::audit::normalized::normalize(&report);
        let bullets =
            build_technical_overview_localized(&ctx, &AccessibilityClaim::from_context(&ctx));
        let find = |prefix: &str| {
            bullets
                .iter()
                .find(|b| b.de.starts_with(prefix))
                .unwrap_or_else(|| panic!("missing bullet {prefix}"))
        };

        let tech = find("Tech-Komplexität");
        assert!(tech.de.contains("7.347"), "{}", tech.de);
        assert!(
            !tech.de.contains("Gering") && !tech.en.contains("Low"),
            "7347 DOM nodes must not be classed as low complexity: {}",
            tech.de
        );

        let security = find("Security-Level");
        assert!(
            !security.de.contains("vollständig gesetzt"),
            "no all-clear next to an open CSP finding and missing headers: {}",
            security.de
        );
        assert!(
            !security.en.contains("fully set"),
            "no all-clear next to an open CSP finding and missing headers: {}",
            security.en
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
            score_deduction: 0,
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

    fn severe_csp_issue() -> SecurityIssue {
        SecurityIssue {
            header: "Content-Security-Policy".to_string(),
            issue_type: "unsafe_inline_script".to_string(),
            message: "CSP allows unsafe-inline scripts without nonce/hash protection".to_string(),
            severity: Severity::High,
            tier: crate::security::HeaderTier::Baseline,
            values: Default::default(),
            score_deduction: 0,
        }
    }

    /// The Excellent band claims "no significant security issues
    /// found within the scope checked". Confirmed live on casoon.de
    /// (2026-09-19): security score 95 with three open CSP findings, one of
    /// them High, still printed that sentence.
    #[test]
    fn interpret_security_excellent_band_never_claims_a_clean_result_with_open_issues() {
        let issues = vec![severe_csp_issue()];
        assert_eq!(ScoreBand::from_score(95.0), ScoreBand::Excellent);

        let text = interpret_security_score_localized(95.0, &issues);
        assert!(
            !text
                .de
                .contains("keine wesentlichen Sicherheitsauffälligkeiten"),
            "claims a clean result above an open High finding: {}",
            text.de,
        );
        assert!(
            !text.en.contains("no significant security issues"),
            "claims a clean result above an open High finding: {}",
            text.en,
        );
        // A severe issue caps the wording at "needs improvement" — not at
        // "critical", which the score does not support.
        assert!(
            text.de.starts_with("Verbesserungswürdig"),
            "got: {}",
            text.de
        );
        assert!(text.en.starts_with("Needs improvement"), "got: {}", text.en);
    }

    /// Minor issues alone must not claim a clean result either, but they also
    /// must not be dressed up as severe.
    #[test]
    fn interpret_security_minor_issues_cap_the_band_at_good() {
        use crate::security::HeaderTier;

        let issues = vec![low_severity_context_issue(
            "Cross-Origin-Resource-Policy",
            HeaderTier::ContextDependent,
        )];
        let text = interpret_security_score_localized(95.0, &issues);
        assert!(text.de.starts_with("Gut"), "got: {}", text.de);
        assert!(text.en.starts_with("Good"), "got: {}", text.en);
    }

    /// The claim is still available when it is true — a genuinely clean run
    /// must keep reading as Excellent.
    #[test]
    fn interpret_security_clean_run_still_reads_excellent() {
        let text = interpret_security_score_localized(95.0, &[]);
        assert!(
            text.de
                .contains("keine wesentlichen Sicherheitsauffälligkeiten"),
            "got: {}",
            text.de,
        );
        assert!(
            text.en.contains("no significant security issues"),
            "got: {}",
            text.en
        );
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
                score_deduction: 0,
            },
            low_severity_context_issue("Cross-Origin-Opener-Policy", HeaderTier::ContextDependent),
        ];

        let text = interpret_security_score_localized(20.0, &issues);
        assert!(text.de.starts_with("Kritisch"), "got: {}", text.de);
        assert!(text.en.starts_with("Critical"), "got: {}", text.en);
    }

    /// A band that is already accurate must pass through untouched: at score
    /// 80 with one minor issue the generic band (Good) already says "minor
    /// weaknesses were identified", so the correction has nothing to do.
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

        let bullets = build_technical_overview_localized(
            &normalized,
            &AccessibilityClaim::from_context(&normalized),
        );
        assert_eq!(bullets.len(), 4, "always exactly 4 overview bullets");
        for bullet in &bullets {
            assert!(!bullet.de.is_empty());
            assert!(!bullet.en.is_empty());
        }
    }

    fn clean_context() -> crate::audit::normalized::AuditContext<'static> {
        let report = Box::leak(Box::new(crate::audit::report::AuditReport::new(
            "https://example.com".to_string(),
            crate::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            100,
        )));
        crate::audit::normalized::normalize(report)
    }

    /// #782: a complete run without findings is "no automated violations",
    /// its own state — the stored statement names the scope and never claims
    /// conformance; the JSON says conformance and BFSG applicability are open.
    #[test]
    fn zero_findings_never_claim_conformance() {
        let ctx = clean_context();
        let interp = Interpretation::from_context(&ctx);
        let claim = &interp.accessibility_claim;
        assert_eq!(
            claim.kind,
            AccessibilityClaimKind::NoAutomatedViolationsFound
        );
        assert!(!claim.manual_review_criteria.is_empty());
        assert!(claim.automated_criteria < claim.total_criteria);

        let bullet = &interp.technical_overview[0];
        for text in [&bullet.en, &bullet.de] {
            let lower = text.to_lowercase();
            assert!(!lower.contains("fully conformant"), "{text}");
            assert!(!lower.contains("vollständig konform"), "{text}");
        }
        assert!(bullet.en.contains("conformance not determined"));
        assert!(bullet.de.contains("keine Konformitätsaussage"));

        let json = serde_json::to_value(claim).unwrap();
        assert_eq!(json["kind"], "no_automated_violations_found");
        assert_eq!(json["conformance"], "not_determined");
        assert_eq!(json["legal_applicability"], "not_determined");
        assert_eq!(
            json["remaining_manual_review"][0],
            "criteria_outside_automated_scope"
        );
    }

    /// #782: a partial run or one whose accessibility module failed found
    /// nothing — that must stay "incomplete"/"not measured", not read clean.
    #[test]
    fn partial_or_failed_runs_without_findings_are_not_clean() {
        let mut ctx = clean_context();
        ctx.normalized.execution.quality.status = AuditQualityStatus::Partial;
        let interp = Interpretation::from_context(&ctx);
        assert_eq!(
            interp.accessibility_claim.kind,
            AccessibilityClaimKind::IncompleteRun
        );
        assert!(interp
            .accessibility_claim
            .limits
            .remaining_manual_review
            .contains(&ManualReview::RepeatIncompleteRun));
        assert_eq!(
            interp.business_consequence_key,
            "business-consequence-unverified"
        );
        assert!(interp.technical_overview[0].en.contains("incomplete"));

        let mut ctx = clean_context();
        ctx.normalized
            .execution
            .module_runs
            .push(crate::audit::ModuleRun {
                module: "accessibility".to_string(),
                status: ExecutionStatus::Failed,
                ..Default::default()
            });
        let interp = Interpretation::from_context(&ctx);
        assert_eq!(
            interp.accessibility_claim.kind,
            AccessibilityClaimKind::NotMeasured
        );
        assert!(interp.technical_overview[0].en.contains("Not measured"));
    }

    /// `per_module` serializes in key order, not in insertion order (which
    /// puts security before mobile) or a run-dependent hash order, so a
    /// serialized `NormalizedReport` is the same in every run.
    #[test]
    fn per_module_serializes_in_key_order() {
        use crate::security::{SecurityAnalysis, SecurityHeaders, SslInfo};

        let mut report = crate::audit::AuditReport::new(
            "https://example.com".to_string(),
            crate::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            100,
        )
        .with_mobile(crate::mobile::MobileFriendliness {
            score: 80,
            viewport: Default::default(),
            touch_targets: Default::default(),
            font_sizes: Default::default(),
            content_sizing: Default::default(),
            issues: vec![],
        });
        report.security = Some(SecurityAnalysis {
            score: 95,
            grade: "A".to_string(),
            headers: SecurityHeaders::default(),
            ssl: SslInfo::default(),
            issues: vec![],
            protection: Default::default(),
            sourcemap_leaks: Default::default(),
            fetch: Default::default(),
            recommendations: vec![],
        });
        let ctx = crate::audit::normalized::normalize(&report);
        let json = serde_json::to_string(&Interpretation::from_context(&ctx)).unwrap();
        let positions: Vec<usize> = ["accessibility", "mobile", "security"]
            .iter()
            .map(|key| json.find(&format!(r#""{key}":{{"#)).expect(key))
            .collect();
        assert!(positions.windows(2).all(|w| w[0] < w[1]), "{json}");
    }

    /// Guard (#406): the English claim texts carry no German characters.
    #[test]
    fn accessibility_claim_texts_en_have_no_umlauts() {
        let ctx = clean_context();
        let mut claim = AccessibilityClaim::from_context(&ctx);
        let severity = &ctx.normalized.severity_counts;
        for kind in [
            AccessibilityClaimKind::NotMeasured,
            AccessibilityClaimKind::IncompleteRun,
            AccessibilityClaimKind::ViolationsFound,
            AccessibilityClaimKind::NoAutomatedViolationsFound,
        ] {
            claim.kind = kind;
            let text = accessibility_claim_text(&claim, severity, 0, true);
            assert!(
                !text.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "{kind:?}: {text}"
            );
        }
    }
}
