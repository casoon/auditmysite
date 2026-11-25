# Criteria of the shared rules

`a11y_rules::Meta.wcag` lists the *related* criteria of a rule (a set, no order, no level).
A finding carries one criterion, and the taxonomy scores by it, so `SHARED_RULES`
(`src/wcag/shared.rs`) names the one criterion and the level a finding is reported under.
`tests/rule_metadata_source.rs` (#1119) checks, for every shared id, that this criterion is
one of the criteria in `Meta.wcag` and that the taxonomy entry (`src/taxonomy/rules.rs`)
says the same criterion and level.

The picks that are not self-evident:

| Id | Reported under | `Meta.wcag` | Why |
| --- | --- | --- | --- |
| `headings/empty` | 2.4.6 | 1.3.1, 2.4.1, 2.4.6 | Empty headings were always reported under 2.4.6. |
| `landmarks/main-missing` | 2.4.1 | 1.3.1, 2.4.1 | The main landmark is the bypass target, as in axe `landmark-one-main`. |
| `ids/duplicate` | 4.1.2 | 4.1.2 | WCAG 2.2 dropped 4.1.1; a duplicate id only breaks something where an IDREF points to it (axe `duplicate-id-aria`). |
| `keyboard/hidden-focusable` | 4.1.2 | 1.3.1, 4.1.2 | Same as axe `aria-hidden-focus`. |
| `zoom/viewport-*` | 1.4.4 | 1.4.4 | All viewport findings sit together under 1.4.4. |
| `frames/name-missing` | 2.4.1 | 4.1.2 | **Deviation.** Reported under 2.4.1 since the rule `frame-title` it replaced; `Meta` moved to 4.1.2 only. |
| `images/server-side-map` | 1.1.1 | 2.1.1 | **Deviation.** Reported under 1.1.1 since the rule `server_side_image_map` it replaced. |
| `headings/h1-missing` | 1.3.1 | 1.3.1, 2.4.1, 2.4.6 | The finding names 1.3.1 but is a best practice; the taxonomy entry carries no criterion on purpose (#779). |

Deviations are pinned in the test together with the `Meta` value they deviate from; when
barrierlab changes that value the test fails and the deviation is decided again.

Differences in behaviour from the auditmysite rules the shared rules replaced (for example
multiple `h1` is a `REVIEW`, a missing navigation/banner/contentinfo landmark is a `REVIEW`,
`heading` without `aria-level` is no finding, no cap of 250 findings per rule) are in the
rule contracts (`tests/fixtures/rule_contracts/`) and in the git history of
`src/wcag/shared.rs` before #1119, which carried the adoption notes of each rule.
