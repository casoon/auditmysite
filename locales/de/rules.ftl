# Rule texts: one message per rule id of `src/taxonomy/rules.rs` (`RULES`),
# in the order of that table. The message value is the title, shown for the
# rule everywhere (finding, `fix_guidance[]`, PDF, table). The attributes
# `.user-impact` and `.technical-impact` are the catalogue texts and exist for
# every rule. A rule that has an explanation entry under its own id (the
# entry key is the rule id, see `RULE_ENTRIES` in `src/output/explanations.rs`)
# also carries the explanation texts `.description`, `.explanation-user-impact`,
# `.typical-cause`, `.recommendation` and `.technical-note`.
# The key is `rule-` plus the rule id with `/` written as `-`
# (`taxonomy::rules::rule_text_key`).

## Content & alternatives

rule-images-alt-missing = Fehlende Alternativtexte bei Bildern
    .user-impact = Screenreader-Nutzer erhalten keine Bildinformation.
    .technical-impact = Ein <img> hat kein alt-Attribut (oder nur Leerraum darin) und weder aria-label, aria-labelledby noch title und ist nicht als dekorativ ausgezeichnet (role="presentation", aria-hidden="true"), daher steht es ohne Namen im Accessibility-Tree und ein Screenreader sagt nur den Dateinamen oder „Grafik“ an.

rule-images-alt-suspicious = Prüfen, ob der Alternativtext das Bild beschreibt
    .user-impact = Screenreader-Nutzer hören einen Text, der den Bildinhalt nicht vermittelt.
    .technical-impact = Der Alternativtext eines <img> ist kürzer als drei Zeichen, ein bloßes Füllwort (etwa „Bild“, „Foto“, „Grafik“, „Logo“, „Icon“, „Platzhalter“ oder das englische „image“, „photo“) oder endet auf eine Bilddateiendung wie .jpg oder .png, daher liest ein Screenreader etwas vor, das das Bild nicht beschreibt; als Prüfhinweis gemeldet, weil nur ein Mensch das beurteilen kann.
    .description = Ein Bild hat einen Alternativtext, der den Inhalt vermutlich nicht beschreibt: einen Dateinamen, ein Füllwort wie „Bild“ oder „Logo“ oder nur ein, zwei Zeichen.
    .explanation-user-impact = Screenreader-Nutzer hören einen Text, der ihnen den Bildinhalt nicht vermittelt.
    .typical-cause = Das CMS übernimmt den Dateinamen als Alternativtext, oder das Pflichtfeld wurde mit einem Platzhalter gefüllt.
    .recommendation = Prüfen, ob der Text das Bild beschreibt; sonst durch eine Beschreibung ersetzen, bei dekorativen Bildern durch alt="".
    .technical-note = Heuristisch erkannt, deshalb zur Prüfung: alt="5" am Bild einer Fünf ist richtig.

rule-video-caption = Untertitel von Videos prüfen
    .user-impact = Gehörlose oder schwerhörige Nutzer können gesprochene Inhalte nicht wahrnehmen.
    .technical-impact = Ein dargestelltes <video> hat ein <track kind="captions"> oder kind="subtitles" (oder ein <track> ohne kind, das HTML als subtitles liest), aber keine seiner Dateien wird vom Ursprung der Seite als Untertitelformat (.vtt oder .srt) geladen, oder die Seite bettet einen Player eines Drittanbieters ein (YouTube, Vimeo und ähnliche), daher hat gesprochener Inhalt möglicherweise kein Textäquivalent. Hat keine Spur eines Videos ein src oder antwortet der Ursprung der Seite mit einem Fehler oder ohne Untertiteldatei, hat das Video keine funktionierenden Untertitel: ein Prüfhinweis wie bei einer fehlenden Spur. Eine Spur von einem anderen Ursprung oder jenseits des Prüfbudgets und ein Player eines Drittanbieters bleiben eine ungeprüfte Hinweisnotiz zur manuellen Prüfung, weil das Werkzeug diese Untertitel nicht sieht. Ein Video ohne eine solche Spur meldet stattdessen media/video-captions-missing.

rule-media-video-captions-missing = Videos ohne Untertitelspur prüfen
    .user-impact = Gehörlose oder schwerhörige Nutzer können gesprochene Inhalte nicht wahrnehmen.
    .technical-impact = Ein <video> außerhalb von hidden- und aria-hidden-Inhalten, das nicht stumm ohne Bedienelemente ist, hat kein <track kind="captions"> oder kind="subtitles" und kein <track> ohne kind als Kind (HTML liest ein fehlendes kind als subtitles, ein ungültiges, auch leer oder mit Leerzeichen, als metadata), daher hat gesprochener Inhalt möglicherweise kein Textäquivalent; als Prüfhinweis gemeldet, weil Untertitel ins Bild eingebrannt oder vom Player angeboten sein können und das Video womöglich keine Sprache und keine informationstragenden Töne enthält.

rule-media-alt = Fehlende Medienalternative (AAA)
    .user-impact = Nutzer, die keine Videos oder Audios abspielen können, erhalten keine Textalternative.
    .technical-impact = Die Prüfung erkennt keine fehlende Alternative: Auf jeder mit Stufe AAA geprüften Seite steht ein Hinweis zur manuellen Prüfung, dass aufgezeichnete Video- und Audioinhalte eine vollständige Textalternative brauchen; ein gefundenes <video> oder Video-Embed wird genannt, ein Link in der Nähe mit einem Wort wie „Transkript“ nur als Hinweis, nicht als Beleg.

## Structure & semantics

rule-order-visual-mismatch = Prüfen, ob die visuelle Reihenfolge der Lesereihenfolge entspricht
    .user-impact = Screenreader-Nutzer hören Inhalte in einer anderen Reihenfolge, als sie sehende Nutzer visuell wahrnehmen.
    .technical-impact = Ein Flex-Container mit flex-direction row-reverse oder column-reverse oder ein Kind mit einer CSS-order ungleich 0 zeigt Inhalt in anderer Reihenfolge als das DOM, während DOM und Accessibility-Tree ihre Reihenfolge behalten, daher können Tab- und Lesereihenfolge vom Sichtbaren abweichen; als Prüfhinweis gemeldet, wenn das Layout gemessen wurde.

rule-headings-section-without-heading = Prüfen, ob Abschnitte eine Überschrift haben
    .user-impact = Nutzer können Inhalte nicht gezielt ansteuern.
    .technical-impact = Ein <article> oder ein benanntes <section> (mit aria-label oder aria-labelledby) enthält keine eigene Überschrift, daher bietet die Überschriftennavigation eines Screenreaders keinen Eintrag dafür an; als Prüfhinweis gemeldet, weil vom Inhalt abhängt, ob ein Abschnitt eine Überschrift braucht (WCAG 2.4.10, AAA).

rule-orientation-root-rotated = Prüfen, ob die gedrehte Wurzel eine Bildschirmausrichtung erzwingt
    .user-impact = Nutzer mit fest montierten Bildschirmen (z. B. am Rollstuhl) können die Seite nicht lesen.
    .technical-impact = Berechnetes transform (Matrix) oder rotate an html oder body dreht um eine Vierteldrehung, gemessen im geprüften Viewport.
    .description = Das Wurzelelement (html oder body) ist im geprüften Viewport um eine Vierteldrehung gedreht. Gilt die Drehung nur in einer Ausrichtung, lässt sich die Seite nur in dieser nutzen, außer die Ausrichtung ist wesentlich.
    .explanation-user-impact = Menschen, deren Gerät fest montiert ist, etwa am Rollstuhl, können die Ausrichtung nicht wechseln und sehen die Seite gedreht oder abgeschnitten.
    .typical-cause = Ein transform: rotate(90deg) oder rotate: 90deg auf html oder body, das im Hochformat das Querformat nachbilden soll.
    .recommendation = Die Drehung entfernen und das Layout in beiden Ausrichtungen nutzbar machen. Nur wenn eine Ausrichtung wesentlich ist (Klavier, Bankscheck), darf sie festgelegt werden.
    .technical-note = Gemessen wird die berechnete transform-Matrix und die Eigenschaft rotate von html und body im geprüften Viewport, der eine Ausrichtung zeigt; eine Drehung in beiden Ausrichtungen sperrt keine. orientation/locked vergleicht beide Ausrichtungen aus den Stilregeln. Ob eine Ausrichtung wesentlich ist, entscheidet ein Mensch.

rule-orientation-content-hidden = Prüfen, ob Inhalte in beiden Bildschirmausrichtungen verfügbar bleiben
    .user-impact = Nutzer mit fest montierten Bildschirmen (z. B. am Rollstuhl) können die Seite nicht lesen.
    .technical-impact = Eine Stilregel in @media (orientation: ...) setzt display: none oder visibility: hidden auf Elemente, die auf der Seite vorkommen, daher verschwindet Inhalt in einer Ausrichtung; als Prüfhinweis gemeldet, mit hoher Schwere, wenn html, body oder main ausgeblendet wird.

rule-orientation-locked = Element nur in einer Ausrichtung um eine Vierteldrehung gedreht
    .user-impact = Nutzer mit fest montierten Bildschirmen (z. B. am Rollstuhl) sehen den Inhalt quer und können das Gerät nicht drehen, um ihn zu lesen.
    .technical-impact = rotate oder ein drehendes transform unter @media (orientation) weicht um 90 Grad vom Wert in der anderen Ausrichtung ab.
    .description = Ein Element ist in einer Ausrichtung um eine Vierteldrehung anders gedreht als in der anderen. Dreht man das Gerät, dreht es sich zurück und bleibt stehen; aufrecht zeigt es nur eine Ausrichtung.
    .explanation-user-impact = Menschen, deren Gerät fest montiert ist, etwa am Rollstuhl, können die Ausrichtung nicht wechseln und sehen den Inhalt quer.
    .typical-cause = Ein transform: rotate(90deg) oder rotate: 90deg in @media (orientation: portrait) oder (orientation: landscape), das die andere Ausrichtung nachbilden soll.
    .recommendation = Die Drehung aus der Ausrichtungsabfrage entfernen und das Layout in beiden Ausrichtungen nutzbar machen. Nur wenn eine Ausrichtung wesentlich ist (Klavier, Bankscheck), darf sie festgelegt werden.
    .technical-note = Verglichen werden rotate und transform (rotate, rotateZ, rotate3d, matrix, matrix3d) jedes dargestellten Elements unter @media (orientation: landscape|portrait) mit dem Wert in der anderen Ausrichtung (ACT b33eff). Weitere Media-Bedingungen, nicht entschiedene Selektoren oder nicht auswertbare Werte ergeben eine Prüfung. Ob eine Ausrichtung wesentlich ist, entscheidet ein Mensch.

## Navigation & operation

rule-keyboard-interactive-not-focusable = Prüfen, ob interaktive Elemente den Tastaturfokus erhalten können
    .user-impact = Tastaturnutzer sind von Funktionen ausgeschlossen.
    .technical-impact = Ein Element mit interaktiver Rolle (button, link, checkbox, radio, switch, menuitem, tab, option oder treeitem) ist nicht von sich aus fokussierbar, hat keinen tabindex und ist weder deaktiviert noch über aria-activedescendant verwaltet, daher überspringt die Tabtaste es, obwohl ein Screenreader es als bedienbar ansagt; als Prüfhinweis gemeldet, weil ein Skript die Tasten behandeln kann.

rule-keyboard-trap = Tastaturfalle nicht automatisch prüfbar
    .user-impact = Seite wird faktisch unbenutzbar für Tastaturnutzer.
    .technical-impact = Eine Falle wird nicht erkannt: Auf jeder Seite mit einem fokussierbaren Element fordert ein Hinweis zur manuellen Prüfung dazu auf, die Seite per Tab zu durchlaufen, und ein modaler Dialog im Accessibility-Tree (Rolle dialog oder alertdialog mit gesetztem modal) fügt eine Warnung hinzu, zu prüfen, ob der Fokus ihn per Escape oder Tab verlassen kann; ob skriptgesteuerte Widgets den Fokus einfangen, ist ohne Bedienung nicht erkennbar.
    .description = Ob der Tastaturfokus in einem Dialog, Karussell oder individuellen JavaScript-Bedienelement hängen bleiben kann, lässt sich nicht automatisiert feststellen. Das erfordert manuelles Navigieren ausschließlich mit der Tab-Taste.
    .explanation-user-impact = Personen, die ausschließlich mit der Tastatur oder einem Screenreader navigieren, können in einem Bedienelement stecken bleiben und die Seite nicht mehr per Tastatur verlassen.
    .typical-cause = Individuelle Dialoge, Karussells oder Widgets mit eigener JavaScript-Fokusverwaltung, die keinen Weg zurück per Escape-Taste, erreichbarem Schließen-Button oder dokumentiertem Tastaturkürzel vorsehen.
    .recommendation = Für jeden fokussierbaren Bereich einen Weg zum Verlassen per Tastatur sicherstellen: Escape-Taste, ein per Tab erreichbarer, sichtbarer Schließen-Button, oder ein dokumentiertes Tastaturkürzel.
    .technical-note = Nur mit Tab/Shift+Tab durch die Seite navigieren und prüfen, ob jeder Dialog/jedes Widget verlassen werden kann. Bei eigener Fokusverwaltung: Escape-Handler und einen erreichbaren Schließen-Button implementieren.

rule-headings-none = Fehlende Sprungnavigation
    .user-impact = Tastaturnutzer müssen bei jedem Seitenwechsel durch die Navigation tabben.
    .technical-impact = Das Dokument hat keine Überschrift (<h1> bis <h6> oder role="heading"), keine main-Landmarke und keinen Sprunglink, daher lässt sich wiederholte Navigation oder ein Kopfbereich weder für Tastatur- noch für Screenreader-Nutzer überspringen; ein Verstoß, wenn eine Navigations- oder Kopf-Landmarke vorhanden ist und kein Skript-Bedienelement sie ausblenden könnte, sonst ein Prüfhinweis (auch wenn beim Messen ein modaler Dialog offen war).

rule-document-title = Fehlender oder unzureichender Seitentitel
    .user-impact = Screenreader-Nutzer können die Seite nicht identifizieren.
    .technical-impact = Das <title> ist vorhanden, aber generisch (zum Beispiel „Untitled“ oder „Home“), daher identifizieren Tab-Titel, Verlaufseintrag und der beim Laden angesagte Titel die Seite nicht.

rule-document-title-empty = Fehlender oder unzureichender Seitentitel
    .user-impact = Screenreader-Nutzer können die Seite nicht identifizieren.
    .technical-impact = Das <title>-Element ist vorhanden, enthält aber keinen Text, daher sind Browser-Tab und der beim Laden angesagte Titel leer oder fallen auf die URL zurück.

rule-document-title-missing = Fehlender oder unzureichender Seitentitel
    .user-impact = Screenreader-Nutzer können die Seite nicht identifizieren.
    .technical-impact = Das Dokument hat kein <title>-Element im <head>, daher nutzt der Browser die URL für Tab und Verlaufseintrag und ein Screenreader sagt beim Laden keinen Seitennamen an.

rule-landmarks-not-unique = Landmarks nicht eindeutig benannt
    .user-impact = Screenreader-Nutzer können gleichartige Bereiche nicht unterscheiden.
    .technical-impact = Zwei oder mehr Landmarks mit derselben Rolle haben auch denselben zugänglichen Namen oder gar keinen (etwa zwei <nav> ohne aria-label), daher zeigt die Landmark-Liste eines Screenreaders gleichlautende Einträge und Nutzer können die Bereiche nicht unterscheiden; an jedem betroffenen Element als Prüfhinweis gemeldet.

rule-dialog-focusable-missing = Dialog ohne fokussierbares Element
    .user-impact = Tastaturnutzer können den Dialog weder bedienen noch schließen.
    .technical-impact = Ein Element mit der Rolle dialog oder alertdialog (ein offenes <dialog> zählt, ein geschlossenes nicht) enthält keinen fokussierbaren Nachfahren, also weder ein von sich aus fokussierbares Element noch ein Element mit tabindex, daher kann der Tastaturfokus nicht hineingelangen; als Prüfhinweis gemeldet, weil der Dialog selbst mit tabindex="-1" den Fokus nehmen und Escape ihn schließen kann.

rule-links-generic-name = Prüfen, ob der Linktext das Linkziel benennt
    .user-impact = Screenreader-Linklisten zeigen nichtssagende Texte.
    .technical-impact = Der zugängliche Name eines <a> ist eine generische Wendung wie „hier klicken“ oder „mehr“, die ohne Umgebung keinen Zweck nennt, daher klingen in der Linkliste eines Screenreaders alle solchen Links gleich.

rule-links-single-character-name = Prüfen, ob der Linktext das Linkziel benennt
    .user-impact = Screenreader-Linklisten zeigen nichtssagende Texte.
    .technical-impact = Der zugängliche Name eines <a> besteht aus einem einzelnen Zeichen wie „>“ oder „i“, das ein Screenreader als Symbolname oder gar nicht vorliest und das das Linkziel nicht benennt.

rule-links-url-name = Prüfen, ob der Linktext das Linkziel benennt
    .user-impact = Screenreader-Linklisten zeigen nichtssagende Texte.
    .technical-impact = Der zugängliche Name eines <a> ist die rohe URL, die ein Screenreader Zeichen für Zeichen vorliest und die nichts über den Inhalt des Ziels aussagt.

rule-focus-visible = Seite ohne fokussierbare Elemente
    .user-impact = Tastaturnutzer finden nichts, was sie per Tab erreichen können.
    .technical-impact = Der Accessibility-Tree einer Seite mit mehr als fünf Knoten enthält außer der Dokumentwurzel keinen fokussierbaren oder interaktiven Knoten, daher hat ein Tastaturnutzer keinen Fokus, den er sehen oder bewegen könnte, oft weil Bedienelemente aus einfachen Elementen gebaut sind; nur als Warnung gemeldet, weil WCAG 2.4.7 ohne tastaturbedienbare Oberfläche nicht gilt.

rule-focus-obscured = Fokussiertes Element vollständig verdeckt
    .user-impact = Tastaturnutzer springen per Tab auf ein Element, das durch einen Sticky-Header, -Footer, ein Cookie-Banner o. ä. vollständig verdeckt ist und daher nicht sichtbar ist.
    .technical-impact = Fokuslauf: Element fokussiert, mit scrollIntoView({"{"}block:'nearest'{"}"}) ins Bild gescrollt, 3×3 Punkte per elementsFromPoint abgetastet; alle Punkte unter einem position: fixed/sticky-Element.
    .description = Nach dem Fokussieren und Scrollen liegt jeder Abtastpunkt eines Bedienelements unter einem fixierten oder klebenden Element, etwa einer Kopfleiste oder einem Cookie-Banner.
    .explanation-user-impact = Tastaturnutzer springen mit Tab auf ein Element, das sie nicht sehen, und wissen nicht, wo der Fokus ist.
    .typical-cause = Eine fixierte oder klebende Leiste, die tiefer reicht als scroll-padding-top, oder ein Banner, das Bedienelemente der Seite überdeckt.
    .recommendation = scroll-padding-top mindestens so groß wie die fixierte Leiste setzen und Banner so einbauen, dass sie keine Bedienelemente überdecken.
    .technical-note = Gemessen wird nach dem Fokussieren mit scrollIntoView({"{"} block: 'nearest' {"}"}), 3 mal 3 Punkte je Bedienelement per elementsFromPoint. Ein echtes Tab scrollt womöglich anders; der Wert ist deshalb ein Hinweis zum Prüfen, kein Beleg. Auch der Hinweis auf eine Leiste, die tiefer reicht als scroll-padding-top, gehört zu dieser Kennung.

rule-focus-obscured-enhanced = Fokussiertes Element teilweise verdeckt
    .user-impact = Tastaturnutzer springen per Tab auf ein Element, dessen sichtbarer Bereich durch einen Sticky-Header, -Footer, ein Cookie-Banner o. ä. teilweise verdeckt wird.
    .technical-impact = Fokuslauf wie bei focus/obscured; mindestens ein Abtastpunkt unter einem position: fixed/sticky-Element.
    .description = Nach dem Fokussieren und Scrollen liegt mindestens ein Abtastpunkt eines Bedienelements unter einem fixierten oder klebenden Element (Stufe AAA).
    .explanation-user-impact = Tastaturnutzer sehen ein fokussiertes Bedienelement nur zum Teil und erkennen Beschriftung oder Zustand womöglich nicht.
    .typical-cause = Ein klebender Fuß, Chat-Button oder Cookie-Banner, der den Rand der Seite überdeckt.
    .recommendation = Platz unter und über fixierten Elementen freihalten (scroll-padding, Seitenabstand) oder die Elemente verkleinern.
    .technical-note = Dieselbe Messung wie bei focus/obscured, andere Schwelle: jeder überdeckte Abtastpunkt wird gemeldet. Die Abtastung nähert „irgendein Teil“ nur an, das Ergebnis bleibt ein Hinweis zum Prüfen.

rule-label-in-name-mismatch = Sichtbarer Text ≠ zugänglicher Name
    .user-impact = Sprachsteuerungsnutzer können Element nicht ansprechen.
    .technical-impact = Der zugängliche Name, den aria-label oder aria-labelledby einem Button, Link oder anderem aus seinem Inhalt benannten Bedienelement gibt, enthält dessen sichtbare Beschriftung nicht, daher kann ein Sprachsteuerungs-Nutzer, der sagt, was zu sehen ist, es nicht auslösen; ein Prüfhinweis, wenn dieselben Wörter nur in anderer Reihenfolge stehen oder der sichtbare Text mehr als doppelt so lang ist wie der Name; reine Symbol- und Ein-Zeichen-Beschriftungen sind ausgenommen.

