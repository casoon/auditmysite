//! The one SEO issue type shared by the technical and page-health collectors (#939).

use serde::{Deserialize, Serialize};

use crate::taxonomy::Severity;

/// An SEO issue: stable `issue_type` id, message text and severity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoIssue {
    pub issue_type: String,
    pub message: String,
    pub severity: Severity,
}
