# German counterpart of `locales/en/pdf-analysis.ftl`: the texts of the search
# experience composite and of the PDF's analysis sections
# (`output::search_experience`, `output::pdf::root_causes`,
# `output::pdf::problem_profile`, `output::pdf::wcag_coverage`,
# `pdf-analysis-*`), #936. The same keys as the English file. The arguments
# `found`, `warnings_text`, `manual_text`, `more` and `recommendations` carry
# the count messages of report.ftl.

## Search experience
pdf-analysis-search-label = Sichtbarkeit & Nutzerverständnis
pdf-analysis-search-content-clarity = Inhaltsverständlichkeit
pdf-analysis-search-trust = Vertrauenssignale
pdf-analysis-search-structure = Struktur und Semantik
pdf-analysis-search-ai = KI-Lesbarkeit
pdf-analysis-search-mobile = Mobile Lesbarkeit
pdf-analysis-search-technical-seo-explanation = Title, Description, Canonical, Sitemap, strukturierte Daten und crawlbare Links.
pdf-analysis-search-content-estimated = Aus Seitentyp und Content-Tiefe abgeleitet.
pdf-analysis-search-structure-estimated = Aus Überschriften, strukturierten Daten und Seitenprofil abgeleitet.
pdf-analysis-search-ai-explanation = Lesbarkeit, Abschnittsqualität und kompakte Abschnittsstruktur für KI-gestützte Systeme (ungeprüfte Heuristik).
pdf-analysis-search-mobile-explanation = Mobile Nutzbarkeit, lesbarer Text und gut antippbare Bedienelemente.
pdf-analysis-search-warning-seo = Technisches SEO liegt bei { $score } / 100, aber Content-, Trust- oder KI-Signale senken die kundennahe Sichtbarkeit.
pdf-analysis-search-warning-images = { $words } Wörter bei { $images } Bildern: wichtige Aussagen können eher im Bild als im lesbaren HTML liegen.
pdf-analysis-search-warning-media = Das Seitenprofil ist medienlastig; Bildtext sollte durch lesbaren HTML-Text abgesichert werden.
pdf-analysis-search-warning-local = LocalBusiness- oder lokale Vertrauensdaten sind unvollständig oder nicht maschinenlesbar.
pdf-analysis-search-interpretation-gap = Technisch auffindbar, aber für Nutzer und KI-Systeme nicht gleich stark verständlich. Schwächere Bestandteile: { $names }.
pdf-analysis-search-interpretation-strong = Starke Sichtbarkeit und Nutzerverständnis: { $parts } passen zusammen.
pdf-analysis-search-interpretation-strong-technical = Starke technische Auffindbarkeit.
pdf-analysis-search-interpretation-solid = Solide Basis mit sichtbaren Lücken bei { $parts }.
pdf-analysis-search-interpretation-solid-technical = Solide technische Basis mit sichtbaren Lücken.
pdf-analysis-search-interpretation-limited = Eingeschränkte Sichtbarkeit und Nutzerverständnis: Die Seite ist nicht ausreichend { $parts }.
pdf-analysis-search-interpretation-limited-technical = Eingeschränkte technische Auffindbarkeit.
pdf-analysis-search-strong-part-technical = technische Auffindbarkeit
pdf-analysis-search-strong-part-content = Inhaltsverständlichkeit
pdf-analysis-search-strong-part-trust = Vertrauen
pdf-analysis-search-solid-part-content = Inhalt
pdf-analysis-search-solid-part-trust = Vertrauen
pdf-analysis-search-solid-part-ai = maschineller Lesbarkeit
pdf-analysis-search-limited-part-content = verständlich
pdf-analysis-search-limited-part-trust = vertrauensstark
pdf-analysis-search-limited-part-ai = extrahierbar
pdf-analysis-search-join-and-two = { $first } und { $second }
pdf-analysis-search-join-and-many = { $initial } und { $last }
pdf-analysis-search-join-or-two = { $first } oder { $second }
pdf-analysis-search-join-or-many = { $initial } oder { $last }

