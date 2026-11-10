
(function() {
    var origin = window.location.origin;
    var currentHref = window.location.href;
    var links = Array.from(document.querySelectorAll('a[href]'));
    var candidates = [];
    links.forEach(function(a) {
        var href = a.href;
        if (!href.startsWith(origin)) return;         // external
        if (href === currentHref) return;              // same page
        if (a.getAttribute('href').startsWith('#')) return; // hash-only
        if (a.download) return;                        // download
        if (a.target === '_blank') return;             // new tab
        // Only path-different links (ignore query/hash for classification).
        var aPath = new URL(href).pathname;
        var curPath = new URL(currentHref).pathname;
        if (aPath === curPath) return;                 // same path, different query
        candidates.push({ selector: cssPath(a), href: href });
        if (candidates.length >= 5) return;
    });
    function cssPath(el) {
        if (el.id) return '#' + el.id;
        var parts = [];
        var n = el;
        while (n && n.nodeType === 1 && parts.length < 5) {
            var tag = n.nodeName.toLowerCase();
            if (n.id) { parts.unshift(tag + '#' + n.id); break; }
            parts.unshift(tag);
            n = n.parentNode;
        }
        return parts.join(' > ');
    }
    return candidates;
})()
