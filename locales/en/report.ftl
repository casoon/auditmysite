section-summary = Executive Summary
section-methodology = Scope and Methodology
section-modules = Module Scores
section-findings-overview = Issues Overview
section-findings = Detected Issues
section-findings-technical = Technical Details
section-actions = Improvement Areas
section-appendix = Appendix: Technical Details
callout-limitations-title = Limits of Automated Testing
callout-note-title = Note
callout-no-issues-title = Excellent Accessibility
callout-no-issues-body = No automatically detectable accessibility issues were found.
label-priority = Classification
label-issue = Issue
label-owner = Owner
label-effort = Effort
label-module = Module
label-type = Type
label-tech-note = Technical Note
label-user-impact = User Impact
label-typical-cause = Typical Cause
label-affected-urls = Affected URLs
label-code-example = Code Example
label-wrong = Wrong
label-right = Correct
label-decorative = Decorative
priority-critical = Critical
priority-high = High
priority-medium = Medium
priority-low = Low
role-development = Development
role-editorial = Editorial
role-designux = Design / UX
role-projectmanagement = Project Management
effort-quick = Low effort
effort-medium = Medium effort
effort-structural = Structural
effort-short-quick = low
effort-short-medium = medium
effort-short-structural = structural
severity-critical = Critical
severity-high = High
severity-medium = Medium
severity-low = Low
severity-error = Error
severity-warning = Warning
severity-info = Info
cover-fact-domain = Domain
cover-fact-scope = Scope
cover-fact-scope-single = Single URL
cover-fact-scope-batch = Sitemap / Batch
cover-fact-scope-comparison = Competitive comparison
cover-fact-modules = Modules
cover-fact-date = Audit date
metric-score = Overall score
metric-issues-detected = Issues detected
metric-critical-high = Critical / High
metric-risk = Risk
panel-quick-actions = Recommended improvements
panel-strengths = What is already strong
label-strength = Strength
section-top-issues = Top issues at a glance
section-all-violations = All violations (aggregated by rule)
audit-data-title = Audit data
audit-data-area = Area
audit-data-signal = Signal
audit-data-value = Value
audit-data-row-audit = Audit
audit-data-row-module = Module
audit-data-row-finding = Finding
scope-box-title = Scope
scope-box-wcag-level = WCAG level
scope-box-checked-nodes = Checked nodes
scope-box-runtime = Runtime
scope-box-findings-total = Findings total
scope-box-critical-high = Critical / High
scope-box-audit-notes = Audit notes

# Cover & narrative
narrative-cover-eyebrow = Automated audit report
narrative-cover-kicker = Technical website check
narrative-status-title = Site status
narrative-metrics-title = Executive snapshot
narrative-key-points-title = Key points
narrative-impact-title = Impact
narrative-quick-actions-title = Recommended improvements
narrative-spotlight-eyebrow = MAIN ISSUE
narrative-leverage-title = Effect of remediation
narrative-findings-title = Key findings
narrative-action-plan-title = Improvement areas
narrative-action-plan-intro = Audit-derived improvements grouped by effect, effort and affected user experience.
narrative-action-plan-callout-title = How to read these improvements
narrative-action-plan-callout-body = The following points describe audit-derived improvement areas. They are not a project plan and do not imply a timeline or delivery order.
narrative-technical-title = Technical implementation
narrative-technical-intro = From here onward you find the concrete implementation for development, design and editorial. Each issue includes affected elements, direct fix, and code examples.
narrative-next-steps-title = Additional verification
narrative-next-steps-intro = Notes on which checks usefully complement the automated audit.
narrative-next-steps-callout-title = Audit note
narrative-next-steps-callout-body = For a complete accessibility verification we additionally recommend a manual audit with assistive technologies (screen reader, keyboard navigation).
narrative-findings-intro-strong = The site is technically strong. The following items are last optimization levers without structural pressure.
narrative-findings-intro-solid = Stable foundation — the following items are targeted improvement levers.
narrative-findings-intro-default = The following issues have the largest impact on usability and risk. Technical details follow in the next section.

# Verdict (single audit)
verdict-tier-excellent = { $url } reaches { $score }/100 within the automated accessibility scope — no structural problem. Any remaining findings are polish; this score does not replace a manual review.
verdict-tier-excellent-medium = { $url } reaches { $score }/100 within the automated accessibility scope. The foundation is very good; the open medium-severity findings are targeted improvements. This score does not replace a manual review.
verdict-tier-excellent-high = { $url } reaches { $score }/100 within the automated accessibility scope. The foundation is very good, but at least one high-severity finding is still open — fix it first. This score does not replace a manual review.
verdict-tier-excellent-critical = { $url } reaches { $score }/100 within the automated accessibility scope. Despite the high score, at least one critical finding is open and should be fixed first. This score does not replace a manual review.
verdict-tier-solid = { $url } reaches { $score }/100 within the automated accessibility scope. The foundation is good — the remaining findings are targeted improvement levers at manageable effort.
verdict-tier-stable = { $url } reaches { $score }/100 within the automated accessibility scope. The foundation is stable but not free of barriers — several improvement levers should be planned.
verdict-tier-deficient = { $url } reaches { $score }/100 within the automated accessibility scope. There are significant barriers — not isolated details but structural backlog.
verdict-tier-critical = { $url } only reaches { $score }/100 within the automated accessibility scope. Urgent action needed: essential content and functions are not accessible for a part of users.
score-note-high-with-critical = The score weighs frequency and severity. Individual critical topics can persist despite a high overall score.
score-note-perfect-automated-scope = No confirmed accessibility violations were found within the automatically tested scope. Context-dependent WCAG criteria still require manual review.

# Verdict (batch audit)
verdict-batch-excellent = Across { $total_urls } audited URLs accessibility averages { $score }/100 — a very good result within the automated scope.
verdict-batch-solid = The { $total_urls } audited URLs average { $score }/100 for accessibility. The foundation is good, but recurring issues exist.
verdict-batch-stable = The { $total_urls } audited URLs average { $score }/100 for accessibility. The foundation is stable, but the recurring barriers should be planned for.
verdict-batch-deficient = The { $total_urls } audited URLs average { $score }/100 for accessibility. There are significant recurring problems.
verdict-batch-critical = The { $total_urls } audited URLs average only { $score }/100 for accessibility. The recurring barriers should be fixed with priority.

# Site state (audit summary)
site-state-polished = Strong
site-state-needs-work = Stable foundation
site-state-weak = Unstable
site-state-critical = Critical

# Risk levels
risk-level-critical = Critical
risk-level-high = High
risk-level-medium = Medium
risk-level-low = Low

# Yes / No
yes = Yes
no = No

# Grade labels
grade-excellent = Excellent
grade-good = Good
grade-satisfactory = Satisfactory
grade-deficient = Needs work
grade-critical = Critical

# Business / forward-looking consequence
business-consequence-clean = No barriers found in the automated checks — a sound starting point; the remaining criteria still need a manual review.
business-consequence-unverified = The automated checks did not cover the page completely — the absence of findings is no evidence of accessibility.
business-consequence-severe = Large parts of the site are unusable or barely usable for certain user groups.
business-consequence-seo-headings = The site is harder to find and structurally inaccessible to part of users.
business-consequence-screenreader = Individual key functions are blocked or unreliable for screen-reader users.
business-consequence-default = Usability is given — targeted improvements raise quality and reach.
consequence-severe = New content and features inherit the existing defects — remediation effort grows with every extension.
consequence-many-weak-modules = Effort for later corrections grows significantly — especially during a relaunch or major content expansion.
consequence-stable = No urgent pressure to act. Regular checks preserve the level after updates and extensions.
consequence-default = Without correction the site stays below the achievable standard — improvement potential is left on the table.

# Cover score row
cover-card-grade = Grade
cover-card-accessibility = Accessibility
cover-card-issues = Issues
cover-card-critical-high-suffix = critical/high
cover-card-average = Avg. technical score
batch-cover-overall-score-note = Mean technical score per URL (0–100, higher is better).
cover-card-urls = URLs
cover-card-violations-suffix = finding occurrences (all modules)
batch-cover-eyebrow = Automated batch audit report
batch-cover-title = Accessibility audit report
batch-cover-kicker = Cross-page audit of the selected URLs: accessibility as the main score, SEO, performance, security and further modules as context
batch-cover-frame-title = Audit scope
batch-cover-frame-domain = Domain
batch-cover-frame-date = Audit date
batch-cover-frame-urls = Audited URLs
batch-cover-frame-modules = Active modules
batch-cover-frame-version = Tool version
batch-cover-frame-scope = Coverage
batch-scope-sample = Sample — { $audited } of { $total } URLs ({ $pct } %, { $source }): the first { $audited } in source order, not chosen by page type; representativeness for the remaining pages is not established
batch-scope-sample-filtered = Sample — { $audited } of { $total } URLs ({ $pct } %, { $source }, by path filter); representativeness for the remaining pages is not established
batch-scope-sample-stratified = Sample — { $audited } of { $total } URLs ({ $pct } %, { $source }): drawn across structural groups (language, section, file type), not by page type or process state; representativeness for the remaining pages is not established
batch-cover-frame-failed = Not audited
batch-cover-frame-failed-value = { $failed } of { $selected } selected URLs failed
batch-cover-frame-http = HTTP check
batch-cover-frame-http-value = { $checked } sitemap URLs checked for HTTP status, { $audited } of them fully audited in the browser
batch-scope-full = Complete — all { $total } URLs ({ $source })
batch-source-sitemap = sitemap
batch-source-crawl = crawl
batch-source-url_file = URL list
batch-source-display_visual = pages with 3D/interactive visualisations
batch-cover-frame-display = Display mode
batch-display-offered = { $count } of { $total } pages offer display modes
batch-cover-frame-score-basis = Score basis
batch-score-coverage = { $scored } of { $attempted } URLs — { $failed } could not be audited and are not in any score
batch-unaudited-title = Scores cover { $scored } of { $attempted } URLs
batch-unaudited-body =
    { $failed ->
        [one] One URL could not be audited. It is
       *[other] { $failed } URLs could not be audited. They are
    } not included in any score or count of this report; the JSON report lists the error per URL.
