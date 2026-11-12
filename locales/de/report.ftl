section-summary = Kurzfazit
section-methodology = Prüfumfang und Methodik
section-modules = Bewertung nach Modulen
section-findings-overview = Übersicht der Probleme
section-findings = Erkannte Probleme
section-findings-technical = Technische Detailanalyse
section-actions = Verbesserungsfelder
section-appendix = Anhang: Technische Details
callout-limitations-title = Grenzen automatisierter Tests
callout-note-title = Hinweis
callout-no-issues-title = Ausgezeichnete Barrierefreiheit
callout-no-issues-body = Keine automatisch erkennbaren Barrierefreiheitsprobleme gefunden.
label-priority = Einstufung
label-issue = Problem
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
effort-quick = Geringer Aufwand
effort-medium = Mittlerer Aufwand
effort-structural = Strukturell
effort-short-quick = gering
effort-short-medium = mittel
effort-short-structural = strukturell
severity-critical = Kritisch
severity-high = Hoch
severity-medium = Mittel
severity-low = Niedrig
severity-error = Fehler
severity-warning = Warnung
severity-info = Hinweis
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
panel-quick-actions = Naheliegende Verbesserungen
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
narrative-cover-kicker = Technischer Website-Check
narrative-status-title = Status der Website
narrative-metrics-title = Executive Snapshot
narrative-key-points-title = Kernaussagen
narrative-impact-title = Auswirkungen
narrative-quick-actions-title = Empfohlene Verbesserungen
narrative-spotlight-eyebrow = HAUPTPROBLEM
narrative-leverage-title = Wirkung einer Behebung
narrative-findings-title = Key Findings
narrative-action-plan-title = Verbesserungsfelder
narrative-action-plan-intro = Aus dem Audit abgeleitete Verbesserungen, gegliedert nach Wirkung, Aufwand und betroffener Nutzererfahrung.
narrative-action-plan-callout-title = Einordnung der Verbesserungen
narrative-action-plan-callout-body = Die folgenden Punkte beschreiben fachliche Verbesserungsfelder aus dem Audit. Sie sind keine Projektplanung und enthalten keine zeitliche Reihenfolge.
narrative-technical-title = Technische Umsetzung
narrative-technical-intro = Ab hier folgt die konkrete Umsetzung für Entwicklung, Design und Redaktion. Jedes Problem enthält: betroffene Elemente, direkte Umsetzung, Code-Beispiele.
narrative-next-steps-title = Ergänzende Prüfung
narrative-next-steps-intro = Hinweise, welche Prüfung die automatisierte Auswertung sinnvoll ergänzt.
narrative-next-steps-callout-title = Fachlicher Hinweis
narrative-next-steps-callout-body = Für eine vollständige Barrierefreiheits-Prüfung empfehlen wir ergänzend einen manuellen Audit mit assistiven Technologien (Screenreader, Tastaturnavigation).
narrative-findings-intro-strong = Die Seite ist technisch stark aufgestellt. Die folgenden Punkte sind letzte Optimierungshebel ohne strukturellen Druck.
narrative-findings-intro-solid = Tragfähige Basis — die folgenden Punkte sind gezielte Verbesserungshebel.
narrative-findings-intro-default = Die folgenden Probleme haben den größten Einfluss auf Nutzbarkeit und Risiko. Technische Details folgen im nächsten Abschnitt.

# Verdict (single audit)
verdict-tier-excellent = { $url } erreicht { $score }/100 im automatisierten Accessibility-Prüfumfang — kein strukturelles Problem. Verbleibende Befunde sind Feinschliff; dieser Wert ersetzt keine manuelle Prüfung.
verdict-tier-excellent-medium = { $url } erreicht { $score }/100 im automatisierten Accessibility-Prüfumfang. Die Basis ist sehr gut; die offenen Befunde mittlerer Schwere sind gezielte Verbesserungen. Dieser Wert ersetzt keine manuelle Prüfung.
verdict-tier-excellent-high = { $url } erreicht { $score }/100 im automatisierten Accessibility-Prüfumfang. Die Basis ist sehr gut, doch mindestens ein Befund hoher Schwere ist noch offen — diesen zuerst beheben. Dieser Wert ersetzt keine manuelle Prüfung.
verdict-tier-excellent-critical = { $url } erreicht { $score }/100 im automatisierten Accessibility-Prüfumfang. Trotz des hohen Werts ist mindestens ein kritischer Befund offen und sollte zuerst behoben werden. Dieser Wert ersetzt keine manuelle Prüfung.
verdict-tier-solid = { $url } erreicht { $score }/100 im automatisierten Accessibility-Prüfumfang. Die Grundlage ist gut — die verbleibenden Befunde sind gezielte Verbesserungshebel mit überschaubarem Aufwand.
verdict-tier-stable = { $url } erreicht { $score }/100 im automatisierten Accessibility-Prüfumfang. Die Grundlage ist stabil, aber nicht frei von Barrieren — mehrere Verbesserungshebel sollten eingeplant werden.
verdict-tier-deficient = { $url } erreicht { $score }/100 im automatisierten Accessibility-Prüfumfang. Es bestehen deutliche Barrieren — nicht nur Detailprobleme, sondern struktureller Nachholbedarf.
verdict-tier-critical = { $url } erreicht nur { $score }/100 im automatisierten Accessibility-Prüfumfang. Akuter Handlungsbedarf: Wesentliche Inhalte und Funktionen sind für einen Teil der Nutzer nicht zugänglich.
score-note-high-with-critical = Der Score berücksichtigt Gewichtung und Häufigkeit. Einzelne kritische Themen können trotz hoher Gesamtbewertung bestehen.
score-note-perfect-automated-scope = Im automatisch geprüften Umfang wurden keine bestätigten Accessibility-Verstöße gefunden. Kontextabhängige WCAG-Kriterien benötigen weiterhin manuelle Prüfung.

# Verdict (batch audit)
verdict-batch-excellent = Über { $total_urls } geprüfte URLs erreicht die Barrierefreiheit im Schnitt { $score }/100 — ein sehr gutes Ergebnis im automatisch geprüften Umfang.
verdict-batch-solid = Die { $total_urls } geprüften URLs erreichen bei der Barrierefreiheit im Schnitt { $score }/100. Die Grundlage ist gut, es bestehen aber wiederkehrende Probleme.
verdict-batch-stable = Die { $total_urls } geprüften URLs erreichen bei der Barrierefreiheit im Schnitt { $score }/100. Die Grundlage ist stabil, aber wiederkehrende Barrieren sollten eingeplant werden.
verdict-batch-deficient = Die { $total_urls } geprüften URLs erreichen bei der Barrierefreiheit im Schnitt { $score }/100. Es bestehen erhebliche wiederkehrende Probleme.
verdict-batch-critical = Die { $total_urls } geprüften URLs erreichen bei der Barrierefreiheit im Schnitt nur { $score }/100. Die wiederkehrenden Barrieren sollten vorrangig behoben werden.

# Site state (audit summary)
site-state-polished = Stark
site-state-needs-work = Tragfähige Basis
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
grade-satisfactory = Verbesserungswürdig
grade-deficient = Ausbaufähig
grade-critical = Kritisch

# Business / forward-looking consequence
business-consequence-clean = Keine Barrieren in der automatisierten Prüfung gefunden — eine solide Ausgangslage; die übrigen Kriterien brauchen noch eine manuelle Prüfung.
business-consequence-unverified = Die automatisierte Prüfung hat die Seite nicht vollständig erfasst — das Ausbleiben von Befunden belegt keine Barrierefreiheit.
business-consequence-severe = Weite Teile der Seite sind für bestimmte Nutzergruppen nicht oder kaum nutzbar.
business-consequence-seo-headings = Seite wird schlechter gefunden und ist für Teile der Nutzer strukturell nicht zugänglich.
business-consequence-screenreader = Einzelne Kernfunktionen sind für Screenreader-Nutzer blockiert oder fehleranfällig.
business-consequence-default = Nutzbarkeit ist gegeben — gezielte Verbesserungen erhöhen Qualität und Reichweite.
consequence-severe = Neue Inhalte und Funktionen erben die bestehenden Fehler — Korrekturaufwand wächst mit jeder Erweiterung.
consequence-many-weak-modules = Aufwand für spätere Korrekturen steigt deutlich — besonders bei Relaunch oder größerem Content-Ausbau.
consequence-stable = Kein akuter Handlungsdruck. Regelmäßige Checks sichern das Niveau nach Updates und Erweiterungen.
consequence-default = Ohne Korrektur bleibt die Seite hinter erreichbarem Standard — Verbesserungspotenzial wird nicht genutzt.

# Cover score row
cover-card-grade = Note
cover-card-accessibility = Accessibility
cover-card-issues = Issues
cover-card-critical-high-suffix = kritisch/hoch
cover-card-average = Technischer Wert Ø
batch-cover-overall-score-note = Mittel der technischen Werte je URL (0–100, höher ist besser).
cover-card-urls = URLs
cover-card-wcag-occurrences = { $count } WCAG-Vorkommen
batch-cover-eyebrow = Automatisierter Batch-Audit-Report
batch-cover-title = Barrierefreiheits-Prüfbericht
batch-cover-kicker = Seitenübergreifende Prüfung der ausgewählten URLs: Barrierefreiheit als Hauptbewertung, SEO, Performance, Sicherheit und weitere Module als Kontext
batch-cover-frame-title = Audit-Rahmen
batch-cover-frame-domain = Domain
batch-cover-frame-date = Prüfdatum
batch-cover-frame-urls = Geprüfte URLs
batch-cover-frame-modules = Aktive Module
batch-cover-frame-version = Tool-Version
batch-cover-frame-scope = Prüfumfang
batch-scope-sample = Stichprobe — { $audited } von { $total } URLs ({ $pct } %, { $source }): die ersten { $audited } in Quellreihenfolge, nicht nach Seitentyp gewählt; die Repräsentativität für die übrigen Seiten ist nicht gesichert
batch-scope-sample-filtered = Stichprobe — { $audited } von { $total } URLs ({ $pct } %, { $source }, nach Pfadfilter); die Repräsentativität für die übrigen Seiten ist nicht gesichert
batch-scope-sample-stratified = Stichprobe — { $audited } von { $total } URLs ({ $pct } %, { $source }): über Strukturgruppen (Sprache, Bereich, Dateityp) gezogen, nicht nach Seitentyp oder Prozesszustand; die Repräsentativität für die übrigen Seiten ist nicht gesichert
batch-cover-frame-failed = Nicht auditiert
batch-cover-frame-failed-value = { $failed } von { $selected } ausgewählten URLs fehlgeschlagen
batch-cover-frame-http = HTTP-Prüfung
batch-cover-frame-http-value = { $checked } Sitemap-URLs per HTTP-Status geprüft, { $audited } davon vollständig im Browser auditiert
batch-scope-full = Vollständig — alle { $total } URLs ({ $source })
batch-source-sitemap = Sitemap
batch-source-crawl = Crawl
batch-source-url_file = URL-Liste
batch-source-display_visual = Seiten mit 3D-/interaktiven Visualisierungen
batch-cover-frame-display = Darstellungsmodus
batch-display-offered = { $count } von { $total } Seiten bieten Darstellungsmodi an
batch-cover-frame-score-basis = Bewertungsbasis
batch-score-coverage = { $scored } von { $attempted } URLs — { $failed } nicht auditierbar und in keinem Score enthalten
batch-unaudited-title = Scores decken { $scored } von { $attempted } URLs ab
batch-unaudited-body =
    { $failed ->
        [one] Eine URL konnte nicht auditiert werden. Sie fließt
       *[other] { $failed } URLs konnten nicht auditiert werden. Sie fließen
    } in keinen Score und keine Zählung dieses Berichts ein; der JSON-Bericht nennt den Fehler je URL.
