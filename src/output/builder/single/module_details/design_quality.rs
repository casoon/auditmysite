use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
use crate::output::localized::is_english;
use crate::output::report_model::{DesignQualityFindingPresentation, DesignQualityPresentation};

pub(super) fn build_design_quality_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<DesignQualityPresentation> {
    let en = is_english(i18n);
    normalized.raw_design_quality.map(|dq| {
        let warning_count = dq.warnings().count();
        let advisory_count = dq.advisories().count();
        DesignQualityPresentation {
            warning_count,
            advisory_count,
            findings: dq
                .findings
                .iter()
                .map(|f| {
                    let level_label = match f.level {
                        crate::design_quality::FindingLevel::Warning => {
                            if en {
                                "Warning"
                            } else {
                                "Warnung"
                            }
                        }
                        crate::design_quality::FindingLevel::Advisory => {
                            if en {
                                "Advisory"
                            } else {
                                "Hinweis"
                            }
                        }
                    };
                    let confidence_label = match f.confidence {
                        crate::design_quality::Confidence::High => {
                            if en {
                                "High"
                            } else {
                                "Hoch"
                            }
                        }
                        crate::design_quality::Confidence::Medium => {
                            if en {
                                "Medium"
                            } else {
                                "Mittel"
                            }
                        }
                        crate::design_quality::Confidence::Low => {
                            if en {
                                "Low"
                            } else {
                                "Niedrig"
                            }
                        }
                    };
                    DesignQualityFindingPresentation {
                        rule_id: f.rule_id.clone(),
                        level_label: level_label.to_string(),
                        confidence_label: confidence_label.to_string(),
                        selector: f.selector.clone(),
                        evidence: f.evidence.clone(),
                        message: crate::design_quality::finding_message_text(
                            &f.rule_id, f.level, en,
                        ),
                    }
                })
                .collect(),
        }
    })
}
