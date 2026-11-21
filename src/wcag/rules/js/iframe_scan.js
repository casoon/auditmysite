
  // Frames inside open shadow roots belong to the page as well.
  var iframes = [];
  (function collect(root) {
    var all = root.querySelectorAll('*');
    for (var ai = 0; ai < all.length; ai++) {
      var el = all[ai];
      if (el.tagName === 'IFRAME' || el.tagName === 'FRAME') iframes.push(el);
      if (el.shadowRoot) collect(el.shadowRoot);
    }
  })(document);
  var results = [];

  // Mirrors `awaited_source` in src/audit/frames.rs: no `srcdoc`, and a `src`
  // that is not empty, about:blank or javascript:, or loading=lazy with a
  // data-*src* attribute a lazy-load script fills `src` from.
  function realSource(value) {
    var v = (value || '').trim().toLowerCase();
    return v !== '' && v !== 'about:blank' && v.indexOf('javascript:') !== 0;
  }
  function awaitsSource(frame) {
    if (frame.hasAttribute('srcdoc')) return false;
    if (realSource(frame.getAttribute('src'))) return true;
    if ((frame.getAttribute('loading') || '').trim().toLowerCase() !== 'lazy') return false;
    var attrs = frame.attributes;
    for (var ai = 0; ai < attrs.length; ai++) {
      var name = attrs[ai].name.toLowerCase();
      if (name.indexOf('data-') === 0 && name.indexOf('src') !== -1 && realSource(attrs[ai].value)) return true;
    }
    return false;
  }

  for (var fi = 0; fi < iframes.length; fi++) {
    var iframe = iframes[fi];
    var iframeRole = (iframe.getAttribute('role') || '').toLowerCase();
    if (iframeRole === 'none' || iframeRole === 'presentation') continue;
    if (iframe.hasAttribute('hidden') || iframe.getAttribute('aria-hidden') === 'true') continue;
    var iframeStyle = window.getComputedStyle(iframe);
    if (iframeStyle && (iframeStyle.display === 'none' ||
        iframeStyle.visibility === 'hidden' || iframeStyle.visibility === 'collapse')) continue;
    var iframeRect = iframe.getBoundingClientRect();
    if (iframeRect.width <= 1 || iframeRect.height <= 1) continue;

    var iframeDoc;
    try {
      iframeDoc = iframe.contentDocument;
      if (!iframeDoc || !iframeDoc.body) continue;
    } catch(e) {
      continue;
    }

    // A frame that has a source but still shows its initial blank document
    // never loaded: its empty placeholder is no document to judge (#1109).
    if (iframeDoc.URL === 'about:blank' && awaitsSource(iframe)) continue;

    var ifSel = __amsCssSelector(iframe);
    var ifSrc = iframe.getAttribute('src') || '';

    // 3.1.1 html-has-lang
    var iframeHtmlEl = iframeDoc.documentElement;
    if (iframeHtmlEl && !(iframeHtmlEl.getAttribute('lang') || '').trim()) {
      results.push({
        rule_id: 'html-has-lang', rule: '3.1.1', severity: 'medium',
        message: 'Iframe document is missing a lang attribute on the html element',
        iframe_selector: ifSel, iframe_src: ifSrc,
        selector: 'html', snippet: ''
      });
    }
  }

  return results;
