label-certificate = Auditstatus
section-summary = Kurzfazit
section-methodology = Prüfumfang und Methodik
section-modules = Bewertung nach Modulen
section-findings-overview = Übersicht der Probleme
section-findings = Erkannte Probleme
section-findings-technical = Technische Detailanalyse
section-actions = Maßnahmenplan
section-appendix = Anhang: Technische Details
callout-limitations-title = Grenzen automatisierter Tests
callout-note-title = Hinweis
callout-no-issues-title = Ausgezeichnete Barrierefreiheit
callout-no-issues-body = Keine automatisch erkennbaren Barrierefreiheitsprobleme gefunden.
certificate-thresholds = Einstufungsstufen (Gesamtscore): SEHR GUT ab 95, GUT ab 85, SOLIDE ab 75, AUSBAUFÄHIG ab 65, darunter UNGENÜGEND.
label-priority = Priorität
label-owner = Zuständig
label-effort = Aufwand
label-module = Modul
label-type = Typ
label-tech-note = Technischer Hinweis
label-user-impact = Auswirkung auf Nutzer
label-typical-cause = Typische Ursache
label-affected-urls = Betroffene URLs
label-code-example = Codebeispiel
label-wrong = Falsch
label-right = Richtig
label-decorative = Dekorativ
priority-critical = Kritisch
priority-high = Hoch
priority-medium = Mittel
priority-low = Niedrig
role-development = Entwicklung
role-editorial = Redaktion
role-designux = Design / UX
role-projectmanagement = Projektleitung
effort-quick = Quick Win
effort-medium = Mittelfristig
effort-structural = Strukturell
severity-critical = Kritisch
severity-high = Hoch
severity-medium = Mittel
severity-low = Niedrig
cover-fact-domain = Domain
cover-fact-scope = Prüfumfang
cover-fact-scope-single = Einzel-URL
cover-fact-scope-batch = Sitemap / Batch
cover-fact-scope-comparison = Wettbewerbsvergleich
cover-fact-modules = Module
cover-fact-date = Prüfdatum
metric-score = Gesamtscore
metric-issues-detected = Probleme erkannt
metric-critical-high = Kritisch / Hoch
metric-risk = Risiko
metric-certificate = Auditstatus
panel-quick-actions = Sofort umsetzbar
panel-strengths = Was bereits stark ist
label-strength = Stärke
section-top-issues = Die wichtigsten Probleme im Überblick
section-all-violations = Alle Verstöße (aggregiert nach Regel)
audit-data-title = Audit-Daten
audit-data-area = Bereich
audit-data-signal = Signal
audit-data-value = Wert
audit-data-row-audit = Audit
audit-data-row-module = Modul
audit-data-row-finding = Finding
scope-box-title = Prüfumfang
scope-box-wcag-level = WCAG-Level
scope-box-checked-nodes = Geprüfte Knoten
scope-box-runtime = Laufzeit
scope-box-findings-total = Findings gesamt
scope-box-critical-high = Kritisch / Hoch
scope-box-audit-notes = Audit-Hinweise

# Cover & narrative
narrative-cover-eyebrow = Automatisierter Audit-Report
narrative-cover-kicker = Technischer Website-Check mit Fokus auf Accessibility, SEO und Performance
narrative-status-title = Status der Website
narrative-metrics-title = Executive Snapshot
narrative-key-points-title = Kernaussagen
narrative-impact-title = Auswirkungen
narrative-quick-actions-title = Empfohlene Sofortmaßnahmen
narrative-spotlight-eyebrow = HAUPTPROBLEM
narrative-leverage-title = Wirkung einer Behebung
narrative-findings-title = Key Findings
narrative-action-plan-title = Maßnahmenplan
narrative-action-plan-intro = Priorisiert nach Wirkung und Aufwand. Die Maßnahmen sind klar umrissen und direkt planbar.
narrative-action-plan-callout-title = Empfohlene Vorgehensweise
narrative-action-plan-callout-body = Beginne mit den Quick Wins: hoher Impact bei geringem Aufwand. Die nachfolgende Tabelle zeigt alle Maßnahmen in empfohlener Reihenfolge.
narrative-technical-title = Technische Umsetzung
narrative-technical-intro = Ab hier folgt die konkrete Umsetzung für Entwicklung, Design und Redaktion. Jedes Problem enthält: betroffene Elemente, direkte Umsetzung, Code-Beispiele.
narrative-next-steps-title = Empfohlene nächste Schritte
narrative-next-steps-intro = Konkrete Handlungsempfehlung für die nächsten 1–4 Wochen.
narrative-next-steps-callout-title = Nächster Schritt
narrative-next-steps-callout-body = Für eine vollständige Barrierefreiheits-Prüfung empfehlen wir ergänzend einen manuellen Audit mit assistiven Technologien (Screenreader, Tastaturnavigation).
narrative-findings-intro-strong = Die Seite ist technisch stark aufgestellt. Die folgenden Punkte sind letzte Optimierungshebel ohne strukturellen Druck.
narrative-findings-intro-solid = Solide Basis — die folgenden Punkte sind gezielte Verbesserungshebel.
narrative-findings-intro-default = Die folgenden Probleme haben den größten Einfluss auf Nutzbarkeit und Risiko. Technische Details folgen im nächsten Abschnitt.

# Verdict (single audit)
verdict-tier-excellent = { $url } erreicht { $score }/100 im Accessibility-Audit. Die verbleibenden Findings sind letzte Optimierungshebel — kein strukturelles Problem, sondern Feinschliff.
verdict-tier-solid = { $url } erreicht { $score }/100 im Accessibility-Audit. Die Basis ist solide — klarer Verbesserungshebel mit überschaubarem Aufwand.
verdict-tier-deficient = { $url } erreicht { $score }/100 im Accessibility-Audit. Es bestehen deutliche Barrieren — nicht nur Detailprobleme, sondern struktureller Nachholbedarf.
verdict-tier-critical = { $url } erreicht nur { $score }/100 im Accessibility-Audit. Akuter Handlungsbedarf: Wesentliche Inhalte und Funktionen sind für einen Teil der Nutzer nicht zugänglich.
score-note-high-with-critical = Der Score berücksichtigt Gewichtung und Häufigkeit. Einzelne kritische Themen können trotz hoher Gesamtbewertung bestehen.

# Verdict (batch audit)
verdict-batch-excellent = Über { $total_urls } geprüfte URLs hinweg erreicht die Website einen Gesamtscore von { $score }/100 — ein sehr gutes Ergebnis.
verdict-batch-solid = Im Durchschnitt erreichen die { $total_urls } geprüften URLs einen Gesamtscore von { $score }/100. Die Basis ist solide, es bestehen aber wiederkehrende Probleme in einzelnen Modulen.
verdict-batch-deficient = Die { $total_urls } geprüften URLs erreichen im Schnitt nur { $score }/100 Punkte. Es bestehen erhebliche systematische Probleme.
verdict-batch-critical = Die { $total_urls } geprüften URLs erreichen im Schnitt nur { $score }/100 Punkte. Dringender Handlungsbedarf in mehreren Modulen.

