# Texte des Builders des Einzelberichts, #936: positive Signale und Aspekte,
# die Kernaussagen der Management-Zusammenfassung und die Score-Treiber-Notiz,
# der Hinweis zur Prüfqualität, die Erklärung der Website-Qualität, die Zeilen
# der Seitengesundheit und die Detailzeilen für Performance, Sicherheit und
# Mobile (`output::builder::single::{executive, positive, serp,
# modules_block, module_details::{performance, platform}}` und der
# View-Model-Builder in `output::builder::single`) sowie die Zeilen der
# experimentellen Indikatoren in der Terminal-Tabelle (`output::terminal`),
# `report-builder-*`. Welche Meldung gilt, die Schwellen, die Zählwerte und
# die Zahlenformate bleiben in Rust. „Barrierefreiheit“ und „Sicherheit“ sind
# die Meldungen `module-accessibility` und `module-security` der report.ftl.

## Positive Signale (Muster-Titel)
report-builder-pattern-main-navigation = Semantische Hauptnavigation
report-builder-pattern-disclosure-menu = Disclosure-Menü
report-builder-pattern-modal-dialog = Modaler Dialog
report-builder-pattern-tab-list = Tab-Liste
report-builder-pattern-skip-link = Skip-Link
report-builder-pattern-accordion = Accordion
report-builder-pattern-easy-language = Leichte-Sprache-Version
report-builder-pattern-form = Formular
report-builder-pattern-add-to-cart = Warenkorb-Button
report-builder-pattern-quantity-stepper = Mengenauswahl
report-builder-wcag-signal = WCAG-Signal

## Positive Aspekte
report-builder-aspect-no-violations = Keine automatisch erkennbaren Verstöße gefunden.
report-builder-aspect-stable-base = Stabile Grundqualität mit gezielt priorisierbaren Restpunkten.
report-builder-aspect-performance = Stabile Ladezeiten und insgesamt reaktionsschneller Seitenaufbau.
report-builder-aspect-seo = Saubere Basis für Auffindbarkeit, Struktur und Meta-Daten.
report-builder-aspect-security = Wichtige Sicherheitsmechanismen sind grundsätzlich vorhanden.
report-builder-aspect-mobile = Die Seite ist auf kleinen Displays gut bedienbar und lesbar.
report-builder-aspect-base-area = Grundstruktur
report-builder-aspect-base = Die Seite ist grundsätzlich funktional und erreichbar.

## Kernaussagen der Management-Zusammenfassung
report-builder-key-critical = Mehrere kritische WCAG-Barrieren blockieren Nutzer von Hilfstechnologien — Screenreader und Tastaturnavigation sind betroffen.
report-builder-key-high = Deutliche Barrierefreiheitslücken erschweren die Nutzung für Menschen mit Behinderungen — keine vollständigen Zugangssperren, aber relevante Reibungspunkte.
report-builder-key-main-issue = Hauptproblem: { $title } ({ $share }% aller kritischen/hohen Befunde)
report-builder-key-most-frequent = Häufigstes Problem: { $title }
report-builder-key-legal = WCAG-Level-A/AA-Befunde mit BFSG/EAA-Relevanz automatisiert erkannt — manuelle Prüfung für belastbare BFSG-Einordnung nötig
report-builder-key-no-level-violations = Keine Level-A/AA-Verstöße, aber strukturelle Optimierungspotenziale
report-builder-key-audit-notes = Audit-Hinweise vorhanden — einzelne Signale sollten fachlich gegengeprüft werden.
report-builder-key-no-barriers = Keine automatisiert erkennbaren kritischen Barrieren — manuelle Prüfung empfohlen.
report-builder-key-scope = WCAG-Prüfumfang: { $automated } von ca. { $total } WCAG-2.2-AA-Kriterien werden automatisch geprüft; kontextabhängige Kriterien bleiben manuell zu prüfen.
report-builder-driver-none = Score-Matrix: kein negativer Haupttreiber in den gewichteten Accessibility-Themen erkannt.
report-builder-driver-areas = Score-Matrix: stärkste negative Treiber sind { $areas }; gewichtet werden Semantik, Formulare, Tastatur, Fokus, Bilder, ARIA, Überschriften und Landmarks.

