
(function() {
    window.__ams_observers = true;
    window.__ams_tbt_available = false;
    window.__ams_cls_available = false;

    // LCP observer — the latest candidate is the LCP. Candidates stop with
    // the first user input, which a lab run never fires, so the value is the
    // last candidate before the read (Lighthouse: before the trace ends).
    // There is deliberately no load-event cut-off: a hero rendered after
    // load *is* the largest contentful paint (#770).
    window.__ams_lcp = window.__ams_lcp || 0;
    try {
        var _lcpObs = new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                window.__ams_lcp = entries[i].startTime;
            }
        });
        _lcpObs.observe({ type: 'largest-contentful-paint', buffered: true });
        window.__ams_lcp_obs = _lcpObs;
    } catch(e) {}

    // Long tasks, raw. TTI and TBT are computed from this list in Rust over
    // the reference window FCP→TTI (#770).
    window.__ams_long_tasks = window.__ams_long_tasks || [];
    window.__ams_long_task_seen = window.__ams_long_task_seen || new Set();
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                if (window.__ams_long_task_seen.has(e.startTime)) continue;
                window.__ams_long_task_seen.add(e.startTime);
                window.__ams_long_tasks.push({ start: e.startTime, duration: e.duration });
            }
        }).observe({ type: 'longtask', buffered: true });
        window.__ams_tbt_available = PerformanceObserver.supportedEntryTypes.includes('longtask');
    } catch(e) {}

    // CLS — accumulate total and capture per-shift attribution (#135)
    window.__ams_cls = window.__ams_cls || 0;
    window.__ams_cls_shifts = window.__ams_cls_shifts || [];
    window.__ams_cls_seen = window.__ams_cls_seen || new Set();
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                if (window.__ams_cls_seen.has(e.startTime)) continue;
                window.__ams_cls_seen.add(e.startTime);
                if (!e.hadRecentInput) {
                    window.__ams_cls += e.value;
                    var sources = [];
                    try {
                        if (e.sources) {
                            for (var j = 0; j < e.sources.length; j++) {
                                var s = e.sources[j];
                                var nodeDesc = 'unknown';
                                try {
                                    if (s.node) {
                                        nodeDesc = s.node.nodeName;
                                        if (s.node.id) nodeDesc += '#' + s.node.id;
                                        else if (s.node.className && typeof s.node.className === 'string') {
                                            var cls = s.node.className.trim().split(/\s+/)[0];
                                            if (cls) nodeDesc += '.' + cls;
                                        }
                                    }
                                } catch(_) {}
                                sources.push({
                                    node: nodeDesc,
                                    previousRect: s.previousRect ? {top: s.previousRect.top, left: s.previousRect.left, bottom: s.previousRect.bottom, right: s.previousRect.right} : null,
                                    currentRect: s.currentRect ? {top: s.currentRect.top, left: s.currentRect.left, bottom: s.currentRect.bottom, right: s.currentRect.right} : null
                                });
                            }
                        }
                    } catch(_) {}
                    window.__ams_cls_shifts.push({value: e.value, startTime: e.startTime, sources: sources});
                }
            }
        }).observe({ type: 'layout-shift', buffered: true });
        window.__ams_cls_available = PerformanceObserver.supportedEntryTypes.includes('layout-shift');
    } catch(e) {}

    // INP via event-timing (#134) — captures real interaction durations
    window.__ams_inp = window.__ams_inp || 0;
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                if (e.duration > window.__ams_inp) {
                    window.__ams_inp = e.duration;
                }
            }
        }).observe({ type: 'event', durationThreshold: 16, buffered: true });
    } catch(e) {}
    // Also observe first-input for environments that don't support event-timing
    try {
        new PerformanceObserver(function(list) {
            var entries = list.getEntries();
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                var delay = e.processingStart - e.startTime;
                if (delay > window.__ams_inp) window.__ams_inp = delay;
            }
        }).observe({ type: 'first-input', buffered: true });
    } catch(e) {}
})();