batch-unaudited-list-title = Not audited
batch-unaudited-more = … and { $count } more
panel-modules-overview = Module overview
section-tech-detail-metrics = Technical detail metrics
column-action = Action
column-priority = Classification
column-timeframe = Effort
column-component = Component
column-area = Area
section-quick-wins = Quick wins
section-medium-actions = Medium-effort actions
section-structural-actions = Structural actions
section-next-steps-recommended = Recommended next steps
section-next-steps-block = Next steps

# Batch sections
batch-section-status = Site status
batch-section-most-frequent = Most frequent issues
batch-section-most-frequent-violations = Most frequent violations
batch-col-problem = Problem
batch-col-occurrences = Occurrences
batch-col-pages = Pages
batch-col-priority = Priority
batch-col-source = Source
batch-col-status-code = Status
batch-col-final-url = Final URL
batch-col-page-a = Page A
batch-col-page-b = Page B
batch-col-similarity = Similarity
batch-col-risk = Risk
batch-col-page-type = Page type
batch-col-attributes = Attributes
batch-col-top-issues = Top issues
batch-col-pages-list = Pages
batch-col-share = Share
batch-col-schema-type = Schema type
batch-col-profile = Profile
batch-col-severity = Severity
batch-col-description = Description
batch-col-metric = Metric
batch-col-budget = Budget
batch-col-links-to = Links to
batch-col-links-from = Links from
batch-col-words = Words
batch-action-plan-title = Action plan
batch-section-broken-links-internal = Internal broken links
batch-section-broken-links-external = External broken links
batch-section-external-links = External links
batch-section-redirect-chains = Redirect chains
batch-redirect-chains-title = Redirect chains (> 1 hop)
batch-col-target = Target
batch-url-ranking-title = URL ranking
batch-url-ranking-intro = Sorted by accessibility (column A11y), lowest first; ties put more critical/high findings first. "Score" is the combined technical score. A low value shows where most is lost — single critical functional findings can affect pages with a good score too.
batch-render-blocking-section = Render blocking & assets
batch-render-blocking-kv-title = Render-blocking overview (domain-wide)
batch-render-blocking-intro = Render-blocking resources and third-party traffic, aggregated across all audited pages.
batch-col-group-a11y = Avg. accessibility
batch-col-group-common-finding = Most common finding in the group
batch-group-single-page = Single page, not comparable
batch-group-common-finding = { $title } ({ $pages } of { $total } pages)
batch-group-no-common-finding = No finding shared by several pages of the group
batch-page-type-groups-note = The page type is a classification from page structure and content, not a verified template. Averages and the most common finding refer to the pages of each group; the most common finding counts only the top three findings per page.
batch-shared-blocking-title = The same render-blocking files on several pages
batch-col-size = Size
batch-minification-inconsistency-section = Build consistency
batch-minification-inconsistency-title = Inconsistent minification → build-config gap by template/route
batch-minification-inconsistency-intro = Assets served minified on some pages and unminified on others — a sign that the build configuration doesn't apply to every template/route.
batch-col-minify-asset = Asset
batch-col-minify-kind = Type
batch-col-minify-minified-on = Minified on
batch-col-minify-unminified-on = Unminified on
batch-minify-kind-script = JavaScript
batch-minify-kind-css = CSS
batch-redirect-chain-issues-section = Redirect chains
batch-redirect-chain-issues-title = Long redirect chains & loops → wasted crawl budget
batch-redirect-chain-issues-intro = Audited URLs that pass through 3 or more HTTP redirects before reaching a final page, or end in a cyclical redirect — every hop costs extra load time and crawl budget.
batch-col-redirect-url = URL
batch-col-redirect-hops = Hops
batch-col-redirect-type = Type
batch-col-redirect-chain = Chain
batch-redirect-chain-type-long = Long chain
batch-redirect-chain-type-loop = Loop
batch-section-tech-url-matrix = Technical URL matrix
batch-section-tech-url-matrix-intro = Condensed overview of all audited URLs focused on technical prioritization. Each row shows score, issue intensity, and the biggest lever for further optimization.
batch-table-pages-overview = Pages overview
batch-table-focus-pages = Focus on problematic pages
batch-table-page-type-distribution = Page-type distribution
batch-table-schema-distribution = Schema-type distribution
batch-table-top-pages = Strongest content pages
batch-table-broken-internal = Broken internal links
batch-table-broken-external = Broken external links
batch-section-performance-budgets = Performance budgets
findings-card-key-problem = Problem
findings-card-key-cause = Cause
label-improvement-suggestions = Improvement suggestions
label-recommendations = Recommendations
label-severity = Severity
label-classification = Classification
col-aspect = Aspect
col-value = Value
col-metric = Metric
section-perf-budget-violations = Performance budget violations
section-user-experience = User Experience
section-technical-complexity = Technical Complexity
section-seo-analysis = SEO analysis
section-serp-analysis = SERP analysis
section-page-health = Page health
section-robots-audit = robots.txt audit
section-seo-content-profile = SEO content profile
section-security = Security
section-html-conform = HTML Conformance
section-mobile-usability = Mobile usability
section-dark-mode = Dark mode
section-content-sections = Content sections
section-detected-entities = Detected entities
section-ux = User Experience
section-journey = User Journey
section-issue-overview = Issue overview
section-link-suggestions = Link suggestions
comparison-cover-title = Competitive comparison
comparison-domain-ranking = Domain ranking
comparison-module-comparison = Module comparison
comparison-top-findings-per-domain = Top findings per domain

# Device preview
section-device-preview = Device Preview
section-device-preview-no-screenshots = No screenshots available — screenshots could not be captured during this audit. Check that Chrome is installed and accessible.

# Diagnosis section tables
diagnosis-table-categories = Category breakdown
diagnosis-clusters-note = A cluster needs at least two findings in the same category; categories with a single finding appear in the category breakdown but not here.
diagnosis-table-clusters = Problem clusters
diagnosis-col-category = Category
diagnosis-col-findings = Findings
diagnosis-col-worst-severity = Worst severity
diagnosis-col-occurrences = Occurrences
diagnosis-col-max-severity = Max. severity

# Batch module overview
batch-panel-module-averages = Module averages (avg. across all URLs)

# Heuristic indicator (shared by UX, Journey, AI Visibility)
label-heuristic-indicator = Heuristic estimate based on structural signals, not externally validated

# Optional/non-normative feature (Dark Mode, #577)
label-optional-feature = Optional feature

# Performance section
perf-score-card = Performance score
perf-technical-indicators = Supporting measurements
perf-lab-data-note = Lab data
perf-lab-data-body = LCP, CLS and TBT are measured locally in headless Chrome under a real slow-4G throttle (CDP: 1.6 Mbps, 150 ms latency, 4x CPU slowdown). These values can differ from Google PageSpeed / Lighthouse: PageSpeed measures unthrottled and simulates the throttle in software (the Lantern model), whereas we throttle for real — the real measurement often reads more optimistically, LCP especially. Only field data (CrUX/real-user monitoring) reflects the actual user experience; the "performance under throttled conditions" table below shows the range across several network profiles. Of these, LCP and CLS are Core Web Vitals; TBT is a lab-only metric used here as a proxy for interactivity problems, because the third Core Web Vital (INP) cannot be measured without user input. Metrics marked "lab estimate" (INP, TTI, Speed Index) are derived approximations, not direct measurements.
perf-render-blocking-analysis = Render-blocking analysis
perf-measurement-warnings-title = Measurement limitations
perf-warning-lcp-missing = LCP not captured: the PerformanceObserver returned no LCP entry (typical under heavy network throttling) — LCP score is absent from the rating.
perf-warning-si-fallback = Speed Index equals LCP: formula estimate without an independent measurement (0.35 × FCP + 0.65 × LCP).
perf-warning-tti-fallback = TTI equals LCP: formula estimate without a direct interactivity value.
perf-warning-inp-missing = INP not measurable: headless browsers generate no user input — interaction latency could not be captured.
perf-warning-lh-mobile-gap = Desktop score: { $desktop } · LhMobile score: { $mobile } — gap { $gap } points. The primary score is based on the unthrottled measurement.
perf-capped-size = The performance score was capped at { $cap } due to a very large payload size ({ $size } MB).
perf-capped-js = The performance score was capped at { $cap } due to a critical JavaScript size ({ $size } MB).
perf-capped-requests = The performance score was capped at { $cap } due to a high request count ({ $count }).
perf-capped-dom = The performance score was capped at { $cap } due to extremely high DOM complexity ({ $nodes } nodes).
perf-worst-case-xref = ({ $profile } profile, see the "{ pdf-perf-throttled-title }" table below)

# SEO section
seo-score-card = Technical SEO
seo-score-card-description = Measures technical signals (meta, structure, schema, hreflang). Content depth is evaluated separately.
seo-maturity = Maturity

# UX section
ux-score-card = UX score (indicator)
ux-dimensions = UX dimensions

# Journey section
journey-score-card = Journey score (indicator)
journey-page-type-dimensions = Journey classification & dimensions

# Budget section
budget-callout-exceeded = Budget exceeded
budget-callout-warnings = Budget warnings
budget-table-metric = Metric
budget-table-actual = Actual value
budget-table-overage = Overage
budget-table-title = Budget details

# Finding narrative arc labels (Diagnose → Cause → Effect → Implementation)
finding-narrative-diagnose = Diagnosis
finding-narrative-ursache = Cause
finding-narrative-wirkung = Effect
finding-narrative-umsetzung = Implementation

