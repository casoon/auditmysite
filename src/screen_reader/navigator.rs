use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::types::ReadingItem;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationViews {
    pub headings: Vec<HeadingNavItem>,
    pub landmarks: Vec<LandmarkNavItem>,
    pub links: Vec<LinkNavItem>,
    pub form_controls: Vec<FormControlNavItem>,
    pub tables: Vec<TableNavItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadingNavItem {
    pub level: Option<u8>,
    pub text: Option<String>,
    pub seq: usize,
    pub node_id: String,
    pub quality: HeadingQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadingQuality {
    Good,
    SkippedLevel,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LandmarkNavItem {
    pub role: String,
    pub name: Option<String>,
    pub seq: usize,
    pub node_id: String,
    pub quality: LandmarkQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandmarkQuality {
    Ok,
    UnlabeledDuplicate,
    MissingMain,
    NestedMain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkNavItem {
    pub text: Option<String>,
    pub count: usize,
    pub seq_positions: Vec<usize>,
    pub node_ids: Vec<String>,
    pub quality: LinkQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkQuality {
    Good,
    ContextDependent,
    NonDescriptive,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormControlNavItem {
    pub label: Option<String>,
    pub control_type: String,
    pub required: bool,
    pub seq: usize,
    pub node_id: String,
    pub quality: FormControlQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormControlQuality {
    Good,
    EmptyLabel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableNavItem {
    pub caption: Option<String>,
    pub header_strategy: TableHeaderStrategy,
    pub seq: usize,
    pub node_id: String,
    pub quality: TableQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableHeaderStrategy {
    Unknown,
    RowOrColumnHeaders,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableQuality {
    Good,
    MissingCaption,
}

pub fn navigation_views(items: &[ReadingItem]) -> NavigationViews {
    NavigationViews {
        headings: headings(items),
        landmarks: landmarks(items),
        links: links(items),
        form_controls: form_controls(items),
        tables: tables(items),
    }
}

fn headings(items: &[ReadingItem]) -> Vec<HeadingNavItem> {
    let mut previous_level: Option<u8> = None;

    items
        .iter()
        .filter(|item| item.role.as_deref() == Some("heading"))
        .map(|item| {
            let level = state_value(&item.states, "level").and_then(|value| value.parse().ok());
            let quality = if is_empty(&item.name) {
                HeadingQuality::Empty
            } else if let (Some(previous), Some(current)) = (previous_level, level) {
                if current > previous + 1 {
                    HeadingQuality::SkippedLevel
                } else {
                    HeadingQuality::Good
                }
            } else {
                HeadingQuality::Good
            };
            previous_level = level.or(previous_level);

            HeadingNavItem {
                level,
                text: item.name.clone(),
                seq: item.seq,
                node_id: item.node_id.clone(),
                quality,
            }
        })
        .collect()
}

fn landmarks(items: &[ReadingItem]) -> Vec<LandmarkNavItem> {
    let mut landmarks: Vec<_> = items
        .iter()
        .filter_map(|item| {
            let role = item.role.as_deref()?;
            is_landmark_role(role).then(|| LandmarkNavItem {
                role: role.to_string(),
                name: item.name.clone(),
                seq: item.seq,
                node_id: item.node_id.clone(),
                quality: LandmarkQuality::Ok,
            })
        })
        .collect();

    let has_main = landmarks.iter().any(|item| item.role == "main");
    let main_count = landmarks.iter().filter(|item| item.role == "main").count();
    let unlabeled_role_counts = landmarks.iter().fold(HashMap::new(), |mut acc, item| {
        if is_empty(&item.name) {
            *acc.entry(item.role.clone()).or_insert(0usize) += 1;
        }
        acc
    });

    for item in &mut landmarks {
        if item.role == "main" && main_count > 1 {
            item.quality = LandmarkQuality::NestedMain;
        } else if item.role == "main" && !has_main {
            item.quality = LandmarkQuality::MissingMain;
        } else if is_empty(&item.name)
            && unlabeled_role_counts
                .get(&item.role)
                .copied()
                .unwrap_or_default()
                > 1
        {
            item.quality = LandmarkQuality::UnlabeledDuplicate;
        }
    }

    if !has_main {
        landmarks.push(LandmarkNavItem {
            role: "main".into(),
            name: None,
            seq: 0,
            node_id: String::new(),
            quality: LandmarkQuality::MissingMain,
        });
    }

    landmarks
}

fn links(items: &[ReadingItem]) -> Vec<LinkNavItem> {
    let mut grouped: BTreeMap<String, LinkNavItem> = BTreeMap::new();

    for item in items
        .iter()
        .filter(|item| item.role.as_deref() == Some("link"))
    {
        let key = item
            .name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or("")
            .to_lowercase();

        let entry = grouped.entry(key).or_insert_with(|| LinkNavItem {
            text: item.name.clone(),
            count: 0,
            seq_positions: Vec::new(),
            node_ids: Vec::new(),
            quality: link_quality(item.name.as_deref()),
        });

        entry.count += 1;
        entry.seq_positions.push(item.seq);
        entry.node_ids.push(item.node_id.clone());
    }

    grouped.into_values().collect()
}

fn form_controls(items: &[ReadingItem]) -> Vec<FormControlNavItem> {
    items
        .iter()
        .filter_map(|item| {
            let role = item.role.as_deref()?;
            is_form_control_role(role).then(|| FormControlNavItem {
                label: item.name.clone(),
                control_type: role.to_string(),
                required: has_state(&item.states, "required"),
                seq: item.seq,
                node_id: item.node_id.clone(),
                quality: if is_empty(&item.name) {
                    FormControlQuality::EmptyLabel
                } else {
                    FormControlQuality::Good
                },
            })
        })
        .collect()
}

fn tables(items: &[ReadingItem]) -> Vec<TableNavItem> {
    items
        .iter()
        .filter(|item| matches!(item.role.as_deref(), Some("table") | Some("grid")))
        .map(|item| {
            let header_strategy = if has_state(&item.states, "rowheader")
                || has_state(&item.states, "columnheader")
            {
                TableHeaderStrategy::RowOrColumnHeaders
            } else {
                TableHeaderStrategy::Unknown
            };

            TableNavItem {
                caption: item.name.clone(),
                header_strategy,
                seq: item.seq,
                node_id: item.node_id.clone(),
                quality: if is_empty(&item.name) {
                    TableQuality::MissingCaption
                } else {
                    TableQuality::Good
                },
            }
        })
        .collect()
}

fn is_landmark_role(role: &str) -> bool {
    matches!(
        role,
        "banner"
            | "navigation"
            | "main"
            | "contentinfo"
            | "complementary"
            | "search"
            | "form"
            | "region"
    )
}

fn is_form_control_role(role: &str) -> bool {
    matches!(
        role,
        "textbox"
            | "searchbox"
            | "checkbox"
            | "radio"
            | "combobox"
            | "listbox"
            | "spinbutton"
            | "slider"
    )
}

fn link_quality(text: Option<&str>) -> LinkQuality {
    let Some(text) = text.map(str::trim).filter(|text| !text.is_empty()) else {
        return LinkQuality::Empty;
    };

    let normalized = text.to_lowercase();
    if matches!(
        normalized.as_str(),
        "hier" | "here" | "mehr" | "more" | "weiter" | "link" | "click here" | "hier klicken"
    ) {
        LinkQuality::NonDescriptive
    } else if matches!(
        normalized.as_str(),
        "mehr erfahren" | "weiterlesen" | "read more" | "learn more" | "details"
    ) {
        LinkQuality::ContextDependent
    } else {
        LinkQuality::Good
    }
}

fn state_value<'a>(states: &'a [String], name: &str) -> Option<&'a str> {
    states
        .iter()
        .filter_map(|state| state.split_once('='))
        .find_map(|(state_name, value)| (state_name == name).then_some(value))
}

fn has_state(states: &[String], name: &str) -> bool {
    states.iter().any(|state| {
        state == name
            || state
                .strip_prefix(name)
                .is_some_and(|rest| rest.starts_with('='))
    })
}

fn is_empty(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(|value| value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::{
        navigation_views, HeadingQuality, LandmarkQuality, LinkQuality, TableHeaderStrategy,
        TableQuality,
    };
    use crate::screen_reader::ReadingItem;

    fn item(seq: usize, role: &str, name: Option<&str>, states: Vec<&str>) -> ReadingItem {
        ReadingItem {
            seq,
            role: Some(role.to_string()),
            name: name.map(String::from),
            description: None,
            value: None,
            states: states.into_iter().map(String::from).collect(),
            tab_stop: false,
            depth: 0,
            node_id: format!("node-{seq}"),
        }
    }

    #[test]
    fn extracts_heading_list_and_detects_skipped_levels() {
        let views = navigation_views(&[
            item(0, "heading", Some("Start"), vec!["level=1"]),
            item(1, "heading", Some("Details"), vec!["level=2"]),
            item(2, "heading", Some("Deep"), vec!["level=4"]),
        ]);

        assert_eq!(views.headings[0].quality, HeadingQuality::Good);
        assert_eq!(views.headings[1].quality, HeadingQuality::Good);
        assert_eq!(views.headings[2].quality, HeadingQuality::SkippedLevel);
    }

    #[test]
    fn groups_identical_link_texts_with_count() {
        let views = navigation_views(&[
            item(0, "link", Some("Mehr erfahren"), vec![]),
            item(1, "link", Some("Mehr erfahren"), vec![]),
            item(2, "link", Some("Kontakt"), vec![]),
        ]);

        let grouped = views
            .links
            .iter()
            .find(|link| link.text.as_deref() == Some("Mehr erfahren"))
            .expect("link group exists");

        assert_eq!(grouped.count, 2);
        assert_eq!(grouped.seq_positions, vec![0, 1]);
        assert_eq!(grouped.quality, LinkQuality::ContextDependent);
    }

    #[test]
    fn marks_unlabeled_duplicate_navigation_landmarks() {
        let views = navigation_views(&[
            item(0, "navigation", None, vec![]),
            item(1, "main", Some("Inhalt"), vec![]),
            item(2, "navigation", None, vec![]),
        ]);

        let navs: Vec<_> = views
            .landmarks
            .iter()
            .filter(|landmark| landmark.role == "navigation")
            .collect();

        assert_eq!(navs.len(), 2);
        assert!(navs
            .iter()
            .all(|nav| nav.quality == LandmarkQuality::UnlabeledDuplicate));
    }

    #[test]
    fn emits_missing_main_landmark_problem() {
        let views = navigation_views(&[item(0, "navigation", Some("Main nav"), vec![])]);

        assert!(views
            .landmarks
            .iter()
            .any(|landmark| landmark.quality == LandmarkQuality::MissingMain));
    }

    #[test]
    fn extracts_forms_and_tables() {
        let views = navigation_views(&[
            item(0, "textbox", Some("E-Mail"), vec!["required"]),
            item(1, "table", None, vec!["columnheader"]),
        ]);

        assert_eq!(views.form_controls.len(), 1);
        assert!(views.form_controls[0].required);
        assert_eq!(views.tables[0].quality, TableQuality::MissingCaption);
        assert_eq!(
            views.tables[0].header_strategy,
            TableHeaderStrategy::RowOrColumnHeaders
        );
    }
}
