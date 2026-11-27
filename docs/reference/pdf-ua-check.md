---
title: "Accessibility of the report PDFs: PDF/UA check"
description: "What was checked on the generated report PDFs against PDF/UA-1, what passed, what is open and where it belongs (#1234)."
---

This is a measurement, not a conformance claim. AuditMySite does not claim that
its PDFs conform to PDF/UA. A validator run is one input; it does not replace
a manual check with a screen reader.

## Method

- Subjects: the twelve PDFs of the 2.3.0 smoke run (single report of four
  sites and the 5-URL batch of two sites, each in DE and EN). The PDFs were
  produced by 2.3.0 (renderreport 0.8.1, Typst 0.15.1); no site was re-audited.
- Validator: veraPDF 1.30.0 (installed locally, not downloaded for this check),
  profile `ua1` (PDF/UA-1, ISO 14289-1). The same files were also run against
  `ua2` and `wt1a` (PDF/UA-2 and WTPDF 1.0 accessibility) to see what the next
  profile would ask for.
- Structure dump: `qpdf --json` read with a small script (catalog, `/Info`,
  `/MarkInfo`, `/Lang`, viewer preferences, outlines, structure tree, link
  annotations); `pdffonts` for font embedding.
- Command to repeat the validator run:

  ```bash
  verapdf --flavour ua1 --format text report.pdf
  ```

## Result of the validator

| Profile | Result on all 12 PDFs |
| --- | --- |
| PDF/UA-1 (`ua1`) | pass: 106 rules, 0 failed checks (for example 164,646 checks passed on the 39-page single report) |
| PDF/UA-2 (`ua2`) | fail (9 rules) |
| WTPDF 1.0 accessibility (`wt1a`) | fail (8 rules) |

veraPDF passing PDF/UA-1 means the machine-checkable part of ISO 14289-1 is
met on these files. veraPDF cannot judge whether alternative texts are
meaningful, the reading order is sensible or headings are used correctly; those
parts of PDF/UA need a human. Do not read "veraPDF passes" as "conforms".

The `ua2` / `wt1a` failures are the same set on every file:

- file version must be 2.0 (Typst writes 1.7), `pdfuaid:part` / `pdfuaid:rev`
  and the WTPDF declaration are not in the XMP metadata;
- structure tree root must have a single `Document` child in the PDF 2.0
  namespace;
- every internal link and bookmark destination must be a structure destination;
- each `TOCI` needs a `Ref` entry;
- content-model rules from ISO 32005: `Document` contains `Span`, `P` contains
  `P` / `Div` (151 / 34 / 210 checks on the 39-page report).

None of these are decisions of this repository; they follow from what the Typst
version and renderreport emit.

## Checklist

Result per item: pass, fail, not checkable (with the available tools).

| Item | Result | Evidence |
| --- | --- | --- |
| Tagged PDF, `/MarkInfo /Marked true`, `/Suspects false` | pass | all 12 files |
| `StructTreeRoot` with `ParentTree` and `RoleMap` | pass | all 12 files |
| Document language `/Lang` | pass | `de` / `en` per `--lang` on all files |
| Whole-document language only; passages in another language | not checkable | no per-span `/Lang`; quoted page text in a report of another language is read with the document language |
| `/Title` present and `DisplayDocTitle true` | pass | title present on all files |
| `/Title` identifies the document | fail (weak) | the same generic text for every site ("Website Health Report", "Website-Qualitätsbericht"; batch: "Accessibility audit report"); no domain or date. Open on the renderreport side, see below |
| `/Author`, `/Subject` | fail (minor) | not set; the author passed by this crate (`metadata("author")`) is not written to the document info. renderreport side |
| Bookmarks (outlines) | pass | 6 top-level entries on the single report, nested to depth 4, every entry has a `/Dest`; the batch report has its sections and sub-sections |
| Heading tags | pass with a note | `H1`-`H4` present, no skipped level. There are several `H1` (one per main section) and no `H1` for the document title itself |
| Tables | pass | 29 tables in the 39-page single report, each with `THead`, `TR`, `TH` (`Scope /Column`), `TD`; no table without header cells |
| Row headers in tables | not checkable | only column header cells were found (95 `TH`, all `Scope /Column`); which components emit tables whose first column is a row header was not examined |
| Figures with alternative text | pass with a note | the two page screenshots are `Figure` with `/Alt` (2 of 2). Before this change the text was generic ("Desktop screenshot preview" in EN); it now names the audited page; a PDF rendered with the new text (URL with quotes, backslash, `#`, `$`, umlaut) still passes veraPDF `ua1` and carries the text in `/Alt` |
| Element crops of findings (evidence screenshots) | fixed here | rendered as decorative artifacts because no `alt` was passed; now carry an alt text naming selector and viewport. The smoke PDFs contain none (element evidence is off by default), so this is covered by a unit test on the component data, not by a validator run on a file with crops |
| Decoration as artifacts | pass | veraPDF PDF/UA-1 rule on tagged-or-artifact content passes; gauges and score strips carry their values as text |
| Custom cover logo | not checkable | decorative artifact, `CoverPage` has no alt setting; only relevant for `--cover-logo`, not in the smoke PDFs |
| Links | pass | every `Link` annotation (18 to 23 per single report, 4 per batch report) has `/Contents` and a `StructParent`, so it sits in the tree as `Link` |
| Fonts | pass | all embedded as subsets with a Unicode map (Arimo, JetBrains Mono) |
| Reading order | not checkable | the structure tree follows the Typst layout flow with no manual reordering; no tool here evaluates whether that flow is the sensible order. Needs a read with a screen reader or PAC |
| Contrast, colour-only meaning | not checked | not part of the validator profile; the report design is not measured by this check |

## Where each open item belongs

Fixable here and fixed in this change:

- Alternative text of the two page screenshots now names the audited page URL
  (`pdf-screenshot-alt-desktop` / `-mobile`, DE and EN).
- Evidence crops of findings now carry an alternative text
  (`pdf-findings-screenshot-alt-desktop` / `-mobile`); before, they were
  invisible to screen readers.

Not fixable here without a workaround; to be filed in the renderreport repository:

1. Document title: `#set document(title: ..)` takes `RenderRequest.title`, and
   with no title the cover page title. Setting `title` here would also add a
   built-in title page and change the running header (there is no pack
   template for these reports). renderreport needs a separate document-title
   (and author, subject, keywords) metadata key that touches neither. Wanted
   result: "Website Health Report - example.com - 2026-11-27" in the viewer's
   title bar.
2. `/Author` and the other document info fields from the request metadata.
3. A language setting for single passages (`#text(lang: ..)` spans) so that
   quoted page text in another language is tagged.
4. Row header cells (`TH` with `Scope /Row`) for key-value tables, if those are
   emitted as tables.
5. An alt setting for the cover logo.
6. PDF/UA-2 / WTPDF readiness, as far as Typst allows it: PDF 2.0 output,
   `pdfuaid` part 2 and revision in XMP, structure destinations for links and
   bookmarks, `TOCI` references and the `Document` / `Span` / `P` content-model
   rules listed above. This is not needed for PDF/UA-1 and depends on Typst.

## What stays unverified

- Conformance with PDF/UA-1 as a whole. Run veraPDF (or the PDF Accessibility
  Checker, PAC) on the PDF you deliver, and read it once with a screen reader
  before stating a conformance level to a client.
- Reading order, meaning of the alternative texts, row headers, language of
  quoted passages and contrast, as listed above.
- Only 2.3.0 smoke files were examined: four single-page reports and two
  batch reports. Other report levels (executive, technical) and the optional
  annexes were not part of the sample.
