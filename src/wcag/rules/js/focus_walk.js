
            (() => {
              const originalX = scrollX, originalY = scrollY, original = document.activeElement;
              /*CSS_SELECTOR*/
              const selector = 'a[href],button,input:not([type="hidden"]),select,textarea,[tabindex]:not([tabindex="-1"]),[contenteditable="true"]';
              const controls = [];
              for (const element of document.querySelectorAll(selector)) {
                const style = getComputedStyle(element);
                const rect = element.getBoundingClientRect();
                if (element.disabled || style.display === 'none' || style.visibility === 'hidden' || rect.width <= 3 || rect.height <= 3) continue;
                __amsPush(controls, element, element, 60);
                if (__amsReal(controls) >= 60) break;
              }
              const findings = [];
              for (const control of controls) {
                control.focus({preventScroll: true});
                control.scrollIntoView({block: 'nearest', inline: 'nearest'});
                const rect = control.getBoundingClientRect();
                const points = [
                  [rect.left + 1, rect.top + 1], [rect.right - 1, rect.top + 1],
                  [rect.left + 1, rect.bottom - 1], [rect.right - 1, rect.bottom - 1],
                  [rect.left + rect.width / 2, rect.top + rect.height / 2]
                ].filter(([x,y]) => x >= 0 && y >= 0 && x < innerWidth && y < innerHeight);
                if (!points.length) continue;
                const blockers = points.map(([x,y]) => {
                  const painted = document.elementsFromPoint(x, y);
                  const controlIndex = painted.findIndex(element => element === control || control.contains(element));
                  if (controlIndex <= 0) return null;
                  return painted.slice(0, controlIndex).find(element => {
                    if (element.contains(control) || control.contains(element)) return false;
                    const style = getComputedStyle(element);
                    return style.position === 'fixed' || style.position === 'sticky';
                  }) || null;
                });
                if (blockers.every(Boolean)) {
                  __amsPush(findings, control, {control: __amsCssSelector(control), blocker: __amsCssSelector(blockers[0])}, 5);
                  if (__amsReal(findings) >= 5) break;
                }
              }
              scrollTo(originalX, originalY);
              if (original && original.focus) original.focus({preventScroll:true});
              return { findings };
            })()
            