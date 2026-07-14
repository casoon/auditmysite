//! EN 301 549 (chapter 9, "Web") clause mapping.
//!
//! Promotes the WCAG↔EN table that used to live only inside
//! `screen_reader::bfsg` into a canonical module: all 50 WCAG 2.1 A/AA
//! success criteria mapped 1:1 onto their EN 301 549 chapter-9 clause number
//! (`9.x.y.z` mirrors WCAG `x.y.z`), with English/German titles, plus a pure
//! per-clause status roll-up over `NormalizedFinding`s.
//!
//! This module produces technical evidence only — never a
//! Barrierefreiheitserklärung. See `derive_annex` and the disclaimer text
//! that accompanies it in the output layer.

use crate::audit::normalized::NormalizedFinding;

/// EN 301 549 edition this mapping targets. WCAG 2.1 A/AA aligns 1:1 with
/// V3.2.1's chapter 9 ("Web"); a future WCAG 2.2 / EN 301 549 v4 mapping is
/// tracked via a version bump here, not by editing this table in place.
pub const EN301549_VERSION: &str = "EN 301 549 V3.2.1 (2021-03)";

/// Version of this table's own derivation logic/shape (not the standard
/// edition — see `EN301549_VERSION`). Bump when `EN301549_WEB_CLAUSES` or the
/// `derive_annex` status semantics change in a way consumers should notice.
pub const EN301549_MAPPING_VERSION: u32 = 1;

/// Cautious, non-legal-conclusion disclaimer shown with the EN 301 549 annex
/// (JSON `en301549_annex.disclaimer` and the PDF annex callout). Single
/// source of truth for both output layers — do not duplicate this text
/// elsewhere. No statutory citations, no "compliant"/conformity claims.
///
/// **This wording still needs a lawyer's review before customer-facing use**
/// (see `plans/bfsg-en301549-mapping.md`, "Risks" — legal wording).
pub const EN301549_DISCLAIMER_DE: &str = "Diese Übersicht ist keine Barrierefreiheitserklärung und ersetzt keine solche. Sie deckt ausschließlich den automatisch prüfbaren Teil von EN 301 549, Kapitel 9 (Web), im geprüften Umfang ab. Kriterien mit manuellem Prüfbedarf sowie die Kapitel 5–8 und 10–13 wurden nicht bewertet. Die Angaben können als technische Zuarbeit für eine Erklärung dienen.";

/// English equivalent of `EN301549_DISCLAIMER_DE`, sentence-for-sentence —
/// no added claims beyond the German draft.
pub const EN301549_DISCLAIMER_EN: &str = "This overview is not an accessibility statement and does not replace one. It covers only the automatically testable part of EN 301 549, chapter 9 (Web), within the audited scope. Criteria requiring manual review, as well as chapters 5\u{2013}8 and 10\u{2013}13, were not assessed. This information can serve as technical input for drafting a statement.";

/// One WCAG 2.1 A/AA success criterion mapped onto its EN 301 549 clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct En301549Clause {
    /// WCAG success criterion id, e.g. `"1.1.1"`.
    pub wcag: &'static str,
    /// EN 301 549 clause number, always `"9." + wcag` for the Web chapter.
    pub en_clause: &'static str,
    /// WCAG conformance level: `"A"` or `"AA"`.
    pub wcag_level: &'static str,
    /// Official English WCAG 2.1 success criterion title.
    pub title_en: &'static str,
    /// German success criterion title (BIK/DIAS WCAG 2.1 translation).
    pub title_de: &'static str,
}

