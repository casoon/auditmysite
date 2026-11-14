# Texts of the single PDF's frame (`output::pdf` mod.rs, `pdf-frame-*`), #936:
# the cover labels, the table of contents title, the three part intros with
# their audience titles and bodies, the confidence notes with the audit flag
# titles and customer texts, and the side checks that got no answer. Which
# message applies, the cover band phrase and scope line (messages of the
# modules that own them) and the joining of the side check names stay in
# Rust. The titles of the audit flags consent banner, skip link verification,
# conflicting signal, desktop/mobile difference, consent wall artifact and
# the fallback "Audit note" are the `batch-management-flag-*` messages of
# batch-consistency.ftl.

## Cover
pdf-frame-cover-score-label = BARRIEREFREIHEIT · AUTOMATISIERTER PRÜFUMFANG
pdf-frame-cover-findings-label = WCAG-VORKOMMEN
pdf-frame-cover-modules-label = MODULE IM GESAMTWERT · 0–100 IM AUTOMATISIERTEN PRÜFUMFANG · HÖHER IST BESSER
pdf-frame-cover-critical-label = kritisch/hoch
pdf-frame-cover-no-critical = 0 kritisch/hoch
pdf-frame-cover-indicators-label = ABGELEITETE INDIKATOREN · NICHT IM GESAMTWERT

## Table of contents
pdf-frame-toc-title = Inhalt

## Part dividers
pdf-frame-part-audience-title = Zielgruppe
pdf-frame-part1-title = Befunde nach Ursache
pdf-frame-part1-intro = Welche Ursachen die meisten Befunde erklären – und der priorisierte Plan, sie zu beheben.
pdf-frame-part1-audience = Inhaber, Entscheider und Entwickler. Dieser Teil erklärt, was die Befunde verursacht und was als Nächstes zu tun ist.
pdf-frame-part2-title = Technische Details für Entwickler
pdf-frame-part2-intro = Detaillierte WCAG-Verstöße, HTML-Evidenzen und Code-Beispiele, gruppiert nach systemischen Komponenten vs. Einzelfällen.
pdf-frame-part2-audience = Entwickler und QA-Teams. Dieser Teil liefert konkrete Code-Befunde zur Behebung aller Barrieren.
pdf-frame-part3-title = Qualitäts-Analysen
pdf-frame-part3-intro = Suchmaschinenoptimierung, Ladezeiten, mobiles Layout und Sicherheits-Metriken.
pdf-frame-part3-audience = Marketing, SEO-Spezialisten und Product Owner. Dieser Teil ergänzt die Barrierefreiheit um Qualitäts-Indikatoren.

## Confidence notes
pdf-frame-confidence-title = Hinweise zur Aussagekraft
pdf-frame-confidence-base = Automatisierte Befunde sind belastbar für erkennbare Muster, aber kontextabhängige WCAG-Kriterien, semantische Inhaltsqualität und Nutzerwege benötigen weiterhin manuelle Prüfung. Indikator-Module wie KI-Sichtbarkeit, Content Visibility und UX sind Hinweiswerte, keine Garantien.
pdf-frame-confidence-caveats-title = Audit-Hinweise

## Audit flag titles
# Aligned with the "Coverage" framing on the management summary page: a normal
# stability-budget limitation must not read as "the audit is broken".
pdf-frame-flag-title-incomplete-audit = Eingeschränkte Prüfabdeckung
pdf-frame-flag-title-side-check-unanswered = Zusatzprüfung ohne Antwort

## Audit flag customer texts
pdf-frame-flag-text-consent-banner = Ein Consent-Banner wurde erkannt. Teile der Seite können ohne Zustimmung verborgen oder nicht vollständig messbar gewesen sein.
pdf-frame-flag-text-bypass-blocks-untested = Ein Skip-Link ist vorhanden, der Nutzungstest deutet aber darauf hin, dass der Tastaturfokus sein Ziel nicht zuverlässig erreicht.
pdf-frame-flag-text-conflicting-signal = Zwei Prüfungen melden unterschiedliche Signale. Dieses Ergebnis sollte als Prüfhilfe statt als endgültige Aussage gelesen werden.
pdf-frame-flag-text-viewport-gap = Desktop- und Mobile-Ergebnis unterscheiden sich deutlich. Die Nutzung sollte für beide Ansichten getrennt geprüft werden.
# The same sentence as the management summary's partial-run callout: one
# wording for the same limitation.
pdf-frame-flag-text-incomplete-audit = Einzelne Messungen haben ein Stabilitäts-/Wiederholungsbudget erreicht und sind daher nicht in den ausgewiesenen Scores enthalten; diese beschreiben den erfolgreich gemessenen Umfang.

## Side checks without an answer
pdf-frame-side-check-www = www-/non-www-Konsolidierung (die andere Host-Variante antwortete nicht)
pdf-frame-side-check-certificate = Zertifikatsdetails (vom Browser nicht gemeldet)
pdf-frame-side-check-other = eine Zusatzprüfung
pdf-frame-side-check-text = Kein Ergebnis für: { $list }. Die geprüfte Seite wurde vollständig gemessen; die Einstufung ist deshalb nicht vorläufig.
