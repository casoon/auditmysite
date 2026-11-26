//! The analysis card of a finding (#1171): the one place outside the evidence
//! level where a finding's problem, impact and recommendation are told in
//! full. The single report sets its cards under their cause, the batch report
//! sets one per recurring problem; the rows are the same.

use renderreport::components::advanced::KeyValueList;

use super::findings::report_code_label;
use super::helpers::{effort_short_label_i18n, priority_label_i18n, role_label_i18n};
use crate::i18n::I18n;
use crate::output::report_model::FindingGroup;

/// What the caller knows about a card and the group does not.
pub(super) struct AnalysisCard {
    /// How far the finding reaches, as the caller counts it.
    pub(super) reach: String,
    /// The cause and how sure the report is about it, when the card names it
    /// itself; the single report names it once for the group of cards.
    pub(super) cause: Option<String>,
    /// Rows between the cause and the problem, such as the origin of the markup.
    pub(super) context_rows: Vec<(String, String)>,
    /// Points the score would gain from the fix, when quantified.
    pub(super) score_effect: Option<String>,
}

/// The problem and user-impact lines of a finding. An HTML conformance
/// finding's text is one occurrence's checker message, with its line:column:
/// page detail, not a statement about the rule. A rule without its own
/// user-impact text repeats the problem there. Both are left out rather than
/// printed as page detail or twice (#1039).
fn problem_and_impact(group: &FindingGroup) -> (Option<&str>, Option<&str>) {
    if group.finding.rule_id.starts_with("html_conform.") {
        return (None, None);
    }
    let problem = Some(group.customer_description.as_str()).filter(|t| !t.trim().is_empty());
    let user_impact = Some(group.user_impact.as_str())
        .filter(|t| !t.trim().is_empty() && Some(t.trim()) != problem.map(str::trim));
    (problem, user_impact)
}

