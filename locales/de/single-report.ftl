# Texts of the single PDF's page sections (`output::pdf::single_report`,
# `single-report-*`), #936: the part dividers, the management summary, the
# findings classification and matrix, the appendix with its manual-review
# notes, the recognized structural patterns and the dual viewport summary.
# Which message applies, the thresholds and how lists are joined stay in Rust.
# The severity labels, the overall rating title, verdict words and reasons,
# the module names and the column headers "Priority", "Occurrences" and
# "Effort" are messages of report.ftl.

## Part dividers
single-report-part-eyebrow = TEIL { $part } VON 3
single-report-part-eyebrow-appendix = TEIL 3 VON 3 · TECHNISCHER ANHANG

## Management summary
single-report-scope-line = Geprüft: 1 URL · Desktop und Mobile · kein Website-Crawl
single-report-management-title = Management-Sicht
single-report-management-intro = Gesamtstatus, Hauptrisiken, wichtigste Maßnahmen und Hebel auf einen Blick.
single-report-management-eyebrow = MANAGEMENT-SUMMARY
single-report-wcag-occurrences = WCAG-Vorkommen
single-report-occurrences-definition = Vorkommen = einzelne betroffene Elemente, nicht die Anzahl unterschiedlicher Regeln — vollständige Aufschlüsselung im Anhang.
# $warnings_text and $manual_text are the count-heuristic-warnings and
# count-manual-points messages of report.ftl.
single-report-evidence-note = Gezählt sind hier ausschließlich bestätigte WCAG-Verstöße. Zusätzlich dokumentiert, aber nicht in den Score eingerechnet: { $warnings_text } und { $manual_text } auf dieser Seite zur manuellen Bestätigung. Weitere { $manual_only } WCAG-Kriterien lassen sich nur manuell prüfen (Anhang).
single-report-overall-verdict = Gesamturteil
single-report-audit-quality = Audit-Qualität
single-report-coverage = Prüfabdeckung
single-report-clean-scope = Diese Einschätzung bezieht sich ausschließlich auf den automatisierten Prüfumfang ({ $automated } von ca. { $total } testbaren WCAG-2.2-AA-Kriterien); Kriterien mit manuellem Prüfbedarf sind im Anhang aufgeführt.
single-report-quality-profile = Qualitätsprofil

## Indicator module dashboards
single-report-dashboard-strong = Was besonders gut funktioniert
single-report-dashboard-weak = Wo sich Optimierung lohnt
single-report-dashboard-good-but-gated = Die zusätzlichen Indikator-Module (UX, Journey, …) sind in gutem Zustand, dennoch zeigt dieser Report an anderer Stelle Barrieren mit Handlungsbedarf (Bedienbarkeit, mögliche Rechtsrelevanz) — siehe Management-Zusammenfassung.
single-report-dashboard-all-good = Die zusätzlichen Indikator-Module (UX, Journey, …) sind in gutem Zustand — kein Modul erfordert vorrangige Optimierung.

## Overall rating
single-report-rating-body = { $rating } — automatisierte Prüfung { $verdict }.
single-report-rating-provisional = Dieser Lauf ist nicht vollständig durchgelaufen; die Einstufung ist daher vorläufig und beschreibt nur den erfolgreich gemessenen Umfang.
single-report-rating-note = Die Einstufung ist ein Bandlabel für den Barrierefreiheitswert im automatisierten Prüfumfang, kein Konformitätsnachweis.

## Risks and strengths
single-report-risks-none = Keine prioritären Risiken im automatisierten Prüfumfang erkannt. Diese Einschätzung bezieht sich ausschließlich auf den automatisierten Prüfumfang ({ $automated } von ca. { $total } testbaren WCAG-2.2-AA-Kriterien); Kriterien mit manuellem Prüfbedarf sind im Anhang aufgeführt.
single-report-risks-title = Wichtigste Risiken
single-report-risks-title-count = Wichtigste Risiken ({ $count } erkannt)
single-report-strengths-title = Stärken im geprüften Umfang

