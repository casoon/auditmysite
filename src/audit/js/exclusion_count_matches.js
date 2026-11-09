(function(sels) {
  return sels.map(function(s) {
    try { return document.querySelectorAll(s).length; } catch (e) { return -1; }
  });
})(__SELECTORS__)