## Problem profile
pdf-analysis-profile-title = Problemprofil (Verteilung über die Module): { $kind }
pdf-analysis-profile-kind-isolated = punktuell
pdf-analysis-profile-kind-concentrated = konzentriert & systemisch
pdf-analysis-profile-kind-broad = breit & systemisch
pdf-analysis-profile-names-two = { $first } und { $second }
pdf-analysis-profile-names-many = { $initial } und { $last }
pdf-analysis-profile-structural = { $count ->
        [0] Einzelne lokale Optimierungen ohne strukturelles Problem.
        [one] Einzelne lokale Optimierungen; { $count } Befund mit großer Reichweite oder struktureller Änderung wird unter den wichtigsten Risiken genannt.
       *[other] Einzelne lokale Optimierungen; { $count } Befunde mit großer Reichweite oder struktureller Änderung werden unter den wichtigsten Risiken genannt.
    }
pdf-analysis-profile-isolated-good = Sehr guter technischer Zustand. { $tail }
pdf-analysis-profile-isolated-named = { $names } bestimmt den Handlungsbedarf. { $tail }
pdf-analysis-profile-isolated-weak = { $names } { $several ->
        [yes] liegen
       *[no] liegt
    } unter dem Bereich „gut“ und { $several ->
        [yes] bestimmen
       *[no] bestimmt
    } den Handlungsbedarf. Kein Bereich ist kritisch (unter { $critical } Punkten), daher kein systemisches Profil.
pdf-analysis-profile-concentrated-systemic = { $names } { $several ->
        [yes] bestimmen
       *[no] bestimmt
    } den Handlungsbedarf. Ein großer Teil der Accessibility-Befunde konzentriert sich auf wenige wiederkehrende Muster.
pdf-analysis-profile-concentrated-stable = { $names } { $several ->
        [yes] bestimmen
       *[no] bestimmt
    } den Handlungsbedarf bei ansonsten stabilen technischen Bereichen.
pdf-analysis-profile-broad-unnamed = Mehrere technische Bereiche sind gleichzeitig betroffen und benötigen strukturelle Korrekturen.
pdf-analysis-profile-broad-named = Mehrere technische Bereiche sind gleichzeitig betroffen. Besonders { $names } benötigen strukturelle Korrekturen.
pdf-analysis-profile-few = { $found } — zu wenige, um ein wiederkehrendes Muster abzuleiten.
pdf-analysis-profile-concentration = { $total } Vorkommen wurden erkannt. Rund { $pct } % davon entfallen auf nur { $top } Befunde. Besonders betroffen: { $title }.
pdf-analysis-profile-leverage = Die { $top } wichtigsten Ursachen erklären rund { $pct } % der { $total } erkannten Vorkommen — ihre Behebung hat die größte Hebelwirkung auf die folgende Liste.

## Root causes
pdf-analysis-causes-none = Es wurden keine Barrierefreiheits-Befunde erkannt, daher ist keine Ursachenanalyse erforderlich.
pdf-analysis-causes-none-evidence = Keine bestätigten WCAG-Verstöße, daher ist keine Ursachenanalyse erforderlich. Offen bleiben { $warnings_text } und { $manual_text } — dokumentiert in den eigenen Abschnitten, ohne Einfluss auf den Score.
pdf-analysis-causes-lead-in = Wo sich die Befunde ballen: Liegt ein Befund in einer wiederverwendeten Komponente oder einem Template, behebt ein Fix dort alle seine Vorkommen — ob das so ist, lässt sich an einer einzelnen Seite nicht erkennen.

## Action plan
pdf-analysis-roadmap-modules-title = Weitere Module
pdf-analysis-roadmap-modules-subtitle = Empfehlungen aus den Modulen Sicherheit, Mobile, Performance und SEO, die in den wichtigsten Maßnahmen genannt sind.
pdf-analysis-roadmap-empty-modules = Keine priorisierten Barrierefreiheits-Maßnahmen. Zu anderen Modulen (Performance, SEO, Sicherheit, Mobile): { $recommendations } — sie stehen in den jeweiligen Modulabschnitten und in den wichtigsten Maßnahmen oben.

