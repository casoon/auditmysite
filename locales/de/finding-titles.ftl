# Titles of the findings outside the WCAG taxonomy (`audit::finding_titles`,
# #1039): one message per rule id `<source>.<check>`. The key is
# `finding-title-` plus the rule id with every character other than a letter or
# digit written as `-`. The English title is the finding's `title` in the JSON;
# the PDF and the table show the title in the run language. Checks whose title
# is page content or open-ended (budget metrics, library names, heading issues)
# have no entry here and keep their stored title.

## Best practices
finding-title-best-practices-console-error = Konsolenfehler

## SEO: meta tags
finding-title-seo-meta-title = Seitentitel
finding-title-seo-meta-description = Meta-Beschreibung
finding-title-seo-meta-viewport = Viewport-Meta-Tag
finding-title-seo-meta-lang = Seitensprache
finding-title-seo-meta-canonical = Canonical-URL
finding-title-seo-meta-twitter-card = Twitter-Card-Typ
finding-title-seo-meta-og-image = Open-Graph-Bild

## SEO: technical checks
finding-title-seo-technical-broken-internal-links = Defekte interne Links
finding-title-seo-technical-canonical-domain-mismatch = Canonical zeigt auf andere Domain
finding-title-seo-technical-form-insecure-transport = Formular über unsichere Verbindung
finding-title-seo-technical-form-password-autocomplete = Passwortfeld ohne Autocomplete-Hinweis
finding-title-seo-technical-form-sensitive-get = Sensible Daten per GET gesendet
finding-title-seo-technical-hreflang-no-self-reference = hreflang ohne Selbstverweis
finding-title-seo-technical-hreflang-no-x-default = hreflang ohne x-default
finding-title-seo-technical-internal-links-with-query-params = Interne Links mit Parametern
finding-title-seo-technical-mixed-content = Gemischte Inhalte (HTTP auf HTTPS-Seite)
finding-title-seo-technical-no-canonical = Canonical-URL fehlt
finding-title-seo-technical-no-https = Kein HTTPS
finding-title-seo-technical-no-internal-links = Keine internen Links
finding-title-seo-technical-no-lang = Sprachangabe fehlt
finding-title-seo-technical-noindex = Seite auf noindex gesetzt
finding-title-seo-technical-og-image-missing-alt = og:image ohne Alternativtext
finding-title-seo-technical-pagination-duplicate-prev-next = Doppelte Paginierungs-Links
finding-title-seo-technical-pagination-self-referential-rel-link = Paginierung verweist auf sich selbst
finding-title-seo-technical-pwa-invalid-manifest = Ungültiges Web-App-Manifest
finding-title-seo-technical-pwa-missing-manifest = Web-App-Manifest fehlt
finding-title-seo-technical-remote-google-fonts = Google Fonts von externem Server
finding-title-seo-technical-thin-content = Wenig Textinhalt
finding-title-seo-technical-twitter-image-missing-alt = twitter:image ohne Alternativtext
finding-title-seo-technical-www-no-redirect = www-Variante ohne Weiterleitung
finding-title-seo-technical-amp-missing-html-amp-attribute = AMP: html-Element ohne amp-Attribut
finding-title-seo-technical-amp-missing-canonical = AMP: Canonical-Link fehlt
finding-title-seo-technical-amp-missing-amp-runtime = AMP: Runtime-Script fehlt
finding-title-seo-technical-amp-missing-amp-boilerplate = AMP: Boilerplate-CSS fehlt
finding-title-seo-technical-amp-custom-script-not-allowed = AMP: eigenes Script nicht erlaubt

## SEO: structured data
finding-title-seo-schema-jsonld-parse-error = Strukturierte Daten: ungültiges JSON-LD
finding-title-seo-schema-jsonld-invalid-root = Strukturierte Daten: ungültige Wurzelstruktur
finding-title-seo-schema-jsonld-missing-context = Strukturierte Daten: schema.org-Kontext fehlt
finding-title-seo-schema-jsonld-missing-type = Strukturierte Daten: Knoten ohne @type
finding-title-seo-schema-jsonld-graph-not-array = Strukturierte Daten: @graph ist keine Liste
finding-title-seo-schema-jsonld-empty-document = Strukturierte Daten: leerer JSON-LD-Block
finding-title-seo-schema-rule-missing = { $feature }: Pflichtangabe fehlt