/// A row of [`RETESTS`]: which rule ids it covers, the Fluent key of the check
/// and whether the other occurrences are checked the same way.
struct Retest(fn(&str) -> bool, &'static str, bool);

/// How the fix of a rule is checked, by the kind of rule (#1211): the Fluent
/// key of the check and whether the other occurrences are checked the same
/// way. The first row that matches the rule id decides; the last row catches
/// every rule without a check of its own.
const RETESTS: &[Retest] = &[
    Retest(
        |r| r.starts_with("html_conform."),
        "pdf-analysis-retest-validator",
        false,
    ),
    Retest(
        |r| r.starts_with("security."),
        "pdf-analysis-retest-headers",
        false,
    ),
    Retest(|r| r.starts_with("seo."), "pdf-analysis-retest-seo", false),
    Retest(
        |r| {
            [
                "names/",
                "buttons/",
                "links/",
                "svg/",
                "summary/",
                "label-in-name",
            ]
            .iter()
            .any(|p| r.starts_with(p))
                || r.ends_with("name-missing")
                || r.ends_with("label-missing")
        },
        "pdf-analysis-retest-name",
        true,
    ),
    Retest(
        |r| r.starts_with("contrast/") || r.starts_with("color/"),
        "pdf-analysis-retest-contrast",
        true,
    ),
    Retest(
        |r| r.starts_with("images/") || r.starts_with("media-alt") || r.starts_with("objects/"),
        "pdf-analysis-retest-alt",
        true,
    ),
    Retest(
        |r| r.starts_with("headings/"),
        "pdf-analysis-retest-headings",
        false,
    ),
    Retest(
        |r| r.starts_with("landmarks/"),
        "pdf-analysis-retest-landmarks",
        false,
    ),
    Retest(
        |r| r.starts_with("document/lang") || r.starts_with("language/") || r == "html-has-lang",
        "pdf-analysis-retest-lang",
        false,
    ),
    Retest(|r| r.starts_with("ids/"), "pdf-analysis-retest-ids", false),
    Retest(
        |r| r.starts_with("keyboard") || r.starts_with("focus"),
        "pdf-analysis-retest-keyboard",
        true,
    ),
    Retest(|r| r.starts_with("aria/"), "pdf-analysis-retest-aria", true),
    Retest(
        |r| r.starts_with("tables/") || r.starts_with("lists/"),
        "pdf-analysis-retest-structure",
        true,
    ),
    Retest(
        |r| r.starts_with("forms/"),
        "pdf-analysis-retest-forms",
        true,
    ),
    Retest(
        |r| {
            [
                "zoom/",
                "orientation/",
                "text-spacing/",
                "targets/",
                "display/",
            ]
            .iter()
            .any(|p| r.starts_with(p))
        },
        "pdf-analysis-retest-layout",
        true,
    ),
    Retest(|_| true, "pdf-analysis-retest-generic", false),
];

/// The short check after the fix, from the rule of the finding: one sentence
/// for the changed element and, where the finding has several, the same check
/// for the others.
fn retest_text(group: &FindingGroup, i18n: &I18n) -> String {
    let rule = group.finding.rule_id.as_str();
    let Retest(_, key, with_others) = RETESTS
        .iter()
        .find(|Retest(matches, _, _)| matches(rule))
        .expect("the last row matches every rule");
    let mut text = i18n.t(key);
    let others = group.finding.occurrence_count.saturating_sub(1);
    if *with_others && others > 0 {
        text.push(' ');
        text.push_str(&i18n.t_args("pdf-analysis-retest-others", &[("others", others as i64)]));
    }
    text
}

/// One card: reach, cause, problem, impact with its evidence status, fix,
/// the check after the fix, priority with effort and owner, finding id.
pub(super) fn analysis_card(group: &FindingGroup, card: AnalysisCard, i18n: &I18n) -> KeyValueList {
    let mut kv = KeyValueList::new()
        .with_title(&group.title)
        .add(i18n.t("pdf-analysis-card-key-reach"), card.reach);
    if let Some(cause) = card.cause {
        kv = kv.add(i18n.t("pdf-analysis-card-key-cause"), cause);
    }
    for (key, value) in card.context_rows {
        kv = kv.add(key, value);
    }
    let (problem, impact) = problem_and_impact(group);
    if let Some(problem) = problem {
        kv = kv.add(i18n.t("pdf-analysis-card-key-problem"), problem);
    }
    if let Some(impact) = impact {
        kv = kv.add(i18n.t("pdf-analysis-card-key-impact"), impact);
    }
    if problem.is_some() || impact.is_some() {
        // How the finding was established, which bounds what the impact
        // sentence claims.
        kv = kv.add(
            i18n.t("pdf-analysis-card-key-evidence"),
            report_code_label(&group.finding.verification, i18n),
        );
    }
    if !group.recommendation.is_empty() {
        kv = kv.add(i18n.t("pdf-analysis-card-key-fix"), &group.recommendation);
    }
    kv = kv.add(
        i18n.t("pdf-analysis-card-key-retest"),
        retest_text(group, i18n),
    );
    kv = kv.add(
        i18n.t("pdf-analysis-card-key-priority"),
        i18n.t_args(
            "pdf-analysis-card-priority-effort",
            &[
                ("priority", priority_label_i18n(group.priority, i18n)),
                ("effort", effort_short_label_i18n(group.effort, i18n)),
                ("role", role_label_i18n(group.responsible_role, i18n)),
            ],
        ),
    );
    if let Some(effect) = card.score_effect {
        kv = kv.add(i18n.t("pdf-analysis-card-key-score-effect"), effect);
    }
    kv.add(i18n.t("pdf-analysis-card-key-finding"), &group.finding.id)
}

#[cfg(test)]
mod tests {
    use super::{retest_text, Retest, RETESTS};
    use crate::i18n::I18n;
    use crate::output::pdf::single_report::risk_and_strength_tests::test_report_view_model;

    fn group(rule: &str, occurrences: usize) -> crate::output::report_model::FindingGroup {
        let mut group = test_report_view_model().findings.all_findings.remove(0);
        group.finding.rule_id = rule.to_string();
        group.finding.occurrence_count = occurrences;
        group
    }

    /// #1211: the check after the fix comes from the kind of rule, names the
    /// other occurrences only where the check applies to each of them, and is
    /// a Fluent text in both languages.
    #[test]
    fn retest_follows_the_rule_and_counts_the_other_occurrences() {
        let en = I18n::new("en").unwrap();
        let de = I18n::new("de").unwrap();
        let name = retest_text(&group("links/name-missing", 7), &en);
        assert!(name.contains("accessible name"), "{name}");
        assert!(
            name.ends_with("Check the other 6 elements the same way."),
            "{name}"
        );
        let one = retest_text(&group("links/name-missing", 2), &en);
        assert!(
            one.ends_with("Check the other element the same way."),
            "{one}"
        );
        assert!(!retest_text(&group("links/name-missing", 1), &en).contains("other"));

        let ids = retest_text(&group("ids/duplicate", 20), &en);
        assert!(
            ids.contains("changed id") && !ids.contains("other"),
            "{ids}"
        );
        assert!(retest_text(&group("html_conform.schema.html5", 3), &en).contains("validator"));
        assert!(retest_text(&group("security.headers.missing_header", 8), &en).contains("curl"));
        assert!(retest_text(&group("something/unknown", 3), &en).contains("Run the audit again"));
        assert!(retest_text(&group("links/name-missing", 7), &de)
            .ends_with("Die anderen 6 Elemente ebenso prüfen."));
    }

    /// Every row's key exists in both languages and the English text has no
    /// German characters.
    #[test]
    fn every_retest_row_has_a_text_in_both_languages() {
        for Retest(_, key, _) in RETESTS {
            let en = crate::i18n::text(key, true);
            let de = crate::i18n::text(key, false);
            assert_ne!(en, *key);
            assert_ne!(de, *key);
            assert!(!en.contains(['ä', 'ö', 'ü', 'ß']), "{key}: {en}");
        }
    }
}
