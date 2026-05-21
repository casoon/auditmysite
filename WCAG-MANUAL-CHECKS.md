# WCAG Rules: Nicht automatisierbar

Diese Regeln können durch statische DOM-/AXTree-Analyse oder CDP-Evaluation
nicht zuverlässig geprüft werden. Sie erfordern manuelle Inspektion oder
spezialisierte Werkzeuge (OCR, Video-Analyse, Verhaltensbeobachtung).

Kein bekanntes automatisches Tool (axe-core, Pa11y, Lighthouse) implementiert
diese Kriterien vollständig.

---

## 1.4.9 Images of Text (No Exception) — Level AAA

**Kriterium:** Text darf nur in Bildern vorkommen, wenn er rein dekorativ ist
oder eine bestimmte visuelle Darstellung für die Information essenziell ist.

**Warum nicht automatisierbar:** Erfordert OCR / Computer Vision, um Text in
Bildern zu erkennen. Heuristiken über Bilddimensionen oder Dateinamen liefern
zu viele Fehlalarme.

**Manuelle Prüfung:** Alle `<img>`, `<canvas>` und CSS-Hintergrundbilder
visuell prüfen; `<img>` mit `role="img"` und texthaltiger Beschriftung sind
Kandidaten.

---

## 2.3.2 Three Flashes (No Exception) — Level AAA

**Kriterium:** Inhalte dürfen nicht mehr als dreimal pro Sekunde aufblitzen,
ohne Ausnahme für den "sicheren Bereich" (wie in 2.3.1 Level A).

**Warum nicht automatisierbar:** Erfordert frame-genaue visuelle
Rendering-Analyse (Video-Capture mit ≥ 60 fps). Selbst das schwächere 2.3.1
(Level A) wird von nahezu keinem Tool geprüft. CSS-Animations-Dauer ist kein
Proxy für die tatsächliche Blitzrate am Bildschirm.

**Manuelle Prüfung:** Animationen und Videos mit einem
Photosensitivity-Analysetool wie PEAT (Photosensitive Epilepsy Analysis Tool)
prüfen.

---

## 3.2.5 Change on Request — Level AAA

**Kriterium:** Kontextänderungen werden nur auf ausdrückliche Nutzeranforderung
ausgelöst; kein automatisches Weiterleiten, kein Auto-Advance.

**Warum nicht automatisierbar:** Erfordert Verhaltensbeobachtung über Zeit
(z. B. ob eine Slideshow selbstständig weiterscrollt oder ob `onchange` eine
Navigation auslöst). `<meta http-equiv="refresh">` ist erkennbar, aber das
Kriterium ist breiter. Interaktionssequenzen können nicht statisch beurteilt
werden.

**Manuelle Prüfung:** Seite ohne Interaktion beobachten (automatische
Navigation?), alle Formularfelder auf unerwartete Kontextänderungen bei
`onchange`/`onblur` testen.

---

## 3.3.6 Error Prevention (All) — Level AAA

**Kriterium:** Für alle Seiten mit Formulareingaben muss der Nutzer Eingaben
revidieren, bestätigen oder korrigieren können — nicht nur bei
rechtlichen/finanziellen Transaktionen (das ist Level AA 3.3.4).

**Warum nicht automatisierbar:** Ob ein Bestätigungsschritt existiert, ist aus
dem DOM allein nicht ableitbar. Erfordert echte Formular-Submission-Simulation
mit Zustandsverfolgung über mehrere Schritte.

**Manuelle Prüfung:** Alle Formulare bis zum Abschluss durchlaufen; prüfen ob
eine Zusammenfassung, ein Bestätigungsdialog oder eine Korrekturmöglichkeit
angeboten wird.
