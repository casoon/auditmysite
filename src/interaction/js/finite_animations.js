new Promise(resolve => {
            if (typeof document.getAnimations !== 'function') {
                resolve({ settled: true, waited_ms: 0, running: 0 });
                return;
            }
            const started = performance.now();
            const remaining = a => {
                const t = a.effect.getComputedTiming();
                return (t.endTime - (t.localTime || 0)) / Math.abs(a.playbackRate || 1);
            };
            const step = () => {
                const elapsed = performance.now() - started;
                const running = document.getAnimations().filter(a =>
                    a.playState === 'running' && a.effect && isFinite(a.effect.getComputedTiming().endTime));
                const left = __BUDGET_MS__ - elapsed;
                const waitable = running.filter(a => remaining(a) <= left);
                if (running.length === 0 || waitable.length === 0 || left <= 0) {
                    resolve({ settled: running.length === 0, waited_ms: Math.round(elapsed), running: running.length });
                    return;
                }
                Promise.race([
                    Promise.all(waitable.map(a => a.finished.catch(() => null))),
                    new Promise(r => setTimeout(r, left))
                ]).then(() => requestAnimationFrame(() => requestAnimationFrame(step)));
            };
            step();
        })