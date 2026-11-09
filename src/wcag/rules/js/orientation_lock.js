
(function() {
  function rotatedBy(style) {
    var t = style.transform;
    if (t && t !== 'none') {
      var m = t.match(/^matrix(3d)?\(([^)]*)\)$/);
      if (m) {
        var v = m[2].split(',').map(Number);
        var b = v[1];
        var c = m[1] ? v[4] : v[2];
        if (Math.abs(b) > 0.01 && Math.abs(b + c) < 0.01) {
          return Math.round(Math.atan2(b, v[0]) * 180 / Math.PI) + 'deg';
        }
      }
    }
    var r = style.rotate;
    if (r && r !== 'none') {
      var deg = parseFloat(r);
      if (!isNaN(deg) && Math.abs(deg % 180) > 0.5) return r;
    }
    return null;
  }
  var pairs = [['html', document.documentElement], ['body', document.body]];
  for (var i = 0; i < pairs.length; i++) {
    if (!pairs[i][1]) continue;
    var angle = rotatedBy(window.getComputedStyle(pairs[i][1]));
    if (angle) {
      return { locked: true, element: pairs[i][0], detail: pairs[i][0] + ' is rotated by ' + angle + ' in this viewport' };
    }
  }
  return { locked: false, element: '', detail: '' };
})()
