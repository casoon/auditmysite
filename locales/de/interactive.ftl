# Texte der interaktiven (Journey-)Befunde
# (`audit::interactive_finding::interactive_finding_text`, #936): je
# Befundart eine Meldung und eine `-fix`-Meldung für den Behebungsvorschlag,
# dazu der Unsicherheitshinweis (`finding_uncertainty_text`). Welche Art auf
# welche Meldung führt, die Standardwerte, der zitierte Seitentitel und die
# Kürzungsmarke der Beispielliste bleiben in Rust.

interactive-hidden-focusable-aria-hidden = Der Tastaturfokus landet auf einem Element innerhalb eines aria-hidden-Bereichs ({ $selector }). Screenreader-Nutzer erreichen ein Element, das im Accessibility Tree verborgen ist.
interactive-hidden-focusable-aria-hidden-fix = Element aus dem aria-hidden-Bereich entfernen oder tabindex="-1" darauf setzen.
interactive-hidden-focusable-inert = Der Tastaturfokus landet auf einem Element innerhalb eines inert-Bereichs ({ $selector }). Inert-Bereiche sollten per Tastatur nicht erreichbar sein.
interactive-hidden-focusable-inert-fix = Element aus dem inert-Bereich entfernen oder die tabindex-/Fokuskette korrigieren.
interactive-hidden-focusable-style = Der Tastaturfokus landet auf einem visuell versteckten Element ({ $selector }: display:none, visibility:hidden oder opacity:0). Tastaturnutzer verlieren die Orientierung.
interactive-hidden-focusable-style-fix = Element aus der Tab-Reihenfolge entfernen (tabindex="-1") oder es sichtbar machen, bevor es fokussiert wird.
interactive-focus-indicator-not-detected = Element ({ $selector }) zeigt im fokussierten Zustand keinen sichtbaren Fokusindikator (kein Outline, kein Box-Shadow, keine Rahmenänderung). Tastaturnutzer verlieren die Orientierung.
interactive-focus-indicator-not-detected-fix = Eine CSS-:focus-visible-Regel mit deutlichem Outline, Box-Shadow oder Rahmenwechsel gegenüber dem unfokussierten Zustand ergänzen.
interactive-tab-order-backward-jumps =
    Die Tab-Reihenfolge weicht von der DOM-Reihenfolge ab: { $count } rückwärtige { $count ->
        [one] Sprung
       *[other] Sprünge
    } beobachtet. Zuerst betroffene Elemente: { $examples }{ $suffix }. Tastaturnutzer können dem Lesefluss möglicherweise nicht folgen.
