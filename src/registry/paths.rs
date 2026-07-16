//! Interpreting `MetricSpec::json_path` strings against real data.
//!
//! `json_path` was ported verbatim from the pre-registry `metric_context()`
//! text (#506 Phase 1 step 1), so some entries are a single dotted/bracketed
//! path (`"pages[].risk.score"`) and others are a compound, human-readable
//! description (`"summary.grade / summary.certificate"`,
//! `"pages[].detail.modules.*.score and nested dimension scores"`). Both
//! `tests/registry_contract.rs` (checks paths against the JSON *schema*) and
//! `crate::lint` (checks paths against a real report *instance*) need the
//! same first step: pull the real path-like candidates out of that text.

/// Extract path-like candidates from a (possibly compound/prose) `json_path`
/// string, e.g. `"summary.grade / summary.certificate"` yields
/// `["summary.grade", "summary.certificate"]`, and
/// `"pages[].detail.modules.*.score and nested dimension scores"` yields
/// `["pages[].detail.modules.*.score"]` (trailing prose is dropped).
pub fn json_path_candidates(json_path: &str) -> Vec<String> {
    // Only " / " ever separates two distinct real paths in this codebase's
    // json_path text; " and " only ever introduces trailing prose after a
    // single real path, so take_while's stop-at-first-invalid-char below is
    // enough to drop it — splitting on " and " too would wrongly turn that
    // prose into a second bogus candidate.
    json_path
        .split(" / ")
        .map(|segment| {
            segment
                .trim()
                .chars()
                .take_while(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '[' | ']' | '*'))
                .collect::<String>()
        })
        .filter(|segment| !segment.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_compound_paths() {
        assert_eq!(
            json_path_candidates("summary.grade / summary.certificate"),
            vec!["summary.grade", "summary.certificate"]
        );
    }

    #[test]
    fn drops_trailing_prose() {
        assert_eq!(
            json_path_candidates("pages[].detail.modules.*.score and nested dimension scores"),
            vec!["pages[].detail.modules.*.score"]
        );
    }

    #[test]
    fn single_path_is_unchanged() {
        assert_eq!(
            json_path_candidates("pages[].risk.score"),
            vec!["pages[].risk.score"]
        );
    }
}
