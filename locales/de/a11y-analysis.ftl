# Texte der Analyseebene der Barrierefreiheit, #936: die Meldungen der
# Screenreader-Analyse (`screen_reader::analyzer`) und die Meldungen der
# erkannten Muster (`patterns::pattern_message_text`,
# `patterns::easy_language`). Die Analyse speichert kanonisches Englisch im
# JSON (`en = true`), das PDF ruft dieselben Funktionen mit der Laufsprache auf
# (#406). Welche Meldung gilt und die Erkennung (Stoppwörter der Seitensprache)
# bleiben in Rust.

## Screenreader-Meldungen: interaktive Namen
a11y-analysis-sr-non-descriptive = Interaktiver Name "{ $name }" ist ohne Kontext nicht aussagekräftig.
a11y-analysis-sr-duplicated-name = Zugänglicher Name "{ $name }" wiederholt "{ $half }" zweimal hintereinander. Ein Screenreader kündigt ihn als "{ $name }" an, was wie ein Fehler wirkt.
a11y-analysis-sr-icon-font = Link- oder Button-Name enthält Icon-Font-Zeichen (Unicode Private Use Area). Screenreader lesen diese als kryptische Zeichencodes vor.
a11y-analysis-sr-empty-interactive = Interaktive Elemente ohne zugänglichen Namen werden im Screenreader nicht verständlich angekündigt.
a11y-analysis-sr-empty-form-labels = Formularfelder ohne Label sind in der Screenreader-Formularliste nicht verständlich.

## Screenreader-Meldungen: Linkliste
a11y-analysis-sr-link-empty = Leerer Linktext
a11y-analysis-sr-link-context-dependent = Kontextabhängiger Linktext
a11y-analysis-sr-duplicate-links = { $quality } kommt { $count } mal vor und erschwert die Linkliste im Screenreader.

## Screenreader-Meldungen: Lesereihenfolge und Navigation
a11y-analysis-sr-desert = Langer Abschnitt ab Sequenzposition { $start } ohne Landmark, Überschrift oder Fokusziel ({ $entries } Einträge).
a11y-analysis-sr-tab-stops = { $count } Tab-Stops ohne erkennbaren Skip-Link erschweren Tastatur- und Screenreader-Navigation.

## Screenreader-Meldungen: Überschriften
a11y-analysis-sr-heading-level-unknown = einer unbekannten Ebene
a11y-analysis-sr-heading-skipped = Überschriftenebene wird übersprungen: { $level } an Sequenzposition { $seq }.
a11y-analysis-sr-heading-first-not-h1 = Erste Überschrift im Dokument ist H{ $level } statt H1 (Sequenzposition { $seq }).
a11y-analysis-sr-heading-h1-late = H1 erscheint erst an Sequenzposition { $seq } — nach einer H2/H3-Überschrift.

## Screenreader-Meldungen: Landmarks
a11y-analysis-sr-landmarks-unlabeled-duplicate = Mehrere gleichartige Landmarken ohne Namen sind in der Screenreader-Landmarkliste nicht unterscheidbar.
a11y-analysis-sr-landmark-main-missing = Kein Main-Landmark in der Screenreader-Landmarkliste erkennbar.
a11y-analysis-sr-landmark-missing-banner = Kein Header-Bereich (banner-Landmark) vorhanden. Screenreader-Nutzer erreichen den Seitenkopf nicht über die Landmark-Navigation; im Lesemodus bleibt sein Inhalt erreichbar.
a11y-analysis-sr-landmark-missing-navigation = Keine Navigations-Landmark vorhanden. Screenreader-Nutzer können nicht über die Landmark-Navigation zur Navigation springen; im Lesemodus und per Tab bleiben die Links erreichbar.
a11y-analysis-sr-landmark-missing-contentinfo = Keine Fußzeilen-Landmark (contentinfo) vorhanden. Screenreader-Nutzer erreichen die Fußzeile nicht über die Landmark-Navigation; im Lesemodus bleibt ihr Inhalt erreichbar.
a11y-analysis-sr-navigation-behind-disclosure = Keine Navigations-Landmark im erfassten Accessibility-Tree vorhanden, aber ein Ausklapp-/Menü-Element wurde erkannt — die Navigation ist vermutlich eingeklappt statt fehlend. Prüfen, ob sie nach dem Ausklappen erreichbar ist.

