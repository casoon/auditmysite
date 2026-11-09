
(function() {
    // Check visible role=dialog elements with consent-related content
    const dialogs = document.querySelectorAll('[role="dialog"],[role="alertdialog"]');
    for (const d of dialogs) {
        const rect = d.getBoundingClientRect();
        if (rect.width > 0 && rect.height > 0) {
            const text = (d.textContent || '').toLowerCase();
            if (text.includes('cookie') || text.includes('consent') || text.includes('datenschutz') ||
                text.includes('privacy') || text.includes('daten') || text.includes('accept') ||
                text.includes('akzeptieren') || text.includes('zustimmen')) {
                return { detected: true, evidence: 'visible consent-related dialog' };
            }
        }
    }
    // Known CMP container IDs and classes
    const selectors = [
        '#CybotCookiebotDialog',
        '#onetrust-banner-sdk',
        '#onetrust-consent-sdk',
        '#usercentrics-root',
        '#cmp-app-container',
        '#gdpr-consent-tool-wrapper',
        '#klaro',
        '.cookie-consent-banner',
        '.cookie-banner',
        '.cookie-notice',
        '#cookie-law-info-bar',
        '#cookiebanner',
        '#cookieNotice',
        '.cc-window',
        '[id*="cookie"][id*="banner"]',
        '[class*="consent-banner"]',
        '[id*="consent-banner"]'
    ];
    for (const sel of selectors) {
        try {
            const el = document.querySelector(sel);
            if (el) {
                const rect = el.getBoundingClientRect();
                if (rect.width > 0 && rect.height > 0) return { detected: true, evidence: 'visible CMP or consent container: ' + sel };
            }
        } catch(e) {}
    }
    return { detected: false, evidence: 'no visible allowlisted or heuristic consent surface' };
})()