rule-manual-timing = Prüfen, ob sich Zeitbegrenzungen anpassen lassen
    .user-impact = Wenn sich eine Zeitbegrenzung nicht anpassen lässt, werden Nutzer, die mehr Zeit brauchen, abgebrochen und können Eingaben verlieren.
    .technical-impact = Die Seite hat Formularfelder oder ein <meta http-equiv="refresh">, daher ist eine Zeitbegrenzung denkbar; Zeitbegrenzungen per JavaScript (setTimeout, setInterval, Sitzungsablauf) sind im Markup nicht erkennbar, daher erhält die Seite einen Hinweis zur manuellen Prüfung, dass Nutzer eine solche Frist abschalten, anpassen oder verlängern können müssen; die Verzögerung eines <meta http-equiv="refresh"> wird getrennt bewertet.

rule-timing-meta-refresh = Nicht anpassbare Zeitbegrenzung
    .user-impact = Nutzer benötigen mehr Zeit für die Eingabe und können durch Timeouts Daten verlieren.
    .technical-impact = Ein <meta http-equiv="refresh"> mit einer Verzögerung von mehr als 0 Sekunden und höchstens 20 Stunden lädt die Seite nach dieser Frist neu oder leitet um, und das Markup bietet keine Möglichkeit, sie anzuhalten oder zu verlängern, daher wird abgebrochen, wer noch liest; eine sofortige Weiterleitung (0) und Fristen über 20 Stunden werden nicht gemeldet.

rule-timing-meta-refresh-enhanced = Seite lädt sich nach einer Frist neu oder leitet weiter (AAA)
    .user-impact = Nutzer werden unterbrochen: Die Seite wechselt ohne Anforderung, Screenreader-Nutzer verlieren ihre Leseposition.
    .technical-impact = Ein <meta http-equiv="refresh"> mit einer Frist über 0 lädt die Seite nach dieser Frist neu oder leitet weiter, auch über 20 Stunden.
    .description = Die Seite lädt sich per <meta http-equiv="refresh"> nach einer Frist neu oder leitet weiter, ohne dass jemand es angefordert hat.
    .explanation-user-impact = Wer langsam liest oder einen Screenreader nutzt, wird unterbrochen, verliert die Leseposition oder landet auf einer Seite, die er nicht angefordert hat.
    .typical-cause = <meta http-equiv="refresh" content="300">, um Nachrichten oder ein Dashboard aufzufrischen, oder eine verzögerte Weiterleitung auf eine neue Adresse.
    .recommendation = Auf dem Server oder mit Frist 0 weiterleiten und Inhalte von Nutzern selbst neu laden lassen.
    .technical-note = AAA (WCAG 2.2.4 und 3.2.5, ACT bisz58): Jede Frist über 0 zählt, auch über 20 Stunden; die Ausnahme von 2.2.1 gilt hier nicht. Eine sofortige Weiterleitung (0) ist keine Unterbrechung. Es zählt nur die erste gültige Anweisung, wie im Browser.

rule-motion-infinite-animation = Prüfen, ob sich endlose Bewegung anhalten, stoppen oder ausblenden lässt
    .user-impact = Nutzer mit Aufmerksamkeits- oder kognitiven Einschränkungen sowie Screenreader-Nutzer werden durch dauerhaft laufende Bewegung abgelenkt oder gestört und können sie nicht deaktivieren.
    .technical-impact = Ein <marquee> oder ein Element, dessen Animation sich endlos wiederholt (animation-iteration-count: infinite), ist vorhanden und die Seite hat keinen Schalter oder Button, dessen Beschriftung anbietet, sie anzuhalten, zu stoppen oder auszublenden; als Prüfhinweis gemeldet, weil die Dauer der Bewegung und die Wirkung eines solchen Bedienelements nicht gemessen werden.

rule-manual-no-timing = Prüfen, ob keine Aufgabe von Zeit abhängt (AAA)
    .user-impact = Hängt eine Aufgabe von Zeit ab, können Nutzer, die mehr Zeit brauchen, sie nicht in ihrem eigenen Tempo abschließen.
    .technical-impact = Die Seite hat Formularfelder oder ein <meta http-equiv="refresh">, daher sind Zeitbegrenzungen denkbar; ob eine Aufgabe von Zeit abhängt, ist im Markup nicht erkennbar, die Seite erhält deshalb einen Hinweis zur manuellen Prüfung statt eines Urteils, und offen bleibt, ob Nutzer, die mehr Zeit brauchen, abgebrochen werden.

rule-manual-interruptions = Prüfen, ob sich Unterbrechungen aufschieben oder abschalten lassen (AAA)
    .user-impact = Wenn sich Unterbrechungen nicht aufschieben oder abschalten lassen, werden Nutzer mit kognitiven Einschränkungen oder Screenreader-Nutzer abgelenkt.
    .technical-impact = Die Seite enthält ein <dialog>, ein <meta http-equiv="refresh"> oder ein Element mit der Rolle alert, alertdialog, status, log, marquee oder timer oder mit einem aria-live-Wert ungleich off; solche Inhalte können die Screenreader-Ausgabe und das Lesen unterbrechen, ob sie sich aufschieben oder abschalten lassen, ist im Markup nicht erkennbar, daher erhält die Seite einen Hinweis zur manuellen Prüfung.

rule-manual-re-authenticating = Prüfen, ob Eingaben die erneute Anmeldung überstehen (AAA)
    .user-impact = Gehen eingegebene Daten bei der erneuten Anmeldung verloren, müssen Nutzer alle Formulardaten erneut eingeben.
    .technical-impact = Die Seite hat ein Passwortfeld oder ein CAPTCHA, daher ist eine Anmeldung nach abgelaufener Sitzung wahrscheinlich; ob die eingegebenen Daten die erneute Anmeldung überstehen, ist im Markup nicht erkennbar, die Seite erhält deshalb einen Hinweis zur manuellen Prüfung statt eines Urteils.

rule-navigation-location-missing = Prüfen, ob der aktuelle Ort angezeigt wird (AAA)
    .user-impact = Nutzer verlieren die Orientierung auf verschachtelten Webseiten.
    .technical-impact = Die Seite hat einen Link in einem <nav> oder role="navigation", aber kein Element mit aria-current="page" und kein Element, dessen aria-label „breadcrumb“ enthält, daher zeigt das Markup den aktuellen Ort nicht und ein Screenreader sagt in der Navigation keine aktuelle Seite an; nur als Prüfhinweis gemeldet, weil auch Titel, Überschriften oder eine Sitemap das Kriterium erfüllen können.

rule-links-purpose-link-only = Prüfen, ob der Linktext den Zweck ohne Umfeld nennt (AAA)
    .user-impact = Nutzer müssen den gesamten Absatz lesen, um das Ziel eines Links zu verstehen.
    .technical-impact = Der Text eines <a> ist für sich allein eine vage Wendung wie „Mehr“, „Hier klicken“ oder „Details“ und nur sein Umfeld nennt das Ziel, daher liest die Linkliste eines Screenreaders dieselben vagen Namen vor und der Zweck ist aus dem Linktext allein nicht erkennbar (WCAG 2.4.9, AAA); als Prüfhinweis gemeldet.

rule-links-new-window-unannounced = Neues Fenster ohne Ankündigung (AAA)
    .user-impact = Nutzer verlieren unerwartet den Kontext; der Zurück-Button führt nicht zurück.
    .technical-impact = Ein Link mit target="_blank" hat in seinem zugänglichen Namen keinen Hinweis (eine Wendung wie „neues Fenster“, „öffnet in“ oder „externer Link“), daher erfährt ein Screenreader-Nutzer nicht, dass das Aktivieren ein neues Fenster oder einen neuen Tab öffnet; als Prüfhinweis gemeldet.
    .description = Ein Link öffnet ein neues Fenster oder Tab (target="_blank"), ohne das im Linktext anzukündigen. WCAG 3.2.5 (AAA) verlangt, dass solche Kontextwechsel nur auf Anforderung geschehen; die Ankündigung im Linktext ist die übliche Lösung.
    .explanation-user-impact = Screenreader- und Lupennutzer merken den Fensterwechsel oft nicht; der Zurück-Button führt dann nicht zur vorherigen Seite.
    .typical-cause = Externe Links oder PDF-Downloads, die per target="_blank" geöffnet werden.
    .recommendation = Im Linktext ankündigen, z. B. "(öffnet in neuem Fenster)", oder auf target="_blank" verzichten.
    .technical-note = Ein sichtbares Symbol reicht nur mit Textalternative im zugänglichen Namen (Technik G201).

rule-manual-pointer-gestures = Prüfen, ob Gesten eine Bedienung mit einem einzelnen Zeiger haben
    .user-impact = Hat eine Geste keine Alternative mit einem einzelnen Zeiger, können Nutzer mit motorischen Einschränkungen die Funktion nicht bedienen.
    .technical-impact = Die Seite enthält ein Element, das nach Gestensteuerung aussieht: ein Handler-Attribut für touchstart, touchmove, pointermove, gesturestart oder dragstart, draggable="true" oder eine Klasse einer Karussell- oder Kartenbibliothek wie swiper, slick, splide, leaflet oder mapboxgl; ob dieselbe Funktion auch mit einem einzelnen Tippen oder Klick geht, ist nicht erkennbar, daher erhält die Seite einen Hinweis zur manuellen Prüfung.

rule-manual-pointer-cancellation = Prüfen, ob sich Zeigeraktionen abbrechen lassen
    .user-impact = Löst eine Aktion schon beim Herunterdrücken aus, können Nutzer sie versehentlich auslösen, ohne sie abbrechen zu können.
    .technical-impact = Ein Element trägt ein Handler-Attribut onmousedown, onpointerdown oder ontouchstart, daher löst eine Aktion womöglich schon beim Herunterdrücken aus; ob sie sich abbrechen oder zurücknehmen lässt oder erst beim Loslassen auslöst, ist im Markup nicht erkennbar, daher erhält die Seite einen Hinweis zur manuellen Prüfung; per Skript angehängte Handler werden nicht untersucht.

rule-manual-motion-actuation = Prüfen, ob Steuerung per Gerätebewegung eine Alternative hat
    .user-impact = Geht eine Funktion nur durch Bewegen des Geräts, sind Nutzer, die das Gerät nicht bewegen können, von ihr ausgeschlossen.
    .technical-impact = Ein Inline-<script> nennt devicemotion oder deviceorientation, daher reagiert eine Funktion womöglich auf Schütteln oder Neigen des Geräts; ob sie auch über ein Bedienelement auf der Seite geht und ob sich die Reaktion abschalten lässt, ist nicht erkennbar, daher erhält die Seite einen Hinweis zur manuellen Prüfung; externe Skripte werden nicht untersucht.

rule-targets-size-enhanced = Prüfen, ob Klickziele die erweiterte Größe von 44×44 px erreichen (AAA)
    .user-impact = Nutzer mit zittrigen Händen oder auf Mobilgeräten klicken leicht auf falsche Elemente.
    .technical-impact = Die gerenderte Box eines Links, Buttons oder Formularelements ist kleiner als 44 mal 44 CSS-Pixel, daher trifft Zeiger- und Touch-Eingabe leicht ein benachbartes Element; 44 px ist die erweiterte Stufe AAA (2.5.5) und eine Ergonomie-Empfehlung, das Minimum von WCAG 2.2 AA (2.5.8) sind 24 mal 24 CSS-Pixel.

rule-targets-size = Prüfen, ob Klickziele mindestens 24×24 px groß sind
    .user-impact = Nutzer mit zittrigen Händen oder auf Mobilgeräten klicken leicht auf falsche Elemente.
    .technical-impact = Die gerenderte Box eines Links, Buttons oder Formularelements ist kleiner als 24 mal 24 CSS-Pixel und keine Abstandsausnahme greift, daher trifft Zeiger- und Touch-Eingabe leicht ein benachbartes Element.

## Forms & interaction

rule-forms-autocomplete-invalid = Fehlende Eingabezweck-Kennzeichnung
    .user-impact = Kein Autofill möglich, Eingabe fehleranfälliger.
    .technical-impact = Ein Eingabefeld hat ein autocomplete-Attribut, dessen Wert in der HTML-Token-Grammatik ungültig ist, daher ignoriert der Browser es, bietet kein Autofill und gibt der Hilfstechnik keinen Eingabezweck weiter.

rule-forms-purpose-missing = Prüfen, ob Eingabefelder ihren Zweck kennzeichnen
    .user-impact = Kein Autofill möglich, Eingabe fehleranfälliger.
    .technical-impact = Ein Text-, E-Mail-, Telefon-, URL- oder Passwortfeld, das offenbar nach der Person fragt (aus Beschriftung, id, name oder Typ geraten), hat kein autocomplete-Token oder nur on, off oder einen leeren Wert, daher kann der Browser es nicht ausfüllen und sein Zweck ist nicht programmatisch bestimmbar; als Prüfhinweis gemeldet, weil die Vermutung falsch sein kann.

rule-forms-group-name-missing = Formulargruppe ohne Namen
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Ein <fieldset> ohne <legend> oder ein Element mit role="group" bzw. role="radiogroup" ohne zugänglichen Namen sorgt dafür, dass ein Screenreader die enthaltenen Felder ohne die zugehörige Frage ansagt.

rule-forms-instructions-missing = Prüfen, ob Felder Eingabehinweise haben
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Die Beschriftung eines Feldes nennt Daten mit festem Format (etwa Datum, Telefonnummer, Postleitzahl, Karten- oder Passnummer), aber das Feld hat kein aria-describedby, kein Formatbeispiel in der Beschriftung und keinen Platzhalter, daher sagt ein Screenreader den Feldnamen, aber nicht das erwartete Format an; aus dem Beschriftungstext erkannt und deshalb ein Prüfhinweis.

rule-forms-label-missing = Fehlende Formularbeschriftungen
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Ein Formularfeld hat keine programmatisch zugeordnete Beschriftung (<label for>, umschließendes <label>, aria-label, aria-labelledby oder title), daher ist sein zugänglicher Name leer und ein Screenreader sagt nur den Feldtyp an.

rule-forms-placeholder-as-label = Prüfen, ob Felder neben dem Platzhalter eine Beschriftung haben
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Ein Textfeld hat einen Platzhalter, aber weder ein <label for> noch ein umschließendes <label> noch aria-labelledby (ein aria-label allein hilft nicht, da es nicht sichtbar ist), daher stammt sein Name aus dem Platzhalter, der verschwindet, sobald getippt wird; als Prüfhinweis gemeldet.

rule-forms-required-unmarked = Pflichtfeld nicht als Pflicht gekennzeichnet
    .user-impact = Nutzer wissen nicht, welche Eingaben erwartet werden.
    .technical-impact = Ein Bedienelement trägt required oder aria-required="true", aber weder sein zugänglicher Name noch seine aria-describedby-Beschreibung enthält eine Kennzeichnung wie „*“ oder das Wort „Pflicht“ bzw. „required“; der Zustand erreicht den Accessibility-Tree, ob aber ein Hinweis an anderer Stelle, etwa über dem Formular, sehende Nutzer informiert, kann nur ein Mensch beurteilen; als Prüfhinweis gemeldet.

rule-forms-redundant-entry = Prüfen, ob bereits eingegebene Daten nicht erneut verlangt werden
    .user-impact = Nutzer müssen bereits eingegebene Daten erneut eintippen, was Zeit kostet und besonders für Menschen mit motorischen oder kognitiven Einschränkungen eine zusätzliche Hürde darstellt.
    .technical-impact = Ein <form> verlangt dieselbe Art Angabe (Name, E-Mail-Adresse, Telefonnummer, Straße oder Postleitzahl) in mehr als einem Feld, die Felder haben weder einen passenden autocomplete-Wert noch einen vorbelegten Wert, und das Formular bietet kein „wie oben“ an, daher müssen Nutzer die Daten erneut tippen; als Prüfhinweis gemeldet, weil nur ein Mensch beurteilen kann, ob beide Felder dieselbe Angabe meinen (Bestätigungs- und Einmalcode-Felder sind ausgenommen).

rule-auth-paste-blocked = Einfügen in Passwort- oder Code-Feld blockiert
    .user-impact = Nutzer können weder einen Passwort-Manager noch die Zwischenablage verwenden. Wer sich Passwörter nicht merken oder fehlerfrei abschreiben kann, etwa wegen kognitiver oder motorischer Einschränkungen, scheitert an der Anmeldung.
    .technical-impact = Ein synthetisches paste-Ereignis wird abgebrochen, und der Probetext steht danach nicht im Feld (paste-Handler mit preventDefault() oder return false); gemessen auch in gleich-ursprünglichen Rahmen und offenen Shadow Roots.
    .description = Ein Passwort- oder Einmalcode-Feld bricht das paste-Ereignis ab und fügt den Text nicht selbst ein. Passwortmanager und die Zwischenablage funktionieren dort nicht.
    .explanation-user-impact = Wer sich Passwörter nicht merken oder fehlerfrei abschreiben kann, etwa wegen kognitiver oder motorischer Einschränkungen, scheitert an der Anmeldung.
    .typical-cause = Ein paste-Handler mit preventDefault() oder onpaste="return false", meist als vermeintliche Sicherheitsmaßnahme.
    .recommendation = Den Handler entfernen. Wird der Text bereinigt, den bereinigten Text selbst in das Feld einfügen, statt das Einfügen zu verbieten.
    .technical-note = Gemessen wird ein synthetisches, abbrechbares paste-Ereignis an Passwort- und Codefeldern, auch in gleich-ursprünglichen Rahmen und offenen Shadow Roots; der Feldwert wird danach wiederhergestellt. Das Ereignis ist nicht vertrauenswürdig (isTrusted ist false): Ein Handler, der nur auf echte Ereignisse oder Tastendrücke reagiert, bleibt unsichtbar. autocomplete="off" ist kein Befund.

rule-auth-captcha = CAPTCHA im Anmeldeformular prüfen
    .user-impact = Text- und Rätsel-CAPTCHAs sperren Menschen mit kognitiven Einschränkungen, Legasthenie oder Sehbehinderung von der Anmeldung aus, wenn es keine Alternative gibt.
    .technical-impact = Ein Formular mit Passwort- oder Einmalcode-Feld enthält ein sichtbares CAPTCHA-Widget (reCAPTCHA, hCaptcha oder ein Bild, dessen src, alt, id oder class ein Captcha nennt), daher können Menschen, die einen kognitiven Test nicht lösen können, sich nicht anmelden, solange keine Alternative besteht; als Prüfhinweis gemeldet, weil Art des Tests und eine mögliche Alternative nicht im Markup stehen.
    .description = Ein Anmeldeformular enthält ein Captcha. Ob es ein Objekterkennungstest ist oder eine Alternative ohne Denkaufgabe angeboten wird, muss geprüft werden.
    .explanation-user-impact = Wer verzerrte Zeichen nicht lesen, abschreiben oder sich merken kann, kommt an der Anmeldung nicht vorbei.
    .typical-cause = Ein Text- oder Bildrätsel-Captcha gegen Bots, ohne zweiten Weg.
    .recommendation = Eine Anmeldung ohne Denkaufgabe anbieten, etwa Passkey, E-Mail-Link oder eine Bot-Prüfung ohne Interaktion. Ein Objekterkennungstest ist auf Stufe AA zulässig.
    .technical-note = Prüfhinweis: Art des Tests und Alternativen stehen nicht im Markup. Ein Captcha im Kontaktformular gehört nicht zu 3.3.8.

