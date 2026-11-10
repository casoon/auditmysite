# Rule catalogue texts: one message per rule id of `src/taxonomy/rules.rs`
# (`RULES`), in the order of that table. The message value is the title,
# the attributes carry the user impact and the technical impact.
# The key is `rule-` plus the rule id with `/` written as `-`
# (`taxonomy::rules::rule_text_key`).

## Content & alternatives

rule-image-alt = Fehlende Alternativtexte bei Bildern
    .user-impact = Screenreader-Nutzer erhalten keine Bildinformation.
    .technical-impact = Nicht-konforme Bildauszeichnung im Markup.

rule-images-alt-missing = Fehlende Alternativtexte bei Bildern
    .user-impact = Screenreader-Nutzer erhalten keine Bildinformation.
    .technical-impact = Nicht-konforme Bildauszeichnung im Markup.

rule-images-alt-suspicious = Vermutlich nichtssagender Alternativtext
    .user-impact = Screenreader-Nutzer hören einen Text, der den Bildinhalt nicht vermittelt.
    .technical-impact = alt-Attribut vorhanden, aber ohne beschreibenden Inhalt.

rule-video-caption = Fehlende Untertitel für Videos
    .user-impact = Gehörlose oder schwerhörige Nutzer können gesprochene Inhalte nicht wahrnehmen.
    .technical-impact = Fehlendes track-Element für Untertitel.

rule-media-alt = Fehlende Medienalternative (AAA)
    .user-impact = Nutzer, die keine Videos oder Audios abspielen können, erhalten keine Textalternative.
    .technical-impact = Fehlendes Transkript oder Audiobeschreibung.

## Structure & semantics

rule-order-visual-mismatch = Visuelle Reihenfolge weicht von der Vorlese-Reihenfolge ab
    .user-impact = Screenreader-Nutzer hören Inhalte in einer anderen Reihenfolge, als sie sehende Nutzer visuell wahrnehmen.
    .technical-impact = CSS `order` verändert die visuelle Reihenfolge, der DOM- und Accessibility-Baum bleiben unverändert.

rule-headings-section-without-heading = Fehlende Abschnittsüberschriften
    .user-impact = Nutzer können Inhalte nicht gezielt ansteuern.
    .technical-impact = Fehlende Sub-Headings in langen Inhaltsblöcken.

rule-orientation-root-rotated = Eingeschränkte Bildschirmausrichtung
    .user-impact = Nutzer mit fest montierten Bildschirmen (z. B. am Rollstuhl) können die Seite nicht lesen.
    .technical-impact = Berechnetes transform (Matrix) oder rotate an html oder body dreht um eine Vierteldrehung, gemessen im geprüften Viewport.

rule-orientation-content-hidden = Eingeschränkte Bildschirmausrichtung
    .user-impact = Nutzer mit fest montierten Bildschirmen (z. B. am Rollstuhl) können die Seite nicht lesen.
    .technical-impact = CSS-Medienabfragen blockieren die Ausrichtung.

## Navigation & operation

rule-keyboard-interactive-not-focusable = Inhalte nicht per Tastatur bedienbar
    .user-impact = Tastaturnutzer sind von Funktionen ausgeschlossen.
    .technical-impact = Click-Handler ohne Tastatur-Alternative.

rule-keyboard-trap = Tastaturfalle — Fokus kann nicht verlassen werden
    .user-impact = Seite wird faktisch unbenutzbar für Tastaturnutzer.
    .technical-impact = Fehlende Fokus-Verwaltung in modalen Elementen.

rule-headings-none = Fehlende Sprungnavigation
    .user-impact = Tastaturnutzer müssen bei jedem Seitenwechsel durch die Navigation tabben.
    .technical-impact = Fehlender Skip-Link, fehlende HTML5-Landmarks.

rule-document-title = Fehlender oder unzureichender Seitentitel
    .user-impact = Screenreader-Nutzer können die Seite nicht identifizieren.
    .technical-impact = Leeres oder generisches <title>-Tag.

rule-document-title-empty = Fehlender oder unzureichender Seitentitel
    .user-impact = Screenreader-Nutzer können die Seite nicht identifizieren.
    .technical-impact = Leeres oder generisches <title>-Tag.

rule-document-title-missing = Fehlender oder unzureichender Seitentitel
    .user-impact = Screenreader-Nutzer können die Seite nicht identifizieren.
    .technical-impact = Leeres oder generisches <title>-Tag.

rule-landmarks-not-unique = Landmarks nicht eindeutig benannt
    .user-impact = Screenreader-Nutzer können gleichartige Bereiche nicht unterscheiden.
    .technical-impact = Mehrfach verwendete Landmark-Rollen ohne eindeutige aria-label/aria-labelledby-Werte.

rule-dialog-focusable-missing = Unlogische Fokus-Reihenfolge
    .user-impact = Verwirrende Navigation für Tastaturnutzer.
    .technical-impact = DOM-Reihenfolge weicht von visueller Reihenfolge ab.

rule-links-generic-name = Unklare oder generische Linktexte
    .user-impact = Screenreader-Linklisten zeigen nichtssagende Texte.
    .technical-impact = Generische Link-Texte ohne Kontext.

rule-links-single-character-name = Unklare oder generische Linktexte
    .user-impact = Screenreader-Linklisten zeigen nichtssagende Texte.
    .technical-impact = Generische Link-Texte ohne Kontext.

rule-links-url-name = Unklare oder generische Linktexte
    .user-impact = Screenreader-Linklisten zeigen nichtssagende Texte.
    .technical-impact = Generische Link-Texte ohne Kontext.

rule-focus-visible = Fokus-Indikator nicht sichtbar
    .user-impact = Tastaturnutzer sehen nicht, welches Element fokussiert ist.
    .technical-impact = CSS outline:none ohne Alternative.

rule-focus-obscured = Fokussiertes Element vollständig verdeckt
    .user-impact = Tastaturnutzer springen per Tab auf ein Element, das durch einen Sticky-Header, -Footer, ein Cookie-Banner o. ä. vollständig verdeckt ist und daher nicht sichtbar ist.
    .technical-impact = Fokuslauf: Element fokussiert, mit scrollIntoView({"{"}block:'nearest'{"}"}) ins Bild gescrollt, 3×3 Punkte per elementsFromPoint abgetastet; alle Punkte unter einem position: fixed/sticky-Element.

rule-focus-obscured-enhanced = Fokussiertes Element teilweise verdeckt
    .user-impact = Tastaturnutzer springen per Tab auf ein Element, dessen sichtbarer Bereich durch einen Sticky-Header, -Footer, ein Cookie-Banner o. ä. teilweise verdeckt wird.
    .technical-impact = Fokuslauf wie bei focus/obscured; mindestens ein Abtastpunkt unter einem position: fixed/sticky-Element.

rule-label-in-name-mismatch = Sichtbarer Text ≠ zugänglicher Name
    .user-impact = Sprachsteuerungsnutzer können Element nicht ansprechen.
    .technical-impact = aria-label weicht vom sichtbaren Text ab.

rule-meta-refresh = Nicht anpassbare Zeitbegrenzung
    .user-impact = Nutzer benötigen mehr Zeit für die Eingabe und können durch Timeouts Daten verlieren.
    .technical-impact = Fehlendes Refresh- oder Timeout-Handling im Script/Markup.

rule-timing-meta-refresh = Nicht anpassbare Zeitbegrenzung
    .user-impact = Nutzer benötigen mehr Zeit für die Eingabe und können durch Timeouts Daten verlieren.
    .technical-impact = Fehlendes Refresh- oder Timeout-Handling im Script/Markup.

rule-motion-infinite-animation = Fehlende Pause-/Stopp-/Ausblenden-Kontrolle
    .user-impact = Nutzer mit Aufmerksamkeits- oder kognitiven Einschränkungen sowie Screenreader-Nutzer werden durch dauerhaft laufende Bewegung abgelenkt oder gestört und können sie nicht deaktivieren.
    .technical-impact = `<marquee>`-Element ohne Pause-Steuerung oder Element mit animation-iteration-count: infinite über 5 Sekunden pro Zyklus ohne erkennbare Pause-Kontrolle oder prefers-reduced-motion-Unterstützung.

rule-manual-no-timing = Zeitbegrenzung vorhanden (AAA)
    .user-impact = Nutzer können Aufgaben nicht in ihrem eigenen Tempo abschließen.
    .technical-impact = Aktive Zeitbegrenzungen auf der Seite.

rule-manual-interruptions = Nicht unterdrückbare Unterbrechungen (AAA)
    .user-impact = Nutzer mit kognitiven Einschränkungen oder Screenreader-Nutzer werden abgelenkt.
    .technical-impact = Unerwartete Dialoge oder assertive Live-Regionen.

rule-manual-re-authenticating = Keine Datenspeicherung bei Reauthentifizierung (AAA)
    .user-impact = Nutzer müssen alle Formulardaten erneut eingeben.
    .technical-impact = Fehlende Session-Datensicherung.

rule-timeouts = Keine Warnung vor Inaktivitäts-Timeout (AAA)
    .user-impact = Nutzer verlieren ungespeicherte Daten ohne Vorwarnung.
    .technical-impact = Fehlendes Timeout-Warnsystem.

rule-navigation-location-missing = Fehlende Positionsangabe (AAA)
    .user-impact = Nutzer verlieren die Orientierung auf verschachtelten Webseiten.
    .technical-impact = Fehlende Breadcrumb-Navigation oder Sitemap-Verbindung.

rule-links-purpose-link-only = Unklare Linkbeschriftung ohne Kontext (AAA)
    .user-impact = Nutzer müssen den gesamten Absatz lesen, um das Ziel eines Links zu verstehen.
    .technical-impact = Generischer Linktext wie 'Mehr lesen' oder 'Hier klicken'.

rule-links-new-window-unannounced = Neues Fenster ohne Ankündigung (AAA)
    .user-impact = Nutzer verlieren unerwartet den Kontext; der Zurück-Button führt nicht zurück.
    .technical-impact = target="_blank" ohne Hinweis im zugänglichen Namen.

rule-manual-pointer-gestures = Fehlende Alternative für Zeigergesten
    .user-impact = Nutzer mit motorischen Einschränkungen können die Funktionen nicht bedienen.
    .technical-impact = Erforderliche Touch-Gesten ohne Klick-Fallback.

rule-pointer-cancellation = Auslösung von Aktionen bei Zeigerberührung
    .user-impact = Nutzer können versehentlich Aktionen auslösen, ohne diese abbrechen zu können.
    .technical-impact = Event-Handler reagieren auf mousedown oder touchstart statt click oder mouseup.

rule-manual-motion-actuation = Fehlende Alternative für Bewegungseingabe
    .user-impact = Nutzer, die das Gerät nicht bewegen können, sind von den Funktionen ausgeschlossen.
    .technical-impact = Nutzen von Device-Motion-APIs ohne UI-Bedienelemente.

rule-targets-size-enhanced = Unzureichende Klickzielgröße (AAA)
    .user-impact = Nutzer mit zittrigen Händen oder auf Mobilgeräten klicken leicht auf falsche Elemente.
    .technical-impact = Zu kleine Abmessungen von Links, Buttons oder Formularelementen im CSS.

rule-targets-size = Unzureichende Klickzielgröße
    .user-impact = Nutzer mit zittrigen Händen oder auf Mobilgeräten klicken leicht auf falsche Elemente.
    .technical-impact = Zu kleine Abmessungen von Links, Buttons oder Formularelementen im CSS.

## Forms & interaction

rule-forms-autocomplete-invalid = Fehlende Eingabezweck-Kennzeichnung
    .user-impact = Kein Autofill möglich, Eingabe fehleranfälliger.
    .technical-impact = Fehlende autocomplete-Attribute.

rule-forms-purpose-missing = Fehlende Eingabezweck-Kennzeichnung
    .user-impact = Kein Autofill möglich, Eingabe fehleranfälliger.
    .technical-impact = Fehlende autocomplete-Attribute.

rule-forms-group-name-missing = Fehlende Formularbeschriftungen
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Fehlende <label>-Elemente.

rule-forms-instructions-missing = Fehlende Formularbeschriftungen
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Fehlende <label>-Elemente.

rule-forms-label-missing = Fehlende Formularbeschriftungen
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Fehlende <label>-Elemente.

rule-forms-placeholder-as-label = Fehlende Formularbeschriftungen
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Fehlende <label>-Elemente.

rule-forms-required-unmarked = Fehlende Formularbeschriftungen
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Fehlende <label>-Elemente.

rule-forms-redundant-entry = Fehlende Wiederverwendung bereits eingegebener Daten
    .user-impact = Nutzer müssen bereits eingegebene Daten erneut eintippen, was Zeit kostet und besonders für Menschen mit motorischen oder kognitiven Einschränkungen eine zusätzliche Hürde darstellt.
    .technical-impact = Formularfelder ohne autocomplete-Attribut, Vorbefüllung oder Wiederverwendungs-Option für bereits erfasste Werte.

rule-auth-paste-blocked = Einfügen in Passwort- oder Code-Feld blockiert
    .user-impact = Nutzer können weder einen Passwort-Manager noch die Zwischenablage verwenden. Wer sich Passwörter nicht merken oder fehlerfrei abschreiben kann, etwa wegen kognitiver oder motorischer Einschränkungen, scheitert an der Anmeldung.
    .technical-impact = Ein synthetisches paste-Ereignis wird abgebrochen, und der Probetext steht danach nicht im Feld (paste-Handler mit preventDefault() oder return false); gemessen auch in gleich-ursprünglichen Rahmen und offenen Shadow Roots.

rule-auth-captcha = CAPTCHA im Anmeldeformular prüfen
    .user-impact = Text- und Rätsel-CAPTCHAs sperren Menschen mit kognitiven Einschränkungen, Legasthenie oder Sehbehinderung von der Anmeldung aus, wenn es keine Alternative gibt.
    .technical-impact = Interaktives CAPTCHA-Widget (reCAPTCHA, hCaptcha, Bild-CAPTCHA) in einem Formular mit Passwort- oder Einmalcode-Feld.

rule-forms-no-submit = Formular ohne Absende-Button
    .user-impact = Tastatur- und Screenreader-Nutzer erkennen den Abschluss der Eingabe schwerer.
    .technical-impact = Fehlendes Submit-Control im Formular.

rule-manual-help = Fehlende Ausfüllhilfe (AAA)
    .user-impact = Nutzer wissen nicht, welches Format oder welche Daten in ein Feld eingegeben werden müssen.
    .technical-impact = Fehlendes Formular-Hilfesystem.

## Language & clarity

rule-document-lang-missing = Fehlende Sprachangabe
    .user-impact = Screenreader liest mit falscher Aussprache vor.
    .technical-impact = Fehlendes lang-Attribut.

rule-html-has-lang = Fehlende Sprachangabe
    .user-impact = Screenreader liest mit falscher Aussprache vor.
    .technical-impact = Fehlendes lang-Attribut.

rule-context-autofocus = Unerwartete Kontextänderung bei Fokus
    .user-impact = Verwirrende Seitenänderungen für Tastaturnutzer.
    .technical-impact = onfocus-Handler lösen Navigation aus.

rule-context-on-focus = Unerwartete Kontextänderung bei Fokus
    .user-impact = Verwirrende Seitenänderungen für Tastaturnutzer.
    .technical-impact = onfocus-Handler lösen Navigation aus.

rule-context-on-input = Unerwartete Kontextänderung bei Eingabe
    .user-impact = Verwirrende Änderungen beim Ausfüllen.
    .technical-impact = Auto-submit bei onchange.

rule-manual-unusual-words = Fehlende Erklärung unüblicher Wörter (AAA)
    .user-impact = Nutzer mit kognitiven Einschränkungen oder Fremdsprachler verstehen den Text nicht.
    .technical-impact = Fehlende Glossar-Verlinkung oder Erklärung im Text.

rule-language-part-undetermined = Anderssprachiger Textteil ohne lang prüfen
    .user-impact = Screenreader lesen den Abschnitt mit der Aussprache der Seitensprache vor; er wird schwer verständlich.
    .technical-impact = Heuristische Spracherkennung je Textblock; Seiten, die weder Deutsch noch Englisch sind, bleiben ungeprüft.

rule-language-part-unmarked = Anderssprachiger Textteil ohne lang prüfen
    .user-impact = Screenreader lesen den Abschnitt mit der Aussprache der Seitensprache vor; er wird schwer verständlich.
    .technical-impact = Heuristische Spracherkennung je Textblock; Seiten, die weder Deutsch noch Englisch sind, bleiben ungeprüft.

rule-language-abbreviation-unexpanded = Fehlende Erklärung von Abkürzungen (AAA)
    .user-impact = Nutzer verstehen den Inhalt nicht, wenn sie die Abkürzung nicht kennen.
    .technical-impact = Fehlendes abbr-Tag mit title-Attribut.

## Visual presentation / contrast

rule-contrast-text-insufficient = Unzureichender Farbkontrast
    .user-impact = Text bei Sehschwäche oder ungünstigen Lichtverhältnissen schwer lesbar.
    .technical-impact = Kontrastverhältnis unter 4.5:1 für normalen Text.

rule-contrast-text-undetermined = Unzureichender Farbkontrast
    .user-impact = Text bei Sehschwäche oder ungünstigen Lichtverhältnissen schwer lesbar.
    .technical-impact = Kontrastverhältnis unter 4.5:1 für normalen Text.

rule-contrast-text-enhanced = Erhöhter Farbkontrast nicht erreicht (AAA)
    .user-impact = Für Menschen mit deutlich eingeschränktem Sehvermögen bleibt der Text schwer lesbar.
    .technical-impact = Kontrastverhältnis unter 7:1 für normalen Text bzw. 4,5:1 für großen Text.

rule-zoom-viewport-locked = Text nicht ausreichend skalierbar
    .user-impact = Nutzer mit Sehschwäche verlieren Inhalte bei Vergrößerung.
    .technical-impact = Feste Pixelwerte, overflow:hidden.

rule-zoom-viewport-missing = Fehlende Viewport-Angabe
    .user-impact = Auf Mobilgeräten erscheint der Text unter jeder lesbaren Größe; wer vergrößert, muss waagerecht scrollen. Am Desktop ändert sich nichts.
    .technical-impact = Kein <meta name="viewport"> im Dokument.

rule-css-overflow-hidden = Kein Reflow bei 320 CSS-Pixeln
    .user-impact = Nutzer, die auf 400 % zoomen, müssen in zwei Richtungen scrollen — Inhalte werden unzugänglich.
    .technical-impact = Feste Breiten oder overflow:hidden verhindern responsives Reflow.

rule-contrast-control-insufficient = Unzureichender Kontrast bei UI-Elementen
    .user-impact = Interaktive Elemente schwer erkennbar.
    .technical-impact = Kontrastverhältnis UI-Elemente unter 3:1.

rule-contrast-control-undetermined = Unzureichender Kontrast bei UI-Elementen
    .user-impact = Interaktive Elemente schwer erkennbar.
    .technical-impact = Kontrastverhältnis UI-Elemente unter 3:1.

rule-text-spacing-clipped = Inhalt bei erhöhtem Zeilen-/Wortabstand abgeschnitten
    .user-impact = Nutzer, die Zeilen-/Wortabstand für bessere Lesbarkeit anpassen, verlieren Inhalte.
    .technical-impact = Fixe Höhe/Breite mit overflow:hidden auf Textcontainern; gemessen vor und nach dem Überschreiben des Textabstands, nur neu abgeschnittener Text zählt.

rule-color-link-indistinct = Link nur durch Farbe unterscheidbar
    .user-impact = Nutzer mit Farbsehschwäche können Links im Text nicht erkennen.
    .technical-impact = Kein Unterstrich, Fettschrift oder anderes nicht-farbliches Merkmal vorhanden.

rule-motion-reduced-motion-ignored = Animationen ohne prefers-reduced-motion
    .user-impact = Nutzer mit vestibulären Störungen oder Photosensitivität können Bewegung nicht deaktivieren.
    .technical-impact = Kein @media (prefers-reduced-motion) in den Stylesheets.

rule-patterns-tooltip-unreferenced = Unzugängliche Hover- oder Fokus-Inhalte (Tooltips)
    .user-impact = Nutzer können Tooltip-Inhalte nicht lesen, diese nicht per Tastatur schließen, oder sie werden von Screenreadern ignoriert.
    .technical-impact = Unvollständige ARIA-Verknüpfung (aria-describedby) oder fehlende Steuerungsmöglichkeiten.

