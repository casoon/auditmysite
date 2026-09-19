//! WCAG 1.2.8 Media Alternative (Prerecorded) (Level AAA)
//!
//! An alternative for time-based media is provided for all prerecorded
//! synchronized media and for all prerecorded video-only media.
//!
//! Sign-language presence/positioning and audio-description remain out of
//! scope — verifying either requires visual/auditory content judgment this
//! tool cannot make from DOM/structure alone, so this stays a manual-review
//! notice for those two aspects (same precedent as before). What *is*
//! deepened here is presence detection: native `<video>`/embedded players
//! are found via a live DOM query instead of AXTree role matching (the old
//! role list — `video`/`VideoElement`/`EmbeddedObject` — never matched
//! Chrome's actual AXTree role for a `<video>` element, see
//! `media_rules.rs`'s docs), and a same-origin transcript-link heuristic
//! enriches the manual-review message with concrete evidence when found.

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::i18n::I18n;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub const MEDIA_ALTERNATIVE_RULE: RuleMetadata = RuleMetadata {
    id: "1.2.8",
    name: "Media Alternative (Prerecorded)",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Prerecorded media has a text alternative or description track",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/media-alternative-prerecorded.html",
    axe_id: "media-alt",
    tags: &["wcag2aaa", "wcag128", "cat.time-and-media"],
};

/// Locales whose FTL `video-transcript-keywords` list is merged, regardless
/// of report language — mirrors `a11y_journey::link_inventory`'s stopword
/// merge (sites frequently mix languages in link text).
const SUPPORTED_LOCALES: &[&str] = &["de", "en"];

/// Load merged transcript keywords ("Transcript", "Transkript", …) from FTL.
fn transcript_keywords() -> Vec<String> {
    let mut words: Vec<String> = SUPPORTED_LOCALES
        .iter()
        .filter_map(|loc| I18n::new(loc).ok())
        .flat_map(|i18n| {
            let raw = i18n.t("video-transcript-keywords");
            if raw == "video-transcript-keywords" {
                Vec::new() // key missing — I18n returns the key itself as fallback
            } else {
                raw.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }
        })
        .collect();
    words.sort_unstable();
    words.dedup();
    words
}