# Site state (audit summary)
site-state-polished = Stark
site-state-needs-work = Solide Basis
site-state-weak = Instabil
site-state-critical = Kritisch

# Risk levels
risk-level-critical = Kritisch
risk-level-high = Hoch
risk-level-medium = Mittel
risk-level-low = Gering

# Yes / No
yes = Ja
no = Nein

# Grade labels
grade-excellent = Sehr gut
grade-good = Gut
grade-satisfactory = Befriedigend
grade-deficient = Ausbaufähig
grade-critical = Kritisch

# Business / forward-looking consequence
business-consequence-clean = Keine bekannten Barrieren — gutes Fundament für alle Nutzergruppen.
business-consequence-severe = Weite Teile der Seite sind für bestimmte Nutzergruppen nicht oder kaum nutzbar.
business-consequence-seo-headings = Seite wird schlechter gefunden und ist für Teile der Nutzer strukturell nicht zugänglich.
business-consequence-screenreader = Einzelne Kernfunktionen sind für Screenreader-Nutzer blockiert oder fehleranfällig.
business-consequence-default = Nutzbarkeit ist gegeben — gezielte Verbesserungen erhöhen Qualität und Reichweite.
consequence-severe = Neue Inhalte und Funktionen erben die bestehenden Fehler — Korrekturaufwand wächst mit jeder Erweiterung.
consequence-many-weak-modules = Aufwand für spätere Korrekturen steigt deutlich — besonders bei Relaunch oder größerem Content-Ausbau.
consequence-stable = Kein akuter Handlungsdruck. Regelmäßige Checks sichern das Niveau nach Updates und Erweiterungen.
consequence-default = Ohne Korrektur bleibt die Seite hinter erreichbarem Standard — Verbesserungspotenzial wird nicht genutzt.

# Cover score row
cover-card-certificate = Auditstatus
cover-card-accessibility = Accessibility
cover-card-issues = Issues
cover-card-critical-high-suffix = kritisch/hoch
cover-card-average = Durchschnitt
cover-card-urls = URLs
cover-card-violations-suffix = Verstöße
batch-cover-eyebrow = Automatisierter Batch-Audit-Report
batch-cover-title = Barrierefreiheits-Prüfbericht
batch-cover-kicker = Domainweiter Website-Check mit Fokus auf Accessibility, SEO und Performance
batch-cover-frame-title = Audit-Rahmen
batch-cover-frame-domain = Domain
batch-cover-frame-date = Prüfdatum
batch-cover-frame-urls = Geprüfte URLs
batch-cover-frame-certificate = Auditstatus
batch-cover-frame-modules = Aktive Module
batch-cover-frame-version = Tool-Version
batch-cover-frame-scope = Prüfumfang
batch-scope-sample = Stichprobe — { $audited } von { $total } URLs ({ $source }, erste { $audited })
batch-scope-full = Vollständig — alle { $total } URLs ({ $source })
batch-source-sitemap = Sitemap
batch-source-crawl = Crawl
batch-source-url_file = URL-Liste
panel-modules-overview = Modulübersicht
section-tech-detail-metrics = Technische Detailmetriken
column-action = Maßnahme
column-priority = Priorität
column-timeframe = Zeitrahmen
column-component = Komponente
column-area = Bereich
section-quick-wins = Quick Wins
section-medium-actions = Mittelfristige Maßnahmen
section-structural-actions = Strukturelle Maßnahmen
section-next-steps-recommended = Empfohlene nächste Schritte
section-next-steps-block = Weiteres Vorgehen

# Batch sections
batch-section-status = Status der Website
batch-section-most-frequent = Häufigste Probleme
batch-section-most-frequent-violations = Häufigste Verstöße
batch-col-problem = Problem
batch-col-occurrences = Vorkommen
batch-col-pages = Seiten
batch-col-priority = Priorität
batch-col-source = Quelle
batch-col-status-code = Status
batch-col-final-url = Final-URL
batch-col-page-a = Seite A
batch-col-page-b = Seite B
batch-col-similarity = Ähnlichkeit
batch-col-risk = Risiko
batch-col-page-type = Seitentyp
batch-col-attributes = Merkmale
batch-col-top-issues = Top-Probleme
batch-col-pages-list = Seiten
batch-col-share = Anteil
batch-col-relevance = Relevanz
batch-col-schema-type = Schema-Typ
batch-col-profile = Profil
batch-col-severity = Schweregrad
batch-col-description = Beschreibung
batch-col-metric = Metrik
batch-col-budget = Budget
batch-col-links-to = Links zu
batch-col-links-from = Links von
batch-col-words = Wörter
batch-action-plan-title = Maßnahmenplan
batch-section-broken-links-internal = Interne Broken Links
batch-section-broken-links-external = Externe Broken Links
batch-section-external-links = Externe Links
batch-section-redirect-chains = Redirect-Ketten
batch-redirect-chains-title = Redirect-Ketten (> 1 Hop)
batch-col-target = Ziel
batch-url-ranking-title = URL-Ranking
batch-url-ranking-intro = Übersicht aller geprüften URLs, sortiert nach Score. URLs mit niedrigerem Score haben höheren Handlungsbedarf.
batch-render-blocking-section = Render-Blocking & Assets
batch-render-blocking-kv-title = Render-Blocking-Übersicht (domainweit)
batch-render-blocking-intro = Render-blockierende Ressourcen und Third-Party-Traffic, aggregiert über alle geprüften Seiten.
batch-section-tech-url-matrix = Technische URL-Matrix
batch-section-tech-url-matrix-intro = Verdichtete Übersicht aller geprüften URLs mit Fokus auf technische Priorisierung. Jede Zeile zeigt Score, Problemintensität und den größten Hebel für die nächste Optimierungsrunde.
batch-table-pages-overview = Seiten-Übersicht
batch-table-focus-pages = Fokus auf problematische Seiten
batch-table-page-type-distribution = Seitentyp-Verteilung
batch-table-schema-distribution = Schema-Typ-Verteilung
batch-table-top-pages = Stärkste Content-Seiten
batch-table-broken-internal = Kaputte interne Links
batch-table-broken-external = Kaputte externe Links
batch-section-performance-budgets = Performance Budgets
findings-card-key-problem = Problem
findings-card-key-cause = Ursache
label-improvement-suggestions = Verbesserungsvorschläge
label-recommendations = Empfehlungen
label-severity = Schweregrad
label-classification = Einordnung
col-aspect = Aspekt
col-value = Wert
col-metric = Metrik
section-perf-budget-violations = Performance-Budget-Verletzungen
section-user-experience = Nutzererlebnis
section-technical-complexity = Technische Komplexität
section-seo-analysis = SEO-Analyse
section-serp-analysis = SERP-Analyse
section-page-health = Seitengesundheit
section-robots-audit = robots.txt Audit
section-seo-content-profile = SEO-Inhaltsprofil
section-security = Sicherheit
section-mobile-usability = Mobile Nutzbarkeit
section-dark-mode = Dark Mode
section-content-sections = Content-Abschnitte
section-detected-entities = Erkannte Entitäten
section-ux = User Experience
section-journey = User Journey
section-issue-overview = Problemübersicht
section-link-suggestions = Verlinkungsvorschläge
comparison-cover-title = Wettbewerbsvergleich
comparison-domain-ranking = Domain-Ranking
comparison-module-comparison = Modul-Vergleich
comparison-top-findings-per-domain = Wichtigste Findings je Domain

