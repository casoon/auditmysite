
    (() => {
        var NON_COMPOSITED = new Set([
            'top','left','right','bottom',
            'width','height','min-width','max-width','min-height','max-height',
            'margin','margin-top','margin-bottom','margin-left','margin-right',
            'padding','padding-top','padding-bottom','padding-left','padding-right',
            'border','border-width','border-top-width','border-bottom-width',
            'border-left-width','border-right-width',
            'font-size','line-height','letter-spacing','word-spacing',
            'background','background-color','background-size','background-position',
            'border-radius','box-shadow','text-shadow',
            'color','fill','stroke',
            'clip','clip-path',
            'outline','outline-width','outline-offset',
            'flex','flex-basis','flex-grow','flex-shrink',
            'grid-template-columns','grid-template-rows',
            'columns','column-gap','column-width',
            'display','visibility','z-index',
            'vertical-align','text-align','float'
        ]);

        var findings = [];

        function sourceOf(sheet) {
            return sheet.href || 'inline';
        }

        function checkTransition(transitionValue, selector, source) {
            if (!transitionValue) return;
            transitionValue.split(',').forEach(function(part) {
                var prop = part.trim().split(/\s+/)[0].toLowerCase();
                if (prop && prop !== 'all' && prop !== 'none' && NON_COMPOSITED.has(prop)) {
                    findings.push({ kind: 'transition', name: selector, property: prop, source: source });
                }
            });
        }

        for (var si = 0; si < document.styleSheets.length; si++) {
            var sheet = document.styleSheets[si];
            var src = sourceOf(sheet);
            var rules;
            try { rules = sheet.cssRules; } catch(e) { continue; }
            if (!rules) continue;

            for (var ri = 0; ri < rules.length; ri++) {
                var rule = rules[ri];

                // @keyframes — check each frame's animated properties
                if (rule.type === 7 /* CSSKeyframesRule */) {
                    var frames = rule.cssRules || [];
                    var seen = {};
                    for (var fi = 0; fi < frames.length; fi++) {
                        var frame = frames[fi];
                        var style = frame.style;
                        if (!style) continue;
                        for (var pi = 0; pi < style.length; pi++) {
                            var prop = style[pi].toLowerCase();
                            if (NON_COMPOSITED.has(prop) && !seen[prop]) {
                                seen[prop] = true;
                                findings.push({ kind: 'keyframe', name: rule.name, property: prop, source: src });
                            }
                        }
                    }
                }

                // CSSStyleRule — check transition and -webkit-transition
                if (rule.type === 1 /* CSSStyleRule */ && rule.style) {
                    var tr = rule.style.transition || rule.style.webkitTransition || '';
                    if (tr) {
                        checkTransition(tr, rule.selectorText || '', src);
                    }
                }

                // @media — recurse one level
                if (rule.type === 4 /* CSSMediaRule */ && rule.cssRules) {
                    for (var mi = 0; mi < rule.cssRules.length; mi++) {
                        var mr = rule.cssRules[mi];
                        if (mr.type === 1 && mr.style) {
                            var mtr = mr.style.transition || mr.style.webkitTransition || '';
                            if (mtr) {
                                checkTransition(mtr, mr.selectorText || '', src);
                            }
                        }
                    }
                }
            }
        }

        return JSON.stringify(findings);
    })()
    