function() {
                this.focus();
                const href = this.getAttribute('href') || '';
                if (!href.startsWith('#') || href.length < 2) return 'no_fragment';
                let id = href.slice(1);
                try { id = decodeURIComponent(id); } catch (e) {}
                return document.getElementById(id) || document.getElementsByName(id)[0] || 'missing';
            }