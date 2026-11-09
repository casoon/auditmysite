
    (() => {
        const result = {
            viewport: {},
            touchTargets: { total: 0, small: 0, crowded: 0 },
            fonts: { base: 16, smallest: 16, legibleCount: 0, totalCount: 0 },
            content: {}
        };

        // Viewport analysis
        const viewport = document.querySelector('meta[name="viewport"]');
        if (viewport) {
            const content = viewport.getAttribute('content') || '';
            result.viewport.content = content;
            result.viewport.hasDeviceWidth = content.includes('width=device-width');
            result.viewport.hasInitialScale = content.includes('initial-scale=1');
            result.viewport.isScalable = !content.includes('user-scalable=no') &&
                                          !content.includes('user-scalable=0');
        }

        // Touch targets analysis
        const interactiveElements = document.querySelectorAll('a, button, input, select, textarea, [onclick], [role="button"]');
        result.touchTargets.total = interactiveElements.length;
        const smallByContext = {};
        const smallTargetDetails = [];

        interactiveElements.forEach(el => {
            const rect = el.getBoundingClientRect();
            // Skip elements that are fully hidden (behind hamburger, display:none, etc.)
            if (rect.width === 0 && rect.height === 0) return;
            // Skip visually hidden elements (sr-only pattern: clipped to 1×1px)
            if (rect.width <= 1 && rect.height <= 1) return;
            if (rect.width < 44 || rect.height < 44) {
                result.touchTargets.small++;
                // Classify context of small target
                const tag = el.tagName.toLowerCase();
                const parent = el.closest('nav, footer, header, aside, .social, [class*="social"], [class*="lang"]');
                let ctx = 'other';
                if (parent) {
                    const pTag = parent.tagName.toLowerCase();
                    if (pTag === 'nav') ctx = 'navigation';
                    else if (pTag === 'footer') ctx = 'footer';
                    else if (pTag === 'header') ctx = 'header';
                    else if (pTag === 'aside') ctx = 'sidebar';
                    else ctx = 'social/utility';
                } else if (tag === 'input' || tag === 'select' || tag === 'textarea') {
                    ctx = 'form';
                } else if (tag === 'button' || el.getAttribute('role') === 'button') {
                    ctx = 'button';
                }
                smallByContext[ctx] = (smallByContext[ctx] || 0) + 1;
                // Capture selector for debugging (up to 10 samples)
                if (smallTargetDetails.length < 10) {
                    let sel = tag;
                    if (el.id) sel = '#' + el.id;
                    else if (el.className && typeof el.className === 'string') {
                        const first = el.className.trim().split(/\s+/)[0];
                        if (first) sel = tag + '.' + first;
                    }
                    smallTargetDetails.push({ selector: sel, width: Math.round(rect.width), height: Math.round(rect.height), context: ctx });
                }
            }
        });
        result.touchTargets.smallByContext = smallByContext;
        result.touchTargets.details = smallTargetDetails;

        // Font analysis
        const textElements = document.querySelectorAll('p, span, a, li, td, th, div, h1, h2, h3, h4, h5, h6');
        let smallestFont = 100;
        // The smallest size only counts text someone can read: an element
        // whose own text is just whitespace or zero-width characters (KaTeX's
        // `span.vlist-s` alignment helpers hold a lone U+200B at 1-2px, #740)
        // renders nothing to read.
        const readable = /[^\s\u200B\u200C\u200D\u2060\uFEFF]/;
        const hasOwnReadableText = el => Array.from(el.childNodes).some(n =>
            n.nodeType === Node.TEXT_NODE && readable.test(n.nodeValue));

        textElements.forEach(el => {
            const style = window.getComputedStyle(el);
            const fontSize = parseFloat(style.fontSize);
            if (fontSize > 0) {
                result.fonts.totalCount++;
                if (fontSize >= 12) {
                    result.fonts.legibleCount++;
                }
                if (fontSize < smallestFont && hasOwnReadableText(el)) {
                    smallestFont = fontSize;
                }
            }
        });

        result.fonts.smallest = smallestFont < 100 ? smallestFont : 16;
        result.fonts.base = parseFloat(window.getComputedStyle(document.body).fontSize) || 16;

        // Count interactive elements with small font (distinguishes real violations from decorative)
        let smallInteractiveCount = 0;
        document.querySelectorAll('a, button, label, input, select, textarea').forEach(el => {
            const rect = el.getBoundingClientRect();
            if (rect.width === 0 && rect.height === 0) return;
            const style = window.getComputedStyle(el);
            const fontSize = parseFloat(style.fontSize);
            if (fontSize > 0 && fontSize < 12) smallInteractiveCount++;
        });
        result.fonts.smallInteractiveCount = smallInteractiveCount;

        // Content sizing
        result.content.viewportWidth = window.innerWidth;
        result.content.documentWidth = document.documentElement.scrollWidth;
        result.content.hasHorizontalScroll = document.documentElement.scrollWidth > window.innerWidth;

        // Check for responsive images
        const images = document.querySelectorAll('img');
        let responsiveImages = 0;
        images.forEach(img => {
            if (img.srcset || img.sizes || window.getComputedStyle(img).maxWidth === '100%') {
                responsiveImages++;
            }
        });
        result.content.responsiveImages = responsiveImages;
        result.content.totalImages = images.length;

        // Check for media queries (approximate)
        let hasMediaQueries = false;
        for (const sheet of document.styleSheets) {
            try {
                for (const rule of sheet.cssRules) {
                    if (rule.type === CSSRule.MEDIA_RULE) {
                        hasMediaQueries = true;
                        break;
                    }
                }
            } catch (e) {}
            if (hasMediaQueries) break;
        }
        result.content.hasMediaQueries = hasMediaQueries;

        // Check for relative font units (em, rem, %, vw)
        let usesRelativeUnits = false;
        for (const sheet of document.styleSheets) {
            try {
                for (const rule of sheet.cssRules) {
                    if (rule.style && rule.style.fontSize) {
                        const fs = rule.style.fontSize;
                        if (fs.match(/\d+(em|rem|%|vw)/)) {
                            usesRelativeUnits = true;
                            break;
                        }
                    }
                }
            } catch (e) {}
            if (usesRelativeUnits) break;
        }
        result.fonts.usesRelativeUnits = usesRelativeUnits;

        return JSON.stringify(result);
    })()
    