rule-media-audio-autoplay = Selbststartender Ton ohne Steuerung prüfen
    .user-impact = Ton, der von selbst startet, übertönt die Sprachausgabe von Screenreadern.
    .technical-impact = <audio autoplay> ohne muted; ob der Ton länger als drei Sekunden läuft und sich anhalten lässt, steht nicht im Markup.

rule-text-justified = Eingeschränkte visuelle Darstellung von Text (AAA)
    .user-impact = Nutzer mit Sehbehinderungen oder Legasthenie haben Probleme beim Lesen.
    .technical-impact = Unzureichendes CSS-Styling für Zeilenabstand oder Blocksatz-Formatierung.

rule-text-line-height-tight = Eingeschränkte visuelle Darstellung von Text (AAA)
    .user-impact = Nutzer mit Sehbehinderungen oder Legasthenie haben Probleme beim Lesen.
    .technical-impact = Unzureichendes CSS-Styling für Zeilenabstand oder Blocksatz-Formatierung.

rule-visual-presentation = Eingeschränkte visuelle Darstellung von Text (AAA)
    .user-impact = Nutzer mit Sehbehinderungen oder Legasthenie haben Probleme beim Lesen.
    .technical-impact = Unzureichendes CSS-Styling für Zeilenabstand oder Blocksatz-Formatierung.

## Technical robustness

rule-keyboard-hidden-focusable = Fokussierbares Element versteckt vor Screenreadern
    .user-impact = Tastaturnutzer erreichen Elemente, die Screenreader nicht ankündigen — unsichtbare Stolperfallen.
    .technical-impact = aria-hidden="true" auf fokussierbarem Element oder seinem Vorfahren.

rule-status-live-overridden = Statusmeldungen nicht programmatisch bestimmbar
    .user-impact = Screenreader kündigt Statusmeldungen nicht korrekt an.
    .technical-impact = Konflikt zwischen role (alert/status/log/timer) und aria-live-Wert.

rule-aria-required-attribute-missing = Pflicht-ARIA-Attribut fehlt
    .user-impact = Assistive Technologien koennen Zustand oder Wert des Elements nicht verlaesslich vermitteln.
    .technical-impact = Rolle und ARIA-Attributsatz sind unvollstaendig.

rule-aria-required-parent-missing = Element außerhalb des erforderlichen Eltern-Kontexts
    .user-impact = Assistive Technologien können die Beziehung des Elements zu seiner Gruppe (z. B. Tab zu Tableiste) nicht vermitteln.
    .technical-impact = Fehlender struktureller Eltern-Kontext für die verwendete ARIA-Rolle.

rule-inert-dialog-inert = Ungültige moderne Interaktionsattribute
    .user-impact = Screenreader- und Tastaturnutzer erreichen Dialoge, Popover oder gesperrte Bereiche nicht zuverlässig.
    .technical-impact = Defekte popovertarget-Bezüge, unbenannte Oberflächen oder aktiver Fokus innerhalb inert markierter Bereiche.

rule-modern-attribute-misuse = Ungültige moderne Interaktionsattribute
    .user-impact = Screenreader- und Tastaturnutzer erreichen Dialoge, Popover oder gesperrte Bereiche nicht zuverlässig.
    .technical-impact = Defekte popovertarget-Bezüge, unbenannte Oberflächen oder aktiver Fokus innerhalb inert markierter Bereiche.

rule-popover-target-invalid = Ungültige moderne Interaktionsattribute
    .user-impact = Screenreader- und Tastaturnutzer erreichen Dialoge, Popover oder gesperrte Bereiche nicht zuverlässig.
    .technical-impact = Defekte popovertarget-Bezüge, unbenannte Oberflächen oder aktiver Fokus innerhalb inert markierter Bereiche.

rule-popover-target-missing = Ungültige moderne Interaktionsattribute
    .user-impact = Screenreader- und Tastaturnutzer erreichen Dialoge, Popover oder gesperrte Bereiche nicht zuverlässig.
    .technical-impact = Defekte popovertarget-Bezüge, unbenannte Oberflächen oder aktiver Fokus innerhalb inert markierter Bereiche.

rule-aria-attribute-prohibited = Verbotene ARIA-Attribute
    .user-impact = Screenreader erhalten widersprüchliche oder ungültige Semantik.
    .technical-impact = ARIA-Attribute passen nicht zur impliziten oder expliziten Rolle.

rule-aria-attribute-not-allowed = ARIA-Attribut für diese Rolle nicht zulässig
    .user-impact = Assistive Technologien ignorieren das Attribut oder interpretieren es falsch.
    .technical-impact = Attribut-Rollen-Kombination entspricht nicht der ARIA-Spezifikation.

rule-aria-attribute-value-invalid = Ungültiger Wert eines ARIA-Attributs
    .user-impact = Assistive Technologien können Zustand oder Beziehung des Elements nicht korrekt vermitteln.
    .technical-impact = Attributwert entspricht nicht dem in der ARIA-Spezifikation definierten Werttyp.

rule-aria-reference-missing = Leerer Verweis in ARIA-Beziehungsattribut
    .user-impact = Assistive Technologien können die beabsichtigte Beziehung zwischen den Elementen nicht auflösen.
    .technical-impact = Beziehungsattribut ohne gültige ID-Referenz.

rule-aria-role-abstract = Ungültige ARIA-Rolle
    .user-impact = Assistive Technologien können die Funktion des Elements nicht bestimmen.
    .technical-impact = Rollenwert ist in der ARIA-Spezifikation nicht definiert.

rule-aria-role-invalid = Ungültige ARIA-Rolle
    .user-impact = Assistive Technologien können die Funktion des Elements nicht bestimmen.
    .technical-impact = Rollenwert ist in der ARIA-Spezifikation nicht definiert.

rule-aria-attribute-unknown = Ungültiger ARIA-Attributname
    .user-impact = Assistive Technologien ignorieren das Attribut vollständig.
    .technical-impact = Attributname existiert nicht in der ARIA-Spezifikation.

rule-document-lang-mismatch = Widersprüchliche Sprachangaben
    .user-impact = Assistive Technologien erhalten widersprüchliche Angaben zur Sprache des Inhalts.
    .technical-impact = lang und xml:lang stimmen im Primary-Subtag nicht überein.

rule-tables-headers-attr-invalid = Ungültiger Headers-Verweis in Tabellenzelle
    .user-impact = Screenreader können der Zelle keine oder eine falsche Spalten-/Zeilenüberschrift zuordnen.
    .technical-impact = headers-Attribut verweist nicht auf eine gültige th-Zelle derselben Tabelle.

rule-tables-data-undetermined = Tabellenüberschrift ohne zugehörige Datenzellen
    .user-impact = Screenreader kündigen eine Überschrift an, die keine Daten strukturiert.
    .technical-impact = Header-Zelle ohne zugeordnete Datenzellen in der Tabelle.

rule-tables-header-without-data = Tabellenüberschrift ohne zugehörige Datenzellen
    .user-impact = Screenreader kündigen eine Überschrift an, die keine Daten strukturiert.
    .technical-impact = Header-Zelle ohne zugeordnete Datenzellen in der Tabelle.

rule-aria-tabpanel-missing = Tableiste ohne zugehöriges Tab-Panel
    .user-impact = Screenreader-Nutzer können nach Aktivieren eines Tabs den zugehörigen Inhalt nicht auffinden.
    .technical-impact = tablist ohne begleitendes tabpanel-Element.

rule-aria-tab-selected-missing = Kein Tab als ausgewählt ausgezeichnet
    .user-impact = Screenreader-Nutzer erfahren nicht, welcher Tab aktiv ist und zu welchem der sichtbare Inhalt gehört.
    .technical-impact = tablist ohne tab mit aria-selected="true".

rule-patterns-accordion-controls-missing = Aufgeklappter Button ohne Verweis auf seinen Bereich
    .user-impact = Screenreader-Nutzer erfahren nicht, welcher Bereich aufgeklappt wurde.
    .technical-impact = aria-expanded="true" ohne aria-controls; WAI-ARIA verlangt es nicht, die APG nennt es beim Disclosure-Muster optional.

rule-dialog-modal-unmarked = Dialog nicht als modal ausgezeichnet
    .user-impact = Ist der Dialog modal gemeint, lesen Screenreader den Inhalt dahinter weiter, als wäre er erreichbar.
    .technical-impact = role="dialog" ohne aria-modal="true"; ob der Dialog modal gemeint ist, zeigt erst sein Verhalten.

rule-aria-combobox-popup-missing = Combobox ohne Optionsliste
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Auswahlmöglichkeiten die Combobox bietet.
    .technical-impact = combobox ohne verknüpfte listbox/option-Elemente.

rule-frames-name-missing = Iframe ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erkennen Zweck und Inhalt des eingebetteten Frames nicht.
    .technical-impact = Fehlender zugänglicher Name auf iframe/frame.

rule-frames-cross-origin = Cross-origin Iframe: manuell prüfen
    .user-impact = Eingebettete Inhalte (Karten, Videos, Consent-Widgets) sind möglicherweise für Screenreader-Nutzer unzugänglich.
    .technical-impact = Iframe-Inhalt ist über CDP nicht erreichbar — automatische Regelprüfung nicht möglich.

rule-ids-duplicate = Mehrfach vergebene ID wird referenziert
    .user-impact = Assistierende Technik kann die Beziehung nicht eindeutig auflösen und gibt Name oder Beschreibung des falschen Elements aus.
    .technical-impact = Dieselbe ID kommt im DOM mehrfach vor und wird von einem for-, headers- oder aria-Attribut referenziert.

rule-images-server-side-map = Server-seitige Image-Map verwendet
    .user-impact = Tastatur- und Screenreader-Nutzer können die einzelnen Bildbereiche nicht ansteuern.
    .technical-impact = ismap-Attribut auf einem Bild ohne äquivalente clientseitige Alternative.

rule-svg-name-missing = Fehlender Alternativtext bei SVG-Grafiken
    .user-impact = Screenreader-Nutzer erhalten keine Information über den Inhalt der Grafik.
    .technical-impact = SVG ohne <title>-Kindelement, aria-label oder aria-labelledby.

rule-images-area-alt-missing = Fehlender Alternativtext bei Image-Map-Bereichen
    .user-impact = Screenreader-Nutzer erfahren nicht, wohin ein anklickbarer Bildbereich führt.
    .technical-impact = <area>-Element ohne alt-Attribut.

rule-images-input-alt-missing = Fehlender Alternativtext bei Bild-Buttons
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Aktion der Button auslöst.
    .technical-impact = <input type="image"> ohne alt-Attribut.

rule-objects-alt-missing = Fehlende Textalternative bei eingebetteten Objekten
    .user-impact = Screenreader-Nutzer erhalten keine Information über den Inhalt des eingebetteten Objekts.
    .technical-impact = <object> ohne Textinhalt oder aria-label.

rule-html-content-model = Verletzung des HTML5-Content-Models
    .user-impact = Screenreader können die Struktur nicht zuverlässig auswerten, auch wenn sie augenscheinlich korrekt wirkt.
    .technical-impact = Unerwartetes Kindelement innerhalb eines strukturellen Containers (z. B. <ul>, <table>, <select>).

rule-forms-group-missing = Fehlende Gruppierung verwandter Formularfelder
    .user-impact = Screenreader-Nutzer erkennen nicht, dass mehrere Felder zu einer Frage/Gruppe gehören.
    .technical-impact = Radio-Buttons/Checkboxen ohne umschließendes <fieldset> mit <legend>.

rule-landmarks-not-top-level = Landmark ist verschachtelt
    .user-impact = Screenreader-Nutzer, die per Landmark-Navigation springen, landen in einem unerwarteten Kontext.
    .technical-impact = <header>/<footer>/<main> bzw. role="banner"/"contentinfo"/"main" ist Nachfahre eines anderen Landmarks.

rule-landmarks-banner-duplicate = Mehrere Banner-Landmarks
    .user-impact = Screenreader-Nutzer können den 'Kopfbereich' der Seite nicht eindeutig zuordnen.
    .technical-impact = Mehr als ein Element mit role="banner" (implizit oder explizit).

rule-landmarks-contentinfo-duplicate = Mehrere Contentinfo-Landmarks
    .user-impact = Screenreader-Nutzer können den 'Fußbereich' der Seite nicht eindeutig zuordnen.
    .technical-impact = Mehr als ein Element mit role="contentinfo" (implizit oder explizit).

rule-landmarks-main-duplicate = Mehrere Main-Landmarks
    .user-impact = Screenreader-Nutzer, die zum Hauptinhalt springen, wissen nicht, welcher Bereich gemeint ist.
    .technical-impact = Mehr als ein Element mit role="main" (implizit oder explizit).

rule-landmarks-banner-missing = Fehlender Banner-Landmark
    .user-impact = Assistierende Technologien können den Kopfbereich der Seite nicht identifizieren.
    .technical-impact = Kein Element mit role="banner" (implizit über <header> oder explizit) vorhanden.

rule-landmarks-navigation-missing = Fehlender Navigations-Landmark
    .user-impact = Screenreader-Nutzer können nicht direkt zur Navigation springen.
    .technical-impact = Kein Element mit role="navigation" (implizit über <nav> oder explizit) vorhanden.

rule-landmarks-contentinfo-missing = Fehlender Fußbereich-Landmark (contentinfo)
    .user-impact = Screenreader-Nutzer können den Fußbereich mit Kontakt, Impressum und Rechtlichem nicht direkt ansteuern.
    .technical-impact = Kein Element mit role="contentinfo" (implizit über ein <footer> außerhalb von <main>/<article>/<section> oder explizit) vorhanden.

rule-lists-item-outside-list = Listenelemente ohne Listen-Container
    .user-impact = Screenreader-Nutzer erhalten keine Ansage der Listenlänge/-struktur.
    .technical-impact = <li> außerhalb von <ul>/<ol>.

rule-headings-skip-level = Übersprungene Überschriftenebene
    .user-impact = Screenreader-Nutzer, die über Überschriften navigieren, vermuten fehlende Abschnitte.
    .technical-impact = Überschriftenebenen folgen nicht lückenlos aufeinander.

rule-headings-empty = Leere Überschrift
    .user-impact = Screenreader-Nutzer stoßen in der Überschriftenliste auf einen Eintrag ohne Inhalt.
    .technical-impact = <h1>–<h6> bzw. role="heading" ohne Text und ohne aria-label.

rule-headings-h1-missing = Fehlende Hauptüberschrift (h1)
    .user-impact = Screenreader-Nutzer finden den Einstieg in den Hauptinhalt nicht über die Überschriftennavigation.
    .technical-impact = Kein <h1>-Element im Dokument.

rule-headings-h1-multiple = Mehrere Hauptüberschriften (h1)
    .user-impact = Screenreader-Nutzer können den eigentlichen Seitentitel schwerer vom Rest unterscheiden.
    .technical-impact = Mehrere <h1>-Elemente im Dokument.

rule-lists-invalid-structure = Liste mit unzulässigen Kindelementen
    .user-impact = Screenreader-Nutzer hören eine falsche Anzahl von Listeneinträgen oder verlieren Inhalte.
    .technical-impact = Direkte Kinder von <ul>/<ol>/role="list" sind keine <li>/role="listitem".

rule-lists-empty = Leere Liste
    .user-impact = Screenreader kündigen eine Liste ohne Inhalt an.
    .technical-impact = <ul>/<ol>/role="list" ohne <li>/role="listitem".

rule-lists-term-without-definition = Begriff ohne Definition in Definitionsliste
    .user-impact = Screenreader-Nutzer können Begriff und Definition nicht als zusammengehörig erkennen.
    .technical-impact = <dt> ohne folgendes <dd> in der <dl>.

rule-tables-header-missing = Datentabelle ohne Kopfzellen
    .user-impact = Screenreader-Nutzer hören Zellwerte ohne die zugehörige Zeilen- oder Spaltenbeschriftung.
    .technical-impact = Kein <th> und keine Zelle mit role="columnheader"/"rowheader".

rule-tables-name-missing = Datentabelle ohne Namen
    .user-impact = Screenreader-Nutzer erfahren beim Betreten der Tabelle nicht, worum es in ihr geht.
    .technical-impact = Kein <caption> und kein aria-label an der Tabelle.

rule-tables-presentational-with-headers = Layouttabelle mit Kopfzellen
    .user-impact = Screenreader-Nutzer erhalten widersprüchliche Signale, ob es sich um eine Datentabelle handelt.
    .technical-impact = role="presentation"/"none" an einer Tabelle mit <th>.

rule-document-lang-invalid = Ungültige Sprachangabe der Seite
    .user-impact = Screenreader wählen eine falsche oder die Standard-Aussprache.
    .technical-impact = Ungültiger Wert im lang-Attribut des html-Elements.

rule-keyboard-positive-tabindex = Positiver tabindex verändert die Fokusreihenfolge
    .user-impact = Tastaturnutzer erreichen Elemente in einer Reihenfolge, die nicht dem Seitenaufbau entspricht.
    .technical-impact = tabindex-Wert größer als 0.

rule-forms-title-only-label = Formularfeld-Beschriftung nur über title-Attribut
    .user-impact = Das title-Attribut wird von vielen Screenreadern inkonsistent oder gar nicht vorgelesen; sehende Nutzer sehen es nur bei Hover.
    .technical-impact = Formularelement ohne <label>/aria-label, Name kommt nur aus title.

rule-zoom-viewport-scale-limited = Zoom auf 500% durch Viewport-Meta-Tag verhindert
    .user-impact = Sehbehinderte Nutzer können den Seiteninhalt nicht ausreichend vergrößern.
    .technical-impact = maximum-scale im viewport-Meta-Tag unter 5 gesetzt.

rule-keyboard-click-handler-not-focusable = Klick-Handler ohne Tastatur-Äquivalent
    .user-impact = Tastaturnutzer können die Funktion nicht auslösen.
    .technical-impact = onclick auf einem <div>/<span> ohne onkeydown/onkeyup, tabindex oder passende Rolle.

rule-keyboard-pointer-only = Klick-Handler ohne Tastatur-Äquivalent
    .user-impact = Tastaturnutzer können die Funktion nicht auslösen.
    .technical-impact = onclick auf einem <div>/<span> ohne onkeydown/onkeyup, tabindex oder passende Rolle.

rule-landmarks-main-missing = Kein Hauptinhaltsbereich als Sprungziel
    .user-impact = Tastaturnutzer haben kein verlässliches Ziel, um wiederkehrende Navigation zu überspringen.
    .technical-impact = Kein Element mit role="main" (implizit über <main> oder explizit) vorhanden.

rule-keyboard-skip-link-missing = Fehlender Skip-Link
    .user-impact = Tastaturnutzer müssen bei jedem Seitenwechsel durch die gesamte Navigation tabben.
    .technical-impact = Kein Link mit erkennbarem 'skip'/'zum Inhalt'-Text oder Anker auf #main/#content am Seitenanfang.

rule-focus-indicator-missing = Kein sichtbarer Fokusindikator gemessen
    .user-impact = Tastaturnutzer sehen nicht, welches Element gerade fokussiert ist.
    .technical-impact = Fokuslauf: die berechneten Stile des Elements vor und nach dem Fokussieren sind gleich, soweit sie als Indikator taugen (outline, box-shadow, border, background, color, text-decoration).

rule-focus-outline-removed = Fokus-Indikator per CSS entfernt
    .user-impact = Tastaturnutzer sehen nicht, welches Element gerade fokussiert ist.
    .technical-impact = outline: none/0 in :focus-Regeln ohne alternative sichtbare Fokusmarkierung.

rule-keyboard-scrollable-region-not-focusable = Scrollbarer Bereich per Tastatur nicht erreichbar
    .user-impact = Tastaturnutzer können den Bereich nicht scrollen und den verdeckten Inhalt nicht lesen.
    .technical-impact = Element mit overflow: auto/scroll und überlaufendem Inhalt, ohne tabindex und ohne fokussierbare Kinder.

rule-forms-error-unidentified = Ungültiges Feld ohne zugängliche Fehlerbeschreibung
    .user-impact = Screenreader-Nutzer erfahren, dass ein Fehler vorliegt, aber nicht welcher.
    .technical-impact = aria-invalid="true" ohne description/aria-describedby auf ein sichtbares Fehlerelement.

rule-aria-owns-conflict = ARIA-Referenz zeigt auf mehrdeutige ID
    .user-impact = Assistierende Technologien können nicht zuverlässig bestimmen, welches Element gemeint ist.
    .technical-impact = Referenzierte ID kommt im DOM mehr als einmal vor.

