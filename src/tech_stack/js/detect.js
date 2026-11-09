
(function() {
  const result = {};
  const metaGenerator = (document.querySelector('meta[name="generator"]') || {}).content || '';
  const scripts = Array.from(document.querySelectorAll('script[src]')).map(s => s.src);
  const links   = Array.from(document.querySelectorAll('link[href]')).map(l => l.href);
  const allSrc  = scripts.concat(links);

  // WordPress
  const wpMeta   = /WordPress\s*([\d.]+)?/i.exec(metaGenerator);
  const wpSrc    = allSrc.some(s => /\/wp-content\/|\/wp-includes\//.test(s));
  const wpGlobal = typeof window.wp !== 'undefined';
  const wooSrc   = allSrc.some(s => /woocommerce/.test(s));
  const wooClass = !!document.querySelector('.woocommerce, .wc-block');
  if (wpMeta || wpSrc || wpGlobal) {
    result.wordpress = {
      version: wpMeta ? (wpMeta[1] || null) : null,
      signals: [wpMeta ? 'meta generator' : null, wpSrc ? 'wp-content/wp-includes URLs' : null, wpGlobal ? 'window.wp' : null].filter(Boolean),
    };
    if (wooSrc || wooClass) {
      result.woocommerce = { signals: [wooSrc ? 'woocommerce script URLs' : null, wooClass ? '.woocommerce CSS class' : null].filter(Boolean) };
    }
  }

  // Drupal
  const drupalMeta = /Drupal\s*([\d.]+)?/i.exec(metaGenerator);
  const drupalGlob = typeof window.Drupal !== 'undefined';
  const drupalSrc  = allSrc.some(s => /\/sites\/default\/|\/modules\/|drupal/.test(s));
  if (drupalMeta || drupalGlob || drupalSrc) {
    result.drupal = {
      version: drupalMeta ? (drupalMeta[1] || null) : null,
      signals: [drupalMeta ? 'meta generator' : null, drupalGlob ? 'window.Drupal' : null, drupalSrc ? 'Drupal asset URLs' : null].filter(Boolean),
    };
  }

  // Joomla
  const joomlaMeta = /Joomla[!]?\s*([\d.]+)?/i.exec(metaGenerator);
  const joomlaSrc  = allSrc.some(s => /\/media\/jui\/|\/media\/system\//.test(s));
  if (joomlaMeta || joomlaSrc) {
    result.joomla = {
      version: joomlaMeta ? (joomlaMeta[1] || null) : null,
      signals: [joomlaMeta ? 'meta generator' : null, joomlaSrc ? 'Joomla media URLs' : null].filter(Boolean),
    };
  }

  // Next.js
  const nextData  = typeof window.__NEXT_DATA__ !== 'undefined';
  const nextSrc   = scripts.some(s => /\/_next\//.test(s));
  if (nextData || nextSrc) {
    const buildId = (window.__NEXT_DATA__ || {}).buildId || null;
    result.nextjs = { buildId, signals: [nextData ? 'window.__NEXT_DATA__' : null, nextSrc ? '_next script URLs' : null].filter(Boolean) };
  }

  // Nuxt.js
  const nuxtData = typeof window.__NUXT__ !== 'undefined';
  const nuxtSrc  = scripts.some(s => /\/_nuxt\//.test(s));
  if (nuxtData || nuxtSrc) {
    result.nuxt = { signals: [nuxtData ? 'window.__NUXT__' : null, nuxtSrc ? '_nuxt script URLs' : null].filter(Boolean) };
  }

  // Astro
  const astroMeta = /Astro\s*v?([\d.]+)?/i.exec(metaGenerator);
  const astroSrc  = allSrc.some(s => /\/_astro\//.test(s));
  if (astroMeta || astroSrc) {
    result.astro = {
      version: astroMeta ? (astroMeta[1] || null) : null,
      signals: [astroMeta ? 'meta generator' : null, astroSrc ? '_astro asset URLs' : null].filter(Boolean),
    };
  }

  // Gatsby
  const gatsbyGlob = typeof window.___gatsby !== 'undefined';
  const gatsbySrc  = scripts.some(s => /gatsby/.test(s));
  if (gatsbyGlob || gatsbySrc) {
    result.gatsby = { signals: [gatsbyGlob ? 'window.___gatsby' : null, gatsbySrc ? 'gatsby script URLs' : null].filter(Boolean) };
  }

  // Shopify
  const shopifyGlob = typeof window.Shopify !== 'undefined';
  const shopifyMeta = !!document.querySelector('meta[name="shopify-checkout-api-token"]');
  if (shopifyGlob || shopifyMeta) {
    result.shopify = { signals: [shopifyGlob ? 'window.Shopify' : null, shopifyMeta ? 'shopify meta tag' : null].filter(Boolean) };
  }

  // React (standalone — not via Next/Gatsby which already capture it)
  const reactHook = typeof window.__REACT_DEVTOOLS_GLOBAL_HOOK__ !== 'undefined';
  const reactRoot = !!document.querySelector('[data-reactroot]');
  if (!result.nextjs && !result.gatsby && (reactHook || reactRoot)) {
    result.react = { signals: [reactHook ? '__REACT_DEVTOOLS_GLOBAL_HOOK__' : null, reactRoot ? 'data-reactroot attribute' : null].filter(Boolean) };
  }

  // Vue.js
  const vueHook  = typeof window.__vue_devtools_global_hook__ !== 'undefined';
  const vueAttr  = !!document.querySelector('[data-v-app]');
  const vue3glob = typeof window.__VUE__ !== 'undefined';
  if (!result.nuxt && (vueHook || vueAttr || vue3glob)) {
    result.vue = { signals: [vue3glob ? 'window.__VUE__' : null, vueAttr ? 'data-v-app attribute' : null, vueHook ? 'Vue devtools hook' : null].filter(Boolean) };
  }

  // Angular
  const ngVersion = document.querySelector('[ng-version]');
  if (ngVersion) {
    result.angular = { version: ngVersion.getAttribute('ng-version'), signals: ['ng-version attribute'] };
  }

  // Svelte
  const svelteClass = document.querySelector('[class*="svelte-"]');
  if (svelteClass) {
    result.svelte = { signals: ['svelte-* CSS classes'] };
  }

  return result;
})()
