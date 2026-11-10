function() {
        function seg(el) {
            if (!el) return '';
            const tag = el.tagName.toLowerCase();
            if (el.id) return tag + '#' + el.id;
            const cls = (typeof el.className === 'string' && el.className.trim())
                ? '.' + el.className.trim().split(/\s+/)[0]
                : '';
            // Add :nth-of-type when there are multiple siblings with the same tag
            const p = el.parentElement;
            if (p) {
                const siblings = Array.from(p.children).filter(c => c.tagName === el.tagName);
                if (siblings.length > 1) {
                    const idx = siblings.indexOf(el) + 1;
                    return tag + cls + ':nth-of-type(' + idx + ')';
                }
            }
            return tag + cls;
        }
        const parts = [];
        let cur = this;
        for (let i = 0; i < 3 && cur && cur !== document.documentElement && cur !== document.body; i++) {
            parts.unshift(seg(cur));
            // Stop early if this segment is already unique (has id or unique class)
            if (cur.id) break;
            cur = cur.parentElement;
        }
        return parts.join(' > ');
    }