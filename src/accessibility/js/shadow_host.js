function() {
        var root = this.getRootNode();
        return (root && root.host) ? root.host : null;
    }