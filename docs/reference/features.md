---
title: "Feature scope"
description: "WCAG rules, additional modules, journeys, configuration and the deliberate non-goals."
order: 3
---

## WCAG rules (Level A and AA)

Core rules:
- Non-text content (1.1.1)
- Keyboard access (2.1.1)
- Bypass blocks (2.4.1)
- Language of page (3.1.1)
- Name, role, value / form labeling (4.1.2)
- Contrast minimum (1.4.3) and non-text contrast (1.4.11)
- Headings and labels (2.4.6)
- Labels or instructions (3.3.2)
- Focus order (2.4.3), focus visible (2.4.7), and focus not obscured, minimum/enhanced (2.4.11/2.4.12, WCAG 2.2)
- Label in name (2.5.3)
- Meaningful sequence — CSS `order` vs. reading-order mismatches (1.3.2)
- Pause, stop, hide — `<marquee>` and long-running CSS animations without a pause control (2.2.2)
- Redundant entry — same field requested twice with no reuse/autofill hint (3.3.7, WCAG 2.2)
- Accessible authentication — paste blocked on password or one-time-code fields, measured with a synthetic paste event (3.3.8, WCAG 2.2); CAPTCHAs in sign-in forms as review items
- Target size minimum (2.5.8, WCAG 2.2) and text spacing (1.4.12)

ARIA and semantics:
- ARIA role validation — invalid roles, required owned elements, required context
- ARIA attribute checks — allowed attributes per role, required attributes, prohibited attributes
- Accessible name checks — icon-only controls, empty aria-labelledby/describedby, name/description conflicts, naming by role type (command, input, meter, progressbar, toggle, dialog, treeitem)
- ARIA relationship checks — aria-controls, aria-owns, aria-activedescendant, duplicate IDs
- Landmark structure — main, navigation, banner, contentinfo (presence, uniqueness, top-level nesting, no-duplicate for banner/contentinfo/main, required parent for landmarks)
- Content in landmarks — region rule ensuring body content lives inside landmark regions
- Table rules — caption/name, header cells, presentational tables, cell placement
- Form rules — fieldset/legend for grouped controls, required field indication, error description, label-title-only detection
- List structure — listitem context, empty lists, definition list integrity
- Dialog rules — accessible name, aria-modal, alert region labeling
- Widget rules — tab/tabpanel pairing, selected state, combobox options, slider value, tree context, summary element naming
- Media rules — application and image-role elements without accessible names
- Video checks — caption tracks for native `<video>` (a same-origin track file is probed before a pass is confirmed, 1.2.2), media alternatives with a nearby-transcript heuristic (1.2.8), and keyboard operability of native video controls (unnamed or unreachable players)
- ARIA hygiene (best practice, low severity) — explicit roles that restate the element's implicit role, and links without a real target (`href="#"`, `javascript:`) used as button substitutes
- Duplicated accessible names — names that repeat themselves back to back (e.g. "Contact Contact"), typically from an icon label concatenated with adjacent text
- Frame and iframe rules — accessible names on all frames (`frame-title`), manual-review notices for cross-origin frames (`frame-tested`), and a full WCAG content scan inside same-origin iframes: image-alt (1.1.1), button-name (4.1.2), link-name (2.4.4), form labels (1.3.1), referenced duplicate IDs (4.1.2), document language attribute (3.1.1)
- SVG rules — SVG image accessible names
- Server-side image maps — detection and flagging
- Meta viewport — large maximum-scale restrictions

100+ rules with stable `rule_id`, `tags` (e.g. `wcag2a`, `wcag412`, `cat.aria`), and an `impact` field (`critical` / `serious` / `moderate` / `minor`).

Methodology numbers are frozen in `docs/PARITY_CONTRACT.jsonc` and guarded by `tests/parity_contract.rs`: WCAG 2.2 AA has 55 A/AA criteria, 30 are covered by automated AuditMySite checks (a criterion counts only if its rule can report a violation), and 25 are listed as manual-review criteria.

Some criteria (keyboard trap behavior, timed content, captions) cannot be reliably verified by automated means. These are flagged as `not_testable` in the JSON output and listed in the report's audit scope section as requiring manual review.

AAA is not fully implemented yet.

## Additional modules

Modules are classified by how their result is obtained: **compliance** (Accessibility), **measured** (based on real browser data), **composite** (the search-experience roll-up, which blends measured SEO with heuristic sub-scores), **heuristic** (structural-signal estimates, marked with `~` in reports), and **optional** (Dark Mode — reported as a design choice, not a compliance gap).

