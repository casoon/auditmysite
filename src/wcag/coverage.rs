//! WCAG criterion coverage manifest (issue #37).
//!
//! Explicit list of WCAG 2.1 / 2.2 criteria that this tool checks
//! automatically vs. those that fundamentally require behavioral testing.
//! Surfaced in the report so users understand the scope of automated audit.

/// WCAG 2.1 AA totals — Level A: 30, Level AA: 20, sum 50.
/// WCAG 2.2 adds 6 more at AA (the 2.2.x criteria).
pub const WCAG_AA_TOTAL: usize = 50;

/// WCAG criteria with at least one automated rule in this tool.
/// Each entry: (criterion_id, level).
pub const AUTOMATED_CRITERIA: &[(&str, &str)] = &[
    // Level A
    ("1.1.1", "A"),
    ("1.3.1", "A"),
    ("1.4.1", "A"),
    ("2.1.1", "A"),
    ("2.2.1", "A"),
    ("2.4.1", "A"),
    ("2.4.2", "A"),
    ("2.4.3", "A"),
    ("2.4.4", "A"),
    ("3.1.1", "A"),
    ("3.2.1", "A"),
    ("3.2.2", "A"),
    ("3.3.1", "A"),
    ("3.3.2", "A"),
    ("4.1.1", "A"),
    ("4.1.2", "A"),
    // Level AA
    ("1.3.4", "AA"),
    ("1.3.5", "AA"),
    ("1.4.3", "AA"),
    ("1.4.4", "AA"),
    ("1.4.10", "AA"),
    ("1.4.11", "AA"),
    ("1.4.13", "AA"),
    ("2.4.6", "AA"),
    ("2.4.7", "AA"),
    ("2.5.3", "AA"),
    ("4.1.3", "AA"),
    // Level AAA partial
    ("1.4.6", "AAA"),
    ("2.3.3", "AAA"),
    ("2.4.10", "AAA"),
];

/// WCAG criteria that fundamentally require behavioral / manual review and
/// cannot be reliably verified by an automated tool.
pub const MANUAL_REVIEW_CRITERIA: &[(&str, &str, &str)] = &[
    ("1.2.1", "A", "Audio-only and Video-only (Prerecorded)"),
    ("1.2.2", "A", "Captions (Prerecorded)"),
    ("1.2.3", "A", "Audio Description or Media Alternative"),
    ("1.2.5", "AA", "Audio Description (Prerecorded)"),
    ("1.4.2", "A", "Audio Control"),
    ("1.4.5", "AA", "Images of Text"),
    ("2.1.4", "A", "Character Key Shortcuts"),
    ("2.2.2", "A", "Pause, Stop, Hide"),
    ("2.3.1", "A", "Three Flashes or Below Threshold"),
    ("2.5.1", "A", "Pointer Gestures"),
    ("2.5.2", "A", "Pointer Cancellation"),
    ("2.5.4", "A", "Motion Actuation"),
    ("3.2.3", "AA", "Consistent Navigation"),
    ("3.2.4", "AA", "Consistent Identification"),
    ("3.3.3", "AA", "Error Suggestion"),
    ("3.3.4", "AA", "Error Prevention (Legal, Financial, Data)"),
];

/// Returns (automated_count, total_aa_criteria).
pub fn coverage_stats() -> (usize, usize) {
    let aa_count = AUTOMATED_CRITERIA
        .iter()
        .filter(|(_, l)| *l == "A" || *l == "AA")
        .count();
    (aa_count, WCAG_AA_TOTAL)
}