/// All 50 WCAG 2.1 Level A (30) / Level AA (20) success criteria, in WCAG
/// numeric order, mapped onto EN 301 549 V3.2.1 clause 9.
pub const EN301549_WEB_CLAUSES: &[En301549Clause] = &[
    En301549Clause {
        wcag: "1.1.1",
        en_clause: "9.1.1.1",
        wcag_level: "A",
        title_en: "Non-text Content",
        title_de: "Nicht-Text-Inhalt",
    },
    En301549Clause {
        wcag: "1.2.1",
        en_clause: "9.1.2.1",
        wcag_level: "A",
        title_en: "Audio-only and Video-only (Prerecorded)",
        title_de: "Nur Audio oder nur Video (aufgezeichnet)",
    },
    En301549Clause {
        wcag: "1.2.2",
        en_clause: "9.1.2.2",
        wcag_level: "A",
        title_en: "Captions (Prerecorded)",
        title_de: "Untertitel (aufgezeichnet)",
    },
    En301549Clause {
        wcag: "1.2.3",
        en_clause: "9.1.2.3",
        wcag_level: "A",
        title_en: "Audio Description or Media Alternative (Prerecorded)",
        title_de: "Audiodeskription oder Medienalternative (aufgezeichnet)",
    },
    En301549Clause {
        wcag: "1.2.4",
        en_clause: "9.1.2.4",
        wcag_level: "AA",
        title_en: "Captions (Live)",
        title_de: "Untertitel (live)",
    },
    En301549Clause {
        wcag: "1.2.5",
        en_clause: "9.1.2.5",
        wcag_level: "AA",
        title_en: "Audio Description (Prerecorded)",
        title_de: "Audiodeskription (aufgezeichnet)",
    },
    En301549Clause {
        wcag: "1.3.1",
        en_clause: "9.1.3.1",
        wcag_level: "A",
        title_en: "Info and Relationships",
        title_de: "Informationen und Beziehungen",
    },
    En301549Clause {
        wcag: "1.3.2",
        en_clause: "9.1.3.2",
        wcag_level: "A",
        title_en: "Meaningful Sequence",
        title_de: "Sinnvolle Reihenfolge",
    },
    En301549Clause {
        wcag: "1.3.3",
        en_clause: "9.1.3.3",
        wcag_level: "A",
        title_en: "Sensory Characteristics",
        title_de: "Sensorische Eigenschaften",
    },
    En301549Clause {
        wcag: "1.3.4",
        en_clause: "9.1.3.4",
        wcag_level: "AA",
        title_en: "Orientation",
        title_de: "Ausrichtung",
    },
    En301549Clause {
        wcag: "1.3.5",
        en_clause: "9.1.3.5",
        wcag_level: "AA",
        title_en: "Identify Input Purpose",
        title_de: "Eingabezweck erkennen",
    },
    En301549Clause {
        wcag: "1.4.1",
        en_clause: "9.1.4.1",
        wcag_level: "A",
        title_en: "Use of Color",
        title_de: "Verwendung von Farbe",
    },
    En301549Clause {
        wcag: "1.4.2",
        en_clause: "9.1.4.2",
        wcag_level: "A",
        title_en: "Audio Control",
        title_de: "Kontrolle des Tons",
    },
    En301549Clause {
        wcag: "1.4.3",
        en_clause: "9.1.4.3",
        wcag_level: "AA",
        title_en: "Contrast (Minimum)",
        title_de: "Kontrast (Minimum)",
    },
    En301549Clause {
        wcag: "1.4.4",
        en_clause: "9.1.4.4",
        wcag_level: "AA",
        title_en: "Resize Text",
        title_de: "Textgröße ändern",
    },
    En301549Clause {
        wcag: "1.4.5",
        en_clause: "9.1.4.5",
        wcag_level: "AA",
        title_en: "Images of Text",
        title_de: "Als Bild dargestellter Text",
    },
    En301549Clause {
        wcag: "1.4.10",
        en_clause: "9.1.4.10",
        wcag_level: "AA",
        title_en: "Reflow",
        title_de: "Textumfluss",
    },
    En301549Clause {
        wcag: "1.4.11",
        en_clause: "9.1.4.11",
        wcag_level: "AA",
        title_en: "Non-text Contrast",
        title_de: "Kontraste von Nicht-Text-Inhalten",
    },
    En301549Clause {
        wcag: "1.4.12",
        en_clause: "9.1.4.12",
        wcag_level: "AA",
        title_en: "Text Spacing",
        title_de: "Abstände von Text",
    },
    En301549Clause {
        wcag: "1.4.13",
        en_clause: "9.1.4.13",
        wcag_level: "AA",
        title_en: "Content on Hover or Focus",
        title_de: "Inhalt bei Hover oder Fokus",
    },
    En301549Clause {
        wcag: "2.1.1",
        en_clause: "9.2.1.1",
        wcag_level: "A",
        title_en: "Keyboard",
        title_de: "Tastatur",
    },
    En301549Clause {
        wcag: "2.1.2",
        en_clause: "9.2.1.2",
        wcag_level: "A",
        title_en: "No Keyboard Trap",
        title_de: "Keine Tastaturfalle",
    },
    En301549Clause {
        wcag: "2.1.4",
        en_clause: "9.2.1.4",
        wcag_level: "A",
        title_en: "Character Key Shortcuts",
        title_de: "Zeichenbasierte Tastenkombinationen",
    },
    En301549Clause {
        wcag: "2.2.1",
        en_clause: "9.2.2.1",
        wcag_level: "A",
        title_en: "Timing Adjustable",
        title_de: "Zeitlimits anpassbar",
    },
    En301549Clause {
        wcag: "2.2.2",
        en_clause: "9.2.2.2",
        wcag_level: "A",
        title_en: "Pause, Stop, Hide",
        title_de: "Pausieren, Beenden, Ausblenden",
    },
    En301549Clause {
        wcag: "2.3.1",
        en_clause: "9.2.3.1",
        wcag_level: "A",
        title_en: "Three Flashes or Below Threshold",
        title_de: "Schwellenwert von drei Blitzen oder darunter",
    },
    En301549Clause {
        wcag: "2.4.1",
        en_clause: "9.2.4.1",
        wcag_level: "A",
        title_en: "Bypass Blocks",
        title_de: "Bereiche überspringen",
    },
    En301549Clause {
        wcag: "2.4.2",
        en_clause: "9.2.4.2",
        wcag_level: "A",
        title_en: "Page Titled",
        title_de: "Seite hat Titel",
    },
    En301549Clause {
        wcag: "2.4.3",
        en_clause: "9.2.4.3",
        wcag_level: "A",
        title_en: "Focus Order",
        title_de: "Fokus-Reihenfolge",
    },
    En301549Clause {
        wcag: "2.4.4",
        en_clause: "9.2.4.4",
        wcag_level: "A",
        title_en: "Link Purpose (In Context)",
        title_de: "Linkzweck (im Kontext)",
    },
    En301549Clause {
        wcag: "2.4.5",
        en_clause: "9.2.4.5",
        wcag_level: "AA",
        title_en: "Multiple Ways",
        title_de: "Mehrere Wege",
    },
    En301549Clause {
        wcag: "2.4.6",
        en_clause: "9.2.4.6",
        wcag_level: "AA",
        title_en: "Headings and Labels",
        title_de: "Überschriften und Beschriftungen",
    },
    En301549Clause {
        wcag: "2.4.7",
        en_clause: "9.2.4.7",
        wcag_level: "AA",
        title_en: "Focus Visible",
        title_de: "Fokus sichtbar",
    },
    En301549Clause {
        wcag: "2.5.1",
        en_clause: "9.2.5.1",
        wcag_level: "A",
        title_en: "Pointer Gestures",
        title_de: "Zeigergesten",
    },
    En301549Clause {
        wcag: "2.5.2",
        en_clause: "9.2.5.2",
        wcag_level: "A",
        title_en: "Pointer Cancellation",
        title_de: "Abbruch von Zeigerfunktionen",
    },
    En301549Clause {
        wcag: "2.5.3",
        en_clause: "9.2.5.3",
        wcag_level: "A",
        title_en: "Label in Name",
        title_de: "Beschriftung im Namen",
    },
    En301549Clause {
        wcag: "2.5.4",
        en_clause: "9.2.5.4",
        wcag_level: "A",
        title_en: "Motion Actuation",
        title_de: "Bewegungsaktivierung",
    },
    En301549Clause {
        wcag: "3.1.1",
        en_clause: "9.3.1.1",
        wcag_level: "A",
        title_en: "Language of Page",
        title_de: "Sprache der Seite",
    },
    En301549Clause {
        wcag: "3.1.2",
        en_clause: "9.3.1.2",
        wcag_level: "AA",
        title_en: "Language of Parts",
        title_de: "Sprache von Teilen",
    },
    En301549Clause {
        wcag: "3.2.1",
        en_clause: "9.3.2.1",
        wcag_level: "A",
        title_en: "On Focus",
        title_de: "Bei Fokus",
    },
    En301549Clause {
        wcag: "3.2.2",
        en_clause: "9.3.2.2",
        wcag_level: "A",
        title_en: "On Input",
        title_de: "Bei Eingabe",
    },
    En301549Clause {
        wcag: "3.2.3",
        en_clause: "9.3.2.3",
        wcag_level: "AA",
        title_en: "Consistent Navigation",
        title_de: "Konsistente Navigation",
    },
    En301549Clause {
        wcag: "3.2.4",
        en_clause: "9.3.2.4",
        wcag_level: "AA",
        title_en: "Consistent Identification",
        title_de: "Konsistente Identifikation",
    },
    En301549Clause {
        wcag: "3.3.1",
        en_clause: "9.3.3.1",
        wcag_level: "A",
        title_en: "Error Identification",
        title_de: "Fehlererkennung",
    },
    En301549Clause {
        wcag: "3.3.2",
        en_clause: "9.3.3.2",
        wcag_level: "A",
        title_en: "Labels or Instructions",
        title_de: "Beschriftungen oder Anweisungen",
    },
    En301549Clause {
        wcag: "3.3.3",
        en_clause: "9.3.3.3",
        wcag_level: "AA",
        title_en: "Error Suggestion",
        title_de: "Fehlerhinweis",
    },
    En301549Clause {
        wcag: "3.3.4",
        en_clause: "9.3.3.4",
        wcag_level: "AA",
        title_en: "Error Prevention (Legal, Financial, Data)",
        title_de: "Fehlervermeidung (rechtlich, finanziell, Daten)",
    },
    En301549Clause {
        wcag: "4.1.1",
        en_clause: "9.4.1.1",
        wcag_level: "A",
        title_en: "Parsing",
        title_de: "Parsen",
    },
    En301549Clause {
        wcag: "4.1.2",
        en_clause: "9.4.1.2",
        wcag_level: "A",
        title_en: "Name, Role, Value",
        title_de: "Name, Rolle, Wert",
    },
    En301549Clause {
        wcag: "4.1.3",
        en_clause: "9.4.1.3",
        wcag_level: "AA",
        title_en: "Status Messages",
        title_de: "Statusmeldungen",
    },
];

