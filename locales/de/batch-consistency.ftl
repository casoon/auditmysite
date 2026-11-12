# Texts of the batch PDF's cross-page consistency section
# (`output::pdf::batch_report::consistency`, `batch-consistency-*`) and of its
# management overview, audit caveats, internal comparison and decision table
# (`output::pdf::batch_report::management`, `batch-management-*`), #936. Which
# message applies, the thresholds and how lists are joined stay in Rust. The
# column headers "Module", "Risk" and "Schema type" are the `batch-col-*`
# messages of report.ftl.
#
# The `-basis-*` and `-identity-*` messages render a text the analysis stores
# in canonical English: the English message is that stored text (`$basis`,
# `$finding`), the German one its translation.

## Cross-page consistency
batch-consistency-title = Seitenübergreifende Konsistenz
batch-consistency-intro = Prüft, ob geteilte Elemente auf allen geprüften Seiten konsistent sind: Navigation (WCAG 3.2.3), die Hauptüberschrift (H1) je Seite, Canonical-URLs (ein Suchmaschinen-Signal, kein WCAG-Kriterium) und Sicherheits-Header.
batch-consistency-consistent = Konsistent
batch-consistency-missing-on-pages = Fehlt auf { $missing } von { $total } geprüften Seiten.
batch-consistency-nav-title = Navigation ({ $main }/{ $total } mit Hauptnav, { $skip }/{ $total } mit Skip-Link)
batch-consistency-nav-consistent = Keine Navigations-Inkonsistenzen erkannt.
batch-consistency-main-nav = Hauptnavigation
batch-consistency-skip-link = Skip-Link
batch-consistency-headings-title = Überschriften ({ $single }/{ $total } mit einem H1, { $none } ohne H1, { $multiple } mit mehreren)
batch-consistency-headings-consistent = Jede Seite beginnt mit genau einem H1.
batch-consistency-missing-h1 = Fehlende H1
batch-consistency-missing-h1-detail = { $count } geprüfte Seiten haben keine H1.
batch-consistency-multiple-h1 = Mehrere H1-Überschriften
batch-consistency-multiple-h1-detail = { $count } geprüfte Seiten haben mehrere H1-Überschriften.
batch-consistency-canonical-title = Canonical-URLs (www: { $www }, ohne www: { $non_www }, fehlend: { $missing } von { $total })
batch-consistency-canonical-consistent = Alle Seiten kanonisieren auf dieselbe Domain-Variante.
batch-consistency-mixed-hosts = Gemischte Host-Strategie
batch-consistency-mixed-hosts-detail = { $www } Seiten verwenden www- und { $non_www } Seiten nicht-www-Canonicals.
batch-consistency-missing-canonical = Fehlendes Canonical
batch-consistency-missing-canonical-detail = Fehlt auf { $missing } von { $total } geprüften Seiten. Optional: Suchmaschinen wählen die kanonische URL dann selbst. Relevant wird das erst bei URL-Varianten (Parameter, www/ohne www); das wurde hier nicht geprüft.

## Cross-page WCAG criteria
batch-consistency-wcag-title = Seitenübergreifende WCAG-Prüfung
batch-consistency-col-criterion = Kriterium
batch-consistency-col-basis = Prüfgrundlage
batch-consistency-status-no-inconsistency = Keine Inkonsistenz erkannt
batch-consistency-status-warning = Warnung
batch-consistency-status-manual-review = Manuell prüfen
batch-consistency-basis-navigation = Vorhandensein von Hauptnavigation und Skip-Link über das geprüfte Seitenset verglichen
batch-consistency-basis-identification = Die Identität von Komponenten erfordert einen Vergleich der zugänglichen Namen gleichwertiger Bedienelemente; die aktuelle Evidenz reicht nicht für eine Konformitätsaussage
batch-consistency-basis-multiple-ways = Eingehende Links nur innerhalb des geprüften Seitensets beobachtet; Seiten ohne einen solchen Link sind Kandidaten, und Suche, Sitemap-Seiten, Links von nicht geprüften Seiten sowie Ausnahmen für Prozessschritte müssen manuell bestätigt werden

## Consistent help, orphan pages, schema entities
batch-consistency-help-title = Konsistente Hilfe (WCAG 3.2.6): Abweichungen
batch-consistency-help-more = { $count } weitere Abweichungen
batch-consistency-see-json = siehe JSON-Report
batch-consistency-orphan-title = Seiten ohne beobachteten eingehenden Link innerhalb der geprüften URLs
batch-consistency-orphan-row = Kein eingehender Link beobachtet
batch-consistency-orphan-note = Gezählt wurden hier nur Links zwischen den geprüften Seiten; die Links einer Startseite außerhalb der Stichprobe sind nicht enthalten. Eine hier genannte Seite kann von der Startseite, von Seiten außerhalb der Stichprobe, über die Suche oder eine Sitemap-Seite verlinkt sein — sie ist ein Kandidat, keine bestätigte verwaiste Seite.
batch-consistency-schema-conflicts-title = Identitätskonflikte bei Schema-Entitäten

