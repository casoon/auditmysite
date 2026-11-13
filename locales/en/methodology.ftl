# Texts of the single report's methodology section and the score
# methodology of its appendix (`output::builder::single::methodology`, #936).
# The builder keeps the thresholds, which message applies and how lists are
# joined. The audit-fact labels WCAG level, Checked nodes, Runtime, Audit
# notes, Display mode and Active modules reuse `scope-box-*` and
# `batch-cover-frame-*` of report.ftl, the module name in the weight fallback
# `module-accessibility`.

## Appendix
methodology-score-methodology = Score calculation: each accessibility viewport starts at 100 points; taxonomy-defined, rule-specific penalties with logarithmic scaling account for repeated violations. With desktop and mobile data, the canonical accessibility score is 70% mobile plus 30% desktop. The consolidated finding list supports evidence and remediation and does not create a third score. Conflicting signals are flagged but do not alter the score retrospectively.

## Methodology block
methodology-scope = Automated audit of { $url } for accessibility per WCAG 2.2 (level { $level }). Requested modules: { $modules }.
methodology-method = The audit was performed via the Chrome DevTools Protocol (CDP) and the browser's native accessibility tree. { $rules } automated rule executions were recorded across the configured viewports.
methodology-limitations = Automated tests can detect about 30–40% of all accessibility issues. Complex aspects such as correct tab order, meaningful alt texts, or understandable language additionally require manual review.
# Appended to the limitations after a space.
methodology-limitations-incomplete = This run is incomplete: scores describe only the successfully measured scope.
methodology-disclaimer = This report represents an automated technical analysis. It does not replace a complete WCAG 2.2 conformance assessment. A legally defensible accessibility statement requires a comprehensive manual audit by experts. Accessibility is not a one-time achievable state: it is an ongoing process, and every new component or piece of content needs to be re-audited when it is added.

## Audit facts
methodology-fact-accessibility-score = Accessibility score
methodology-fact-combined-score = Combined technical score
methodology-fact-counting = Counting method
methodology-fact-page-display-modes = Page display modes
methodology-fact-failed-modules = Failed modules
methodology-fact-audit-quality = Audit quality
methodology-fact-preview = Preview
methodology-accessibility-score-viewports = { $score } / 100 — 70% mobile ({ $mobile }) + 30% desktop ({ $desktop })
methodology-combined-score = { $score } / 100 — contributing weights: { $weights }
methodology-weight = { $module } { $pct }%
# Appended to the combined score after a space.
methodology-indicator-modules = Indicator modules shown separately: { $modules }.
methodology-counting = { $groups } distinct WCAG finding groups, { $occurrences } WCAG occurrences; { $rows } finding rows and { $all } occurrences across all categories
methodology-failed-modules-none = None
methodology-quality-complete = Complete
methodology-quality-partial = Partial
methodology-quality-insufficient = Insufficient
methodology-preview-captured = Desktop and mobile captured
methodology-preview-not-captured = Not captured

## Confidence summary
methodology-label-confidence = Audit confidence
methodology-label-coverage = Module coverage
methodology-label-signals = Conflict signals
methodology-label-manual = Manual review needed
methodology-manual-review = Yes, for semantic quality and usage context
methodology-confidence-high = High
methodology-confidence-solid = Solid
methodology-confidence-limited = Limited
methodology-confidence-off = Not active
methodology-signals-none = No automatically detected conflict signals
methodology-signals-one = 1 caveat signal
methodology-signals-multiple = Multiple caveat signals
methodology-coverage-broad = Broad
methodology-coverage-extended = Extended
methodology-coverage-core = Core checks

## Capability matrix
methodology-capability-wcag = WCAG rules & occurrences
methodology-capability-wcag-source = Accessibility tree + rule engine
methodology-capability-wcag-note = Primary audit truth for automatically detectable violations.
methodology-capability-performance = Web vitals & loading indicators
methodology-capability-performance-source = Performance module
methodology-capability-performance-note = FCP, CLS and TTFB are reflected in facts and module sections.
methodology-capability-seo = SEO structure & schema
methodology-capability-seo-source = SEO module
methodology-capability-seo-note = Meta, heading and schema signals are condensed into a report-ready form.
methodology-capability-security = Security headers & HTTPS
methodology-capability-security-source = Security module
methodology-capability-security-note = Header presence and HTTPS status remain visible as raw signal.
methodology-capability-heuristic = Mobile, UX, journey
methodology-capability-heuristic-source = Heuristic modules
methodology-capability-heuristic-confidence = Indicator-based
methodology-capability-heuristic-note = Useful for prioritization, not the sole UX truth.
methodology-capability-flags = Audit conflict signals
methodology-capability-flags-source = Normalization / cross-checks
methodology-capability-flags-confidence = Explicitly flagged
methodology-capability-flags-note = { $count } conflict { $count ->
        [one] signal is
       *[other] signals are
    } surfaced openly rather than hidden in the score.
