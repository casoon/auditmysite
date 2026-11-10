
(() => {
  const langTags = new Set();
  const docLang = document.documentElement.getAttribute('lang');
  if (docLang) langTags.add(docLang);
  for (const el of document.querySelectorAll('[lang], [hreflang]')) {
    const lang = el.getAttribute('lang');
    const hreflang = el.getAttribute('hreflang');
    if (lang) langTags.add(lang);
    if (hreflang) langTags.add(hreflang);
  }
  const classNames = [];
  for (const el of document.querySelectorAll('[class]')) {
    const raw = typeof el.className === 'string'
      ? el.className
      : (el.className && el.className.baseVal) || '';
    if (raw) classNames.push(raw);
  }
  return { lang_tags: Array.from(langTags), class_names: classNames };
})()
