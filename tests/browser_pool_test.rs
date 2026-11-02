//! Browser pool reliability (#856): context isolation and destruction,
//! permit accounting, failed and hung pages, cancellation, browser shutdown.
//!
//! Every test needs a real Chrome and a local fixture, so all are `#[ignore]`:
//!
//!   cargo test --test browser_pool_test -- --ignored --test-threads=1
//!
//! The tests share one process, and the shutdown test counts Chrome processes
//! by this process's profile prefix, so they take a common lock and run one
//! at a time even without `--test-threads=1`.

mod common;

use std::io::Read;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use auditmysite::browser::{BrowserOptions, BrowserPool, PoolConfig, SessionPolicy};
use auditmysite::error::AuditError;
use tokio::sync::Mutex;

static ONE_AT_A_TIME: Mutex<()> = Mutex::const_new(());

const PAGE: &str =
    "<!doctype html><html lang=en><title>Pool</title><main><h1>Pool</h1></main></html>";

async fn pool(max_pages: usize, session: SessionPolicy, acquire_timeout_secs: u64) -> BrowserPool {
    BrowserPool::new(PoolConfig {
        max_pages,
        acquire_timeout_secs,
        browser_options: BrowserOptions {
            no_sandbox: std::env::var("CI").is_ok(),
            session,
            timeout_secs: 12,
            ..Default::default()
        },
    })
    .await
    .expect("pool")
}

/// Poll `ok` for up to `secs`; the slot of a dropped page comes back
/// asynchronously (reset or close runs in a spawned task).
async fn settles(secs: u64, mut ok: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        if ok() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    ok()
}

/// Browser contexts Chrome currently holds besides the default one.
async fn context_count(pool: &BrowserPool) -> usize {
    pool.browser()
        .browser_context_count()
        .await
        .expect("Target.getBrowserContexts")
}

fn profile_prefix() -> String {
    format!("auditmysite-chrome-{}-", std::process::id())
}

/// Profile directories of Chrome instances this test process launched.
fn profile_dirs() -> Vec<std::path::PathBuf> {
    let prefix = profile_prefix();
    std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .map(|e| e.path())
        .collect()
}

/// Running processes whose command line carries one of this process's profile
/// directories (the browser and its helpers).
fn chrome_processes() -> usize {
    let out = std::process::Command::new("ps")
        .args(["-axo", "command"])
        .output()
        .expect("ps");
    let needle = format!("--user-data-dir={}", std::env::temp_dir().display());
    let prefix = profile_prefix();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.contains(&needle) && l.contains(&prefix))
        .count()
}

/// A server that accepts connections and never answers.
struct Silent {
    url: String,
    stop: Arc<AtomicBool>,
}