rule-names-required-missing = Interaktives Element ohne aussagekräftigen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, wozu das Element dient.
    .technical-impact = Fehlendes oder rein symbolisches aria-label/aria-labelledby/Textinhalt.

rule-names-symbol-only = Interaktives Element ohne aussagekräftigen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, wozu das Element dient.
    .technical-impact = Fehlendes oder rein symbolisches aria-label/aria-labelledby/Textinhalt.

rule-names-title-only = Natives title-Attribut als einzige Beschreibung
    .user-impact = Der Tooltip des title-Attributs erscheint nur bei Mauszeiger-Hover; per Tastatur und auf Touchgeräten bleibt der Zweck des Elements unsichtbar.
    .technical-impact = Der zugängliche Name stammt nur aus dem title-Attribut; kein Textinhalt, kein aria-label, kein aria-labelledby.

rule-description-duplicates-name = Beschreibung wiederholt den Namen
    .user-impact = Screenreader sagen denselben Text zweimal an, ohne zusätzliche Information.
    .technical-impact = aria-describedby oder title liefert denselben Text wie der Name.

rule-aria-required-children-missing = Erforderliche Kindelemente einer ARIA-Rolle fehlen
    .user-impact = Assistierende Technologien können die Beziehung zwischen den Widget-Teilen nicht auswerten.
    .technical-impact = z. B. role="tablist" ohne zugehörige role="tabpanel"-Elemente.

rule-dialog-name-missing = Dialog ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erfahren beim Öffnen des Dialogs nicht, worum es geht.
    .technical-impact = Dialog-Element ohne aria-label oder aria-labelledby auf den Dialogtitel.

rule-buttons-name-missing = Formularelement, Link oder Button ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Eingabe erwartet wird oder wohin ein Link/Button führt.
    .technical-impact = Formularelement/Link/Button ohne <label>, aria-label, aria-labelledby oder Textinhalt.

rule-links-name-missing = Formularelement, Link oder Button ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Eingabe erwartet wird oder wohin ein Link/Button führt.
    .technical-impact = Formularelement/Link/Button ohne <label>, aria-label, aria-labelledby oder Textinhalt.

rule-links-used-as-button = Link als Button-Ersatz ohne Tastenrolle
    .user-impact = Assistierende Technologien kündigen das Element als Link an, obwohl es sich wie ein Button verhält — Tastaturnutzer erwarten das falsche Bedienverhalten.
    .technical-impact = <a href="#">/<a href="javascript:..."> mit onclick, ohne role="button".

rule-aria-role-redundant = Redundante ARIA-Rolle
    .user-impact = Kein unmittelbarer Nutzerschaden, aber unnötiges Risiko für Inkonsistenzen bei zukünftigen Änderungen.
    .technical-impact = role-Attribut dupliziert die native implizite Rolle des Tags.

rule-lists-role-redundant = Redundante ARIA-Rolle
    .user-impact = Kein unmittelbarer Nutzerschaden, aber unnötiges Risiko für Inkonsistenzen bei zukünftigen Änderungen.
    .technical-impact = role-Attribut dupliziert die native implizite Rolle des Tags.

rule-summary-name-missing = Summary-Element ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, was der Aufklapp-Bereich enthält, bevor sie ihn öffnen.
    .technical-impact = <summary> ohne Textinhalt oder aria-label.

rule-landmarks-content-outside = Inhalt außerhalb einer Landmark-Region
    .user-impact = Inhalt außerhalb einer Landmark-Region.
    .technical-impact = Inhalt außerhalb einer Landmark-Region.

rule-keyboard-focusable-no-role = Fokussierbares Element ohne interaktive Rolle
    .user-impact = Fokussierbares Element ohne interaktive Rolle.
    .technical-impact = Fokussierbares Element ohne interaktive Rolle.

rule-display-toggle-missing = Kein Umschalter für Darstellungsmodi
    .user-impact = Fehlt der Umschalter tatsächlich, kann nicht auf eine ruhige oder reine Textdarstellung wechseln, wer Bewegung oder Grafiken nicht verträgt.
    .technical-impact = Kein Element mit [data-display-toggle] auf einer Seite mit figure[data-viz].

rule-display-init-missing = Darstellungsmodus wird zu spät gesetzt
    .user-impact = Die Seite kann kurz in der falschen Darstellung erscheinen, etwa mit Animation, bevor die gewählte ruhige Darstellung greift.
    .technical-impact = data-display nicht per blockierendem Skript im <head> gesetzt.

rule-display-text-media-visible = Grafiken im Textmodus sichtbar
    .user-impact = Wer den Textmodus gewählt hat, bekommt trotzdem Grafiken oder bewegte Inhalte.
    .technical-impact = [data-viz-live]/[data-viz-static] oder Medien der figure bei html[data-display=text] sichtbar.

rule-display-text-not-visible = Textschicht einer Visualisierung fehlt im Textmodus
    .user-impact = Im Textmodus geht die Aussage der Grafik verloren.
    .technical-impact = [data-viz-text] fehlt, ist leer oder bei html[data-display=text] nicht sichtbar.

rule-viz-text-missing = Textschicht einer Visualisierung fehlt im Textmodus
    .user-impact = Im Textmodus geht die Aussage der Grafik verloren.
    .technical-impact = [data-viz-text] fehlt, ist leer oder bei html[data-display=text] nicht sichtbar.

rule-display-text-hidden = Textschicht einer Visualisierung für Hilfstechnik verborgen
    .user-impact = Screenreader-Nutzer erfahren die Aussage der Grafik nicht.
    .technical-impact = hidden, aria-hidden=true, inert oder display:none/visibility:hidden an [data-viz-text] oder einem Vorfahren.

rule-viz-caption-missing = Visualisierung ohne Beschriftung
    .user-impact = Nichts benennt die Visualisierung kurz; wer sie nicht sieht, erfährt nicht, worum es geht, bevor er sich durch die Textschicht liest.
    .technical-impact = figure[data-viz] ohne <figcaption> als direktes Kind.

rule-viz-static-missing = Bewegte Visualisierung ohne Standbild
    .user-impact = Im ruhigen Modus ersetzt nichts die Bewegung; wer Bewegung nicht verträgt, sieht entweder sie oder gar nichts.
    .technical-impact = figure[data-viz=3d|interactive] ohne Nachfahre mit [data-viz-static].

rule-viz-table-missing = Diagramm ohne Wertetabelle
    .user-impact = Die Beziehungen zwischen den Werten gehen verloren, wenn die Textschicht sie nicht auf andere Weise vermittelt.
    .technical-impact = figure[data-viz=chart] ohne <table> im Teilbaum.

# Explanation texts: one message per entry of `EXPLANATIONS` in
# `src/output/explanations.rs`, in the order of that table. The message value
# is the customer title, the attributes carry the description, the user
# impact, the typical cause, the recommendation and the technical note.
# The key is `explanation-` plus the table key with `.`, `/` and `_` written
# as `-` (`output::explanations::explanation_text_key`).

## 1. Perceivable

explanation-1-1-1 = Fehlende Alternativtexte bei Bildern
    .description = Bilder auf der Website haben keinen beschreibenden Alternativtext. Dadurch können Screenreader den Bildinhalt nicht an blinde oder sehbeeinträchtigte Nutzer vermitteln.
    .user-impact = Menschen mit Sehbeeinträchtigung erhalten an diesen Stellen keine oder nur unvollständige Information. Der Bildinhalt geht für sie vollständig verloren.
    .typical-cause = Teaserbilder, Slider, redaktionell eingepflegte Medien ohne Pflichtfeld im CMS, oder dekorative Bilder, die nicht als solche markiert sind.
    .recommendation = Für informative Bilder einen beschreibenden Alt-Text hinterlegen, der den Bildinhalt oder -zweck vermittelt. Rein dekorative Bilder mit einem leeren Alt-Attribut markieren (alt="").
    .technical-note = Informative Bilder: <img alt="Beschreibung">. Dekorative Bilder: <img alt=""> oder role="presentation". CMS-Felder für Alt-Texte als Pflichtfeld konfigurieren.

explanation-1-2-1 = Video/Audio ohne Alternative nicht automatisch geprüft
    .description = Auf der Seite wurde ein reines Video- oder Audio-Element gefunden. Ob dafür eine gleichwertige Textalternative (z. B. ein Transkript) existiert, kann ein Scanner nicht zuverlässig aus dem HTML ableiten — das erfordert eine inhaltliche Prüfung.
    .user-impact = Menschen, die das Video nicht sehen oder das Audio nicht hören können, erhalten ohne Textalternative keinen Zugang zum vermittelten Inhalt.
    .typical-cause = Eingebettete Video-/Audio-Player (nativ oder als Drittanbieter-Embed), für die keine Transkript-Seite oder kein Textäquivalent verlinkt ist.
    .recommendation = Für jedes reine Video-/Audio-Element manuell prüfen, ob ein Transkript oder eine gleichwertige Textalternative vorhanden und vollständig ist.
    .technical-note = Ein Textlink zum Transkript in der Nähe des Medienelements platzieren, z. B. direkt unterhalb des Players.

explanation-1-2-2 = Untertitel bei Video nicht bestätigt
    .description = Für ein Video auf der Seite konnte keine auflösbare Untertitel-/Caption-Datei (`<track kind="captions">`) technisch bestätigt werden, oder das Video ist ein eingebetteter Drittanbieter-Player, dessen Untertitel-Einstellungen sich nicht von außen prüfen lassen.
    .user-impact = Gehörlose und schwerhörige Nutzer erhalten ohne Untertitel keinen Zugang zum gesprochenen Inhalt des Videos.
    .typical-cause = Video ohne `<track kind="captions">`-Element, ein Track, dessen Datei nicht erreichbar ist, oder ein eingebetteter Player (YouTube, Vimeo u. ä.), dessen Untertitel-Status vom eigentlichen Betreiber der Seite konfiguriert wird.
    .recommendation = Sicherstellen, dass jedes vorab aufgezeichnete Video mit Ton synchronisierte Untertitel hat. Bei eingebetteten Playern die Untertitel-Einstellungen manuell im Player prüfen.
    .technical-note = Natives `<video>`: `<track kind="captions" src="..." srclang="de">` hinzufügen, Datei muss erreichbar sein. Bei Embeds die Caption-Funktion der Plattform aktivieren.

explanation-1-2-3 = Audiodeskription oder Textalternative nicht automatisch geprüft
    .description = Ob ein vorab aufgezeichnetes Video mit visuell vermittelten Informationen (z. B. Handlungen, Ortswechsel) eine Audiodeskription oder eine vollständige Textalternative besitzt, kann ein Scanner nicht aus dem HTML allein feststellen.
    .user-impact = Blinde und sehbeeinträchtigte Nutzer verpassen visuell vermittelte Informationen, die nicht im normalen Ton des Videos genannt werden.
    .typical-cause = Video mit wichtigen visuellen Inhalten (Diagramme, Handlungsabläufe, Textein-blendungen), ohne separate Audiodeskriptionsspur und ohne Textalternative.
    .recommendation = Videos mit relevanten visuellen Inhalten manuell prüfen: Existiert eine Audiodeskription oder eine vollständige Textalternative, die diese Inhalte abdeckt?
    .technical-note = Audiodeskription per `<track kind="descriptions">` oder separater Tonspur einbinden, alternativ eine vollständige Textalternative verlinken.

explanation-1-2-8 = Vollständige Textalternative für Video nicht bestätigt
    .description = Für vorab aufgezeichnete Video-/Audio-Inhalte konnte nicht automatisch bestätigt werden, dass eine vollständige Textalternative (Transkript inkl. relevanter visueller Informationen) existiert.
    .user-impact = Taubblinde Nutzer und Nutzer, die weder Bild noch Ton wahrnehmen können, sind auf eine vollständige Textalternative angewiesen, um auf den Inhalt zuzugreifen.
    .typical-cause = Video-/Audio-Element ohne verlinktes Transkript, oder ein in der Nähe gefundener Link deutet zwar auf ein Transkript hin, dessen Vollständigkeit ist aber nicht automatisch prüfbar.
    .recommendation = Für jedes vorab aufgezeichnete Video-/Audio-Element ein vollständiges Transkript bereitstellen und manuell auf Vollständigkeit prüfen (inkl. gesprochenem Dialog und relevanten visuellen Informationen).
    .technical-note = Transkript als Text auf der Seite oder verlinkt bereitstellen, `<track kind="descriptions">` für Audiodeskription ergänzen, wo zutreffend.

explanation-1-3-1 = Fehlende semantische Struktur
    .description = Inhalte sind visuell strukturiert (z. B. durch Größe oder Farbe), aber die Struktur ist nicht im HTML-Code hinterlegt. Screenreader und andere Hilfstechnologien können die Beziehungen zwischen Inhalten nicht erkennen.
    .user-impact = Nutzer mit Screenreader können Tabellen, Listen und Formulargruppen nicht korrekt navigieren. Die logische Struktur der Seite geht verloren.
    .typical-cause = Tabellen ohne korrekte Tabellenauszeichnung, fehlende Fieldsets bei Formularen, Listen als div-Elemente statt ul/ol, fehlende Landmarks.
    .recommendation = Inhalte semantisch korrekt auszeichnen: Tabellen mit <table>, <th>, <td>; Listen mit <ul>/<ol>; Formulare mit <fieldset> und <legend> gruppieren.
    .technical-note = HTML5-Semantik nutzen: <nav>, <main>, <aside>, <header>, <footer>. Tabellen: scope-Attribute für Kopfzellen. ARIA-Rollen nur als Ergänzung.

explanation-region = Inhalt außerhalb einer Landmark-Region
    .description = Ein Element mit sichtbarem Inhalt — häufig ein Link — liegt außerhalb aller Landmark-Bereiche der Seite (main, nav, header, footer, aside). Screenreader-Nutzer, die per Landmark zwischen Seitenbereichen springen, finden dieses Element dabei nicht.
    .user-impact = Screenreader-Nutzer, die per Landmark-Navigation zwischen Hauptinhalt, Navigation und Fußzeile wechseln, überspringen dieses Element unbemerkt — es erscheint in keiner der angesteuerten Regionen. Im Lesemodus und per Tab bleibt es erreichbar.
    .typical-cause = Markup, das per JavaScript nachträglich außerhalb des Haupt-Layouts eingefügt wird, Reste von Drittanbieter-Widgets, oder Inhalte, die vor dem ersten Landmark stehen, ohne selbst in main/nav/header/footer eingebettet zu sein.
    .recommendation = Das Element in die passende Landmark-Region verschieben (z. B. einen Link in <nav> oder <main>). Passt keine bestehende Region, einen zusätzlichen Bereich mit role="region" und einem beschreibenden aria-label anlegen.
    .technical-note = HTML5-Landmarks konsequent nutzen: <header>, <nav>, <main>, <aside>, <footer>. Skip-Links vor dem ersten Landmark sind bewusst ausgenommen. Für Bereiche ohne passendes semantisches Element role="region" plus aria-label verwenden.

explanation-landmark-unique = Landmarks nicht eindeutig benannt
    .description = Mehrere Bereiche der Seite mit derselben Rolle (z. B. zwei Navigationen oder zwei Aside-Bereiche) tragen keinen unterscheidbaren Namen. Screenreader kündigen beide Bereiche mit demselben generischen Namen an.
    .user-impact = Screenreader-Nutzer, die per Landmark-Navigation zwischen Bereichen springen, hören z. B. zweimal 'Navigation' und können ohne weitere Erkundung nicht unterscheiden, welcher Bereich gemeint ist.
    .typical-cause = Mehrere <nav>-, <aside>- oder role="region"-Elemente auf derselben Seite (z. B. Hauptnavigation und Breadcrumb-Navigation), denen kein aria-label oder aria-labelledby zur Unterscheidung mitgegeben wurde.
    .recommendation = Jedem Landmark derselben Rolle einen eigenen, beschreibenden aria-label geben, z. B. aria-label="Hauptnavigation" und aria-label="Breadcrumb".
    .technical-note = aria-label oder aria-labelledby auf jedem der mehreren gleichrangigen Landmarks ergänzen. Die Namen müssen sich tatsächlich unterscheiden — reine Wiederholung derselben Bezeichnung erfüllt die Regel nicht.

explanation-keyboard-trap = Tastaturfalle nicht automatisch prüfbar
    .description = Ob der Tastaturfokus in einem Dialog, Karussell oder individuellen JavaScript-Bedienelement hängen bleiben kann, lässt sich nicht automatisiert feststellen. Das erfordert manuelles Navigieren ausschließlich mit der Tab-Taste.
    .user-impact = Personen, die ausschließlich mit der Tastatur oder einem Screenreader navigieren, können in einem Bedienelement stecken bleiben und die Seite nicht mehr per Tastatur verlassen.
    .typical-cause = Individuelle Dialoge, Karussells oder Widgets mit eigener JavaScript-Fokusverwaltung, die keinen Weg zurück per Escape-Taste, erreichbarem Schließen-Button oder dokumentiertem Tastaturkürzel vorsehen.
    .recommendation = Für jeden fokussierbaren Bereich einen Weg zum Verlassen per Tastatur sicherstellen: Escape-Taste, ein per Tab erreichbarer, sichtbarer Schließen-Button, oder ein dokumentiertes Tastaturkürzel.
    .technical-note = Nur mit Tab/Shift+Tab durch die Seite navigieren und prüfen, ob jeder Dialog/jedes Widget verlassen werden kann. Bei eigener Fokusverwaltung: Escape-Handler und einen erreichbaren Schließen-Button implementieren.

explanation-1-3-5 = Fehlende Eingabezweck-Kennzeichnung
    .description = Formularfelder haben keine maschinenlesbare Kennzeichnung ihres Zwecks. Browser und Hilfstechnologien können dadurch keine Autofill-Vorschläge machen und Nutzern nicht helfen, Formulare schneller auszufüllen.
    .user-impact = Menschen mit motorischen oder kognitiven Einschränkungen können nicht von automatischer Formularausfüllung profitieren. Das Ausfüllen dauert länger und ist fehleranfälliger.
    .typical-cause = Fehlende autocomplete-Attribute in Formularen für persönliche Daten (Name, E-Mail, Adresse, Telefon).
    .recommendation = Alle Formularfelder für persönliche Daten mit dem passenden autocomplete-Attribut versehen (z. B. autocomplete="email", autocomplete="given-name").
    .technical-note = autocomplete-Werte gemäß HTML-Spezifikation verwenden: name, email, tel, street-address, postal-code, country, etc.

explanation-1-4-1 = Links nur durch Farbe erkennbar
    .description = Links im Fließtext lassen sich ausschließlich durch ihre Farbe von normalem Text unterscheiden. Für Menschen mit Farbsehschwäche sind diese Links nicht als klickbar erkennbar.
    .user-impact = Nutzer mit Rot-Grün-Schwäche oder anderen Farbsehschwächen können Links im Text nicht erkennen und verpassen so wichtige Navigationsmöglichkeiten.
    .typical-cause = CSS setzt `text-decoration: none` auf Links im Fließtext ohne ein alternatives nicht-farbliches Unterscheidungsmerkmal wie Unterstrich, Fettschrift oder ein Icon.
    .recommendation = Links im Fließtext mit einem nicht-farblichen Merkmal kennzeichnen: Unterstrich (Standard und empfohlen), Fettschrift oder ein kleines Icon. Der Unterstrich ist die stärkste Konvention.
    .technical-note = CSS: `a {"{"} text-decoration: underline; {"}"}` oder bei `text-decoration: none` mindestens `font-weight: bold` oder `border-bottom`. Gilt nur für Links im Fließtext, nicht für Links in Navigationselementen.

explanation-1-4-3 = Unzureichender Farbkontrast
    .description = Text auf der Website hat nicht genügend Kontrast zum Hintergrund. Bei ungünstigen Lichtverhältnissen oder für Menschen mit Sehschwäche ist der Text schwer lesbar.
    .user-impact = Menschen mit Sehbeeinträchtigung, ältere Nutzer und alle Nutzer bei ungünstigen Bildschirmbedingungen (Sonnenlicht, schlechte Displays) können Texte schlecht oder gar nicht lesen.
    .typical-cause = Helle Schriftfarbe auf hellem Hintergrund, graue Texte auf weißem Grund, Designentscheidungen ohne Kontrastprüfung.
    .recommendation = Schriftfarben anpassen, sodass normaler Text mindestens ein Kontrastverhältnis von 4,5:1 und großer Text von 3:1 zum Hintergrund hat. Kontrastwerte mit Tools wie dem WebAIM Contrast Checker prüfen.
    .technical-note = WCAG 2.1 Level AA: 4.5:1 für normalen Text (<18pt / <14pt bold), 3:1 für großen Text (≥18pt / ≥14pt bold). CSS-Variablen für konsistente Farbpalette verwenden.