# Device preview
section-device-preview = Gerätevorschau
section-device-preview-no-screenshots = Keine Screenshots verfügbar — die Aufnahme ist bei diesem Audit fehlgeschlagen. Prüfe, ob Chrome installiert und erreichbar ist (`auditmysite doctor`).

# Diagnosis section tables
diagnosis-table-categories = Kategorie-Übersicht
diagnosis-table-clusters = Problemcluster
diagnosis-col-category = Kategorie
diagnosis-col-findings = Befunde
diagnosis-col-worst-severity = Schwerster Schweregrad
diagnosis-col-occurrences = Vorkommen
diagnosis-col-max-severity = Max. Schweregrad

# Batch module overview
batch-panel-module-averages = Modulübersicht (Ø über alle URLs)

# Heuristic indicator (shared by UX, Journey, AI Visibility)
label-heuristic-indicator = Heuristische Schätzung auf Basis struktureller Signale

# Performance section
perf-score-card = Performance Score
perf-technical-indicators = Technische Indikatoren
perf-lab-data-note = Lab-Daten
perf-lab-data-body = Alle Performance-Werte stammen aus einer lokalen Headless-Messung (Chrome/CDP), nicht aus Felddaten (CrUX/Real-User-Monitoring). Mit „Lab-Schätzung" markierte Kennzahlen (INP, TTI, Speed Index) sind abgeleitete Näherungen, keine direkten Messwerte.
perf-render-blocking-analysis = Render-Blocking Analyse
perf-measurement-warnings-title = Messtechnische Einschränkungen
perf-warning-lcp-missing = LCP nicht erfasst: Der PerformanceObserver hat keinen LCP-Eintrag geliefert (typisch bei starker Netzwerkdrosselung) — LCP-Score fehlt in der Bewertung.
perf-warning-tbt-zero = TBT = 0 ms bei schwerer Seite (LCP > 3 s): Headless-Chrome registriert keine Long Tasks — TBT ist wahrscheinlich unterschätzt.
perf-warning-si-fallback = Speed Index entspricht LCP: Formelschätzung ohne unabhängigen Messwert (0,35 × FCP + 0,65 × LCP).
perf-warning-tti-fallback = TTI entspricht LCP: Formelschätzung ohne direkten Interaktivitätswert.
perf-warning-inp-missing = INP nicht messbar: Headless-Browser erzeugen keine Nutzereingaben — Interaktionslatenz konnte nicht erfasst werden.
perf-warning-lh-mobile-gap = Desktop-Score: { $desktop } · LhMobile-Score: { $mobile } — Abweichung { $gap } Punkte. Der Primärscore basiert auf der ungedrosselten Messung.

# SEO section
seo-score-card = Technisches SEO
seo-score-card-description = Misst technische Signale (Meta, Struktur, Schema, hreflang). Inhaltliche Tiefe wird separat bewertet.
seo-maturity = Reifegrad

# UX section
ux-score-card = UX Score (Indikator)
ux-dimensions = UX-Dimensionen

# Journey section
journey-score-card = Journey Score (Indikator)
journey-page-type-dimensions = Seitentyp & Dimensionen

# Budget section
budget-callout-exceeded = Budget überschritten
budget-callout-warnings = Budget-Hinweise
budget-table-metric = Metrik
budget-table-actual = Ist-Wert
budget-table-overage = Überschreitung
budget-table-title = Budget-Details

# Finding narrative arc labels (Diagnose → Ursache → Wirkung → Umsetzung)
finding-narrative-diagnose = Diagnose
finding-narrative-ursache = Ursache
finding-narrative-wirkung = Wirkung
finding-narrative-umsetzung = Umsetzung

# Finding cards (shared across executive / standard / technical)
finding-key-problem = Problem
finding-key-impact = Was Nutzer erleben
finding-key-cause = Ursache
finding-key-fix = Was tun
finding-key-effort = Aufwand
finding-key-quick-win = Quick Win — wenige Stunden, hohe Wirkung
finding-tech-context = Technische Einordnung
finding-tech-rule = Regel
finding-tech-instances = Instanzen
finding-tech-affected-elements = Betroffene Elemente
finding-tech-other-occurrences = Weitere ähnliche Vorkommen
finding-tech-affected-urls = Betroffene URLs
finding-elements = Elemente
finding-occurrences = Vorkommen
finding-element-types = Element-Typen
finding-affected-selectors = Betroffene Selektoren
finding-recommendation = Empfehlung
finding-wrong = ✕ Falsch
finding-right = ✓ Richtig
finding-location = Fundstelle
finding-note = Hinweis
finding-representative-occurrences = Repräsentative Fundstellen
finding-occurrence = Fundstelle
finding-suggested-fix = Vorgeschlagene Code-Korrektur
finding-pattern = Muster
finding-frequent-patterns = Häufige Muster
finding-reference = Referenz

# SEO / Tracking section
seo-tracking-services = Tracking und externe Dienste
seo-kv-title = Technisches SEO
seo-serp-readiness = SERP-Bereitschaft
seo-serp-signals = SERP-Signale
seo-page-health-issues = Gefundene Probleme
seo-page-health-url-analysis = URL-Analyse
seo-page-html-validation = HTML-Validierung

# Security section
security-score-card = Security Score

# Mobile section
mobile-score-card = Mobile Score
mobile-configured = Konfiguriert
mobile-touch-targets = Touch Targets
mobile-viewport-config = Viewport-Konfiguration
mobile-font-analysis = Schriftanalyse
mobile-content-sizing = Inhaltsanpassung

# AI Visibility section
ai-score-card = AI-Sichtbarkeit (Indikator)