# Finding cards (shared across executive / standard / technical)
finding-key-problem = Problem
finding-key-impact = What users experience
finding-key-cause = Cause
finding-key-fix = What to do
finding-key-effort = Effort
finding-key-quick-win = Quick win — a few hours, high impact
finding-tech-context = Technical context
finding-tech-rule = Rule
finding-tech-instances = Instances
finding-tech-affected-elements = Affected elements
finding-tech-other-occurrences = Other similar occurrences
finding-tech-affected-urls = Affected URLs
finding-elements = Elements
finding-occurrences = Occurrences
finding-element-types = Element types
finding-affected-selectors = Affected selectors
finding-recommendation = Recommendation
finding-wrong = ✗ Wrong
finding-right = ✓ Correct
finding-line-column = Line { $line }, column { $column }
finding-location = Location
finding-note = Note
finding-representative-occurrences = Representative occurrences
finding-occurrence = Occurrence
finding-suggested-fix = Suggested code fix
finding-pattern = Pattern
finding-frequent-patterns = Frequent patterns
finding-frequent-patterns-note = This table groups occurrences by structural similarity (same element and selector pattern) across the page, not by exact value match — its counts can therefore differ from an individual finding's own occurrence count.
finding-reference = Reference

# SEO / Tracking section
seo-tracking-services = Tracking and external services
seo-kv-title = Technical SEO signals
seo-serp-readiness = SERP readiness
seo-serp-signals = SERP signals
seo-page-health-issues = Detected issues
seo-page-health-url-analysis = URL analysis
seo-page-html-validation = HTML validation
seo-technical-issues-title = Technical SEO issues

# Security section
security-score-card = Security score

# Mobile section
mobile-score-card = Mobile score
mobile-configured = Configured
mobile-touch-targets = Touch targets
mobile-viewport-config = Viewport configuration
mobile-font-analysis = Font analysis
mobile-content-sizing = Content sizing
mobile-cat-viewport = Viewport
mobile-cat-touch_targets = Touch targets
mobile-cat-fonts = Font sizes
mobile-cat-content = Content sizing

# AI Visibility section
ai-score-card = AI visibility (indicator)

# PM Extras
pm-extra-prioritize = Prioritize the action plan
pm-extra-qa = Quality assurance and testing
pm-extra-responsibilities = Define responsibilities

# Business Impact Prefixes
impact-prefix-widespread = Widespread:{" "}
impact-prefix-frequent = Frequent:{" "}

# Business Impact Bases
impact-base-seo = Can reduce search engine visibility and decrease organic traffic.
impact-base-security = Increases attack surface and risk of data loss.
impact-base-performance = Worsens load time and user experience, increases bounce rate.
impact-base-mobile = Impairs mobile usability for the majority of users.
impact-base-accessibility-readability = Impairs readability for users with visual impairment.
impact-base-accessibility-exclude = Can exclude users and may be legally relevant where the BFSG/EAA applies.
impact-base-accessibility-exclude-plain = Can exclude users.
impact-base-accessibility-voice = Can raise usage barriers and prevent interactions with key elements.
impact-base-accessibility-default = Impairs site quality and user experience.

# Humanized Action Texts
action-human-aria-label = Name interactive elements (buttons, links) clearly
action-human-alt-text = Add descriptive alternative text to images
action-human-contrast = Improve color contrast for text and UI elements
action-human-form-label = Label form fields unambiguously
action-human-heading = Structure heading hierarchy logically
action-human-keyboard = Ensure keyboard navigation and focus order
action-human-skip-link = Add skip links for screen-reader users
action-human-lang-attr = Mark page language correctly in HTML
action-human-page-title = Provide a meaningful page title
action-human-link-text = Label links clearly and unambiguously
action-human-landmark = Mark page structure with landmarks

# User Effects
effect-user-buttons = Users grasp controls instantly — fewer mis-clicks
effect-user-links = Clearer navigation — users find targets faster
effect-user-aria = All controls clearly named — screen-reader use without guessing
effect-user-images = Images understandable for users without sight
effect-user-contrast = Text readable for everyone — even in poor lighting
effect-user-forms = Forms fillable without confusion — fewer drop-offs
effect-user-heading = Content structure instantly graspable — faster orientation
effect-user-skip = Keyboard users jump directly to main content
effect-user-keyboard = Full operability without a mouse
effect-user-language = Screen reader pronounces content in correct language and inflection
effect-user-title = Page clearly identifiable in browser tab and search
effect-user-landmark = Screen-reader users navigate the page with structure
effect-user-default-quick = Direct, noticeable improvement of user experience
effect-user-default-medium = Noticeable improvement for affected user groups
effect-user-default-structural = Sustainably more inclusive experience for everyone

# Conversion Effects

# Narrative Arc Formatters
narrative-diagnose-multiple = Detected { $count } times: { $desc }
narrative-cause-suffix-high = {" "}This is a structural gap with clear impact.
narrative-cause-accessibility = This typically results from missing or incomplete semantic markup.
narrative-cause-seo = This typically results from missing or incomplete meta information.
narrative-cause-performance = This typically results from unoptimized assets or render-blocking resources.
narrative-cause-security = This typically results from missing or misconfigured HTTP security headers.
narrative-cause-mobile = This typically results from a layout or sizing issue not adapted for small screens.
narrative-cause-default = This typically results from a configuration or implementation gap.
narrative-impact-critical = This can seriously impede or prevent a key user interaction for affected user groups; only testing the task itself shows whether it is fully blocked.
narrative-impact-high = This significantly impairs usability for affected user groups.
narrative-impact-medium = This causes a noticeable friction point for some users.
narrative-impact-low = This is a minor issue that may affect a subset of users.
narrative-effort-quick = Quick win — a few hours
narrative-effort-medium = Medium effort — a few days
narrative-effort-structural = Structural change required
narrative-owner-dev = Development
narrative-owner-editorial = Editorial
narrative-owner-designux = Design / UX
narrative-owner-pm = Project Management
narrative-implementation-format = { $recommendation } ({ $effort }, Owner: { $owner })
narrative-implementation-empty = { $effort } — { $owner }

# Levers and Contexts
lever-accessibility-biggest = Biggest lever: { $finding }
lever-accessibility-default = Biggest lever: maintain results and verify manually
context-accessibility-none = No automatically detected barriers in the current run.
context-accessibility-summary = { $total ->
        [one] { $total } detected finding group
       *[other] { $total } detected finding groups
    }, { $high } with high impact.
card-accessibility-none = No high-impact findings
card-accessibility-summary = { $high ->
        [one] { $high } high-impact finding group
       *[other] { $high } high-impact finding groups
    }
lever-performance-dom = Biggest lever: reduce DOM size ({ $dom_nodes } nodes)
lever-performance-load = Biggest lever: reduce load time ({ $load } ms)
lever-performance-default = Biggest lever: keep optimizing the render path and asset size
context-performance-good-vitals = { $fcp } · User experience: fast — technical complexity is pulling the performance score down.
context-performance-dom-nodes = { $n } DOM nodes
context-performance-dom-na = DOM nodes n/a
context-performance-summary = { $fcp }, { $ttfb }, { $dom }.
card-performance-dom = { $dom_nodes } DOM nodes
card-performance-load = Load time { $load } ms
card-performance-default = Continue optimizing the render path
perf-lab-estimate-suffix = {" "}(lab estimate)
recommendation-performance-lcp = Load the largest visible element faster: optimize and prioritize hero images and ship critical styles earlier.
recommendation-performance-fcp = Deliver first visible content earlier: reduce render-blocking CSS and JavaScript files.
recommendation-performance-tbt = Free the main thread: split large JavaScript tasks and load non-essential scripts later.
recommendation-performance-cls = Avoid layout shifts: reserve space for media, banners and dynamic content with fixed placeholders.
recommendation-performance-dom = Slim down the DOM: reduce large components, deep container hierarchies and repeated markup blocks.
recommendation-performance-load = Reduce overall load time: compress large assets, sharpen caching and review third-party scripts.
recommendation-performance-default = Core metrics are stable. Next lever: regularly monitor page size and third-party scripts to keep the level.
lever-security-headers = Biggest lever: add missing security headers ({ $missing_headers } core headers)
lever-security-default = Biggest lever: keep hardening header rules and TLS setup
context-security-summary-https = { $present_headers } of 8 core headers present, HTTPS active.
context-security-summary-nohttps = { $present_headers } of 8 core headers present, HTTPS missing.
card-security-summary = { $present_headers } of 8 core headers present
recommendation-security-https = Enforce HTTPS everywhere and ensure a valid TLS certificate for every variant of the domain.
recommendation-security-csp = Add a Content Security Policy and only allow the script, style and media sources actually needed.
recommendation-security-hsts = Add HSTS so browsers permanently load the site only via HTTPS.
recommendation-security-coop = Cross-Origin-Opener-Policy (COOP) is relevant when the site uses SharedArrayBuffer, high-resolution timers, or communicates with cross-origin popups. Set to same-origin to enable cross-origin isolation — not required for standard sites.
recommendation-security-corp = Cross-Origin-Resource-Policy (CORP) is relevant when the site serves fonts, scripts, or media that should not be loadable by other origins (Spectre mitigation). Set to same-origin or same-site — not required if resources are intentionally public.
recommendation-security-permissions = Define a Permissions-Policy and only enable browser features actually used by the page.
recommendation-security-referrer = Set a Referrer-Policy so redirects and external requests don't expose more information than necessary.
recommendation-security-csp-unsafe-inline = Remove unsafe-inline from the CSP and allow approved inline scripts or styles through nonces or hashes.
recommendation-security-csp-unsafe-eval = Remove unsafe-eval from the CSP and avoid dynamically evaluated scripts.
recommendation-security-csp-wildcard = Replace wildcard sources in the CSP with the specific origins actually needed.
recommendation-security-csp-directive = Add the missing CSP directives: object-src 'none', base-uri 'self', and frame-ancestors matching your embedding policy.
recommendation-security-cors-credentials = Do not combine Access-Control-Allow-Origin: * with credentialed CORS; allowlist trusted origins instead.
recommendation-security-permissions-permissive = Tighten Permissions-Policy so only the browser features the page actually uses are enabled.
recommendation-security-sourcemap = Do not serve source maps publicly, or restrict access to them.
recommendation-security-hsts-preload = Extend the HSTS directive so it meets the preload requirements (max-age >= 1 year, includeSubDomains, preload).
recommendation-security-default = Core security headers are clean. Next step: review policies regularly and adapt them to new script and integration sources.
lever-mobile-small = Biggest lever: enlarge touch targets ({ $small_targets } too small)
lever-mobile-crowded = Biggest lever: increase spacing of mobile controls ({ $crowded_targets })
lever-mobile-default = Biggest lever: keep optimizing mobile readability and touch flows
context-mobile-proper = Viewport properly set, { $small_targets } too-small touch targets, { $crowded_targets } crowded gaps.
context-mobile-improper = Viewport not properly configured, { $small_targets } too-small touch targets, { $crowded_targets } crowded gaps.
card-mobile-small = { $small_targets } too-small touch targets
card-mobile-crowded = { $crowded_targets } crowded gaps
card-mobile-proper = Viewport properly set
card-mobile-improper = Review viewport

