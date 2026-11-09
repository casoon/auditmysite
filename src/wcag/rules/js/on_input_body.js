
  function accessibleName(el) {
    var label = el.getAttribute('aria-label');
    if (label && label.trim()) return label.trim();
    var labelledby = el.getAttribute('aria-labelledby');
    if (labelledby) {
      var ref = document.getElementById(labelledby.split(/\s+/)[0]);
      if (ref && ref.textContent.trim()) return ref.textContent.trim();
    }
    if (el.id) {
      var forLabel = document.querySelector('label[for="' + CSS.escape(el.id) + '"]');
      if (forLabel && forLabel.textContent.trim()) return forLabel.textContent.trim();
    }
    var parentLabel = el.closest('label');
    if (parentLabel && parentLabel.textContent.trim()) return parentLabel.textContent.trim();
    return (el.textContent || '').trim();
  }

  function isChangeControl(el) {
    if (el.tagName === 'SELECT') return true;
    var role = (el.getAttribute('role') || '').toLowerCase();
    return role === 'combobox' || role === 'listbox';
  }

  // The source of a global function the inline handler calls directly
  // (onchange="applyFilter(this.value)"), so the Rust side can see what the
  // handler does. The inline text itself is judged by the shared rule.
  function calledSource(src) {
    var call = /^\s*([A-Za-z_$][\w$]*)\s*\(/.exec(src);
    if (call && typeof window[call[1]] === 'function') {
      try { return Function.prototype.toString.call(window[call[1]]); } catch (e) {}
    }
    return null;
  }

  var buttonNames = [];
  var buttons = document.querySelectorAll('button, input[type="submit"], [role="button"]');
  for (var b = 0; b < buttons.length && buttonNames.length < CAP; b++) {
    buttonNames.push(accessibleName(buttons[b]).toLowerCase());
  }

  // Whether the control's own form has a submit control, whatever its name.
  function formHasSubmit(el) {
    var form = el.form || el.closest('form');
    if (!form) return false;
    var owned = form.elements || [];
    for (var k = 0; k < owned.length; k++) {
      var c = owned[k];
      if (c.disabled) continue;
      var t = (c.getAttribute('type') || '').toLowerCase();
      if ((c.tagName === 'BUTTON' && (t === '' || t === 'submit')) ||
          (c.tagName === 'INPUT' && (t === 'submit' || t === 'image'))) return true;
    }
    return false;
  }

  var controls = [];
  var nodes = document.querySelectorAll('select, [role="combobox"], [role="listbox"], input[type="radio"], [role="radio"]');
  for (var i = 0; i < nodes.length && __amsReal(controls) < CAP; i++) {
    var el = nodes[i];
    var handler = el.getAttribute('onchange');
    __amsPush(controls, el, {
      role: (el.getAttribute('role') || el.tagName.toLowerCase()),
      name: accessibleName(el).toLowerCase(),
      selector: __amsCssSelector(el),
      handler: handler,
      form_submit: formHasSubmit(el),
      called: handler !== null && isChangeControl(el) ? calledSource(handler) : null
    }, CAP);
  }

  return { button_names: buttonNames, controls: controls };
