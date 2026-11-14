# Texts of the page-health issues (`seo::page_health::collect_issues`, #936):
# one message per issue type. Counts reach the messages as numbers, so the
# plural agreement lives here; the thresholds and severities stay in Rust.
# Also the HTML validation texts of the SEO view (end of the file).

page-health-missing-doctype = Fehlende HTML5-Doctype-Deklaration — Browser rendert im Quirks-Mode
page-health-document-write =
    { $count } Inline-{ $count ->
        [one] Script
       *[other] Scripts
    } verwenden document.write() — blockiert HTML-Parsing
page-health-excessive-dom = DOM-Größe kritisch: { $nodes } HTML-Elemente (Empfehlung: <1500, max Tiefe: { $depth })
page-health-large-dom = DOM-Größe erhöht: { $nodes } HTML-Elemente (Empfehlung: <800, max Tiefe: { $depth })
page-health-images-without-dimensions = { $count } <img>-Elemente ohne explizite width/height — CLS-Risiko
page-health-paste-blocked-password =
    { $count } { $count ->
        [one] Passwortfeld
       *[other] Passwortfelder
    } blockieren Einfügen (onpaste-Handler) — beeinträchtigt Passwort-Manager
page-health-offscreen-images-without-lazy = { $count } Bilder unterhalb des Viewports ohne loading="lazy" — verzögern ersten Seitenaufbau
page-health-images-without-srcset = { $count } <img>-Elemente ohne srcset — keine responsiven Bildvarianten für verschiedene Auflösungen
page-health-missing-preconnect = { $count } externe Origins ohne <link rel="preconnect">{ $sample }
page-health-non-crawlable-links = { $count } Links nicht crawlbar (javascript:, leer, kein href) — PageRank-Verlust
page-health-images-without-modern-format = { $count } Bilder als JPEG/PNG ohne WebP/AVIF-Alternative — erhöhte Ladezeit
page-health-oversized-images = { count-images-scaled-down } deutlich kleiner dargestellt als ihre natürliche Auflösung
page-health-gif-images =
    { $count } { $count ->
        [one] GIF-Bild
       *[other] GIF-Bilder
    } gefunden — ggf. als MP4/WebM ersetzen (80–95 % kleiner)
page-health-font-display-missing = { $count } @font-face-Regeln ohne font-display: swap/fallback/optional — FOIT-Risiko
page-health-fonts-not-preloaded =
    { $count } von { $total } { $total ->
        [one] Web-Font
       *[other] Web-Fonts
    } nicht per <link rel="preload" as="font"> vorgeladen — verzögert ersten Textaufbau
page-health-orphaned-preload = { $count } <link rel="preload">-Hinweise auf nicht geladene Ressourcen (orphaned)
page-health-lcp-image-lazy-loaded = LCP-Bildkandidat hat loading="lazy" — verzögert den Largest Contentful Paint
page-health-lcp-image-without-preload = Größtes sichtbares Bild{ $url } hat keinen <link rel="preload" as="image">-Hint
page-health-lcp-image-without-fetchpriority = LCP-Bildkandidat fehlt fetchpriority="high" — Ladepriorität nicht signalisiert
page-health-deprecated-apis =
    { $count } veraltete { $count ->
        [one] Browser-API
       *[other] Browser-APIs
    } in Inline-Scripts erkannt
page-health-http1-only = Seite wird über HTTP/1.1 ausgeliefert — HTTP/2 ermöglicht Multiplexing und Header-Komprimierung
page-health-missing-compression = Seiteninhalt wird ohne Komprimierung übertragen (kein gzip/brotli)
page-health-inefficient-cache = Cache-Control ohne effektive Caching-Dauer: { $value }
page-health-inefficient-resource-cache =
    { $inefficient } von { $total } statischen { $total ->
        [one] Ressource
       *[other] Ressourcen
    } ohne effiziente Cache-Policy{ $sample }
page-health-incorrect-content-type =
    { $total } statische { $total ->
        [one] Ressource
       *[other] Ressourcen
    } mit fehlendem oder falschem Content-Type-Header{ $sample }
page-health-charset-mismatch = Charset-Konflikt: HTTP-Header deklariert "{ $header }", <meta charset> deklariert "{ $declared }"
page-health-hreflang-invalid = { $count ->
        [one] { $count } hreflang-Eintrag mit ungültigem Sprachcode
       *[other] { $count } hreflang-Einträge mit ungültigem Sprachcode
    }
page-health-jsonld-invalid = { $count } JSON-LD-Blöcke ohne @context oder @type
page-health-sync-head-scripts =
    { $count } { $count ->
        [one] Script
       *[other] Scripts
    } im <head> ohne defer/async/module sind syntaktisch render-blockierend; ob sie messbar verzögern, zeigt die Render-Blocking-Analyse.
