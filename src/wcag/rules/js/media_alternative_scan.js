
var keywordRe = new RegExp(__PATTERN_JSON__, 'i');
function ancestors(el, depth) {
  var list = [];
  var cur = el;
  for (var i = 0; i < depth && cur; i++) {
    list.push(cur);
    cur = cur.parentElement;
  }
  return list;
}
var mediaEls = Array.prototype.slice.call(document.querySelectorAll('video, iframe[src]'));
var mediaAncestors = mediaEls.map(function(el) { return ancestors(el, 4); });
var transcriptHref = null;
var transcriptText = null;
var links = Array.prototype.slice.call(document.querySelectorAll('a[href], [role="link"]'));
outer:
for (var i = 0; i < links.length; i++) {
  var link = links[i];
  var text = (link.textContent || '').trim();
  if (!text || !keywordRe.test(text)) continue;
  var linkAncestors = ancestors(link, 4);
  for (var m = 0; m < mediaAncestors.length; m++) {
    var shared = mediaAncestors[m].some(function(a) { return linkAncestors.indexOf(a) !== -1; });
    if (shared) {
      transcriptHref = link.getAttribute('href') || '';
      transcriptText = text.substring(0, 80);
      break outer;
    }
  }
}
return {
  hasVideo: document.querySelectorAll('video').length > 0,
  hasEmbed: document.querySelectorAll('iframe[src]').length > 0,
  transcriptHref: transcriptHref,
  transcriptText: transcriptText
};
