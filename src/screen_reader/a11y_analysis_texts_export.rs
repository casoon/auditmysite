//! Text export of the screen-reader analysis and the pattern messages (#936):
//! every issue message of `screen_reader::analyzer::analyze_reading_sequence`
//! (all detectors, every `LinkQuality`, every heading-order problem, counts of
//! 0, 1 and many where a text takes a count) and every
//! `patterns::pattern_message_text` text in both languages, as records in
//! `reports/a11y-analysis-texts.json` (gitignored; the path can be set with
//! `A11Y_ANALYSIS_TEXT_EXPORT`). The analysis runs with both detection locales
//! so the export also shows that detection and message language stay separate.
//! Run it before and after a change of how the texts are stored and compare
//! the two files byte for byte.

use super::analyzer::analyze_reading_sequence;
use super::navigator::{
    navigation_views, FormControlNavItem, FormControlQuality, HeadingNavItem, HeadingQuality,
    LandmarkNavItem, LandmarkQuality, LinkNavItem, LinkQuality, NavigationViews,
};
use super::ReadingItem;
use crate::patterns::{pattern_message_text, PatternMessage};
use serde_json::{json, Value};

fn item(seq: usize, role: &str, name: Option<&str>, tab_stop: bool) -> ReadingItem {
    ReadingItem {
        seq,
        role: Some(role.to_string()),
        name: name.map(String::from),
        description: None,
        value: None,
        states: vec![],
        tab_stop,
        depth: 0,
        node_id: format!("node-{seq}"),
    }
}

fn heading(seq: usize, level: Option<u8>, quality: HeadingQuality) -> HeadingNavItem {
    HeadingNavItem {
        level,
        text: Some("Title".to_string()),
        seq,
        node_id: format!("h-{seq}"),
        quality,
    }
}

fn landmark(seq: usize, role: &str, quality: LandmarkQuality) -> LandmarkNavItem {
    LandmarkNavItem {
        role: role.to_string(),
        name: None,
        seq,
        node_id: format!("l-{seq}"),
        quality,
    }
}

fn link(count: usize, quality: LinkQuality) -> LinkNavItem {
    LinkNavItem {
        text: Some("More".to_string()),
        count,
        seq_positions: vec![1],
        node_ids: vec![format!("link-{count}")],
        quality,
    }
}

fn views_with(
    headings: Vec<HeadingNavItem>,
    landmarks: Vec<LandmarkNavItem>,
    links: Vec<LinkNavItem>,
    form_controls: Vec<FormControlNavItem>,
) -> NavigationViews {
    let mut views = navigation_views(&[]);
    views.headings = headings;
    views.landmarks = landmarks;
    views.links = links;
    views.form_controls = form_controls;
    views
}

/// The three required landmarks present, so the landmark detectors stay quiet
/// unless a scenario wants them.
fn required_landmarks() -> Vec<LandmarkNavItem> {
    vec![
        landmark(0, "banner", LandmarkQuality::Ok),
        landmark(1, "navigation", LandmarkQuality::Ok),
        landmark(2, "contentinfo", LandmarkQuality::Ok),
    ]
}

fn desert_items(announced: usize) -> Vec<ReadingItem> {
    let mut items = vec![item(0, "heading", Some("Chapter"), false)];
    for seq in 1..=announced {
        items.push(item(seq, "StaticText", Some("A paragraph"), false));
    }
    items
}

fn tab_stop_items(tab_stops: usize, skip_link: bool) -> Vec<ReadingItem> {
    let mut items = Vec::new();
    if skip_link {
        items.push(item(0, "link", Some("Skip to content"), true));
    }
    for seq in items.len()..tab_stops {
        items.push(item(seq, "link", Some(&format!("Link {seq}")), true));
    }
    items
}

type Scenario = (String, Vec<ReadingItem>, NavigationViews);

