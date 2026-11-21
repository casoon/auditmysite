new Promise(function (resolve) {
  var BUDGET_MS = __BUDGET_MS__;
  var MAX_FRAMES = 20;
  var DWELL_MS = 120;
  var started = Date.now();
  var deadline = started + BUDGET_MS;

  // Lazy frames in the page, including those inside open shadow roots.
  var lazy = [];
  (function collect(root) {
    var all = root.querySelectorAll('*');
    for (var i = 0; i < all.length; i++) {
      var el = all[i];
      if (el.tagName === 'IFRAME' && (el.getAttribute('loading') || '').trim().toLowerCase() === 'lazy') {
        lazy.push(el);
      }
      if (el.shadowRoot) collect(el.shadowRoot);
    }
  })(document);

  // Not loaded yet: the frame still shows its initial empty document. A frame
  // that committed to another origin throws here and counts as loaded.
  function isBlank(frame) {
    try {
      return frame.contentWindow.location.href === 'about:blank';
    } catch (e) {
      return false;
    }
  }

  // The frames the audit would skip as not perceivable are no use to wait for
  // (the browser does not load a lazy frame that is not rendered).
  function isHidden(frame) {
    var role = (frame.getAttribute('role') || '').toLowerCase();
    if (role === 'none' || role === 'presentation') return true;
    if (frame.hasAttribute('hidden') || frame.getAttribute('aria-hidden') === 'true') return true;
    var style = window.getComputedStyle(frame);
    if (style && (style.display === 'none' || style.visibility === 'hidden' || style.visibility === 'collapse')) return true;
    var rect = frame.getBoundingClientRect();
    return rect.width <= 1 || rect.height <= 1;
  }

  var waiting = lazy.filter(function (f) { return isBlank(f) && !isHidden(f); }).slice(0, MAX_FRAMES);
  if (waiting.length === 0) {
    resolve({ candidates: 0, loaded: 0, waited_ms: 0 });
    return;
  }

  var loadEvents = waiting.map(function () { return false; });
  waiting.forEach(function (frame, i) {
    frame.addEventListener('load', function () { loadEvents[i] = true; });
  });

  var scrollX = window.scrollX;
  var scrollY = window.scrollY;
  function settled(i) {
    return loadEvents[i] && !isBlank(waiting[i]);
  }
  function finish() {
    window.scrollTo({ left: scrollX, top: scrollY, behavior: 'instant' });
    var loaded = 0;
    for (var i = 0; i < waiting.length; i++) if (settled(i)) loaded++;
    resolve({ candidates: waiting.length, loaded: loaded, waited_ms: Date.now() - started });
  }

  // Bring every frame into view long enough for the browser (or a lazy-load
  // script) to start its load, then wait for the loads together, bounded.
  var index = 0;
  function visit() {
    if (index >= waiting.length || Date.now() >= deadline) {
      poll();
      return;
    }
    waiting[index++].scrollIntoView({ block: 'center', inline: 'nearest', behavior: 'instant' });
    setTimeout(visit, DWELL_MS);
  }
  function poll() {
    var all = true;
    for (var i = 0; i < waiting.length; i++) if (!settled(i)) all = false;
    if (all || Date.now() >= deadline) {
      finish();
      return;
    }
    setTimeout(poll, 50);
  }
  visit();
})