/// EN 301 549 chapters outside "Web" (clause 9) — never assessed by this
/// tool, which audits web content only. Listed by chapter number so the
/// annex names the boundary of its own scope instead of implying full
/// EN 301 549 coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfScopeChapter {
    pub chapter: &'static str,
    pub title_en: &'static str,
    pub title_de: &'static str,
}

pub const OUT_OF_SCOPE_CHAPTERS: &[OutOfScopeChapter] = &[
    OutOfScopeChapter {
        chapter: "5",
        title_en: "Generic requirements",
        title_de: "Allgemeine Anforderungen",
    },
    OutOfScopeChapter {
        chapter: "6",
        title_en: "ICT with two-way voice communication",
        title_de: "IKT mit Zwei-Wege-Sprachkommunikation",
    },
    OutOfScopeChapter {
        chapter: "7",
        title_en: "ICT with video capabilities",
        title_de: "IKT mit Videofunktionen",
    },
    OutOfScopeChapter {
        chapter: "8",
        title_en: "Hardware",
        title_de: "Hardware",
    },
    OutOfScopeChapter {
        chapter: "10",
        title_en: "Non-web documents",
        title_de: "Nicht-Web-Dokumente",
    },
    OutOfScopeChapter {
        chapter: "11",
        title_en: "Software",
        title_de: "Software",
    },
    OutOfScopeChapter {
        chapter: "12",
        title_en: "Documentation and support services",
        title_de: "Dokumentation und Support-Dienste",
    },
    OutOfScopeChapter {
        chapter: "13",
        title_en: "ICT providing relay or emergency service access",
        title_de: "IKT, die Relay- oder Notfalldienste bereitstellt",
    },
];