interactive-tab-order-backward-jumps-fix = Negative oder hohe tabindex-Werte vermeiden. Lese-/DOM-Reihenfolge an die visuelle Reihenfolge angleichen.
interactive-focus-trap-not-entered = Nach dem Öffnen des Modals wechselt der Fokus nicht in den Dialog. Tastaturnutzer können nicht damit interagieren.
interactive-focus-trap-not-entered-fix = Beim Öffnen den Fokus auf das erste fokussierbare Element im Dialog setzen oder auf den Dialog selbst (tabindex="-1").
interactive-focus-trap-background-not-hidden = Der Hintergrundinhalt ist bei geöffnetem Modal nicht vor assistiven Technologien verborgen.
interactive-focus-trap-background-not-hidden-fix = aria-hidden="true" auf dem Anwendungs-Root setzen, solange das Modal geöffnet ist, oder das inert-Attribut verwenden.
interactive-focus-trap-escaped = Der Fokus ist nicht im Modal-Dialog eingeschlossen. Tastaturnutzer können zum Hintergrundinhalt navigieren.
interactive-focus-trap-escaped-fix = Tab und Umschalt+Tab im Dialog abfangen, sodass der Fokus nur zwischen den Dialog-Kindelementen wechselt.
interactive-focus-trap-escape-not-closing = Die Escape-Taste schließt das Modal nicht. Tastaturnutzer können es nicht schließen.
interactive-focus-trap-escape-not-closing-fix = Einen keydown-Handler auf dem Dialog oder Dokument ergänzen, der bei Escape close() aufruft oder den Dialog verbirgt.
interactive-focus-restoration-lost-to-body = Nach dem Schließen des Modals kehrt der Fokus zu body statt zum Auslöser-Element zurück. Tastaturnutzer verlieren ihre Position auf der Seite.
interactive-focus-restoration-lost-to-body-fix = Vor dem Öffnen des Dialogs eine Referenz auf das Auslöser-Element speichern und beim Schließen trigger.focus() aufrufen.
interactive-modal-not-opened = Der Auslöser kündigt über aria-haspopup einen Dialog an, beim Betätigen erscheint im Accessibility-Tree aber keiner.
interactive-modal-not-opened-fix = Entweder beim Betätigen einen Dialog öffnen, oder aria-haspopup="dialog" am Auslöser entfernen, damit er nichts ankündigt, was er nicht tut.
interactive-menu-not-opened = Der Menü-Auslöser wurde geklickt, aber das Menü öffnet sich nicht. Tastaturnutzer erreichen die Menüpunkte nicht.
interactive-menu-not-opened-fix = aria-expanded="true" auf dem Auslöser setzen und die Menüpunkte sichtbar machen, sobald der Auslöser aktiviert wird.
interactive-menu-focus-not-moved = Nach dem Öffnen des Menüs wechselt der Fokus nicht zu den Menüpunkten. Tastaturnutzer bemerken das geöffnete Menü möglicherweise nicht.
interactive-menu-focus-not-moved-fix = Den Fokus nach dem Öffnen auf den ersten Menüpunkt setzen.
interactive-menu-escape-not-closing = Die Escape-Taste schließt das Menü nicht.
interactive-menu-escape-not-closing-fix = Einen keydown-Handler ergänzen, der das Menü bei Escape schließt und den Fokus zum Auslöser zurückgibt.
interactive-tabs-selection-not-moved = Die Pfeiltasten-Navigation verschiebt die Auswahl nicht zwischen den Tabs. Tastaturnutzer können die Tab-Liste nicht bedienen.
interactive-tabs-selection-not-moved-fix = Das Roving-Tabindex-Muster implementieren: Pfeil-rechts verschiebt Fokus und aria-selected zum nächsten Tab.
interactive-tabs-focus-not-on-tab = Nach Pfeil-rechts in der Tab-Liste steht der Fokus nicht auf einem Tab-Element.
interactive-tabs-focus-not-on-tab-fix = Sicherstellen, dass die Pfeiltasten-Navigation im Roving-Tabindex-Muster auch den Fokus (nicht nur die Auswahl) zum nächsten Tab verschiebt.
interactive-disclosure-not-opened = Das Betätigen des Disclosure-Auslösers klappt ihn nicht auf: weder sein eigener Aufklappzustand noch der Accessibility-Tree ändern sich.
interactive-disclosure-not-opened-fix = aria-expanded="true|false" am betätigten Element umschalten und den gesteuerten Bereich einblenden.
interactive-disclosure-not-closed = Das Betätigen des Disclosure-Auslösers klappt ihn nicht zu: weder sein eigener Aufklappzustand noch der Accessibility-Tree ändern sich.
interactive-disclosure-not-closed-fix = Den Handler aria-expanded in beide Richtungen umschalten lassen, nicht nur auf.
interactive-disclosure-state-without-content = Der Disclosure-Auslöser meldet seinen neuen Aufklappzustand, im Accessibility-Tree kommt aber nichts hinzu und verschwindet nichts. Screenreader-Nutzende hören den Zustandswechsel und finden nichts zu lesen.
interactive-disclosure-state-without-content-fix = Prüfen, ob der gesteuerte Bereich tatsächlich in den Accessibility-Tree kommt — display:none, hidden oder aria-hidden am Bereich halten ihn draußen.
interactive-disclosure-content-without-state = Beim Betätigen ändert sich der Accessibility-Tree, der Auslöser selbst meldet aber keinen Aufklappzustand. Screenreader-Nutzende erfahren nicht, dass sich etwas geöffnet oder geschlossen hat.
interactive-disclosure-content-without-state-fix = aria-expanded an das tatsächlich betätigte Element setzen und zusammen mit dem Bereich umschalten.
interactive-spa-no-announcement-signal = Nach der SPA-Navigation hat sich weder der Seitentitel (vorher: { $title }) noch die H1-Überschrift geändert, und der Fokus blieb an derselben Stelle. Screenreader kündigen den neuen Inhalt nicht an.
interactive-spa-no-announcement-signal-fix = Nach jeder clientseitigen Navigation: (1) document.title aktualisieren, (2) den Fokus auf das <main>-Element oder die neue H1-Überschrift setzen, (3) alternativ eine aria-live-Region mit dem neuen Seitennamen befüllen.
interactive-spa-title-unchanged = Nach der SPA-Navigation bleibt document.title unverändert ({ $title }). Screenreader kündigen Seitenwechsel häufig primär über den Titel an.
interactive-spa-title-unchanged-fix = document.title nach jeder clientseitigen Navigation auf den neuen Seitennamen aktualisieren.
interactive-spa-focus-not-moved = Nach der SPA-Navigation wird der Fokus nicht in den neuen Hauptbereich verschoben. Tastaturnutzer müssen manuell zum neuen Inhalt navigieren.
interactive-spa-focus-not-moved-fix = Nach der Navigation den Fokus auf das <main>-Element oder die erste H1-Überschrift des neuen Inhalts setzen.
interactive-skip-link-focus-not-moved = Der Skip-Link ist vorhanden, verschiebt den Fokus aber nicht zum Ziel. Tastaturnutzer können die Navigation nicht überspringen.
interactive-skip-link-focus-not-moved-fix = Sicherstellen, dass das Skip-Link-Ziel tabindex="-1" besitzt und per Anker-Link fokussiert wird, oder explizit target.focus() nach der Navigation aufrufen.
interactive-form-error-silent-failure = Formularfehler werden nicht über eine Live-Region (role="alert" oder aria-live) angekündigt, und aria-invalid wird nicht gesetzt. Screenreader-Nutzer erhalten kein Feedback, wenn ein Pflichtfeld leer bleibt.
interactive-form-error-silent-failure-fix = Fehlermeldungen in einem role="alert"-Element ausgeben und aria-invalid="true" auf jedem ungültigen Feld setzen.
interactive-form-error-invalid-without-live-region = aria-invalid wird nach dem Absenden gesetzt, aber keine Live-Region (role="alert" oder aria-live) kündigt den Fehler an. Screenreader-Nutzer bemerken den Fehlerzustand nur, wenn sie gezielt zum Feld zurücknavigieren.
interactive-form-error-invalid-without-live-region-fix = Einen role="alert"-Container ergänzen, der die Fehlermeldung nach dem Absenden ausgibt.
interactive-form-error-unlinked-fields =
    { $count } { $count ->
        [one] Feld
       *[other] Felder
    } mit aria-invalid="true" sind nicht per aria-describedby oder aria-errormessage mit ihrer Fehlermeldung verknüpft. Screenreader-Nutzer hören den Fehlerzustand, können ihn aber nicht dem Feld zuordnen.
