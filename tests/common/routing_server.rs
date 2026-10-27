//! Path-routing fixture server for corpus cases that need several resources
//! (#783). Owned by the non-WCAG corpus work package.
//!
//! [`super::fixture_server`] answers every request with one fixed document.
//! Some detectors fetch more than that: the source-map check fetches the
//! page, then every script and stylesheet, then each referenced `.map`
//! file; the header probe sends `HEAD` before `GET`. This server answers by
//! method and path, lets a case repeat or re-case a response header (a
//! `BTreeMap` cannot), and can speak TLS.
//!
//! ## TLS
//!
//! `analyze_security` treats a document as served over HTTPS only when the
//! final URL is `https:` and reqwest validated the certificate chain against
//! its roots (rustls with the bundled webpki roots — environment variables
//! such as `SSL_CERT_FILE` do not reach it). The fixture therefore presents a
//! leaf certificate issued by a corpus-only CA; the test hands that CA to
//! `auditmysite::security::analyze_security_trusting`, which adds it to the
//! built-in roots without relaxing any check. The untrusted variant is a
//! self-signed leaf for the same key that no client trusts: the case that
//! proves certificate validation still rejects.
//!
//! The material is generated once per test process (P-256, leaf SAN
//! `127.0.0.1` and `localhost`, EKU serverAuth), so no private key is ever
//! committed.
//!
//! Like `fixture_server`, the accepted socket is switched back to blocking,
//! the request head is read to its end before the response is written, and
//! every transport failure is recorded instead of swallowed.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// One response the server gives.
#[derive(Debug, Clone)]
pub struct Route {
    /// `None` answers every method.
    pub method: Option<String>,
    /// Exact path, without query string.
    pub path: String,
    pub status: u16,
    /// Sent in this order, names as given (repeats and casing preserved).
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Route {
    pub fn new(path: &str, status: u16, body: impl Into<Vec<u8>>) -> Self {
        Route {
            method: None,
            path: path.to_string(),
            status,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = Some(method.to_string());
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }
}

/// Which certificate the server presents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Http,
    /// Leaf issued by the corpus CA ([`corpus_ca_pem`]).
    Https,
    /// Self-signed leaf no client trusts.
    HttpsUntrusted,
}

pub struct RoutingFixture {
    /// Base URL, e.g. `https://127.0.0.1:53124`.
    pub url: String,
    shutdown: Arc<AtomicBool>,
    served: Arc<AtomicUsize>,
    errors: Arc<Mutex<Vec<String>>>,
    requests: Arc<Mutex<Vec<String>>>,
}

impl RoutingFixture {
    /// Responses written in full.
    pub fn served(&self) -> usize {
        self.served.load(Ordering::Relaxed)
    }

    /// Transport failures, in order. A failed TLS handshake counts here too.
    pub fn transport_errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().clone()
    }

    /// `"<METHOD> <path>"` of every request that reached the router.
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }

    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

impl Drop for RoutingFixture {
    fn drop(&mut self) {
        self.stop();
    }
}

struct TlsMaterial {
    ca_pem: Vec<u8>,
    leaf: rustls::pki_types::CertificateDer<'static>,
    untrusted: rustls::pki_types::CertificateDer<'static>,
    key_der: Vec<u8>,
}

fn tls() -> &'static TlsMaterial {
    static MATERIAL: std::sync::OnceLock<TlsMaterial> = std::sync::OnceLock::new();
    MATERIAL.get_or_init(|| {
        use rcgen::{
            BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair,
            SanType,
        };
        let ca_key = KeyPair::generate().expect("CA key");
        let mut ca_params = CertificateParams::new(Vec::<String>::new()).expect("CA params");
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params
            .distinguished_name
            .push(DnType::CommonName, "auditmysite corpus CA");
        let ca = ca_params.self_signed(&ca_key).expect("CA certificate");

        let leaf_key = KeyPair::generate().expect("leaf key");
        let mut leaf_params =
            CertificateParams::new(vec!["localhost".to_string()]).expect("leaf params");
        leaf_params
            .subject_alt_names
            .push(SanType::IpAddress(std::net::Ipv4Addr::LOCALHOST.into()));
        leaf_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        let leaf = leaf_params
            .clone()
            .signed_by(&leaf_key, &ca, &ca_key)
            .expect("leaf certificate");
        let untrusted = leaf_params
            .self_signed(&leaf_key)
            .expect("untrusted certificate");

        TlsMaterial {
            ca_pem: ca.pem().into_bytes(),
            leaf: leaf.der().clone(),
            untrusted: untrusted.der().clone(),
            key_der: leaf_key.serialize_der(),
        }
    })
}

/// PEM of the corpus-only CA that issued the HTTPS fixture's certificate.
pub fn corpus_ca_pem() -> Vec<u8> {
    tls().ca_pem.clone()
}

/// A base URL nothing listens on: bound once, then released.
pub fn unreachable_origin() -> String {
    let socket = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let url = format!("http://{}", socket.local_addr().unwrap());
    drop(socket);
    url
}

