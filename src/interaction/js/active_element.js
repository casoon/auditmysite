
(function () {
    var el = document.activeElement;
    if (!el || el === document.body || el === document.documentElement) {
        return null;
    }
    var rect = el.getBoundingClientRect();
    var vw = window.innerWidth || document.documentElement.clientWidth;
    var vh = window.innerHeight || document.documentElement.clientHeight;
    function selectorFor(node) {
        if (!node || node.nodeType !== 1) return null;
        if (node.id) return '#' + node.id;
        var parts = [];
        while (node && node.nodeType === 1 && parts.length < 6) {
            var tag = node.nodeName.toLowerCase();
            if (node.id) { parts.unshift(tag + '#' + node.id); break; }
            var parent = node.parentNode;
            if (parent) {
                var siblings = Array.from(parent.children).filter(function (c) {
                    return c.nodeName === node.nodeName;
                });
                if (siblings.length > 1) {
                    var idx = siblings.indexOf(node) + 1;
                    tag += ':nth-of-type(' + idx + ')';
                }
            }
            parts.unshift(tag);
            node = parent;
        }
        return parts.join(' > ');
    }
    // Ancestor-chain checks: a focused element is "hidden" if anywhere on
    // the path to <html> there is aria-hidden="true" or an inert attribute.
    var ariaHiddenChain = false;
    var inertChain = false;
    var n = el;
    while (n && n.nodeType === 1) {
        if (n.getAttribute) {
            if (n.getAttribute('aria-hidden') === 'true') ariaHiddenChain = true;
            if (n.hasAttribute('inert')) inertChain = true;
        }
        n = n.parentNode;
    }
    var style = window.getComputedStyle(el);
    var hiddenByStyle =
        style.display === 'none' ||
        style.visibility === 'hidden' ||
        parseFloat(style.opacity) === 0;
    var inViewport = rect.right > 0 && rect.bottom > 0 && rect.left < vw && rect.top < vh;
    // Native <video controls> is keyboard-focusable as a whole (Chrome gives
    // it tabIndex 0 by default) — check it has an accessible name the same
    // way axe-core/HTML-AAM would: aria-label, aria-labelledby, then title.
    var mediaControlMissingName = false;
    if (el.tagName === 'VIDEO' && el.hasAttribute('controls')) {
        var name = (el.getAttribute('aria-label') || '').trim();
        if (!name) {
            var labelledBy = (el.getAttribute('aria-labelledby') || '').trim();
            if (labelledBy) {
                name = labelledBy.split(/\s+/).map(function (id) {
                    var ref = document.getElementById(id);
                    return ref ? ref.textContent.trim() : '';
                }).join(' ').trim();
            }
        }
        if (!name) {
            name = (el.getAttribute('title') || '').trim();
        }
        mediaControlMissingName = !name;
    }
    return {
        selector: selectorFor(el),
        x: rect.x, y: rect.y, w: rect.width, h: rect.height,
        ariaHiddenChain: ariaHiddenChain,
        inertChain: inertChain,
        hiddenByStyle: hiddenByStyle,
        inViewport: inViewport,
        mediaControlMissingName: mediaControlMissingName,
    };
})()
