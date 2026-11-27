use super::*;

const SINGLE_OLD: &str = include_str!("../../tests/fixtures/compare/single-old.json");
const SINGLE_BETTER: &str = include_str!("../../tests/fixtures/compare/single-new-better.json");
const SINGLE_WORSE: &str = include_str!("../../tests/fixtures/compare/single-new-worse.json");
const BATCH_OLD: &str = include_str!("../../tests/fixtures/compare/batch-old.json");
const BATCH_NEW: &str = include_str!("../../tests/fixtures/compare/batch-new.json");

fn report(json: &str) -> Report {
    Report::parse(json.as_bytes(), "test.json").expect("fixture parses")
}

fn rules(list: &[FindingRef]) -> Vec<&str> {
    list.iter().map(|f| f.rule.as_str()).collect()
}

#[test]
fn a_better_run_resolves_findings_and_moves_the_scores() {
    let c = compare(&report(SINGLE_OLD), &report(SINGLE_BETTER));
    let page = &c.pages[0];
    assert_eq!(rules(&page.findings.resolved), ["images/alt-missing"]);
    assert_eq!(rules(&page.findings.unchanged), ["buttons/name-missing"]);
    assert!(page.findings.new.is_empty());

    assert_eq!(c.overall.old, Some(75.0));
    assert_eq!(c.overall.new, Some(81.0));
    assert_eq!(c.overall.delta, Some(6.0));
    assert_eq!(c.accessibility.delta, Some(12.0));
    let seo = c.modules.iter().find(|m| m.name == "SEO").unwrap();
    assert_eq!((seo.score.old, seo.score.new), (Some(70.0), Some(72.0)));

    let verdict = c.verdict.as_ref().unwrap();
    assert_eq!(
        (verdict.old.as_str(), verdict.new.as_str()),
        ("fail", "warn")
    );
    assert!(verdict.changed);

    assert_eq!(
        (c.summary.resolved, c.summary.new, c.summary.unchanged),
        (1, 0, 1)
    );
    assert!(!c.summary.worse);
    assert_eq!(c.exit_code(), 0);
}

#[test]
fn a_new_finding_or_a_lost_module_score_makes_the_run_worse() {
    let c = compare(&report(SINGLE_OLD), &report(SINGLE_WORSE));
    assert_eq!(
        rules(&c.pages[0].findings.new),
        ["contrast/text-insufficient"]
    );
    assert_eq!(rules(&c.pages[0].findings.resolved), ["images/alt-missing"]);
    assert_eq!(c.summary.reasons, ["new_findings"]);
    assert_eq!(c.exit_code(), 2);
    let seo = c.modules.iter().find(|m| m.name == "SEO").unwrap();
    assert_eq!(seo.score.delta, Some(-6.0));
    // The verdict is reported, it does not decide the exit code.
    assert!(!c.verdict.as_ref().unwrap().changed);
}

#[test]
fn a_lower_overall_score_alone_is_worse() {
    let mut worse = report(SINGLE_BETTER);
    worse.pages[0].site_quality_score = 70;
    let c = compare(&report(SINGLE_OLD), &worse);
    assert!(c.pages[0].findings.new.is_empty());
    assert_eq!(c.summary.reasons, ["score_loss"]);
    assert_eq!(c.exit_code(), 2);
}

#[test]
fn identical_reports_are_unchanged_and_not_worse() {
    let c = compare(&report(SINGLE_OLD), &report(SINGLE_OLD));
    assert_eq!(
        (c.summary.resolved, c.summary.new, c.summary.unchanged),
        (0, 0, 2)
    );
    assert_eq!(c.overall.delta, Some(0.0));
    assert!(!c.summary.worse);
    assert_eq!(c.exit_code(), 0);
}

