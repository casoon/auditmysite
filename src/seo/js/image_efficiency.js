
    (() => {
        const imgs = Array.from(document.querySelectorAll('img'));
        return JSON.stringify(imgs.map(img => ({
            src: (img.currentSrc || img.src || '').substring(0, 120),
            naturalWidth: img.naturalWidth || 0,
            naturalHeight: img.naturalHeight || 0,
            displayWidth: img.width || img.clientWidth || 0,
            displayHeight: img.height || img.clientHeight || 0,
        })));
    })()
    