## Hinweis zur Prüfqualität und Kennzahl-Beschriftung
report-builder-stability-view = { $view }-Ansicht nach { $ms } ms
report-builder-stability-mutations = , noch { $count } DOM-Änderungen
report-builder-stability-individual = einzelne Messungen
report-builder-quality-partial = Der automatisierte Audit wurde erfolgreich durchgeführt. Einzelne Messungen haben ein Stabilitäts-/Wiederholungsbudget erreicht und sind daher nicht in den ausgewiesenen Scores enthalten; diese beschreiben den erfolgreich gemessenen Umfang. Betroffen: { $affected }.
report-builder-quality-insufficient = Dieser Prüflauf hat unzureichende Datenqualität: mehrere Messungen sind fehlgeschlagen. Scores und Befunde können unvollständig oder unzuverlässig sein. Details im Methodik-Anhang.
report-builder-metric-quick-wins = Schnell umsetzbar

## Erklärung der Website-Qualität
report-builder-site-quality-indicators = Indikator-Module ({ $modules }) werden separat ausgewiesen und verändern die Website-Qualität nicht.
report-builder-site-quality-dual = Die Website-Qualität nutzt das Dual-Viewport-Ergebnis: 70% Mobile und 30% Desktop. Sicherheit fließt mit 10% ein, wenn aktiv.{ $indicators }
report-builder-site-quality-weighted = Die Website-Qualität ergibt sich aus den gewichteten Einzelmodulen: { $weights }.{ $indicators }
report-builder-site-quality-weight = { $name } { $pct } %
report-builder-site-quality-weight-default = Barrierefreiheit 100 %

## Zeilen der Seitengesundheit
report-builder-page-health-url-length = URL-Länge
report-builder-page-health-url-length-value = { $count } Zeichen
report-builder-page-health-path-depth = Pfadtiefe
report-builder-page-health-query-parameters = Query-Parameter
report-builder-page-health-self-redirect = Eigene Weiterleitung
report-builder-page-health-target-url = Ziel-URL
report-builder-page-health-validator-no-info = Keine Zusatzinformationen verfügbar

## Performance-Details
report-builder-perf-co2e = CO2e pro View
report-builder-perf-dom-nodes = DOM-Knoten
report-builder-perf-guideline-dom-nodes = Richtwert: max. 800
report-builder-perf-load-time = Ladezeit
report-builder-perf-guideline-load-time = Richtwert: max. 3,0 s
report-builder-perf-dom-content-loaded = DOM-Inhalt geladen
report-builder-perf-guideline-dom-content-loaded = Richtwert: max. 1,8 s
report-builder-perf-blocking-scripts = Blockierende Skripte
report-builder-perf-blocking-css = Blockierendes CSS
report-builder-perf-blocking-transfer = Blockierender Transfer
report-builder-perf-third-party = Drittanbieter
report-builder-perf-first-party = Eigene Ressourcen
report-builder-perf-no-measurable-impact = Kein messbarer Einfluss auf die gemessenen Vitals — trotzdem vorbeugend beheben, da langsame Verbindungen oder kalte Caches stärker betroffen sein können.

## Sicherheits-Details
report-builder-security-valid-certificate = Gültiges Zertifikat
report-builder-security-certificate-subject = Zertifikat Subject
report-builder-security-certificate-issuer = Zertifikat Issuer
report-builder-security-expires-in = Läuft ab in
report-builder-security-expires-days = { $days } Tage
report-builder-security-chain-length = Chain-Länge
report-builder-security-certificate-error = Zertifikatsfehler
report-builder-security-header-present = Vorhanden
report-builder-security-header-missing = Fehlt
report-builder-security-band-note-open = Die Einordnung im Text liegt unter dem Band der Zahl ({ $score } allein hieße „{ $numeric }“): { $count ->
        [one] 1 offener Befund steht
       *[other] { $count } offene Befunde stehen
    } weiter unten. Der Wert selbst bleibt unverändert.
