//! Critical request chain / network dependency tree (#132).
//!
//! Builds an approximate dependency tree from PerformanceResourceTiming entries.
//! Resources are grouped into chains by matching each resource's start time
//! against the response window of in-flight requests, giving a best-effort
//! reconstruction of which resources triggered which subsequent fetches.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

/// A single node in the critical request chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainNode {
    /// Full resource URL (truncated to 120 chars)
    pub url: String,
    /// Initiator type: "parser", "script", "css", "other"
    pub initiator_type: String,
    /// Request start time relative to navigation start (ms)
    pub start_time_ms: f64,
    /// Time when the last byte was received (ms)
    pub end_time_ms: f64,
    /// Response time (end − start) in ms
    pub duration_ms: f64,
    /// Compressed transfer size in bytes
    pub transfer_bytes: u64,
    /// Resources whose fetch started during this resource's response window
    pub children: Vec<ChainNode>,
}

impl ChainNode {
    /// Maximum chain depth from this node downward.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Total transfer bytes along the deepest path through this subtree.
    pub fn critical_bytes(&self) -> u64 {
        if self.children.is_empty() {
            self.transfer_bytes
        } else {
            self.transfer_bytes
                + self
                    .children
                    .iter()
                    .map(|c| c.critical_bytes())
                    .max()
                    .unwrap_or(0)
        }
    }

    /// Total duration along the longest sequential path (critical path ms).
    pub fn critical_duration_ms(&self) -> f64 {
        if self.children.is_empty() {
            self.duration_ms
        } else {
            self.duration_ms
                + self
                    .children
                    .iter()
                    .map(|c| c.critical_duration_ms())
                    .fold(0.0_f64, f64::max)
        }
    }
}

/// Critical request chain analysis for the audited page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalChain {
    /// Root-level chains (requests that started closest to navigation start)
    pub chains: Vec<ChainNode>,
    /// Total number of resources in the dependency tree
    pub total_requests: u32,
    /// Estimated total transfer bytes on the longest critical path
    pub critical_path_bytes: u64,
    /// Estimated total latency on the longest critical path (ms)
    pub critical_path_ms: f64,
    /// Maximum chain depth across all root chains
    pub max_depth: usize,
}

/// Build the critical request chain from PerformanceResourceTiming entries.
pub async fn analyze_critical_chain(page: &Page) -> Result<CriticalChain> {
    info!("Building critical request chain...");

    let js = r#"
    (() => {
        var nav = performance.getEntriesByType('navigation')[0];
        var navStart = nav ? nav.startTime : 0;
        var resources = performance.getEntriesByType('resource');
        return JSON.stringify(resources.map(function(r) {
            return {
                url: r.name,
                initiatorType: r.initiatorType || 'other',
                startTime: r.startTime - navStart,
                responseEnd: r.responseEnd - navStart,
                transferSize: r.transferSize || 0,
                duration: r.duration
            };
        }));
    })()
    "#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Critical chain JS failed: {e}")))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("[]");
    let raw: Vec<RawResource> = serde_json::from_str(json_str).unwrap_or_default();

    let total_requests = raw.len() as u32;

    // Build nodes from raw entries
    let mut nodes: Vec<NodeInfo> = raw
        .into_iter()
        .map(|r| NodeInfo {
            url: truncate(&r.url, 120),
            initiator_type: r.initiator_type,
            start_ms: r.start_time.max(0.0),
            end_ms: r.response_end.max(0.0),
            transfer_bytes: r.transfer_size,
            duration_ms: r.duration.max(0.0),
        })
        .collect();

    // Sort by start time so we can process in order
    nodes.sort_by(|a, b| a.start_ms.partial_cmp(&b.start_ms).unwrap());

    let chains = build_chains(&nodes);

    let critical_path_bytes = chains.iter().map(|c| c.critical_bytes()).max().unwrap_or(0);
    let critical_path_ms = chains
        .iter()
        .map(|c| c.critical_duration_ms())
        .fold(0.0_f64, f64::max);
    let max_depth = chains.iter().map(|c| c.depth()).max().unwrap_or(0);

    info!(
        "Critical chain: {} requests, depth {}, {:.0}ms critical path",
        total_requests, max_depth, critical_path_ms
    );

    Ok(CriticalChain {
        chains,
        total_requests,
        critical_path_bytes,
        critical_path_ms,
        max_depth,
    })
}

// ── Tree builder ──────────────────────────────────────────────────────────────

struct NodeInfo {
    url: String,
    initiator_type: String,
    start_ms: f64,
    end_ms: f64,
    transfer_bytes: u64,
    duration_ms: f64,
}

