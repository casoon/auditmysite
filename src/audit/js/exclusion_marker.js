(function(excl) {
  var roots = new WeakSet();
  excl.forEach(function(s) {
    try { document.querySelectorAll(s).forEach(function(el) { roots.add(el); }); } catch (e) {}
  });
  var isExcluded = function(el) {
    var cur = el;
    while (cur) {
      if (roots.has(cur)) return true;
      if (cur.parentElement) { cur = cur.parentElement; continue; }
      var root = cur.getRootNode ? cur.getRootNode() : null;
      cur = (root && root.host) ? root.host : null;
    }
    return false;
  };
  Object.defineProperty(window, '__amsIsExcluded', {
    value: isExcluded, configurable: true, writable: true, enumerable: false
  });
  return true;
})(__EXCLUDED__)