label-certificate = Zertifikat
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
certificate-thresholds = Zertifikatsstufen (Gesamtscore): SEHR GUT ab 95, GUT ab 85, SOLIDE ab 75, AUSBAUFÄHIG ab 65, darunter UNGENÜGEND.
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
metric-certificate = Zertifikat
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
cover-card-certificate = Zertifikat
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
batch-cover-frame-certificate = Zertifikat
batch-cover-frame-modules = Aktive Module
batch-cover-frame-version = Tool-Version
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
perf-render-blocking-analysis = Render-Blocking Analyse

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
