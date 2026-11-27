# Texts of the interpretation layer, #936 (slice 27): the SEO, performance and
# score interpretation sentences (`seo::interpretation`,
# `audit::performance_interpretation`, `audit::interpretation`), the verdict
# reasons (`audit::verdict`), the complexity and expected-impact sentences of a
# finding (`audit::finding_derive`), the audit summary narrative
# (`audit::summary`), the long-heading message (`seo::headings`), the baseline
# line (`audit::baseline`) and the score area labels (`taxonomy::score_area`).
# The functions stay pure `*_text(kind, values, en)` text functions: the
# analysis stores canonical English in the JSON (`en = true`), the PDF calls
# them with the run language (#406). Which message applies, the thresholds and
# the number notation stay in Rust. Measured values come in as arguments
# already formatted (`$share`, `$ms`, `$kb`, `$dom`); `$cross` is the
# cross-impact note with its leading space, or empty. The word "and" of the
# phrase lists is `audit-join-and` of audit-texts.ftl, identical in both
# languages.

## SEO interpretation (`seo::interpretation`)
audit-interpretation-seo-lead-excellent = The technical SEO foundations are largely in place and support visibility in search engines.
audit-interpretation-seo-lead-good = Good SEO base with targeted potential to improve visibility.
audit-interpretation-seo-lead-partial = Some important SEO foundations are missing, which can limit visibility in search engines.
audit-interpretation-seo-lead-insufficient = SEO insufficient — essential foundations are missing and noticeably limit discoverability.
audit-interpretation-seo-lead-critical = SEO critical — fundamental prerequisites are missing; indexing and discoverability are at risk.
audit-interpretation-seo-page-type = Classified as “{ $page_type }”.
audit-interpretation-seo-depth-note = Technical SEO complete — content depth ({ $depth }/100) still has room to grow.
audit-interpretation-seo-quality-coherent = very coherently structured
audit-interpretation-seo-quality-supported = technically and structurally well supported
audit-interpretation-seo-quality-partial = only partly clearly structured
audit-interpretation-seo-quality-weak = currently weak in content and structure
audit-interpretation-seo-no-attributes = no clear extra attributes
audit-interpretation-seo-profile-summary = The page reads as “{ $page_type }” and is { $quality }. Notable: { $traits }.
audit-interpretation-seo-note-depth = More content depth and clearly structured sections would raise utility.
audit-interpretation-seo-note-structure = More subheadings and a clearer content structure would make the page easier to scan.
audit-interpretation-seo-note-media = The page is heavily visual. More explanatory text and clearer context would improve utility and orientation.
audit-interpretation-seo-note-intent = The page does not yet serve its page type cleanly; structure and content should align more strongly with the actual user goal.
audit-interpretation-seo-note-fine = The page fits its page type well overall. The biggest lever is further sharpening content rather than fundamental rebuilds.

## Performance interpretation (`audit::performance_interpretation`)
audit-interpretation-perf-dom = high DOM complexity ({ $nodes } nodes)
audit-interpretation-perf-lcp = a late largest contentful paint under throttling ({ $ms } ms)
audit-interpretation-perf-render-blocking =
    { $count ->
        [one] 1 render-blocking resource
       *[other] { $count } render-blocking resources
    }
audit-interpretation-perf-unminified = likely unminified assets (~{ $kb } KB estimated savings)
audit-interpretation-perf-animations = { $count } non-composited animations
audit-interpretation-perf-positive-layout = layout stability (CLS)
audit-interpretation-perf-positive-main-thread = main-thread blocking (TBT)
audit-interpretation-perf-critical-here = Critical here are { $phrases }.
audit-interpretation-perf-contrast =
    In contrast, { $phrases } { $count ->
        [one] remains
       *[other] remain
    } unobtrusive.
audit-interpretation-perf-reason-dom = DOM size
audit-interpretation-perf-reason-render-blocking = render-blocking resources
audit-interpretation-perf-gap = { $base } Measured load times (LCP) and layout stability (CLS) are good. The score is reduced by { $reasons }, which show technical optimization potential; whether this causes noticeable delays for real users cannot be derived from this lab run.
audit-interpretation-perf-inp-not-measured = Interaction responsiveness (INP) is not measured in this lab run; Total Blocking Time serves as a proxy.

