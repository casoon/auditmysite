//! Design-quality analysis (#528): opt-in, evidence-based UX/readability
//! heuristics from the rendered page.
//!
//! This module is deliberately **not** a WCAG rule and stays outside the
//! accessibility/overall score, grade, and certificate. Every finding uses
//! canonical English and is tagged `measurement_type: "heuristic"` — these
//! are actionable suggestions, not conformance failures. Findings use a
//! dedicated type rather than `wcag::types::Violation` so they can never be
//! folded into WCAG violation counting or scoring by accident.

pub mod module;
pub use module::DesignQualityModule;

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

use crate::error::{AuditError, Result};

/// Severity-like classification for a design-quality finding.
///
/// `Warning` is a functional risk (e.g. interactive content that can be
/// clipped out of view). `Advisory` is a stylistic/readability suggestion —
/// never presented as a defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingLevel {
    Warning,
    Advisory,
}

/// Confidence that the measured condition is a real issue on this page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// A single design-quality finding.
///
/// Deliberately independent of `wcag::types::Violation` — see module doc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignQualityFinding {
    /// Stable identifier, e.g. `"design.overflow_clip"`.
    pub rule_id: String,
    pub level: FindingLevel,
    pub confidence: Confidence,
    /// Always `"heuristic"` — never a measured, conformance-grade value.
    pub measurement_type: String,
    pub selector: String,
    /// Bounded evidence excerpt (measured value + short snippet).
    pub evidence: String,
    /// Canonical English message.
    pub message: String,
}

/// Results of the design-quality analysis for one page.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignQualityAnalysis {
    pub findings: Vec<DesignQualityFinding>,
    /// Rule ids that were evaluated (present even when they produced no
    /// findings), so the report can distinguish "checked, clean" from
    /// "not checked".
    pub rules_run: Vec<String>,
}

impl DesignQualityAnalysis {
    pub fn warnings(&self) -> impl Iterator<Item = &DesignQualityFinding> {
        self.findings
            .iter()
            .filter(|f| f.level == FindingLevel::Warning)
    }

    pub fn advisories(&self) -> impl Iterator<Item = &DesignQualityFinding> {
        self.findings
            .iter()
            .filter(|f| f.level == FindingLevel::Advisory)
    }
}

// ── Tunable thresholds (not specified by the issue — defaults chosen to be
// conservative; treat as constants to tune without touching rule logic) ────
const LINE_LENGTH_CHARS_THRESHOLD: f64 = 90.0;
// Kept below `normal`'s typical browser-computed ratio (~1.2) so ordinary,
// unstyled text does not trigger this heuristic — only line-heights authors
// have deliberately tightened below that baseline.
const LINE_HEIGHT_RATIO_ADVISORY: f64 = 1.15;
const LINE_HEIGHT_RATIO_WARNING: f64 = 1.0;
const ALL_CAPS_MIN_CHARS: usize = 60;
// Practical screen-reader alt-text length guidance (BIK "Barrierefreie Redaktion" guide).
const ALT_TEXT_MAX_CHARS: usize = 125;
// Below this, a normalized alt/adjacent-text equality check is not meaningful.
const ALT_TEXT_MIN_COMPARE_CHARS: usize = 4;
// A run of this many or more consecutive emoji is flagged.
const EMOJI_RUN_THRESHOLD: usize = 2;
// Only long hashtags benefit from word-boundary capitalization; short ones
// are unambiguous either way.
const HASHTAG_MIN_CHARS_FOR_CAMEL_CASE: usize = 12;

/// Known image-file extensions used by the filename-as-alt-text heuristic.
const FILENAME_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "tif", "tiff", "heic", "avif",
];
/// Generic camera/CMS filename prefixes (e.g. `IMG_1234`, `DSC00042`) used by
/// the filename-as-alt-text heuristic.
const GENERIC_FILENAME_PREFIXES: &[&str] = &[
    "img",
    "image",
    "dsc",
    "dscn",
    "photo",
    "pic",
    "picture",
    "screenshot",
    "bild",
    "foto",
    "shot",
];
/// Locales whose `alt-text-boilerplate-prefixes` FTL list is always merged,
/// regardless of report language — mirrors `a11y_journey::link_inventory`'s
/// `SUPPORTED_LOCALES` handling of mixed-language sites.
const SUPPORTED_ALT_BOILERPLATE_LOCALES: &[&str] = &["de", "en"];

/// Layout-affecting properties considered a "layout transition" for #528's
/// rule 5 — a narrower subset than `performance::animations`'s full
/// non-composited-property list (which also flags paint-only properties
/// like `color`/`box-shadow` that don't move content).
const LAYOUT_TRANSITION_PROPERTIES: &[&str] = &[
    "width",
    "height",
    "min-width",
    "max-width",
    "min-height",
    "max-height",
    "margin",
    "margin-top",
    "margin-bottom",
    "margin-left",
    "margin-right",
    "padding",
    "padding-top",
    "padding-bottom",
    "padding-left",
    "padding-right",
    "top",
    "left",
    "right",
    "bottom",
];

