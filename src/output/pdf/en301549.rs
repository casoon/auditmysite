//! EN 301 549 (chapter 9, "Web") clause annex — opt-in PDF appendix section.
//!
//! Only rendered when `--annex en301549` is passed (see `ReportConfig.annex`);
//! the underlying JSON `en301549_annex` block is always emitted regardless.
//! Sibling of `wcag_coverage.rs`; both sit in the appendix, directly after
//! each other.
//!
//! This annex is technical evidence only — never a Barrierefreiheitserklärung.
//! **The disclaimer wording below needs a lawyer's review before
//! customer-facing use** (see `wcag::en301549` module doc and
//! `plans/bfsg-en301549-mapping.md`, "Risks").

use renderreport::components::advanced::{List, SectionHeaderSplit};
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn, TagCloud};
use renderreport::prelude::*;

use crate::audit::normalized::NormalizedFinding;
use crate::i18n::I18n;
use crate::wcag::en301549::{
    derive_annex, out_of_standard_finding_count, ClauseStatus, EN301549_DISCLAIMER_DE,
    EN301549_DISCLAIMER_EN, OUT_OF_SCOPE_CHAPTERS,
};

/// Render the EN 301 549 clause annex. Structure: intro -> disclaimer
/// callout -> violations table (or a clean confirmation line) -> "no
/// violations in automated scope" tag cloud -> "manual review required" tag
/// cloud -> out-of-scope-chapters note. The clean-site case never implies
/// full conformity — the latter three blocks always render regardless of
/// whether violations were found.
pub(super) fn render_en301549_annex(
    mut builder: renderreport::engine::ReportBuilder,
    findings: &[NormalizedFinding],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";

    let (title, intro) = if en {
        (
            "EN 301 549 clause mapping",
            "Maps this audit's automated WCAG 2.1 A/AA findings onto EN 301 549, chapter 9 (Web) clause numbers, with a per-clause status: violations found, no violations found in the automated scope, or manual review required.".to_string(),
        )
    } else {
        (
            "EN-301-549-Klauselzuordnung",
            "Ordnet die automatisch erkannten WCAG-2.1-A/AA-Befunde dieses Audits den Klauselnummern von EN 301 549, Kapitel 9 (Web) zu, mit einem Status je Klausel: Verstöße gefunden, keine Verstöße im automatisch geprüften Umfang, oder manuelle Prüfung erforderlich.".to_string(),
        )
    };
    builder = builder.add_component(SectionHeaderSplit::new(title, &intro).with_level(2));

    let disclaimer_title = if en {
        "Not an accessibility statement"
    } else {
        "Keine Barrierefreiheitserklärung"
    };
    let disclaimer = if en {
        EN301549_DISCLAIMER_EN
    } else {
        EN301549_DISCLAIMER_DE
    };
    builder = builder.add_component(Callout::warning(disclaimer).with_title(disclaimer_title));

    let rollups = derive_annex(findings);
    let violations: Vec<_> = rollups
        .iter()
        .filter(|r| matches!(r.status, ClauseStatus::ViolationsFound))
        .collect();

    if violations.is_empty() {
        builder = builder.add_component(
            Label::new(i18n.t("pdf-section-clean"))
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        );
    } else {
        let table_title = if en {
            format!("Violations found ({})", violations.len())
        } else {
            format!("Verstöße gefunden ({})", violations.len())
        };
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Clause" } else { "Klausel" }).with_width("14%"),
            TableColumn::new(if en { "Title" } else { "Titel" }).with_width("34%"),
            TableColumn::new(if en { "Level" } else { "Stufe" }).with_width("10%"),
            TableColumn::new(if en { "Findings" } else { "Befunde" }).with_width("42%"),
        ])
        .with_title(table_title);
        for r in &violations {
            let clause_title = if en {
                r.clause.title_en
            } else {
                r.clause.title_de
            };
            let findings_text = r
                .findings
                .iter()
                .map(|f| format!("{} ({}\u{00d7})", f.rule_id, f.occurrences))
                .collect::<Vec<_>>()
                .join("; ");
            table = table.add_row(vec![
                r.clause.en_clause.to_string(),
                clause_title.to_string(),
                r.clause.wcag_level.to_string(),
                findings_text,
            ]);
        }
        builder = builder.add_component(table);
    }

    let no_violations: Vec<_> = rollups
        .iter()
        .filter(|r| matches!(r.status, ClauseStatus::NoViolationsAutomated))
        .collect();
    if !no_violations.is_empty() {
        let cloud_title = if en {
            format!("No violations in automated scope ({})", no_violations.len())
        } else {
            format!(
                "Keine Verstöße im automatisch geprüften Umfang ({})",
                no_violations.len()
            )
        };
        let mut cloud = TagCloud::new().with_title(cloud_title).with_gap("5pt");
        for r in &no_violations {
            cloud = cloud.add(
                format!("{} ({})", r.clause.en_clause, r.clause.wcag),
                "good",
            );
        }
        builder = builder.add_component(cloud);
    }

    let manual: Vec<_> = rollups
        .iter()
        .filter(|r| matches!(r.status, ClauseStatus::ManualReviewRequired))
        .collect();
    if !manual.is_empty() {
        let cloud_title = if en {
            format!("Manual review required ({})", manual.len())
        } else {
            format!("Manuelle Prüfung erforderlich ({})", manual.len())
        };
        let mut cloud = TagCloud::new().with_title(cloud_title).with_gap("5pt");
        for r in &manual {
            let clause_title = if en {
                r.clause.title_en
            } else {
                r.clause.title_de
            };
            cloud = cloud.add(
                format!(
                    "{} ({}) \u{2013} {}",
                    r.clause.en_clause, r.clause.wcag, clause_title
                ),
                "info",
            );
        }
        builder = builder.add_component(cloud);
    }

    let out_of_scope_title = if en {
        "Outside this audit's scope"
    } else {
        "Außerhalb dieses Prüfumfangs"
    };
    let mut out_of_scope_list = List::new().with_title(out_of_scope_title);
    for chapter in OUT_OF_SCOPE_CHAPTERS {
        let chapter_title = if en {
            chapter.title_en
        } else {
            chapter.title_de
        };
        out_of_scope_list = out_of_scope_list.add_item(if en {
            format!(
                "Chapter {} \u{2013} {} (not assessed)",
                chapter.chapter, chapter_title
            )
        } else {
            format!(
                "Kapitel {} \u{2013} {} (nicht bewertet)",
                chapter.chapter, chapter_title
            )
        });
    }
    builder = builder.add_component(out_of_scope_list);

    let out_of_standard = out_of_standard_finding_count(findings);
    if out_of_standard > 0 {
        let note = if en {
            format!(
                "{out_of_standard} additional WCAG finding(s) outside WCAG 2.1 A/AA (e.g. AAA or WCAG 2.2-only criteria) are not covered by EN 301 549 V3.2.1 and are not represented in the clause table above."
            )
        } else {
            format!(
                "{out_of_standard} weitere(r) WCAG-Befund(e) außerhalb von WCAG 2.1 A/AA (z. B. AAA- oder nur-WCAG-2.2-Kriterien) sind von EN 301 549 V3.2.1 nicht abgedeckt und werden in der Klausel-Tabelle oben nicht ausgewiesen."
            )
        };
        builder =
            builder.add_component(Label::new(&note).with_size("10.5pt").with_color("#475569"));
    }

    builder
}
