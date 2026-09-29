---
title: Excluding specimens
description: Leave markup that is broken on purpose, such as teaching examples, out of the findings with data-audit-exclude or --exclude-selector. Every exclusion is listed in the report.
order: 5
---

Sites that teach accessibility ship examples that are broken on purpose. Like axe-core's
`exclude`, auditmysite can leave such regions out of the findings — never a page, never a rule.

## data-audit-exclude

Always honoured. Put it on an element and its whole subtree is excluded:

```html
<section data-audit-exclude>…specimen…</section>
```

## --exclude-selector

```sh
auditmysite https://example.com/lesson/ --exclude-selector '[data-specimen]'
```

Repeatable. Excludes the subtree of every element the selector matches. The same list can live in
`auditmysite.toml`:

```toml
[audit]
exclude_selectors = ["[data-specimen]"]
```

## What is dropped

The page is still audited in full. Only findings whose element lies inside an excluded subtree are
dropped before scoring: WCAG violations and warnings, pattern findings, and journey findings that
name their element.

- Page-level findings are never excluded.
- A finding located only by a selector is dropped only when every element that selector matches
  lies inside an excluded subtree.

## Visible in the report

Excluding never happens silently. The JSON lists, per page, every applied selector with the number
of elements it matched — including `0` and invalid selectors — and how many finding occurrences
were dropped, per rule (`pages[].exclusions`; batch totals in `summary.exclusions`). The PDF names
the selectors and counts in the methodology section; a batch report shows them in the audit frame
on the cover.