# Tracking summary
tracking-summary-zaraz-clean = Zaraz is detectable on the page. No additional tracking cookies or external tracking signals were observed during the run.
tracking-summary-zaraz-signals = Tracking or consent-related signals are detectable on the page. Inspect external embeds, cookie behavior and the actual trigger time after consent.
tracking-summary-fonts = Externally hosted Google Fonts are loaded. This is privacy- and performance-relevant and should be reviewed deliberately.
tracking-summary-signals = Tracking signals were detected. Verify consent, trigger time and the origin of embedded services.
tracking-summary-clean = No external Google Fonts, tracking cookies or other tracking signals were detected in the current run.

# PDF detail module additions
pdf-budget-violations-summary =
    { $total } budget { $total ->
        [one] violation
       *[other] violations
    } detected: { $error_count } critical (>50% exceeded), { $warning_count } { $warning_count ->
        [one] warning
       *[other] warnings
    }.
pdf-perf-section-title = Performance — User Experience & Technical Complexity
pdf-perf-intro-user-experience = Core Web Vitals, render blocking — how fast the page feels to users
pdf-perf-intro-technical-complexity = DOM size, resource loading, blocking budget
pdf-perf-overview-title = Performance — Overview
pdf-perf-sub-vitals = Loading Experience & Vitals
pdf-perf-sub-resources = Resources & Payload Size
pdf-perf-sub-bottlenecks = Loading Bottlenecks & Rendering
pdf-perf-throttled-title = Performance under throttled conditions
pdf-perf-throttled-profile = Profile
pdf-perf-throttled-incomplete = Incomplete: not all core metrics measured
pdf-perf-cls-title = CLS — Layout Shifts
pdf-perf-cls-value = Shift
pdf-perf-cls-time = Time
pdf-perf-tp-title = Third-Party Resources
pdf-perf-tp-total-origins = Total origins
pdf-perf-tp-total-transfer = Total transfer
pdf-perf-tp-requests = Requests
pdf-perf-tp-impact = Impact
pdf-perf-tp-significant = Significant (>20% of page bytes)
pdf-perf-tp-types = Types
pdf-perf-tp-isolated-title = Isolated Script Impact
pdf-perf-tp-isolated-baseline = Baseline TBT
pdf-perf-tp-isolated-without = TBT without script
pdf-perf-tp-isolated-impact = Estimated impact
pdf-perf-cc-title = Critical Request Chain
pdf-perf-cc-max-depth = Max depth
pdf-perf-cc-path = Critical path
pdf-perf-cc-total-requests = Total requests
pdf-perf-min-title = Unminified Assets
pdf-perf-min-files = Unminified files
pdf-perf-min-savings = Est. savings
pdf-perf-min-type = Type
pdf-perf-min-saving-col = Savings
pdf-perf-cov-title = Code Coverage
pdf-perf-cov-js-used = JS used
pdf-perf-cov-css-used = CSS used
pdf-perf-resource-focus-title = Largest directly actionable lever
pdf-perf-resource-focus = Minifying { $count } { $count ->
        [one] file
       *[other] files
    } can remove an estimated { $savings } KB from the delivered code. This reduces payload and parsing work without removing functionality. Remeasure under the same conditions afterwards.
pdf-perf-resource-note = JS heap is an observation without a universal threshold; growth and peaks during interactions matter. CO2e per view is a modeled estimate classified by the stated efficiency grade.
pdf-perf-resource-note-heap = JS heap is an observation without a universal threshold; growth and peaks during interactions matter.
pdf-perf-resource-note-carbon = CO2e per view is a modeled estimate classified by the stated efficiency grade.
pdf-perf-coverage-clean-title = Code usage looks healthy
pdf-perf-coverage-clean = No meaningful unused JavaScript was captured during this run. Code removal is therefore not the main lever right now. Coverage only applies to the tested page state.
pdf-perf-coverage-focus-title = Reduce unused code
pdf-perf-coverage-focus = In the tested page state, { $unused } KB of JavaScript remained unused. Defer modules that are not needed or remove them from the bundle, then remeasure.
pdf-perf-bottleneck-critical-title = Interpretation: the page becomes usable late
pdf-perf-bottleneck-critical = Critical measurements: { $metrics }. They indicate too much browser and loading work before the page becomes usable. { $action }
pdf-perf-bottleneck-dom-title = Interpretation: high browser load from DOM size
pdf-perf-bottleneck-dom = Critical measurement: { $metrics }. This burdens the browser's rendering and memory; a measured delay is not shown by this alone. { $action }
pdf-perf-bottleneck-warning-title = Interpretation: review the loading path
pdf-perf-bottleneck-warning = Notable measurements: { $metrics }. { $action }
pdf-perf-bottleneck-dom-action = First simplify the DOM and reduce blocking loading paths; then remeasure under the same conditions.
pdf-perf-bottleneck-load-action = First review large or blocking resources and third-party dependencies; then remeasure under the same conditions.
pdf-perf-priority-actions = Prioritized actions
pdf-perf-anim-title = Non-composited Animations
pdf-perf-anim-total = Total
pdf-perf-anim-properties = Properties
pdf-seo-indicator-note = Indicators are heuristic estimates based on measurable signals — not direct ranking signals, but pointers to optimization potential.
pdf-seo-indicator-title = Indicator-based metrics
pdf-seo-overview-title = SEO — Overview
pdf-seo-maturity = Maturity
pdf-seo-field = Field
pdf-seo-value = Value
pdf-seo-meta-tags-title = Meta tags
pdf-seo-meta-description = Description
pdf-seo-meta-issues-title = Meta tag issues
pdf-seo-headings = Headings
pdf-seo-social-tags = Social tags
pdf-seo-more-signals =
    { $count } additional { $count ->
        [one] signal
       *[other] signals
    } in the detailed appendix.
pdf-seo-ie-title = Image Efficiency
pdf-seo-ie-total = Total images
pdf-seo-ie-modern = Modern formats
pdf-seo-ie-legacy = Legacy formats
pdf-seo-ie-oversized-title = Oversized images (top 5)
pdf-seo-ie-source = Source
pdf-seo-ie-natural = Natural
pdf-seo-ie-displayed = Displayed
pdf-serp-summary = { $total } signals checked — { $pass } OK, { $warning } warnings, { $fail } failures.
pdf-serp-category = Category
pdf-serp-rich-result-types-title = Detected rich-result types
pdf-serp-rich-results-text = These schema types can be associated with enhanced search appearances: { $types }. Type detection alone does not confirm eligibility.
pdf-section-clean = No significant issues found within the audited scope.
pdf-ph-issue = Issue
pdf-ph-html-validator-title = HTML validation
pdf-ph-www-title = www consolidation
pdf-ph-check = Check
pdf-ph-count = Count
pdf-robots-error = robots.txt could not be loaded: { $err }
pdf-robots-no-access = No access
pdf-robots-block-all-body = All crawlers fully blocked (User-agent: * / Disallow: /). On staging domains this is correct — on production it would prevent search engines from crawling the site entirely.
pdf-robots-block-all-title = All crawlers blocked
pdf-robots-limit-ai-body = AI search bots (e.g. PerplexityBot, Amazonbot) are blocked. This is a deliberate choice — content will not appear in AI-generated answers. Blocking AI training bots (GPTBot etc.) is common practice and not a problem.
pdf-robots-limit-ai-title = Limited AI visibility
pdf-robots-training-blocked-body = Policy: { $policy } — AI training bots ({ $bots }) are blocked, AI search bots have access. This matches the recommended default configuration.
pdf-robots-training-blocked-title = AI training blocked (default)
pdf-robots-crawler-rules = Crawler rules
pdf-robots-allowed = Allowed
pdf-robots-blocked = Blocked
pdf-robots-status-fully-blocked = Fully blocked
pdf-robots-status-partially-blocked = Partially blocked
pdf-robots-sitemap-entries = Sitemap entries
pdf-robots-crawl-delay-title = Crawl-delay values
pdf-robots-crawl-delay-value = { $delay } seconds
pdf-seo-profile-aspect = Aspect
pdf-seo-profile-content-profile = Content profile
pdf-seo-profile-structured-data = Structured data
pdf-seo-profile-page-profile = Page profile
pdf-seo-profile-content-depth = Content depth
pdf-seo-profile-structure-quality = Structural quality
pdf-seo-profile-media-balance = Media balance
pdf-seo-profile-website-identity = Website identity
pdf-seo-profile-content-type = Schema.org content type
pdf-seo-profile-language = Language
pdf-seo-profile-schema-types = Schema types
pdf-seo-profile-schema-type-col = Schema type
pdf-seo-profile-completeness = Field coverage
pdf-seo-profile-structured-data-title = Structured data ({ $label })
pdf-seo-schema-status-title = JSON-LD status
pdf-seo-schema-status-ok =
    { $nodes ->
        [one] One evaluable schema node detected.
       *[other] { $nodes } evaluable schema nodes detected.
    } The JSON-LD is syntactically and structurally sound; content suitability is assessed separately.
pdf-seo-schema-status-issues =
    { $nodes ->
        [one] One evaluable schema node detected.
       *[other] { $nodes } evaluable schema nodes detected.
    } { $issues ->
        [one] One syntax or structure issue is present.
       *[other] { $issues } syntax or structure issues are present.
    } Faulty blocks or nodes cannot be interpreted reliably by search systems.
