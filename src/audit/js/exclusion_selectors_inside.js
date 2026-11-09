(function(sels, excl) {
  function inside(el) {
    var cur = el;
    while (cur) {
      for (var i = 0; i < excl.length; i++) {
        try { if (cur.matches && cur.matches(excl[i])) return true; } catch (e) {}
      }
      if (cur.parentElement) { cur = cur.parentElement; continue; }
      var root = cur.getRootNode ? cur.getRootNode() : null;
      cur = (root && root.host) ? root.host : null;
    }
    return false;
  }
  return sels.map(function(s) {
    var list;
    try { list = document.querySelectorAll(s); } catch (e) { return false; }
    if (!list.length) return false;
    for (var i = 0; i < list.length; i++) { if (!inside(list[i])) return false; }
    return true;
  });
})(__SELECTORS__, __EXCLUDED__)