Measured:
- Performance: Core Web Vitals (FCP, LCP, TBT, CLS), throttled profiles, DOM/load targets, render-blocking and third-party resources, critical request chains, unused code, minification potential, JavaScript heap, and modeled transfer emissions
- SEO: meta tags, headings, structured data, page-to-schema fit, content profile, tracking/external services signals, and social-preview images (`og:image`/`twitter:image`) without an alt description
- Security: HTTPS, header checks, and CDN/WAF protection detection. Headers are grouped by risk tier: CSP, HSTS, and clickjacking protection are baseline requirements, while context-dependent headers such as COOP/CORP get a verification question and safe-configuration guidance instead of an unconditional "add this header"
- Mobile: viewport, touch-target, readability checks, UX heuristics (cookie-banner, modal/overlay, CTA detection)
- HTML5 conformance: spec-conformance checking via the `html-conform` crate, part of `--full`; scored per distinct defect cause rather than per raw occurrence, so one templated markup mistake rendered many times doesn't get charged once per render
- DNS configuration (opt-in `--dns-check`): CAA, best-effort DNSSEC, and SPF when an MX record exists; score-neutral

Heuristic (indicator scores — tendency, not measurements). In 2.0 none of these signals is validated against an independent, human-labelled reference set (`signal_contract.validation.status = "unvalidated"` in the JSON, procedure in `docs/calibration.md`); their scores order pages by the stated method and are not evidence of visibility, AI citation, user quality or source truth:
- UX: 5-dimension analysis (CTA clarity, visual hierarchy, content clarity, trust signals, cognitive load) with saturation curve scoring
- Journey: user-flow analysis (entry clarity, orientation, navigation, interaction, conversion) with page-intent-aware weighting
- AI Visibility: structural signals for machine readability and indexing (readability, structured data, AI policy, chunk quality). Unvalidated: the score is not evidence of AI citation, visibility or content quality
- Source Quality: code hygiene signals (inline styles, deprecated elements, semantic structure, asset hygiene)
- Dark Mode: detects dark mode support via `prefers-color-scheme` media queries and CSS custom properties
- Easy language: recognizes an easy-language ("Leichte Sprache") or plain-language version of the page via class names, a `lang` variant, or a link-text marker, and reports it as a positive signal
- Tech Stack: detects CMS and frameworks (WordPress, Drupal, Joomla, Next.js, Astro, React, Vue, etc.) via in-page signals and runs stack-specific security probes (admin panel exposure, user enumeration, version disclosure)
- Commerce: shop audit that only activates when a page is detected as a store (schema-gated). Checks product structured-data completeness, presence of mandatory and trust pages (imprint, returns, shipping, payment), coarse page-kind classification (product detail, category), and rolls findings up across a batch. Derive-only — no extra browser interaction. Product-detail pages also get two commerce-aware interactive journeys — see Accessibility Journey Layer below.

## Display modes

Sites can offer display modes so that nobody depends on 3D, animation or visualisations. The BarrierLab `data-display` convention (draft v0) sets `<html data-display="visual|calm|text">` before the first paint, stores the visitor's choice in `localStorage` under the key `display`, marks each visualisation as `figure[data-viz="chart|diagram|3d|image|interactive"]` with a text layer `[data-viz-text]`, and puts a `[data-display-toggle]` control on every page with a visualisation.

