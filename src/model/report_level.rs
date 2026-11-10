//! Detail level of the PDF report.

use clap::ValueEnum;

/// Report detail level for PDF reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ReportLevel {
    /// Executive summary — compact overview for management
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

impl std::fmt::Display for ReportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportLevel::Executive => write!(f, "executive"),
            ReportLevel::Standard => write!(f, "standard"),
            ReportLevel::Technical => write!(f, "technical"),
        }
    }
}
