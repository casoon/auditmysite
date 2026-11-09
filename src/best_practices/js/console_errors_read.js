
    (() => {
        // Undefined: the collector was never installed — no measurement,
        // not "no errors".
        if (window.__auditConsoleErrors === undefined) return null;
        return JSON.stringify(window.__auditConsoleErrors);
    })()
    