- **Detection (always on):** every page that sets `html[data-display]` or contains a `figure[data-viz]` reports `pages[].display_modes` in the JSON — offered modes, the mode it rendered in, whether the attribute was set before `<body>`, the toggle, and the visualisations per kind. The PDF shows it as "Page display modes: visual · calm · text".
- **`--display calm|text|visual`:** before navigation the choice is stored in `localStorage.display` (`Page.addScriptToEvaluateOnNewDocument`), and `calm`/`text` also emulate `prefers-reduced-motion: reduce` (`Emulation.setEmulatedMedia`) — so sites without the convention that honour the media query get their reduced variant too. Without the flag the site default is audited, as before. `audit_scope.display_mode` in the JSON (`site_default`, `visual`, `calm`, `text`) and the PDF name the mode the scores belong to.
- **`--display all`:** audits each mode as its own run and writes one report per mode (`report-calm.pdf`, `report-text.pdf`, `report-visual.pdf`; batches: one batch report per mode). There is never a blended score. `visual` runs only for pages with `figure[data-viz="3d|interactive"]`, with twice the page timeout. Findings are not cross-marked as "occurs in all modes" — compare the per-mode reports.
- **Convention checks `display/*` and `viz/*`** (best-practice, not WCAG requirements; anchored to 2.2.2, 1.1.1 or 1.3.1). From the markup, shared with `a11y-rules`: `display/toggle-missing` (visualisations without a toggle), `viz/text-missing` (no or an empty `[data-viz-text]`), `viz/caption-missing` (no `<figcaption>`), `viz/static-missing` (`data-viz="3d"`/`"interactive"` without `[data-viz-static]`), `viz/table-missing` (a chart without a `<table>`; a review note). Measured on the running page: `display/init-missing` (`data-display` missing or set only after `<body>` started, measured by an observer injected before navigation), `display/text-media-visible` (in `text` mode a visualisation still shows canvas/SVG/video/static picture), `display/text-not-visible` (in `text` mode a visualisation's `[data-viz-text]` has content but is not visible), `display/text-hidden` (`[data-viz-text]` removed from assistive technology by `hidden`, `aria-hidden`, `inert` or CSS, in any mode). The text-mode checks key on the mode the page actually rendered in.

```bash
auditmysite --sitemap https://example.com/sitemap.xml --display calm --format pdf --output reports/example-calm.pdf
auditmysite https://example.com/page --display all --output reports/example-page.pdf
```

## Structured-data analysis

The SEO module parses JSON-LD objects, arrays, and `@graph` documents; normalizes short, multiple, and full-IRI `@type` values; and reports invalid JSON, missing or invalid context, and untyped nodes. Microdata and RDFa are detected and explicitly marked as detected but not content-validated.

Type-specific rules assess Product/Product Snippet, merchant Product + Offer, Article/BlogPosting/NewsArticle, BreadcrumbList, Organization, LocalBusiness, FAQPage, Event, Recipe, VideoObject, JobPosting, SoftwareApplication/WebApplication/MobileApplication, ProfilePage, CollectionPage/ItemList, WebPage/WebSite, and Person. Each profile records its source and review date. Eligibility blockers, recommendations, and manual checks remain separate; unknown types stay visible in the inventory without being judged incomplete.

Page-to-schema fit is evaluated conservatively from visible page intent, visible facts, and URL evidence. The tool distinguishes product, service/software, job, event, FAQ, person, location, editorial-review, corporate, hub, and lead-generation pages. Missing primary schema is only reported as an opportunity at high classification confidence, and single-item Product, JobPosting, or Event markup is rejected on corresponding overview routes.

For supported types, visible titles, prices, availability, authors, dates, FAQ content, breadcrumbs, job titles, and event dates are compared with JSON-LD. A hard mismatch is emitted only when the visible value is unambiguous; otherwise the result explicitly remains not evaluated or requires manual review. Batch reports additionally show recurring schema blockers, page-type/schema combinations, content-parity mismatches, and conflicting Organization/WebSite identities.

## Runtime and evidence reliability

Page capture uses a bounded stability budget and records whether the DOM became quiet, an application-provided ready signal was observed, or the budget expired. Consent handling reports detected, dismissed, failed, and unknown states with non-sensitive evidence. Ctrl-C and SIGTERM follow the same controlled shutdown path as normal runs, and report files are written atomically so partial files are not presented as successful output.

Every audited page receives Do-Not-Track and Global Privacy Control signals (`DNT: 1` and `Sec-GPC: 1` on all requests, plus the matching `navigator.doNotTrack`/`navigator.globalPrivacyControl` values), so an audit visit is less likely to show up in the site owner's analytics. This is best-effort: not every analytics tool honors these signals.

Sitemap indexes are deduplicated and guarded against cycles, with hard limits of 1,000 sitemap documents and 100,000 discovered URLs. Batch aggregation stays bounded and publishes its atomic report only after collection succeeds.

## Accessibility Journey Layer

Interactive checks run a real browser session after the static AXTree phase. They run in `full` mode by default and can be reduced via `--interactive <off|basic|full>` or `mode` in `auditmysite.toml`.

| Mode | What runs |
|------|-----------|
| `off` | No interactive phase — fastest, no browser interaction after initial load |
| `basic` | Tab-walk (focus order, reverse jumps), skip-link verification, disclosure/accordion, modal focus trap, tab-list, menu journey |
| `full` (default) | Everything in `basic`, plus: SPA-navigation detection, form-error announcement (now covering multiple independent forms per page, e.g. search + login + newsletter, and flagging live regions inserted only after submit, which not every browser/screen-reader combination announces), link-text inventory (generic/duplicate texts, heading outline, landmark structure) |

On a detected shop's product-detail page, `full` mode also runs two commerce-aware journeys: an **add-to-cart feedback check** (does adding an item announce the result via a live region or focus-managed dialog, or only update a visual cart badge — SC 4.1.3) and a **quantity-stepper operability check** (can the quantity field be operated by keyboard, and does its value stay exposed to assistive technology — SC 2.1.1/4.1.2). Both are click-only, single-interaction checks — never a real checkout submission, never a filled-in purchase form.

Results appear in `interactive_findings` and `accessibility_journey` in the JSON output. The execution block records detected, attempted, completed, failed, skipped, and budget-limited journeys separately from findings. Compact focus evidence retains visibility, viewport, focus-indicator, bounding-box, obscuring, `aria-hidden`, and `inert` signals without embedding a full AXTree. Interactive findings do not affect the accessibility score or `legal_flags`; critical interactive findings can raise the risk level.

**`auditmysite.toml` configuration:**

```toml
[interactive]
mode = "full"             # off | basic | full
journey_budget_ms = 8000  # wall-clock budget per URL in milliseconds (default: 6000)
```

## Risk assessment

Risk level is computed independently from the score. A page scoring 81 can still carry "Critical" risk if it has Level A violations relevant under BFSG/EAA. Risk levels: Low, Medium, High, Critical — based on critical/high violations, legal flags, and blocking issues (4.1.2/2.1.1).

## Configuration file

`auditmysite.toml` is an optional project-level config file placed in the working directory. It supports `[audit]`, `[rules]`, `[interactive]`, `[thresholds]`, and `[budget]` sections.

## Rule configuration

Rules can be selectively disabled or filtered via `auditmysite.toml`:

```toml
[rules]
disabled = ["heading-order", "landmark-one-main"]
# enabled_only = ["image-alt", "label"]  # run only these rules
```

## Excluding intentional specimens

Sites that teach accessibility ship examples that are broken on purpose. Like axe-core's
`exclude`, auditmysite can leave such regions out of the findings — never a page, never a rule:

- **`data-audit-exclude`** is always honoured. Put it on an element and its whole subtree is
  excluded: `<section data-audit-exclude>…specimen…</section>`.
- **`--exclude-selector <CSS>`** (repeatable) excludes the subtree of every element the selector
  matches, e.g. `--exclude-selector '[data-specimen]'`. The same list can live in
  `auditmysite.toml`:

  ```toml
  [audit]
  exclude_selectors = ["[data-specimen]"]
  ```

The page is still audited in full; only findings whose element lies inside an excluded subtree
are dropped before scoring (WCAG violations and warnings, pattern findings, and journey findings
that name their element). Page-level findings are never excluded, and a finding located only by
a selector is dropped only when every element that selector matches lies inside an excluded
subtree.

Excluding never happens silently. The JSON report lists, per page, every applied selector with
the number of elements it matched — including `0` and invalid selectors — and how many finding
occurrences were dropped, per rule (`pages[].exclusions`; batch totals in
`summary.exclusions`). The PDF names the selectors and counts in the methodology section (batch:
in the audit frame on the cover).

## AI / LLM output format

Export findings as a task-oriented JSON list for direct LLM processing:

```bash
auditmysite https://example.com -f ai -o findings.json
```

Each entry is a task object with `task_id`, `rule_id`, `impact`, `wcag`, `tags`, `title`, `issue`, `fix`, `selector`, `node_id`, and `help_url` — sorted by impact severity. Suitable for direct use as context in AI-assisted code remediation.

## Non-goals (deliberately deferred)

These are recognized as potentially useful but deliberately not on the roadmap right now —
each would pull the project away from its core shape (a stateless, single-run web-page auditor)
for a disproportionate amount of new surface area:

- **PDF document accessibility checking.** auditmysite audits rendered web pages via the browser's
  accessibility tree; it does not parse or score linked/embedded PDF documents (tagged-PDF
  structure, `/Lang`, alt text on figures, table header scope, etc.). Doing this properly would
  need a new PDF-parsing dependency and a second, PDF-specific SSRF-hardened fetch path
  (comparable in scope to the existing `ai_transparency` module) for a feature that only applies
  when a page happens to link PDFs. Out of scope for now; revisit only as a dedicated,
  separately-scoped module if real demand shows up.
- **Full audit-history / report-diffing as a first-class feature.** Findings carry stable ids
  (`findings[].id`, `findings[].patterns[].id`) that stay the same across runs while rule and
  element pattern stay the same, so two saved JSON reports can be compared by id. A built-in
  "compare this report against an older report" mode with its own CLI flag, PDF section,
  and batch-vs-single/methodology-consistency handling is not planned — that turns the tool from
  a stateless auditor into a report-history manager, a different product shape. A JSON-to-JSON
  diff against two saved reports is straightforward to script externally (e.g. with `jq`) without
  needing this built in.