explanation-1-4-4 = Text nicht ausreichend skalierbar
    .description = Texte auf der Website können nicht ohne Verlust von Inhalt oder Funktion auf 200% vergrößert werden. Nutzer, die auf größere Schrift angewiesen sind, verlieren dadurch Inhalte.
    .user-impact = Menschen mit Sehbeeinträchtigung, die die Schriftgröße im Browser vergrößern, stoßen auf abgeschnittene Texte, überlagerte Elemente oder nicht scrollbare Bereiche.
    .typical-cause = Feste Pixelwerte für Schriftgrößen und Container, overflow: hidden bei Textcontainern, fehlende responsive Anpassungen.
    .recommendation = Schriftgrößen in relativen Einheiten (rem/em) angeben. Container so gestalten, dass sie bei vergrößertem Text mitwachsen. Browser-Zoom auf 200% testen.
    .technical-note = font-size in rem/em statt px. Container mit min-height statt height. overflow: auto statt hidden. @media-Queries für verschiedene Zoom-Stufen.

explanation-1-4-10 = Kein automatischer Textumbruch (Reflow)
    .description = Die Website passt sich bei starker Vergrößerung oder schmalen Bildschirmen (320 CSS-Pixeln) nicht flexibel an. Nutzer müssen horizontal scrollen, um Text zu lesen.
    .user-impact = Menschen mit Sehbehinderungen, die stark zoomen (bis 400%), müssen in zwei Dimensionen scrollen. Das Lesen langer Texte wird extrem mühsam oder unmöglich.
    .typical-cause = Verwendung von festen Pixelbreiten (width: 960px) oder overflow: hidden, was den Umbruch von Texten verhindert.
    .recommendation = Nutze responsives Design mit relativen Einheiten (%, vw, rem). Vermeide feste Breiten auf Containern, die Text enthalten. Teste Zoom auf 400%.
    .technical-note = WCAG 2.1 Level AA: No loss of content or function at width 320px (horizontal scroll must be avoided for text-flow containers). Use max-width: 100% and flexbox/grid.

explanation-1-4-11 = Unzureichender Kontrast bei UI-Elementen
    .description = Bedienelemente wie Buttons, Eingabefelder oder Icons haben nicht genügend Kontrast zum Hintergrund. Sie sind dadurch für manche Nutzer schwer erkennbar.
    .user-impact = Nutzer mit eingeschränktem Sehvermögen können interaktive Elemente nicht zuverlässig erkennen und bedienen.
    .typical-cause = Subtile Rahmenfarben bei Formularelementen, Icons mit geringem Kontrast, Fokus-Indikatoren, die zu unauffällig sind.
    .recommendation = Alle interaktiven Elemente und ihre Zustände (Normal, Hover, Fokus) mit mindestens 3:1 Kontrast zum Hintergrund gestalten.
    .technical-note = 3:1 Kontrastverhältnis für UI-Komponenten und grafische Objekte. Gilt für Rahmen, Icons, Slider, Checkboxen, etc.

explanation-1-4-12 = Inhalt wird bei größerem Zeilen-/Wortabstand abgeschnitten
    .description = Wenn Nutzer den Zeilen-, Wort- oder Buchstabenabstand für bessere Lesbarkeit erhöhen, schneidet ein Container mit fester Höhe oder overflow:hidden Teile des Textes ab.
    .user-impact = Nutzer mit Leseschwäche oder Sehbeeinträchtigung, die größere Abstände zur besseren Lesbarkeit einstellen, verlieren Inhalte oder können sie nicht mehr vollständig lesen.
    .typical-cause = Fixe Höhen oder Breiten mit overflow:hidden auf Textcontainern, etwa bei Karten, Buttons oder Navigationselementen mit knapp bemessenem Platz.
    .recommendation = Textcontainer flexibel gestalten (min-height statt fester Höhe, kein overflow:hidden auf Textbereichen) und mit den WCAG-Mindestabständen testen.
    .technical-note = WCAG-1.4.12-Mindestwerte simulieren (Zeilenhöhe 1.5x, Absatzabstand 2x, Buchstabenabstand 0.12x, Wortabstand 0.16x der Schriftgröße) und prüfen, ob Container Inhalte abschneiden statt zu wachsen.

explanation-1-4-13 = Fehlende Barrierefreiheit bei Zusatzinhalten (Tooltips)
    .description = Zusatzinhalte, die erst bei Hover (Mauszeiger) oder Fokus (Tastatur) erscheinen (wie Tooltips), sind nicht barrierefrei zugänglich oder unvollständig verknüpft.
    .user-impact = Nutzer von Screenreadern hören diese Inhalte gar nicht, Tastaturnutzer können sie oft nicht schließen, und auf Touchgeräten (Mobilgeräten) sind sie in der Regel unbrauchbar.
    .typical-cause = Unvollständige Zuweisung per aria-describedby oder fehlende Tastaturbedienbarkeit bei Custom-Tooltips.
    .recommendation = Verwende eine sichtbare Beschriftung oder moderne, barrierefreie Tooltips (schließbar mit Escape, verknüpft über aria-describedby und hoverbar).
    .technical-note = Zusatzinhalte müssen (1) hoverbar sein (Maus kann in den Tooltip bewegt werden), (2) dismissible sein (Esc-Taste blendet ihn aus), (3) persistent sein. Tooltips mit role="tooltip" per aria-describedby verknüpfen.

## 2. Operable

explanation-2-1-1 = Inhalte nicht per Tastatur bedienbar
    .description = Bestimmte Funktionen der Website können nur mit der Maus bedient werden. Nutzer, die auf die Tastatur angewiesen sind, können diese Inhalte nicht erreichen oder nutzen.
    .user-impact = Menschen mit motorischen Einschränkungen, die keine Maus verwenden können, sowie Screenreader-Nutzer sind von diesen Funktionen ausgeschlossen.
    .typical-cause = Click-Handler nur auf div/span statt auf interaktive Elemente, fehlende tabindex-Attribute, JavaScript-Widgets ohne Tastaturunterstützung.
    .recommendation = Alle interaktiven Elemente mit nativen HTML-Elementen (button, a, input) umsetzen. Bei Custom-Widgets Tastaturnavigation (Tab, Enter, Escape, Pfeiltasten) implementieren.
    .technical-note = Native HTML-Elemente bevorzugen. Bei Custom-Widgets: tabindex="0", keydown/keyup-Handler, ARIA-Rollen und -Zustände.

explanation-focusable-no-role = Fokussierbares Element ohne interaktive Rolle
    .description = Ein Element kann per Tastatur fokussiert werden, trägt aber keine (oder eine nicht-interaktive) ARIA-Rolle. Assistive Technologien wissen dadurch nicht, was beim Fokussieren dieses Elements möglich ist.
    .user-impact = Screenreader-Nutzer hören beim Fokussieren keine oder eine irreführende Rollenangabe (z. B. "Bereich" statt "Dialog" oder "Schaltfläche") und können nicht einschätzen, welche Interaktion erwartet wird.
    .typical-cause = Ein Container — etwa ein Consent-Banner oder ein Drittanbieter-Widget — erhält technisch bedingt Tastaturfokus, ohne dass ihm eine passende ARIA-Rolle oder ein natives interaktives Element zugrunde liegt.
    .recommendation = Dem Element eine passende ARIA-Rolle geben, die seine tatsächliche Funktion beschreibt, oder es durch ein natives interaktives HTML-Element ersetzen. Ein Element lediglich fokussierbar zu machen, ohne seine Funktion für Screenreader erkennbar zu machen, behebt das Problem nicht.
    .technical-note = Prüfen, warum das Element fokussierbar ist (tabindex, Drittanbieter-Skript). Ist es tatsächlich interaktiv: passende Rolle (z. B. role="dialog", role="button") und zugänglichen Namen ergänzen. Hat es keine eigene Interaktion: Fokussierbarkeit entfernen (tabindex="-1" oder tabindex-Attribut löschen).

explanation-2-1-2 = Tastaturfalle — Fokus kann nicht verlassen werden
    .description = An bestimmten Stellen der Website bleibt der Tastaturfokus hängen. Nutzer können mit der Tastatur nicht mehr vor- oder zurücknavigieren.
    .user-impact = Tastaturnutzer sind an dieser Stelle gefangen und können den Rest der Seite nicht mehr erreichen — die Seite wird faktisch unbenutzbar.
    .typical-cause = Modale Dialoge ohne korrekte Fokus-Verwaltung, Widgets, die den Fokus abfangen, fehlende Escape-Taste zum Verlassen.
    .recommendation = Sicherstellen, dass der Fokus jedes Element mit Tab und Shift+Tab verlassen kann. Modale Dialoge mit Escape schließbar machen und den Fokus danach korrekt zurücksetzen.
    .technical-note = Focus trapping nur in Modals mit korrekter Implementierung. Escape zum Schließen. Fokus-Rückgabe an das auslösende Element.

explanation-2-4-1 = Fehlende Sprungnavigation
    .description = Die Website bietet keinen Mechanismus, um wiederkehrende Inhaltsblöcke (z. B. Navigation, Header) zu überspringen und direkt zum Hauptinhalt zu gelangen.
    .user-impact = Tastaturnutzer und Screenreader-Nutzer müssen bei jedem Seitenwechsel erneut durch die gesamte Navigation tabben, bevor sie den Inhalt erreichen.
    .typical-cause = Fehlender Skip-Link als erstes Element der Seite. WCAG 2.4.1 verlangt einen Mechanismus zum Überspringen wiederkehrender Blöcke — nicht zwingend Landmarks, aber ein sichtbarer Skip-Link ist die direkteste Lösung.
    .recommendation = Einen sichtbaren Skip-Link ('Zum Inhalt springen') als erstes interaktives Element der Seite einbauen. HTML5-Landmarks (<nav>, <main>) sind ergänzend sinnvoll, ersetzen aber den Skip-Link nicht.
    .technical-note = Skip-Link: <a href="#main" class="skip-link">Zum Inhalt springen</a>. Bei Fokus sichtbar machen via CSS :focus {"{"} clip: auto; position: static; {"}"}. Landmarks (<nav>, <main id="main">) verbessern zusätzlich die Screenreader-Navigation.

explanation-2-4-2 = Fehlender oder unzureichender Seitentitel
    .description = Die Seite hat keinen aussagekräftigen Titel im Browser-Tab. Nutzer können nicht erkennen, auf welcher Seite sie sich befinden.
    .user-impact = Screenreader-Nutzer hören den Seitentitel als Erstes — ohne klaren Titel fehlt die Orientierung. Auch bei vielen offenen Tabs ist die Seite nicht zuzuordnen.
    .typical-cause = Leeres <title>-Tag, generischer Titel ('Home', 'Untitled'), identischer Titel auf allen Seiten.
    .recommendation = Jeder Seite einen eindeutigen, beschreibenden Titel geben, der den Seiteninhalt und die Website-Zugehörigkeit klar macht (z. B. 'Kontakt — Firmenname').
    .technical-note = <title>Seiteninhalt — Website</title>. Titel im CMS als Pflichtfeld. Pattern: Seitenspezifisch + Seitenname.

explanation-2-4-3 = Unlogische Fokus-Reihenfolge
    .description = Die Reihenfolge, in der interaktive Elemente per Tastatur erreicht werden, entspricht nicht der visuellen oder logischen Reihenfolge der Seite.
    .user-impact = Tastaturnutzer erleben eine verwirrende Navigation — der Fokus springt zwischen unzusammenhängenden Bereichen hin und her.
    .typical-cause = Visuelles Layout per CSS umgeordnet ohne DOM-Reihenfolge anzupassen, positive tabindex-Werte, dynamisch eingefügte Elemente an falscher Stelle.
    .recommendation = Die DOM-Reihenfolge an die visuelle Reihenfolge anpassen. Auf positive tabindex-Werte verzichten. Bei dynamischen Inhalten den Fokus programmatisch steuern.
    .technical-note = tabindex nur 0 oder -1 verwenden. CSS-Order nicht für inhaltlich relevante Umordnungen nutzen. DOM-Reihenfolge = Lesereihenfolge.

explanation-2-4-4 = Unklare oder generische Linktexte
    .description = Links auf der Seite haben Texte wie 'hier', 'mehr', 'weiterlesen', die ohne Kontext nicht verständlich sind. Nutzer können nicht erkennen, wohin ein Link führt.
    .user-impact = Screenreader-Nutzer navigieren häufig über eine Linkliste — generische Texte wie 'mehr' sind dort völlig nichtssagend. Auch für alle anderen Nutzer ist die Orientierung erschwert.
    .typical-cause = Redaktionelle Gewohnheit, 'hier klicken' oder 'mehr erfahren' als Linktext zu verwenden. Teaser-Komponenten mit generischem 'Weiterlesen'.
    .recommendation = Linktexte so formulieren, dass sie auch ohne umgebenden Text verständlich sind (z. B. 'Leistungen im Bereich Webentwicklung' statt 'mehr erfahren').
    .technical-note = Sprechende Linktexte verwenden. Falls visuell knapper Text gewünscht: aria-label oder aria-labelledby für erweiterte Beschreibung.

explanation-links-new-window-unannounced = Neues Fenster ohne Ankündigung
    .description = Ein Link öffnet ein neues Fenster oder Tab (target="_blank"), ohne das im Linktext anzukündigen. WCAG 3.2.5 (AAA) verlangt, dass solche Kontextwechsel nur auf Anforderung geschehen; die Ankündigung im Linktext ist die übliche Lösung.
    .user-impact = Screenreader- und Lupennutzer merken den Fensterwechsel oft nicht; der Zurück-Button führt dann nicht zur vorherigen Seite.
    .typical-cause = Externe Links oder PDF-Downloads, die per target="_blank" geöffnet werden.
    .recommendation = Im Linktext ankündigen, z. B. "(öffnet in neuem Fenster)", oder auf target="_blank" verzichten.
    .technical-note = Ein sichtbares Symbol reicht nur mit Textalternative im zugänglichen Namen (Technik G201).

explanation-link-name-context = Generischer Linktext mit möglichem Kontextbezug
    .description = Der Linktext allein ist unspezifisch (z. B. "Start", "weiter"), steht aber neben beschreibendem Text, der den Linkzweck erklären könnte. WCAG 2.4.4 erlaubt es ausdrücklich, den Linkzweck aus dem Linktext zusammen mit seinem Kontext abzuleiten — ob das hier zutrifft, kann nur eine manuelle Prüfung bestätigen.
    .user-impact = Ob der Linkzweck für Screenreader-Nutzer verständlich ist, hängt davon ab, ob der umgebende Text tatsächlich vorgelesen wird, bevor der Link erreicht wird, und ob die Verbindung eindeutig ist. Ohne manuelle Prüfung ist unklar, ob Nutzer davon betroffen sind.
    .typical-cause = Teaser- oder Karten-Komponenten, in denen ein generischer Button- oder Linktext ("Start", "weiter") direkt auf eine Überschrift oder einen Beschreibungstext folgt.
    .recommendation = Manuell prüfen, ob der umgebende Text den Linkzweck visuell und für Screenreader eindeutig macht. Ist das der Fall, ist keine Änderung nötig. Andernfalls den Linktext selbst aussagekräftig formulieren oder per aria-label/aria-labelledby ergänzen.
    .technical-note = Kontext zählt nur, wenn er programmatisch mit dem Link verknüpft ist oder ihm unmittelbar vorausgeht (gleicher Satz, gleiche Listenzelle, gleiche Überschrift). Bei Unsicherheit aria-labelledby auf das Kontext-Element setzen, um die Verbindung eindeutig zu machen.

explanation-2-4-6 = Fehlende oder unzureichende Überschriftenstruktur
    .description = Die Seite verwendet keine oder unlogische Überschriften. Die inhaltliche Gliederung ist für assistive Technologien nicht erkennbar.
    .user-impact = Screenreader-Nutzer können nicht per Überschrift durch die Seite navigieren. Die Seite wird zu einem undifferenzierten Textblock — schnelles Finden relevanter Abschnitte ist unmöglich.
    .typical-cause = Keine Überschriften verwendet, nur visuelle Formatierung (fett, groß). Überschriftenhierarchie übersprungen (z. B. H1 → H3). Mehrere oder fehlende H1.
    .recommendation = Eine logische Überschriftenhierarchie aufbauen: genau eine H1 pro Seite, darunter H2 für Hauptabschnitte, H3 für Unterabschnitte. Keine Ebenen überspringen.
    .technical-note = H1 = Seitentitel (einmal). H2-H6 hierarchisch verschachtelt. Keine Ebenen überspringen. Heading-Outline mit Browser-Extension prüfen.

explanation-2-4-7 = Fokus-Indikator nicht sichtbar
    .description = Beim Navigieren mit der Tastatur ist nicht erkennbar, welches Element gerade den Fokus hat. Der visuelle Fokus-Rahmen fehlt oder ist unsichtbar.
    .user-impact = Tastaturnutzer verlieren die Orientierung — sie wissen nicht, wo sie sich auf der Seite befinden und welches Element sie gerade aktivieren würden.
    .typical-cause = CSS-Reset entfernt outline (outline: none / outline: 0). Kein eigener Fokus-Stil definiert. Fokus-Stil zu unauffällig.
    .recommendation = Einen gut sichtbaren Fokus-Indikator für alle interaktiven Elemente gestalten. Mindestens 2px Umrandung mit ausreichendem Kontrast.
    .technical-note = :focus-visible statt :focus für Tastatur-only Fokus. outline: 2px solid #005fcc; outline-offset: 2px; Niemals outline: none ohne Alternative.

explanation-2-4-10 = Fehlende Abschnittsüberschriften
    .description = Längere Inhalte sind nicht durch Abschnittsüberschriften gegliedert. Die Seite wirkt als ein zusammenhängender Block ohne erkennbare Struktur.
    .user-impact = Nutzer können Inhalte nicht gezielt ansteuern und müssen lange Textpassagen komplett durchlesen, um relevante Informationen zu finden.
    .typical-cause = Lange Inhaltsseiten ohne Zwischenüberschriften. Redaktionelle Inhalte als Fließtext ohne Gliederung.
    .recommendation = Längere Inhalte mit aussagekräftigen Zwischenüberschriften gliedern. Alle 2-3 Absätze eine Überschrift einsetzen.
    .technical-note = Korrekte Heading-Hierarchie (H2, H3...) innerhalb von Abschnitten. WAI-ARIA: role="heading" nur als Fallback, native HTML bevorzugen.

explanation-2-5-3 = Sichtbarer Text stimmt nicht mit dem zugänglichen Namen überein
    .description = Der sichtbare Text eines Bedienelements stimmt nicht mit dem Namen überein, den assistive Technologien vorlesen. Sprachsteuerungsnutzer können das Element nicht ansprechen.
    .user-impact = Nutzer, die Sprachsteuerung verwenden, können Elemente nicht per Sprachbefehl aktivieren, weil der vorgelesene Name nicht dem sichtbaren Text entspricht.
    .typical-cause = aria-label überschreibt sichtbaren Text mit abweichendem Wortlaut. Sichtbarer Text und zugänglicher Name sind unterschiedlich formuliert.
    .recommendation = Sicherstellen, dass der zugängliche Name (aria-label) den sichtbaren Text enthält oder identisch ist.
    .technical-note = aria-label muss den sichtbaren Text als Teilstring enthalten. Besser: aria-label ganz weglassen, wenn der sichtbare Text ausreicht.

explanation-2-5-8 = Unzureichende Klickzielgröße
    .description = Bedienelemente wie Buttons, Links oder Icons haben eine Klickfläche von weniger als 24×24 CSS-Pixeln.
    .user-impact = Nutzer mit motorischen Einschränkungen oder auf Touchscreens treffen kleine Zielflächen leicht daneben und lösen versehentlich falsche Aktionen aus.
    .typical-cause = Icon-Buttons ohne zusätzliches Padding, dicht gepackte Link- oder Icon-Listen, kleine Formularsteuerelemente im mobilen Layout.
    .recommendation = Klickflächen auf mindestens 24×24 CSS-Pixel vergrößern, z. B. durch Padding, oder ausreichend Abstand zu benachbarten Zielen einhalten.
    .technical-note = Ausnahmen gelten u. a. für Inline-Links im Fließtext und wenn ein gleichwertiges, größeres Ziel für dieselbe Funktion existiert.

