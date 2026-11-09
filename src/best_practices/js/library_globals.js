
    (() => {
        var out = [];
        try {
            if (typeof jQuery !== 'undefined' && jQuery.fn && jQuery.fn.jquery)
                out.push({name: 'jQuery', version: jQuery.fn.jquery, source: 'global'});
        } catch(e) {}
        try {
            if (typeof $ !== 'undefined' && $ && $.fn && $.fn.jquery)
                out.push({name: 'jQuery', version: $.fn.jquery, source: 'global'});
        } catch(e) {}
        try {
            if (typeof angular !== 'undefined' && angular.version && angular.version.full)
                out.push({name: 'AngularJS', version: angular.version.full, source: 'global'});
        } catch(e) {}
        try {
            if (typeof Handlebars !== 'undefined' && Handlebars.VERSION)
                out.push({name: 'Handlebars', version: Handlebars.VERSION, source: 'global'});
        } catch(e) {}
        try {
            // Both Lodash and Underscore expose `_` with `_.VERSION` and (in
            // modern versions) `_.chunk`. `_.runInContext` is Lodash-exclusive,
            // so use it as the distinguisher — otherwise a Lodash 4.x build was
            // mislabelled "Underscore 4.17.5" (Underscore is at 1.x).
            if (typeof _ !== 'undefined' && _.VERSION && typeof _.runInContext === 'function')
                out.push({name: 'Lodash', version: _.VERSION, source: 'global'});
        } catch(e) {}
        try {
            if (typeof _ !== 'undefined' && _.VERSION && typeof _.runInContext !== 'function')
                out.push({name: 'Underscore', version: _.VERSION, source: 'global'});
        } catch(e) {}
        try {
            if (typeof Prototype !== 'undefined' && Prototype.Version)
                out.push({name: 'Prototype', version: Prototype.Version, source: 'global'});
        } catch(e) {}
        try {
            if (typeof MooTools !== 'undefined' && MooTools.version)
                out.push({name: 'MooTools', version: MooTools.version, source: 'global'});
        } catch(e) {}
        try {
            if (typeof moment !== 'undefined' && moment.version)
                out.push({name: 'Moment.js', version: moment.version, source: 'global'});
        } catch(e) {}
        // Bootstrap: window.bootstrap (v5+) or data-bs-version attribute
        try {
            if (typeof bootstrap !== 'undefined' && bootstrap.Tooltip && bootstrap.Tooltip.VERSION)
                out.push({name: 'Bootstrap', version: bootstrap.Tooltip.VERSION, source: 'global'});
            else {
                var bsEl = document.querySelector('[data-bs-version]');
                if (bsEl) out.push({name: 'Bootstrap', version: bsEl.getAttribute('data-bs-version'), source: 'global'});
            }
        } catch(e) {}
        return JSON.stringify(out);
    })()
    