
(function () {
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
    var sel = 'a[href], button, input:not([type="hidden"]), select, textarea, ' +
              '[tabindex], [contenteditable=""], [contenteditable="true"], video[controls]';
    var els = Array.from(document.querySelectorAll(sel)).filter(function (el) {
        if (el.disabled) return false;
        if (typeof el.tabIndex === 'number' && el.tabIndex < 0) return false;
        return true;
    });
    return els.map(selectorFor).filter(function (s) { return s !== null; });
})()
