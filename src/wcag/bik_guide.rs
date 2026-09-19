//! "BIK für Alle" editorial accessibility guide — chapter mapping.
//!
//! Promotes *no new detection*. This is a pure re-projection of findings this
//! tool already computes elsewhere onto the chapter structure editors already
//! know from the Bundesfachstelle für Barrierefreiheit's "Barrierefreie
//! Redaktion" guide (reviewed 2026-09-03): Bilder/Alt-Text, Linktext,
//! Struktur, Leichte Sprache, PDFs, Videos. Same pattern as the EN 301 549
//! annex (`wcag::en301549`): a pure projection function, no I/O, nothing new
//! stored on `NormalizedReport`.
//!
//! ## Why this takes more than `&[NormalizedFinding]`
//!
//! Unlike EN 301 549 (which maps 1:1 onto WCAG success criteria already
//! normalized into `NormalizedFinding`), three of this guide's chapters
//! source from data that is deliberately *not* folded into
//! `NormalizedFinding`:
//! - Alt-text quality (`design.alt_*`) is a score-neutral `design_quality`
//!   finding, kept out of WCAG violation counting by construction (#528).
//! - `og_image_missing_alt`/`twitter_image_missing_alt` are SEO
//!   `TechnicalIssue`s (`seo::technical::collect_technical_issues`), never
//!   normalized into `findings[]` either.
//! - Easy-language ("Leichte Sprache") presence is a *positive* pattern
//!   signal (`patterns::PatternAnalysis::recognized`), not a violation at
//!   all.
//!
//! `derive_bik_chapters` therefore takes these three as separate slices/flag
//! rather than only `&NormalizedReport`. All three are only available from a
//! live single-report audit's raw module data (`AuditContext`) — a cached or
//! batch-processed page has already discarded that raw data by the time this
//! runs (see `output::json::detail::build_detail_cached`/`build_batch_detail`),
//! so callers without it pass empty slices / `false`. The other three
//! chapters (Images' WCAG 1.1.1 share, Link Text, Structure, Videos) are
//! fully `NormalizedReport`-derivable and work identically in every report
//! path.
//!
//! ## Why Videos also takes `&[AccessibilityAssessment]`
//!
//! `findings[]` (`NormalizedFinding`) is built exclusively from
//! `wcag_results.violations` (`audit::normalized::normalize`) — it never
//! includes `Outcome::Review`/`NotTestable` findings. The video checks
//! (1.2.1/1.2.2/1.2.3/1.2.8) resolve to exactly those kinds far more often
//! than to a confirmed violation (e.g. an unresolving caption `<track>` is a
//! manual-review notice, not a hard violation), so relying on `findings[]`
//! alone made the Videos chapter report `NoFindingsDetected` even when a
//! real, PDF-visible caption notice existed (#20). `accessibility_assessments`
//! (`NormalizedReport`'s own field, populated by `normalize_assessments`) is
//! the pre-existing carrier for exactly this warning/manual-review data and
//! is available in every report path that already populates `findings[]`.

use crate::audit::normalized::{
    AccessibilityAssessment, InteractiveFinding, InteractiveFindingKind, NormalizedFinding,
};
use crate::design_quality::DesignQualityFinding;
use crate::screen_reader::SrAuditIssue;
use crate::seo::technical::TechnicalIssue;

/// Source guide this mapping targets. Not a legal/statutory reference (unlike
/// EN 301 549) — a widely used editorial checklist.
pub const BIK_GUIDE_SOURCE: &str =
    "BIK für Alle \u{2013} Barrierefreie Redaktion (Bundesfachstelle für Barrierefreiheit)";

/// One BIK-für-Alle guide chapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BikChapterId {
    Images,
    LinkText,
    Structure,
    EasyLanguage,
    Pdf,
    Videos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BikChapter {
    pub id: BikChapterId,
    pub title_en: &'static str,
    pub title_de: &'static str,
}

/// All six chapters, in the guide's own order.
pub const BIK_CHAPTERS: &[BikChapter] = &[
    BikChapter {
        id: BikChapterId::Images,
        title_en: "Images & alt text",
        title_de: "Bilder & Alt-Text",
    },
    BikChapter {
        id: BikChapterId::LinkText,
        title_en: "Link text",
        title_de: "Linktext",
    },
    BikChapter {
        id: BikChapterId::Structure,
        title_en: "Structure",
        title_de: "Struktur",
    },
    BikChapter {
        id: BikChapterId::EasyLanguage,
        title_en: "Easy language",
        title_de: "Leichte Sprache",
    },
    BikChapter {
        id: BikChapterId::Pdf,
        title_en: "PDF documents",
        title_de: "PDF-Dokumente",
    },
    BikChapter {
        id: BikChapterId::Videos,
        title_en: "Videos",
        title_de: "Videos",
    },
];

