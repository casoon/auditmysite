//! Text export of the interactive (journey) finding texts (#936): every text
//! of `interactive_finding_text` and `finding_uncertainty_text` in both
//! languages across an enumerated input space, as records in
//! `reports/interactive-finding-texts.json` (gitignored; the path can be set
//! with `INTERACTIVE_FINDING_TEXT_EXPORT`). Run it before and after a change
//! of how the texts are stored and compare the two files byte for byte.

use super::{
    finding_uncertainty_text, interactive_finding_text, FindingUncertainty, InteractiveFindingKind,
    InteractiveFindingValues,
};
use serde_json::{json, Value};

pub(super) const KINDS: [InteractiveFindingKind; 43] = {
    use InteractiveFindingKind::*;
    [
        HiddenFocusableAriaHidden,
        HiddenFocusableInert,
        HiddenFocusableStyle,
        FocusIndicatorNotDetected,
        TabOrderBackwardJumps,
        FocusTrapNotEntered,
        FocusTrapBackgroundNotHidden,
        FocusTrapEscaped,
        FocusTrapEscapeNotClosing,
        FocusRestorationLostToBody,
        ModalNotOpened,
        MenuNotOpened,
        MenuFocusNotMoved,
        MenuEscapeNotClosing,
        TabsSelectionNotMoved,
        TabsFocusNotOnTab,
        DisclosureNotOpened,
        DisclosureNotClosed,
        DisclosureStateWithoutContent,
        DisclosureContentWithoutState,
        SpaNoAnnouncementSignal,
        SpaTitleUnchanged,
        SpaFocusNotMoved,
        SkipLinkFocusNotMoved,
        FormErrorSilentFailure,
        FormErrorInvalidWithoutLiveRegion,
        FormErrorUnlinkedFields,
        FormErrorFocusNotManaged,
        FormErrorLiveRegionLateInsertion,
        AddToCartNoStatusAnnouncement,
        AddToCartNoFeedbackDetected,
        QuantityStepperKeyboardInoperable,
        QuantityStepperValueNotExposed,
        LinkTextGeneric,
        LinkTextDuplicate,
        HeadingMissingH1,
        HeadingMultipleH1,
        HeadingLevelSkip,
        LandmarkMissingMain,
        LandmarkNavWithoutLabels,
        LandmarkDuplicateUnique,
        MediaControlsMissingName,
        MediaControlsNotReachable,
    ]
};

/// Text values: empty, plain, padded, Fluent syntax, quotes and control
/// characters, umlauts and a long URL.
const TEXTS: [&str; 7] = [
    "",
    "a#one",
    " padded value ",
    "{braces} and {$var}",
    "say \"hi\"\n\ttab \\ back",
    "äöüß — „quotes“ <b>; semi",
    "https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/and-going/and-going/and-going/and-going/file.min.js?v=1234567890",
];

/// Counts: none, 0, 1, 2, 5, one that Fluent could group and the largest.
const COUNTS: [Option<u32>; 7] = [
    None,
    Some(0),
    Some(1),
    Some(2),
    Some(5),
    Some(1_234_567),
    Some(u32::MAX),
];

const TRUNCATED: [Option<bool>; 3] = [None, Some(false), Some(true)];

fn texts() -> impl Iterator<Item = Option<String>> {
    std::iter::once(None).chain(TEXTS.iter().map(|t| Some(t.to_string())))
}

/// Value sets: every text field on its own, the count with the example list
/// and its truncation, the count with the landmark role, and all fields set.
pub(super) fn values() -> Vec<InteractiveFindingValues> {
    let mut out = vec![InteractiveFindingValues::default()];
    for text in texts().skip(1) {
        out.push(InteractiveFindingValues {
            selector: text.clone(),
            ..Default::default()
        });
        out.push(InteractiveFindingValues {
            title_before: text.clone(),
            ..Default::default()
        });
    }
    for count in COUNTS {
        for examples in texts() {
            for truncated in TRUNCATED {
                out.push(InteractiveFindingValues {
                    count,
                    examples: examples.clone(),
                    truncated,
                    ..Default::default()
                });
            }
        }
        for role in texts() {
            out.push(InteractiveFindingValues {
                count,
                role,
                ..Default::default()
            });
        }
    }
    out.push(InteractiveFindingValues {
        selector: Some("a#one".to_string()),
        count: Some(2),
        examples: Some("a, b".to_string()),
        truncated: Some(true),
        title_before: Some("Home".to_string()),
        role: Some("main".to_string()),
    });
    out
}

pub(super) const WAITED_MS: [u64; 5] = [0, 1, 210, 1_234_567, u64::MAX];

const LANGS: [(&str, bool); 2] = [("en", true), ("de", false)];

#[test]
#[ignore = "writes reports/interactive-finding-texts.json; run before and after a change of the text storage"]
fn export_interactive_finding_texts() {
    let mut records: Vec<Value> = Vec::new();
    let values = values();
    for kind in KINDS {
        for v in &values {
            for (lang, en) in LANGS {
                let (message, fix) = interactive_finding_text(kind, v, en);
                records.push(json!([
                    "interactive_finding_text",
                    format!("{kind:?}"),
                    format!("{v:?}"),
                    lang,
                    message,
                    fix
                ]));
            }
        }
    }
    for waited_ms in WAITED_MS {
        let uncertainty = FindingUncertainty::LateReactionPossible { waited_ms };
        for (lang, en) in LANGS {
            records.push(json!([
                "finding_uncertainty_text",
                format!("{uncertainty:?}"),
                "",
                lang,
                finding_uncertainty_text(&uncertainty, en),
                null
            ]));
        }
    }
    let path = std::env::var("INTERACTIVE_FINDING_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/interactive-finding-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} interactive finding text records written to {path}",
        records.len()
    );
}