## SEO: page health
finding-title-seo-page-health-broken-fragment-links = Defekte Sprungmarken
finding-title-seo-page-health-charset-mismatch = Zeichensatz widersprüchlich
finding-title-seo-page-health-custom-404-invalid-status = 404-Seite mit falschem Statuscode
finding-title-seo-page-health-deprecated-apis = Veraltete Browser-Schnittstellen
finding-title-seo-page-health-document-write = document.write() im Einsatz
finding-title-seo-page-health-dynamic-url = Dynamische URL
finding-title-seo-page-health-excessive-dom = DOM zu groß
finding-title-seo-page-health-font-display-missing = font-display fehlt
finding-title-seo-page-health-fonts-not-preloaded = Schriften nicht vorgeladen
finding-title-seo-page-health-frames = Frames im Einsatz
finding-title-seo-page-health-generic-404-page = Generische 404-Seite
finding-title-seo-page-health-generic-link-text = Allgemeine Linktexte
finding-title-seo-page-health-gif-images = Animierte GIF-Bilder
finding-title-seo-page-health-hreflang-invalid = Ungültige hreflang-Angaben
finding-title-seo-page-health-http1-only = Nur HTTP/1.1
finding-title-seo-page-health-http-not-redirected-to-https = HTTP wird nicht auf HTTPS umgeleitet
finding-title-seo-page-health-images-without-dimensions = Bilder ohne Größenangaben
finding-title-seo-page-health-images-without-modern-format = Bilder ohne modernes Format
finding-title-seo-page-health-images-without-srcset = Bilder ohne srcset
finding-title-seo-page-health-incorrect-content-type = Falscher Content-Type
finding-title-seo-page-health-inefficient-cache = HTML-Dokument ohne wirksames Caching
finding-title-seo-page-health-inefficient-resource-cache = Statische Ressourcen ohne effizientes Caching
finding-title-seo-page-health-jsonld-invalid = Ungültiges JSON-LD
finding-title-seo-page-health-large-dom = DOM erhöht
finding-title-seo-page-health-lcp-image-lazy-loaded = LCP-Bild wird verzögert geladen
finding-title-seo-page-health-lcp-image-without-fetchpriority = LCP-Bild ohne fetchpriority
finding-title-seo-page-health-lcp-image-without-preload = LCP-Bild ohne Preload
finding-title-seo-page-health-meta-refresh = Meta-Refresh-Weiterleitung
finding-title-seo-page-health-missing-compression = Komprimierung fehlt
finding-title-seo-page-health-missing-doctype = Doctype fehlt
finding-title-seo-page-health-missing-preconnect = Preconnect fehlt
finding-title-seo-page-health-missing-sri = Subresource Integrity fehlt
finding-title-seo-page-health-multiple-redirects = Mehrfache Weiterleitungen
finding-title-seo-page-health-non-crawlable-links = Nicht crawlbare Links
finding-title-seo-page-health-offscreen-images-without-lazy = Bilder außerhalb des Sichtbereichs ohne Lazy Loading
finding-title-seo-page-health-orphaned-preload = Preload ohne Verwendung
finding-title-seo-page-health-oversized-images = Zu große Bilder
finding-title-seo-page-health-paste-blocked-password = Einfügen im Passwortfeld blockiert
finding-title-seo-page-health-redirect = Weiterleitung
finding-title-seo-page-health-soft-404 = Soft-404
finding-title-seo-page-health-sync-head-scripts = Blockierende Skripte im Head
finding-title-seo-page-health-trailing-slash-inconsistent = Uneinheitlicher Schrägstrich am URL-Ende
finding-title-seo-page-health-url-too-deep = URL zu tief verschachtelt
finding-title-seo-page-health-url-too-long = URL zu lang
finding-title-seo-page-health-www-not-consolidated = www-Variante nicht zusammengeführt

