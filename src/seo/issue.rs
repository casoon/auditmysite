//! The one SEO issue type shared by the technical and page-health collectors (#939).

use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};

use crate::taxonomy::Severity;

/// An SEO issue: stable `issue_type` id, message text and severity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoIssue {
    pub issue_type: String,
    pub message: String,
    pub severity: Severity,
}

impl SeoIssue {
    /// The issue `issue_type`: its message is the Fluent message `key` with
    /// `args`, in the requested language (#406).
    pub(crate) fn localized(
        issue_type: &str,
        severity: Severity,
        key: &str,
        args: &[(&str, FluentValue)],
        en: bool,
    ) -> Self {
        Self {
            issue_type: issue_type.to_string(),
            message: crate::i18n::text_args(key, args, en),
            severity,
        }
    }
}

/// A count as a Fluent argument (plural selectors need a number).
pub(crate) fn num<T: TryInto<i64>>(n: T) -> FluentValue<'static> {
    FluentValue::from(n.try_into().unwrap_or(i64::MAX))
}

/// Text as a Fluent argument.
pub(crate) fn txt(s: &str) -> FluentValue<'static> {
    FluentValue::from(s.to_string())
}
