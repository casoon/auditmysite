---
title: Architecture
description: How an audit runs, from launching Chrome to the written report, and which parts come from the shared barrierlab libraries.
order: 1
---

## The pipeline

```text
CLI → Browser manager → Chrome (CDP) → Accessibility tree → WCAG engine → Report output
```

1. **Browser.** auditmysite starts Chrome or Chromium headless as a child process and talks to
   it over the Chrome DevTools Protocol, a WebSocket on localhost.
2. **Capture.** The page loads in a tab. Capture waits within a bounded stability budget and
   records whether the DOM became quiet, a ready signal was seen or the budget ran out. Then,
   bounded by a budget of the same size, it waits for finite CSS animations and transitions to
   end, so target sizes are measured on the settled layout; a target still animating is
   reported as not measured. Consent banners are detected and reported; `--dismiss-consent`
   tries to dismiss them first.
3. **Accessibility tree.** Chrome's native accessibility tree is read together with computed
   styles. That is what makes JavaScript-rendered content, contrast after the cascade and the
   browser's own accessible names available.
4. **Rules.** The WCAG engine runs the rules on the tree. The interactive journey layer then
   drives the page with the keyboard: tab order, skip links, modal focus traps, disclosure
   widgets, form errors (`--interactive off|basic|full`, default `full`).
5. **Modules.** Performance (lab data from the same Chrome), SEO, security headers, mobile, HTML
   conformance and heuristic indicators run on the same capture.
6. **Normalize and score.** Results become one normalized report: scores, counts, verdict,
   audit quality. Risk is computed separately from the score.
7. **Output.** JSON, PDF, table, `ai`, `summary` or `sarif`. Files are written atomically, so a
   partial file is never presented as a result.

Each audited request carries `DNT: 1` and `Sec-GPC: 1`, so an audit visit is less likely to show
up in the site owner's analytics.

## Shared libraries from barrierlab

Part of what auditmysite checks lives in [barrierlab](https://github.com/casoon/barrierlab), a Rust
monorepo of accessibility and web-conformance libraries. The same checks, rule IDs and wording
serve auditmysite, [astro-post-audit](https://github.com/casoon/astro-post-audit) and
[liveaudit](https://github.com/casoon/liveaudit). auditmysite uses the published crates.io
versions:

| Crate | Provides |
| --- | --- |
| `a11y-rules` | the shared WCAG rules (lists, headings, landmarks, IDs, language, …) |
| `a11y-dom` | the document model those rules run on |
| `a11y-report` | the finding, outcome and rule-run model |
| `accname` | accessible name computation (accname 1.2, HTML-AAM) |
| `a11y-perception` | accessibility tree, snapshot, reading-order projection and snapshot diff |
| `web-checks` | robots.txt rules and bot classification, meta lengths, OpenGraph and JSON-LD checks |
| `html-conform` | HTML5 conformance checking |

A fix or a new rule in these areas is made in barrierlab, released there and taken over here by
raising the version. What stays in auditmysite: its own WCAG rules, the keyboard journeys, the
Chrome capture, scoring and taxonomy, the PDF report, the CLI and the report texts.

## In the repository

- [docs/ARCHITECTURE.md](https://github.com/casoon/auditmysite/blob/main/docs/ARCHITECTURE.md) — module-by-module description of `src/`
- [docs/chrome-dependency.md](https://github.com/casoon/auditmysite/blob/main/docs/chrome-dependency.md) — why Chrome, and how it is launched (German)
- [docs/PARITY_CONTRACT.md](https://github.com/casoon/auditmysite/blob/main/docs/PARITY_CONTRACT.md) — WCAG coverage numbers and the comparison with axe-core and pa11y