pdf-seo-schema-status-non-jsonld = Structured data detected as Microdata or RDFa. The format was detected but not validated in detail.
pdf-seo-schema-status-none = No structured data detected. Whether a useful schema is missing depends on the actual page type.
pdf-seo-schema-status-check = Check
pdf-seo-schema-status-detail = Impact
pdf-seo-schema-issues-title = Detected structure issues
pdf-seo-schema-fit-title = Page type and schema fit
pdf-seo-schema-fit-page-type = Page type for schema fit
pdf-seo-schema-fit-confidence = Detection confidence
pdf-seo-schema-fit-expected = Plausible primary schemas
pdf-seo-schema-fit-detected = Detected primary schemas
pdf-seo-schema-fit-none = None
pdf-seo-schema-rule-title = Feature-specific schema requirements
pdf-seo-schema-rule-feature = Schema and search feature
pdf-seo-schema-rule-status = Requirement status
pdf-seo-schema-rule-detail = Details
pdf-seo-schema-rule-limited = limited visibility
pdf-seo-schema-review-title = Context and content review
pdf-seo-schema-review-feature-col = Schema / feature
pdf-seo-schema-review-detail-col = Manual review
pdf-seo-schema-parity-title = Visible-content parity
pdf-seo-schema-parity-property-col = Schema / property
pdf-seo-schema-parity-status-col = Status
pdf-seo-schema-parity-evidence-col = Short evidence
pdf-seo-schema-rule-context-dependent = only for matching page context
pdf-seo-schema-rule-missing-required = Missing required properties: { $fields }
pdf-seo-schema-rule-missing-recommended = Recommended properties not present: { $fields }
pdf-seo-schema-rule-not-evaluated = The stricter check is applied only when the page context is clearly established.
pdf-seo-schema-rule-complete = Complete within the automatically testable scope.
pdf-seo-profile-rating = Rating
pdf-seo-profile-signals-met = Signals met
pdf-seo-profile-classification = Classification
pdf-seo-profile-strength-title = SEO signal strength ({ $passed } of { $total } signals met)
pdf-seo-profile-maturity-title = SEO maturity
pdf-seo-profile-techniques = Techniques
pdf-seo-profile-techniques-value = { $used } of { $total } detected
pdf-sec-https-yes = Yes
pdf-sec-https-unclear = Unclear
pdf-sec-protection-title = Protection Infrastructure
pdf-sec-header-tier-column = Category
pdf-sec-header-tier-note = Categories: Baseline headers apply to virtually every site; architecture-dependent and context-dependent headers are good practice but only relevant for specific setups — see the finding text for each header's concrete scenario. "Not assessable remotely" headers can't be judged as present/missing at all from an automated check.
pdf-sec-score-title = How the security score arises
pdf-sec-score-component = Component
pdf-sec-score-points = Points
pdf-html-conform-not-checked = HTML conformance could not be checked for this page.
pdf-html-conform-dedup-note = The score is charged against distinct defects, not occurrences: one faulty template element emits a finding per render but is still a single fix. Repetition raises the penalty in a damped way instead of multiplying it.
pdf-html-conform-defects = Distinct causes
pdf-html-conform-errors = Errors
pdf-html-conform-warnings = Warnings
pdf-html-conform-info = Info
pdf-html-conform-occurrences = Occurrences
pdf-html-conform-rule = Rule
pdf-html-conform-severity = Severity
pdf-html-conform-message = Description
pdf-html-conform-location = Location
section-commerce = Commerce
pdf-commerce-scope-note = Checks product structured data and mandatory-page linking on this one page — not a checkout, payment-processing, or cart audit (this tool has no cross-page session state).
pdf-commerce-overview-title = Overview
pdf-commerce-page-kind = Page type
pdf-commerce-completeness = Completeness
pdf-commerce-product-title = Product structured data
pdf-commerce-price = Price
pdf-commerce-availability = Availability
pdf-commerce-shipping = Shipping details
pdf-commerce-returns = Return policy
pdf-commerce-rating = Rating
pdf-commerce-not-exposed = Not exposed
pdf-commerce-trust-pages-title = Mandatory-page linking
pdf-commerce-linked = Linked
pdf-commerce-not-linked = Not linked
pdf-ux-overview-title = UX — Overview
pdf-ux-more-issues =
    { $count } additional { $count ->
        [one] issue
       *[other] issues
    } in the detailed appendix.
pdf-journey-overview-title = Journey — Overview
pdf-journey-detected-page-type = Journey classification
pdf-journey-more-issues =
    { $count } additional { $count ->
        [one] issue
       *[other] issues
    } in the detailed appendix.
pdf-dm-status-supported = Supported
pdf-dm-status-not-supported = Not supported
pdf-dm-optional-note = Dark mode is an optional product feature, not a WCAG conformance criterion — the score on the left describes the detected implementation depth, not a pass/fail assessment.
pdf-dm-score-title = Dark mode score
pdf-dm-methods = Methods
pdf-dm-css-variables = CSS variables
pdf-dm-overview-title = Dark mode overview
pdf-dm-support = Support
pdf-dm-methods-impl = Implementation methods
pdf-dm-yes = Yes
pdf-dm-no = No
pdf-dm-css-custom-props = CSS custom properties (colors)
pdf-dm-contrast-violations = Contrast violations in dark mode
pdf-dm-only-issues = Dark-mode-only issues
pdf-dm-only-issues-val = { $count } (not in light mode)
pdf-dm-resolved-issues = Resolved in dark mode
pdf-dm-resolved-issues-val = { $count } light-mode { $count ->
        [one] issue disappears
       *[other] issues disappear
    } in dark mode
pdf-dm-issue-title = Dark mode issue
pdf-dm-note-title = Dark mode note
pdf-dm-print = Print
pdf-dm-print-stylesheet = Print stylesheet
pdf-dm-print-chrome-hidden = Interactive page chrome hidden in print
pdf-dm-print-clipped-elements = Potentially clipped print elements
pdf-dm-forced-colors = Forced colors
pdf-dm-forced-colors-active = Emulation active
pdf-dm-forced-color-adjust = forced-color-adjust elements
pdf-dm-forced-focus-visible = Focus/controls visible
pdf-dm-vision-title = Color vision emulation
pdf-dm-vision-mode = Mode
pdf-dm-vision-contrast = Contrast issues
pdf-dm-vision-new = New issues
pdf-dm-vision-use-color = Use-of-color issues
pdf-sq-section-title = Technical preparation for machine processing
pdf-sq-overview-title = What this indicator measures
pdf-sq-success = All scored signals of the technical preparation are met.
pdf-sq-unweighted = Unweighted indicator — it does not count toward the overall score and does not judge whether content is accurate or trustworthy.
pdf-sq-not-checked = Not checked: none of the on-page comparisons applied (language lang/og:locale, titles, structured data against visible content).
pdf-sq-cross-page-note = Consistency across pages (such as company name or contact details on different pages) cannot be checked on a single page; see the sitemap report.
pdf-sq-indicators-intro = Observed values without a rating. They describe the structure of the page, not whether its content is substantial.
pdf-ts-section-title = Tech Stack
pdf-ts-technology = Technology
pdf-ts-confidence = Confidence
pdf-ts-detected-title = Detected technologies
pdf-ts-confidence-high = High
pdf-ts-confidence-medium = Medium
pdf-ts-confidence-low = Low
pdf-ai-indicator-note = Experimental heuristics — not a standardised metric. Indicators are heuristic estimates based on measurable signals, not direct ranking signals. Use them as pointers to optimisation potential, not as evidence of LLM behaviour.
pdf-ai-section-title = AI Visibility (indicator)
pdf-ai-overview-title = AI Visibility — Overview
pdf-ai-score-title = AI visibility (indicator)
pdf-ai-score-desc-format = Grade: { $grade } — { $quality } · Heuristic estimate, not a measured value
pdf-ai-success = All AI visibility signals look good. No optimization required.
pdf-ai-readability = AI readability
pdf-ai-citability = Compact section structure (unvalidated)
pdf-ai-tech-readability = Technical AI readability
pdf-ai-policy = AI policy
pdf-ai-more-signals =
    { $count } additional { $count ->
        [one] signal
       *[other] signals
    } in the detailed appendix.
pdf-ai-sections-title = Sections
pdf-ai-section-col = Section
pdf-ai-words-col = Words
pdf-ai-rec-title = Recommendation
pdf-ai-entities-title = Entities
pdf-ai-entity-col = Entity
pdf-ai-relations-title = Relationships
pdf-ai-subject-col = Subject
pdf-ai-relation-col = Relation
pdf-ai-object-col = Object
pdf-ai-policy-blocks-all-body = All crawlers blocked (Disallow: *) — AI search bots have no access either.
pdf-ai-policy-blocks-all-title = No AI access
pdf-ai-policy-limited-title = AI visibility limited
pdf-ai-policy-limited-body = AI search bots blocked — content will not appear in AI-generated answers
pdf-ai-policy-training-body = AI training bots blocked, AI search bots have access — recommended default
pdf-cv-section-title = Content Visibility & Trust (indicator)
pdf-cv-overview-title = Content Visibility — Overview
pdf-cv-overview-body = Aggregated indicator from SEO, Source Quality, and AI Visibility signals. Covers organic search indexability, E-E-A-T authority markers, local business presence, content depth, and topical authority heuristics. Heuristic estimate — not a directly measured value.
pdf-cv-signals-analyzed = { $signals ->
        [one] { $signals } signal
       *[other] { $signals } signals
    } analyzed, { $problems } with optimization potential.
pdf-cv-manual-review-title = Manual review required
pdf-cv-success = All content visibility signals look good. No optimization required.
pdf-bp-score-title = Best Practices score
pdf-bp-success = No console errors and no vulnerable libraries detected.
pdf-bp-console-errors-title = Console Errors
pdf-bp-message-col = Message
pdf-bp-vuln-libs-title = Vulnerable Libraries
pdf-bp-lib-col = Library
pdf-bp-severity-col = Severity
pdf-bp-fix-col = Fix
pdf-bp-libs-up-to-date = Detected libraries appear to be up to date — no known vulnerabilities found.
pdf-bp-vuln-libs-unreviewed = (Unreviewed: the affected version range has not been checked against its source yet.)
pdf-bp-duplicate-libs-title = Duplicate Library Versions
pdf-bp-duplicate-libs-item = { $name }: { $versions }
pdf-bp-scope = Scope: console messages from page load to the end of the analysis (no clicks, scrolling or form submissions). Versions checked for { $libs }; other scripts are not covered.
pdf-bp-not-measured = Not measured: neither console messages nor library versions could be collected, so there is no score.
pdf-bp-console-not-collected = Console messages could not be collected; the score rests on the library check only.
pdf-bp-libs-not-checked = Library versions could not be checked; the score rests on the console messages only.
pdf-perf-cov-js-val = { $pct }% ({ $unused } KB unused)
pdf-perf-cov-css-val = { $pct }% ({ $used } of { $total } style rules used, from load to the end of the analysis, without clicks or scrolling)
pdf-perf-cov-css-partial = — readable stylesheets only
pdf-perf-dup-title = { $count } duplicate asset { $count ->
        [one] group
       *[other] groups
    } found (~{ $wasted } KB wasted)
