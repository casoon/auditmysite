# Texts of the single report's methodology section and the score
# methodology of its appendix (`output::builder::single::methodology`, #936).
# The builder keeps the thresholds, which message applies and how lists are
# joined. The audit-fact labels WCAG level, Checked nodes, Runtime, Audit
# notes, Display mode and Active modules reuse `scope-box-*` and
# `batch-cover-frame-*` of report.ftl, the module name in the weight fallback
# `module-accessibility`.

## Appendix
methodology-score-methodology = Score-Berechnung: Jeder Barrierefreiheits-Viewport startet bei 100 Punkten; taxonomiebasierte, regelspezifische Abzüge mit logarithmischer Skalierung berücksichtigen wiederholte Verstöße. Bei Desktop- und Mobile-Daten ist der kanonische Barrierefreiheits-Score 70 % Mobile plus 30 % Desktop. Die zusammengeführte Befundliste dient Evidenz und Maßnahmenplanung und erzeugt keinen dritten Score. Widersprüchliche Signale werden markiert, verändern den Score aber nicht nachträglich.

## Methodology block
methodology-scope = Automatisierte Prüfung der Seite { $url } auf Barrierefreiheit nach WCAG 2.2 (Level { $level }). Angeforderte Module: { $modules }.
methodology-method = Die Prüfung erfolgte über das Chrome DevTools Protocol (CDP) und den nativen Accessibility Tree des Browsers. Über die konfigurierten Viewports { $count ->
        [one] wurde { $rules } automatisierte Regelausführung
       *[other] wurden { $rules } automatisierte Regelausführungen
    } erfasst.
methodology-limitations = Automatisierte Tests können ca. 30–40% aller Barrierefreiheitsprobleme erkennen. Komplexe Aspekte wie korrekte Tab-Reihenfolge, sinnvolle Alt-Texte oder verständliche Sprache erfordern zusätzlich manuelle Prüfung.
# Appended to the limitations after a space.
methodology-limitations-incomplete = Dieser Lauf ist unvollständig: Die Scores beschreiben nur den erfolgreich gemessenen Umfang.
methodology-disclaimer = Dieser Report stellt eine automatisierte technische Analyse dar. Er ersetzt keine vollständige Konformitätsbewertung nach WCAG 2.2. Für eine rechtsverbindliche Aussage zur Barrierefreiheit ist eine umfassende manuelle Prüfung durch Experten erforderlich. Barrierefreiheit ist kein einmalig erreichbarer Zustand, sondern ein fortlaufender Prozess: Jede neue Komponente und jeder neue Inhalt muss bei Einführung erneut geprüft werden.

## Audit facts
methodology-fact-accessibility-score = Barrierefreiheitswert
methodology-fact-combined-score = Kombinierter technischer Wert
methodology-fact-counting = Zählweise
methodology-fact-page-display-modes = Darstellungsmodi der Seite
methodology-fact-failed-modules = Fehlgeschlagene Module
methodology-fact-audit-quality = Audit-Qualität
methodology-fact-preview = Vorschau
methodology-accessibility-score-viewports = { $score } / 100 — 70 % Mobile ({ $mobile }) + 30 % Desktop ({ $desktop })
methodology-combined-score = { $score } / 100 — beitragende Gewichtung: { $weights }
methodology-weight = { $module } { $pct } %
# Appended to the combined score after a space.
methodology-indicator-modules = Separat ausgewiesene Indikator-Module: { $modules }.
methodology-counting = { $groups } { $groups_n ->
        [one] WCAG-Regelgruppe
       *[other] unterschiedliche WCAG-Regelgruppen
    }, { $occurrences } WCAG-Vorkommen; { $rows } { $rows_n ->
        [one] Befundzeile
       *[other] Befundzeilen
    } und { $all } Vorkommen über alle Kategorien
methodology-failed-modules-none = Keine
methodology-quality-complete = Vollständig
methodology-quality-partial = Teilweise
methodology-quality-insufficient = Unzureichend
methodology-preview-captured = Desktop und Mobile erfasst
methodology-preview-not-captured = Nicht erfasst

## Confidence summary
methodology-label-confidence = Audit-Vertrauen
methodology-label-coverage = Modul-Abdeckung
methodology-label-signals = Konfliktsignale
methodology-label-manual = Manuelle Prüfung nötig
methodology-manual-review = Ja, für semantische Qualität und Nutzungskontext
methodology-confidence-high = Hoch
methodology-confidence-solid = Solide
methodology-confidence-limited = Begrenzt
methodology-confidence-off = Nicht aktiv
methodology-signals-none = Keine automatisiert erkannten Konfliktsignale
methodology-signals-one = 1 Hinweissignal
methodology-signals-multiple = Mehrere Hinweissignale
methodology-coverage-broad = Breit
methodology-coverage-extended = Erweitert
methodology-coverage-core = Kern-Checks

## Capability matrix
methodology-capability-wcag = WCAG-Regeln & Vorkommen
methodology-capability-wcag-source = Accessibility Tree + Regelengine
methodology-capability-wcag-note = Primäre Audit-Wahrheit für automatisiert erkennbare Verstöße.
methodology-capability-performance = Web Vitals & Ladeindikatoren
methodology-capability-performance-source = Performance-Modul
methodology-capability-performance-note = FCP, CLS und TTFB werden in Facts und Modulkapiteln gespiegelt.
methodology-capability-seo = SEO-Struktur & Schema
methodology-capability-seo-source = SEO-Modul
methodology-capability-seo-note = Meta-, Heading- und Schema-Signale sind reportfähig verdichtet.
methodology-capability-security = Security Header & HTTPS
methodology-capability-security-source = Security-Modul
methodology-capability-security-note = Header-Präsenz und HTTPS-Status bleiben als Rohsignal sichtbar.
methodology-capability-heuristic = Mobile, UX, Journey
methodology-capability-heuristic-source = Heuristik-Module
methodology-capability-heuristic-confidence = Hinweisbasiert
methodology-capability-heuristic-note = Zur Priorisierung geeignet, nicht als alleinige UX-Gesamtwahrheit.
methodology-capability-flags = Audit-Konfliktsignale
methodology-capability-flags-source = Normalisierung / Cross-Checks
methodology-capability-flags-confidence = Explizit markiert
methodology-capability-flags-note = { $count } { $count ->
        [one] Konfliktsignal wird
       *[other] Konfliktsignale werden
    } offen ausgewiesen statt im Score versteckt.