page-health-missing-sri =
    { $count } externe { $count ->
        [one] Ressource
       *[other] Ressourcen
    } ohne Subresource Integrity (integrity-Attribut fehlt)
page-health-broken-fragment-links =
    { $count } Anker-{ $count ->
        [one] Link
       *[other] Links
    } verweisen auf nicht existierende IDs{ $sample }
page-health-generic-link-text =
    { $count } { $count ->
        [one] Link
       *[other] Links
    } mit nicht-beschreibendem Text ("hier", "mehr", "click here" u.ä.)
page-health-soft-404 = Server gibt HTTP { $status } für nicht-existierende URLs zurück (Soft 404)
page-health-custom-404-invalid-status = Nicht-existente URL liefert HTTP { $status } statt 404/410
page-health-generic-404-page = 404-Seite wirkt generisch oder sehr knapp; eine hilfreiche Custom-404-Seite verbessert Orientierung und Crawling-Signale
page-health-meta-refresh-immediate = Sofort-Weiterleitung via meta-refresh (SEO-schädlich, nutze 301-Redirect)
page-health-meta-refresh-delayed = meta-refresh mit { $delay }s Verzögerung gefunden
page-health-frames = { $count } veraltete <frame>-Elemente gefunden
page-health-url-too-long = URL mit { $length } Zeichen überschreitet Empfehlung (>115)
page-health-url-too-deep = URL-Pfadtiefe { $depth } überschreitet Empfehlung (>5 Ebenen)
page-health-dynamic-url = URL enthält Query-Parameter (dynamische URL)
page-health-multiple-redirects = { $count } HTTP-Weiterleitungen vor der finalen Seite — jeder Hop kostet ~100–300 ms
page-health-redirect = Seite leitet weiter zu: { $url }
page-health-www-not-consolidated = www und non-www Version sind nicht konsolidiert (kein 301-Redirect)
page-health-trailing-slash-inconsistent = URL ist mit und ohne abschließendem Slash als zwei getrennte 200-Antworten erreichbar — Duplicate-Content-Risiko ohne kanonischen Redirect
page-health-http-not-redirected-to-https = http://-Version leitet nicht auf https:// weiter (Status: { $status })

# Parts the messages above take as arguments. The samples start with a space
# (`{" "}`) because they are appended to a sentence.
page-health-sample-examples = {" "}(z.B. { $list })
page-health-sample-cache-resource = {" "}(z.B. { $url }: { $reason })
page-health-sample-content-type = {" "}(z.B. { $url }: erwartet { $expected }, erhalten { $actual })
page-health-none = keiner
page-health-unreachable = nicht erreichbar
page-health-unknown-url = (unbekannt)

# Why a static resource's cache policy is inefficient (`cache_reason_text`).
page-health-cache-reason-missing = Cache-Control fehlt
page-health-cache-reason-disabled = Caching ist ausdrücklich abgeschaltet
page-health-cache-reason-short = max-age/s-maxage fehlt oder ist zu kurz

# HTML validation findings of the SEO view (`html_validation_check_text`,
# `html_validation_detail_text`): the check label per kind and its detail.
# The sample lists are joined in Rust; `-more` is used only when samples were
# left out. The German "(z.B. …)" and "weitere" parts are also the shape the
# legacy cache parser in `page_health.rs` reads.
page-health-html-validation-check-duplicate-ids = Doppelte IDs
page-health-html-validation-check-images-without-alt = Bilder ohne alt-Attribut
page-health-html-validation-check-tables-without-headers = Tabellen ohne Kopfzeile
page-health-html-validation-check-empty-headings = Leere Überschriften
page-health-html-validation-check-nested-interactive = Verschachtelte interaktive Elemente
page-health-html-validation-check-parse-errors = HTML5-Parsing-Fehler
page-health-html-validation-detail-duplicate-ids = { $count } gefunden
page-health-html-validation-detail-duplicate-ids-examples = { $count } (z.B. { $list })
page-health-html-validation-detail-images-without-alt = { $count } <img> ohne alt
page-health-html-validation-detail-tables-without-headers = { $count } <table> ohne <th> oder <caption>
page-health-html-validation-detail-empty-headings = { $count } leere h1–h6 Elemente
page-health-html-validation-detail-nested-interactive = { $count } button/a in button/a
page-health-html-validation-detail-parse-errors-more = { $list } | +{ $more } weitere

# Detail of the SEO view's HTML validation status
# (`html_validator_delegated_text`, `html_validator_executed_text`).
page-health-html-validator-delegated = Die HTML5-Validierung ist Teil der HTML-Konformitätsprüfung (html-conform), standardmäßig aktiv oder mit --modules html-conform
page-health-html-validator-executed = HTML5-Validierung lokal via html5ever
