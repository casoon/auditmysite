
    (() => {
        // Headings in open shadow roots count too, in the composed tree's
        // order (a shadow root's content right after its host): the shared
        // `headings/*` rules read that tree, and search engines index
        // rendered shadow DOM content (#724). Closed roots stay out of reach.
        const headingEls = [];
        const collect = (root) => {
            for (const el of root.querySelectorAll('*')) {
                if (/^H[1-6]$/.test(el.tagName)) headingEls.push(el);
                if (el.shadowRoot) collect(el.shadowRoot);
            }
        };
        collect(document);
        const countWords = (s) => {
            const t = s.trim();
            return t.length ? t.split(/\s+/).length : 0;
        };
        const headingText = (h) => {
            // Lines of a split headline are block-level boxes or <br>: textContent
            // drops the boundary and glued the words together ("Webseiten &Webanwendungen").
            // Inline-block children (per-letter spans) stay glued.
            const blockish = new Set(['block', 'flex', 'grid', 'list-item', 'flow-root', 'table', 'table-row', 'table-cell']);
            let out = '';
            const walk = (node) => {
                for (const child of node.childNodes) {
                    if (child.nodeType === 3) {
                        out += child.nodeValue;
                    } else if (child.nodeType === 1) {
                        const tag = child.tagName;
                        if (tag === 'SCRIPT' || tag === 'STYLE') continue;
                        const brk = tag === 'BR' || blockish.has(getComputedStyle(child).display);
                        if (brk) out += ' ';
                        walk(child);
                        if (brk) out += ' ';
                    }
                }
            };
            walk(h);
            return out.replace(/\s+/g, ' ').trim();
        };
        const headings = headingEls.map((h, i) => {
            const level = parseInt(h.tagName.charAt(1));
            const text = headingText(h);
            const is_question = text.endsWith('?');
            const in_faq_context = !!(
                h.closest('[itemtype*="Question"]') ||
                h.closest('[itemtype*="FAQPage"]') ||
                h.closest('details') ||
                h.closest('.faq') ||
                h.closest('[class*="faq"]') ||
                h.closest('[id*="faq"]')
            );

            // Word count of the visible text between this heading and the next
            // heading (or end of the content area for the last heading).
            let word_count_after = 0;
            try {
                // A range cannot cross a shadow boundary: it ends at the next
                // heading of the same tree, else at the end of that tree.
                const range = document.createRange();
                range.setStartAfter(h);
                const root = h.getRootNode();
                const next = headingEls.slice(i + 1).find(n => n.getRootNode() === root);
                const container = root === document ? document.body : root;
                if (next) {
                    range.setEndBefore(next);
                } else if (container.lastChild) {
                    range.setEndAfter(container.lastChild);
                } else {
                    range.setEnd(container, 0);
                }
                const frag = range.cloneContents();
                const holder = document.createElement('div');
                holder.appendChild(frag);
                holder.querySelectorAll('script, style, noscript').forEach(e => e.remove());
                word_count_after = countWords(holder.textContent || '');
            } catch (e) {
                word_count_after = 0;
            }

            // Hidden from everyone: `hidden`/`inert`/`aria-hidden` ancestors or
            // not rendered (display:none, visibility:hidden).
            let hidden = false;
            try {
                hidden = !!h.closest('[hidden],[inert],[aria-hidden="true"]')
                    || (typeof h.checkVisibility === 'function'
                        && !h.checkVisibility({ visibilityProperty: true }));
            } catch (e) {
                hidden = false;
            }

            return { level, text, length: text.length, is_question, in_faq_context, word_count_after, hidden };
        });
        return JSON.stringify(headings);
    })()
    