## Key measures
single-report-measures-title = Die 5 wichtigsten Maßnahmen
single-report-measures-none = Keine dringenden Maßnahmen erforderlich.
single-report-measure-more = (+{ $count } weitere gleicher Art)

## Findings
single-report-findings-title = Klassifizierung der Befunde
single-report-findings-intro = Alle technischen Befunde, getrennt nach systemischen Komponentenfehlern und Einzelfällen.
single-report-findings-eyebrow = BEFUNDE
single-report-strip-systemic-mandatory = Systemisch (Pflicht)
single-report-strip-systemic-optimization = Systemisch (Optimierung)
single-report-strip-local-mandatory = Lokal (Pflicht)
single-report-strip-local-optimization = Lokal (Optimierung)
single-report-matrix-title = Befundmatrix
single-report-matrix-intro = Eine Zeile je Regelgruppe, sortiert nach Priorität – als Überblick zur Priorisierung, bevor die Detailkarten unten jede Zeile vertiefen.
single-report-matrix-col-finding = Befund
single-report-matrix-systemic = Systemisch · { $count }
single-report-matrix-local = Lokal · { $count }
# $more is the count-more-findings message of report.ftl.
single-report-matrix-more = In den Detailkarten unten aufgeführt: { $more }.
single-report-systemic-mandatory-title = Systemische Template- & Komponentenfehler (WCAG A/AA)
single-report-systemic-mandatory-intro = Dieses Muster deutet auf eine wiederverwendete Komponente oder Vorlage hin — bestätigt ist das erst mit Belegen von weiteren Seiten. Eine zentrale Behebung würde die Ursache voraussichtlich auch auf anderen Seiten mit demselben Muster beheben.
single-report-systemic-mandatory-eyebrow = SYSTEMISCHE PFLICHT
single-report-systemic-optimization-title = Systemische Qualitäts- & SEO-Optimierungen
single-report-systemic-optimization-intro = Diese Empfehlungen wiederholen sich innerhalb dieser einen Seite, was auf eine gemeinsame Vorlage hindeutet — bestätigt ist das erst nach Prüfung weiterer Seiten.
single-report-systemic-optimization-eyebrow = SYSTEMISCHE OPTIMIERUNG
single-report-local-mandatory-title = Einzelfälle & Redaktionelle Befunde (WCAG A/AA)
single-report-local-mandatory-intro = Punktuelle Barrieren, die nur einzelne Seiten, spezifische Bilder oder redaktionelle Texte betreffen und meist individuell behoben werden müssen.
single-report-local-mandatory-eyebrow = LOKALE PFLICHT
single-report-local-optimization-title = Ergänzende Qualitäts- & SEO-Empfehlungen
single-report-local-optimization-intro = Ergänzende Empfehlungen zur Verbesserung der Ladezeiten, Suchmaschinenoptimierung und Benutzerfreundlichkeit auf bestimmten Seiten.
single-report-local-optimization-eyebrow = LOKALE OPTIMIERUNG

## Appendix
single-report-appendix-title = Anhang & Methodik
single-report-appendix-intro = Prüfumfang, Methodik, WCAG-Coverage und die vollständige Liste der Vorkommen.
single-report-appendix-elements-affected = { $count ->
        [one] { $count } Element betroffen
       *[other] { $count } Elemente betroffen
    }
