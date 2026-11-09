
    (() => {
        const result = {};
        result.https = window.location.protocol === 'https:';

        // Canonical
        const canonical = document.querySelector('link[rel="canonical"]');
        result.canonical = canonical ? canonical.getAttribute('href') : null;

        // Language
        result.lang = document.documentElement.getAttribute('lang');

        // Robots meta
        const robots = document.querySelector('meta[name="robots"]');
        result.robots = robots ? robots.getAttribute('content') : null;

        // Hreflang
        result.hreflang = [];
        document.querySelectorAll('link[rel="alternate"][hreflang]').forEach(el => {
            result.hreflang.push({
                lang: el.getAttribute('hreflang'),
                url: el.getAttribute('href')
            });
        });
        result.hreflangHasXDefault = result.hreflang.some(h => h.lang === 'x-default');
        result.hreflangHasSelfReference = result.hreflang.some(h => {
            try {
                const norm = u => new URL(u, window.location.href).href.replace(/\/$/, '');
                return norm(h.url) === norm(window.location.href);
            } catch(e) { return false; }
        });

        // Pagination
        const relHref = rel => {
            const el = Array.from(document.querySelectorAll('link[rel][href]')).find(link => {
                const rels = (link.getAttribute('rel') || '').toLowerCase().split(/\s+/);
                return rels.includes(rel);
            });
            return el ? el.href : null;
        };
        result.paginationPrev = relHref('prev');
        result.paginationNext = relHref('next');
        result.paginationLinkCount = document.querySelectorAll(
            'nav[class*="pag" i], [class*="pagination" i], [aria-label*="pagination" i], [aria-label*="seiten" i]'
        ).length;
        result.paginationAnchorCount = Array.from(document.querySelectorAll('a[href]')).filter(a => {
            const text = (a.textContent || '').trim().toLowerCase();
            const label = (a.getAttribute('aria-label') || '').toLowerCase();
            const rel = (a.getAttribute('rel') || '').toLowerCase();
            const href = a.getAttribute('href') || '';
            return rel.includes('next') || rel.includes('prev') ||
                /^(next|prev|previous|weiter|zurück|vorherige|nächste|[0-9]+)$/.test(text) ||
                label.includes('next') || label.includes('previous') ||
                label.includes('weiter') || label.includes('zurück') ||
                /([?&](page|paged|p)=\d+|\/page\/\d+\/?)/i.test(href);
        }).length;

        // Word count (approximate)
        const text = document.body ? document.body.innerText : '';
        result.wordCount = text.split(/\s+/).filter(w => w.length > 0).length;
        // Excerpt from the main content where it is unambiguous and long
        // enough; header, navigation and consent texts otherwise make every
        // page look alike (plan 90).
        const mains = Array.from(document.querySelectorAll('main, [role="main"]'))
            .filter(m => !m.parentElement || !m.parentElement.closest('main, [role="main"]'));
        const mainText = mains.length === 1 ? (mains[0].innerText || '').trim() : '';
        const useMain = mainText.length >= 200;
        result.textExcerpt = (useMain ? mainText : text).slice(0, 4000);
        result.textExcerptSource = useMain ? 'main' : 'body';

        // Links
        const links = document.querySelectorAll('a[href]');
        let internal = 0, external = 0;
        const currentHost = window.location.host;
        let dofollow = 0, nofollow = 0;

        const internalTargets = [];
        const internalLinkCheckTargets = [];
        links.forEach(a => {
            try {
                const rel = (a.getAttribute('rel') || '').toLowerCase().split(/\s+/);
                const isNofollow = rel.includes('nofollow') || rel.includes('ugc') || rel.includes('sponsored');
                if (isNofollow) { nofollow++; } else { dofollow++; }

                const href = a.getAttribute('href');
                if (!href || /^(mailto|tel|javascript):/i.test(href)) return;
                if (href.startsWith('#')) {
                    internal++;
                    return;
                }
                const linkUrl = new URL(href, window.location.href);
                if (linkUrl.protocol !== 'http:' && linkUrl.protocol !== 'https:') return;
                const isInternal = linkUrl.host === currentHost;
                if (isInternal) {
                    internal++;
                    linkUrl.hash = '';
                    internalLinkCheckTargets.push(linkUrl.href);
                    internalTargets.push(linkUrl.pathname);
                } else {
                    external++;
                }
            } catch (e) {}
        });

        result.internalLinks = internal;
        result.externalLinks = external;
        result.dofollowLinks = dofollow;
        result.nofollowLinks = nofollow;
        result.internalLinkTargets = internalTargets.slice(0, 500);
        result.internalLinkCheckTargets = Array.from(new Set(internalLinkCheckTargets)).slice(0, 50);

        // Crawl budget: internal links with query parameters
        let internalWithQP = 0;
        links.forEach(a => {
            try {
                const href = a.getAttribute('href');
                if (!href || !href.includes('?')) return;
                if (href.startsWith('/')) { internalWithQP++; return; }
                if (href.startsWith('http') && new URL(href).host === currentHost) internalWithQP++;
            } catch(e) {}
        });
        result.internalLinksWithQueryParams = internalWithQP;

        result.stylesheetUrls = Array.from(
            document.querySelectorAll('link[rel=\"stylesheet\"][href]'),
            el => el.href
        );
        result.scriptUrls = Array.from(document.querySelectorAll('script[src]'), el => el.src);
        result.resourceUrls = performance
            .getEntriesByType('resource')
            .map(entry => entry.name)
            .filter(Boolean);
        result.mixedContentResources = [];
        if (window.location.protocol === 'https:') {
            const pushMixed = (url, type, blockable) => {
                try {
                    const resolved = new URL(url, window.location.href);
                    if (resolved.protocol === 'http:') {
                        result.mixedContentResources.push({
                            url: resolved.href,
                            type,
                            blockable
                        });
                    }
                } catch(e) {}
            };
            document.querySelectorAll('script[src]').forEach(el => pushMixed(el.getAttribute('src'), 'script', true));
            document.querySelectorAll('link[rel="stylesheet"][href]').forEach(el => pushMixed(el.getAttribute('href'), 'stylesheet', true));
            document.querySelectorAll('iframe[src], object[data], embed[src]').forEach(el => pushMixed(el.getAttribute('src') || el.getAttribute('data'), 'frame', true));
            document.querySelectorAll('img[src], source[src], video[src], audio[src]').forEach(el => pushMixed(el.getAttribute('src'), 'media', false));
            performance.getEntriesByType('resource').forEach(entry => {
                if (entry.name && entry.name.startsWith('http://')) {
                    const type = entry.initiatorType || 'resource';
                    pushMixed(entry.name, type, ['script', 'link', 'css', 'xmlhttprequest', 'fetch', 'iframe'].includes(type));
                }
            });
        }
        result.cookieNames = document.cookie
            .split(';')
            .map(part => part.trim().split('=')[0])
            .filter(Boolean);
        result.storageItems = [];
        const readStorageKeys = area => {
            try {
                const storage = window[area];
                if (!storage) return;
                for (let i = 0; i < storage.length; i++) {
                    const key = storage.key(i);
                    if (key) result.storageItems.push({ area, key });
                }
            } catch(e) {}
        };
        readStorageKeys('localStorage');
        readStorageKeys('sessionStorage');
        result.hasZarazGlobal = typeof window.zaraz !== 'undefined';

        const htmlEl = document.documentElement;
        const hasAmpAttr = htmlEl.hasAttribute('amp') ||
            Array.from(htmlEl.attributes).some(attr => attr.name === '\u26a1');
        const ampRuntime = !!document.querySelector('script[src^="https://cdn.ampproject.org/v0.js"], script[src="//cdn.ampproject.org/v0.js"]');
        const ampBoilerplate = !!document.querySelector('style[amp-boilerplate]');
        const ampCustomScripts = Array.from(document.querySelectorAll('script[custom-element], script[custom-template]'));
        const ampDetected = hasAmpAttr || ampRuntime || ampBoilerplate || ampCustomScripts.length > 0;
        const ampIssues = [];
        if (ampDetected) {
            if (!hasAmpAttr) ampIssues.push('missing_html_amp_attribute');
            if (!result.canonical) ampIssues.push('missing_canonical');
            if (!ampRuntime) ampIssues.push('missing_amp_runtime');
            if (!ampBoilerplate) ampIssues.push('missing_amp_boilerplate');
            document.querySelectorAll('script:not([type="application/ld+json"]):not([src^="https://cdn.ampproject.org/"]):not([src^="//cdn.ampproject.org/"])')
                .forEach(script => {
                    if (!script.hasAttribute('custom-element') && !script.hasAttribute('custom-template')) {
                        ampIssues.push('custom_script_not_allowed');
                    }
                });
        }
        result.amp = {
            detected: ampDetected,
            canonicalPresent: !!result.canonical,
            runtimeScriptPresent: ampRuntime,
            boilerplatePresent: ampBoilerplate,
            issues: Array.from(new Set(ampIssues))
        };

        // Social-preview image alt text (og:image:alt / twitter:image:alt)
        const ogImageEl = document.querySelector('meta[property="og:image"]');
        const ogImageAltEl = document.querySelector('meta[property="og:image:alt"]');
        result.hasOgImage = !!ogImageEl;
        result.hasOgImageAlt = !!(ogImageAltEl && (ogImageAltEl.getAttribute('content') || '').trim());
        const twitterImageEl = document.querySelector('meta[name="twitter:image"]');
        const twitterImageAltEl = document.querySelector('meta[name="twitter:image:alt"]');
        result.hasTwitterImage = !!twitterImageEl;
        result.hasTwitterImageAlt = !!(twitterImageAltEl && (twitterImageAltEl.getAttribute('content') || '').trim());

        // Favicon
        const faviconEl = document.querySelector(
            'link[rel="icon"], link[rel="shortcut icon"], link[rel="apple-touch-icon"]'
        );
        result.hasFavicon = !!faviconEl;

        const manifestEl = document.querySelector('link[rel~="manifest"][href]');
        result.webManifestUrl = manifestEl ? manifestEl.href : null;
        const themeColorEl = document.querySelector('meta[name="theme-color"]');
        result.themeColor = themeColorEl ? themeColorEl.getAttribute('content') : null;
        result.serviceWorkerSupported = 'serviceWorker' in navigator;
        result.serviceWorkerControlled = !!(navigator.serviceWorker && navigator.serviceWorker.controller);

        const sensitiveNamePattern = /(pass|pwd|credit|card|cc-|cc_|cvc|cvv|iban|account|konto|login|email|mail|tel|phone|token|secret)/i;
        const isSensitiveInput = input => {
            const type = (input.getAttribute('type') || 'text').toLowerCase();
            const name = [
                input.getAttribute('name'),
                input.getAttribute('id'),
                input.getAttribute('autocomplete'),
                input.getAttribute('placeholder'),
                input.getAttribute('aria-label')
            ].filter(Boolean).join(' ');
            return ['password', 'email', 'tel'].includes(type) ||
                ['cc-number', 'cc-csc', 'cc-exp', 'one-time-code'].includes((input.getAttribute('autocomplete') || '').toLowerCase()) ||
                sensitiveNamePattern.test(name);
        };
        result.formSecurity = {
            sensitiveForms: 0,
            insecureTransportForms: 0,
            getSensitiveForms: 0,
            passwordAutocompleteIssues: 0
        };
        document.querySelectorAll('form').forEach(form => {
            const inputs = Array.from(form.querySelectorAll('input, textarea, select'));
            const sensitiveInputs = inputs.filter(isSensitiveInput);
            const passwordInputs = inputs.filter(input => (input.getAttribute('type') || '').toLowerCase() === 'password');
            if (sensitiveInputs.length === 0 && passwordInputs.length === 0) return;

            result.formSecurity.sensitiveForms++;

            const method = (form.getAttribute('method') || 'get').toLowerCase();
            if (method === 'get') {
                result.formSecurity.getSensitiveForms++;
            }

            try {
                const action = form.getAttribute('action') || window.location.href;
                const target = new URL(action, window.location.href);
                if (target.protocol === 'http:') {
                    result.formSecurity.insecureTransportForms++;
                }
            } catch(e) {
                if (window.location.protocol === 'http:') {
                    result.formSecurity.insecureTransportForms++;
                }
            }

            passwordInputs.forEach(input => {
                const autocomplete = (input.getAttribute('autocomplete') || '').trim().toLowerCase();
                if (autocomplete === 'off' || (autocomplete && !['current-password', 'new-password', 'one-time-code'].includes(autocomplete))) {
                    result.formSecurity.passwordAutocompleteIssues++;
                }
            });
        });

        return JSON.stringify(result);
    })()
    