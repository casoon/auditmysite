# Texts of the single report's action plan block
# (`output::builder::single::actions_block`, `actions-block-*`), #936: the
# systemic and local column labels and descriptions, the block title and intro
# for a good site and for the default case, and the risk effect of an action
# per priority. Which message applies and the thresholds stay in Rust. The
# remediation leverage tiers are `pdf-findings-code-very-high`,
# `severity-high` and `severity-low`; the role, priority, execution priority
# and effort labels are messages of their own enums.

## Roadmap columns
actions-block-systemic-label = Systemic actions
# Reworded from an assertion into a hypothesis: this report audits exactly one
# page, so "affects all pages" is never observed; the verification task is part
# of the text (#575).
actions-block-systemic-desc = Fix once in the template or component — the pattern suggests other pages using it are affected too. Note: verify on additional page types whether the same pattern also occurs there.
actions-block-local-label = Local actions & individual cases
actions-block-local-desc = Targeted corrections on individual pages, images or editorial content.

## Block title and intro
actions-block-title-good = Last optimization steps
actions-block-title-default = Action plan by priority
actions-block-intro-good = The site is technically well positioned. The following are final optimization levers without structural pressure.
actions-block-intro-default = Priority 1 issues carry the highest risk and should be addressed first. Priority 2 and 3 items follow. Priority 4 items are recommended improvements.

## Risk effect per priority
actions-block-risk-critical = Directly reduces critical WCAG violation risk
actions-block-risk-high = Reduces high accessibility risk
actions-block-risk-medium = Lowers medium accessibility risk
actions-block-risk-low = Improves WCAG conformance in detail
