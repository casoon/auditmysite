
(function() {
  var PROBE = 'amsPasteProbe7!';
  var AUTH_FIELD = 'input[type="password"], input[autocomplete~="one-time-code"], input[autocomplete~="current-password"]';

  function isVisible(el) {
    var r = el.getBoundingClientRect();
    if (r.width === 0 && r.height === 0) return false;
    var view = el.ownerDocument === document ? window : el.ownerDocument.defaultView;
    var s = view === window ? getComputedStyle(el) : view.getComputedStyle(el);
    return s.display !== 'none' && s.visibility !== 'hidden';
  }

  /*CSS_SELECTOR*/

  function kindOf(el) {
    var ac = (el.getAttribute('autocomplete') || '').toLowerCase();
    return ac.indexOf('one-time-code') !== -1 ? 'one-time code' : 'password';
  }

  function pasteBlocked(el) {
    // The event comes from the field's own window, so a frame's handlers
    // see an event of their realm.
    var win = el.ownerDocument.defaultView || window;
    var original = el.value;
    var dt;
    try {
      dt = new win.DataTransfer();
      dt.setData('text/plain', PROBE);
    } catch (e) {
      dt = null;
    }
    var ev;
    try {
      ev = new win.ClipboardEvent('paste', { bubbles: true, cancelable: true, clipboardData: dt });
    } catch (e) {
      ev = new win.Event('paste', { bubbles: true, cancelable: true });
    }
    var notCancelled = el.dispatchEvent(ev);
    var inserted = (el.value || '').indexOf(PROBE) !== -1;
    el.value = original;
    return !notCancelled && !inserted;
  }

  // Auth fields of the document, of open shadow roots and of same-origin
  // frames: a sign-in form inside a web component or an embedded frame blocks
  // pasting just the same (#783). A frame's fields carry the frames.rs
  // selector shape "{iframe} [frame] {element}".
  var fields = [];
  var windows = [window];
  function collect(root, frameSel) {
    var found = root.querySelectorAll(AUTH_FIELD);
    for (var i = 0; i < found.length; i++) fields.push({ el: found[i], frame: frameSel });
    var all = root.querySelectorAll('*');
    for (var j = 0; j < all.length; j++) {
      var el = all[j];
      if (el.shadowRoot) collect(el.shadowRoot, frameSel);
      if (el.tagName !== 'IFRAME' && el.tagName !== 'FRAME') continue;
      var doc = null;
      try { doc = el.contentDocument; } catch (e) {}
      if (!doc || !doc.documentElement) continue;
      if (doc.defaultView) windows.push(doc.defaultView);
      var sel = __amsCssSelector(el);
      collect(doc, frameSel ? frameSel + ' [frame] ' + sel : sel);
    }
  }
  collect(document, null);

  var saved = windows.map(function(w) {
    var s = { w: w, alert: w.alert, confirm: w.confirm, prompt: w.prompt };
    w.alert = function() {};
    w.confirm = function() { return false; };
    w.prompt = function() { return null; };
    return s;
  });

  var blocked = [];
  try {
    // Probe and report caps count real and excluded (#645) fields separately.
    var probed = [];
    for (var i = 0; i < fields.length && __amsReal(probed) < 20; i++) {
      var f = fields[i].el;
      if (f.disabled || f.readOnly || !isVisible(f)) continue;
      if (!__amsPush(probed, f, f, 20)) continue;
      if (!pasteBlocked(f)) continue;
      var selector = __amsCssSelector(f);
      if (fields[i].frame) selector = fields[i].frame + ' [frame] ' + selector;
      __amsPush(blocked, f, { selector: selector, kind: kindOf(f) }, /*MAX_FINDINGS*/);
    }
  } finally {
    saved.forEach(function(s) {
      s.w.alert = s.alert;
      s.w.confirm = s.confirm;
      s.w.prompt = s.prompt;
    });
  }

  return { blocked: blocked };
})()
