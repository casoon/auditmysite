# PDF Report Contract

This document defines the minimum quality contract for `auditmysite` PDF reports.
It complements `docs/OUTPUT_CONTRACT.md`, which covers JSON compatibility.

## Goals

PDF reports are customer-facing artifacts. They should help readers understand:

- what was audited,
- how severe the current state is,
- which issues matter most,
- what to fix next,
- which technical details developers need for implementation.

## Report Levels

### Executive

Audience: management, project leads, non-technical stakeholders.

Target page count: 5–8 pages (8 pages maximum with a realistic finding load).

Minimum content:

- branded cover with score, grade, date, domain and active modules,
- concise risk/status summary,
- top findings without raw technical snapshots,
- business/user impact,
- prioritized next actions,
- methodology note.

Technical selectors, HTML snippets and code examples must not dominate this level.
Dense implementation tables, raw selector lists and grouped code examples are not
appropriate at the Executive level.

### Standard

Audience: product owners, project leads, UX/design, developers.

Minimum content:

- everything from the executive report,
- action plan,
- module overview,
- prioritized finding cards,
- technical handoff section,
- representative findings and module details.

### Technical

Audience: developers and implementers.

Minimum content:

- full standard report,
- complete technical appendix where useful,
- selectors, affected elements, code examples and grouped finding patterns,
- per-URL detail tables for batch reports when available.

## Batch Reports

Minimum content:

- branded cover,
- audited domain/source and number of URLs,
- portfolio score and grade,
- ranking or URL overview,
- recurring issue patterns,
- action plan,
- crawl/link diagnostics when the audit came from crawl mode.

Batch reports are not a stack of single-page reports. They should focus on
domain-wide patterns, repeated issues and prioritization.

## Comparison Reports

Minimum content:

- branded cover,
- compared domains and average score,
- domain ranking,
- module comparison when module scores are available,
- top findings by domain.

## Module Classification: Measured vs. Heuristic

Modules are divided into two classes:

**Measured** — score derived from quantitative, reproducible signals (CDP data, DOM counts,
network timings, WCAG rule checks). Examples: Accessibility, Performance, Security.

**Heuristic indicators** — score derived from structural signals and inferred patterns.
These are best-effort estimates, not definitive values. Examples: UX, User Journey,
Source Quality, AI Visibility and Content Visibility.

In the rendered PDF:

- Heuristic score cards and module overview entries carry the "(Indicator)" / "(Indikator)" suffix in their title.
- Heuristic score cards carry the description "Heuristic estimate based on structural
  signals" in the subtitle.
- Indicator modules may be displayed in the module dashboard, but do not contribute to
  the combined technical score unless explicitly marked as contributing in the JSON model.
  They carry no letter grade — a grade on a module that feeds nothing claims an authority
  it does not have.
- An indicator that re-reads another module's result instead of measuring anything new names
  the modules it derives from in the JSON model (`derived_from` on its module score).
