use crate::audit::normalized::AuditContext;
use crate::i18n::I18n;
use crate::output::report_model::{AiTransparencyPresentation, ImageProvenanceFindingPresentation};

pub(super) fn build_ai_transparency_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<AiTransparencyPresentation> {
    let en = i18n.locale() == "en";
    normalized.raw_ai_transparency.map(|a| AiTransparencyPresentation {
        images_checked: a.images_checked,
        findings: a
            .findings
            .iter()
            .map(|f| {
                let provenance_label = if en {
                    match f.provenance {
                        crate::ai_transparency::AiProvenanceKind::TrainedAlgorithmicMedia => {
                            "AI-trained model (trainedAlgorithmicMedia)"
                        }
                        crate::ai_transparency::AiProvenanceKind::CompositeWithTrainedAlgorithmicMedia => {
                            "AI-assisted composite (compositeWithTrainedAlgorithmicMedia)"
                        }
                        crate::ai_transparency::AiProvenanceKind::CompositeSynthetic => {
                            "Composite with AI content (compositeSynthetic)"
                        }
                        crate::ai_transparency::AiProvenanceKind::VirtualRecording => {
                            "Virtual recording with AI content (virtualRecording)"
                        }
                        crate::ai_transparency::AiProvenanceKind::TrainedAlgorithmicData => {
                            "Algorithmically generated data (trainedAlgorithmicData)"
                        }
                    }
                } else {
                    match f.provenance {
                        crate::ai_transparency::AiProvenanceKind::TrainedAlgorithmicMedia => {
                            "KI-trainiertes Modell (trainedAlgorithmicMedia)"
                        }
                        crate::ai_transparency::AiProvenanceKind::CompositeWithTrainedAlgorithmicMedia => {
                            "KI-gestützte Komposition (compositeWithTrainedAlgorithmicMedia)"
                        }
                        crate::ai_transparency::AiProvenanceKind::CompositeSynthetic => {
                            "Komposition mit KI-Anteil (compositeSynthetic)"
                        }
                        crate::ai_transparency::AiProvenanceKind::VirtualRecording => {
                            "Virtuelle Aufzeichnung mit KI-Anteil (virtualRecording)"
                        }
                        crate::ai_transparency::AiProvenanceKind::TrainedAlgorithmicData => {
                            "Algorithmisch erzeugte Daten (trainedAlgorithmicData)"
                        }
                    }
                }
                .to_string();
                let validation_label = if en {
                    match f.validation {
                        crate::ai_transparency::ManifestValidation::Invalid => "not validated",
                        crate::ai_transparency::ManifestValidation::Valid => "valid",
                        crate::ai_transparency::ManifestValidation::Trusted => "trusted",
                    }
                } else {
                    match f.validation {
                        crate::ai_transparency::ManifestValidation::Invalid => "nicht validiert",
                        crate::ai_transparency::ManifestValidation::Valid => "gültig",
                        crate::ai_transparency::ManifestValidation::Trusted => "vertrauenswürdig",
                    }
                }
                .to_string();
                ImageProvenanceFindingPresentation {
                    image_url: f.image_url.clone(),
                    provenance_label,
                    validation_label,
                    generator: f.generator.clone(),
                    message: crate::ai_transparency::finding_message_text(&f.rule_id, en),
                }
            })
            .collect(),
    })
}
