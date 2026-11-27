# Texte der Audit-(Analyse-)Schicht, #936: die technischen Auswirkungen der
# SEO-Überschriftenprobleme (`audit::normalized::seo_heading_finding_text`),
# Bewertungsgrundlage, Bereichsnamen und Abweichungssätze zu 3.2.6
# (`audit::batch_consistency`) sowie Dimensionsnamen und Begründungen des
# Management-Risikos (`audit::management_risk`). Die Funktionen bleiben reine
# `*_text(kind, values, en)`-Textfunktionen: die Analyse speichert kanonisches
# Englisch im JSON (`en = true`), das PDF ruft sie mit der Laufsprache auf
# (#406). Welche Meldung gilt, bleibt in Rust. Die Titel der
# Überschriftenprobleme sind die Titel der `explanation-seo-headings-*`-
# Meldungen aus rules.ftl, in beiden Sprachen identisch.

## Heading issues: technical impact per issue type
audit-heading-impact-skipped-level = Übersprungene Heading-Ebenen lassen Lücken in der Gliederung, an der sich Screenreader-Nutzer und SEO-Crawler orientieren — ein Strukturhinweis, für sich kein WCAG-Verstoß; empfohlen ist eine logische Hierarchie H1→H2→H3.
audit-heading-impact-missing-h1 = Fehlende H1-Überschrift — Seitenzweck für Suchmaschinen und Screenreader nicht erkennbar.
audit-heading-impact-multiple-h1 = Mehrere H1-Überschriften sind zulässig, können die inhaltliche Hierarchie aber unklar machen, wenn nicht erkennbar ist, welche das Seitenthema benennt — ein Best-Practice-Hinweis, für sich kein WCAG-Verstoß.
audit-heading-impact-long-heading = Überlange Überschriften werden in SERPs abgeschnitten und erschweren das schnelle Scannen für Nutzer.
audit-heading-impact-empty-heading = Leere Überschriften erzeugen Navigationsprobleme für Screenreader und werden von SEO-Crawlern als schlechtes Signal gewertet.

## Shared
audit-join-and = { " und " }
audit-not-measured = nicht gemessen

## 3.2.6 Consistent Help: assessment basis, region names, deviations
audit-help-mechanisms =
    { $count ->
        [one] 1 wiederkehrender Hilfemechanismus
       *[other] { $count } wiederkehrende Hilfemechanismen
    }
audit-help-deviations =
    { $count ->
        [one] 1 Abweichung
       *[other] { $count } Abweichungen
    }
audit-help-pages =
    { $count ->
        [one] 1 Seite
       *[other] { $count } Seiten
    }
audit-help-basis-manual-review = Kein Hilfemechanismus (Kontakt- oder Hilfe-Link, E-Mail, Telefon, bekanntes Chat-Widget) wiederholt sich außerhalb des Hauptinhalts auf mindestens zwei geprüften Seiten; andere Formen von Hilfe sind manuell zu prüfen.
audit-help-basis-consistent = { $mechanisms } über die geprüften Seiten nach Seitenbereich und relativer Reihenfolge verglichen; keine Abweichung gefunden.
audit-help-basis-deviations = { $mechanisms } nach Seitenbereich und relativer Reihenfolge verglichen; { $deviations } auf { $pages }.
audit-help-region-header = Kopfbereich
audit-help-region-navigation = Navigation
audit-help-region-complementary = Seitenleiste
audit-help-region-footer = Fußbereich
audit-help-region-floating = schwebende Einblendung
audit-help-region-other = Seitenkörper außerhalb des Hauptinhalts
audit-help-deviation-region = „{ $mechanism }“ steht im Bereich { $found } statt wie auf den meisten Seiten im Bereich { $expected }.
audit-help-deviation-order = Im Bereich { $region } steht „{ $second }“ vor „{ $first }“; die meisten Seiten nutzen die umgekehrte Reihenfolge.

## Management risk: dimension names
audit-risk-dimension-legal = Rechtliches / BFSG-EAA
audit-risk-dimension-assistive = Nutzung mit Hilfsmitteln
audit-risk-dimension-conversion = Conversion & Nutzbarkeit
audit-risk-dimension-seo = SEO & Sichtbarkeit
audit-risk-dimension-performance-mobile = Ladezeit & Mobilfreundlichkeit
audit-risk-dimension-trust = Markenwirkung (aus Barrierefreiheit abgeleitet)
audit-risk-dimension-project = Projektrisiko

