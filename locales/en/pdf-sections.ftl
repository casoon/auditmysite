# PDF annex and section helper texts (#936, slice 25): the EN 301 549 and BIK
# annexes (`output::pdf::{en301549, bik_guide}`), the search experience and
# budget sections (`output::pdf::detail_modules::overview`), the self-check
# instructions and the indicator suffix (`output::pdf::helpers`), the cover
# phrase of a failed or provisional run (`output::pdf::cover`) and the
# occurrences word of the appendix table (`output::pdf::appendix`). The clause
# and chapter titles and the disclaimers that `wcag::en301549` owns stay in
# Rust, as do the table headings and labels that `batch-report.ftl`,
# `diagnosis.ftl` and `report.ftl` already define.

pdf-sections-en301549-title = EN 301 549 clause mapping
pdf-sections-en301549-intro = Maps this audit's automated WCAG 2.2 A/AA findings onto EN 301 549 V4.1.1 (2026-09), chapter 9 (Web) clause numbers, with a per-clause status: violations found, no violations found in the automated scope, or manual review required.
pdf-sections-en301549-col-level = Level
pdf-sections-en301549-no-violations = No violations in automated scope
pdf-sections-en301549-out-of-standard =
    { $count ->
        [one] 1 additional WCAG finding outside WCAG 2.2 A/AA (Level AAA criteria) has
       *[other] { $count } additional WCAG findings outside WCAG 2.2 A/AA (Level AAA criteria) have
    } no normative clause in EN 301 549 V4.1.1 and { $count ->
        [one] is
       *[other] are
    } not represented in the clause table above.
pdf-sections-bik-title = BIK für Alle guide mapping
pdf-sections-bik-intro = Groups this audit's automated findings under the chapter structure of the "BIK für Alle" editorial accessibility guide ({ $source }) — the same chapters editorial teams already use: Images & alt text, Link text, Structure, Easy language, PDF documents, Videos. No new checks; this re-groups findings already reported elsewhere in this document.
pdf-sections-bik-col-chapter = Chapter
pdf-sections-bik-findings-title = Findings by chapter
pdf-sections-bik-clean-title = Checked, no findings
pdf-sections-bik-not-checked = Not automatically checked: { $chapters }: this tool does not currently check this chapter's content automatically. Not represented in the findings table above.
pdf-sections-search-title = Search Experience
pdf-sections-search-body = This composite combines { $components }. Classic SEO remains visible in the next detail section.
pdf-sections-search-not-measured = Not measured in this run: { $components }.
pdf-sections-search-warnings = What still goes wrong
pdf-sections-search-col-weight = Weight
pdf-sections-search-col-meaning = Meaning
pdf-sections-search-table-title = How the score is composed
pdf-sections-budget-js-size = JS size
pdf-sections-budget-css-size = CSS size
pdf-sections-budget-page-size = Page size
pdf-sections-recheck-landmarks = Self-check: open VoiceOver's Rotor (VO+U, macOS) or NVDA's Elements List (Insert+F7) and switch to "Landmarks" — listen whether the areas you'd expect (header, navigation, footer) are actually announced.
pdf-sections-recheck-headings-landmarks = Self-check: use VoiceOver's Rotor (VO+U) or NVDA's Elements List (Insert+F7) and check both the "Headings" and "Landmarks" views — confirm the order/uniqueness you hear matches what this finding describes.
pdf-sections-recheck-form-names = Self-check: tab into the form with VoiceOver (VO+Cmd+J jumps between form controls) or NVDA (F jumps between fields) and listen whether every field announces a name — not just a generic "edit text".
pdf-sections-recheck-status-messages = Self-check: trigger the message (e.g. submit the form with an invalid value) while VoiceOver or NVDA is running and confirm it is announced automatically, without moving keyboard focus to it.
pdf-sections-indicator-suffix = Indicator
pdf-sections-cover-fail = Not passed in the automated audit scope — action required
pdf-sections-cover-provisional = provisional