#[test]
fn a_batch_is_compared_per_url_present_in_both() {
    let c = compare(&report(BATCH_OLD), &report(BATCH_NEW));
    let urls: Vec<&str> = c.pages.iter().map(|p| p.url.as_str()).collect();
    assert_eq!(urls, ["https://example.com/", "https://example.com/about"]);
    assert_eq!(c.only_in_old, ["https://example.com/gone"]);
    assert_eq!(c.only_in_new, ["https://example.com/contact"]);
    // Averages over the two pages in both runs: (75 + 90) / 2 and (81 + 90) / 2.
    assert_eq!((c.overall.old, c.overall.new), (Some(82.5), Some(85.5)));
    // Findings of a URL in one report only enter no count.
    assert_eq!(
        (c.summary.resolved, c.summary.new, c.summary.unchanged),
        (1, 0, 1)
    );
    assert_eq!(c.summary.pages_compared, 2);
    assert!(!c.summary.worse);
}

#[test]
fn a_single_report_is_compared_with_a_batch_by_url() {
    let c = compare(&report(SINGLE_OLD), &report(BATCH_NEW));
    assert_eq!(c.pages.len(), 1);
    assert_eq!(c.pages[0].url, "https://example.com/");
    assert_eq!(c.only_in_new.len(), 2);
}

#[test]
fn two_single_reports_of_different_urls_are_one_pair() {
    let mut other = report(SINGLE_BETTER);
    other.pages[0].url = "https://example.org/".to_string();
    let c = compare(&report(SINGLE_OLD), &other);
    assert_eq!(c.pages.len(), 1);
    assert_eq!(c.pages[0].url, "https://example.org/");
    assert_eq!(c.pages[0].old_url.as_deref(), Some("https://example.com/"));
}

#[test]
fn a_module_missing_on_one_side_has_no_delta() {
    let mut new = report(SINGLE_BETTER);
    new.pages[0]
        .module_scores
        .retain(|m| m.name != "Performance");
    let c = compare(&report(SINGLE_OLD), &new);
    let perf = c.modules.iter().find(|m| m.name == "Performance").unwrap();
    assert_eq!(perf.score.old, None);
    assert_eq!(perf.score.delta, None);
    let page_perf = c.pages[0]
        .modules
        .iter()
        .find(|m| m.name == "Performance")
        .unwrap();
    assert_eq!(
        (page_perf.score.old, page_perf.score.new),
        (Some(60.0), None)
    );
}

fn error_of(json: &str) -> String {
    Report::parse(json.as_bytes(), "x.json")
        .unwrap_err()
        .to_string()
}

#[test]
fn malformed_input_is_refused_with_its_reason() {
    assert!(error_of("not json").contains("not an auditmysite JSON report"));
    assert!(error_of("{}").contains("not an auditmysite JSON report"));
    let unknown = SINGLE_OLD.replace("\"single\"", "\"weekly\"");
    assert!(error_of(&unknown).contains("neither \"single\" nor \"batch\""));
    let two = BATCH_OLD.replace("\"batch\"", "\"single\"");
    assert!(error_of(&two).contains("a single report has one page"));
    let twice = BATCH_OLD.replace("example.com/about", "example.com/gone");
    assert!(error_of(&twice).contains("listed twice"));
}

#[test]
fn a_finding_id_this_tool_would_not_derive_is_refused() {
    let moved = SINGLE_OLD.replace("F-12a4e078", "F-00000000");
    let message = error_of(&moved);
    assert!(message.contains("finding id scheme"), "{message}");
    assert!(message.contains("F-00000000"), "{message}");
    assert!(message.contains("images/alt-missing"), "{message}");
}

#[test]
fn the_pinned_fixture_ids_are_the_ids_of_the_rules() {
    use crate::audit::baseline::finding_id_of;
    assert_eq!(finding_id_of("buttons/name-missing"), "F-84ea8500");
    assert_eq!(finding_id_of("images/alt-missing"), "F-12a4e078");
}

#[test]
fn the_json_form_carries_the_contract_and_the_counts() {
    let c = compare(&report(SINGLE_OLD), &report(SINGLE_WORSE));
    let json = crate::output::compare::format_compare_json(&c);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["contract"], COMPARE_CONTRACT);
    assert_eq!(value["id_scheme"], FINDING_ID_SCHEME);
    assert_eq!(value["summary"]["worse"], true);
    assert_eq!(
        value["pages"][0]["findings"]["new"][0]["rule"],
        "contrast/text-insufficient"
    );
    assert_eq!(
        value["modules"][0]["delta"],
        serde_json::json!(c.modules[0].score.delta)
    );
}
