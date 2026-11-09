
    (() => {
        var scripts = Array.from(document.querySelectorAll('script[src]'));
        return JSON.stringify(scripts.map(s => s.src.substring(0, 200)));
    })()
    