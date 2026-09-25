//! Aus der Ansage-Struktur einen Satz machen.
//!
//! Woraus eine Ansage besteht, entscheidet `a11y-perception`: welche Teile, in
//! welcher Reihenfolge, welcher Zustand nichts hinzufügt. Hier stehen nur die
//! Wörter — die Zuordnung von jedem Teil auf einen Schlüssel der
//! Berichtssprache, und das Trennzeichen dazwischen.
//!
//! Die Trennung ist der Grund, warum dieser Renderer bei auditmysite bleibt und
//! nicht mitgewandert ist: `a11y-perception` kennt keine Locale, und es soll
//! auch keine kennen.

use a11y_perception::{AnnouncedRole, AnnouncedState};

use crate::i18n::I18n;

use super::types::ReadingItem;

/// Announce a reading item using the default German locale.
pub fn announce(item: &ReadingItem) -> String {
    let i18n = I18n::new("de").expect("German report locale must parse");
    announce_localized(item, &i18n)
}

/// Announce a reading item in the supplied locale.
pub fn announce_localized(item: &ReadingItem, i18n: &I18n) -> String {
    let announcement = a11y_perception::announce(item);

    // Ein namenloser Knoten bekommt Worte statt Schweigen -- „(kein Name)" ist
    // selbst die Auskunft, denn ein Schalter ohne Namen ist ein Befund.
    let mut parts = vec![announcement.name.unwrap_or_else(|| i18n.t("sr-no-name"))];

    parts.push(role_label(&announcement.role, i18n));
    parts.extend(
        announcement
            .states
            .into_iter()
            .map(|state| state_label(state, i18n)),
    );

    parts.join(", ")
}

fn role_label(role: &AnnouncedRole, i18n: &I18n) -> String {
    match role {
        AnnouncedRole::Heading { level: Some(level) } => {
            // Als Zeichenkette, nicht als Zahl: Fluent würde eine Zahl nach
            // Locale formatieren, und eine Überschriftenebene ist keine Menge.
            let level = level.to_string();
            i18n.t_args("sr-role-heading-level", &[("level", level.as_str())])
        }
        AnnouncedRole::Heading { level: None } => i18n.t("sr-role-heading"),
        AnnouncedRole::Button => i18n.t("sr-role-button"),
        AnnouncedRole::Link => i18n.t("sr-role-link"),
        AnnouncedRole::TextBox => i18n.t("sr-role-textbox"),
        AnnouncedRole::CheckBox => i18n.t("sr-role-checkbox"),
        AnnouncedRole::Radio => i18n.t("sr-role-radio"),
        AnnouncedRole::ComboBox => i18n.t("sr-role-combobox"),
        AnnouncedRole::ListBox => i18n.t("sr-role-listbox"),
        AnnouncedRole::Slider => i18n.t("sr-role-slider"),
        AnnouncedRole::SpinButton => i18n.t("sr-role-spinbutton"),
        AnnouncedRole::Tab => i18n.t("sr-role-tab"),
        AnnouncedRole::Navigation => i18n.t("sr-role-navigation"),
        AnnouncedRole::Main => i18n.t("sr-role-main"),
        AnnouncedRole::Banner => i18n.t("sr-role-banner"),
        AnnouncedRole::ContentInfo => i18n.t("sr-role-contentinfo"),
        // Eine Rolle ohne eigene Benennung wird unübersetzt genannt. Besser der
        // rohe Rollenname als gar keine Rolle.
        AnnouncedRole::Other(role) => role.clone(),
    }
}

fn state_label(state: AnnouncedState, i18n: &I18n) -> String {
    match state {
        AnnouncedState::Expanded => i18n.t("sr-state-expanded"),
        AnnouncedState::Collapsed => i18n.t("sr-state-collapsed"),
        AnnouncedState::Checked => i18n.t("sr-state-checked"),
        AnnouncedState::Unchecked => i18n.t("sr-state-unchecked"),
        AnnouncedState::Mixed => i18n.t("sr-state-mixed"),
        AnnouncedState::Selected => i18n.t("sr-state-selected"),
        AnnouncedState::NotSelected => i18n.t("sr-state-not-selected"),
        AnnouncedState::Required => i18n.t("sr-state-required"),
        AnnouncedState::Invalid => i18n.t("sr-state-invalid"),
        AnnouncedState::Disabled => i18n.t("sr-state-disabled"),
        AnnouncedState::Pressed => i18n.t("sr-state-pressed"),
        AnnouncedState::NotPressed => i18n.t("sr-state-not-pressed"),
        AnnouncedState::Focusable => i18n.t("sr-state-tab-stop"),
    }
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

    #[test]
    fn every_announced_role_and_state_has_a_german_word() {
        // Die Zuordnung ist der ganze Zweck dieses Moduls. Faellt in
        // a11y-perception eine Variante hinzu, muss der Match hier
        // nachgezogen werden -- der Compiler erzwingt das. Was er nicht
        // erzwingt, ist ein *vorhandener* Schluessel: `I18n::t` gibt den
        // Schluessel selbst zurueck, wenn er fehlt, und eine Ansage wie
        // "Suche, sr-role-textbox" wuerde sonst durchgehen.
        use a11y_perception::{AnnouncedRole, AnnouncedState};

        let i18n = I18n::new("de").expect("German report locale must parse");

        let roles = [
            AnnouncedRole::Button,
            AnnouncedRole::Link,
            AnnouncedRole::TextBox,
            AnnouncedRole::CheckBox,
            AnnouncedRole::Radio,
            AnnouncedRole::ComboBox,
            AnnouncedRole::ListBox,
            AnnouncedRole::Slider,
            AnnouncedRole::SpinButton,
            AnnouncedRole::Tab,
            AnnouncedRole::Heading { level: Some(2) },
            AnnouncedRole::Heading { level: None },
            AnnouncedRole::Navigation,
            AnnouncedRole::Main,
            AnnouncedRole::Banner,
            AnnouncedRole::ContentInfo,
        ];
        for role in &roles {
            let label = super::role_label(role, &i18n);
            assert!(
                !label.starts_with("sr-role-"),
                "kein deutsches Wort fuer {role:?}, nur der Schluessel {label}"
            );
        }

        let states = [
            AnnouncedState::Expanded,
            AnnouncedState::Collapsed,
            AnnouncedState::Checked,
            AnnouncedState::Unchecked,
            AnnouncedState::Mixed,
            AnnouncedState::Selected,
            AnnouncedState::NotSelected,
            AnnouncedState::Required,
            AnnouncedState::Invalid,
            AnnouncedState::Disabled,
            AnnouncedState::Pressed,
            AnnouncedState::NotPressed,
            AnnouncedState::Focusable,
        ];
        for state in states {
            let label = super::state_label(state, &i18n);
            assert!(
                !label.starts_with("sr-state-"),
                "kein deutsches Wort fuer {state:?}, nur der Schluessel {label}"
            );
        }
    }
}
