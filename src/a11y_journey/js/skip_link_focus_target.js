function(trigger) {
                const a = document.activeElement;
                if (!a || a === document.body || a === document.documentElement) return 'none';
                if (a === this || this.contains(a)) return 'inside';
                if (a === trigger) return 'trigger';
                return (this.compareDocumentPosition(a) & Node.DOCUMENT_POSITION_FOLLOWING)
                    ? 'after' : 'before';
            }