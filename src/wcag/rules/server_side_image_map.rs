//! WCAG 1.1.1 - Server-side Image Map
//!
//! axe-core rule: `server-side-image-map`
//! Server-side image maps are inaccessible because the coordinates are sent
//! to the server and keyboard users cannot interact with them meaningfully.
//! Recommends replacing with client-side image maps or text links.
//!
//! DOM-level rule: `ismap` is a boolean HTML attribute, never exposed as an
//! AX property — an earlier tree-based implementation of this check read a
//! non-existent AX property and never fired in production (#QA-030).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const RULE_SERVER_SIDE_IMAGE_MAP: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Server-side Image Map",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description:
        "Server-side image maps must not be used; use client-side image maps or text links instead",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "server-side-image-map",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

const SERVER_SIDE_IMAGE_MAP_CAP: usize = 250;

const SERVER_SIDE_IMAGE_MAP_BODY: &str = r#"
  var issues = [];
  var images = document.querySelectorAll('img[ismap]');
  for (var i = 0; i < images.length && issues.length < CAP; i++) {
    issues.push({ selector: __amsCssSelector(images[i]) });
  }
  return { issues: issues };
"#;

/// Check for server-side image maps (`<img ismap>`).
pub async fn check_server_side_image_map_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &SERVER_SIDE_IMAGE_MAP_BODY.replace("CAP", &SERVER_SIDE_IMAGE_MAP_CAP.to_string()),
        "})()",
    ]
    .concat();

    let val = match crate::wcag::types::evaluate_or_fail_for(
        page,
        "server-side-image-map",
        crate::cli::WcagLevel::A,
        js.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let issues = match val.get("issues").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();

            Some(
                Violation::new(
                    RULE_SERVER_SIDE_IMAGE_MAP.id,
                    RULE_SERVER_SIDE_IMAGE_MAP.name,
                    RULE_SERVER_SIDE_IMAGE_MAP.level,
                    RULE_SERVER_SIDE_IMAGE_MAP.severity,
                    "Image uses a server-side image map (ismap attribute), which is inaccessible to keyboard and screen reader users",
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix("Replace server-side image map with client-side image map or text links")
                .with_rule_id(RULE_SERVER_SIDE_IMAGE_MAP.axe_id)
                .with_help_url(RULE_SERVER_SIDE_IMAGE_MAP.help_url),
            )
        })
        .collect()
}