## Structured data
batch-consistency-structured-types-title = Verteilung strukturierter Daten
batch-consistency-col-audited-pages = Geprüfte Seiten
batch-consistency-blockers-title = Wiederkehrende Blocker bei strukturierten Daten
batch-consistency-blocker-detail = Pflichtbedingung fehlt auf { $count } geprüften Seiten
batch-consistency-parity-title = Wiederkehrende Abweichungen zwischen Schema und Inhalt
batch-consistency-parity-detail = Sichtbarer Inhalt weicht auf { $count } geprüften Seiten ab
batch-consistency-matrix-col-types = Vorgeschlagen / erkannt
batch-consistency-coverage-complete = Abgedeckt
batch-consistency-coverage-opportunity = Optionale Ergänzung
batch-consistency-coverage-manual-review = Manuell prüfen
batch-consistency-coverage-not-applicable = Nicht anwendbar
batch-consistency-identity-title = Websiteweite Entitätsidentität
batch-consistency-identity-row = Organisationsidentität
batch-consistency-identity-ids-differ = Die Organisationsidentität verwendet auf den geprüften Seiten unterschiedliche @id-Werte.
batch-consistency-identity-id-missing = Mindestens ein Organisationsknoten besitzt keine stabile @id zur seitenübergreifenden Identitätsprüfung.

## Security headers and topics
batch-consistency-headers-title = Sicherheits-Header über die Seiten ({ $checked } Seiten mit Sicherheitsprüfung)
batch-consistency-header-partial = Auf { $present } von { $checked } Seiten gesetzt; fehlt z. B. auf { $examples }.
batch-consistency-header-values = { $values } verschiedene Werte auf { $present } Seiten.
batch-consistency-header-uniform = Einheitlich
batch-consistency-header-uniform-set = überall gleich gesetzt: { $headers }
batch-consistency-header-uniform-missing = auf keiner Seite gesetzt: { $headers }
batch-consistency-overlap-title = Starke Themenüberschneidungen zwischen Seiten
batch-consistency-overlap-detail = Begriffsähnlichkeit { $percent } % · gemeinsame Begriffe: { $terms }

## Audit caveats
batch-management-flags-title = Wiederkehrende Audit-Hinweise
batch-management-flag-consent-banner = Consent-Banner
batch-management-flag-bypass-blocks-untested = Skip-Link-Prüfung
batch-management-flag-conflicting-signal = Widersprüchliches Signal
batch-management-flag-viewport-gap = Desktop-/Mobile-Unterschied
batch-management-flag-incomplete-audit = Unvollständiger Messumfang
batch-management-flag-consent-wall-artifact = Consent-Wall-Artefakt
batch-management-flag-other = Audit-Hinweis
batch-management-flag-pages = { $count ->
        [one] { $count } betroffene Seite
       *[other] { $count } betroffene Seiten
    }

## Assessment and key points
batch-management-assessment-critical = Kritische Barrieren — nicht WCAG-konform
batch-management-assessment-serious = Gravierende Barrieren — nicht WCAG-konform
batch-management-assessment-substantial-gaps = Erhebliche Barrierefreiheitslücken
batch-management-assessment-usable-critical = Nutzbar, aber mit kritischen Barrieren
batch-management-assessment-usable-high = Nutzbare Basis, aber Barrieren hoher Schwere gefunden
batch-management-assessment-needs-improvement = Verbesserungswürdig auf dem Weg zur Barrierefreiheit
batch-management-assessment-stable-critical = Technisch stabil, aber mit kritischen Barrieren
batch-management-assessment-good-high = Gute Basis, aber Barrieren hoher Schwere gefunden
batch-management-assessment-fine-tuning = Weitgehend barrierefrei — Feinschliff
batch-management-assessment-polish = Weitgehend barrierefrei — Feinschliff
batch-management-point-severe = { $occurrences } kritische/hohe WCAG-Vorkommen auf { $urls } URLs ({ $findings } Befunde, je Seite gruppiert)
batch-management-point-main-issue = Hauptproblem: { $title } ({ $occurrences } Vorkommen auf { $urls } URLs)
batch-management-point-legal-flags = WCAG-Level-A/AA-Verstöße automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig
batch-management-point-structural = Keine Level-A/AA-Verstöße, aber strukturelle Optimierungspotenziale auf mehreren Seiten
batch-management-point-no-barriers = Keine automatisiert erkennbaren kritischen Barrieren — manuelle Prüfung empfohlen.
batch-management-risks-title = Management-Risikoansicht

## Internal comparison
batch-management-comparison-too-few = Der seitenübergreifende Vergleich benötigt mindestens 3 URLs für aussagekräftige domainweite Durchschnitte; dieser Report umfasst { $count }. Maßgeblich ist hier die Einzel-URL-Auswertung.
batch-management-comparison-title = Interner Vergleich nach Modul
batch-management-col-best-url = Beste URL
batch-management-col-weakest-url = Schwächste URL
batch-management-comparison-same = { $score }/100 auf allen { $count } Seiten mit Messung
batch-management-outliers-title = Ausreißer-URLs
batch-management-outlier = { $url }: { $score } / 100 ({ $delta } Punkte unter Durchschnitt)

## Decision-oriented top actions
batch-management-actions-title = Entscheidungsorientierte Top-Maßnahmen
batch-management-col-action = Maßnahme / Ursache
batch-management-col-scope = Umfang der Änderung
batch-management-col-reach = Reichweite
batch-management-root-recurring = auf { $count } Seiten, Ursache offen
batch-management-root-one-page = { $count } Vorkommen auf einer Seite
batch-management-root-page-specific = seitenspezifisch
batch-management-scope-high = hoch
batch-management-scope-medium = mittel
batch-management-scope-low = gering
batch-management-reach = { $occurrences } Vorkommen / { $urls } { $urls ->
        [one] URL
       *[other] URLs
    }
