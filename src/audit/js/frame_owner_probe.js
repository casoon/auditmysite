
  var el = this;
  var role = (el.getAttribute('role') || '').toLowerCase();
  var hidden = role === 'none' || role === 'presentation'
    || el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true';
  var st = window.getComputedStyle(el);
  if (st && (st.display === 'none' || st.visibility === 'hidden' || st.visibility === 'collapse')) hidden = true;
  var r = el.getBoundingClientRect();
  if (r.width <= 1 || r.height <= 1) hidden = true;
  return { selector: __amsCssSelector(el), hidden: hidden };
