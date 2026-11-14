# Texts of the report builder helpers, the batch builder, the PDF detail modules
# for performance, SEO, tech stack and best practices, and the display mode
# texts (`output::builder::helpers`, `output::builder::batch`,
# `output::pdf::detail_modules::{indicators,seo,performance}`, `display`,
# `output-*`), #936. Which message applies, the thresholds, the number formats
# and the values that go into a message stay in Rust. "Yes", "No", "Metric"
# and "Value" are the messages `yes`, `no`, `col-metric` and `col-value` of
# report.ftl.

## Report title and security labels
output-report-title = Website-Qualitätsbericht
output-report-subtitle = Automatisierter Accessibility-Report mit ergänzenden Qualitätsmodulen.
output-header-tier-baseline = Basisschutz
output-header-tier-architecture-dependent = Architekturabhängig
output-header-tier-context-dependent = Kontextabhängig
output-header-tier-not-assessable = Remote nicht bewertbar
output-issue-kind-missing = Fehlt
output-issue-kind-misconfigured = Fehlkonfiguriert

## Batch builder
output-batch-distribution-thin-content = Hoher Anteil an Thin-Content-Seiten: Das kann Informationswert und SEO-Potenzial begrenzen.
output-batch-distribution-no-editorial = Editoriale Inhaltsseiten fehlen: Wissensaufbau und Suchintentionen werden kaum bedient.
output-batch-distribution-marketing-dominated = Marketing- und Landingpages dominieren: Mehr strukturierter Tiefeninhalt würde die Domain ausbalancieren.
output-batch-lever-fallback = Ergebnisse stabil halten
output-batch-cover-title = Barrierefreiheits-Batch-Audit-Report
output-batch-cover-urls = { $count } URLs geprüft
output-batch-blocking-pages-analyzed = Seiten analysiert
output-batch-blocking-pages-blocking = Seiten mit Blocking
output-batch-blocking-distinct = Verschiedene Blocking-Ressourcen
output-batch-blocking-third-party = Third-Party-Traffic gesamt
output-batch-blocking-pages-of = { $pages } von { $total }
output-batch-blocking-loads = { $distinct } ({ $loads } Einbindungen über alle Seiten)

## Display modes
output-display-site-default = Voreinstellung der Seite (kein Modus gewählt)
output-display-chosen = { $mode } (gewählt per --display)
output-display-chosen-reduced-motion = { $mode } (gewählt per --display, reduzierte Bewegung emuliert)
output-display-offered-none = Keine (kein html[data-display])
output-display-offered = { $modes } (dargestellt: { $active })

## Tech stack and best practices
output-ts-takeaway-none = Keine gängigen Technologien erkannt.
output-ts-takeaway-count = { $count } Technologien erkannt.
output-bp-takeaway-not-measured = Nicht gemessen.
output-bp-takeaway-clean = Keine Konsolenfehler oder bekannten verwundbaren Bibliotheken erkannt.
output-bp-takeaway-issues = Konsolenfehler oder verwundbare Bibliotheken erkannt — siehe unten.

## SEO
output-seo-schema-none = keine
output-seo-schema-explainer = Schema.org kennzeichnet strukturierte Daten für Suchmaschinen; der Reifegrad zeigt, wie vollständig sie auf der Seite genutzt werden.
output-seo-schema-count = { $count } { $count ->
    [one] Schema
   *[other] Schemas
}

## Performance
output-perf-further-metrics = Weitere Lab-Metriken
output-perf-score-per-viewport = Performance-Score je Ansicht: 0–100, höher ist besser.
output-perf-score-basis = Der Performance-Wert dieses Berichts ({ $score }) ist der Wert der Ansicht { $basis }; die andere Ansicht wird getrennt bewertet.
output-perf-vitals-legend = LCP: größter sichtbarer Inhalt (gut ≤ 2,5 s); FCP: erster sichtbarer Inhalt (gut ≤ 1,8 s); CLS: Layoutstabilität (gut ≤ 0,1); TTFB: Serverantwort (gut ≤ 0,8 s). Bei Zeit- und Verschiebungswerten ist niedriger besser.
output-perf-desktop-lab-metrics = Desktop — Kennzahlen (Labormessung)
output-perf-mobile-lab-metrics = Mobile — Kennzahlen (Labormessung)
