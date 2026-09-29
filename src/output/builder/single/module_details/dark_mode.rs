use crate::audit::normalized::AuditContext;
use crate::dark_mode::dark_mode_issue_text;
use crate::i18n::I18n;
use crate::output::report_model::{DarkModePresentation, VisionDeficiencyModePresentation};

pub(super) fn build_dark_mode_details(
    normalized: &AuditContext<'_>,
    i18n: &I18n,
) -> Option<DarkModePresentation> {
    let en = i18n.locale() == "en";
    normalized.raw_dark_mode.map(|dm| DarkModePresentation {
        supported: dm.supported,
        score: dm.score,
        detection_methods: dm.detection_methods.clone(),
        color_scheme_css: dm.color_scheme_css,
        meta_color_scheme: dm.meta_color_scheme.clone(),
        css_custom_properties: dm.css_custom_properties,
        dark_contrast_violations: dm.dark_contrast_violations,
        dark_only_violations: dm.dark_only_violations,
        light_only_violations: dm.light_only_violations,
        print_stylesheet_detected: dm.print.stylesheet_detected,
        print_interactive_chrome_hidden: dm.print.interactive_chrome_hidden,
        print_content_not_clipped: dm.print.content_not_clipped,
        print_clipped_elements: dm.print.clipped_elements,
        forced_colors_detected: dm.forced_colors.stylesheet_detected,
        forced_colors_active_matches: dm.forced_colors.active_matches,
        forced_color_adjust_count: dm.forced_colors.forced_color_adjust_count,
        forced_colors_focus_visible: dm.forced_colors.focus_indicators_visible,
        vision_deficiency_modes: dm
            .vision_deficiency
            .modes
            .iter()
            .map(|mode| VisionDeficiencyModePresentation {
                mode: mode.mode.clone(),
                contrast_violations: mode.contrast_violations,
                new_contrast_violations: mode.new_contrast_violations,
                use_of_color_violations: mode.use_of_color_violations,
            })
            .collect(),
        issues: dm
            .issues
            .iter()
            .map(|i| {
                (
                    i.severity.clone(),
                    dark_mode_issue_text(&i.kind, &i.selectors, en),
                )
            })
            .collect(),
    })
}