## Score interpretation per module and band (`audit::interpretation`)
audit-interpretation-score-accessibility-excellent = Excellent — accessibility is implemented cleanly, with only minor limitations.
audit-interpretation-score-accessibility-good = Good — accessibility is sound overall; minor improvements are worthwhile.
audit-interpretation-score-accessibility-needs-improvement = Needs improvement — individual barriers can restrict use.
audit-interpretation-score-accessibility-weak = Inadequate — significant barriers impair usability and accessibility.
audit-interpretation-score-accessibility-critical = Critical — essential accessibility requirements are not met.
audit-interpretation-score-performance-excellent = Excellent — the page responds quickly and feels smooth to use.
audit-interpretation-score-performance-good = Good — performance is stable, with occasional room for optimization.
audit-interpretation-score-performance-needs-improvement = Needs improvement — load times and responsiveness are inconsistent in places.
audit-interpretation-score-performance-weak = Inadequate — performance issues can impair use and conversion.
audit-interpretation-score-performance-critical = Critical — significant performance problems severely impair the user experience.
audit-interpretation-score-security-excellent = Excellent — no significant security issues found within the scope checked.
audit-interpretation-score-security-good = Good — basic security mechanisms are in place; minor weaknesses were identified.
audit-interpretation-score-security-needs-improvement = Needs improvement — individual security aspects should be reviewed and hardened.
audit-interpretation-score-security-weak = Inadequate — relevant security issues or misconfigurations were found.
audit-interpretation-score-security-critical = Critical — basic protections are missing within the scope checked. That is no proof of an exploitable flaw, but it should be fixed with priority.
audit-interpretation-score-html-conform-excellent = Excellent — the rendered HTML is technically clean and spec-conformant; browsers and assistive technologies can parse it reliably.
audit-interpretation-score-html-conform-good = Good — the HTML is largely conformant, with a few deviations from the specification.
audit-interpretation-score-html-conform-needs-improvement = Needs improvement — several HTML specification violations can affect processing by browsers or assistive technologies.
audit-interpretation-score-html-conform-weak = Inadequate — widespread specification violations make reliable markup processing harder.
audit-interpretation-score-html-conform-critical = Critical — the HTML deviates significantly from the specification, directly affecting rendering and accessibility.
audit-interpretation-score-mobile-excellent = Excellent — the site works reliably on mobile devices, with no noticeable limitations.
audit-interpretation-score-mobile-good = Good — the site works reliably on mobile devices overall.
audit-interpretation-score-mobile-needs-improvement = Needs improvement — layout and usability issues appear in places on mobile devices.
audit-interpretation-score-mobile-weak = Inadequate — layout and usability on mobile devices are noticeably impaired.
audit-interpretation-score-mobile-critical = Critical — the site is barely usable on mobile devices.
audit-interpretation-score-ux-excellent = Excellent — the structural signals point to a clear, well-guided interface.
audit-interpretation-score-ux-good = Good — the structural signals point to clear user guidance; individual spots can be tightened.
audit-interpretation-score-ux-needs-improvement = Needs improvement — the structural signals suggest needlessly complex user guidance in places.
audit-interpretation-score-ux-weak = Inadequate — the detected friction points can get in the way of clear user guidance.
audit-interpretation-score-ux-critical = Critical — the structural signals point to a confusing interface that can make it hard to reach goals.
audit-interpretation-score-journey-excellent = Excellent — the page structure supports the key user paths consistently and clearly.
audit-interpretation-score-journey-good = Good — the page structure supports the core user paths; individual steps can be improved.
audit-interpretation-score-journey-needs-improvement = Needs improvement — the page structure suggests cumbersome or unclear steps in the user flow.
audit-interpretation-score-journey-weak = Inadequate — the detected breaks in the user flow can make it harder to reach key goals.
audit-interpretation-score-journey-critical = Critical — the page structure points to needlessly complicated or broken steps in the user flow.