batch-unaudited-list-title = Nicht auditiert
batch-unaudited-more = … und { $count } weitere
panel-modules-overview = Modulübersicht
section-tech-detail-metrics = Technische Detailmetriken
column-action = Maßnahme
column-priority = Einstufung
column-timeframe = Aufwand
column-component = Komponente
column-area = Bereich
section-quick-wins = Naheliegende Verbesserungen
section-medium-actions = Maßnahmen mit mittlerem Aufwand
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
batch-col-schema-type = Schema-Typ
batch-col-profile = Profil
batch-col-severity = Schweregrad
batch-col-description = Beschreibung
batch-col-metric = Metrik
batch-col-budget = Budget
batch-col-links-to = Eingehende Links
batch-col-links-from = Ausgehende Links
batch-col-words = Wörter
batch-action-plan-title = Maßnahmenplan
batch-section-broken-links-internal = Interne Broken Links
batch-section-broken-links-external = Externe Broken Links
batch-section-external-links = Externe Links
batch-section-redirect-chains = Redirect-Ketten
batch-redirect-chains-title = Redirect-Ketten (> 1 Hop)
batch-col-target = Ziel
batch-url-ranking-title = URL-Ranking
batch-url-ranking-intro = Sortiert nach Barrierefreiheit (Spalte A11y), niedrigster Wert zuerst; bei Gleichstand mehr kritische/hohe Befunde zuerst. „Score“ ist der kombinierte technische Wert. Ein niedriger Wert zeigt, wo am meisten verloren geht — einzelne kritische Funktionsbefunde können auch Seiten mit gutem Wert betreffen.
batch-render-blocking-section = Render-Blocking & Assets
batch-render-blocking-kv-title = Render-Blocking-Übersicht (domainweit)
batch-render-blocking-intro = Render-blockierende Ressourcen und Third-Party-Traffic, aggregiert über alle geprüften Seiten.
batch-col-group-a11y = Ø Barrierefreiheit
batch-col-group-common-finding = Häufigster Befund in der Gruppe
batch-group-single-page = Einzelseite, nicht vergleichbar
batch-group-common-finding = { $title } ({ $pages } von { $total } Seiten)
batch-group-no-common-finding = Kein Befund auf mehreren Seiten der Gruppe
batch-page-type-groups-note = Der Seitentyp ist eine Einordnung aus Seitenstruktur und Inhalt, kein belegtes Template. Mittelwerte und häufigster Befund beziehen sich auf die Seiten der jeweiligen Gruppe; der häufigste Befund zählt nur die drei wichtigsten Befunde je Seite.
batch-shared-blocking-title = Dieselben render-blockierenden Dateien auf mehreren Seiten
batch-col-size = Größe
batch-minification-inconsistency-section = Build-Konsistenz
batch-minification-inconsistency-title = Uneinheitliche Minifizierung → Build-Konfigurationslücke je Template/Route
batch-minification-inconsistency-intro = Assets, die auf manchen Seiten minifiziert und auf anderen unminifiziert ausgeliefert werden — ein Hinweis auf eine Build-Konfiguration, die nicht für alle Templates/Routen greift.
batch-col-minify-asset = Asset
batch-col-minify-kind = Typ
batch-col-minify-minified-on = Minifiziert auf
batch-col-minify-unminified-on = Unminifiziert auf
batch-minify-kind-script = JavaScript
batch-minify-kind-css = CSS
batch-redirect-chain-issues-section = Weiterleitungsketten
batch-redirect-chain-issues-title = Lange Weiterleitungsketten & Loops → verschwendetes Crawl-Budget
batch-redirect-chain-issues-intro = Geprüfte URLs, die vor der finalen Seite über 3 oder mehr HTTP-Weiterleitungen laufen oder in einer zyklischen Weiterleitung enden — jeder Hop kostet zusätzliche Ladezeit und Crawl-Budget.
batch-col-redirect-url = URL
batch-col-redirect-hops = Hops
batch-col-redirect-type = Typ
batch-col-redirect-chain = Kette
batch-redirect-chain-type-long = Lange Kette
batch-redirect-chain-type-loop = Loop
batch-section-tech-url-matrix = Technische URL-Matrix
batch-section-tech-url-matrix-intro = Verdichtete Übersicht aller geprüften URLs mit Fokus auf technische Priorisierung. Jede Zeile zeigt Score, Problemintensität und den größten Hebel für die weitere Optimierung.
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
section-html-conform = HTML-Konformität
section-mobile-usability = Mobile Nutzbarkeit
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
diagnosis-clusters-note = Ein Cluster entsteht ab zwei Befunden in derselben Kategorie; Kategorien mit nur einem Befund stehen in der Kategorie-Übersicht, aber nicht hier.
diagnosis-table-clusters = Problemcluster
diagnosis-col-category = Kategorie
diagnosis-col-findings = Befunde
diagnosis-col-worst-severity = Schwerster Schweregrad
diagnosis-col-occurrences = Vorkommen
diagnosis-col-max-severity = Max. Schweregrad

# Batch module overview
batch-panel-module-averages = Modulübersicht (Ø über alle URLs)

# Heuristic indicator (shared by UX, Journey, AI Visibility)
label-heuristic-indicator = Heuristische Schätzung auf Basis struktureller Signale, nicht extern validiert