## Mustermeldungen: gezählte Nomen
a11y-analysis-pattern-nav-landmark-count =
    { $count ->
        [one] 1 Navigationsbereich
       *[other] { $count } Navigationsbereiche
    }
a11y-analysis-pattern-landmark-count =
    { $count ->
        [one] 1 Navigationsbereich
       *[other] { $count } Navigationsbereiche
    }
a11y-analysis-pattern-native-disclosure-count =
    { $count ->
        [one] 1 natives Aufklappelement
       *[other] { $count } native Aufklappelemente
    }
a11y-analysis-pattern-disclosure-count =
    { $count ->
        [one] 1 Aufklapp-Auslöser
       *[other] { $count } Aufklapp-Auslöser
    }
a11y-analysis-pattern-dialog-count =
    { $count ->
        [one] 1 Dialog
       *[other] { $count } Dialoge
    }
a11y-analysis-pattern-tablist-count =
    { $count ->
        [one] 1 Tab-Liste
       *[other] { $count } Tab-Listen
    }
a11y-analysis-pattern-tab-count =
    { $count ->
        [one] 1 Tab
       *[other] { $count } Tabs
    }
a11y-analysis-pattern-accordion-count =
    { $count ->
        [one] 1 Accordion-Auslöser
       *[other] { $count } Accordion-Auslöser
    }
a11y-analysis-pattern-field-count =
    { $count ->
        [one] 1 Pflichtfeld
       *[other] { $count } Pflichtfelder
    }

## Mustermeldungen
a11y-analysis-pattern-navigation-unnamed = { $landmarks } gefunden, davon { $unnamed } ohne zugänglichen Namen. Screenreader-Nutzende können sie nicht unterscheiden.
a11y-analysis-pattern-navigation-without-links = { $landmarks } gefunden, aber keiner enthält Links. Die Struktur ist unvollständig.
a11y-analysis-pattern-navigation = Semantische Hauptnavigation erkannt ({ $landmarks } mit nativen Links). Gute Grundlage für Tastatur- und Screenreader-Nutzung.
a11y-analysis-pattern-disclosure-native = { $native } (<details>/<summary>) – die Semantik kommt aus dem Element, ARIA ist nicht nötig.
a11y-analysis-pattern-disclosure-complete = { $triggers } mit aria-expanded und aria-controls – vollständig ausgezeichnet.
a11y-analysis-pattern-disclosure-partial = { $triggers } mit aria-expanded ({ $controls } mit aria-controls). Die Verknüpfung über aria-controls verbessert die Screenreader-Ansage.
a11y-analysis-pattern-disclosure-native-extra =
    { $count ->
        [one] Dazu 1 natives Element mit <details>/<summary>.
       *[other] Dazu { $count } native Elemente mit <details>/<summary>.
    }
a11y-analysis-pattern-modal-dialog = { $dialogs } erkannt, davon { $well_formed } strukturell vollständig (zugänglicher Name, aria-modal, fokussierbarer Inhalt).
a11y-analysis-pattern-tab-list = { $tablists } mit { $tabs }; { $selected } mit aria-selected, { $controls } mit aria-controls.
a11y-analysis-pattern-skip-link-first = Sprunglink erkannt und korrekt als erstes fokussierbares Element platziert („{ $name }“).
a11y-analysis-pattern-skip-link-not-first = Sprunglink erkannt („{ $name }“), aber nicht als erstes fokussierbares Element – Tastaturnutzende müssen eventuell erst an anderen Elementen vorbei.
a11y-analysis-pattern-accordion = { $triggers }; { $controls } mit aria-controls; { $non_button } ohne Button-Rolle.
a11y-analysis-pattern-form = { $fields } erkannt; Absenden über: { $trigger }
a11y-analysis-pattern-add-to-cart = Warenkorb-Button erkannt: { $trigger }
a11y-analysis-pattern-quantity-stepper = Mengenauswahl (Spinbutton) erkannt: { $name }
a11y-analysis-pattern-unnamed-button = (Button ohne Namen)
a11y-analysis-pattern-unnamed = (ohne Namen)
a11y-analysis-pattern-easy-language = Die Seite bietet sich selbst in Leichter Sprache an oder verlinkt auf eine entsprechende Version.