/// Four-way scope status for one EN 301 549 clause. Exactly one of the first
/// three applies per clause (see `derive_annex`); the fourth "out of scope"
/// case is not a per-clause status at all — it is the static
/// `OUT_OF_SCOPE_CHAPTERS` list plus the out-of-standard-findings footnote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseStatus {
    /// At least one automated finding matched this clause.
    ViolationsFound,
    /// This clause is covered by an automated rule, and no finding matched it.
    /// Wording is deliberately "no violations found in automated scope", never
    /// "passed" or "conformant".
    NoViolationsAutomated,
    /// This clause fundamentally requires manual review (or falls into
    /// neither known list — the safe fallback is manual review, never
    /// silently "clean").
    ManualReviewRequired,
}

/// One violated rule contributing to a clause's `ViolationsFound` status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClauseFindingRef {
    pub rule_id: String,
    pub occurrences: usize,
}

/// Per-clause roll-up produced by `derive_annex`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClauseRollup {
    pub clause: En301549Clause,
    pub status: ClauseStatus,
    /// Populated only when `status == ViolationsFound`.
    pub findings: Vec<ClauseFindingRef>,
}

/// Derive a per-clause status roll-up over a page's findings — a pure
/// projection, no I/O, nothing stored in `NormalizedReport`. Priority order
/// per clause:
///   1. `ViolationsFound` — at least one `category == "wcag"` finding whose
///      `wcag_criterion` matches the clause.
///   2. `ManualReviewRequired` — clause is in `manual_review_criteria()`.
///   3. `NoViolationsAutomated` — clause is in `automated_criteria()`.
///   4. Fallback `ManualReviewRequired` for a clause in neither list (e.g. a
///      rule was removed) — never silently reads as "clean".
pub fn derive_annex(findings: &[NormalizedFinding]) -> Vec<ClauseRollup> {
    let automated = crate::wcag::coverage::automated_criteria();
    let manual = crate::wcag::coverage::manual_review_criteria();

    EN301549_WEB_CLAUSES
        .iter()
        .map(|clause| {
            let matching: Vec<&NormalizedFinding> = findings
                .iter()
                .filter(|f| f.category == "wcag" && f.wcag_criterion == clause.wcag)
                .collect();

            if !matching.is_empty() {
                let mut by_rule: std::collections::BTreeMap<String, usize> =
                    std::collections::BTreeMap::new();
                for f in &matching {
                    *by_rule.entry(f.rule_id.clone()).or_insert(0) += f.occurrence_count;
                }
                let findings = by_rule
                    .into_iter()
                    .map(|(rule_id, occurrences)| ClauseFindingRef {
                        rule_id,
                        occurrences,
                    })
                    .collect();
                return ClauseRollup {
                    clause: *clause,
                    status: ClauseStatus::ViolationsFound,
                    findings,
                };
            }

            if manual.iter().any(|(id, _, _)| *id == clause.wcag) {
                return ClauseRollup {
                    clause: *clause,
                    status: ClauseStatus::ManualReviewRequired,
                    findings: Vec::new(),
                };
            }

            if automated.iter().any(|(id, _)| *id == clause.wcag) {
                return ClauseRollup {
                    clause: *clause,
                    status: ClauseStatus::NoViolationsAutomated,
                    findings: Vec::new(),
                };
            }

            // Fallback: a clause in neither manifest never silently reads as
            // clean — treat it as requiring manual review.
            ClauseRollup {
                clause: *clause,
                status: ClauseStatus::ManualReviewRequired,
                findings: Vec::new(),
            }
        })
        .collect()
}