/// Status of one chapter's automated coverage for this report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BikChapterStatus {
    /// At least one finding was mapped into this chapter.
    FindingsPresent,
    /// This chapter has an active detector in this report, and it found
    /// nothing to report — a genuine "checked, clean" result (or, for
    /// `EasyLanguage`, "no easy-language offering detected").
    NoFindingsDetected,
    /// No detector currently feeds this chapter at all (e.g. `Pdf` — this
    /// tool does not yet check linked/embedded PDF documents). Never
    /// conflated with `NoFindingsDetected`, which implies an actual check ran.
    NotChecked,
}

/// One finding rolled up into a chapter. `label` is the finding's `rule_id`
/// when one exists (WCAG/design-quality/SEO-technical rule ids), or a
/// synthetic identifier for sources that don't carry one (screen-reader
/// structural issues, the easy-language positive signal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BikFindingRef {
    pub label: String,
    pub occurrences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BikChapterRollup {
    pub chapter: BikChapter,
    pub status: BikChapterStatus,
    pub findings: Vec<BikFindingRef>,
}

/// WCAG 2.1 criteria mapped onto the "Images & alt text" chapter.
const IMAGES_WCAG_CRITERIA: &[&str] = &["1.1.1"];
/// WCAG 2.1 criteria mapped onto the "Videos" chapter (time-based media).
const VIDEOS_WCAG_CRITERIA: &[&str] = &["1.2.1", "1.2.2", "1.2.3", "1.2.4", "1.2.5", "1.2.8"];
/// Screen-reader structural issue criteria mapped onto the "Structure"
/// chapter — heading order/level and landmark presence, mirroring
/// `screen_reader::analyzer::detect_skipped_heading_levels`/
/// `detect_heading_order_issues`/`detect_missing_required_landmarks`.
const STRUCTURE_SR_CRITERIA: &[&str] = &["1.3.1", "1.3.6"];
/// SEO `TechnicalIssue::issue_type` values mapped onto "Images & alt text".
const IMAGES_SEO_ISSUE_TYPES: &[&str] = &["og_image_missing_alt", "twitter_image_missing_alt"];

fn rollup_from_map(
    chapter: BikChapter,
    map: std::collections::BTreeMap<String, usize>,
) -> BikChapterRollup {
    let findings: Vec<BikFindingRef> = map
        .into_iter()
        .map(|(label, occurrences)| BikFindingRef { label, occurrences })
        .collect();
    let status = if findings.is_empty() {
        BikChapterStatus::NoFindingsDetected
    } else {
        BikChapterStatus::FindingsPresent
    };
    BikChapterRollup {
        chapter,
        status,
        findings,
    }
}

fn add(map: &mut std::collections::BTreeMap<String, usize>, label: &str, occurrences: usize) {
    *map.entry(label.to_string()).or_insert(0) += occurrences;
}

