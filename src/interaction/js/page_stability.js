new Promise(resolve => {
            const started = performance.now();
            let mutations = 0;
            let addedElementNodes = 0;
            const targets = new Set();
            let quietTimer;
            let done = false;
            // A client-rendered app can sit quiet behind an empty splash
            // screen while it fetches its first view (eesti.ee: ~400 ms
            // without a single mutation, #718). Quiet without content is
            // not settled: keep waiting until something is rendered.
            const hasContent = () => {
                const body = document.body;
                if (!body) return false;
                if (document.querySelector('main, [role="main"], h1, h2, h3')) return true;
                return (body.innerText || '').trim().length > 0;
            };
            let awaitedContent = false;
            const quiet = () => {
                if (hasContent()) {
                    finish('stable', awaitedContent ? 'waited for the first rendered content' : null);
                } else {
                    awaitedContent = true;
                    quietTimer = setTimeout(quiet, 200);
                }
            };
            const finish = (status, reason) => {
                if (done) return;
                done = true;
                observer.disconnect();
                clearTimeout(quietTimer);
                clearTimeout(budgetTimer);
                resolve({ status, waited_ms: Math.round(performance.now() - started), mutation_count: mutations, reason });
            };
            const observer = new MutationObserver(records => {
                mutations += records.length;
                for (const r of records) {
                    if (targets.size < 25) targets.add(r.target);
                    if (r.type === 'childList') {
                        for (const n of r.addedNodes) {
                            if (n.nodeType === 1) addedElementNodes++;
                        }
                    }
                }
                clearTimeout(quietTimer);
                quietTimer = setTimeout(quiet, 200);
            });
            observer.observe(document.documentElement, {subtree:true, childList:true, attributes:true, characterData:true});
            const budgetTimer = setTimeout(() => {
                if (!hasContent()) {
                    finish('budget_exhausted', 'the page rendered no content within the configured budget');
                    return;
                }
                let hasRunningAnimation = false;
                try {
                    hasRunningAnimation = typeof document.getAnimations === 'function' &&
                        document.getAnimations({subtree:true}).some(a => a.playState === 'running');
                } catch (e) {}
                const boundedTargets = targets.size > 0 && targets.size <= 6;
                const noContentGrowth = addedElementNodes === 0;
                if (boundedTargets && (noContentGrowth || hasRunningAnimation)) {
                    finish('ongoing_animation', 'DOM mutations stayed confined to a small, stable set of elements without content growth, consistent with a running animation');
                } else {
                    finish('budget_exhausted', 'DOM did not remain quiet within the configured budget');
                }
            }, __BUDGET_MS__);
            if (window.__AUDITMYSITE_READY__ === true || document.documentElement.dataset.auditReady === 'true') {
                finish('ready_signal', null);
            } else {
                quietTimer = setTimeout(quiet, 200);
            }
        })