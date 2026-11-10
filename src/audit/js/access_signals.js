(() => {
    const nav = performance.getEntriesByType('navigation')[0];
    const resources = [];
    for (const el of document.querySelectorAll('script[src], iframe[src], link[href]')) {
        resources.push(el.getAttribute('src') || el.getAttribute('href') || '');
    }
    for (const entry of performance.getEntriesByType('resource')) resources.push(entry.name);
    return {
        status: nav && Number.isFinite(nav.responseStatus) && nav.responseStatus > 0 ? nav.responseStatus : null,
        title: document.title || '',
        elements: document.getElementsByTagName('*').length,
        resources: resources.slice(0, 500),
    };
})()