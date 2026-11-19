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
pdf-analysis-search-interpretation-strong = Starke Sichtbarkeit und Nutzerverständnis: technische Auffindbarkeit, Inhaltsverständlichkeit und Vertrauen passen zusammen.
pdf-analysis-search-interpretation-solid = Solide Basis mit sichtbaren Lücken bei Inhalt, Vertrauen oder maschineller Lesbarkeit.
pdf-analysis-search-interpretation-limited = Eingeschränkte Sichtbarkeit und Nutzerverständnis: Die Seite ist nicht ausreichend verständlich, vertrauensstark oder extrahierbar.

## Problem profile
pdf-analysis-profile-title = Problemprofil: { $kind }
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
pdf-analysis-causes-eyebrow = URSACHEN
pdf-analysis-causes-title = Ursachenanalyse
pdf-analysis-causes-subtitle = Viele Einzelbefunde gehen auf wenige wiederkehrende Ursachen zurück – deren Behebung wirkt gebündelt.
pdf-analysis-causes-none = Es wurden keine Barrierefreiheits-Befunde erkannt, daher ist keine Ursachenanalyse erforderlich.
pdf-analysis-causes-none-evidence = Keine bestätigten WCAG-Verstöße, daher ist keine Ursachenanalyse erforderlich. Offen bleiben { $warnings_text } und { $manual_text } — dokumentiert in den eigenen Abschnitten, ohne Einfluss auf den Score.
pdf-analysis-causes-lead-in = Wo sich die Befunde ballen: Liegt ein Befund in einer wiederverwendeten Komponente oder einem Template, behebt ein Fix dort alle seine Vorkommen — ob das so ist, lässt sich an einer einzelnen Seite nicht erkennen.
pdf-analysis-causes-table-title = Wo sich die Vorkommen ballen
pdf-analysis-causes-other = Sonstige ({ $more })

## Action plan
pdf-analysis-roadmap-eyebrow = MASSNAHMEN
pdf-analysis-roadmap-title = Maßnahmenplan
pdf-analysis-roadmap-subtitle = Empfohlene Maßnahmen, gruppiert nach Ebene des Problems — inklusive ergänzender SEO- und Qualitätsempfehlungen ohne Rechtsbezug.
pdf-analysis-roadmap-modules-title = Weitere Module
pdf-analysis-roadmap-modules-subtitle = Empfehlungen aus den Modulen Sicherheit, Mobile, Performance und SEO, die in den wichtigsten Maßnahmen genannt sind.
pdf-analysis-roadmap-empty-modules = Keine priorisierten Barrierefreiheits-Maßnahmen. Zu anderen Modulen (Performance, SEO, Sicherheit, Mobile): { $recommendations } — sie stehen in den jeweiligen Modulabschnitten und in den wichtigsten Maßnahmen oben.
pdf-analysis-roadmap-empty = Keine priorisierten Maßnahmen — keine Befunde mit Handlungsbedarf.
pdf-analysis-roadmap-leverage = Reichweite der Korrektur: { $leverage } — { $count ->
        [one] 1 Vorkommen
       *[other] { $count } Vorkommen
    } betroffen · Aufwand: { $effort }
pdf-analysis-roadmap-root-cause = → Ursache { $letter }, { $count } Vorkommen ({ $share } %)

## WCAG coverage
pdf-analysis-coverage-legend = Grün: auf dieser Seite geprüft und bestanden. Rot: Verstoß gefunden. Gelb: nicht bestanden oder prüfbedürftig. Grau: auf dieser Seite nichts zu prüfen.
pdf-analysis-coverage-principles-title = Prinzip-Abdeckung (ausgeführte Kriterien bestanden)
pdf-analysis-coverage-perceivable = Wahrnehmbar
pdf-analysis-coverage-operable = Bedienbar
pdf-analysis-coverage-understandable = Verständlich
pdf-analysis-coverage-robust = Robust
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
