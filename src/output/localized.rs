use crate::i18n::I18n;

pub(crate) fn is_english(i18n: &I18n) -> bool {
    i18n.locale() == "en"
}

#[cfg(feature = "pdf")]
pub(crate) fn pick<'a>(i18n: &I18n, de: &'a str, en: &'a str) -> &'a str {
    if is_english(i18n) {
        en
    } else {
        de
    }
}

#[cfg(feature = "pdf")]
/// The one convention for report rows whose value may be missing: a row with
/// an empty value, a dash or a bare field name is omitted instead of printed
/// as "Key —" (#952). Absence that matters is stated in its own sentence.
pub(crate) fn has_value(value: &str) -> bool {
    !matches!(value.trim(), "" | "—" | "-" | "–")
}

#[cfg(all(test, feature = "pdf"))]
mod tests {
    use super::*;

    #[test]
    fn rows_without_a_value_are_not_printed() {
        assert!(!has_value(""));
        assert!(!has_value("  "));
        assert!(!has_value("—"));
        assert!(has_value("0"));
        assert!(has_value("de"));
    }
}