## 3. Understandable

explanation-3-1-1 = Fehlende Sprachangabe der Seite
    .description = Die Hauptsprache der Seite ist nicht im HTML-Code angegeben. Screenreader und Übersetzungstools können die Sprache nicht automatisch erkennen.
    .user-impact = Screenreader lesen den Text mit falscher Aussprache vor — deutsche Inhalte werden z. B. mit englischer Phonetik gelesen, was unverständlich ist.
    .typical-cause = Fehlendes lang-Attribut im <html>-Tag. Häufig bei Templates vergessen.
    .recommendation = Das lang-Attribut im <html>-Tag setzen (z. B. lang="de" für Deutsch). Bei mehrsprachigen Seiten zusätzlich Abschnitte mit lang-Attribut kennzeichnen.
    .technical-note = <html lang="de"> für deutschsprachige Seiten. ISO 639-1 Sprachcodes verwenden. Für fremdsprachige Abschnitte: <span lang="en">...</span>.

explanation-3-2-1 = Unerwartete Kontextänderung bei Fokus
    .description = Wenn ein Element den Tastaturfokus erhält, ändert sich unerwartet der Kontext (z. B. Seite wird gewechselt, neues Fenster öffnet sich).
    .user-impact = Tastaturnutzer und Screenreader-Nutzer erleben unvorhersehbare Seitenveränderungen. Das ist verwirrend und stört den Arbeitsfluss.
    .typical-cause = JavaScript-Events auf onfocus, die Navigation oder DOM-Änderungen auslösen. Select-Elemente, die bei Fokus bereits eine Aktion auslösen.
    .recommendation = Kontextänderungen nur durch explizite Benutzeraktionen auslösen (Klick, Enter), nicht durch bloßen Fokuswechsel.
    .technical-note = Keine onchange/onfocus-Handler für Navigation. Select-Menüs mit Submit-Button statt auto-submit.

explanation-3-2-2 = Unerwartete Kontextänderung bei Eingabe
    .description = Wenn ein Nutzer Eingaben in ein Formularelement macht oder eine Auswahl trifft, ändert sich unerwartet der Kontext der Seite.
    .user-impact = Nutzer erleben verwirrende Änderungen beim Ausfüllen von Formularen. Besonders für Screenreader-Nutzer sind unangekündigte Änderungen problematisch.
    .typical-cause = Auto-Submit bei Formularänderungen. Seitenweiterleitung bei Select-Auswahl ohne Bestätigung.
    .recommendation = Formularänderungen erst nach expliziter Bestätigung (Submit-Button) verarbeiten. Vorab ankündigen, wenn eine Eingabe sofortige Änderungen auslöst.
    .technical-note = Kein auto-submit bei onchange. Explizite Submit-Buttons verwenden. Alternativ: ARIA-Live-Region für Vorab-Hinweis.

explanation-3-3-2 = Fehlende Beschriftungen und Anleitungen bei Formularen
    .description = Formularfelder haben keine sichtbare Beschriftung oder Anleitung. Nutzer wissen nicht, welche Eingaben erwartet werden.
    .user-impact = Alle Nutzer, besonders Menschen mit kognitiven Einschränkungen und Screenreader-Nutzer, können Formulare nicht korrekt ausfüllen.
    .typical-cause = Fehlende <label>-Elemente, nur Platzhaltertext statt sichtbarer Labels, fehlende Hinweise auf Pflichtfelder oder Eingabeformat.
    .recommendation = Jedes Formularfeld mit einem sichtbaren, per <label> verknüpften Label versehen. Pflichtfelder kennzeichnen. Bei speziellen Formaten (z. B. Datum) das erwartete Format angeben.
    .technical-note = <label for="id"> für alle Formularfelder. Pflichtfelder: required + aria-required="true". Platzhalter nicht als einzige Beschriftung verwenden.

## 4. Robust

explanation-ids-duplicate = Mehrfach vergebene ID wird referenziert
    .description = Mehrere Elemente auf der Seite tragen dieselbe id, und ein anderes Element verweist darauf (z. B. label for, aria-labelledby). Welches der Elemente gemeint ist, ist damit nicht mehr bestimmbar.
    .user-impact = Screenreader lösen die Beziehung auf das falsche Element auf und geben einen falschen Namen oder eine falsche Beschreibung aus.
    .typical-cause = Wiederholt eingebundene Komponenten oder Templates (z. B. Menüpunkte, Widgets), die eine statische ID statt einer pro Instanz eindeutigen ID verwenden.
    .recommendation = Jede ID nur einmal im Dokument vergeben. Bei wiederverwendeten Komponenten die ID dynamisch generieren (z. B. mit Index oder Slug).
    .technical-note = IDs eindeutig machen, etwa durch Anhängen eines Index oder einer eindeutigen Kennung pro Komponenteninstanz.

explanation-4-1-2 = Fehlende Name/Rolle bei Bedienelementen
    .description = Interaktive Elemente (Buttons, Links, Formularfelder) haben keinen zugänglichen Namen oder keine erkennbare Rolle. Assistive Technologien können nicht vermitteln, worum es sich handelt.
    .user-impact = Screenreader-Nutzer hören z. B. nur 'Button' ohne Beschreibung der Funktion, oder ein klickbares Element wird gar nicht als interaktiv erkannt.
    .typical-cause = Buttons oder Links ohne Text oder aria-label. Icon-only-Buttons ohne zugänglichen Namen. Custom-Widgets ohne ARIA-Rollen.
    .recommendation = Alle interaktiven Elemente mit einem verständlichen, zugänglichen Namen versehen. Native HTML-Elemente bevorzugen. Bei Icon-Buttons: aria-label verwenden.
    .technical-note = Buttons: sichtbarer Text oder aria-label. Links: sprechender Linktext. Inputs: verknüpftes <label>. Custom-Widgets: role + aria-label + aria-Zustände.

explanation-keyboard-hidden-focusable = Tastaturfokus auf unsichtbaren Elementen
    .description = Ein interaktives Element ist über aria-hidden="true" vor Screenreadern versteckt, kann aber dennoch mit der Tastatur (Tabulator) fokussiert werden.
    .user-impact = Tastaturnutzer steuern auf unsichtbare Elemente (Fokus-Stolperfallen), während Screenreader-Nutzer keinerlei Rückmeldung über dieses Element erhalten.
    .typical-cause = Es wurde aria-hidden="true" verwendet, um ein Element visuell auszublenden, ohne tabindex="-1" zu setzen oder das Element per display:none zu verstecken.
    .recommendation = Wenn ein Element vollständig versteckt sein soll, display:none oder visibility:hidden verwenden. Soll es nur vor Screenreadern versteckt sein, muss es unfokussierbar sein (tabindex="-1").
    .technical-note = Focusable elements (a, button, input) inside an aria-hidden="true" tree must either be removed from tab order or hidden completely.

explanation-aria-attribute-prohibited = Ungültige oder verbotene ARIA-Attribute
    .description = Für bestimmte HTML-Elemente oder ARIA-Rollen werden Attribute verwendet, die für diese Rolle nicht zulässig oder sinnvoll sind.
    .user-impact = Screenreader erhalten widersprüchliche oder fehlerhafte Signale, was die Bedienbarkeit der Website beeinträchtigen kann.
    .typical-cause = Verwendung von ARIA-Attributen (wie aria-label oder aria-expanded) auf Elementen, die keine interaktive Rolle haben (z. B. span oder div ohne role).
    .recommendation = Prüfe die ARIA-Spezifikation für die jeweilige Rolle. Verwende ARIA-Attribute nur auf passenden Elementen und Rollen.
    .technical-note = Prohibited attributes on generic elements: <span aria-label="text"> is invalid. Use native elements or valid roles (e.g. role="button").

explanation-modern-attribute-misuse = Fehlerhafte Dialog- oder Popover-Steuerung
    .description = Moderne HTML-Attribute für Popover, Dialoge oder gesperrte Bereiche sind unvollständig oder widersprüchlich eingesetzt.
    .user-impact = Tastatur- und Screenreader-Nutzer können Oberflächen öffnen, fokussieren oder verlassen, ohne eine verständliche Rückmeldung zu erhalten.
    .typical-cause = popovertarget verweist auf ein fehlendes Element, ein sichtbarer Dialog hat keinen zugänglichen Namen, oder ein aktiver Bereich ist gleichzeitig inert markiert.
    .recommendation = Popover-Ziele prüfen, sichtbare Dialoge und Menüs benennen und inert nur für tatsächlich inaktive Bereiche verwenden.
    .technical-note = popovertarget muss auf ein Element mit popover zeigen. Offene Dialoge, Menüs und Popover brauchen aria-label, aria-labelledby oder einen geeigneten Namen.

## SEO rules (no WCAG criterion)

explanation-seo-headings-long-heading = Überschrift zu lang
    .description = Überschriften auf der Seite überschreiten die empfohlene Länge von 70 Zeichen. Lange Überschriften werden in Suchergebnissen abgeschnitten und sind für Leser schwerer zu erfassen.
    .user-impact = In Suchergebnissen und sozialen Medien werden die Überschriften abgeschnitten, was den Klickanreiz verringert. Leser müssen mehr kognitive Arbeit leisten, um den Inhalt einzuordnen.
    .typical-cause = Redaktionelle Texte werden ungekürzt als Überschrift eingesetzt. CMS-Felder für Überschriften haben keine Längenbeschränkung.
    .recommendation = Überschriften auf unter 70 Zeichen kürzen und den Kern der Aussage voranstellen. Längere Beschreibungen gehören in den Fließtext.
    .technical-note = Google zeigt typischerweise 50–60 Zeichen im Title-Tag an. Für H2/H3 gilt keine feste Grenze, aber über 70 Zeichen sinkt die Lesbarkeit deutlich. CMS-Validierung empfohlen.

explanation-seo-headings-skipped-level = Übersprungene Überschriftenebene
    .description = Die Überschriften auf der Seite überspringen eine Ebene — zum Beispiel folgt auf eine H1 direkt eine H3, ohne dass eine H2 dazwischen steht. Das zerstört die logische Hierarchie, die Suchmaschinen und Screenreader zur Navigation nutzen.
    .user-impact = Screenreader-Nutzer navigieren Seiten über Überschriften. Fehlende Ebenen unterbrechen diesen Pfad und erschweren die Orientierung. Suchmaschinen bewerten Seiten mit konsistenter Überschriftenstruktur als inhaltlich hochwertiger.
    .typical-cause = CMS-Templates oder Komponenten setzen fest kodierte Überschriftenebenen (z. B. immer H3 in Footer-Spalten), unabhängig davon, was der Seitenkontext erfordert. Entwickler wählen Ebenen nach visuellem Stil statt nach Hierarchie.
    .recommendation = Überschriftenhierarchie von H1 abwärts lückenlos einhalten: H1 → H2 → H3. Wenn eine Überschrift visuell kleiner wirken soll, CSS-Klassen verwenden statt die HTML-Ebene zu ändern.
    .technical-note = WCAG 1.3.1 (Info and Relationships, Level A) erfordert, dass Struktur programmatisch erkennbar ist. Übersprungene Ebenen verletzen diese Anforderung. Screenreader wie NVDA/JAWS springen mit H-Tasten — Lücken erzeugen tote Punkte.

explanation-seo-headings-missing-h1 = Fehlende H1-Überschrift
    .description = Der Seite fehlt eine H1-Überschrift. Ohne sie ist der Seitenzweck für Suchmaschinen und Screenreader-Nutzer nicht auf den ersten Blick erkennbar.
    .user-impact = Screenreader-Nutzer, die sich per Überschriftenliste orientieren, finden keinen Einstiegspunkt, der das Hauptthema der Seite benennt.
    .typical-cause = Das Seitentemplate setzt keine H1, oder ein Baustein, der normalerweise die H1 liefert, wurde entfernt oder falsch konfiguriert.
    .recommendation = Jeder Seite genau eine H1 geben, die das Hauptthema benennt und möglichst weit oben im Inhalt steht.
    .technical-note = Für sich kein WCAG-Verstoß: WCAG 2.2 verlangt in 1.3.1 (Technik H42) und 2.4.6, dass Überschriften als solche ausgezeichnet sind und ihr Thema beschreiben, nicht eine H1. Eine H1 für das Seitenthema ist eine verbreitete Empfehlung für Orientierung und Suchmaschinen.

explanation-seo-headings-multiple-h1 = Mehrere H1-Überschriften
    .description = Die Seite enthält mehr als eine H1-Überschrift. Das ist zulässig, kann aber den Hauptfokus der Seite unklar machen, wenn nicht erkennbar ist, welche H1 das Seitenthema benennt.
    .user-impact = Wer per Überschriftenliste navigiert, sieht mehrere Überschriften der obersten Ebene. Ob das stört, hängt davon ab, ob sie das Seitenthema und sinnvolle Hauptbereiche bezeichnen — das lässt sich nur von Hand beurteilen.
    .typical-cause = Ein wiederverwendbarer Baustein (z. B. ein Hero- oder Logo-Bereich) setzt selbst eine H1, unabhängig davon, ob die Seite bereits eine eigene hat.
    .recommendation = Prüfen, ob die Überschriften den Inhalt sinnvoll gliedern. Empfohlen ist eine H1 für das Seitenthema; wiederverwendbare Bausteine sollten ihre Überschriftenebene konfigurierbar machen statt sie fest auf H1 zu setzen.
    .technical-note = Für sich kein WCAG-Verstoß: WCAG 2.2 (1.3.1, Technik H42; 2.4.6) und HTML lassen mehrere H1 zu. Ein Verstoß entsteht erst, wenn die Hierarchie die Gliederung falsch wiedergibt. Eine H1 pro Seite ist eine Empfehlung für Orientierung und Suchmaschinen.

explanation-seo-headings-empty-heading = Leere Überschrift
    .description = Eine Überschrift auf der Seite enthält keinen Text. Leere Überschriften erzeugen Navigationsprobleme für Screenreader-Nutzer und werden von Suchmaschinen als schwaches Signal gewertet.
    .user-impact = Screenreader-Nutzer, die per Überschrift springen, landen auf einem unbenannten Punkt ohne erkennbaren Inhalt.
    .typical-cause = Eine Komponente rendert ihr Überschriften-Element unabhängig davon, ob ein Titel-Feld im CMS befüllt wurde, oder der Text wird ausschließlich über ein Hintergrundbild statt echten Text vermittelt.
    .recommendation = Leere Überschriften-Elemente entfernen oder mit echtem, beschreibendem Text befüllen. Komponenten sollten das Überschriften-Element nur rendern, wenn ein Titel vorhanden ist.
    .technical-note = WCAG 1.3.1 (Info and Relationships, Level A) und 2.4.6 (Headings and Labels, Level AA) erfordern, dass Überschriften einen erkennbaren, programmatisch verfügbaren Text tragen.

## Rule-specific texts replacing the WCAG-criterion fallback

explanation-server-side-image-map = Server-seitige Image-Map verwendet
    .description = Ein Bild nutzt eine server-seitige Image-Map (ismap-Attribut) statt einer client-seitigen Variante.
    .user-impact = Tastatur- und Screenreader-Nutzer können die einzelnen Bildbereiche nicht ansteuern.
    .typical-cause = ismap-Attribut auf einem Bild ohne äquivalente clientseitige Alternative.
    .recommendation = Die server-seitige Image-Map durch eine client-seitige Image-Map (<map>/<area>) oder einfache Textlinks ersetzen.
    .technical-note = ismap-Attribut auf einem Bild ohne äquivalente clientseitige Alternative.

explanation-svg-img-alt = Fehlender Alternativtext bei SVG-Grafiken
    .description = Eine SVG-Grafik mit informativem Inhalt hat keinen Alternativtext (title, aria-label).
    .user-impact = Screenreader-Nutzer erhalten keine Information über den Inhalt der Grafik.
    .typical-cause = SVG ohne <title>-Kindelement, aria-label oder aria-labelledby.
    .recommendation = Ein <title> als erstes Kindelement des <svg> ergänzen, aria-label verwenden, oder rein dekorative SVGs mit aria-hidden="true" verstecken.
    .technical-note = SVG ohne <title>-Kindelement, aria-label oder aria-labelledby.

explanation-area-alt = Fehlender Alternativtext bei Image-Map-Bereichen
    .description = Ein aktiver <area>-Bereich einer Image-Map hat keinen Alternativtext.
    .user-impact = Screenreader-Nutzer erfahren nicht, wohin ein anklickbarer Bildbereich führt.
    .typical-cause = <area>-Element ohne alt-Attribut.
    .recommendation = Ein alt-Attribut auf dem <area>-Element ergänzen, das das Ziel beschreibt.
    .technical-note = <area>-Element ohne alt-Attribut.

explanation-input-image-alt = Fehlender Alternativtext bei Bild-Buttons
    .description = Ein <input type="image">-Button (z. B. eine Bild-Absende-Schaltfläche) hat keinen Alternativtext.
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Aktion der Button auslöst.
    .typical-cause = <input type="image"> ohne alt-Attribut.
    .recommendation = Ein alt-Attribut auf dem <input type="image">-Element ergänzen.
    .technical-note = <input type="image"> ohne alt-Attribut.

explanation-object-alt = Fehlende Textalternative bei eingebetteten Objekten
    .description = Ein <object>-Element (eingebettetes Bild/Dokument) hat keine Textalternative.
    .user-impact = Screenreader-Nutzer erhalten keine Information über den Inhalt des eingebetteten Objekts.
    .typical-cause = <object> ohne Textinhalt oder aria-label.
    .recommendation = Eine Textalternative innerhalb des <object>-Elements oder via aria-label bereitstellen.
    .technical-note = <object> ohne Textinhalt oder aria-label.

explanation-html-content-model = Verletzung des HTML5-Content-Models
    .description = Ein strukturelles Element (Liste, Tabelle, Formular, Select) enthält Inhalt, der sein HTML5-Content-Model verletzt — das kann Screenreader-Semantik brechen, selbst wenn der resultierende Accessibility-Tree oberflächlich gültig aussieht.
    .user-impact = Screenreader können die Struktur nicht zuverlässig auswerten, auch wenn sie augenscheinlich korrekt wirkt.
    .typical-cause = Unerwartetes Kindelement innerhalb eines strukturellen Containers (z. B. <ul>, <table>, <select>).
    .recommendation = Das Markup an das HTML5-Content-Model des Elements anpassen (z. B. darf ein <dl> nur direkt <dt>/<dd>-Gruppen, <div>, <script> oder <template> enthalten). Der Browser kann ungültige Kindelemente beim Aufbau des Accessibility-Trees still 'heilen' — der Defekt bleibt für rollenbasierte Prüfungen unsichtbar, verwirrt assistierende Technologie aber trotzdem.
    .technical-note = Unerwartetes Kindelement innerhalb eines strukturellen Containers (z. B. <ul>, <table>, <select>).

explanation-form-field-group = Fehlende Gruppierung verwandter Formularfelder
    .description = Zusammengehörige Radio-Buttons oder Checkboxen sind nicht mit <fieldset>/<legend> oder role="group" gruppiert.
    .user-impact = Screenreader-Nutzer erkennen nicht, dass mehrere Felder zu einer Frage/Gruppe gehören.
    .typical-cause = Radio-Buttons/Checkboxen ohne umschließendes <fieldset> mit <legend>.
    .recommendation = Zusammengehörige Radio-Buttons oder Checkboxen in ein <fieldset> mit <legend> einbetten, oder role="group" mit aria-labelledby verwenden.
    .technical-note = Radio-Buttons/Checkboxen ohne umschließendes <fieldset> mit <legend>.

explanation-landmark-banner-is-top-level = Banner-Landmark ist verschachtelt
    .description = Der Banner-Bereich (<header>/role="banner") liegt innerhalb eines anderen Landmarks statt auf oberster Ebene.
    .user-impact = Screenreader-Nutzer, die per Landmark-Navigation zum Kopfbereich springen, landen in einem unerwarteten Kontext.
    .typical-cause = <header>/role="banner" ist Nachfahre eines anderen Landmarks (z. B. <main>).
    .recommendation = Das <header>-/role="banner"-Element auf die oberste Ebene der Seite verschieben, außerhalb jedes anderen Landmarks.
    .technical-note = <header>/role="banner" ist Nachfahre eines anderen Landmarks (z. B. <main>).

