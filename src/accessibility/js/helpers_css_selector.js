
function __amsIsExcludedEl(el) {
  try {
    return !!(el && typeof window.__amsIsExcluded === 'function' && window.__amsIsExcluded(el));
  } catch (e) { return false; }
}
var __amsCapState;
function __amsCapCounts(list) {
  // Lazy: some rules call the helpers above the spot this snippet is spliced in.
  if (!__amsCapState) __amsCapState = new WeakMap();
  var s = __amsCapState.get(list);
  if (!s) { s = { real: 0, excluded: 0 }; __amsCapState.set(list, s); }
  return s;
}
function __amsPush(list, el, item, limit) {
  var s = __amsCapCounts(list);
  if (__amsIsExcludedEl(el)) {
    if (s.excluded >= limit) return false;
    s.excluded++;
  } else {
    if (s.real >= limit) return false;
    s.real++;
  }
  list.push(item);
  return true;
}
function __amsReal(list) {
  return __amsCapCounts(list).real;
}
function __amsCssSelector(el) {
  if (!el || !el.tagName) return '';
  var esc = function(s) {
    return (window.CSS && CSS.escape) ? CSS.escape(s) : String(s).replace(/[^a-zA-Z0-9_-]/g, '\\$&');
  };
  var seg = function(node) {
    var tag = node.tagName.toLowerCase();
    if (node.id) return tag + '#' + esc(node.id);
    var cls = Array.prototype.slice.call(node.classList || [])
      .filter(function(c) { return c.length > 0 && c.length < 30; })
      .slice(0, 2)
      .map(function(c) { return '.' + esc(c); })
      .join('');
    var nth = '';
    var p = node.parentElement;
    if (p) {
      var sameTag = Array.prototype.slice.call(p.children)
        .filter(function(c) { return c.tagName === node.tagName; });
      if (sameTag.length > 1) {
        nth = ':nth-of-type(' + (sameTag.indexOf(node) + 1) + ')';
      }
    }
    return tag + cls + nth;
  };
  var parts = [];
  var cur = el;
  for (var i = 0; i < 5 && cur && cur.tagName && cur !== document.documentElement; i++) {
    parts.unshift(seg(cur));
    if (cur.id) break;
    cur = cur.parentElement;
  }
  return parts.join(' > ');
}