/// The scenarios: items and views, each hitting one or more detectors.
fn scenarios() -> Vec<Scenario> {
    let quiet = || views_with(vec![], required_landmarks(), vec![], vec![]);
    let mut out: Vec<Scenario> = Vec::new();

    // Non-descriptive names (medium and the downgraded "low"), duplicated
    // names, icon-font characters, empty interactive elements.
    let names = vec![
        item(0, "link", Some("Hier"), true),
        item(1, "button", Some("Weiter"), true),
        item(2, "link", Some("click here"), true),
        item(3, "button", Some("x"), true),
        item(4, "link", Some("Kontakt Kontakt"), true),
        item(
            5,
            "link",
            Some("Read the full annual report Read the full annual report"),
            true,
        ),
        item(6, "link", Some("Mehr Mehr"), true),
        item(7, "link", Some("\u{E003}"), true),
        item(8, "button", None, true),
        item(9, "link", Some("  "), true),
    ];
    out.push(("names".to_string(), names, quiet()));

    // Announcement deserts: around the threshold and a long one.
    for announced in [15, 16, 17, 1234] {
        out.push((
            format!("desert {announced}"),
            desert_items(announced),
            quiet(),
        ));
    }
    let mut two_deserts = desert_items(20);
    two_deserts.push(item(21, "navigation", Some("Menu"), false));
    for seq in 22..=40 {
        two_deserts.push(item(seq, "StaticText", Some("Another paragraph"), false));
    }
    out.push(("two deserts".to_string(), two_deserts, quiet()));

    // Tab stops past the threshold without a skip link, with one, below it.
    for tab_stops in [50, 51, 120, 1234] {
        out.push((
            format!("tab stops {tab_stops}"),
            tab_stop_items(tab_stops, false),
            quiet(),
        ));
    }
    out.push((
        "tab stops with skip link".to_string(),
        tab_stop_items(60, true),
        quiet(),
    ));

    // Skipped heading levels: known levels and an unknown level.
    out.push((
        "skipped levels".to_string(),
        vec![],
        views_with(
            vec![
                heading(3, Some(1), HeadingQuality::Good),
                heading(4, Some(3), HeadingQuality::SkippedLevel),
                heading(9, Some(6), HeadingQuality::SkippedLevel),
                heading(12, None, HeadingQuality::SkippedLevel),
            ],
            required_landmarks(),
            vec![],
            vec![],
        ),
    ));

    // Heading order: first heading not an H1 (H2, H3, unknown level) and an H1
    // after a subheading.
    for (label, level) in [("H2", Some(2)), ("H3", Some(3)), ("unknown", None)] {
        out.push((
            format!("first heading {label}"),
            vec![],
            views_with(
                vec![heading(5, level, HeadingQuality::Good)],
                required_landmarks(),
                vec![],
                vec![],
            ),
        ));
    }
    out.push((
        "h1 after subheading".to_string(),
        vec![],
        views_with(
            vec![
                heading(57, Some(2), HeadingQuality::Good),
                heading(128, Some(1), HeadingQuality::Good),
            ],
            required_landmarks(),
            vec![],
            vec![],
        ),
    ));
    out.push((
        "h3 first and h1 late".to_string(),
        vec![],
        views_with(
            vec![
                heading(7, Some(3), HeadingQuality::Good),
                heading(1234, Some(1), HeadingQuality::Good),
            ],
            required_landmarks(),
            vec![],
            vec![],
        ),
    ));

    // Landmarks: none present, each one missing alone, unlabeled duplicates,
    // missing main.
    out.push((
        "no landmarks".to_string(),
        vec![],
        views_with(vec![], vec![], vec![], vec![]),
    ));
    for missing in ["banner", "navigation", "contentinfo"] {
        let landmarks: Vec<LandmarkNavItem> = required_landmarks()
            .into_iter()
            .filter(|l| l.role != missing)
            .collect();
        out.push((
            format!("without {missing}"),
            vec![],
            views_with(vec![], landmarks, vec![], vec![]),
        ));
    }
    let mut duplicates = required_landmarks();
    duplicates.push(landmark(3, "region", LandmarkQuality::UnlabeledDuplicate));
    duplicates.push(landmark(4, "region", LandmarkQuality::UnlabeledDuplicate));
    duplicates.push(landmark(5, "main", LandmarkQuality::MissingMain));
    duplicates.push(landmark(6, "main", LandmarkQuality::MissingMain));
    out.push((
        "duplicate and missing main".to_string(),
        vec![],
        views_with(vec![], duplicates, vec![], vec![]),
    ));
    let mut single_duplicate = required_landmarks();
    single_duplicate.push(landmark(3, "region", LandmarkQuality::UnlabeledDuplicate));
    out.push((
        "single unlabeled duplicate".to_string(),
        vec![],
        views_with(vec![], single_duplicate, vec![], vec![]),
    ));

    // Duplicate link texts: every quality at counts 1, 2, 3 and many.
    let mut links = Vec::new();
    for quality in [
        LinkQuality::Good,
        LinkQuality::ContextDependent,
        LinkQuality::NonDescriptive,
        LinkQuality::Empty,
    ] {
        for count in [1, 2, 3, 1234] {
            links.push(link(count, quality));
        }
    }
    out.push((
        "duplicate link texts".to_string(),
        vec![],
        views_with(vec![], required_landmarks(), links, vec![]),
    ));

    // Form fields without a label.
    out.push((
        "empty form labels".to_string(),
        vec![],
        views_with(
            vec![],
            required_landmarks(),
            vec![],
            vec![FormControlNavItem {
                label: None,
                control_type: "textbox".to_string(),
                required: false,
                is_identification_control: false,
                seq: 1,
                node_id: "f-1".to_string(),
                quality: FormControlQuality::EmptyLabel,
            }],
        ),
    ));

    out
}

