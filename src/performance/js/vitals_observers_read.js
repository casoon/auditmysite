
(function() {
    if (window.__ams_observers !== true) return null;
    // Finalize LCP — flush and disconnect observer
    try {
        if (window.__ams_lcp_obs) {
            var recs = window.__ams_lcp_obs.takeRecords();
            for (var i = 0; i < recs.length; i++) {
                window.__ams_lcp = recs[i].startTime;
            }
            window.__ams_lcp_obs.disconnect();
        }
    } catch(e) {}

    return JSON.stringify({
        lcp: window.__ams_lcp || 0,
        longTasks: window.__ams_long_tasks || [],
        tbtAvailable: window.__ams_tbt_available === true,
        cls: window.__ams_cls || 0,
        clsAvailable: window.__ams_cls_available === true,
        clsShifts: window.__ams_cls_shifts || [],
        inpDuration: window.__ams_inp || 0
    });
})()