# Performance section
perf-score-card = Performance Score
perf-technical-indicators = Begleitende Messwerte
perf-lab-data-note = Lab-Daten
perf-lab-data-body = LCP, CLS und TBT werden lokal in Headless-Chrome unter einer realen Slow-4G-Drosselung gemessen (CDP: 1,6 Mbps, 150 ms Latenz, vierfache CPU-Drosselung). Diese Werte können von Google PageSpeed / Lighthouse abweichen: PageSpeed misst ungedrosselt und simuliert die Drosselung rechnerisch (Lantern-Modell), während wir real drosseln — die reale Messung fällt häufig optimistischer aus, besonders beim LCP. Maßgeblich für die tatsächliche Nutzererfahrung sind allein Felddaten (CrUX/Real-User-Monitoring); die Tabelle „Performance unter gedrosselten Bedingungen" weiter unten zeigt die Bandbreite über mehrere Netzprofile. Von diesen Kennzahlen sind LCP und CLS Core Web Vitals; TBT ist eine reine Lab-Metrik und dient hier als Näherung für Interaktivitätsprobleme, weil das dritte Core Web Vital (INP) ohne Nutzereingaben nicht messbar ist. Mit „Lab-Schätzung" markierte Kennzahlen (INP, TTI, Speed Index) sind abgeleitete Näherungen, keine direkten Messwerte.
perf-render-blocking-analysis = Render-Blocking Analyse
perf-measurement-warnings-title = Messtechnische Einschränkungen
perf-warning-lcp-missing = LCP nicht erfasst: Der PerformanceObserver hat keinen LCP-Eintrag geliefert (typisch bei starker Netzwerkdrosselung) — LCP-Score fehlt in der Bewertung.
perf-warning-si-fallback = Speed Index entspricht LCP: Formelschätzung ohne unabhängigen Messwert (0,35 × FCP + 0,65 × LCP).
perf-warning-tti-fallback = TTI entspricht LCP: Formelschätzung ohne direkten Interaktivitätswert.
perf-warning-inp-missing = INP nicht messbar: Headless-Browser erzeugen keine Nutzereingaben — Interaktionslatenz konnte nicht erfasst werden.
perf-warning-lh-mobile-gap = Desktop-Score: { $desktop } · LhMobile-Score: { $mobile } — Abweichung { $gap } Punkte. Der Primärscore basiert auf der ungedrosselten Messung.
perf-capped-size = Der Performance-Score wurde aufgrund einer sehr großen Datenmenge ({ $size } MB) auf { $cap } gedeckelt.
perf-capped-js = Der Performance-Score wurde aufgrund einer kritischen JavaScript-Größe ({ $size } MB) auf { $cap } gedeckelt.
perf-capped-requests = Der Performance-Score wurde aufgrund einer hohen Anzahl an Anfragen ({ $count }) auf { $cap } gedeckelt.
perf-capped-dom = Der Performance-Score wurde aufgrund einer hohen DOM-Komplexität ({ $nodes } Knoten) auf { $cap } gedeckelt.
perf-worst-case-xref = ({ $profile }-Profil, siehe Tabelle „{ pdf-perf-throttled-title }" unten)

# SEO section
seo-score-card = Technisches SEO
seo-score-card-description = Misst technische Signale (Meta, Struktur, Schema, hreflang). Inhaltliche Tiefe wird separat bewertet.
seo-maturity = Reifegrad

# UX section
ux-score-card = UX Score (Indikator)
ux-dimensions = UX-Dimensionen

# Journey section
journey-score-card = Journey Score (Indikator)
journey-page-type-dimensions = Journey-Einordnung & Dimensionen

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
finding-key-quick-win = Schnell umsetzbar — geringer Aufwand, hohe Wirkung
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
finding-wrong = ✗ Falsch
finding-right = ✓ Richtig
finding-location = Fundstelle
finding-line-column = Zeile { $line }, Spalte { $column }
finding-note = Hinweis
finding-representative-occurrences = Repräsentative Vorkommen
finding-occurrence = Fundstelle
finding-suggested-fix = Vorgeschlagene Code-Korrektur
finding-pattern = Muster
finding-frequent-patterns = Häufige Muster
finding-frequent-patterns-note = Diese Tabelle gruppiert Vorkommen nach struktureller Ähnlichkeit (gleiches Element- und Auswahlmuster) über die gesamte Seite hinweg, nicht nach exakt identischem Wert — die Zahl kann daher von der Vorkommenszahl eines einzelnen Befunds abweichen.
finding-reference = Referenz

# SEO / Tracking section
seo-tracking-services = Tracking und externe Dienste
seo-kv-title = Technische SEO-Signale
seo-serp-readiness = SERP-Bereitschaft
seo-serp-signals = SERP-Signale
seo-page-health-issues = Gefundene Probleme
seo-page-health-url-analysis = URL-Analyse
seo-page-html-validation = HTML-Validierung
seo-technical-issues-title = Technische SEO-Probleme

# Security section
security-score-card = Security Score

# Mobile section
mobile-score-card = Mobile Score
mobile-configured = Konfiguriert
mobile-touch-targets = Touch-Targets
mobile-viewport-config = Viewport-Konfiguration
mobile-font-analysis = Schriftanalyse
mobile-content-sizing = Inhaltsanpassung
mobile-cat-viewport = Viewport
mobile-cat-touch_targets = Touch-Targets
mobile-cat-fonts = Schriftgrößen
mobile-cat-content = Inhaltsanpassung

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
impact-base-accessibility-exclude = Kann Nutzende ausschließen und im Anwendungsbereich von BFSG/EAA rechtlich relevant sein.
impact-base-accessibility-exclude-plain = Kann Nutzende ausschließen.
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
effect-user-default-structural = Nachhaltig inklusivere Nutzererfahrung für alle

# Conversion Effects

# Narrative Arc Formatters
narrative-diagnose-multiple = { $count }× festgestellt: { $desc }
narrative-cause-suffix-high = {" "}Dies ist eine strukturelle Lücke mit deutlicher Auswirkung.
narrative-cause-accessibility = Dies resultiert typischerweise aus fehlendem oder unvollständigem semantischen Markup.
narrative-cause-seo = Dies resultiert typischerweise aus fehlenden oder unvollständigen Meta-Informationen.
narrative-cause-performance = Dies resultiert typischerweise aus nicht optimierten Assets oder render-blockierenden Ressourcen.
narrative-cause-security = Dies resultiert typischerweise aus fehlenden oder falsch konfigurierten HTTP-Sicherheits-Headern.
narrative-cause-mobile = Dies resultiert typischerweise aus einem Layout- oder Größenproblem, das nicht für kleine Bildschirme angepasst wurde.
narrative-cause-default = Dies resultiert typischerweise aus einer Konfigurations- oder Implementierungslücke.
narrative-impact-critical = Dies kann eine zentrale Nutzerinteraktion für betroffene Nutzergruppen erheblich erschweren oder verhindern; ob ein Ablauf vollständig blockiert ist, zeigt erst dessen Prüfung.
narrative-impact-high = Dies beeinträchtigt die Nutzbarkeit für betroffene Nutzergruppen erheblich.
narrative-impact-medium = Dies verursacht einen spürbaren Reibungspunkt für bestimmte Nutzer.
narrative-impact-low = Dies ist ein kleineres Problem, das eine Teilgruppe der Nutzer betreffen kann.
narrative-effort-quick = Schnell umsetzbar — geringer Aufwand
narrative-effort-medium = Mittlerer Aufwand
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
context-accessibility-summary = { $total ->
        [one] { $total } erkannte Problemgruppe
       *[other] { $total } erkannte Problemgruppen
    }, davon { $high } mit hoher Auswirkung.
card-accessibility-none = Keine High-Priority-Funde
card-accessibility-summary = { $high ->
        [one] { $high } Problemgruppe
       *[other] { $high } Problemgruppen
    } mit hoher Auswirkung
lever-performance-dom = Größter Hebel: DOM-Größe reduzieren ({ $dom_nodes } Knoten)
lever-performance-load = Größter Hebel: Ladezeit senken ({ $load } ms)
lever-performance-default = Größter Hebel: Render-Pfad und Asset-Größe weiter optimieren
context-performance-good-vitals = { $fcp } · Nutzererlebnis: schnell — technische Komplexität zieht den Performance-Wert herunter.
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
recommendation-security-csp-unsafe-inline = unsafe-inline aus der CSP entfernen und freigegebene Inline-Skripte bzw. -Styles über Nonces oder Hashes zulassen.
recommendation-security-csp-unsafe-eval = unsafe-eval aus der CSP entfernen und auf dynamisch ausgewertete Skripte verzichten.
recommendation-security-csp-wildcard = Platzhalter-Quellen in der CSP durch die konkret benötigten Origins ersetzen.
recommendation-security-csp-directive = Die CSP um die fehlenden Direktiven ergänzen: object-src 'none', base-uri 'self' und frame-ancestors passend zur Einbettung.
recommendation-security-cors-credentials = Access-Control-Allow-Origin: * nicht mit credentialed CORS kombinieren, sondern vertrauenswürdige Origins einzeln freigeben.
recommendation-security-permissions-permissive = Permissions-Policy enger fassen und nur die Browser-Funktionen freigeben, die die Seite wirklich nutzt.
recommendation-security-sourcemap = Source Maps nicht öffentlich ausliefern oder den Zugriff darauf einschränken.
recommendation-security-hsts-preload = Die HSTS-Direktive so ergänzen, dass sie die Preload-Anforderungen erfüllt (max-age >= 1 Jahr, includeSubDomains, preload).
recommendation-security-default = Die grundlegenden Security-Header sind sauber gesetzt. Nächster Schritt: Richtlinien regelmäßig prüfen und an neue Skript- oder Integrationsquellen anpassen.
lever-mobile-small = Größter Hebel: Touch-Targets vergrößern ({ $small_targets } zu klein)
lever-mobile-crowded = Größter Hebel: Abstände mobiler Bedienelemente erhöhen ({ $crowded_targets })
lever-mobile-default = Größter Hebel: mobile Lesbarkeit und Touch-Flows weiter optimieren
context-mobile-proper = Viewport korrekt gesetzt, { $small_targets } zu kleine Touch-Targets, { $crowded_targets } zu enge Abstände.
context-mobile-improper = Viewport nicht sauber konfiguriert, { $small_targets } zu kleine Touch-Targets, { $crowded_targets } zu enge Abstände.
card-mobile-small = { $small_targets } zu kleine Touch-Targets
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
pdf-perf-sub-vitals = Lade-Erfahrung & Vitals
pdf-perf-sub-resources = Ressourcen & Datenmenge
pdf-perf-sub-bottlenecks = Lade-Engpässe & Rendering
pdf-perf-throttled-title = Performance unter gedrosselten Bedingungen
pdf-perf-throttled-profile = Profil
pdf-perf-throttled-incomplete = Unvollständig: nicht alle Kernmetriken gemessen
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
pdf-perf-tp-isolated-title = Isolierter Skript-Einfluss
pdf-perf-tp-isolated-baseline = TBT (Baseline)
pdf-perf-tp-isolated-without = TBT ohne Skript
pdf-perf-tp-isolated-impact = Geschätzter Einfluss
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
pdf-perf-resource-focus-title = Größter direkt nutzbarer Hebel
pdf-perf-resource-focus = Durch die Minifizierung von { $count } { $count ->
        [one] Datei
       *[other] Dateien
    } lassen sich voraussichtlich { $savings } KB aus dem ausgelieferten Code entfernen. Das reduziert Datenmenge und Parsing-Aufwand, ohne Funktionen zu streichen. Danach unter denselben Bedingungen erneut messen.
pdf-perf-resource-note = JS-Heap ist ein Beobachtungswert ohne universellen Grenzwert; relevant sind Wachstum und Spitzen bei Interaktionen. CO2e pro View ist eine modellierte Schätzung und wird durch die angegebene Effizienzklasse eingeordnet.
pdf-perf-resource-note-heap = JS-Heap ist ein Beobachtungswert ohne universellen Grenzwert; relevant sind Wachstum und Spitzen bei Interaktionen.
pdf-perf-resource-note-carbon = CO2e pro View ist eine modellierte Schätzung und wird durch die angegebene Effizienzklasse eingeordnet.
pdf-perf-coverage-clean-title = Code-Nutzung unauffällig
pdf-perf-coverage-clean = Im Messlauf wurde kein nennenswerter ungenutzter JavaScript-Code erfasst. Code-Reduktion ist damit aktuell kein Haupthebel. Die Abdeckung gilt nur für den geprüften Seitenzustand.
pdf-perf-coverage-focus-title = Ungenutzten Code reduzieren
pdf-perf-coverage-focus = Im geprüften Seitenzustand blieben { $unused } KB JavaScript ungenutzt. Nicht benötigte Module später laden oder aus dem Bundle entfernen und anschließend erneut messen.
pdf-perf-bottleneck-critical-title = Einordnung: Seite wird spät nutzbar
pdf-perf-bottleneck-critical = Kritische Messwerte: { $metrics }. Sie zeigen zu viel Browser- und Ladearbeit vor der Nutzbarkeit. { $action }
pdf-perf-bottleneck-dom-title = Einordnung: Hohe Browserlast durch DOM-Größe
pdf-perf-bottleneck-dom = Kritischer Messwert: { $metrics }. Das belastet Rendering und Speicher des Browsers; ein gemessener Zeitverzug ist damit nicht belegt. { $action }
pdf-perf-bottleneck-warning-title = Einordnung: Ladepfad prüfen
pdf-perf-bottleneck-warning = Auffällige Messwerte: { $metrics }. { $action }
pdf-perf-bottleneck-dom-action = Zuerst DOM-Struktur verschlanken und blockierende Ladepfade reduzieren; danach unter denselben Bedingungen erneut messen.
pdf-perf-bottleneck-load-action = Zuerst große oder blockierende Ressourcen sowie Drittanbieter-Abhängigkeiten prüfen; danach unter denselben Bedingungen erneut messen.
pdf-perf-priority-actions = Priorisierte Maßnahmen
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
pdf-seo-more-signals =
    { $count } { $count ->
        [one] weiteres Signal
       *[other] weitere Signale
    } im detaillierten Anhang.
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
pdf-serp-rich-result-types-title = Erkannte Rich-Result-Typen
pdf-serp-rich-results-text = Diese Schema-Typen können zu Suchdarstellungen mit Zusatzinformationen gehören: { $types }. Die Typ-Erkennung allein bestätigt keine Eignung.
pdf-section-clean = Im geprüften Umfang keine wesentlichen Auffälligkeiten festgestellt.
pdf-ph-issue = Problem
pdf-ph-html-validator-title = HTML-Validierung
pdf-ph-www-title = www-Konsolidierung
pdf-ph-check = Prüfung
pdf-ph-count = Anzahl
pdf-robots-error = robots.txt konnte nicht geladen werden: { $err }
pdf-robots-no-access = Kein Zugriff
pdf-robots-block-all-body = Alle Crawler vollständig gesperrt (User-agent: * / Disallow: /). Auf Staging-Domains ist das korrekt — auf der Produktiv-Domain würde dies das vollständige Crawling durch Suchmaschinen verhindern.
pdf-robots-block-all-title = Alle Crawler gesperrt
pdf-robots-limit-ai-body = KI-Suchbots (z. B. PerplexityBot, Amazonbot) sind blockiert. Das ist eine bewusste Entscheidung — Inhalte erscheinen nicht in KI-generierten Antworten. Das Sperren von KI-Trainingsbots (GPTBot etc.) ist dagegen übliche Praxis und kein Problem.
pdf-robots-limit-ai-title = Eingeschränkte KI-Sichtbarkeit
pdf-robots-training-blocked-body = Richtlinie: { $policy } — KI-Trainingsbots ({ $bots }) sind gesperrt, KI-Suchbots haben Zugang. Das entspricht der empfohlenen Standardkonfiguration.
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
pdf-seo-profile-content-type = Schema.org-Inhaltstyp
pdf-seo-profile-language = Sprache
pdf-seo-profile-schema-types = Schema-Typen
pdf-seo-profile-schema-type-col = Schema-Typ
pdf-seo-profile-completeness = Feldabdeckung
pdf-seo-profile-structured-data-title = Strukturierte Daten ({ $label })
pdf-seo-schema-status-title = JSON-LD-Status
pdf-seo-schema-status-ok =
    { $nodes ->
        [one] Ein auswertbarer Schema-Knoten erkannt.
       *[other] { $nodes } auswertbare Schema-Knoten erkannt.
    } JSON-LD ist syntaktisch und strukturell unauffällig; die inhaltliche Eignung wird getrennt bewertet.
pdf-seo-schema-status-issues =
    { $nodes ->
        [one] Ein auswertbarer Schema-Knoten erkannt.
       *[other] { $nodes } auswertbare Schema-Knoten erkannt.
    } { $issues ->
        [one] Ein Syntax-/Strukturproblem liegt vor.
       *[other] { $issues } Syntax-/Strukturprobleme liegen vor.
    } Fehlerhafte Blöcke oder Knoten sind für Suchsysteme nicht zuverlässig interpretierbar.
pdf-seo-schema-status-non-jsonld = Strukturierte Daten als Microdata oder RDFa erkannt. Dieses Format wurde erkannt, aber nicht inhaltlich validiert.
pdf-seo-schema-status-none = Keine strukturierten Daten erkannt. Ob ein Schema sinnvoll fehlt, hängt vom tatsächlichen Seitentyp ab.
pdf-seo-schema-status-check = Prüfung
pdf-seo-schema-status-detail = Auswirkung
pdf-seo-schema-issues-title = Festgestellte Strukturprobleme
pdf-seo-schema-fit-title = Seitentyp und Schema-Eignung
pdf-seo-schema-fit-page-type = Seitentyp im Schema-Abgleich
pdf-seo-schema-fit-confidence = Erkennungssicherheit
pdf-seo-schema-fit-expected = Plausible Haupt-Schemas
pdf-seo-schema-fit-detected = Erkannte Haupt-Schemas
pdf-seo-schema-fit-none = Keine
pdf-seo-schema-rule-title = Funktionsbezogene Schema-Anforderungen
pdf-seo-schema-rule-feature = Schema und Suchfunktion
pdf-seo-schema-rule-status = Anforderungsstatus
pdf-seo-schema-rule-detail = Details
pdf-seo-schema-rule-limited = eingeschränkte Sichtbarkeit
pdf-seo-schema-review-title = Kontext- und Inhaltsprüfung
pdf-seo-schema-review-feature-col = Schema / Merkmal
pdf-seo-schema-review-detail-col = Manuell zu prüfen
pdf-seo-schema-parity-title = Abgleich mit sichtbaren Inhalten
pdf-seo-schema-parity-property-col = Schema / Feld
pdf-seo-schema-parity-status-col = Status
pdf-seo-schema-parity-evidence-col = Kurzbeleg
pdf-seo-schema-rule-context-dependent = nur bei passendem Seitenkontext
pdf-seo-schema-rule-missing-required = Fehlende Pflichtangaben: { $fields }
pdf-seo-schema-rule-missing-recommended = Nicht vorhandene empfohlene Angaben: { $fields }
pdf-seo-schema-rule-not-evaluated = Die strengere Prüfung wird erst bei eindeutig passendem Seitenkontext angewendet.
pdf-seo-schema-rule-complete = Im automatisiert prüfbaren Umfang vollständig.
pdf-seo-profile-rating = Bewertung
pdf-seo-profile-signals-met = Signale erfüllt
pdf-seo-profile-classification = Einstufung
pdf-seo-profile-strength-title = SEO-Signalstärke ({ $passed } von { $total } Signalen erfüllt)
pdf-seo-profile-maturity-title = SEO-Reifegrad
pdf-seo-profile-techniques = Techniken
pdf-seo-profile-techniques-value = { $used } von { $total } erkannt
pdf-sec-https-yes = Ja
pdf-sec-https-unclear = Unklar
pdf-sec-protection-title = Schutzinfrastruktur
pdf-sec-header-tier-column = Kategorie
pdf-sec-header-tier-note = Kategorien: Header mit „Basisschutz" gelten für praktisch jede Seite; architektur- und kontextabhängige Header sind gute Praxis, aber nur für bestimmte Konstellationen relevant — das jeweilige Finding nennt das konkrete Szenario. Header mit „Remote nicht bewertbar" lassen sich per automatisiertem Check gar nicht sinnvoll als vorhanden/fehlend beurteilen.
pdf-sec-score-title = Wie der Sicherheitswert entsteht
pdf-sec-score-component = Bestandteil
pdf-sec-score-points = Punkte
pdf-html-conform-not-checked = HTML-Konformität konnte für diese Seite nicht geprüft werden.
pdf-html-conform-dedup-note = Der Score bewertet die Ursachen, nicht die Vorkommen: ein fehlerhaftes Template-Element erzeugt bei jeder Ausgabe ein Vorkommen, bleibt aber eine Korrektur. Wiederholungen erhöhen den Abzug gedämpft, statt ihn zu vervielfachen.
pdf-html-conform-defects = Ursachen
pdf-html-conform-errors = Fehler
pdf-html-conform-warnings = Warnungen
pdf-html-conform-info = Hinweise
pdf-html-conform-occurrences = Vorkommen
pdf-html-conform-rule = Regel
pdf-html-conform-severity = Schweregrad
pdf-html-conform-message = Beschreibung
pdf-html-conform-location = Position
section-commerce = Commerce
pdf-commerce-scope-note = Prüft Produkt-Strukturdaten und die Verlinkung von Pflichtseiten auf dieser einen Seite — keine Prüfung von Checkout, Zahlungsabwicklung oder Warenkorb (das Tool hat keinen seitenübergreifenden Sitzungszustand).
pdf-commerce-overview-title = Übersicht
pdf-commerce-page-kind = Seitentyp
pdf-commerce-completeness = Vollständigkeit
pdf-commerce-product-title = Produkt-Strukturdaten
pdf-commerce-price = Preis
pdf-commerce-availability = Verfügbarkeit
pdf-commerce-shipping = Versandangaben
pdf-commerce-returns = Rückgaberichtlinie
pdf-commerce-rating = Bewertung
pdf-commerce-not-exposed = Nicht ausgewiesen
pdf-commerce-trust-pages-title = Pflichtseiten-Verlinkung
pdf-commerce-linked = Verlinkt
pdf-commerce-not-linked = Nicht verlinkt
pdf-ux-overview-title = UX — Überblick
pdf-ux-more-issues =
    { $count } { $count ->
        [one] weiterer Befund
       *[other] weitere Befunde
    } im detaillierten Anhang.
pdf-journey-overview-title = Journey — Überblick
pdf-journey-detected-page-type = Journey-Einordnung
pdf-journey-more-issues =
    { $count } { $count ->
        [one] weiterer Befund
       *[other] weitere Befunde
    } im detaillierten Anhang.
pdf-dm-yes = Ja
pdf-dm-no = Nein
pdf-sq-section-title = Technische Aufbereitung für maschinelle Verarbeitung
pdf-sq-overview-title = Was dieser Indikator misst
pdf-sq-success = Alle bewerteten Signale der technischen Aufbereitung sind erfüllt.
pdf-sq-unweighted = Ungewichteter Indikator — fließt nicht in den Gesamtwert ein und bewertet nicht, ob Inhalte zutreffend oder vertrauenswürdig sind.
pdf-sq-not-checked = Nicht geprüft: Keiner der seiteninternen Vergleiche war anwendbar (Sprache lang/og:locale, Titel, strukturierte Daten gegen sichtbaren Inhalt).
pdf-sq-cross-page-note = Seitenübergreifende Konsistenz (etwa Firmenname oder Kontaktdaten auf verschiedenen Seiten) ist auf einer einzelnen Seite nicht prüfbar; dafür den Sitemap-Report heranziehen.
pdf-sq-indicators-intro = Beobachtete Werte ohne Bewertung. Sie beschreiben die Struktur der Seite, nicht, ob der Inhalt substanziell ist.
pdf-ts-section-title = Tech-Stack
pdf-ts-technology = Technologie
pdf-ts-confidence = Konfidenz
pdf-ts-detected-title = Erkannte Technologien
pdf-ts-confidence-high = Hoch
pdf-ts-confidence-medium = Mittel
pdf-ts-confidence-low = Niedrig
pdf-ai-indicator-note = Experimentelle Heuristiken — keine standardisierte Metrik. Die Indikatoren sind heuristische Schätzwerte auf Basis messbarer Signale, kein direktes Ranking-Signal. Sie zeigen Optimierungspotenzial, beweisen aber kein konkretes LLM-Verhalten.
pdf-ai-section-title = AI-Sichtbarkeit (Indikator)
pdf-ai-overview-title = KI-Sichtbarkeit — Überblick
pdf-ai-score-title = AI-Sichtbarkeit (Indikator)
pdf-ai-score-desc-format = Grade: { $grade } — { $quality } · Heuristische Schätzung, kein Messwert
pdf-ai-success = Alle KI-Sichtbarkeits-Signale sind in Ordnung. Kein Optimierungsbedarf.
pdf-ai-readability = KI-Lesbarkeit
pdf-ai-citability = Kompakte Abschnittsstruktur (ungeprüft)
pdf-ai-tech-readability = Technische KI-Lesbarkeit
pdf-ai-policy = AI-Policy
pdf-ai-more-signals =
    { $count } { $count ->
        [one] weiteres Signal
       *[other] weitere Signale
    } im detaillierten Anhang.
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
pdf-cv-signals-analyzed = { $signals ->
        [one] { $signals } Signal analysiert
       *[other] { $signals } Signale analysiert
    }, { $problems ->
        [one] { $problems } Hinweis
       *[other] { $problems } Hinweise
    } auf Optimierungsbedarf.
pdf-cv-manual-review-title = Manuell prüfen
pdf-cv-success = Alle Content-Visibility-Signale sind in Ordnung. Kein Optimierungsbedarf.
pdf-bp-score-title = Best Practices Score
pdf-bp-success = Keine Konsolfehler und keine anfälligen Bibliotheken erkannt.
pdf-bp-console-errors-title = Konsolenfehler
pdf-bp-message-col = Meldung
pdf-bp-vuln-libs-title = Anfällige Bibliotheken
pdf-bp-lib-col = Bibliothek
pdf-bp-severity-col = Schwere
pdf-bp-fix-col = Lösung
pdf-bp-libs-up-to-date = Erkannte Bibliotheken scheinen aktuell zu sein — keine bekannten Schwachstellen.
pdf-bp-vuln-libs-unreviewed = (Ungeprüft: Der betroffene Versionsbereich wurde noch nicht gegen die Quelle geprüft.)
pdf-bp-duplicate-libs-title = Doppelte Bibliotheksversionen
pdf-bp-duplicate-libs-item = { $name }: { $versions }
pdf-bp-scope = Prüfumfang: Konsolenmeldungen vom Laden der Seite bis zum Ende der Analyse (kein Klicken, Scrollen oder Absenden). Versionen geprüft für { $libs }; andere Skripte sind nicht abgedeckt.
pdf-bp-not-measured = Nicht gemessen: Weder Konsolenmeldungen noch Bibliotheksversionen konnten erfasst werden. Es gibt deshalb keinen Wert.
pdf-bp-console-not-collected = Konsolenmeldungen konnten nicht erfasst werden; der Wert beruht nur auf der Bibliotheksprüfung.
pdf-bp-libs-not-checked = Bibliotheksversionen konnten nicht geprüft werden; der Wert beruht nur auf den Konsolenmeldungen.
pdf-perf-cov-js-val = { $pct }% ({ $unused } KB ungenutzt)
pdf-perf-cov-css-val = { $pct }% ({ $used } von { $total } Stilregeln genutzt, vom Laden bis zum Ende der Analyse, ohne Klicken oder Scrollen)
pdf-perf-cov-css-partial = — nur lesbare Stylesheets
pdf-perf-dup-title = { $count } { $count ->
        [one] doppelte Asset-Gruppe
       *[other] doppelte Asset-Gruppen
    } erkannt (~{ $wasted } KB verschwendet)
pdf-cv-area-organic-visibility = Organische Sichtbarkeit
pdf-cv-area-local-business = Local Business & Vertrauensdaten
pdf-cv-area-eeat = E-E-A-T-Indizien
pdf-cv-area-content-depth = Inhaltstiefe & Lokalisierung
pdf-cv-area-topical-authority = Topical Authority (Heuristik)

batch-top-issues-intro = Die folgenden Problemgruppen treten über mehrere URLs hinweg auf. Durch Behebung dieser Probleme wird die größte Verbesserung erzielt, da sie viele Seiten gleichzeitig betreffen.
batch-col-affected-urls = Betr. URLs
batch-meta-global = Alle geprüften Seiten
batch-meta-some-pages = { $count } von { $total } Seiten
batch-meta-single-page = Eine Seite — seitenspezifisch
batch-meta-occurrences = { $count } Vorkommen
batch-meta-affected-urls = { $count ->
    [one] betroffene URL
   *[other] betroffene URLs
}
batch-meta-effort = Aufwand je Stelle (Schätzung)
batch-meta-scope = Reichweite
batch-meta-impact-user = Auswirkung (Nutzer)
batch-meta-impact-business = Auswirkung (Business)
batch-meta-fix = Empfehlung
batch-meta-classification = Einordnung
batch-template-section-title = Template-Ursachen
batch-template-section-intro = Die folgenden Befunde wiederholen sich an derselben Stelle auf mehreren Seiten. Das deutet auf eine gemeinsame Komponente oder ein Template hin; dass eine Korrektur dort alle Vorkommen behebt, ist eine Annahme, bis der Quellcode sie bestätigt.
batch-template-confirmed-headline = Identisches Markup an ‚{ $selector }‘ auf { $n } von { $total } geprüften Seiten. Stammt es aus einer gemeinsamen Komponente, wirkt eine Korrektur dort voraussichtlich auf allen { $n } Seiten — ob das so ist, zeigt der Quellcode.
batch-template-likely-headline = Wiederkehrend an ‚{ $selector }‘ auf { $n } von { $total } geprüften Seiten; gleiches Markup ist nicht belegt — eine gemeinsame Ursache ist möglich, nicht nachgewiesen.
batch-template-decision-confirmed = identisches Markup ({ $n } Seiten)
batch-template-decision-likely = wiederkehrend, gleiches Markup nicht belegt ({ $n } Seiten)
batch-template-decision-open = auf { $n } Seiten, Ursache offen
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
batch-seo-weakest-title = Seiten mit dem niedrigsten Inhaltsprofil
batch-seo-weakest-value = { $page_type } · Profil { $score }/100
batch-seo-weakest-note = Das Inhaltsprofil fasst Struktur- und Umfangssignale zusammen. Ein niedriger Wert ist kein festgestellter Mangel: Kurze Übersichts-, Standort- oder Meldungsseiten können vollständig sein. Konkrete Mängel stehen bei den einzelnen Seiten.
batch-seo-action-needed = Handlungsbedarf
batch-seo-patterns-impact-title = Content-Auffälligkeiten → Business-Impact
batch-seo-impact-thin = { $insight } → schwächere Rankings, geringere Verweildauer
batch-seo-impact-duplicate = { $insight } → Keyword-Kannibalisierung, Aufteilung der Sichtbarkeit in Suchmaschinen
batch-seo-near-dup-title = Near-Duplicate-Content → Keyword-Kannibalisierung
batch-seo-risk-high = Hoch — konsolidieren
batch-seo-risk-medium = Mittel — differenzieren
batch-seo-duplicate-title = Gleichlautende Metadaten auf mehreren Seiten
batch-col-dup-type = Typ
batch-col-dup-value = Geteilter Wert
batch-col-dup-count = Seiten
batch-dup-kind-title = Title
batch-dup-kind-description = Meta-Description
batch-dup-kind-h1 = H1-Überschrift
batch-dup-kind-og-image = og:image
batch-seo-missing-tags-title = Fehlende optionale Angaben über Seiten → fehlen sie überall, liegt es meist am Template
batch-col-missing-count = Seiten ohne Tag
batch-col-missing-total = Geprüfte Seiten
batch-missing-kind-canonical = Canonical-URL
batch-missing-kind-og-image = og:image
batch-missing-kind-og-title = og:title
batch-seo-canonical-title = Canonical-Konflikte → widersprüchliche Indexierungssignale auflösen
batch-canonical-noindex = Canonical + noindex
batch-canonical-ogurl = Canonical ≠ og:url
batch-canonical-target-redirects = Canonical-Ziel leitet weiter
batch-canonical-target-unreachable = Canonical-Ziel nicht erreichbar
batch-seo-hreflang-title = Nicht-reziprokes hreflang → Rücklinks ergänzen
batch-col-hreflang-source = Seite
batch-col-hreflang-target = Verweist auf (kein Rücklink)
batch-col-hreflang-lang = Sprache
batch-seo-sitemap-http-title = Sitemap-HTTP-Prüfung
batch-sitemap-http-summary = { $checked } Sitemap-URLs per HTTP abgerufen, Weiterleitungen nicht verfolgt: { $distribution }. Das prüft Erreichbarkeit und Status, nicht Inhalt oder Barrierefreiheit — vollständig auditiert sind nur die Seiten der Stichprobe.
batch-sitemap-class-other = sonstige
batch-sitemap-redirects-title = Weiterleitungen in der Sitemap, nach Ziel
batch-col-redirect-target = Ziel
batch-col-redirect-count = Anzahl
batch-col-redirect-example = Beispiel aus der Sitemap
batch-sitemap-redirects-more = Weitere { $more } Ziele siehe JSON.
batch-sitemap-other-issues-title = Weitere HTTP-Auffälligkeiten
batch-sitemap-http-json-note = Alle einzelnen Quell- und Ziel-URLs stehen im JSON unter sitemap_http_issues.
batch-seo-sitemap-orphan-title = Sitemap-Linkgraph → Seiten ohne beobachteten Eingang und fehlende Sitemap-Einträge
batch-sitemap-kind-status = HTTP-Status
batch-sitemap-kind-redirect = Redirect
batch-sitemap-kind-noindex = noindex
batch-sitemap-kind-fetch-error = Abruffehler
batch-sitemap-kind-orphan = Kein eingehender Link beobachtet
batch-sitemap-orphan-note = { $count ->
        [one] 1 geprüfte Seite
       *[other] { $count } geprüfte Seiten
    } ohne beobachteten eingehenden Link. Beobachtet wurden nur die Links der geprüften Seiten und der Startseite — Seiten außerhalb der Stichprobe, Suche und Menüs aus Skripten fehlen. Das sind Kandidaten, keine bestätigt verwaisten Seiten.
batch-sitemap-kind-linked-missing = Intern verlinkt, fehlt in Sitemap
batch-seo-robots-conflicts-title = Sitemap vs. robots.txt → Indexierungsabsicht durch Disallow-Regel blockiert
batch-col-robots-rule = Blockierende Disallow-Regel
batch-crawl-depth-section = Interner Linkgraph
batch-crawl-depth-title = Crawl-Tiefe → Klicks von der Startseite
batch-crawl-depth-intro = Klick-Distanz von { $start_url } zu jeder geprüften Seite, ermittelt über die in diesem Batch-Lauf bereits erfassten internen Links. Verteilung nach Tiefe: { $histogram }.
batch-crawl-depth-histogram-entry = Tiefe { $depth }: { $count }
batch-crawl-depth-caveat = Dieser Lauf deckt nicht die vollständige Sitemap ab (Stichprobe oder fehlgeschlagene Seiten) — Tiefe und Erreichbarkeit sind daher nur eine Teilansicht des tatsächlichen Linkgraphen, nicht die vollständige Site-Struktur.
batch-col-crawl-depth = Klicks
batch-crawl-depth-kind-deepest = Am weitesten entfernt
batch-crawl-depth-kind-unreachable = Kein Pfad beobachtet (in den erfassten Seiten)
batch-crawl-depth-unobserved-count = { $count ->
        [one] 1 geprüfte Seite
       *[other] { $count } geprüfte Seiten
    } ohne beobachteten Pfad von der Startseite (bis zu 20 davon im JSON unter crawl_depths.unreachable_pages).
batch-crawl-depth-fetched-homepage = Die Startseite wurde dafür nur abgerufen, um ihre Links zu lesen — sie ist nicht auditiert und zählt in keinem Wert.
batch-crawl-depth-unavailable = Klicktiefen nicht ermittelt: Die Startseite ließ sich nicht abrufen. Eine andere Seite wird bewusst nicht als Ersatz-Startseite verwendet.
batch-module-portfolio-title = Modul-Portfolio
batch-module-portfolio-intro = Aggregierte Modulwerte über das geprüfte URL-Set: Schnitt, Spannweite über die Seiten mit Messung und Wirkung je Modul.
batch-module-portfolio-table-title = Modulwerte und Spannweite
batch-col-module = Modul
batch-col-score = Durchschnitt
batch-col-spread = Spannweite
batch-module-spread-constant = { $pages ->
        [one] { $value } auf der 1 Seite mit Messung
       *[other] { $value } auf allen { $pages } Seiten mit Messung
    }
batch-module-spread-range = { $min }–{ $max } ({ $pages ->
        [one] 1 Seite
       *[other] { $pages } Seiten
    })
batch-col-effect = Wirkung
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
metric-accessibility-score = Barrierefreiheit{"\u00A0"}Score
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
finding-structural-cause-component = { $count } Vorkommen folgen demselben Muster — das deutet stark auf eine gemeinsam genutzte Komponente oder ein Template hin, bestätigt ist das erst mit Belegen von weiteren Seiten. Eine Behebung an dieser einen Stelle würde diese Vorkommen voraussichtlich gemeinsam adressieren.
finding-structural-cause-shared = Dieses Problem tritt bei { $count } Elementen auf — möglicherweise eine gemeinsam genutzte Komponente oder ein Template.
module-accessibility = Barrierefreiheit
module-security = Sicherheit
module-performance = Performance
module-seo = SEO
module-html-conformance = HTML-Konformität
module-mobile = Mobile
module-ux = UX
module-journey = Journey
module-best-practices = Best Practices
module-ai-visibility = KI-Sichtbarkeit
module-source-quality = Technische Aufbereitung
module-tech-stack = Tech-Stack
linktext-generic-stopwords = mehr erfahren,weiterlesen,hier klicken,hier,mehr,weiter,lesen,anzeigen,öffnen,details,link,klicken
alt-text-boilerplate-prefixes = bild von,bild:,foto von,foto:,fotografie von,grafik von,symbol für,dies ist ein bild von,dies zeigt ein bild von
topic-stopwords = 2026,aber,allem,alle,auch,auf,aus,autor,bei,bereits,bietet,bild,bilder,casoon,checker,cloud,content,damit,dass,deine,diese,dieser,drei,durch,eine,einem,einen,einer,eines,einfach,entwickelt,entwicklung,erfahren,fuer,für,gmbh,heute,hier,ihre,ihren,ihrer,ihres,inklusive,inhalt,jetzt,keine,kunden,launch,lesen,mehr,moderne,klare,oder,page,pages,projekt,projekten,recht,rund,seite,seiten,seine,seiner,sich,sind,site,statt,systeme,technik,themen,thema,über,und,unsere,unserer,unsers,unter,transparent,viele,vom,von,web,websites,webentwicklung,website,weiter,werden,wird,wenig,willkommen,zeigen,ziel,haben,sowie,digitale,alles,andere,anderen,beim,dann,dein,deinen,deiner,dich,dies,dieses,diesem,diesen,doch,dort,etwas,euch,eure,ganz,gibt,ihnen,immer,jede,jeden,jeder,jedes,kann,kein,keinen,können,mein,meine,mich,muss,nach,neue,neuen,neues,nicht,noch,ohne,schon,sehr,sein,seit,soll,sollen,unser,während,weil,weitere,weiteren,welche,welcher,welches,wenn,wieder,wurde,wurden,zwischen
video-transcript-keywords = transkript
sr-no-name = (kein Name)
sr-role-button = Schalter
sr-role-link = Link
sr-role-textbox = Textfeld
sr-role-checkbox = Kontrollkästchen
sr-role-radio = Optionsfeld
sr-role-combobox = Kombinationsfeld
sr-role-listbox = Listenfeld
sr-role-slider = Schieberegler
sr-role-spinbutton = Drehfeld
sr-role-tab = Registerkarte
sr-role-heading = Überschrift
sr-role-heading-level = Überschrift Ebene { $level }
sr-role-navigation = Navigation
sr-role-main = Hauptbereich
sr-role-banner = Banner
sr-role-contentinfo = Inhaltsinfo
sr-state-expanded = ausgeklappt
sr-state-collapsed = eingeklappt
sr-state-checked = aktiviert
sr-state-unchecked = nicht aktiviert
sr-state-mixed = teilweise aktiviert
sr-state-selected = ausgewählt
sr-state-not-selected = nicht ausgewählt
sr-state-required = erforderlich
sr-state-invalid = ungültig
sr-state-disabled = deaktiviert
sr-state-pressed = gedrückt
sr-state-not-pressed = nicht gedrückt
sr-state-tab-stop = fokussierbar

# Plan 09: easy-language ("Leichte Sprache") link-text markers, comma-separated,
# merged with the English list regardless of report language (sites mix languages).
easy-language-linktext-markers = leichte sprache,einfache sprache

# Audit exclusions (#645): methodology note (single) and audit frame (batch).
exclusion-fact-label = Ausgeschlossene Bereiche
exclusion-selector-matched =
    { $count ->
        [one] { $selector }: 1 Element ausgeschlossen
       *[other] { $selector }: { $count } Elemente ausgeschlossen
    }
exclusion-selector-unmatched = { $selector }: kein Element gefunden, nichts ausgeschlossen
exclusion-selector-invalid = { $selector }: ungültiger Selektor, nicht angewendet
exclusion-findings =
    { $occurrences ->
        [0] kein Befund entfiel
        [one] 1 Befundvorkommen entfiel, nicht im Score
       *[other] { $occurrences } Befundvorkommen aus { $rules ->
            [one] 1 Regel
           *[other] { $rules } Regeln
        } entfielen, nicht im Score
    }
exclusion-interactive = , dazu { $interactive } Journey-Befunde
exclusion-screen-reader = , dazu { $issues } Screenreader-Befunde
exclusion-landmarks =
    { $landmarks ->
        [one] ; 1 Landmark nicht in die Landmark-Zählung der Seite einbezogen
       *[other] ; { $landmarks } Landmarks nicht in die Landmark-Zählung der Seite einbezogen
    }
batch-exclusion-selector-matched = { $selector }: { $count } Elemente auf { $pages } von { $total } Seiten ausgeschlossen
batch-exclusion-findings =
    { $occurrences ->
        [0] kein Befund entfiel
        [one] 1 Befundvorkommen entfiel, nicht im Score
       *[other] { $occurrences } Befundvorkommen entfielen, nicht im Score
    }

# ── Privacy indicators (#872) ─────────────────────────────────────────
privacy-title = Datenschutz-Indikatoren — Erstbesuch
privacy-scope = Technische Beobachtungen eines Erstbesuchs in einem sauberen Browserprofil (keine gespeicherte Einwilligung, kein Klick, kein Scrollen). Das ist keine rechtliche Bewertung und kein Compliance-Wert. Was die Seite nach einer Einwilligungsentscheidung tut, wird nicht gemessen. Der Sitz eines Anbieters ist dessen eigene Angabe und sagt nichts über den tatsächlichen Datenfluss.
privacy-takeaway = Kontaktierte Anbieter: { $providers } · technische Indikatoren: { $findings }. Nur Beobachtungen, keine rechtliche Bewertung.
privacy-takeaway-none = Beim Erstbesuch wurde kein technischer Indikator erfasst. Das ist eine Beobachtung im gemessenen Umfang, keine Aussage, dass die Seite irgendeine Anforderung erfüllt.
privacy-takeaway-not-measured = Der Erstbesuch konnte nicht gemessen werden. Zu Anfragen, Cookies und Speicher vor einer Einwilligungsentscheidung wird keine Aussage getroffen.
privacy-takeaway-unmeasured = Nicht gemessene Prüfungen: { $count } — dort ist „kein Indikator“ kein sauberes Ergebnis.
privacy-metric-contacts = Kontaktierte Anbieter
privacy-metric-findings = Technische Indikatoren
privacy-metric-unmeasured = Nicht gemessene Prüfungen
privacy-measurement-failed = Nicht gemessen: { $reason }
privacy-measurement-partial = Teilweise gemessen, es fehlt: { $gaps }. Prüfungen, die davon abhängen, stehen auf „nicht gemessen“ statt auf „nichts gefunden“.
privacy-table-checks = Prüfungen und Ergebnisse
privacy-table-contacts = Beim Erstbesuch kontaktiert
privacy-table-findings = Technische Indikatoren
privacy-col-check = Prüfung
privacy-col-result = Ergebnis
privacy-col-note = Hinweis
privacy-col-provider = Anbieter
privacy-col-category = Kategorie
privacy-col-seat = Sitz (Hinweis)
privacy-col-requests = Anfragen
privacy-col-hosts = Hosts
privacy-col-indicator = Indikator
privacy-col-confidence = Belastbarkeit
privacy-col-evidence = Beleg
privacy-no-contacts = Beim Erstbesuch wurde kein klassifizierter Anbieter kontaktiert.
privacy-unclassified = Kontaktierte Hosts, die nicht in der Anbieterliste stehen (nicht klassifiziert, kein Befund): { $hosts }
privacy-ruleset = Klassifizierungs-Regelwerk: { $version }
privacy-more = +{ $count } weitere
privacy-check-tracking_requests = Tracking- und Werbe-Anfragen
privacy-check-non_essential_storage = Cookies und Speichereinträge nicht notwendiger Anbieter
privacy-check-visitor_ip_transfer = Anfragen, die die Besucher-IP an Dritte übermitteln
privacy-check-third_country_provider = Anbietersitz außerhalb der EU
privacy-check-consent_platform = Consent-Plattform, wenn Tracker vorhanden sind
privacy-check-policy_links = Links zu Datenschutzerklärung und Impressum
privacy-check-short-tracking_requests = Tracking
privacy-check-short-non_essential_storage = Speicher
privacy-check-short-visitor_ip_transfer = IP-Übermittlung
privacy-check-short-third_country_provider = Drittland
privacy-check-short-consent_platform = Consent-Plattform
privacy-check-short-policy_links = Rechtslinks
privacy-outcome-found = Indikator
privacy-outcome-not_found = Nichts gefunden
privacy-outcome-not_applicable = Nicht zutreffend
privacy-outcome-not_measured = Nicht gemessen
privacy-confidence-evidence = Beleg
privacy-confidence-indication = Hinweis
privacy-confidence-manual-check = Manuell prüfen
privacy-source-request = Anfrage
privacy-source-cookie = Cookie
privacy-source-storage_key = Speicherschlüssel
privacy-source-link = Link
privacy-source-signal = Signal
privacy-source-provider = Anbieter
privacy-seat-eu = EU
privacy-seat-us = USA
privacy-seat-other = Außerhalb der EU
privacy-seat-unknown = Nicht angegeben
privacy-category-analytics = Analyse
privacy-category-advertising = Werbung
privacy-category-social = Soziale Medien
privacy-category-tag_manager = Tag-Manager
privacy-category-fonts = Schriften
privacy-category-cdn = CDN
privacy-category-video_embed = Video-Einbettung
privacy-category-maps = Karten
privacy-category-consent_platform = Consent-Plattform
privacy-gap-cookies = Cookies
privacy-gap-storage = Web-Speicher
privacy-gap-page_signals = Seitensignale (Consent-Spuren, Links)
privacy-gap-network_not_idle = Netzwerk wurde nicht ruhig
privacy-reason-no_third_party_requests = keine Drittanfrage
privacy-reason-network_not_idle = Netzwerk wurde nicht ruhig, späte Anfragen sind nicht auszuschließen
privacy-reason-storage_unreadable = Cookies oder Speicher nicht lesbar
privacy-reason-nothing_stored = nichts gespeichert
privacy-reason-no_classified_provider = kein klassifizierter Anbieter kontaktiert
privacy-reason-no_tracking_requests = keine Tracking-Anfrage beobachtet
privacy-reason-consent_traces_unreadable = Consent-Spuren nicht lesbar
privacy-reason-page_signals_unreadable = Seitensignale nicht lesbar
privacy-reason-no_links_on_page = keine Links auf der Seite
privacy-reason-cdp_command_failed = Browser-Befehl fehlgeschlagen
privacy-reason-evaluation_failed = Seitenauswertung fehlgeschlagen
privacy-kind-tracking_request_before_consent = Tracking- oder Werbe-Anfrage beim Erstbesuch
privacy-kind-non_essential_storage_before_consent = Cookie oder Speichereintrag eines nicht notwendigen Anbieters beim Erstbesuch
privacy-kind-visitor_ip_transfer = Anfrage, die die Besucher-IP an Dritte übermittelt
privacy-batch-title = Datenschutz-Indikatoren — aggregiert
privacy-batch-intro = Technische Beobachtungen je eines Erstbesuchs pro Seite (sauberes Browserprofil, kein Klick), aggregiert über die geprüften Seiten. Keine rechtliche Bewertung und kein Compliance-Wert. Die Zahlen beziehen sich auf die Seiten, deren Erstbesuch gemessen wurde.
privacy-batch-coverage = Erstbesuch auf { $measured } von { $total } geprüften Seiten gemessen; nicht gemessen: { $failed }.
privacy-batch-affected = { $pages } von { $measured } gemessenen Seiten zeigen mindestens einen technischen Indikator.
privacy-batch-none = Keine gemessene Seite zeigt einen technischen Indikator.
privacy-batch-nothing-measured = Der Erstbesuch konnte auf keiner Seite gemessen werden. Es wird keine Aussage getroffen.
privacy-batch-table-kinds = Indikatoren nach betroffenen Seiten
privacy-batch-table-providers = Wiederkehrende Anbieter
privacy-batch-table-checks = Prüfergebnisse über alle Seiten
privacy-batch-table-matrix = Seiten im Überblick
privacy-batch-col-pages = Seiten
privacy-batch-col-spread = Verbreitung
privacy-batch-col-found = Indikator
privacy-batch-col-not_found = Nichts gefunden
privacy-batch-col-not_applicable = Nicht zutreffend
privacy-batch-col-not_measured = Nicht gemessen
privacy-batch-pages-of = { $pages } von { $measured }
privacy-batch-spread-all = auf allen gemessenen Seiten
privacy-batch-spread-several = auf mehreren Seiten
privacy-batch-spread-single = auf einer einzelnen Seite
privacy-batch-providers-single = { $count } weitere Anbieter nur auf einer einzelnen Seite.
privacy-batch-matrix-legend = Zahl: gefundene Indikatoren. 0: nichts gefunden. –: nicht zutreffend. n. g.: nicht gemessen.
privacy-batch-matrix-more = Gezeigt: { $shown } von { $total } Seiten; der Rest steht im JSON-Bericht.
privacy-batch-matrix-nm = n. g.

# HTML conformance: written recommendation per rule family (#899)
html-conform-parser-title = HTML-Syntaxfehler beheben
html-conform-parser-fix = Die Syntaxfehler beheben, die der Browser beim Einlesen stillschweigend korrigiert (nicht geschlossene oder falsch verschachtelte Tags, überzählige End-Tags, doppelte Attribute). Den gerenderten HTML-Code gegen den Prüfer laufen lassen und die Ursache im Template oder Redaktionssystem korrigieren, nicht nur das Ergebnis.
html-conform-schema-title = Unzulässige Elemente und Attribute entfernen
html-conform-schema-fix = Elemente und Attribute so einsetzen, wie es der HTML-Standard an dieser Stelle erlaubt: unbekannte oder veraltete Attribute entfernen, Elemente nur in erlaubte Eltern-Elemente setzen und ungültige Attributwerte korrigieren. Die Meldung nennt jeweils das betroffene Element.
html-conform-ids-title = Doppelte IDs eindeutig machen
html-conform-ids-fix = Jede id im Dokument nur einmal vergeben, auch bei wiederholten Komponenten oder Karten. Label-Bezüge (for), ARIA-Verweise und Anker zeigen sonst auf das falsche Element. Wiederholte Komponenten brauchen eine pro Instanz erzeugte id.
html-conform-headings-title = Überschriftenstruktur korrigieren
html-conform-headings-fix = Überschriftenebenen lückenlos und ohne leere Überschriften einsetzen: keine Ebene überspringen (h1 direkt zu h3), jedes section- und article-Element mit einer Überschrift versehen und genau eine Hauptüberschrift setzen. Die Ebene folgt der Gliederung, nicht der gewünschten Schriftgröße.
html-conform-headings-empty-fix = Jedem h1–h6-Element einen Text geben, der seinen Abschnitt benennt, oder die leere Überschrift entfernen; eine leere Überschrift wird ohne Namen angesagt.
html-conform-headings-article-lacks-heading-fix = Jedem article-Element eine Überschrift (h2–h6) geben, die seinen Inhalt benennt, oder ein div verwenden, wo der Inhalt kein eigenständiger Artikel ist.
html-conform-headings-section-lacks-heading-fix = Jedem section-Element eine Überschrift (h2–h6) geben, die seinen Inhalt benennt, oder ein div verwenden, wo der Inhalt kein thematischer Abschnitt der Gliederung ist.
html-conform-headings-no-top-level-fix = Die Hauptüberschrift der Seite als h1 auszeichnen; die Seite hat Überschriften, aber keine der Ebene 1.
html-conform-headings-skip-level-fix = Keine Überschriftenebene überspringen: Ein h3 direkt nach dem h1 braucht ein h2 dazwischen oder wird selbst zum h2. Die Ebene folgt der Gliederung, nicht der gewünschten Schriftgröße.
html-conform-headings-offset-range-fix = Das Attribut headingoffset auf eine ganze Zahl von 0 bis 8 setzen.
html-conform-language-title = Seitensprache am html-Element angeben
html-conform-language-fix = Das lang-Attribut am html-Element mit dem passenden Sprachcode setzen (zum Beispiel lang=de) und, falls xml:lang vorhanden ist, denselben Wert verwenden. Screenreader wählen die Aussprache danach.
html-conform-image-alt-title = Bildern eine Textalternative geben
html-conform-image-alt-fix = Jedem img-Element ein alt-Attribut geben: einen kurzen beschreibenden Text für inhaltliche Bilder, ein leeres alt für rein dekorative. Bilder mit role oder aria-Attributen brauchen einen zugänglichen Namen oder ein leeres alt ohne widersprüchliche Rolle.
html-conform-responsive-images-title = srcset, sizes und source-Angaben korrigieren
html-conform-responsive-images-fix = Bei srcset mit Breitenangaben (w) das sizes-Attribut ergänzen, sizes nur zusammen mit srcset verwenden und source-Elemente in picture mit media oder type versehen. Widersprüchliche Kombinationen führen dazu, dass Browser das Bild falsch oder gar nicht auswählen.
html-conform-image-maps-title = Bildkarten (Image Maps) korrigieren
html-conform-image-maps-fix = Bildkarten vollständig definieren: usemap muss auf eine vorhandene map mit passendem name und id zeigen, area-Elemente gehören in die map, und ismap benötigt einen umschließenden Link mit href. Wo möglich, Bildkarten durch echte Links ersetzen.
html-conform-nested-interactive-title = Verschachtelte interaktive Elemente auflösen
html-conform-nested-interactive-fix = Keine Links, Buttons, Formularfelder oder fokussierbaren Elemente innerhalb von Links oder Buttons verschachteln. Den Klickbereich stattdessen über einen einzigen Link mit Block-Darstellung lösen oder die Elemente nebeneinander statt ineinander setzen.
html-conform-forms-title = Formularelemente korrekt auszeichnen
html-conform-forms-fix = Formularfelder gemäß Standard auszeichnen: label mit for auf ein vorhandenes Feld beziehen, nur je Feld ein Label, passende Attribute je type (pattern, maxlength, required, readonly nur, wo der Typ sie zulässt), Auswahllisten mit gültigem Platzhalter-Eintrag und gültiger Struktur aus select, optgroup und option, und bei accept-charset nur utf-8.
html-conform-document-meta-title = Metadaten im Dokumentkopf korrigieren
html-conform-document-meta-fix = Den Dokumentkopf bereinigen: einen nicht leeren title, genau ein meta charset mit UTF-8 am Anfang, ein viewport-Meta ohne user-scalable=no, höchstens eine description und base vor allen Links und Skripten. Nicht mehr benötigte Altlasten wie X-UA-Compatible entfernen.
html-conform-link-rel-title = link-Elemente im Kopf korrigieren
html-conform-link-rel-fix = link-Elemente vollständig angeben: href vorhanden, rel passend zu den weiteren Attributen (as bei preload und modulepreload mit gültigem Wert, imagesizes nur mit imagesrcset, integrity nur bei erlaubten rel-Werten, alternate stylesheet mit title).
html-conform-scripts-title = Skript-Attribute und Skriptarten korrigieren
html-conform-scripts-fix = Skript-Attribute nur in zulässigen Kombinationen setzen: kein defer an Modulen, kein async oder defer an Inline-Skripten, keine Attribute wie nomodule, integrity oder fetchpriority an Import-Maps und Speculation-Rules, und den unnötigen Typ text/javascript sowie das veraltete language-Attribut entfernen. Inhalte von importmap und speculationrules müssen gültiges JSON sein.
html-conform-csp-title = Content-Security-Policy im meta-Element einhalten
html-conform-csp-fix = Inline-Skripte und Inline-Styles entweder in Dateien auslagern, per Nonce oder Hash in der Richtlinie freigeben oder die Richtlinie anpassen. Die per meta gesetzte Richtlinie blockiert sie sonst im Browser. Bevorzugt die Richtlinie als HTTP-Header ausliefern.
html-conform-tables-title = Tabellenstruktur korrigieren
html-conform-tables-fix = Tabellen als regelmäßiges Raster aufbauen: jede Zeile hat gleich viele Zellen, colspan und rowspan überschreiten Tabellenrand und Gruppen nicht, Zeilen sind nicht leer, th erhält ein gültiges scope und headers verweist auf vorhandene th-Zellen. Tabellen nicht für Layout verwenden.
html-conform-aria-redundant-role-title = Überflüssige ARIA-Rollen entfernen
html-conform-aria-redundant-role-fix = Rollen entfernen, die das HTML-Element bereits von Haus aus mitbringt (zum Beispiel role=button an button, role=navigation an nav, role=list an ul). Redundante Rollen erzeugen Wartungsaufwand und können native Eigenschaften überschreiben.
html-conform-aria-naming-title = ARIA-Namen nur dort vergeben, wo sie erlaubt sind
html-conform-aria-naming-fix = aria-label, aria-labelledby und aria-braillelabel nur an Elementen und Rollen setzen, die einen Namen tragen dürfen; bei Rollen ohne Namensunterstützung (zum Beispiel generic) den Text sichtbar machen oder eine passende Rolle wählen. Bei label-Elementen mit for kein zusätzliches aria-label verwenden.
html-conform-aria-state-title = Unzulässige ARIA-Attribute und -Zustände entfernen
html-conform-aria-state-fix = ARIA-Zustände nur dort setzen, wo Rolle und Element sie erlauben: keine aria-valuemin, aria-valuemax oder aria-checked an Elementen, die min, max oder checked nativ ausdrücken, kein aria-hidden an fokussierbaren Elementen, aria-activedescendant und aria-controls nur mit vorhandenem Ziel. Native Attribute haben Vorrang vor ARIA.
html-conform-aria-structure-title = ARIA-Rollen im richtigen Zusammenhang einsetzen
html-conform-aria-structure-fix = Rollen nur im passenden Umfeld einsetzen: option in listbox, row in table, grid oder treegrid, listitem in list, tab in tablist, cell in row. Keine Buttons oder Überschriften in Elementen platzieren, deren Kinder als rein darstellend gelten (role=img, button), und keine Rollen innerhalb von Überschriften oder summary verwenden, die dort verboten sind. Wo möglich, das native Element nehmen.
html-conform-obsolete-title = Veraltete HTML-Elemente ersetzen
html-conform-obsolete-fix = Veraltete Elemente wie font, center, marquee oder big durch semantisches HTML und CSS ersetzen. Sie werden nicht mehr weiterentwickelt, und Hilfstechnologien behandeln sie uneinheitlich.
html-conform-microdata-title = Microdata-Attribute vervollständigen
html-conform-microdata-fix = Microdata-Attribute nur gemeinsam setzen: itemtype, itemid und itemref benötigen ein umschließendes itemscope, itemprop gehört in ein Element mit itemscope, und itemref muss auf eine vorhandene id zeigen. Alternativ strukturierte Daten als JSON-LD ausliefern.
html-conform-meter-progress-title = Wertebereiche von meter und progress korrigieren
html-conform-meter-progress-fix = Die Werte von meter und progress in eine schlüssige Reihenfolge bringen: min kleiner als max, value innerhalb dieses Bereichs, bei meter low kleiner als high und optimum innerhalb von min und max. Bei progress ohne max gilt der Bereich 0 bis 1.
html-conform-sectioning-title = Gliederungselemente korrekt verschachteln
html-conform-sectioning-fix = Gliederungselemente nach Standard einsetzen: nur ein sichtbares main, nicht innerhalb von article, aside, footer, header oder nav; kein header oder footer in header oder footer; keine adresse in address; dt ohne Überschriften, header, footer oder Gliederungselemente und ohne doppelte Namen; figure mit figcaption statt caption.
html-conform-media-text-title = Medien- und Textelemente korrekt einsetzen
html-conform-media-text-fix = Medien- und Spezialelemente vollständig angeben: track mit label und höchstens einem default, ruby und rtc in der vorgesehenen Struktur, bdo mit dir und custom elements ohne is-Attribut. Nicht standardkonforme Varianten durch die vorgesehene Auszeichnung ersetzen.
html-conform-text-normalization-title = Text in Unicode-Normalform C speichern
html-conform-text-normalization-fix = Den genannten Text in NFC speichern, wie es der HTML-Standard Autoren empfiehlt: Buchstabe und getrenntes Kombinationszeichen werden zum einen vorkomponierten Zeichen. Zerlegter Text stammt meist aus kopierten Inhalten oder Dateinamen; ihn im Editor, CMS oder Build-Schritt normalisieren, damit Suche und Textabgleich ihn finden.
html-conform-generic-title = HTML-Konformitätsverstoß beheben
html-conform-generic-fix = Das in der Meldung genannte Element oder Attribut gemäß HTML-Standard korrigieren und den gerenderten HTML-Code erneut prüfen. Tritt der Befund in jeder Seite auf, die Ursache im gemeinsamen Template beheben.

# Count messages with singular/plural agreement (#902). Used through `I18n::count`.
count-occurrences-found = { $count ->
        [one] Es wurde { $count } Vorkommen erkannt
       *[other] Es wurden { $count } Vorkommen erkannt
    }
count-problem-groups = { $count ->
        [one] { $count } Problemgruppe
       *[other] { $count } Problemgruppen
    }
count-more-findings = { $count ->
        [one] { $count } weiterer Befund
       *[other] { $count } weitere Befunde
    }
count-notes = { $count ->
        [one] { $count } Hinweis
       *[other] { $count } Hinweise
    }
count-warnings = { $count ->
        [one] { $count } Warnung
       *[other] { $count } Warnungen
    }
count-h1-headings = { $count ->
        [one] { $count } H1-Überschrift
       *[other] { $count } H1-Überschriften
    }
count-link-texts-repeated = { $count ->
        [one] { $count } Linktext kommt
       *[other] { $count } Linktexte kommen
    }
count-links-generic = { $count ->
        [one] { $count } Link trägt
       *[other] { $count } Links tragen
    }
count-heuristic-warnings = { $count ->
        [one] { $count } heuristische Barrierefreiheits-Warnung
       *[other] { $count } heuristische Barrierefreiheits-Warnungen
    }
count-manual-points = { $count ->
        [one] { $count } Stelle
       *[other] { $count } Stellen
    }
count-a11y-warnings = { $count ->
        [one] { $count } Barrierefreiheits-Warnung
       *[other] { $count } Barrierefreiheits-Warnungen
    }
count-manual-hints = { $count ->
        [one] { $count } manueller Prüfhinweis
       *[other] { $count } manuelle Prüfhinweise
    }
count-recommendations = { $count ->
        [one] { $count } Empfehlung
       *[other] { $count } Empfehlungen
    }
count-images-scaled-down = { $count ->
        [one] { $count } Bild wird
       *[other] { $count } Bilder werden
    }

pdf-ai-policy-label = Richtlinie
pdf-ai-policy-status-label = Status

module-commerce = Shop-Prüfung
module-privacy = Datenschutz
count-findings = { $count ->
        [one] { $count } Befund
       *[other] { $count } Befunde
    }
pdf-metric-issues = Probleme
pdf-sec-headers-title = Sicherheits-Header
pdf-screenshot-alt-desktop = Desktop-Screenshot der geprüften Seite
pdf-screenshot-alt-mobile = Mobile-Screenshot der geprüften Seite

seo-cookie-inventory =
    { $count } { $count ->
        [one] Cookie
       *[other] Cookies
    }
seo-cookie-inventory-risky =
    { $count } { $count ->
        [one] ohne Secure/SameSite-Signal
       *[other] ohne Secure/SameSite-Signal
    }

annotations-section-title = Einschätzungen
annotations-section-intro = Externe Einschätzungen aus einer Annotationsdatei. Jede steht bei dem Befund, auf den sie sich bezieht. Es sind keine Befunde, und sie haben weder Ergebnis, Schweregrad, Wert, Risikostufe noch Exit-Code verändert.
annotations-counts = { $total } Einschätzungen: { $agent } von Agenten, { $human } von Reviewern. { $matched } trafen einen Befund dieses Laufs, { $unmatched } keinen.
annotations-label-agent = Einschätzung (Agent)
annotations-label-human = Einschätzung (Reviewer)
annotations-value-likely-issue = wahrscheinlich ein Problem
annotations-value-likely-fine = wahrscheinlich unproblematisch
annotations-value-undetermined = offen
annotations-scope-reading-flow = Lesefluss
annotations-evidence = Belege
annotations-pattern = Elementmuster
annotations-matched-title = Einschätzungen je Seite
annotations-unmatched-title = Nicht zugeordnete Einschätzungen
annotations-unmatched-intro = Diese Einschätzungen treffen in diesem Lauf nichts (der Befund kann behoben oder das Element geändert sein). Sie werden aufgeführt, nicht verworfen.
annotations-col-page = Seite
annotations-col-target = Befund
annotations-col-author = Urheber
annotations-col-assessment = Einschätzung
annotations-col-reasoning = Begründung
annotations-col-reason = Grund
annotations-reason-finding-not-found = kein Befund mit dieser Kennung
annotations-reason-pattern-not-found = Befund vorhanden, Elementmuster nicht
annotations-reason-url-not-audited = Seite in diesem Lauf nicht geprüft
annotations-reason-url-required = Lesefluss braucht in einem Lauf über mehrere Seiten eine URL
annotations-reason-no-reading-flow = Seite hat keine Screenreader-Prüfung
annotations-warnings-title = Hinweise zur Annotationsdatei

# Batch decision, risk and portfolio texts (#1039)
batch-rating-title = Gesamteinstufung
batch-rating-body = { $rating } — automatisierte Prüfung { $verdict }: { $passed } von { $total ->
        [one] { $total } geprüften URL
       *[other] { $total } geprüften URLs
    } { $passed ->
        [one] hat
       *[other] haben
    } für sich bestanden.
batch-rating-verdict-pass = bestanden
batch-rating-verdict-warn = mit Vorbehalt bestanden
batch-rating-verdict-fail = nicht bestanden
batch-rating-reasons = Gründe: { $reasons }.
batch-rating-note = Die Einstufung ist ein Bandlabel für den durchschnittlichen Barrierefreiheitswert im automatisierten Prüfumfang, kein Konformitätsnachweis.
batch-reason-no-url-audited = keine der { $count } angeforderten URLs hat einen Report ergeben
batch-reason-legal-flags = { $count ->
        [one] 1 WCAG-Regel der Stufe A/AA mit hoher oder kritischer Schwere ist auf den geprüften Seiten verletzt (mögliche Rechtsrelevanz, BFSG/EAA)
       *[other] { $count } WCAG-Regeln der Stufe A/AA mit hoher oder kritischer Schwere sind auf den geprüften Seiten verletzt (mögliche Rechtsrelevanz, BFSG/EAA)
    }
batch-reason-blocking-issues = { $count ->
        [one] 1 Bedienelement ohne zugänglichen Namen oder nicht per Tastatur bedienbar
       *[other] { $count } Bedienelemente ohne zugänglichen Namen oder nicht per Tastatur bedienbar
    }
batch-reason-technical-blockers = { $count ->
        [one] 1 technischer Blocker
       *[other] { $count } technische Blocker
    }
batch-reason-average-below-fail = durchschnittlicher Barrierefreiheitswert { $score } unter der Fail-Schwelle von { $threshold }
batch-reason-incomplete-scope = { $audited } von { $requested } angeforderten URLs geprüft
batch-reason-audit-quality = der Lauf ist nicht vollständig durchgelaufen
batch-reason-average-below-warn = durchschnittlicher Barrierefreiheitswert { $score } unter der Warnschwelle von { $threshold }
batch-reason-failed-urls = { $count ->
        [one] 1 URL hat die eigene Prüfung nicht bestanden
       *[other] { $count } URLs haben die eigene Prüfung nicht bestanden
    }
batch-reason-violations-found = { $count } WCAG-Vorkommen gefunden
batch-risk-summary-low = Die geprüften Seiten weisen insgesamt ein geringes Barrierefreiheits-Risiko auf.
batch-risk-summary-medium = { $count } von { $total } geprüften Seiten weisen mindestens mittleres Risiko auf. Gezielte Verbesserungen empfohlen.
batch-risk-summary-high = { $count } von { $total } geprüften Seiten weisen hohes Risiko auf. Zeitnahe Behebung empfohlen, besonders bei WCAG-Level-A/AA-Verstößen.
batch-risk-summary-critical = { $count } von { $total } geprüften Seiten weisen kritisches Risiko auf. WCAG-Level-A/AA-Verstöße automatisiert erkannt — vorrangige Behebung empfohlen; ob daraus rechtliche Pflichten folgen, hängt vom anwendbaren Rahmen (z. B. BFSG/EAA) ab und braucht eine manuelle Prüfung.
batch-matrix-col-accessibility = Barrierefreiheit
batch-schema-matrix-col-kind = Seitenart (Schema-Prüfung)
batch-schema-matrix-title = Schema-Abdeckung nach Seitenart
batch-schema-matrix-note = Die Schema-Prüfung ordnet jede Seite eigenständig aus Seitenzweck, URL und sichtbarem Inhalt ein, um die passenden schema.org-Typen zu wählen; diese Seitenart kann vom inhaltlichen Seitentyp der übrigen Abschnitte abweichen. Vorgeschlagene Typen sind optional, keine Pflicht.
batch-scope-note-title = Einordnung dieses Reports
batch-scope-note-appendix = Dieser PDF-Report ist ein verdichteter Audit-Bericht. Er zeigt die wichtigsten Befunde, Risiken und Verbesserungsfelder, enthält aber nicht jede technische Einzelstelle. Details je URL stehen im technischen Anhang am Ende dieses Reports, die Rohdaten im JSON-Report desselben Laufs.
batch-scope-note-json = Dieser PDF-Report ist ein verdichteter Audit-Bericht. Er zeigt die wichtigsten Befunde, Risiken und Verbesserungsfelder, enthält aber nicht jede technische Einzelstelle. Die Rohdaten stehen im JSON-Report desselben Laufs; die Report-Stufe technical (--report-level technical) ergänzt einen Anhang je URL.
batch-next-steps-manual = Für eine vollständige WCAG-Konformitätsprüfung empfehlen wir ergänzend einen manuellen Audit mit assistiven Technologien (Screenreader, Tastaturnavigation). Dieser Audit prüft { $automated } von { $total } Erfolgskriterien der WCAG 2.2 (Stufe A/AA) automatisiert; die übrigen brauchen eine manuelle Prüfung.
batch-next-steps-intro = Was der automatisierte Audit offen lässt.
batch-col-role = Rolle
batch-topics-title = Inhaltsthemen auf mehreren Seiten
batch-table-pages-profile = Seitentyp und häufigste Befunde je Seite
batch-seo-profiles-title = Inhaltsprofil je Seite
batch-col-dup-urls = URLs
batch-dup-pages-more = { $list } und { $count } weitere
batch-user-impact-critical = Kritisch — auf den geprüften Seiten erschweren Barrieren die Nutzung mit Hilfstechnologien erheblich
batch-user-impact-limited = Eingeschränkt — strukturelle Probleme behindern Nutzer mit Hilfstechnologien
batch-user-impact-good = Gut — einzelne Barrieren für Hilfstechnologien auf mehreren Seiten
batch-user-impact-very-good = Sehr gut — Hilfstechnologien werden weitgehend unterstützt
batch-user-impact-barriers-fail = Mit Barrieren nutzbar — hoher Durchschnitt, aber Barrieren auf den geprüften Seiten lassen die automatisierte Prüfung scheitern (siehe Gesamteinstufung)
batch-effect-open-high = { $count ->
        [one] Guter Durchschnitt, aber 1 Befund hoher Schwere dieses Moduls ist auf den geprüften Seiten noch offen.
       *[other] Guter Durchschnitt, aber { $count } Befunde hoher Schwere dieses Moduls sind auf den geprüften Seiten noch offen.
    }
batch-effect-uneven = Guter Durchschnitt, aber ungleichmäßig: Die schwächste geprüfte Seite erreicht { $min }/100.
batch-effect-accessibility-good = Barrierefreiheit ist breit stabil; auf den geprüften Seiten ist kein Befund hoher Schwere offen.
batch-effect-accessibility-weak = Barrieren betreffen wiederkehrende Nutzerpfade und sollten nach Reichweite priorisiert werden.
batch-effect-performance-good = Das Ladeverhalten ist über die geprüften Seiten zuverlässig.
batch-effect-performance-weak = Langsameres Laden kann wichtige Seitengruppen verzögern und Abschlüsse erschweren.
batch-effect-seo-good = Grundlagen für Sichtbarkeit in Suchmaschinen sind im geprüften URL-Set konsistent.
batch-effect-seo-weak = SEO-Inkonsistenzen können die Auffindbarkeit über Seitentemplates hinweg schwächen.
batch-effect-security-good = Die automatisierten Checks fanden auf den geprüften Seiten kein Sicherheitsproblem hoher Schwere; das ist keine Sicherheitsgarantie.
batch-effect-security-weak = Security-Befunde sollten zentral gelöst werden, da sie häufig alle Templates betreffen.
batch-effect-mobile-good = Die Nutzung auf Mobilgeräten ist über die geprüften Seiten stabil.
batch-effect-mobile-weak = Mobile Probleme können die Nutzung auf kleineren Displays über Seitengruppen erschweren.
batch-effect-html-conformance-good = Das HTML-Markup zeigt auf den geprüften Seiten keinen Konformitätsfehler hoher Schwere, sodass Browser und assistive Technologien es konsistent interpretieren können.
batch-effect-html-conformance-weak = HTML-Konformitätsprobleme stammen häufig aus einem gemeinsamen Template und können die Interpretation durch Browser und assistive Technologien seitenübergreifend beeinträchtigen.
batch-effect-other-good = Das Modul ist über die geprüften Seiten stabil.
batch-effect-other-usable = Das Modul hat eine tragfähige Basis mit gezieltem Bereinigungsbedarf.
batch-effect-other-recurring = Das Modul zeigt wiederkehrende Schwächen im geprüften URL-Set.
batch-effect-other-structural = Das Modul benötigt strukturelle Aufmerksamkeit über mehrere Seiten.
batch-ranking-col-rank = Nr.
batch-ranking-col-page = Seite
batch-ranking-col-overall = Gesamt
batch-ranking-col-accessibility = Barrierefr.
batch-ranking-col-critical-high = Krit./hoch
batch-ranking-col-seo = SEO
batch-ranking-col-performance = Perf
batch-ranking-col-security = Sich.
batch-ranking-col-html-conformance = HTML
batch-topic-pages = { $count ->
        [one] Auf 1 geprüften Seite vertreten
       *[other] Auf { $count } geprüften Seiten vertreten
    }
batch-interactive-none = { $total ->
        [one] Keine interaktiven Befunde auf der geprüften Seite.
       *[other] Keine interaktiven Befunde auf den { $total } geprüften Seiten.
    }
batch-interactive-some = { $count ->
        [one] 1 von { $total } geprüften Seiten hat
       *[other] { $count } von { $total } geprüften Seiten haben
    } interaktive Befunde — { $clean ->
        [one] 1 Seite
       *[other] { $clean } Seiten
    } ohne Befunde.