# PM Extras
pm-extra-prioritize = Priorisierung der Maßnahmen
pm-extra-qa = Qualitätssicherung und Testing
pm-extra-responsibilities = Verantwortlichkeiten festlegen

# Business Impact Prefixes
impact-prefix-widespread = Weitverbreitet:{" "}
impact-prefix-frequent = Häufig:{" "}

# Business Impact Bases
impact-base-seo = Kann Sichtbarkeit in Suchmaschinen reduzieren und organischen Traffic senken.
impact-base-security = Erhöht Angriffsfläche und Risiko für Datenverlust.
impact-base-performance = Verschlechtert Ladezeit und Nutzererlebnis, erhöht Absprungrate.
impact-base-mobile = Beeinträchtigt mobile Nutzbarkeit für die Mehrheit der Nutzer.
impact-base-accessibility-readability = Beeinträchtigt Lesbarkeit für Nutzer mit Sehschwäche.
impact-base-accessibility-exclude = Kann Nutzer ausschließen und rechtliches Risiko erhöhen.
impact-base-accessibility-voice = Kann Nutzungshürden erhöhen und Interaktionen mit zentralen Elementen verhindern.
impact-base-accessibility-default = Beeinträchtigt Qualität und Nutzererlebnis der Website.

# Humanized Action Texts
action-human-aria-label = Interaktive Elemente (Buttons, Links) verständlich benennen
action-human-alt-text = Bilder mit beschreibendem Alternativtext versehen
action-human-contrast = Farbkontraste für Text und UI-Elemente verbessern
action-human-form-label = Formularfelder eindeutig beschriften
action-human-heading = Überschriften-Hierarchie logisch strukturieren
action-human-keyboard = Tastaturnavigation und Fokus-Reihenfolge sicherstellen
action-human-skip-link = Sprunglinks für Screenreader-Nutzer ergänzen
action-human-lang-attr = Seitensprache korrekt im HTML auszeichnen
action-human-page-title = Aussagekräftigen Seitentitel vergeben
action-human-link-text = Links verständlich und eindeutig beschriften
action-human-landmark = Seitenstruktur mit Orientierungspunkten auszeichnen

# User Effects
effect-user-buttons = Nutzer verstehen Schaltflächen sofort — weniger Fehlklicks
effect-user-links = Navigation klarer — Nutzer finden Ziele schneller
effect-user-aria = Alle Bedienelemente klar benannt — Screenreader-Nutzung ohne Ratespiel
effect-user-images = Bilder verständlich für Nutzer ohne Sehvermögen
effect-user-contrast = Text für alle Nutzer gut lesbar — auch bei schlechten Lichtverhältnissen
effect-user-forms = Formulare ausfüllbar ohne Verwirrung — weniger Abbrüche
effect-user-heading = Inhaltsstruktur sofort erfassbar — schnellere Orientierung
effect-user-skip = Tastaturnutzer gelangen direkt zum Hauptinhalt
effect-user-keyboard = Vollständige Bedienbarkeit ohne Maus
effect-user-language = Screenreader liest Inhalte in korrekter Sprache und Betonung
effect-user-title = Seite klar identifizierbar in Browser-Tab und Suche
effect-user-landmark = Screenreader-Nutzer navigieren strukturiert durch die Seite
effect-user-default-quick = Direkte, spürbare Verbesserung der Nutzererfahrung
effect-user-default-medium = Merkliche Verbesserung für betroffene Nutzergruppen
effect-user-default-structural = Langfristig inklusivere Nutzererfahrung für alle

# Conversion Effects
effect-conversion-links = Klarere Navigation → weniger Absprünge
effect-conversion-contrast = Bessere Lesbarkeit → höhere Verweildauer
effect-conversion-heading = Strukturklarheit → schnellere Orientierung
effect-conversion-language = Korrekte Sprachausgabe → keine Abbrüche durch Vorlesefehler
effect-conversion-default-quick = Schnell wirksam — messbar innerhalb von Tagen
effect-conversion-default-medium = Mittelfristig messbare UX-Verbesserung
effect-conversion-default-structural = Solide technische Basis für weiteres Wachstum

# Narrative Arc Formatters
narrative-diagnose-multiple = { $count }× festgestellt: { $desc }
narrative-cause-suffix-high = {" "}Dies ist eine strukturelle Lücke mit hoher Priorität.
narrative-cause-accessibility = Dies resultiert typischerweise aus fehlendem oder unvollständigem semantischen Markup.
narrative-cause-seo = Dies resultiert typischerweise aus fehlenden oder unvollständigen Meta-Informationen.
narrative-cause-performance = Dies resultiert typischerweise aus nicht optimierten Assets oder render-blockierenden Ressourcen.
narrative-cause-security = Dies resultiert typischerweise aus fehlenden oder falsch konfigurierten HTTP-Sicherheits-Headern.
narrative-cause-mobile = Dies resultiert typischerweise aus einem Layout- oder Größenproblem, das nicht für kleine Bildschirme angepasst wurde.
narrative-cause-default = Dies resultiert typischerweise aus einer Konfigurations- oder Implementierungslücke.
narrative-impact-critical = Dies blockiert eine kritische Nutzerinteraktion — betroffene Nutzer können die Aufgabe nicht abschließen.
narrative-impact-high = Dies beeinträchtigt die Nutzbarkeit für betroffene Nutzergruppen erheblich.
narrative-impact-medium = Dies verursacht einen spürbaren Reibungspunkt für bestimmte Nutzer.
narrative-impact-low = Dies ist ein kleineres Problem, das eine Teilgruppe der Nutzer betreffen kann.
narrative-effort-quick = Quick Win — wenige Stunden
narrative-effort-medium = Mittelfristig — einige Tage
narrative-effort-structural = Strukturelle Änderung erforderlich
narrative-owner-dev = Entwicklung
narrative-owner-editorial = Redaktion
narrative-owner-designux = Design / UX
narrative-owner-pm = Projektleitung
narrative-implementation-format = { $recommendation } ({ $effort }, Verantwortlich: { $owner })
narrative-implementation-empty = { $effort } — { $owner }

