function() {
        this.scrollIntoView({block: 'center', inline: 'center'});
        const r = this.getBoundingClientRect();
        const prevOutline = this.style.outline;
        const prevOutlineOffset = this.style.outlineOffset;
        this.style.outline = '2px solid #C0392B';
        this.style.outlineOffset = '2px';
        return {
            x: r.x, y: r.y, width: r.width, height: r.height,
            vw: window.innerWidth || 1280, vh: window.innerHeight || 800,
            prevOutline: prevOutline, prevOutlineOffset: prevOutlineOffset
        };
    }