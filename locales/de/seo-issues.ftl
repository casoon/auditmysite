# Texts of the SEO issue lists (#936): the technical issues
# (`seo::technical::collect_technical_issues`), the meta-tag findings
# (`seo::meta::meta_issue_text`) and the structured-data issues
# (`seo::schema::schema_issue_text`). One message per issue; counts reach the
# messages as numbers, so the plural agreement lives here. Which issues exist
# and their severity stay in Rust. Also the table labels of the structured-data
# issues (`seo::schema::schema_issue_label`) and of the robots.txt audit (bot
# classes and policy labels of `seo::robots`), the labels of the SEO content
# profile (`seo::profile`), the SERP status labels (`seo::serp`) and the
# structured-data fit and content-parity texts (`seo::schema_fit`,
# `seo::schema_parity`).

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

seo-schema-label-jsonld-parse-error = JSON-Syntax
seo-schema-label-jsonld-invalid-root = Wurzelstruktur
seo-schema-label-jsonld-missing-context = Schema-Kontext
seo-schema-label-jsonld-missing-type = Schema-Typ
seo-schema-label-jsonld-graph-not-array = @graph
seo-schema-label-jsonld-empty-document = Leerer Block
seo-schema-label-other = Schema.org

seo-robots-bot-class-wildcard = Alle Crawler (*)
seo-robots-bot-class-search-engine = Suchmaschine
seo-robots-bot-class-ai-training = KI-Training
seo-robots-bot-class-ai-citation = KI-Suche / Zitation
seo-robots-bot-class-ai-mixed = KI-Training & Suche
seo-robots-bot-class-unknown-ai = Unbekannter Scraper
seo-robots-bot-class-general = Allgemein
seo-robots-bot-class-unknown = Nicht klassifiziert

seo-robots-policy-everything-blocked = Alles gesperrt (Disallow: *)
seo-robots-policy-ai-fully-blocked = KI vollständig blockiert
seo-robots-policy-ai-training-blocked-conservative = KI-Training blockiert (konservativ)
seo-robots-policy-ai-training-blocked-search-allowed = KI-Training blockiert, KI-Suche erlaubt
seo-robots-policy-ai-access-open = KI-Zugang offen
seo-robots-policy-no-explicit-ai-rule = Keine explizite KI-Regel

# Labels of the SEO content profile (`seo::profile`): the maturity level with
# its description and the page type. `PageType::NavigationHub` takes the hub
# label of the page intent (labels.ftl).
seo-profile-maturity-basic = Basis
seo-profile-maturity-standard = Standard
seo-profile-maturity-advanced = Fortgeschritten
seo-profile-maturity-professional = Professionell
seo-profile-maturity-description-basic = Grundlegende SEO-Maßnahmen fehlen weitgehend.
seo-profile-maturity-description-standard = Basis-SEO ist vorhanden, fortgeschrittene Techniken fehlen.
seo-profile-maturity-description-advanced = Gute SEO-Abdeckung mit strukturierten Daten und Social Tags.
seo-profile-maturity-description-professional = Sehr breite technische SEO-Abdeckung; keine Aussage zur strategischen Content-Qualität.
seo-profile-page-type-editorial = Editorial / Artikel
seo-profile-page-type-structured-content = Strukturierter Wissensinhalt
seo-profile-page-type-marketing-landing = Marketing / Landing Page
seo-profile-page-type-media-heavy = Medienorientierte Seite
seo-profile-page-type-utility = Transaktional / Utility
seo-profile-page-type-thin-content = Thin / Minimal Content

# Status labels of the SERP signals (`seo::serp::SerpSignalStatus::label`).
seo-serp-status-ok = OK
seo-serp-status-warning = Warnung
seo-serp-status-fail = Fehler

# Labels of the structured-data fit (`seo::schema_fit`): the page kind
# (`SchemaPageKind::label`) and the fit status (`SchemaFitAssessment::fit_text`).
seo-schema-fit-page-kind-merchant-product-detail = Kaufbare Produktdetailseite
seo-schema-fit-page-kind-product-collection = Produktübersicht
seo-schema-fit-page-kind-service-detail = Leistungsdetailseite
seo-schema-fit-page-kind-software-detail = Software / SaaS
seo-schema-fit-page-kind-job-detail = Stellendetailseite
seo-schema-fit-page-kind-job-listing = Stellenübersicht
seo-schema-fit-page-kind-event-detail = Veranstaltungsdetailseite
seo-schema-fit-page-kind-event-listing = Veranstaltungsübersicht
seo-schema-fit-page-kind-faq-page = FAQ
seo-schema-fit-page-kind-person-profile = Personenprofil
seo-schema-fit-page-kind-location-detail = Standortdetailseite
seo-schema-fit-page-kind-editorial-product-review = Redaktioneller Produkttest
seo-schema-fit-page-kind-editorial-content = Redaktioneller Inhalt
seo-schema-fit-page-kind-marketing-landing-page = Marketing / Landing Page
seo-schema-fit-page-kind-corporate-page = Unternehmensseite
seo-schema-fit-page-kind-hub-page = Hub / Portal
seo-schema-fit-page-kind-lead-generation-page = Lead-Generierungsseite
seo-schema-fit-page-kind-indeterminate = Nicht bestimmt
seo-schema-fit-status-matched = Haupt-Schema passt zum erkannten Seitentyp
seo-schema-fit-status-plausible = Schema ist plausibel; der Seitentyp ist für eine strikte Anforderung nicht spezifisch genug
seo-schema-fit-status-mismatch = Haupt-Schema passt nicht zum deutlich erkannten Seitentyp
seo-schema-fit-status-indeterminate = Seitentyp und Schema-Abdeckung müssen manuell geprüft werden

# Content parity of structured data (`seo::schema_parity`): the status label
# (`ContentParityAssessment::status_text`) and the evidence
# (`ContentParityAssessment::evidence_text`); a missing value is "-" (Rust).
seo-schema-parity-status-match = Stimmt mit sichtbarem Inhalt überein
seo-schema-parity-status-mismatch = Abweichung
seo-schema-parity-status-not-evaluated = Nicht geprüft
seo-schema-parity-evidence-match = Schema und sichtbarer Inhalt stimmen überein: { $value }
seo-schema-parity-evidence-mismatch = Schema: { $schema }; sichtbar: { $visible }
seo-schema-parity-evidence-not-evaluated = Kein einzelner eindeutiger sichtbarer Wert erkannt
