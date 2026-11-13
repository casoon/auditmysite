//! Test support for the installer: a local HTTP server standing in for the
//! publisher, zip builders and a probe double. No test touches the network.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::future::BoxFuture;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::install_plan::{InstallSource, Platform, SourcePolicy};
use super::install_txn::{BrowserProbe, Installer, Limits};

#[derive(Clone)]
pub(super) enum Route {
    Ok(Vec<u8>),
    /// Announce `announce` bytes, send `body`, close the connection.
    Truncated {
        announce: u64,
        body: Vec<u8>,
    },
    /// Accept the connection and never answer.
    Hang,
    Redirect(String),
}

pub(super) struct TestServer {
    pub(super) addr: std::net::SocketAddr,
    routes: Arc<Mutex<HashMap<String, Route>>>,
    hits: Arc<Mutex<HashMap<String, usize>>>,
}

impl TestServer {
    pub(super) async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        warm_up_loopback(addr);
        let routes: Arc<Mutex<HashMap<String, Route>>> = Arc::default();
        let hits: Arc<Mutex<HashMap<String, usize>>> = Arc::default();
        let (r, h) = (routes.clone(), hits.clone());
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    return;
                };
                let (r, h) = (r.clone(), h.clone());
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 1024];
                    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
                        match sock.read(&mut chunk).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                    }
                    let head = String::from_utf8_lossy(&buf).to_string();
                    let mut parts = head.lines().next().unwrap_or("").split(' ');
                    let method = parts.next().unwrap_or("").to_string();
                    let path = parts.next().unwrap_or("").to_string();
                    *h.lock().unwrap().entry(path.clone()).or_default() += 1;
                    let route = r.lock().unwrap().get(&path).cloned();
                    let head_only = method == "HEAD";
                    match route {
                        None => {
                            let _ = sock
                                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                                .await;
                        }
                        Some(Route::Redirect(to)) => {
                            let _ = sock
                                .write_all(format!("HTTP/1.1 302 Found\r\nLocation: {to}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes())
                                .await;
                        }
                        Some(Route::Ok(body)) => {
                            let _ = sock
                                .write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes())
                                .await;
                            if !head_only {
                                let _ = sock.write_all(&body).await;
                            }
                        }
                        Some(Route::Truncated { announce, body }) => {
                            let _ = sock
                                .write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {announce}\r\nConnection: close\r\n\r\n").as_bytes())
                                .await;
                            let _ = sock.write_all(&body).await;
                            let _ = sock.shutdown().await;
                        }
                        Some(Route::Hang) => {
                            tokio::time::sleep(Duration::from_secs(60)).await;
                        }
                    }
                });
            }
        });
        Self { addr, routes, hits }
    }

    pub(super) fn route(&self, path: &str, route: Route) {
        self.routes.lock().unwrap().insert(path.to_string(), route);
    }

    pub(super) fn hits(&self, path: &str) -> usize {
        self.hits.lock().unwrap().get(path).copied().unwrap_or(0)
    }

    pub(super) fn total_hits(&self) -> usize {
        self.hits.lock().unwrap().values().sum()
    }

    pub(super) fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }

    /// Serve a Stable manifest for `version` and the archive at the canonical
    /// path for `platform`; returns the archive path.
    pub(super) fn publish_chrome(&self, version: &str, platform: Platform, zip: Vec<u8>) -> String {
        let id = platform.manifest_id();
        let path = format!("/{version}/{id}/chrome-{id}.zip");
        let manifest = serde_json::json!({"channels": {"Stable": {"version": version, "downloads": {
            "chrome": [{"platform": id, "url": self.url(&path)}]}}}});
        self.route("/latest.json", Route::Ok(manifest.to_string().into_bytes()));
        self.route(&path, Route::Ok(zip));
        path
    }
}

/// Zip with the given `(name, content, unix_mode)` entries.
pub(super) fn make_zip(entries: &[(&str, &[u8], Option<u32>)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, content, mode) in entries {
        let mut opts = zip::write::SimpleFileOptions::default();
        if let Some(m) = mode {
            opts = opts.unix_permissions(*m);
        }
        zip.start_file(*name, opts).unwrap();
        zip.write_all(content).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// A well-formed Linux x86_64 archive.
pub(super) fn good_linux_zip() -> Vec<u8> {
    make_zip(&[("chrome-linux64/chrome", b"#!/bin/sh\n", Some(0o755))])
}

pub(super) struct FakeProbe {
    pub(super) version_output: String,
    pub(super) smoke_ok: AtomicBool,
    pub(super) smoke_calls: AtomicUsize,
}

impl FakeProbe {
    pub(super) fn new(version: &str) -> Arc<Self> {
        Arc::new(Self {
            version_output: format!("Google Chrome for Testing {version}"),
            smoke_ok: AtomicBool::new(true),
            smoke_calls: AtomicUsize::new(0),
        })
    }
}

impl BrowserProbe for FakeProbe {
    fn version<'a>(&'a self, _binary: &'a Path) -> BoxFuture<'a, Result<String, String>> {
        Box::pin(async move { Ok(self.version_output.clone()) })
    }
    fn cdp_smoke<'a>(
        &'a self,
        _binary: &'a Path,
        _scratch: &'a Path,
    ) -> BoxFuture<'a, Result<(), String>> {
        Box::pin(async move {
            self.smoke_calls.fetch_add(1, Ordering::SeqCst);
            if self.smoke_ok.load(Ordering::SeqCst) {
                Ok(())
            } else {
                Err("no DevTools endpoint".to_string())
            }
        })
    }
}

/// Opens one loopback connection per test process, with a generous timeout,
/// before any test measures against `test_limits()`. A host network filter
/// (seen with LuLu on macOS) holds the first connection of a new process
/// while it identifies the binary, which can take longer than the 500 ms
/// connect timeout; later connections are not held. The connection completes
/// in the kernel's accept backlog, so the server need not be running yet, and
/// it sends no request, so no route is hit. Parallel tests wait for it.
fn warm_up_loopback(addr: std::net::SocketAddr) {
    static WARM_UP: std::sync::Once = std::sync::Once::new();
    WARM_UP.call_once(|| {
        let _ = std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(30));
    });
}

pub(super) fn test_limits() -> Limits {
    Limits {
        max_download_bytes: 10 * 1024 * 1024,
        connect_timeout: Duration::from_millis(500),
        total_timeout: Duration::from_secs(5),
        stall_timeout: Duration::from_millis(500),
        retries: 0,
        retry_backoff: Duration::from_millis(1),
        max_entries: 1000,
        max_extracted_bytes: 10 * 1024 * 1024,
    }
}

/// Installer for Linux x86_64 against `server`, with a probe double.
pub(super) fn test_installer(
    root: &Path,
    server: &TestServer,
    probe: Arc<dyn BrowserProbe>,
) -> Installer {
    Installer {
        root: root.to_path_buf(),
        source: InstallSource {
            latest_manifest: server.url("/latest.json"),
            known_good_manifest: server.url("/known.json"),
            policy: SourcePolicy::loopback(),
        },
        limits: test_limits(),
        platform: Platform::LinuxX64,
        probe,
        // Tests share one process; a sibling test spawning a shell can briefly
        // inherit the lock descriptor across fork.
        lock_wait: Duration::from_secs(5),
        show_progress: false,
    }
}
