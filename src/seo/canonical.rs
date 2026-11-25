//! The canonical link in one place (#939): which findings it raises, how it is
//! compared with the primary host and with the audited URL.
//!
//! The tag itself is collected twice (`js/meta_tags.js` for the meta report,
//! `js/technical_signals.js` for the technical one); both fields are part of
//! the JSON. Everything that *judges* the value lives here.

use super::issue::{txt, SeoIssue};
use super::meta::{MetaIssueKind, MetaIssueValues, MetaValidation};
use crate::taxonomy::Severity;

/// Whether the canonical points to `primary_host`: `Some(true)` if it does,
/// `Some(false)` if it points elsewhere, `None` without a (parsable) canonical.
pub(super) fn matches_host(primary_host: &str, canonical: Option<&str>) -> Option<bool> {
    let parsed = url::Url::parse(canonical?).ok()?;
    Some(parsed.host_str() == Some(primary_host))
}

/// The Fluent key of the note on the canonical in the www-without-redirect issue.
pub(super) fn www_note_key(matches_primary: Option<bool>) -> &'static str {
    match matches_primary {
        Some(true) => "seo-technical-www-note-canonical-ok",
        Some(false) => "seo-technical-www-note-canonical-other",
        None => "seo-technical-www-note-canonical-none",
    }
}

/// The page declares no canonical link (technical report).
pub(super) fn missing_issue(en: bool) -> SeoIssue {
    SeoIssue::localized(
        "no_canonical",
        Severity::Medium,
        "seo-technical-no-canonical",
        &[],
        en,
    )
}

/// The canonical names another domain than the primary host.
pub(super) fn domain_mismatch_issue(primary: &str, en: bool) -> SeoIssue {
    SeoIssue::localized(
        "canonical_domain_mismatch",
        Severity::Medium,
        "seo-technical-canonical-domain-mismatch",
        &[("primary", txt(primary))],
        en,
    )
}

/// The page declares no canonical link (meta report; the score charges it once,
/// in the technical section).
pub(super) fn missing_meta_validation() -> MetaValidation {
    MetaValidation::new(
        "canonical",
        MetaIssueKind::MissingCanonical,
        MetaIssueValues::default(),
        Severity::Low,
        "Add <link rel=\"canonical\" href=\"...\">",
    )
}

/// Whether the canonical names a URL other than the audited one (trailing
/// slash and case of the whole URL ignored).
pub(super) fn points_elsewhere(canonical: &str, url: &str) -> bool {
    let canonical = canonical.trim_end_matches('/');
    !canonical.is_empty() && !canonical.eq_ignore_ascii_case(url.trim_end_matches('/'))
}
