//! Help-mechanism inventory for WCAG 3.2.6 Consistent Help (plan 54 §3).
//!
//! 3.2.6 is a cross-page criterion: a help mechanism that is repeated on
//! several pages must appear in the same order relative to the other page
//! content. A single page can't violate it, so this module only records, per
//! page, which help mechanisms exist and where; `audit::batch_consistency`
//! compares the inventories across the audited set.
//!
//! Recorded mechanisms (the four kinds the criterion names, as far as they
//! are recognisable in the DOM):
//! - human contact details — `mailto:` and `tel:` links,
//! - contact/help pages — links whose text or path names contact, support,
//!   help or FAQ,
//! - chat widgets — a short list of known vendors' container elements.
//!
//! Only page chrome is inventoried: anything inside `main` is page-specific
//! content (a contact link in a blog post), not a repeated mechanism, and
//! would turn every article into a false "inconsistency". Invisible elements
//! are skipped for the same reason — a collapsed mobile menu is not where the
//! mechanism is offered at this viewport.
//!
//! Score-neutral: the inventory feeds the batch report only.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::PatternAnalysis;

/// What kind of help a mechanism offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpKind {
    Email,
    Phone,
    ContactPage,
    Chat,
}

/// Where on the page a mechanism sits, relative to the page's landmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpRegion {
    Header,
    Navigation,
    Complementary,
    Footer,
    /// Fixed-position overlay, e.g. a chat launcher.
    Floating,
    /// Outside `main` but in no landmark.
    Other,
}

/// One help mechanism occurrence, in document order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpMechanism {
    pub kind: HelpKind,
    /// Identity across pages: `mailto:<address>`, `tel:<digits>`,
    /// `page:<host><path>` or `chat:<vendor>`.
    pub key: String,
    pub region: HelpRegion,
}

const HELP_MECHANISMS_JS: &str = r#"
(() => {
  const HELP_WORDS = /(^|[^a-zäöüß])(kontakt|contact|support|hilfe|help|faq|kundenservice|kundendienst|customer service)([^a-zäöüß]|$)/i;
  const CHAT = [
    ['intercom', '.intercom-launcher, .intercom-lightweight-app, iframe[name="intercom-launcher-frame"]'],
    ['zendesk', 'iframe#launcher'],
    ['tawk', 'iframe[title="chat widget"]'],
    ['crisp', '.crisp-client'],
    ['tidio', '#tidio-chat'],
    ['userlike', '#userlike-tab, [id^="userlike-"]'],
    ['hubspot', '#hubspot-messages-iframe-container'],
    ['livechat', '#chat-widget-container'],
    ['drift', '#drift-widget-container, #drift-frame-controller']
  ];

  function visible(el) {
    if (!el.getClientRects().length) return false;
    const s = getComputedStyle(el);
    return s.visibility !== 'hidden' && s.display !== 'none';
  }

  function isFixed(el) {
    for (let n = el; n && n !== document.body; n = n.parentElement) {
      if (getComputedStyle(n).position === 'fixed') return true;
    }
    return false;
  }

  function region(el) {
    if (el.closest('main, [role="main"]')) return null;
    if (el.closest('header, [role="banner"]')) return 'header';
    if (el.closest('footer, [role="contentinfo"]')) return 'footer';
    if (el.closest('nav, [role="navigation"]')) return 'navigation';
    if (el.closest('aside, [role="complementary"]')) return 'complementary';
    if (isFixed(el)) return 'floating';
    return 'other';
  }

  const out = [];
  const seen = new Set();
  function push(el, kind, key) {
    if (out.length >= 30) return;
    const r = region(el);
    if (!r) return;
    const id = key + '|' + r;
    if (seen.has(id)) return;
    seen.add(id);
    out.push({ kind: kind, key: key, region: r, el: el });
  }

  for (const a of document.querySelectorAll('a[href]')) {
    if (!visible(a)) continue;
    const href = (a.getAttribute('href') || '').trim();
    const lower = href.toLowerCase();
    if (lower.startsWith('mailto:')) {
      const addr = lower.slice(7).split('?')[0].trim();
      if (addr) push(a, 'email', 'mailto:' + addr);
      continue;
    }
    if (lower.startsWith('tel:')) {
      const digits = lower.slice(4).replace(/[^0-9+]/g, '');
      if (digits) push(a, 'phone', 'tel:' + digits);
      continue;
    }
    let url;
    try { url = new URL(href, location.href); } catch (e) { continue; }
    if (url.protocol !== 'http:' && url.protocol !== 'https:') continue;
    const path = url.pathname.toLowerCase().replace(/\/+$/, '') || '/';
    const text = (a.textContent || '') + ' ' + (a.getAttribute('aria-label') || '');
    if (HELP_WORDS.test(text) || HELP_WORDS.test(path.replace(/[-_/.]/g, ' '))) {
      push(a, 'contact_page', 'page:' + url.host.toLowerCase() + path);
    }
  }

  for (const [vendor, selector] of CHAT) {
    const el = document.querySelector(selector);
    if (el && visible(el)) push(el, 'chat', 'chat:' + vendor);
  }

  // Document order, so the batch comparison can read relative order directly.
  out.sort((x, y) => {
    if (x.el === y.el) return 0;
    return x.el.compareDocumentPosition(y.el) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1;
  });
  return out.map(m => ({ kind: m.kind, key: m.key, region: m.region }));
})()
"#;

/// Inventory the page's help mechanisms into `out.help_mechanisms`. A failed
/// evaluation leaves the inventory empty; the batch comparison then simply
/// has one page less to compare, which it reports through its page counts.
pub(crate) async fn detect(page: &Page, out: &mut PatternAnalysis) {
    let value = match page.evaluate(HELP_MECHANISMS_JS).await {
        Ok(eval) => eval.value().cloned(),
        Err(e) => {
            warn!("Help-mechanism detection JS failed: {}", e);
            return;
        }
    };
    let Some(value) = value else { return };
    match serde_json::from_value::<Vec<HelpMechanism>>(value) {
        Ok(mechanisms) => out.help_mechanisms = mechanisms,
        Err(e) => warn!("Failed to parse help-mechanism inventory: {}", e),
    }
}
