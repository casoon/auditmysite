
    (() => {
        const d = document.doctype;
        const doctype = d
            ? `<!DOCTYPE ${d.name}${d.publicId ? ` PUBLIC "${d.publicId}"` : ''}${d.systemId ? ` "${d.systemId}"` : ''}>`
            : '<!DOCTYPE html>';
        return doctype + '\n' + document.documentElement.outerHTML;
    })()
    