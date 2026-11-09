
(function() {
  // Rules inside `@media` blocks that match the current viewport, appended as
  // plain rules: the shared stylesheet rules know no viewport, so they would
  // miss a declaration that only applies at this width (#783). The blocks
  // themselves stay, for rules that read the condition (reduced motion).
  function matched(rules, out, inMatch) {
    for (var j = 0; j < rules.length; j++) {
      var r = rules[j];
      if (typeof CSSMediaRule !== 'undefined' && r instanceof CSSMediaRule) {
        var cond = r.conditionText || (r.media && r.media.mediaText) || '';
        if (window.matchMedia(cond).matches) matched(r.cssRules, out, true);
      } else if (inMatch) {
        out.push(r.cssText);
      }
    }
  }
  var out = [];
  for (var i = 0; i < document.styleSheets.length; i++) {
    var rules;
    try { rules = document.styleSheets[i].cssRules; } catch (e) { continue; }
    if (!rules) continue;
    var extra = [];
    matched(rules, extra, false);
    out.push(Array.from(rules).map(function (r) { return r.cssText; }).concat(extra).join('\n'));
  }
  return out;
})()
