# Texts of the SEO module detail presentation of the single-page report
# (`output::builder::single::module_details::seo`, `seo-details-*`), #936.
# Which message applies, the rating bands and how lists and values are joined
# stay in Rust. The labels "Description", "Schema.org content type" and
# "Language" are the `pdf-seo-*` messages, "Yes"/"No" the `yes`/`no` messages
# of report.ftl; "present"/"missing" are the `seo-profile-detail-*` messages of
# seo-issues.ftl.

## Meta tags
seo-details-meta-title = Titel
seo-details-meta-also-accessibility = { $message } — zählt auch in der Barrierefreiheit (WCAG { $criterion })
seo-details-meta-viewport-elsewhere = { $message } — zählt bei Mobile und Barrierefreiheit, nicht im SEO-Wert

## Headings and social tags
seo-details-heading-summary =
    { $h1 ->
        [one] { $h1 } H1-Überschrift
       *[other] { $h1 } H1-Überschriften
    }, { $total } Überschriften gesamt, { $issues } Probleme
seo-details-social-summary = Open Graph: { $og }, Twitter Card: { $twitter }, Vollständigkeit: { $completeness }%

## Schema inventory
seo-details-schema-questions = { $count } Fragen
seo-details-schema-with-search = { $name } (Suche)
seo-details-schema-regions = { $base } · { $count } Regionen
seo-details-schema-levels = { $count } Ebenen

## Signal strength ratings
seo-details-signal-excellent = Sehr gut
seo-details-signal-good = Gut
seo-details-signal-partial = Teilweise
seo-details-signal-minimal = Minimal
seo-details-signal-missing = Fehlt

## Content identity and page profile
seo-details-identity-page-title = Seitentitel
seo-details-identity-topic-hints = Themenhinweise
seo-details-identity-no-topic-hints = Keine klaren Themenhinweise erkannt
seo-details-profile-page-type = SEO-Seitentyp
seo-details-profile-characteristics = Merkmale
seo-details-profile-no-characteristics = Keine prägenden Merkmale erkannt
seo-details-profile-classification = Einordnung
seo-details-profile-recommendation = Empfehlung

## Technical summary
seo-details-language-tag = Sprachangabe
seo-details-word-count = Wortanzahl
seo-details-internal-links = Interne Links
seo-details-external-links = Externe Links
seo-details-dofollow-links = Dofollow-Links
seo-details-nofollow-links = Nofollow-Links
seo-details-not-detected = Nicht erkannt
seo-details-amp-complete = Erkannt, Basis-Signale vollständig
seo-details-amp-issues = Erkannt, { $count } Basisproblem(e)

## Tracking summary
seo-details-google-fonts = Google Fonts (extern)
seo-details-google-fonts-used = Ja ({ $sources })
seo-details-tracking-cookies = Tracking-Cookies
seo-details-none-detected = Keine erkannt
seo-details-cookie-inventory = Cookie-Inventar
seo-details-cookie-inventory-not-collected = Nicht erhoben: Der Cookie-Speicher des Browsers war nicht lesbar
seo-details-no-cookies = Keine Cookies erkannt
seo-details-categories = Kategorien: { $summary }
seo-details-no-storage-keys = Keine Keys erkannt
seo-details-tracking-signals = Tracking-Signale
seo-details-consent-cookies = Consent-Cookies
seo-details-consent-summary = { $before } vor Interaktion, { $after } nach Interaktion, { $added } neu
seo-details-not-collected = Nicht erhoben
seo-details-zaraz-detected = Erkannt ({ $signals })