explanation-landmark-contentinfo-is-top-level = Contentinfo-Landmark ist verschachtelt
    .description = Der Fußbereich (<footer>/role="contentinfo") liegt innerhalb eines anderen Landmarks statt auf oberster Ebene.
    .user-impact = Screenreader-Nutzer, die per Landmark-Navigation zum Fußbereich springen, landen in einem unerwarteten Kontext.
    .typical-cause = <footer>/role="contentinfo" ist Nachfahre eines anderen Landmarks.
    .recommendation = Das <footer>-/role="contentinfo"-Element auf die oberste Ebene der Seite verschieben, außerhalb jedes anderen Landmarks.
    .technical-note = <footer>/role="contentinfo" ist Nachfahre eines anderen Landmarks.

explanation-landmark-main-is-top-level = Main-Landmark ist verschachtelt
    .description = Der Hauptinhaltsbereich (<main>/role="main") liegt innerhalb eines anderen Landmarks statt auf oberster Ebene.
    .user-impact = Screenreader-Nutzer, die per Landmark-Navigation zum Hauptinhalt springen, landen in einem unerwarteten Kontext.
    .typical-cause = <main>/role="main" ist Nachfahre eines anderen Landmarks.
    .recommendation = Das <main>-/role="main"-Element auf die oberste Ebene der Seite verschieben, außerhalb jedes anderen Landmarks.
    .technical-note = <main>/role="main" ist Nachfahre eines anderen Landmarks.

explanation-landmark-no-duplicate-banner = Mehrere Banner-Landmarks
    .description = Die Seite hat mehr als einen Banner-Bereich (<header>/role="banner"); erlaubt ist höchstens einer.
    .user-impact = Screenreader-Nutzer können den 'Kopfbereich' der Seite nicht eindeutig zuordnen.
    .typical-cause = Mehr als ein Element mit role="banner" (implizit oder explizit).
    .recommendation = Sicherstellen, dass die Seite höchstens ein <header>-/role="banner"-Element hat.
    .technical-note = Mehr als ein Element mit role="banner" (implizit oder explizit).

explanation-landmark-no-duplicate-contentinfo = Mehrere Contentinfo-Landmarks
    .description = Die Seite hat mehr als einen Fußbereich (<footer>/role="contentinfo"); erlaubt ist höchstens einer.
    .user-impact = Screenreader-Nutzer können den 'Fußbereich' der Seite nicht eindeutig zuordnen.
    .typical-cause = Mehr als ein Element mit role="contentinfo" (implizit oder explizit).
    .recommendation = Sicherstellen, dass die Seite höchstens ein <footer>-/role="contentinfo"-Element hat.
    .technical-note = Mehr als ein Element mit role="contentinfo" (implizit oder explizit).

explanation-landmark-no-duplicate-main = Mehrere Main-Landmarks
    .description = Die Seite hat mehr als einen Hauptinhaltsbereich (<main>/role="main"); erlaubt ist genau einer.
    .user-impact = Screenreader-Nutzer, die zum Hauptinhalt springen, wissen nicht, welcher Bereich gemeint ist.
    .typical-cause = Mehr als ein Element mit role="main" (implizit oder explizit).
    .recommendation = Sicherstellen, dass die Seite genau ein <main>-/role="main"-Element hat.
    .technical-note = Mehr als ein Element mit role="main" (implizit oder explizit).

explanation-landmark-banner-present = Fehlender Banner-Landmark
    .description = Die Seite hat keinen Banner-Bereich (<header> bzw. role="banner").
    .user-impact = Assistierende Technologien können den Kopfbereich der Seite nicht identifizieren.
    .typical-cause = Kein Element mit role="banner" (implizit über <header> oder explizit) vorhanden.
    .recommendation = Ein <header>-Element auf oberster Ebene (oder ein Element mit role="banner") ergänzen, das den Seitenkopf umschließt.
    .technical-note = Kein Element mit role="banner" (implizit über <header> oder explizit) vorhanden.

explanation-label-title-only = Formularfeld-Beschriftung nur über title-Attribut
    .description = Ein Formularfeld bezieht seinen zugänglichen Namen ausschließlich aus dem title-Attribut statt aus einem sichtbaren Label.
    .user-impact = Das title-Attribut wird von vielen Screenreadern inkonsistent oder gar nicht vorgelesen; sehende Nutzer sehen es nur bei Hover.
    .typical-cause = Formularelement ohne <label>/aria-label, Name kommt nur aus title.
    .recommendation = Ein sichtbares <label>-Element oder aria-label statt des title-Attributs verwenden.
    .technical-note = Formularelement ohne <label>/aria-label, Name kommt nur aus title.

explanation-click-events-have-key-events = Klick-Handler ohne Tastatur-Äquivalent
    .description = Ein nicht-interaktives Element (z. B. <div>) hat einen Klick-Handler, aber keinen entsprechenden Tastatur-Event-Handler.
    .user-impact = Tastaturnutzer können die Funktion nicht auslösen.
    .typical-cause = onclick auf einem <div>/<span> ohne onkeydown/onkeyup, tabindex oder passende Rolle.
    .recommendation = Einen keydown/keyup-Handler ergänzen, der auf Enter/Leertaste reagiert, tabindex="0" und eine passende ARIA-Rolle setzen, oder stattdessen ein natives <button>-Element verwenden.
    .technical-note = onclick auf einem <div>/<span> ohne onkeydown/onkeyup, tabindex oder passende Rolle.

explanation-landmark-one-main = Kein Hauptinhaltsbereich als Sprungziel
    .description = Die Seite hat keinen <main>-Bereich, der als Sprungziel zum Überspringen wiederkehrender Blöcke dienen könnte.
    .user-impact = Tastaturnutzer haben kein verlässliches Ziel, um wiederkehrende Navigation zu überspringen.
    .typical-cause = Kein Element mit role="main" (implizit über <main> oder explizit) vorhanden.
    .recommendation = Ein <main>-Element oder ein Element mit role="main" ergänzen, damit Tastaturnutzer ein verlässliches Sprungziel haben.
    .technical-note = Kein Element mit role="main" (implizit über <main> oder explizit) vorhanden.

explanation-skip-link = Fehlender Skip-Link
    .description = Die Seite hat eine Navigation, aber keinen Skip-Link, um sie zu überspringen und direkt zum Inhalt zu springen.
    .user-impact = Tastaturnutzer müssen bei jedem Seitenwechsel durch die gesamte Navigation tabben.
    .typical-cause = Kein Link mit erkennbarem 'skip'/'zum Inhalt'-Text oder Anker auf #main/#content am Seitenanfang.
    .recommendation = Einen visuell versteckten oder sichtbaren Link am Seitenanfang ergänzen, der auf den Hauptinhalt verweist, z. B. <a href="#main">Zum Inhalt springen</a>.
    .technical-note = Kein Link mit erkennbarem 'skip'/'zum Inhalt'-Text oder Anker auf #main/#content am Seitenanfang.

explanation-focus-visible-outline-none = Fokus-Indikator per CSS entfernt
    .description = CSS entfernt den sichtbaren Fokusrahmen (outline: none/0) ohne einen gleichwertigen Ersatz bereitzustellen.
    .user-impact = Tastaturnutzer sehen nicht, welches Element gerade fokussiert ist.
    .typical-cause = outline: none/0 in :focus-Regeln ohne alternative sichtbare Fokusmarkierung.
    .recommendation = Den Standard-Fokusrahmen (outline: none/0) nicht entfernen, ohne einen gleichwertigen sichtbaren Ersatz bereitzustellen, z. B. einen eigenen Outline/Box-Shadow auf :focus-visible.
    .technical-note = outline: none/0 in :focus-Regeln ohne alternative sichtbare Fokusmarkierung.

explanation-scrollable-region-focusable = Scrollbarer Bereich per Tastatur nicht erreichbar
    .description = Ein Bereich mit eigenem Scrollbalken lässt sich nur mit Maus oder Touch scrollen: Er ist nicht fokussierbar und enthält nichts, das den Fokus hineinholt.
    .user-impact = Tastaturnutzer erreichen den Inhalt unterhalb des sichtbaren Ausschnitts nicht.
    .typical-cause = Feste Höhe mit overflow: auto an Menü-, Text- oder Tabellencontainern, die nur Text enthalten.
    .recommendation = Den Bereich mit tabindex="0" fokussierbar machen und ihm einen Namen geben (role="region" mit aria-label), oder die feste Höhe entfernen.
    .technical-note = Wie axe-core scrollable-region-focusable: Überlauf von mehr als 13 px bei overflow: auto/scroll, sichtbarer Textinhalt, weder das Element noch ein Nachfahre per Tastatur fokussierbar. Chrome macht solche Bereiche seit Version 130 selbst fokussierbar, andere Browser nicht.

explanation-input-error-message = Ungültiges Feld ohne zugängliche Fehlerbeschreibung
    .description = Ein als ungültig markiertes Formularfeld hat keine zugängliche Beschreibung des Fehlers.
    .user-impact = Screenreader-Nutzer erfahren, dass ein Fehler vorliegt, aber nicht welcher.
    .typical-cause = aria-invalid="true" ohne description/aria-describedby auf ein sichtbares Fehlerelement.
    .recommendation = aria-describedby auf ein Element mit der Fehlermeldung setzen, oder aria-errormessage verwenden.
    .technical-note = aria-invalid="true" ohne description/aria-describedby auf ein sichtbares Fehlerelement.

explanation-duplicate-id-aria = ARIA-Referenz zeigt auf mehrdeutige ID
    .description = Ein ARIA-Attribut wie aria-owns/aria-describedby/aria-labelledby verweist auf eine ID, die im Dokument mehrfach vergeben ist.
    .user-impact = Assistierende Technologien können nicht zuverlässig bestimmen, welches Element gemeint ist.
    .typical-cause = Referenzierte ID kommt im DOM mehr als einmal vor.
    .recommendation = Sicherstellen, dass die von aria-owns/aria-describedby/aria-labelledby referenzierte ID im Dokument eindeutig ist, oder ein anderes, eindeutig identifiziertes Element referenzieren.
    .technical-note = Referenzierte ID kommt im DOM mehr als einmal vor.

explanation-aria-label = Interaktives Element ohne aussagekräftigen Namen
    .description = Ein interaktives Element hat keinen zugänglichen Namen, oder der Name besteht nur aus einem Icon/Symbol ohne Beschreibung.
    .user-impact = Screenreader-Nutzer erfahren nicht, wozu das Element dient.
    .typical-cause = Fehlendes oder rein symbolisches aria-label/aria-labelledby/Textinhalt.
    .recommendation = aria-label, aria-labelledby oder sichtbaren Textinhalt ergänzen, der den Zweck des Elements beschreibt (nicht nur sein Icon).
    .technical-note = Fehlendes oder rein symbolisches aria-label/aria-labelledby/Textinhalt.

explanation-title-only-description = Natives title-Attribut als einzige Beschreibung
    .description = Ein interaktives Element wird nur über das native title-Attribut beschrieben. Das ist Best Practice, kein Verstoß gegen WCAG 4.1.2: title zählt als zugänglicher Name.
    .user-impact = Der Tooltip erscheint nur bei Mauszeiger-Hover; per Tastatur und auf Touchgeräten bleibt der Zweck des Elements unsichtbar.
    .typical-cause = Icon-Buttons, Logo-Links oder Suchfelder ohne sichtbaren Text, bei denen title als Beschriftung dient.
    .recommendation = Eine sichtbare Beschriftung ergänzen oder den Namen über aria-label/aria-labelledby bzw. ein <label> setzen; title nur für ergänzende Hinweise verwenden.
    .technical-note = Best Practice, verankert an WCAG 4.1.2. Nicht 1.4.13: Der Browser-Tooltip des title-Attributs wird vom User Agent gesteuert und ist dort ausgenommen. Geprüft werden button, a[href] und input mit title, ohne Textinhalt, aria-label und aria-labelledby.

explanation-description-duplicates-name = Beschreibung wiederholt den Namen
    .description = Zugänglicher Name und zugängliche Beschreibung eines Elements sind identisch. Das Element hat einen Namen; die doppelte Ansage ist Best Practice, kein fehlender Name.
    .user-impact = Screenreader sagen denselben Text zweimal an, ohne zusätzliche Information.
    .typical-cause = title oder aria-describedby wiederholt den sichtbaren Text oder das aria-label, etwa title="Schließen" an einem Button „Schließen".
    .recommendation = Die Beschreibung entfernen oder mit einer Information füllen, die über den Namen hinausgeht.
    .technical-note = Best Practice, verankert an WCAG 4.1.2. Verglichen werden Name und Beschreibung aus dem Accessibility-Baum des Browsers.

explanation-aria-required-children = Erforderliche Kindelemente einer ARIA-Rolle fehlen
    .description = Ein Element mit einer Composite-ARIA-Rolle (z. B. tablist) fehlt eines der laut ARIA-Spezifikation erforderlichen Kindelemente (z. B. zugehörige tabpanels).
    .user-impact = Assistierende Technologien können die Beziehung zwischen den Widget-Teilen nicht auswerten.
    .typical-cause = z. B. role="tablist" ohne zugehörige role="tabpanel"-Elemente.
    .recommendation = Die laut ARIA-Rolle erforderlichen Kindelemente ergänzen, z. B. role="tabpanel"-Elemente für jeden Tab in einem role="tablist".
    .technical-note = z. B. role="tablist" ohne zugehörige role="tabpanel"-Elemente.

explanation-dialog-name = Dialog ohne zugänglichen Namen
    .description = Ein Dialog (role="dialog"/"alertdialog") hat keinen zugänglichen Namen.
    .user-impact = Screenreader-Nutzer erfahren beim Öffnen des Dialogs nicht, worum es geht.
    .typical-cause = Dialog-Element ohne aria-label oder aria-labelledby auf den Dialogtitel.
    .recommendation = aria-labelledby auf den sichtbaren Dialogtitel setzen, oder aria-label verwenden.
    .technical-note = Dialog-Element ohne aria-label oder aria-labelledby auf den Dialogtitel.

explanation-control-missing-label = Formularelement, Link oder Button ohne zugänglichen Namen
    .description = Ein Formularfeld, Link oder Button hat keinen zugänglichen Namen (weder sichtbares Label noch aria-label/aria-labelledby).
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Eingabe erwartet wird oder wohin ein Link/Button führt.
    .typical-cause = Formularelement/Link/Button ohne <label>, aria-label, aria-labelledby oder Textinhalt.
    .recommendation = Ein <label>-Element mit passendem for-Attribut (Formularfelder) bzw. Textinhalt (Links/Buttons) ergänzen, oder aria-label/aria-labelledby verwenden.
    .technical-note = Formularelement/Link/Button ohne <label>, aria-label, aria-labelledby oder Textinhalt.

explanation-link-as-button = Link als Button-Ersatz ohne Tastenrolle
    .description = Ein Link (<a>) mit href="#" oder javascript:-Pseudo-URL und onclick-Handler wird wie ein Button verwendet, ohne role="button" zu tragen.
    .user-impact = Assistierende Technologien kündigen das Element als Link an, obwohl es sich wie ein Button verhält — Tastaturnutzer erwarten das falsche Bedienverhalten.
    .typical-cause = <a href="#">/<a href="javascript:..."> mit onclick, ohne role="button".
    .recommendation = role="button" auf den Link setzen (plus Tastaturbehandlung für die Leertaste), oder stattdessen ein echtes <button>-Element verwenden.
    .technical-note = <a href="#">/<a href="javascript:..."> mit onclick, ohne role="button".

explanation-redundant-role = Redundante ARIA-Rolle
    .description = Ein Element trägt ein role-Attribut, das exakt der bereits impliziten Rolle des HTML-Elements entspricht (z. B. <button role="button">).
    .user-impact = Kein unmittelbarer Nutzerschaden, aber unnötiges Risiko für Inkonsistenzen bei zukünftigen Änderungen.
    .typical-cause = role-Attribut dupliziert die native implizite Rolle des Tags.
    .recommendation = Das redundante role-Attribut entfernen; das native HTML-Element bringt diese Rolle bereits implizit mit.
    .technical-note = role-Attribut dupliziert die native implizite Rolle des Tags.

explanation-summary-name = Summary-Element ohne zugänglichen Namen
    .description = Ein <summary>-Element (Aufklapp-Auslöser in <details>) hat keinen zugänglichen Namen.
    .user-impact = Screenreader-Nutzer erfahren nicht, was der Aufklapp-Bereich enthält, bevor sie ihn öffnen.
    .typical-cause = <summary> ohne Textinhalt oder aria-label.
    .recommendation = Textinhalt im <summary>-Element ergänzen, der beschreibt, was der Aufklapp-Bereich enthält.
    .technical-note = <summary> ohne Textinhalt oder aria-label.

## Shared rule ids with their own taxonomy entry (plan 56)

explanation-headings-skip-level = Übersprungene Überschriftenebene
    .description = Die Überschriften springen über eine Ebene, etwa von h2 direkt auf h4. Die Gliederung der Seite ist dadurch lückenhaft.
    .user-impact = Screenreader-Nutzer navigieren über die Überschriftenliste und vermuten bei einer fehlenden Ebene übersehene Inhalte.
    .typical-cause = Überschriftenebenen werden nach Schriftgröße statt nach Gliederung gewählt, oder Komponenten bringen eine feste Ebene mit.
    .recommendation = Überschriften lückenlos verschachteln (h1, h2, h3 …) und die Optik per CSS steuern, nicht über die Ebene.
    .technical-note = Auf eine Überschrift der Ebene n folgt höchstens Ebene n+1. Komponenten sollten die Ebene als Parameter annehmen.

explanation-headings-empty = Leere Überschrift
    .description = Ein Überschriften-Element hat keinen Text. Es steht in der Gliederung der Seite, sagt aber nichts.
    .user-impact = Screenreader-Nutzer navigieren über die Überschriftenliste und treffen auf einen Eintrag ohne Namen.
    .typical-cause = Überschriften-Tags als Abstandshalter oder für Icons, oder CMS-Felder, die leer bleiben, während das Tag trotzdem ausgegeben wird.
    .recommendation = Leere Überschriften entfernen oder mit einem sinnvollen Text füllen; Abstände per CSS setzen.
    .technical-note = Das Tag nur rendern, wenn der Titel vorhanden ist. Ein Icon allein benennt keine Überschrift.

explanation-headings-h1-missing = Fehlende Hauptüberschrift (h1)
    .description = Die Seite hat keine Überschrift der Ebene 1, die ihr Thema benennt.
    .user-impact = Screenreader-Nutzer springen gezielt zur h1, um das Thema der Seite zu erfahren, und finden keine.
    .typical-cause = Das Logo oder ein Bild übernimmt die Rolle der Überschrift, oder das Template setzt die Hauptüberschrift als h2 oder als formatierten Text.
    .recommendation = Jeder Seite genau eine h1 mit ihrem Thema geben, am Anfang des Hauptinhalts.
    .technical-note = Die h1 gehört in <main>, nicht ins Logo im Header.

explanation-headings-h1-multiple = Mehrere Hauptüberschriften (h1)
    .description = Die Seite hat mehr als eine Überschrift der Ebene 1. HTML erlaubt das, es sollte aber gewollt sein.
    .user-impact = Screenreader-Nutzer können den eigentlichen Seitentitel schwerer von anderen Abschnitten unterscheiden.
    .typical-cause = Logo, Teaser oder Seitenleiste verwenden ebenfalls h1, oft aus einem gemeinsamen Template.
    .recommendation = Prüfen, ob jede h1 wirklich ein Hauptthema der Seite benennt; alle anderen auf h2 oder tiefer setzen.
    .technical-note = Ein Hinweis zur Prüfung, kein Verstoß: Die Einordnung hängt vom Aufbau der Seite ab.

explanation-lists-invalid-structure = Liste mit unzulässigen Kindelementen
    .description = Eine Liste (<ul>, <ol> oder role="list") enthält direkte Kindelemente, die keine Listeneinträge sind.
    .user-impact = Screenreader nennen eine falsche Zahl an Einträgen oder lesen Teile der Liste ohne Listenzusammenhang vor.
    .typical-cause = Wrapper-<div>s, Trennlinien oder Überschriften direkt in der Liste, oft aus einem Slider- oder Menü-Baustein.
    .recommendation = Alle Inhalte der Liste in <li>-Elemente legen; Wrapper und Trenner in die Einträge verschieben oder per CSS erzeugen.
    .technical-note = Direkte Kinder von <ul>/<ol> dürfen nur <li>, <script> und <template> sein.

