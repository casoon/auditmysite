//! Issue aggregation for the page-health analysis.

use super::{PageHealthAnalysis, PageHealthIssue};

/// Build the page-health issue list in the requested language.
///
/// Pure function of the analysis struct — called with English at analysis time
/// (canonical, for JSON) and re-called with the report locale by the PDF
/// presentation builder (#406).
pub fn collect_issues(a: &PageHealthAnalysis, en: bool) -> Vec<PageHealthIssue> {
    let mut issues = Vec::new();

    if !a.has_doctype && a.dom_node_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "missing_doctype".to_string(),
            message: if en {
                "Missing HTML5 doctype declaration — browser renders in quirks mode".to_string()
            } else {
                "Fehlende HTML5-Doctype-Deklaration — Browser rendert im Quirks-Mode".to_string()
            },
            severity: "high".to_string(),
        });
    }

    if a.document_write_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "document_write".to_string(),
            message: if en {
                format!(
                    "{} inline {} use document.write() — blocks HTML parsing",
                    a.document_write_count,
                    if a.document_write_count == 1 {
                        "script"
                    } else {
                        "scripts"
                    }
                )
            } else {
                format!(
                    "{} Inline-{} verwenden document.write() — blockiert HTML-Parsing",
                    a.document_write_count,
                    if a.document_write_count == 1 {
                        "Script"
                    } else {
                        "Scripts"
                    }
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.dom_node_count >= 1500 {
        issues.push(PageHealthIssue {
            issue_type: "excessive_dom".to_string(),
            message: if en {
                format!(
                    "DOM size critical: {} elements (recommended: <1500, max depth: {})",
                    a.dom_node_count, a.dom_max_depth
                )
            } else {
                format!(
                    "DOM-Größe kritisch: {} Elemente (Empfehlung: <1500, max Tiefe: {})",
                    a.dom_node_count, a.dom_max_depth
                )
            },
            severity: "high".to_string(),
        });
    } else if a.dom_node_count >= 800 {
        issues.push(PageHealthIssue {
            issue_type: "large_dom".to_string(),
            message: if en {
                format!(
                    "DOM size elevated: {} elements (recommended: <800, max depth: {})",
                    a.dom_node_count, a.dom_max_depth
                )
            } else {
                format!(
                    "DOM-Größe erhöht: {} Elemente (Empfehlung: <800, max Tiefe: {})",
                    a.dom_node_count, a.dom_max_depth
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.images_without_dimensions > 0 {
        issues.push(PageHealthIssue {
            issue_type: "images_without_dimensions".to_string(),
            message: if en {
                format!(
                    "{} <img> elements without explicit width/height — CLS risk",
                    a.images_without_dimensions
                )
            } else {
                format!(
                    "{} <img>-Elemente ohne explizite width/height — CLS-Risiko",
                    a.images_without_dimensions
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.paste_blocking_password_fields > 0 {
        issues.push(PageHealthIssue {
            issue_type: "paste_blocked_password".to_string(),
            message: if en {
                format!(
                    "{} password {} block pasting (onpaste handler) — interferes with password managers",
                    a.paste_blocking_password_fields,
                    if a.paste_blocking_password_fields == 1 { "field" } else { "fields" }
                )
            } else {
                format!(
                    "{} {} blockieren Einfügen (onpaste-Handler) — beeinträchtigt Passwort-Manager",
                    a.paste_blocking_password_fields,
                    if a.paste_blocking_password_fields == 1 { "Passwortfeld" } else { "Passwortfelder" }
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.offscreen_images_without_lazy > 0 {
        issues.push(PageHealthIssue {
            issue_type: "offscreen_images_without_lazy".to_string(),
            message: if en {
                format!(
                    "{} images below the viewport without loading=\"lazy\" — delay initial page render",
                    a.offscreen_images_without_lazy
                )
            } else {
                format!(
                    "{} Bilder unterhalb des Viewports ohne loading=\"lazy\" — verzögern ersten Seitenaufbau",
                    a.offscreen_images_without_lazy
                )
            },
            severity: if a.offscreen_images_without_lazy >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.images_without_srcset > 0 {
        issues.push(PageHealthIssue {
            issue_type: "images_without_srcset".to_string(),
            message: if en {
                format!(
                    "{} <img> elements without srcset — no responsive image variants for different resolutions",
                    a.images_without_srcset
                )
            } else {
                format!(
                    "{} <img>-Elemente ohne srcset — keine responsiven Bildvarianten für verschiedene Auflösungen",
                    a.images_without_srcset
                )
            },
            severity: if a.images_without_srcset >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.missing_preconnect_count > 0 {
        let sample = if a.missing_preconnect_origins.is_empty() {
            String::new()
        } else if en {
            format!(" (e.g. {})", a.missing_preconnect_origins.join(", "))
        } else {
            format!(" (z.B. {})", a.missing_preconnect_origins.join(", "))
        };
        issues.push(PageHealthIssue {
            issue_type: "missing_preconnect".to_string(),
            message: if en {
                format!(
                    "{} external origins without <link rel=\"preconnect\">{}",
                    a.missing_preconnect_count, sample
                )
            } else {
                format!(
                    "{} externe Origins ohne <link rel=\"preconnect\">{}",
                    a.missing_preconnect_count, sample
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.non_crawlable_links > 0 {
        issues.push(PageHealthIssue {
            issue_type: "non_crawlable_links".to_string(),
            message: if en {
                format!(
                    "{} links not crawlable (javascript:, empty, no href) — loss of PageRank",
                    a.non_crawlable_links
                )
            } else {
                format!(
                    "{} Links nicht crawlbar (javascript:, leer, kein href) — PageRank-Verlust",
                    a.non_crawlable_links
                )
            },
            severity: if a.non_crawlable_links >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.images_without_modern_format > 0 {
        issues.push(PageHealthIssue {
            issue_type: "images_without_modern_format".to_string(),
            message: if en {
                format!(
                    "{} images as JPEG/PNG without a WebP/AVIF alternative — increased load time",
                    a.images_without_modern_format
                )
            } else {
                format!(
                    "{} Bilder als JPEG/PNG ohne WebP/AVIF-Alternative — erhöhte Ladezeit",
                    a.images_without_modern_format
                )
            },
            severity: if a.images_without_modern_format >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.oversized_images > 0 {
        issues.push(PageHealthIssue {
            issue_type: "oversized_images".to_string(),
            message: if en {
                format!(
                    "{} images are displayed significantly smaller than their natural resolution",
                    a.oversized_images
                )
            } else {
                format!(
                    "{} Bilder werden deutlich kleiner dargestellt als ihre natürliche Auflösung",
                    a.oversized_images
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.gif_images > 0 {
        issues.push(PageHealthIssue {
            issue_type: "gif_images".to_string(),
            message: if en {
                format!(
                    "{} GIF {} found — consider replacing with MP4/WebM (80–95% smaller)",
                    a.gif_images,
                    if a.gif_images == 1 { "image" } else { "images" }
                )
            } else {
                format!(
                    "{} {} gefunden — ggf. als MP4/WebM ersetzen (80–95 % kleiner)",
                    a.gif_images,
                    if a.gif_images == 1 {
                        "GIF-Bild"
                    } else {
                        "GIF-Bilder"
                    }
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.font_display_issues > 0 {
        issues.push(PageHealthIssue {
            issue_type: "font_display_missing".to_string(),
            message: if en {
                format!(
                    "{} @font-face rules without font-display: swap/fallback/optional — FOIT risk",
                    a.font_display_issues
                )
            } else {
                format!(
                    "{} @font-face-Regeln ohne font-display: swap/fallback/optional — FOIT-Risiko",
                    a.font_display_issues
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.fonts_without_preload_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "fonts_not_preloaded".to_string(),
            message: if en {
                format!(
                    "{} of {} web font{} not preloaded via <link rel=\"preload\" as=\"font\"> — delays first text render",
                    a.fonts_without_preload_count,
                    a.font_face_count,
                    if a.font_face_count == 1 { "" } else { "s" }
                )
            } else {
                format!(
                    "{} von {} Web-Font{} nicht per <link rel=\"preload\" as=\"font\"> vorgeladen — verzögert ersten Textaufbau",
                    a.fonts_without_preload_count,
                    a.font_face_count,
                    if a.font_face_count == 1 { "" } else { "s" }
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.orphaned_preload_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "orphaned_preload".to_string(),
            message: if en {
                format!(
                    "{} <link rel=\"preload\"> hints point to resources that were never loaded (orphaned)",
                    a.orphaned_preload_count
                )
            } else {
                format!(
                    "{} <link rel=\"preload\">-Hinweise auf nicht geladene Ressourcen (orphaned)",
                    a.orphaned_preload_count
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.lcp_image_lazy_loaded {
        issues.push(PageHealthIssue {
            issue_type: "lcp_image_lazy_loaded".to_string(),
            message: if en {
                "LCP image candidate has loading=\"lazy\" — delays the Largest Contentful Paint"
                    .to_string()
            } else {
                "LCP-Bildkandidat hat loading=\"lazy\" — verzögert den Largest Contentful Paint"
                    .to_string()
            },
            severity: "high".to_string(),
        });
    }

    if a.lcp_image_without_preload {
        let url_hint = a
            .lcp_image_url
            .as_deref()
            .map(|u| format!(" ({})", u))
            .unwrap_or_default();
        issues.push(PageHealthIssue {
            issue_type: "lcp_image_without_preload".to_string(),
            message: if en {
                format!(
                    "Largest visible image{} has no <link rel=\"preload\" as=\"image\"> hint",
                    url_hint
                )
            } else {
                format!(
                    "Größtes sichtbares Bild{} hat keinen <link rel=\"preload\" as=\"image\">-Hint",
                    url_hint
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.lcp_image_without_fetchpriority && !a.lcp_image_lazy_loaded {
        issues.push(PageHealthIssue {
            issue_type: "lcp_image_without_fetchpriority".to_string(),
            message: if en {
                "LCP image candidate is missing fetchpriority=\"high\" — load priority not signaled"
                    .to_string()
            } else {
                "LCP-Bildkandidat fehlt fetchpriority=\"high\" — Ladepriorität nicht signalisiert"
                    .to_string()
            },
            severity: "low".to_string(),
        });
    }

    if a.deprecated_api_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "deprecated_apis".to_string(),
            message: if en {
                format!(
                    "{} deprecated browser {} detected in inline scripts",
                    a.deprecated_api_count,
                    if a.deprecated_api_count == 1 {
                        "API"
                    } else {
                        "APIs"
                    }
                )
            } else {
                format!(
                    "{} veraltete {} in Inline-Scripts erkannt",
                    a.deprecated_api_count,
                    if a.deprecated_api_count == 1 {
                        "Browser-API"
                    } else {
                        "Browser-APIs"
                    }
                )
            },
            severity: "medium".to_string(),
        });
    }

    if !a.uses_http2 {
        issues.push(PageHealthIssue {
            issue_type: "http1_only".to_string(),
            message: if en {
                "Page is served over HTTP/1.1 — HTTP/2 enables multiplexing and header compression".to_string()
            } else {
                "Seite wird über HTTP/1.1 ausgeliefert — HTTP/2 ermöglicht Multiplexing und Header-Komprimierung".to_string()
            },
            severity: "medium".to_string(),
        });
    }

    if !a.has_compression {
        issues.push(PageHealthIssue {
            issue_type: "missing_compression".to_string(),
            message: if en {
                "Page content is transferred without compression (no gzip/brotli)".to_string()
            } else {
                "Seiteninhalt wird ohne Komprimierung übertragen (kein gzip/brotli)".to_string()
            },
            severity: "high".to_string(),
        });
    }

    if !a.has_efficient_cache && a.cache_control.is_some() {
        issues.push(PageHealthIssue {
            issue_type: "inefficient_cache".to_string(),
            message: if en {
                format!(
                    "Cache-Control without an effective caching lifetime: {}",
                    a.cache_control.as_deref().unwrap_or("")
                )
            } else {
                format!(
                    "Cache-Control ohne effektive Caching-Dauer: {}",
                    a.cache_control.as_deref().unwrap_or("")
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.resource_cache.inefficient_resources > 0 {
        let sample = a
            .resource_cache
            .samples
            .first()
            .map(|finding| {
                if en {
                    format!(" (e.g. {}: {})", finding.url, finding.reason)
                } else {
                    format!(" (z.B. {}: {})", finding.url, finding.reason)
                }
            })
            .unwrap_or_default();
        issues.push(PageHealthIssue {
            issue_type: "inefficient_resource_cache".to_string(),
            message: if en {
                format!(
                    "{} of {} static {} without an efficient cache policy{}",
                    a.resource_cache.inefficient_resources,
                    a.resource_cache.cacheable_resources,
                    if a.resource_cache.cacheable_resources == 1 {
                        "resource"
                    } else {
                        "resources"
                    },
                    sample
                )
            } else {
                format!(
                    "{} von {} statischen {} ohne effiziente Cache-Policy{}",
                    a.resource_cache.inefficient_resources,
                    a.resource_cache.cacheable_resources,
                    if a.resource_cache.cacheable_resources == 1 {
                        "Ressource"
                    } else {
                        "Ressourcen"
                    },
                    sample
                )
            },
            severity: if a.resource_cache.inefficient_resources >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.resource_cache.missing_content_type > 0 || a.resource_cache.incorrect_content_type > 0 {
        let sample = a
            .resource_cache
            .content_type_samples
            .first()
            .map(|finding| {
                let actual =
                    finding
                        .actual
                        .as_deref()
                        .unwrap_or(if en { "none" } else { "keiner" });
                if en {
                    format!(
                        " (e.g. {}: expected {}, got {})",
                        finding.url, finding.expected, actual
                    )
                } else {
                    format!(
                        " (z.B. {}: erwartet {}, erhalten {})",
                        finding.url, finding.expected, actual
                    )
                }
            })
            .unwrap_or_default();
        let total = a.resource_cache.missing_content_type + a.resource_cache.incorrect_content_type;
        issues.push(PageHealthIssue {
            issue_type: "incorrect_content_type".to_string(),
            message: if en {
                format!(
                    "{total} static {} with a missing or incorrect Content-Type header{sample}",
                    if total == 1 { "resource" } else { "resources" }
                )
            } else {
                format!(
                    "{total} statische {} mit fehlendem oder falschem Content-Type-Header{sample}",
                    if total == 1 {
                        "Ressource"
                    } else {
                        "Ressourcen"
                    }
                )
            },
            severity: "medium".to_string(),
        });
    }

    // Charset consistency between the HTTP Content-Type header and the
    // declared <meta charset> (#543). Only meaningful when both are present —
    // a page missing one or the other is covered by its own separate check
    // elsewhere (meta validation), not here.
    if let (Some(header), Some(declared)) = (&a.charset_header, &a.declared_charset) {
        let declared_normalized = declared.to_ascii_lowercase();
        if header != &declared_normalized {
            issues.push(PageHealthIssue {
                issue_type: "charset_mismatch".to_string(),
                message: if en {
                    format!(
                        "Charset mismatch: HTTP header declares \"{header}\", <meta charset> declares \"{declared}\""
                    )
                } else {
                    format!(
                        "Charset-Konflikt: HTTP-Header deklariert \"{header}\", <meta charset> deklariert \"{declared}\""
                    )
                },
                severity: "medium".to_string(),
            });
        }
    }

    if a.hreflang_invalid_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "hreflang_invalid".to_string(),
            message: if en {
                format!(
                    "{} hreflang entries with an invalid language code",
                    a.hreflang_invalid_count
                )
            } else {
                format!(
                    "{} hreflang-Einträge mit ungültigem Sprachcode",
                    a.hreflang_invalid_count
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.jsonld_invalid_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "jsonld_invalid".to_string(),
            message: if en {
                format!(
                    "{} JSON-LD blocks without @context or @type",
                    a.jsonld_invalid_count
                )
            } else {
                format!(
                    "{} JSON-LD-Blöcke ohne @context oder @type",
                    a.jsonld_invalid_count
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.sync_head_scripts > 0 {
        issues.push(PageHealthIssue {
            issue_type: "sync_head_scripts".to_string(),
            message: if en {
                format!(
                    "{} {} in <head> without defer/async/module are syntactically render-blocking; whether they delay measurably is shown by the render-blocking analysis.",
                    a.sync_head_scripts,
                    if a.sync_head_scripts == 1 { "script" } else { "scripts" }
                )
            } else {
                format!(
                    "{} {} im <head> ohne defer/async/module sind syntaktisch render-blockierend; ob sie messbar verzögern, zeigt die Render-Blocking-Analyse.",
                    a.sync_head_scripts,
                    if a.sync_head_scripts == 1 { "Script" } else { "Scripts" }
                )
            },
            severity: "medium".to_string(),
        });
    }

    let sri_total = a.external_scripts_without_sri + a.external_styles_without_sri;
    if sri_total > 0 {
        issues.push(PageHealthIssue {
            issue_type: "missing_sri".to_string(),
            message: if en {
                format!(
                    "{} external {} without Subresource Integrity (integrity attribute missing)",
                    sri_total,
                    if sri_total == 1 {
                        "resource"
                    } else {
                        "resources"
                    }
                )
            } else {
                format!(
                    "{} externe {} ohne Subresource Integrity (integrity-Attribut fehlt)",
                    sri_total,
                    if sri_total == 1 {
                        "Ressource"
                    } else {
                        "Ressourcen"
                    }
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.broken_fragment_links > 0 {
        let sample = if a.broken_fragment_samples.is_empty() {
            String::new()
        } else if en {
            format!(" (e.g. {})", a.broken_fragment_samples.join(", "))
        } else {
            format!(" (z.B. {})", a.broken_fragment_samples.join(", "))
        };
        issues.push(PageHealthIssue {
            issue_type: "broken_fragment_links".to_string(),
            message: if en {
                format!(
                    "{} anchor {} point to non-existent IDs{}",
                    a.broken_fragment_links,
                    if a.broken_fragment_links == 1 {
                        "link"
                    } else {
                        "links"
                    },
                    sample
                )
            } else {
                format!(
                    "{} Anker-{} verweisen auf nicht existierende IDs{}",
                    a.broken_fragment_links,
                    if a.broken_fragment_links == 1 {
                        "Link"
                    } else {
                        "Links"
                    },
                    sample
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.generic_link_text_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "generic_link_text".to_string(),
            message: if en {
                format!(
                    "{} {} with non-descriptive text (\"here\", \"more\", \"click here\" and similar)",
                    a.generic_link_text_count,
                    if a.generic_link_text_count == 1 { "link" } else { "links" }
                )
            } else {
                format!(
                    "{} {} mit nicht-beschreibendem Text (\"hier\", \"mehr\", \"click here\" u.ä.)",
                    a.generic_link_text_count,
                    if a.generic_link_text_count == 1 { "Link" } else { "Links" }
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.is_soft_404 {
        issues.push(PageHealthIssue {
            issue_type: "soft_404".to_string(),
            message: if en {
                format!(
                    "Server returns HTTP {} for non-existent URLs (soft 404)",
                    a.soft_404_status.unwrap_or(200)
                )
            } else {
                format!(
                    "Server gibt HTTP {} für nicht-existierende URLs zurück (Soft 404)",
                    a.soft_404_status.unwrap_or(200)
                )
            },
            severity: "high".to_string(),
        });
    }

    if let Some(custom_404) = &a.custom_404 {
        if !custom_404.proper_status {
            issues.push(PageHealthIssue {
                issue_type: "custom_404_invalid_status".to_string(),
                message: if en {
                    format!(
                        "Non-existent URL returns HTTP {} instead of 404/410",
                        custom_404.status
                    )
                } else {
                    format!(
                        "Nicht-existente URL liefert HTTP {} statt 404/410",
                        custom_404.status
                    )
                },
                severity: "high".to_string(),
            });
        } else if !custom_404.custom_page {
            issues.push(PageHealthIssue {
                issue_type: "generic_404_page".to_string(),
                message: if en {
                    "404 page looks generic or very sparse; a helpful custom 404 page improves orientation and crawling signals".to_string()
                } else {
                    "404-Seite wirkt generisch oder sehr knapp; eine hilfreiche Custom-404-Seite verbessert Orientierung und Crawling-Signale".to_string()
                },
                severity: "low".to_string(),
            });
        }
    }

    if a.has_meta_refresh {
        let delay = a
            .meta_refresh_content
            .as_deref()
            .and_then(|c| c.split(';').next())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);
        issues.push(PageHealthIssue {
            issue_type: "meta_refresh".to_string(),
            message: match (en, delay == 0) {
                (true, true) => {
                    "Immediate redirect via meta-refresh (harmful for SEO, use a 301 redirect)"
                        .to_string()
                }
                (true, false) => format!("meta-refresh with {}s delay found", delay),
                (false, true) => {
                    "Sofort-Weiterleitung via meta-refresh (SEO-schädlich, nutze 301-Redirect)"
                        .to_string()
                }
                (false, false) => format!("meta-refresh mit {}s Verzögerung gefunden", delay),
            },
            severity: if delay == 0 { "high" } else { "medium" }.to_string(),
        });
    }

    if a.frame_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "frames".to_string(),
            message: if en {
                format!("{} deprecated <frame> elements found", a.frame_count)
            } else {
                format!("{} veraltete <frame>-Elemente gefunden", a.frame_count)
            },
            severity: "high".to_string(),
        });
    }

    if a.url_is_too_long {
        issues.push(PageHealthIssue {
            issue_type: "url_too_long".to_string(),
            message: if en {
                format!(
                    "URL with {} characters exceeds the recommendation (>115)",
                    a.url_length
                )
            } else {
                format!(
                    "URL mit {} Zeichen überschreitet Empfehlung (>115)",
                    a.url_length
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.url_is_too_deep {
        issues.push(PageHealthIssue {
            issue_type: "url_too_deep".to_string(),
            message: if en {
                format!(
                    "URL path depth {} exceeds the recommendation (>5 levels)",
                    a.url_path_depth
                )
            } else {
                format!(
                    "URL-Pfadtiefe {} überschreitet Empfehlung (>5 Ebenen)",
                    a.url_path_depth
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.url_has_query_params {
        issues.push(PageHealthIssue {
            issue_type: "dynamic_url".to_string(),
            message: if en {
                "URL contains query parameters (dynamic URL)".to_string()
            } else {
                "URL enthält Query-Parameter (dynamische URL)".to_string()
            },
            severity: "low".to_string(),
        });
    }

    if a.redirect_count >= 2 {
        issues.push(PageHealthIssue {
            issue_type: "multiple_redirects".to_string(),
            message: if en {
                format!(
                    "{} HTTP redirects before the final page — each hop costs ~100–300 ms",
                    a.redirect_count
                )
            } else {
                format!(
                    "{} HTTP-Weiterleitungen vor der finalen Seite — jeder Hop kostet ~100–300 ms",
                    a.redirect_count
                )
            },
            severity: if a.redirect_count >= 3 {
                "high"
            } else {
                "medium"
            }
            .to_string(),
        });
    } else if a.own_redirect_detected {
        issues.push(PageHealthIssue {
            issue_type: "redirect".to_string(),
            message: if en {
                format!(
                    "Page redirects to: {}",
                    a.own_final_url.as_deref().unwrap_or("(unknown)")
                )
            } else {
                format!(
                    "Seite leitet weiter zu: {}",
                    a.own_final_url.as_deref().unwrap_or("(unbekannt)")
                )
            },
            severity: "medium".to_string(),
        });
    }

    if let Some(ref www) = a.www_consolidation {
        if !www.is_consolidated {
            issues.push(PageHealthIssue {
                issue_type: "www_not_consolidated".to_string(),
                message: if en {
                    "www and non-www versions are not consolidated (no 301 redirect)".to_string()
                } else {
                    "www und non-www Version sind nicht konsolidiert (kein 301-Redirect)"
                        .to_string()
                },
                severity: "medium".to_string(),
            });
        }
    }

    if let Some(ref canon) = a.url_canonicalization {
        if canon.trailing_slash_inconsistent {
            issues.push(PageHealthIssue {
                issue_type: "trailing_slash_inconsistent".to_string(),
                message: if en {
                    "URL is reachable with and without a trailing slash as two separate 200 responses — duplicate-content risk without a canonical redirect".to_string()
                } else {
                    "URL ist mit und ohne abschließendem Slash als zwei getrennte 200-Antworten erreichbar — Duplicate-Content-Risiko ohne kanonischen Redirect".to_string()
                },
                severity: "medium".to_string(),
            });
        }
        if canon.http_to_https_missing {
            let status_text = canon.http_status.map(|s| s.to_string()).unwrap_or_else(|| {
                if en {
                    "unreachable".to_string()
                } else {
                    "nicht erreichbar".to_string()
                }
            });
            issues.push(PageHealthIssue {
                issue_type: "http_not_redirected_to_https".to_string(),
                message: if en {
                    format!("http:// version does not redirect to https:// (status: {status_text})")
                } else {
                    format!(
                        "http://-Version leitet nicht auf https:// weiter (Status: {status_text})"
                    )
                },
                severity: "high".to_string(),
            });
        }
    }

    issues
}
