//! Shared utility functions

/// Truncate a URL for display purposes
pub fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        format!("{}...", &url[..max_len.saturating_sub(3)])
    }
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