## Accessibility claim bullet (`audit::interpretation`)
audit-interpretation-claim-not-measured = Accessibility pattern: Not measured — the automated accessibility checks did not run
audit-interpretation-claim-incomplete = Accessibility pattern: No violations found, but the run was incomplete — no evidence either way
audit-interpretation-claim-clean = Accessibility pattern: No violations in the automated checks ({ $automated } of { $all } WCAG { $version } A/AA criteria testable) — conformance not determined, manual review needed
audit-interpretation-claim-systematic = Accessibility pattern: Systematic patterns ({ $rules } rule types, { $total } instances) — process problem, not a one-off
audit-interpretation-claim-blockers = Accessibility pattern: Multiple critical blockers ({ $critical } critical, { $high } high) — direct screen-reader barriers
audit-interpretation-claim-distributed = Accessibility pattern: Distributed across { $rules } rule types — no pattern, fixable individually
audit-interpretation-claim-focused = Accessibility pattern: { $total } violations across { $rules } areas — focused and fixable

## Technical overview bullets (`audit::interpretation`)
audit-interpretation-overview-seo-excellent = SEO level: { $score } pts — technical ranking prerequisites met
audit-interpretation-overview-seo-good = SEO level: { $score } pts — base in place, targeted optimizations possible
audit-interpretation-overview-seo-partial = SEO level: { $score } pts — relevant signals missing, visibility limited
audit-interpretation-overview-seo-weak = SEO level: { $score } pts — structural base missing, ranking potential is severely limited
audit-interpretation-overview-seo-none = SEO level: Not audited (use --full for full analysis)
audit-interpretation-overview-security-full = Security level: { $score } pts — HTTP security headers fully set
audit-interpretation-overview-security-core = Security level: { $score } pts — { $present } of { $total } core headers set, open security findings remain
audit-interpretation-overview-security-basic = Security level: { $score } pts — basic protection in place, individual headers missing
audit-interpretation-overview-security-several = Security level: { $score } pts — several baseline security headers missing
audit-interpretation-overview-security-almost = Security level: { $score } pts — security headers almost entirely missing — a priority, low effort to fix
audit-interpretation-overview-security-none = Security level: Not audited (use --full for full analysis)
audit-interpretation-overview-tech-high = Tech complexity: High — { $dom } DOM nodes, performance { $perf } pts — refactoring recommended
audit-interpretation-overview-tech-strain = Tech complexity: Medium-high — { $dom } DOM nodes add strain to only middling performance ({ $perf } pts) — optimization potential
audit-interpretation-overview-tech-holding = Tech complexity: Medium-high — { $dom } DOM nodes, performance currently holding up ({ $perf } pts)
audit-interpretation-overview-tech-perf-critical = Tech complexity: Performance critical ({ $perf } pts) — { $dom } DOM nodes analyzed
audit-interpretation-overview-tech-low-optimizable = Tech complexity: Low — { $dom } DOM nodes, performance can be optimized ({ $perf } pts)
audit-interpretation-overview-tech-low-stable = Tech complexity: Low — { $dom } DOM nodes, performance { $perf } pts — technical baseline stable
audit-interpretation-overview-tech-unmeasured-high = Tech complexity: High — { $nodes } nodes checked (performance not audited)
audit-interpretation-overview-tech-unmeasured = Tech complexity: { $nodes } nodes checked (performance not audited, use --full)

