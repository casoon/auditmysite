
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