- The cover headline is the accessibility score, with its grade. The cover carries no certificate or badge: an automated check covers only part of the criteria and certifies nothing (#911). The weighted
  combined technical score is a separate, labelled card carrying its weight basis, never the
  headline.
- No visual distinction between measured and heuristic modules in the module overview strip —
  the indicator suffix is the only differentiator.

## Numeric Context and Cross-Format Consistency

- Specialized scores must state their scale or denominator where they appear, for example `20 / 100`; nearby copy must explain whether higher or lower values are better.
- Technical metrics must include a unit and, where interpretation is not self-evident, a target range or short definition. Ordinary counts and familiar quantities such as dates, durations, file sizes, and page totals do not need repeated explanations.
- Dual-viewport accessibility uses one canonical formula everywhere: 70% mobile plus 30% desktop, rounded to an integer. The merged finding inventory remains evidence and does not produce another score.
- Labels must distinguish distinct WCAG finding groups, WCAG occurrences, and findings from all categories. A number must not be labelled “critical” when it combines critical and high severities.
- Single and batch PDF scores, grades, and scoped counts must match the corresponding JSON fields.

## Language

- CLI output (terminal table, progress messages) is always English.
- PDF reports are localized via `--lang en|de`. Default is English.
- All user-visible labels in PDF output must use Fluent (`.ftl`) keys.
  Hard-coded German strings in PDF renderers are a contract violation.

### Localization Boundaries

AuditMySite currently has nine localization surfaces. Keep new text in the
surface that owns its lifecycle:

- `locales/{de,en}/report.ftl`: runtime report labels, headings, table columns,
  callout titles, status labels, and short reusable PDF/JSON presentation text.
  New PDF renderer text should start here.
- `locales/{de,en}/rules.ftl`: the texts of the rule catalogue
  (`src/taxonomy/rules.rs`): one message `rule-<id>` per rule id, the one title
  of the rule as its value, user impact and technical impact as attributes. Read
  through `Rule::title`, `Rule::user_impact` and `Rule::technical_impact`. A
  rule that has an explanation entry under its own id carries the explanation
  texts (`.description`, `.explanation-user-impact`, `.typical-cause`,
  `.recommendation`, `.technical-note`) on the same message. The explanation
  entries keyed by a WCAG criterion number or an axe id (shared texts) are
  messages `explanation-<key>` in the same files, the customer title as value
  and the same five attributes (`.user-impact` here). Read through the methods
  of `RuleExplanation` (`customer_title`, `customer_description`,
  `user_impact`, `typical_cause`, `recommendation`, `technical_note`), which
  pick the right message.
- `locales/{de,en}/page-health.ftl`: the texts of the page-health issues
  (`seo::page_health::collect_issues`): one message `page-health-<issue-type>` per
  issue type with the counts as named arguments, plural agreement as a Fluent
  select, plus the argument parts (samples, "none", cache reasons). The function
  keeps the thresholds, severities and the arguments. Also the HTML validation
  texts of the SEO view (`seo::page_health::html_validation_check_text`,
  `html_validation_detail_text`, `html_validator_delegated_text`,
  `html_validator_executed_text`, `page-health-html-validation-*`,
  `page-health-html-validator-*`); the sample lists are joined in Rust.
- `locales/{de,en}/seo-issues.ftl`: the texts of the technical-SEO issues
  (`seo::technical::collect_technical_issues`, `seo-technical-*`), the meta-tag
  findings (`seo::meta::meta_issue_text`, `seo-meta-*`) and the structured-data
  issues (`seo::schema::schema_issue_text`, `seo-schema-*`), built the same way as
  `page-health.ftl`; also the table labels of the structured-data issues
  (`seo::schema::schema_issue_label`, `seo-schema-label-*`) and of the
  robots.txt audit (`seo::robots::BotClass::label`, `infer_robots_policy`,
  `localize_policy_label`, `seo-robots-*`), the content-profile labels
  (`seo::profile::SeoMaturityLevel::label`/`description`, `PageType::label`,
  `seo-profile-*`) and texts (`seo::profile::build_content_profile`: content
  type, summary fallback, page attributes and the names, check labels and
  details of the signal categories, `seo-profile-*`; which check and detail
  applies stays in Rust, language-neutral names such as `Meta-Tags` and
  `HTTPS` are written there), the SERP status labels
  (`seo::serp::SerpSignalStatus::label`, `seo-serp-status-*`) and signal
  texts (`seo::serp::build_serp_analysis`: the category, label and detail of
  each signal, `seo-serp-*`; which signal and detail applies stays in Rust,
  language-neutral names such as `Description`, `Canonical` and `Favicon` are
  written there), the
  structured-data fit labels (`seo::schema_fit::SchemaPageKind::label`,
  `SchemaFitAssessment::fit_text`, `seo-schema-fit-*`), the content-parity
  texts (`seo::schema_parity::ContentParityAssessment::status_text`/
  `evidence_text`, `seo-schema-parity-*`) and the rich-result rule wording
  (`seo::schema_rules::status_text`, `manual_review_text` and the breadcrumb
  name of `feature_label`, `seo-schema-rules-*`; the other feature names are
  language-neutral and written in Rust).
- `locales/{de,en}/security-issues.ftl`: the texts of the security findings
  (`security::security_issue_text`, `security-*`), built the same way; the
  function keeps the default values (`unset`, `object-src`). Also the category
  labels (`security::SecurityCategory::label`, `security-category-*`).
- `locales/{de,en}/mobile-issues.ftl`: the texts of the mobile findings
  (`mobile::mobile_issue_text`, `mobile-*`) with the parts of the touch-target
  sentence and the touch-target context labels (`mobile::mobile_context_label`,
  `mobile-context-*`), built the same way; the number format stays in
  `i18n::number`.
- `locales/{de,en}/labels.ftl`: the labels of cross-cutting enums, one
  `labels-*` message per variant: page intent (`page_flow::PageIntent::label`),
  priority, effort and execution priority (`audit::prioritization`), role
  (`output::report_model::Role::label`), issue class
  (`taxonomy::IssueClass::label`) and subcategory
  (`taxonomy::Subcategory::label`), and the evidence source labels of content
  signals (`EvidenceSource::label`, `labels-evidence-source-*`). The enum methods keep the variant-to-message
  mapping. The PDF tables of priority, role and effort use their own
  `report.ftl` messages (`priority-*`, `role-*`, `effort-*`), which word some of
  them differently; the `labels-` prefix keeps the two apart.
- `locales/{de,en}/interactive.ftl`: the texts of the interactive (journey)
  findings (`audit::interactive_finding::interactive_finding_text`): one
  message `interactive-<kind>` and one `interactive-<kind>-fix` per finding
  kind, plural agreement as a Fluent select, plus the uncertainty note
  (`finding_uncertainty_text`, `interactive-uncertainty-*`). The function keeps
  the kind-to-message mapping and the arguments (default values, the quoted
  page title, the truncation marker of the example list).
- `locales/{de,en}/batch-consistency.ftl`: the texts of the batch PDF's
  cross-page consistency section (`output::pdf::batch_report::consistency`,
  `batch-consistency-*`) and of its management assessment, key points, audit
  caveats, internal comparison and decision table
  (`output::pdf::batch_report::management`, `batch-management-*`). The render
  functions keep the thresholds, which message applies and how lists are
  joined. A basis or identity finding the analysis stores in canonical English
  is the English message itself (`$basis`, `$finding`); the German message
  translates it.
- `locales/{de,en}/single-report.ftl`: the texts of the single PDF's page
  sections (`output::pdf::single_report`, `single-report-*`): the part
  dividers, the management summary (scope line, evidence note, overall
  rating, risks and strengths, key measures), the findings classification and
  matrix, the appendix with its manual-review notes, the recognized structural
  patterns and the dual viewport summary. The render functions keep the
  thresholds, which message applies and how lists are joined; the severity
  labels, the overall rating title, verdict words and reasons, the module
  names and the matrix column headers are reused `report.ftl` messages.
- `locales/{de,en}/methodology.ftl`: the texts of the single report's
  methodology section (`output::builder::single::methodology::build_methodology`:
  scope, method, limitations, disclaimer, the audit-fact labels and values, the
  confidence summary and the capability matrix, `methodology-*`) and the score
  methodology of its appendix (`build_appendix_block_from_normalized`). The
  builder keeps the thresholds, which message applies and how lists are
  joined; the runtime keeps its number format in `i18n::number`. The
  confidence summary and capability matrix also feed the JSON detail, which
  builds them in English. The fact labels WCAG level, Checked nodes, Runtime,
  Audit notes, Display mode and Active modules reuse the identical `report.ftl`
  messages (`scope-box-*`, `batch-cover-frame-*`).
- `locales/{de,en}/seo-details.ftl`: the texts of the single report's SEO
  module detail presentation (`output::builder::single::module_details::seo`,
  `seo-details-*`): meta-tag labels and the cross-module notes of meta issues,
  the heading, social, technical and tracking summaries, the schema inventory
  details, the signal ratings and the content identity and page profile
  labels. The builder keeps the rating bands, which message applies and how
  lists and values are joined; the labels "Description", "Schema.org content
  type" and "Language" are `pdf-seo-*` messages of `report.ftl`.
- `locales/{de,en}/diagnosis.ftl`: the texts of the single report's findings
  overview and system diagnosis (`output::builder::single::diagnosis`:
  issue-pattern labels and descriptions, the criticality-tier labels, eyebrows
  and intros and the thematic-cluster labels, `diagnosis-*`) and the
  dominant-issue spotlight of the PDF diagnosis section
  (`output::pdf::diagnosis`). The builders keep which message applies to an
  issue pattern, a tier and a finding category; the severity labels
  (`severity-*`), the finding count (`count-findings`), the module names of
  other categories (`module-*`) and the table headings (`diagnosis-col-*`,
  `diagnosis-table-*`, `diagnosis-clusters-note`) are reused `report.ftl`
  messages.
- `locales/{de,en}/accessibility-details.ftl`: the texts of the single PDF's
  accessibility detail sections (`output::pdf::detail_modules::accessibility`,
  `a11y-details-*`): the keyboard accessibility journey (title, overview,
  overflow note, tested sequences, journey labels, disclaimer), the
  screen-reader reading order (title, intro, metric labels, score note,
  findings table title) and the BFSG callout per verdict. The renderer keeps
  the deduplication, which message applies and the score format; the counts
  are Fluent numbers and the minor-issue plural is a Fluent select. The
  severity labels of the screen-reader findings reuse `severity-*` of
  `report.ftl`.
- `locales/{de,en}/score-drivers.ftl`: the texts of the single PDF's score
  drivers in the management summary (`output::pdf::score_drivers`,
  `score-drivers-*`): the combined technical score card, the score-driver
  table (title per calculation method, column headings, the assessment per
  module), the weight-basis note, the viewport-weighted score derivation
  (title, column headings, step and input labels, note), the additional
  score drivers title and the subcategory breakdown lines. The renderer
  keeps which assessment applies (weakest module, risk driver, corrected
  band), the module lists, the score and weight cells and the formula
  cells; the weight basis is a Fluent number. The module column heading
  reuses `label-module` of `report.ftl`.
- `locales/{de,en}/pdf-findings.ftl`: the texts of the PDF finding cards
  (`output::pdf::findings`, `pdf-findings-*`): the severity prefix, the
  code labels of the metadata strip and rows, the metadata and assessment
  row keys, the BFSG/EAA label spelled out and abbreviated, the occurrence
  evidence rows (DOM path, contrast, color scheme), the screenshot
  captions, the hidden-occurrence note, the structural-cause labels and
  the customer-perspective titles; and of the shared module detail helpers
  (`output::pdf::detail_modules`, `pdf-details-*`): the module customer
  context, the score card caption, the Web Vitals rating and the journey
  category labels. The renderers keep which message applies and the decimal notation of the
  contrast ratio; counts are Fluent numbers and the hidden-occurrence
  plural is a Fluent select. The code labels High, Medium and Low reuse
  `severity-*` of `report.ftl`, the journey categories SPA navigation and
  Quantity stepper the `a11y-details-journey-label-*` messages of
  `accessibility-details.ftl`.
- `locales/{de,en}/pdf-frame.ftl`: the texts of the single PDF's frame
  (`output::pdf` mod.rs, `pdf-frame-*`): the cover labels, the table of
  contents title, the three part intros with their audience titles and
  bodies, the confidence notes and the audit flag customer texts, and the
  side checks that got no answer. The frame keeps which message applies, the
  joining of the side check names and the cover band phrase and scope line
  (messages of the modules that own them); the titles of the audit flags
  reuse the `batch-management-flag-*` messages of `batch-consistency.ftl`.
- `locales/{de,en}/actions-block.ftl`: the texts of the single report's
  action plan block (`output::builder::single::actions_block`,
  `actions-block-*`): the systemic and local column labels and descriptions,
  the block title and intro for a good site and for the default case and the
  risk effect per priority. The block keeps which message applies and the
  thresholds; the leverage tiers reuse `pdf-findings-code-very-high`,
  `severity-high` and `severity-low`, and the role, priority, execution
  priority and effort labels are the messages of their own enums. The JSON
  detail builds the block with the locale `en`, so it stays canonical English.
- `locales/{de,en}/audit-texts.ftl`: the texts of the audit (analysis) layer
  (`audit-*`): the technical impacts of the SEO heading issues
  (`audit::normalized::seo_heading_finding_text`; the titles are the
  `explanation-seo-headings-*` messages of `rules.ftl`), the 3.2.6 assessment
  basis, the region names and the deviation sentences
  (`audit::batch_consistency`, which the batch PDF's consistency section
  calls with the run language) and the management risk dimension names and
  rationales (`audit::management_risk`, also used by the single PDF). The
  functions stay pure `*_text(kind, values, en)` functions: the JSON calls
  them with `en = true` and stays canonical English, the PDF with the run
  language. Which message applies, the thresholds and the score format
  (`n/100`) stay in Rust; the plurals are Fluent selects.
- `locales/{de,en}/audit-interpretation.ftl`: the texts of the
  interpretation layer (`audit-interpretation-*`): the SEO interpretation
  lead, page-type context, profile summary and optimization note
  (`seo::interpretation`), the performance qualifiers and gap sentence
  (`audit::performance_interpretation`), the score interpretation per
  module and band, the accessibility claim bullet, the technical overview
  bullets, the overall impact table and the benchmark sentence
  (`audit::interpretation`), the verdict reasons (`audit::verdict`), the
  complexity and expected-impact sentences of a finding
  (`audit::finding_derive`), the audit summary narrative (`audit::summary`),
  the long-heading message (`seo::headings`), the baseline line
  (`audit::baseline`) and the score area labels (`taxonomy::score_area`).
  The functions stay pure `*_text(kind, values, en)` functions: the JSON
  calls them with `en = true` and stays canonical English, the PDF with the
  run language. Which message applies, the thresholds and the number
  notation (the DOM node count of the performance qualifier) stay in Rust;
  the plurals are Fluent selects; the word "and" of the phrase lists is
  `audit-join-and`.
- `locales/{de,en}/a11y-analysis.ftl`: the texts of the accessibility
  analysis layer (`a11y-analysis-*`): the screen reader issue messages
  (`screen_reader::analyzer::analyze_reading_sequence`: non-descriptive and
  duplicated names, link list, announcement deserts, tab stops, heading
  levels and order, landmarks, icon-font characters, empty elements and
  form labels) and the recognized-pattern messages
  (`patterns::pattern_message_text`, with the counted nouns as Fluent plural
  selects). The functions keep their `en: bool` parameter: the JSON calls
  them with `en = true` and stays canonical English, the PDF with the run
  language. The detection language (page-language stopwords of
  `analyze_reading_sequence`), which issue applies and the thresholds stay
  in Rust.
- `locales/{de,en}/batch-report.ftl`: the texts of the batch PDF's keyboard
  journey summary (`output::pdf::batch_report::interactive`: intro, table
  title and headings, the interactive category names), the module name
  "HTML Conformance", the impact summary (`portfolio`: the user, business
  and risk rows), the cover contents title and the EN 301 549 clause
  roll-up and overview cards (`overview`:
  title, intro, column headings, clause statuses, out-of-scope chapter
  line, card titles), all `batch-report-*`. The renderers keep which
  message applies, the number formats (word counts, shares), the display
  mode, rating band and baseline texts of other modules, and the clause
  titles and disclaimers that `wcag::en301549` owns. The journey title, the
  severity labels, the module names and "Share" reuse
  `a11y-details-journey-title`, `severity-*`, `module-*` and
  `batch-col-share`.
- `locales/{de,en}/pdf-analysis.ftl`: the texts of the search experience
  composite (`output::search_experience`: component labels and
  explanations, warnings, interpretation), the root cause analysis and the
  action plan (`output::pdf::root_causes`: titles, lead-in, table, the
  leverage and root cause tags, the other modules' measures), the problem
  profile (`output::pdf::problem_profile`: kind labels, descriptions,
  concentration and leverage notes) and the WCAG coverage section
  (`output::pdf::wcag_coverage`: principle names, audit scope, the legend
  and the manual test guide), all `pdf-analysis-*`. The structural findings
  tail and the verb of the profile sentences are Fluent selects. The
  renderers keep which message applies, the thresholds, the criterion tags
  and the texts that other modules own (the UX dimension summaries, the
  German names of the manual review criteria of `wcag::coverage`). The
  label "Technical SEO" and the table headings "Finding", "Occurrences"
  and "Share" reuse `seo-score-card`, `single-report-matrix-col-finding`,
  `batch-col-occurrences` and `batch-col-share`.
- `locales/{de,en}/report-builder.ftl`: the texts of the single report's
  builder, all `report-builder-*`: the positive signals (pattern titles, the
  WCAG signal) and aspects (`output::builder::single::{executive,
  positive}`), the executive key points and the score driver note, the audit
  quality note with the stability measurements it names and the quick-win
  metric label (`output::builder::single`), the overall score explanation
  (`modules_block`), the page health rows (`serp`) and the performance,
  security and mobile detail rows (`module_details::{performance,
  platform}`: labels, guideline texts, the security score rows and band note,
  the touch-target sentence). The terminal table's experimental indicator
  lines (`output::terminal`) are `report-builder-terminal-*` messages of the
  same file and follow `--lang`. The builders keep which message applies, the
  thresholds, the counts and the number formats (the decimal comma and the
  CO2e notation); the sentence-leading space of the indicator note and the
  scope note stays in Rust because Fluent trims leading blanks. "Accessibility"
  and "Security" reuse `module-accessibility` and `module-security`; the batch
  cover date takes its format from `date-format-str`. The JSON stays canonical
  English: `output/json/detail.rs` builds the same view model with the locale
  `en`.
- `locales/{de,en}/output-texts.ftl`: the texts of the report builder helpers
  (`output::builder::helpers`: report title and subtitle, the security
  header tier and issue kind labels), the batch builder
  (`output::builder::batch`: the page-type distribution insights, the lever
  fallback, the render-blocking summary labels and values, the cover title
  and URL line), the display mode texts (`display`: the audited mode and the
  offered modes) and the tech stack, best practices, SEO and performance
  detail modules (`output::pdf::detail_modules::{indicators, seo,
  performance}`: chapter takeaways, the schema count with its plural select,
  the viewport score notes and legend, the lab metric headings), all
  `output-*`. The code keeps which message applies, the number formats and
  the values that go into a message (counts, the viewport name; the cover
  date takes its format from `date-format-str`); "Yes", "No", "Metric" and
  "Value" reuse the `yes`,
  `no`, `col-metric` and `col-value` messages of `report.ftl`.
- `locales/{de,en}/privacy-findings.ftl`: the texts of the privacy findings
  (`privacy-finding-*`): the seven finding messages and the policy document
  names of `privacy::findings::finding_text`, which the single PDF's privacy
  chapter and the run-language copy of the findings (`audit::finding_text`)
  call with the run language. The function
  stays a pure `finding_text(kind, provider, document, en)`: the analysis
  stores `en = true` in `PrivacyFinding.message`, so the JSON stays canonical
  English. Which message applies stays in Rust; the provider name goes in as
  an argument.
- `locales/{de,en}/pdf-sections.ftl`: the texts of the opt-in annexes of the
  single PDF (`output::pdf::{en301549, bik_guide}`: titles, intros, the
  level column, the "no violations" tag cloud title, the footnote on
  findings outside the standard, the BIK chapter column, tag cloud and
  note titles), the search experience chapter and the budget violations
  section (`output::pdf::detail_modules::overview`: title, body, warning
  list title, column headings, the three translated budget metric names),
  the four VoiceOver/NVDA self-check instructions (`helpers`:
  `manual_recheck_instruction`, which takes `en` and returns a `String`),
  the "Indicator" suffix of a heuristic module name, the cover phrase of a
  failed run and the "provisional" suffix (`cover`) and the occurrences
  word of the appendix table (`appendix`), all `pdf-sections-*`. The
  renderers keep which message applies and the `(n)` count suffixes of
  the table and tag cloud titles; the clause, chapter and BIK chapter
  titles and the disclaimers that `wcag::en301549` and `wcag::bik_guide`
  own stay a Rust choice between their two fields. The column headings
  Clause, Title and Findings, the status titles (violations found, manual
  review required, evaluation incomplete), the disclaimer and out-of-scope
  titles and the chapter line reuse `batch-report-en301549-*` and
  `diagnosis-col-findings`, the component heading `pdf-sec-score-component`.
- `src/output/explanations.rs`: the explanation table itself: which entries
  exist, owner role, effort, and the code examples (canonical English), plus
  `RULE_ENTRIES`, the explicit table of which entry each rule id uses. These are
  rule catalog facts rather than layout labels; their prose lives in
  `rules.ftl`. Do not add generic report UI copy here.
- `src/audit/interpretation.rs`: computed interpretation sentences that depend
  on normalized audit signals and must be available before the output layer is
  built. Prefer Fluent keys in the interpretation model when the text is a fixed
  report sentence; use `LocalizedText` only when the sentence is derived from
  audit data and stored with the normalized report.

When adding another language, Fluent bundles are the primary expansion point.
Any remaining paired `*_en` fields or `LocalizedText { de, en }` values are
known migration seams from issue #417 and should not grow without a specific
reason.

## Visual Quality Requirements

Every PDF should:

- render successfully to valid PDF bytes,
- have a recognizable first page when rasterized,
- include a brand/logo asset or the AuditMySite fallback wordmark,
- avoid internal/debug labels such as `Raw`, `CLI parity` or Rust debug output,
- keep executive pages free from dense technical implementation tables,
- use consistent score/status colors,
- keep tables and code examples readable without obvious clipping.

## Regression Checks

Current automated checks:

- single-report PDF smoke test renders a valid PDF,
- single-report Executive/Standard/Technical smoke test renders valid PDFs,
- batch PDF smoke test renders a valid PDF,
- comparison PDF smoke test renders a valid PDF,
- first Executive PDF page is rasterized to PNG with `pdftoppm` when available.

Future checks:

- rasterize representative pages for Standard, Technical, Batch and Comparison,
- assert page counts or minimum page ranges per report type,
- add golden-master screenshots for selected first pages,
- verify custom logo assets are visible in rendered output,
- run visual checks in CI with a pinned PDF rasterizer.
