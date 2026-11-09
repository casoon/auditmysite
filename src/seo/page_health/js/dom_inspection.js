
    (() => {
        const r = {};
        const host = window.location.host;

        // Final URL (redirect detection)
        r.finalUrl = window.location.href;

        // Meta-refresh
        const mr = document.querySelector('meta[http-equiv="refresh"]');
        r.hasMetaRefresh = !!mr;
        r.metaRefreshContent = mr ? mr.getAttribute('content') : null;

        // Frames
        r.frameCount = document.querySelectorAll('frame, frameset').length;
        r.iframeCount = document.querySelectorAll('iframe').length;
        r.crossOriginIframeCount = Array.from(document.querySelectorAll('iframe[src]'))
            .filter(f => {
                try { return new URL(f.src).host !== host; } catch(e) { return false; }
            }).length;

        // Duplicate IDs
        const allIds = Array.from(document.querySelectorAll('[id]')).map(el => el.id);
        const idCounts = {};
        allIds.forEach(id => { idCounts[id] = (idCounts[id] || 0) + 1; });
        const dupIds = Object.entries(idCounts).filter(([_, c]) => c > 1);
        r.duplicateIdCount = dupIds.length;
        r.duplicateIdSamples = dupIds.slice(0, 5).map(([id]) => id);

        // Images without alt
        r.imagesWithoutAlt = document.querySelectorAll('img:not([alt])').length;

        // Tables without headers
        r.tablesWithoutHeaders = Array.from(document.querySelectorAll('table'))
            .filter(t => !t.querySelector('th') && !t.querySelector('caption')).length;

        // Empty headings
        r.emptyHeadings = document.querySelectorAll(
            'h1:empty,h2:empty,h3:empty,h4:empty,h5:empty,h6:empty'
        ).length;

        // Nested interactive elements
        r.nestedInteractive = document.querySelectorAll('button button, a a').length;

        // Doctype detection
        r.hasDoctype = !!(document.doctype && document.doctype.name === 'html');

        // document.write() usage in inline scripts
        const inlineScripts = Array.from(document.querySelectorAll('script:not([src])'));
        r.documentWriteCount = inlineScripts.filter(s => s.textContent.includes('document.write(')).length;

        // Images without explicit width+height (CLS risk)
        r.imagesWithoutDimensions = Array.from(document.querySelectorAll('img'))
            .filter(img => {
                const style = window.getComputedStyle(img);
                const pos = style.position;
                if (pos === 'absolute' || pos === 'fixed') return false;
                const hasAttrs = img.hasAttribute('width') && img.hasAttribute('height');
                const hasAspectRatio = style.aspectRatio && style.aspectRatio !== 'auto';
                return !hasAttrs && !hasAspectRatio;
            }).length;

        // Paste-blocking password fields (inline handler only)
        r.pasteBlockingPasswords = Array.from(document.querySelectorAll('input[type="password"]'))
            .filter(inp => {
                const attr = inp.getAttribute('onpaste') || '';
                return attr.includes('return false') || attr.includes('preventDefault');
            }).length;

        // DOM size
        r.domNodeCount = document.querySelectorAll('*').length;
        let domMaxDepth = 0;
        const domQueue = [[document.documentElement, 0]];
        while (domQueue.length && domQueue.length < 50000) {
            const [el, d] = domQueue.shift();
            if (d > domMaxDepth) domMaxDepth = d;
            const kids = el && el.children ? Array.from(el.children) : [];
            for (const child of kids) domQueue.push([child, d + 1]);
        }
        r.domMaxDepth = domMaxDepth;

        // Offscreen images without lazy loading
        const vh = window.innerHeight;
        r.offscreenWithoutLazy = Array.from(document.querySelectorAll('img'))
            .filter(img => {
                const rect = img.getBoundingClientRect();
                return rect.top > vh && img.getAttribute('loading') !== 'lazy';
            }).length;

        // Images without srcset (no responsive variants)
        r.imagesWithoutSrcset = Array.from(document.querySelectorAll('img'))
            .filter(img => {
                if (img.closest('picture')) return false;
                const src = img.getAttribute('src') || '';
                if (src.endsWith('.svg') || src.startsWith('data:image/svg')) return false;
                if (img.naturalWidth > 0 && img.naturalWidth < 80) return false;
                return !img.hasAttribute('srcset');
            }).length;

        // Missing preconnect hints for third-party origins
        const preconnected = new Set(
            Array.from(document.querySelectorAll('link[rel="preconnect"]'))
                .map(l => { try { return new URL(l.href).origin; } catch(e) { return null; } })
                .filter(Boolean)
        );
        const extOrigins = new Set();
        document.querySelectorAll('script[src],link[rel="stylesheet"][href],img[src],iframe[src]')
            .forEach(el => {
                const src = el.src || el.href;
                try {
                    const o = new URL(src).origin;
                    // data:/blob:/javascript: URLs have an opaque origin that
                    // serializes to the literal string "null" (WHATWG URL
                    // spec) -- not a real host, can't take a preconnect hint,
                    // and reads as a bug when printed in a report ("null" was
                    // confirmed leaking into säfte.com's preconnect-origins
                    // list, 2026-08-31).
                    if (o !== window.location.origin && o !== 'null') extOrigins.add(o);
                } catch(e) {}
            });
        const missingPreconnect = [...extOrigins].filter(o => !preconnected.has(o));
        r.missingPreconnectCount = missingPreconnect.length;
        r.missingPreconnectOrigins = missingPreconnect.slice(0, 5);

        // Non-crawlable links
        r.nonCrawlableLinks = Array.from(document.querySelectorAll('a')).filter(a => {
            const href = a.getAttribute('href');
            if (href === null) return a.hasAttribute('onclick');
            return href === '' || href === '#' || href.startsWith('javascript:');
        }).length;

        // Images without modern format (no WebP/AVIF in picture or srcset type)
        r.imagesWithoutModernFormat = Array.from(document.querySelectorAll('img')).filter(img => {
            const src = img.getAttribute('src') || '';
            if (!src.match(/\.(jpe?g|png)(\?|$)/i)) return false;
            const picture = img.closest('picture');
            if (picture && picture.querySelector('source[type="image/webp"], source[type="image/avif"]')) return false;
            const srcset = img.getAttribute('srcset') || '';
            if (srcset.match(/\.(webp|avif)/i)) return false;
            return true;
        }).length;

        // Oversized images (natural > 4x display area)
        r.oversizedImages = Array.from(document.querySelectorAll('img')).filter(img => {
            if (!img.naturalWidth || !img.clientWidth) return false;
            const naturalPixels = img.naturalWidth * img.naturalHeight;
            const displayPixels = img.clientWidth * img.clientHeight;
            return displayPixels > 0 && naturalPixels > displayPixels * 4;
        }).length;

        // GIF images
        r.gifImages = Array.from(document.querySelectorAll('img[src]'))
            .filter(img => (img.getAttribute('src') || '').match(/\.gif(\?|$)/i)).length;

        // Font-display issues in @font-face rules, and each rule's resolved
        // font file URL (for the preload-coverage check below, #533).
        let fontDisplayIssues = 0;
        const fontFaceUrls = new Set();
        for (const sheet of document.styleSheets) {
            try {
                for (const rule of sheet.cssRules) {
                    if (rule.type === CSSRule.FONT_FACE_RULE) {
                        const display = rule.style.getPropertyValue('font-display');
                        if (!display || display === 'block' || display === 'auto') fontDisplayIssues++;
                        const src = rule.style.getPropertyValue('src') || '';
                        const match = src.match(/url\(["']?([^"')]+)["']?\)/);
                        if (match) {
                            try { fontFaceUrls.add(new URL(match[1], document.baseURI).href); } catch(e) {}
                        }
                    }
                }
            } catch(e) {}
        }
        r.fontDisplayIssues = fontDisplayIssues;

        // Resource hints inventory
        let preloadCount = 0, prefetchCount = 0, dnsPrefetchCount = 0;
        const preloadHrefs = new Set();
        document.querySelectorAll('link[rel]').forEach(link => {
            const rel = link.getAttribute('rel');
            if (rel === 'preload') { preloadCount++; preloadHrefs.add(link.href); }
            else if (rel === 'prefetch') prefetchCount++;
            else if (rel === 'dns-prefetch') dnsPrefetchCount++;
        });
        r.preloadHints = preloadCount;
        r.prefetchHints = prefetchCount;
        r.dnsPrefetchHints = dnsPrefetchCount;
        // Orphaned preloads: preloaded but not in Resource Timing
        const loadedResources = new Set(performance.getEntriesByType('resource').map(e => e.name));
        r.orphanedPreloadCount = [...preloadHrefs].filter(href => href && !loadedResources.has(href)).length;

        // Web fonts with no matching <link rel="preload" as="font"> (#533)
        r.fontFaceCount = fontFaceUrls.size;
        r.fontsWithoutPreloadCount = [...fontFaceUrls].filter(u => !preloadHrefs.has(u)).length;

        const nav = performance.getEntriesByType('navigation')[0];
        r.documentDecodedBytes = nav ? Math.round(nav.decodedBodySize || 0) : 0;
        r.documentTransferBytes = nav ? Math.round(nav.transferSize || 0) : 0;
        r.cacheProbeUrls = Array.from(new Set(
            performance.getEntriesByType('resource')
                .filter(entry => {
                    const type = entry.initiatorType || '';
                    if (['script', 'link', 'css', 'img', 'font'].includes(type)) return true;
                    return /\.(js|css|mjs|woff2?|ttf|otf|png|jpe?g|gif|webp|avif|svg)(\?|$)/i.test(entry.name || '');
                })
                .map(entry => entry.name)
                .filter(Boolean)
        )).slice(0, 30);

        // LCP image candidate: largest visible img by display area
        let lcpImg = null, lcpArea = 0;
        document.querySelectorAll('img[src]').forEach(img => {
            const rect = img.getBoundingClientRect();
            if (rect.top < 0 || rect.top > window.innerHeight) return;
            const area = rect.width * rect.height;
            if (area > lcpArea) { lcpArea = area; lcpImg = img; }
        });
        if (lcpImg) {
            const lcpSrc = lcpImg.src;
            const hasPreload = [...preloadHrefs].some(h => h === lcpSrc);
            r.lcpImageWithoutPreload = !hasPreload;
            r.lcpImageUrl = lcpSrc;
            r.lcpImageWithoutFetchpriority = lcpImg.getAttribute('fetchpriority') !== 'high';
            r.lcpImageLazyLoaded = lcpImg.getAttribute('loading') === 'lazy';
        } else {
            r.lcpImageWithoutPreload = false;
            r.lcpImageWithoutFetchpriority = false;
            r.lcpImageLazyLoaded = false;
        }

        // Deprecated API detection in inline scripts
        const DEPRECATED = ['AppCache', 'document.domain =', 'webkitStorageInfo',
            'webkitIndexedDB', 'navigator.userAgentData', 'importScripts'];
        const inlineText = Array.from(document.querySelectorAll('script:not([src])')).map(s => s.textContent).join('\n');
        r.deprecatedApiCount = DEPRECATED.filter(p => inlineText.includes(p)).length;

        // Hreflang validation
        const hreflangLinks = Array.from(document.querySelectorAll('link[rel="alternate"][hreflang]'));
        r.hreflangCount = hreflangLinks.length;
        const langPattern = /^[a-z]{2,3}(-[A-Z]{2})?$|^x-default$/;
        r.hreflangInvalidCount = hreflangLinks.filter(l => !langPattern.test(l.getAttribute('hreflang') || '')).length;

        // JSON-LD validation
        const jsonldBlocks = Array.from(document.querySelectorAll('script[type="application/ld+json"]'));
        r.jsonldCount = jsonldBlocks.length;
        r.jsonldInvalidCount = jsonldBlocks.filter(s => {
            try {
                const data = JSON.parse(s.textContent);
                return !data['@context'] || !data['@type'];
            } catch(e) { return true; }
        }).length;

        // Render-blocking: sync <script src> in <head> without defer/async/type=module
        r.syncHeadScripts = Array.from(document.querySelectorAll('head script[src]'))
            .filter(s => {
                const t = (s.getAttribute('type') || '').toLowerCase();
                return !s.hasAttribute('async') && !s.hasAttribute('defer') && t !== 'module';
            }).length;

        // SRI: external scripts and stylesheets without integrity attribute
        const pageOrigin = window.location.origin;
        r.externalScriptsWithoutSri = Array.from(document.querySelectorAll('script[src]'))
            .filter(s => {
                try { return new URL(s.src).origin !== pageOrigin && !s.hasAttribute('integrity'); }
                catch(e) { return false; }
            }).length;
        r.externalStylesWithoutSri = Array.from(document.querySelectorAll('link[rel="stylesheet"][href]'))
            .filter(l => {
                try { return new URL(l.href).origin !== pageOrigin && !l.hasAttribute('integrity'); }
                catch(e) { return false; }
            }).length;

        // Fragment anchor validation: #anchor links where target ID does not exist on this page
        const pageIds = new Set(Array.from(document.querySelectorAll('[id]')).map(el => el.id));
        const brokenFragmentEls = Array.from(document.querySelectorAll('a')).filter(a => {
            const href = a.getAttribute('href') || '';
            if (href.charAt(0) !== '#') return false;
            const frag = href.slice(1);
            if (!frag) return false;
            try { return !pageIds.has(frag) && !pageIds.has(decodeURIComponent(frag)); }
            catch(e) { return !pageIds.has(frag); }
        });
        r.brokenFragmentLinks = brokenFragmentEls.length;
        r.brokenFragmentSamples = brokenFragmentEls.slice(0, 5).map(a => a.getAttribute('href'));

        // Generic link text: links with non-descriptive anchor text
        const GENERIC_TEXTS = new Set(['hier', 'mehr', 'weiter', 'klick', 'link', 'details', 'ansehen',
            'click here', 'read more', 'learn more', 'more', 'here']);
        const genericLinkEls = Array.from(document.querySelectorAll('a')).filter(a => {
            const text = (a.textContent || '').trim().toLowerCase();
            return text && GENERIC_TEXTS.has(text);
        });
        r.genericLinkTextCount = genericLinkEls.length;
        r.genericLinkTextSamples = genericLinkEls.slice(0, 5).map(a => a.getAttribute('href') || '(no href)');

        return JSON.stringify(r);
    })()
    