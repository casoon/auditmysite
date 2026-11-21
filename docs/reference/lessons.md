---
title: Lessons from past defects
sidebarLabel: Lessons
description: Three lasting lessons from fixed defects that are not visible in the code alone - CSS coverage, PDF heading levels and the rules for report texts.
order: 92
---

Short notes on defects that were fixed and whose cause is easy to repeat. The fixes are in the
code; the reasons are here.

## CSS coverage is a ratio against all rules, not against the used ones

`CSS.stopRuleUsageTracking` returns only the rules that were used. Counting its entries as both
numerator and denominator made every page "100 % used". The denominator is the number of style
rules of the captured stylesheets (`stylesheet-parse`); where the stylesheets cannot be read the
result is `partial` or `not_available`, never 100 %. A measurement whose denominator comes from
the same list as its numerator cannot be below 100 %: check where the denominator comes from
before trusting a ratio.

## A heading level may not jump in the PDF

The PDF is tagged (PDF/UA). A level 3 heading directly under a level 1 heading makes the whole
document fail to compile, so a batch report loses every section. Sections added to a report start
at the level that follows their parent. `test_batch_pdf_renders_sitemap_status_section` pins the
case; a new section should be rendered in a test as well.

## Rules for report texts

- One number, one unit. Groups of findings, occurrences, elements and URLs are never mixed in one
  sentence without saying which is meant; a count and the share it is taken from name the same
  unit.
- No statement of impact without its evidence status. An impact is measured, plausible from the
  structure, or not checked, and the sentence says which. Conversion loss, brand effect and a
  fully blocked task are not measured by this tool.
- An unknown value is not 0 and not a perfect result. A module that measured nothing says so.
- Effort is not a time. Wording follows effort and scope of the change, never days or weeks.
- Report texts exist in German and English; the JSON stays English. Detection follows the page
  language, the wording follows the report language (see the localisation rule in `CLAUDE.md`).