pdf-cv-area-organic-visibility = Organic Visibility
pdf-cv-area-local-business = Local Business & Trust Data
pdf-cv-area-eeat = E-E-A-T Indicators
pdf-cv-area-content-depth = Content Depth & Localization
pdf-cv-area-topical-authority = Topical Authority (heuristic)

batch-top-issues-intro = The following problem groups occur across multiple URLs. Fixing them yields the largest improvement because they affect many pages simultaneously.
batch-col-affected-urls = Affected URLs
batch-meta-global = All audited pages
batch-meta-some-pages = { $count } of { $total } pages
batch-meta-single-page = One page — page-specific
batch-meta-occurrences = occurrences
batch-meta-affected-urls = { $count ->
    [one] affected URL
   *[other] affected URLs
}
batch-meta-effort = Effort per instance (estimate)
batch-meta-scope = Scope
batch-meta-impact-user = Impact (user)
batch-meta-impact-business = Impact (business)
batch-meta-fix = Fix
batch-meta-classification = Meta
batch-template-section-title = Template causes
batch-template-section-intro = The findings below recur at the same place on several pages. That points to a shared component or template; that one fix there resolves every occurrence is an assumption until the source code confirms it.
batch-template-confirmed-headline = Identical markup at '{ $selector }' on { $n } of { $total } audited pages. If it comes from one shared component, a fix there will likely take effect on all { $n } pages — the source code tells whether it does.
batch-template-likely-headline = Recurring at '{ $selector }' on { $n } of { $total } audited pages; identical markup is not established — a shared cause is possible, not shown.
batch-template-decision-confirmed = identical markup ({ $n } pages)
batch-template-decision-likely = recurring, identical markup not established ({ $n } pages)
batch-template-decision-open = recurs on { $n } pages, cause open
batch-action-plan-intro = The following actions are prioritized by effort and impact. Actions that improve many pages simultaneously take precedence.
batch-budget-pages-col = Affected pages
batch-budget-table-title = Performance budget violations (domain-wide)
batch-budget-intro = On how many pages each performance budget was exceeded.
batch-crawl-internal-intro = For the crawl starting at { $seed } we checked { $checked } internal link targets. { $broken } broken internal links detected.
batch-crawl-col-target = Target
batch-crawl-col-type = Type
batch-crawl-label-direct = direct
batch-crawl-label-hops = hops
batch-crawl-no-broken-internal = No broken internal links detected in the audited crawl set.
batch-crawl-external-intro = { $checked } external link targets checked. { $broken } broken external links detected.
batch-crawl-external-clean = { $checked } external link targets checked — no broken external links detected.
batch-crawl-redirect-chains-intro = { $count } links with more than one redirect hop detected.
batch-matrix-col-page = Page
batch-matrix-col-title = Title
batch-seo-potential-title = Content & SEO potential
batch-seo-potential-intro = Content strengths and weaknesses with direct relevance to rankings, visibility and conversion. Each finding is tied to a concrete action.
batch-seo-weakest-title = Pages with the lowest content profile
batch-seo-weakest-value = { $page_type } · profile { $score }/100
batch-seo-weakest-note = The content profile combines structure and length signals. A low value is no observed defect: short overview, location or news pages can be complete. Concrete defects are listed per page.
batch-seo-action-needed = Action needed
batch-seo-patterns-impact-title = Content patterns → business impact
batch-seo-impact-thin = { $insight } → weaker rankings, lower dwell time
batch-seo-impact-duplicate = { $insight } → keyword cannibalization, split ranking signals
batch-seo-near-dup-title = Near-duplicate content → keyword cannibalization
batch-seo-risk-high = High — consolidate
batch-seo-risk-medium = Medium — differentiate
batch-seo-duplicate-title = Identical metadata across pages → differentiate titles, descriptions and H1s
batch-col-dup-type = Type
batch-col-dup-value = Shared value
batch-col-dup-count = Pages
batch-dup-kind-title = Title
batch-dup-kind-description = Meta description
batch-dup-kind-h1 = H1 heading
batch-dup-kind-og-image = og:image
batch-seo-missing-tags-title = Missing optional tags across pages → missing everywhere usually means the template
batch-col-missing-count = Pages missing it
batch-col-missing-total = Pages checked
batch-missing-kind-canonical = Canonical URL
batch-missing-kind-og-image = og:image
batch-missing-kind-og-title = og:title
batch-seo-canonical-title = Canonical conflicts → resolve mixed indexing signals
batch-canonical-noindex = Canonical + noindex
batch-canonical-ogurl = Canonical ≠ og:url
batch-canonical-target-redirects = Canonical target redirects
batch-canonical-target-unreachable = Canonical target unreachable
batch-seo-hreflang-title = Non-reciprocal hreflang → add return links
batch-col-hreflang-source = Page
batch-col-hreflang-target = Points to (no return link)
batch-col-hreflang-lang = Lang
batch-seo-sitemap-http-title = Sitemap HTTP check
batch-sitemap-http-summary = { $checked } sitemap URLs fetched over HTTP, redirects not followed: { $distribution }. This checks reachability and status, not content or accessibility — only the sampled pages are fully audited.
batch-sitemap-class-other = other
batch-sitemap-redirects-title = Redirects in the sitemap, by target
batch-col-redirect-target = Target
batch-col-redirect-count = Count
batch-col-redirect-example = Example from the sitemap
batch-sitemap-redirects-more = { $more } more targets in the JSON.
batch-sitemap-other-issues-title = Other HTTP issues
batch-sitemap-http-json-note = Every source and target URL is in the JSON under sitemap_http_issues.
batch-seo-sitemap-orphan-title = Sitemap link graph → pages without observed inbound link and missing sitemap entries
batch-sitemap-kind-status = HTTP status
batch-sitemap-kind-redirect = Redirect
batch-sitemap-kind-noindex = noindex
batch-sitemap-kind-fetch-error = Fetch error
batch-sitemap-kind-orphan = No inbound link observed
batch-sitemap-orphan-note = { $count } audited pages without an observed inbound link. Only the links of the audited pages and the homepage were observed — pages outside the sample, search and script-built menus are missing. These are candidates, not confirmed orphans.
batch-sitemap-kind-linked-missing = Internally linked, missing from sitemap
batch-seo-robots-conflicts-title = Sitemap vs. robots.txt → indexing intent blocked by a Disallow rule
batch-col-robots-rule = Blocking Disallow rule
batch-crawl-depth-section = Internal link graph
batch-crawl-depth-title = Crawl depth → clicks from the start page
batch-crawl-depth-intro = Click distance from { $start_url } to every audited page, derived from the internal links already extracted in this batch run. Distribution by depth: { $histogram }.
batch-crawl-depth-histogram-entry = Depth { $depth }: { $count }
batch-crawl-depth-caveat = This run doesn't cover the full sitemap (a sample, or pages that failed to audit) — depth and reachability are therefore only a partial view of the actual link graph, not the complete site structure.
batch-col-crawl-depth = Clicks
batch-crawl-depth-kind-deepest = Furthest away
batch-crawl-depth-kind-unreachable = No path observed (within the captured pages)
batch-crawl-depth-unobserved-count = { $count } audited pages without an observed path from the homepage (up to 20 of them in the JSON under crawl_depths.unreachable_pages).
batch-crawl-depth-fetched-homepage = The homepage was only fetched to read its links — it is not audited and counts in no score.
batch-crawl-depth-unavailable = Click depths not determined: the homepage could not be fetched. No other page is used as a substitute start page.
batch-module-portfolio-title = Module portfolio
batch-module-portfolio-intro = Aggregated module values across the audited URL set: average, spread over the pages with a measurement, and effect per module.
batch-module-portfolio-table-title = Module values and spread
batch-col-module = Module
batch-col-score = Score
batch-col-spread = Spread
batch-module-spread-constant = { $value } on all { $pages } measured pages
batch-module-spread-range = { $min }–{ $max } ({ $pages } pages)
batch-col-effect = Effect
batch-schema-summary-all = All { $total } pages have structured data.
batch-schema-summary-some = { $without } of { $total } pages without structured data.
batch-schema-callout-title = Structured data (Schema.org)
batch-appendix-intro = Complete listing of all detected violations per URL with technical details for implementation.
batch-appendix-col-rule = Rule
batch-appendix-col-elements = Affected elements

