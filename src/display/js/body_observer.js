(function () {
  if (window.__amsDisplayObserverInstalled) return;
  window.__amsDisplayObserverInstalled = true;
  function record() {
    window.__amsDisplayObserved = true;
    window.__amsDisplayAtBody = document.documentElement
      ? document.documentElement.getAttribute('data-display')
      : null;
  }
  if (document.body) { record(); return; }
  try {
    var obs = new MutationObserver(function () {
      if (document.body) { record(); obs.disconnect(); }
    });
    obs.observe(document, { childList: true, subtree: true });
  } catch (e) {}
})();