#[derive(Debug, Deserialize)]
struct RawTextCandidate {
    #[serde(rename = "cssPath")]
    css_path: Option<String>,
    snippet: Option<String>,
    text: String,
    #[serde(rename = "textLength")]
    text_length: usize,
    #[serde(rename = "lineHeightRatio")]
    line_height_ratio: f64,
    #[serde(rename = "textTransform")]
    text_transform: String,
    #[serde(rename = "lineCount")]
    line_count: u32,
}

#[derive(Debug, Deserialize)]
struct RawClipCandidate {
    #[serde(rename = "cssPath")]
    css_path: Option<String>,
    snippet: Option<String>,
    #[serde(rename = "clippingAncestor")]
    clipping_ancestor: String,
}

#[derive(Debug, Deserialize)]
struct RawAltCandidate {
    #[serde(rename = "cssPath")]
    css_path: Option<String>,
    snippet: Option<String>,
    alt: String,
    #[serde(rename = "adjacentText", default)]
    adjacent_text: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawCandidates {
    #[serde(rename = "textCandidates")]
    text_candidates: Vec<RawTextCandidate>,
    #[serde(rename = "clipCandidates")]
    clip_candidates: Vec<RawClipCandidate>,
    #[serde(rename = "altCandidates")]
    alt_candidates: Vec<RawAltCandidate>,
}

const EXTRACT_JS: &str = include_str!("extract.js");

/// Run the design-quality analysis on the already-loaded page.
///
/// Reuses the live CDP session (`page`) — no new navigation, browser, or
/// Node runtime. A single `page.evaluate()` pass collects both the
/// text-based candidates (line length, line height, all-caps) and the
/// positioned/interactive candidates (overflow-clip) in one round trip.
pub async fn analyze_design_quality(page: &Page) -> Result<DesignQualityAnalysis> {
    let js_code = [
        "(() => {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        crate::accessibility::js_helpers::IS_VISUALLY_HIDDEN_JS,
        crate::accessibility::js_helpers::IS_ARIA_HIDDEN_JS,
        EXTRACT_JS,
        "})();",
    ]
    .concat();

    let eval_result = page
        .evaluate(js_code.as_str())
        .await
        .map_err(|e| AuditError::CdpError(format!("Design quality extraction JS failed: {e}")))?;

    let raw: RawCandidates =
        serde_json::from_value(eval_result.value().cloned().ok_or_else(|| {
            AuditError::CdpError("Design quality collector returned no result".into())
        })?)
        .map_err(|_| {
            AuditError::CdpError("Design quality collector returned an invalid result".into())
        })?;

    let mut findings = Vec::new();
    findings.extend(check_overflow_clip(&raw.clip_candidates));
    findings.extend(check_line_length(&raw.text_candidates));
    findings.extend(check_line_height(&raw.text_candidates));
    findings.extend(check_all_caps(&raw.text_candidates));
    findings.extend(check_alt_filename(&raw.alt_candidates));
    findings.extend(check_alt_boilerplate(
        &raw.alt_candidates,
        &load_boilerplate_prefixes(),
    ));
    findings.extend(check_alt_length(&raw.alt_candidates));
    findings.extend(check_alt_redundant(&raw.alt_candidates));
    findings.extend(check_emoji_usage(&raw.text_candidates));
    findings.extend(check_hashtag_camel_case(&raw.text_candidates));

    Ok(DesignQualityAnalysis {
        findings,
        rules_run: vec![
            "design.overflow_clip".to_string(),
            "design.line_length".to_string(),
            "design.line_height".to_string(),
            "design.all_caps".to_string(),
            "design.alt_filename".to_string(),
            "design.alt_boilerplate".to_string(),
            "design.alt_too_long".to_string(),
            "design.alt_redundant".to_string(),
            "design.emoji_usage".to_string(),
            "design.hashtag_camel_case".to_string(),
            // "design.layout_transition" is evaluated in `derive()` against
            // `report.performance.animations`, not here — see module.rs.
        ],
    })
}

fn selector_or_unknown(sel: &Option<String>) -> String {
    sel.clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Bounded evidence suffix appending the (already client-side-truncated)
/// HTML snippet, when present.
fn with_snippet(evidence: String, snippet: &Option<String>) -> String {
    match snippet.as_deref().filter(|s| !s.is_empty()) {
        Some(s) => format!("{evidence} Element: {s}"),
        None => evidence,
    }
}

/// Localized presentation text for a design-quality finding, keyed by its
/// canonical `rule_id` (#406 message-baked-struct pattern: the analysis layer
/// bakes `DesignQualityFinding.message` by calling this with `en=true` — JSON
/// stays canonical English — and the PDF builder calls it again with the run
/// locale). Per-instance specifics (selector, ancestor, measured values) live
/// in `evidence`, not in this generic prose.
pub fn finding_message_text(rule_id: &str, level: FindingLevel, en: bool) -> String {
    match rule_id {
        "design.overflow_clip" => if en {
            "This positioned/interactive element can be clipped by an overflow-hidden ancestor, making part of it inaccessible."
        } else {
            "Dieses positionierte/interaktive Element kann von einem Vorfahren mit overflow:hidden abgeschnitten werden, wodurch ein Teil davon unzugänglich wird."
        }
        .to_string(),
        "design.line_length" => if en {
            "This body text wraps at a line length that may reduce readability."
        } else {
            "Dieser Fließtext bricht bei einer Zeilenlänge um, die die Lesbarkeit beeinträchtigen kann."
        }
        .to_string(),
        "design.line_height" => {
            if level == FindingLevel::Warning {
                if en {
                    "This multi-line text has a line height tight enough that lines may visually overlap."
                } else {
                    "Dieser mehrzeilige Text hat einen Zeilenabstand, der so knapp ist, dass sich Zeilen optisch überlappen können."
                }
            } else if en {
                "This multi-line body text has a line height that may reduce readability."
            } else {
                "Dieser mehrzeilige Fließtext hat einen Zeilenabstand, der die Lesbarkeit beeinträchtigen kann."
            }
            .to_string()
        }
        "design.all_caps" => if en {
            "Long all-caps body text can be harder to read than mixed case."
        } else {
            "Langer Fließtext in Großbuchstaben kann schwerer lesbar sein als gemischte Groß-/Kleinschreibung."
        }
        .to_string(),
        "design.layout_transition" => if en {
            "Animating a layout-affecting CSS property can cause jank; consider transform/opacity instead."
        } else {
            "Das Animieren einer layoutwirksamen CSS-Eigenschaft kann Ruckeln verursachen; transform/opacity sind die bessere Wahl."
        }
        .to_string(),
        "design.alt_filename" => if en {
            "This image's alt text looks like an unedited filename rather than a description of its content."
        } else {
            "Der Alt-Text dieses Bildes wirkt wie ein unbearbeiteter Dateiname statt einer Beschreibung des Inhalts."
        }
        .to_string(),
        "design.alt_boilerplate" => if en {
            "This alt text starts with a boilerplate phrase (e.g. \"image of\") that adds no information for screen-reader users."
        } else {
            "Dieser Alt-Text beginnt mit einer Floskel (z. B. \"Bild von\"), die für Screenreader-Nutzende keine zusätzliche Information liefert."
        }
        .to_string(),
        "design.alt_too_long" => if en {
            "This alt text exceeds the practical screen-reader length guideline and may be read out in full before the surrounding content."
        } else {
            "Dieser Alt-Text überschreitet die praktische Längenempfehlung für Screenreader und wird unter Umständen vollständig vor dem umgebenden Inhalt vorgelesen."
        }
        .to_string(),
        "design.alt_redundant" => if en {
            "This image's alt text duplicates the immediately adjacent visible text or caption, so screen-reader users hear the same content twice."
        } else {
            "Der Alt-Text dieses Bildes wiederholt den unmittelbar angrenzenden sichtbaren Text oder die Bildunterschrift, wodurch Screenreader-Nutzende denselben Inhalt zweimal hören."
        }
        .to_string(),
        "design.emoji_usage" => if en {
            "Emoji in this text are read out individually by screen readers and can disrupt comprehension, especially in runs or before the main content."
        } else {
            "Emoji in diesem Text werden von Screenreadern einzeln vorgelesen und können das Verständnis beeinträchtigen, besonders in Gruppen oder vor dem eigentlichen Inhalt."
        }
        .to_string(),
        "design.hashtag_camel_case" => if en {
            "This hashtag has no internal capitalization to mark word boundaries, so a screen reader may read it as one unbroken word."
        } else {
            "Dieser Hashtag hat keine interne Großschreibung zur Kennzeichnung von Wortgrenzen, wodurch ein Screenreader ihn als ein einziges, ununterbrochenes Wort vorlesen kann."
        }
        .to_string(),
        _ => String::new(),
    }
}

fn check_overflow_clip(candidates: &[RawClipCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .map(|c| DesignQualityFinding {
            rule_id: "design.overflow_clip".to_string(),
            level: FindingLevel::Warning,
            confidence: Confidence::High,
            measurement_type: "heuristic".to_string(),
            selector: selector_or_unknown(&c.css_path),
            evidence: with_snippet(
                format!(
                    "Positioned element extends outside ancestor '{}', which clips overflow.",
                    c.clipping_ancestor
                ),
                &c.snippet,
            ),
            message: finding_message_text("design.overflow_clip", FindingLevel::Warning, true),
        })
        .collect()
}

fn check_line_length(candidates: &[RawTextCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .filter(|c| c.line_count >= 2)
        .filter_map(|c| {
            let chars_per_line = c.text_length as f64 / c.line_count as f64;
            if chars_per_line <= LINE_LENGTH_CHARS_THRESHOLD {
                return None;
            }
            Some(DesignQualityFinding {
                rule_id: "design.line_length".to_string(),
                level: FindingLevel::Advisory,
                confidence: Confidence::Medium,
                measurement_type: "heuristic".to_string(),
                selector: selector_or_unknown(&c.css_path),
                evidence: with_snippet(
                    format!(
                        "~{:.0} characters per line across {} lines (threshold {:.0}).",
                        chars_per_line, c.line_count, LINE_LENGTH_CHARS_THRESHOLD
                    ),
                    &c.snippet,
                ),
                message: finding_message_text("design.line_length", FindingLevel::Advisory, true),
            })
        })
        .collect()
}

fn check_line_height(candidates: &[RawTextCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .filter(|c| c.line_count >= 2)
        .filter_map(|c| {
            if c.line_height_ratio >= LINE_HEIGHT_RATIO_ADVISORY {
                return None;
            }
            let level = if c.line_height_ratio < LINE_HEIGHT_RATIO_WARNING {
                FindingLevel::Warning
            } else {
                FindingLevel::Advisory
            };
            let confidence = if level == FindingLevel::Warning {
                Confidence::High
            } else {
                Confidence::Medium
            };
            Some(DesignQualityFinding {
                rule_id: "design.line_height".to_string(),
                level,
                confidence,
                measurement_type: "heuristic".to_string(),
                selector: selector_or_unknown(&c.css_path),
                evidence: with_snippet(
                    format!(
                        "line-height/font-size ratio {:.2} across {} lines.",
                        c.line_height_ratio, c.line_count
                    ),
                    &c.snippet,
                ),
                message: finding_message_text("design.line_height", level, true),
            })
        })
        .collect()
}

fn check_all_caps(candidates: &[RawTextCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .filter(|c| c.text_length > ALL_CAPS_MIN_CHARS)
        .filter(|c| {
            c.text_transform == "uppercase"
                || (c.text.to_uppercase() == c.text && c.text.to_lowercase() != c.text)
        })
        .map(|c| DesignQualityFinding {
            rule_id: "design.all_caps".to_string(),
            level: FindingLevel::Advisory,
            confidence: Confidence::Medium,
            measurement_type: "heuristic".to_string(),
            selector: selector_or_unknown(&c.css_path),
            evidence: with_snippet(
                format!("{} characters of all-caps body text.", c.text_length),
                &c.snippet,
            ),
            message: finding_message_text("design.all_caps", FindingLevel::Advisory, true),
        })
        .collect()
}

/// Heuristic: does this alt text look like an unedited camera/CMS filename
/// (`img_1234.jpg`, `DSC00042`) rather than a description of the image?
///
/// Deliberately requires the text to contain no whitespace — any alt text
/// with a space is treated as prose and never flagged here, which avoids
/// false-positiving on genuine multi-word alt text. A single generic word
/// without trailing digits (e.g. plain "Foto") is also not flagged — that is
/// a different problem ("alt text too generic"), out of scope for this
/// heuristic.
fn is_filename_like_alt(alt: &str) -> bool {
    let trimmed = alt.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return false;
    }
    let lower = trimmed.to_lowercase();

    if FILENAME_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
    {
        return true;
    }

    let prefix_len = lower
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(lower.len());
    if prefix_len == 0 {
        return false;
    }
    let prefix = &lower[..prefix_len];
    if !GENERIC_FILENAME_PREFIXES.contains(&prefix) {
        return false;
    }
    let rest = lower[prefix_len..].trim_start_matches(['_', '-']);
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

fn check_alt_filename(candidates: &[RawAltCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .filter(|c| is_filename_like_alt(&c.alt))
        .map(|c| DesignQualityFinding {
            rule_id: "design.alt_filename".to_string(),
            level: FindingLevel::Advisory,
            confidence: Confidence::High,
            measurement_type: "heuristic".to_string(),
            selector: selector_or_unknown(&c.css_path),
            evidence: with_snippet(
                format!(
                    "alt text '{}' looks like a filename rather than a description.",
                    c.alt.trim()
                ),
                &c.snippet,
            ),
            message: finding_message_text("design.alt_filename", FindingLevel::Advisory, true),
        })
        .collect()
}

/// Load boilerplate alt-text prefixes ("Bild von …", "image of …") from FTL
/// for the given locale. Reuses the exact FTL-stopword-list mechanism #299
/// established for generic-linktext detection (see
/// `a11y_journey::link_inventory::stopwords_for_locale`), but under its own
/// key — different vocabulary, different check.
fn boilerplate_prefixes_for_locale(locale: &str) -> Vec<String> {
    let Ok(i18n) = crate::i18n::I18n::new(locale) else {
        return Vec::new();
    };
    let raw = i18n.t("alt-text-boilerplate-prefixes");
    if raw == "alt-text-boilerplate-prefixes" {
        return Vec::new(); // key missing — I18n returns the key itself as fallback
    }
    raw.split(',').map(|s| s.trim().to_lowercase()).collect()
}

/// Build merged boilerplate-prefix list from all supported locales — sites
/// frequently mix languages, so both are always checked regardless of report
/// language (mirrors `link_inventory::load_stopwords`).
fn load_boilerplate_prefixes() -> Vec<String> {
    let mut words: Vec<String> = SUPPORTED_ALT_BOILERPLATE_LOCALES
        .iter()
        .flat_map(|loc| boilerplate_prefixes_for_locale(loc))
        .collect();
    words.sort_unstable();
    words.dedup();
    words
}

fn check_alt_boilerplate(
    candidates: &[RawAltCandidate],
    prefixes: &[String],
) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .filter_map(|c| {
            let lower = c.alt.trim().to_lowercase();
            let matched = prefixes.iter().find(|p| lower.starts_with(p.as_str()))?;
            Some(DesignQualityFinding {
                rule_id: "design.alt_boilerplate".to_string(),
                level: FindingLevel::Advisory,
                confidence: Confidence::Medium,
                measurement_type: "heuristic".to_string(),
                selector: selector_or_unknown(&c.css_path),
                evidence: with_snippet(
                    format!("alt text starts with the boilerplate prefix '{matched}'."),
                    &c.snippet,
                ),
                message: finding_message_text(
                    "design.alt_boilerplate",
                    FindingLevel::Advisory,
                    true,
                ),
            })
        })
        .collect()
}

fn check_alt_length(candidates: &[RawAltCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .filter(|c| c.alt.trim().chars().count() > ALT_TEXT_MAX_CHARS)
        .map(|c| {
            let len = c.alt.trim().chars().count();
            DesignQualityFinding {
                rule_id: "design.alt_too_long".to_string(),
                level: FindingLevel::Advisory,
                confidence: Confidence::Medium,
                measurement_type: "heuristic".to_string(),
                selector: selector_or_unknown(&c.css_path),
                evidence: with_snippet(
                    format!(
                        "alt text is {len} characters long (practical screen-reader guidance: ~{ALT_TEXT_MAX_CHARS} or fewer)."
                    ),
                    &c.snippet,
                ),
                message: finding_message_text("design.alt_too_long", FindingLevel::Advisory, true),
            }
        })
        .collect()
}

/// Flags alt text that exactly duplicates (case-insensitively) the
/// immediately adjacent visible text/caption extracted alongside it.
/// Deliberately requires exact equality rather than containment — a
/// containment check would false-positive on short, coincidentally
/// overlapping captions.
fn check_alt_redundant(candidates: &[RawAltCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .filter(|c| {
            let alt_norm = c.alt.trim().to_lowercase();
            let adjacent_norm = c.adjacent_text.trim().to_lowercase();
            alt_norm.chars().count() >= ALT_TEXT_MIN_COMPARE_CHARS
                && adjacent_norm.chars().count() >= ALT_TEXT_MIN_COMPARE_CHARS
                && alt_norm == adjacent_norm
        })
        .map(|c| DesignQualityFinding {
            rule_id: "design.alt_redundant".to_string(),
            level: FindingLevel::Advisory,
            confidence: Confidence::Medium,
            measurement_type: "heuristic".to_string(),
            selector: selector_or_unknown(&c.css_path),
            evidence: with_snippet(
                "alt text duplicates the immediately adjacent visible text/caption.".to_string(),
                &c.snippet,
            ),
            message: finding_message_text("design.alt_redundant", FindingLevel::Advisory, true),
        })
        .collect()
}

/// Codepoint ranges covering the emoji blocks relevant to authored copy
/// (pictographs, emoticons, transport, misc symbols/dingbats, regional
/// indicators for flag emoji). Not an exhaustive Unicode emoji-property
/// implementation — deliberately dependency-free (no new crate), matching
/// this module's other deterministic string heuristics.
fn is_emoji_char(c: char) -> bool {
    matches!(c as u32,
        0x1F300..=0x1F5FF
        | 0x1F600..=0x1F64F
        | 0x1F680..=0x1F6FF
        | 0x1F900..=0x1F9FF
        | 0x1FA00..=0x1FAFF
        | 0x2600..=0x26FF
        | 0x2700..=0x27BF
        | 0x1F1E6..=0x1F1FF
    )
}

/// Variation selector / ZWJ / skin-tone modifier — continues an emoji run
/// without counting as a separate emoji occurrence.
fn is_emoji_modifier(c: char) -> bool {
    matches!(c as u32, 0xFE0F | 0x200D | 0x1F3FB..=0x1F3FF)
}

struct EmojiScan {
    max_consecutive_run: usize,
    /// True when a non-whitespace, non-emoji character follows the first
    /// emoji in the text — i.e. the emoji is not confined to the trailing
    /// position.
    mid_sentence: bool,
}

fn scan_emoji(text: &str) -> EmojiScan {
    let mut max_run = 0usize;
    let mut current_run = 0usize;
    let mut first_emoji_idx: Option<usize> = None;
    let mut last_non_emoji_content_idx: Option<usize> = None;

    for (i, c) in text.chars().enumerate() {
        if is_emoji_char(c) {
            current_run += 1;
            max_run = max_run.max(current_run);
            first_emoji_idx.get_or_insert(i);
        } else if is_emoji_modifier(c) {
            // continuation marker — does not break the run, not itself counted.
        } else {
            current_run = 0;
            if !c.is_whitespace() {
                last_non_emoji_content_idx = Some(i);
            }
        }
    }

    let mid_sentence = matches!(
        (first_emoji_idx, last_non_emoji_content_idx),
        (Some(first), Some(last)) if last > first
    );

    EmojiScan {
        max_consecutive_run: max_run,
        mid_sentence,
    }
}

fn check_emoji_usage(candidates: &[RawTextCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .filter_map(|c| {
            let scan = scan_emoji(&c.text);
            if scan.max_consecutive_run < EMOJI_RUN_THRESHOLD && !scan.mid_sentence {
                return None;
            }
            let evidence = match (
                scan.max_consecutive_run >= EMOJI_RUN_THRESHOLD,
                scan.mid_sentence,
            ) {
                (true, true) => format!(
                    "{} consecutive emoji, with emoji appearing before/mid-sentence rather than only trailing.",
                    scan.max_consecutive_run
                ),
                (true, false) => {
                    format!("{} consecutive emoji in a row.", scan.max_consecutive_run)
                }
                (false, true) => {
                    "Emoji appears before/mid-sentence rather than only trailing.".to_string()
                }
                (false, false) => unreachable!("filtered out above"),
            };
            Some(DesignQualityFinding {
                rule_id: "design.emoji_usage".to_string(),
                level: FindingLevel::Advisory,
                confidence: Confidence::Medium,
                measurement_type: "heuristic".to_string(),
                selector: selector_or_unknown(&c.css_path),
                evidence: with_snippet(evidence, &c.snippet),
                message: finding_message_text("design.emoji_usage", FindingLevel::Advisory, true),
            })
        })
        .collect()
}

/// Extracts `#hashtag` tokens (alphanumeric/underscore run following `#`)
/// from free text, without a regex crate.
fn extract_hashtags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for (byte_idx, ch) in text.char_indices() {
        if ch != '#' {
            continue;
        }
        let start = byte_idx + ch.len_utf8();
        if start >= text.len() {
            continue;
        }
        let rest = &text[start..];
        let end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if end > 0 {
            tags.push(rest[..end].to_string());
        }
    }
    tags
}

/// Optional add-on (#06): flags long hashtags with no internal
/// capitalization to mark word boundaries (e.g. `#derbundwirdbarrierefrei`
/// vs. `#DerBundWirdBarrierefrei`), which a screen reader reads as one
/// unbroken word. Only fires on hashtags long enough that word segmentation
/// plausibly matters, and only when the hashtag is a single case throughout
/// (already-mixed-case hashtags are not flagged).
fn check_hashtag_camel_case(candidates: &[RawTextCandidate]) -> Vec<DesignQualityFinding> {
    candidates
        .iter()
        .flat_map(|c| {
            extract_hashtags(&c.text)
                .into_iter()
                .filter(|tag| tag.chars().count() >= HASHTAG_MIN_CHARS_FOR_CAMEL_CASE)
                .filter(|tag| tag.chars().any(|ch| ch.is_alphabetic()))
                .filter(|tag| {
                    let has_upper = tag.chars().any(|ch| ch.is_uppercase());
                    let has_lower = tag.chars().any(|ch| ch.is_lowercase());
                    !(has_upper && has_lower)
                })
                .map(|tag| DesignQualityFinding {
                    rule_id: "design.hashtag_camel_case".to_string(),
                    level: FindingLevel::Advisory,
                    confidence: Confidence::Low,
                    measurement_type: "heuristic".to_string(),
                    selector: selector_or_unknown(&c.css_path),
                    evidence: with_snippet(
                        format!(
                            "Hashtag '#{tag}' has no internal capitalization to mark word boundaries."
                        ),
                        &c.snippet,
                    ),
                    message: finding_message_text(
                        "design.hashtag_camel_case",
                        FindingLevel::Advisory,
                        true,
                    ),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Re-project `performance::animations` findings restricted to layout-affecting
/// properties as design-quality advisories (#528 rule 5). Deliberately does
/// not re-detect transitions independently — see `module.rs`'s `derive()`.
fn layout_transition_advisories(
    animations: &crate::performance::AnimationAnalysis,
) -> Vec<DesignQualityFinding> {
    animations
        .findings
        .iter()
        .filter(|f| f.kind == "transition")
        .filter(|f| {
            LAYOUT_TRANSITION_PROPERTIES
                .iter()
                .any(|p| *p == f.property)
        })
        .map(|f| DesignQualityFinding {
            rule_id: "design.layout_transition".to_string(),
            level: FindingLevel::Advisory,
            confidence: Confidence::Medium,
            measurement_type: "heuristic".to_string(),
            selector: f.name.clone(),
            evidence: format!("CSS transition animates layout property '{}'.", f.property),
            message: finding_message_text("design.layout_transition", FindingLevel::Advisory, true),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_candidate(
        text_length: usize,
        line_count: u32,
        _font_size: f64,
        line_height_ratio: f64,
        text_transform: &str,
        text: &str,
    ) -> RawTextCandidate {
        RawTextCandidate {
            css_path: Some("p.test".to_string()),
            snippet: None,
            text: text.to_string(),
            text_length,
            line_height_ratio,
            text_transform: text_transform.to_string(),
            line_count,
        }
    }

    #[test]
    fn line_length_fires_when_multiline_and_over_threshold() {
        let c = text_candidate(400, 3, 16.0, 1.4, "none", "lorem ipsum");
        let findings = check_line_length(&[c]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "design.line_length");
        assert_eq!(findings[0].level, FindingLevel::Advisory);
    }

    #[test]
    fn line_length_ignores_single_line() {
        let c = text_candidate(400, 1, 16.0, 1.4, "none", "lorem ipsum");
        assert!(check_line_length(&[c]).is_empty());
    }

    #[test]
    fn line_length_ignores_short_lines() {
        let c = text_candidate(120, 3, 16.0, 1.4, "none", "lorem ipsum");
        assert!(check_line_length(&[c]).is_empty());
    }

    #[test]
    fn line_height_warning_below_one() {
        let c = text_candidate(120, 3, 16.0, 0.9, "none", "lorem ipsum");
        let findings = check_line_height(&[c]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].level, FindingLevel::Warning);
    }

    #[test]
    fn line_height_advisory_between_one_and_threshold() {
        let c = text_candidate(120, 3, 16.0, 1.1, "none", "lorem ipsum");
        let findings = check_line_height(&[c]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].level, FindingLevel::Advisory);
    }

    #[test]
    fn line_height_passes_above_threshold() {
        let c = text_candidate(120, 3, 16.0, 1.5, "none", "lorem ipsum");
        assert!(check_line_height(&[c]).is_empty());
    }

    #[test]
    fn all_caps_fires_on_css_transform() {
        let c = text_candidate(80, 1, 16.0, 1.4, "uppercase", "lorem ipsum dolor sit amet");
        let findings = check_all_caps(&[c]);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn all_caps_fires_on_literal_caps_text() {
        let long_caps = "A".repeat(70);
        let c = text_candidate(70, 1, 16.0, 1.4, "none", &long_caps);
        let findings = check_all_caps(&[c]);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn all_caps_ignores_short_text() {
        let c = text_candidate(10, 1, 16.0, 1.4, "uppercase", "SHORT");
        assert!(check_all_caps(&[c]).is_empty());
    }

    #[test]
    fn all_caps_ignores_mixed_case() {
        let c = text_candidate(
            80,
            1,
            16.0,
            1.4,
            "none",
            "lorem ipsum dolor sit amet consectetur",
        );
        assert!(check_all_caps(&[c]).is_empty());
    }

    #[test]
    fn overflow_clip_produces_warning() {
        let c = RawClipCandidate {
            css_path: Some("button.cta".to_string()),
            snippet: None,
            clipping_ancestor: "div.hero".to_string(),
        };
        let findings = check_overflow_clip(&[c]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].level, FindingLevel::Warning);
        assert_eq!(findings[0].confidence, Confidence::High);
    }

    #[test]
    fn layout_transition_keeps_only_layout_properties() {
        let animations = crate::performance::AnimationAnalysis {
            findings: vec![
                crate::performance::NonCompositedAnimation {
                    kind: "transition".to_string(),
                    name: "div.card".to_string(),
                    property: "width".to_string(),
                    source: "inline".to_string(),
                },
                crate::performance::NonCompositedAnimation {
                    kind: "transition".to_string(),
                    name: "div.card".to_string(),
                    property: "color".to_string(),
                    source: "inline".to_string(),
                },
                crate::performance::NonCompositedAnimation {
                    kind: "keyframe".to_string(),
                    name: "spin".to_string(),
                    property: "margin".to_string(),
                    source: "inline".to_string(),
                },
            ],
            total_count: 3,
            affected_properties: vec![
                "color".to_string(),
                "margin".to_string(),
                "width".to_string(),
            ],
        };
        let findings = layout_transition_advisories(&animations);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].selector, "div.card");
    }

    #[test]
    fn no_measurement_type_ever_deviates_from_heuristic() {
        let c = text_candidate(80, 1, 16.0, 1.4, "uppercase", "lorem ipsum dolor sit amet");
        for f in check_all_caps(&[c]) {
            assert_eq!(f.measurement_type, "heuristic");
        }
    }

    fn alt_candidate(alt: &str, adjacent_text: &str) -> RawAltCandidate {
        RawAltCandidate {
            css_path: Some("img.test".to_string()),
            snippet: None,
            alt: alt.to_string(),
            adjacent_text: adjacent_text.to_string(),
        }
    }

    #[test]
    fn alt_filename_fires_on_camera_style_name() {
        let c = alt_candidate("IMG_1234.jpg", "");
        let findings = check_alt_filename(&[c]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "design.alt_filename");
    }

    #[test]
    fn alt_filename_fires_on_generic_prefix_and_digits_without_extension() {
        let c = alt_candidate("DSC_0042", "");
        assert_eq!(check_alt_filename(&[c]).len(), 1);
    }

    #[test]
    fn alt_filename_ignores_meaningful_multi_word_alt() {
        let c = alt_candidate("Sonnenuntergang über dem Meer", "");
        assert!(check_alt_filename(&[c]).is_empty());
    }

    #[test]
    fn alt_filename_ignores_meaningful_single_word_alt() {
        // Accepted tradeoff: a single-word alt text with a known image
        // extension (e.g. literally "sonnenuntergang.jpg") would still be
        // flagged — that combination is rare enough in practice not to be
        // worth a more elaborate heuristic.
        let c = alt_candidate("Sonnenuntergang", "");
        assert!(check_alt_filename(&[c]).is_empty());
    }

    #[test]
    fn alt_boilerplate_fires_on_known_prefix() {
        let prefixes = vec!["image of".to_string(), "bild von".to_string()];
        let c = alt_candidate("Image of a red bicycle", "");
        let findings = check_alt_boilerplate(&[c], &prefixes);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "design.alt_boilerplate");
    }

    #[test]
    fn alt_boilerplate_ignores_descriptive_alt() {
        let prefixes = vec!["image of".to_string()];
        let c = alt_candidate("A red bicycle leaning against a brick wall", "");
        assert!(check_alt_boilerplate(&[c], &prefixes).is_empty());
    }

    #[test]
    fn alt_too_long_fires_over_threshold() {
        let long_alt = "a".repeat(ALT_TEXT_MAX_CHARS + 1);
        let c = alt_candidate(&long_alt, "");
        assert_eq!(check_alt_length(&[c]).len(), 1);
    }

    #[test]
    fn alt_too_long_ignores_short_alt() {
        let c = alt_candidate("A red bicycle leaning against a wall", "");
        assert!(check_alt_length(&[c]).is_empty());
    }

    #[test]
    fn alt_redundant_fires_when_matching_caption() {
        let c = alt_candidate(
            "Team photo at the conference",
            "Team photo at the conference",
        );
        assert_eq!(check_alt_redundant(&[c]).len(), 1);
    }

    #[test]
    fn alt_redundant_ignores_distinct_caption() {
        let c = alt_candidate("Team photo at the conference", "Figure 3");
        assert!(check_alt_redundant(&[c]).is_empty());
    }

    #[test]
    fn alt_redundant_ignores_missing_caption() {
        let c = alt_candidate("Team photo at the conference", "");
        assert!(check_alt_redundant(&[c]).is_empty());
    }

    #[test]
    fn emoji_usage_fires_on_consecutive_trailing_run() {
        let c = text_candidate(20, 1, 16.0, 1.4, "none", "Jetzt anmelden! 🚀✨");
        assert_eq!(check_emoji_usage(&[c]).len(), 1);
    }

    #[test]
    fn emoji_usage_fires_on_mid_sentence_emoji() {
        let c = text_candidate(20, 1, 16.0, 1.4, "none", "🎉 Jetzt anmelden");
        assert_eq!(check_emoji_usage(&[c]).len(), 1);
    }

    #[test]
    fn emoji_usage_ignores_single_trailing_emoji() {
        let c = text_candidate(20, 1, 16.0, 1.4, "none", "Jetzt anmelden 🚀");
        assert!(check_emoji_usage(&[c]).is_empty());
    }

    #[test]
    fn emoji_usage_ignores_plain_text() {
        let c = text_candidate(20, 1, 16.0, 1.4, "none", "Jetzt anmelden");
        assert!(check_emoji_usage(&[c]).is_empty());
    }

    #[test]
    fn hashtag_camel_case_fires_on_long_lowercase_hashtag() {
        let c = text_candidate(30, 1, 16.0, 1.4, "none", "#derbundwirdbarrierefrei");
        assert_eq!(check_hashtag_camel_case(&[c]).len(), 1);
    }

    #[test]
    fn hashtag_camel_case_ignores_mixed_case_hashtag() {
        let c = text_candidate(30, 1, 16.0, 1.4, "none", "#DerBundWirdBarrierefrei");
        assert!(check_hashtag_camel_case(&[c]).is_empty());
    }

    #[test]
    fn hashtag_camel_case_ignores_short_hashtag() {
        let c = text_candidate(10, 1, 16.0, 1.4, "none", "#wcag");
        assert!(check_hashtag_camel_case(&[c]).is_empty());
    }

    #[test]
    fn english_message_text_has_no_known_german_leaks() {
        // #406 guard: canonical (en=true) message text must be real English,
        // never a German sentence with the umlaut/ß left in.
        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));
        for rule_id in [
            "design.overflow_clip",
            "design.line_length",
            "design.all_caps",
            "design.layout_transition",
            "design.alt_filename",
            "design.alt_boilerplate",
            "design.alt_too_long",
            "design.alt_redundant",
            "design.emoji_usage",
            "design.hashtag_camel_case",
        ] {
            let msg = finding_message_text(rule_id, FindingLevel::Advisory, true);
            assert!(!msg.is_empty(), "{rule_id} must produce a message");
            assert!(
                !has_umlaut(&msg),
                "{rule_id} EN message leaks German: {msg}"
            );
        }
        for level in [FindingLevel::Warning, FindingLevel::Advisory] {
            let msg = finding_message_text("design.line_height", level, true);
            assert!(
                !has_umlaut(&msg),
                "design.line_height EN message leaks German: {msg}"
            );
        }
    }

    #[test]
    fn german_message_text_differs_from_english() {
        for rule_id in [
            "design.overflow_clip",
            "design.line_length",
            "design.all_caps",
            "design.layout_transition",
            "design.alt_filename",
            "design.alt_boilerplate",
            "design.alt_too_long",
            "design.alt_redundant",
            "design.emoji_usage",
            "design.hashtag_camel_case",
        ] {
            let en = finding_message_text(rule_id, FindingLevel::Advisory, true);
            let de = finding_message_text(rule_id, FindingLevel::Advisory, false);
            assert_ne!(en, de, "{rule_id} must have a distinct German translation");
        }
    }
}
