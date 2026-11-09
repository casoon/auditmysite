
    (() => {
        const result = {};

        // Title
        result.title = document.title || null;

        // Meta tags
        const getMeta = (name) => {
            const el = document.querySelector(`meta[name="${name}"], meta[property="${name}"]`);
            return el ? el.getAttribute('content') : null;
        };

        result.description = getMeta('description');
        result.keywords = getMeta('keywords');
        result.robots = getMeta('robots');
        result.author = getMeta('author');
        result.viewport = getMeta('viewport');

        // Charset
        const charset = document.querySelector('meta[charset]');
        result.charset = charset ? charset.getAttribute('charset') : null;

        // Canonical
        const canonical = document.querySelector('link[rel="canonical"]');
        result.canonical = canonical ? canonical.getAttribute('href') : null;

        // Language
        result.lang = document.documentElement.getAttribute('lang');

        return JSON.stringify(result);
    })()
    