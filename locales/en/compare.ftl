# Texts of `auditmysite compare` (#1235): the table and the PDF of the before/after comparison, `output::compare`.
compare-title = Before/after comparison
compare-run-old = Old run
compare-run-new = New run
compare-run-single = Single report
compare-run-batch = Batch report
compare-run-pages =
    { $count ->
        [one] 1 page
       *[other] { $count } pages
    }
compare-run-tool = auditmysite { $version }
compare-pages-compared =
    { $count ->
        [one] 1 page compared
       *[other] { $count } pages compared
    }
compare-scores = Scores
compare-scores-average = Scores (average over the pages compared)
compare-col-score = Score
compare-col-old = Old
compare-col-new = New
compare-col-change = Change
compare-col-page = Page
compare-col-findings = Findings
compare-score-overall = Overall (site quality)
compare-score-accessibility = Accessibility
compare-verdict = Verdict
compare-verdict-pass = pass
compare-verdict-warn = warn
compare-verdict-fail = fail
compare-verdict-unchanged = { $verdict } (unchanged)
compare-verdict-changed = { $old } to { $new }
compare-findings = Findings
compare-findings-counts = { $resolved } resolved, { $new } new, { $unchanged } unchanged
compare-resolved = Resolved
compare-new = New
compare-unchanged = Unchanged
compare-none = none
compare-page-old-url = old run: { $url }
compare-pages = Pages
compare-only-old = Only in the old report
compare-only-new = Only in the new report
compare-reason-new-findings = new findings
compare-reason-score-loss = score loss
compare-result-worse = Result: the new run is worse ({ $reasons })
compare-result-ok = Result: no new findings, no score loss
compare-note = Findings are matched by their stable id (one rule on one page), not by element: a new element violating a known rule is not a new finding.
compare-more =
    { $count ->
        [one] and 1 more
       *[other] and { $count } more
    }
