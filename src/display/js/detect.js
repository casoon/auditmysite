(function () {
  var KNOWN = ['visual', 'calm', 'text'];
  var html = document.documentElement;
  var kinds = {};
  var figs = document.querySelectorAll('figure[data-viz]');
  for (var i = 0; i < figs.length; i++) {
    var k = (figs[i].getAttribute('data-viz') || '').trim().toLowerCase() || 'unspecified';
    kinds[k] = (kinds[k] || 0) + 1;
  }
  var toggles = document.querySelectorAll('[data-display-toggle]');
  var offered = [];
  for (var t = 0; t < toggles.length; t++) {
    var cands = [toggles[t]].concat(Array.prototype.slice.call(
      toggles[t].querySelectorAll('option, [value], [data-display-value], [data-display]')));
    for (var c = 0; c < cands.length; c++) {
      var vals = [cands[c].getAttribute('value'), cands[c].getAttribute('data-display-value'),
                  cands[c].getAttribute('data-display')];
      for (var v = 0; v < vals.length; v++) {
        var val = (vals[v] || '').trim().toLowerCase();
        if (KNOWN.indexOf(val) >= 0 && offered.indexOf(val) < 0) offered.push(val);
      }
    }
  }
  return JSON.stringify({
    active: html ? html.getAttribute('data-display') : null,
    toggle: toggles.length > 0,
    offered: offered,
    kinds: kinds,
    observed: window.__amsDisplayObserved === true,
    atBody: window.__amsDisplayAtBody === undefined ? null : window.__amsDisplayAtBody,
    reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches
  });
})()