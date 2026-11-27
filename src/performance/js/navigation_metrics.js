
    (() => {
        const result = {};

        // Navigation Timing
        const nav = performance.getEntriesByType('navigation')[0];
        if (nav) {
            result.ttfb = nav.responseStart - nav.requestStart;
            result.domInteractive = nav.domInteractive;
            if (nav.loadEventEnd > 0) {
                result.loadTime = nav.loadEventEnd - nav.startTime;
            }
            if (nav.domContentLoadedEventEnd > 0) {
                result.domContentLoaded = nav.domContentLoadedEventEnd - nav.startTime;
            }
        }

        // Paint Timing
        const paints = performance.getEntriesByType('paint');
        for (const paint of paints) {
            if (paint.name === 'first-contentful-paint') {
                result.fcp = paint.startTime;
            }
        }

        // LCP (if available)
        const lcpEntries = performance.getEntriesByType('largest-contentful-paint');
        if (lcpEntries.length > 0) {
            result.lcp = lcpEntries[lcpEntries.length - 1].startTime;
        }

        // Nodes of this document only. The CDP `Nodes` metric also counts the
        // detached documents of earlier loads in the same tab, so it grows with
        // every reload (three runs read three times the page).
        const walker = document.createTreeWalker(document, NodeFilter.SHOW_ALL);
        let domNodes = 0;
        while (walker.nextNode()) domNodes++;
        result.domNodes = domNodes;
        // Elements only: the unit of Lighthouse's DOM-size audit (800 / 1,500).
        result.domElements = document.querySelectorAll('*').length;

        return JSON.stringify(result);
    })()
    