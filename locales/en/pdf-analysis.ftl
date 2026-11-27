# Texts of the search experience composite and of the PDF's analysis sections
# (`output::search_experience`, `output::pdf::root_causes`,
# `output::pdf::problem_profile`, `output::pdf::wcag_coverage`,
# `pdf-analysis-*`), #936. Which message applies, the thresholds, the ratios
# and the criterion tags stay in Rust, and so do the texts that other modules
# own (the UX dimension summaries, the German names of the manual review
# criteria). The column headings "Finding", "Occurrences" and "Share" and the
# label "Technical SEO" are the messages `single-report-matrix-col-finding`,
# `batch-col-occurrences`, `batch-col-share` and `seo-score-card` of
# report.ftl and single-report.ftl. The arguments `found`, `warnings_text`,
# `manual_text`, `more` and `recommendations` carry the count messages of
# report.ftl for the German wording; the English one does not use them.

## Search experience
pdf-analysis-search-label = Technical findability and content structure
pdf-analysis-search-content-clarity = Content structure
pdf-analysis-search-trust = Trust signals
pdf-analysis-search-structure = Structure and semantics
pdf-analysis-search-ai = AI readability
pdf-analysis-search-mobile = Mobile readability
pdf-analysis-search-technical-seo-explanation = Title, description, canonical, sitemap, structured data and crawlable links.
pdf-analysis-search-content-estimated = Derived from page type and content depth, not from readability or correctness of the text.
pdf-analysis-search-structure-estimated = Estimated from headings, structured data and page profile signals.
pdf-analysis-search-ai-explanation = Readability, chunk quality and compact section structure for AI-assisted systems (unvalidated heuristic).
pdf-analysis-search-mobile-explanation = Mobile usability, readable text and tappable controls.
pdf-analysis-search-warning-seo = Technical SEO is { $score } / 100, but content, trust or AI signals reduce the customer-facing visibility score.
pdf-analysis-search-warning-images = { $words } words with { $images } images: important messages may be image-led rather than readable HTML.
pdf-analysis-search-warning-media = The page profile is media-heavy; image text should be backed by readable HTML copy.
pdf-analysis-search-warning-local = LocalBusiness or local trust data is incomplete or not machine-readable.
pdf-analysis-search-interpretation-gap = Technically findable, but content structure and signals are weaker for users and AI systems. Weak parts: { $names }.
pdf-analysis-search-interpretation-strong = Strong technical findability and content structure: { $parts } align.
pdf-analysis-search-interpretation-strong-technical = Strong technical visibility.
pdf-analysis-search-interpretation-solid = Solid base with visible gaps in { $parts }.
pdf-analysis-search-interpretation-solid-technical = Solid technical base with visible gaps.
pdf-analysis-search-interpretation-limited = Limited technical findability and content structure: the page is not sufficiently { $parts }.
pdf-analysis-search-interpretation-limited-technical = Limited technical visibility.
pdf-analysis-search-strong-part-technical = technical visibility
pdf-analysis-search-strong-part-content = content structure
pdf-analysis-search-strong-part-trust = trust signals
pdf-analysis-search-solid-part-content = content
pdf-analysis-search-solid-part-trust = trust
pdf-analysis-search-solid-part-ai = machine readability
pdf-analysis-search-limited-part-content = structured
pdf-analysis-search-limited-part-trust = trustworthy
pdf-analysis-search-limited-part-ai = extractable
pdf-analysis-search-join-and-two = { $first } and { $second }
pdf-analysis-search-join-and-many = { $initial }, and { $last }
pdf-analysis-search-join-or-two = { $first } or { $second }
pdf-analysis-search-join-or-many = { $initial }, or { $last }

## Problem profile
pdf-analysis-profile-title = Problem profile (distribution across modules): { $kind }
pdf-analysis-profile-kind-isolated = isolated
pdf-analysis-profile-kind-concentrated = concentrated & systemic
pdf-analysis-profile-kind-broad = broad & systemic
pdf-analysis-profile-names-two = { $first } and { $second }
pdf-analysis-profile-names-many = { $initial }, and { $last }
pdf-analysis-profile-structural = { $count ->
        [0] Isolated local optimizations without structural problems.
        [one] Isolated local optimizations; { $count } finding with wide reach or a structural fix is named under key risks.
       *[other] Isolated local optimizations; { $count } findings with wide reach or a structural fix are named under key risks.
    }