# Levers and Contexts
lever-accessibility-biggest = Größter Hebel: { $finding }
lever-accessibility-default = Größter Hebel: Ergebnisse stabil halten und manuell nachprüfen
context-accessibility-none = Keine automatisch erkannten Barrieren im aktuellen Lauf.
context-accessibility-summary = { $total } erkannte Problemgruppe(n), davon { $high } mit hoher Priorität.
card-accessibility-none = Keine High-Priority-Funde
card-accessibility-summary = { $high } Problemgruppe(n) mit hoher Priorität
lever-performance-dom = Größter Hebel: DOM-Größe reduzieren ({ $dom_nodes } Knoten)
lever-performance-load = Größter Hebel: Ladezeit senken ({ $load } ms)
lever-performance-default = Größter Hebel: Render-Pfad und Asset-Größe weiter optimieren
context-performance-good-vitals = { $fcp } · Nutzererlebnis: schnell — technische Komplexität zieht den Gesamtscore herunter.
context-performance-dom-nodes = { $n } DOM-Knoten
context-performance-dom-na = DOM-Knoten n/a
context-performance-summary = { $fcp }, { $ttfb }, { $dom }.
card-performance-dom = { $dom_nodes } DOM-Knoten
card-performance-load = Ladezeit { $load } ms
card-performance-default = Render-Pfad weiter optimieren
perf-lab-estimate-suffix = {" "}(Lab-Schätzung)
recommendation-performance-lcp = Größtes sichtbares Element schneller laden: Hero-Bilder optimieren, priorisieren und kritische Styles früher ausliefern.
recommendation-performance-fcp = Ersten sichtbaren Inhalt früher ausliefern: render-blockierende CSS- und JavaScript-Dateien reduzieren.
recommendation-performance-tbt = Haupt-Thread entlasten: große JavaScript-Aufgaben aufteilen und nicht benötigte Skripte später laden.
recommendation-performance-cls = Layout-Verschiebungen vermeiden: Medien, Banner und dynamische Inhalte mit festen Platzhaltern reservieren.
recommendation-performance-dom = DOM-Struktur verschlanken: große Komponenten, tiefe Container-Hierarchien und wiederholte Markup-Blöcke reduzieren.
recommendation-performance-load = Gesamte Ladezeit senken: große Assets komprimieren, Caching schärfen und Drittanbieter-Skripte prüfen.
recommendation-performance-default = Die Kernmetriken sind stabil. Nächster Hebel: Seitengröße und Drittanbieter-Skripte regelmäßig überwachen, damit das Niveau gehalten wird.
lever-seo-meta = Größter Hebel: Meta-Daten bereinigen ({ $open_issues } offene Punkte)
lever-seo-social = Größter Hebel: Social-Meta-Daten vervollständigen
lever-seo-default = Größter Hebel: Struktur- und Inhalts-Signale weiter schärfen
context-seo-summary = { $meta_issues } Meta-Probleme, { $h1 } H1, { $schema_count } strukturierte Daten erkannt.
card-seo-meta = { $meta_issues } Meta-Probleme offen
card-seo-schema = { $schema_count } strukturierte Daten erkannt
lever-security-headers = Größter Hebel: fehlende Security-Header ergänzen ({ $missing_headers } Kernheader)
lever-security-default = Größter Hebel: Header-Regeln und TLS-Setup weiter härten
context-security-summary-https = { $present_headers } von 8 Kern-Headern vorhanden, HTTPS aktiv.
context-security-summary-nohttps = { $present_headers } von 8 Kern-Headern vorhanden, HTTPS fehlt.
card-security-summary = { $present_headers } von 8 Kern-Headern vorhanden
recommendation-security-https = HTTPS durchgängig erzwingen und ein gültiges TLS-Zertifikat für alle Varianten der Domain sicherstellen.
recommendation-security-csp = Content-Security-Policy ergänzen und nur die tatsächlich benötigten Skript-, Style- und Medienquellen erlauben.
recommendation-security-hsts = HSTS ergänzen, damit Browser die Seite dauerhaft nur noch per HTTPS laden.
recommendation-security-coop = Cross-Origin-Opener-Policy (COOP) ist relevant, wenn die Seite SharedArrayBuffer, hochauflösende Timer oder Cross-Origin-Popup-Kommunikation nutzt. Mit same-origin Cross-Origin-Isolation aktivieren — für Standardseiten nicht erforderlich.
recommendation-security-corp = Cross-Origin-Resource-Policy (CORP) ist relevant, wenn die Seite Schriften, Skripte oder Medien ausliefert, die von fremden Origins nicht geladen werden sollen (Spectre-Mitigation). Auf same-origin oder same-site setzen — nicht erforderlich, wenn Ressourcen bewusst öffentlich sind.
recommendation-security-permissions = Permissions-Policy definieren und nur die Browser-Funktionen freigeben, die auf der Seite wirklich benötigt werden.
recommendation-security-referrer = Referrer-Policy setzen, damit bei Weiterleitungen und externen Aufrufen nicht mehr Informationen als nötig übergeben werden.
recommendation-security-default = Die grundlegenden Security-Header sind sauber gesetzt. Nächster Schritt: Richtlinien regelmäßig prüfen und an neue Skript- oder Integrationsquellen anpassen.
lever-mobile-small = Größter Hebel: Touch Targets vergrößern ({ $small_targets } zu klein)
lever-mobile-crowded = Größter Hebel: Abstände mobiler Bedienelemente erhöhen ({ $crowded_targets })
lever-mobile-default = Größter Hebel: mobile Lesbarkeit und Touch-Flows weiter optimieren
context-mobile-proper = Viewport korrekt gesetzt, { $small_targets } zu kleine Touch Targets, { $crowded_targets } zu enge Abstände.
context-mobile-improper = Viewport nicht sauber konfiguriert, { $small_targets } zu kleine Touch Targets, { $crowded_targets } zu enge Abstände.
card-mobile-small = { $small_targets } zu kleine Touch Targets
card-mobile-crowded = { $crowded_targets } zu enge Abstände
card-mobile-proper = Viewport korrekt gesetzt
card-mobile-improper = Viewport prüfen

# Tracking summary
tracking-summary-zaraz-clean = Zaraz ist auf der Seite erkennbar. Zusätzlich wurden im Lauf keine weiteren Tracking-Cookies oder externen Tracking-Signale festgestellt.
tracking-summary-zaraz-signals = Auf der Seite sind Tracking- oder Consent-nahe Signale erkennbar. Prüfen Sie insbesondere externe Einbindungen, Cookie-Setzung und den tatsächlichen Auslösezeitpunkt nach Einwilligung.
tracking-summary-fonts = Es werden extern gehostete Google Fonts geladen. Das ist datenschutz- und performance-relevant und sollte bewusst geprüft werden.
tracking-summary-signals = Es wurden Tracking-Signale erkannt. Prüfen Sie Einwilligung, Auslösezeitpunkt und die Herkunft der eingebundenen Dienste.
tracking-summary-clean = Im aktuellen Lauf wurden keine externen Google Fonts, keine Tracking-Cookies und keine weiteren Tracking-Signale erkannt.

# PDF detail module additions
pdf-budget-violations-summary =
    { $total } Budget-{ $total ->
        [one] Verletzung
       *[other] Verletzungen
    } erkannt: { $error_count } kritisch (>50% überschritten), { $warning_count } { $warning_count ->
        [one] Warnung
       *[other] Warnungen
    }.
