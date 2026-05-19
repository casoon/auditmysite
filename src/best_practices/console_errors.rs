//! Browser console error and warning collection (#121).
//!
//! **Protocol flow:**
//! 1. Before navigation: call `prepare_console_collection` →
//!    injects a script via `addScriptToEvaluateOnNewDocument` that wraps
//!    console.error/warn and captures window errors.
//! 2. After page load (in `extract_snapshot`): call `take_console_results` →
//!    reads the captured messages from `window.__auditConsoleErrors`.

use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

/// A single captured console message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Severity level: "error" or "warn"
    pub level: String,
    /// The message text (truncated to 200 chars)
    pub message: String,
}

/// Console error and warning analysis (#121).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleErrorsAnalysis {
    /// Captured console.error() calls and uncaught exceptions
    pub errors: Vec<ConsoleMessage>,
    /// Captured console.warn() calls
    pub warnings: Vec<ConsoleMessage>,
    /// Total error count
    pub error_count: usize,
    /// Total warning count
    pub warning_count: usize,
}

/// Inject a console interceptor script **before** the page navigates.
///
/// Must be called before `browser.navigate()`. Errors are non-fatal.
pub async fn prepare_console_collection(page: &Page) -> Result<()> {
    let script = r#"
    (function() {
        window.__auditConsoleErrors = [];
        var _origError = console.error;
        var _origWarn = console.warn;
        console.error = function() {
            var msg = Array.prototype.slice.call(arguments).map(function(a) {
                return typeof a === 'object' ? JSON.stringify(a) : String(a);
            }).join(' ');
            window.__auditConsoleErrors.push({level: 'error', msg: msg.substring(0, 200)});
            _origError.apply(console, arguments);
        };
        console.warn = function() {
            var msg = Array.prototype.slice.call(arguments).map(function(a) {
                return typeof a === 'object' ? JSON.stringify(a) : String(a);
            }).join(' ');
            window.__auditConsoleErrors.push({level: 'warn', msg: msg.substring(0, 200)});
            _origWarn.apply(console, arguments);
        };
        window.addEventListener('error', function(e) {
            var loc = e.filename ? (' at ' + e.filename + ':' + e.lineno) : '';
            window.__auditConsoleErrors.push({
                level: 'error',
                msg: (e.message + loc).substring(0, 200)
            });
        });
        window.addEventListener('unhandledrejection', function(e) {
            var reason = e.reason ? String(e.reason) : 'unknown';
            window.__auditConsoleErrors.push({
                level: 'error',
                msg: ('Unhandled Promise Rejection: ' + reason).substring(0, 200)
            });
        });
    })();
    "#;

    page.execute(AddScriptToEvaluateOnNewDocumentParams::new(script))
        .await
        .map_err(|e| AuditError::CdpError(format!("Console collection setup failed: {e}")))?;

    Ok(())
}

/// Read console errors collected after the page has loaded.
pub async fn take_console_results(page: &Page) -> Result<ConsoleErrorsAnalysis> {
    info!("Taking console error results...");

    let js = r#"
    (() => {
        var items = window.__auditConsoleErrors || [];
        return JSON.stringify(items);
    })()
    "#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Console results JS failed: {e}")))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("[]");

    #[derive(serde::Deserialize)]
    struct RawMsg {
        level: String,
        msg: String,
    }

    let raw: Vec<RawMsg> = serde_json::from_str(json_str).unwrap_or_default();

    let mut errors: Vec<ConsoleMessage> = Vec::new();
    let mut warnings: Vec<ConsoleMessage> = Vec::new();

    for item in raw {
        let msg = ConsoleMessage {
            level: item.level.clone(),
            message: item.msg,
        };
        if item.level == "error" {
            errors.push(msg);
        } else {
            warnings.push(msg);
        }
    }

    let error_count = errors.len();
    let warning_count = warnings.len();

    info!(
        "Console: {} errors, {} warnings",
        error_count, warning_count
    );

    Ok(ConsoleErrorsAnalysis {
        errors,
        warnings,
        error_count,
        warning_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_errors_analysis_empty() {
        let analysis = ConsoleErrorsAnalysis {
            errors: vec![],
            warnings: vec![],
            error_count: 0,
            warning_count: 0,
        };
        assert_eq!(analysis.error_count, 0);
        assert_eq!(analysis.warning_count, 0);
    }
}