pdf-analysis-profile-isolated-good = Very good technical condition. { $tail }
pdf-analysis-profile-isolated-named = { $names } determines the primary need for action. { $tail }
pdf-analysis-profile-isolated-weak = { $names } { $several ->
        [yes] fall
       *[no] falls
    } below the "good" band and { $several ->
        [yes] determine
       *[no] determines
    } the need for action. No area is critical (below { $critical } points) and the shortfall stays limited to { $several ->
        [yes] these areas
       *[no] this area
    }, hence no systemic profile.
pdf-analysis-profile-concentrated-systemic = { $names } { $several ->
        [yes] determine
       *[no] determines
    } the need for action. A large portion of the accessibility findings is concentrated in a few recurring patterns.
pdf-analysis-profile-concentrated-stable = { $names } { $several ->
        [yes] determine
       *[no] determines
    } the need for action while other technical areas remain stable.
pdf-analysis-profile-concentrated-driver = { $names } { $several ->
        [yes] determine
       *[no] determines
    } the need for action.
pdf-analysis-profile-also-below = { $names } { $several ->
        [yes] fall
       *[no] falls
    } below the "good" band as well.
pdf-analysis-profile-broad-unnamed = Multiple technical areas are affected simultaneously and require structural corrections.
pdf-analysis-profile-broad-named = Multiple technical areas are affected simultaneously. In particular, { $names } require structural corrections.
pdf-analysis-profile-few = { $found } — too few to identify a recurring pattern.
pdf-analysis-profile-concentration = { $total } occurrences were detected. About { $pct }% of them fall on just { $top } findings. Most affected: { $title }.
pdf-analysis-profile-leverage = The { $top } most significant causes account for about { $pct }% of the { $total } occurrences detected — addressing those first has the biggest leverage on the list below.

## Root causes
pdf-analysis-causes-none = No accessibility findings were detected, so no root cause analysis is necessary.
pdf-analysis-causes-none-evidence = No confirmed WCAG violations, so no root cause analysis is necessary. { $warnings_text } and { $manual_text } remain open — they are documented in their own sections and are not scored.
pdf-analysis-causes-lead-in = Where the findings concentrate: if a finding sits in a reused component or template, one fix there resolves all of its occurrences — whether it does cannot be seen from a single page.

## Action plan
pdf-analysis-roadmap-modules-title = Other modules
pdf-analysis-roadmap-modules-subtitle = Recommendations from the security, mobile, performance and SEO modules that the key measures name.
pdf-analysis-roadmap-empty-modules = No prioritized accessibility actions. Other modules (performance, SEO, security, mobile): { $recommendations } — listed in their module sections and in the key measures above.

## Analysis by cause (#1171)
pdf-analysis-eyebrow = ANALYSIS
pdf-analysis-title = Analysis by cause
pdf-analysis-subtitle = Each finding once in full: problem, impact and how it was established, fix and reach, grouped by the cause the audit observed.
pdf-analysis-cause-markup = Recurring markup at { $selector }: { $count } identical elements on this page. One shared component or template is possible; the source code has to confirm it.
pdf-analysis-cause-none = No recurring element pattern observed; this finding stands for itself.
pdf-analysis-card-key-reach = Reach
pdf-analysis-card-key-cause = Cause and confidence
pdf-analysis-card-key-problem = Problem
pdf-analysis-card-key-impact = Impact
pdf-analysis-card-key-evidence = Evidence status
pdf-analysis-card-key-fix = Fix
pdf-analysis-card-key-priority = Priority, effort, owner
pdf-analysis-card-key-score-effect = Score effect
pdf-analysis-card-key-finding = Finding id
pdf-analysis-card-key-retest = Check after the fix
pdf-analysis-retest-others = { $others ->
        [one] Check the other element the same way.
       *[other] Check the other { $others } elements the same way.
    }
