
    (() => {
        const result = { og: {}, twitter: {} };

        // OpenGraph tags
        const ogTags = ['title', 'description', 'image', 'url', 'type', 'site_name', 'locale'];
        ogTags.forEach(tag => {
            const el = document.querySelector(`meta[property="og:${tag}"]`);
            if (el) result.og[tag] = el.getAttribute('content');
        });

        // Twitter Card tags
        const twitterTags = ['card', 'title', 'description', 'image', 'site', 'creator'];
        twitterTags.forEach(tag => {
            const el = document.querySelector(`meta[name="twitter:${tag}"]`);
            if (el) result.twitter[tag] = el.getAttribute('content');
        });

        return JSON.stringify(result);
    })()
    