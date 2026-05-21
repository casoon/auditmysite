label-certificate = Certificate
section-summary = Executive Summary
section-methodology = Scope and Methodology
section-modules = Module Scores
section-findings-overview = Issues Overview
section-findings = Detected Issues
section-findings-technical = Technical Details
section-actions = Action Plan
section-appendix = Appendix: Technical Details
callout-limitations-title = Limits of Automated Testing
callout-note-title = Note
callout-no-issues-title = Excellent Accessibility
callout-no-issues-body = No automatically detectable accessibility issues were found.
certificate-thresholds = Certificate levels (overall score): SEHR GUT from 95, GUT from 85, SOLIDE from 75, AUSBAUFÄHIG from 65, below that UNGENÜGEND.
label-priority = Priority
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
effort-quick = Quick Win
effort-medium = Medium-term
effort-structural = Structural
severity-critical = Critical
severity-high = High
severity-medium = Medium
severity-low = Low
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
metric-certificate = Certificate
panel-quick-actions = Immediate actions
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
narrative-cover-kicker = Technical website check focused on accessibility, SEO and performance
narrative-status-title = Site status
narrative-metrics-title = Executive snapshot
narrative-key-points-title = Key points
narrative-impact-title = Impact
narrative-quick-actions-title = Recommended immediate actions
narrative-spotlight-eyebrow = MAIN ISSUE
narrative-leverage-title = Effect of remediation
narrative-findings-title = Key findings
narrative-action-plan-title = Action plan
narrative-action-plan-intro = Prioritized by impact and effort. Each action is clearly scoped and ready to plan.
narrative-action-plan-callout-title = Recommended approach
narrative-action-plan-callout-body = Start with the quick wins: high impact at low effort. The table below lists all actions in the recommended order.
narrative-technical-title = Technical implementation
narrative-technical-intro = From here onward you find the concrete implementation for development, design and editorial. Each issue includes affected elements, direct fix, and code examples.
narrative-next-steps-title = Recommended next steps
narrative-next-steps-intro = Concrete recommendation for the next 1–4 weeks.
narrative-next-steps-callout-title = Next step
narrative-next-steps-callout-body = For a complete accessibility verification we additionally recommend a manual audit with assistive technologies (screen reader, keyboard navigation).
narrative-findings-intro-strong = The site is technically strong. The following items are last optimization levers without structural pressure.
narrative-findings-intro-solid = Solid foundation — the following items are targeted improvement levers.
narrative-findings-intro-default = The following issues have the largest impact on usability and risk. Technical details follow in the next section.

# Verdict (single audit)
verdict-tier-excellent = { $url } reaches { $score }/100 in the accessibility audit. The remaining findings are last optimization levers — not a structural problem but polish.
verdict-tier-solid = { $url } reaches { $score }/100 in the accessibility audit. The foundation is solid — clear improvement levers at manageable effort.
verdict-tier-deficient = { $url } reaches { $score }/100 in the accessibility audit. There are significant barriers — not isolated details but structural backlog.
verdict-tier-critical = { $url } only reaches { $score }/100 in the accessibility audit. Urgent action needed: essential content and functions are not accessible for a part of users.
score-note-high-with-critical = The score weighs frequency and severity. Individual critical topics can persist despite a high overall score.

# Verdict (batch audit)
verdict-batch-excellent = Across { $total_urls } audited URLs the site reaches an overall score of { $score }/100 — a very good result.
verdict-batch-solid = On average the { $total_urls } audited URLs reach an overall score of { $score }/100. The foundation is solid, but recurring issues exist in individual modules.
verdict-batch-deficient = The { $total_urls } audited URLs reach on average only { $score }/100 points. There are significant systematic problems.
verdict-batch-critical = The { $total_urls } audited URLs reach on average only { $score }/100 points. Urgent action required across multiple modules.

# Site state (audit summary)
site-state-polished = Strong
site-state-needs-work = Solid foundation
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
business-consequence-clean = No known barriers — solid foundation for all user groups.
business-consequence-severe = Large parts of the site are unusable or barely usable for certain user groups.
business-consequence-seo-headings = The site is harder to find and structurally inaccessible to part of users.
business-consequence-screenreader = Individual key functions are blocked or unreliable for screen-reader users.
business-consequence-default = Usability is given — targeted improvements raise quality and reach.
consequence-severe = New content and features inherit the existing defects — remediation effort grows with every extension.
consequence-many-weak-modules = Effort for later corrections grows significantly — especially during a relaunch or major content expansion.
consequence-stable = No urgent pressure to act. Regular checks preserve the level after updates and extensions.
consequence-default = Without correction the site stays below the achievable standard — improvement potential is left on the table.

