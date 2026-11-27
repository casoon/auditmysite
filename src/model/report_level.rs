//! Detail level of the PDF report.

use clap::ValueEnum;

/// Report detail level for PDF reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[non_exhaustive]
pub enum ReportLevel {
    /// Quick report — first contact, 2-3 pages: verdict, check status, the top
    /// findings with the first step, scope and caveat, next step
    #[value(name = "quick")]
    Quick,
    /// Management report — 5-8 pages: verdict, score drivers, problem profile,
    /// risks and the priority action list, without per-element evidence or
    /// module detail chapters
    #[value(name = "executive")]
    Executive,
    /// Standard report — all chapters (default)
    #[default]
    #[value(name = "standard")]
    Standard,
    /// Technical report — extended appendix with full details
    #[value(name = "technical")]
    Technical,
}

impl ReportLevel {
    /// The two short levels (`quick`, `executive`): a summary only, no
    /// contents page, no finding cards, no detail chapters. Batch reports,
    /// which have no per-finding chapters in the first place, treat both like
    /// `executive`.
    pub fn is_summary(self) -> bool {
        matches!(self, ReportLevel::Quick | ReportLevel::Executive)
    }
}

impl std::fmt::Display for ReportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportLevel::Quick => write!(f, "quick"),
            ReportLevel::Executive => write!(f, "executive"),
            ReportLevel::Standard => write!(f, "standard"),
            ReportLevel::Technical => write!(f, "technical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_level_parses_from_its_display_name() {
        for level in [
            ReportLevel::Quick,
            ReportLevel::Executive,
            ReportLevel::Standard,
            ReportLevel::Technical,
        ] {
            let parsed = <ReportLevel as ValueEnum>::from_str(&level.to_string(), false);
            assert_eq!(parsed, Ok(level));
        }
        assert_eq!(ReportLevel::default(), ReportLevel::Standard);
    }

    #[test]
    fn only_quick_and_executive_are_summary_levels() {
        assert!(ReportLevel::Quick.is_summary());
        assert!(ReportLevel::Executive.is_summary());
        assert!(!ReportLevel::Standard.is_summary());
        assert!(!ReportLevel::Technical.is_summary());
    }
}