single-report-appendix-selectors = Selektoren: { $list }
single-report-appendix-eyebrow = ANHANG
single-report-manual-only-title = Strukturell nur manuell prüfbare Kriterien
single-report-manual-only-focus-order = 2.4.3 · Fokusreihenfolge
single-report-manual-only-focus-order-text = Ob die technische Fokusreihenfolge inhaltlich sinnvoll ist, hängt vom Seitenaufbau ab und lässt sich nicht automatisiert bewerten.
single-report-manual-only-error-identification = 3.3.1 · Fehlerkennzeichnung
single-report-manual-only-error-identification-text = Ob eine Fehlermeldung für Menschen tatsächlich verständlich ist, ist eine inhaltliche Frage und kein struktureller Check.
single-report-manual-only-reflow = 1.4.4 / 1.4.10 · Textgröße & Reflow
single-report-manual-only-reflow-text = Ob die Seite bei 400 % Zoom praktisch bedienbar bleibt, geht über die technische Reflow-Struktur hinaus und erfordert eine manuelle Prüfung.
single-report-manual-only-non-text = 1.1.1 · Nicht-Text-Inhalte
single-report-manual-only-non-text-text = Ob ein Alternativtext den Bildinhalt korrekt beschreibt, kann nur ein Mensch beurteilen; automatisiert prüfbar ist nur, ob überhaupt ein Alternativtext vorhanden ist.
single-report-manual-only-media = 1.2.1 / 1.2.2 · Zeitbasierte Medien
single-report-manual-only-media-text = Ob eine Untertitel- oder Transkript-Zusammenfassung den Videoinhalt inhaltlich korrekt wiedergibt, lässt sich nicht automatisiert prüfen.
single-report-manual-review = Manuelle Prüfung
single-report-manual-review-fallback = Dieses Kriterium manuell an der gerenderten Seite prüfen.
single-report-warning = Hinweis
single-report-warning-fallback = Dieses heuristische Signal manuell bestätigen.
single-report-manual-and-heuristic-title = Manuelle Prüfpunkte und heuristische Hinweise
single-report-journey-coverage = Interaktive Abdeckung: { $completed } von { $attempted } versuchten Journeys abgeschlossen; { $failed } fehlgeschlagen, { $skipped } übersprungen{ $budget }.
single-report-journey-budget-exhausted = ; Budget ausgeschöpft
single-report-findings-list-title = Alle Vorkommen
single-report-findings-list-intro = Rohdaten des Audits und die vollständige Liste aller erkannten Verstöße.
single-report-json-note = Der begleitende JSON-Report enthält die vollständige maschinenlesbare Fehlerliste mit Selektoren, Vorkommen und allen Detaildaten für automatisierte Weiterverarbeitung.
single-report-json-note-title = Rohdaten & Weiterverarbeitung

## Recognized structural patterns
single-report-patterns-title = Erkannte Strukturmuster
single-report-pattern-strong = Stark
single-report-pattern-present = Vorhanden
single-report-patterns-panel-title = Im Barrierefreiheitsbaum erkannt, ohne Interaktionstest

## Dual viewport summary
single-report-dual-title = Dual-Viewport-Zusammenfassung
single-report-dual-desktop = Barrierefreiheit - Desktop
single-report-dual-mobile = Barrierefreiheit - Mobile
single-report-dual-overall = Barrierefreiheit - Gesamt
single-report-dual-weighted = 70/30 gewichtet
single-report-preview = Vorschau
single-report-preview-captured = Erfasst
single-report-screenshot-failed = Screenshot-Erfassung fehlgeschlagen: { $reason }
single-report-dual-occurrences = WCAG-Vorkommen vor der ansichtsübergreifenden Zusammenführung: { $desktop } auf Desktop und { $mobile } auf Mobile. Die normalisierte gemeinsame Befundliste enthält { $combined } Vorkommen; diese Anzahl dient Evidenz und Maßnahmenplanung, nicht als dritter Score.
single-report-dual-explanation = Jeder Barrierefreiheitswert bewertet Schwere und Vielfalt der automatisiert erkannten WCAG-Befunde seiner Ansicht; 100 bedeutet, dass im automatisierten Prüfumfang kein Problem erkannt wurde. Niedrigere Werte bedeuten höheren Handlungsdruck. { $occurrence_context }Der ausgewiesene Barrierefreiheits-Gesamtwert wird mit 70 % Mobile und 30 % Desktop berechnet.

## AI & Trust chapter
single-report-trust-title = KI & Vertrauen
single-report-trust-intro = Technische Aufbereitung, KI-Lesbarkeit und Content-Sichtbarkeit — ungewichtete Indikatoren aus technischen Signalen.
single-report-trust-eyebrow = VERTRAUEN