/// One clause's worst status across a batch of pages, plus how many of those
/// pages have a confirmed violation for it. Used for the batch-only,
/// domain-wide roll-up (JSON `summary.en301549_rollup` and the batch PDF
/// annex) — single reports use the full per-page `derive_annex` instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchClauseRollup {
    pub clause: En301549Clause,
    pub status: ClauseStatus,
    pub affected_pages: usize,
}

fn status_rank(status: ClauseStatus) -> u8 {
    match status {
        ClauseStatus::NoViolationsAutomated => 0,
        ClauseStatus::ManualReviewRequired => 1,
        ClauseStatus::ViolationsFound => 2,
    }
}

/// Derive the domain-wide, per-clause roll-up across all pages in a batch:
/// the worst status observed for each clause (`ViolationsFound` >
/// `ManualReviewRequired` > `NoViolationsAutomated`), plus the count of pages
/// where that clause actually has `ViolationsFound`.
pub fn derive_batch_rollup<'a>(
    pages_findings: impl IntoIterator<Item = &'a [NormalizedFinding]>,
) -> Vec<BatchClauseRollup> {
    let per_page_rollups: Vec<Vec<ClauseRollup>> =
        pages_findings.into_iter().map(derive_annex).collect();

    EN301549_WEB_CLAUSES
        .iter()
        .enumerate()
        .map(|(idx, clause)| {
            let mut worst = ClauseStatus::NoViolationsAutomated;
            let mut affected_pages = 0usize;
            for page_rollup in &per_page_rollups {
                let status = page_rollup[idx].status;
                if matches!(status, ClauseStatus::ViolationsFound) {
                    affected_pages += 1;
                }
                if status_rank(status) > status_rank(worst) {
                    worst = status;
                }
            }
            BatchClauseRollup {
                clause: *clause,
                status: worst,
                affected_pages,
            }
        })
        .collect()
}