pdf-perf-section-title = Performance — Nutzererlebnis & Technische Metriken
pdf-perf-intro-user-experience = Core Web Vitals, Render-Blocking — wie schnell die Seite für Nutzer wirkt
pdf-perf-intro-technical-complexity = DOM-Größe, Ressourcen-Loading, Blocking-Budget
pdf-perf-overview-title = Performance — Überblick
pdf-perf-throttled-title = Performance unter gedrosselten Bedingungen
pdf-perf-throttled-profile = Profil
pdf-perf-cls-title = CLS — Layout-Verschiebungen
pdf-perf-cls-value = Wert
pdf-perf-cls-time = Zeitpunkt
pdf-perf-tp-title = Drittanbieter-Ressourcen
pdf-perf-tp-total-origins = Drittanbieter gesamt
pdf-perf-tp-total-transfer = Übertragung gesamt
pdf-perf-tp-requests = Anfragen
pdf-perf-tp-impact = Einfluss
pdf-perf-tp-significant = Signifikant (>20% der Seitengröße)
pdf-perf-tp-types = Typen
pdf-perf-cc-title = Kritische Request-Kette
pdf-perf-cc-max-depth = Max. Tiefe
pdf-perf-cc-path = Kritischer Pfad
pdf-perf-cc-total-requests = Anfragen gesamt
pdf-perf-min-title = Unminifizierte Assets
pdf-perf-min-files = Unminifizierte Dateien
pdf-perf-min-savings = Geschätzte Einsparung
pdf-perf-min-type = Typ
pdf-perf-min-saving-col = Einsparung
pdf-perf-cov-title = Code-Abdeckung
pdf-perf-cov-js-used = JS genutzt
pdf-perf-cov-css-used = CSS genutzt
pdf-perf-anim-title = Nicht-composited Animationen
pdf-perf-anim-total = Gesamt
pdf-perf-anim-properties = Eigenschaften
pdf-seo-indicator-note = Indikatoren sind heuristische Schätzwerte auf Basis messbarer Signale — kein direktes Ranking-Signal, sondern Hinweis auf Optimierungspotenzial.
pdf-seo-indicator-title = Indikator-basierte Metriken
pdf-seo-overview-title = SEO — Überblick
pdf-seo-maturity = Reifegrad
pdf-seo-field = Feld
pdf-seo-value = Wert
pdf-seo-meta-tags-title = Meta-Tags
pdf-seo-meta-description = Beschreibung
pdf-seo-meta-issues-title = Meta-Tag Probleme
pdf-seo-headings = Überschriften
pdf-seo-social-tags = Social Tags
pdf-seo-more-signals = { $count } weitere Signale im detaillierten Anhang.
pdf-seo-ie-title = Bild-Effizienz
pdf-seo-ie-total = Bilder gesamt
pdf-seo-ie-modern = Moderne Formate
pdf-seo-ie-legacy = Legacy-Formate
pdf-seo-ie-oversized-title = Übergroße Bilder (Top 5)
pdf-seo-ie-source = Quelle
pdf-seo-ie-natural = Nativ
pdf-seo-ie-displayed = Angezeigt
pdf-serp-summary = { $total } Signale geprüft — { $pass } OK, { $warning } Warnungen, { $fail } Fehler.
pdf-serp-category = Kategorie
pdf-serp-rich-results-text = Rich-Result-Typen möglich: { $types }
pdf-ph-issue = Problem
pdf-ph-w3c-title = W3C HTML Validator
pdf-ph-www-title = www-Konsolidierung
pdf-ph-check = Prüfung
pdf-ph-count = Anzahl
pdf-robots-error = robots.txt konnte nicht geladen werden: { $err }
pdf-robots-no-access = Kein Zugriff
pdf-robots-block-all-body = Alle Crawler vollständig gesperrt (User-agent: * / Disallow: /). Auf Staging-Domains ist das korrekt — auf der Produktiv-Domain würde dies das vollständige Crawling durch Suchmaschinen verhindern.
pdf-robots-block-all-title = Alle Crawler gesperrt
pdf-robots-limit-ai-body = KI-Suchbots (z. B. PerplexityBot, Amazonbot) sind blockiert. Das ist eine bewusste Entscheidung — Inhalte erscheinen nicht in KI-generierten Antworten. Das Sperren von KI-Trainingsbots (GPTBot etc.) ist dagegen übliche Praxis und kein Problem.
pdf-robots-limit-ai-title = Eingeschränkte KI-Sichtbarkeit
pdf-robots-training-blocked-body = Policy: { $policy } — KI-Trainingsbots ({ $bots }) sind gesperrt, KI-Suchbots haben Zugang. Das entspricht der empfohlenen Standardkonfiguration.
pdf-robots-training-blocked-title = KI-Training blockiert (Standard)
pdf-robots-crawler-rules = Crawler-Regeln
pdf-robots-allowed = Erlaubt
pdf-robots-blocked = Gesperrt
pdf-robots-status-fully-blocked = Vollständig gesperrt
pdf-robots-status-partially-blocked = Teilweise gesperrt
pdf-robots-sitemap-entries = Sitemap-Einträge
pdf-robots-crawl-delay-title = Crawl-Delay-Werte
pdf-robots-crawl-delay-value = { $delay } Sekunden
pdf-seo-profile-aspect = Aspekt
pdf-seo-profile-content-profile = Inhaltsprofil
pdf-seo-profile-structured-data = Strukturierte Daten
pdf-seo-profile-page-profile = Seitenprofil
pdf-seo-profile-content-depth = Content-Tiefe
pdf-seo-profile-structure-quality = Strukturqualität
pdf-seo-profile-media-balance = Medienbalance
pdf-seo-profile-website-identity = Website-Identität
pdf-seo-profile-content-type = Inhaltstyp
pdf-seo-profile-language = Sprache
pdf-seo-profile-schema-types = Schema-Typen
pdf-seo-profile-schema-type-col = Schema-Typ
pdf-seo-profile-completeness = Vollständigkeit
pdf-seo-profile-structured-data-title = Strukturierte Daten ({ $count } Schemas)
pdf-seo-profile-rating = Bewertung
pdf-seo-profile-classification = Einstufung
pdf-seo-profile-strength-title = SEO-Signalstärke (Gesamt: { $pct }%)
pdf-seo-profile-maturity-title = SEO-Reifegrad
pdf-seo-profile-techniques = Techniken
pdf-seo-profile-techniques-value = { $used } von { $total } erkannt
pdf-sec-https-yes = Ja
pdf-sec-https-unclear = Unklar
pdf-sec-protection-title = Schutzinfrastruktur
pdf-ux-overview-title = UX — Überblick
pdf-ux-more-issues = { $count } weitere Befunde im detaillierten Anhang.
pdf-journey-overview-title = Journey — Überblick
pdf-journey-detected-page-type = Erkannter Seitentyp
pdf-journey-more-issues = { $count } weitere Befunde im detaillierten Anhang.
pdf-dm-status-supported = Unterstützt
pdf-dm-status-not-supported = Nicht unterstützt
pdf-dm-score-title = Dark Mode Score
pdf-dm-methods = Methoden
pdf-dm-css-variables = CSS Variablen
pdf-dm-overview-title = Dark Mode Übersicht
pdf-dm-support = Unterstützung
pdf-dm-methods-impl = Implementierungsmethoden
pdf-dm-yes = Ja
pdf-dm-no = Nein
pdf-dm-css-custom-props = CSS Custom Properties (Farben)
pdf-dm-contrast-violations = Kontrast-Violations im Dark Mode
pdf-dm-only-issues = Nur-Dark-Mode-Probleme
pdf-dm-only-issues-val = { $count } (nicht im Light Mode)
pdf-dm-resolved-issues = Im Dark Mode behoben
pdf-dm-resolved-issues-val = { $count } Light-Mode-Probleme verschwinden im Dark Mode
pdf-dm-issue-title = Dark Mode Problem
pdf-dm-note-title = Dark Mode Hinweis
pdf-sq-section-title = Quellenqualität
pdf-sq-overview-title = Quellqualität — Überblick
pdf-sq-score-title = Quellenqualität (Indikator)
pdf-sq-score-desc-format = Grade: { $grade } — { $quality } · Heuristische Schätzung, kein Messwert
pdf-sq-success = Alle Quellenqualitäts-Signale sind in Ordnung. Keine kritischen Probleme erkannt.
pdf-ts-section-title = Tech-Stack
pdf-ts-score-title = Stack-Sicherheitsscore
pdf-ts-technology = Technologie
pdf-ts-confidence = Konfidenz
pdf-ts-detected-title = Erkannte Technologien
pdf-ts-confidence-high = Hoch
pdf-ts-confidence-medium = Mittel
pdf-ts-confidence-low = Niedrig
pdf-ts-finding = Befund
pdf-ts-severity = Schweregrad
pdf-ts-findings-title = Stack-Sicherheitsbefunde
pdf-ai-indicator-note = Experimentelle Heuristiken — keine standardisierte Metrik. Die Indikatoren sind heuristische Schätzwerte auf Basis messbarer Signale, kein direktes Ranking-Signal. Sie zeigen Optimierungspotenzial, beweisen aber kein konkretes LLM-Verhalten.
pdf-ai-section-title = AI-Sichtbarkeit (Indikator)
pdf-ai-overview-title = KI-Sichtbarkeit — Überblick
pdf-ai-score-title = AI-Sichtbarkeit (Indikator)
pdf-ai-score-desc-format = Grade: { $grade } — { $quality } · Heuristische Schätzung, kein Messwert
pdf-ai-success = Alle KI-Sichtbarkeits-Signale sind in Ordnung. Kein Optimierungsbedarf.
pdf-ai-readability = KI-Lesbarkeit
pdf-ai-citability = Zitierbarkeit
pdf-ai-tech-readability = Technische KI-Lesbarkeit
pdf-ai-policy = AI-Policy
pdf-ai-more-signals = { $count } weitere Signale im detaillierten Anhang.
pdf-ai-sections-title = Abschnitte
pdf-ai-section-col = Abschnitt
pdf-ai-words-col = Wörter
pdf-ai-rec-title = Empfehlung
pdf-ai-entities-title = Entitäten
pdf-ai-entity-col = Entität
pdf-ai-relations-title = Beziehungen
pdf-ai-subject-col = Subjekt
pdf-ai-relation-col = Beziehung
pdf-ai-object-col = Objekt
pdf-ai-policy-blocks-all-body = Alle Crawler gesperrt (Disallow: *) — auch KI-Suchbots haben keinen Zugang.
pdf-ai-policy-blocks-all-title = Kein KI-Zugang
pdf-ai-policy-limited-title = KI-Sichtbarkeit eingeschränkt
pdf-ai-policy-limited-body = KI-Suchbots blockiert — Inhalte erscheinen nicht in KI-generierten Antworten
pdf-ai-policy-training-body = KI-Trainingsbots blockiert, KI-Suchbots haben Zugang — empfohlener Standard
pdf-cv-section-title = Content Visibility & Trust (Indikator)
pdf-cv-overview-title = Content-Sichtbarkeit — Überblick
pdf-cv-overview-body = Aggregierter Indikator aus SEO-, Quellenqualitäts- und KI-Sichtbarkeitssignalen. Umfasst organische Indexierbarkeit, E-E-A-T-Autoritätssignale, lokale Geschäftspräsenz, Inhaltstiefe und topische Relevanz-Heuristiken. Heuristischer Schätzwert — kein direkt gemessener Wert.
pdf-cv-signals-analyzed = { $signals } Signale analysiert, { $problems } Hinweise auf Optimierungsbedarf.
pdf-cv-manual-review-title = Manuell prüfen
pdf-bp-score-title = Best Practices Score
pdf-bp-success = Keine Konsolfehler und keine anfälligen Bibliotheken erkannt.
pdf-bp-console-errors-title = Konsolenfehler
pdf-bp-message-col = Meldung
pdf-bp-vuln-libs-title = Anfällige Bibliotheken
pdf-bp-lib-col = Bibliothek
pdf-bp-severity-col = Schwere
pdf-bp-fix-col = Lösung
pdf-bp-libs-up-to-date = Erkannte Bibliotheken scheinen aktuell zu sein — keine bekannten Schwachstellen.
pdf-perf-cov-js-val = { $pct }% ({ $unused } KB ungenutzt)
pdf-perf-cov-css-val = { $pct }% ({ $used }/{ $total } Regeln genutzt)
pdf-cv-area-organic-visibility = Organische Sichtbarkeit
pdf-cv-area-local-business = Local Business & Vertrauensdaten
pdf-cv-area-eeat = E-E-A-T-Indizien
pdf-cv-area-content-depth = Inhaltstiefe & Lokalisierung
pdf-cv-area-topical-authority = Topical Authority (Heuristik)

