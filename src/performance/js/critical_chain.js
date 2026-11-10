
    (() => {
        var nav = performance.getEntriesByType('navigation')[0];
        var navStart = nav ? nav.startTime : 0;
        var resources = performance.getEntriesByType('resource');
        return JSON.stringify(resources.map(function(r) {
            return {
                url: r.name,
                initiatorType: r.initiatorType || 'other',
                startTime: r.startTime - navStart,
                responseEnd: r.responseEnd - navStart,
                transferSize: r.transferSize || 0,
                duration: r.duration
            };
        }));
    })()
    