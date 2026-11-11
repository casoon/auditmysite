//! Text export of the SEO table labels (#936): every text of
//! `schema::schema_issue_label`, `robots::BotClass::label` (and its `Display`),
//! `robots::infer_robots_policy` and `robots::localize_policy_label` in both
//! languages across an enumerated input space, as records in
//! `reports/seo-label-texts.json` (gitignored; the path can be set with
//! `SEO_LABEL_TEXT_EXPORT`). Run it before and after a change of how the texts
//! are stored and compare the two files byte for byte.

use super::issue_texts_export::schema_issues;
use super::robots::{
    infer_robots_policy, localize_policy_label, BotClass, RobotsAudit, RobotsGroup,
};
use super::schema::schema_issue_label;
use serde_json::json;

pub(super) const BOT_CLASSES: [BotClass; 8] = [
    BotClass::Wildcard,
    BotClass::SearchEngine,
    BotClass::AiTraining,
    BotClass::AiCitation,
    BotClass::AiMixed,
    BotClass::UnknownAi,
    BotClass::General,
    BotClass::Unknown,
];

fn group(bot_class: BotClass, blocked: bool) -> RobotsGroup {
    RobotsGroup {
        user_agent: format!("{bot_class:?}"),
        bot_class,
        allows: vec![],
        disallows: if blocked {
            vec!["/".to_string()]
        } else {
            vec!["/private/".to_string()]
        },
    }
}

/// Every combination of the flags and group kinds `infer_robots_policy` reads.
pub(super) fn robots_audits() -> Vec<(String, RobotsAudit)> {
    let mut out = Vec::new();
    for wildcard in [false, true] {
        for citation in [false, true] {
            for training in [false, true] {
                for groups in 0..6usize {
                    let groups = match groups {
                        0 => vec![],
                        1 => vec![group(BotClass::AiMixed, true)],
                        2 => vec![group(BotClass::AiMixed, false)],
                        3 => vec![group(BotClass::AiTraining, false)],
                        4 => vec![group(BotClass::AiCitation, false)],
                        _ => vec![
                            group(BotClass::SearchEngine, true),
                            group(BotClass::UnknownAi, true),
                        ],
                    };
                    let label = format!(
                        "wildcard={wildcard} citation={citation} training={training} groups={:?}",
                        groups
                            .iter()
                            .map(|g| (g.bot_class, g.disallows.clone()))
                            .collect::<Vec<_>>()
                    );
                    out.push((
                        label,
                        RobotsAudit {
                            has_wildcard_disallow_all: wildcard,
                            blocks_ai_citation: citation,
                            blocks_ai_training: training,
                            groups,
                            ..Default::default()
                        },
                    ));
                }
            }
        }
    }
    out
}

/// Stored policy strings `localize_policy_label` gets besides the canonical
/// English labels: text it passes through unchanged.
pub(super) const OTHER_POLICIES: [&str; 6] = [
    "",
    "something else",
    "KI-Zugang offen",
    "AI access open ",
    "ai access open",
    "{braces} and {$var}",
];

#[test]
#[ignore = "writes reports/seo-label-texts.json; run before and after a change of the text storage"]
fn export_seo_label_texts() {
    let mut schema = Vec::new();
    for issue in schema_issues() {
        for (lang, en) in [("en", true), ("de", false)] {
            schema.push(json!([
                issue.issue_type,
                issue.schema_type,
                lang,
                schema_issue_label(&issue, en)
            ]));
        }
    }
    let mut bot_classes = Vec::new();
    for class in BOT_CLASSES {
        for (lang, en) in [("en", true), ("de", false)] {
            bot_classes.push(json!([format!("{class:?}"), lang, class.label(en)]));
        }
        bot_classes.push(json!([format!("{class:?}"), "display", class.to_string()]));
    }
    let mut policies = Vec::new();
    let mut canonical = Vec::new();
    for (label, audit) in robots_audits() {
        for (lang, en) in [("en", true), ("de", false)] {
            let policy = infer_robots_policy(&audit, en);
            if en && !canonical.contains(&policy) {
                canonical.push(policy.clone());
            }
            policies.push(json!([label, lang, policy]));
        }
    }
    let mut localized = Vec::new();
    for stored in canonical.iter().map(String::as_str).chain(OTHER_POLICIES) {
        for (lang, en) in [("en", true), ("de", false)] {
            localized.push(json!([stored, lang, localize_policy_label(stored, en)]));
        }
    }
    let path = std::env::var("SEO_LABEL_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/seo-label-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    let export = json!({
        "schema_labels": schema,
        "bot_classes": bot_classes,
        "policies": policies,
        "localized_policies": localized,
    });
    std::fs::write(&path, serde_json::to_string_pretty(&export).unwrap()).unwrap();
    println!(
        "{} schema label, {} bot class, {} policy and {} localized policy records written to {path}",
        schema.len(),
        bot_classes.len(),
        policies.len(),
        localized.len()
    );
}