batch-top-issues-intro = Die folgenden Problemgruppen treten über mehrere URLs hinweg auf. Durch Behebung dieser Probleme wird die größte Verbesserung erzielt, da sie viele Seiten gleichzeitig betreffen.
batch-col-affected-urls = Betr. URLs
batch-meta-global = Global — betrifft alle Seiten
batch-meta-individual = Einzelne Seiten — isolierter Inhalt
batch-meta-occurrences = Vorkommen
batch-meta-affected-urls = betroffene URLs
batch-meta-effort = Aufwand
batch-meta-scope = Reichweite
batch-meta-impact-user = Auswirkung (Nutzer)
batch-meta-impact-business = Auswirkung (Business)
batch-meta-fix = Empfehlung
batch-meta-classification = Einordnung
batch-action-plan-intro = Die folgenden Maßnahmen sind nach Aufwand und Wirkung priorisiert. Maßnahmen, die viele Seiten gleichzeitig verbessern, haben Vorrang.
batch-budget-pages-col = Betr. Seiten
batch-budget-table-title = Performance-Budget-Verstöße (domainweit)
batch-budget-intro = Auf wie vielen Seiten welche Performance-Budgets überschritten wurden.
batch-crawl-internal-intro = Für den Crawl ab { $seed } wurden { $checked } interne Linkziele geprüft. { $broken } kaputte interne Verlinkungen wurden erkannt.
batch-crawl-col-target = Ziel
batch-crawl-col-type = Typ
batch-crawl-label-direct = direkt
batch-crawl-label-hops = Hops
batch-crawl-no-broken-internal = Keine kaputten internen Links im geprüften Crawl-Set erkannt.
batch-crawl-external-intro = { $checked } externe Linkziele geprüft. { $broken } kaputte externe Verlinkungen erkannt.
batch-crawl-external-clean = { $checked } externe Linkziele geprüft — keine kaputten externen Links erkannt.
batch-crawl-redirect-chains-intro = { $count } Links mit mehr als einem Redirect-Hop erkannt.
batch-matrix-col-page = Seite
batch-matrix-col-title = Titel
batch-seo-potential-title = Content & SEO-Potenzial
batch-seo-potential-intro = Content-Stärken und -Schwächen mit direktem Bezug zu Rankings, Sichtbarkeit und Conversion. Jede Auffälligkeit ist an eine konkrete Handlung geknüpft.
batch-seo-issues-title = Content-Probleme mit Handlungsbedarf
batch-seo-impact-ranking-loss = Rankingverlust + geringere Conversion wahrscheinlich
batch-seo-impact-weak-visibility = Schwache organische Sichtbarkeit
batch-seo-impact-opt-potential = Optimierungspotenzial für SEO
batch-seo-recommendation-words = { $page_type } — { $impact } → +300–800 Wörter strukturierter Inhalt empfohlen
batch-seo-profile-label = { $url } (Profil: { $score }/100)
batch-seo-action-needed = Handlungsbedarf
batch-seo-patterns-impact-title = Content-Auffälligkeiten → Business-Impact
batch-seo-impact-thin = { $insight } → schwächere Rankings, geringere Verweildauer
batch-seo-impact-duplicate = { $insight } → Keyword-Kannibalisierung, Split der Ranking-Signale
batch-seo-near-dup-title = Near-Duplicate-Content → Keyword-Kannibalisierung
batch-seo-risk-high = Hoch — konsolidieren
batch-seo-risk-medium = Mittel — differenzieren
batch-relevance-high = hoch
batch-relevance-medium = mittel
batch-relevance-low = niedrig
batch-schema-summary-all = Alle { $total } Seiten haben strukturierte Daten.
batch-schema-summary-some = { $without } von { $total } Seiten ohne strukturierte Daten.
batch-schema-callout-title = Strukturierte Daten (Schema.org)
batch-appendix-intro = Vollständige Auflistung aller erkannten Verstöße pro URL mit technischen Details für die Umsetzung.
batch-appendix-col-rule = Regel
batch-appendix-col-elements = Betr. Elemente