interactive-form-error-unlinked-fields-fix = aria-describedby="error-message-id" auf jedem Feld mit aria-invalid="true" ergänzen.
interactive-form-error-focus-not-managed = Nach dem Absenden wurde ein Fehler angekündigt, der Tastaturfokus blieb jedoch auf dem Dokument-Body, statt zum ersten ungültigen Feld oder einer Fehlerzusammenfassung zu wechseln. Screenreader- und Tastaturnutzer werden nicht zum Fehler geführt und müssen ihn manuell suchen.
interactive-form-error-focus-not-managed-fix = Fokus nach einem fehlgeschlagenen Absenden zum ersten ungültigen Feld oder zu einer Fehlerzusammenfassung (z. B. role="alert") bewegen.
interactive-form-error-live-region-late-insertion = Die den Fehler ankündigende Live-Region (role="alert" oder aria-live) war beim initialen Laden der Seite nicht vorhanden — sie wurde erst nach dem Absenden des Formulars ins DOM eingefügt. Screenreader registrieren Live-Regions beim initialen Aufbau des Accessibility Tree; manche Kombinationen aus Browser und Screenreader kündigen eine erst nachträglich eingefügte Live-Region nicht zuverlässig an.
interactive-form-error-live-region-late-insertion-fix = Den Live-Region-Container bereits im initialen Markup leer rendern (z. B. <div role="status"></div>) und nur den Textinhalt beim Absenden befüllen, statt den Container selbst erst nach der Interaktion einzufügen.
interactive-add-to-cart-no-status-announcement = Das Hinzufügen zum Warenkorb hat die Seite sichtbar verändert (z. B. einen Warenkorb-Zähler), aber keine Live-Region (role="status"/"alert" oder aria-live) hat dies angekündigt, und der Fokus ist nicht in einen Warenkorb-Dialog gewechselt. Screenreader-Nutzer erhalten keine Bestätigung, dass der Artikel hinzugefügt wurde.
interactive-add-to-cart-no-status-announcement-fix = Das Ergebnis über eine role="status"-Live-Region ankündigen (z. B. „Artikel zum Warenkorb hinzugefügt“), oder den Fokus beim Öffnen in den Warenkorb-Dialog verschieben.
interactive-add-to-cart-no-feedback-detected = Der Klick auf den Warenkorb-Button führte zu keiner erkennbaren Änderung: keine Live-Region-Ankündigung, kein Fokuswechsel in einen Dialog und keine erkennbare Warenkorb-Zähler-Aktualisierung. Eine manuelle Prüfung ist nötig, um zu bestätigen, ob die Aktion erfolgreich war und wie sie kommuniziert wird.
interactive-add-to-cart-no-feedback-detected-fix = Manuell prüfen, ob das Hinzufügen zum Warenkorb funktioniert und angekündigt wird (z. B. über eine role="status"-Live-Region oder einen fokusverwalteten Warenkorb-Dialog).
interactive-quantity-stepper-keyboard-inoperable = Das Mengenfeld ließ sich nicht per Tastatur bedienen: Es konnte entweder nicht fokussiert werden, oder sein Wert änderte sich nach Drücken von Pfeil-nach-oben nicht. Reine Tastaturnutzer können die Menge nicht anpassen.
interactive-quantity-stepper-keyboard-inoperable-fix = Sicherstellen, dass das Mengenfeld ein natives <input type="number"> oder ein vollständig tastaturbedienbares ARIA-Spinbutton ist (fokussierbar, reagiert auf Pfeil-hoch/-runter).
interactive-quantity-stepper-value-not-exposed = Pfeil-nach-oben hat den Mengenwert geändert, aber dieses benutzerdefinierte (nicht native) Spinbutton-Widget hat aria-valuenow nicht entsprechend aktualisiert. Screenreader-Nutzer hören keine Änderung, obwohl sich der Wert geändert hat.
interactive-quantity-stepper-value-not-exposed-fix = aria-valuenow (und idealerweise aria-valuetext) auf dem Spinbutton-Element bei jeder Wertänderung aktualisieren.
interactive-link-text-generic =
    { $count } { $count ->
        [one] Link trägt
       *[other] Links tragen
    } generischen oder wenig aussagekräftigen Text ({ $examples }). Ohne den umgebenden Kontext sind sie für Screenreader-Nutzer schwer unterscheidbar. Heuristische Warnung zu WCAG 2.4.4 — ob der Linkzweck aus dem Kontext hervorgeht, muss manuell geprüft werden.
