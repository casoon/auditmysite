# Texts of the SEO issue lists (#936): the technical issues
# (`seo::technical::collect_technical_issues`), the meta-tag findings
# (`seo::meta::meta_issue_text`) and the structured-data issues
# (`seo::schema::schema_issue_text`). One message per issue; counts reach the
# messages as numbers, so the plural agreement lives here. Which issues exist
# and their severity stay in Rust. Also the table labels of the structured-data
# issues (`seo::schema::schema_issue_label`) and of the robots.txt audit (bot
# classes and policy labels of `seo::robots`), the labels and texts of the SEO
# content profile (`seo::profile`), the SERP status labels (`seo::serp`) and the
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

# Texts of the SEO content profile (`seo::profile::build_content_profile`):
# the page attributes, the content type, the summary without meta
# information, the entry count of a generic schema field and the names,
# check labels and details of the signal categories. Which attribute, check
# and detail applies stays in Rust; language-neutral names (Meta-Tags, Social
# Media, HTTPS, Canonical URL, Website) are written there.
seo-profile-attribute-text-rich = textstark
seo-profile-attribute-very-short = sehr kurz
seo-profile-attribute-structured = strukturiert
seo-profile-attribute-navigation-heavy = navigationslastig
seo-profile-attribute-visually-oriented = visuell geprägt
seo-profile-attribute-knowledge-oriented = wissensorientiert
seo-profile-attribute-conversion-oriented = conversionorientiert
seo-profile-attribute-purpose-bound = zweckgebunden
seo-profile-attribute-thin = dünn
seo-profile-content-type-article = Artikel
seo-profile-content-type-product = Produkt
seo-profile-content-type-profile = Profil
seo-profile-content-type-article-blog = Artikel / Blog
seo-profile-content-type-ecommerce-product = E-Commerce / Produkt
seo-profile-content-type-local-business = Lokales Unternehmen
seo-profile-content-type-event = Veranstaltung
seo-profile-content-type-recipe = Rezept
seo-profile-content-type-faq = FAQ / Informationsseite
seo-profile-summary-none = Keine Meta-Informationen gefunden.
seo-profile-generic-array-entries = [{ $count } Einträge]
seo-profile-detail-characters = { $count } Zeichen
seo-profile-detail-missing = fehlt
seo-profile-detail-present = vorhanden
seo-profile-detail-active = aktiv
seo-profile-check-title = Title vorhanden & Länge optimal
seo-profile-check-description = Description vorhanden & Länge optimal
seo-profile-check-viewport = Viewport konfiguriert
seo-profile-check-charset = Charset definiert
seo-profile-check-lang-set = Sprache (lang) gesetzt
seo-profile-category-headings = Überschriften
seo-profile-check-single-h1 = Genau ein H1
seo-profile-detail-h1-count = { $count } H1-Tags gefunden
seo-profile-check-h1-text = H1-Text vorhanden
seo-profile-detail-no-h1-text = kein H1-Text
seo-profile-check-no-skipped-levels = Keine übersprungenen Ebenen
seo-profile-detail-no-gaps = keine Lücken
seo-profile-detail-levels-skipped = Ebenen übersprungen
seo-profile-check-logical-hierarchy = Logische Hierarchie
seo-profile-detail-no-issues = keine Probleme
seo-profile-detail-order-problems =
    { $count } { $count ->
        [one] Problem
       *[other] Probleme
    }
seo-profile-detail-order-problems-first-heading =
    { $count } { $count ->
        [one] Problem
       *[other] Probleme
    }, erste Überschrift ist H{ $level }
