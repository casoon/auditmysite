//! German titles of non-WCAG findings (#898, #406).
//!
//! A finding outside the WCAG engine is titled with the readable form of its
//! check code (`excessive_dom` becomes `Excessive dom`) — canonical English in
//! the JSON. A German report shows the title from here instead. The key is the
//! finding's `rule_id` (`<source>.<check>`).

/// [`german_finding_title`] with the values its static table cannot hold: a
/// missing CSP directive is named, so findings for different directives do
/// not share one title (#1039).
pub(crate) fn german_finding_title_text(rule_id: &str) -> Option<String> {
    if let Some(directive) = rule_id
        .strip_prefix("security.headers.missing_")
        .filter(|directive| crate::security::CSP_REQUIRED_DIRECTIVES.contains(directive))
    {
        return Some(crate::i18n::text_args(
            "security-csp-missing-directive",
            &[(
                "directive",
                fluent_bundle::FluentValue::from(directive.to_string()),
            )],
            false,
        ));
    }
    german_finding_title(rule_id).map(str::to_string)
}

/// German title of the finding with `rule_id`, or `None` for checks whose title
/// is page content or open-ended (technology names, validator rule ids, library
/// names): those keep the stored title.
pub fn german_finding_title(rule_id: &str) -> Option<&'static str> {
    if let Some(check) = rule_id.strip_prefix("seo.schema.") {
        return Some(match check {
            "jsonld_parse_error" => "Strukturierte Daten: ungültiges JSON-LD",
            c if c.starts_with("schema_rule_missing_") => {
                "Strukturierte Daten: Pflichtangabe fehlt"
            }
            _ => "Strukturierte Daten: Hinweis",
        });
    }
    if let Some(check) = rule_id.strip_prefix("security.headers.") {
        if check.starts_with("missing_") && check != "missing_header" && check != "missing_https" {
            return Some("CSP: Direktive fehlt");
        }
    }
    // HTML conformance: the rule ids come from the validator and are open
    // ended, the finding text is validator output.
    if rule_id.starts_with("html_conform.schema.") {
        return Some("HTML-Schema-Verstoß");
    }
    if rule_id.starts_with("html_conform.assertion.") {
        return Some("HTML-Zusicherung verletzt");
    }
    if rule_id.starts_with("screen_reader.") {
        return Some("Screenreader-Lesereihenfolge");
    }
    Some(match rule_id {
        // best practices
        "best_practices.console.error" => "Konsolenfehler",
        // seo.meta
        "seo.meta.title" => "Seitentitel",
        "seo.meta.description" => "Meta-Beschreibung",
        "seo.meta.viewport" => "Viewport-Meta-Tag",
        "seo.meta.lang" => "Seitensprache",
        "seo.meta.canonical" => "Canonical-URL",
        "seo.meta.twitter:card" => "Twitter-Card-Typ",
        "seo.meta.og:image" => "Open-Graph-Bild",
        // seo.technical
        "seo.technical.broken_internal_links" => "Defekte interne Links",
        "seo.technical.canonical_domain_mismatch" => "Canonical zeigt auf andere Domain",
        "seo.technical.form_insecure_transport" => "Formular über unsichere Verbindung",
        "seo.technical.form_password_autocomplete" => "Passwortfeld ohne Autocomplete-Hinweis",
        "seo.technical.form_sensitive_get" => "Sensible Daten per GET gesendet",
        "seo.technical.hreflang_no_self_reference" => "hreflang ohne Selbstverweis",
        "seo.technical.hreflang_no_x_default" => "hreflang ohne x-default",
        "seo.technical.internal_links_with_query_params" => "Interne Links mit Parametern",
        "seo.technical.mixed_content" => "Gemischte Inhalte (HTTP auf HTTPS-Seite)",
        "seo.technical.no_canonical" => "Canonical-URL fehlt",
        "seo.technical.no_https" => "Kein HTTPS",
        "seo.technical.no_internal_links" => "Keine internen Links",
        "seo.technical.no_lang" => "Sprachangabe fehlt",
        "seo.technical.noindex" => "Seite auf noindex gesetzt",
        "seo.technical.og_image_missing_alt" => "og:image ohne Alternativtext",
        "seo.technical.pagination_duplicate_prev_next" => "Doppelte Paginierungs-Links",
        "seo.technical.pagination_self_referential_rel_link" => {
            "Paginierung verweist auf sich selbst"
        }
        "seo.technical.pwa_invalid_manifest" => "Ungültiges Web-App-Manifest",
        "seo.technical.pwa_missing_manifest" => "Web-App-Manifest fehlt",
        "seo.technical.remote_google_fonts" => "Google Fonts von externem Server",
        "seo.technical.thin_content" => "Wenig Textinhalt",
        "seo.technical.twitter_image_missing_alt" => "twitter:image ohne Alternativtext",
        "seo.technical.www_no_redirect" => "www-Variante ohne Weiterleitung",
        // seo.page_health
        "seo.page_health.broken_fragment_links" => "Defekte Sprungmarken",
        "seo.page_health.charset_mismatch" => "Zeichensatz widersprüchlich",
        "seo.page_health.custom_404_invalid_status" => "404-Seite mit falschem Statuscode",
        "seo.page_health.deprecated_apis" => "Veraltete Browser-Schnittstellen",
        "seo.page_health.document_write" => "document.write() im Einsatz",
        "seo.page_health.dynamic_url" => "Dynamische URL",
        "seo.page_health.excessive_dom" => "DOM zu groß",
        "seo.page_health.font_display_missing" => "font-display fehlt",
        "seo.page_health.fonts_not_preloaded" => "Schriften nicht vorgeladen",
        "seo.page_health.frames" => "Frames im Einsatz",
        "seo.page_health.generic_404_page" => "Generische 404-Seite",
        "seo.page_health.generic_link_text" => "Allgemeine Linktexte",
        "seo.page_health.gif_images" => "Animierte GIF-Bilder",
        "seo.page_health.hreflang_invalid" => "Ungültige hreflang-Angaben",
        "seo.page_health.http1_only" => "Nur HTTP/1.1",
        "seo.page_health.http_not_redirected_to_https" => "HTTP wird nicht auf HTTPS umgeleitet",
        "seo.page_health.images_without_dimensions" => "Bilder ohne Größenangaben",
        "seo.page_health.images_without_modern_format" => "Bilder ohne modernes Format",
        "seo.page_health.images_without_srcset" => "Bilder ohne srcset",
        "seo.page_health.incorrect_content_type" => "Falscher Content-Type",
        "seo.page_health.inefficient_cache" => "Ineffizientes Caching",
        "seo.page_health.inefficient_resource_cache" => "Ineffizientes Ressourcen-Caching",
        "seo.page_health.jsonld_invalid" => "Ungültiges JSON-LD",
        "seo.page_health.large_dom" => "DOM erhöht",
        "seo.page_health.lcp_image_lazy_loaded" => "LCP-Bild wird verzögert geladen",
        "seo.page_health.lcp_image_without_fetchpriority" => "LCP-Bild ohne fetchpriority",
        "seo.page_health.lcp_image_without_preload" => "LCP-Bild ohne Preload",
        "seo.page_health.meta_refresh" => "Meta-Refresh-Weiterleitung",
        "seo.page_health.missing_compression" => "Komprimierung fehlt",
        "seo.page_health.missing_doctype" => "Doctype fehlt",
        "seo.page_health.missing_preconnect" => "Preconnect fehlt",
        "seo.page_health.missing_sri" => "Subresource Integrity fehlt",
        "seo.page_health.multiple_redirects" => "Mehrfache Weiterleitungen",
        "seo.page_health.non_crawlable_links" => "Nicht crawlbare Links",
        "seo.page_health.offscreen_images_without_lazy" => {
            "Bilder außerhalb des Sichtbereichs ohne Lazy Loading"
        }
        "seo.page_health.orphaned_preload" => "Preload ohne Verwendung",
        "seo.page_health.oversized_images" => "Zu große Bilder",
        "seo.page_health.paste_blocked_password" => "Einfügen im Passwortfeld blockiert",
        "seo.page_health.redirect" => "Weiterleitung",
        "seo.page_health.soft_404" => "Soft-404",
        "seo.page_health.sync_head_scripts" => "Blockierende Skripte im Head",
        "seo.page_health.trailing_slash_inconsistent" => "Uneinheitlicher Schrägstrich am URL-Ende",
        "seo.page_health.url_too_deep" => "URL zu tief verschachtelt",
        "seo.page_health.url_too_long" => "URL zu lang",
        "seo.page_health.www_not_consolidated" => "www-Variante nicht zusammengeführt",
        // security.headers
        "security.headers.missing_https" => "HTTPS fehlt",
        "security.headers.missing_header" => "Sicherheits-Header fehlt",
        "security.headers.cors_wildcard_credentials" => "CORS: Wildcard mit Zugangsdaten",
        "security.headers.hsts_preload_ineligible" => "HSTS-Preload nicht erfüllt",
        "security.headers.permissions_policy_permissive" => "Permissions-Policy zu offen",
        "security.headers.public_source_map" => "Öffentliche Source Maps",
        "security.headers.unsafe_inline_script" => "CSP erlaubt Inline-Skripte",
        "security.headers.unsafe_eval_script" => "CSP erlaubt eval()",
        "security.headers.wildcard_script_source" => "CSP erlaubt Wildcard-Skriptquellen",
        "security.headers.unsafe_inline_style" => "CSP erlaubt Inline-Styles",
        "security.headers.wildcard_source" => "CSP erlaubt Wildcard-Quellen",
        // mobile
        "mobile.missing_viewport" => "Viewport fehlt",
        "mobile.improper_viewport" => "Viewport falsch konfiguriert",
        "mobile.not_scalable" => "Zoomen unterbunden",
        "mobile.small_targets" => "Zu kleine Bedienelemente",
        "mobile.small_fonts" => "Zu kleine Schrift",
        "mobile.horizontal_scroll" => "Horizontales Scrollen",
        // ux
        "ux.no_cta" => "Kein erkennbarer Call-to-Action",
        "ux.competing_ctas" => "Konkurrierende Handlungsaufforderungen",
        "ux.generic_links" => "Allgemeine Linktexte",
        "ux.no_h1" => "H1 fehlt",
        "ux.multiple_h1" => "Mehrere H1",
        "ux.heading_skips" => "Übersprungene Überschriftenebenen",
        "ux.large_dom" => "Großer DOM",
        "ux.little_text" => "Wenig Text",
        "ux.no_subheadings" => "Keine Zwischenüberschriften",
        "ux.no_contact" => "Kein Kontakt erkennbar",
        "ux.no_imprint" => "Impressum nicht erkennbar",
        "ux.no_privacy" => "Datenschutz nicht erkennbar",
        "ux.weak_trust_signals" => "Schwache Vertrauenssignale",
        "ux.too_many_links" => "Zu viele Links",
        "ux.too_many_interactive" => "Zu viele interaktive Elemente",
        // journey
        "journey.no_h1" => "H1 fehlt",
        "journey.multiple_h1" => "Mehrere H1",
        "journey.vague_h1" => "H1 wenig aussagekräftig",
        "journey.no_page_title" => "Seitentitel fehlt",
        "journey.little_early_text" => "Wenig Text im oberen Bereich",
        "journey.navigation_missing" => "Navigation fehlt",
        "journey.main_hidden_from_ax" => "Hauptbereich für Hilfstechnik verborgen",
        "journey.main_missing" => "Hauptbereich fehlt",
        "journey.few_subheadings" => "Wenige Zwischenüberschriften",
        "journey.no_breadcrumb" => "Kein Brotkrumen-Pfad",
        "journey.no_footer" => "Kein Footer",
        "journey.generic_links" => "Allgemeine Linktexte",
        "journey.empty_links" => "Links ohne Text",
        "journey.duplicate_links" => "Doppelte Linktexte",
        "journey.too_many_links" => "Zu viele Links",
        "journey.unnamed_buttons" => "Schaltflächen ohne Namen",
        "journey.unlabeled_forms" => "Formularfelder ohne Beschriftung",
        "journey.generic_buttons" => "Allgemeine Schaltflächentexte",
        "journey.no_call_to_action" => "Kein Call-to-Action",
        "journey.competing_ctas" => "Konkurrierende Handlungsaufforderungen",
        "journey.blocking_dialog" => "Blockierender Dialog",
        "journey.complex_form" => "Komplexes Formular",
        // commerce
        "commerce.missing_price" => "Preis fehlt",
        "commerce.missing_availability" => "Verfügbarkeit fehlt",
        "commerce.missing_shipping_details" => "Versandangaben fehlen",
        "commerce.missing_return_policy" => "Rückgabebedingungen fehlen",
        "commerce.missing_reviews" => "Bewertungen fehlen",
        "commerce.missing_impressum_link" => "Impressum-Link fehlt",
        "commerce.missing_agb_link" => "AGB-Link fehlt",
        "commerce.missing_widerruf_link" => "Widerrufs-Link fehlt",
        "commerce.missing_shipping_page_link" => "Versand-Link fehlt",
        "commerce.missing_payment_link" => "Zahlungs-Link fehlt",
        "commerce.missing_contact_link" => "Kontakt-Link fehlt",
        // privacy
        "privacy.tracking_request_before_consent" => "Tracking-Anfrage vor Einwilligung",
        "privacy.non_essential_storage_before_consent" => {
            "Nicht notwendige Speicherung vor Einwilligung"
        }
        "privacy.visitor_ip_transfer" => "Übermittlung der Besucher-IP",
        "privacy.third_country_provider" => "Anbieter mit Sitz im Drittland",
        "privacy.no_consent_platform_detected" => "Keine Einwilligungsplattform erkannt",
        "privacy.policy_link_missing" => "Link zu Datenschutzhinweisen fehlt",
        "privacy.policy_link_unusable" => "Link zu Datenschutzhinweisen unbrauchbar",
        // accessibility journey
        "a11y_journey.hidden_focusable_aria_hidden" => "Fokussierbar trotz aria-hidden",
        "a11y_journey.hidden_focusable_inert" => "Fokussierbar trotz inert",
        "a11y_journey.hidden_focusable_style" => "Fokussierbar trotz Ausblendung",
        "a11y_journey.focus_indicator_not_detected" => "Fokusanzeige nicht erkannt",
        "a11y_journey.tab_order_backward_jumps" => "Tab-Reihenfolge springt rückwärts",
        "a11y_journey.focus_trap_not_entered" => "Fokus gelangt nicht in den Dialog",
        "a11y_journey.focus_trap_background_not_hidden" => {
            "Hintergrund bei offenem Dialog nicht verborgen"
        }
        "a11y_journey.focus_trap_escaped" => "Fokus verlässt den Dialog",
        "a11y_journey.focus_trap_escape_not_closing" => "Escape schließt den Dialog nicht",
        "a11y_journey.focus_restoration_lost_to_body" => "Fokus nach Dialog verloren",
        "a11y_journey.modal_not_opened" => "Dialog öffnet sich nicht",
        "a11y_journey.menu_not_opened" => "Menü öffnet sich nicht",
        "a11y_journey.menu_focus_not_moved" => "Fokus gelangt nicht ins Menü",
        "a11y_journey.menu_escape_not_closing" => "Escape schließt das Menü nicht",
        "a11y_journey.tabs_selection_not_moved" => "Tab-Auswahl folgt dem Fokus nicht",
        "a11y_journey.tabs_focus_not_on_tab" => "Fokus nicht auf dem Tab",
        "a11y_journey.disclosure_not_opened" => "Aufklappbereich öffnet sich nicht",
        "a11y_journey.disclosure_not_closed" => "Aufklappbereich schließt sich nicht",
        "a11y_journey.disclosure_state_without_content" => "Aufklappzustand ohne Inhalt",
        "a11y_journey.disclosure_content_without_state" => "Aufklappinhalt ohne Zustand",
        "a11y_journey.spa_no_announcement_signal" => "Seitenwechsel ohne Ansage",
        "a11y_journey.spa_title_unchanged" => "Seitentitel ändert sich nicht",
        "a11y_journey.spa_focus_not_moved" => "Fokus bleibt beim Seitenwechsel",
        "a11y_journey.skip_link_focus_not_moved" => "Skip-Link verschiebt den Fokus nicht",
        "a11y_journey.form_error_silent_failure" => "Formularfehler ohne Rückmeldung",
        "a11y_journey.form_error_invalid_without_live_region" => "Formularfehler ohne Live-Region",
        "a11y_journey.form_error_unlinked_fields" => "Fehlermeldung nicht mit Feld verknüpft",
        "a11y_journey.form_error_focus_not_managed" => "Fokus bei Formularfehler nicht geführt",
        "a11y_journey.form_error_live_region_late_insertion" => {
            "Live-Region erst nach dem Fehler eingefügt"
        }
        "a11y_journey.add_to_cart_no_status_announcement" => "Warenkorb-Aktion ohne Statusansage",
        "a11y_journey.add_to_cart_no_feedback_detected" => {
            "Warenkorb-Aktion ohne erkennbare Rückmeldung"
        }
        "a11y_journey.quantity_stepper_keyboard_inoperable" => {
            "Mengenwähler nicht per Tastatur bedienbar"
        }
        "a11y_journey.quantity_stepper_value_not_exposed" => {
            "Menge für Hilfstechnik nicht erkennbar"
        }
        "a11y_journey.link_text_generic" => "Allgemeine Linktexte",
        "a11y_journey.link_text_duplicate" => "Doppelte Linktexte",
        "a11y_journey.heading_missing_h1" => "H1 fehlt",
        "a11y_journey.heading_multiple_h1" => "Mehrere H1",
        "a11y_journey.heading_level_skip" => "Übersprungene Überschriftenebene",
        "a11y_journey.landmark_missing_main" => "Hauptbereich fehlt",
        "a11y_journey.landmark_nav_without_labels" => "Navigationen ohne Bezeichnung",
        "a11y_journey.landmark_duplicate_unique" => "Doppelte Landmarken",
        "a11y_journey.media_controls_missing_name" => "Mediensteuerung ohne Namen",
        "a11y_journey.media_controls_not_reachable" => "Mediensteuerung nicht erreichbar",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each required CSP directive gets its own title naming it; two
    /// directives never share "CSP: Direktive fehlt" (#1039).
    #[test]
    fn missing_csp_directive_titles_name_the_directive() {
        let titles: Vec<String> = crate::security::CSP_REQUIRED_DIRECTIVES
            .iter()
            .map(|directive| {
                let title =
                    german_finding_title_text(&format!("security.headers.missing_{directive}"))
                        .expect("German title");
                assert!(title.contains(directive), "{title}");
                title
            })
            .collect();
        let distinct: std::collections::BTreeSet<_> = titles.iter().collect();
        assert_eq!(distinct.len(), titles.len());
    }
}
