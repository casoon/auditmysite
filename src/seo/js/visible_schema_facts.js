
            (() => {
                const clean = value => (value || '').replace(/\s+/g, ' ').trim().slice(0, 240);
                const headingText = (h) => {
                    // Lines of a split headline are block-level boxes or <br>: textContent
                    // drops the boundary and glued the words together ("Webseiten &Webanwendungen").
                    // Inline-block children (per-letter spans) stay glued.
                    const blockish = new Set(['block', 'flex', 'grid', 'list-item', 'flow-root', 'table', 'table-row', 'table-cell']);
                    let out = '';
                    const walk = (node) => {
                        for (const child of node.childNodes) {
                            if (child.nodeType === 3) {
                                out += child.nodeValue;
                            } else if (child.nodeType === 1) {
                                const tag = child.tagName;
                                if (tag === 'SCRIPT' || tag === 'STYLE') continue;
                                const brk = tag === 'BR' || blockish.has(getComputedStyle(child).display);
                                if (brk) out += ' ';
                                walk(child);
                                if (brk) out += ' ';
                            }
                        }
                    };
                    walk(h);
                    return out.replace(/\s+/g, ' ').trim();
                };
                const unique = values => [...new Set(values.map(clean).filter(Boolean))].slice(0, 20);
                const values = selectors => unique(selectors.flatMap(selector =>
                    [...document.querySelectorAll(selector)].map(element =>
                        element.getAttribute('content') ||
                        element.getAttribute('datetime') ||
                        element.getAttribute('data-price') ||
                        element.textContent
                    )
                ));
                const priceValues = values([
                    '[itemprop="price"]', '[data-price]', '.price', '.product-price',
                    '[class*="price__"]', '[class*="product-price"]'
                ]);
                const currencyValues = values([
                    '[itemprop="priceCurrency"]', '[data-currency]', '.price-currency'
                ]);
                const availabilityValues = values([
                    '[itemprop="availability"]', '[data-availability]', '.availability',
                    '[class*="stock-status"]'
                ]);
                const authors = values([
                    '[rel="author"]', '[itemprop="author"]', '.author', '.byline',
                    '[class*="author-name"]'
                ]);
                const dates = values([
                    'time[datetime]', '[itemprop="datePublished"]', '[itemprop="datePosted"]',
                    '[itemprop="startDate"]'
                ]);
                const faqQuestions = unique([
                    ...document.querySelectorAll(
                        '[itemtype*="Question"] [itemprop="name"], details > summary, .faq h2, .faq h3, [class*="faq"] h2, [class*="faq"] h3'
                    )
                ].map(element => element.textContent));
                const faqAnswers = unique([
                    ...document.querySelectorAll(
                        '[itemtype*="Answer"] [itemprop="text"], details > :not(summary), .faq-answer, [class*="faq-answer"]'
                    )
                ].map(element => element.textContent));
                const breadcrumbRoot = document.querySelector(
                    'nav[aria-label*="breadcrumb" i], [itemtype*="BreadcrumbList"], .breadcrumb, [class*="breadcrumb"]'
                );
                const breadcrumbs = breadcrumbRoot ? unique(
                    [...breadcrumbRoot.querySelectorAll('a, [aria-current="page"], [itemprop="name"]')]
                        .map(element => element.textContent)
                ) : [];
                return JSON.stringify({
                    document_title: clean(document.title) || null,
                    h1: clean(document.querySelector('h1') ? headingText(document.querySelector('h1')) : null) || null,
                    prices: priceValues,
                    currencies: currencyValues,
                    availability: availabilityValues,
                    authors,
                    dates,
                    faq_questions: faqQuestions,
                    faq_answers: faqAnswers,
                    breadcrumbs
                });
            })()
            