
var videos = [];
Array.prototype.forEach.call(document.querySelectorAll('video'), function(v) {
  // A video that is not rendered (in a hidden panel, display:none) is not
  // presented in this state; 1.2.2 has nothing to judge there.
  var vstyle = window.getComputedStyle(v);
  if (v.getClientRects().length === 0 || (vstyle && vstyle.visibility !== 'visible')) return;
  var tracks = [];
  Array.prototype.forEach.call(v.querySelectorAll('track'), function(t) {
    tracks.push({
      kind: (t.getAttribute('kind') || '').toLowerCase(),
      src: t.src || '',
      srclang: t.getAttribute('srclang') || '',
      label: t.getAttribute('label') || ''
    });
  });
  videos.push({ selector: __amsCssSelector(v), tracks: tracks });
});
var embeds = [];
Array.prototype.forEach.call(document.querySelectorAll('iframe[src]'), function(f) {
  embeds.push({ selector: __amsCssSelector(f), src: f.src || '' });
});
return { videos: videos, embeds: embeds };