report-builder-security-band-note-severe = Die Einordnung im Text liegt unter dem Band der Zahl ({ $score } allein hieße „{ $numeric }“): { $count ->
        [one] 1 offener Befund hoher Schwere steht
       *[other] { $count } offene Befunde hoher Schwere stehen
    } weiter unten. Der Wert selbst bleibt unverändert.
report-builder-security-score-start = Ausgangswert
report-builder-security-score-not-charged = 0 (nicht belastet)
report-builder-security-score-vulnerable-libraries = Bekannt verwundbare Bibliotheken (Best Practices)
report-builder-security-score-total = Sicherheitswert

## Mobile-Details
report-builder-mobile-scalable = Skalierbar
report-builder-mobile-configured = Korrekt konfiguriert
report-builder-mobile-targets-total = Gesamt
report-builder-mobile-targets-adequate = Erfüllt die Empfehlung (≥44px)
report-builder-mobile-targets-small = Zu klein
report-builder-mobile-targets-crowded = Zu eng beieinander
report-builder-mobile-font-base = Basis-Schriftgröße
report-builder-mobile-font-smallest = Kleinste Schrift
report-builder-mobile-font-legible = Lesbarer Text
report-builder-mobile-font-relative = Relative Einheiten
report-builder-mobile-sizing-fits = Passt in Viewport
report-builder-mobile-sizing-no-scroll = Kein hor. Scrollen
report-builder-mobile-sizing-responsive-images = Responsive Bilder
report-builder-mobile-context = { $count } im Bereich { $area }
report-builder-mobile-locally-scoped = — lokal begrenzt
report-builder-mobile-small-targets = { $base } { $count } Touch-Targets kleiner als die Ergonomie-Empfehlung (44×44 px){ $context }{ $scope }.

## Terminal-Tabelle
report-builder-terminal-experimental-title = Experimentelle Indikatoren (nicht validiert, ohne Einfluss auf Score und Urteil)
report-builder-terminal-no-score = { $label }: kein Score
report-builder-terminal-content-visibility = Content Visibility: { $problems } von { $signals } Signalen auffällig
report-builder-terminal-commerce =
    Commerce: { $findings ->
        [one] 1 Hinweis
       *[other] { $findings } Hinweise
    }

# Terminal table chrome (single and batch): the table's own labels follow the run language.
report-builder-terminal-metric-accessibility = Barrierefreiheit
report-builder-terminal-metric-site-quality = Website-Qualität (optionale Module)
report-builder-terminal-metric-grade = Note
report-builder-terminal-metric-violations = Verstöße
report-builder-terminal-metric-duration = Dauer
report-builder-terminal-metric-urls = URLs
report-builder-terminal-metric-passed = Bestanden
report-builder-terminal-metric-failed = Fehlgeschlagen
report-builder-terminal-metric-warned = Mit Warnung
report-builder-terminal-metric-avg-score = Ø Score
report-builder-terminal-details-hint = alle Befunde: --format json oder --format pdf
report-builder-terminal-batch-title = Batch-Audit — { $domain }
report-builder-terminal-scope-title = Umfang
report-builder-terminal-scope-sample = { $audited } von { $total } gefundenen URLs ({ $source })
report-builder-terminal-baseline-title = Baseline
report-builder-terminal-score-basis-title = Score-Grundlage
report-builder-terminal-score-basis = Die Scores decken { $audited } von { $total } URLs ab; { $failed } konnten nicht geprüft werden
report-builder-terminal-top-rules = Häufigste wiederkehrende Regeln
report-builder-terminal-fix-audit-error = Prüffehler beheben für { $url }: { $error }
