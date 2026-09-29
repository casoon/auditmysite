//! "BIK für Alle" editorial accessibility guide — opt-in PDF appendix section.
//!
//! Only rendered when `--annex bik` is passed (see `ReportConfig.annex`); the
//! underlying JSON `bik_guide` block is always emitted regardless. Sibling of
//! `en301549.rs`; both sit in the appendix.
//!
//! Pure re-projection of already-computed findings — no new detection, see
//! the module doc on `wcag::bik_guide`.

use crate::output::localized::is_english;
use renderreport::components::advanced::SectionHeaderSplit;
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn, TagCloud};

use crate::audit::normalized::{AccessibilityAssessment, InteractiveFinding, NormalizedFinding};
use crate::design_quality::DesignQualityFinding;
use crate::i18n::I18n;
use crate::screen_reader::SrAuditIssue;
use crate::seo::technical::TechnicalIssue;
use crate::wcag::bik_guide::{derive_bik_chapters, BikChapterStatus, BIK_GUIDE_SOURCE};

/// Render the BIK-für-Alle chapter mapping. Structure: intro -> a findings
/// table for chapters with at least one mapped finding -> a "checked, clean"
/// tag cloud for chapters with an active detector and nothing found -> a
/// "not automatically checked" note for chapters with no detector at all
/// (currently PDFs only). Every chapter always renders in one of these three
/// states — none is silently omitted.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_bik_guide_annex(
    mut builder: renderreport::engine::ReportBuilder,
    findings: &[NormalizedFinding],
    interactive_findings: &[InteractiveFinding],
    screen_reader_issues: &[SrAuditIssue],
    design_quality_findings: &[DesignQualityFinding],
    seo_technical_issues: &[TechnicalIssue],
    easy_language_detected: bool,
    accessibility_assessments: &[AccessibilityAssessment],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);

    let (title, intro) = if en {
        (
            "BIK für Alle guide mapping",
            format!(
                "Groups this audit's automated findings under the chapter structure of the \"BIK für Alle\" editorial accessibility guide ({BIK_GUIDE_SOURCE}) — the same chapters editorial teams already use: Images & alt text, Link text, Structure, Easy language, PDF documents, Videos. No new checks; this re-groups findings already reported elsewhere in this document."
            ),
        )
    } else {
        (
            "BIK-für-Alle-Leitfadenzuordnung",
            format!(
                "Ordnet die automatisch erkannten Befunde dieses Audits der Kapitelstruktur des redaktionellen Leitfadens \"BIK für Alle\" zu ({BIK_GUIDE_SOURCE}) — denselben Kapiteln, die Redaktionsteams bereits kennen: Bilder & Alt-Text, Linktext, Struktur, Leichte Sprache, PDF-Dokumente, Videos. Keine neuen Prüfungen; dies gruppiert an anderer Stelle in diesem Report bereits ausgewiesene Befunde um."
            ),
        )
    };
    builder = builder.add_component(SectionHeaderSplit::new(title, &intro).with_level(2));

    let rollups = derive_bik_chapters(
        findings,
        interactive_findings,
        screen_reader_issues,
        design_quality_findings,
        seo_technical_issues,
        easy_language_detected,
        accessibility_assessments,
    );

    let with_findings: Vec<_> = rollups
        .iter()
        .filter(|r| matches!(r.status, BikChapterStatus::FindingsPresent))
        .collect();
    if !with_findings.is_empty() {
        let table_title = if en {
            format!("Findings by chapter ({})", with_findings.len())
        } else {
            format!("Befunde nach Kapitel ({})", with_findings.len())
        };
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Chapter" } else { "Kapitel" }).with_width("28%"),
            TableColumn::new(if en { "Findings" } else { "Befunde" }).with_width("72%"),
        ])
        .with_title(table_title);
        for r in &with_findings {
            let chapter_title = if en {
                r.chapter.title_en
            } else {
                r.chapter.title_de
            };
            let findings_text = r
                .findings
                .iter()
                .map(|f| format!("{} ({}\u{00d7})", f.label, f.occurrences))
                .collect::<Vec<_>>()
                .join("; ");
            table = table.add_row(vec![chapter_title.to_string(), findings_text]);
        }
        builder = builder.add_component(table);
    }

    let clean: Vec<_> = rollups
        .iter()
        .filter(|r| matches!(r.status, BikChapterStatus::NoFindingsDetected))
        .collect();
    if !clean.is_empty() {
        let cloud_title = if en {
            format!("Checked, no findings ({})", clean.len())
        } else {
            format!("Geprüft, keine Befunde ({})", clean.len())
        };
        let mut cloud = TagCloud::new().with_title(cloud_title).with_gap("5pt");
        for r in &clean {
            let chapter_title = if en {
                r.chapter.title_en
            } else {
                r.chapter.title_de
            };
            cloud = cloud.add(chapter_title.to_string(), "good");
        }
        builder = builder.add_component(cloud);
    }

    let not_checked: Vec<_> = rollups
        .iter()
        .filter(|r| matches!(r.status, BikChapterStatus::NotChecked))
        .collect();
    if !not_checked.is_empty() {
        let note_title = if en {
            "Not automatically checked"
        } else {
            "Nicht automatisch geprüft"
        };
        let chapter_list = not_checked
            .iter()
            .map(|r| {
                if en {
                    r.chapter.title_en
                } else {
                    r.chapter.title_de
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let note = if en {
            format!("{chapter_list}: this tool does not currently check this chapter's content automatically. Not represented in the findings table above.")
        } else {
            format!("{chapter_list}: dieses Kapitel wird von diesem Tool aktuell nicht automatisch geprüft. Es ist in der Befunde-Tabelle oben nicht enthalten.")
        };
        builder = builder.add_component(
            Label::new(format!("{note_title}: {note}"))
                .with_size("10.5pt")
                .with_color("#475569"),
        );
    }

    builder
}
