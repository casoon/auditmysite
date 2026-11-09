
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
