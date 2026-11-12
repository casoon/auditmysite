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
- An indicator that re-reads another module's result instead of measuring anything new says
  so in a sentence below its score, and an accessibility penalty applied to it is named with
  its point value rather than folded in silently.
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
  `seo-profile-*`) and the SERP status labels
  (`seo::serp::SerpSignalStatus::label`, `seo-serp-status-*`), the
  structured-data fit labels (`seo::schema_fit::SchemaPageKind::label`,
  `SchemaFitAssessment::fit_text`, `seo-schema-fit-*`) and the content-parity
  texts (`seo::schema_parity::ContentParityAssessment::status_text`/
  `evidence_text`, `seo-schema-parity-*`).
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
  (`taxonomy::Subcategory::label`), and the evidence labels of content signals
  (`EvidenceSource::label`, `assessment::evidence_field_label`,
  `labels-evidence-*`). The enum methods keep the variant-to-message
  mapping. The PDF tables of priority, role and effort use their own
  `report.ftl` messages (`priority-*`, `role-*`, `effort-*`), which word some of
  them differently; the `labels-` prefix keeps the two apart.
- `locales/{de,en}/batch-consistency.ftl`: the texts of the batch PDF's
  cross-page consistency section (`output::pdf::batch_report::consistency`,
  `batch-consistency-*`) and of its management assessment, key points, audit
  caveats, internal comparison and decision table
  (`output::pdf::batch_report::management`, `batch-management-*`). The render
  functions keep the thresholds, which message applies and how lists are
  joined. A basis or identity finding the analysis stores in canonical English
  is the English message itself (`$basis`, `$finding`); the German message
  translates it.
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