explanation-lists-empty = Leere Liste
    .description = Eine Liste enthält keinen einzigen Listeneintrag.
    .user-impact = Screenreader kündigen eine Liste an, auf die nichts folgt.
    .typical-cause = Listen, die per Skript oder aus dem CMS befüllt werden und leer bleiben, oder Listen-Markup als reiner Layout-Container.
    .recommendation = Leere Listen nicht ausgeben, oder das Listen-Element erst rendern, wenn Einträge vorhanden sind.
    .technical-note = Beim Rendern prüfen, ob Einträge vorliegen; ein Layout-Container braucht kein <ul>.

explanation-lists-item-outside-list = Listeneintrag außerhalb einer Liste
    .description = Ein Listeneintrag (<li> oder role="listitem") steht nicht innerhalb einer Liste.
    .user-impact = Screenreader kündigen einen Listeneintrag an, ohne dass es eine Liste mit Länge und Position gibt.
    .typical-cause = Das umschließende <ul>/<ol> fehlt, etwa weil eine Komponente nur die Einträge rendert, oder role="listitem" steht auf einem beliebigen Element.
    .recommendation = Listeneinträge in ein <ul> oder <ol> (bzw. role="list") einbetten, oder die Listen-Semantik entfernen, wenn es keine Liste ist.
    .technical-note = Ein <li> braucht ein <ul>, <ol> oder <menu> als Elternelement; role="listitem" braucht role="list" als Vorfahren.

explanation-lists-term-without-definition = Begriff ohne Definition
    .description = In einer Beschreibungsliste (<dl>) steht ein Begriff (<dt>) ohne zugehörige Beschreibung (<dd>).
    .user-impact = Screenreader lesen einen Begriff vor, zu dem keine Erklärung folgt.
    .typical-cause = Leere Felder im CMS, oder <dl> als Layout für Beschriftung und Wert, bei dem der Wert fehlt.
    .recommendation = Zu jedem <dt> mindestens ein <dd> ausgeben, oder den Begriff weglassen, wenn es keine Beschreibung gibt.
    .technical-note = Geprüft wird unter demselben Elternelement, auch innerhalb von <div>-Gruppen in der <dl>.

explanation-tables-header-missing = Datentabelle ohne Kopfzellen
    .description = Eine Tabelle hat keine Kopfzellen, die ihre Zeilen oder Spalten beschriften.
    .user-impact = Screenreader-Nutzer hören beim Durchgehen der Zellen nur Werte, ohne zu erfahren, zu welcher Spalte oder Zeile sie gehören.
    .typical-cause = Die Kopfzeile ist mit <td> und fetter Schrift gestaltet statt mit <th>, oder die Tabelle dient nur dem Layout.
    .recommendation = Kopfzellen als <th> auszeichnen (mit scope="col" bzw. scope="row"). Dient die Tabelle nur dem Layout, durch CSS-Layout ersetzen oder role="presentation" setzen.
    .technical-note = Kopfzellen sind <th> oder Elemente mit role="columnheader"/"rowheader" irgendwo in der Tabelle.

explanation-tables-name-missing = Datentabelle ohne Namen
    .description = Eine Tabelle hat weder eine <caption> noch einen Namen über aria-label oder aria-labelledby.
    .user-impact = Wer sich die Tabellen einer Seite auflisten lässt, sieht nur „Tabelle“ und muss jede öffnen, um die richtige zu finden.
    .typical-cause = Die Beschriftung steht als Überschrift oder Absatz über der Tabelle, ohne mit ihr verknüpft zu sein.
    .recommendation = Der Tabelle eine <caption> geben oder die vorhandene Überschrift per aria-labelledby verknüpfen. Ob ein Name nötig ist, hängt vom Kontext ab.
    .technical-note = Ein Hinweis zur Prüfung, kein Verstoß: <caption> als erstes Kind der <table>, oder aria-labelledby auf die Überschrift.

explanation-tables-presentational-with-headers = Layouttabelle mit Kopfzellen
    .description = Eine Tabelle ist als reines Layout ausgezeichnet (role="presentation" oder "none"), enthält aber Kopfzellen.
    .user-impact = Assistierende Technik erhält widersprüchliche Angaben: keine Tabelle laut Rolle, Tabellenstruktur laut Kopfzellen.
    .typical-cause = role="presentation" wurde pauschal auf Tabellen gesetzt, darunter auch echte Datentabellen.
    .recommendation = Bei Datentabellen role="presentation" entfernen. Bei Layouttabellen <th> durch <td> ersetzen.
    .technical-note = Eine Tabelle mit <th> oder role="columnheader"/"rowheader" darf nicht role="presentation"/"none" tragen.

explanation-document-lang-invalid = Ungültige Sprachangabe der Seite
    .description = Das lang-Attribut der Seite ist gesetzt, enthält aber keinen gültigen Sprachcode (etwa „english“ statt „en“).
    .user-impact = Screenreader erkennen die Sprache nicht und lesen den Text mit der falschen oder der Standard-Aussprache vor.
    .typical-cause = Sprachname statt Sprachcode, Unterstrich statt Bindestrich (de_DE) oder ein leerer Wert aus dem Template.
    .recommendation = Einen gültigen BCP-47-Code eintragen, etwa lang="de" oder lang="de-DE".
    .technical-note = Primärkennung aus zwei oder drei Buchstaben, danach nur alphanumerische Untertags, getrennt durch Bindestriche.

explanation-keyboard-positive-tabindex = Positiver tabindex verändert die Fokusreihenfolge
    .description = Ein Element hat einen tabindex größer als 0 und wird damit vor allen anderen Elementen fokussiert, unabhängig von seiner Position auf der Seite.
    .user-impact = Tastaturnutzer springen beim Tabben unvorhersehbar durch die Seite und verlieren die Orientierung.
    .typical-cause = Versuche, einzelne Elemente per tabindex="1" nach vorne zu holen, oft aus Formular- oder Such-Widgets.
    .recommendation = Positive tabindex-Werte entfernen und die Reihenfolge über die Reihenfolge im HTML herstellen; tabindex="0" oder "-1" genügen.
    .technical-note = Nur tabindex="0" (in der natürlichen Reihenfolge fokussierbar) und "-1" (nur per Skript fokussierbar) verwenden.

explanation-zoom-viewport-scale-limited = Zoom durch Viewport-Meta-Tag begrenzt
    .description = Das viewport-Meta-Tag begrenzt maximum-scale auf einen Wert unter 5. Nutzer können die Seite auf Mobilgeräten damit nicht auf 500 % vergrößern.
    .user-impact = Sehbehinderte Nutzer können den Seiteninhalt nicht ausreichend vergrößern.
    .typical-cause = maximum-scale wurde gesetzt, um versehentliches Zoomen zu verhindern oder ein Layout zu stabilisieren.
    .recommendation = maximum-scale entfernen oder auf mindestens 5 setzen.
    .technical-note = Best Practice: 1.4.4 verlangt 200 % und ist bei dieser Grenze erfüllt. Liegt die Grenze unter 2 oder ist user-scalable=no gesetzt, meldet die Prüfung einen eigenen 1.4.4-Verstoß.

## Missing navigation and contentinfo landmark (plan 56)

explanation-landmark-navigation-present = Fehlender Navigations-Landmark
    .description = Die Seite hat keinen als Navigation ausgezeichneten Bereich (<nav> bzw. role="navigation").
    .user-impact = Screenreader-Nutzer können nicht direkt zur Navigation springen und müssen sie im Seiteninhalt suchen.
    .typical-cause = Das Menü ist aus <div>- und <ul>-Elementen gebaut, ohne umschließendes <nav>.
    .recommendation = Das Hauptmenü (und weitere Navigationsblöcke) in ein <nav>-Element legen; mehrere Navigationen per aria-label unterscheiden.
    .technical-note = Seiten ohne jede Navigation (etwa eine einzelne Landingpage) brauchen keinen Navigations-Landmark.

explanation-landmark-contentinfo-present = Fehlender Fußbereich-Landmark (contentinfo)
    .description = Die Seite hat keinen als Fußbereich ausgezeichneten Bereich (<footer> auf oberster Ebene bzw. role="contentinfo").
    .user-impact = Screenreader-Nutzer können den Fußbereich mit Kontakt, Impressum und Rechtlichem nicht direkt ansteuern.
    .typical-cause = Der Fußbereich ist ein <div>, oder das <footer> steht innerhalb von <main>, <article> oder <section> und gilt dort nicht als Seitenfuß.
    .recommendation = Den Seitenfuß als <footer> direkt unter <body> auszeichnen, außerhalb von <main>.
    .technical-note = Nur ein <footer> außerhalb von <main>, <article>, <aside>, <nav> und <section> erhält implizit role="contentinfo".

## Measurements on the running page (#1002–#1007)

explanation-frames-cross-origin = Eingebetteter Rahmen konnte nicht geprüft werden
    .description = Das Dokument in einem iframe war für das Prüfwerkzeug nicht lesbar und blieb deshalb ungeprüft. Der Rahmen hat einen fremden Ursprung, oder seine Navigation war nicht abgeschlossen.
    .user-impact = Eingebettete Inhalte wie Karten, Buchungs- oder Einwilligungsdialoge können für Screenreader- und Tastaturnutzer unzugänglich sein, ohne dass der Bericht es zeigt.
    .typical-cause = Inhalte von Drittanbietern in einem iframe mit fremdem Ursprung; die Seite kann deren Dokument nicht lesen.
    .recommendation = Den Rahmeninhalt von Hand prüfen oder seine Adresse für sich prüfen; beim Anbieter nach einer barrierefreien Einbettung fragen.
    .technical-note = Eine Abdeckungsnotiz, keine Konformitätsprüfung und an kein Erfolgskriterium gebunden. Gemessen wird, ob contentDocument des Rahmens lesbar ist; Rahmen, die präsentational, versteckt oder höchstens 1 px groß sind, bleiben außen vor. Ohne die Messung steht die Regel als nicht gelaufen im Bericht.

explanation-orientation-root-rotated = Seite erzwingt eine Bildschirmausrichtung durch Drehung der Wurzel
    .description = Das Wurzelelement (html oder body) ist um eine Vierteldrehung gedreht. Damit lässt sich die Seite nur in einer Ausrichtung nutzen, außer die Ausrichtung ist wesentlich.
    .user-impact = Menschen, deren Gerät fest montiert ist, etwa am Rollstuhl, können die Ausrichtung nicht wechseln und sehen die Seite gedreht oder abgeschnitten.
    .typical-cause = Ein transform: rotate(90deg) oder rotate: 90deg auf html oder body, das im Hochformat das Querformat nachbilden soll.
    .recommendation = Die Drehung entfernen und das Layout in beiden Ausrichtungen nutzbar machen. Nur wenn eine Ausrichtung wesentlich ist (Klavier, Bankscheck), darf sie festgelegt werden.
    .technical-note = Gemessen wird die berechnete transform-Matrix und die Eigenschaft rotate von html und body im geprüften Viewport; eine Sperre nur in der anderen Ausrichtung zeigt der Lauf im jeweils anderen Viewport. Ob eine Ausrichtung wesentlich ist, entscheidet ein Mensch.

explanation-auth-paste-blocked = Einfügen in Passwort- oder Code-Feld blockiert
    .description = Ein Passwort- oder Einmalcode-Feld bricht das paste-Ereignis ab und fügt den Text nicht selbst ein. Passwortmanager und die Zwischenablage funktionieren dort nicht.
    .user-impact = Wer sich Passwörter nicht merken oder fehlerfrei abschreiben kann, etwa wegen kognitiver oder motorischer Einschränkungen, scheitert an der Anmeldung.
    .typical-cause = Ein paste-Handler mit preventDefault() oder onpaste="return false", meist als vermeintliche Sicherheitsmaßnahme.
    .recommendation = Den Handler entfernen. Wird der Text bereinigt, den bereinigten Text selbst in das Feld einfügen, statt das Einfügen zu verbieten.
    .technical-note = Gemessen wird ein synthetisches, abbrechbares paste-Ereignis an Passwort- und Codefeldern, auch in gleich-ursprünglichen Rahmen und offenen Shadow Roots; der Feldwert wird danach wiederhergestellt. Das Ereignis ist nicht vertrauenswürdig (isTrusted ist false): Ein Handler, der nur auf echte Ereignisse oder Tastendrücke reagiert, bleibt unsichtbar. autocomplete="off" ist kein Befund.

explanation-focus-obscured = Fokussiertes Element vollständig verdeckt
    .description = Nach dem Fokussieren und Scrollen liegt jeder Abtastpunkt eines Bedienelements unter einem fixierten oder klebenden Element, etwa einer Kopfleiste oder einem Cookie-Banner.
    .user-impact = Tastaturnutzer springen mit Tab auf ein Element, das sie nicht sehen, und wissen nicht, wo der Fokus ist.
    .typical-cause = Eine fixierte oder klebende Leiste, die tiefer reicht als scroll-padding-top, oder ein Banner, das Bedienelemente der Seite überdeckt.
    .recommendation = scroll-padding-top mindestens so groß wie die fixierte Leiste setzen und Banner so einbauen, dass sie keine Bedienelemente überdecken.
    .technical-note = Gemessen wird nach dem Fokussieren mit scrollIntoView({"{"} block: 'nearest' {"}"}), 3 mal 3 Punkte je Bedienelement per elementsFromPoint. Ein echtes Tab scrollt womöglich anders; der Wert ist deshalb ein Hinweis zum Prüfen, kein Beleg. Auch der Hinweis auf eine Leiste, die tiefer reicht als scroll-padding-top, gehört zu dieser Kennung.

explanation-focus-obscured-enhanced = Fokussiertes Element teilweise verdeckt
    .description = Nach dem Fokussieren und Scrollen liegt mindestens ein Abtastpunkt eines Bedienelements unter einem fixierten oder klebenden Element (Stufe AAA).
    .user-impact = Tastaturnutzer sehen ein fokussiertes Bedienelement nur zum Teil und erkennen Beschriftung oder Zustand womöglich nicht.
    .typical-cause = Ein klebender Fuß, Chat-Button oder Cookie-Banner, der den Rand der Seite überdeckt.
    .recommendation = Platz unter und über fixierten Elementen freihalten (scroll-padding, Seitenabstand) oder die Elemente verkleinern.
    .technical-note = Dieselbe Messung wie bei focus/obscured, andere Schwelle: jeder überdeckte Abtastpunkt wird gemeldet. Die Abtastung nähert „irgendein Teil“ nur an, das Ergebnis bleibt ein Hinweis zum Prüfen.

## Shared rule ids from #690 without a fitting text

explanation-zoom-viewport-locked = Zoom durch Viewport-Meta-Tag gesperrt
    .description = Das viewport-Meta-Tag verhindert das Zoomen (user-scalable=no) oder begrenzt es auf weniger als 200 %.
    .user-impact = Menschen mit Sehbeeinträchtigung können die Seite auf Mobilgeräten nicht vergrößern und verlieren Inhalte.
    .typical-cause = user-scalable=no oder maximum-scale=1, gesetzt, um versehentliches Zoomen zu verhindern oder ein Layout zu stabilisieren.
    .recommendation = user-scalable=no und maximum-scale aus dem viewport-Meta-Tag entfernen.
    .technical-note = Eine Grenze zwischen 200 % und 500 % ist kein Verstoß gegen 1.4.4 und wird getrennt gemeldet.

explanation-zoom-viewport-missing = Fehlende Viewport-Angabe
    .description = Das Dokument hat kein viewport-Meta-Tag. Mobile Browser legen dann eine Desktop-Breite zugrunde und verkleinern die Seite.
    .user-impact = Auf Mobilgeräten erscheint der Text unter jeder lesbaren Größe; wer vergrößert, muss waagerecht scrollen. Am Desktop ändert sich nichts.
    .typical-cause = Das Template setzt kein viewport-Meta-Tag, etwa bei älteren oder rein für Desktop gebauten Seiten.
    .recommendation = Im <head> ein viewport-Meta-Tag mit width=device-width ergänzen.
    .technical-note = Desktop-Browser ignorieren das Tag; betroffen ist nur die Darstellung auf Mobilgeräten. Das Layout muss danach auch in schmalen Viewports funktionieren (1.4.10 Reflow).

explanation-aria-reference-missing = ARIA-Verweis ins Leere
    .description = Ein ARIA-Beziehungsattribut (etwa aria-controls oder aria-labelledby) ist leer oder verweist auf eine ID, die es auf der Seite nicht gibt.
    .user-impact = Screenreader können die Beziehung nicht herstellen: Beschriftung, Beschreibung oder gesteuerter Bereich fehlen.
    .typical-cause = Die ID des Ziels wurde umbenannt oder wird erst später per Skript erzeugt; Komponenten setzen das Attribut, bevor das Ziel existiert.
    .recommendation = Das Attribut auf die ID eines vorhandenen Elements setzen oder es entfernen.
    .technical-note = Geprüft werden aria-labelledby, aria-describedby, aria-controls, aria-owns und aria-activedescendant.

explanation-aria-required-attribute-missing = Erforderliches ARIA-Attribut fehlt
    .description = Ein Element trägt eine Rolle, die einen Zustand ansagt (etwa checkbox oder slider), aber nicht das Attribut, das diesen Zustand trägt.
    .user-impact = Screenreader-Nutzer erfahren nicht, ob ein Schalter an ist oder welchen Wert ein Regler hat.
    .typical-cause = Selbst gebaute Bedienelemente setzen die Rolle, pflegen den Zustand aber nur visuell.
    .recommendation = Das geforderte Attribut setzen und bei jeder Zustandsänderung aktualisieren, oder ein natives Element verwenden.
    .technical-note = Etwa aria-checked für checkbox/radio/switch, aria-expanded für combobox, aria-valuenow für slider/spinbutton/meter.

explanation-images-alt-suspicious = Vermutlich nichtssagender Alternativtext
    .description = Ein Bild hat einen Alternativtext, der den Inhalt vermutlich nicht beschreibt: einen Dateinamen, ein Füllwort wie „Bild" oder „Logo" oder nur ein, zwei Zeichen.
    .user-impact = Screenreader-Nutzer hören einen Text, der ihnen den Bildinhalt nicht vermittelt.
    .typical-cause = Das CMS übernimmt den Dateinamen als Alternativtext, oder das Pflichtfeld wurde mit einem Platzhalter gefüllt.
    .recommendation = Prüfen, ob der Text das Bild beschreibt; sonst durch eine Beschreibung ersetzen, bei dekorativen Bildern durch alt="".
    .technical-note = Heuristisch erkannt, deshalb zur Prüfung: alt="5" am Bild einer Fünf ist richtig.

## Shared rule ids from #691 (B1) without a fitting text

explanation-aria-role-invalid = Ungültige ARIA-Rolle
    .description = Ein role-Attribut enthält einen Wert, den es in WAI-ARIA nicht gibt, oder eine abstrakte Rolle, die nicht ausgezeichnet werden darf.
    .user-impact = Browser ignorieren die Rolle; Screenreader sagen das Element anders an als beabsichtigt.
    .typical-cause = Tippfehler, erfundene Rollen oder abstrakte Rollen wie widget oder landmark.
    .recommendation = Eine Rolle aus der ARIA-Spezifikation verwenden oder das role-Attribut entfernen und ein natives Element nutzen.
    .technical-note = Rollen unterscheiden Groß- und Kleinschreibung: role="Button" ist ungültig.

explanation-aria-attribute-unknown = Unbekanntes ARIA-Attribut
    .description = Ein Element trägt ein aria-*-Attribut, das es in WAI-ARIA nicht gibt – meist ein Tippfehler.
    .user-impact = Der Browser ignoriert das Attribut; die gemeinte Beschriftung, Beziehung oder der gemeinte Zustand fehlt.
    .typical-cause = Falsch geschriebene Attribute wie aria-labeledby statt aria-labelledby.
    .recommendation = Den Namen gegen die ARIA-Spezifikation prüfen und korrigieren oder das Attribut entfernen.
    .technical-note = Attribute aus dem Entwurf von WAI-ARIA 1.3, die Browser schon umsetzen (etwa aria-description), gelten als bekannt.

explanation-aria-attribute-value-invalid = Ungültiger Wert eines ARIA-Attributs
    .description = Ein ARIA-Attribut hat einen Wert außerhalb seines Wertebereichs, etwa aria-expanded="yes".
    .user-impact = Browser fallen auf den Vorgabewert zurück; der Zustand wird Screenreader-Nutzern falsch oder gar nicht angesagt.
    .typical-cause = Werte aus anderen Konventionen (yes/no, 1/0) oder Tippfehler in Token-Werten.
    .recommendation = Nur die Werte verwenden, die die ARIA-Spezifikation für das Attribut vorsieht, etwa true oder false.
    .technical-note = Unbekannte Werte von aria-current und aria-invalid gelten nach ARIA 1.2 als true und werden nicht gemeldet.

