---
title: Display modes
description: Detect the data-display convention and audit a site in its calm, text or visual mode, with one report per mode and never a blended score.
order: 4
---

Sites can offer display modes so that nobody depends on 3D, animation or visualisations. The
BarrierLab `data-display` convention (draft v0):

- sets `<html data-display="visual|calm|text">` before the first paint,
- stores the visitor's choice in `localStorage` under the key `display`,
- marks each visualisation as `figure[data-viz="chart|diagram|3d|image|interactive"]` with a text
  layer `[data-viz-text]`,
- puts a `[data-display-toggle]` control on every page with a visualisation.

## Detection

Always on. Every page that sets `html[data-display]` or contains a `figure[data-viz]` reports
`pages[].display_modes` in the JSON: the offered modes, the mode the page rendered in, whether the
attribute was set before `<body>`, the toggle, and the visualisations per kind. The PDF shows it
as "Page display modes: visual · calm · text".

## Auditing one mode

```sh
auditmysite --sitemap https://example.com/sitemap.xml --display calm -f pdf -o reports/example-calm.pdf
```

`--display calm|text|visual` stores the choice in `localStorage.display` before navigation.
`calm` and `text` also emulate `prefers-reduced-motion: reduce`, so sites without the convention
that honour the media query get their reduced variant too. Without the flag the site default is
audited, as before.

`audit_scope.display_mode` in the JSON (`site_default`, `visual`, `calm`, `text`) and the PDF
name the mode the scores belong to.

## Auditing every mode

```sh
auditmysite https://example.com/page --display all -o reports/example-page.pdf
```

`--display all` audits each mode as its own run and writes one report per mode
(`report-calm.pdf`, `report-text.pdf`, `report-visual.pdf`; for a batch, one batch report per
mode). There is never a blended score.

- `visual` runs only for pages with `figure[data-viz="3d|interactive"]`, with twice the page
  timeout.
- Findings are not cross-marked as "occurs in all modes"; compare the per-mode reports.

## Convention checks

The `display/*` rules are best practice, not WCAG requirements; they are anchored to 2.2.2 or
1.1.1.

| Rule | Reports |
| --- | --- |
| `display/toggle-missing` | Visualisations without a `[data-display-toggle]` |
| `display/init-missing` | `data-display` missing, or set only after `<body>` started |
| `display/text-media-visible` | In `text` mode a visualisation still shows canvas, SVG, video or a static picture |
| `display/text-not-visible` | In `text` mode a visualisation has no visible, non-empty `[data-viz-text]` |
| `display/text-hidden` | `[data-viz-text]` removed from assistive technology by `hidden`, `aria-hidden`, `inert` or CSS, in any mode |

The text-mode checks key on the mode the page actually rendered in.