## Analysis by cause (#1171)
pdf-analysis-eyebrow = ANALYSE
pdf-analysis-title = Analyse nach Ursache
pdf-analysis-subtitle = Jeder Befund einmal vollständig: Problem, Auswirkung und wie sie belegt ist, Maßnahme und Reichweite, gruppiert nach der Ursache, die die Prüfung beobachtet hat.
pdf-analysis-cause-markup = Wiederkehrendes Markup bei { $selector }: { $count } identische Elemente auf dieser Seite. Eine gemeinsame Komponente oder ein Template ist möglich; der Quellcode muss es bestätigen.
pdf-analysis-cause-none = Kein wiederkehrendes Elementmuster beobachtet; der Befund steht für sich.
pdf-analysis-card-key-reach = Reichweite
pdf-analysis-card-key-cause = Ursache und Sicherheit
pdf-analysis-card-key-problem = Problem
pdf-analysis-card-key-impact = Auswirkung
pdf-analysis-card-key-evidence = Belegstatus
pdf-analysis-card-key-fix = Maßnahme
pdf-analysis-card-key-priority = Priorität, Aufwand, Zuständigkeit
pdf-analysis-card-key-score-effect = Score-Effekt
pdf-analysis-card-key-finding = Befund-ID
pdf-analysis-card-key-retest = Prüfung nach dem Fix
pdf-analysis-retest-others = { $others ->
        [one] Das andere Element ebenso prüfen.
       *[other] Die anderen { $others } Elemente ebenso prüfen.
    }
pdf-analysis-retest-name = Den zugänglichen Namen des geänderten Elements im Accessibility-Inspektor des Browsers lesen: Er muss sagen, was das Element bewirkt.
pdf-analysis-retest-contrast = Den Kontrast des geänderten Texts neu messen (Browser-Inspektor, Farbwähler), auch im Hover-, Fokus- und Dunkelzustand, soweit die Seite sie hat.
pdf-analysis-retest-alt = Die Textalternative des geänderten Bildes im Accessibility-Inspektor prüfen; ein dekoratives Bild darf keinen Namen zeigen.
pdf-analysis-retest-headings = Die Überschriftengliederung der Seite öffnen (Accessibility-Inspektor oder Überschriftenliste) und prüfen, dass die Ebenen dem Inhalt folgen.
pdf-analysis-retest-landmarks = Die Landmarks der Seite im Accessibility-Inspektor auflisten und prüfen, dass jeder Bereich einmal vorkommt und bei mehreren einen Namen trägt.
pdf-analysis-retest-lang = Das lang-Attribut im Seitenquelltext prüfen: Es muss ein gültiger Sprachcode sein, der zum Inhalt passt.
pdf-analysis-retest-ids = Den Seitenquelltext nach der geänderten id durchsuchen: Sie darf nur einmal vorkommen, und Labels und ARIA-Verweise müssen weiter auf das gemeinte Element zeigen.
pdf-analysis-retest-keyboard = Das geänderte Element nur mit der Tastatur bedienen (Tab, Enter, Leertaste): Es muss erreichbar sein, einen sichtbaren Fokus zeigen und seine Aktion auslösen.
pdf-analysis-retest-aria = Das geänderte Element im Accessibility-Inspektor prüfen: Rolle, Zustände und verwiesene Elemente müssen dem entsprechen, was die Seite tut.
pdf-analysis-retest-structure = Die geänderte Tabelle oder Liste im Accessibility-Inspektor prüfen: Struktur, Kopfzellen und Einträge müssen wie gedacht angekündigt werden.
pdf-analysis-retest-forms = Name und Hinweise jedes geänderten Felds im Accessibility-Inspektor lesen und das Formular nur mit der Tastatur ausfüllen.
pdf-analysis-retest-layout = Den geänderten Bereich bei 200 % Zoom und in einem kleinen Viewport erneut prüfen: Nichts darf abgeschnitten sein oder sich überlagern, und Bedienelemente müssen groß genug zum Treffen bleiben.
pdf-analysis-retest-validator = Die Seite erneut durch den W3C-Validator schicken: Die Meldung für diese Stelle muss weg sein, und es darf keine neue entstehen.
pdf-analysis-retest-headers = Die Seite erneut abrufen (zum Beispiel mit curl -I) und prüfen, dass die Antwort den Header mit dem gewollten Wert trägt.
pdf-analysis-retest-seo = Den Seitenquelltext neu laden und das geänderte Tag oder den Wert prüfen, dann den Audit erneut laufen lassen und die SEO-Befunde vergleichen.
pdf-analysis-retest-generic = Den Audit für diese Seite erneut laufen lassen: Der Befund muss weg sein oder eine geringere Reichweite haben als zuvor.
pdf-analysis-card-priority-effort = Priorität { $priority } · Aufwand { $effort } · { $role }

