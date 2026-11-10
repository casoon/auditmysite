(() => {
  const globals = ['__tcfapi', '__cmp', '__gpp', 'Cookiebot', 'OneTrust', 'Optanon', 'UC_UI',
    'Didomi', 'klaro', 'BorlabsCookie', 'cookieconsent', 'CookieConsent', 'cmplz_cookiebanner',
    'Cookiefirst', 'CookieScript', 'axeptio', '_axcb'];
  const elements = {
    'cookiebot': '#CybotCookiebotDialog',
    'onetrust-banner-sdk': '#onetrust-banner-sdk',
    'usercentrics': '#usercentrics-root, #usercentrics-cmp-ui',
    'didomi': '#didomi-host',
    'borlabs': '#BorlabsCookieBox, [data-borlabs-cookie-wrap]',
    'klaro': '.klaro',
    'complianz': '.cmplz-cookiebanner, #cmplz-cookiebanner-container',
    'cookie-law-info': '#cookie-law-info-bar',
    'cookie-notice': '#cookie-notice',
    'cookieconsent': '.cc-window, .cc-banner',
    'generic-banner': '#cookiebanner, #cookie-banner, .cookie-banner, #cookie-consent, .cookie-consent, [class*="cookie-consent" i], [class*="consent-banner" i], [id*="consent-banner" i]',
    'consent-dialog': '[role="dialog"][aria-label*="cookie" i], [role="dialog"][aria-label*="consent" i], [role="dialog"][aria-label*="datenschutz" i], dialog[id*="cookie" i], dialog[id*="consent" i]'
  };
  const out = { consent_signals: [], links: [] };
  for (const g of globals) { try { if (typeof window[g] !== 'undefined') out.consent_signals.push('global:' + g); } catch (e) {} }
  for (const [id, sel] of Object.entries(elements)) {
    try { if (document.querySelector(sel)) out.consent_signals.push('element:' + id); } catch (e) {}
  }
  for (const a of Array.from(document.querySelectorAll('a[href]')).slice(0, 2000)) {
    const text = ((a.textContent || '').replace(/\s+/g, ' ').trim() || a.getAttribute('aria-label') || a.getAttribute('title') || '').slice(0, 120);
    out.links.push({ href: a.getAttribute('href') || '', resolved: a.href || '', text });
  }
  return JSON.stringify(out);
})()