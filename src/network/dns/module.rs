//! `AuditModule` implementation for the DNS-configuration module (#545).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use crate::audit::collection::{CollectionMethod, CollectorSpec};

use super::{build_analysis, query_dns, DnsAnswer, DnsCheckResult};

const fn dns_collector(id: &'static str) -> CollectorSpec {
    CollectorSpec {
        id,
        module: "network_dns",
        method: CollectionMethod::Dns,
        method_version: 2,
        scoring: false,
    }
}

/// CAA for the host or its closest parent.
pub(crate) static CAA: CollectorSpec = dns_collector("network_dns.caa");
/// DNSKEY for the host or a parent (no chain validation).
pub(crate) static DNSKEY: CollectorSpec = dns_collector("network_dns.dnskey");
pub(crate) static MX: CollectorSpec = dns_collector("network_dns.mx");
/// SPF TXT record, when mail is routed to the host.
pub(crate) static SPF: CollectorSpec = dns_collector("network_dns.spf");

/// Per-host memoization (#545: "runs once per host, not once per page").
/// A DNS lookup set is host-scoped, not page-scoped — repeating it for every
/// page of the same batch run would just repeat identical network queries.
/// Nothing in the catalog/`ModuleContext` machinery threads shared state
/// between `AuditModule::collect` calls, so this is process-wide (not tied
/// to a single `PipelineConfig`) — the simplest mechanism that achieves
/// "one DNS check per unique host per process run", including the rare case
/// of a multi-host batch (e.g. a URL file spanning several domains), which
/// still gets one lookup per distinct host rather than one for the whole run.
///
/// Only complete answers are cached: a timeout on one page must not stand in
/// for the whole run (#776).
static DNS_CACHE: OnceLock<Mutex<HashMap<String, Arc<DnsCheckResult>>>> = OnceLock::new();

fn dns_cache() -> &'static Mutex<HashMap<String, Arc<DnsCheckResult>>> {
    DNS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct NetworkDnsModule;

#[async_trait]
impl AuditModule for NetworkDnsModule {
    fn id(&self) -> &'static str {
        "network_dns"
    }

    fn label(&self) -> &'static str {
        "DNS Configuration"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_dns
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        let Ok(parsed) = url::Url::parse(ctx.url) else {
            return Ok(ModuleData::None);
        };
        let Some(host) = parsed.host_str() else {
            return Ok(ModuleData::None);
        };
        let host = host.to_string();

        let result = cached_result(dns_cache(), &host, || query_dns(&host)).await;

        // Every check records its answer; unknown ones as failed (#776).
        let log = ctx.collection;
        log.record(&CAA, result.caa.collected());
        log.record(&DNSKEY, result.dnssec.collected());
        log.record(&MX, result.mx.collected());
        log.record(&SPF, result.spf.collected());

        Ok(ModuleData::NetworkDns(Box::new(build_analysis(
            &host, &result,
        ))))
    }
}

/// The cached answer for `host`, or the result of `query`. Only a complete
/// result is stored: a timeout or SERVFAIL must not stand in for the whole
/// run (#776), so a later success is seen.
async fn cached_result<F, Fut>(
    cache: &Mutex<HashMap<String, Arc<DnsCheckResult>>>,
    host: &str,
    query: F,
) -> DnsCheckResult
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = DnsCheckResult>,
{
    if let Some(result) = cache.lock().await.get(host).cloned() {
        return *result;
    }
    let result = query().await;
    let complete = [result.caa, result.dnssec, result.mx, result.spf]
        .iter()
        .all(|answer| !matches!(answer, DnsAnswer::Unknown(_)));
    if complete {
        cache
            .lock()
            .await
            .insert(host.to_string(), Arc::new(result));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use DnsAnswer::{Absent, Present, Unknown};

    fn answers(caa: DnsAnswer, mx: DnsAnswer) -> DnsCheckResult {
        DnsCheckResult {
            caa,
            dnssec: Present,
            mx,
            spf: Present,
        }
    }

    /// #776: an incomplete answer (SERVFAIL, timeout) is not cached; the
    /// next page of the run asks again and sees the success. A complete one
    /// is asked for once.
    #[tokio::test]
    async fn incomplete_answers_are_not_cached_and_a_later_success_shows() {
        let cache = Mutex::new(HashMap::new());
        let calls = std::sync::atomic::AtomicUsize::new(0);
        let ask = |result: DnsCheckResult| {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async move { result }
        };

        let first = cached_result(&cache, "example.com", || {
            ask(answers(Unknown("dns_timeout"), Present))
        })
        .await;
        assert_eq!(first.caa, Unknown("dns_timeout"));
        assert!(cache.lock().await.is_empty(), "unknown must not be cached");

        let second = cached_result(&cache, "example.com", || ask(answers(Absent, Present))).await;
        assert_eq!(second.caa, Absent, "the later success is visible");
        assert_eq!(cache.lock().await.len(), 1);

        let third = cached_result(&cache, "example.com", || {
            ask(answers(
                Unknown("dns_lookup_failed"),
                Unknown("dns_timeout"),
            ))
        })
        .await;
        assert_eq!(third.caa, Absent, "a complete answer is reused");
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