pdf-analysis-retest-name = Read the accessible name of the changed element in the browser's accessibility inspector: it must say what the element does.
pdf-analysis-retest-contrast = Measure the contrast of the changed text again (browser inspector, colour picker), also in the hover, focus and dark states where the page has them.
pdf-analysis-retest-alt = Check the text alternative of the changed image in the accessibility inspector; a decorative image must show no name.
pdf-analysis-retest-headings = Open the heading outline of the page (accessibility inspector or a headings list) and check that the levels follow the content.
pdf-analysis-retest-landmarks = List the landmarks of the page in the accessibility inspector and check that each region exists once and has a name where there are several.
pdf-analysis-retest-lang = Check the lang attribute in the page source: it must be a valid language code that matches the content.
pdf-analysis-retest-ids = Search the page source for the changed id: it may occur only once, and labels and ARIA references must still point to the intended element.
pdf-analysis-retest-keyboard = Operate the changed element with the keyboard only (Tab, Enter, Space): it must be reachable, show a visible focus and trigger its action.
pdf-analysis-retest-aria = Check the changed element in the accessibility inspector: its role, states and referenced elements must match what the page does.
pdf-analysis-retest-structure = Check the changed table or list in the accessibility inspector: structure, headers and items must be announced as intended.
pdf-analysis-retest-forms = Read the name and instructions of each changed field in the accessibility inspector and fill in the form with the keyboard only.
pdf-analysis-retest-layout = Check the changed area again at 200 % zoom and on a small viewport: nothing may be cut off or overlap, and controls must stay large enough to hit.
pdf-analysis-retest-validator = Run the page through the W3C validator again: the message for this location must be gone and no new one may appear.
pdf-analysis-retest-headers = Request the page again (for example with curl -I) and check that the response carries the header with the intended value.
pdf-analysis-retest-seo = Load the page source again and check the changed tag or value, then run the audit again and compare the SEO findings.
pdf-analysis-retest-generic = Run the audit again for this page: the finding must be gone or its reach lower than before.
pdf-analysis-card-priority-effort = { $priority } priority · { $effort } effort · { $role }

## WCAG coverage
pdf-analysis-coverage-legend = Green: checked and passed on this page. Red: violation found. Amber: not passed or needs review. Grey: nothing to check on this page.
pdf-analysis-coverage-principles-title = Principle coverage (executed criteria passed)
pdf-analysis-coverage-perceivable = Perceivable
pdf-analysis-coverage-operable = Operable
pdf-analysis-coverage-understandable = Understandable
pdf-analysis-coverage-robust = Robust
pdf-analysis-coverage-state-violation = { $count } with a violation
pdf-analysis-coverage-state-review = { $count } need manual review
pdf-analysis-coverage-state-not-testable = { $count } not testable or not run
pdf-analysis-coverage-state-not-applicable = { $count } not applicable
pdf-analysis-coverage-scope-title = Audit scope
pdf-analysis-coverage-intro = This audit supports automatic checks for { $automated } of ~{ $total } testable WCAG 2.2 AA criteria; { $executed } of them were evaluated in this run (the others did not apply or could not be tested on this page). The criteria listed below require manual review.
pdf-analysis-coverage-intro-new = This audit supports automatic checks for { $automated } of ~{ $total } testable WCAG 2.2 AA criteria; { $executed } of them were evaluated in this run (the others did not apply or could not be tested on this page). { $count } of the supported criteria ({ $list }, marked "WCAG 2.2" below and on individual findings) were added by WCAG 2.2 and are part of EN 301 549 V4.1.1 but not of V3.2.1. The criteria listed below require manual review.
pdf-analysis-coverage-automated-title = Automatically checked ({ $listed })
pdf-analysis-coverage-listed-note = { $listed } criteria in total: the { $automated } WCAG 2.2 A/AA criteria the ratio above is scoped to, plus { $aaa } AAA criteria this tool also checks.
pdf-analysis-coverage-manual-title = Requires manual review ({ $count })
pdf-analysis-coverage-how-title = How to test manually
pdf-analysis-coverage-test-keyboard-title = Keyboard navigation
pdf-analysis-coverage-test-keyboard-text = Tab through the entire page. No focus loss, no keyboard trap, every interactive element reachable.
pdf-analysis-coverage-test-screen-reader-title = Screen reader
pdf-analysis-coverage-test-screen-reader-text = Test with NVDA/JAWS (Windows) or VoiceOver (Mac/iOS) — landmark navigation and form interaction.
pdf-analysis-coverage-test-zoom-title = 400% zoom
pdf-analysis-coverage-test-zoom-text = At 400% browser zoom: page operable without horizontal scrolling, no content lost.
pdf-analysis-coverage-test-motion-title = Reduced motion
pdf-analysis-coverage-test-motion-text = Enable the OS 'reduce motion' setting and verify animations are disabled or significantly diminished.
pdf-analysis-coverage-test-modal-title = Modal / dropdown interaction
pdf-analysis-coverage-test-modal-text = Full keyboard interaction: Tab, Enter, Space, Escape, Arrow keys. Focus returns to the trigger on close.
pdf-analysis-coverage-test-color-title = Color blindness simulation
pdf-analysis-coverage-test-color-text = Use a tool like 'Color Oracle' to verify information is conveyed by more than color alone.