interactive-link-text-generic-fix = Linktext so formulieren, dass er auch ohne den umgebenden Seitenkontext verständlich ist, z. B. 'Mehr über Barrierefreiheit erfahren' statt 'Mehr erfahren'.
interactive-link-text-duplicate =
    { $count } { $count ->
        [one] Linktext kommt
       *[other] Linktexte kommen
    } 3-mal oder häufiger auf der Seite vor: { $examples }. Verweisen sie auf unterschiedliche Ziele, können Screenreader-Nutzer sie nicht unterscheiden.
interactive-link-text-duplicate-fix = Wiederholte Linktexte durch eindeutige Formulierungen ersetzen oder den sichtbaren Text durch aria-label / aria-labelledby ergänzen.
interactive-heading-missing-h1 = Die Seite hat keine H1-Überschrift. Ohne H1 können Screenreader-Nutzer die Hauptstruktur der Seite nicht erkennen.
interactive-heading-missing-h1-fix = Genau eine H1-Überschrift pro Seite verwenden, die den Hauptinhalt beschreibt.
interactive-heading-multiple-h1 = { $count } H1-Überschriften gefunden. Das ist zulässig; es kann die Orientierung erschweren, wenn unklar ist, welche das Seitenthema benennt.
interactive-heading-multiple-h1-fix = Überschriftenstruktur prüfen; empfohlen ist eine H1 für das Seitenthema.
interactive-heading-level-skip = Die Heading-Hierarchie überspringt Ebenen ({ $examples }). Screenreader-Nutzer können die Seitenstruktur unter Umständen nicht zuverlässig erfassen.
interactive-heading-level-skip-fix = Heading-Ebenen nie überspringen. Nach H1 folgt H2, nach H2 folgt H3 und so weiter.
interactive-landmark-missing-main = Kein <main>-Landmark gefunden. Screenreader-Nutzer können nicht direkt zum Hauptinhalt springen.
interactive-landmark-missing-main-fix = Den Hauptinhalt in ein <main>-Element einbetten oder role="main" auf dem passenden Container setzen.
interactive-landmark-nav-without-labels = { $count } Navigations-Landmarks ohne eindeutige Beschriftung. Screenreader-Nutzer können nicht unterscheiden, welche Navigation welchen Bereich abdeckt.
interactive-landmark-nav-without-labels-fix = Jeden <nav>-Bereich mit aria-label beschriften, z. B. aria-label="Hauptnavigation" und aria-label="Footer-Navigation".
interactive-landmark-duplicate-unique = Die Landmark-Rolle "{ $role }" kommt { $count }-mal auf der Seite vor. Diese Rolle sollte nur einmal pro Seite vorkommen.
interactive-landmark-duplicate-unique-fix = Nur ein Element mit role="{ $role }" (oder dem entsprechenden HTML-Element) pro Seite verwenden.
interactive-media-controls-missing-name = Der native Video-Player ({ $selector }) ist per Tastatur erreichbar, hat aber keinen zugänglichen Namen (kein aria-label, aria-labelledby oder title). Screenreader-Nutzer können nicht erkennen, um welches Video es sich handelt.
interactive-media-controls-missing-name-fix = Ein aria-label-, aria-labelledby- oder title-Attribut mit einer Beschreibung des Videoinhalts zum video-Element hinzufügen.
interactive-media-controls-not-reachable =
    { $count } { $count ->
        [one] natives Video-Element
       *[other] native Video-Elemente
    } mit sichtbaren Steuerelementen waren beim Tastatur-Tab-Durchlauf nicht erreichbar: { $examples }{ $suffix }. Tastaturnutzer können die Wiedergabe möglicherweise nicht bedienen.
interactive-media-controls-not-reachable-fix = Prüfen, ob ein vorheriges Element den Fokus einfängt und ob das video-Element selbst nicht per tabindex="-1" aus der Tab-Reihenfolge entfernt wurde.

interactive-uncertainty-late-reaction = Unsicher: Die Seite kam { $waited_ms } ms nach dem Klick zur Ruhe; eine langsamere Reaktion wäre nicht erfasst worden.
