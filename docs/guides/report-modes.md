---
title: "Report modes"
description: "Single-page versus sitemap/batch reports, and the report levels."
order: 9
---

Single-page reports and sitemap/batch reports are intentionally different.

**Single-page report** is a product-grade PDF organized as a top-down narrative:
- Cover: a composed dashboard — dominant overall score with a score-band label (no A–F grade, no "/100"), a module gauge strip, and the WCAG findings scope.
- Management view: severity counters, a "quality profile" spider radar, a score-driver table (which weighted module pulls the overall score down most), Accessibility and Security subcategory breakdowns, a one-line problem profile (isolated, concentrated in a few systemic patterns, or broad and systemic), and strengths / optimization cards.
- Findings overview: a compact, priority-sorted findings matrix precedes the detailed finding cards.
- Manual review: a fixed list of WCAG criteria that are structurally outside automated testing, and concrete VoiceOver/NVDA self-test steps where a quick manual check is realistic.
- Accessible output: PDFs are tagged and PDF/UA-1 conformant (structure tree, document language, title, bookmarks); rendering fails with a diagnostic instead of silently producing a non-conformant PDF.
- Module chapters: each module is its own chapter with a magazine-style opener and a one-line key takeaway. AI Visibility, Content Visibility, and Source Quality are merged into a single "KI & Vertrauen" (AI & Trust) chapter.
- Action plan: recommendations as action cards grouped by where the problem lives (systemic vs. local), without time or effort estimates, plus a root-cause distribution chart.
- Evidence-grade findings: each finding card can include a cropped, highlighted screenshot of the affected element, its DOM path, and (where applicable, e.g. contrast) the measured vs. required value — so a finding stands on its own without re-running the tool.
- Audit coverage: requested and completed checks, partial measurements, manual-review items, and Journey execution coverage are surfaced explicitly instead of treating missing data as a pass.
- Performance decisions: raw resource and loading metrics are paired with target ranges, the largest directly actionable lever, and prioritized actions.

The design follows a consistent four-color status system; reports use no emoji and report effort by priority rather than by time windows.

**Sitemap/batch report** is aggregated and domain-wide: averages, ranking, recurring issues, URL matrix, near-duplicate content, broken links, crawl diagnostics. It also verifies which recurring findings share the same underlying template component across pages — reporting "one fix resolves N pages" instead of N near-identical findings, with a confirmed/likely confidence distinction so the claim is never overstated. Template clusters require at least three affected pages and 60% site coverage, can identify selector-less document findings, and retain header/nav/main/footer context. Cross-page WCAG assessments for consistent navigation, identification, help, and multiple ways are kept separate from single-page automation and explicitly mark evidence gaps as manual review.

Batch reports are not a stack of single-page reports.
