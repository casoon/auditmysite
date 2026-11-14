# German counterpart of `locales/en/batch-report.ftl`: the texts of the batch
# PDF's keyboard journey summary, module name, impact summary, commerce
# roll-up, cover contents, EN 301 549 clause roll-up and overview cards
# (`output::pdf::batch_report::interactive`, `portfolio`, `appendix`,
# `overview`, `batch-report-*`), #936. The same keys as the English file.

## Keyboard journey summary
batch-report-interactive-intro = Interaktive Tests prüfen, ob Fokusführung, Zustandswechsel und Tastaturnavigation korrekt funktionieren — nicht nur ob der initiale Seitenbaum korrekt ausgezeichnet ist.
batch-report-interactive-table-title = Befunde nach Kategorie
batch-report-interactive-col-category = Kategorie
batch-report-interactive-col-affected = Betroffene URLs
batch-report-interactive-col-severity = Max. Schwere
batch-report-interactive-cat-tab-order = Tab-Reihenfolge
batch-report-interactive-cat-focus-trap = Fokus-Falle (Modal)
batch-report-interactive-cat-state-transition = Zustandswechsel
batch-report-interactive-cat-focus-restoration = Fokus-Rückführung
batch-report-interactive-cat-form-error = Formularfehler-Ankündigung
batch-report-interactive-cat-spa-navigation = SPA-Navigation
batch-report-interactive-cat-hidden-focusable = Verstecktes fokussierbares Element
batch-report-interactive-cat-skip-link = Skip-Link
batch-report-interactive-cat-focus-indicator = Fokus-Indikator
batch-report-interactive-cat-menu = Menü / Navigation
batch-report-interactive-cat-tabs = Tab-Widget
batch-report-interactive-cat-link-text = Linktext
batch-report-interactive-cat-landmark = Landmarke
batch-report-interactive-cat-media-controls = Medienbedienung
batch-report-interactive-cat-heading-outline = Überschriftengliederung

## Module names
batch-report-module-html-conformance = HTML-Konformität

## Impact summary
batch-report-impact-user = Nutzer
batch-report-impact-business = Business
batch-report-impact-risk = Risiko
batch-report-impact-business-critical = Die geprüften Seiten enthalten kritische Barrieren für bestimmte Nutzergruppen; wie weit sie über diese Seiten hinausreichen, ist nicht gemessen.
batch-report-impact-business-high = Einzelne Funktionsbereiche sind für Nutzer mit Einschränkungen problematisch.
batch-report-impact-business-low = Geringe Auswirkung auf den geprüften Seiten — automatisiert keine kritischen oder hohen Barrieren erkannt.
batch-report-impact-legal-flags = WCAG-Level-A/AA-Verstöße automatisiert erkannt — für belastbare BFSG-Einordnung ist manuelle Prüfung nötig.
batch-report-impact-legal-none = Automatisiert keine kritischen Verstöße erkannt — manuelle Prüfung für vollständige Einordnung empfohlen.

## Commerce roll-up
batch-report-commerce-intro = Produkt-Strukturdaten-Vollständigkeit und Pflichtseiten-Verlinkung, aggregiert über das geprüfte Set.
batch-report-commerce-scope-note = Prüft Produkt-Strukturdaten und die Verlinkung von Pflichtseiten über die geprüften Seiten hinweg — keine Prüfung von Checkout, Zahlungsabwicklung oder Warenkorb (das Tool hat keinen seitenübergreifenden Sitzungszustand).
batch-report-commerce-product-pages = Produktseiten
batch-report-commerce-product-pages-audited =
    { $count } { $count ->
        [one] Produktseite
       *[other] Produktseiten
    } geprüft
batch-report-commerce-trust-title = Pflichtseiten-Verlinkung (site-weit)
batch-report-commerce-not-linked = Nirgends im geprüften Set verlinkt
batch-report-commerce-trust-impressum = Impressum
batch-report-commerce-trust-agb = AGB
batch-report-commerce-trust-widerruf = Widerruf / Retoure
batch-report-commerce-trust-versand = Versand / Lieferung
batch-report-commerce-trust-zahlungsarten = Zahlungsarten
batch-report-commerce-trust-kontakt = Kontakt

## Cover
batch-report-contents = Inhalt

## EN 301 549 clause roll-up
batch-report-en301549-title = EN-301-549-Klauselzuordnung
batch-report-en301549-intro = Domainweite Zusammenfassung der automatisch erkannten WCAG-2.2-A/AA-Befunde dieses Batches je Klausel von EN 301 549 V4.1.1 (2026-09), Kapitel 9 (Web) — der schlechteste über alle geprüften Seiten beobachtete Status, plus die Anzahl betroffener Seiten.
batch-report-en301549-disclaimer-title = Keine Barrierefreiheitserklärung
batch-report-en301549-status-violations = Verstöße gefunden
batch-report-en301549-status-no-violations = Keine Verstöße (automatisch)
batch-report-en301549-status-manual = Manuelle Prüfung erforderlich
batch-report-en301549-incomplete = Automatisierte Prüfung unvollständig
batch-report-en301549-col-clause = Klausel
batch-report-en301549-col-title = Titel
batch-report-en301549-col-pages = Betroffene Seiten
batch-report-en301549-table-title = Klausel-Übersicht (alle 50 WCAG-2.1-A/AA-Klauseln)
batch-report-en301549-out-of-scope-title = Außerhalb dieses Prüfumfangs
batch-report-en301549-chapter = Kapitel { $chapter } – { $title } (nicht bewertet)

## Overview cards
batch-report-grid-accessibility-average = Barrierefreiheit Ø
batch-report-grid-urls-audited = Geprüfte URLs
batch-report-grid-wcag-occurrences = WCAG-Vorkommen
batch-report-grid-critical-high = Kritisch + hoch (WCAG-Vorkommen)
