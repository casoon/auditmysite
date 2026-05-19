//! Cookie consent banner detection and dismissal.
//!
//! Two-strategy approach:
//! 1. Cookie injection: set known CMP consent cookies before navigation (preferred)
//! 2. JS dismiss: click accept buttons after load (fallback)
//! 3. Detection only: warn in report if banner is still visible after audit

use chromiumoxide::Page;
use tracing::{debug, info};

pub struct ConsentResult {
    pub banner_detected: bool,
    /// Which CMP was identified ("cookiebot", "onetrust", "usercentrics", etc.) or None
    pub cmp_name: Option<String>,
    /// Whether we successfully dismissed the banner
    pub dismissed: bool,
}

/// Inject consent cookies for all known CMPs for the target URL.
/// Call this BEFORE the first page.goto().
/// Uses CDP Network.setCookies directly so no prior navigation is needed.
pub async fn inject_consent_cookies(page: &Page, url: &str) {
    use chromiumoxide::cdp::browser_protocol::network::{CookieParam, SetCookiesParams};

    let scheme_host = extract_scheme_host(url);
    if scheme_host.is_empty() {
        return;
    }

    // Cookie value for Cookiebot
    let cookiebot_value = "{\"stamp\":\"\",\"necessary\":true,\"preferences\":true,\"statistics\":true,\"marketing\":true,\"ver\":1}";

    // OneTrust consent groups
    let onetrust_consent = "groups=1%3A1%2C2%3A1%2C3%3A1%2C4%3A1&isGpcEnabled=0&isIABGlobal=false&datestamp=Mon+Jan+01+2024+00%3A00%3A00+GMT%2B0000";
    let today = "2024-01-01T00:00:00.000Z";

    let make = |name: &str, value: &str| -> CookieParam {
        let mut c = CookieParam::new(name.to_string(), value.to_string());
        c.url = Some(scheme_host.clone());
        c.path = Some("/".to_string());
        c
    };

    let cookies: Vec<CookieParam> = vec![
        // Cookiebot
        make("CookieConsent", cookiebot_value),
        // OneTrust
        make("OptanonAlertBoxClosed", today),
        make("OptanonConsent", onetrust_consent),
        // Usercentrics
        make("uc_user_interaction", "true"),
        // Consentmanager
        make("cmp_a", "1"),
        // Borlabs Cookie (WordPress)
        make(
            "borlabs-cookie",
            "%7B%22consents%22%3A%7B%22essential%22%3Atrue%2C%22marketing%22%3Atrue%2C%22statistics%22%3Atrue%7D%7D",
        ),
        // Klaro
        make("klaro", "e30K"),
        // Generic EU consent cookie
        make("euconsent-v2", "accepted"),
        // Cookie Script
        make(
            "CookieScriptConsent",
            "{\"action\":\"accept\",\"categories\":[\"functionality\",\"performance\",\"targeting\"]}",
        ),
        // Deutsche Bahn (bahn.de) — custom CMP
        make("db_cc_al", "1"),
        make("db_cc_nx", "1"),
        // Sourcepoint (used by many German publishers)
        make("_sp_v1_consent", "1%3A1%7C2%3A1%7C3%3A1%7C4%3A1"),
    ];

    let count = cookies.len();
    if page.execute(SetCookiesParams::new(cookies)).await.is_ok() {
        debug!("Injected {} consent cookies for: {}", count, scheme_host);
    } else {
        debug!("Consent cookie injection failed (continuing without)");
    }
}

/// After page load: detect visible consent banner and optionally dismiss it via JS click.
/// Returns a ConsentResult describing what was found and done.
pub async fn handle_post_navigation(page: &Page, try_dismiss: bool) -> ConsentResult {
    let detected = detect_consent_banner(page).await;
    if !detected {
        return ConsentResult {
            banner_detected: false,
            cmp_name: None,
            dismissed: false,
        };
    }

    let cmp_name = identify_cmp(page).await;
    info!(
        "Consent banner detected (CMP: {})",
        cmp_name.as_deref().unwrap_or("unknown")
    );

    if !try_dismiss {
        return ConsentResult {
            banner_detected: true,
            cmp_name,
            dismissed: false,
        };
    }

    let dismissed = dismiss_banner_js(page).await;
    if dismissed {
        // Allow 600ms for page to react (animation, DOM updates)
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        info!("Consent banner dismissed via JS click");
    } else {
        debug!("JS dismiss attempted but no button found");
    }

    ConsentResult {
        banner_detected: true,
        cmp_name,
        dismissed,
    }
}

/// Detect if a consent banner/dialog is currently visible on the page.
async fn detect_consent_banner(page: &Page) -> bool {
    let js = r#"
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
                return true;
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
                if (rect.width > 0 && rect.height > 0) return true;
            }
        } catch(e) {}
    }
    return false;
})()
"#;

    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.value().and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

/// Identify which CMP is present.
async fn identify_cmp(page: &Page) -> Option<String> {
    let js = r#"
(function() {
    if (document.querySelector('#CybotCookiebotDialog,#CybotCookiebotDialogBody')) return 'cookiebot';
    if (document.querySelector('#onetrust-banner-sdk,#onetrust-consent-sdk')) return 'onetrust';
    if (document.querySelector('#usercentrics-root')) return 'usercentrics';
    if (document.querySelector('#klaro,.klaro')) return 'klaro';
    if (document.querySelector('#cmp-app-container')) return 'consentmanager';
    if (document.querySelector('.cc-window,.cc-banner')) return 'cookieconsent';
    if (document.querySelector('[id*="borlabs"]')) return 'borlabs';
    if (document.querySelector('#cookie-law-info-bar,[id*="cli_"]')) return 'cookie-law-info';
    return null;
})()
"#;

    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
}

/// Try to dismiss the consent banner by clicking the accept button.
/// Returns true if a button was found and clicked.
async fn dismiss_banner_js(page: &Page) -> bool {
    let js = r#"
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
"#;

    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.value().and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

fn extract_scheme_host(url: &str) -> String {
    // Returns e.g. "https://example.com" for use as cookie URL
    let after_scheme = if let Some(rest) = url.strip_prefix("https://") {
        format!("https://{}", rest.split('/').next().unwrap_or(""))
    } else if let Some(rest) = url.strip_prefix("http://") {
        format!("http://{}", rest.split('/').next().unwrap_or(""))
    } else {
        return String::new();
    };
    after_scheme
}