/// Every screen-reader issue text in one message language, for both detection
/// locales and both values of the disclosure-menu pattern flag.
pub(crate) fn screen_reader_texts(en: bool) -> Vec<Value> {
    let mut out = Vec::new();
    for (label, items, views) in scenarios() {
        for detect_locale in ["de", "en"] {
            for disclosure in [false, true] {
                for issue in analyze_reading_sequence(&items, &views, detect_locale, en, disclosure)
                {
                    out.push(json!([
                        "screen_reader_issue",
                        format!("{label} / detect {detect_locale} / disclosure {disclosure}"),
                        issue.wcag_criterion,
                        issue.severity,
                        issue.review,
                        issue.affected_node_ids,
                        issue.message
                    ]));
                }
            }
        }
    }
    out
}

const COUNTS: [usize; 4] = [0, 1, 2, 1234];

/// Every pattern message variant over counts of 0, 1, 2 and many and the
/// present and absent names.
pub(crate) fn pattern_messages() -> Vec<(String, PatternMessage)> {
    let mut out: Vec<(String, PatternMessage)> = Vec::new();
    for landmarks in COUNTS {
        for unnamed in [0, 1, 5] {
            out.push((
                format!("NavigationUnnamed {landmarks}/{unnamed}"),
                PatternMessage::NavigationUnnamed { landmarks, unnamed },
            ));
        }
        out.push((
            format!("NavigationWithoutLinks {landmarks}"),
            PatternMessage::NavigationWithoutLinks { landmarks },
        ));
        out.push((
            format!("Navigation {landmarks}"),
            PatternMessage::Navigation { landmarks },
        ));
    }
    for native in COUNTS {
        out.push((
            format!("DisclosureNative {native}"),
            PatternMessage::DisclosureNative { native },
        ));
    }
    for triggers in COUNTS {
        for with_controls in COUNTS {
            for native in COUNTS {
                out.push((
                    format!("Disclosure {triggers}/{with_controls}/{native}"),
                    PatternMessage::Disclosure {
                        triggers,
                        with_controls,
                        native,
                    },
                ));
            }
        }
    }
    for dialogs in COUNTS {
        for well_formed in [0, 1, 3] {
            out.push((
                format!("ModalDialog {dialogs}/{well_formed}"),
                PatternMessage::ModalDialog {
                    dialogs,
                    well_formed,
                },
            ));
        }
    }
    for tablists in COUNTS {
        for tabs in COUNTS {
            out.push((
                format!("TabList {tablists}/{tabs}"),
                PatternMessage::TabList {
                    tablists,
                    tabs,
                    selected: 2,
                    controls: 1,
                },
            ));
        }
    }
    for first in [true, false] {
        out.push((
            format!("SkipLink first={first}"),
            PatternMessage::SkipLink {
                name: "Zum Inhalt springen".to_string(),
                first,
            },
        ));
    }
    for triggers in COUNTS {
        for non_button in [0, 1, 3] {
            out.push((
                format!("Accordion {triggers}/{non_button}"),
                PatternMessage::Accordion {
                    triggers,
                    with_controls: 2,
                    non_button,
                },
            ));
        }
    }
    for fields in COUNTS {
        for trigger in [None, Some("Send".to_string())] {
            out.push((
                format!("Form {fields}/{trigger:?}"),
                PatternMessage::Form { fields, trigger },
            ));
        }
    }
    for trigger in [None, Some("In den Warenkorb".to_string())] {
        out.push((
            format!("AddToCart {trigger:?}"),
            PatternMessage::AddToCart { trigger },
        ));
    }
    for name in [None, Some("Menge".to_string())] {
        out.push((
            format!("QuantityStepper {name:?}"),
            PatternMessage::QuantityStepper { name },
        ));
    }
    out.push(("EasyLanguage".to_string(), PatternMessage::EasyLanguage));
    out
}

/// Every pattern message text in one language.
pub(crate) fn pattern_texts(en: bool) -> Vec<Value> {
    pattern_messages()
        .into_iter()
        .map(|(label, message)| {
            json!(["pattern_message", label, pattern_message_text(&message, en)])
        })
        .collect()
}

/// Every text of both files in one language.
pub(crate) fn a11y_analysis_texts(en: bool) -> Vec<Value> {
    let mut out = screen_reader_texts(en);
    out.extend(pattern_texts(en));
    out
}

#[test]
#[ignore = "writes reports/a11y-analysis-texts.json; run before and after a change of the text storage"]
fn export_a11y_analysis_texts() {
    let mut records = Vec::new();
    for lang in ["en", "de"] {
        for text in a11y_analysis_texts(lang == "en") {
            records.push(json!({ "lang": lang, "text": text }));
        }
    }
    let path = std::env::var("A11Y_ANALYSIS_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/a11y-analysis-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} a11y analysis text records written to {path}",
        records.len()
    );
}