## Security headers
finding-title-security-headers-missing-https = HTTPS fehlt
finding-title-security-headers-missing-header = Sicherheits-Header fehlt
finding-title-security-headers-cors-wildcard-credentials = CORS: Wildcard mit Zugangsdaten
finding-title-security-headers-hsts-preload-ineligible = HSTS-Preload nicht erfüllt
finding-title-security-headers-permissions-policy-permissive = Permissions-Policy zu offen
finding-title-security-headers-public-source-map = Öffentliche Source Maps
finding-title-security-headers-unsafe-inline-script = CSP erlaubt Inline-Skripte
finding-title-security-headers-unsafe-eval-script = CSP erlaubt eval()
finding-title-security-headers-wildcard-script-source = CSP erlaubt Wildcard-Skriptquellen
finding-title-security-headers-unsafe-inline-style = CSP erlaubt Inline-Styles
finding-title-security-headers-wildcard-source = CSP erlaubt Wildcard-Quellen
finding-title-security-csp-missing-directive = CSP: { $directive }-Direktive fehlt

## Mobile
finding-title-mobile-missing-viewport = Viewport fehlt
finding-title-mobile-improper-viewport = Viewport falsch konfiguriert
finding-title-mobile-not-scalable = Zoomen unterbunden
finding-title-mobile-small-targets = Zu kleine Bedienelemente
finding-title-mobile-small-fonts = Zu kleine Schrift
finding-title-mobile-horizontal-scroll = Horizontales Scrollen

## HTML conformance: one title per rule family (`html_conform::recommendation`);
## a Schematron assertion also names its rule id, so two rules of one family
## never share a title.
finding-title-html-conform-parser = HTML-Syntaxfehler
finding-title-html-conform-schema = Element oder Attribut an dieser Stelle nicht erlaubt
finding-title-html-conform-ids = Doppelte ID
finding-title-html-conform-headings = Überschriftenstruktur verletzt den Standard
finding-title-html-conform-language = Sprachangabe der Seite ungültig
finding-title-html-conform-image-alt = Bild ohne Textalternative
finding-title-html-conform-responsive-images = Ungültige srcset-, sizes- oder source-Angaben
finding-title-html-conform-image-maps = Ungültige Bildkarte
finding-title-html-conform-nested-interactive = Verschachtelte interaktive Elemente
finding-title-html-conform-forms = Ungültige Formularauszeichnung
finding-title-html-conform-document-meta = Ungültige Metadaten im Dokumentkopf
finding-title-html-conform-link-rel = Ungültiges link-Element
finding-title-html-conform-scripts = Ungültige Skript-Attribute oder Skriptart
finding-title-html-conform-csp = Content-Security-Policy im meta-Element verletzt
finding-title-html-conform-tables = Unregelmäßige Tabellenstruktur
finding-title-html-conform-aria-redundant-role = Überflüssige ARIA-Rolle
finding-title-html-conform-aria-naming = ARIA-Name, wo keiner erlaubt ist
finding-title-html-conform-aria-state = Unzulässiges ARIA-Attribut oder unzulässiger Zustand
finding-title-html-conform-aria-structure = ARIA-Rolle außerhalb ihres Zusammenhangs
finding-title-html-conform-obsolete = Veraltetes HTML-Element
finding-title-html-conform-microdata = Unvollständige Microdata-Attribute
finding-title-html-conform-meter-progress = Ungültiger Wertebereich von meter oder progress
finding-title-html-conform-sectioning = Gliederungselemente falsch verschachtelt
finding-title-html-conform-media-text = Medien- oder Textelement falsch eingesetzt
finding-title-html-conform-text-normalization = Text nicht in Unicode-Normalform C
finding-title-html-conform-generic = HTML-Konformitätsverstoß
finding-title-html-conform-rule = { $family } ({ $rule })

## Screen reader reading sequence: one title per WCAG criterion
finding-title-screen-reader-wcag-1-3-1 = Screenreader: Struktur nicht vermittelt
finding-title-screen-reader-wcag-1-3-6 = Screenreader: Seitenbereiche nicht erkennbar
finding-title-screen-reader-wcag-2-4-1 = Screenreader: kein Skip-Link
finding-title-screen-reader-wcag-2-4-4 = Screenreader: unklare Link- oder Schaltflächennamen
finding-title-screen-reader-wcag-3-3-2 = Screenreader: Formularfelder ohne Beschriftung
finding-title-screen-reader-wcag-4-1-2 = Screenreader: Bedienelemente ohne Namen
finding-title-screen-reader-reading-sequence = Screenreader-Lesereihenfolge

