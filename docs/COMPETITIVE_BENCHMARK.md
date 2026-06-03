# Competitive Benchmark

This note captures the current product bar set by common accessibility tooling and what `auditmysite` should do to stay differentiated.

## Reference Products

### Lighthouse

Official docs:
- https://developer.chrome.com/docs/lighthouse
- https://developer.chrome.com/docs/lighthouse/accessibility/scoring

What matters:
- Combines accessibility with adjacent categories such as performance and SEO.
- Explains every failing audit and links remediation guidance.
- Uses an explicit scoring model. The accessibility score is a weighted average of pass/fail audits.

Implication for `auditmysite`:
- Keep accessibility as the primary score contract.
- Keep adjacent modules, but never blur them into the accessibility value without making that explicit.
- Preserve explainable scoring and remediation text.

### Pa11y CI

Official repo/docs:
- https://github.com/pa11y/pa11y-ci

What matters:
- Strong CI orientation.
- Built-in `cli` and `json` reporters.
- Supports custom reporters, HTML reporters, and JSON file output.

Implication for `auditmysite`:
- Deterministic JSON and schema validation are mandatory.
- Reporting must stay automation-friendly.
- Custom or richer reporting should not compromise stable machine output.

### Accessibility Insights

Official docs:
- https://accessibilityinsights.io/docs/windows/getstarted/fastpass/
- https://accessibilityinsights.io/docs/windows/getstarted/automatedchecks/
- https://accessibilityinsights.io/docs/web/getstarted/fastpass/

What matters:
- Combines automated checks with guided manual validation.
- Explicitly pushes users from automated failures into targeted follow-up checks.
- Makes issue export and issue handoff straightforward.

Implication for `auditmysite`:
- Reports should clearly separate automated findings from what still requires manual review.
- Action plans should map findings to concrete ownership and next steps.
- The report should be usable by developers, QA, and stakeholders, not only auditors.

### Siteimprove Accessibility Code Checker / Platform

Official pages:
- https://www.siteimprove.com/platform/accessibility-digital-governance/accessibility-code-checker/
- https://help.siteimprove.com/support/solutions/articles/80001151769-accessibility-code-checker
- https://www.siteimprove.com/acr/

What matters:
- Strong workflow integration with Selenium, Playwright, Puppeteer, and Cypress.
- Historical tracking and dashboarding are prominent product strengths.
- Detailed page reports and AI/code-fix guidance are part of the value proposition.
- Executive dashboards matter as much as technical issue lists.

Implication for `auditmysite`:
- PDF reports must be presentation-grade, not just technically correct.
- History, trend comparison, and batch benchmarking are strategic gaps worth closing.
- The split between executive summary and technical appendix is a product requirement, not a nice-to-have.

## Where `auditmysite` is already strong

- Real browser rendering via CDP and the native Accessibility Tree.
- Single-binary distribution and local CLI ergonomics.
- Deterministic JSON contracts with schema validation (Unified Report Envelope v2.0).
- Multi-module view across accessibility, performance, SEO, security, mobile, UX, journey, AI visibility, and more.
- Stakeholder-oriented PDF output via renderreport/Typst — presentation-grade typography.
- Accessibility Journey Layer: interactive checks (tab-walk, focus-trap, SPA navigation, form-error announcement).
- Semantic AI evaluation (local Fastembed + optional Mistral) for content discoverability signals.
- Historical tracking and trend comparison across audit runs.
- Stable scoring semantics documented in OUTPUT_CONTRACT.md and enforced via JSON schema.
- Actionable fix guidance (`fix_guidance`) in every JSON report, per finding.

## Current gaps to close

1. Interactive/manual follow-up guidance is present in journey findings but not yet a first-class story in the PDF executive report.
2. Trend dashboarding across repeated audits is functional in JSON but not yet surfaced as a visual PDF chapter.
3. Iframe and embedded content analysis is limited compared to axe-core's `frame-tested` scope.

## Product Direction

To beat typical competitor setups, `auditmysite` should aim for this combination:

- Browser-accurate audit data
- Stable automation contract
- Executive-ready PDF reporting
- Clear scoring semantics
- Lightweight installation and CI usage
- Practical remediation guidance with ownership and effort
