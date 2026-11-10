
    (() => {
        const result = { jsonLd: [], microdata: false, rdfa: false };

        // Return raw script contents. Rust owns parsing and normalization so
        // top-level arrays, @graph and parse failures share one testable path.
        document.querySelectorAll('script[type="application/ld+json"]').forEach(script => {
            result.jsonLd.push(script.textContent || '');
        });

        // Check for microdata
        result.microdata = document.querySelectorAll('[itemscope]').length > 0;

        // Check for RDFa
        result.rdfa = document.querySelectorAll('[typeof], [property]').length > 0;

        return JSON.stringify(result);
    })()
    