fn server_config(transport: Transport) -> Option<Arc<rustls::ServerConfig>> {
    use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
    let cert = match transport {
        Transport::Http => return None,
        Transport::Https => tls().leaf.clone(),
        Transport::HttpsUntrusted => tls().untrusted.clone(),
    };
    let certs = vec![cert];
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(tls().key_der.clone()));
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("TLS versions")
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .expect("fixture certificate and key match");
    Some(Arc::new(config))
}

/// Start a server answering `routes` (first match wins; anything else is
/// 404) over `transport`.
pub fn serve_routes(routes: Vec<Route>, transport: Transport) -> RoutingFixture {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let port = listener.local_addr().unwrap().port();
    let scheme = if transport == Transport::Http {
        "http"
    } else {
        "https"
    };
    let fixture = RoutingFixture {
        url: format!("{scheme}://127.0.0.1:{port}"),
        shutdown: Arc::new(AtomicBool::new(false)),
        served: Arc::new(AtomicUsize::new(0)),
        errors: Arc::new(Mutex::new(Vec::new())),
        requests: Arc::new(Mutex::new(Vec::new())),
    };
    let tls = server_config(transport);
    let routes = Arc::new(routes);
    let shutdown = fixture.shutdown.clone();
    let served = fixture.served.clone();
    let errors = fixture.errors.clone();
    let requests = fixture.requests.clone();

    thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("cannot set non-blocking");
        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            match listener.accept() {
                Ok((stream, _)) => {
                    let ctx = Conn {
                        routes: routes.clone(),
                        served: served.clone(),
                        errors: errors.clone(),
                        requests: requests.clone(),
                    };
                    let tls = tls.clone();
                    thread::spawn(move || ctx.handle(stream, tls));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    errors.lock().unwrap().push(format!("accept failed: {e}"));
                    break;
                }
            }
        }
    });
    fixture
}

struct Conn {
    routes: Arc<Vec<Route>>,
    served: Arc<AtomicUsize>,
    errors: Arc<Mutex<Vec<String>>>,
    requests: Arc<Mutex<Vec<String>>>,
}

impl Conn {
    fn note(&self, msg: String) {
        self.errors.lock().unwrap().push(msg);
    }

    fn handle(self, stream: TcpStream, tls: Option<Arc<rustls::ServerConfig>>) {
        // The accepted socket inherits the listener's non-blocking flag on
        // macOS/BSD (see fixture_server).
        if let Err(e) = stream.set_nonblocking(false) {
            self.note(format!("set_nonblocking(false) failed: {e}"));
            return;
        }
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
        match tls {
            None => self.exchange(stream),
            Some(config) => {
                let conn = match rustls::ServerConnection::new(config) {
                    Ok(conn) => conn,
                    Err(e) => return self.note(format!("TLS setup failed: {e}")),
                };
                let mut tls = rustls::StreamOwned::new(conn, stream);
                self.exchange(&mut tls);
                tls.conn.send_close_notify();
                let _ = tls.flush();
            }
        }
    }

    fn exchange(&self, mut stream: impl Read + Write) {
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
                // A connection the client opened but never used (a pooled or
                // speculative connect) times out before sending a byte; that
                // is no failed exchange. A timeout mid-request still is.
                Err(e)
                    if request.is_empty()
                        && matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) =>
                {
                    return;
                }
                Err(e) => {
                    self.note(format!("read failed: {e}"));
                    return;
                }
            }
        }
        if request.is_empty() {
            // A client that connected and left (connection-pool probe).
            return;
        }
        let head = String::from_utf8_lossy(&request);
        let mut parts = head.lines().next().unwrap_or("").split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let target = parts.next().unwrap_or("/");
        let path = target.split('?').next().unwrap_or("/").to_string();
        self.requests
            .lock()
            .unwrap()
            .push(format!("{method} {path}"));

        let route = self.routes.iter().find(|r| {
            r.path == path
                && r.method
                    .as_deref()
                    .is_none_or(|m| m.eq_ignore_ascii_case(&method))
        });
        let fallback = Route::new(&path, 404, "not found").header("Content-Type", "text/plain");
        let route = route.unwrap_or(&fallback);

        let mut out = format!("HTTP/1.1 {} {}\r\n", route.status, reason(route.status));
        if !route
            .headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        {
            out.push_str("Content-Type: text/html; charset=utf-8\r\n");
        }
        for (name, value) in &route.headers {
            out.push_str(&format!("{name}: {value}\r\n"));
        }
        out.push_str(&format!(
            "Content-Length: {}\r\nConnection: close\r\n\r\n",
            route.body.len()
        ));
        let mut bytes = out.into_bytes();
        if !method.eq_ignore_ascii_case("HEAD") {
            bytes.extend_from_slice(&route.body);
        }
        match stream.write_all(&bytes).and_then(|()| stream.flush()) {
            Ok(()) => {
                self.served.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => self.note(format!("write failed: {e}")),
        }
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Status",
    }
}
