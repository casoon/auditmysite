use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BfsgMapping {
    pub en_301549_clause: &'static str,
    pub bfsg_paragraph: &'static str,
    pub fix_required: bool,
    pub deadline: &'static str,
}

pub const BFSG_PARAGRAPH_WEB: &str = "§12 Abs. 1";
pub const BFSG_DEADLINE: &str = "2025-06-28";

pub fn map_to_bfsg(wcag: &str) -> Option<BfsgMapping> {
    WCAG_21_AA_CRITERIA
        .iter()
        .find(|criterion| criterion.wcag == wcag)
        .map(|criterion| BfsgMapping {
            en_301549_clause: criterion.en_301549_clause,
            bfsg_paragraph: BFSG_PARAGRAPH_WEB,
            fix_required: true,
            deadline: BFSG_DEADLINE,
        })
}

pub fn wcag_21_aa_criteria() -> &'static [WcagCriterionMapping] {
    WCAG_21_AA_CRITERIA
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WcagCriterionMapping {
    pub wcag: &'static str,
    pub en_301549_clause: &'static str,
}

pub const WCAG_21_AA_CRITERIA: &[WcagCriterionMapping] = &[
    WcagCriterionMapping {
        wcag: "1.1.1",
        en_301549_clause: "9.1.1.1",
    },
    WcagCriterionMapping {
        wcag: "1.2.1",
        en_301549_clause: "9.1.2.1",
    },
    WcagCriterionMapping {
        wcag: "1.2.2",
        en_301549_clause: "9.1.2.2",
    },
    WcagCriterionMapping {
        wcag: "1.2.3",
        en_301549_clause: "9.1.2.3",
    },
    WcagCriterionMapping {
        wcag: "1.2.4",
        en_301549_clause: "9.1.2.4",
    },
    WcagCriterionMapping {
        wcag: "1.2.5",
        en_301549_clause: "9.1.2.5",
    },
    WcagCriterionMapping {
        wcag: "1.3.1",
        en_301549_clause: "9.1.3.1",
    },
    WcagCriterionMapping {
        wcag: "1.3.2",
        en_301549_clause: "9.1.3.2",
    },
    WcagCriterionMapping {
        wcag: "1.3.3",
        en_301549_clause: "9.1.3.3",
    },
    WcagCriterionMapping {
        wcag: "1.3.4",
        en_301549_clause: "9.1.3.4",
    },
    WcagCriterionMapping {
        wcag: "1.3.5",
        en_301549_clause: "9.1.3.5",
    },
    WcagCriterionMapping {
        wcag: "1.4.1",
        en_301549_clause: "9.1.4.1",
    },
    WcagCriterionMapping {
        wcag: "1.4.2",
        en_301549_clause: "9.1.4.2",
    },
    WcagCriterionMapping {
        wcag: "1.4.3",
        en_301549_clause: "9.1.4.3",
    },
    WcagCriterionMapping {
        wcag: "1.4.4",
        en_301549_clause: "9.1.4.4",
    },
    WcagCriterionMapping {
        wcag: "1.4.5",
        en_301549_clause: "9.1.4.5",
    },
    WcagCriterionMapping {
        wcag: "1.4.10",
        en_301549_clause: "9.1.4.10",
    },
    WcagCriterionMapping {
        wcag: "1.4.11",
        en_301549_clause: "9.1.4.11",
    },
    WcagCriterionMapping {
        wcag: "1.4.12",
        en_301549_clause: "9.1.4.12",
    },
    WcagCriterionMapping {
        wcag: "1.4.13",
        en_301549_clause: "9.1.4.13",
    },
    WcagCriterionMapping {
        wcag: "2.1.1",
        en_301549_clause: "9.2.1.1",
    },
    WcagCriterionMapping {
        wcag: "2.1.2",
        en_301549_clause: "9.2.1.2",
    },
    WcagCriterionMapping {
        wcag: "2.1.4",
        en_301549_clause: "9.2.1.4",
    },
    WcagCriterionMapping {
        wcag: "2.2.1",
        en_301549_clause: "9.2.2.1",
    },
    WcagCriterionMapping {
        wcag: "2.2.2",
        en_301549_clause: "9.2.2.2",
    },
    WcagCriterionMapping {
        wcag: "2.3.1",
        en_301549_clause: "9.2.3.1",
    },
    WcagCriterionMapping {
        wcag: "2.4.1",
        en_301549_clause: "9.2.4.1",
    },
    WcagCriterionMapping {
        wcag: "2.4.2",
        en_301549_clause: "9.2.4.2",
    },
    WcagCriterionMapping {
        wcag: "2.4.3",
        en_301549_clause: "9.2.4.3",
    },
    WcagCriterionMapping {
        wcag: "2.4.4",
        en_301549_clause: "9.2.4.4",
    },
    WcagCriterionMapping {
        wcag: "2.4.5",
        en_301549_clause: "9.2.4.5",
    },
    WcagCriterionMapping {
        wcag: "2.4.6",
        en_301549_clause: "9.2.4.6",
    },
    WcagCriterionMapping {
        wcag: "2.4.7",
        en_301549_clause: "9.2.4.7",
    },
    WcagCriterionMapping {
        wcag: "2.5.1",
        en_301549_clause: "9.2.5.1",
    },
    WcagCriterionMapping {
        wcag: "2.5.2",
        en_301549_clause: "9.2.5.2",
    },
    WcagCriterionMapping {
        wcag: "2.5.3",
        en_301549_clause: "9.2.5.3",
    },
    WcagCriterionMapping {
        wcag: "2.5.4",
        en_301549_clause: "9.2.5.4",
    },
    WcagCriterionMapping {
        wcag: "3.1.1",
        en_301549_clause: "9.3.1.1",
    },
    WcagCriterionMapping {
        wcag: "3.1.2",
        en_301549_clause: "9.3.1.2",
    },
    WcagCriterionMapping {
        wcag: "3.2.1",
        en_301549_clause: "9.3.2.1",
    },
    WcagCriterionMapping {
        wcag: "3.2.2",
        en_301549_clause: "9.3.2.2",
    },
    WcagCriterionMapping {
        wcag: "3.2.3",
        en_301549_clause: "9.3.2.3",
    },
    WcagCriterionMapping {
        wcag: "3.2.4",
        en_301549_clause: "9.3.2.4",
    },
    WcagCriterionMapping {
        wcag: "3.3.1",
        en_301549_clause: "9.3.3.1",
    },
    WcagCriterionMapping {
        wcag: "3.3.2",
        en_301549_clause: "9.3.3.2",
    },
    WcagCriterionMapping {
        wcag: "3.3.3",
        en_301549_clause: "9.3.3.3",
    },
    WcagCriterionMapping {
        wcag: "3.3.4",
        en_301549_clause: "9.3.3.4",
    },
    WcagCriterionMapping {
        wcag: "4.1.1",
        en_301549_clause: "9.4.1.1",
    },
    WcagCriterionMapping {
        wcag: "4.1.2",
        en_301549_clause: "9.4.1.2",
    },
    WcagCriterionMapping {
        wcag: "4.1.3",
        en_301549_clause: "9.4.1.3",
    },
];

#[cfg(test)]
mod tests {
    use super::{map_to_bfsg, wcag_21_aa_criteria};

    #[test]
    fn maps_required_wcag_examples() {
        let mapping = map_to_bfsg("1.1.1").expect("mapped");
        assert_eq!(mapping.en_301549_clause, "9.1.1.1");
        assert_eq!(mapping.bfsg_paragraph, "§12 Abs. 1");
        assert!(mapping.fix_required);
        assert_eq!(mapping.deadline, "2025-06-28");

        assert_eq!(
            map_to_bfsg("4.1.2").expect("mapped").en_301549_clause,
            "9.4.1.2"
        );
    }

    #[test]
    fn covers_wcag_21_a_and_aa_criteria() {
        assert_eq!(wcag_21_aa_criteria().len(), 50);
        assert!(wcag_21_aa_criteria()
            .iter()
            .all(|criterion| map_to_bfsg(criterion.wcag).is_some_and(|m| m.fix_required)));
    }

    #[test]
    fn aaa_and_best_practice_criteria_are_not_required() {
        assert!(map_to_bfsg("2.4.9").is_none());
        assert!(map_to_bfsg("best-practice").is_none());
    }
}
