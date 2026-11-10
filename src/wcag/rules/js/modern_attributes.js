
        const issues = [];
        const isVisible = el => {
          // The element's own window: frame content has its own styles.
          const style = el.ownerDocument.defaultView.getComputedStyle(el);
          const rect = el.getBoundingClientRect();
          return style.display !== 'none' &&
            style.visibility !== 'hidden' &&
            rect.width > 0 &&
            rect.height > 0;
        };
        // Every element matching `selector` in the document, in all open
        // shadow roots below it and in same-origin frames: a popover or menu
        // inside a web component or an embedded frame is just as open, inert
        // or unnamed as one in the light DOM (#783). A frame's elements carry
        // the frames.rs selector shape "{iframe} [frame] {element}".
        const deepQuery = selector => {
          const found = [];
          const walk = (root, frameSel) => {
            root.querySelectorAll(selector).forEach(el => found.push({ el, frameSel }));
            root.querySelectorAll('*').forEach(el => {
              if (el.shadowRoot) walk(el.shadowRoot, frameSel);
              if (el.tagName !== 'IFRAME' && el.tagName !== 'FRAME') return;
              let doc = null;
              try { doc = el.contentDocument; } catch (e) {}
              if (!doc || !doc.documentElement) return;
              const sel = __amsCssSelector(el);
              walk(doc, frameSel ? frameSel + ' [frame] ' + sel : sel);
            });
          };
          walk(document, null);
          return found;
        };
        const accessibleName = el => {
          const labelledby = el.getAttribute('aria-labelledby');
          if (labelledby) {
            // IDREFs resolve within the element's own tree (document or
            // shadow root).
            const root = el.getRootNode();
            const text = labelledby
              .split(/\s+/)
              .map(id => root.getElementById(id)?.textContent || '')
              .join(' ')
              .trim();
            if (text) return text;
          }
          return (el.getAttribute('aria-label') || el.getAttribute('title') || '').trim();
        };
        const push = (el, frameSel, issue_type, message, attr, value, outcome) => {
          const sel = __amsCssSelector(el);
          issues.push({
            selector: frameSel ? frameSel + ' [frame] ' + sel : sel,
            issue_type,
            outcome,
            message,
            attr,
            value,
            snippet: el.outerHTML.substring(0, 200)
          });
        };

        deepQuery('[popover], [role="menu"]').forEach(({ el, frameSel }) => {
          const isOpenPopover = el.matches('[popover]:popover-open');
          const isMenu = el.getAttribute('role') === 'menu' && isVisible(el);
          // Ein Dialog -- auch als Popover -- ist `dialog/name-missing` im
          // geteilten Bestand (#692).
          const isDialog = el.matches('dialog, [role="dialog"], [role="alertdialog"]');
          // WAI-ARIA 1.2 requires no name for `menu`, HTML none for a
          // popover: whether the open surface is identifiable without one is a
          // judgement call, never a confirmed violation.
          if ((isOpenPopover || isMenu) && !isDialog && !accessibleName(el)) {
            push(el, frameSel, 'interactive_surface_missing_name', 'Open menu or popover has no accessible name', 'aria-label', null, 'review');
          }
          // Der inerte Dialog ist `inert/dialog-inert`, das sichtbare Menue
          // mit `inert` bewusst kein Befund mehr (#691); das offene Popover
          // deckt die geteilte Regel nicht ab.
          if (isOpenPopover && el.hasAttribute('inert')) {
            push(el, frameSel, 'active_surface_inert', 'Open popover is marked inert', 'inert', '', 'fail');
          }
        });

        return issues;
        