metric-violations-total = Verstöße{"\u00A0"}gesamt
metric-critical = Kritisch
metric-checked-nodes = Geprüfte{"\u00A0"}Knoten
metric-wcag-level = WCAG-Level
metric-warnings = Heuristische{"\u00A0"}Warnungen
metric-not-testable = Manuell{"\u00A0"}zu{"\u00A0"}prüfen
metric-overall-score = Gesamtscore{"\u00A0"}Website
date-format-str = %d.%m.%Y
trend-significantly-improved = Deutlich verbessert
trend-improved = Verbessert
trend-stable = Stabil
trend-slightly-regressed = Leicht zurückgegangen
trend-significantly-regressed = Deutlich verschlechtert
history-trend-significantly-improved = Die Barrierefreiheit hat sich gegenüber dem letzten Lauf vom { $previous_date } deutlich verbessert (+{ $delta_accessibility } Punkte, { $delta_issues_abs } Issues weniger).
history-trend-improved = Die Barrierefreiheit hat sich gegenüber dem letzten Lauf vom { $previous_date } verbessert.
history-trend-stable = Die Barrierefreiheit ist gegenüber dem letzten Lauf vom { $previous_date } unverändert stabil.
history-trend-significantly-regressed = Die Barrierefreiheit ist gegenüber dem letzten Lauf vom { $previous_date } deutlich zurückgegangen ({ $delta_accessibility } Punkte, +{ $delta_total_issues } Issues). Handlungsbedarf.
history-trend-slightly-regressed = Die Barrierefreiheit ist gegenüber dem letzten Lauf vom { $previous_date } leicht zurückgegangen.
history-summary = { $trend_interpretation } Die Historie umfasst { $timeline_entries } verwertbare Snapshots.
history-metric-acc-delta = Accessibility-Delta
history-metric-total-delta = Gesamt-Delta
history-metric-issue-delta = Issue-Delta
history-metric-crit-delta = Kritisch+Hoch-Delta
history-metric-prev-acc = Vorher Accessibility
history-metric-prev-total = Vorher Gesamt
ux-lever-cta = CTA-Texte klarer und spezifischer formulieren
ux-lever-trust = Vertrauenssignale (Kontakt, Impressum) ergänzen
ux-lever-hierarchy = Heading-Struktur bereinigen (H1 → H2 → H3)
ux-lever-default = UX-Qualität auf gutem Niveau halten
journey-intent-label = Seitenabsicht
journey-lever-default = Journey-Klarheit auf aktuellem Niveau halten
finding-structural-cause-component = Root Cause: 1 Komponentenproblem erzeugt { $count } Vorkommen. Wahrscheinlich ein gemeinsam genutztes Template oder eine Komponente — ein einmaliger Fix behebt alle Vorkommen gleichzeitig.
finding-structural-cause-shared = Dieses Problem tritt bei { $count } Elementen auf — möglicherweise eine gemeinsam genutzte Komponente oder ein Template.
module-accessibility = Barrierefreiheit
module-security = Sicherheit
module-performance = Performance
module-seo = SEO
module-mobile = Mobile
module-ux = UX
module-journey = Journey
linktext-generic-stopwords = mehr erfahren,weiterlesen,hier klicken,hier,mehr,weiter,lesen,anzeigen,öffnen,details,link,klicken