seo-profile-check-og-present = OpenGraph-Tags vorhanden
seo-profile-check-og-complete = OpenGraph ≥ 80% vollständig
seo-profile-check-twitter-present = Twitter Card vorhanden
seo-profile-check-twitter-complete = Twitter Card ≥ 75% vollständig
seo-profile-category-technical = Technisch
seo-profile-check-lang = Sprache (lang)
seo-profile-check-hreflang = Hreflang (Mehrsprachigkeit)
seo-profile-detail-hreflang-languages = { $count } Sprachen
seo-profile-detail-no-hreflang = keine Hreflang-Tags (bei einsprachigen Seiten nicht nötig)
seo-profile-check-robots-indexing = Robots erlaubt Indexierung
seo-profile-detail-no-robots-meta = kein Robots-Meta-Tag (Standard: index)
seo-profile-category-structured-data = Strukturierte Daten
seo-profile-check-structured-data = Strukturierte Daten vorhanden
seo-profile-detail-no-structured-data = keine strukturierten Daten
seo-profile-detail-jsonld-not-evaluable = JSON-LD erkannt, aber nicht auswertbar
seo-profile-detail-microdata-only = strukturierte Daten erkannt (Microdata/RDFa, kein JSON-LD)
seo-profile-check-rich-result = Rich-Result-Typ erkannt
seo-profile-detail-rich-result-types = { $types } (nur Typ-Erkennung; Eignung erfordert zusätzlich vollständige, passende Inhalte)
seo-profile-detail-no-rich-result-types = keine Rich-Result-Typen erkannt
seo-profile-check-local-business-nap = LocalBusiness NAP vollständig
seo-profile-detail-nap-missing = Fehlt: { $list }
seo-profile-check-local-business-geo = LocalBusiness Geo-Koordinaten
seo-profile-detail-geo-missing = geo.latitude/longitude fehlt
seo-profile-check-local-business-same-as = LocalBusiness sameAs (Autoritätslinks)
seo-profile-detail-same-as-missing = sameAs fehlt — verhindert Knowledge-Panel
seo-profile-detail-rating-missing = aggregateRating fehlt — keine Sterne im SERP
seo-profile-check-local-business-consistent = LocalBusiness-Schemas konsistent
seo-profile-detail-local-business-consistent = { $count } Schemas, Name einheitlich
seo-profile-detail-local-business-conflicting = { $count } Schemas mit widersprüchlichem Namen
seo-profile-check-jsonld-structure = JSON-LD-Syntax und -Struktur
seo-profile-detail-missing-required = Pflichtangaben fehlen: { $list }
seo-profile-detail-missing-recommended = Empfohlene Angaben nicht vorhanden: { $list }
seo-profile-category-content-base = Technische Inhaltsbasis
seo-profile-check-text-volume = Lesbarer Textumfang (≥ 300 Wörter, Richtwert)
seo-profile-detail-words = { $count } Wörter
seo-profile-check-internal-linking = Interne Verlinkung (≥ 3 Links)
seo-profile-detail-internal-links = { $count } interne Links
seo-profile-check-link-density = Linkdichte > 0,5%

# Status labels of the SERP signals (`seo::serp::SerpSignalStatus::label`).
seo-serp-status-ok = OK
seo-serp-status-warning = Warnung
seo-serp-status-fail = Fehler

# Texts of the SERP signals (`seo::serp::build_serp_analysis`): the
# category, label and detail of each signal. Which signal and detail applies
# stays in Rust; language-neutral names (Description, Canonical, Rich Result,
# robots noindex, robots nosnippet, Favicon, hreflang x-default) are written
# there.
seo-serp-category-indexing = Indexierung
seo-serp-category-title = Titel
seo-serp-category-appearance = Erscheinungsbild
seo-serp-category-internationalisation = Internationalisierung
seo-serp-detail-noindex = Seite ist als noindex markiert und wird nicht indexiert ({ $robots })
seo-serp-detail-nosnippet = nosnippet verhindert die Meta-Description im SERP-Eintrag.
seo-serp-label-title-present = Titel vorhanden
seo-serp-detail-no-title = Kein <title>-Tag gefunden — Pflichtfeld für Google-Listeneinträge.
seo-serp-label-title-length = Titellänge
seo-serp-detail-title-length = { $count } Zeichen (empfohlen 30–60)
seo-serp-label-truncation-risk = Abschneidungsrisiko
seo-serp-detail-title-truncated = { $count } Zeichen — Titel wird in schmalen SERPs abgeschnitten (ca. 55 Zeichen sicher).
seo-serp-label-description-present = Description vorhanden
seo-serp-detail-no-description = Keine Meta-Description — Google zeigt unkontrollierten Seitentext.
seo-serp-label-description-length = Description-Länge
seo-serp-detail-description-length = { $count } Zeichen (empfohlen 120–160)
seo-serp-label-canonical-tag = Canonical-Tag
seo-serp-detail-no-canonical = Kein Canonical-Tag — Google wählt selbst die kanonische URL.
seo-serp-label-canonical-self-reference = Canonical-Selbstreferenz
seo-serp-detail-canonical-other-url = Canonical zeigt auf eine andere URL: { $url }
seo-serp-detail-favicon-present = Favicon vorhanden (erscheint in mobilen SERPs und Browser-Tabs).
seo-serp-detail-no-favicon = Kein Favicon gefunden — Google zeigt in mobilen SERPs ein generisches Icon.
seo-serp-detail-hreflang-x-default = { $count } hreflang-Tags inkl. x-default
seo-serp-detail-hreflang-no-x-default = { $count } hreflang-Tags, kein x-default — Google-Empfehlung für mehrsprachige Seiten.
seo-serp-label-breadcrumb-schema = Breadcrumb-Schema
seo-serp-detail-breadcrumb-complete = BreadcrumbList erkannt, Pflichtangaben erfüllt — für eine Pfadangabe grundsätzlich geeignet.
seo-serp-detail-breadcrumb-incomplete = BreadcrumbList erkannt, aber unvollständig ({ $list }) — für eine Pfadangabe nicht geeignet.
seo-serp-detail-no-breadcrumb = Kein BreadcrumbList-Schema — Pfadangabe im SERP-Eintrag nicht möglich.

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
