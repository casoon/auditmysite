
    (() => {
        var resources = performance.getEntriesByType('resource');
        return JSON.stringify(resources.map(function(r) {
            return {
                url: r.name,
                initiatorType: r.initiatorType || 'other',
                decodedBodySize: r.decodedBodySize || 0,
                transferSize: r.transferSize || 0
            };
        }));
    })()
    