## Management risk: blocking controls
audit-controls-unnamed =
    { $count ->
        [one] 1 Bedienelement ohne zugänglichen Namen oder Rolle
       *[other] { $count } Bedienelemente ohne zugänglichen Namen oder Rolle
    }
audit-controls-keyboard =
    { $count ->
        [one] 1 Bedienelement nicht per Tastatur bedienbar
       *[other] { $count } Bedienelemente nicht per Tastatur bedienbar
    }
audit-controls-consequence-unnamed = Screenreader können deren Zweck nicht vermitteln; eine Tastaturblockade wurde an diesen Stellen nicht erkannt.
audit-controls-consequence-keyboard = Tastaturnutzende können sie nicht erreichen oder auslösen.
audit-controls-consequence-both = Screenreader können den Zweck der namenlosen nicht vermitteln; die übrigen sind per Tastatur nicht erreichbar oder auslösbar.

## Management risk: rationales
audit-risk-legal-base =
    { $count ->
        [one] 1 Befund
       *[other] { $count } Befunde
    } mit möglicher Rechtsrelevanz, { $critical } kritische und { $high } hohe WCAG-Befunde automatisiert erkannt.
audit-risk-legal-controls = { $base } Zusätzlich: { $controls } (WCAG 4.1.2/2.1.1, auf die auch BFSG/EAA verweisen).
audit-risk-assistive-none = Im automatisierten Prüfumfang keine blockierende Barriere für Screenreader- oder Tastaturnutzung erkannt.
audit-risk-assistive-findings = { $critical } kritische und { $high } hohe Befunde.
audit-risk-assistive-blocking = { $critical } kritische und { $high } hohe Befunde; { $controls }. { $consequence }
audit-risk-conversion-blocking = { $controls }: Abläufe, die darüber führen, sind mit Hilfsmitteln erschwert. Ob ein Ablauf vollständig blockiert ist, wurde nicht geprüft.
audit-risk-seo-findings =
    { $count ->
        [one] 1 SEO-Befund
       *[other] { $count } SEO-Befunde
    } mit mittlerem oder höherem Schweregrad (SEO-Score { $score }).
audit-risk-performance-mobile-none = Weder das Performance- noch das Mobile-Modul wurde ausgeführt.
audit-risk-performance-mobile-base = Performance { $performance }, mobile Nutzbarkeit { $mobile } (Labormessung).
audit-risk-performance-mobile-high =
    { $base } { $count ->
        [one] 1 Befund
       *[other] { $count } Befunde
    } mit hohem Schweregrad in diesen Modulen.
audit-risk-trust = Nur aus der Barrierefreiheit abgeleitet: Score { $accessibility }/100 mit { $critical } kritischen und { $high } hohen Befunden. Ein Vertrauens- oder Markensignal wird nicht direkt gemessen.
audit-risk-project-recurring =
    { $count ->
        [one] 1 Regel tritt
       *[other] { $count } Regeln treten
    } auf zwei oder mehr der { $pages } geprüften Seiten auf — vermutlich Komponenten- oder Template-Fehler, die gebündelt behoben werden können. { $severe ->
        [0] Keine davon verbindet hohen Schweregrad mit einer Reichweite auf mindestens die Hälfte der Seiten; Wiederholung allein macht daraus kein hohes Risiko.
        [one] 1 davon hat hohen Schweregrad und reicht auf mindestens die Hälfte der Seiten — das macht es zu einem hohen Risiko.
       *[other] { $severe } davon haben hohen Schweregrad und reichen auf mindestens die Hälfte der Seiten — das macht es zu einem hohen Risiko.
    }
audit-risk-project-single =
    { $count ->
        [one] 1 Befund betrifft 10 oder mehr Elemente dieser Seite oder erfordert eine strukturelle Änderung.
       *[other] { $count } Befunde betreffen 10 oder mehr Elemente dieser Seite oder erfordern eine strukturelle Änderung.
    } { $severe ->
        [0] Keiner davon hat hohen Schweregrad.
        [one] 1 davon hat hohen Schweregrad — das macht es zu einem hohen Risiko.
       *[other] { $severe } davon haben hohen Schweregrad — das macht es zu einem hohen Risiko.
    } Eine einzelne Seite zeigt nicht, ob Template oder Komponente die Ursache sind — dafür weitere Seiten prüfen.