/// Recursively build a dependency tree.
///
/// For each node, "children" are resources whose start time falls within
/// [node.start_ms, node.end_ms] — i.e. they were requested while this
/// resource was still being fetched (the typical trigger pattern for
/// parser-discovered and script-injected resources).
///
/// We cap tree depth at 8 to avoid O(n²) blow-up on pages with many parallel
/// requests, and we only recurse into nodes that haven't been assigned to a
/// parent yet (greedy single-parent assignment).
fn build_chains(nodes: &[NodeInfo]) -> Vec<ChainNode> {
    let n = nodes.len();
    if n == 0 {
        return vec![];
    }

    // Track which indices have been claimed as children
    let mut claimed = vec![false; n];

    // Root nodes: the first request plus any request that starts within the
    // first 50 ms (parser-phase resources that kick off sub-chains).
    let earliest_start = nodes[0].start_ms;
    let root_window_end = (earliest_start + 50.0).max(nodes[0].end_ms);

    let roots: Vec<usize> = (0..n)
        .filter(|&i| nodes[i].start_ms <= root_window_end)
        .collect();

    for &r in &roots {
        claimed[r] = true;
    }

    fn build_node(
        idx: usize,
        nodes: &[NodeInfo],
        claimed: &mut Vec<bool>,
        depth: usize,
    ) -> ChainNode {
        let node = &nodes[idx];
        let children = if depth < 8 {
            let child_indices: Vec<usize> = (0..nodes.len())
                .filter(|&i| {
                    !claimed[i]
                        && nodes[i].start_ms >= node.start_ms
                        && nodes[i].start_ms <= node.end_ms
                })
                .collect();
            for &ci in &child_indices {
                claimed[ci] = true;
            }
            child_indices
                .into_iter()
                .map(|ci| build_node(ci, nodes, claimed, depth + 1))
                .collect()
        } else {
            vec![]
        };

        ChainNode {
            url: node.url.clone(),
            initiator_type: node.initiator_type.clone(),
            start_time_ms: node.start_ms,
            end_time_ms: node.end_ms,
            duration_ms: node.duration_ms,
            transfer_bytes: node.transfer_bytes,
            children,
        }
    }

    roots
        .into_iter()
        .map(|ri| build_node(ri, nodes, &mut claimed, 1))
        .collect()
}

// ── Serde helpers ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawResource {
    url: String,
    #[serde(rename = "initiatorType")]
    initiator_type: String,
    #[serde(rename = "startTime")]
    start_time: f64,
    #[serde(rename = "responseEnd")]
    response_end: f64,
    #[serde(rename = "transferSize")]
    transfer_size: u64,
    duration: f64,
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let boundary = s
        .char_indices()
        .take_while(|(i, _)| *i <= max.saturating_sub(3))
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}…", &s[..boundary])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(url: &str, start: f64, end: f64, bytes: u64) -> ChainNode {
        ChainNode {
            url: url.to_string(),
            initiator_type: "parser".to_string(),
            start_time_ms: start,
            end_time_ms: end,
            duration_ms: end - start,
            transfer_bytes: bytes,
            children: vec![],
        }
    }

    #[test]
    fn test_chain_node_depth_leaf() {
        let node = make_node("a.css", 0.0, 100.0, 5000);
        assert_eq!(node.depth(), 1);
    }

    #[test]
    fn test_chain_node_depth_nested() {
        let child = make_node("font.woff", 50.0, 150.0, 2000);
        let mut parent = make_node("a.css", 0.0, 100.0, 5000);
        parent.children = vec![child];
        assert_eq!(parent.depth(), 2);
    }

    #[test]
    fn test_chain_node_critical_bytes() {
        let child1 = make_node("b.js", 50.0, 200.0, 3000);
        let child2 = make_node("c.js", 50.0, 200.0, 8000);
        let mut parent = make_node("a.html", 0.0, 80.0, 1000);
        parent.children = vec![child1, child2];
        // critical path goes through child2 (8000 > 3000)
        assert_eq!(parent.critical_bytes(), 1000 + 8000);
    }

    #[test]
    fn test_chain_node_critical_duration() {
        let child = make_node("app.js", 50.0, 250.0, 10_000);
        let mut parent = make_node("index.html", 0.0, 80.0, 500);
        parent.duration_ms = 80.0;
        parent.children = vec![child];
        // 80 + 200 = 280 ms
        assert!((parent.critical_duration_ms() - 280.0).abs() < 0.01);
    }
}