## WCAG coverage
pdf-analysis-coverage-legend = Grün: auf dieser Seite geprüft und bestanden. Rot: Verstoß gefunden. Gelb: nicht bestanden oder prüfbedürftig. Grau: auf dieser Seite nichts zu prüfen.
pdf-analysis-coverage-principles-title = Prinzip-Abdeckung (ausgeführte Kriterien bestanden)
pdf-analysis-coverage-perceivable = Wahrnehmbar
pdf-analysis-coverage-operable = Bedienbar
pdf-analysis-coverage-understandable = Verständlich
pdf-analysis-coverage-robust = Robust
pdf-analysis-coverage-state-violation = { $count } mit Verstoß
pdf-analysis-coverage-state-review = { $count } zur manuellen Prüfung
pdf-analysis-coverage-state-not-testable = { $count } nicht prüfbar oder nicht ausgeführt
pdf-analysis-coverage-state-not-applicable = { $count } nicht anwendbar
pdf-analysis-coverage-scope-title = Prüfumfang
pdf-analysis-coverage-intro = Dieses Audit prüft { $automated } von ca. { $total } WCAG-2.2-AA-Kriterien automatisch. Die unten aufgeführten Kriterien benötigen manuelle Prüfung.
pdf-analysis-coverage-intro-new = Dieses Audit prüft { $automated } von ca. { $total } WCAG-2.2-AA-Kriterien automatisch. { $count } davon ({ $list }, unten und bei einzelnen Befunden mit „WCAG 2.2“ gekennzeichnet) sind mit WCAG 2.2 hinzugekommen und Teil von EN 301 549 V4.1.1, aber nicht von V3.2.1. Die unten aufgeführten Kriterien benötigen manuelle Prüfung.
pdf-analysis-coverage-automated-title = Automatisch geprüft ({ $listed })
pdf-analysis-coverage-listed-note = { $listed } Kriterien insgesamt: die { $automated } WCAG-2.2-A/AA-Kriterien, auf die sich die Quote oben bezieht, dazu { $aaa } AAA-Kriterien, die dieses Werkzeug ebenfalls prüft.
pdf-analysis-coverage-manual-title = Manuelle Prüfung erforderlich ({ $count })
pdf-analysis-coverage-how-title = So testen Sie manuell
pdf-analysis-coverage-test-keyboard-title = Tastaturnavigation
pdf-analysis-coverage-test-keyboard-text = Komplette Seite per Tab navigieren. Kein Fokus verloren, kein Keyboard-Trap, jedes interaktive Element erreichbar.
pdf-analysis-coverage-test-screen-reader-title = Screenreader
pdf-analysis-coverage-test-screen-reader-text = Test mit NVDA/JAWS (Windows) oder VoiceOver (Mac/iOS) — Landmark-Navigation und Formular-Interaktion.
pdf-analysis-coverage-test-zoom-title = 400% Zoom
pdf-analysis-coverage-test-zoom-text = Bei 400% Browser-Zoom: Seite ohne horizontales Scrollen bedienbar, kein Inhalt verloren.
pdf-analysis-coverage-test-motion-title = Reduced Motion
pdf-analysis-coverage-test-motion-text = Betriebssystem-Einstellung „Bewegung reduzieren“ aktivieren und prüfen, ob Animationen deaktiviert oder reduziert werden.
pdf-analysis-coverage-test-modal-title = Modal- / Dropdown-Interaktion
pdf-analysis-coverage-test-modal-text = Vollständige Tastaturbedienung: Tab, Enter, Space, Escape, Pfeiltasten. Fokus kehrt nach Schließen zum Trigger zurück.
pdf-analysis-coverage-test-color-title = Farbenblindheit
pdf-analysis-coverage-test-color-text = Mit einem Werkzeug wie „Color Oracle“ prüfen, ob Informationen nicht ausschließlich über Farbe vermittelt werden.
