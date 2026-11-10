
        (async () => {
            if (!('serviceWorker' in navigator) || !navigator.serviceWorker.getRegistrations) {
                return 0;
            }
            try {
                const registrations = await navigator.serviceWorker.getRegistrations();
                return registrations.length || 0;
            } catch (e) {
                return 0;
            }
        })()
    