/// Escape a plain keyword for safe use inside a JS `RegExp` alternation.
fn escape_regex(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for c in word.chars() {
        if "\\^$.|?*+()[]{}".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Build the `word1|word2|…` alternation pattern used by the in-page
/// transcript-link scan. Falls back to a literal "transcript" if the FTL
/// key is ever missing, so the scan never turns into a match-nothing no-op.
fn transcript_keyword_pattern() -> String {
    let words = transcript_keywords();
    if words.is_empty() {
        return "transcript".to_string();
    }
    words
        .iter()
        .map(|w| escape_regex(w))
        .collect::<Vec<_>>()
        .join("|")
}

/// JS that detects native `<video>`/video-embed `<iframe>` presence and a
/// nearby transcript link. "Nearby" is a shared-ancestor check (looking up
/// to 4 levels from both the media element and the candidate link) rather
/// than pixel-distance — deterministic across responsive layouts and
/// testable against static fixture HTML.
fn media_alt_scan_js(keyword_pattern: &str) -> String {
    let pattern_json =
        serde_json::to_string(keyword_pattern).unwrap_or_else(|_| "\"transcript\"".to_string());
    format!(
        r#"
var keywordRe = new RegExp({pattern_json}, 'i');
function ancestors(el, depth) {{
  var list = [];
  var cur = el;
  for (var i = 0; i < depth && cur; i++) {{
    list.push(cur);
    cur = cur.parentElement;
  }}
  return list;
}}
var mediaEls = Array.prototype.slice.call(document.querySelectorAll('video, iframe[src]'));
var mediaAncestors = mediaEls.map(function(el) {{ return ancestors(el, 4); }});
var transcriptHref = null;
var transcriptText = null;
var links = Array.prototype.slice.call(document.querySelectorAll('a[href], [role="link"]'));
outer:
for (var i = 0; i < links.length; i++) {{
  var link = links[i];
  var text = (link.textContent || '').trim();
  if (!text || !keywordRe.test(text)) continue;
  var linkAncestors = ancestors(link, 4);
  for (var m = 0; m < mediaAncestors.length; m++) {{
    var shared = mediaAncestors[m].some(function(a) {{ return linkAncestors.indexOf(a) !== -1; }});
    if (shared) {{
      transcriptHref = link.getAttribute('href') || '';
      transcriptText = text.substring(0, 80);
      break outer;
    }}
  }}
}}
return {{
  hasVideo: document.querySelectorAll('video').length > 0,
  hasEmbed: document.querySelectorAll('iframe[src]').length > 0,
  transcriptHref: transcriptHref,
  transcriptText: transcriptText
}};
"#
    )
}

/// 1.2.8 Media Alternative (Prerecorded) — DOM deepening (#video-caption-checks).
///
/// Always emits exactly one manual-review notice (unchanged stance — sign
/// language / audio description cannot be verified automatically), but the
/// message is enriched with a same-origin transcript-link match when found.
/// Presence-only: finding a transcript link is evidence an alternative may
/// already exist, not proof it is complete or accurate.
pub async fn check_media_alternative_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &media_alt_scan_js(&transcript_keyword_pattern()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("media-alt DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                MEDIA_ALTERNATIVE_RULE.axe_id,
                MEDIA_ALTERNATIVE_RULE.level,
                "page_evaluation_failed",
            )];
        }
    };
    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            MEDIA_ALTERNATIVE_RULE.axe_id,
            MEDIA_ALTERNATIVE_RULE.level,
            "missing_evaluation_value",
        )];
    };

    let has_video = value
        .get("hasVideo")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let has_embed = value
        .get("hasEmbed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let transcript_text = value.get("transcriptText").and_then(|v| v.as_str());

    if !has_video && !has_embed {
        // No video found — emit a general notice anyway since the DOM query
        // may not capture every embedded media pattern (e.g. custom players).
        return vec![Violation::new(
            MEDIA_ALTERNATIVE_RULE.id,
            MEDIA_ALTERNATIVE_RULE.name,
            MEDIA_ALTERNATIVE_RULE.level,
            Severity::Low,
            "WCAG 1.2.8 requires manual verification that all prerecorded video and \
             audio content has a full text alternative.",
            "page",
        )
        .with_fix(
            "Ensure every prerecorded multimedia element has a complete text transcript \
             and, where applicable, an audio description.",
        )
        .with_rule_id(MEDIA_ALTERNATIVE_RULE.axe_id)
        .with_help_url(MEDIA_ALTERNATIVE_RULE.help_url)
        .with_kind(Outcome::Untested)];
    }

    let message = if let Some(text) = transcript_text {
        format!(
            "Video content detected, and a nearby link mentioning a transcript (\"{text}\") \
             was found. This does not confirm completeness or accuracy — verify the transcript \
             covers all spoken dialogue and, where applicable, that audio descriptions exist for \
             visual-only content."
        )
    } else {
        "Video content detected. Verify that a full text alternative or audio description \
         track exists for all prerecorded video content."
            .to_string()
    };

    vec![Violation::new(
        MEDIA_ALTERNATIVE_RULE.id,
        MEDIA_ALTERNATIVE_RULE.name,
        MEDIA_ALTERNATIVE_RULE.level,
        Severity::Medium,
        message,
        "video",
    )
    .with_fix(
        "Provide a text transcript or synchronized audio description for prerecorded \
         video. Add a <track kind=\"descriptions\"> element or link to a text alternative.",
    )
    .with_rule_id(MEDIA_ALTERNATIVE_RULE.axe_id)
    .with_help_url(MEDIA_ALTERNATIVE_RULE.help_url)
    .with_kind(Outcome::Untested)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_keyword_pattern_contains_both_locales() {
        let pattern = transcript_keyword_pattern();
        assert!(pattern.to_lowercase().contains("transcript"));
        assert!(pattern.to_lowercase().contains("transkript"));
    }

    #[test]
    fn test_escape_regex_escapes_special_chars() {
        assert_eq!(escape_regex("a.b"), "a\\.b");
        assert_eq!(escape_regex("plain"), "plain");
    }

    #[test]
    fn test_rule_metadata_unchanged() {
        assert_eq!(MEDIA_ALTERNATIVE_RULE.id, "1.2.8");
        assert_eq!(MEDIA_ALTERNATIVE_RULE.axe_id, "media-alt");
        assert_eq!(MEDIA_ALTERNATIVE_RULE.level, WcagLevel::AAA);
    }
}