## UX
finding-title-ux-no-cta = Kein erkennbarer Call-to-Action
finding-title-ux-competing-ctas = Konkurrierende Handlungsaufforderungen
finding-title-ux-generic-links = Allgemeine Linktexte
finding-title-ux-no-h1 = H1 fehlt
finding-title-ux-multiple-h1 = Mehrere H1
finding-title-ux-heading-skips = Übersprungene Überschriftenebenen
finding-title-ux-large-dom = Großer DOM
finding-title-ux-little-text = Wenig Text
finding-title-ux-no-subheadings = Keine Zwischenüberschriften
finding-title-ux-no-contact = Kein Kontakt erkennbar
finding-title-ux-no-imprint = Impressum nicht erkennbar
finding-title-ux-no-privacy = Datenschutz nicht erkennbar
finding-title-ux-weak-trust-signals = Schwache Vertrauenssignale
finding-title-ux-too-many-links = Zu viele Links
finding-title-ux-too-many-interactive = Zu viele interaktive Elemente

## Journey
finding-title-journey-no-h1 = H1 fehlt
finding-title-journey-multiple-h1 = Mehrere H1
finding-title-journey-vague-h1 = H1 wenig aussagekräftig
finding-title-journey-no-page-title = Seitentitel fehlt
finding-title-journey-little-early-text = Wenig Text im oberen Bereich
finding-title-journey-navigation-missing = Navigation fehlt
finding-title-journey-main-hidden-from-ax = Hauptbereich für Hilfstechnik verborgen
finding-title-journey-main-missing = Hauptbereich fehlt
finding-title-journey-few-subheadings = Wenige Zwischenüberschriften
finding-title-journey-no-breadcrumb = Kein Brotkrumen-Pfad
finding-title-journey-no-footer = Kein Footer
finding-title-journey-generic-links = Allgemeine Linktexte
finding-title-journey-empty-links = Links ohne Text
finding-title-journey-duplicate-links = Doppelte Linktexte
finding-title-journey-too-many-links = Zu viele Links
finding-title-journey-unnamed-buttons = Schaltflächen ohne Namen
finding-title-journey-unlabeled-forms = Formularfelder ohne Beschriftung
finding-title-journey-generic-buttons = Allgemeine Schaltflächentexte
finding-title-journey-no-call-to-action = Kein Call-to-Action
finding-title-journey-competing-ctas = Konkurrierende Handlungsaufforderungen
finding-title-journey-blocking-dialog = Blockierender Dialog
finding-title-journey-complex-form = Komplexes Formular

## Commerce
finding-title-commerce-missing-price = Preis fehlt
finding-title-commerce-missing-availability = Verfügbarkeit fehlt
finding-title-commerce-missing-shipping-details = Versandangaben fehlen
finding-title-commerce-missing-return-policy = Rückgabebedingungen fehlen
finding-title-commerce-missing-reviews = Bewertungen fehlen
finding-title-commerce-missing-impressum-link = Impressum-Link fehlt
finding-title-commerce-missing-agb-link = AGB-Link fehlt
finding-title-commerce-missing-widerruf-link = Widerrufs-Link fehlt
finding-title-commerce-missing-shipping-page-link = Versand-Link fehlt
finding-title-commerce-missing-payment-link = Zahlungs-Link fehlt
finding-title-commerce-missing-contact-link = Kontakt-Link fehlt

## Privacy
finding-title-privacy-tracking-request-before-consent = Tracking-Anfrage vor Einwilligung
finding-title-privacy-non-essential-storage-before-consent = Nicht notwendige Speicherung vor Einwilligung
finding-title-privacy-visitor-ip-transfer = Übermittlung der Besucher-IP
finding-title-privacy-third-country-provider = Anbieter mit Sitz im Drittland
finding-title-privacy-no-consent-platform-detected = Keine Einwilligungsplattform erkannt
finding-title-privacy-policy-link-missing = Link zu Datenschutzhinweisen fehlt
finding-title-privacy-policy-link-unusable = Link zu Datenschutzhinweisen unbrauchbar

