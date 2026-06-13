//! Best practices analysis — console errors (#121) and vulnerable libraries (#124).

mod console_errors;
pub mod module;
mod vulnerable_libs;

pub use console_errors::{
    prepare_console_collection, take_console_results, ConsoleErrorsAnalysis, ConsoleMessage,
};
pub use module::BestPracticesModule;
pub use vulnerable_libs::{
    analyze_vulnerable_libraries, DetectedLibrary, VulnerableLibrariesAnalysis, VulnerableLibrary,
};

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::Result;

/// Combined best practices analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPracticesAnalysis {
    /// Browser console errors and warnings captured during page load
    pub console_errors: ConsoleErrorsAnalysis,
    /// Detected JavaScript libraries with known security vulnerabilities
    pub vulnerable_libraries: VulnerableLibrariesAnalysis,
    /// Overall best practices score (0–100)
    pub score: u32,
}

/// Run best practices analysis after page load.
///
/// `console_errors` must have been prepared via `prepare_console_collection`
/// before navigation for results to be meaningful.
pub async fn analyze_best_practices(page: &Page) -> Result<BestPracticesAnalysis> {
    let console_errors = match take_console_results(page).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Console error collection failed: {}", e);
            ConsoleErrorsAnalysis {
                errors: vec![],
                warnings: vec![],
                error_count: 0,
                warning_count: 0,
            }
        }
    };

    let vulnerable_libraries = match analyze_vulnerable_libraries(page).await {
        Ok(v) => v,
        Err(e) => {
            warn!("Vulnerable library detection failed: {}", e);
            VulnerableLibrariesAnalysis {
                detected: vec![],
                vulnerable: vec![],
                has_vulnerabilities: false,
            }
        }
    };

    let score = calculate_score(&console_errors, &vulnerable_libraries);

    Ok(BestPracticesAnalysis {
        console_errors,
        vulnerable_libraries,
        score,
    })
}

fn calculate_score(console: &ConsoleErrorsAnalysis, libs: &VulnerableLibrariesAnalysis) -> u32 {
    let mut score = 100u32;

    // Console errors: penalise *distinct* error messages, not raw occurrences.
    // A single error logged on every render or poll cycle is one problem, not
    // many — counting occurrences over-penalises noisy-but-singular bugs. 5
    // points each, capped at 30.
    let distinct_errors = {
        let mut seen = std::collections::HashSet::new();
        console
            .errors
            .iter()
            .filter(|e| seen.insert(e.message.as_str()))
            .count()
    };
    let error_penalty = (distinct_errors as u32 * 5).min(30);
    score = score.saturating_sub(error_penalty);

    // Vulnerable libraries: high = 20, medium = 10, low = 5 per lib, capped at 40
    let vuln_penalty: u32 = libs
        .vulnerable
        .iter()
        .map(|v| match v.severity.as_str() {
            "high" => 20,
            "medium" => 10,
            _ => 5,
        })
        .sum::<u32>()
        .min(40);
    score = score.saturating_sub(vuln_penalty);

    score
}
