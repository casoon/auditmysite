use crate::i18n::I18n;

pub(crate) fn is_english(i18n: &I18n) -> bool {
    i18n.locale() == "en"
}

pub(crate) fn pick<'a>(i18n: &I18n, de: &'a str, en: &'a str) -> &'a str {
    if is_english(i18n) {
        en
    } else {
        de
    }
}
