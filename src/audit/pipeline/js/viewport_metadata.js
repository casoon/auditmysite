(() => {
                return {
                    dpr: window.devicePixelRatio || 1.0,
                    width: window.innerWidth || 1280,
                    height: window.innerHeight || 800,
                    scrollX: window.scrollX || window.pageXOffset || 0,
                    scrollY: window.scrollY || window.pageYOffset || 0
                };
             })()