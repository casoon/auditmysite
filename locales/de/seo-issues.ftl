# Texts of the SEO issue lists (#936): the technical issues
# (`seo::technical::collect_technical_issues`), the meta-tag findings
# (`seo::meta::meta_issue_text`) and the structured-data issues
# (`seo::schema::schema_issue_text`). One message per issue; counts reach the
# messages as numbers, so the plural agreement lives here. Which issues exist
# and their severity stay in Rust.

seo-technical-no-https = Seite wird nicht über HTTPS ausgeliefert
seo-technical-no-canonical = Fehlende Canonical-URL
seo-technical-no-lang = Fehlendes lang-Attribut am html-Element
seo-technical-thin-content = Seite hat dünnen Inhalt ({ $words } Wörter, empfohlen: 300+)
seo-technical-no-internal-links = Seite hat keine internen Links
seo-technical-remote-google-fonts = Extern gehostete Google Fonts erkannt
seo-technical-noindex = Seite hat noindex-Direktive — wird von Suchmaschinen nicht indexiert
seo-technical-hreflang-no-x-default = hreflang-Annotierungen ohne x-default-Eintrag
seo-technical-hreflang-no-self-reference = hreflang-Set enthält keine Self-Reference für die aktuelle Seite
seo-technical-internal-links-with-query-params =
    { $count } interne { $count ->
        [one] Link
       *[other] Links
    } mit Query-Parametern — können Crawl-Budget verwässern
seo-technical-pagination-self-referential-rel-link = Pagination-rel-Link zeigt auf die aktuelle Seite statt auf eine Nachbarseite
seo-technical-pagination-duplicate-prev-next = Pagination rel="prev" und rel="next" zeigen auf dieselbe URL
seo-technical-broken-internal-links =
    { $count } defekte interne { $count ->
        [one] Link
       *[other] Links
    } erkannt: { $list }
seo-technical-mixed-content =
    { $count } { $count ->
        [one] HTTP-Subressource
       *[other] HTTP-Subressourcen
    } auf HTTPS-Seite referenziert ({ $blockable } blockierbar): { $samples }
seo-technical-og-image-missing-alt = og:image ist gesetzt, aber ohne og:image:alt-Beschreibung für das Vorschaubild
seo-technical-twitter-image-missing-alt = twitter:image ist gesetzt, aber ohne twitter:image:alt-Beschreibung für das Vorschaubild
seo-technical-pwa-missing-manifest = Fehlendes Web-App-Manifest
seo-technical-pwa-invalid-manifest = Web-App-Manifest fehlen erforderliche PWA-Felder
seo-technical-amp-missing-html-amp-attribute = AMP-Markup erkannt, aber das html-Element hat kein amp-Attribut
seo-technical-amp-missing-canonical = AMP-Seite hat keinen Canonical-Link
seo-technical-amp-missing-amp-runtime = AMP-Seite bindet das AMP-Runtime-Script nicht ein
seo-technical-amp-missing-amp-boilerplate = AMP-Seite enthält kein amp-boilerplate-CSS
seo-technical-amp-custom-script-not-allowed = AMP-Seite enthält ein eigenes Script außerhalb der AMP-Runtime
seo-technical-form-insecure-transport =
    { $count } { $count ->
        [one] sensibles Formular
       *[other] sensible Formulare
    } senden an HTTP oder werden über HTTP ausgeliefert
seo-technical-form-sensitive-get =
    { $count } { $count ->
        [one] sensibles Formular
       *[other] sensible Formulare
    } verwenden GET und können Werte in URLs und Logs preisgeben
seo-technical-form-password-autocomplete =
    { $count } { $count ->
        [one] Passwortfeld
       *[other] Passwortfelder
    } haben autocomplete deaktiviert oder fehlkonfiguriert
seo-technical-www-no-redirect = Sowohl { $primary } als auch { $alternative } liefern Inhalt (HTTP 200). Richte eine 301-Weiterleitung von der Alternative auf die primäre Variante ein, um Duplicate Content zu vermeiden.{ $note }
seo-technical-www-note-canonical-ok = {" "}Canonical-Tag ist korrekt gesetzt, ein 301 wird dennoch empfohlen — Canonicals sind Hinweise, keine Direktiven.
seo-technical-www-note-canonical-other = {" "}Zusätzlich zeigt das Canonical nicht auf die primäre Domain.
seo-technical-www-note-canonical-none = {" "}Kein Canonical-Tag gefunden.
seo-technical-canonical-domain-mismatch = Canonical-URL zeigt auf eine andere Domain-Variante als die geprüfte URL ({ $primary } vs. Canonical).

seo-meta-missing-title = Seitentitel fehlt
seo-meta-title-too-short = Titel ist zu kurz ({ $length } Zeichen, Empfehlung: 30-60)
seo-meta-title-too-long = Titel ist zu lang ({ $length } Zeichen, Empfehlung: 30-60)
seo-meta-missing-description = Meta-Beschreibung fehlt
seo-meta-description-too-short = Beschreibung ist zu kurz ({ $length } Zeichen, Empfehlung: 120-160)
seo-meta-description-too-long = Beschreibung ist zu lang ({ $length } Zeichen, Empfehlung: 120-160)
seo-meta-missing-viewport = Viewport-Meta-Tag fehlt
seo-meta-missing-lang = lang-Attribut am <html>-Element fehlt
seo-meta-missing-canonical = Canonical-URL fehlt
seo-meta-invalid-twitter-card = Ungültiger twitter:card-Wert "{ $value }" (erlaubt: { $allowed })
seo-meta-relative-og-image = og:image ist keine absolute URL: "{ $value }"

seo-schema-jsonld-parse-error = Der JSON-LD-Block enthält ungültiges JSON und kann nicht ausgewertet werden.
seo-schema-jsonld-invalid-root = Die JSON-LD-Wurzel muss ein Objekt oder eine Liste von Objekten sein.
seo-schema-jsonld-missing-context = Kein schema.org-@context gefunden; die Schema-Begriffe sind nicht zuverlässig interpretierbar.
seo-schema-jsonld-missing-type = Ein JSON-LD-Knoten hat keinen @type und kann keinem Schema-Typ zugeordnet werden.
seo-schema-jsonld-graph-not-array = Der Wert von @graph muss eine Liste von JSON-LD-Knoten sein.
seo-schema-jsonld-empty-document = Der JSON-LD-Block enthält keine Knoten.
seo-schema-recommended-missing = { $type }: Empfohlenes Feld "{ $property }" fehlt
seo-schema-required-missing = { $type }: Pflichtfeld "{ $property }" fehlt
