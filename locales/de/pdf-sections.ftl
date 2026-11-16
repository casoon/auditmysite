# PDF annex and section helper texts (#936, slice 25): the EN 301 549 and BIK
# annexes (`output::pdf::{en301549, bik_guide}`), the search experience and
# budget sections (`output::pdf::detail_modules::overview`), the self-check
# instructions and the indicator suffix (`output::pdf::helpers`), the cover
# phrase of a failed or provisional run (`output::pdf::cover`) and the
# occurrences word of the appendix table (`output::pdf::appendix`). The clause
# and chapter titles and the disclaimers that `wcag::en301549` owns stay in
# Rust, as do the table headings and labels that `batch-report.ftl`,
# `diagnosis.ftl` and `report.ftl` already define.

pdf-sections-en301549-title = EN-301-549-Klauselzuordnung
pdf-sections-en301549-intro = Ordnet die automatisch erkannten WCAG-2.2-A/AA-Befunde dieses Audits den Klauselnummern von EN 301 549 V4.1.1 (2026-09), Kapitel 9 (Web) zu, mit einem Status je Klausel: Verstöße gefunden, keine Verstöße im automatisch geprüften Umfang, oder manuelle Prüfung erforderlich.
pdf-sections-en301549-col-level = Stufe
pdf-sections-en301549-no-violations = Keine Verstöße im automatisch geprüften Umfang
pdf-sections-en301549-out-of-standard = { $count } weitere(r) WCAG-Befund(e) außerhalb von WCAG 2.2 A/AA (Kriterien der Stufe AAA) haben in EN 301 549 V4.1.1 keine normative Klausel und werden in der Klausel-Tabelle oben nicht ausgewiesen.
pdf-sections-bik-title = BIK-für-Alle-Leitfadenzuordnung
pdf-sections-bik-intro = Ordnet die automatisch erkannten Befunde dieses Audits der Kapitelstruktur des redaktionellen Leitfadens "BIK für Alle" zu ({ $source }) — denselben Kapiteln, die Redaktionsteams bereits kennen: Bilder & Alt-Text, Linktext, Struktur, Leichte Sprache, PDF-Dokumente, Videos. Keine neuen Prüfungen; dies gruppiert an anderer Stelle in diesem Report bereits ausgewiesene Befunde um.
pdf-sections-bik-col-chapter = Kapitel
pdf-sections-bik-findings-title = Befunde nach Kapitel
pdf-sections-bik-clean-title = Geprüft, keine Befunde
pdf-sections-bik-not-checked = Nicht automatisch geprüft: { $chapters }: dieses Kapitel wird von diesem Tool aktuell nicht automatisch geprüft. Es ist in der Befunde-Tabelle oben nicht enthalten.
pdf-sections-search-title = Sichtbarkeit & Nutzerverständnis
pdf-sections-search-body = Dieser Gesamtwert verbindet klassisches technisches SEO mit Inhaltsverständlichkeit, Vertrauenssignalen, KI-Lesbarkeit, semantischer Struktur und mobiler Lesbarkeit. Das klassische SEO bleibt im nächsten Detailkapitel sichtbar.
pdf-sections-search-warnings = Was noch schief läuft
pdf-sections-search-col-weight = Gewicht
pdf-sections-search-col-meaning = Einordnung
pdf-sections-search-table-title = Wie sich der Wert zusammensetzt
pdf-sections-budget-js-size = JS-Größe
pdf-sections-budget-css-size = CSS-Größe
pdf-sections-budget-page-size = Seitengröße
pdf-sections-recheck-landmarks = Selbsttest: VoiceOver-Rotor (VO+U, macOS) oder NVDA-Elementliste (Einfg+F7) öffnen und auf „Landmarken" umschalten — anhören, ob die erwarteten Bereiche (Header, Navigation, Footer) tatsächlich angekündigt werden.
pdf-sections-recheck-headings-landmarks = Selbsttest: VoiceOver-Rotor (VO+U) oder NVDA-Elementliste (Einfg+F7) nutzen und sowohl „Überschriften" als auch „Landmarken" prüfen — abgleichen, ob Reihenfolge/Eindeutigkeit dem hier beschriebenen Befund entspricht.
pdf-sections-recheck-form-names = Selbsttest: mit VoiceOver (VO+Cmd+J springt zwischen Formularfeldern) oder NVDA (F springt zwischen Feldern) durch das Formular tabben und hören, ob jedes Feld einen Namen ansagt — nicht nur ein generisches „Textfeld".
pdf-sections-recheck-status-messages = Selbsttest: die Meldung auslösen (z. B. Formular mit ungültigem Wert absenden), während VoiceOver oder NVDA läuft, und prüfen, ob sie automatisch angesagt wird, ohne dass der Tastaturfokus dorthin springt.
pdf-sections-indicator-suffix = Indikator
pdf-sections-cover-fail = Im automatischen Prüfumfang nicht bestanden — Handlungsbedarf
pdf-sections-cover-provisional = vorläufig
pdf-sections-occurrences = Vorkommen
