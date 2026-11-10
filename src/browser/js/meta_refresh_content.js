(() => {
  const m = document.querySelector('meta[http-equiv="refresh" i]');
  return m ? (m.getAttribute('content') || '') : null;
})()