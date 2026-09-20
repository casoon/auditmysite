//! One local fixture HTTP server for every integration test that needs one.
//!
//! Six copies of this server existed, identical apart from where the HTML came
//! from — and all six carried the same race (plan 48):
//!
//! The listener is set non-blocking so the accept loop can poll a shutdown
//! flag. On macOS and the BSDs the *accepted* socket inherits that flag, so
//! `read` and `write_all` on it can return `WouldBlock` before the request has
//! even arrived. Both results were discarded with `let _ =`, so the response
//! went out unwritten, the client saw nothing, and whatever the test was
//! measuring came back empty. Measured on the security corpus test: the read
//! failed on every single run, and under 16-way parallel load the test failed
//! 29 of 48 times — reporting a detection FALSE NEGATIVE, which is exactly the
//! message that should never be dismissed as noise.
//!
//! So: the accepted connection is switched back to blocking, the request head
//! is read to its end before the response is written, and a failure to serve
//! is recorded instead of swallowed — see [`Fixture::transport_errors`].

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// A running fixture server, plus what it managed to do.
pub struct Fixture {
    /// Base URL, e.g. `http://127.0.0.1:53124`.
    pub url: String,
    /// Set to `true` to stop the accept loop.
    pub shutdown: Arc<AtomicBool>,
    served: Arc<AtomicUsize>,
    errors: Arc<Mutex<Vec<String>>>,
}

impl Fixture {
    /// How many responses were written in full.
    ///
    /// Worth asserting before reading anything into a test's result: several
    /// analyses deliberately tolerate a failed fetch and carry on with empty
    /// data (`analyze_security`, for one), so a server that never answered is
    /// otherwise indistinguishable from a detector that found nothing.
    pub fn served(&self) -> usize {
        self.served.load(Ordering::Relaxed)
    }

    /// Everything that went wrong while serving, in order.
    pub fn transport_errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().clone()
    }

    /// Stop the accept loop.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

/// Serve `html` with no extra response headers.
pub fn serve_html(html: String) -> Fixture {
    serve(html, BTreeMap::new())
}

/// Serve `html`, adding `headers` to every response.
///
/// Answers every request the same way regardless of method or path — the
/// tests using this need one fixed response, not an HTTP server.
pub fn serve(html: String, headers: BTreeMap<String, String>) -> Fixture {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{port}");

    let shutdown = Arc::new(AtomicBool::new(false));
    let served = Arc::new(AtomicUsize::new(0));
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let shutdown_thread = shutdown.clone();
    let served_thread = served.clone();
    let errors_thread = errors.clone();

    thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("cannot set non-blocking");

        let mut header_lines = String::new();
        for (key, value) in &headers {
            header_lines.push_str(&format!("{key}: {value}\r\n"));
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n{}\r\n{}",
            html.len(),
            header_lines,
            html
        );

        let response = Arc::new(response);

        loop {
            if shutdown_thread.load(Ordering::Relaxed) {
                break;
            }
            match listener.accept() {
                Ok((stream, _)) => {
                    // One thread per connection. Reading a request to its end
                    // means waiting for a client, and a single accept loop
                    // that waits is a loop that starves every other
                    // connection — `wait_for_stable`'s concurrency test opens
                    // several at once and timed out at exactly the read
                    // timeout below when this ran inline.
                    let response = response.clone();
                    let served = served_thread.clone();
                    let errors = errors_thread.clone();
                    thread::spawn(move || {
                        serve_one(stream, &response, &served, &errors);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    errors_thread
                        .lock()
                        .unwrap()
                        .push(format!("accept failed: {e}"));
                    break;
                }
            }
        }
    });

    Fixture {
        url,
        shutdown,
        served,
        errors,
    }
}

/// Answer one connection.
fn serve_one(
    mut stream: std::net::TcpStream,
    response: &str,
    served: &Arc<AtomicUsize>,
    errors: &Arc<Mutex<Vec<String>>>,
) {
    let note = |msg: String| errors.lock().unwrap().push(msg);

    // The accepted socket inherits the listener's non-blocking flag on
    // macOS/BSD — see the module docs.
    if let Err(e) = stream.set_nonblocking(false) {
        note(format!("set_nonblocking(false) failed: {e}"));
        return;
    }
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    // Read the request head to its end, so the response is never written to a
    // client that is still mid-request.
    let mut request = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                request.extend_from_slice(&buf[..n]);
                if request.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            Err(e) => {
                note(format!("read failed: {e}"));
                break;
            }
        }
    }

    match stream
        .write_all(response.as_bytes())
        .and_then(|()| stream.flush())
    {
        Ok(()) => {
            served.fetch_add(1, Ordering::Relaxed);
        }
        Err(e) => note(format!("write failed: {e}")),
    }
}
