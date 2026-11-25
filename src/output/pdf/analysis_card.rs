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

/// One card: reach, cause, problem, impact with its evidence status, fix,
/// priority with effort and owner, finding id.
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
