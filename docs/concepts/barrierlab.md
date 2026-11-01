---
title: "Shared libraries from barrierlab"
description: "Which checks live in barrierlab, how they come in, and where the boundary is."
order: 2
---

Parts of what auditmysite checks no longer live in this repository. They moved
to [barrierlab](https://github.com/casoon/barrierlab), a Rust monorepo of
accessibility and web-conformance libraries, so that the same checks, rule IDs
and wording serve auditmysite, [astro-post-audit](https://github.com/casoon/astro-post-audit)
and [liveaudit](https://github.com/casoon/liveaudit) alike. auditmysite pulls
them in as published crates.io versions:

| Crate | Provides |
|---|---|
| `a11y-rules` | the shared WCAG rules (lists, headings, landmarks, IDs, language, …) |
| `a11y-dom` | the document model those rules run on |
| `a11y-report` | the finding, outcome and rule-run model |
| `accname` | accessible name computation (accname 1.2, HTML-AAM) |
| `a11y-perception` | accessibility tree, snapshot, reading-order projection and snapshot diff |
| `web-checks` | robots.txt rules and bot classification, meta lengths, OpenGraph and JSON-LD structured-data checks |
| `html-conform` | HTML5 conformance checking, comparable to the W3C validator |

This is a real dependency: a fix or a new rule in one of these areas is made in
barrierlab, released there, and then taken over here by raising the version.
auditmysite does not patch or fork them. What stays here is everything specific
to this tool: its own WCAG rules, the keyboard journeys, the Chrome/CDP
capture, scoring and taxonomy, the PDF report, the CLI and the report texts.
For work on both sides at once, point `[patch.crates-io]` in a local,
uncommitted `.cargo/config.toml` at a barrierlab checkout; commits always
build against the published versions.

More detail:
- Current implementation: [docs/ARCHITECTURE.md](https://github.com/casoon/auditmysite/blob/main/docs/ARCHITECTURE.md)
- Browser dependency details: [docs/chrome-dependency.md](https://github.com/casoon/auditmysite/blob/main/docs/chrome-dependency.md)
- Troubleshooting: [docs/TROUBLESHOOTING.md](https://github.com/casoon/auditmysite/blob/main/docs/TROUBLESHOOTING.md)
