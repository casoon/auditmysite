var videos = [];
Array.prototype.forEach.call(document.querySelectorAll('video'), function(v) {
  // A video that is not rendered (in a hidden panel, display:none) is not
  // presented in this state; 1.2.2 has nothing to judge there.
  var vstyle = window.getComputedStyle(v);
  if (v.getClientRects().length === 0 || (vstyle && vstyle.visibility !== 'visible')) return;
  // Same applicability as the shared media/video-captions-missing: a muted
  // video nobody can unmute has no sound to caption, and a video under
  // aria-hidden="true" is left out there too.
  if (v.hasAttribute('muted') && !v.hasAttribute('controls')) return;
  if (v.closest('[aria-hidden="true"]')) return;
  // Only direct <track> children, as in the shared rule (and the HTML
  // content model of <video>).
  var tracks = [];
  Array.prototype.forEach.call(v.children, function(t) {
    if (t.localName !== 'track') return;
    tracks.push({
      kind: (t.getAttribute('kind') || '').trim().toLowerCase(),
      src: t.getAttribute('src') ? t.src : '',
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