fn silent_server() -> Silent {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    listener.set_nonblocking(true).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    std::thread::spawn(move || {
        let mut held = Vec::new();
        while !flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
                    let mut buf = [0u8; 1024];
                    let _ = stream.read(&mut buf);
                    held.push(stream);
                }
                Err(_) => std::thread::sleep(Duration::from_millis(20)),
            }
        }
    });
    Silent { url, stop }
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn isolated_pages_get_their_own_context_and_it_is_destroyed_on_return() {
    let _one = ONE_AT_A_TIME.lock().await;
    let pool = pool(2, SessionPolicy::Isolated, 30).await;

    let first = pool.acquire().await.expect("first");
    let second = pool.acquire().await.expect("second");
    assert_eq!(
        context_count(&pool).await,
        2,
        "one context per isolated page"
    );

    drop(second);
    assert!(
        settles(15, || pool.stats().pages_created == 1).await,
        "{}",
        pool.stats()
    );
    assert_eq!(
        context_count(&pool).await,
        1,
        "the returned page's context must be disposed, not kept"
    );

    drop(first);
    assert!(settles(15, || pool.stats().pages_created == 0).await);
    // A new page gets a context of its own again.
    let third = pool.acquire().await.expect("third");
    assert_eq!(context_count(&pool).await, 1);
    drop(third);
    pool.close().await.ok();
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn shared_pages_use_no_extra_context_and_are_reused() {
    let _one = ONE_AT_A_TIME.lock().await;
    let pool = pool(2, SessionPolicy::Shared, 30).await;
    let page = pool.acquire().await.expect("page");
    assert_eq!(context_count(&pool).await, 0);
    drop(page);
    assert!(settles(15, || pool.stats().permits_available == 2).await);
    assert_eq!(pool.available_pages().await, 1, "the page goes back");
    let _again = pool.acquire().await.expect("again");
    assert_eq!(pool.stats().pages_created, 1, "reused, not created anew");
    pool.close().await.ok();
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn permits_run_out_at_max_pages_and_come_back_on_drop() {
    let _one = ONE_AT_A_TIME.lock().await;
    let pool = pool(2, SessionPolicy::Shared, 1).await;

    let a = pool.acquire().await.expect("a");
    let b = pool.acquire().await.expect("b");
    let stats = pool.stats();
    assert_eq!((stats.pages_created, stats.permits_available), (2, 0));

    let started = Instant::now();
    let third = pool.acquire().await;
    assert!(
        matches!(third, Err(AuditError::PoolTimeout { timeout_secs: 1 })),
        "a third page must wait and time out, not exceed max_pages"
    );
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(pool.stats().pages_created, 2, "a timeout creates nothing");

    drop(a);
    assert!(settles(15, || pool.stats().permits_available == 1).await);
    let c = pool.acquire().await.expect("slot is free again");
    drop((b, c));
    assert!(settles(15, || pool.stats().permits_available == 2).await);
    assert!(pool.stats().pages_created <= 2);
    pool.close().await.ok();
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn isolated_return_gives_the_slot_back_not_only_the_permit() {
    let _one = ONE_AT_A_TIME.lock().await;
    let pool = pool(1, SessionPolicy::Isolated, 30).await;
    for _ in 0..3 {
        let page = pool.acquire().await.expect("page");
        drop(page);
        assert!(
            settles(15, || {
                let s = pool.stats();
                s.permits_available == 1 && s.pages_created == 0
            })
            .await,
            "{}",
            pool.stats()
        );
    }
    pool.close().await.ok();
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn a_waiting_acquire_that_is_cancelled_leaks_no_permit() {
    let _one = ONE_AT_A_TIME.lock().await;
    let pool = pool(1, SessionPolicy::Shared, 30).await;
    let held = pool.acquire().await.expect("held");

    let waiting = tokio::time::timeout(Duration::from_millis(300), pool.acquire()).await;
    assert!(waiting.is_err(), "the pool has no free page");

    drop(held);
    assert!(settles(15, || pool.stats().permits_available == 1).await);
    let page = pool
        .acquire()
        .await
        .expect("the cancelled waiter took nothing");
    drop(page);
    pool.close().await.ok();
}

/// `acquire` forgets its permit and counts the slot before it awaits the new
/// page. A caller that gives up at that moment (its own timeout, a cancelled
/// task) must not leave the pool one slot smaller for the rest of the run.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn an_acquire_cancelled_during_page_creation_leaks_no_slot() {
    let _one = ONE_AT_A_TIME.lock().await;
    for session in [SessionPolicy::Shared, SessionPolicy::Isolated] {
        let pool = pool(1, session, 30).await;
        // Creating a page takes tens of milliseconds; cut it off at several
        // points inside that window.
        for micros in [100u64, 1_000, 5_000, 20_000] {
            let cancelled =
                tokio::time::timeout(Duration::from_micros(micros), pool.acquire()).await;
            if let Ok(Ok(page)) = cancelled {
                drop(page); // creation was faster than the cut: nothing to see
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        let acquired = tokio::time::timeout(Duration::from_secs(5), pool.acquire()).await;
        assert!(
            matches!(acquired, Ok(Ok(_))),
            "{session:?}: a cancelled acquire left the pool without a slot ({})",
            pool.stats()
        );
        drop(acquired);
        pool.close().await.ok();
    }
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn a_page_that_cannot_be_reset_is_discarded_and_its_slot_restored() {
    let _one = ONE_AT_A_TIME.lock().await;
    let pool = pool(1, SessionPolicy::Shared, 30).await;
    let pooled = pool.acquire().await.expect("page");
    // Closed behind the pool's back: the reset to about:blank will fail.
    pooled.page().unwrap().clone().close().await.ok();
    drop(pooled);

    assert!(
        settles(30, || {
            let s = pool.stats();
            s.permits_available == 1 && s.pages_created == 0
        })
        .await,
        "{}",
        pool.stats()
    );
    assert_eq!(pool.available_pages().await, 0, "a dead page is not pooled");
    let fresh = pool.acquire().await.expect("a fresh page");
    pool.browser()
        .navigate(fresh.page().unwrap(), "about:blank")
        .await
        .ok();
    drop(fresh);
    pool.close().await.ok();
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn a_page_stuck_in_a_script_loop_frees_its_slot_within_the_return_budget() {
    let _one = ONE_AT_A_TIME.lock().await;
    let fixture = common::fixture_server::serve_html(PAGE.to_string());
    let pool = pool(1, SessionPolicy::Shared, 30).await;
    let pooled = pool.acquire().await.expect("page");
    pool.browser()
        .navigate(pooled.page().unwrap(), &fixture.url)
        .await
        .expect("navigate");
    // Block the renderer's main thread; the evaluate never answers.
    let page = pooled.page().unwrap().clone();
    let stuck = tokio::spawn(async move { page.evaluate("for(;;){}").await.ok() });
    tokio::time::sleep(Duration::from_millis(500)).await;

    let dropped = Instant::now();
    drop(pooled);
    // Reset 10 s + close 5 s (PAGE_RETURN_BUDGET_SECS), plus scheduling slack.
    let budget = 15 + 5;
    assert!(
        settles(budget, || pool.stats().permits_available == 1).await,
        "slot not back after {budget}s: {}",
        pool.stats()
    );
    eprintln!("stuck page: slot back after {:?}", dropped.elapsed());
    let next = tokio::time::timeout(Duration::from_secs(20), pool.acquire()).await;
    assert!(matches!(next, Ok(Ok(_))), "the next page must be servable");
    drop(next);
    stuck.abort();
    pool.close().await.ok();
    fixture.stop();
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn a_server_that_never_answers_ends_in_an_error_within_the_timeout_budget() {
    let _one = ONE_AT_A_TIME.lock().await;
    let silent = silent_server();
    let pool = pool(1, SessionPolicy::Isolated, 30).await;
    let pooled = pool.acquire().await.expect("page");

    let started = Instant::now();
    let result = pool
        .browser()
        .navigate(pooled.page().unwrap(), &silent.url)
        .await;
    let took = started.elapsed();
    eprintln!("silent server: {result:?} after {took:?}");
    assert!(
        result.is_err(),
        "a page that never loaded must not count as loaded"
    );
    // timeout_secs = 12: goto budget 10 s plus the remainder, once more after
    // the retry pause.
    assert!(took < Duration::from_secs(60), "{took:?}");

    drop(pooled);
    assert!(settles(20, || pool.stats().permits_available == 1).await);
    pool.close().await.ok();
    silent.stop.store(true, Ordering::Relaxed);
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn a_cancelled_audit_task_returns_its_page() {
    let _one = ONE_AT_A_TIME.lock().await;
    let silent = silent_server();
    let pool = Arc::new(pool(1, SessionPolicy::Isolated, 30).await);

    let worker = {
        let pool = Arc::clone(&pool);
        let url = silent.url.clone();
        tokio::spawn(async move {
            let pooled = pool.acquire().await.expect("page");
            let _ = pool.browser().navigate(pooled.page().unwrap(), &url).await;
        })
    };
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(
        pool.stats().permits_available,
        0,
        "the worker holds the slot"
    );
    worker.abort();
    let _ = worker.await;

    assert!(
        settles(20, || pool.stats().permits_available == 1).await,
        "an aborted task must hand its slot back: {}",
        pool.stats()
    );
    let pool = Arc::try_unwrap(pool).ok().expect("sole owner");
    pool.close().await.ok();
    silent.stop.store(true, Ordering::Relaxed);
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn close_ends_the_browser_and_removes_its_profile() {
    let _one = ONE_AT_A_TIME.lock().await;
    assert_eq!(chrome_processes(), 0, "stale Chrome of this process");
    let profiles_before = profile_dirs().len();
    let pool = pool(2, SessionPolicy::Isolated, 30).await;
    let a = pool.acquire().await.expect("a");
    let b = pool.acquire().await.expect("b");
    assert!(chrome_processes() > 0, "Chrome should be running");
    assert_eq!(profile_dirs().len(), profiles_before + 1);

    // The last pages are dropped right before close: the return tasks still
    // hold the pool's state when close starts.
    drop((a, b));
    pool.close().await.expect("close");

    assert!(
        settles(10, || chrome_processes() == 0).await,
        "{} Chrome process(es) outlived close()",
        chrome_processes()
    );
    assert_eq!(
        profile_dirs().len(),
        profiles_before,
        "{:?}",
        profile_dirs()
    );
}
