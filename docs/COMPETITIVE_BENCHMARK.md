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
- Deterministic JSON contracts with repository-level validation.
- Multi-module view across accessibility, performance, SEO, security, and mobile.
- Stakeholder-oriented PDF output with roadmap sections.

## Current gaps to close

1. Score semantics must stay unambiguous across CLI, JSON, and PDF.
2. PDF reports need cleaner typography, better text wrapping, and reliable umlaut handling.
3. Competitive products emphasize historical tracking; `auditmysite` currently focuses on point-in-time reporting.
4. Manual follow-up guidance exists, but it can be made much more explicit and actionable.
5. Executive benchmarking is available in batch mode, but not yet a first-class story for repeated audits over time.

## Product Direction

To beat typical competitor setups, `auditmysite` should aim for this combination:

- Browser-accurate audit data
- Stable automation contract
- Executive-ready PDF reporting
- Clear scoring semantics
- Lightweight installation and CI usage
- Practical remediation guidance with ownership and effort
