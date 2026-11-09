
(function() {
    var el = document.activeElement;
    if (!el || el === document.body || el === document.documentElement) return 'ambiguous';
    var cs = window.getComputedStyle(el);
    var outlineStyle = cs.outlineStyle;
    var outlineWidth = cs.outlineWidth;
    var boxShadow = cs.boxShadow;
    var hasOutline = outlineStyle !== 'none' && outlineWidth !== '0px';
    var hasBoxShadow = boxShadow !== 'none';
    if (hasOutline || hasBoxShadow) return 'detected';
    var borderWidth = cs.borderWidth;
    if (borderWidth !== '0px') return 'ambiguous';
    return 'not_detected';
})()
