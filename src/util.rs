//! Shared utility functions

/// Truncate a string for display purposes (safe for multi-byte UTF-8)
pub fn truncate_url(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let target = max_len.saturating_sub(3);
    // Find the last char boundary at or before target
    let boundary = s
        .char_indices()
        .take_while(|(i, _)| *i <= target)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}...", &s[..boundary])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_url() {
        assert_eq!(
            truncate_url("https://example.com", 30),
            "https://example.com"
        );
    }

    #[test]
    fn test_truncate_long_url() {
        let url = "https://example.com/very/long/path/that/exceeds/limit";
        let result = truncate_url(url, 30);
        assert!(result.len() <= 30);
        assert!(result.ends_with("..."));
    }
}
