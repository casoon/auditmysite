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
