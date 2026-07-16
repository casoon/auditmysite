//! Conservative WCAG 3.1.2 language-of-parts heuristic.
//!
//! Only long German/English passages with a strong function-word signal are
//! considered. Names, code, addresses, short phrases and quotations are
//! excluded, and findings remain manual-review warnings.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const LANGUAGE_OF_PARTS_RULE: RuleMetadata = RuleMetadata {
    id: "3.1.2",
    name: "Language of Parts",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Long passages in another language identify that language programmatically",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/language-of-parts.html",
    axe_id: "language-of-parts",
    tags: &["wcag2aa", "wcag312", "cat.language", "heuristic"],
};

const LANGUAGE_OF_PARTS_JS: &str = r#"
(() => {
  const documentLanguage = (document.documentElement.lang || '').split('-')[0].toLowerCase();
  if (!['de', 'en'].includes(documentLanguage)) return { supported: false, candidates: [] };
  const markers = {
    de: new Set(['der','die','das','und','ist','sind','mit','für','von','auf','eine','einer','nicht','werden','wird','auch','dass']),
    en: new Set(['the','and','is','are','with','for','from','this','that','not','will','also','have','has','your','our'])
  };
  const selectorFor = element => {
    if (element.id) return `${element.tagName.toLowerCase()}#${CSS.escape(element.id)}`;
    const parent = element.parentElement;
    if (!parent) return element.tagName.toLowerCase();
    const peers = [...parent.children].filter(child => child.tagName === element.tagName);
    return `${element.tagName.toLowerCase()}:nth-of-type(${peers.indexOf(element) + 1})`;
  };
  const candidates = [];
  for (const element of document.querySelectorAll('p, li, dd, td, figcaption')) {
    if (candidates.length >= 5 || element.closest('code, pre, address, blockquote, [translate="no"]')) continue;
    const text = (element.innerText || '').replace(/\s+/g, ' ').trim();
    const words = (text.toLowerCase().match(/[\p{L}]+/gu) || []);
    if (text.length < 100 || words.length < 16) continue;
    const declared = (element.closest('[lang]')?.lang || documentLanguage).split('-')[0].toLowerCase();
    if (declared !== documentLanguage) continue;
    const de = words.filter(word => markers.de.has(word)).length;
    const en = words.filter(word => markers.en.has(word)).length;
    const detected = de >= 5 && en <= 1 ? 'de' : en >= 5 && de <= 1 ? 'en' : null;
    if (detected && detected !== documentLanguage) {
      candidates.push({ selector: selectorFor(element), detected, sample: text.slice(0, 120) });
    }
  }
  return { supported: true, documentLanguage, candidates };
})()
"#;

pub async fn check_language_of_parts_with_page(page: &Page) -> Vec<Violation> {
    let value = match page.evaluate(LANGUAGE_OF_PARTS_JS).await {
        Ok(result) => result.value().cloned().unwrap_or_default(),
        Err(_) => {
            return vec![crate::wcag::technical_rule_failure(
                &LANGUAGE_OF_PARTS_RULE,
                "page_evaluation_failed",
            )]
        }
    };
    if !value
        .get("supported")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Vec::new();
    }
    value
        .get("candidates")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|candidate| {
            let selector = candidate.get("selector")?.as_str()?;
            let detected = candidate.get("detected")?.as_str()?;
            let sample = candidate.get("sample")?.as_str()?;
            Some(
                Violation::new(
                    LANGUAGE_OF_PARTS_RULE.id,
                    LANGUAGE_OF_PARTS_RULE.name,
                    LANGUAGE_OF_PARTS_RULE.level,
                    LANGUAGE_OF_PARTS_RULE.severity,
                    format!(
                        "A long passage strongly resembles language '{detected}' but inherits the page language: \"{sample}\""
                    ),
                    selector,
                )
                .with_selector(selector)
                .with_rule_id(LANGUAGE_OF_PARTS_RULE.axe_id)
                .with_kind(FindingKind::Warning)
                .with_fix(format!(
                    "Confirm the passage language and add lang=\"{detected}\" when it differs from the surrounding content"
                ))
                .with_help_url(LANGUAGE_OF_PARTS_RULE.help_url),
            )
        })
        .collect()
}
