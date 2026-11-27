//! The PDF of the before/after comparison (#1235): a short report (one to
//! three pages) of the `output::compare::View`, built from the same
//! renderreport components as the audit reports. The texts are those of the
//! view, so the PDF and the terminal table cannot disagree.

use renderreport::components::advanced::KeyValueList;
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::count_text;
use crate::output::compare::{FindingGroup, View};

use super::helpers::create_engine;
use super::sanitize::render_pdf_sanitized;

/// Unchanged findings shown before "and n more": they are what did not move.
const UNCHANGED_SHOWN: usize = 12;
/// Pages and URLs listed before "and n more"; the JSON has all of them.
const LIST_SHOWN: usize = 25;

fn more(shown: usize, total: usize, en: bool) -> Option<String> {
    (total > shown).then(|| count_text("compare-more", total - shown, en))
}

fn group_rows(group: &FindingGroup, cap: usize, en: bool) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = group
        .items
        .iter()
        .take(cap)
        .enumerate()
        .map(|(i, item)| {
            vec![
                if i == 0 {
                    group.heading.clone()
                } else {
                    String::new()
                },
                item.clone(),
            ]
        })
        .collect();
    if let Some(text) = more(cap, group.items.len(), en) {
        rows.push(vec![String::new(), text]);
    }
    rows
}

/// Renders the PDF of `view`; `lang` is `en` or `de`.
pub(crate) fn generate_compare_pdf(view: &View, lang: &str) -> anyhow::Result<Vec<u8>> {
    let en = lang != "de";
    let engine = create_engine();
    let mut builder = engine
        .report("wcag-compare")
        .metadata("date", chrono::Utc::now().format("%Y-%m-%d").to_string())
        .metadata("version", env!("CARGO_PKG_VERSION"))
        .metadata("author", "auditmysite")
        .metadata("lang", lang)
        .metadata("footer_prefix", "Audit:")
        .metadata("footer_link_url", "")
        .metadata(
            "footer_tagline",
            "A technical auditing platform by casoon.de",
        );

    // A top-level section is also the document title (PDF/UA) without a title page.
    builder = builder.add_component(Section::new(&view.title).with_level(1));
    let mut runs = KeyValueList::new();
    for (label, value) in &view.runs {
        runs = runs.add(label, value);
    }
    builder = builder.add_component(runs);
    builder = builder.add_component(if view.worse {
        Callout::warning(&view.result)
    } else {
        Callout::success(&view.result)
    });

    // Scores
    builder = builder.add_component(Section::new(&view.scores_heading).with_level(2));
    let [name, old, new, change] = &view.score_columns;
    let mut scores = AuditTable::new(vec![
        TableColumn::new(name).with_width("46%"),
        TableColumn::new(old).with_width("18%").with_align("right"),
        TableColumn::new(new).with_width("18%").with_align("right"),
        TableColumn::new(change)
            .with_width("18%")
            .with_align("right"),
    ]);
    for row in &view.scores {
        scores = scores.add_row(vec![&row.name, &row.old, &row.new, &row.change]);
    }
    builder = builder.add_component(scores);
    if let Some((label, value)) = &view.verdict {
        builder = builder.add_component(KeyValueList::new().add(label, value));
    }

    // Findings
    builder = builder.add_component(Section::new(&view.findings_heading).with_level(2));
    builder = builder.add_component(TextBlock::new(&view.findings_counts));
    if !view.groups.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("").with_width("18%"),
            TableColumn::new("").with_width("82%"),
        ]);
        for (i, group) in view.groups.iter().enumerate() {
            // The unchanged group is the last one and the one that may be long.
            let cap = if i + 1 == view.groups.len() {
                UNCHANGED_SHOWN
            } else {
                usize::MAX
            };
            for row in group_rows(group, cap, en) {
                table = table.add_row(row);
            }
        }
        builder = builder.add_component(table);
    }

    // Pages of a batch, the ones that lost most first.
    if let Some(heading) = &view.pages_heading {
        builder = builder.add_component(Section::new(heading).with_level(2));
        let [page, score, findings] = &view.page_columns;
        let mut table = AuditTable::new(vec![
            TableColumn::new(page).with_width("42%"),
            TableColumn::new(score).with_width("20%"),
            TableColumn::new(findings).with_width("38%"),
        ]);
        for row in view.pages.iter().take(LIST_SHOWN) {
            table = table.add_row(vec![
                row.label.clone(),
                row.overall.clone(),
                format!("{}; {}", row.findings, row.verdict),
            ]);
        }
        builder = builder.add_component(table);
        if let Some(text) = more(LIST_SHOWN, view.pages.len(), en) {
            builder = builder.add_component(TextBlock::new(text));
        }
    }
    for group in [&view.only_old, &view.only_new].into_iter().flatten() {
        builder = builder.add_component(Section::new(&group.heading).with_level(2));
        let mut urls = KeyValueList::new();
        for url in group.items.iter().take(LIST_SHOWN) {
            urls = urls.add("", url);
        }
        builder = builder.add_component(urls);
        if let Some(text) = more(LIST_SHOWN, group.items.len(), en) {
            builder = builder.add_component(TextBlock::new(text));
        }
    }
    builder = builder.add_component(TextBlock::new(&view.note));

    let mut request = builder.build();
    render_pdf_sanitized(&engine, &mut request)
}