## Accessibility journey
finding-title-a11y-journey-hidden-focusable-aria-hidden = Fokussierbar trotz aria-hidden
finding-title-a11y-journey-hidden-focusable-inert = Fokussierbar trotz inert
finding-title-a11y-journey-hidden-focusable-style = Fokussierbar trotz Ausblendung
finding-title-a11y-journey-focus-indicator-not-detected = Fokusanzeige nicht erkannt
finding-title-a11y-journey-tab-order-backward-jumps = Tab-Reihenfolge springt rückwärts
finding-title-a11y-journey-focus-trap-not-entered = Fokus gelangt nicht in den Dialog
finding-title-a11y-journey-focus-trap-background-not-hidden = Hintergrund bei offenem Dialog nicht verborgen
finding-title-a11y-journey-focus-trap-escaped = Fokus verlässt den Dialog
finding-title-a11y-journey-focus-trap-escape-not-closing = Escape schließt den Dialog nicht
finding-title-a11y-journey-focus-restoration-lost-to-body = Fokus nach Dialog verloren
finding-title-a11y-journey-modal-not-opened = Dialog öffnet sich nicht
finding-title-a11y-journey-menu-not-opened = Menü öffnet sich nicht
finding-title-a11y-journey-menu-focus-not-moved = Fokus gelangt nicht ins Menü
finding-title-a11y-journey-menu-escape-not-closing = Escape schließt das Menü nicht
finding-title-a11y-journey-tabs-selection-not-moved = Tab-Auswahl folgt dem Fokus nicht
finding-title-a11y-journey-tabs-focus-not-on-tab = Fokus nicht auf dem Tab
finding-title-a11y-journey-disclosure-not-opened = Aufklappbereich öffnet sich nicht
finding-title-a11y-journey-disclosure-not-closed = Aufklappbereich schließt sich nicht
finding-title-a11y-journey-disclosure-state-without-content = Aufklappzustand ohne Inhalt
finding-title-a11y-journey-disclosure-content-without-state = Aufklappinhalt ohne Zustand
finding-title-a11y-journey-spa-no-announcement-signal = Seitenwechsel ohne Ansage
finding-title-a11y-journey-spa-title-unchanged = Seitentitel ändert sich nicht
finding-title-a11y-journey-spa-focus-not-moved = Fokus bleibt beim Seitenwechsel
finding-title-a11y-journey-skip-link-focus-not-moved = Skip-Link verschiebt den Fokus nicht
finding-title-a11y-journey-form-error-silent-failure = Formularfehler ohne Rückmeldung
finding-title-a11y-journey-form-error-invalid-without-live-region = Formularfehler ohne Live-Region
finding-title-a11y-journey-form-error-unlinked-fields = Fehlermeldung nicht mit Feld verknüpft
finding-title-a11y-journey-form-error-focus-not-managed = Fokus bei Formularfehler nicht geführt
finding-title-a11y-journey-form-error-live-region-late-insertion = Live-Region erst nach dem Fehler eingefügt
finding-title-a11y-journey-add-to-cart-no-status-announcement = Warenkorb-Aktion ohne Statusansage
finding-title-a11y-journey-add-to-cart-no-feedback-detected = Warenkorb-Aktion ohne erkennbare Rückmeldung
finding-title-a11y-journey-quantity-stepper-keyboard-inoperable = Mengenwähler nicht per Tastatur bedienbar
finding-title-a11y-journey-quantity-stepper-value-not-exposed = Menge für Hilfstechnik nicht erkennbar
finding-title-a11y-journey-link-text-generic = Allgemeine Linktexte
finding-title-a11y-journey-link-text-duplicate = Doppelte Linktexte
finding-title-a11y-journey-heading-missing-h1 = H1 fehlt
finding-title-a11y-journey-heading-multiple-h1 = Mehrere H1
finding-title-a11y-journey-heading-level-skip = Übersprungene Überschriftenebene
finding-title-a11y-journey-landmark-missing-main = Hauptbereich fehlt
finding-title-a11y-journey-landmark-nav-without-labels = Navigationen ohne Bezeichnung
finding-title-a11y-journey-landmark-duplicate-unique = Doppelte Landmarken
finding-title-a11y-journey-media-controls-missing-name = Mediensteuerung ohne Namen
finding-title-a11y-journey-media-controls-not-reachable = Mediensteuerung nicht erreichbar