rule-forms-no-submit = Formular ohne Absende-Button
    .user-impact = Tastatur- und Screenreader-Nutzer erkennen nicht, wann ihre Eingabe gilt, und ein Skript, das beim Ändern abschickt, versetzt sie unangekündigt in einen neuen Kontext.
    .technical-impact = Ein <form> besitzt mindestens ein aktives input (außer hidden), select, textarea oder button (auch über das form-Attribut), aber kein aktives Absende-Element, also keinen <button> außer mit type button oder reset, kein <input type="submit"> oder type "image" und kein role="button"-Element mit data-submit oder einer Beschriftung mit submit, send oder search; immer ein Prüfbefund, weil 3.2.2 erst verletzt ist, wenn eine Feldänderung den Kontext wechselt, und mit niedriger Schwere, wenn das Formular weder action noch ein einzeiliges Eingabefeld hat (textarea, Checkbox, Radio, Datei, Schieberegler und Farbfeld zählen nicht), in dem die Eingabetaste es absenden würde, und nur ein Skript es absenden kann.
    .description = Ein Formular sammelt Eingaben, hat aber keinen Button und kein anderes Element, mit dem es ausdrücklich abgeschickt wird.
    .explanation-user-impact = Ohne Absende-Element bleibt unklar, wann die Eingabe gilt. Wird das Formular stattdessen beim Ändern eines Felds abgeschickt, wechselt der Kontext unangekündigt.
    .typical-cause = Filter- und Suchformulare, die per Skript beim Ändern eines Felds abgeschickt werden, oder ein Button außerhalb des Formulars ohne form-Attribut.
    .recommendation = Einen Button type="submit" ins Formular setzen oder per form-Attribut zuordnen. Ändert ein Formular nur Inhalte auf der Seite, genügt eine Prüfung, dass dabei kein Kontextwechsel passiert.
    .technical-note = Ohne action und ohne einzeiliges Eingabefeld kann nur ein Skript das Formular abschicken; dann ist der Befund ein Prüfhinweis (#728).

rule-manual-help = Prüfen, ob Formularfelder Hilfe bieten (AAA)
    .user-impact = Bieten Felder keine Hilfe, wissen Nutzer womöglich nicht, welches Format oder welche Daten einzugeben sind.
    .technical-impact = Die Seite enthält Formularfelder (input, select oder textarea; versteckte Felder und Schaltflächen zählen nicht); ob sie kontextbezogene Hilfe wie Anleitung, Beispiele oder einen Hilfelink haben, lässt sich aus dem Markup nicht entscheiden, daher erhält die Seite einen Hinweis zur manuellen Prüfung statt eines Urteils.

## Language & clarity

rule-document-lang-missing = Fehlende Sprachangabe
    .user-impact = Screenreader liest mit falscher Aussprache vor.
    .technical-impact = Das <html>-Element hat kein lang-Attribut oder ein leeres, daher ist die Dokumentsprache undefiniert und ein Screenreader fällt auf seine Standardstimme zurück und spricht den Text falsch aus.

rule-html-has-lang = Fehlende Sprachangabe
    .user-impact = Screenreader liest mit falscher Aussprache vor.
    .technical-impact = Das <html>-Element eines Dokuments in einem <iframe> gleichen Ursprungs hat kein lang-Attribut (das Hauptdokument meldet die Regel zur Dokumentsprache), daher liest ein Screenreader den eingebetteten Inhalt mit seiner Standardstimme.

rule-context-autofocus = Prüfen, ob autofocus den Fokus nicht unerwartet verschiebt
    .user-impact = Verwirrende Seitenänderungen für Tastaturnutzer.
    .technical-impact = Ein Element, das kein Eingabefeld ist (<input>, <select>, <textarea> oder eine Textrolle), trägt das autofocus-Attribut, daher springt der Fokus beim Laden dorthin und ein Screenreader beginnt mitten auf der Seite, ohne den Inhalt davor; als Prüfhinweis gemeldet, weil autofocus allein den Kontext nicht wechselt.

rule-context-on-focus = Prüfen, ob der Fokuserhalt den Kontext nicht ändert
    .user-impact = Verwirrende Seitenänderungen für Tastaturnutzer.
    .technical-impact = Ein Element, das weder von sich aus interaktiv ist noch eine interaktive Rolle hat, trägt einen onfocus-Handler, der in dem Moment läuft, in dem der Fokus eintrifft, und den Kontext wechseln kann (navigieren, absenden, ein Fenster öffnen); was er tut, zeigt das Markup nicht, daher wird er als Prüfhinweis gemeldet.

rule-context-on-input = Unerwartete Kontextänderung bei Eingabe
    .user-impact = Verwirrende Änderungen beim Ausfüllen.
    .technical-impact = Ein <select>, Optionsfeld, Kontrollkästchen, eine Combobox oder Listbox hat einen onchange-Handler, dessen Code navigiert, das Formular absendet, ein Fenster öffnet oder den Fokus verschiebt (oder eine Funktion aufruft, die das tut), daher wechselt schon das Auswählen einer Option per Tastatur den Kontext ohne Vorwarnung; ein Handler, dessen Wirkung das Markup nicht zeigt, ist nur ein Prüfhinweis.

rule-manual-unusual-words = Prüfen, ob ungewöhnliche Wörter erklärt werden (AAA)
    .user-impact = Werden ungewöhnliche Wörter nicht erklärt, verstehen Nutzer mit kognitiven Einschränkungen oder Fremdsprachler den Text womöglich nicht.
    .technical-impact = Die Seite enthält Text, daher fragt ein Hinweis zur manuellen Prüfung, ob Fachbegriffe, Redewendungen und ungewöhnliche Wörter im Text, mit <dfn> oder in einem Glossar erklärt werden; aus dem Markup lässt sich nicht erkennen, welche Wörter für die Leser ungewöhnlich sind.

rule-language-part-undetermined = Anderssprachige Passagen von Hand auf lang prüfen
    .user-impact = Screenreader lesen den Abschnitt mit der Aussprache der Seitensprache vor; er wird schwer verständlich.
    .technical-impact = Die Sprache von Textblöcken lässt sich nicht prüfen, weil die Seite weder Deutsch noch Englisch ist, die einzigen Sprachen der heuristischen Erkennung, daher bleiben fehlende lang-Attribute an Textstellen ungeprüft.

rule-language-part-invalid = Ungültiger Sprachcode an einem Textteil
    .user-impact = Screenreader können für diesen Abschnitt nicht auf die richtige Aussprache wechseln.
    .technical-impact = Ein Element unterhalb von <html> hat ein lang, das kein gültiges BCP-47-Kürzel ist (etwa „dutch“ oder „eng“), und trägt eigenen Text, daher kann ein Screenreader die Stimme für diese Passage nicht umschalten; ein leeres lang="" wird nicht gemeldet.

rule-language-part-unmarked = Anderssprachiger Textteil ohne lang prüfen
    .user-impact = Screenreader lesen den Abschnitt mit der Aussprache der Seitensprache vor; er wird schwer verständlich.
    .technical-impact = Ein Textblock wird als andere Sprache als <html lang> erkannt, steht aber in einem Element ohne eigenes lang-Attribut, daher liest ihn ein Screenreader mit der Aussprache der Seitensprache vor.
    .description = Ein Textabschnitt scheint in einer anderen Sprache verfasst als die Seite, ohne dass ein lang-Attribut das auszeichnet.
    .explanation-user-impact = Screenreader lesen den Abschnitt mit der Aussprache der Seitensprache vor; er wird schwer verständlich.
    .typical-cause = Zitate, Produktnamen oder Teaser in einer anderen Sprache ohne lang.
    .recommendation = Den Abschnitt mit lang auszeichnen, etwa <p lang="en">.
    .technical-note = Prüfhinweis: Die Spracherkennung ist eine Heuristik und kennt Deutsch und Englisch. Auf Seiten in anderen Sprachen bleibt die Prüfung als ungeprüft vermerkt.

rule-language-abbreviation-unexpanded = Prüfen, ob Abkürzungen erklärt werden (AAA)
    .user-impact = Nutzer verstehen den Inhalt nicht, wenn sie die Abkürzung nicht kennen.
    .technical-impact = Ein <abbr>- oder <acronym>-Element hat kein title-Attribut, daher liefert das Markup keine Langform der Abkürzung; als Prüfhinweis gemeldet, weil die Langform im Text oder in einem Glossar stehen kann (WCAG 3.1.4, AAA).

## Visual presentation / contrast

rule-contrast-text-insufficient = Unzureichender Farbkontrast
    .user-impact = Text bei Sehschwäche oder ungünstigen Lichtverhältnissen schwer lesbar.
    .technical-impact = Die berechnete Textfarbe hat gegenüber dem aufgelösten Hintergrund ein Kontrastverhältnis unter 4,5:1 (3:1 bei großem Text), gemessen an den gerenderten Stilen; Hilfstechnik kann das nicht ausgleichen.

rule-contrast-text-undetermined = Prüfen, ob der Kontrast von nicht messbarem Text ausreicht
    .user-impact = Text bei Sehschwäche oder ungünstigen Lichtverhältnissen schwer lesbar.
    .technical-impact = Der Kontrast dieses Textes ließ sich nicht entscheiden: Die Vorder- oder Hintergrundfarbe war nicht auflösbar (etwa bei einem Bild- oder Verlaufshintergrund), ein fixiertes oder klebendes Element überdeckt den Text, das gemessene Verhältnis liegt nahe an der Grenze oder schwankt über den Text, oder der Text unterschreitet es in einer aria-hidden-svg-Illustration, wo nur ein Mensch Dekoration (ausgenommen) von Information unterscheiden kann, daher bleibt das geforderte Verhältnis von 4,5:1 (3:1 bei großem Text) ungeprüft und braucht eine manuelle Prüfung.

rule-contrast-text-enhanced = Erhöhter Farbkontrast nicht erreicht (AAA)
    .user-impact = Für Menschen mit deutlich eingeschränktem Sehvermögen bleibt der Text schwer lesbar.
    .technical-impact = Kontrastverhältnis unter 7:1 für normalen Text bzw. 4,5:1 für großen Text.

rule-zoom-viewport-locked = Zoom durch Viewport-Meta-Tag gesperrt
    .user-impact = Nutzer mit Sehschwäche verlieren Inhalte bei Vergrößerung.
    .technical-impact = <meta name="viewport"> setzt user-scalable=no oder ein maximum-scale unter 2, daher sperrt der Browser das Zoomen per Pinch über diese Grenze hinaus, auf das Nutzer mit Sehbeeinträchtigung angewiesen sind.
    .description = Das viewport-Meta-Tag verhindert das Zoomen (user-scalable=no) oder begrenzt es auf weniger als 200 %.
    .explanation-user-impact = Menschen mit Sehbeeinträchtigung können die Seite auf Mobilgeräten nicht vergrößern und verlieren Inhalte.
    .typical-cause = user-scalable=no oder maximum-scale=1, gesetzt, um versehentliches Zoomen zu verhindern oder ein Layout zu stabilisieren.
    .recommendation = user-scalable=no und maximum-scale aus dem viewport-Meta-Tag entfernen.
    .technical-note = Eine Grenze zwischen 200 % und 500 % ist kein Verstoß gegen 1.4.4 und wird getrennt gemeldet.

rule-zoom-viewport-missing = Fehlende Viewport-Angabe
    .user-impact = Auf Mobilgeräten erscheint der Text unter jeder lesbaren Größe; wer vergrößert, muss waagerecht scrollen. Am Desktop ändert sich nichts.
    .technical-impact = Das Dokument hat kein <meta name="viewport">, daher legen mobile Browser eine Desktop-Breite zugrunde und verkleinern die Seite, und Text erscheint unter lesbarer Größe, bis die Nutzer hineinzoomen; ein Best-Practice-Befund ohne WCAG-Kriterium.
    .description = Das Dokument hat kein viewport-Meta-Tag. Mobile Browser legen dann eine Desktop-Breite zugrunde und verkleinern die Seite.
    .explanation-user-impact = Auf Mobilgeräten erscheint der Text unter jeder lesbaren Größe; wer vergrößert, muss waagerecht scrollen. Am Desktop ändert sich nichts.
    .typical-cause = Das Template setzt kein viewport-Meta-Tag, etwa bei älteren oder rein für Desktop gebauten Seiten.
    .recommendation = Im <head> ein viewport-Meta-Tag mit width=device-width ergänzen.
    .technical-note = Desktop-Browser ignorieren das Tag; betroffen ist nur die Darstellung auf Mobilgeräten. Das Layout muss danach auch in schmalen Viewports funktionieren (1.4.10 Reflow).

rule-css-overflow-hidden = Kein Reflow bei 320 CSS-Pixeln
    .user-impact = Nutzer, die auf 400 % zoomen, müssen in zwei Richtungen scrollen — Inhalte werden unzugänglich.
    .technical-impact = Bei einem Viewport von 320 CSS-Pixeln Breite läuft die Seite wegen eines Elements außerhalb der Ausnahmen von 1.4.10 (Datentabellen, Bilder, Karten, Diagramme und Video dürfen scrollen) weiterhin waagerecht über, daher müssen Nutzer, die auf 400 % vergrößern, in zwei Richtungen scrollen; im Browser gemessen, nicht aus dem CSS gelesen.

rule-contrast-control-insufficient = Unzureichender Kontrast bei UI-Elementen
    .user-impact = Interaktive Elemente schwer erkennbar.
    .technical-impact = Ein von der Seite selbst gezeichnetes Kontrollkästchen, Optionsfeld, Schalter oder Range-Eingabefeld (appearance: none) hat an seinem Rahmen, ohne Rahmen an der eigenen Fläche, ein Kontrastverhältnis unter 3:1 gegenüber dem Hintergrund des Elternelements, gemessen an den berechneten Stilen, daher ist die Begrenzung des Elements schwer wahrnehmbar; native Bedienelemente werden nicht gemessen.

rule-contrast-control-undetermined = Prüfen, ob der Kontrast nicht messbarer UI-Elemente ausreicht
    .user-impact = Interaktive Elemente schwer erkennbar.
    .technical-impact = Der Hintergrund hinter einem selbst gezeichneten Kontrollkästchen, Optionsfeld, Schalter oder Range-Eingabefeld ließ sich aus dem berechneten Stil des Elternelements nicht auflösen, daher bleibt das Verhältnis von 3:1 für seine Begrenzung ungemessen und braucht eine manuelle Prüfung.

rule-text-spacing-clipped = Inhalt bei erhöhtem Zeilen-/Wortabstand abgeschnitten
    .user-impact = Nutzer, die Zeilen-/Wortabstand für bessere Lesbarkeit anpassen, verlieren Inhalte.
    .technical-impact = Fixe Höhe/Breite mit overflow:hidden auf Textcontainern; gemessen vor und nach dem Überschreiben des Textabstands, nur neu abgeschnittener Text zählt.

rule-text-spacing-letter-important = Zeichenabstand mit !important festgesetzt
    .user-impact = Nutzer können den Zeichenabstand zum Lesen nicht vergrößern.
    .technical-impact = An einem Element mit Text setzt das style-Attribut letter-spacing mit !important auf weniger als das 0,12-Fache der Schriftgröße, daher kann ein Nutzer-Stylesheet oder Bookmarklet ihn für die Lesbarkeit nicht vergrößern; ein Prüfhinweis, wenn das Element außerhalb des Flusses steht oder der Wert ohne Layout nicht bestimmbar ist.

rule-text-spacing-word-important = Wortabstand mit !important festgesetzt
    .user-impact = Nutzer können den Wortabstand zum Lesen nicht vergrößern.
    .technical-impact = An einem Element mit Text setzt das style-Attribut word-spacing mit !important auf weniger als das 0,16-Fache der Schriftgröße, daher kann ein Nutzer-Stylesheet oder Bookmarklet ihn für die Lesbarkeit nicht vergrößern; ein Prüfhinweis, wenn das Element außerhalb des Flusses steht oder der Wert ohne Layout nicht bestimmbar ist.

rule-text-spacing-line-height-important = Zeilenhöhe mit !important festgesetzt
    .user-impact = Nutzer können die Zeilenhöhe zum Lesen nicht vergrößern.
    .technical-impact = An einem Element mit Text setzt das style-Attribut line-height mit !important auf weniger als das 1,5-Fache der Schriftgröße, daher kann ein Nutzer-Stylesheet oder Bookmarklet sie für die Lesbarkeit nicht vergrößern; ein Prüfhinweis, wenn das Element außerhalb des Flusses steht, in einem waagerecht scrollenden Container liegt oder der Wert ohne Layout nicht bestimmbar ist.

rule-color-link-indistinct = Link nur durch Farbe unterscheidbar
    .user-impact = Nutzer mit Farbsehschwäche können Links im Text nicht erkennen.
    .technical-impact = Ein Link im Fließtext hat keine Unterstreichung und dasselbe Gewicht, denselben Schnitt, dieselbe Schrift, Unterkante und denselben Hintergrund wie der umgebende Text, daher unterscheidet er sich nur durch die Farbe; ein Verstoß, wenn der Farbkontrast zum umgebenden Text unter 3:1 liegt, sonst ein Prüfhinweis, ob Hover und Fokus einen Hinweis außer der Farbe ergänzen.

rule-motion-reduced-motion-ignored = Animationen ohne prefers-reduced-motion
    .user-impact = Nutzer mit vestibulären Störungen oder Photosensitivität können Bewegung nicht deaktivieren.
    .technical-impact = Stilregeln lösen Bewegung aus (eine Animation mit bewegten @keyframes oder einen Übergang einer Bewegungseigenschaft wie transform oder position) an Elementen, die es auf der Seite gibt, und werden unter @media (prefers-reduced-motion: reduce) nicht abgeschaltet, daher können Menschen mit Vestibularstörungen die Bewegung nicht vermeiden; als Prüfhinweis gemeldet.

rule-patterns-tooltip-unreferenced = Tooltip nicht vom auslösenden Element referenziert
    .user-impact = Screenreader-Nutzer erfahren den Tooltip-Text nie, weil nichts darauf verweist.
    .technical-impact = Ein Element mit role="tooltip" wird von keinem aria-describedby oder aria-labelledby referenziert, daher sagt die Hilfstechnik seinen Text für das auslösende Element nie an; als Best Practice gemeldet.

rule-media-audio-autoplay = Selbststartender Ton ohne Steuerung prüfen
    .user-impact = Ton, der von selbst startet, übertönt die Sprachausgabe von Screenreadern.
    .technical-impact = Ein <audio> oder <video> hat autoplay, aber weder muted noch sichtbare Bedienelemente und kein Medienfragment von höchstens drei Sekunden, daher kann es beim Laden Ton über die Sprachausgabe eines Screenreaders legen; als Prüfhinweis gemeldet, weil das Markup nicht zeigt, wie lange der Ton läuft.
    .description = Ein Audio- oder Video-Element startet beim Laden der Seite von selbst und ist weder stummgeschaltet noch bietet es sichtbare Bedienelemente.
    .explanation-user-impact = Ton, der von selbst startet, übertönt die Sprachausgabe von Screenreadern; Nutzer finden die Steuerung dann kaum noch.
    .typical-cause = <audio autoplay> oder <video autoplay> für Hintergrundmusik, ein Hintergrundvideo oder Werbung.
    .recommendation = Ton nicht automatisch starten oder eine Steuerung zum Anhalten bzw. Leiserstellen am Anfang der Seite anbieten.
    .technical-note = Prüfhinweis (WCAG 1.4.2): Ob der Ton länger als drei Sekunden läuft und sich anhalten lässt, steht nicht im Markup.

rule-media-audio-autoplay-measured = Selbststartender Ton spielt länger als drei Sekunden ohne Steuerung
    .user-impact = Ton, der von selbst startet, übertönt die Sprachausgabe von Screenreadern, und am Element hält ihn nichts an.
    .technical-impact = Im Browser gemessen: Ein audio- oder video-Element mit autoplay spielt, ist nicht stummgeschaltet, hat eine Tonspur, dauert länger als drei Sekunden und hat keine nutzbaren controls.
    .description = Ein Audio- oder Video-Element startet beim Laden der Seite von selbst und spielt länger als drei Sekunden Ton, ohne Bedienelement zum Anhalten oder Leiserstellen.
    .explanation-user-impact = Screenreader-Nutzer hören den Ton der Seite über ihrer Sprachausgabe und finden die Steuerung zum Anhalten kaum.
    .typical-cause = <video autoplay> oder <audio autoplay> für ein Hintergrundvideo, Musik oder Werbung, ohne controls und ohne muted.
    .recommendation = Ton nicht automatisch starten: das Medium stummgeschaltet starten, den Ton auf höchstens drei Sekunden begrenzen oder Bedienelemente zum Anhalten und Leiserstellen anbieten.
    .technical-note = Gemessen in Chrome nach dem Laden, mit erlaubtem Autoplay mit Ton (der ungünstigste Fall, den Besucher erleben können): paused, muted, wie lange es spielt (ein Medienfragment kürzt, loop oder ein Datenstrom endet nie) und ob es eine Tonspur hat. Unbekannte Tonspur oder Dauer ergibt eine Prüfung; ohne Messung ist die Regel nicht gelaufen (ACT 80f0bf). Bedienelemente, die ein Skript der Seite baut, sieht die Messung nicht.

rule-text-justified = Prüfen, ob Text nicht im Blocksatz gesetzt ist (AAA)
    .user-impact = Nutzer mit Sehbehinderungen oder Legasthenie haben Probleme beim Lesen.
    .technical-impact = Ein Textblock nutzt text-align: justify, was ungleichmäßige Wortabstände und Lücken im Satzbild erzeugt, die Leser mit Legasthenie oder Sehbeeinträchtigung nicht anpassen können, da der Stil im CSS fest vorgegeben ist.

rule-text-line-height-tight = Prüfen, ob Absätze eine Zeilenhöhe von mindestens 1,5 haben (AAA)
    .user-impact = Nutzer mit Sehbehinderungen oder Legasthenie haben Probleme beim Lesen.
    .technical-impact = Ein Stylesheet gibt den <p>-Absätzen der Seite eine line-height unter 1,5 (als Zahl, em- oder Prozentwert), daher stehen die Zeilen dicht gedrängt und Leser, die Text vergrößern oder Legasthenie haben, verlieren die Zeile; der Wert stammt aus den Stilregeln, nicht aus gerenderten Stilen, und wird als Prüfhinweis gemeldet.

## Technical robustness

rule-keyboard-hidden-focusable = Fokussierbares Element versteckt vor Screenreadern
    .user-impact = Tastaturnutzer erreichen Elemente, die Screenreader nicht ankündigen — unsichtbare Stolperfallen.
    .technical-impact = Ein per Tab erreichbares Element (nativ fokussierbar oder mit tabindex ab 0) steht in einem mit aria-hidden="true" markierten Teilbaum und ist nicht inert, daher landet der Tastaturfokus darauf, während ein Screenreader nichts ansagt; hinter einem offenen modalen Dialog ist das nur ein Prüfhinweis.
    .description = Ein interaktives Element ist über aria-hidden="true" vor Screenreadern versteckt, kann aber dennoch mit der Tastatur (Tabulator) fokussiert werden.
    .explanation-user-impact = Tastaturnutzer steuern auf unsichtbare Elemente (Fokus-Stolperfallen), während Screenreader-Nutzer keinerlei Rückmeldung über dieses Element erhalten.
    .typical-cause = Es wurde aria-hidden="true" verwendet, um ein Element visuell auszublenden, ohne tabindex="-1" zu setzen oder das Element per display:none zu verstecken.
    .recommendation = Wenn ein Element vollständig versteckt sein soll, display:none oder visibility:hidden verwenden. Soll es nur vor Screenreadern versteckt sein, muss es unfokussierbar sein (tabindex="-1").
    .technical-note = Fokussierbare Elemente (a, button, input) in einem aria-hidden="true"-Teilbaum müssen entweder aus der Tab-Reihenfolge genommen oder vollständig versteckt werden.

rule-status-live-overridden = Statusmeldungen nicht programmatisch bestimmbar
    .user-impact = Screenreader kündigt Statusmeldungen nicht korrekt an.
    .technical-impact = Ein Element mit der Rolle alert, status oder log hat einen aria-live-Wert, der die Vorgabe seiner Rolle überschreibt: aria-live="off" schaltet die Ansagen ab (ein Verstoß), eine andere Dringlichkeit (polite oder assertive statt der Vorgabe) ändert nur den Zeitpunkt und ist ein Prüfhinweis.

rule-aria-required-attribute-missing = Pflicht-ARIA-Attribut fehlt
    .user-impact = Assistive Technologien können Zustand oder Wert des Elements nicht verlässlich vermitteln.
    .technical-impact = Einem Element mit einer Rolle, die Zustands- oder Wertattribute braucht, etwa einer checkbox ohne aria-checked oder einem slider ohne aria-valuenow, fehlen sie, daher kann die Hilfstechnik Zustand oder Wert nicht ausgeben; native Checkbox-, Radio- und Range-Eingaben, meter und progress sind ausgenommen.
    .description = Ein Element trägt eine Rolle, die einen Zustand ansagt (etwa checkbox oder slider), aber nicht das Attribut, das diesen Zustand trägt.
    .explanation-user-impact = Screenreader-Nutzer erfahren nicht, ob ein Schalter an ist oder welchen Wert ein Regler hat.
    .typical-cause = Selbst gebaute Bedienelemente setzen die Rolle, pflegen den Zustand aber nur visuell.
    .recommendation = Das geforderte Attribut setzen und bei jeder Zustandsänderung aktualisieren, oder ein natives Element verwenden.
    .technical-note = Etwa aria-checked für checkbox/radio/switch, aria-expanded für combobox, aria-valuenow für slider/spinbutton/meter.

rule-aria-required-parent-missing = Element außerhalb des erforderlichen Eltern-Kontexts
    .user-impact = Assistive Technologien können die Beziehung des Elements zu seiner Gruppe (z. B. Tab zu Tableiste) nicht vermitteln.
    .technical-impact = Ein Element, dessen Rolle in einer bestimmten Elternrolle stehen muss, etwa tab in tablist, option in listbox oder listitem in list, hat keinen solchen Vorfahren oder Besitzer, daher kann die Hilfstechnik nicht vermitteln, zu welcher Gruppe es gehört.
    .description = Ein Element mit einer Rolle wie tab, option oder listitem steht nicht in einem Element mit der dafür erforderlichen Rolle.
    .explanation-user-impact = Screenreader können das Element keinem Widget zuordnen; Position und Anzahl („Tab 2 von 3“) werden nicht angesagt.
    .typical-cause = Ein Element mit eigener Rolle dazwischen, etwa ein <li> zwischen role="tablist" und role="tab".
    .recommendation = Das Element direkt in den erforderlichen Behälter setzen oder die Zwischenelemente mit role="presentation" neutralisieren.
    .technical-note = Geprüft wird am DOM. Ein <li> in einer Liste, die per role etwas anderes geworden ist, bleibt ein listitem, auch wenn der Browser es im Accessibility-Tree glättet.

rule-inert-dialog-inert = Ungültige moderne Interaktionsattribute
    .user-impact = Screenreader- und Tastaturnutzer erreichen Dialoge, Popover oder gesperrte Bereiche nicht zuverlässig.
    .technical-impact = Ein offener <dialog> (ein Verstoß) oder ein Element mit role="dialog" oder "alertdialog" (ein Prüfhinweis, da CSS es geschlossen haben kann) ist selbst inert oder liegt in einem inerten Vorfahren, daher lässt es sich weder fokussieren noch bedienen und die Hilfstechnik erhält es nicht, obwohl es sichtbar ist.

rule-modern-attribute-misuse = Offenes Popover oder Menü ohne Namen oder als inert markiert
    .user-impact = Screenreader- und Tastaturnutzer erreichen Dialoge, Popover oder gesperrte Bereiche nicht zuverlässig.
    .technical-impact = Ein offenes Popover ([popover] im geöffneten Zustand) oder ein sichtbares role="menu" hat keinen zugänglichen Namen (kein aria-label, aria-labelledby oder title), oder ein offenes Popover trägt das inert-Attribut und lässt sich daher sichtbar weder fokussieren noch bedienen; eine unbenannte Oberfläche ist nur ein Prüfhinweis, weil weder HTML noch ARIA dort einen Namen verlangen.
    .description = Ein offenes Popover oder ein sichtbares Menü hat keinen zugänglichen Namen, oder ein offenes Popover ist als inert markiert.
    .explanation-user-impact = Tastatur- und Screenreader-Nutzer können Oberflächen öffnen, fokussieren oder verlassen, ohne eine verständliche Rückmeldung zu erhalten.
    .typical-cause = Ein Popover oder Menü aus einem schlichten <div> ohne aria-label oder aria-labelledby, oder inert an einem Popover, das gleichzeitig angezeigt wird.
    .recommendation = Offene Popover und Menüs mit aria-label oder aria-labelledby benennen und inert nur an tatsächlich inaktiven Bereichen verwenden, nie an einem angezeigten Popover.
    .technical-note = Geprüft in der gerenderten Seite, auch in offenen Shadow Roots und Frames gleichen Ursprungs. Ein unbenanntes offenes Popover oder Menü ist ein Prüfhinweis, ein offenes Popover mit inert ein Verstoß. Ungültige popovertarget-Werte, inerte Dialoge und unbenannte Dialoge melden eigene Regeln.

rule-popover-target-invalid = Ungültige moderne Interaktionsattribute
    .user-impact = Screenreader- und Tastaturnutzer erreichen Dialoge, Popover oder gesperrte Bereiche nicht zuverlässig.
    .technical-impact = Ein popovertarget-Attribut verweist auf ein vorhandenes Element ohne popover-Attribut, daher bewirkt der auslösende Button nichts und das Ziel wird nicht als Popover bereitgestellt.

rule-popover-target-missing = Ungültige moderne Interaktionsattribute
    .user-impact = Screenreader- und Tastaturnutzer erreichen Dialoge, Popover oder gesperrte Bereiche nicht zuverlässig.
    .technical-impact = Ein popovertarget-Attribut verweist auf eine id, die im Dokument nicht existiert, daher ist der auslösende Button an nichts gebunden und seine Aktivierung bleibt für alle Nutzer wirkungslos.

rule-aria-attribute-prohibited = Verbotene ARIA-Attribute
    .user-impact = Screenreader erhalten widersprüchliche oder ungültige Semantik.
    .technical-impact = Ein Element trägt ein ARIA-Attribut, das WAI-ARIA 1.2 für seine implizite oder explizite Rolle verbietet, etwa aria-label an role="presentation"; leere Werte werden nicht gemeldet.
    .description = Für bestimmte HTML-Elemente oder ARIA-Rollen werden Attribute verwendet, die für diese Rolle nicht zulässig oder sinnvoll sind.
    .explanation-user-impact = Screenreader erhalten widersprüchliche oder fehlerhafte Signale, was die Bedienbarkeit der Website beeinträchtigen kann.
    .typical-cause = Verwendung von ARIA-Attributen (wie aria-label oder aria-expanded) auf Elementen, die keine interaktive Rolle haben (z. B. span oder div ohne role).
    .recommendation = Prüfe die ARIA-Spezifikation für die jeweilige Rolle. Verwende ARIA-Attribute nur auf passenden Elementen und Rollen.
    .technical-note = Prohibited attributes on generic elements: <span aria-label="text"> is invalid. Use native elements or valid roles (e.g. role="button").

rule-aria-attribute-not-allowed = ARIA-Attribut für diese Rolle nicht zulässig
    .user-impact = Assistive Technologien ignorieren das Attribut oder interpretieren es falsch.
    .technical-impact = Ein ARIA-Attribut wird von der Rolle des Elements nicht unterstützt (etwa aria-checked an role="img" oder aria-expanded an einem <div>), doppelt ein natives Attribut (aria-disabled neben disabled) oder ist an <audio> oder <video> kein globales Attribut, daher ignoriert die Hilfstechnik es oder liest einen falschen Zustand.
    .description = Ein Element trägt ein ARIA-Attribut, das seine Rolle nicht unterstützt, oder eines, das neben einem nativen Attribut nicht stehen darf.
    .explanation-user-impact = Screenreader ignorieren das Attribut oder erhalten widersprüchliche Angaben zu Zustand und Wert.
    .typical-cause = Zustände wie aria-expanded oder aria-checked an Elementen ohne passende Rolle, etwa an einem div.
    .recommendation = Das Attribut an das Element mit der passenden Rolle verschieben, eine passende Rolle setzen oder das Attribut entfernen.
    .technical-note = Geurteilt wird über die explizite oder implizite Rolle. Dazu die Fälle aus ARIA in HTML, etwa aria-checked an einer nativen Checkbox.

rule-aria-attribute-value-invalid = Ungültiger Wert eines ARIA-Attributs
    .user-impact = Assistive Technologien können Zustand oder Beziehung des Elements nicht korrekt vermitteln.
    .technical-impact = Ein aria-*-Attribut hat einen Wert, den sein Typ nicht zulässt (etwa aria-expanded="yes", ein nicht ganzzahliges aria-level oder zwei IDs, wo eine erwartet wird), oder eine Combobox oder ein Scrollbar hat ein erforderliches aria-controls, das auf kein vorhandenes Element zeigt, daher fallen Browser auf den Standard zurück und ein Screenreader sagt den Zustand falsch oder gar nicht an; leere Werte werden nicht gemeldet.
    .description = Ein ARIA-Attribut hat einen Wert außerhalb seines Wertebereichs, etwa aria-expanded="yes".
    .explanation-user-impact = Browser fallen auf den Vorgabewert zurück; der Zustand wird Screenreader-Nutzern falsch oder gar nicht angesagt.
    .typical-cause = Werte aus anderen Konventionen (yes/no, 1/0) oder Tippfehler in Token-Werten.
    .recommendation = Nur die Werte verwenden, die die ARIA-Spezifikation für das Attribut vorsieht, etwa true oder false.
    .technical-note = Unbekannte Werte von aria-current und aria-invalid gelten nach ARIA 1.2 als true und werden nicht gemeldet.

rule-aria-reference-missing = ARIA-Verweis ohne Ziel
    .user-impact = Assistive Technologien können die beabsichtigte Beziehung zwischen den Elementen nicht auflösen.
    .technical-impact = Ein ARIA-Beziehungsattribut (aria-labelledby, aria-describedby, aria-controls, aria-owns, aria-activedescendant, aria-details, aria-flowto oder aria-errormessage an einem als ungültig markierten Feld) nennt eine id, die es nicht gibt, oder aria-controls, aria-owns oder aria-activedescendant ist leer, daher kann ein Screenreader Beschriftung, Beschreibung oder gesteuerten Bereich nicht auflösen.
    .description = Ein ARIA-Beziehungsattribut (etwa aria-controls oder aria-labelledby) ist leer oder verweist auf eine ID, die es auf der Seite nicht gibt.
    .explanation-user-impact = Screenreader können die Beziehung nicht herstellen: Beschriftung, Beschreibung oder gesteuerter Bereich fehlen.
    .typical-cause = Die ID des Ziels wurde umbenannt oder wird erst später per Skript erzeugt; Komponenten setzen das Attribut, bevor das Ziel existiert.
    .recommendation = Das Attribut auf die ID eines vorhandenen Elements setzen oder es entfernen.
    .technical-note = Geprüft werden aria-labelledby, aria-describedby, aria-controls, aria-owns, aria-activedescendant, aria-details, aria-flowto und, an einem ungültigen Feld, aria-errormessage. Ein leeres aria-labelledby schadet nicht und wird nicht gemeldet; ein eingeklapptes Bedienelement (aria-expanded="false") darf auf ein Ziel zeigen, das erst beim Öffnen entsteht.

rule-aria-role-abstract = Ungültige ARIA-Rolle
    .user-impact = Assistive Technologien können die Funktion des Elements nicht bestimmen.
    .technical-impact = Das role-Attribut enthält eine abstrakte Rolle wie widget oder landmark, die nur in der ARIA-Taxonomie existiert und im Markup nicht verwendet werden darf, daher ignorieren Browser sie und geben die native Rolle weiter.

rule-aria-role-invalid = Ungültige ARIA-Rolle
    .user-impact = Assistive Technologien können die Funktion des Elements nicht bestimmen.
    .technical-impact = Das role-Attribut enthält einen Wert, den es in WAI-ARIA nicht gibt, zum Beispiel einen Tippfehler, daher ignorieren Browser ihn und das Element behält seine native Rolle ohne die beabsichtigte Semantik.
    .description = Ein role-Attribut enthält einen Wert, den es in WAI-ARIA nicht gibt, oder eine abstrakte Rolle, die nicht ausgezeichnet werden darf.
    .explanation-user-impact = Browser ignorieren die Rolle; Screenreader sagen das Element anders an als beabsichtigt.
    .typical-cause = Tippfehler, erfundene Rollen oder abstrakte Rollen wie widget oder landmark.
    .recommendation = Eine Rolle aus der ARIA-Spezifikation verwenden oder das role-Attribut entfernen und ein natives Element nutzen.
    .technical-note = Rollen unterscheiden Groß- und Kleinschreibung: role="Button" ist ungültig.

rule-aria-attribute-unknown = Ungültiger ARIA-Attributname
    .user-impact = Assistive Technologien ignorieren das Attribut vollständig.
    .technical-impact = Ein Element trägt ein mit aria- beginnendes Attribut, das WAI-ARIA nicht definiert (etwa den Tippfehler aria-labeledby; die aria-*-Attribute des 1.3-Entwurfs, die Browser schon umsetzen, gelten als bekannt), daher ignorieren Browser es und der gemeinte Name, die Beziehung oder der Zustand erreicht die Hilfstechnik nie.
    .description = Ein Element trägt ein aria-*-Attribut, das es in WAI-ARIA nicht gibt – meist ein Tippfehler.
    .explanation-user-impact = Der Browser ignoriert das Attribut; die gemeinte Beschriftung, Beziehung oder der gemeinte Zustand fehlt.
    .typical-cause = Falsch geschriebene Attribute wie aria-labeledby statt aria-labelledby.
    .recommendation = Den Namen gegen die ARIA-Spezifikation prüfen und korrigieren oder das Attribut entfernen.
    .technical-note = Attribute aus dem Entwurf von WAI-ARIA 1.3, die Browser schon umsetzen (etwa aria-description), gelten als bekannt.

rule-document-lang-mismatch = lang und xml:lang widersprechen sich
    .user-impact = Assistive Technologien erhalten widersprüchliche Angaben zur Sprache des Inhalts.
    .technical-impact = Das <html>-Element trägt lang und xml:lang mit verschiedenen Primärsprachen (etwa lang="en" und xml:lang="de"), daher kann ein Screenreader eines von beiden wählen und die Seite in der falschen Sprache vorlesen.
    .description = Das <html>-Element trägt lang und xml:lang mit unterschiedlichen Sprachen.
    .explanation-user-impact = Screenreader wählen je nach Attribut eine andere Sprache und sprechen die Seite womöglich falsch aus.
    .typical-cause = Aus XHTML übernommene Vorlagen, bei denen nur eines der Attribute angepasst wurde.
    .recommendation = xml:lang entfernen oder auf denselben Wert wie lang setzen.
    .technical-note = Verglichen wird die Primärsprache beider Attribute.

rule-tables-headers-attr-invalid = Ungültiger Headers-Verweis in Tabellenzelle
    .user-impact = Screenreader können der Zelle keine oder eine falsche Spalten-/Zeilenüberschrift zuordnen.
    .technical-impact = Das headers-Attribut einer Zelle ist leer oder nennt eine id, die keine Kopfzelle (<th> oder eine Kopfrolle) derselben Tabelle ist, oder die Zelle selbst, daher sagt ein Screenreader für diese Zelle keine oder die falsche Überschrift an.
    .description = Eine Tabellenzelle verweist per headers auf eine ID, die keine Kopfzelle derselben Tabelle ist.
    .explanation-user-impact = Screenreader lesen zur Zelle den falschen oder gar keinen Kopf vor.
    .typical-cause = Umbenannte oder kopierte IDs, Verweise auf Zellen anderer Tabellen.
    .recommendation = headers nur auf IDs von <th> derselben Tabelle zeigen lassen oder bei einfachen Tabellen ganz auf scope setzen.
    .technical-note = Geprüft wird nur an Zellen (td, th und ihre Rollen); nur dort definiert HTML das Attribut.

rule-tables-data-undetermined = Tabellenkopfzellen auf Datenzellen prüfen
    .user-impact = Bleiben die Zeilen leer, kündigen Screenreader eine Kopfzelle an, die keine Daten gliedert.
    .technical-impact = Ein <th> ließ sich keinen Datenzellen zuordnen, weil die Tabellenzeilen zum Prüfzeitpunkt nicht gerendert waren (nachgeladene oder skriptgenerierte Inhalte), daher bleibt die Zuordnung von Überschrift und Daten ungeprüft.

rule-tables-header-without-data = Tabellenkopf ohne Datenzellen
    .user-impact = Screenreader kündigen eine Überschrift an, die keine Daten strukturiert.
    .technical-impact = Ein <th> hat in seiner Zeile oder Spalte keine Datenzelle, die es beschriften könnte, daher sagt ein Screenreader eine Überschrift an, die keine Daten gliedert.
    .description = Eine Kopfzelle (<th>) bezieht sich auf keine Datenzelle – die Tabelle hat Überschriften, aber keine Daten darunter oder daneben.
    .explanation-user-impact = Screenreader können Datenzellen keinem Kopf zuordnen; die Tabelle wirkt leer oder falsch aufgebaut.
    .typical-cause = Tabellen, die nur zur Anordnung dienen, oder Datenzeilen, die per Skript erst später erscheinen.
    .recommendation = Kopfzellen nur dort einsetzen, wo Daten folgen; Layout ohne <table> umsetzen.
    .technical-note = Leere Datenzellen, Zellen mit aria-hidden-Inhalt und Spaltenköpfe über Zeilenköpfen zählen nicht als Fehler. Ein noch nicht dargestellter Zeilenvorrat ergibt einen Hinweis (ungeprüft) statt eines Verstoßes.

rule-aria-tabpanel-missing = Tableiste ohne zugehöriges Tab-Panel
    .user-impact = Screenreader-Nutzer können nach Aktivieren eines Tabs den zugehörigen Inhalt nicht auffinden.
    .technical-impact = Es gibt ein Element mit role="tablist", aber im ganzen Dokument kein Element mit role="tabpanel", daher zeigt das Aktivieren eines Tabs keinen Inhalt, den die Hilfstechnik ihm zuordnen kann; als Prüfhinweis gemeldet.

rule-aria-tab-selected-missing = Kein Tab als ausgewählt ausgezeichnet
    .user-impact = Screenreader-Nutzer erfahren nicht, welcher Tab aktiv ist und zu welchem der sichtbare Inhalt gehört.
    .technical-impact = Eine role="tablist" mit Tabs hat keinen Tab mit aria-selected="true" (eine tablist mit aria-multiselectable ist ausgenommen), daher sagt ein Screenreader nicht an, welcher Tab aktiv ist; als Prüfhinweis gemeldet, weil ARIA es nur empfiehlt.
    .description = Keiner der Tabs einer Tableiste trägt aria-selected="true".
    .explanation-user-impact = Screenreader-Nutzer erfahren nicht, welcher Tab aktiv ist und zu welchem der sichtbare Inhalt gehört.
    .typical-cause = Der aktive Tab wird nur per CSS-Klasse hervorgehoben.
    .recommendation = Am aktiven Tab aria-selected="true" setzen und beim Wechsel mitführen.
    .technical-note = Zur Prüfung: ARIA 1.2 sagt SHOULD. Tabs ohne das Attribut gelten als nicht ausgewählt, das ist für die inaktiven richtig.

rule-patterns-accordion-controls-missing = Aufgeklappter Button ohne Verweis auf seinen Bereich
    .user-impact = Screenreader-Nutzer erfahren nicht, welcher Bereich aufgeklappt wurde.
    .technical-impact = aria-expanded="true" ohne aria-controls; WAI-ARIA verlangt es nicht, die APG nennt es beim Disclosure-Muster optional.
    .description = Ein aufgeklappter Button (aria-expanded="true") verweist nicht per aria-controls auf den Bereich, den er steuert.
    .explanation-user-impact = Screenreader-Nutzer erfahren nicht, welcher Bereich aufgeklappt wurde, und können nicht direkt dorthin springen.
    .typical-cause = Akkordeon- oder Aufklapp-Skripte, die nur aria-expanded umschalten.
    .recommendation = Am Button aria-controls auf die ID des gesteuerten Bereichs setzen.
    .technical-note = Prüfhinweis: WAI-ARIA verlangt aria-controls nicht, die APG nennt es beim Disclosure-Muster optional. Ein <summary> braucht es nicht.

rule-dialog-modal-unmarked = Dialog nicht als modal ausgezeichnet
    .user-impact = Ist der Dialog modal gemeint, lesen Screenreader den Inhalt dahinter weiter, als wäre er erreichbar.
    .technical-impact = role="dialog" ohne aria-modal="true"; ob der Dialog modal gemeint ist, zeigt erst sein Verhalten.
    .description = Ein Element mit role="dialog" trägt kein aria-modal="true". Ob der Dialog modal gemeint ist, muss geprüft werden.
    .explanation-user-impact = Ist der Dialog modal gemeint, lesen Screenreader den Inhalt dahinter weiter, als wäre er erreichbar.
    .typical-cause = Ein selbst gebauter Dialog aus einem div mit role="dialog", ohne aria-modal.
    .recommendation = Einen modalen Dialog mit aria-modal="true" auszeichnen oder ein natives <dialog> mit showModal() verwenden. Ein nicht modaler Dialog braucht das Attribut nicht.
    .technical-note = ARIA verlangt aria-modal nicht; der Hinweis ist eine Prüfempfehlung, kein Verstoß.

rule-aria-combobox-popup-missing = Aufgeklappte Combobox ohne Verweis auf ihr Popup
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Auswahlmöglichkeiten die Combobox bietet.
    .technical-impact = Ein Element mit role="combobox" und aria-expanded="true" hat weder aria-controls noch aria-owns und enthält keine listbox, tree, grid oder dialog, daher findet die Hilfstechnik das geöffnete Popup nicht.
    .description = Eine Combobox ist als aufgeklappt ausgezeichnet, verweist aber nicht per aria-controls auf ihre Auswahlliste.
    .explanation-user-impact = Screenreader-Nutzer erfahren nicht, welche Auswahlmöglichkeiten die Combobox bietet, und erreichen sie nicht.
    .typical-cause = Die Liste steht neben der Combobox statt darin, und aria-controls fehlt.
    .recommendation = aria-controls an der Combobox auf die ID der Liste setzen.
    .technical-note = ARIA 1.2 verlangt aria-controls (MUST). Ein Popup im eigenen Teilbaum oder per aria-owns (ARIA 1.1) wird weiter anerkannt.

rule-frames-name-missing = Iframe ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erkennen Zweck und Inhalt des eingebetteten Frames nicht.
    .technical-impact = Ein <iframe>, den die Tabtaste erreicht und der sichtbar ist (kein negativer tabindex, nicht 0 oder 1 Pixel breit und hoch, nicht display: none), hat keinen zugänglichen Namen aus title, aria-label oder aria-labelledby, daher sagt ein Screenreader beim Betreten des Rahmens keinen Namen an und sein Inhalt bleibt unbestimmt; <frame> wird nicht geprüft.
    .description = Ein <iframe> hat keinen zugänglichen Namen (title, aria-label oder aria-labelledby).
    .explanation-user-impact = Screenreader nennen beim Betreten des Rahmens nur „Rahmen“; Nutzer erfahren nicht, was darin steckt, und können ihn nicht gezielt überspringen.
    .typical-cause = Eingebettete Karten, Videos, Widgets oder Werbung ohne title-Attribut.
    .recommendation = Jedem sichtbaren <iframe> ein title geben, das seinen Inhalt beschreibt.
    .technical-note = WCAG-Technik H64. Rahmen mit role="none"/"presentation", hidden, aria-hidden oder Breite und Höhe 0/1 zählen nicht.

rule-frames-cross-origin = Cross-origin Iframe: manuell prüfen
    .user-impact = Eingebettete Inhalte (Karten, Videos, Consent-Widgets) sind möglicherweise für Screenreader-Nutzer unzugänglich.
    .technical-impact = Das Dokument eines sichtbaren <iframe> war für das Werkzeug nicht lesbar (fremder Ursprung oder nicht abgeschlossene Navigation), daher läuft keine Regel darin und sein Inhalt bleibt ungeprüft; eine Abdeckungsnotiz, kein Urteil über den Rahmen.
    .description = Das Dokument in einem iframe war für das Prüfwerkzeug nicht lesbar und blieb deshalb ungeprüft. Der Rahmen hat einen fremden Ursprung, oder seine Navigation war nicht abgeschlossen.
    .explanation-user-impact = Eingebettete Inhalte wie Karten, Buchungs- oder Einwilligungsdialoge können für Screenreader- und Tastaturnutzer unzugänglich sein, ohne dass der Bericht es zeigt.
    .typical-cause = Inhalte von Drittanbietern in einem iframe mit fremdem Ursprung; die Seite kann deren Dokument nicht lesen.
    .recommendation = Den Rahmeninhalt von Hand prüfen oder seine Adresse für sich prüfen; beim Anbieter nach einer barrierefreien Einbettung fragen.
    .technical-note = Eine Abdeckungsnotiz, keine Konformitätsprüfung und an kein Erfolgskriterium gebunden. Gemessen wird, ob contentDocument des Rahmens lesbar ist; Rahmen, die präsentational, versteckt oder höchstens 1 px groß sind, bleiben außen vor. Ohne die Messung steht die Regel als nicht gelaufen im Bericht.

rule-ids-duplicate = Mehrfach vergebene ID wird referenziert
    .user-impact = Assistierende Technik kann die Beziehung nicht eindeutig auflösen und gibt Name oder Beschreibung des falschen Elements aus.
    .technical-impact = Dieselbe id steht an mehr als einem Element und ein IDREF-Attribut (for, form, list, headers oder ein aria-*-Verweis wie aria-labelledby) zeigt darauf; der Browser löst den Verweis auf das erste Element auf, daher kann ein Screenreader Name, Beschreibung oder Beziehung vom falschen Element übernehmen; je id einmal als Prüfhinweis gemeldet.
    .description = Mehrere Elemente auf der Seite tragen dieselbe id, und ein anderes Element verweist darauf (z. B. label for, aria-labelledby). Welches der Elemente gemeint ist, ist damit nicht mehr bestimmbar.
    .explanation-user-impact = Screenreader lösen die Beziehung auf das falsche Element auf und geben einen falschen Namen oder eine falsche Beschreibung aus.
    .typical-cause = Wiederholt eingebundene Komponenten oder Templates (z. B. Menüpunkte, Widgets), die eine statische ID statt einer pro Instanz eindeutigen ID verwenden.
    .recommendation = Jede ID nur einmal im Dokument vergeben. Bei wiederverwendeten Komponenten die ID dynamisch generieren (z. B. mit Index oder Slug).
    .technical-note = IDs eindeutig machen, etwa durch Anhängen eines Index oder einer eindeutigen Kennung pro Komponenteninstanz.

rule-images-server-side-map = Server-seitige Image-Map verwendet
    .user-impact = Tastatur- und Screenreader-Nutzer können die einzelnen Bildbereiche nicht ansteuern.
    .technical-impact = Ein <img ismap> in einem Link mit href ist eine serverseitige Image-Map: Die Klickposition, nicht ein Linkziel, wählt das Ziel, daher können Tastatur- und Screenreader-Nutzer keinen Bereich auswählen; als Prüfhinweis gemeldet, weil gleichwertige Textlinks neben dem Bild das Kriterium erfüllen.

rule-images-role-img-name-missing = Fehlender Alternativtext bei Elementen mit Bildrolle
    .user-impact = Screenreader-Nutzer erhalten keine Information über das Bild.
    .technical-impact = Ein Element, das erst durch role="img" zum Bild wird (ein <div>, <span> oder <canvas>, oft ein Icon oder ein CSS-Hintergrundbild), hat keinen zugänglichen Namen aus aria-label, aria-labelledby oder title, daher sagt ein Screenreader ein unbenanntes Bild an; ein leeres aria-label ist kein Name, und ein Icon in einem benannten Link oder Button ist keine Ausnahme.

rule-names-indicator-name-missing = Messanzeige oder Fortschrittsbalken ohne Namen
    .user-impact = Screenreader-Nutzer hören einen Wert, erfahren aber nicht, was er misst.
    .technical-impact = Ein <meter>, <progress> oder ein Element mit role="meter" oder role="progressbar" hat keinen zugänglichen Namen aus <label>, aria-label, aria-labelledby oder title, daher sagt ein Screenreader nur Rolle und Wert an; eine Anzeige ist Nicht-Text-Inhalt, und ihr Name ist die Textalternative, die sagt, was gemessen wird.

rule-svg-name-missing = Fehlender Alternativtext bei SVG-Grafiken
    .user-impact = Screenreader-Nutzer erhalten keine Information über den Inhalt der Grafik.
    .technical-impact = Ein Inline-<svg> (oder ein Element mit Grafikrolle darin) hat keinen zugänglichen Namen (kein <title>-Kind, aria-label oder aria-labelledby) und ist nicht als dekorativ ausgezeichnet (aria-hidden oder role="none"), daher überspringt ein Screenreader es oder sagt eine unbenannte Grafik an; ein Verstoß, wenn das SVG role="img" oder eine andere Grafikrolle trägt, sonst ein Prüfhinweis, weil es Schmuck neben Text sein kann.

rule-images-area-alt-missing = Fehlender Alternativtext bei Image-Map-Bereichen
    .user-impact = Screenreader-Nutzer erfahren nicht, wohin ein anklickbarer Bildbereich führt.
    .technical-impact = Ein <area>-Element in einer Image-Map hat kein alt-Attribut, daher ist sein zugänglicher Name leer und ein Screenreader kann nicht ansagen, was der klickbare Bereich bewirkt.

rule-images-input-alt-missing = Fehlender Alternativtext bei Bild-Buttons
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Aktion der Button auslöst.
    .technical-impact = Ein <input type="image"> hat weder alt noch title und kein aria-label oder aria-labelledby, daher greift der Browser auf einen Standardnamen wie „Submit Query“ zurück, der nichts über die Aktion der Bildschaltfläche sagt.

rule-objects-alt-missing = Fehlende Textalternative bei eingebetteten Objekten
    .user-impact = Screenreader-Nutzer erhalten keine Information über den Inhalt des eingebetteten Objekts.
    .technical-impact = Ein <object> hat keine Textalternative: keinen Ersatztext zwischen den Tags, keinen title und kein aria-label oder aria-labelledby, und es ist nicht als präsentational ausgezeichnet, daher sagt ein Screenreader ein unbenanntes eingebettetes Objekt an und sein Inhalt bleibt unzugänglich.

rule-html-content-model = Verletzung des HTML5-Content-Models
    .user-impact = Screenreader können die Struktur nicht zuverlässig auswerten, auch wenn sie augenscheinlich korrekt wirkt.
    .technical-impact = Die HTML5-Schemaprüfung des Seitenquelltexts meldet einen Fehler innerhalb eines <dl>, <table> (oder seiner Zeilengruppen), <select>, <optgroup>, <fieldset>, <ul> oder <ol>, etwa ein Kindelement, das das Inhaltsmodell nicht erlaubt; Browser reparieren solches Markup beim Aufbau des Accessibility-Trees stillschweigend, daher kann das, was ein Screenreader erhält, vom Quelltext abweichen.
    .description = Ein strukturelles Element (Liste, Tabelle, Formular, Select) enthält Inhalt, der sein HTML5-Content-Model verletzt — das kann Screenreader-Semantik brechen, selbst wenn der resultierende Accessibility-Tree oberflächlich gültig aussieht.
    .explanation-user-impact = Screenreader können die Struktur nicht zuverlässig auswerten, auch wenn sie augenscheinlich korrekt wirkt.
    .typical-cause = Ein <div> oder ein anderes Element steht dort, wo das Inhaltsmodell es nicht erlaubt, etwa direkt in einem <ul> oder <select>, oder Text steht direkt in einem <tr>.
    .recommendation = Das Markup an das HTML5-Content-Model des Elements anpassen (z. B. darf ein <dl> nur direkt <dt>/<dd>-Gruppen, <div>, <script> oder <template> enthalten). Der Browser kann ungültige Kindelemente beim Aufbau des Accessibility-Trees still 'heilen' — der Defekt bleibt für rollenbasierte Prüfungen unsichtbar, verwirrt assistierende Technologie aber trotzdem.
    .technical-note = Gefunden durch die HTML5-Schemaprüfung des Seitenquelltexts (Attributfehler ausgenommen) und nur behalten, wenn der Fehler in einem der oben genannten Strukturelemente liegt. Das umgebende Element wird aus der Quelltextposition rekonstruiert, die Zuordnung ist daher eine Näherung.

rule-forms-group-missing = Prüfen, ob verwandte Formularfelder gruppiert sind
    .user-impact = Screenreader-Nutzer erkennen nicht, dass mehrere Felder zu einer Frage/Gruppe gehören.
    .technical-impact = Optionsfelder oder Kontrollkästchen, die im Formular denselben name tragen, haben keinen übergeordneten <fieldset>, <details> oder role="group"/"radiogroup", daher sagt ein Screenreader jede Option ohne die Frage an, die die ganze Auswahl beantwortet; als Prüfhinweis gemeldet, weil klare Beschriftungen die Gruppierung überflüssig machen können.

rule-landmarks-not-top-level = Landmark ist verschachtelt
    .user-impact = Screenreader-Nutzer, die per Landmark-Navigation springen, landen in einem unerwarteten Kontext.
    .technical-impact = Ein <main> oder ein Element mit role="banner", "contentinfo" oder "main" liegt innerhalb eines anderen Landmarks wie main, navigation, complementary oder einem benannten Bereich, daher ist es kein Landmark der obersten Ebene und die Landmark-Liste eines Screenreaders zeigt es an unerwarteter Stelle.
    .description = Ein Kopf-, Fuß- oder Hauptinhaltsbereich (banner, contentinfo, main) liegt innerhalb eines anderen Landmarks statt auf oberster Ebene.
    .explanation-user-impact = Screenreader-Nutzer, die per Landmark-Navigation zu Kopf, Fuß oder Hauptinhalt springen, landen in einem unerwarteten Kontext.
    .typical-cause = role="banner", role="contentinfo" oder <main> innerhalb von <main>, <nav>, <aside> oder einem benannten Bereich.
    .recommendation = Kopf-, Fuß- und Hauptinhaltsbereich auf die oberste Ebene der Seite verschieben, außerhalb jedes anderen Landmarks.
    .technical-note = Ein <header>/<footer> in <main>, <article> oder <section> ist ohnehin kein banner/contentinfo; gemeldet werden explizite Rollen und <main>. Welche Rolle betroffen ist, steht im Befundtext.

rule-landmarks-banner-duplicate = Mehrere Banner-Landmarks
    .user-impact = Screenreader-Nutzer können den 'Kopfbereich' der Seite nicht eindeutig zuordnen.
    .technical-impact = Die Seite hat mehr als einen banner-Landmark (ein <header> außerhalb von article, aside, main, nav und section oder role="banner"); der Befund zeigt auf den zweiten, daher führt die Landmark-Liste eines Screenreaders mehrere Banner und Nutzer erkennen nicht, welcher der Seitenkopf ist.

rule-landmarks-contentinfo-duplicate = Mehrere Contentinfo-Landmarks
    .user-impact = Screenreader-Nutzer können den 'Fußbereich' der Seite nicht eindeutig zuordnen.
    .technical-impact = Die Seite hat mehr als einen contentinfo-Landmark (ein <footer> außerhalb von article, aside, main, nav und section oder role="contentinfo"); der Befund zeigt auf den zweiten, daher führt die Landmark-Liste eines Screenreaders mehrere Fußbereiche und Nutzer erkennen nicht, welcher der Seitenfuß ist.

rule-landmarks-main-duplicate = Mehrere Main-Landmarks
    .user-impact = Screenreader-Nutzer, die zum Hauptinhalt springen, wissen nicht, welcher Bereich gemeint ist.
    .technical-impact = Die Seite hat mehr als einen main-Landmark (<main> oder role="main"); der Befund zeigt auf den zweiten, daher hat die Sprungfunktion „zum Hauptinhalt“ eines Screenreaders mehrere Ziele und Nutzer erkennen nicht, welches den Seiteninhalt enthält.

rule-landmarks-banner-missing = Prüfen, ob die Seite einen Banner-Landmark hat
    .user-impact = Assistierende Technologien können den Kopfbereich der Seite nicht identifizieren.
    .technical-impact = Die Seite hat keinen banner-Landmark: weder ein <header> außerhalb von article, aside, main, nav und section noch ein Element mit role="banner", daher hat die Landmark-Navigation eines Screenreaders keinen Eintrag für den Seitenkopf; als Prüfhinweis gemeldet, weil eine Seite auch ohne Kopfbereich auskommen kann.

rule-landmarks-navigation-missing = Prüfen, ob die Seite einen Navigations-Landmark hat
    .user-impact = Screenreader-Nutzer können nicht direkt zur Navigation springen.
    .technical-impact = Die Seite hat weder ein <nav> noch ein Element mit role="navigation", daher hat die Landmark-Navigation eines Screenreaders keinen Eintrag für das Seitenmenü; als Prüfhinweis gemeldet, weil eine Seite auch ohne Navigation auskommen kann.

rule-landmarks-contentinfo-missing = Prüfen, ob die Seite einen Fußbereich-Landmark (contentinfo) hat
    .user-impact = Screenreader-Nutzer können den Fußbereich mit Kontakt, Impressum und Rechtlichem nicht direkt ansteuern.
    .technical-impact = Die Seite hat keinen contentinfo-Landmark: weder ein <footer> außerhalb von article, aside, main, nav und section noch ein Element mit role="contentinfo", daher hat die Landmark-Navigation eines Screenreaders keinen Eintrag für den Fußbereich mit Kontakt und Rechtlichem; als Prüfhinweis gemeldet, weil eine Seite auch ohne Fußbereich auskommen kann.

rule-lists-item-outside-list = Listeneintrag außerhalb einer Liste
    .user-impact = Screenreader-Nutzer erhalten keine Ansage der Listenlänge/-struktur.
    .technical-impact = Ein <li> hat keine Liste über sich (<ul>, <ol>, <menu> oder role="list") oder die umgebende Liste trägt eine Rolle, die keine Liste ist, daher steht der Eintrag im Accessibility-Tree ohne Liste und ein Screenreader sagt weder die Anzahl der Einträge noch die Position an; ein Element mit role="listitem" prüft die ARIA-Regel zum erforderlichen Elternelement.
    .description = Ein Listeneintrag (<li>) steht nicht innerhalb einer Liste.
    .explanation-user-impact = Screenreader kündigen einen Listeneintrag an, ohne dass es eine Liste mit Länge und Position gibt.
    .typical-cause = Das umschließende <ul>/<ol> fehlt, etwa weil eine Komponente nur die Einträge rendert.
    .recommendation = Listeneinträge in ein <ul> oder <ol> (bzw. role="list") einbetten, oder die Listen-Semantik entfernen, wenn es keine Liste ist.
    .technical-note = Ein <li> braucht irgendwo über sich eine Liste (<ul>, <ol>, <menu> oder role="list"); in einer Liste mit role="none" oder "presentation" wird es nicht gemeldet. Ein Element mit role="listitem" prüft die ARIA-Regel zum erforderlichen Elternelement.

rule-headings-skip-level = Übersprungene Überschriftenebene
    .user-impact = Screenreader-Nutzer, die über Überschriften navigieren, vermuten fehlende Abschnitte.
    .technical-impact = Eine Überschrift liegt mehr als eine Ebene tiefer als die vorherige (etwa ein h4 nach einem h2; auch role="heading" mit aria-level zählt), daher hat die Gliederung eine Lücke und Screenreader-Nutzer, die nach Überschriftenebene navigieren, vermuten einen fehlenden Abschnitt; ein Best-Practice-Befund ohne WCAG-Kriterium.
    .description = Die Überschriften springen über eine Ebene, etwa von h2 direkt auf h4. Die Gliederung der Seite ist dadurch lückenhaft.
    .explanation-user-impact = Screenreader-Nutzer navigieren über die Überschriftenliste und vermuten bei einer fehlenden Ebene übersehene Inhalte.
    .typical-cause = Überschriftenebenen werden nach Schriftgröße statt nach Gliederung gewählt, oder Komponenten bringen eine feste Ebene mit.
    .recommendation = Überschriften lückenlos verschachteln (h1, h2, h3 …) und die Optik per CSS steuern, nicht über die Ebene.
    .technical-note = Auf eine Überschrift der Ebene n folgt höchstens Ebene n+1. Komponenten sollten die Ebene als Parameter annehmen.

rule-headings-empty = Leere Überschrift
    .user-impact = Screenreader-Nutzer stoßen in der Überschriftenliste auf einen Eintrag ohne Inhalt.
    .technical-impact = Eine Überschrift (<h1> bis <h6> oder role="heading") enthält keinen Text, kein Bild mit alt, kein aria-label und kein auflösbares aria-labelledby, daher listet ein Screenreader eine leere Überschrift und Nutzer finden nichts, wohin sie navigieren; ein Prüfhinweis, wenn ihr aria-labelledby auf ein verstecktes oder fehlendes Element zeigt.
    .description = Ein Überschriften-Element hat keinen Text. Es steht in der Gliederung der Seite, sagt aber nichts.
    .explanation-user-impact = Screenreader-Nutzer navigieren über die Überschriftenliste und treffen auf einen Eintrag ohne Namen.
    .typical-cause = Überschriften-Tags als Abstandshalter oder für Icons, oder CMS-Felder, die leer bleiben, während das Tag trotzdem ausgegeben wird.
    .recommendation = Leere Überschriften entfernen oder mit einem sinnvollen Text füllen; Abstände per CSS setzen.
    .technical-note = Das Tag nur rendern, wenn der Titel vorhanden ist. Ein Icon allein benennt keine Überschrift.

rule-headings-h1-missing = Prüfen, ob die Seite eine Hauptüberschrift (h1) hat
    .user-impact = Screenreader-Nutzer finden den Einstieg in den Hauptinhalt nicht über die Überschriftennavigation.
    .technical-impact = Das Dokument hat Überschriften, aber weder ein <h1> noch ein role="heading" mit aria-level="1", daher hat die Überschriftennavigation eines Screenreaders keinen Eintrag der Ebene 1 und das Seitenthema fehlt in der Gliederung.
    .description = Die Seite hat keine Überschrift der Ebene 1, die ihr Thema benennt.
    .explanation-user-impact = Screenreader-Nutzer springen gezielt zur h1, um das Thema der Seite zu erfahren, und finden keine.
    .typical-cause = Das Logo oder ein Bild übernimmt die Rolle der Überschrift, oder das Template setzt die Hauptüberschrift als h2 oder als formatierten Text.
    .recommendation = Jeder Seite genau eine h1 mit ihrem Thema geben, am Anfang des Hauptinhalts.
    .technical-note = Die h1 gehört in <main>, nicht ins Logo im Header.

rule-headings-h1-multiple = Mehrere Hauptüberschriften (h1)
    .user-impact = Screenreader-Nutzer können den eigentlichen Seitentitel schwerer vom Rest unterscheiden.
    .technical-impact = Das Dokument enthält mehr als ein <h1>, daher hat die Überschriftengliederung mehrere Einträge der obersten Ebene und Screenreader-Nutzer erkennen nicht, welcher die Seite benennt.
    .description = Die Seite hat mehr als eine Überschrift der Ebene 1. HTML erlaubt das, es sollte aber gewollt sein.
    .explanation-user-impact = Screenreader-Nutzer können den eigentlichen Seitentitel schwerer von anderen Abschnitten unterscheiden.
    .typical-cause = Logo, Teaser oder Seitenleiste verwenden ebenfalls h1, oft aus einem gemeinsamen Template.
    .recommendation = Prüfen, ob jede h1 wirklich ein Hauptthema der Seite benennt; alle anderen auf h2 oder tiefer setzen.
    .technical-note = Ein Hinweis zur Prüfung, kein Verstoß: Die Einordnung hängt vom Aufbau der Seite ab.

rule-lists-invalid-structure = Liste mit unzulässigen Kindelementen
    .user-impact = Screenreader-Nutzer hören eine falsche Anzahl von Listeneinträgen oder verlieren Inhalte.
    .technical-impact = Ein <ul>, <ol>, <menu> oder role="list" hat ein direktes Kind, das kein Listeneintrag ist (<li> oder role="listitem"; <script> und <template> sind erlaubt), etwa ein Wrapper-<div> oder ein <li> mit anderer Rolle, daher zählt der Accessibility-Tree es nicht als Eintrag und ein Screenreader sagt eine falsche Anzahl an oder liest diesen Inhalt außerhalb der Liste vor.
    .description = Eine Liste (<ul>, <ol> oder role="list") enthält direkte Kindelemente, die keine Listeneinträge sind.
    .explanation-user-impact = Screenreader nennen eine falsche Zahl an Einträgen oder lesen Teile der Liste ohne Listenzusammenhang vor.
    .typical-cause = Wrapper-<div>s, Trennlinien oder Überschriften direkt in der Liste, oft aus einem Slider- oder Menü-Baustein.
    .recommendation = Alle Inhalte der Liste in <li>-Elemente legen; Wrapper und Trenner in die Einträge verschieben oder per CSS erzeugen.
    .technical-note = Direkte Kinder von <ul>/<ol> dürfen nur <li>, <script> und <template> sein.

rule-lists-empty = Leere Liste
    .user-impact = Screenreader kündigen eine Liste ohne Inhalt an.
    .technical-impact = Ein <ul>, <ol>, <menu> oder role="list" hat kein Listeneintrag-Kind (Kinder, die keine Einträge sind, sind ein anderer Befund), daher kündigt ein Screenreader eine Liste ohne Inhalt an; ein Best-Practice-Befund ohne WCAG-Kriterium.
    .description = Eine Liste enthält keinen einzigen Listeneintrag.
    .explanation-user-impact = Screenreader kündigen eine Liste an, auf die nichts folgt.
    .typical-cause = Listen, die per Skript oder aus dem CMS befüllt werden und leer bleiben, oder Listen-Markup als reiner Layout-Container.
    .recommendation = Leere Listen nicht ausgeben, oder das Listen-Element erst rendern, wenn Einträge vorhanden sind.
    .technical-note = Beim Rendern prüfen, ob Einträge vorliegen; ein Layout-Container braucht kein <ul>.

rule-lists-term-without-definition = Begriff ohne Definition in Definitionsliste
    .user-impact = Screenreader-Nutzer können Begriff und Definition nicht als zusammengehörig erkennen.
    .technical-impact = Auf ein <dt> in einem <dl> folgt kein <dd>, daher hat der Begriff keine Definition und ein Screenreader kündigt eine Beschreibungsliste mit unvollständigen Einträgen an.
    .description = In einer Beschreibungsliste (<dl>) steht ein Begriff (<dt>) ohne zugehörige Beschreibung (<dd>).
    .explanation-user-impact = Screenreader lesen einen Begriff vor, zu dem keine Erklärung folgt.
    .typical-cause = Leere Felder im CMS, oder <dl> als Layout für Beschriftung und Wert, bei dem der Wert fehlt.
    .recommendation = Zu jedem <dt> mindestens ein <dd> ausgeben, oder den Begriff weglassen, wenn es keine Beschreibung gibt.
    .technical-note = Geprüft wird unter demselben Elternelement, auch innerhalb von <div>-Gruppen in der <dl>.

rule-tables-header-missing = Tabelle ohne Kopfzellen
    .user-impact = Screenreader-Nutzer hören Zellwerte ohne die zugehörige Zeilen- oder Spaltenbeschriftung.
    .technical-impact = Eine Tabelle (<table>, role="table" oder role="grid"), die nicht als präsentational ausgezeichnet ist, hat keine Kopfzelle (kein <th> und kein role="columnheader" oder "rowheader"), daher kann ein Screenreader zu einem Zellwert keine Zeilen- oder Spaltenbeschriftung ansagen; als Prüfhinweis gemeldet, weil es sich um eine Layouttabelle handeln kann, und als Verstoß (F91), wenn die erste Zeile nur aus fetten <td>-Zellen (<b>, <strong>) über weiteren Zeilen besteht und die Tabelle kein <th> hat.
    .description = Eine Tabelle hat keine Kopfzellen, die ihre Zeilen oder Spalten beschriften.
    .explanation-user-impact = Screenreader-Nutzer hören beim Durchgehen der Zellen nur Werte, ohne zu erfahren, zu welcher Spalte oder Zeile sie gehören.
    .typical-cause = Die Kopfzeile ist mit <td> und fetter Schrift gestaltet statt mit <th>, oder die Tabelle dient nur dem Layout.
    .recommendation = Kopfzellen als <th> auszeichnen (mit scope="col" bzw. scope="row"). Dient die Tabelle nur dem Layout, durch CSS-Layout ersetzen oder role="presentation" setzen.
    .technical-note = Kopfzellen sind <th> oder Elemente mit role="columnheader"/"rowheader" irgendwo in der Tabelle.

rule-tables-name-missing = Prüfen, ob Datentabellen einen Namen haben
    .user-impact = Screenreader-Nutzer erfahren beim Betreten der Tabelle nicht, worum es in ihr geht.
    .technical-impact = Eine Tabelle, die nicht als präsentational ausgezeichnet ist, hat weder eine <caption> als direktes Kind noch aria-label, aria-labelledby oder title, daher sagt ein Screenreader beim Betreten oder in der Tabellenliste der Seite nur „Tabelle“ an; als Prüfhinweis gemeldet, weil nicht jede Tabelle einen Namen braucht.
    .description = Eine Tabelle hat weder eine <caption> noch einen Namen über aria-label oder aria-labelledby.
    .explanation-user-impact = Wer sich die Tabellen einer Seite auflisten lässt, sieht nur „Tabelle“ und muss jede öffnen, um die richtige zu finden.
    .typical-cause = Die Beschriftung steht als Überschrift oder Absatz über der Tabelle, ohne mit ihr verknüpft zu sein.
    .recommendation = Der Tabelle eine <caption> geben oder die vorhandene Überschrift per aria-labelledby verknüpfen. Ob ein Name nötig ist, hängt vom Kontext ab.
    .technical-note = Ein Hinweis zur Prüfung, kein Verstoß: <caption> als erstes Kind der <table>, oder aria-labelledby auf die Überschrift.

rule-tables-presentational-with-headers = Layouttabelle mit Kopfzellen
    .user-impact = Screenreader-Nutzer erhalten widersprüchliche Signale, ob es sich um eine Datentabelle handelt.
    .technical-impact = Eine Tabelle mit role="presentation" oder "none" enthält Kopfzellen (<th> oder role="columnheader"/"rowheader"), daher entfernt die Rolle die Tabellensemantik, während die Kopfzellen sie weiter ankündigen, und ein Screenreader erhält widersprüchliche Angaben.
    .description = Eine Tabelle ist als reines Layout ausgezeichnet (role="presentation" oder "none"), enthält aber Kopfzellen.
    .explanation-user-impact = Assistierende Technik erhält widersprüchliche Angaben: keine Tabelle laut Rolle, Tabellenstruktur laut Kopfzellen.
    .typical-cause = role="presentation" wurde pauschal auf Tabellen gesetzt, darunter auch echte Datentabellen.
    .recommendation = Bei Datentabellen role="presentation" entfernen. Bei Layouttabellen <th> durch <td> ersetzen.
    .technical-note = Eine Tabelle mit <th> oder role="columnheader"/"rowheader" darf nicht role="presentation"/"none" tragen.

rule-document-lang-invalid = Ungültige Sprachangabe der Seite
    .user-impact = Screenreader wählen eine falsche oder die Standard-Aussprache.
    .technical-impact = Das lang-Attribut von <html> ist gesetzt, aber kein gültiges BCP-47-Kürzel (etwa „english“, „de_DE“ oder das dreibuchstabige „eng“), daher kann ein Screenreader keine Stimme für die Seite wählen und fällt auf seine Standardaussprache zurück.
    .description = Das lang-Attribut der Seite ist gesetzt, enthält aber keinen gültigen Sprachcode (etwa „english“ statt „en“).
    .explanation-user-impact = Screenreader erkennen die Sprache nicht und lesen den Text mit der falschen oder der Standard-Aussprache vor.
    .typical-cause = Sprachname statt Sprachcode, Unterstrich statt Bindestrich (de_DE) oder ein leerer Wert aus dem Template.
    .recommendation = Einen gültigen BCP-47-Code eintragen, etwa lang="de" oder lang="de-DE".
    .technical-note = Primärkennung aus zwei oder drei Buchstaben, danach nur alphanumerische Untertags, getrennt durch Bindestriche.

rule-keyboard-positive-tabindex = Positiver tabindex verändert die Fokusreihenfolge
    .user-impact = Tastaturnutzer erreichen Elemente in einer Reihenfolge, die nicht dem Seitenaufbau entspricht.
    .technical-impact = Ein Element hat einen tabindex über 0, daher rückt es unabhängig von seiner DOM-Position vor alle Elemente mit tabindex 0 oder ohne tabindex in die Tab-Reihenfolge, und Fokus- und Lesereihenfolge laufen auseinander.
    .description = Ein Element hat einen tabindex größer als 0 und wird damit vor allen anderen Elementen fokussiert, unabhängig von seiner Position auf der Seite.
    .explanation-user-impact = Tastaturnutzer springen beim Tabben unvorhersehbar durch die Seite und verlieren die Orientierung.
    .typical-cause = Versuche, einzelne Elemente per tabindex="1" nach vorne zu holen, oft aus Formular- oder Such-Widgets.
    .recommendation = Positive tabindex-Werte entfernen und die Reihenfolge über die Reihenfolge im HTML herstellen; tabindex="0" oder "-1" genügen.
    .technical-note = Nur tabindex="0" (in der natürlichen Reihenfolge fokussierbar) und "-1" (nur per Skript fokussierbar) verwenden.

rule-forms-title-only-label = Formularfeld-Beschriftung nur über title-Attribut
    .user-impact = Das title-Attribut wird von vielen Screenreadern inkonsistent oder gar nicht vorgelesen; sehende Nutzer sehen es nur bei Hover.
    .technical-impact = Ein Textfeld, eine Combobox, Listbox, ein Schieberegler oder Zahlenfeld erhält seinen zugänglichen Namen nur aus dem title-Attribut (kein <label>, aria-label oder aria-labelledby, kein Platzhalter), das Tastatur- und Touch-Nutzer nie sehen und das manche Screenreader uneinheitlich ansagen; als Prüfhinweis gemeldet.

rule-zoom-viewport-scale-limited = Prüfen, ob der Viewport Zoom auf 500 % erlaubt
    .user-impact = Sehbehinderte Nutzer können den Seiteninhalt nicht ausreichend vergrößern.
    .technical-impact = maximum-scale im viewport-Meta-Tag liegt bei mindestens 2, aber unter 5, daher endet das Pinch-Zoomen vor 500 %, was Nutzer mit Sehbehinderung brauchen können; ein Best-Practice-Befund, da 200 % weiterhin möglich sind.
    .description = Das viewport-Meta-Tag begrenzt maximum-scale auf einen Wert unter 5. Nutzer können die Seite auf Mobilgeräten damit nicht auf 500 % vergrößern.
    .explanation-user-impact = Sehbehinderte Nutzer können den Seiteninhalt nicht ausreichend vergrößern.
    .typical-cause = maximum-scale wurde gesetzt, um versehentliches Zoomen zu verhindern oder ein Layout zu stabilisieren.
    .recommendation = maximum-scale entfernen oder auf mindestens 5 setzen.
    .technical-note = Best Practice: 1.4.4 verlangt 200 % und ist bei dieser Grenze erfüllt. Liegt die Grenze unter 2 oder ist user-scalable=no gesetzt, meldet die Prüfung einen eigenen 1.4.4-Verstoß.

rule-keyboard-click-handler-not-focusable = Klick-Handler ohne Tastatur-Äquivalent
    .user-impact = Tastaturnutzer können die Funktion nicht auslösen.
    .technical-impact = Ein <div>, <span>, <p>, <li>, <section>, <article> oder ein <a> ohne href hat ein onclick-Attribut, aber keine interaktive Rolle und keinen tabindex, daher erreichen Tastatur- und Schalternutzer es nicht und der Accessibility-Tree zeigt keine Aktion; nur ein Prüfhinweis, wenn ein fokussierbares Element darin wahrscheinlich den Klick erhält.

rule-keyboard-pointer-only = Prüfen, ob klickbare Elemente per Tastatur bedienbar sind
    .user-impact = Tastaturnutzer können die Funktion nicht auslösen.
    .technical-impact = Ein Element sieht anklickbar aus (cursor: pointer oder ein onclick-Attribut), hat aber weder Rolle noch tabindex und enthält kein Bedienelement und steht in keinem, daher erreichen Tastaturnutzer es nicht, falls es etwas auslöst, und ein Screenreader gibt es als einfachen Text aus; als Prüfhinweis gemeldet.

rule-landmarks-main-missing = Prüfen, ob die Seite einen Hauptinhaltsbereich als Sprungziel hat
    .user-impact = Tastaturnutzer haben kein verlässliches Ziel, um wiederkehrende Navigation zu überspringen.
    .technical-impact = Die Seite hat Inhalt, aber weder ein <main> noch ein Element mit role="main", daher hat die Sprungfunktion „zum Hauptinhalt“ eines Screenreaders kein Ziel und Nutzer müssen sich zuerst durch die wiederkehrenden Kopf- und Navigationsbereiche bewegen; bei einem zum Messzeitpunkt offenen modalen Dialog hat der Befund geringe Schwere, weil der Dialog den main-Landmark verdecken kann.

rule-keyboard-skip-link-missing = Prüfen, ob die Seite einen Skip-Link hat
    .user-impact = Tastaturnutzer müssen bei jedem Seitenwechsel durch die gesamte Navigation tabben.
    .technical-impact = Die Seite hat wiederkehrende Blöcke (Links, einen Navigations- oder Banner-Landmark), aber keinen Link, der sie überspringt: keinen Fragmentlink auf den Anfang des Hauptinhalts, keine Folge von Seitenlinks vor dem ersten echten Link und keinen Fragmentlink, dessen Text oder class nach „Überspringen“ klingt, daher müssen Tastaturnutzer zuerst durch Kopfbereich und Navigation tabben; ein Prüfhinweis, weil auch ein main-Landmark oder Überschriften die Blöcke umgehen können.

rule-focus-indicator-missing = Kein sichtbarer Fokusindikator gemessen
    .user-impact = Tastaturnutzer sehen nicht, welches Element gerade fokussiert ist.
    .technical-impact = Fokuslauf: die berechneten Stile des Elements vor und nach dem Fokussieren sind gleich, soweit sie als Indikator taugen (outline, box-shadow, border, background, color, text-decoration).

rule-focus-outline-removed = Fokus-Indikator per CSS entfernt
    .user-impact = Tastaturnutzer sehen nicht, welches Element gerade fokussiert ist.
    .technical-impact = Ein Stylesheet entfernt den Rahmen (outline: none oder 0) unter :focus oder :focus-visible und keine Regel der Seite ersetzt ihn (eine :focus-visible-Regel, die den Rahmen behält, oder eine andere sichtbare Gestaltung wie box-shadow an :focus), daher sehen Tastaturnutzer nicht, wo der Fokus steht; ein Prüfhinweis, wenn onfocus-Handler einen anderen Hinweis zeigen könnten.

rule-keyboard-scrollable-region-not-focusable = Scrollbarer Bereich per Tastatur nicht erreichbar
    .user-impact = Tastaturnutzer können den Bereich nicht scrollen und den verdeckten Inhalt nicht lesen.
    .technical-impact = Ein Bereich scrollt (mehr als 13 px Inhalt über den Kasten hinaus) und enthält Text oder Bilder mit alt, aber weder er noch etwas darin ist mit der Tabulatortaste erreichbar, daher können Tastaturnutzer ihn nicht scrollen und erreichen den abgeschnittenen Teil nie.

rule-forms-error-unidentified = Ungültiges Feld ohne zugängliche Fehlerbeschreibung
    .user-impact = Screenreader-Nutzer erfahren, dass ein Fehler vorliegt, aber nicht welcher.
    .technical-impact = Ein Feld hat aria-invalid (jeder Wert außer false), aber weder aria-describedby noch aria-errormessage mit Text, daher sagt ein Screenreader „ungültig“ an, ohne zu sagen, was falsch ist; als Prüfhinweis gemeldet, weil eine sichtbare Fehlermeldung neben dem Feld als Beschreibung dienen kann.

rule-aria-owns-conflict = Element gehört per aria-owns zu mehreren Elementen
    .user-impact = Assistierende Technologien können nicht zuverlässig bestimmen, welches Element gemeint ist.
    .technical-impact = Eine id, die im aria-owns eines Elements steht, steht schon im aria-owns eines anderen Elements, daher hat das besessene Element zwei Besitzer und die Hilfstechnik kann nicht entscheiden, zu welchem es gehört.
    .description = Dieselbe ID steht im aria-owns mehrerer Elemente. Welches Element der Besitzer ist, bleibt offen.
    .explanation-user-impact = Assistierende Technologien können das Element keinem Widget eindeutig zuordnen; Name, Rolle und Wert werden unzuverlässig.
    .typical-cause = Mehrfach eingebundene Komponenten, die auf dieselbe Liste zeigen, oder eine doppelt vergebene ID.
    .recommendation = Jedes Element nur im aria-owns eines einzigen Elements aufführen.
    .technical-note = WAI-ARIA 1.2: Eine ID darf zu keinem Zeitpunkt im aria-owns mehr als eines anderen Elements stehen.

rule-names-required-missing = Interaktives Element ohne aussagekräftigen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, wozu das Element dient.
    .technical-impact = Ein Element, dessen Rolle einen Namen verlangt (etwa checkbox, combobox, listbox, radio, slider, switch, tab, textbox, menuitem oder ein Formularfeld mit leerer Beschriftung), hat keinen zugänglichen Namen aus aria-label, aria-labelledby, einem Label oder seinem Inhalt, daher sagt ein Screenreader nur die Rolle an; Buttons, Links, Dialoge, <summary>, Felder ganz ohne Beschriftung, Elemente mit Bildrolle, Messanzeigen und Fortschrittsbalken haben eigene Regeln.

rule-names-symbol-only = Prüfen, ob Namen aus reinen Symbolen die Funktion beschreiben
    .user-impact = Screenreader-Nutzer erfahren nicht, wozu das Element dient.
    .technical-impact = Der zugängliche Name stammt aus einem Attribut wie aria-label und besteht nur aus Symbolen wie „×“ oder „☰“, daher liest ein Screenreader einen Symbolnamen oder nichts vor und der Name beschreibt die Funktion nicht.

rule-names-title-only = Natives title-Attribut als einzige Beschreibung
    .user-impact = Der Tooltip des title-Attributs erscheint nur bei Mauszeiger-Hover; per Tastatur und auf Touchgeräten bleibt der Zweck des Elements unsichtbar.
    .technical-impact = Ein <button>, <a href>, <input type="button"> oder "image" oder eine unbeschriftete Checkbox oder ein Optionsfeld hat einen title, aber weder Text, alt, aria-label noch aria-labelledby, daher stammt sein zugänglicher Name allein aus title, den Tastatur- und Touch-Nutzer nie sehen und den manche Screenreader uneinheitlich ansagen; als Prüfhinweis gemeldet.

rule-names-description-duplicates-name = Prüfen, ob Beschreibungen mehr sagen als der Name
    .user-impact = Screenreader sagen denselben Text zweimal an, ohne zusätzliche Information.
    .technical-impact = Ein interaktives Element (Button, Link, Formularelement, Menüpunkt, Tab und Ähnliches) hat im Accessibility-Tree des Browsers eine zugängliche Beschreibung, die nach Angleichen der Leerzeichen seinem zugänglichen Namen gleicht, typischerweise ein aria-describedby oder aria-description, das den sichtbaren Text wiederholt, daher sagt ein Screenreader denselben Text doppelt an; als Prüfhinweis gemeldet.
    .description = Zugänglicher Name und zugängliche Beschreibung eines Elements sind identisch. Das Element hat einen Namen; die doppelte Ansage ist überflüssig, kein fehlender Name.
    .explanation-user-impact = Screenreader sagen denselben Text zweimal an, ohne zusätzliche Information.
    .typical-cause = aria-describedby oder aria-description wiederholt den sichtbaren Text oder das aria-label, etwa ein Button „Schließen“, beschrieben durch ein verstecktes „Schließen“.
    .recommendation = Die Beschreibung entfernen oder mit einer Information füllen, die über den Namen hinausgeht.
    .technical-note = Prüfhinweis unter WCAG 4.1.2, kein Verstoß: Das Kriterium verlangt einen Namen, keine Beschreibung, die sich von ihm unterscheidet. Verglichen werden Name und Beschreibung aus dem Accessibility-Baum des Browsers.

rule-aria-required-children-missing = Erforderliche Bestandteile einer ARIA-Rolle fehlen
    .user-impact = Assistierende Technologien können die Beziehung zwischen den Widget-Teilen nicht auswerten.
    .technical-impact = Ein Element, dessen Rolle besessene Teile verlangt, etwa tablist ohne tab, list ohne listitem oder menu ohne menuitem, enthält oder besitzt keines davon, außer es ist mit aria-expanded="false" eingeklappt oder beschäftigt.
    .description = Ein Behälter wie role="tablist", "list" oder "menu" enthält keines der Elemente, die seine Rolle verlangt.
    .explanation-user-impact = Screenreader sagen einen leeren oder fehlerhaften Behälter an; Nutzer finden die Einträge nicht.
    .typical-cause = Die Einträge tragen keine oder eine andere Rolle, oder ein Element mit eigener Rolle steht zwischen Behälter und Einträgen.
    .recommendation = Den Einträgen die geforderte Rolle geben, etwa role="tab" in einer tablist, oder die Rolle des Behälters entfernen.
    .technical-note = Geprüft werden nur Behälter mit expliziter Rolle; native Tabellen und Listen geben ihre Bestandteile per HTML vor.

rule-dialog-name-missing = Dialog ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erfahren beim Öffnen des Dialogs nicht, worum es geht.
    .technical-impact = Ein offener <dialog> oder ein Element mit role="dialog" oder "alertdialog" hat keinen zugänglichen Namen (kein aria-label, kein auflösbares aria-labelledby, kein title), daher sagt ein Screenreader beim Öffnen „Dialog“, ohne zu sagen, worum es geht.

rule-buttons-name-missing = Formularelement, Link oder Button ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Eingabe erwartet wird oder wohin ein Link/Button führt.
    .technical-impact = Ein <button> oder ein Element mit role="button" (kein <summary>) hat keinen zugänglichen Namen aus Textinhalt, Bild-alt, aria-label, aria-labelledby oder title, daher sagt ein Screenreader nur „Schaltfläche“ an.

rule-links-name-missing = Formularelement, Link oder Button ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Eingabe erwartet wird oder wohin ein Link/Button führt.
    .technical-impact = Ein <a href> hat weder Textinhalt noch Bild-alt, aria-label oder aria-labelledby, daher ist sein zugänglicher Name leer und ein Screenreader sagt nur „Link“ an oder liest die URL vor.

rule-links-used-as-button = Link als Button-Ersatz ohne Tastenrolle
    .user-impact = Assistierende Technologien kündigen das Element als Link an, obwohl es sich wie ein Button verhält — Tastaturnutzer erwarten das falsche Bedienverhalten.
    .technical-impact = <a href="#">/<a href="javascript:..."> mit onclick, ohne role="button".

rule-aria-role-redundant = Redundante ARIA-Rolle
    .user-impact = Kein unmittelbarer Nutzerschaden, aber unnötiges Risiko für Inkonsistenzen bei zukünftigen Änderungen.
    .technical-impact = Ein role-Attribut wiederholt die implizite native Rolle des Elements, zum Beispiel <button role="button">, was im Accessibility-Tree nichts ändert und veraltet, sobald der Elementtyp später geändert wird.

rule-aria-presentation-conflict = Schmückendes Element bleibt für Hilfstechnik sichtbar
    .user-impact = Screenreader-Nutzer treffen auf ein Element, das ausgeblendet sein sollte, ohne Namen oder Zweck.
    .technical-impact = Ein Element mit role="none" oder "presentation" oder ein <img alt=""> trägt zusätzlich ein globales ARIA-Attribut (etwa aria-label oder aria-describedby) oder ist fokussierbar, daher ignorieren Browser die schmückende Rolle und ein Screenreader sagt das Element trotzdem an.

rule-aria-presentational-children-focusable = Fokussierbares Element in einem Bedienelement mit präsentationalen Kindern
    .user-impact = Tastaturnutzer erreichen ein Element, das Screenreader nicht ansagen.
    .technical-impact = Ein fokussierbares Element steckt in einem Button, einer Checkbox, einem Optionsfeld, Tab, Schalter, Schieberegler, einer Option, einem Bild oder einer anderen Rolle mit präsentationalen Kindern (auch native <button> und <option>), daher hält der Tastaturfokus an etwas, das im Accessibility-Tree weder Rolle noch Namen hat.

rule-lists-role-redundant = Redundante ARIA-Rolle
    .user-impact = Kein unmittelbarer Nutzerschaden, aber unnötiges Risiko für Inkonsistenzen bei zukünftigen Änderungen.
    .technical-impact = Ein <ul> oder <ol> trägt role="list", obwohl sein list-style-type nicht none ist, daher dupliziert die Rolle die native Listensemantik und ändert im Accessibility-Tree nichts.

rule-summary-name-missing = Summary-Element ohne zugänglichen Namen
    .user-impact = Screenreader-Nutzer erfahren nicht, was der Aufklapp-Bereich enthält, bevor sie ihn öffnen.
    .technical-impact = Das <summary>, das sein <details> bedient (das erste <summary>-Kind), hat keinen zugänglichen Namen aus Text, aria-label, aria-labelledby oder title, daher sagt ein Screenreader eine unbenannte Aufklappschaltfläche an und Nutzer erkennen vor dem Öffnen nicht, was der Abschnitt enthält.

rule-landmarks-content-outside = Inhalt außerhalb einer Landmark-Region
    .user-impact = Screenreader-Nutzer, die zwischen Landmarken springen, erreichen diesen Inhalt nie.
    .technical-impact = Sichtbarer Inhalt liegt außerhalb jeder Landmark-Region (<header>, <nav>, <main>, <footer> oder eine entsprechende Rolle), daher überspringt ihn die Landmark-Navigation eines Screenreaders und Regionen-Kürzel erreichen ihn nicht.

rule-keyboard-focusable-no-role = Fokussierbares Element ohne interaktive Rolle
    .user-impact = Screenreader-Nutzer hören einen Tabstopp ohne Rolle und erkennen weder, was er ist, noch, wie er zu bedienen ist.
    .technical-impact = Ein Element, das nicht von sich aus fokussierbar ist, hat einen tabindex ab 0 und keine interaktive Rolle (generic, group, article, section, paragraph, none, presentation oder gar keine Rolle), daher erreicht die Tabtaste es, ein Screenreader sagt es aber ohne Rolle und ohne bedienbaren Zustand an; als Prüfhinweis gemeldet, weil ein Skript es dennoch bedienbar machen kann.

rule-display-toggle-missing = Kein Umschalter für Darstellungsmodi
    .user-impact = Fehlt der Umschalter tatsächlich, kann nicht auf eine ruhige oder reine Textdarstellung wechseln, wer Bewegung oder Grafiken nicht verträgt.
    .technical-impact = Kein Element mit [data-display-toggle] auf einer Seite mit figure[data-viz].
    .description = Die Seite zeigt Visualisierungen, aber es wurde kein mit [data-display-toggle] markierter Umschalter gefunden.
    .explanation-user-impact = Fehlt der Umschalter tatsächlich, bleibt an die volle Darstellung gebunden, wer Bewegung, 3D oder Grafiken nicht verträgt.
    .typical-cause = Visualisierungen nach der Konvention ausgezeichnet, der Umschalter aber nicht eingebaut oder nicht mit [data-display-toggle] markiert.
    .recommendation = Auf jeder Seite mit Visualisierungen einen bedienbaren Umschalter anbieten, der zwischen visual, calm und text wechselt und die Wahl speichert.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 2.2.2 (Pause, Stop, Hide). Geprüft wird das Element [data-display-toggle] im gerenderten DOM.

rule-display-init-missing = Darstellungsmodus wird zu spät gesetzt
    .user-impact = Die Seite kann kurz in der falschen Darstellung erscheinen, etwa mit Animation, bevor die gewählte ruhige Darstellung greift.
    .technical-impact = Auf einer Seite, die die Darstellungskonvention nutzt (figure[data-viz] oder [data-display-toggle]), wird html[data-display] nie gesetzt oder war beim Parsen des Seitenrumpfs noch nicht gesetzt, daher erscheint die Seite zuerst im falschen Modus, etwa mit Animation, bevor der gewählte ruhige Modus greift.
    .description = Die gewählte Darstellung wird nicht vor dem ersten Anzeigen der Seite angewendet.
    .explanation-user-impact = Die Seite kann kurz mit Animation oder Grafiken erscheinen, obwohl eine ruhige Darstellung gewählt wurde.
    .typical-cause = data-display wird in einem verzögerten oder Modul-Skript gesetzt statt in einem kleinen blockierenden Skript im <head>.
    .recommendation = Den Modus in einem kleinen, blockierenden Skript im <head> setzen, bevor der Seitenkörper beginnt.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 2.2.2. Gemessen per CDP: ein vor der Navigation injizierter Beobachter hält fest, ob html[data-display] beim Einfügen von <body> bereits gesetzt war.

rule-display-text-media-visible = Grafiken im Textmodus sichtbar
    .user-impact = Wer den Textmodus gewählt hat, bekommt trotzdem Grafiken oder bewegte Inhalte.
    .technical-impact = Bei gesetztem html[data-display="text"] zeigt eine Visualisierung weiter Medien: einen [data-viz-live]- oder [data-viz-static]-Teil oder ein canvas, svg, img, video, iframe, object oder embed über 32 × 32 px außerhalb ihrer Textschicht, daher zeigt der Textmodus Grafiken statt nur der Aussage als Text.
    .description = Im Textmodus zeigt eine Visualisierung weiterhin Grafik, Canvas oder Video.
    .explanation-user-impact = Wer den Textmodus gewählt hat, bekommt trotzdem Grafiken oder bewegte Inhalte.
    .typical-cause = Die CSS-Regeln für html[data-display="text"] blenden [data-viz-live] oder [data-viz-static] nicht aus.
    .recommendation = Im Textmodus Live- und Standbild-Teile der Visualisierung ausblenden und nur die Textschicht zeigen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 2.2.2. Geprüft wird das gerenderte Layout im Modus text; Elemente bis 32×32 px gelten als Icons.

rule-display-text-not-visible = Textschicht fehlt im Textmodus
    .user-impact = Im Textmodus geht die Aussage der Grafik verloren.
    .technical-impact = Bei gesetztem html[data-display="text"] hat eine Visualisierung eine [data-viz-text]-Ebene mit Inhalt, aber keine ihrer Ebenen wird dargestellt und ist sichtbar (etwa per CSS für den Textmodus ausgeblendet), daher erscheint die Aussage im Textmodus nicht und die Textalternative geht dort verloren.
    .description = Im Textmodus zeigt eine Visualisierung keine lesbare Aussage.
    .explanation-user-impact = Wer den Textmodus gewählt hat, erfährt nicht, was die Grafik aussagt.
    .typical-cause = Das CSS für den Textmodus blendet [data-viz-text] aus, oder die Ebene wird in diesem Modus nicht dargestellt.
    .recommendation = Jede Visualisierung mit einer Textschicht versehen (Kernaussage, bei Diagrammen die Werte als Tabelle, Quelle) und sie im Textmodus anzeigen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.1.1 (Nicht-Text-Inhalt). Geprüft wird das gerenderte Layout im Modus text.

rule-viz-text-missing = Visualisierung ohne Textschicht
    .user-impact = Im Textmodus geht die Aussage der Grafik verloren.
    .technical-impact = Eine figure[data-viz]-Visualisierung hat kein [data-viz-text]-Element oder nur ein leeres, daher gibt es in keinem Anzeigemodus eine Textebene und die Hilfstechnik sieht nur die Grafik.
    .description = Eine Visualisierung hat keine mit [data-viz-text] markierte Textschicht oder nur eine leere.
    .explanation-user-impact = Wer die Grafik nicht sehen kann, erfährt ihre Aussage nicht; der Textmodus hat nichts, was statt der Grafik erscheint.
    .typical-cause = Die Beschreibung steht in einem Element ohne die Marke [data-viz-text], oder die Visualisierung wurde ohne Textschicht eingebaut.
    .recommendation = Jeder figure[data-viz] eine Textschicht mit der Aussage geben (bei Diagrammen die Werte, am besten als <table>) und die Quelle nennen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.1.1. Geprüft wird nur, dass Text da ist, nicht ob er die Kernaussage trifft. Aus dem Markup, in jedem Modus.

rule-display-text-hidden = Textschicht einer Visualisierung für Hilfstechnik verborgen
    .user-impact = Screenreader-Nutzer erfahren die Aussage der Grafik nicht.
    .technical-impact = hidden, aria-hidden=true, inert oder display:none/visibility:hidden an [data-viz-text] oder einem Vorfahren.
    .description = Die Textschicht einer Visualisierung ist für Screenreader ausgeblendet.
    .explanation-user-impact = Screenreader-Nutzer erfahren die Aussage der Grafik nicht.
    .typical-cause = hidden, aria-hidden="true", inert oder display:none an [data-viz-text] oder einem umgebenden Element, meist um sie in der grafischen Darstellung zu verstecken.
    .recommendation = Die Textschicht nur optisch verstecken (visually-hidden), nie für Hilfstechnik.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.1.1. Geprüft im gerenderten DOM in jedem Modus, einschließlich berechneter Stile.

rule-viz-caption-missing = Visualisierung ohne Beschriftung
    .user-impact = Nichts benennt die Visualisierung kurz; wer sie nicht sieht, erfährt nicht, worum es geht, bevor er sich durch die Textschicht liest.
    .technical-impact = figure[data-viz] ohne <figcaption> als direktes Kind.
    .description = Eine Visualisierung hat kein <figcaption>, das sie kurz benennt.
    .explanation-user-impact = Nichts benennt die Visualisierung kurz; wer sie nicht sieht, muss erst die Textschicht lesen, um zu wissen, worum es geht.
    .typical-cause = Das <figcaption> fehlt oder steht außerhalb der <figure>.
    .recommendation = Ein <figcaption> als direktes Kind der figure[data-viz] ergänzen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.1.1. Geprüft wird das Vorhandensein, nicht der Inhalt.

rule-viz-static-missing = Bewegte Visualisierung ohne Standbild
    .user-impact = Im ruhigen Modus ersetzt nichts die Bewegung; wer Bewegung nicht verträgt, sieht entweder sie oder gar nichts.
    .technical-impact = figure[data-viz=3d|interactive] ohne Nachfahre mit [data-viz-static].
    .description = Eine 3D- oder interaktive Visualisierung hat kein Standbild für den ruhigen Modus.
    .explanation-user-impact = Wer Bewegung nicht verträgt, bekommt im ruhigen Modus nichts, was die Bewegung ersetzt.
    .typical-cause = Nur die bewegte Fassung wurde gebaut, ohne ein mit [data-viz-static] markiertes Standbild.
    .recommendation = Ein Standbild mit [data-viz-static] in die Visualisierung aufnehmen und im ruhigen Modus anstelle der Bewegung zeigen.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 2.2.2. Gilt für data-viz="3d" und "interactive".

rule-viz-table-missing = Diagramm ohne Wertetabelle
    .user-impact = Screenreader-Nutzer können die Werte des Diagramms nicht als Zeilen und Spalten lesen und erkennen den Zusammenhang nicht.
    .technical-impact = figure[data-viz=chart] ohne <table> im Teilbaum.
    .description = Ein Diagramm (data-viz="chart") hat keine <table> mit seinen Werten.
    .explanation-user-impact = Die Beziehungen zwischen den Werten können verloren gehen, wenn sie nur als Fließtext oder gar nicht vorliegen.
    .typical-cause = Die Werte stehen nur in der Grafik oder in einem Satz, bei Sparklines oft bewusst.
    .recommendation = Die Werte als <table> in die Textschicht aufnehmen, sofern ein Satz sie nicht ebenso vollständig trägt.
    .technical-note = Best Practice aus der BarrierLab-Konvention für Darstellungsmodi (Entwurf v0), keine WCAG-Anforderung; verankert an WCAG 1.3.1. Prüfhinweis, kein Verstoß: ob ein Satz genügt, kann nur ein Mensch beurteilen.

# Explanation texts of the entries of `EXPLANATIONS` in
# `src/output/explanations.rs` that are keyed by a WCAG criterion number or an
# axe id, in the order of that table: one message per entry. The message value
# is the customer title, the attributes carry the description, the user
# impact, the typical cause, the recommendation and the technical note.
# The key is `explanation-` plus the table key with `.`, `/` and `_` written
# as `-` (`output::explanations::explanation_text_key`). An entry keyed by a
# rule id has no message here: its texts are attributes of the rule message
# above.

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
    .typical-cause = Helle Schriftfarbe auf hellem Hintergrund, graue Texte auf weißem Grund, Designentscheidungen ohne Kontrastprüfung. Auch verringerte `opacity` am Text oder an einem Elternelement (ein abgedunkelter inaktiver Zustand, ein Einblendeffekt): Die Textfarbe wird mit dem Hintergrund gemischt, das Verhältnis liegt dann unter dem, was die Farbwerte vermuten lassen.
    .recommendation = Schriftfarben anpassen, sodass normaler Text mindestens ein Kontrastverhältnis von 4,5:1 und großer Text von 3:1 zum Hintergrund hat. Dämpft `opacity` den Text, die Deckkraft erhöhen oder die Farbe abdunkeln, bis auch der gedämpfte Zustand das Verhältnis erreicht. Kontrastwerte mit Tools wie dem WebAIM Contrast Checker prüfen.
    .technical-note = WCAG 2.1 Level AA: 4.5:1 für normalen Text (<18pt / <14pt bold), 3:1 für großen Text (≥18pt / ≥14pt bold). Deckkraft wird in die gemessene Farbe eingerechnet, und ein Zustand wird so gemessen, wie er gerendert ist: Eine mit `opacity: .55` abgedunkelte Zeile wird in diesem Zustand beurteilt, nicht in ihrem aktiven, und der Befund nennt die Deckkraft und die Textfarbe danach. Text in einer `aria-hidden`-svg-Illustration ist ein Prüfhinweis, kein Verstoß: Reine Dekoration ist ausgenommen, Text mit Information braucht den Kontrast. Text in anderen `aria-hidden`-Elementen wird wie üblich gemessen, weil das Kriterium sich auf das Gesehene bezieht. CSS-Variablen für konsistente Farbpalette verwenden.

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
    .description = Ein Element liegt in der Tabfolge (tabindex ab 0), ohne von sich aus fokussierbar zu sein, trägt aber keine (oder eine nicht-interaktive) ARIA-Rolle. Assistive Technologien wissen dadurch nicht, was beim Fokussieren dieses Elements möglich ist.
    .user-impact = Screenreader-Nutzer hören beim Fokussieren keine oder nur eine allgemeine Rollenangabe wie "Gruppe" und können nicht einschätzen, welche Interaktion erwartet wird.
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
    .recommendation = Klickflächen auf mindestens 24×24 CSS-Pixel vergrößern, z. B. durch Padding, oder ausreichend Abstand zu benachbarten Zielen einhalten. 44×44 Pixel sind eine Ergonomie-Empfehlung, keine AA-Anforderung.
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

explanation-4-1-2 = Fehlende Name/Rolle bei Bedienelementen
    .description = Interaktive Elemente (Buttons, Links, Formularfelder) haben keinen zugänglichen Namen oder keine erkennbare Rolle. Assistive Technologien können nicht vermitteln, worum es sich handelt.
    .user-impact = Screenreader-Nutzer hören z. B. nur 'Button' ohne Beschreibung der Funktion, oder ein klickbares Element wird gar nicht als interaktiv erkannt.
    .typical-cause = Buttons oder Links ohne Text oder aria-label. Icon-only-Buttons ohne zugänglichen Namen. Custom-Widgets ohne ARIA-Rollen.
    .recommendation = Alle interaktiven Elemente mit einem verständlichen, zugänglichen Namen versehen. Native HTML-Elemente bevorzugen. Bei Icon-Buttons: aria-label verwenden.
    .technical-note = Buttons: sichtbarer Text oder aria-label. Links: sprechender Linktext. Inputs: verknüpftes <label>. Custom-Widgets: role + aria-label + aria-Zustände.

## SEO rules (no WCAG criterion)

explanation-seo-headings-long-heading = Überschrift zu lang
    .description = Überschriften auf der Seite überschreiten die empfohlene Länge von 70 Zeichen. Lange Überschriften werden in Suchergebnissen abgeschnitten und sind für Leser schwerer zu erfassen.
    .user-impact = In Suchergebnissen und sozialen Medien werden die Überschriften abgeschnitten, was den Klickanreiz verringert. Leser müssen mehr kognitive Arbeit leisten, um den Inhalt einzuordnen.
    .typical-cause = Redaktionelle Texte werden ungekürzt als Überschrift eingesetzt. CMS-Felder für Überschriften haben keine Längenbeschränkung.
    .recommendation = Überschriften auf unter 70 Zeichen kürzen und den Kern der Aussage voranstellen. Längere Beschreibungen gehören in den Fließtext.
    .technical-note = Google zeigt typischerweise 50–60 Zeichen im Title-Tag an. Für H2/H3 gilt keine feste Grenze, aber über 70 Zeichen sinkt die Lesbarkeit deutlich. CMS-Validierung empfohlen.

explanation-seo-headings-skipped-level = Übersprungene Überschriftenebene
    .description = Die Überschriften auf der Seite überspringen eine Ebene — zum Beispiel folgt auf eine H1 direkt eine H3, ohne dass eine H2 dazwischen steht. Das lässt eine Lücke in der Hierarchie, die Suchmaschinen und Screenreader zur Navigation nutzen.
    .user-impact = Screenreader-Nutzer navigieren Seiten über Überschriften. Fehlende Ebenen unterbrechen diesen Pfad und erschweren die Orientierung. Suchmaschinen bewerten Seiten mit konsistenter Überschriftenstruktur als inhaltlich hochwertiger.
    .typical-cause = CMS-Templates oder Komponenten setzen fest kodierte Überschriftenebenen (z. B. immer H3 in Footer-Spalten), unabhängig davon, was der Seitenkontext erfordert. Entwickler wählen Ebenen nach visuellem Stil statt nach Hierarchie.
    .recommendation = Überschriftenhierarchie von H1 abwärts lückenlos einhalten: H1 → H2 → H3. Wenn eine Überschrift visuell kleiner wirken soll, CSS-Klassen verwenden statt die HTML-Ebene zu ändern.
    .technical-note = Best Practice, für sich kein WCAG-Verstoß: WCAG 1.3.1 (Info and Relationships, Level A) erfordert, dass Struktur programmatisch erkennbar ist; eine übersprungene Ebene macht eine Überschrift nicht allein unerkennbar. Zu einem Problem nach 1.3.1 oder 2.4.6 wird sie erst, wenn die Ebenen die Gliederung falsch wiedergeben. Screenreader-Nutzer, die nach Überschriftenebene springen, vermuten an einer Lücke möglicherweise fehlende Abschnitte.

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

## Rule-specific texts keyed by an axe id

explanation-server-side-image-map = Server-seitige Image-Map verwendet
    .description = Ein Bild nutzt eine server-seitige Image-Map (ismap-Attribut) statt einer client-seitigen Variante.
    .user-impact = Tastatur- und Screenreader-Nutzer können die einzelnen Bildbereiche nicht ansteuern.
    .typical-cause = Ein <img ismap> in einem Link, oft in älteren Seiten mit einer anklickbaren Karte oder Navigationsgrafik.
    .recommendation = Die server-seitige Image-Map durch eine client-seitige Image-Map (<map>/<area>) oder einfache Textlinks ersetzen.
    .technical-note = Nur ein <img ismap> in einem <a href> zählt. Ein Prüfhinweis: Gleichwertige Textlinks neben dem Bild erfüllen das Kriterium.

explanation-svg-img-alt = Fehlender Alternativtext bei SVG-Grafiken
    .description = Eine SVG-Grafik mit informativem Inhalt hat keinen Alternativtext (title, aria-label).
    .user-impact = Screenreader-Nutzer erhalten keine Information über den Inhalt der Grafik.
    .typical-cause = Ein Icon- oder Diagramm-SVG, das ohne <title>, aria-label oder aria-hidden inline eingefügt wurde.
    .recommendation = Ein <title> als erstes Kindelement des <svg> ergänzen, aria-label verwenden, oder rein dekorative SVGs mit aria-hidden="true" verstecken.
    .technical-note = Geprüft werden Inline-<svg> und Elemente mit Grafikrolle darin. Ein Verstoß, wenn das SVG role="img" oder eine andere Grafikrolle trägt, sonst ein Prüfhinweis; SVGs mit aria-hidden oder role="none" und Icons in einem benannten Link oder Button sind ausgenommen.

explanation-area-alt = Fehlender Alternativtext bei Image-Map-Bereichen
    .description = Ein aktiver <area>-Bereich einer Image-Map hat keinen Alternativtext.
    .user-impact = Screenreader-Nutzer erfahren nicht, wohin ein anklickbarer Bildbereich führt.
    .typical-cause = Ein <area href> einer Image-Map ohne alt, oft in generiertem oder älterem Markup.
    .recommendation = Ein alt-Attribut auf dem <area>-Element ergänzen, das das Ziel beschreibt.
    .technical-note = Geprüft werden <area>-Elemente mit href. Ein nicht leeres alt, aria-label oder aria-labelledby benennt den Bereich; ein leeres alt tut es nicht.

explanation-input-image-alt = Fehlender Alternativtext bei Bild-Buttons
    .description = Ein <input type="image">-Button (z. B. eine Bild-Absende-Schaltfläche) hat keinen Alternativtext.
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Aktion der Button auslöst.
    .typical-cause = Eine grafische Absende-Schaltfläche (<input type="image">), deren Alternativtext fehlt.
    .recommendation = Ein alt-Attribut auf dem <input type="image">-Element ergänzen.
    .technical-note = alt, title, aria-label oder aria-labelledby benennen die Schaltfläche. Ohne sie greift der Browser auf einen Standardnamen wie „Submit Query“ zurück, der nichts über die Aktion sagt.

explanation-object-alt = Fehlende Textalternative bei eingebetteten Objekten
    .description = Ein <object>-Element (eingebettetes Bild/Dokument) hat keine Textalternative.
    .user-impact = Screenreader-Nutzer erhalten keine Information über den Inhalt des eingebetteten Objekts.
    .typical-cause = Ein eingebettetes PDF oder Plug-in-<object> ohne Ersatzinhalt.
    .recommendation = Eine Textalternative innerhalb des <object>-Elements oder via aria-label bereitstellen.
    .technical-note = Ein <object> gilt als benannt mit Text zwischen seinen Tags, einem title oder aria-label bzw. aria-labelledby; role="none" nimmt es aus. <embed> wird nicht geprüft.

explanation-form-field-group = Fehlende Gruppierung verwandter Formularfelder
    .description = Zusammengehörige Radio-Buttons oder Checkboxen sind nicht mit <fieldset>/<legend> oder role="group" gruppiert.
    .user-impact = Screenreader-Nutzer erkennen nicht, dass mehrere Felder zu einer Frage/Gruppe gehören.
    .typical-cause = Die Optionsfelder oder Kontrollkästchen einer Frage stehen als einzelne Felder ohne gemeinsamen Container.
    .recommendation = Zusammengehörige Radio-Buttons oder Checkboxen in ein <fieldset> mit <legend> einbetten, oder role="group" mit aria-labelledby verwenden.
    .technical-note = Geprüft werden Optionsfelder und Kontrollkästchen, die im Formular denselben name tragen. Ein <fieldset>, <details> oder role="group"/"radiogroup" darüber gilt als Gruppierung; ob die Gruppe auch einen Namen hat, prüft eine eigene Regel. Ein Prüfhinweis.

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
    .typical-cause = Ein <header>, das je Seitenabschnitt außerhalb von article und section wiederholt wird, oder ein zweites role="banner" aus einer eingebetteten Vorlage.
    .recommendation = Sicherstellen, dass die Seite höchstens ein <header>-/role="banner"-Element hat.
    .technical-note = Gezählt werden banner-Landmarks: ein <header> außerhalb von article, aside, main, nav und section sowie role="banner". Der Befund zeigt auf den zweiten.

explanation-landmark-no-duplicate-contentinfo = Mehrere Contentinfo-Landmarks
    .description = Die Seite hat mehr als einen Fußbereich (<footer>/role="contentinfo"); erlaubt ist höchstens einer.
    .user-impact = Screenreader-Nutzer können den 'Fußbereich' der Seite nicht eindeutig zuordnen.
    .typical-cause = Ein <footer>, das je Seitenabschnitt außerhalb von article und section wiederholt wird, oder ein zweites role="contentinfo" aus einer eingebetteten Vorlage.
    .recommendation = Sicherstellen, dass die Seite höchstens ein <footer>-/role="contentinfo"-Element hat.
    .technical-note = Gezählt werden contentinfo-Landmarks: ein <footer> außerhalb von article, aside, main, nav und section sowie role="contentinfo". Der Befund zeigt auf den zweiten.

explanation-landmark-no-duplicate-main = Mehrere Main-Landmarks
    .description = Die Seite hat mehr als einen Hauptinhaltsbereich (<main>/role="main"); erlaubt ist genau einer.
    .user-impact = Screenreader-Nutzer, die zum Hauptinhalt springen, wissen nicht, welcher Bereich gemeint ist.
    .typical-cause = Ein <main> in der Seitenvorlage und ein zweites in einer eingebetteten Komponente oder einem Inhaltsblock.
    .recommendation = Sicherstellen, dass die Seite genau ein <main>-/role="main"-Element hat.
    .technical-note = Gezählt werden main-Landmarks: <main> und role="main". Der Befund zeigt auf den zweiten.

explanation-landmark-banner-present = Fehlender Banner-Landmark
    .description = Die Seite hat keinen Banner-Bereich (<header> bzw. role="banner").
    .user-impact = Assistierende Technologien können den Kopfbereich der Seite nicht identifizieren.
    .typical-cause = Der Seitenkopf ist ein <div>, oder das <header> steht in <main>, <article> oder <section>, wo es nicht als Seitenkopf zählt.
    .recommendation = Ein <header>-Element auf oberster Ebene (oder ein Element mit role="banner") ergänzen, das den Seitenkopf umschließt.
    .technical-note = Nur ein <header> außerhalb von article, aside, main, nav und section oder role="banner" zählt. Ein Prüfhinweis: Eine Seite kann auch ohne Kopfbereich auskommen.

explanation-label-title-only = Formularfeld-Beschriftung nur über title-Attribut
    .description = Ein Formularfeld bezieht seinen zugänglichen Namen ausschließlich aus dem title-Attribut statt aus einem sichtbaren Label.
    .user-impact = Das title-Attribut wird von vielen Screenreadern inkonsistent oder gar nicht vorgelesen; sehende Nutzer sehen es nur bei Hover.
    .typical-cause = Such- oder Filterfelder, deren einziger Name ein title-Tooltip ist.
    .recommendation = Ein sichtbares <label>-Element oder aria-label statt des title-Attributs verwenden.
    .technical-note = Gilt für Textfelder, Comboboxen, Listboxen, Schieberegler und Zahlenfelder, deren Name nur aus title stammt; ein Feld mit Platzhalter ist eine andere Regel. Ein Prüfhinweis.

explanation-click-events-have-key-events = Klick-Handler ohne Tastatur-Äquivalent
    .description = Ein nicht-interaktives Element (z. B. <div>) hat einen Klick-Handler, aber keinen entsprechenden Tastatur-Event-Handler.
    .user-impact = Tastaturnutzer können die Funktion nicht auslösen.
    .typical-cause = onclick auf einem <div>/<span> ohne onkeydown/onkeyup, tabindex oder passende Rolle.
    .recommendation = Einen keydown/keyup-Handler ergänzen, der auf Enter/Leertaste reagiert, tabindex="0" und eine passende ARIA-Rolle setzen, oder stattdessen ein natives <button>-Element verwenden.
    .technical-note = onclick auf einem <div>/<span> ohne onkeydown/onkeyup, tabindex oder passende Rolle.

explanation-landmark-one-main = Kein Hauptinhaltsbereich als Sprungziel
    .description = Die Seite hat keinen <main>-Bereich, der als Sprungziel zum Überspringen wiederkehrender Blöcke dienen könnte.
    .user-impact = Tastaturnutzer haben kein verlässliches Ziel, um wiederkehrende Navigation zu überspringen.
    .typical-cause = Der Inhalt steht in einem <div id="content"> statt in einem <main>.
    .recommendation = Ein <main>-Element oder ein Element mit role="main" ergänzen, damit Tastaturnutzer ein verlässliches Sprungziel haben.
    .technical-note = Verlangt ein <main> oder role="main" auf einer Seite mit Inhalt. Bei einem zum Messzeitpunkt offenen modalen Dialog ist die Schwere gering, weil der Dialog den main-Landmark verdecken kann.

explanation-skip-link = Fehlender Skip-Link
    .description = Die Seite hat eine Navigation, aber keinen Skip-Link, um sie zu überspringen und direkt zum Inhalt zu springen.
    .user-impact = Tastaturnutzer müssen bei jedem Seitenwechsel durch die gesamte Navigation tabben.
    .typical-cause = Eine Seite mit langem Kopf und Menü, aber ohne Link zum Inhalt, oder einer, den erst ein Skript nach dem Laden ergänzt.
    .recommendation = Einen visuell versteckten oder sichtbaren Link am Seitenanfang ergänzen, der auf den Hauptinhalt verweist, z. B. <a href="#main">Zum Inhalt springen</a>.
    .technical-note = Ein Skip-Link wird erkannt an einem Fragmentlink auf den Anfang des Hauptinhalts, einer Folge von Seitenlinks vor dem ersten echten Link oder einem Fragmentlink, dessen Text oder class nach „Überspringen“ klingt. Ein Prüfhinweis: Auch ein main-Landmark oder Überschriften können die Blöcke umgehen.

explanation-focus-visible-outline-none = Fokus-Indikator per CSS entfernt
    .description = CSS entfernt den sichtbaren Fokusrahmen (outline: none/0) ohne einen gleichwertigen Ersatz bereitzustellen.
    .user-impact = Tastaturnutzer sehen nicht, welches Element gerade fokussiert ist.
    .typical-cause = Ein CSS-Reset mit outline: none oder outline: 0 an :focus für eine „aufgeräumtere“ Optik.
    .recommendation = Den Standard-Fokusrahmen (outline: none/0) nicht entfernen, ohne einen gleichwertigen sichtbaren Ersatz bereitzustellen, z. B. einen eigenen Outline/Box-Shadow auf :focus-visible.
    .technical-note = Stylesheet-Prüfung: Der Rahmen wird unter :focus oder :focus-visible entfernt und keine Regel der Seite ersetzt ihn (eine :focus-visible-Regel, die den Rahmen behält, oder eine sichtbare Gestaltung wie box-shadow an :focus). Ein Prüfhinweis, wenn onfocus-Handler einen anderen Hinweis zeigen könnten.

explanation-scrollable-region-focusable = Scrollbarer Bereich per Tastatur nicht erreichbar
    .description = Ein Bereich mit eigenem Scrollbalken lässt sich nur mit Maus oder Touch scrollen: Er ist nicht fokussierbar und enthält nichts, das den Fokus hineinholt.
    .user-impact = Tastaturnutzer erreichen den Inhalt unterhalb des sichtbaren Ausschnitts nicht.
    .typical-cause = Feste Höhe mit overflow: auto an Menü-, Text- oder Tabellencontainern, die nur Text enthalten.
    .recommendation = Den Bereich mit tabindex="0" fokussierbar machen und ihm einen Namen geben (role="region" mit aria-label), oder die feste Höhe entfernen.
    .technical-note = Wie axe-core scrollable-region-focusable: Überlauf von mehr als 13 px bei overflow: auto/scroll, sichtbarer Textinhalt, weder das Element noch ein Nachfahre per Tastatur fokussierbar. Chrome macht solche Bereiche seit Version 130 selbst fokussierbar, andere Browser nicht.

explanation-input-error-message = Ungültiges Feld ohne zugängliche Fehlerbeschreibung
    .description = Ein als ungültig markiertes Formularfeld hat keine zugängliche Beschreibung des Fehlers.
    .user-impact = Screenreader-Nutzer erfahren, dass ein Fehler vorliegt, aber nicht welcher.
    .typical-cause = Ein Skript setzt aria-invalid, zeigt den Fehlertext aber an anderer Stelle, ohne ihn mit dem Feld zu verknüpfen.
    .recommendation = aria-describedby auf ein Element mit der Fehlermeldung setzen, oder aria-errormessage verwenden.
    .technical-note = aria-invalid mit einem beliebigen Wert außer false, ohne aria-describedby oder aria-errormessage mit Text. Ein Prüfhinweis: Eine sichtbare Fehlermeldung neben dem Feld kann als Beschreibung dienen.

explanation-duplicate-id-aria = ARIA-Referenz zeigt auf mehrdeutige ID
    .description = Ein ARIA-Attribut wie aria-owns/aria-describedby/aria-labelledby verweist auf eine ID, die im Dokument mehrfach vergeben ist.
    .user-impact = Assistierende Technologien können nicht zuverlässig bestimmen, welches Element gemeint ist.
    .typical-cause = Referenzierte ID kommt im DOM mehr als einmal vor.
    .recommendation = Sicherstellen, dass die von aria-owns/aria-describedby/aria-labelledby referenzierte ID im Dokument eindeutig ist, oder ein anderes, eindeutig identifiziertes Element referenzieren.
    .technical-note = Referenzierte ID kommt im DOM mehr als einmal vor.

explanation-aria-label = Interaktives Element ohne aussagekräftigen Namen
    .description = Ein interaktives Element hat keinen zugänglichen Namen, oder der Name besteht nur aus einem Icon/Symbol ohne Beschreibung.
    .user-impact = Screenreader-Nutzer erfahren nicht, wozu das Element dient.
    .typical-cause = Reine Icon-Bedienelemente oder eigene Widgets ohne Beschriftung, oder ein Symbol wie × oder ☰ als einziger Name.
    .recommendation = aria-label, aria-labelledby oder sichtbaren Textinhalt ergänzen, der den Zweck des Elements beschreibt (nicht nur sein Icon).
    .technical-note = Deckt zwei Prüfungen ab: Eine Rolle, die einen Namen verlangt, hat keinen, und ein Name, der aus einem einzelnen Symbolzeichen aus aria-label, aria-labelledby oder title besteht (ein Prüfhinweis).

explanation-title-only-description = Natives title-Attribut als einzige Beschreibung
    .description = Ein interaktives Element wird nur über das native title-Attribut beschrieben. Das ist Best Practice, kein Verstoß gegen WCAG 4.1.2: title zählt als zugänglicher Name.
    .user-impact = Der Tooltip erscheint nur bei Mauszeiger-Hover; per Tastatur und auf Touchgeräten bleibt der Zweck des Elements unsichtbar.
    .typical-cause = Icon-Buttons, Logo-Links oder Suchfelder ohne sichtbaren Text, bei denen title als Beschriftung dient.
    .recommendation = Eine sichtbare Beschriftung ergänzen oder den Namen über aria-label/aria-labelledby bzw. ein <label> setzen; title nur für ergänzende Hinweise verwenden.
    .technical-note = Best Practice, verankert an WCAG 4.1.2. Nicht 1.4.13: Der Browser-Tooltip des title-Attributs wird vom User Agent gesteuert und ist dort ausgenommen. Geprüft werden button, a[href] und input mit title, ohne Textinhalt, aria-label und aria-labelledby.

explanation-aria-required-children = Erforderliche Kindelemente einer ARIA-Rolle fehlen
    .description = Ein Element mit einer Composite-ARIA-Rolle (z. B. tablist) fehlt eines der laut ARIA-Spezifikation erforderlichen Kindelemente (z. B. zugehörige tabpanels).
    .user-impact = Assistierende Technologien können die Beziehung zwischen den Widget-Teilen nicht auswerten.
    .typical-cause = z. B. role="tablist" ohne zugehörige role="tabpanel"-Elemente.
    .recommendation = Die laut ARIA-Rolle erforderlichen Kindelemente ergänzen, z. B. role="tabpanel"-Elemente für jeden Tab in einem role="tablist".
    .technical-note = z. B. role="tablist" ohne zugehörige role="tabpanel"-Elemente.

explanation-dialog-name = Dialog ohne zugänglichen Namen
    .description = Ein Dialog (role="dialog"/"alertdialog") hat keinen zugänglichen Namen.
    .user-impact = Screenreader-Nutzer erfahren beim Öffnen des Dialogs nicht, worum es geht.
    .typical-cause = Ein eigener Dialog aus einem <div role="dialog"> ohne aria-labelledby oder aria-label.
    .recommendation = aria-labelledby auf den sichtbaren Dialogtitel setzen, oder aria-label verwenden.
    .technical-note = Geprüft werden offene <dialog>-Elemente und Elemente mit role="dialog" oder "alertdialog" auf einen Namen aus aria-label, aria-labelledby oder title; ein geschlossener <dialog> steht nicht im Accessibility-Tree und wird übersprungen.

explanation-control-missing-label = Formularelement, Link oder Button ohne zugänglichen Namen
    .description = Ein Formularfeld, Link oder Button hat keinen zugänglichen Namen (weder sichtbares Label noch aria-label/aria-labelledby).
    .user-impact = Screenreader-Nutzer erfahren nicht, welche Eingabe erwartet wird oder wohin ein Link/Button führt.
    .typical-cause = Icon-Buttons und Bild-Links ohne Text, alt oder aria-label.
    .recommendation = Ein <label>-Element mit passendem for-Attribut (Formularfelder) bzw. Textinhalt (Links/Buttons) ergänzen, oder aria-label/aria-labelledby verwenden.
    .technical-note = Geprüft werden <button>, role="button" (nicht <summary>) und <a href> auf einen zugänglichen Namen aus Inhalt, Bild-alt, aria-label, aria-labelledby oder title.

explanation-link-as-button = Link als Button-Ersatz ohne Tastenrolle
    .description = Ein Link (<a>) mit href="#" oder javascript:-Pseudo-URL und onclick-Handler wird wie ein Button verwendet, ohne role="button" zu tragen.
    .user-impact = Assistierende Technologien kündigen das Element als Link an, obwohl es sich wie ein Button verhält — Tastaturnutzer erwarten das falsche Bedienverhalten.
    .typical-cause = Ein Link mit href="#" oder einer javascript:-Adresse, der beim Klick ein Skript ausführt.
    .recommendation = role="button" auf den Link setzen (plus Tastaturbehandlung für die Leertaste), oder stattdessen ein echtes <button>-Element verwenden.
    .technical-note = Ein <a> mit onclick und einem href, das nirgendwohin führt (leer, „#“ oder javascript:), ohne role="button". Ein Prüfhinweis: Der Handler kann doch navigieren.

explanation-redundant-role = Redundante ARIA-Rolle
    .description = Ein Element trägt ein role-Attribut, das exakt der bereits impliziten Rolle des HTML-Elements entspricht (z. B. <button role="button">).
    .user-impact = Kein unmittelbarer Nutzerschaden, aber unnötiges Risiko für Inkonsistenzen bei zukünftigen Änderungen.
    .typical-cause = role="button" an einem <button>, role="link" an einem <a href> oder role="list" an einer Liste mit sichtbaren Listenzeichen, oft aus Gewohnheit ergänzt.
    .recommendation = Das redundante role-Attribut entfernen; das native HTML-Element bringt diese Rolle bereits implizit mit.
    .technical-note = Verglichen wird mit der impliziten Rolle von <button>, <a href>, <textarea>, Text-, Checkbox- und Radio-Eingaben, Überschriften, <li>, <table> und <img alt>. role="list" an einer Liste mit list-style-type none ist nicht redundant (es stellt die Listensemantik in Safari und VoiceOver wieder her) und wird nicht gemeldet.

explanation-summary-name = Summary-Element ohne zugänglichen Namen
    .description = Ein <summary>-Element (Aufklapp-Auslöser in <details>) hat keinen zugänglichen Namen.
    .user-impact = Screenreader-Nutzer erfahren nicht, was der Aufklapp-Bereich enthält, bevor sie ihn öffnen.
    .typical-cause = Ein <summary>, das nur ein Icon enthält.
    .recommendation = Textinhalt im <summary>-Element ergänzen, der beschreibt, was der Aufklapp-Bereich enthält.
    .technical-note = Geprüft wird das <summary>, das sein <details> bedient (das erste); sein Name stammt aus Text, aria-label, aria-labelledby oder title.

## Missing navigation and contentinfo landmark

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

