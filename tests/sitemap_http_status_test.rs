//! Plan 87: the sitemap HTTP check records an outcome for every URL it
//! fetched, not only the ones that raised an issue — so "checked minus
//! issues" is never presented as a count of successful responses.

mod common;

use common::fixture_server;

#[tokio::test]
async fn status_counts_cover_every_checked_url() {
    let fixture = fixture_server::serve_html("<html><body>ok</body></html>".to_string());
    // A port nothing listens on: bind, read the port, release it.
    let closed = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        format!(
            "http://127.0.0.1:{}/gone",
            listener.local_addr().unwrap().port()
        )
    };
    let sitemap = vec![format!("{}/page", fixture.url), closed];

    let diagnostics = auditmysite::audit::analyze_sitemap_diagnostics(&sitemap, &[]).await;
    fixture.stop();
    assert!(fixture.served() >= 1, "{:?}", fixture.transport_errors());

    assert_eq!(diagnostics.checked_urls, 2);
    assert_eq!(diagnostics.status_counts.get("2xx"), Some(&1));
    assert_eq!(diagnostics.status_counts.get("fetch_error"), Some(&1));
    assert_eq!(diagnostics.status_counts.values().sum::<usize>(), 2);
    assert_eq!(diagnostics.http_issues.len(), 1);
}
