//! Session policy (#787): pages of one run share cookies and storage only
//! when the measurement profile says so. Local fixture only — the page
//! writes a cookie and a localStorage entry and reports what it found
//! before doing so.
//!
//!   cargo test --test session_policy_test -- --ignored

mod common;

use auditmysite::browser::{BrowserOptions, BrowserPool, PoolConfig, SessionPolicy};

const PAGE: &str = "<!doctype html><html lang=en><title>State</title><main><h1>State</h1></main>\
<script>window.__seen = {cookie: document.cookie, storage: localStorage.getItem('k')};\
document.cookie = 'k=1'; localStorage.setItem('k', '1');</script></html>";

/// What the n-th page of a one-slot pool found when it loaded.
async fn seen_by_consecutive_pages(session: SessionPolicy, pages: usize) -> Vec<String> {
    let fixture = common::fixture_server::serve_html(PAGE.to_string());
    let pool = BrowserPool::new(PoolConfig {
        max_pages: 1,
        browser_options: BrowserOptions {
            no_sandbox: std::env::var("CI").is_ok(),
            session,
            ..Default::default()
        },
        ..Default::default()
    })
    .await
    .expect("pool");
    let mut seen = Vec::new();
    for _ in 0..pages {
        let pooled = pool.acquire().await.expect("page");
        let page = pooled.page().expect("page");
        pool.browser()
            .navigate(page, &fixture.url)
            .await
            .expect("navigate");
        let value: String = page
            .evaluate("JSON.stringify(window.__seen)")
            .await
            .expect("evaluate")
            .into_value()
            .expect("value");
        seen.push(value);
        drop(pooled);
        // The page goes back to the pool (or is discarded) asynchronously.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    pool.close().await.ok();
    fixture.stop();
    seen
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn isolated_pages_do_not_see_what_the_page_before_stored() {
    let seen = seen_by_consecutive_pages(SessionPolicy::Isolated, 2).await;
    assert_eq!(seen[0], seen[1], "order must not change the measurement");
    assert!(
        !seen[1].contains("k=1") && !seen[1].contains("\"1\""),
        "{seen:?}"
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn shared_pages_carry_state_to_the_next_page() {
    let seen = seen_by_consecutive_pages(SessionPolicy::Shared, 2).await;
    assert_ne!(seen[0], seen[1]);
    assert!(seen[1].contains("k=1"), "{seen:?}");
}

/// What a page finds at load when the run starts from `state`, then what the
/// next page finds when it starts from `next` (a login/consent switch).
async fn seen_with_states(
    first: Option<&auditmysite::browser::StorageState>,
    next: Option<&auditmysite::browser::StorageState>,
) -> Vec<String> {
    let fixture = common::fixture_server::serve_html(PAGE.to_string());
    let pool = BrowserPool::new(PoolConfig {
        max_pages: 1,
        browser_options: BrowserOptions {
            no_sandbox: std::env::var("CI").is_ok(),
            ..Default::default()
        },
        ..Default::default()
    })
    .await
    .expect("pool");
    let mut seen = Vec::new();
    for state in [first, next] {
        let pooled = pool.acquire().await.expect("page");
        let page = pooled.page().expect("page");
        if let Some(state) = state {
            state.apply(page, &fixture.url).await;
        }
        pool.browser()
            .navigate(page, &fixture.url)
            .await
            .expect("navigate");
        seen.push(
            page.evaluate("JSON.stringify(window.__seen)")
                .await
                .expect("evaluate")
                .into_value::<String>()
                .expect("value"),
        );
        drop(pooled);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    pool.close().await.ok();
    fixture.stop();
    seen
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn declared_state_is_present_on_each_page_and_does_not_leak_to_the_next() {
    // The fixture server's port is not known before it runs, so the state
    // names the host and path scope and the origin is matched at runtime
    // through a cookie only; localStorage is checked for the origin-bound
    // script separately in the unit tests.
    let state = auditmysite::browser::StorageState::from_bytes(
        br#"{"cookies":[{"name":"login","value":"user-a","domain":"127.0.0.1","path":"/"}]}"#,
    )
    .unwrap();
    let seen = seen_with_states(Some(&state), None).await;
    assert!(seen[0].contains("login=user-a"), "{seen:?}");
    assert!(
        !seen[1].contains("login"),
        "a login switch must not carry the previous state: {seen:?}"
    );
}
