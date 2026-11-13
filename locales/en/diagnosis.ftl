# Texts of the single report's findings overview and system diagnosis
# (`output::builder::single::diagnosis`) and of the diagnosis section of the
# single PDF (`output::pdf::diagnosis`), #936. The builders keep which
# message applies to an issue pattern, a criticality tier and a finding
# category. The severity labels reuse `severity-*` of report.ftl, the finding
# count `count-findings`, the table headings the `diagnosis-col-*` and
# `diagnosis-table-*` messages of report.ftl.

## Issue pattern (findings overview, dominant-issue spotlight)
diagnosis-pattern-minimal = No findings
diagnosis-pattern-single-dominant = Single dominant issue
diagnosis-pattern-clustered = Clustered problems
diagnosis-pattern-scattered = Scattered issues

## System diagnosis
diagnosis-section-title = System diagnosis
diagnosis-pattern-single-dominant-problem = Single dominant problem
diagnosis-pattern-distributed = Distributed individual issues
diagnosis-pattern-minimal-description = No measurable accessibility barriers detected.
diagnosis-pattern-single-dominant-description = One rule type accounts for the majority of all critical/high findings. Fixing this one root cause will have the largest single impact.
diagnosis-pattern-clustered-description = Issues are grouped in related problem areas. Addressing one cluster reduces several findings at once.
diagnosis-pattern-scattered-description = Issues are spread across many independent rules. No single root cause dominates — each finding requires individual attention.
diagnosis-dominant-spotlight = "{ $issue }" accounts for the majority of critical/high findings.

## Criticality tiers
diagnosis-tier-mandatory = Legally relevant barriers (WCAG A/AA)
diagnosis-tier-mandatory-eyebrow = SECTION 1 · WCAG COMPLIANCE
diagnosis-tier-mandatory-intro = Accessibility issues under WCAG Level A/AA. Remediating these issues is recommended for compliance with legal standards (like BFSG).
diagnosis-tier-optimization = Additional quality & SEO recommendations
diagnosis-tier-optimization-eyebrow = SECTION 2 · OPTIMIZATION
diagnosis-tier-optimization-intro = SEO, AI visibility, UX signals and further quality benchmarks. These improve findability and user experience.

## Thematic clusters ($findings: the finding count, `count-findings`)
diagnosis-cluster-accessibility = Accessibility barriers ({ $findings })
diagnosis-cluster-seo = SEO issues ({ $findings })
diagnosis-cluster-performance = Performance issues ({ $findings })
diagnosis-cluster-security = Security gaps ({ $findings })
diagnosis-cluster-mobile = Mobile issues ({ $findings })
diagnosis-cluster-other = { $category } ({ $findings })
