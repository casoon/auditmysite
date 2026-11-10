
    (() => {
        const resources = performance.getEntriesByType('resource');
        const nav = performance.getEntriesByType('navigation')[0];
        const navEntry = nav ? [{
            name: nav.name,
            type: 'navigation',
            transferSize: nav.transferSize || 0,
            decodedSize: nav.decodedBodySize || 0,
            duration: nav.duration
        }] : [];
        return JSON.stringify([...navEntry, ...resources.map(r => ({
            name: r.name,
            type: r.initiatorType,
            transferSize: r.transferSize || 0,
            decodedSize: r.decodedBodySize || 0,
            duration: r.duration
        }))]);
    })()
    