
(function () {
    var el = document.activeElement;
    while (el && el.shadowRoot && el.shadowRoot.activeElement) {
        el = el.shadowRoot.activeElement;
    }
    if (!el || el === document.body || el === document.documentElement) return null;
    return el;
})()
