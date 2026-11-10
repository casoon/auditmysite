
  var findings = [];
  var MAX = 20;
  var html = document.documentElement;
  var mode = (html.getAttribute('data-display') || '').trim().toLowerCase();
  var figs = Array.prototype.slice.call(document.querySelectorAll('figure[data-viz]'));
  var hasToggle = !!document.querySelector('[data-display-toggle]');

  function rendered(el) {
    if (el.checkVisibility && !el.checkVisibility({ checkOpacity: true, checkVisibilityCSS: true })) {
      return false;
    }
    var r = el.getBoundingClientRect();
    return r.width > 1 && r.height > 1;
  }
  function hiddenFromAt(el) {
    for (var n = el; n && n.nodeType === 1; n = n.parentElement) {
      if (n.hasAttribute('hidden')) return 'hidden';
      if ((n.getAttribute('aria-hidden') || '').trim().toLowerCase() === 'true') return 'aria-hidden';
      if (n.hasAttribute('inert')) return 'inert';
    }
    if (el.checkVisibility && !el.checkVisibility({ checkVisibilityCSS: true })) return 'css';
    return null;
  }
  // A text layer that a rendered, reachable element of the same figure
  // (the figure itself included) references by aria-describedby or
  // aria-details (#704).
  function describedIn(fig, el) {
    if (!el.id) return false;
    var refs = [fig].concat(Array.prototype.slice.call(fig.querySelectorAll('[aria-describedby], [aria-details]')));
    for (var i = 0; i < refs.length; i++) {
      var r = refs[i];
      if (r === el || el.contains(r)) continue;
      var ids = ((r.getAttribute('aria-describedby') || '') + ' ' + (r.getAttribute('aria-details') || '')).split(/\s+/);
      if (ids.indexOf(el.id) >= 0 && rendered(r) && !hiddenFromAt(r)) return true;
    }
    return false;
  }
  // Capped through __amsPush so excluded specimens never spend the budget (#645).
  function push(id, el, detail) {
    __amsPush(findings, el, { id: id, selector: el === html ? 'html' : __amsCssSelector(el), detail: detail || '' }, MAX);
  }

  if (figs.length || hasToggle) {
    if (!html.hasAttribute('data-display')) {
      push('init', html, 'never');
    } else if (window.__amsDisplayObserved === true && window.__amsDisplayAtBody == null) {
      push('init', html, 'late');
    }
  }

  for (var i = 0; i < figs.length && __amsReal(findings) < MAX; i++) {
    var fig = figs[i];
    var texts = Array.prototype.slice.call(fig.querySelectorAll('[data-viz-text]'));
    var hiddenReported = false;
    for (var t = 0; t < texts.length; t++) {
      var reason = hiddenFromAt(texts[t]);
      // A figure that is itself not rendered hides its visual together with
      // its text layer — by CSS (closed <details>, collapsed tab) or by
      // `hidden`/aria-hidden/inert on the figure or an ancestor (an inactive
      // scope panel, #725). Nobody gets the statement in another form, so
      // this is not a convention breach.
      var hiddenWithFig = !rendered(fig) && (reason === 'css' || hiddenFromAt(fig) !== null);
      if (reason && !hiddenWithFig) {
        // accname 1.2 computes a description from a directly referenced
        // node even when it is hidden — by CSS, `hidden` or aria-hidden
        // alike (ARIA15 is a sufficient technique for 1.1.1). The statement
        // then still reaches assistive technology, flattened into a
        // description, so this is a note, not a breach. `inert` is outside
        // accname's notion of hidden; HTML lets user agents drop inert
        // content from the accessibility tree altogether, so it stays one.
        if (!texts[t].closest('[inert]') && describedIn(fig, texts[t])) {
          push('described', texts[t], reason);
        } else {
          push('hidden', texts[t], reason);
          hiddenReported = true;
        }
      }
    }
    if (mode !== 'text') continue;

    var textOk = texts.some(function (el) {
      return el.textContent.trim().length > 0 && rendered(el) && !hiddenFromAt(el);
    });
    // No text layer, or only empty ones, is `viz/text-missing` (any mode,
    // from the HTML). Here: there is a statement, but text mode hides it.
    var hasStatement = texts.some(function (el) { return el.textContent.trim().length > 0; });
    if (!textOk && !hiddenReported && hasStatement) push('text', fig, 'invisible');

    var media = fig.querySelectorAll('[data-viz-live], [data-viz-static], canvas, video, svg, img, iframe, object, embed');
    for (var m = 0; m < media.length; m++) {
      var el = media[m];
      var textLayer = el.closest('[data-viz-text]');
      if (textLayer && fig.contains(textLayer)) continue;
      if (!rendered(el)) continue;
      var r = el.getBoundingClientRect();
      // Icons stay allowed; the draft leaves the exact boundary open, 32x32
      // is the common icon ceiling.
      if (r.width * r.height <= 32 * 32) continue;
      push('media', el, el.tagName.toLowerCase());
      break;
    }
  }
  return findings;