# Cover score row
cover-card-certificate = Certificate
cover-card-accessibility = Accessibility
cover-card-issues = Issues
cover-card-critical-high-suffix = critical/high
cover-card-average = Average
cover-card-urls = URLs
cover-card-violations-suffix = violations
batch-cover-eyebrow = Automated batch audit report
batch-cover-title = Accessibility audit report
batch-cover-kicker = Domain-wide website check focused on accessibility, SEO and performance
batch-cover-frame-title = Audit scope
batch-cover-frame-domain = Domain
batch-cover-frame-date = Audit date
batch-cover-frame-urls = Audited URLs
batch-cover-frame-certificate = Certificate
batch-cover-frame-modules = Active modules
batch-cover-frame-version = Tool version
batch-cover-frame-scope = Coverage
batch-scope-sample = Sample — { $audited } of { $total } URLs ({ $source }, first { $audited })
batch-scope-full = Complete — all { $total } URLs ({ $source })
batch-source-sitemap = sitemap
batch-source-crawl = crawl
batch-source-url_file = URL list
panel-modules-overview = Module overview
section-tech-detail-metrics = Technical detail metrics
column-action = Action
column-priority = Priority
column-timeframe = Timeframe
column-component = Component
column-area = Area
section-quick-wins = Quick wins
section-medium-actions = Medium-term actions
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
batch-col-relevance = Relevance
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
batch-url-ranking-intro = All audited URLs, sorted by score. Lower-scoring URLs need attention first.
batch-render-blocking-section = Render blocking & assets
batch-render-blocking-kv-title = Render-blocking overview (domain-wide)
batch-render-blocking-intro = Render-blocking resources and third-party traffic, aggregated across all audited pages.
batch-section-tech-url-matrix = Technical URL matrix
batch-section-tech-url-matrix-intro = Condensed overview of all audited URLs focused on technical prioritization. Each row shows score, issue intensity, and the biggest lever for the next optimization round.
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
diagnosis-table-clusters = Problem clusters
diagnosis-col-category = Category
diagnosis-col-findings = Findings
diagnosis-col-worst-severity = Worst severity
diagnosis-col-occurrences = Occurrences
diagnosis-col-max-severity = Max. severity

# Batch module overview
batch-panel-module-averages = Module averages (avg. across all URLs)

# Heuristic indicator (shared by UX, Journey, AI Visibility)
label-heuristic-indicator = Heuristic estimate based on structural signals

# Performance section
perf-score-card = Performance score
perf-technical-indicators = Technical indicators
perf-lab-data-note = Lab data
perf-lab-data-body = All performance values come from a local headless measurement (Chrome/CDP), not from field data (CrUX/real-user monitoring). Metrics marked "lab estimate" (INP, TTI, Speed Index) are derived approximations, not direct measurements.
perf-render-blocking-analysis = Render-blocking analysis

# SEO section
seo-score-card = Technical SEO
seo-score-card-description = Measures technical signals (meta, structure, schema, hreflang). Content depth is evaluated separately.
seo-maturity = Maturity

# UX section
ux-score-card = UX score (indicator)
ux-dimensions = UX dimensions

# Journey section
journey-score-card = Journey score (indicator)
journey-page-type-dimensions = Page type & dimensions

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
finding-wrong = ✕ Wrong
finding-right = ✓ Correct
finding-location = Location
finding-note = Note
finding-representative-occurrences = Representative occurrences
finding-occurrence = Occurrence
finding-suggested-fix = Suggested code fix
finding-pattern = Pattern
finding-frequent-patterns = Frequent patterns
finding-reference = Reference

# SEO / Tracking section
seo-tracking-services = Tracking and external services
seo-kv-title = Technical SEO
seo-serp-readiness = SERP readiness
seo-serp-signals = SERP signals
seo-page-health-issues = Detected issues
seo-page-health-url-analysis = URL analysis
seo-page-html-validation = HTML validation

# Security section
security-score-card = Security score

# Mobile section
mobile-score-card = Mobile score
mobile-configured = Configured
mobile-touch-targets = Touch targets
mobile-viewport-config = Viewport configuration
mobile-font-analysis = Font analysis
mobile-content-sizing = Content sizing

# AI Visibility section
ai-score-card = AI visibility (indicator)
