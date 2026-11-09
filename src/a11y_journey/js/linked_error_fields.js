(function() {
            var fields = document.querySelectorAll(
                'input[aria-invalid="true"], textarea[aria-invalid="true"], select[aria-invalid="true"]'
            );
            var linked = 0;
            Array.from(fields).forEach(function(f) {
                if (f.getAttribute('aria-describedby') || f.getAttribute('aria-errormessage')) {
                    linked++;
                }
            });
            return linked;
        })()