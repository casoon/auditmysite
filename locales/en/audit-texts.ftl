# Texts of the audit (analysis) layer, #936: the SEO heading-issue technical
# impacts (`audit::normalized::seo_heading_finding_text`), the 3.2.6 assessment
# basis, region names and deviation sentences (`audit::batch_consistency`) and
# the management risk dimension names and rationales (`audit::management_risk`).
# The functions stay pure `*_text(kind, values, en)` text functions: the
# analysis stores canonical English in the JSON (`en = true`), the PDF calls
# them with the run language (#406). Which message applies stays in Rust. The
# heading-issue titles are the titles of the `explanation-seo-headings-*`
# messages of rules.ftl, identical in both languages.

## Heading issues: technical impact per issue type
audit-heading-impact-skipped-level = Skipped heading levels leave gaps in the outline that screen reader users and SEO crawlers rely on — a structure hint, not a WCAG failure on its own; a logical H1→H2→H3 hierarchy is recommended.
audit-heading-impact-missing-h1 = Missing H1 heading — page purpose not recognizable for search engines and screen readers.
audit-heading-impact-multiple-h1 = Multiple H1 headings are allowed, but can blur the content hierarchy when it is unclear which one names the page topic — a best-practice hint, not a WCAG failure on its own.
audit-heading-impact-long-heading = Overly long headings are truncated in SERPs and make quick scanning harder for users.
audit-heading-impact-empty-heading = Empty headings cause navigation problems for screen readers and are treated as a poor signal by SEO crawlers.

## Shared
audit-join-and = { " and " }
audit-not-measured = not measured

## 3.2.6 Consistent Help: assessment basis, region names, deviations
audit-help-mechanisms =
    { $count ->
        [one] 1 recurring help mechanism
       *[other] { $count } recurring help mechanisms
    }
audit-help-deviations =
    { $count ->
        [one] 1 deviation
       *[other] { $count } deviations
    }
audit-help-pages =
    { $count ->
        [one] 1 page
       *[other] { $count } pages
    }
audit-help-basis-manual-review = No help mechanism (contact or help link, e-mail, phone, known chat widget) recurs outside the main content on two or more audited pages; other forms of help must be checked manually.
audit-help-basis-consistent = { $mechanisms } compared across the audited pages by landmark region and relative order; no deviation found.
audit-help-basis-deviations = { $mechanisms } compared by landmark region and relative order; { $deviations } on { $pages }.
audit-help-region-header = header
audit-help-region-navigation = navigation
audit-help-region-complementary = sidebar
audit-help-region-footer = footer
audit-help-region-floating = floating overlay
audit-help-region-other = page body outside the main content
audit-help-deviation-region = '{ $mechanism }' sits in the { $found } instead of the { $expected } used on most pages.
audit-help-deviation-order = In the { $region }, '{ $second }' comes before '{ $first }'; most pages use the reverse order.

## Management risk: dimension names
audit-risk-dimension-legal = Legal / BFSG-EAA
audit-risk-dimension-assistive = Assistive technology use
audit-risk-dimension-conversion = Conversion / usability
audit-risk-dimension-seo = SEO / visibility
audit-risk-dimension-performance-mobile = Performance & mobile
audit-risk-dimension-trust = Trust / brand
audit-risk-dimension-project = Project risk

## Management risk: blocking controls
audit-controls-unnamed =
    { $count ->
        [one] 1 control without an accessible name or role
       *[other] { $count } controls without an accessible name or role
    }
audit-controls-keyboard =
    { $count ->
        [one] 1 control not operable by keyboard
       *[other] { $count } controls not operable by keyboard
    }
audit-controls-consequence-unnamed = Screen readers cannot convey their purpose; no keyboard blocker was detected at those points.
audit-controls-consequence-keyboard = Keyboard users cannot reach or trigger them.
audit-controls-consequence-both = Screen readers cannot convey the purpose of the unnamed ones; keyboard users cannot reach or trigger the others.

## Management risk: rationales
audit-risk-legal-base =
    { $count ->
        [one] 1 finding
       *[other] { $count } findings
    } with possible legal relevance, { $critical } critical and { $high } high WCAG findings detected automatically.
audit-risk-legal-controls = { $base } In addition: { $controls } (WCAG 4.1.2/2.1.1, also referenced by BFSG/EAA).
audit-risk-assistive-none = No blocking barrier for screen reader or keyboard use detected within the automated scope.
audit-risk-assistive-findings = { $critical } critical and { $high } high findings.
audit-risk-assistive-blocking = { $critical } critical and { $high } high findings; { $controls }. { $consequence }
audit-risk-conversion-blocking = { $controls }: tasks routed through them are harder to complete with assistive technology. Whether a task is fully blocked was not tested.
audit-risk-seo-findings =
    { $count ->
        [one] 1 SEO finding
       *[other] { $count } SEO findings
    } of medium or higher severity (SEO score { $score }).
audit-risk-performance-mobile-none = Neither the performance nor the mobile module was run.
audit-risk-performance-mobile-base = Performance { $performance }, mobile usability { $mobile } (lab measurement).
audit-risk-performance-mobile-high =
    { $base } { $count ->
        [one] 1 high-severity finding
       *[other] { $count } high-severity findings
    } in these modules.
audit-risk-trust = Inferred from accessibility only: score { $accessibility }/100 with { $critical } critical and { $high } high findings. No trust or brand signal is measured directly.
audit-risk-project-recurring =
    { $count ->
        [one] 1 rule recurs
       *[other] { $count } rules recur
    } on two or more of the { $pages } audited pages — likely component or template issues that can be fixed together. { $severe ->
        [0] No rule combines high severity with reach to at least half of the pages, so recurrence alone does not make this a high risk.
        [one] 1 of them is of high severity and reaches at least half of the pages, which makes it a high risk.
       *[other] { $severe } of them are of high severity and reach at least half of the pages, which makes it a high risk.
    }
audit-risk-project-single =
    { $count ->
        [one] 1 finding affects 10 or more elements on this page or needs a structural fix.
       *[other] { $count } findings affect 10 or more elements on this page or need a structural fix.
    } { $severe ->
        [0] No finding is of high severity.
        [one] 1 of them is of high severity, which makes it a high risk.
       *[other] { $severe } of them are of high severity, which makes it a high risk.
    } A single page cannot show whether a template or component is the cause — audit further pages to confirm.