/// Derive the six-chapter BIK-für-Alle rollup for one page — a pure
/// projection over already-computed findings, no I/O, nothing stored on
/// `NormalizedReport`. See the module doc for why this takes more than a
/// single `NormalizedFinding` slice, and why `design_quality_findings`/
/// `seo_technical_issues`/`easy_language_detected` may legitimately be
/// empty/`false` (data not retained past a cached or batch run).
pub fn derive_bik_chapters(
    findings: &[NormalizedFinding],
    interactive_findings: &[InteractiveFinding],
    screen_reader_issues: &[SrAuditIssue],
    design_quality_findings: &[DesignQualityFinding],
    seo_technical_issues: &[TechnicalIssue],
    easy_language_detected: bool,
    accessibility_assessments: &[AccessibilityAssessment],
) -> Vec<BikChapterRollup> {
    BIK_CHAPTERS
        .iter()
        .map(|chapter| match chapter.id {
            BikChapterId::Images => {
                let mut map = std::collections::BTreeMap::new();
                for f in findings.iter().filter(|f| {
                    f.category == "wcag"
                        && IMAGES_WCAG_CRITERIA.contains(&f.wcag_criterion.as_str())
                }) {
                    add(&mut map, &f.rule_id, f.occurrence_count);
                }
                for f in design_quality_findings
                    .iter()
                    .filter(|f| f.rule_id.starts_with("design.alt_"))
                {
                    add(&mut map, &f.rule_id, 1);
                }
                for issue in seo_technical_issues
                    .iter()
                    .filter(|i| IMAGES_SEO_ISSUE_TYPES.contains(&i.issue_type.as_str()))
                {
                    add(&mut map, &issue.issue_type, 1);
                }
                rollup_from_map(*chapter, map)
            }
            BikChapterId::LinkText => {
                let mut map = std::collections::BTreeMap::new();
                for f in interactive_findings
                    .iter()
                    .filter(|f| f.kind == InteractiveFindingKind::LinkTextGeneric)
                {
                    add(
                        &mut map,
                        "a11y_journey.link_text_generic",
                        f.values.count.unwrap_or(1) as usize,
                    );
                }
                rollup_from_map(*chapter, map)
            }
            BikChapterId::Structure => {
                let mut map = std::collections::BTreeMap::new();
                // Occurrences sum `affected_node_ids.len()` across every
                // `1.3.6`-tagged issue on the page, not just one — a page can
                // have several independent `detect_announcement_deserts`
                // "long section without a landmark" findings, each
                // contributing its own (already sanitized, see
                // `analyzer::analyze_reading_sequence`'s `is_real_node_id`
                // filter) node count. Verified against a live casoon.de run
                // (2026-09-05, plan/22): a page-level total of 200 was the
                // exact sum of 14 separate desert findings' filtered node
                // counts, not a double-count across viewports or an
                // overly-broad selector — `build_sr_audit_report` runs once
                // per report on a single AXTree.
                for issue in screen_reader_issues.iter().filter(|i| {
                    i.wcag_criterion
                        .as_deref()
                        .is_some_and(|c| STRUCTURE_SR_CRITERIA.contains(&c))
                }) {
                    add(
                        &mut map,
                        issue.wcag_criterion.as_deref().unwrap_or("structure"),
                        issue.affected_node_ids.len().max(1),
                    );
                }
                rollup_from_map(*chapter, map)
            }
            BikChapterId::EasyLanguage => {
                if easy_language_detected {
                    let mut map = std::collections::BTreeMap::new();
                    add(&mut map, "patterns.easy_language.detected", 1);
                    BikChapterRollup {
                        chapter: *chapter,
                        status: BikChapterStatus::FindingsPresent,
                        findings: map
                            .into_iter()
                            .map(|(label, occurrences)| BikFindingRef { label, occurrences })
                            .collect(),
                    }
                } else {
                    BikChapterRollup {
                        chapter: *chapter,
                        status: BikChapterStatus::NoFindingsDetected,
                        findings: Vec::new(),
                    }
                }
            }
            BikChapterId::Pdf => BikChapterRollup {
                chapter: *chapter,
                status: BikChapterStatus::NotChecked,
                findings: Vec::new(),
            },
            BikChapterId::Videos => {
                let mut map = std::collections::BTreeMap::new();
                for f in findings.iter().filter(|f| {
                    f.category == "wcag"
                        && VIDEOS_WCAG_CRITERIA.contains(&f.wcag_criterion.as_str())
                }) {
                    add(&mut map, &f.rule_id, f.occurrence_count);
                }
                // Video checks (1.2.1/1.2.2/1.2.3/1.2.8) resolve to a
                // manual-review notice or heuristic warning far more often
                // than to a confirmed `Violation` — those never enter
                // `findings[]` (which only sources from
                // `wcag_results.violations`, see `audit::normalized::normalize`),
                // so without this, a real, PDF-visible caption/transcript
                // notice silently rendered as `NoFindingsDetected` here (#20).
                for a in accessibility_assessments.iter().filter(|a| {
                    (a.kind == "warning" || a.kind == "manual_review")
                        && VIDEOS_WCAG_CRITERIA.contains(&a.wcag_criterion.as_str())
                }) {
                    add(&mut map, &a.rule_id, 1);
                }
                for f in interactive_findings.iter().filter(|f| {
                    matches!(
                        f.kind,
                        InteractiveFindingKind::MediaControlsMissingName
                            | InteractiveFindingKind::MediaControlsNotReachable
                    )
                }) {
                    let label = match f.kind {
                        InteractiveFindingKind::MediaControlsMissingName => {
                            "a11y_journey.media_controls_missing_name"
                        }
                        InteractiveFindingKind::MediaControlsNotReachable => {
                            "a11y_journey.media_controls_not_reachable"
                        }
                        _ => unreachable!("filtered to the two media-control kinds above"),
                    };
                    add(&mut map, label, f.values.count.unwrap_or(1) as usize);
                }
                rollup_from_map(*chapter, map)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::{
        ComplexityKind, ExpectedImpactKind, InteractiveFindingValues, ReportVisibilityData,
        ScoreEffect, ScoreImpactData,
    };
    use crate::taxonomy::Severity;

    fn wcag_finding(
        wcag_criterion: &str,
        rule_id: &str,
        occurrence_count: usize,
    ) -> NormalizedFinding {
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

    fn interactive_finding(kind: InteractiveFindingKind, count: u32) -> InteractiveFinding {
        InteractiveFinding::new(
            "Test",
            kind,
            None,
            Severity::Medium,
            "test".into(),
            None,
            None,
            InteractiveFindingValues {
                count: Some(count),
                ..Default::default()
            },
        )
    }

    fn design_quality_finding(rule_id: &str) -> DesignQualityFinding {
        DesignQualityFinding {
            rule_id: rule_id.into(),
            level: crate::design_quality::FindingLevel::Advisory,
            confidence: crate::design_quality::Confidence::High,
            measurement_type: "heuristic".into(),
            selector: "img".into(),
            evidence: "test evidence".into(),
            message: "test message".into(),
        }
    }

    fn seo_issue(issue_type: &str) -> TechnicalIssue {
        TechnicalIssue {
            issue_type: issue_type.into(),
            message: "test message".into(),
            severity: Severity::Low,
        }
    }

    #[test]
    fn produces_exactly_six_chapters_in_guide_order() {
        let rollups = derive_bik_chapters(&[], &[], &[], &[], &[], false, &[]);
        assert_eq!(rollups.len(), 6);
        assert_eq!(rollups[0].chapter.id, BikChapterId::Images);
        assert_eq!(rollups[1].chapter.id, BikChapterId::LinkText);
        assert_eq!(rollups[2].chapter.id, BikChapterId::Structure);
        assert_eq!(rollups[3].chapter.id, BikChapterId::EasyLanguage);
        assert_eq!(rollups[4].chapter.id, BikChapterId::Pdf);
        assert_eq!(rollups[5].chapter.id, BikChapterId::Videos);
    }

    #[test]
    fn empty_input_is_clean_not_checked_or_absent_per_chapter_semantics() {
        let rollups = derive_bik_chapters(&[], &[], &[], &[], &[], false, &[]);
        for r in &rollups {
            match r.chapter.id {
                BikChapterId::Pdf => {
                    assert_eq!(r.status, BikChapterStatus::NotChecked);
                }
                _ => {
                    assert_eq!(r.status, BikChapterStatus::NoFindingsDetected);
                    assert!(r.findings.is_empty());
                }
            }
        }
    }

    /// Combines findings across four of the six chapters (Images, Link Text,
    /// Structure, Videos) plus the Easy-Language positive signal in one pass —
    /// verifies correct grouping (right chapter, right count) and that an
    /// untouched chapter (Pdf) still renders its documented clean state
    /// rather than silently vanishing.
    #[test]
    fn groups_findings_into_correct_chapters_across_multiple_sources() {
        let findings = vec![
            wcag_finding("1.1.1", "a11y.alt_text.missing", 3),
            wcag_finding("1.2.2", "a11y.media_caption.missing", 1),
            // Not mapped to any chapter — must not leak into e.g. Structure.
            wcag_finding("2.1.1", "a11y.keyboard.trap", 2),
        ];
        let interactive = vec![
            interactive_finding(InteractiveFindingKind::LinkTextGeneric, 5),
            interactive_finding(InteractiveFindingKind::MediaControlsMissingName, 2),
        ];
        let design_quality = vec![
            design_quality_finding("design.alt_filename"),
            design_quality_finding("design.emoji_usage"), // not alt-related, must not count
        ];
        let seo_issues = vec![seo_issue("og_image_missing_alt")];

        let sr_issues = vec![SrAuditIssue {
            wcag_criterion: Some("1.3.1".into()),
            severity: "medium".into(),
            affected_node_ids: vec!["n1".into()],
            message: "Heading level is skipped.".into(),
        }];

        let rollups = derive_bik_chapters(
            &findings,
            &interactive,
            &sr_issues,
            &design_quality,
            &seo_issues,
            true,
            &[],
        );

        let by_id = |id: BikChapterId| rollups.iter().find(|r| r.chapter.id == id).unwrap();

        let images = by_id(BikChapterId::Images);
        assert_eq!(images.status, BikChapterStatus::FindingsPresent);
        assert_eq!(
            images.findings.len(),
            3,
            "1.1.1 wcag + alt_filename + og_image, but not emoji_usage"
        );
        assert!(images
            .findings
            .iter()
            .any(|f| f.label == "a11y.alt_text.missing" && f.occurrences == 3));
        assert!(images
            .findings
            .iter()
            .any(|f| f.label == "design.alt_filename"));
        assert!(images
            .findings
            .iter()
            .any(|f| f.label == "og_image_missing_alt"));

        let link_text = by_id(BikChapterId::LinkText);
        assert_eq!(link_text.status, BikChapterStatus::FindingsPresent);
        assert_eq!(link_text.findings.len(), 1);
        assert_eq!(link_text.findings[0].occurrences, 5);

        let structure = by_id(BikChapterId::Structure);
        assert_eq!(structure.status, BikChapterStatus::FindingsPresent);
        assert_eq!(structure.findings.len(), 1);
        assert_eq!(structure.findings[0].label, "1.3.1");

        let easy_language = by_id(BikChapterId::EasyLanguage);
        assert_eq!(easy_language.status, BikChapterStatus::FindingsPresent);
        assert_eq!(
            easy_language.findings[0].label,
            "patterns.easy_language.detected"
        );

        let pdf = by_id(BikChapterId::Pdf);
        assert_eq!(pdf.status, BikChapterStatus::NotChecked);
        assert!(pdf.findings.is_empty());

        let videos = by_id(BikChapterId::Videos);
        assert_eq!(videos.status, BikChapterStatus::FindingsPresent);
        assert_eq!(
            videos.findings.len(),
            2,
            "1.2.2 wcag + MediaControlsMissingName"
        );
        assert!(videos
            .findings
            .iter()
            .any(|f| f.label == "a11y.media_caption.missing" && f.occurrences == 1));
        assert!(videos
            .findings
            .iter()
            .any(|f| f.label == "a11y_journey.media_controls_missing_name" && f.occurrences == 2));
    }

    /// Regression test for #20's "Wichtiger Nebeneffekt": a video finding
    /// that only ever resolves to a manual-review notice (never a confirmed
    /// `Violation`) — like the real 1.2.2 caption check — must still make the
    /// Videos chapter `FindingsPresent`, even though `findings[]`
    /// (`NormalizedFinding`, sourced only from `wcag_results.violations`)
    /// contains nothing for it at all.
    #[test]
    fn videos_chapter_picks_up_manual_review_only_finding_absent_from_normalized_findings() {
        let assessments = vec![crate::audit::normalized::AccessibilityAssessment {
            kind: "manual_review".to_string(),
            rule_id: "video-caption".to_string(),
            wcag_criterion: "1.2.2".to_string(),
            severity: Severity::High,
            message: "Video detected without a resolving caption track.".to_string(),
            fix_suggestion: None,
            selector: None,
            viewport: None,
            evidence: vec![],
        }];

        // No NormalizedFinding at all — the bug this guards against.
        let rollups = derive_bik_chapters(&[], &[], &[], &[], &[], false, &assessments);

        let videos = rollups
            .iter()
            .find(|r| r.chapter.id == BikChapterId::Videos)
            .unwrap();
        assert_eq!(videos.status, BikChapterStatus::FindingsPresent);
        assert!(videos.findings.iter().any(|f| f.label == "video-caption"));
    }

    /// A `warning`/`manual_review` assessment for a criterion outside the
    /// Videos chapter (e.g. 1.4.3 contrast) must not leak in.
    #[test]
    fn videos_chapter_ignores_assessments_for_unrelated_criteria() {
        let assessments = vec![crate::audit::normalized::AccessibilityAssessment {
            kind: "warning".to_string(),
            rule_id: "color-contrast".to_string(),
            wcag_criterion: "1.4.3".to_string(),
            severity: Severity::High,
            message: "Verify contrast against the rendered image/gradient background.".to_string(),
            fix_suggestion: None,
            selector: None,
            viewport: None,
            evidence: vec![],
        }];

        let rollups = derive_bik_chapters(&[], &[], &[], &[], &[], false, &assessments);

        let videos = rollups
            .iter()
            .find(|r| r.chapter.id == BikChapterId::Videos)
            .unwrap();
        assert_eq!(videos.status, BikChapterStatus::NoFindingsDetected);
        assert!(videos.findings.is_empty());
    }

    #[test]
    fn easy_language_absent_reads_as_no_findings_detected_not_finding() {
        let rollups = derive_bik_chapters(&[], &[], &[], &[], &[], false, &[]);
        let easy_language = rollups
            .iter()
            .find(|r| r.chapter.id == BikChapterId::EasyLanguage)
            .unwrap();
        assert_eq!(easy_language.status, BikChapterStatus::NoFindingsDetected);
        assert!(easy_language.findings.is_empty());
    }
}
