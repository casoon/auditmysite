
    (function() {
        window.__auditConsoleErrors = [];
        var _origError = console.error;
        var _origWarn = console.warn;
        console.error = function() {
            var msg = Array.prototype.slice.call(arguments).map(function(a) {
                return typeof a === 'object' ? JSON.stringify(a) : String(a);
            }).join(' ');
            window.__auditConsoleErrors.push({level: 'error', msg: msg.substring(0, 200)});
            _origError.apply(console, arguments);
        };
        console.warn = function() {
            var msg = Array.prototype.slice.call(arguments).map(function(a) {
                return typeof a === 'object' ? JSON.stringify(a) : String(a);
            }).join(' ');
            window.__auditConsoleErrors.push({level: 'warn', msg: msg.substring(0, 200)});
            _origWarn.apply(console, arguments);
        };
        window.addEventListener('error', function(e) {
            var loc = e.filename ? (' at ' + e.filename + ':' + e.lineno) : '';
            window.__auditConsoleErrors.push({
                level: 'error',
                msg: (e.message + loc).substring(0, 200)
            });
        });
        window.addEventListener('unhandledrejection', function(e) {
            var reason = e.reason ? String(e.reason) : 'unknown';
            window.__auditConsoleErrors.push({
                level: 'error',
                msg: ('Unhandled Promise Rejection: ' + reason).substring(0, 200)
            });
        });
    })();
    