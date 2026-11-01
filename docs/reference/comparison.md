---
title: "Compared to typical setups"
description: "How auditmysite differs from static scanners and from axe-core and Pa11y."
order: 4
---

- Better fit for JavaScript-heavy sites than static HTML-only checks
- Easier to distribute than a multi-package browser toolchain
- More automation-friendly than ad hoc console output because the JSON contract is explicit and tested
- Broader reporting surface than a pure accessibility-only checker when you also want performance, SEO, security, and mobile signals
- Violations carry stable `rule_id`, `tags`, and `impact` — easier to integrate with existing tooling or dashboards

## Screen Reader Audit vs. axe-core / Pa11y

Standard accessibility checkers verify individual rules in isolation. `auditmysite` additionally simulates the sequential experience of a screen reader user navigating the page — detecting problems that only emerge in context.

| Capability | axe-core | Pa11y | auditmysite |
|---|---|---|---|
| Rule-based WCAG checks | ✓ | ✓ | ✓ |
| Reading sequence simulation | — | — | ✓ |
| Out-of-context link text analysis (duplicate "Read more" × 8) | — | — | ✓ |
| Accessible name quality score (not just present/absent) | — | — | ✓ |
| Landmark navigation strategy (can a SR user reach main content?) | — | — | ✓ |
| BFSG / EN 301 549 legal mapping per finding | — | — | ✓ |

Every finding also carries a per-criterion EN 301 549 (chapter 9, "Web") clause reference. A structured version — all 50 WCAG 2.1 A/AA clauses split into "violations found", "no violations in the automated scope", or "manual review required", plus which chapters (5–8, 10–13) sit outside this tool's audit scope entirely — is always in the JSON (`en301549_annex`) and can be added to the PDF as an appendix with `--annex en301549`. This is explicitly **not** an accessibility statement and doesn't claim to be one — it's a technical building block for a human-authored one, with an explicit scope disclaimer in both languages.

When the screen reader module runs, a JSON sidecar is written automatically next to the primary report:

```
example-com-YYYY-MM-DD-single-report.pdf
example-com-YYYY-MM-DD-single-report-screen-reader-audit.json  ← automatic sidecar
```

The sidecar models what a screen reader would typically announce, node by node, based on rule-based conventions (name, role, and state) — not a verified 1:1 reproduction of what NVDA, JAWS, or VoiceOver actually announce, which can vary by assistive technology, browser, and locale. It also flags which announcements are ambiguous or missing, suitable as a developer reference and as supporting evidence for BFSG compliance audits. No extra flag is required; the file is created whenever screen reader data is available in the audit result.
