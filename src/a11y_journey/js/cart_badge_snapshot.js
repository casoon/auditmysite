(function() {
            var el = document.querySelector(
                '[class*="cart-count"],[class*="cart-badge"],[data-cart-count],[class*="basket-count"]'
            );
            return el ? (el.textContent || '').trim() : null;
        })()