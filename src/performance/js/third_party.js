
    (() => {
        var resources = performance.getEntriesByType('resource');
        return JSON.stringify(resources.map(function(r) {
            return {
                url: r.name,
                transferSize: r.transferSize || 0,
                initiatorType: r.initiatorType || 'other'
            };
        }));
    })()
    