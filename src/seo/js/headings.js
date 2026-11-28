
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
        // A heading inside a dialog or a consent layer belongs to that layer,
        // not to the page's content (#1132): `<dialog>`, `role="dialog"`,
        // `role="alertdialog"`, `aria-modal="true"`, or the root of a known
        // consent manager. The walk leaves a shadow root through its host.
        const consentLayer = /onetrust|cookiebot|usercentrics|didomi|borlabs|complianz|cookieconsent|truste|cmp-?(banner|container|wrapper)/i;
        const inDialogLayer = (node) => {
            let el = node;
            while (el) {
                if (el.nodeType === 1) {
                    const role = (el.getAttribute('role') || '').toLowerCase();
                    const names = `${el.id || ''} ${typeof el.className === 'string' ? el.className : ''}`;
                    if (el.tagName === 'DIALOG' || role === 'dialog' || role === 'alertdialog'
                        || el.getAttribute('aria-modal') === 'true' || consentLayer.test(names)) {
                        return true;
                    }
                }
                el = el.parentElement || (el.getRootNode && el.getRootNode().host) || null;
            }
            return false;
        };
        const countWords = (s) => {
            const t = s.trim();
            return t.length ? t.split(/\s+/).length : 0;
        };
        const headingText = (h, skipAriaHidden = false) => {
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
                        if (skipAriaHidden && child.getAttribute('aria-hidden') === 'true') continue;
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
        const rawExtractedText = (h) => {
            let out = '';
            const walk = (node) => {
                for (const child of node.childNodes) {
                    if (child.nodeType === 3) {
                        out += child.nodeValue;
                    } else if (child.nodeType === 1) {
                        const tag = child.tagName;
                        if (tag === 'SCRIPT' || tag === 'STYLE') continue;
                        walk(child);
                    }
                }
            };
            walk(h);
            return out.replace(/\s+/g, ' ').trim();
        };
        const resolveAriaLabel = (h) => {
            const label = (h.getAttribute('aria-label') || '').trim();
            if (label) return label.replace(/\s+/g, ' ');
            const labelledBy = (h.getAttribute('aria-labelledby') || '').trim();
            if (labelledBy) {
                const root = h.getRootNode();
                const parts = labelledBy.split(/\s+/).map(id => {
                    const target = root && typeof root.getElementById === 'function'
                        ? root.getElementById(id)
                        : document.getElementById(id);
                    return target ? (target.textContent || '').trim() : '';
                }).filter(Boolean);
                if (parts.length) return parts.join(' ').replace(/\s+/g, ' ');
            }
            return '';
        };
        const singleCharSpanStats = (h, visibleText) => {
            const totalChars = visibleText.replace(/\s+/g, '').length;
            if (totalChars === 0) return { count: 0, ratio: 0 };
            let count = 0;
            for (const el of h.querySelectorAll('span, i, b, em, strong, div')) {
                if (el.children.length > 0) continue;
                const t = (el.textContent || '').trim();
                if (Array.from(t).length === 1) count += 1;
            }
            return { count, ratio: Math.min(1, count / totalChars) };
        };
        const headings = headingEls.map((h, i) => {
            const level = parseInt(h.tagName.charAt(1));
            const text = headingText(h, false);
            const extracted_text = rawExtractedText(h);
            const ariaLabel = resolveAriaLabel(h);
            const nonAriaHiddenText = headingText(h, true);
            const accessible_name = ariaLabel || nonAriaHiddenText;
            const has_aria_label_only = Boolean(ariaLabel) && nonAriaHiddenText.length === 0 && text.length > 0;
            const spanStats = singleCharSpanStats(h, text);
            const single_char_span_count = spanStats.count;
            const single_char_span_ratio = Math.round(spanStats.ratio * 1000) / 1000;
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

            let in_dialog = false;
            try {
                in_dialog = inDialogLayer(h);
            } catch (e) {
                in_dialog = false;
            }

            return {
                level,
                text,
                accessible_name,
                extracted_text,
                has_aria_label_only,
                single_char_span_count,
                single_char_span_ratio,
                length: text.length,
                is_question,
                in_faq_context,
                word_count_after,
                hidden,
                in_dialog,
            };
        });
        return JSON.stringify(headings);
    })()
    