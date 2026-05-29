use crate::i18n::I18n;

use super::types::ReadingItem;

/// Announce a reading item using the default German locale.
pub fn announce(item: &ReadingItem) -> String {
    let i18n = I18n::new("de").expect("German report locale must parse");
    announce_localized(item, &i18n)
}

/// Announce a reading item in the supplied locale.
pub fn announce_localized(item: &ReadingItem, i18n: &I18n) -> String {
    let mut parts = vec![item
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(String::from)
        .unwrap_or_else(|| i18n.t("sr-no-name"))];

    parts.push(role_label(item, i18n));
    parts.extend(state_labels(item, i18n));

    parts.join(", ")
}

fn role_label(item: &ReadingItem, i18n: &I18n) -> String {
    if item.role.as_deref() == Some("heading") {
        if let Some(level) = state_value(&item.states, "level") {
            return i18n.t_args("sr-role-heading-level", &[("level", level)]);
        }
    }

    match item.role.as_deref().unwrap_or("generic") {
        "button" => i18n.t("sr-role-button"),
        "link" => i18n.t("sr-role-link"),
        "textbox" | "searchbox" => i18n.t("sr-role-textbox"),
        "checkbox" => i18n.t("sr-role-checkbox"),
        "radio" => i18n.t("sr-role-radio"),
        "combobox" => i18n.t("sr-role-combobox"),
        "listbox" => i18n.t("sr-role-listbox"),
        "slider" => i18n.t("sr-role-slider"),
        "spinbutton" => i18n.t("sr-role-spinbutton"),
        "tab" => i18n.t("sr-role-tab"),
        "heading" => i18n.t("sr-role-heading"),
        "navigation" => i18n.t("sr-role-navigation"),
        "main" => i18n.t("sr-role-main"),
        "banner" => i18n.t("sr-role-banner"),
        "contentinfo" => i18n.t("sr-role-contentinfo"),
        role => role.to_string(),
    }
}

fn state_labels(item: &ReadingItem, i18n: &I18n) -> Vec<String> {
    let mut labels = Vec::new();

    for state in &item.states {
        match state_parts(state) {
            ("expanded", Some("false")) => labels.push(i18n.t("sr-state-collapsed")),
            ("expanded", _) => labels.push(i18n.t("sr-state-expanded")),
            ("checked", Some("false")) => labels.push(i18n.t("sr-state-unchecked")),
            ("checked", Some("mixed")) => labels.push(i18n.t("sr-state-mixed")),
            ("checked", _) => labels.push(i18n.t("sr-state-checked")),
            ("selected", Some("false")) => labels.push(i18n.t("sr-state-not-selected")),
            ("selected", _) => labels.push(i18n.t("sr-state-selected")),
            ("required", Some("false")) => {}
            ("required", _) => labels.push(i18n.t("sr-state-required")),
            ("invalid", Some("false")) => {}
            ("invalid", _) => labels.push(i18n.t("sr-state-invalid")),
            ("disabled", Some("false")) => {}
            ("disabled", _) => labels.push(i18n.t("sr-state-disabled")),
            ("pressed", Some("false")) => labels.push(i18n.t("sr-state-not-pressed")),
            ("pressed", _) => labels.push(i18n.t("sr-state-pressed")),
            ("level", _) => {}
            _ => {}
        }
    }

    if item.tab_stop && !is_natively_interactive(item.role.as_deref()) {
        labels.push(i18n.t("sr-state-tab-stop"));
    }

    labels
}

fn state_value<'a>(states: &'a [String], name: &str) -> Option<&'a str> {
    states
        .iter()
        .filter_map(|state| state.split_once('='))
        .find_map(|(state_name, value)| (state_name == name).then_some(value))
}

fn state_parts(state: &str) -> (&str, Option<&str>) {
    match state.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (state, None),
    }
}

fn is_natively_interactive(role: Option<&str>) -> bool {
    matches!(
        role,
        Some(
            "button"
                | "link"
                | "textbox"
                | "searchbox"
                | "checkbox"
                | "radio"
                | "combobox"
                | "listbox"
                | "slider"
                | "spinbutton"
                | "tab"
        )
    )
}

#[cfg(test)]
mod tests {
    use crate::i18n::I18n;

    use super::{announce, announce_localized};
    use crate::screen_reader::ReadingItem;

    fn item(role: &str, name: Option<&str>, states: Vec<&str>) -> ReadingItem {
        ReadingItem {
            seq: 0,
            role: Some(role.to_string()),
            name: name.map(String::from),
            description: None,
            value: None,
            states: states.into_iter().map(String::from).collect(),
            tab_stop: false,
            depth: 0,
            node_id: "1".into(),
        }
    }

    #[test]
    fn announces_common_german_role_and_state_combinations() {
        insta::assert_snapshot!(announce(&item("link", Some("Mehr erfahren"), vec![])), @"Mehr erfahren, Link");
        insta::assert_snapshot!(
            announce(&item("textbox", Some("Suche"), vec!["required", "invalid"])),
            @"Suche, Textfeld, erforderlich, ungültig"
        );
        insta::assert_snapshot!(
            announce(&item("heading", Some("Willkommen"), vec!["level=1"])),
            @"Willkommen, Überschrift Ebene 1"
        );
        insta::assert_snapshot!(
            announce(&item("button", Some("Filter"), vec!["expanded=false"])),
            @"Filter, Schalter, eingeklappt"
        );
    }

    #[test]
    fn announces_empty_name_instead_of_silence() {
        insta::assert_snapshot!(announce(&item("button", None, vec![])), @"(kein Name), Schalter");
    }

    #[test]
    fn announces_english_locale() {
        let i18n = I18n::new("en").expect("locale parses");
        insta::assert_snapshot!(
            announce_localized(&item("checkbox", Some("Newsletter"), vec!["checked"]), &i18n),
            @"Newsletter, checkbox, checked"
        );
    }

    #[test]
    fn announces_tab_stop_for_non_interactive_role() {
        let mut generic = item("generic", Some("Card"), vec![]);
        generic.tab_stop = true;

        insta::assert_snapshot!(announce(&generic), @"Card, generic, fokussierbar");
    }
}
