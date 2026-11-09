
        var results = [];
        // Frames in open shadow roots are part of the page as much as those
        // in the light DOM; `querySelectorAll` does not reach into them.
        var frames = [];
        (function collect(root) {
          Array.prototype.push.apply(frames, root.querySelectorAll('iframe, frame'));
          var all = root.querySelectorAll('*');
          for (var j = 0; j < all.length; j++) {
            if (all[j].shadowRoot) collect(all[j].shadowRoot);
          }
        })(document);
        for (var i = 0; i < frames.length; i++) {
          var el = frames[i];
          var role = (el.getAttribute('role') || '').toLowerCase();
          if (role === 'none' || role === 'presentation') continue;
          if (el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true') continue;
          var style = window.getComputedStyle(el);
          if (style && (style.display === 'none' || style.visibility === 'hidden' || style.visibility === 'collapse')) continue;
          var rect = el.getBoundingClientRect();
          if (rect.width <= 1 || rect.height <= 1) continue;

          var crossOrigin = false;
          try {
            crossOrigin = (el.contentDocument === null);
          } catch(e) {
            crossOrigin = true;
          }
          if (!crossOrigin) continue;

          results.push({
            selector: __amsCssSelector(el),
            snippet: el.outerHTML.substring(0, 200),
            src: el.src || ''
          });
        }
        return results;
        