## Overall impact table and benchmark (`audit::interpretation`)
audit-interpretation-impact-label-user = User experience
audit-interpretation-impact-label-risk = Risk level
audit-interpretation-impact-label-conversion = Conversion effect
audit-interpretation-impact-user-excellent = Excellent — no relevant barriers
audit-interpretation-impact-user-good = Good — individual barriers for assistive technologies
audit-interpretation-impact-user-limited = Limited — noticeable barriers for screen-reader users
audit-interpretation-impact-user-heavy = Heavily limited — essential content not accessible
audit-interpretation-impact-risk-high = High — acute BITV/WCAG violation risk
audit-interpretation-impact-risk-critical-topics = Medium — critical topics present
audit-interpretation-impact-risk-backlog = Medium — cumulative backlog
audit-interpretation-impact-risk-low = Low
audit-interpretation-impact-conversion-negative = Likely negative
audit-interpretation-impact-conversion-possible = Possibly negative (navigation, forms)
audit-interpretation-impact-conversion-low = Low — good usability
audit-interpretation-benchmark-top5 = Top 5% — exceptional level. No structural pressure to act.
audit-interpretation-benchmark-top15 = Top 15% — clearly above the majority. Polish is enough.
audit-interpretation-benchmark-upper-third = Upper third — good standing, individual optimizations pay off.
audit-interpretation-benchmark-middle = Middle pack — improvement potential, no acute emergency.
audit-interpretation-benchmark-lower-middle = Lower middle — clear gap to comparable websites.
audit-interpretation-benchmark-lower-third = Lower third — significant gap, structural deficits common.
audit-interpretation-benchmark-critical = Critical — among the weakest audited sites. Immediate action required.

## Verdict reasons (`audit::verdict`)
audit-interpretation-verdict-legal-flags =
    { $count ->
        [one] 1 finding with possible legal relevance (BFSG/EAA)
       *[other] { $count } findings with possible legal relevance (BFSG/EAA)
    }
audit-interpretation-verdict-fail-threshold = score { $score } below the fail threshold of { $threshold }
audit-interpretation-verdict-warn-threshold = score { $score } below the warning threshold of { $threshold }
audit-interpretation-verdict-incomplete = the run did not complete in full
audit-interpretation-verdict-technical-blockers =
    { $count ->
        [one] 1 technical blocker (page meant to be indexed carries noindex)
       *[other] { $count } technical blockers
    }
audit-interpretation-verdict-findings-present =
    { $count ->
        [one] 1 open finding
       *[other] { $count } open findings
    }

## Complexity and expected impact of a finding (`audit::finding_derive`)
audit-interpretation-complexity-high =
    { $count ->
        [one] 1 occurrence
       *[other] { $count } occurrences
    } indicate a component- or template-level issue.
audit-interpretation-complexity-technical = The fix is technical but affects a limited number of patterns.
audit-interpretation-complexity-moderate =
    { $count ->
        [one] 1 occurrence
       *[other] { $count } occurrences
    } require consistent updates across content or templates.
audit-interpretation-complexity-low = Few occurrences and a clearly scoped fix.
audit-interpretation-expected-impact-wcag =
    Addresses { $count ->
        [one] 1 occurrence
       *[other] { $count } occurrences
    }; WCAG level: { $level }.
audit-interpretation-expected-impact-other =
    Addresses { $count ->
        [one] 1 occurrence
       *[other] { $count } occurrences
    }; affects visibility/structure.
# Score effect of an action (#1169): the score recomputed without the action's findings, against the current findings
audit-interpretation-score-effect-not-quantified = Score effect not quantified.
audit-interpretation-score-gain =
    If only this action is done: up to +{ $points } { $points ->
        [one] point
       *[other] points
    } on the accessibility score (simulated).
audit-interpretation-score-gain-capped =
    If only this action is done: up to +{ $points } { $points ->
        [one] point
       *[other] points
    } on the accessibility score (simulated); it remains bounded by the cap of { $cap } while other critical or high findings remain.
audit-interpretation-score-gain-none = If only this action is done, the accessibility score does not change (simulated).
audit-interpretation-score-gain-none-capped = If only this action is done, the accessibility score does not change (simulated): it remains bounded by the cap of { $cap } while other critical or high findings remain.
audit-interpretation-score-gain-mean =
    If only this action is done on every page: up to +{ $points } { $points ->
        [one] point
       *[other] points
    } on the average accessibility score (simulated per page).
audit-interpretation-score-gain-mean-none = If only this action is done on every page, the average accessibility score does not change (simulated per page).