explanation-aria-owns-conflict = Element gehört per aria-owns zu mehreren Elementen
    .description = Dieselbe ID steht im aria-owns mehrerer Elemente. Welches Element der Besitzer ist, bleibt offen.
    .user-impact = Assistierende Technologien können das Element keinem Widget eindeutig zuordnen; Name, Rolle und Wert werden unzuverlässig.
    .typical-cause = Mehrfach eingebundene Komponenten, die auf dieselbe Liste zeigen, oder eine doppelt vergebene ID.
    .recommendation = Jedes Element nur im aria-owns eines einzigen Elements aufführen.
    .technical-note = WAI-ARIA 1.2: Eine ID darf zu keinem Zeitpunkt im aria-owns mehr als eines anderen Elements stehen.

explanation-aria-tab-selected-missing = Kein Tab als ausgewählt ausgezeichnet
    .description = Keiner der Tabs einer Tableiste trägt aria-selected="true".
    .user-impact = Screenreader-Nutzer erfahren nicht, welcher Tab aktiv ist und zu welchem der sichtbare Inhalt gehört.
    .typical-cause = Der aktive Tab wird nur per CSS-Klasse hervorgehoben.
    .recommendation = Am aktiven Tab aria-selected="true" setzen und beim Wechsel mitführen.
    .technical-note = Zur Prüfung: ARIA 1.2 sagt SHOULD. Tabs ohne das Attribut gelten als nicht ausgewählt, das ist für die inaktiven richtig.

explanation-aria-combobox-popup-missing = Aufgeklappte Combobox ohne Verweis auf ihr Popup
    .description = Eine Combobox ist als aufgeklappt ausgezeichnet, verweist aber nicht per aria-controls auf ihre Auswahlliste.
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Auswahlmöglichkeiten die Combobox bietet, und erreichen sie nicht.
    .typical-cause = Die Liste steht neben der Combobox statt darin, und aria-controls fehlt.
    .recommendation = aria-controls an der Combobox auf die ID der Liste setzen.
    .technical-note = ARIA 1.2 verlangt aria-controls (MUST). Ein Popup im eigenen Teilbaum oder per aria-owns (ARIA 1.1) wird weiter anerkannt.

explanation-aria-attribute-not-allowed = ARIA-Attribut für diese Rolle nicht zulässig
    .description = Ein Element trägt ein ARIA-Attribut, das seine Rolle nicht unterstützt, oder eines, das neben einem nativen Attribut nicht stehen darf.
    .user-impact = Screenreader ignorieren das Attribut oder erhalten widersprüchliche Angaben zu Zustand und Wert.
    .typical-cause = Zustände wie aria-expanded oder aria-checked an Elementen ohne passende Rolle, etwa an einem div.
    .recommendation = Das Attribut an das Element mit der passenden Rolle verschieben, eine passende Rolle setzen oder das Attribut entfernen.
    .technical-note = Geurteilt wird über die explizite oder implizite Rolle. Dazu die Fälle aus ARIA in HTML, etwa aria-checked an einer nativen Checkbox.

explanation-aria-required-parent-missing = Element außerhalb des erforderlichen Eltern-Kontexts
    .description = Ein Element mit einer Rolle wie tab, option oder listitem steht nicht in einem Element mit der dafür erforderlichen Rolle.
    .user-impact = Screenreader können das Element keinem Widget zuordnen; Position und Anzahl („Tab 2 von 3") werden nicht angesagt.
    .typical-cause = Ein Element mit eigener Rolle dazwischen, etwa ein <li> zwischen role="tablist" und role="tab".
    .recommendation = Das Element direkt in den erforderlichen Behälter setzen oder die Zwischenelemente mit role="presentation" neutralisieren.
    .technical-note = Geprüft wird am DOM. Ein <li> in einer Liste, die per role etwas anderes geworden ist, bleibt ein listitem, auch wenn der Browser es im Accessibility-Tree glättet.

explanation-aria-required-children-missing = Erforderliche Bestandteile einer ARIA-Rolle fehlen
    .description = Ein Behälter wie role="tablist", "list" oder "menu" enthält keines der Elemente, die seine Rolle verlangt.
    .user-impact = Screenreader sagen einen leeren oder fehlerhaften Behälter an; Nutzer finden die Einträge nicht.
    .typical-cause = Die Einträge tragen keine oder eine andere Rolle, oder ein Element mit eigener Rolle steht zwischen Behälter und Einträgen.
    .recommendation = Den Einträgen die geforderte Rolle geben, etwa role="tab" in einer tablist, oder die Rolle des Behälters entfernen.
    .technical-note = Geprüft werden nur Behälter mit expliziter Rolle; native Tabellen und Listen geben ihre Bestandteile per HTML vor.

## Shared rule ids from #692 (B2) without a fitting text

explanation-dialog-modal-unmarked = Dialog nicht als modal ausgezeichnet
    .description = Ein Element mit role="dialog" trägt kein aria-modal="true". Ob der Dialog modal gemeint ist, muss geprüft werden.
    .user-impact = Ist der Dialog modal gemeint, lesen Screenreader den Inhalt dahinter weiter, als wäre er erreichbar.
    .typical-cause = Ein selbst gebauter Dialog aus einem div mit role="dialog", ohne aria-modal.
    .recommendation = Einen modalen Dialog mit aria-modal="true" auszeichnen oder ein natives <dialog> mit showModal() verwenden. Ein nicht modaler Dialog braucht das Attribut nicht.
    .technical-note = ARIA verlangt aria-modal nicht; der Hinweis ist eine Prüfempfehlung, kein Verstoß.

## Shared rule ids from #693 (B3) without a fitting text

explanation-forms-no-submit = Formular ohne Absende-Element
    .description = Ein Formular sammelt Eingaben, hat aber keinen Button und kein anderes Element, mit dem es ausdrücklich abgeschickt wird.
    .user-impact = Ohne Absende-Element bleibt unklar, wann die Eingabe gilt. Wird das Formular stattdessen beim Ändern eines Felds abgeschickt, wechselt der Kontext unangekündigt.
    .typical-cause = Filter- und Suchformulare, die per Skript beim Ändern eines Felds abgeschickt werden, oder ein Button außerhalb des Formulars ohne form-Attribut.
    .recommendation = Einen Button type="submit" ins Formular setzen oder per form-Attribut zuordnen. Ändert ein Formular nur Inhalte auf der Seite, genügt eine Prüfung, dass dabei kein Kontextwechsel passiert.
    .technical-note = Ohne action und ohne Textfeld kann nur ein Skript das Formular abschicken; dann ist der Befund ein Prüfhinweis (#728).

explanation-auth-captcha = Captcha im Anmeldeformular prüfen
    .description = Ein Anmeldeformular enthält ein Captcha. Ob es ein Objekterkennungstest ist oder eine Alternative ohne Denkaufgabe angeboten wird, muss geprüft werden.
    .user-impact = Wer verzerrte Zeichen nicht lesen, abschreiben oder sich merken kann, kommt an der Anmeldung nicht vorbei.
    .typical-cause = Ein Text- oder Bildrätsel-Captcha gegen Bots, ohne zweiten Weg.
    .recommendation = Eine Anmeldung ohne Denkaufgabe anbieten, etwa Passkey, E-Mail-Link oder eine Bot-Prüfung ohne Interaktion. Ein Objekterkennungstest ist auf Stufe AA zulässig.
    .technical-note = Prüfhinweis: Art des Tests und Alternativen stehen nicht im Markup. Ein Captcha im Kontaktformular gehört nicht zu 3.3.8.

## Shared rule ids from #694 without a fitting text

explanation-landmarks-not-top-level = Landmark ist verschachtelt
    .description = Ein Kopf-, Fuß- oder Hauptinhaltsbereich (banner, contentinfo, main) liegt innerhalb eines anderen Landmarks statt auf oberster Ebene.
    .user-impact = Screenreader-Nutzer, die per Landmark-Navigation zu Kopf, Fuß oder Hauptinhalt springen, landen in einem unerwarteten Kontext.
    .typical-cause = role="banner", role="contentinfo" oder <main> innerhalb von <main>, <nav>, <aside> oder einem benannten Bereich.
    .recommendation = Kopf-, Fuß- und Hauptinhaltsbereich auf die oberste Ebene der Seite verschieben, außerhalb jedes anderen Landmarks.
    .technical-note = Ein <header>/<footer> in <main>, <article> oder <section> ist ohnehin kein banner/contentinfo; gemeldet werden explizite Rollen und <main>. Welche Rolle betroffen ist, steht im Befundtext.

explanation-patterns-accordion-controls-missing = Aufgeklappter Button ohne Verweis auf seinen Bereich
    .description = Ein aufgeklappter Button (aria-expanded="true") verweist nicht per aria-controls auf den Bereich, den er steuert.
    .user-impact = Screenreader-Nutzer erfahren nicht, welcher Bereich aufgeklappt wurde, und können nicht direkt dorthin springen.
    .typical-cause = Akkordeon- oder Aufklapp-Skripte, die nur aria-expanded umschalten.
    .recommendation = Am Button aria-controls auf die ID des gesteuerten Bereichs setzen.
    .technical-note = Prüfhinweis: WAI-ARIA verlangt aria-controls nicht, die APG nennt es beim Disclosure-Muster optional. Ein <summary> braucht es nicht.

## Shared rule ids from #697 without a fitting text

explanation-tables-header-without-data = Tabellenkopf ohne Datenzellen
    .description = Eine Kopfzelle (<th>) bezieht sich auf keine Datenzelle – die Tabelle hat Überschriften, aber keine Daten darunter oder daneben.
    .user-impact = Screenreader können Datenzellen keinem Kopf zuordnen; die Tabelle wirkt leer oder falsch aufgebaut.
    .typical-cause = Tabellen, die nur zur Anordnung dienen, oder Datenzeilen, die per Skript erst später erscheinen.
    .recommendation = Kopfzellen nur dort einsetzen, wo Daten folgen; Layout ohne <table> umsetzen.
    .technical-note = Leere Datenzellen, Zellen mit aria-hidden-Inhalt und Spaltenköpfe über Zeilenköpfen zählen nicht als Fehler. Ein noch nicht dargestellter Zeilenvorrat ergibt einen Hinweis (ungeprüft) statt eines Verstoßes.

explanation-tables-headers-attr-invalid = headers-Attribut verweist ins Leere
    .description = Eine Tabellenzelle verweist per headers auf eine ID, die keine Kopfzelle derselben Tabelle ist.
    .user-impact = Screenreader lesen zur Zelle den falschen oder gar keinen Kopf vor.
    .typical-cause = Umbenannte oder kopierte IDs, Verweise auf Zellen anderer Tabellen.
    .recommendation = headers nur auf IDs von <th> derselben Tabelle zeigen lassen oder bei einfachen Tabellen ganz auf scope setzen.
    .technical-note = Geprüft wird nur an Zellen (td, th und ihre Rollen); nur dort definiert HTML das Attribut.

explanation-document-lang-mismatch = lang und xml:lang widersprechen sich
    .description = Das <html>-Element trägt lang und xml:lang mit unterschiedlichen Sprachen.
    .user-impact = Screenreader wählen je nach Attribut eine andere Sprache und sprechen die Seite womöglich falsch aus.
    .typical-cause = Aus XHTML übernommene Vorlagen, bei denen nur eines der Attribute angepasst wurde.
    .recommendation = xml:lang entfernen oder auf denselben Wert wie lang setzen.
    .technical-note = Verglichen wird die Primärsprache beider Attribute.

explanation-language-part-unmarked = Anderssprachiger Textteil ohne lang prüfen
    .description = Ein Textabschnitt scheint in einer anderen Sprache verfasst als die Seite, ohne dass ein lang-Attribut das auszeichnet.
    .user-impact = Screenreader lesen den Abschnitt mit der Aussprache der Seitensprache vor; er wird schwer verständlich.
    .typical-cause = Zitate, Produktnamen oder Teaser in einer anderen Sprache ohne lang.
    .recommendation = Den Abschnitt mit lang auszeichnen, etwa <p lang="en">.
    .technical-note = Prüfhinweis: Die Spracherkennung ist eine Heuristik und kennt Deutsch und Englisch. Auf Seiten in anderen Sprachen bleibt die Prüfung als ungeprüft vermerkt.

## Shared rule ids from #696 without a fitting text

explanation-media-audio-autoplay = Selbststartender Ton ohne Steuerung prüfen
    .description = Ein Audio-Element startet beim Laden der Seite von selbst und ist nicht stummgeschaltet.
    .user-impact = Ton, der von selbst startet, übertönt die Sprachausgabe von Screenreadern; Nutzer finden die Steuerung dann kaum noch.
    .typical-cause = <audio autoplay> für Hintergrundmusik oder Werbung.
    .recommendation = Ton nicht automatisch starten oder eine Steuerung zum Anhalten bzw. Leiserstellen am Anfang der Seite anbieten.
    .technical-note = Prüfhinweis (WCAG 1.4.2): Ob der Ton länger als drei Sekunden läuft und sich anhalten lässt, steht nicht im Markup.

explanation-frames-name-missing = Eingebetteter Rahmen ohne Namen
    .description = Ein <iframe> hat keinen zugänglichen Namen (title, aria-label oder aria-labelledby).
    .user-impact = Screenreader nennen beim Betreten des Rahmens nur „Rahmen“; Nutzer erfahren nicht, was darin steckt, und können ihn nicht gezielt überspringen.
    .typical-cause = Eingebettete Karten, Videos, Widgets oder Werbung ohne title-Attribut.
    .recommendation = Jedem sichtbaren <iframe> ein title geben, das seinen Inhalt beschreibt.
    .technical-note = WCAG-Technik H64. Rahmen mit role="none"/"presentation", hidden, aria-hidden oder Breite und Höhe 0/1 zählen nicht.

explanation-display-toggle-missing = Kein Umschalter für Darstellungsmodi
    .description = Die Seite zeigt Visualisierungen, aber es wurde kein mit [data-display-toggle] markierter Umschalter gefunden.
    .user-impact = Fehlt der Umschalter tatsächlich, bleibt an die volle Darstellung gebunden, wer Bewegung, 3D oder Grafiken nicht verträgt.
    .typical-cause = Visualisierungen nach der Konvention ausgezeichnet, der Umschalter aber nicht eingebaut oder nicht mit [data-display-toggle] markiert.
    .recommendation = Auf jeder Seite mit Visualisierungen einen bedienbaren Umschalter anbieten, der zwischen visual, calm und text wechselt und die Wahl speichert.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 2.2.2 (Pause, Stop, Hide). Geprüft wird das Element [data-display-toggle] im gerenderten DOM.

explanation-display-init-missing = Darstellungsmodus wird zu spät gesetzt
    .description = Die gewählte Darstellung wird nicht vor dem ersten Anzeigen der Seite angewendet.
    .user-impact = Die Seite kann kurz mit Animation oder Grafiken erscheinen, obwohl eine ruhige Darstellung gewählt wurde.
    .typical-cause = data-display wird in einem verzögerten oder Modul-Skript gesetzt statt in einem kleinen blockierenden Skript im <head>.
    .recommendation = Den Modus in einem kleinen, blockierenden Skript im <head> setzen, bevor der Seitenkörper beginnt.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 2.2.2. Gemessen per CDP: ein vor der Navigation injizierter Beobachter hält fest, ob html[data-display] beim Einfügen von <body> bereits gesetzt war.

explanation-display-text-media-visible = Grafiken im Textmodus sichtbar
    .description = Im Textmodus zeigt eine Visualisierung weiterhin Grafik, Canvas oder Video.
    .user-impact = Wer den Textmodus gewählt hat, bekommt trotzdem Grafiken oder bewegte Inhalte.
    .typical-cause = Die CSS-Regeln für html[data-display="text"] blenden [data-viz-live] oder [data-viz-static] nicht aus.
    .recommendation = Im Textmodus Live- und Standbild-Teile der Visualisierung ausblenden und nur die Textschicht zeigen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 2.2.2. Geprüft wird das gerenderte Layout im Modus text; Elemente bis 32×32 px gelten als Icons.

explanation-display-text-not-visible = Textschicht fehlt im Textmodus
    .description = Im Textmodus zeigt eine Visualisierung keine lesbare Aussage.
    .user-impact = Wer den Textmodus gewählt hat, erfährt nicht, was die Grafik aussagt.
    .typical-cause = [data-viz-text] fehlt, ist leer oder bleibt im Textmodus ausgeblendet.
    .recommendation = Jede Visualisierung mit einer Textschicht versehen (Kernaussage, bei Diagrammen die Werte als Tabelle, Quelle) und sie im Textmodus anzeigen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.1.1 (Nicht-Text-Inhalt). Geprüft wird das gerenderte Layout im Modus text.

explanation-display-text-hidden = Textschicht für Hilfstechnik verborgen
    .description = Die Textschicht einer Visualisierung ist für Screenreader ausgeblendet.
    .user-impact = Screenreader-Nutzer erfahren die Aussage der Grafik nicht.
    .typical-cause = hidden, aria-hidden="true", inert oder display:none an [data-viz-text] oder einem umgebenden Element, meist um sie in der grafischen Darstellung zu verstecken.
    .recommendation = Die Textschicht nur optisch verstecken (visually-hidden), nie für Hilfstechnik.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.1.1. Geprüft im gerenderten DOM in jedem Modus, einschließlich berechneter Stile.

explanation-viz-text-missing = Visualisierung ohne Textschicht
    .description = Eine Visualisierung hat keine mit [data-viz-text] markierte Textschicht oder nur eine leere.
    .user-impact = Wer die Grafik nicht sehen kann, erfährt ihre Aussage nicht; der Textmodus hat nichts, was statt der Grafik erscheint.
    .typical-cause = Die Beschreibung steht in einem Element ohne die Marke [data-viz-text], oder die Visualisierung wurde ohne Textschicht eingebaut.
    .recommendation = Jeder figure[data-viz] eine Textschicht mit der Aussage geben (bei Diagrammen die Werte, am besten als <table>) und die Quelle nennen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.1.1. Geprüft wird nur, dass Text da ist, nicht ob er die Kernaussage trifft. Aus dem Markup, in jedem Modus.

explanation-viz-caption-missing = Visualisierung ohne Beschriftung
    .description = Eine Visualisierung hat kein <figcaption>, das sie kurz benennt.
    .user-impact = Nichts benennt die Visualisierung kurz; wer sie nicht sieht, muss erst die Textschicht lesen, um zu wissen, worum es geht.
    .typical-cause = Das <figcaption> fehlt oder steht außerhalb der <figure>.
    .recommendation = Ein <figcaption> als direktes Kind der figure[data-viz] ergänzen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.1.1. Geprüft wird das Vorhandensein, nicht der Inhalt.

explanation-viz-static-missing = Bewegte Visualisierung ohne Standbild
    .description = Eine 3D- oder interaktive Visualisierung hat kein Standbild für den ruhigen Modus.
    .user-impact = Wer Bewegung nicht verträgt, bekommt im ruhigen Modus nichts, was die Bewegung ersetzt.
    .typical-cause = Nur die bewegte Fassung wurde gebaut, ohne ein mit [data-viz-static] markiertes Standbild.
    .recommendation = Ein Standbild mit [data-viz-static] in die Visualisierung aufnehmen und im ruhigen Modus anstelle der Bewegung zeigen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 2.2.2. Gilt für data-viz="3d" und "interactive".

explanation-viz-table-missing = Diagramm ohne Wertetabelle
    .description = Ein Diagramm hat keine Tabelle mit seinen Werten. Zu prüfen ist, ob die Textschicht sie auch ohne Tabelle vermittelt.
    .user-impact = Die Beziehungen zwischen den Werten können verloren gehen, wenn sie nur als Fließtext oder gar nicht vorliegen.
    .typical-cause = Die Werte stehen nur in der Grafik oder in einem Satz, bei Sparklines oft bewusst.
    .recommendation = Die Werte als <table> in die Textschicht aufnehmen, sofern ein Satz sie nicht ebenso vollständig trägt.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.3.1. Prüfhinweis, kein Verstoß: ob ein Satz genügt, kann nur ein Mensch beurteilen.
