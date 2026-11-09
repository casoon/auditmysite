
(function() {
    window.__ams_spa_nav_count = 0;
    var orig_push = history.pushState.bind(history);
    var orig_replace = history.replaceState.bind(history);
    history.pushState = function() {
        window.__ams_spa_nav_count++;
        return orig_push.apply(this, arguments);
    };
    history.replaceState = function() {
        window.__ams_spa_nav_count++;
        return orig_replace.apply(this, arguments);
    };
    window.addEventListener('popstate', function() {
        window.__ams_spa_nav_count++;
    });
    true;
})()