## Audit summary narrative (`audit::summary`); `$cross` is the cross-impact note with its leading space or empty
audit-interpretation-summary-groups =
    { $count ->
        [one] 1 finding group
       *[other] { $count } finding groups
    }
audit-interpretation-summary-occurrences-note = , { $count } occurrences
audit-interpretation-summary-dominant-note = "{ $title }" accounts for { $share }{ "\u202F" }% of critical/high findings ({ $groups }{ $occurrences }).
audit-interpretation-summary-detail-occurrences = { $count } occurrences
audit-interpretation-summary-detail-hits = { $count } hits
audit-interpretation-summary-intro-systematic = No isolated issue — { $total } violations across { $critical } critical and { $high } high topics form a systematic pattern. It affects large parts of the site, not individual spots.{ $cross }
audit-interpretation-summary-intro-dominant = One issue dominates: "{ $title }" causes { $share } % of critical/high findings ({ $detail }). Focus here — one cause, high impact.{ $cross }
audit-interpretation-summary-intro-polished-clear = No pressing barriers detected by automation.{ $cross } Maintain the level and re-check regularly.
audit-interpretation-summary-intro-polished-topics =
    Stable foundation. { $urgent ->
        [one] 1 prioritized topic — fix it deliberately before it piles up.
       *[other] { $urgent } prioritized topics — fix them deliberately before they pile up.
    }{ $cross }
audit-interpretation-summary-intro-clustered = Stable foundation, but { $urgent } prioritized topics span several independent areas. Prioritize systematically — not all at once.{ $cross }
audit-interpretation-summary-intro-needs-work-clear = Stable foundation without acute risks. { $total } improvements possible — easy to prioritize.{ $cross }
audit-interpretation-summary-intro-needs-work-urgent =
    Good foundation, but { $urgent ->
        [one] 1 prioritized topic needs
       *[other] { $urgent } prioritized topics need
    } immediate attention.{ $cross }
audit-interpretation-summary-intro-weak = Relevant barriers present — { $urgent } of them critical or high. Prioritize systematically and start phase 1.{ $cross }
audit-interpretation-summary-intro-critical = Urgent action required: { $critical } critical, { $high } high issues. The site is hard to use for part of users — start phase 1 immediately.{ $cross }
audit-interpretation-summary-problem-minimal = No violations found within the automated audit scope
audit-interpretation-summary-problem-dominant = Dominant single issue: "{ $title }" — focused and fixable
audit-interpretation-summary-problem-single = Single issue — focused and fixable
audit-interpretation-summary-problem-structural = Structural deficits — pervasive barriers across multiple areas
audit-interpretation-summary-problem-clusters = Multiple problem clusters — spread across areas, targetable
audit-interpretation-summary-problem-polish = Polish — no structural deficits, last optimization levers
audit-interpretation-summary-problem-isolated = Multiple critical isolated issues — focused and fixable
audit-interpretation-summary-cross-headings = Missing heading structure simultaneously acts as an SEO weakness and an accessibility barrier.
audit-interpretation-summary-cross-security-mobile = Security weaknesses and mobile issues appear together — optimization effort spans multiple areas.
audit-interpretation-summary-cross-performance-mobile = Poor load times amplify mobile problems — mobile usability is doubly impaired.

## Small texts: long heading, baseline line, score area labels
audit-interpretation-long-heading = H{ $level } is too long ({ $length } chars): "{ $excerpt }..."
audit-interpretation-baseline-summary = Baseline: { $known } known, { $new } new, { $fixed } fixed findings; known findings do not affect the verdict.
audit-interpretation-baseline-unmatched = { $count } baseline pages were not part of this run.
audit-interpretation-score-area-semantics = Semantics
audit-interpretation-score-area-forms = Forms
audit-interpretation-score-area-keyboard = Keyboard
audit-interpretation-score-area-focus = Focus management
audit-interpretation-score-area-images = Images / alternative text
audit-interpretation-score-area-aria = ARIA
audit-interpretation-score-area-headings = Heading structure
audit-interpretation-score-area-landmarks = Landmarks / page structure
