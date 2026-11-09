//! Help-mechanism inventory for WCAG 3.2.6 Consistent Help (plan 54 §3).
//!
//! 3.2.6 is a cross-page criterion: a help mechanism that is repeated on
//! several pages must appear in the same order relative to the other page
//! content. A single page can't violate it, so this module only records, per
//! page, which help mechanisms exist and where; `audit::batch_consistency`
//! compares the inventories across the audited set.
//!
//! Recorded mechanisms (the four kinds the criterion names, as far as they
//! are recognisable in the DOM):
//! - human contact details — `mailto:` and `tel:` links,
//! - contact/help pages — links whose text or path names contact, support,
//!   help or FAQ,
//! - chat widgets — a short list of known vendors' container elements.
//!
//! Only page chrome is inventoried: anything inside `main` is page-specific
//! content (a contact link in a blog post), not a repeated mechanism, and
//! would turn every article into a false "inconsistency". Invisible elements
//! are skipped for the same reason — a collapsed mobile menu is not where the
//! mechanism is offered at this viewport.
//!
//! Score-neutral: the inventory feeds the batch report only.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::PatternAnalysis;

/// What kind of help a mechanism offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpKind {
    Email,
    Phone,
    ContactPage,
    Chat,
}

/// Where on the page a mechanism sits, relative to the page's landmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpRegion {
    Header,
    Navigation,
    Complementary,
    Footer,
    /// Fixed-position overlay, e.g. a chat launcher.
    Floating,
    /// Outside `main` but in no landmark.
    Other,
}

/// One help mechanism occurrence, in document order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpMechanism {
    pub kind: HelpKind,
    /// Identity across pages: `mailto:<address>`, `tel:<digits>`,
    /// `page:<host><path>` or `chat:<vendor>`.
    pub key: String,
    pub region: HelpRegion,
}

const HELP_MECHANISMS_JS: &str = include_str!("js/help_mechanisms.js");

/// Help mechanisms (contact, chat, help pages) and their regions.
pub(crate) static HELP_MECHANISMS: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "accessibility.help_mechanisms",
        module: "accessibility",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 1,
        scoring: false,
    };

/// Inventory the page's help mechanisms into `out.help_mechanisms`. A failed
/// evaluation leaves the inventory empty and is recorded as a failed
/// collection, so the batch comparison can tell it from a page without
/// help mechanisms (#777).
pub(crate) async fn detect(
    page: &Page,
    out: &mut PatternAnalysis,
    log: &crate::audit::collection::CollectionLog,
) {
    let inventory =
        crate::audit::collection::evaluate_js_value::<Vec<HelpMechanism>>(page, HELP_MECHANISMS_JS)
            .await
            .empty_if(Vec::is_empty);
    if let Some(mechanisms) = log.record(&HELP_MECHANISMS, inventory).into_value() {
        out.help_mechanisms = mechanisms;
    } else {
        warn!("Help-mechanism detection failed");
    }
}

/// Counter-cases of [`HELP_MECHANISMS`] (#777).
#[cfg(test)]
pub(crate) fn counter_cases() {
    crate::audit::collection::assert_js_value_contract::<Vec<HelpMechanism>>(serde_json::json!([]));
}