/// Number of WCAG findings (category `"wcag"`) whose criterion is not one of
/// the 50 clauses in `EN301549_WEB_CLAUSES` — WCAG-2.2-only criteria and AAA
/// findings. These are intentionally excluded from the clause table (EN
/// 301 549 V3.2.1 == WCAG 2.1 A/AA) but must not be silently dropped from the
/// annex, hence this footnote count.
pub fn out_of_standard_finding_count(findings: &[NormalizedFinding]) -> usize {
    findings
        .iter()
        .filter(|f| {
            f.category == "wcag"
                && !EN301549_WEB_CLAUSES
                    .iter()
                    .any(|c| c.wcag == f.wcag_criterion)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::{
        ComplexityKind, ExpectedImpactKind, ReportVisibilityData, ScoreEffect, ScoreImpactData,
    };
    use crate::taxonomy::Severity;

    fn finding(wcag_criterion: &str, rule_id: &str, occurrence_count: usize) -> NormalizedFinding {
        NormalizedFinding {
            category: "wcag".into(),
            rule_id: rule_id.into(),
            wcag_criterion: wcag_criterion.into(),
            axe_id: None,
            wcag_level: "A".into(),
            dimension: "Accessibility".into(),
            subcategory: "Images".into(),
            issue_class: "Missing".into(),
            dimension_kind: crate::taxonomy::Dimension::Accessibility,
            subcategory_kind: crate::taxonomy::Subcategory::ContentAlternatives,
            issue_class_kind: crate::taxonomy::IssueClass::Missing,
            severity: Severity::Medium,
            user_impact: String::new(),
            technical_impact: String::new(),
            score_impact: ScoreImpactData {
                base_penalty: 5.0,
                max_penalty: 20.0,
                scaling: "Logarithmic".into(),
            },
            report_visibility: ReportVisibilityData::default(),
            aggregation_key: rule_id.into(),
            title: "Test finding".into(),
            description: String::new(),
            help_url: None,
            occurrence_count,
            priority_score: 1.0,
            confidence: "very_high".into(),
            false_positive_risk: "very_low".into(),
            verification: "automatically_confirmed".into(),
            complexity: "low".into(),
            complexity_reason: "Test fixture".into(),
            complexity_kind: ComplexityKind::LowScope,
            expected_impact: "Test fixture".into(),
            expected_impact_kind: ExpectedImpactKind::Wcag {
                occurrence_count,
                score_effect: ScoreEffect::Low,
                wcag_level: "A".into(),
            },
            bfsg_relevance: "medium".into(),
            remediation_priority: "normal".into(),
            occurrences: vec![],
        }
    }

    #[test]
    fn table_has_exactly_50_entries_30_a_20_aa() {
        assert_eq!(EN301549_WEB_CLAUSES.len(), 50);
        let a = EN301549_WEB_CLAUSES
            .iter()
            .filter(|c| c.wcag_level == "A")
            .count();
        let aa = EN301549_WEB_CLAUSES
            .iter()
            .filter(|c| c.wcag_level == "AA")
            .count();
        assert_eq!(a, 30, "WCAG 2.1 has exactly 30 Level A criteria");
        assert_eq!(aa, 20, "WCAG 2.1 has exactly 20 Level AA criteria");
    }

    #[test]
    fn en_clause_is_always_9_prefixed_wcag_id() {
        for clause in EN301549_WEB_CLAUSES {
            assert_eq!(clause.en_clause, format!("9.{}", clause.wcag));
        }
    }

    #[test]
    fn no_duplicate_wcag_ids() {
        let mut seen = std::collections::HashSet::new();
        for clause in EN301549_WEB_CLAUSES {
            assert!(
                seen.insert(clause.wcag),
                "duplicate wcag id {}",
                clause.wcag
            );
        }
    }

    /// Partition guard: every one of the 50 clauses must land in exactly one
    /// of the three `ClauseStatus` values for a representative findings set
    /// (some violations, most clauses clean) — no clause is dropped, none
    /// duplicated, and a clause with a matching finding always wins over
    /// automated/manual classification.
    #[test]
    fn partition_covers_all_50_clauses_no_overlap_no_gap() {
        // Give at least one clause an actual violation.
        let findings = vec![finding("1.1.1", "a11y.alt_text.missing", 3)];
        let rollups = derive_annex(&findings);

        assert_eq!(rollups.len(), 50, "every clause must appear exactly once");

        let mut seen = std::collections::HashSet::new();
        for r in &rollups {
            assert!(
                seen.insert(r.clause.wcag),
                "clause {} appeared more than once",
                r.clause.wcag
            );
        }
        // 1:1 with the canonical table (no gap).
        for clause in EN301549_WEB_CLAUSES {
            assert!(
                rollups.iter().any(|r| r.clause.wcag == clause.wcag),
                "clause {} missing from roll-up",
                clause.wcag
            );
        }

        let violated = rollups
            .iter()
            .find(|r| r.clause.wcag == "1.1.1")
            .expect("1.1.1 present");
        assert!(matches!(violated.status, ClauseStatus::ViolationsFound));
        assert_eq!(violated.findings.len(), 1);
        assert_eq!(violated.findings[0].occurrences, 3);
    }

    #[test]
    fn violations_found_outranks_automated_and_manual_classification() {
        // Pick a clause this build currently considers "automated" and give
        // it a finding anyway — ViolationsFound must win.
        let automated = crate::wcag::coverage::automated_criteria();
        let Some((wcag_id, _)) = automated
            .iter()
            .find(|(id, _)| EN301549_WEB_CLAUSES.iter().any(|c| c.wcag == *id))
        else {
            return; // nothing automated maps onto the 2.1 table right now
        };
        let findings = vec![finding(wcag_id, "some.rule", 1)];
        let rollups = derive_annex(&findings);
        let rollup = rollups
            .iter()
            .find(|r| r.clause.wcag == *wcag_id)
            .expect("clause present");
        assert!(matches!(rollup.status, ClauseStatus::ViolationsFound));
    }

    #[test]
    fn empty_findings_split_between_automated_and_manual_review() {
        let rollups = derive_annex(&[]);
        let automated = crate::wcag::coverage::automated_criteria();
        let manual = crate::wcag::coverage::manual_review_criteria();

        for r in &rollups {
            match r.status {
                ClauseStatus::ViolationsFound => panic!("no findings were given"),
                ClauseStatus::NoViolationsAutomated => {
                    assert!(automated.iter().any(|(id, _)| *id == r.clause.wcag));
                }
                ClauseStatus::ManualReviewRequired => {
                    // Either a genuine manual-review criterion, or the
                    // documented fallback for a clause in neither manifest.
                    let in_manual = manual.iter().any(|(id, _, _)| *id == r.clause.wcag);
                    let in_automated = automated.iter().any(|(id, _)| *id == r.clause.wcag);
                    assert!(in_manual || !in_automated);
                }
            }
        }
    }

    #[test]
    fn out_of_standard_findings_are_counted_not_dropped() {
        // 2.5.8 ("Target Size (Minimum)") is a WCAG 2.2-only criterion, not
        // part of the 50-clause WCAG 2.1 table.
        let findings = vec![finding("2.5.8", "a11y.target_size.minimum", 2)];
        assert_eq!(out_of_standard_finding_count(&findings), 1);
        // It must not silently appear as a violation against any 2.1 clause.
        let rollups = derive_annex(&findings);
        assert!(!rollups
            .iter()
            .any(|r| matches!(r.status, ClauseStatus::ViolationsFound)));
    }

    #[test]
    fn batch_rollup_takes_worst_status_and_counts_affected_pages() {
        let clean_page: Vec<NormalizedFinding> = vec![];
        let violating_page = vec![finding("1.1.1", "a11y.alt_text.missing", 2)];
        let pages = [clean_page.as_slice(), violating_page.as_slice()];

        let rollup = derive_batch_rollup(pages);
        assert_eq!(rollup.len(), 50);

        let clause_111 = rollup
            .iter()
            .find(|r| r.clause.wcag == "1.1.1")
            .expect("1.1.1 present");
        assert!(matches!(clause_111.status, ClauseStatus::ViolationsFound));
        assert_eq!(
            clause_111.affected_pages, 1,
            "only one of the two pages actually violated 1.1.1"
        );
    }

    #[test]
    fn out_of_scope_chapters_exclude_web_chapter_9() {
        assert!(!OUT_OF_SCOPE_CHAPTERS.iter().any(|c| c.chapter == "9"));
        // Chapter 13 (relay/emergency services) exists in V3.2.1 and must be
        // named, not silently dropped after chapter 12.
        assert!(OUT_OF_SCOPE_CHAPTERS.iter().any(|c| c.chapter == "13"));
    }
}
