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

        let cached = dns_cache().lock().await.get(&host).cloned();
        let result = match cached {
            Some(result) => *result,
            None => {
                let result = query_dns(&host).await;
                let complete = [result.caa, result.dnssec, result.mx, result.spf]
                    .iter()
                    .all(|answer| !matches!(answer, DnsAnswer::Unknown(_)));
                if complete {
                    dns_cache()
                        .lock()
                        .await
                        .insert(host.clone(), Arc::new(result));
                }
                result
            }
        };

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