metric-violations-total = Total{"\u00A0"}violations
metric-critical = Critical
metric-checked-nodes = Checked{"\u00A0"}nodes
metric-wcag-level = WCAG level
metric-warnings = Heuristic{"\u00A0"}warnings
metric-not-testable = Manual{"\u00A0"}testing{"\u00A0"}required
metric-overall-score = Overall{"\u00A0"}score
metric-accessibility-score = Accessibility{"\u00A0"}score
date-format-str = %Y-%m-%d
trend-significantly-improved = Significantly improved
trend-improved = Improved
trend-stable = Stable
trend-slightly-regressed = Slightly regressed
trend-significantly-regressed = Significantly regressed
history-trend-significantly-improved = Accessibility has improved significantly versus the run on { $previous_date } (+{ $delta_accessibility } points, { $delta_issues_abs } fewer issues).
history-trend-improved = Accessibility has improved versus the run on { $previous_date }.
history-trend-stable = Accessibility is unchanged compared with the run on { $previous_date }.
history-trend-significantly-regressed = Accessibility has regressed significantly versus the run on { $previous_date } ({ $delta_accessibility } points, +{ $delta_total_issues } issues). Action needed.
history-trend-slightly-regressed = Accessibility has slightly regressed versus the run on { $previous_date }.
history-summary = { $trend_interpretation } The history covers { $timeline_entries } usable snapshots.
history-metric-acc-delta = Accessibility delta
history-metric-total-delta = Overall delta
history-metric-issue-delta = Issue delta
history-metric-crit-delta = Critical+High delta
history-metric-prev-acc = Previous accessibility
history-metric-prev-total = Previous overall
ux-lever-cta = Phrase CTA texts more clearly and specifically
ux-lever-trust = Add trust signals (contact, imprint)
ux-lever-hierarchy = Clean up heading structure (H1 → H2 → H3)
ux-lever-default = Maintain UX quality at the current good level
journey-intent-label = Intent
journey-lever-default = Maintain journey clarity at the current level
finding-structural-cause-component = { $count } occurrences follow the same pattern — this strongly suggests a shared component or template, though that is only confirmed with evidence from further pages. A fix at that one shared point would likely address these occurrences together.
finding-structural-cause-shared = This issue appears on { $count } elements — possibly a shared component or template.
module-accessibility = Accessibility
module-security = Security
module-performance = Performance
module-seo = SEO
module-html-conformance = HTML conformance
module-mobile = Mobile
module-ux = UX
module-journey = Journey
module-best-practices = Best Practices
module-dark-mode = Dark Mode
module-ai-visibility = AI Visibility
module-source-quality = Technical preparation
module-tech-stack = Tech Stack
linktext-generic-stopwords = read more,learn more,click here,here,more,details,link,view,open,see more,find out more,discover
alt-text-boilerplate-prefixes = image of,picture of,photo of,photograph of,graphic of,icon of,this is an image of,this is a picture of
topic-stopwords = with,your,about,into,that,this,from,content,cloud,page,pages,site,web,website,websites,after,also,been,before,because,being,both,could,does,each,have,here,just,many,more,most,much,only,other,over,same,should,some,such,than,their,them,then,there,these,they,those,very,were,what,when,where,which,while,will,would,yours
video-transcript-keywords = transcript
sr-no-name = (no name)
sr-role-button = button
sr-role-link = link
sr-role-textbox = text field
sr-role-checkbox = checkbox
sr-role-radio = radio button
sr-role-combobox = combo box
sr-role-listbox = list box
sr-role-slider = slider
sr-role-spinbutton = spin button
sr-role-tab = tab
sr-role-heading = heading
sr-role-heading-level = heading level { $level }
sr-role-navigation = navigation
sr-role-main = main
sr-role-banner = banner
sr-role-contentinfo = content info
sr-state-expanded = expanded
sr-state-collapsed = collapsed
sr-state-checked = checked
sr-state-unchecked = not checked
sr-state-mixed = mixed
sr-state-selected = selected
sr-state-not-selected = not selected
sr-state-required = required
sr-state-invalid = invalid
sr-state-disabled = disabled
sr-state-pressed = pressed
sr-state-not-pressed = not pressed
sr-state-tab-stop = focusable

# Plan 09: easy-language ("Leichte Sprache") link-text markers, comma-separated,
# merged with the German list regardless of report language (sites mix languages).
easy-language-linktext-markers = easy language,easy-to-read,easy to read,plain language

# Audit exclusions (#645): methodology note (single) and audit frame (batch).
exclusion-fact-label = Excluded regions
exclusion-selector-matched =
    { $count ->
        [one] { $selector }: 1 element excluded
       *[other] { $selector }: { $count } elements excluded
    }
exclusion-selector-unmatched = { $selector }: matched no element, nothing excluded
exclusion-selector-invalid = { $selector }: invalid selector, not applied
exclusion-findings =
    { $occurrences ->
        [0] no finding was dropped
        [one] 1 finding occurrence dropped, not scored
       *[other] { $occurrences } finding occurrences from { $rules ->
            [one] 1 rule
           *[other] { $rules } rules
        } dropped, not scored
    }
exclusion-interactive = , plus { $interactive } journey findings
exclusion-screen-reader = , plus { $issues } screen-reader findings
exclusion-landmarks =
    { $landmarks ->
        [one] ; 1 landmark left out of the page's landmark counts
       *[other] ; { $landmarks } landmarks left out of the page's landmark counts
    }
batch-exclusion-selector-matched = { $selector }: { $count } elements excluded on { $pages } of { $total } pages
batch-exclusion-findings =
    { $occurrences ->
        [0] no finding was dropped
        [one] 1 finding occurrence dropped, not scored
       *[other] { $occurrences } finding occurrences dropped, not scored
    }

# ── Privacy indicators (#872) ─────────────────────────────────────────
privacy-title = Privacy indicators — first visit
privacy-scope = Technical observations of one first visit in a clean browser profile (no stored consent, no click, no scroll). This is not a legal assessment and not a compliance score. What the page does after a consent choice is not measured. A provider's seat is its own statement and says nothing about the actual data flow.
privacy-takeaway = Providers contacted: { $providers } · technical indicators: { $findings }. Observations only, no legal assessment.
privacy-takeaway-none = No technical indicator was recorded on the first visit. This is an observation within the measured scope, not a statement that the page meets any requirement.
privacy-takeaway-not-measured = The first visit could not be measured. No statement is made about requests, cookies or storage before a consent choice.
privacy-takeaway-unmeasured = Checks not measured: { $count } — there, "no indicator" is not a clean result.
privacy-metric-contacts = Providers contacted
privacy-metric-findings = Technical indicators
privacy-metric-unmeasured = Checks not measured
privacy-measurement-failed = Not measured: { $reason }
privacy-measurement-partial = Partly measured, missing: { $gaps }. Checks that depend on it read "not measured" instead of "nothing found".
privacy-table-checks = Checks and results
privacy-table-contacts = Contacted on the first visit
privacy-table-findings = Technical indicators
privacy-col-check = Check
privacy-col-result = Result
privacy-col-note = Note
privacy-col-provider = Provider
privacy-col-category = Category
privacy-col-seat = Seat (hint)
privacy-col-requests = Requests
privacy-col-hosts = Hosts
privacy-col-indicator = Indicator
privacy-col-confidence = Confidence
privacy-col-evidence = Evidence
privacy-no-contacts = No classified provider was contacted on the first visit.
privacy-unclassified = Hosts contacted but not in the provider list (not classified, not a finding): { $hosts }
privacy-ruleset = Classification ruleset: { $version }
privacy-more = +{ $count } more
privacy-check-tracking_requests = Tracking and advertising requests
privacy-check-non_essential_storage = Cookies and storage entries of non-essential providers
privacy-check-visitor_ip_transfer = Requests that hand the visitor IP to a third party
privacy-check-third_country_provider = Provider seat outside the EU
privacy-check-consent_platform = Consent platform present when trackers are
privacy-check-policy_links = Privacy policy and imprint links
privacy-check-short-tracking_requests = Tracking
privacy-check-short-non_essential_storage = Storage
privacy-check-short-visitor_ip_transfer = IP transfer
privacy-check-short-third_country_provider = Third country
privacy-check-short-consent_platform = Consent platform
privacy-check-short-policy_links = Policy links
privacy-outcome-found = Indicator
privacy-outcome-not_found = Nothing found
privacy-outcome-not_applicable = Not applicable
privacy-outcome-not_measured = Not measured
privacy-confidence-evidence = Evidence
privacy-confidence-indication = Indication
privacy-confidence-manual-check = Manual check
privacy-source-request = Request
privacy-source-cookie = Cookie
privacy-source-storage_key = Storage key
privacy-source-link = Link
privacy-source-signal = Signal
privacy-source-provider = Provider
privacy-seat-eu = EU
privacy-seat-us = USA
privacy-seat-other = Outside the EU
privacy-seat-unknown = Not stated
privacy-category-analytics = Analytics
privacy-category-advertising = Advertising
privacy-category-social = Social media
privacy-category-tag_manager = Tag manager
privacy-category-fonts = Fonts
privacy-category-cdn = CDN
privacy-category-video_embed = Video embed
privacy-category-maps = Maps
privacy-category-consent_platform = Consent platform
privacy-gap-cookies = cookies
privacy-gap-storage = web storage
privacy-gap-page_signals = page signals (consent traces, links)
privacy-gap-network_not_idle = network did not go quiet
privacy-reason-no_third_party_requests = no third-party request
privacy-reason-network_not_idle = network did not go quiet, late requests cannot be ruled out
privacy-reason-storage_unreadable = cookies or storage unreadable
privacy-reason-nothing_stored = nothing stored
privacy-reason-no_classified_provider = no classified provider contacted
privacy-reason-no_tracking_requests = no tracking request observed
privacy-reason-consent_traces_unreadable = consent traces unreadable
privacy-reason-page_signals_unreadable = page signals unreadable
privacy-reason-no_links_on_page = no links on the page
privacy-reason-cdp_command_failed = browser command failed
privacy-reason-evaluation_failed = page evaluation failed
privacy-kind-tracking_request_before_consent = Tracking or advertising request on the first visit
privacy-kind-non_essential_storage_before_consent = Cookie or storage entry of a non-essential provider on the first visit
privacy-kind-visitor_ip_transfer = Request that hands the visitor IP to a third party
privacy-batch-title = Privacy indicators — aggregated
privacy-batch-intro = Technical observations of one first visit per page (clean browser profile, no click), aggregated across the audited pages. Not a legal assessment and not a compliance score. Counts refer to the pages whose first visit was measured.
privacy-batch-coverage = First visit measured on { $measured } of { $total } audited pages; not measured: { $failed }.
privacy-batch-affected = { $pages } of { $measured } measured pages show at least one technical indicator.
privacy-batch-none = No measured page shows a technical indicator.
privacy-batch-nothing-measured = The first visit could not be measured on any page. No statement is made.
privacy-batch-table-kinds = Indicators by affected pages
privacy-batch-table-providers = Recurring providers
privacy-batch-table-checks = Check results across pages
privacy-batch-table-matrix = Pages at a glance
privacy-batch-col-pages = Pages
privacy-batch-col-spread = Spread
privacy-batch-col-found = Indicator
privacy-batch-col-not_found = Nothing found
privacy-batch-col-not_applicable = Not applicable
privacy-batch-col-not_measured = Not measured
privacy-batch-pages-of = { $pages } of { $measured }
privacy-batch-spread-all = on all measured pages
privacy-batch-spread-several = on several pages
privacy-batch-spread-single = on a single page
privacy-batch-providers-single = { $count } further providers on a single page only.
privacy-batch-matrix-legend = Number: indicators found. 0: nothing found. –: not applicable. n/m: not measured.
privacy-batch-matrix-more = Showing { $shown } of { $total } pages; the rest is in the JSON report.
privacy-batch-matrix-nm = n/m

