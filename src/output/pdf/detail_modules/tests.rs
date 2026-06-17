use super::module_customer_context;
use crate::i18n::I18n;
use renderreport::components::Component;

fn text(i18n: &I18n, module: &str, score: u32, interp: &str) -> String {
    module_customer_context(i18n, module, score, interp)
        .to_data()
        .to_string()
}

// These tests assert the *behavior* of the explanation (varies by module,
// score band, locale; includes the caller's interpretation) rather than
// exact prose, which changes frequently per the wording rules (#452).

#[test]
fn module_customer_context_varies_by_module() {
    let i18n = I18n::new("de").expect("de locale");
    assert_ne!(
        text(&i18n, "performance", 60, ""),
        text(&i18n, "ai_visibility", 60, "")
    );
}

#[test]
fn module_customer_context_reflects_score_band() {
    // A weakness clause is added below 75 and differs below 50, so the same
    // module at different score bands must produce different text.
    let i18n = I18n::new("de").expect("de locale");
    let critical = text(&i18n, "performance", 40, "");
    let mid = text(&i18n, "performance", 60, "");
    let good = text(&i18n, "performance", 90, "");
    assert_ne!(critical, mid);
    assert_ne!(mid, good);
    assert_ne!(critical, good);
}

#[test]
fn content_visibility_has_no_score_band_weakness_clause() {
    // content_visibility is an indicator, so its text is identical across
    // score bands — unlike a scored module such as performance.
    let i18n = I18n::new("de").expect("de locale");
    assert_eq!(
        text(&i18n, "content_visibility", 40, ""),
        text(&i18n, "content_visibility", 90, "")
    );
    assert_ne!(
        text(&i18n, "performance", 40, ""),
        text(&i18n, "performance", 90, "")
    );
}

#[test]
fn module_customer_context_is_localized() {
    let de = I18n::new("de").expect("de locale");
    let en = I18n::new("en").expect("en locale");
    assert_ne!(
        text(&de, "performance", 60, ""),
        text(&en, "performance", 60, "")
    );
}

#[test]
fn module_customer_context_excludes_caller_interpretation() {
    let i18n = I18n::new("de").expect("de locale");
    let sentinel = "XZ_SENTINEL_INTERPRETATION_42";
    let out = text(&i18n, "performance", 60, sentinel);
    // The technical interpretation must NOT be echoed into the plain-language
    // customer passage — it is already shown in the module overview, and
    // duplicating it dumped jargon into the customer text (#446 readability).
    assert!(
        !out.contains(sentinel),
        "customer passage must not echo the technical interpretation"
    );
    // No meta-label prefix anymore; the plain-language module sentence is present.
    assert!(
        out.contains("Besucher"),
        "expected the plain performance customer text, got: {out}"
    );
}
