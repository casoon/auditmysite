
(function() {
    const selectors = [
        // Cookiebot
        '#CybotCookiebotDialogBodyLevelButtonLevelOptinAllowAll',
        '#CybotCookiebotDialogBodyButtonAccept',
        // OneTrust
        '#onetrust-accept-btn-handler',
        '.onetrust-accept-btn-handler',
        // Usercentrics
        '[data-testid="uc-accept-all-button"]',
        // Consentmanager
        '#cmp_btn-acceptall',
        '.cmp_btn-acceptall',
        // Klaro
        '.klaro .cm-btn-accept-all',
        // Cookie Consent (osano)
        '.cc-accept',
        '.cc-btn.cc-allow',
        // Borlabs
        '.borlabs-cookie__btn--accept',
        // Generic patterns (English)
        'button[id*="accept-all" i]',
        'button[id*="acceptall" i]',
        'button[id*="accept_all" i]',
        'button[class*="accept-all" i]',
        'button[class*="acceptall" i]',
        'a[id*="accept-all" i]',
        'a[class*="accept-all" i]',
        // Generic patterns (German)
        'button[id*="zustimmen" i]',
        'button[class*="zustimmen" i]',
        'button[id*="akzeptieren" i]',
        'button[class*="akzeptieren" i]',
        'button[id*="alle-akzeptieren" i]',
        // Data attributes
        '[data-accept-all]',
        '[data-consent="accept"]',
        // Last resort: any button with accept-like text in a dialog
        '[role="dialog"] button',
    ];

    for (const sel of selectors) {
        try {
            const btns = document.querySelectorAll(sel);
            for (const btn of btns) {
                const rect = btn.getBoundingClientRect();
                if (rect.width > 0 && rect.height > 0) {
                    if (sel === '[role="dialog"] button') {
                        const text = (btn.textContent || '').toLowerCase().trim();
                        const isAccept = text.includes('accept') || text.includes('allow') ||
                            text.includes('agree') || text.includes('akzeptieren') ||
                            text.includes('zustimmen') || text.includes('alle erlauben') ||
                            text.includes('alle akzeptieren');
                        if (!isAccept) continue;
                    }
                    btn.click();
                    return true;
                }
            }
        } catch(e) {}
    }
    return false;
})()