# HTML conformance: written recommendation per rule family (#899)
html-conform-parser-title = Fix HTML syntax errors
html-conform-parser-fix = Fix the syntax errors the browser silently repairs while parsing (unclosed or misnested tags, stray end tags, duplicate attributes). Run the rendered HTML through the checker and correct the cause in the template or CMS, not just the output.
html-conform-schema-title = Remove disallowed elements and attributes
html-conform-schema-fix = Use elements and attributes only where the HTML Standard allows them: remove unknown or obsolete attributes, place elements only inside permitted parents and correct invalid attribute values. Each message names the affected element.
html-conform-ids-title = Make duplicate IDs unique
html-conform-ids-fix = Assign each id only once per document, including repeated components and cards. Label references (for), ARIA references and anchors otherwise point to the wrong element. Repeated components need an id generated per instance.
html-conform-headings-title = Correct the heading structure
html-conform-headings-fix = Use heading levels without gaps and without empty headings: do not skip a level (h1 straight to h3), give every section and article element a heading and set exactly one main heading. The level follows the outline, not the wanted font size.
html-conform-language-title = Declare the page language on the html element
html-conform-language-fix = Set the lang attribute on the html element to the matching language code (for example lang=en) and, if xml:lang is present, use the same value. Screen readers pick the pronunciation from it.
html-conform-image-alt-title = Give images a text alternative
html-conform-image-alt-fix = Give every img element an alt attribute: a short descriptive text for meaningful images, an empty alt for purely decorative ones. Images with a role or aria attributes need an accessible name, or an empty alt without a conflicting role.
html-conform-responsive-images-title = Correct srcset, sizes and source markup
html-conform-responsive-images-fix = Add the sizes attribute when srcset uses width descriptors (w), use sizes only together with srcset and give source elements inside picture a media or type attribute. Contradicting combinations make browsers pick the wrong image or none.
html-conform-image-maps-title = Correct image maps
html-conform-image-maps-fix = Define image maps completely: usemap must point to an existing map with a matching name and id, area elements belong inside the map and ismap needs an enclosing link with href. Where possible replace image maps with real links.
html-conform-nested-interactive-title = Resolve nested interactive elements
html-conform-nested-interactive-fix = Do not nest links, buttons, form fields or focusable elements inside links or buttons. Solve a large click area with a single block-level link instead, or place the elements side by side rather than inside each other.
html-conform-forms-title = Mark up form elements correctly
html-conform-forms-fix = Mark up form fields as the standard requires: point label for at an existing field, one label per field, attributes that fit the type (pattern, maxlength, required, readonly only where the type allows them) and selects with a valid placeholder entry and valid select, optgroup and option structure, and only utf-8 for accept-charset.
html-conform-document-meta-title = Correct metadata in the document head
html-conform-document-meta-fix = Clean up the document head: a non-empty title, exactly one meta charset with UTF-8 near the top, a viewport meta without user-scalable=no, at most one description and base before all links and scripts. Remove leftovers such as X-UA-Compatible.
html-conform-link-rel-title = Correct link elements in the head
html-conform-link-rel-fix = Specify link elements completely: href present, rel matching the other attributes (as on preload and modulepreload with a valid value, imagesizes only with imagesrcset, integrity only for permitted rel values, alternate stylesheet with a title).
html-conform-scripts-title = Correct script attributes and script types
html-conform-scripts-fix = Use script attributes only in permitted combinations: no defer on modules, no async or defer on inline scripts, no attributes such as nomodule, integrity or fetchpriority on import maps and speculation rules, and remove the unnecessary type text/javascript and the obsolete language attribute. The content of importmap and speculationrules must be valid JSON.
html-conform-csp-title = Comply with the Content-Security-Policy meta element
html-conform-csp-fix = Move inline scripts and inline styles into files, allow them by nonce or hash in the policy, or adjust the policy. A policy set through meta otherwise blocks them in the browser. Prefer delivering the policy as an HTTP header.
html-conform-tables-title = Correct the table structure
html-conform-tables-fix = Build tables as a regular grid: every row has the same number of cells, colspan and rowspan do not exceed the table edge or groups, rows are not empty, th gets a valid scope and headers points to existing th cells. Do not use tables for layout.
html-conform-aria-redundant-role-title = Remove redundant ARIA roles
html-conform-aria-redundant-role-fix = Remove roles the HTML element already has natively (for example role=button on button, role=navigation on nav, role=list on ul). Redundant roles add maintenance cost and can override native behaviour.
html-conform-aria-naming-title = Use ARIA names only where permitted
html-conform-aria-naming-fix = Set aria-label, aria-labelledby and aria-braillelabel only on elements and roles that may carry a name; for roles without name support (for example generic) make the text visible or choose a fitting role. Do not add aria-label on label elements that use for.
html-conform-aria-state-title = Remove disallowed ARIA attributes and states
html-conform-aria-state-fix = Set ARIA states only where role and element allow them: no aria-valuemin, aria-valuemax or aria-checked on elements that express min, max or checked natively, no aria-hidden on focusable elements, aria-activedescendant and aria-controls only with an existing target. Native attributes take precedence over ARIA.
html-conform-aria-structure-title = Use ARIA roles in the right context
html-conform-aria-structure-fix = Use roles only in a fitting context: option in listbox, row in table, grid or treegrid, listitem in list, tab in tablist, cell in row. Do not place buttons or headings inside elements whose children are presentational (role=img, button) and do not use roles inside headings or summary where they are forbidden. Use the native element where possible.
html-conform-obsolete-title = Replace obsolete HTML elements
html-conform-obsolete-fix = Replace obsolete elements such as font, center, marquee or big with semantic HTML and CSS. They are no longer developed and assistive technologies handle them inconsistently.
html-conform-microdata-title = Complete the microdata attributes
html-conform-microdata-fix = Set microdata attributes only together: itemtype, itemid and itemref need an enclosing itemscope, itemprop belongs inside an element with itemscope and itemref must point to an existing id. Alternatively deliver structured data as JSON-LD.
html-conform-meter-progress-title = Correct the value ranges of meter and progress
html-conform-meter-progress-fix = Put the values of meter and progress in a consistent order: min below max, value within that range, and for meter low below high and optimum between min and max. For progress without max the range is 0 to 1.
html-conform-sectioning-title = Nest sectioning elements correctly
html-conform-sectioning-fix = Use sectioning elements as the standard requires: only one visible main, not inside article, aside, footer, header or nav; no header or footer inside header or footer; no address inside address; dt without headings, header, footer or sectioning elements and without duplicate names; figure with figcaption instead of caption.
html-conform-media-text-title = Use media and text elements correctly
html-conform-media-text-fix = Specify media and special elements completely: track with a label and at most one default, ruby and rtc in the intended structure, bdo with dir and custom elements without an is attribute. Replace non-conforming variants with the intended markup.
html-conform-generic-title = Fix the HTML conformance violation
html-conform-generic-fix = Correct the element or attribute named in the message as the HTML Standard requires and check the rendered HTML again. If the finding appears on every page, fix the cause in the shared template.

# Count messages with singular/plural agreement (#902). Used through `I18n::count`.
count-occurrences-found = { $count ->
        [one] { $count } occurrence was found
       *[other] { $count } occurrences were found
    }
count-problem-groups = { $count ->
        [one] { $count } problem group
       *[other] { $count } problem groups
    }
count-more-findings = { $count ->
        [one] { $count } more finding
       *[other] { $count } more findings
    }
count-notes = { $count ->
        [one] { $count } note
       *[other] { $count } notes
    }
count-warnings = { $count ->
        [one] { $count } warning
       *[other] { $count } warnings
    }
count-h1-headings = { $count ->
        [one] { $count } H1 heading
       *[other] { $count } H1 headings
    }
count-link-texts-repeated = { $count ->
        [one] { $count } link text appears
       *[other] { $count } link texts appear
    }
count-links-generic = { $count ->
        [one] { $count } link carries
       *[other] { $count } links carry
    }
count-heuristic-warnings = { $count ->
        [one] { $count } heuristic accessibility warning
       *[other] { $count } heuristic accessibility warnings
    }
count-manual-points = { $count ->
        [one] { $count } point
       *[other] { $count } points
    }
count-a11y-warnings = { $count ->
        [one] { $count } accessibility warning
       *[other] { $count } accessibility warnings
    }
count-manual-hints = { $count ->
        [one] { $count } manual check
       *[other] { $count } manual checks
    }
count-recommendations = { $count ->
        [one] { $count } recommendation
       *[other] { $count } recommendations
    }
count-images-scaled-down = { $count ->
        [one] { $count } image is
       *[other] { $count } images are
    }

pdf-ai-policy-label = Policy
pdf-ai-policy-status-label = Status

module-commerce = Commerce
module-privacy = Privacy
count-findings = { $count ->
        [one] { $count } finding
       *[other] { $count } findings
    }
pdf-metric-issues = Issues
pdf-sec-headers-title = Security Headers
pdf-screenshot-alt-desktop = Desktop screenshot preview
pdf-screenshot-alt-mobile = Mobile screenshot preview

seo-cookie-inventory =
    { $count } { $count ->
        [one] cookie
       *[other] cookies
    }
seo-cookie-inventory-risky =
    { $count } { $count ->
        [one] without Secure/SameSite signal
       *[other] without Secure/SameSite signal
    }
