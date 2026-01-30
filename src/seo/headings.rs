//! SEO heading structure analysis
//!
//! Analyzes H1-H6 heading hierarchy for SEO best practices.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

/// Heading structure analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadingStructure {
    /// Number of H1 elements
    pub h1_count: usize,
    /// H1 text content (first one if multiple)
    pub h1_text: Option<String>,
    /// All headings in order
    pub headings: Vec<HeadingInfo>,
    /// Heading issues found
    pub issues: Vec<HeadingIssue>,
    /// Total heading count
    pub total_count: usize,
}

/// Information about a single heading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingInfo {
    /// Heading level (1-6)
    pub level: u8,
    /// Heading text content
    pub text: String,
    /// Character count
    pub length: usize,
}

/// Heading-related SEO issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingIssue {
    /// Issue type
    pub issue_type: String,
    /// Issue description
    pub message: String,
    /// Severity: "error", "warning"
    pub severity: String,
}

/// Analyze heading structure of a page
pub async fn analyze_heading_structure(page: &Page) -> Result<HeadingStructure> {
    info!("Analyzing heading structure...");

    let js_code = r#"
    (() => {
        const headings = [];
        document.querySelectorAll('h1, h2, h3, h4, h5, h6').forEach(h => {
            const level = parseInt(h.tagName.charAt(1));
            const text = h.textContent.trim();
            headings.push({ level, text, length: text.length });
        });
        return JSON.stringify(headings);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Heading analysis failed: {}", e)))?;

    let json_str = js_result
        .value()
        .and_then(|v| v.as_str())
        .unwrap_or("[]");

    let headings: Vec<HeadingInfo> = serde_json::from_str(json_str).unwrap_or_default();

    // Analyze structure
    let h1_headings: Vec<_> = headings.iter().filter(|h| h.level == 1).collect();
    let h1_count = h1_headings.len();
    let h1_text = h1_headings.first().map(|h| h.text.clone());

    let mut issues = Vec::new();

    // Check for missing H1
    if h1_count == 0 {
        issues.push(HeadingIssue {
            issue_type: "missing_h1".to_string(),
            message: "Page is missing an H1 heading".to_string(),
            severity: "error".to_string(),
        });
    }

    // Check for multiple H1s
    if h1_count > 1 {
        issues.push(HeadingIssue {
            issue_type: "multiple_h1".to_string(),
            message: format!("Page has {} H1 headings (should have exactly 1)", h1_count),
            severity: "warning".to_string(),
        });
    }

    // Check for skipped heading levels
    let mut prev_level = 0u8;
    for heading in &headings {
        if prev_level > 0 && heading.level > prev_level + 1 {
            issues.push(HeadingIssue {
                issue_type: "skipped_level".to_string(),
                message: format!(
                    "Heading level skipped: H{} to H{} (\"{}\")",
                    prev_level,
                    heading.level,
                    truncate(&heading.text, 40)
                ),
                severity: "warning".to_string(),
            });
        }
        prev_level = heading.level;
    }

    // Check for empty headings
    for heading in &headings {
        if heading.text.is_empty() {
            issues.push(HeadingIssue {
                issue_type: "empty_heading".to_string(),
                message: format!("Empty H{} heading found", heading.level),
                severity: "error".to_string(),
            });
        }
    }

    // Check for very long headings
    for heading in &headings {
        if heading.length > 70 {
            issues.push(HeadingIssue {
                issue_type: "long_heading".to_string(),
                message: format!(
                    "H{} is too long ({} chars): \"{}...\"",
                    heading.level,
                    heading.length,
                    truncate(&heading.text, 30)
                ),
                severity: "warning".to_string(),
            });
        }
    }

    info!(
        "Heading structure: {} total, {} H1s, {} issues",
        headings.len(),
        h1_count,
        issues.len()
    );

    Ok(HeadingStructure {
        h1_count,
        h1_text,
        total_count: headings.len(),
        headings,
        issues,
    })
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_info() {
        let heading = HeadingInfo {
            level: 1,
            text: "Test Heading".to_string(),
            length: 12,
        };

        assert_eq!(heading.level, 1);
        assert_eq!(heading.length, 12);
    }
}
