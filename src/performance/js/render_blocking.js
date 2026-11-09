
    (() => {
        const result = {};

        // Scripts in <head> without defer or async (render-blocking)
        result.blockingScriptUrls = Array.from(
            document.head ? document.head.querySelectorAll('script[src]') : []
        )
        .filter(s => !s.defer && !s.async && !(s.type || '').includes('module'))
        .map(s => s.src)
        .filter(Boolean);

        // CSS <link rel=stylesheet> in <head> for screen/all media (render-blocking)
        result.blockingCssUrls = Array.from(
            document.head ? document.head.querySelectorAll('link[rel="stylesheet"]') : []
        )
        .filter(l => {
            const m = (l.media || 'all').trim().toLowerCase();
            return m === '' || m === 'all' || m === 'screen';
        })
        .map(l => l.href)
        .filter(Boolean);

        // Resource timing: build url → transferSize map
        const timing = {};
        for (const r of performance.getEntriesByType('resource')) {
            timing[r.name] = r.transferSize || 0;
        }
        result.resourceTiming = timing;

        // All resource URLs and sizes for first/third party split
        result.allResources = Array.from(performance.getEntriesByType('resource')).map(r => ({
            url: r.name,
            transferSize: r.transferSize || 0,
        }));

        return JSON.stringify(result);
    })()
    