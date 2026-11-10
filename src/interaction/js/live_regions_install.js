
(function () {
    if (window.__ams_live_log) { return true; }
    window.__ams_live_log = [];
    var lastText = new WeakMap();

    function roleOf(el) {
        var r = (el.getAttribute('role') || '').toLowerCase();
        return (r === 'alert' || r === 'status' || r === 'log') ? r : null;
    }
    function isLive(el) {
        if (!el || el.nodeType !== 1) return false;
        if (el.hasAttribute('aria-live')) return true;
        return roleOf(el) !== null;
    }
    function politenessOf(el) {
        var declared = (el.getAttribute('aria-live') || '').toLowerCase();
        if (declared) return declared;
        var r = roleOf(el);
        if (r === 'alert') return 'assertive';
        return 'polite';
    }
    // Aufwärts bis zur nächsten Live-Region, über Shadow-Grenzen hinweg.
    function regionFor(node) {
        var el = (node && node.nodeType === 1) ? node : (node ? node.parentElement : null);
        var guard = 0;
        while (el && guard++ < 200) {
            if (isLive(el)) return el;
            if (el.parentElement) { el = el.parentElement; continue; }
            var root = el.getRootNode && el.getRootNode();
            el = (root && root.host) ? root.host : null;
        }
        return null;
    }

    var observer = new MutationObserver(function (records) {
        for (var i = 0; i < records.length; i++) {
            var region = regionFor(records[i].target);
            if (!region) continue;
            var text = (region.textContent || '').trim().slice(0, 300);
            if (lastText.get(region) === text) continue;
            lastText.set(region, text);
            window.__ams_live_log.push({
                at: Math.round((window.performance && performance.now()) || 0),
                politeness: politenessOf(region),
                role: roleOf(region),
                text: text
            });
            if (window.__ams_live_log.length > 200) window.__ams_live_log.shift();
        }
    });
    var opts = { subtree: true, childList: true, characterData: true, attributes: false };

    function observeRoot(root) {
        try { observer.observe(root, opts); } catch (e) { /* abgelöste Wurzel */ }
    }
    observeRoot(document.documentElement);

    // Vorhandene Shadow Roots: der Observer durchdringt sie nicht von selbst.
    (function walk(root, depth) {
        if (depth > 12) return;
        var all = root.querySelectorAll('*');
        for (var i = 0; i < all.length; i++) {
            if (all[i].shadowRoot) { observeRoot(all[i].shadowRoot); walk(all[i].shadowRoot, depth + 1); }
        }
    })(document, 0);

    // Später erzeugte Shadow Roots.
    var origAttach = Element.prototype.attachShadow;
    Element.prototype.attachShadow = function () {
        var root = origAttach.apply(this, arguments);
        observeRoot(root);
        return root;
    };
    return true;
})()
