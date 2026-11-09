//! Experimental indicators (#916).
//!
//! UX, Journey, AI Visibility, Content Visibility, Source Quality and
//! Commerce are heuristics whose signal contract
//! (`validation.status: unvalidated`, `audit::signal_contract`) has no
//! calibration behind it, and none of them feeds a score. They run only when
//! asked for with `--experimental <name>[,<name>]`, are in no `--full` run,
//! and their results live apart from the audit result: after the audit
//! assembled them they are moved out of the ordinary report fields into
//! [`AuditReport::experimental`](crate::audit::AuditReport), so the PDF, the
//! findings, the scores and the decision never see them. The JSON carries
//! them in one `experimental` block, the terminal table lists them when
//! requested.

use std::collections::BTreeSet;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::ai_visibility::AiVisibilityAnalysis;
use crate::audit::AuditReport;
use crate::commerce::CommerceAnalysis;
use crate::content_visibility::ContentVisibilityAnalysis;
use crate::journey::JourneyAnalysis;
use crate::source_quality::SourceQualityAnalysis;
use crate::ux::UxAnalysis;

/// One experimental indicator `--experimental` can switch on.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExperimentalIndicator {
    /// Structural usability heuristics from the accessibility tree.
    #[value(name = "ux")]
    Ux,
    /// Page-intent and friction heuristics.
    #[value(name = "journey")]
    Journey,
    /// Machine readability for LLM crawlers and answer engines.
    #[value(name = "ai-visibility")]
    AiVisibility,
    /// Organic visibility, E-E-A-T and topical-depth signals.
    #[value(name = "content-visibility")]
    ContentVisibility,
    /// Technical preparation for machine processing.
    #[value(name = "source-quality")]
    SourceQuality,
    /// Shop and conversion signals, and the add-to-cart and quantity-stepper
    /// journeys (the latter also need `--allow-state-changes`).
    #[value(name = "commerce")]
    Commerce,
}

impl ExperimentalIndicator {
    pub const ALL: [ExperimentalIndicator; 6] = [
        Self::Ux,
        Self::Journey,
        Self::AiVisibility,
        Self::ContentVisibility,
        Self::SourceQuality,
        Self::Commerce,
    ];

    /// The module id, and the key of the indicator in the JSON.
    pub fn id(self) -> &'static str {
        match self {
            Self::Ux => "ux",
            Self::Journey => "journey",
            Self::AiVisibility => "ai_visibility",
            Self::ContentVisibility => "content_visibility",
            Self::SourceQuality => "source_quality",
            Self::Commerce => "commerce",
        }
    }

    /// The name `--experimental` takes.
    pub fn flag_name(self) -> &'static str {
        match self {
            Self::Ux => "ux",
            Self::Journey => "journey",
            Self::AiVisibility => "ai-visibility",
            Self::ContentVisibility => "content-visibility",
            Self::SourceQuality => "source-quality",
            Self::Commerce => "commerce",
        }
    }

    /// Indicators this one is derived from. They run with it and appear next
    /// to it, so no number is computed from an input that was switched off.
    fn requires(self) -> &'static [ExperimentalIndicator] {
        match self {
            Self::SourceQuality => &[Self::Ux],
            Self::ContentVisibility => &[Self::SourceQuality, Self::AiVisibility],
            _ => &[],
        }
    }

    /// The indicators that run for a request: the requested ones and
    /// everything they are derived from.
    pub fn closure(requested: &[ExperimentalIndicator]) -> BTreeSet<ExperimentalIndicator> {
        let mut out = BTreeSet::new();
        let mut pending: Vec<ExperimentalIndicator> = requested.to_vec();
        while let Some(indicator) = pending.pop() {
            if out.insert(indicator) {
                pending.extend_from_slice(indicator.requires());
            }
        }
        out
    }
}

/// The results of the experimental indicators that ran, moved out of the
/// ordinary report fields at the end of the audit assembly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentalIndicators {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux: Option<UxAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey: Option<JourneyAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_visibility: Option<AiVisibilityAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_visibility: Option<ContentVisibilityAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_quality: Option<SourceQualityAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commerce: Option<CommerceAnalysis>,
}

impl ExperimentalIndicators {
    pub fn is_empty(&self) -> bool {
        self.ux.is_none()
            && self.journey.is_none()
            && self.ai_visibility.is_none()
            && self.content_visibility.is_none()
            && self.source_quality.is_none()
            && self.commerce.is_none()
    }

    /// Moves the experimental results out of `report`'s ordinary fields.
    pub(crate) fn take_from(report: &mut AuditReport) -> Self {
        Self {
            ux: report.ux.take(),
            journey: report.journey.take(),
            ai_visibility: report.discoverability.ai_visibility.take(),
            content_visibility: report.discoverability.content_visibility.take(),
            source_quality: report.discoverability.source_quality.take(),
            commerce: report.commerce.take(),
        }
    }

    /// Ids of the indicators present, in the order of
    /// [`ExperimentalIndicator::ALL`].
    pub fn present(&self) -> Vec<ExperimentalIndicator> {
        ExperimentalIndicator::ALL
            .into_iter()
            .filter(|indicator| match indicator {
                ExperimentalIndicator::Ux => self.ux.is_some(),
                ExperimentalIndicator::Journey => self.journey.is_some(),
                ExperimentalIndicator::AiVisibility => self.ai_visibility.is_some(),
                ExperimentalIndicator::ContentVisibility => self.content_visibility.is_some(),
                ExperimentalIndicator::SourceQuality => self.source_quality.is_some(),
                ExperimentalIndicator::Commerce => self.commerce.is_some(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_derived_indicator_brings_its_inputs() {
        let all = ExperimentalIndicator::closure(&[ExperimentalIndicator::ContentVisibility]);
        assert_eq!(
            all.into_iter().collect::<Vec<_>>(),
            vec![
                ExperimentalIndicator::Ux,
                ExperimentalIndicator::AiVisibility,
                ExperimentalIndicator::ContentVisibility,
                ExperimentalIndicator::SourceQuality,
            ]
        );
        assert_eq!(
            ExperimentalIndicator::closure(&[ExperimentalIndicator::Commerce]).len(),
            1
        );
        assert!(ExperimentalIndicator::closure(&[]).is_empty());
    }

    #[test]
    fn ids_match_the_module_catalog() {
        let catalog = crate::audit::catalog::AuditCatalog::standard();
        let ids: Vec<&str> = catalog.iter().map(|m| m.id()).collect();
        for indicator in ExperimentalIndicator::ALL {
            assert!(ids.contains(&indicator.id()), "{}", indicator.id());
        }
    }

    #[test]
    fn flag_names_are_the_value_enum_names() {
        for indicator in ExperimentalIndicator::ALL {
            let value = indicator.to_possible_value().expect("named value");
            assert_eq!(value.get_name(), indicator.flag_name());
        }
    }

    #[test]
    fn take_from_moves_the_results_out_of_the_ordinary_fields() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            crate::cli::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            0,
        );
        report.ux = Some(crate::analyze_ux(&crate::AXTree::new()));
        let experimental = ExperimentalIndicators::take_from(&mut report);
        assert!(report.ux.is_none());
        assert_eq!(experimental.present(), vec![ExperimentalIndicator::Ux]);
        assert!(!experimental.is_empty());
        assert!(ExperimentalIndicators::default().is_empty());
    }
}
