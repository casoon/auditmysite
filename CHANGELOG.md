# Changelog

Detailed, chronological development history for auditmysite — each entry documents a finding,
the fix, and how it was verified. Extracted from `CLAUDE.md`'s former "Current State" section
(plan/11-claude-md-version-drift.md) so `CLAUDE.md` itself stays focused on working rules and a
short current-state summary. Newest entries first (unchanged order from before the extraction).

- **Unreleased — Regelfilter gilt fuer die geteilten Regeln (#698):** `[rules] disabled` /
  `enabled_only` und `--disable-rule` griffen bisher nur fuer die lokalen Regeln; seit Kontrast
  geteilt ist, schaltete `--disable-rule color-contrast` nichts mehr ab. Jetzt nimmt
  `wcag::shared::retain_allowed` eine geteilte Kennung heraus, wenn der Filter sie unter ihrem
  eigenen Namen oder einer alten Kennung aus `LEGACY_RULE_IDS` abschaltet (`color-contrast` →
  `contrast/text-*`, `link-in-text-block`, `landmark-one-main`, …, nur vollstaendig abgeloeste
  Regeln), und vermerkt sie als `disabled_by_rule_filter`; ebenso in iframes.

- **Unreleased — Kontrast ueber Bildern wieder per Abtastung (#698, barrierlab#47/#48):**
  `RenderedCdpDocument` liefert `Rendering::sampled_backdrop`: ein Bildschirmfoto des sichtbaren
  Ausschnitts, einmal in die Seite geladen, je Element die WCAG-Leuchtdichten der Pixel im Kasten
  (Alpha gegen Weiss, hoechstens 2500 je Element auf einem Raster). Abgetastet werden hoechstens 60
  Elemente, die groessten zuerst: eigener Text im Hauptdokument, Textfarbe bekannt, Hintergrund
  nicht bestimmbar, weder optisch verborgen noch ueberdeckt, ganz im visuellen Ausschnitt
  (`visualViewport` — in der mobilen Emulation ist er schmaler als `innerWidth`). Scheitert das
  Bildschirmfoto, bleibt es bei `UNTESTED`. Die geteilte Regel urteilt darueber wie die frueher
  lokale (Median, 40. Perzentil): `FAIL`, bestanden oder Pruefhinweis. Die Integrationstests
  `test_image_contrast_pixel_sampling` und `test_opacity_overlay_contrast_pixel_sampling` laufen
  wieder unveraendert gegen ihre alten Urteile. Die PDF-Zeile „Kontrast X (erforderlich Y)" liest
  die Messwerte der geteilten Regel (`contrast_ratio`, `required_ratio`, ohne `:1` geliefert).
  Abgetastet gesehen (2026-10-05, Desktop, ohne `--dismiss-consent`): berlin.de 8; gov.uk,
  bundesregierung.de, wetter.com, n-tv.de, spiegel.de 0 — dort liegt im sichtbaren Ausschnitt
  kein Text ohne bestimmbaren Hintergrund, oder ein Consent-Overlay deckt ihn ab
  (wetter.com 199, n-tv.de 79, spiegel.de 38 ueberdeckte Elemente).

- **Unreleased — Kontrast aus `a11y-rules` (#698, barrierlab#47):** `contrast/text-insufficient`
  (1.4.3 AA), `contrast/text-undetermined` (1.4.3, `UNTESTED`) und `contrast/text-enhanced`
  (1.4.6 AAA, nur mit `--level aaa`; meldet nur, was 1.4.3 besteht) ersetzen `color-contrast`.
  Geloescht: `wcag::rules::contrast` samt Seitenregel und Pixelvergleich mit dem Bildschirmfoto,
  `accessibility::styles` (`extract_text_styles`); `Color` liegt jetzt in
  `non_text_contrast_css` (bleibt `wcag::rules::Color`). Die alte Kennung loest ueber die
  Taxonomie weiter auf (`a11y.contrast.weak`, `axe_id: color-contrast`); 1.4.6 hat einen eigenen
  AAA-Eintrag `a11y.contrast_enhanced.weak` mit AAA-Gewicht (vorher zaehlten AAA-Verfehlungen als
  1.4.3 mit dem vollen Abzug). Dunkelmodus und Farbsehschwaeche-Ansicht zaehlen die geteilten
  Verstoesse ueber einen frischen DOMSnapshot; der Vergleich hell/dunkel paart Elemente ueber die
  Backend-ID. Gewollt anders: Text unter einem fixierten oder klebenden Element (Cookie-Banner)
  ist `UNTESTED` (#716 Fall 7); Text unter `aria-hidden` wird gemessen (#395 nahm ihn aus).
  Vergleich geteilt gegen lokal (2026-10-05): Korpus (`contrast_shadow_and_inline`,
  `gradient_text_contrast`, `low_contrast_text`, `text_fill_color_contrast`) deckungsgleich, AA
  und AAA; gov.uk deckungsgleich; bundesregierung.de AA gleich, AAA +4 (Metanavigation, lokal als
  verborgen uebersprungen: absolut positionierte Liste in 0 px breitem `nav` mit
  `overflow: hidden`); wetter.com ohne Consent-Overlay AA gleich (15), mit Overlay die zehn
  Navigationspunkte `UNTESTED` statt 2,37:1, AAA zusaetzlich der Consent-Dialog (lokal als
  verborgen uebersprungen) und Teaser-Beschriftungen mit eigener deckender Flaeche (lokal
  Pruefhinweis); n-tv.de AA beide ohne Verstoss, AAA 25 lokale Verstoesse unter dem
  Consent-Overlay jetzt `UNTESTED`; spiegel.de AA +19 echte Verstoesse (4,04:1, Weiss auf
  #e64415 und umgekehrt), die die lokale Regel uebersah (Klassenname `overflow-hidden` galt ihr als
  verborgen), AAA entsprechend mehr. Korpus-Erwartungen auf die neuen Kennungen umgestellt.
  Live gegen `main` (2026-10-05, mit Abtastung), Barrierefreiheit AA / AAA vorher → nachher:
  gov.uk 93 → 93 / 70 → 89, bundesregierung.de 49 → 49 / 46 → 49, wetter.com 22 → 22 / 18 → 20,
  n-tv.de 36 → 36 / 23 → 29, spiegel.de 28 → 24 / 20 → 21 (AA-Verstoesse vorher → nachher:
  0 → 0, 0 → 0, 15 → 12, 25 → 23, 3 → 22; die Abnahmen sind Text unter dem Consent-Overlay, jetzt
  `UNTESTED`). AAA steigt, weil 1.4.6 jetzt mit AAA-Gewicht zaehlt. Referenzseite berlin.de
  45 → 49 (drei Laeufe: 46, 49, 49), knapp ueber dem Band 15–48: Die lokale Regel meldete Desktop 11 /
  mobil 12 Verstoesse, ueberwiegend Bildnachweise und Teaser-Titel ueber Fotos
  (`p.image__copyright`, `a.title`), gemessen gegen eine Vorfahrenfarbe (2,56:1 bzw. 1,00:1). Jetzt
  im sichtbaren Ausschnitt abgetastet (bestanden, oder `a.title` 4,02:1 `FAIL`), unterhalb des
  Ausschnitts `UNTESTED` (6 je Durchgang); es bleiben 1 / 3 Verstoesse. Das Band ist neu zu
  bewerten.

- **Unreleased — Optisch verborgener und ueberdeckter Text, Hintergrund ueber Bildern (Teil von
  #698, Host-Seite von barrierlab#47):** `RenderedCdpDocument` liefert
  `Rendering::visually_hidden` aus dem `DOMSnapshot` (neue Stilwerte `clip`, `clip-path`,
  `text-indent`, `position`, `opacity`): `opacity: 0` am Element oder einem Vorfahren,
  `clip: rect(≤ 1px …)` an einem absolut positionierten Kasten, `clip-path: inset(≥ 50 %)`,
  `text-indent` ab 999 px, ein Kasten von hoechstens 1 px mit `overflow: hidden|clip` (am Element
  oder bis zwoelf Ebenen darueber; ein absolut positionierter Nachfahre entkommt einem statischen,
  ein fixierter jedem Kasten) und ein Rahmen ganz links oder oberhalb des Dokuments. Von `Layout`
  fuellt das Dokument nur `obscured`, an Elementen mit eigenem Text im Hauptdokument: ein
  Skriptdurchgang vor dem Snapshot prueft per `elementFromPoint` an der Mitte, ob ein fremdes
  fixiertes oder klebendes Element den Text verdeckt; nur die Treffer werden per `describeNode`
  auf Backend-IDs abgebildet. Die effektive Hintergrundfarbe ist `None`, wenn ein malendes
  Geschwister des Elements oder eines Vorfahren (bis sechs Ebenen, bis zur ersten deckenden
  Flaeche) den Kasten ueberschneidet und eines von beiden absolut oder fest positioniert ist
  (#716 Fall 6: wetter.com `h4.newsCarousel__headline`, spiegel.de `figcaption > p`).
  Gefuellt gesehen (DoD 4, 2026-10-05, `visually_hidden` gemessen / wahr, `obscured` gemessen /
  wahr): gov.uk 444/15, 132/0 (darunter `button.gem-c-search__submit`); bundesregierung.de
  6706/5634 (fast alles SVG-Sprite-Pfade in einem 0-px-`<svg>`), 232/0; wetter.com 1897/569
  (Hover-Menues mit `opacity: 0`), 577/199 mit Consent-Overlay bzw. 0 ohne; n-tv.de 3832/16,
  1097/79; spiegel.de 6793/135, 1027/36.

- **Unreleased — Statische Darstellungsregeln aus `a11y-rules` 0.19.1 (#699):** Die Regeln der
  Darstellungskonvention, die nur das Markup brauchen, kommen jetzt aus dem geteilten Bestand
  (barrierlab#22): `display/toggle-missing` (ersetzt die lokale Regel gleichen Namens, derselbe
  Text) und die neuen `viz/text-missing`, `viz/caption-missing`, `viz/static-missing`,
  `viz/table-missing` (Letzteres ein Pruefhinweis). Die neuen Kennungen haben eigene
  Taxonomie-Eintraege (`a11y.display_caption.missing`, `a11y.display_static.missing`,
  `a11y.display_table.review`; `viz/text-missing` teilt `a11y.display_text_layer.missing`),
  Erklaerungen und Score-Bereiche; wie `display/*` gelten sie als Konventionsregeln, nicht als
  Rechtshinweis. In `display_modes` bleibt, was nur die laufende Seite zeigt:
  `display/init-missing` (Beobachter vor der Navigation), `display/text-hidden` (samt berechneter
  Sichtbarkeit und der ausgeblendeten Figur, #725), `display/text-media-visible` und
  `display/text-not-visible`. Diese beiden Kennungen gibt es auch in `a11y-rules`, aber nur als
  statischer Teil; auditmysite uebernimmt sie nicht, damit eine Kennung nie zwei Quellen hat.
  Gewollt anders: `display/text-not-visible` meldet im Textmodus nur noch eine Textschicht, die
  Inhalt hat, aber nicht erscheint; fehlt sie oder ist leer, ist das `viz/text-missing` und gilt
  in jedem Modus, nicht nur im Textmodus. Neue Corpus-Fixture `display_modes_viz_static`, die
  Erwartung in `display_modes_violations` zieht auf `viz/text-missing` um. Geprueft: Unit-Tests,
  Detection-Corpus; Gegenprobe an geographia.eu (Referenzumsetzung, z. B. `climate-monitor` mit
  26 Figuren): alte und neue Fassung melden dort nichts, kein Fehlalarm auf konformen Seiten.

- **Unreleased — Rendering-Schicht aus dem DOMSnapshot, drei Darstellungsregeln aus `a11y-rules`
  (Teil von #698, Host-Seite von barrierlab#21 zweiter Teil; `a11y-*` 0.20.0):**
  `RenderedCdpDocument` fuellt `ComputedStyle` jetzt vollstaendig aus demselben
  `DOMSnapshot.captureSnapshot`: Text- und effektive Hintergrundfarbe (ueber die Vorfahren im
  flachen Baum verrechnet, zuunterst weiss; `None` bei Hintergrundbild, Verlauf oder unlesbarer
  Farbe), Schriftgroesse, -gewicht, -schnitt, -familie, `list-style-type`, `text-decoration-line`,
  `border-bottom-style`; durchsichtige Textfuellung (`background-clip: text`, #640) ergibt keine
  Farbe, `display: contents` (etwa `<slot>`) nimmt Farbe und Schrift vom gerenderten Text.
  `bounds()` liefert den Rahmen des Layout-Objekts, `scroll_overflow_px()` den Scroll-Ueberhang
  aus `includeDOMRects` (`scrollRects`/`clientRects`, nur auf Achsen mit `overflow: auto|scroll`).
  Ein `Layout` liefert das Dokument nicht; die Heuristiken darauf bleiben ungelaufen. Gefuellt gesehen
  (DoD 4, 2026-10-05, Elemente mit Layout-Objekt / alle): gov.uk 444/617, bundesregierung.de
  6695/7591, wetter.com 1905/2612, n-tv.de 3636/4789, spiegel.de 6673/8891 — jedes neue Feld auf
  jeder Seite gefuellt. Uebernommen: `redundant-role` (`<ul>`/`<ol>`) → `lists/role-redundant`
  (4.1.2 A), `link-in-text-block` → `color/link-indistinct` (1.4.1 A; zusaetzlich zaehlen
  Schriftfamilie, Unterkante und Hintergrund als Unterscheidung), `scrollable-region-focusable` →
  `keyboard/scrollable-region-not-focusable` (2.1.1 A). Geloescht: `redundant_role`,
  `use_of_color`, `scrollable_region`; die alten Kennungen loesen in Taxonomie und Erklaerungen
  weiter auf. Die Farbsehschwaeche-Ansicht zaehlt `color/link-indistinct` einmal statt je Modus
  (die Emulation aendert keine berechneten Stile). Korpus unveraendert erfuellt
  (`redundant_role_list_style`, `link_in_text_block_context`, `scrollable_region_focusable`,
  `misc_content_checks`). Die drei Regeln laufen nur im obersten Dokument wie die abgeloesten.
  **Kontrast bleibt lokal:** `contrast/text-*` deckt sich am Korpus mit
  `contrast` (Shadow DOM, 4,46:1 ungerundet, Float-Container, Verlaufstext undeterminiert), live
  aber fehlen AAA-Schwellen, visuell versteckter Text wird gemessen (gov.uk-Suchknopf,
  `text-indent: -5000px`), und Text ueber positionierten Bildern (#716 Fall 6) wird `FAIL` statt
  `UNTESTED` (wetter.com `newsCarousel__headline`, spiegel.de Bildnachweise). Live gegen `main`
  (2026-10-05), Barrierefreiheit AA / AAA vorher → nachher: gov.uk 93 → 93 / 70 → 70,
  bundesregierung.de 49 → 49 / 46 → 46, wetter.com 22 → 22 / 18 → 18, n-tv.de 36 → 36 / 22 → 22,
  spiegel.de 26 → 26 / 20 → 20; die drei Regeln liefen vorher und nachher ohne Befund, die
  uebrigen Abweichungen (`color-contrast` ±1, `target-size` 8 → 10 auf AAA) sind Seitenwechsel
  zwischen den Laeufen in unveraenderten Regeln.

- **Unreleased — Reduzierte Bewegung aus `a11y-rules` 0.19.1 (Teil von #698):** Seit 0.19.1 loest
  `a11y-rules` die Kurzform `animation` nach CSS Animations auf; Chromes Serialisierung
  (`2s linear 0s infinite normal none running spin`) ergibt wieder den Namen `spin` statt `none`
  (Fehler aus diesem PR, Fix in barrierlab). Damit laeuft `prefers-reduced-motion` →
  `motion/reduced-motion-ignored` (2.3.3 AAA, Hinweis statt Verstoss, eine Meldung je Seite) ueber
  die Stylesheets; `reduced_motion` ist geloescht, die alte Kennung loest in der Taxonomie weiter
  auf. 2.3.3 zaehlt als Hinweis-Kriterium (AAA). Korpus: `media_and_motion`,
  `reduced_motion_transform` und `target_size_animation` melden den Hinweis,
  `reduced_motion_override` und `reduced_motion_colour_only` nicht. Gewollt anders: Die abgeloeste
  Page-Rule lief trotz AAA schon ab `--level aa` (`min_level: AA`); der geteilte Hinweis erscheint
  wie die uebrigen AAA-Kennungen nur mit `--level aaa`. Live gegen `main` (2026-10-04) auf AAA
  gleiche Treffer (bundesregierung.de und n-tv.de je 1, gov.uk, wetter.com, spiegel.de 0); auf AA
  faellt der bisherige 2.3.3-Befund weg, Barrierefreiheit AA / AAA vorher → nachher: gov.uk
  92 → 93 / 67 → 70, bundesregierung.de 49 → 49 / 35 → 46, wetter.com 21 → 22 / 17 → 18, n-tv.de
  22 → 31 / 18 → 22, spiegel.de 25 → 28 / 18 → 20.

- **Unreleased — Regeln ueber Stylesheets aus `a11y-rules` (Host-Seite von #698, barrierlab#21
  erster Teil):** auditmysite liest die Stylesheets der Seite (`document.styleSheets`, je Sheet
  der `cssText` aller Regeln; fremde Sheets ohne lesbare `cssRules` fehlen wie bisher),
  parst sie mit `stylesheet-parse` 0.1.0 (neue direkte Abhaengigkeit) und gibt sie mit dem Dokument
  der geteilten Regeln an `a11y_rules::run_stylesheets_in`
  (`accessibility::fetch_stylesheets`, `wcag::shared::run_shared_rules_with_stylesheets`).
  Scheitert das Lesen, bleiben die Regeln `NotRun::CapabilityMissing`. Frames bekommen keine
  Sheets; die Regeln urteilen ueber die Seite wie die abgeloesten, die nur im obersten Dokument
  liefen. Uebernommen: `focus-visible-outline-none` → `focus/outline-removed` (2.4.7 AA, hoch; ein
  in `:focus` wieder gesetzter Rahmen zaehlt als Ersatz, sueddeutsche.de),
  `css-orientation-lock` (Stylesheet-Teil) → `orientation/content-hidden` (1.3.4 AA, Hinweis, nur
  Selektoren mit Treffer auf der Seite, ohne Pseudo-Elemente), `visual-presentation` (Blocksatz,
  Zeilenabstand) → `text/justified` und `text/line-height-tight` (1.4.8 AAA, Hinweise, gemessen an
  den `<p>` der Seite). Geloescht: `focus_visible_css`; aus `orientation` die Stylesheet-Suche (der
  berechnete `transform: rotate` an `body`/`html` bleibt), aus `visual_presentation` die
  Stylesheet-Suche (der `UNTESTED`-Vermerk zu Farbwahl und Spaltenbreite bleibt). 1.4.8 zaehlt als
  Hinweis-Kriterium (AAA). **Nicht uebernommen wegen eines Fehlers in `a11y-rules` 0.19.0:**
  `motion/reduced-motion-ignored` -- Chrome serialisiert die Kurzform `animation` vollstaendig
  (`2s linear 0s infinite normal none running spin`), und die Regel nimmt `none` aus
  `animation-fill-mode` als Animationsnamen, jede Animation ueber die Kurzform faellt heraus
  (Korpus `media_and_motion`, `target_size_animation`). `reduced_motion` bleibt deshalb vorerst
  hier (mit 0.19.1 behoben, siehe oben). Abweichungen im Changelog von `a11y-rules` 0.19.0. Korpus: `media_and_motion` meldet
  `focus/outline-removed` und `orientation/content-hidden`, `text_and_layout` `text/justified`
  (die enge Zeilenhoehe dort steht an einem `<div>`, nicht an einem `<p>`).
  Live gegen `main` (2026-10-04), Barrierefreiheit AA / AAA vorher → nachher, B4–B7 und
  Stylesheets zusammen: gov.uk 92 → 93 / 67 → 70, bundesregierung.de 49 → 49 / 35 → 43, wetter.com
  21 → 22 / 17 → 18, n-tv.de 22 → 29 / 18 → 21, spiegel.de 25 → 28 / 18 → 20. Entfallen sind die
  Fehlalarme `css-orientation-lock` auf bundesregierung.de und n-tv.de (Breakpoint-Marker
  `body:before`, Schliessknopf) und `visual-presentation` auf spiegel.de (Selektor-Praefix);
  neu auf AAA `text/line-height-tight` auf gov.uk und bundesregierung.de.

- **Unreleased — `<area>` und selbststartender Ton aus `a11y-rules` (#696, Nachtrag):** Mit
  `a11y-rules` 0.19.0 sieht die Sicht `<area>` und `<audio>` ohne `controls` auch mit berechneten
  Stilen (UA `display: none`; gemeldet aus diesem PR, Fix in barrierlab). Damit laufen `area-alt` →
  `images/area-alt-missing` (1.1.1 A; `aria-label`/`aria-labelledby` zaehlen) und
  `background-audio` → `media/audio-autoplay` ueber den geteilten Bestand; `image_input_rules` und
  `background_audio` sind geloescht. `media/audio-autoplay` ist ein Pruefhinweis unter 1.4.2 (A)
  statt Verstoss unter 1.4.7 (AAA) und laeuft damit schon auf AA; Taxonomie
  `a11y.audio_control.missing`, 1.4.2 zaehlt als Hinweis-Kriterium (stand schon in der manuellen
  Liste). Beide laufen auch in Frames. Korpus: `misc_content_checks` und `media_and_visual`
  melden unter den neuen Kennungen.

- **Unreleased — Tabellen-, Dokument-, Sprach- und Rollenregeln aus `a11y-rules` statt eigener
  (#697, B7):** Zwoelf Kennungen laufen ueber den geteilten Bestand: `th-has-data-cells` →
  `tables/header-without-data` (samt `tables/data-undetermined`, `UNTESTED`, fuer einen noch nicht
  dargestellten Zeilenvorrat, #654), `td-headers-attr` → `tables/headers-attr-invalid`,
  `html-xml-lang-mismatch` → `document/lang-mismatch`, `language-of-parts` →
  `language/part-unmarked` (neu `language/part-undetermined` auf Seiten, die weder Deutsch noch
  Englisch sind), `abbreviations` → `language/abbreviation-unexpanded` (Hinweis statt Verstoss),
  `meta-refresh` → `timing/meta-refresh` (0 s ist keine Frist), `heading-order` aus
  `section_headings` → `headings/section-without-heading` (Hinweis, nur Artikel und benannte
  Abschnitte), `redundant-role` → `aria/role-redundant`, `title-only-description` →
  `names/title-only` (Hinweis; Textfelder meldet weiter `forms/title-only-label`),
  `content-on-hover-focus` → `patterns/tooltip-unreferenced`. Geloescht: `table_extended`,
  `language_extended`, `language_of_parts`, `abbreviations`, `content_on_hover`,
  `info_relationships`, `section_headings`, die Baumregeln `th-has-data-cells` und `heading-order`.
  **Ersatzlos gestrichen:** `presentation-semantic-children` -- nach WAI-ARIA 1.2 nimmt
  `role="presentation"`/`"none"` nur dem Element selbst die Semantik, nicht den Nachfahren; die
  Regel haette das APG-Menueleistenmuster (`<li role="none">` um Menuelinks) gemeldet (barrierlab:
  187 Fehlalarme auf 48 Seiten), der Tabellenfall ist `tables/presentational-with-headers`. Aus
  `section_headings` entfallen die Gliederungsluecken (`headings/skip-level`) und „mehr als 10
  Absaetze, weniger als 3 Ueberschriften" (kein Beleg). **Im Host bleiben:** aus
  `redundant_role` nur `<ul>`/`<ol>` mit `role="list"` -- ob die Rolle ueberfluessig ist, haengt am
  berechneten `list-style-type` (#644), den `a11y-dom` noch nicht liefert (barrierlab#21); der
  Korpusfall `ol#numbered-list` meldet weiter. Aus `timing_adjustable` die seitenweiten
  `UNTESTED`-Vermerke fuer Skript-Fristen (2.2.1, jetzt als Page-Rule `2.2.1/timing-adjustable`)
  und `timeouts` (2.2.6), die barrierlab der manuellen Checkliste zuweist. Die
  Darstellungsregeln aus barrierlab#22 (`viz/*`, statische Haelften von `display/*`) sind nicht
  uebernommen: `display/text-hidden`, `display/init-missing` und `display/toggle-missing` tragen in
  `display_modes` dieselben Kennungen, pruefen dort aber auch berechnete Sichtbarkeit und den
  Zeitpunkt von `data-display`; ein Aufteilen derselben Kennung auf zwei Quellen ist ein eigener
  Schritt, `display_modes` bleibt unveraendert. Taxonomie: neu `a11y.language_of_parts.unmarked`
  (3.1.2 AA; `language-of-parts` hatte keinen Eintrag); 3.1.2, 2.4.10 und 3.1.4 zaehlen als reine
  Hinweis-Kriterien, 3.1.2 steht damit in der Liste der manuell zu pruefenden Kriterien (21 → 22,
  `docs/PARITY_CONTRACT.jsonc`, README). Tabellen-, Rollen-, Namens- und Tooltip-Regeln sowie
  Abkuerzungen laufen auch in Frames. Geprueft: Detection-Corpus vorher/nachher (Umbenennungen,
  Hinweise statt Verstoesse wie im barrierlab-Changelog, `language/part-undetermined` auf
  `invalid_lang_code` und `skip_link_language`, neue Abschnittshinweise auf fuenf Faellen).
  Pruefung B4–B7 gesamt gegen `main` (Detection-Corpus 76 Faelle auf AAA: elf Barrierefreiheits-
  Scores steigen um 1–9 Punkte, keiner faellt; Live-Seiten auf AA, Barrierefreiheit vorher →
  nachher: gov.uk 92 → 93, bundesregierung.de 49 → 49, wetter.com 21 → 22, n-tv.de 22 → 25,
  spiegel.de 25 → 28). Neu auf echten Seiten: `names/title-only` (spiegel.de 132 Hinweise statt
  5 Befunden), `frames/name-missing` (n-tv.de 6 statt 2, darunter die drei Sportdaten-Rahmen mit
  `title=""`), `aria/role-redundant` (wetter.com 9 statt 20 gedeckelte, ausgeblendete Menues
  zaehlen nicht mehr); `navigation/location-missing` und die anderen AAA-Hinweise laufen auf AA
  nicht. Referenzlauf und Score-Kalibrierung bleiben in ihren Baendern.

- **Unreleased — Bild- und Medienregeln aus `a11y-rules` statt eigener (#696, B6):** Ueber den
  geteilten Bestand laufen jetzt `input-image-alt` → `images/input-alt-missing`, `object-alt` →
  `objects/alt-missing`, `server-side-image-map` → `images/server-side-map` (alle 1.1.1 A) und
  `frame-title` → `frames/name-missing` (2.4.1 A, hoch; `a11y-rules` fuehrt zusaetzlich 4.1.2).
  Geloescht: `server_side_image_map`, aus `image_input_rules` die Pruefungen fuer
  `<input type="image">` und `<object>`, aus `media_rules` die Rahmennamen und die Baumregel
  `video-caption` (unbenanntes `application`/`img`, benanntes dekoratives Element -- barrierlab hat
  sie mangels Beleg bzw. wegen `svg/name-missing`, `images/alt-missing` und
  `aria/attribute-prohibited` nicht uebernommen). Gewollt anders: `aria-labelledby` zaehlt als
  Alternative, `<embed>` wird nicht mehr geprueft, nur `<iframe>` (kein `<frame>`), keine
  Obergrenze je Seite; die gerenderte Groesse ≤ 1 px kennt `frames/name-missing` ohne Geometrie nicht
  (nur Breite/Hoehe als Attribut). `frames/name-missing` hat erstmals einen eigenen Erklaerungstext
  (`frame-title` fiel auf den 2.4.1-Text „Fehlende Sprungnavigation" zurueck). Im Host bleiben:
  die Untertitel-Pruefung `video-caption` (laedt die `<track>`-Datei ueber das Netz),
  `frame-tested` (fremde Rahmen kennt nur der Browser) und `media-alt` (1.2.8) -- barrierlab
  verweist dafuer auf `manual/media-alternatives`, die manuelle Checkliste fuehrt auditmysite noch
  nicht. **Nicht uebernommen wegen eines Fehlers in `a11y-rules` 0.18.0:**
  `images/area-alt-missing` und `media/audio-autoplay` melden mit berechneten Stilen nie, weil die
  Sicht alles mit `display: none` herausnimmt und das UA-Stylesheet das jedem `<area>` und jedem
  `<audio>` ohne `controls` gibt (Korpus `misc_content_checks`, `media_and_visual`). `area-alt`
  (`image_input_rules`) und `background-audio` bleiben deshalb vorerst hier. Bild- und
  Rahmenregeln laufen jetzt auch in Frames. Abdeckung: 2.4.8 (`navigation/location-missing`, seit
  B5 nur Hinweis) zaehlt nicht mehr als automatisch geprueftes AAA-Kriterium. Geprueft:
  Detection-Corpus vorher/nachher (nur die umbenannten Kennungen, Selektor jetzt `tag#id`).

- **Unreleased — Links- und Zeigerregeln aus `a11y-rules` statt eigener (#695, B5):** Mit
  `a11y-rules` 0.18.0 laufen drei Kennungen ueber den geteilten Bestand: `click-events-have-key-events`
  → `keyboard/click-handler-not-focusable` (2.1.1 A, hoch), `link-as-button` →
  `links/used-as-button` (4.1.2 A, niedrig), `location` → `navigation/location-missing` (2.4.8 AAA,
  niedrig). Geloescht: `click_handlers`, `fake_navigation_link`, `location` samt ihren
  `PAGE_RULES`-Eintraegen; die alten Kennungen loesen in der Taxonomie und den Erklaerungstexten
  ueber die neuen auf. Gewollte Unterschiede (Changelog von `a11y-rules`): keine Obergrenze je
  Seite mehr (bisher 10 bzw. 20), `<a onclick>` ohne `href` ist ein Klick-Handler statt Scheinlink,
  `navigation/location-missing` ist ein Pruefhinweis statt Verstoss und sieht auch Navigation im
  Shadow DOM. Von `pointer_cancellation` (2.5.2) ist der statische Teil (`onmousedown`/
  `ontouchstart` an Bedienelementen) geloescht -- barrierlab hat ihn mangels Beleg nicht
  uebernommen --, der seitenweite `UNTESTED`-Vermerk bleibt im Host: Die manuelle Checkliste
  (`manual/*`, barrierlab#39) hat keinen Punkt fuer 2.5.2, und auditmysite fuehrt sie noch nicht.
  Neu: Geteilte Kennungen oberhalb der geprueften Stufe (`--level`) werden herausgenommen und als
  `NotRun::Disabled` (`above_wcag_level`) vermerkt (`wcag::shared::retain_up_to_level`), so wie
  die abgeloesten AAA-Regeln nur mit `--level aaa` liefen. Klick-Handler und Scheinlinks laufen
  jetzt auch in Frames. Geprueft: Detection-Corpus vorher/nachher (nur die umbenannten Kennungen,
  `location` als Hinweis, zusaetzlich `landmark_main_in_shadow_root`), Ausschluss-Integrationstests
  mit den ungedeckelten Zaehlungen (12 bzw. 22 ausgeschlossene Treffer).

- **Unreleased — Landmark-, Tastatur- und Strukturregeln aus `a11y-rules` statt eigener (#694,
  B4):** Die Landmark-Pruefungen, die Tastaturerreichbarkeit, die Seite ohne Ueberschriften und
  zwei Musterpruefungen laufen ueber den geteilten Bestand (`SHARED_RULES` in
  `src/wcag/shared.rs`). Geloescht: `region`, `landmark_granular` (mit `is_landmark`) und
  `bypass_blocks`; aus `keyboard` die beiden 2.1.1-Pruefungen; aus `patterns/` die Befunde
  `dialog-no-focusable`, `accordion-no-controls`, `accordion-trigger-not-button` und
  `aria-expanded-required` -- Mustererkennung und Journeys bleiben, `PatternAnalysis.violations`
  ist jetzt immer leer. Im Host bleibt `keyboard-trap` (2.1.2): Hinweis je modalem Dialog und der
  seitenweite `UNTESTED`-Vermerk brauchen echte Tastaturbedienung. Kennungen: `landmark-unique` →
  `landmarks/not-unique`; `landmark-banner-is-top-level`, `landmark-contentinfo-is-top-level` und
  `landmark-main-is-top-level` → `landmarks/not-top-level`; `landmark-no-duplicate-banner` und
  `landmark-no-duplicate-contentinfo` → `landmarks/banner-duplicate` und
  `landmarks/contentinfo-duplicate`; `region` → `landmarks/content-outside` (alle 1.3.1 A);
  `bypass` („No headings found") → `headings/none` (2.4.1 A, samt #709); `focusable-no-role` und
  `keyboard` („appears not keyboard-focusable") → `keyboard/focusable-no-role` und
  `keyboard/interactive-not-focusable` (2.1.1 A); `dialog-no-focusable` →
  `dialog/focusable-missing` (2.4.3 A); `accordion-no-controls` →
  `patterns/accordion-controls-missing` (4.1.2 A). Ersatzlos entfallen, wie in barrierlab#17
  begruendet: `accordion-trigger-not-button` (an Rollen ohne `aria-expanded` meldet das
  `aria/attribute-not-allowed`, an `link`, `tab`, `treeitem` erlaubt WAI-ARIA den Zustand,
  fehlender Fokus ist `keyboard/interactive-not-focusable`) und `aria-expanded-required` (riet ein
  Aufklappmenue aus dem Wort „menu"/„Menü" im Namen). Gewollte Unterschiede (Einzelheiten im
  Changelog von `a11y-rules` und an den Eintraegen in `SHARED_RULES`): `landmarks/not-unique` und
  `patterns/accordion-controls-missing` sind Pruefhinweise statt Verstoesse; doppelte
  banner/contentinfo zeigen auf die zweite Landmark statt die erste; `landmarks/content-outside`
  meldet einmal je Block statt je Textknoten; `keyboard/focusable-no-role` zaehlt nur die Tabfolge
  und keinen benannten scrollbaren Bereich; `keyboard/interactive-not-focusable` nimmt
  deaktivierte Felder, native `<option>`, `aria-activedescendant` und Inertes aus;
  `dialog/focusable-missing` sucht in allen Nachfahren und prueft kein geschlossenes `<dialog>`;
  `headings/none` zaehlt `role="heading"` mit. Die Landmark-Rolle kommt aus dem Markup statt aus
  Chromes Rolle (#639, #727). Ausschluesse (#726): Die geteilten Zaehlregeln
  (`landmarks/not-unique`, `landmarks/main-duplicate`, `landmarks/*-duplicate`) lesen bei aktivem
  `--exclude-selector` das Dokument ohne die ausgeschlossenen Teilbaeume
  (`ExclusionScope::without_excluded_dom`, `wcag::shared::adopt_page_counts`); die AX-Variante
  `without_excluded` und der Parameter `counted` von `check_all_excluding` entfallen,
  `exclusion.excluded_landmarks` zaehlt wie bisher. In Frames laufen jetzt
  `keyboard/focusable-no-role` und `keyboard/interactive-not-focusable` (bisher die Baumregel
  `keyboard`) sowie neu `dialog/focusable-missing` und `patterns/accordion-controls-missing`; die
  Landmark-Regeln und `headings/none` bleiben beim Hauptdokument. Taxonomie: die Kennungen zeigen
  auf die Eintraege der abgeloesten Regeln, deren Legacy-Kennungen weiter aufloesen
  (`headings/none` auf den von `bypass`); neu `a11y.landmark_nested.invalid` und
  `a11y.accordion_controls.missing` samt Erklaerungstexten. Mit `a11y-rules` 0.17.0. Geprueft mit Unit-Tests, Detection-Corpus, Score-Kalibrierung,
  Referenzlauf und den Integrationstests zu Frames, Ausschluessen und Landmarks (Chrome) sowie einem
  Vorher/Nachher-Lauf auf 14 Live-Seiten; Referenzbaender gov.uk (75–95) und bundesregierung.de
  (35–76) neu bewertet (je +1: Inhalt ausserhalb von Landmarks je Block statt je Textstueck,
  Dialog-Fokusziel auch in tieferen Nachfahren).

- **Unreleased — Fehlalarme aus upscale.casoon.dev (#739, #740):** Das Dark-Mode-Modul meldete
  eine reine Dunkel-Seite (`html.dark` fest gesetzt, `color-scheme: dark`, kein helles Layout) als
  `no_dark_mode_support` mit Score 50. Neu ist das Signal "dunkel im Standard-Rendering":
  wirksamer Seitenhintergrund (body, sonst html, sonst die Leinwand nach `color-scheme`) dunkel und
  Body-Text hell; die Farben werden ueber ein Canvas-Pixel gelesen, damit auch `oklch()` aus
  Tailwind v4 zaehlt. Eine solche Seite gilt als unterstuetzt (`detection_methods`: "Dark by
  default (no light view)"), ohne `no_dark_mode_support` und ohne den Klassen-Hinweis (#739).
  Das Mobile-Modul nahm als kleinste Schrift auch Elemente ohne lesbaren Text; KaTeX'
  `span.vlist-s` (ein Nullbreiten-Leerzeichen bei 1 px) ergab "Smallest font size is 1.0px". Fuer
  die kleinste Schrift zaehlen jetzt nur Elemente mit eigenem Text aus mehr als Leerraum und
  Nullbreiten-Zeichen (#740). Live: upscale.casoon.dev Dark-Mode 50 → 85, Pythagoras-Lektion
  kleinste Schrift 1 px → 11 px (echter Text); gov.uk (hell) weiter ohne Dark-Mode-Unterstuetzung,
  casoon.de unveraendert. #738 (`<legend>` nach Leerraum) liegt in html-conform:
  casoon/barrierlab#41.

- **Unreleased — Formularregeln aus `a11y-rules` statt eigener (#693, B3):** Die Formular-,
  Kontextwechsel- und Captcha-Pruefungen laufen ueber den geteilten Bestand (`SHARED_RULES` in
  `src/wcag/shared.rs`), die abgeloesten eigenen Regeln sind geloescht: `form_rules`,
  `error_identification`, `input_purpose`, `identify_purpose`, `redundant_entry`,
  `label_title_only`, `on_focus`, `instructions` und `labels` (`check_form_control`, der letzte
  Rest der Datei). Im Host bleibt, was die laufende Seite braucht: in `on_input` das Nachschlagen
  einer aufgerufenen Funktion ueber `window` (Verstoss, wenn erst sie den Kontext wechselt) und die
  Namensvermutung („Language") ohne Absende-Button, beide jetzt unter derselben Kennung wie die
  geteilte Regel (`context/on-input`), damit der Verstoss deren Pruefhinweis am selben Element verdraengt;
  in `accessible_authentication` der Einfuege-Test (`accessible-auth-paste-blocked`). Kennungen:
  `label` „no accessible label" und `control-missing-label` an Formularfeldern →
  `forms/label-missing` (3.3.2 A); `label` „Placeholder used as only label" →
  `forms/placeholder-as-label`; `label` „Form group has no legend or label" →
  `forms/group-name-missing`; `label` „Required field not clearly indicated" und „may not indicate
  required status" → `forms/required-unmarked`; `label` „may require format instructions" →
  `forms/instructions-missing` (alle 3.3.2 A); `autocomplete-valid` → `forms/autocomplete-invalid`
  (ungueltiger Wert) und `forms/purpose-missing` (fehlender Wert, 1.3.5 AA), dazu
  `identify-purpose` (1.3.6 AAA) → `forms/purpose-missing`; `input-error-message` und
  `aria-invalid-without-describedby` → `forms/error-unidentified` (3.3.1 A); `form-field-group` →
  `forms/group-missing` und `label-title-only` → `forms/title-only-label` (1.3.1 A);
  `form-no-submit` → `forms/no-submit` (3.2.2 A, samt #728); `redundant-entry` →
  `forms/redundant-entry` (3.3.7 A); `input-no-context-change` (Handler im Markup) →
  `context/on-input` (3.2.2 A); `focus-no-context-change` → `context/on-focus` und
  `context/autofocus` (3.2.1 A); `accessible-auth-captcha` → `auth/captcha` (3.3.8 AA). Gewollte
  Unterschiede (Einzelheiten im Changelog von `a11y-rules` und an den Eintraegen in
  `SHARED_RULES`): ein fehlendes Label, eine fehlende Fehlerbeschreibung, ein unmarkiertes
  Pflichtfeld und ein fehlender Eingabezweck stehen einmal statt zweimal im Bericht;
  `forms/purpose-missing`, `forms/required-unmarked`, `forms/title-only-label`, `context/on-focus`
  und `context/autofocus` sind Pruefhinweise statt Verstoesse; `forms/autocomplete-invalid` prueft
  die ganze Autofill-Grammatik und ist niedrig statt mittel; `context/on-input` prueft auch
  Optionsfelder; `forms/no-submit` zaehlt per `form`-Attribut zugeordnete Felder und Buttons mit;
  der seitenweite `UNTESTED`-Vermerk zu 1.3.6 entfaellt. Die Formularregeln
  laufen jetzt auch in Frames, auch die bisher nur im Hauptdokument geprueften
  (`autocomplete`, Absenden, Wiederholung, Kontextwechsel, Captcha). Taxonomie: alle Kennungen
  zeigen auf die Eintraege der abgeloesten Regeln, deren Legacy-Kennungen weiter aufloesen; neue
  Erklaerungstexte fuer `forms/no-submit` und `auth/captcha`. Mit `a11y-rules` 0.16.0, das ein
  unbenanntes ARIA-Formularfeld (combobox, listbox, textbox, …) wie das native Feld ohne Label als
  kritisch meldet. Geprueft mit Unit-Tests, Detection-Corpus, Score-Kalibrierung und Referenzlauf
  (Chrome) sowie einem Vorher/Nachher-Lauf auf 14 Live-Seiten.

- **Unreleased — SEO-Ueberschriften aus den geteilten Regeln, kein zweiter HTML-Validator
  (#724):** Die SEO-Ansicht pruefte fehlende und mehrfache H1, uebersprungene Ebenen und leere
  Ueberschriften ein zweites Mal. Diese Befunde kommen jetzt aus `headings/*` (a11y-rules) der
  Barrierefreiheitspruefung, nach Ausschluessen und einmal je Befund ueber beide Viewports;
  `SeoModule::derive` setzt sie vor die SEO-eigene Regel fuer lange Ueberschriften und berechnet
  Score, Content-Profil und SERP-Sicht neu. Die SEO-Ueberschriftenliste (`h1_count`, `h1_text`,
  Laengen) liest jetzt auch offene Shadow Roots, damit sie zur Sicht der geteilten Regeln passt
  (dm.de: die h1 des Usercentrics-Banners im Shadow DOM zaehlt mit; SEO-Score 75 → 67). Der
  lokale html5ever-Check (`validate_html_locally`) ist gestrichen: Er zaehlte Parse-Fehler
  desselben serialisierten Live-DOM, den `html_conform` gruendlicher prueft. Der Validator-Status
  im SEO-Teil heisst jetzt `delegated` und verweist auf html-conform (`--full`,
  `--html-conform`); `ParseErrors` bleibt nur, damit gespeicherte Berichte laden. Gegenprobe alt
  gegen neu mit `--seo`: casoon.de, gov.uk, berlin.de und mit.edu mit gleichen
  Ueberschriftenbefunden und gleichem Score.

- **Unreleased — CSP ueber `csp-parse` (#723):** `src/security` zerlegte die
  Content-Security-Policy selbst (`split(';')`); jetzt liest es sie mit `csp-parse` 0.1.0 aus
  barrierlab (CSP3: Policy-Liste, Direktiven, Source-Listen mit Schluesselwoertern, Nonces, Hashes,
  Host-Wildcards). Die Bewertung (welche Schwaeche wie schwer wiegt) bleibt hier. Gewollte
  Unterschiede: mehrere `Content-Security-Policy`-Header werden alle gelesen und als Policy-Liste
  verbunden statt nur der erste; in einer Policy-Liste zaehlt eine Schwaeche nur, wenn jede Policy
  sie hat (der Browser setzt alle durch); eine doppelte Direktive behaelt ihren ersten Wert statt
  des letzten; Nonce und Hash zaehlen nur mit gueltigem Base64-Wert; die Wildcard-Pruefung ueber
  alle Direktiven liest nur Source-Listen (nicht `report-uri`, `sandbox` u. a.). Neue Unit-Tests
  fuer die drei Faelle; Security-Corpus gruen; die acht Referenzseiten liefern mit altem und neuem
  Parser dieselben CSP-Befunde.


- **1.7.3, 2026-10-01:** ARIA- und Namensregeln kommen jetzt aus `a11y-rules` statt aus eigenen
  Implementierungen (#691 B1, #692 B2, a11y-rules 0.15.0), mit neuen Kennungen nach dem Schema
  `aria/*`, `names/*`, `dialog/*`, `popover/*`, `inert/*`; ein unbenannter Dialog steht nur noch
  einmal im Bericht. Cross-Site-iframes werden geprueft, weil der Audit-Browser ohne
  Site-Isolation startet (#720). SPA-Portale werden erst nach dem ersten gerenderten Inhalt
  aufgenommen (#718). Fehlalarme behoben: Textebene in verborgenen Panels (#725), unbenannte
  `form`/`region` als Landmark (#727), Landmark-Zaehlung trotz Ausschluss (#726, neues JSON-Feld
  `exclusion.excluded_landmarks`), `form-no-submit` bei reinen Schalter-Formularen (#728).
  Referenzlauf (`reference_sites_test`) und Detection-Corpus gruen. Einzelheiten in den Eintraegen
  darunter.

- **Namensregeln aus `a11y-rules` statt eigener (#692, B2):** Die Pruefungen auf
  fehlende und unzureichende zugaengliche Namen laufen ueber den geteilten Bestand
  (`SHARED_RULES` in `src/wcag/shared.rs`), die abgeloesten eigenen Regeln sind geloescht:
  `aria_naming_rules`, `dialog_rules`, `summary_name`, `status_messages`, `label_in_name` und aus
  `accessible_name` alles ausser `description-duplicates-name` (#713) -- die bleibt, weil
  `a11y_dom` keine Accessible Description liefert; ihr Schalter in `wcag::engine` heisst jetzt so
  statt `aria-label`. Ebenfalls entfallen: `aria-dialog-name` in `patterns::modal_dialog` und der
  Dialog-Teil des fehlenden Namens in `modern_attributes` (Menue und Popover bleiben dort).
  Kennungen: `aria-command-name` (ohne Button und `a[href]`), `aria-input-field-name`,
  `aria-meter-name`, `aria-progressbar-name`, `aria-toggle-field-name`, `aria-treeitem-name` und
  `aria-label` „kein Name" → `names/required-missing`; `aria-label` „Icon Only" →
  `names/symbol-only`; `aria-dialog-name` und `dialog-name` → `dialog/name-missing`; `dialog-name`
  „Dialog Modal" → `dialog/modal-unmarked`; `summary-name` → `summary/name-missing`;
  `aria-live-region-role` → `status/live-overridden` (4.1.3 AA); `label-content-name-mismatch` →
  `label-in-name/mismatch` (2.5.3 A); alle uebrigen unter 4.1.2 A. Schwere wie bisher. Gewollte
  Unterschiede (Einzelheiten im Changelog von `a11y-rules` und an den Eintraegen in
  `SHARED_RULES`): ein unbenannter Dialog steht einmal statt bis zu dreimal im Bericht, ein
  unbenanntes Widget einmal statt zweimal; unbenannte `menu`, `tab` und native `<option>` sind kein
  Befund mehr (ARIA 1.2 verlangt dort keinen Namen); `meter` und `progressbar` stehen unter 4.1.2
  statt 1.1.1; nicht fokussierbare unbenannte Widgets sind ein Verstoss wie bisher in
  `aria_naming_rules`, die zusaetzliche Warnung aus `accessible_name` entfaellt; der rein
  symbolische Name, das fehlende `aria-modal` und eine umgestellte, aber nicht abgeschaltete
  Dringlichkeit einer Live-Region (`alert` mit `polite`) sind Pruefhinweise statt Verstoesse;
  `dialog/modal-unmarked` prueft nur `role="dialog"`, kein natives `<dialog>`; `summary/name-missing`
  meldet nur die `<summary>`, nicht zusaetzlich das `<details>`; leeres `aria-labelledby` und
  `aria-describedby` an einem benannten Element sind kein Befund mehr; `label-in-name/mismatch`
  prueft alle Rollen mit Namen aus dem Inhalt und Namen aus `aria-labelledby`, nicht nur Buttons
  mit `aria-label`, ohne den Deckel von 50 -- und laeuft jetzt auch in Frames. Neuer
  Taxonomie-Eintrag `a11y.dialog_modal.missing` samt Erklaerung (bisher teilte das fehlende
  `aria-modal` Kennung und Titel mit dem fehlenden Dialognamen); die uebrigen Kennungen zeigen auf
  die Eintraege der abgeloesten Regeln, deren Legacy-Kennungen weiter aufloesen. Geprueft mit
  Unit-Tests, Detection-Corpus und Score-Kalibrierung (Chrome). Mit `a11y-rules` 0.15.0. Der #527-Filter (Verstoss verdraengt Hinweis) vergleicht jetzt die Backend-Node-ID, wo beide Befunde eine tragen: geteilte Befunde haben kurze Selektoren (`button`), und ein Verstoss an einem Button verschluckte den Hinweis an einem anderen. Referenzband bundesregierung.de neu bewertet (35–72): drei Label-in-Name-Fehlalarme an per CSS verstecktem Text entfallen, der Cookie-Dialog ohne aria-modal ist ein Hinweis.


- **Frames in fremden Prozessen (#720):** Nach #715 blieben iframes, die Chrome wegen
  Site-Isolation in einem eigenen Renderer zeigt, ungeprueft (`skipped: cross_origin`) — etwa der
  Webchat auf gov.cy (`digital-assistant.gov.cy`, `gov.cy` ist ein Public Suffix), in dem axe einen
  unbenannten `role="dialog"` meldet. chromiumoxide 0.8 legt fuer solche Frames zwar intern eine
  Session an, bietet aber keine Befehle an sie. Statt eines eigenen CDP-Clients startet der
  Audit-Browser jetzt ohne Site-Isolation (`--disable-site-isolation-trials`,
  `IsolateOrigins`/`site-per-process` aus): Cross-Site-Frames laufen im Prozess der Seite, der
  bestehende Frame-Durchgang erreicht sie samt Anreicherung, Selektoren und Ausschluessen. Das
  Profil ist frisch und ohne Nutzerdaten; ignoriert ein kuenftiges Chrome die Flags, steht der
  Frame wieder sichtbar als `cross_origin` im Bericht. Der Chrome-Test erwartet den Cross-Site-Frame
  jetzt als geprueft mit `dialog-name`-Befund; gov.cy live: Frame geprueft, Dialog gefunden.

- **SPA-Portale: Aufnahme vor dem ersten Rendern (#718):** eesti.ee zeigt nach dem Laden einen
  leeren Splash-Screen, waehrend die Angular-App ihre erste Ansicht holt — rund 400 ms ohne eine
  einzige DOM-Mutation. `wait_for_page_stability` wertete 200 ms Ruhe als stabil, der AX-Baum wurde
  vor der ersten Ueberschrift gelesen und `bypass` meldete "No headings found". Ruhe ohne Inhalt
  (kein `main`, keine `h1`-`h3`, kein sichtbarer Text) gilt jetzt nicht mehr als stabil; die
  Provenienz vermerkt das Warten als `reason`, eine Seite ohne Inhalt bis Budgetende endet als
  `budget_exhausted`. Neuer Chrome-Test mit Splash-Fixture (scheitert ohne den Fix);
  Detection-Corpus gruen; eesti.ee und latvija.gov.lv ohne den Fehlbefund.

- **ARIA-Regeln aus `a11y-rules` statt eigener (#691, B1):** Die ARIA-Pruefungen
  laufen ueber den geteilten Bestand (`SHARED_RULES` in `src/wcag/shared.rs`, alle unter 4.1.2
  A), die abgeloesten eigenen Regeln sind geloescht: `aria_allowed_attr`, `aria_prohibited_attr`,
  `aria_valid_attr_value`, `aria_required_parent`, `parsing`, `aria_roles`, `widget_rules`, die
  Popover- und Dialog-Teile von `modern_attributes` und die Tab-Meldung aus `patterns::tab_list`.
  Kennungen: `aria-roles` → `aria/role-invalid` (neu dazu `aria/role-abstract`) und, fuer die
  Bestandteile, `aria/required-children-missing`; `aria-attr-name-invalid` →
  `aria/attribute-unknown`; `aria-valid-attr-value` → `aria/attribute-value-invalid` (die
  IDREF-Haelfte ist `aria/reference-missing`); `duplicate-id-aria` → `aria/owns-conflict`;
  `aria-allowed-attr` → `aria/attribute-not-allowed`; `aria-prohibited-attr` →
  `aria/attribute-prohibited`; `aria-required-parent` → `aria/required-parent-missing`;
  `aria-tab-selected-state` und `tab-no-aria-selected` → `aria/tab-selected-missing`;
  `aria-tablist-tabpanel` → `aria/tabpanel-missing`; `aria-combobox-options` →
  `aria/combobox-popup-missing`; aus `modern-attribute-misuse` → `popover/target-missing`,
  `popover/target-invalid`, `inert/dialog-inert`. Gewollte Unterschiede (Einzelheiten im
  Changelog von `a11y-rules` und an den Eintraegen in `SHARED_RULES`): Kontext und Bestandteile
  werden am DOM geprueft -- `ul[role=tablist] > li > a[role=tab]` (#715) meldet jetzt wie axe
  fehlende Bestandteile an der Tabliste und fehlenden Kontext am Tab; erlaubte und verbotene
  Attribute urteilen ueber die implizite Rolle mit (`<div aria-expanded>` ist ein Befund);
  Bestandteile nur fuer Behaelter mit expliziter Rolle, die native Tabelle nur mit `<caption>`
  ist kein Befund mehr; fehlender ausgewaehlter Tab und fehlendes Tab-Panel sind Pruefhinweise,
  der Tab-Hinweis steht einmal an der Tabliste statt je Tab; unbekannte Werte von `aria-current`
  und `aria-invalid` sind kein Befund; das sichtbare `role="menu"` mit `inert` ist keiner mehr.
  In `modern_attributes` bleiben der fehlende Name offener Dialoge, Menues und Popover, das offene
  Popover mit `inert` und der Fokus in einem inerten Teilbaum. Chromes eigene Rollennamen
  erreichen die geteilten Regeln nicht mehr (`CdpDocument`): `image` wird `img`, die gross
  geschriebenen (`LayoutTable*`, `DescriptionList`, `LabelText`, ...) werden zu „keine Rolle" --
  sonst blieb `role="img"` ohne Attributurteil und brach eine Layouttabelle jede Kontextpruefung
  ueber ihr ab. Neuer Taxonomie-Eintrag `a11y.tab_selected.missing`; die uebrigen Kennungen zeigen
  auf die Eintraege der abgeloesten Regeln, deren Legacy-Kennungen weiter aufloesen. Erklaerungen
  und Detection-Corpus sind nachgezogen. Geprueft mit Unit-Tests und dem Detection-Corpus (Chrome),
  darin der #715-Fall im iframe. Referenzbaender bundesregierung.de (35–65), berlin.de (15–48) und dm.de (15–48) neu bewertet: eingeklapptes `aria-controls` ohne Ziel ist kein Befund mehr (dm.de), Tab-Auswahl nur noch einmal je Tablist (ARIA SHOULD), Swiper-Listen als `lists/invalid-structure` (Medium) statt `aria-roles` (High) — a11y-rules 0.14.1 faengt `li[role=group]` dafuer wieder ab.

- **Fehlalarme aus barrierlab.eu und geographia.eu (#725-#728):**
  `display/text-hidden` meldete die Textebene von Visualisierungen in inaktiven Bereichs-Panels
  (`hidden` am `section`) als entzogen; die ganze Figur samt Grafik war aber verborgen. Eine nicht
  gerenderte Figur zaehlt jetzt auch bei `hidden`/`aria-hidden`/`inert` an Figur oder Vorfahr nicht
  als Verstoss (#725; die sr-only-Vermutung im Issue traf nicht zu, `checkVisibility` liess das
  Clip-Muster schon durch). `landmark-unique` und die Top-Level-Pruefungen zaehlen `form` und
  `region` nur mit zugaenglichem Namen als Landmark (#727). Die zaehlenden Landmark-Regeln
  (`landmark-unique`, doppeltes banner/contentinfo) lesen einen Baum ohne ausgeschlossene
  Teilbaeume; das JSON nennt die herausgenommenen Landmarks als `exclusion.excluded_landmarks`,
  die PDF-Notiz ebenso (#726). `form-no-submit` meldet ein Formular ohne `action`, ohne Textfeld
  und ohne Absende-Element (nur Skript kann es absenden, typisch Schalter fuer Inhalte) nur noch
  als niedrigen Pruefhinweis statt als 3.2.2-Verstoss (#728). Neue Corpus-Faelle
  `display_modes_hidden_panel`, `form_toggles_without_submit`; Detection-Corpus gruen;
  Gegenprobe gegen beide Live-Seiten ohne die Fehlbefunde.

- **1.7.2, 2026-09-30:** Fehlalarm-Korrekturen und Luecken aus dem axe-Gegencheck der
  barrierlab.eu-Erhebung (21 EU-Portale) und aus geographia.eu: Zielgroesse nur fuer sichtbare
  Ziele im fertigen Layout (#705, #706), Ausschluesse auch in der Screenreader-Schicht und im Cache
  (#703, #708), Darstellungsregeln ohne Rechtsurteil (#704), `main` und Sprunglink hinter offenem
  Dialog bzw. Cookie-Banner (#709, a11y-* 0.13.5), Linkfarbe nur im Fliesstext (#710), Bewegung nur
  mit Beleg (#712), title-only und doppelte Beschreibung als Best Practice (#711, #713). Neu:
  Kontrast in Shadow DOM und ueber Float-Containern (#716), Regel `scrollable-region-focusable`
  (#717), elementbezogene Regeln in iframes samt Frame-Abdeckung im JSON (#715, teilweise; Frames
  in fremden Prozessen folgen in #720), geringeres Gewicht fuer `zoom/viewport-missing` (#702).
  Referenzlauf (`reference_sites_test`) und Detection-Corpus gruen. Einzelheiten in den Eintraegen
  darunter.

- **Regeln laufen in iframes (#715):** Ein `role="dialog"` ohne Namen in einem
  iframe (gov.cy) blieb unentdeckt, weil `getFullAXTree` und die geteilten DOM-Regeln nur das
  Hauptdokument sahen; axe prueft jeden Frame. Neu ist ein Frame-Durchgang (`audit::frames`):
  Fuer jedes im Prozess der Seite gerenderte iframe (auch verschachtelt) holt er den AX-Baum per
  `frameId` und baut aus dem `content_document` desselben `DOM.getDocument`-Abrufs ein eigenes
  Dokument, dann laufen dort die **elementbezogenen** Baum- und geteilten Regeln (Namen von
  Dialogen, Bildern, Links, Buttons, Formularfeldern; Rollen samt geforderten Eltern/Kindern;
  Listen- und Tabellenstruktur; referenzierte doppelte IDs; `aria-hidden`-Fokus u. a.) — per
  expliziter Positivliste. Seitenbezogene Regeln (Landmarks, `h1`, Titel, Viewport, Sprunglink,
  `bypass`, `region`) bleiben wie bei axe dem Hauptdokument vorbehalten. Befunde tragen den
  Selektor `"{iframe} [frame] {element}"` und eine `frame:`-Knotenkennung, Ausschluesse (#645)
  gelten auch hier. Uebersprungen werden verborgene Frames (`hidden`, `aria-hidden`,
  `role=none`, unsichtbar, hoechstens 1 px, im AX-Baum ignoriert), ausgeschlossene und
  prozessfremde (cross-origin) Frames; `execution.frames` im JSON fuehrt geprueft und
  uebersprungen samt Grund (`cross_origin`, `hidden`, `excluded`, `ax_tree_failed`,
  `document_unavailable`) je Viewport. Der JavaScript-Scan `iframe/same-origin-content` prueft
  nur noch `html-has-lang`, seine uebrigen Pruefungen deckt der Durchgang ab. Neuer
  Detection-Corpus-Fall `iframe_widget_rules` und ein Chrome-Integrationstest, der auch belegt,
  dass keine Seitenregel im Frame meldet. Offen bleibt die Tablist aus `ul`/`li`
  (magyarorszag.hu): Chrome glaettet das `li` zwischen Tablist und Tab im AX-Baum, die
  baumbasierten Regeln sehen dort — auch im Hauptdokument — eine gueltige Struktur. Live auf
  gov.cy liegt der Dialog in einem prozessfremden Frame (`digital-assistant.gov.cy`, `gov.cy` ist
  ein Public Suffix): Er steht jetzt als `cross_origin` uebersprungen im Bericht, geprueft wird er
  erst mit einer eigenen CDP-Sitzung je Out-of-Process-Frame.

- **Screenreader-Ausschluesse ueberleben den Cache (#708):** Bei `--reuse-cache`
  wird der Screenreader-Bericht aus dem gespeicherten AX-Baum neu gebaut; die in #703
  ausgeschlossenen Knoten standen danach wieder im Sidecar und im PDF. Die ausgeschlossenen
  Knoten-IDs liegen jetzt im Cache-Eintrag (`snapshot.json`,
  `screen_reader_excluded_node_ids`) und werden beim Wiederaufbau erneut angewendet. Aeltere
  Eintraege ohne das Feld verhalten sich wie bisher.

- **Kontrast in Shadow DOM und ueber Float-Containern (#716):** Zwei Luecken
  gegenueber axe. (1) Die Kontrastpruefung lief nur ueber den Light DOM; Text in Webkomponenten
  (Cookie-Banner `p-cookie-banner` auf verwaltung.bund.de, 3,17:1) fehlte. Sie geht jetzt durch
  offene Shadow Roots, der Hintergrund wird ueber den zusammengesetzten Baum bestimmt (Slot,
  Shadow-Host), Text direkt unter einem Shadow-Host nimmt Farbe und Schrift von seinem Slot (ohne
  Slot wird er nicht gerendert und nicht geprueft), `aria-hidden` am Host wirkt nach innen, und der
  Selektor lautet `host >>> inner` (auch in der Pixelprobe). Die Overlay-Erkennung sieht jetzt auch
  positionierte Geschwister, die ein Bild enthalten (Hero-Bild als `<p-responsive-image>`), bis
  sechs Ebenen hoch; die Pixelprobe meldet verdeckten Text (Banner ueber dem Text) als
  Pruefhinweis statt ihn gegen den Banner zu messen. (2) `__amsIsVisuallyHidden` hielt jeden
  Vorfahren ohne Hoehe fuer versteckt; ein `div`, das nur Floats enthaelt, ist aber 0 px hoch und
  zeigt seinen Inhalt. Nur eine Box, die ihren Ueberlauf abschneidet, versteckt ohne Flaeche
  etwas; Nicht-Gerendertes erkennt `checkVisibility()`. Damit wird der 4,46:1-Link auf
  slovensko.sk gemeldet (axe: 4,45:1; der Vergleich mit der Schwelle war schon ungerundet). Der
  Helfer wird auch von Design-Qualitaet, Journey und Medienregel genutzt. Live: bund.de meldet den
  Datenschutz-Link und den weissen Hero-Text ueber dem Foto (Pixelprobe 3,64:1), slovensko.sk den
  Link. Neuer Detection-Corpus-Fall `contrast_shadow_and_inline`.

- **Scrollbare Bereiche ohne Tastaturzugang (#717):** Neue Regel
  `scrollable-region-focusable` (2.1.1, A) → `a11y.scrollable_region_focus.missing`: ein
  sichtbares Element mit `overflow: auto|scroll`, das mehr als 13 px ueberlaeuft (wie axe),
  Text enthaelt und weder selbst noch ueber einen Nachfahren per Tastatur fokussierbar ist.
  Chrome macht solche Bereiche seit Version 130 selbst fokussierbar, andere Browser nicht —
  deshalb bleibt es ein Befund. Neuer Detection-Corpus-Fall `scrollable_region_focusable`.

- **Gewicht von `zoom/viewport-missing` (#702):** Desktop-Browser ignorieren das
  viewport-Meta-Tag, Zoomen per Geste bleibt moeglich; die Folge trifft nur Mobilgeraete. Der
  Score-Abzug sinkt von 2,5/5 auf 1,5/3, Bericht und Erklaerung nennen die Folge (Desktop-Breite,
  verkleinert, waagerechtes Scrollen beim Vergroessern). Befundstufe (FAIL) und Schwere kommen
  weiter aus `a11y-rules`; ob es dort ein Pruefhinweis sein sollte, ist eine Frage an barrierlab.

- **Farbe als Linkmerkmal nur im Fliesstext, Bewegung nur mit Beleg (#710, #712):**
  `link-in-text-block` (1.4.1) prueft einen Link nur noch, wenn er im Fliesstext steht — wie
  axe `isInTextBlock`: Der Abschnitt seines Blocks (begrenzt durch `<br>`/`<hr>`) muss mehr
  Nicht-Link-Text als Link-Text enthalten, und zwar mindestens zwei Woerter. Links in `nav`
  bzw. Menues und Logo-Links (Bild/SVG im Link, Logo-Container) fallen heraus. Damit melden
  Navigations-, Header-, Footer- und Logo-Links keinen Befund mehr (12 von 21 EU-Portalen,
  axe bestaetigte keinen); der Link im Absatz bleibt ein Befund. `prefers-reduced-motion`
  (2.3.3, AAA) meldet nicht mehr jede `transition`/`animation`, sondern nur Bewegung —
  Animationen, deren `@keyframes` transform/translate/scale/rotate, Lage oder margin aendern,
  und Transitions solcher Eigenschaften (`all` nur, wenn eine `:hover`/`:focus`-Regel sie
  setzt) —, die kein Block `@media (prefers-reduced-motion: reduce)` fuer denselben oder den
  Universalselektor abschaltet; Regeln unter `no-preference` zaehlen nicht. Farb- und
  Deckkraft-Uebergaenge zaehlen nie. Die Meldung nennt die Belege und die Stufe AAA; AAA war
  bereits weder rechtliches Signal noch hoch BFSG-relevant, ein Test sichert das jetzt ab.
  Neue Detection-Corpus-Faelle `link_in_text_block_context`, `reduced_motion_colour_only`,
  `reduced_motion_transform`, `reduced_motion_override`. Gezaehlt wird nur Bewegung, deren Selektor ein dargestelltes Element der Seite trifft
  (bundesregierung.de: Keyframes eines nie eingebundenen Player-Spinners).

- **title-only und doppelte Beschreibung als eigene Best-Practice-Regeln (#711,
  #713):** Das `title`-Attribut als einzige Beschreibung eines interaktiven Elements stand als
  `a11y.hover.content_visibility` unter 1.4.13, obwohl der Browser-Tooltip dort ausgenommen ist;
  es ist jetzt `title-only-description` → `a11y.title_only_description.weak`. Ein Element, dessen
  Name und Beschreibung identisch sind, stand als `a11y.interactive_name.missing` im Bericht,
  obwohl es einen Namen hat; es ist jetzt `description-duplicates-name` →
  `a11y.description_duplicates_name.redundant`. Beide sind `best-practice`, an 4.1.2 verankert
  (`title` ist nach accname eine gueltige Namensquelle, 2.5.3 gilt nur bei sichtbarer
  Beschriftung), haben `bfsg_relevance: low` und nie ein Rechtsflag — `is_convention_rule`
  fuehrt dafuer eine Liste von Best-Practice-Kennungen neben den `display/*`-Regeln. 1.4.13
  meldet nur noch verwaiste `role="tooltip"`-Elemente, `interactive_name.missing` nur noch
  Elemente ohne aussagekraeftigen Namen. Neuer Detection-Corpus-Fall
  `name_description_best_practice`, `forms_and_misc` um einen verwaisten Tooltip ergaenzt.

- **a11y-rules 0.13.5 (#709):** Der Sprunglink wird auch hinter den Links eines
  Cookie-Banners erkannt, wenn er auf den Anfang des Hauptinhalts zeigt (bund.de,
  casoon/barrierlab#26). Ist ein modaler Dialog offen (etwa ein Consent-Dialog, der `<main>` per
  `aria-hidden` ausblendet), ist `landmarks/main-missing` ein Pruefhinweis „gemessen hinter einem
  offenen Dialog" statt eines Verstosses (#709: fuenf von 21 EU-Portalen), auch bei einem Dialog
  ohne `aria-modal` (administracion.gob.es). Ebenso wird `bypass` („keine Ueberschriften") hinter
  einem offenen Dialog zum Pruefhinweis. Referenzband gov.ie neu bewertet (80–98): die bis 1.7.1
  gezaehlten zehn Linkfarben-Befunde waren freistehende Listenlinks, Fehlalarme nach #710.

- **Zielgroesse misst nur sichtbare Ziele im fertigen Layout (#705, #706):**
  `target-size-minimum` (2.5.8) und `target-size` (2.5.5) zaehlen ein Element nur noch als Ziel,
  wenn es sichtbar und anklickbar ist: nicht `checkVisibility()`-verborgen (mit
  `opacityProperty`, `visibilityProperty`; bewusst ohne `contentVisibilityAuto`, damit Abschnitte
  ausserhalb des Viewports gemessen bleiben), nicht in `[inert]`, ohne
  `pointer-events: none` — wie schon beim gleichwertigen Link. Das gilt fuer das gemessene Ziel
  und fuer die Nachbarn der Abstandspruefung; Links in einem geschlossenen `<details>` (Chrome
  behaelt ihre Box, verbirgt sie per `content-visibility: hidden`) machten auf geographia.eu drei
  Links im Footer und in einer Liste zum Befund. Ohne `checkVisibility` bleibt es beim bisherigen
  Groessenfilter. Ausserdem wartet jeder Viewport-Durchgang nach der Stabilitaetspruefung auf
  laufende endliche CSS-Animationen und -Transitions (`document.getAnimations()`, unendliche
  werden ignoriert), begrenzt durch ein zweites `--stability-budget-ms`; eine Animation, deren
  Restzeit das Budget uebersteigt, wird gar nicht erst abgewartet. Ein Ziel, das danach selbst
  oder ueber einen Vorfahren noch animiert, meldet die Regel als nicht gemessen (`untested`)
  statt mit seiner Zwischengroesse: Das Logo-Intro auf geographia.eu/atmosphere/ stand als 3×39
  px im Bericht, fertig misst es 147×39 px. Neue Detection-Corpus-Faelle
  `target_size_hidden_neighbours` und `target_size_animation`; gegen den Stand vor dem Fix
  schlagen `a#beside-details` und `a#slow` fehl. Geprueft mit `cargo test`, Clippy und dem
  Detection-Corpus (Chrome).

- **Darstellungsregeln entscheiden kein Rechtsurteil mehr (#704):**
  `display/text-hidden` meldet ein per CSS, `hidden` oder `aria-hidden` verstecktes
  `[data-viz-text]` nicht mehr als Verstoss, wenn ein gerendertes, erreichbares Element derselben
  `figure[data-viz]` (oder die figure selbst) es per `aria-describedby`/`aria-details` referenziert:
  accname 1.2 berechnet die Beschreibung auch aus verstecktem, direkt referenziertem Inhalt (ARIA15
  genuegt 1.1.1). Stattdessen ein Pruefhinweis mit Schwere niedrig („nur als Beschreibung,
  visually-hidden empfohlen"). `inert` bleibt ein Verstoss. Alle `display/*`-Regeln sind
  Best Practice der Darstellungs-Konvention: Sie setzen `bfsg_relevance` hoechstens auf `low` und
  zaehlen nicht mehr als `legal_flags` (Seiten- und Batch-Urteil, `passed` im JSON); ein
  gemeinsames `NormalizedFinding::is_legal_flag` ersetzt die gleichlautenden Filter. Bewusst
  nur fuer `display/*` (`taxonomy::is_convention_rule`), nicht generisch ueber das
  `best-practice`-Tag — `region`, `link-as-button` und `redundant-role` behalten ihre Einordnung.
  `display/toggle-missing` sagt jetzt „kein mit `[data-display-toggle]` markierter Umschalter
  gefunden" statt zu behaupten, man koenne nicht umschalten. Belegt am Fall geographia.eu
  (Detection-Corpus `display_modes_described_text`).

- **Ausschluesse gelten auch fuer die Screenreader-Schicht (#703):** Knoten in
  `--exclude-selector`/`[data-audit-exclude]`-Teilbaeumen (#645) fallen jetzt auch aus
  `screen_reader.issues` und damit aus `bfsg_compliance` und dem daraus abgeleiteten
  `risk.legal_flags` (#484). Verortet wird ueber die Backend-Knoten-ID des AX-Knotens, wie bei den
  Baum-Regeln (`ExclusionScope::excludes_ax_node`). Ein Befund faellt nur weg, wenn alle seine
  Knoten ausgeschlossen sind; sonst bleibt er mit den uebrigen Knoten. Befunde ohne Knoten
  (seitenweit) bleiben immer. Gezaehlt in `exclusions.excluded_screen_reader_issues` (auch im
  Batch-Aggregat und im PDF-Hinweis); das PDF, das die Befunde in der Laufsprache neu ableitet,
  laesst dieselben Knoten weg. Belegt an barrierlab.eu `/tasks/ticket/` (Detection-Corpus
  `audit_exclude_screen_reader`, der dafuer `screen_reader_bfsg_verdict` pruefen kann).

- **1.7.1, 2026-09-30:** Erster Schritt des Regelumzugs nach barrierlab (casoon/barrierlab#13):
  21 Kennungen kommen aus `a11y-rules` 0.13.2, die abgeloesten eigenen Regeln sind geloescht (#690,
  B0). Neue Befunde `zoom/viewport-missing` (Gewichtung folgt in #702) und `images/alt-suspicious`;
  die geteilten Regeln pruefen per CSS Verstecktes nicht mehr. Referenzlauf
  (`reference_sites_test`) und Detection-Corpus gruen. Einzelheiten im Eintrag darunter.

- **Geteilte Kennungen statt eigener Regeln (#690, B0):** Weitere Befunde laufen
  ueber den geteilten Bestand aus `a11y-rules` 0.13.2 (`SHARED_RULES` in `src/wcag/shared.rs`),
  die abgeloesten eigenen Regeln sind geloescht, sodass kein Befund doppelt im Bericht steht.
  Kennungen: `meta-viewport` → `zoom/viewport-locked`; `document-title` (fehlend/leer) →
  `document/title-missing`/`document/title-empty`, der nichtssagende Titel bleibt als
  `document-title`; `aria-valid-attr` → `aria/reference-missing`; `aria-required-attr` →
  `aria/required-attribute-missing`; `aria-hidden-focus` und `focus-order-semantics` →
  `keyboard/hidden-focusable` (bisher derselbe Fall zweimal, unter 4.1.2 und 2.4.3);
  `landmark-one-main` und `landmark-main-present` → `landmarks/main-missing` (bisher zweimal);
  `landmark-no-duplicate-main` → `landmarks/main-duplicate`; `landmark-banner-present`,
  `landmark-navigation-present`, `landmark-contentinfo-present` → `landmarks/*-missing`, jetzt
  als Pruefhinweis statt Verstoss; `svg-img-alt` und `image-alt` an `<svg>` → `svg/name-missing`;
  `image-alt` an `<img>` → `images/alt-missing` (`image-alt` bleibt fuer `role="img"` an anderen
  Elementen und die Icon-Heuristik); Link und Button ohne Namen aus `control-missing-label` →
  `links/name-missing`/`buttons/name-missing`; die Sprunglink-Erkennung aus `bypass` →
  `keyboard/skip-link-missing` (Pruefhinweis, erkennt den Sprunglink am Ziel statt an einer
  Wortliste). `bypass` meldet nur noch eine Seite ganz ohne Ueberschriften; die Sammelmeldung
  „weder Sprunglink noch main" entfaellt. Neu: `zoom/viewport-missing` (kein viewport-Meta-Tag,
  1.4.4) und `images/alt-suspicious` (Dateiname, Fuellwort oder ein, zwei Zeichen als Alt-Text;
  Pruefhinweis). Die geteilten Regeln laufen jetzt mit berechnetem `display`/`visibility` aus
  einem DOMSnapshot (`run_full_in` ueber `CdpDocument::rendered`) und pruefen per CSS
  Verstecktes damit nicht mehr; scheitert der Snapshot, laufen sie ohne Stile. Ein Element ohne
  Gegenstueck im AX-Baum gilt als ignoriert (Inhalt eines geschlossenen `<details>`: geographia.eu
  bekam sonst 22 `links/name-missing`). `skip-link` aus `landmark_granular` entfaellt, der Fall ist
  `keyboard/skip-link-missing`. User-Agent-Shadow-Roots (Datumsfelder) kommen nicht mehr in den
  geteilten DOM. Taxonomie, Erklaerungen und Detection-Corpus sind nachgezogen. Geprueft mit
  Detection-Corpus (Chrome) und einem Vorher/Nachher-Lauf gegen 1.7.0 auf acht Live-Seiten; jede
  Abweichung ist erklaert (Umbenennung, entfallene Doppelmeldung, per CSS Verstecktes, oder neu
  und berechtigt), zwei Fehlalarme dabei in `a11y-rules` 0.13.2 behoben.

- **Score-Kalibrierung: `table_missing_caption` ist eine saubere Seite, 2026-09-30:** Das Fixture
  stand im Band „genau ein Level-A-High"; dieser Befund war der Fehlalarm aus #659/#674 (implizites
  `<tbody>` unter `<table><tr>` verdeckte die Zeilen vor der Pflicht-Kind-Pruefung). Seit #680
  traegt die Seite nur noch den Caption-Pruefhinweis und liegt mit 95 im Band ≥ 95; der Tag-Lauf
  von v1.7.0 war daran gescheitert.

- **1.7.0, 2026-09-30:** Neue Funktionen: Techniker-Modus `--technician` (JSON je Seite plus
  `index.json` und `findings.jsonl`, Pfadfilter `--include-path`/`--exclude-path`, Plan 67),
  Darstellungsmodi `--display calm|text|visual|all` (#653), Ausschluss absichtlicher Beispiele ueber
  `data-audit-exclude`/`--exclude-selector`, sichtbar im Bericht (#645), und vollstaendig
  englische PDF-Berichte mit `--lang en`. Dazu die Fehlalarm-Korrekturen aus #638-#659 und
  #673/#674 sowie der Umbau aus Plan 66 (WP0-WP8, Ausgabe per Golden-Harness unveraendert). Crate-
  Metadaten: `documentation` zeigt auf https://casoon.github.io/auditmysite/, `homepage` auf
  https://auditmysite.casoon.de/. Einzelheiten in den Eintraegen darunter.

- **Refactoring API-Oberflaeche (Plan 66 WP8), 2026-09-29:** Reiner Umbau
  ohne Aenderung an Ausgabe oder oeffentlicher Bibliotheks-API. Die 199 Stellen, die der Lint
  `unreachable_pub` meldet (158 in der Bibliothek, 41 im Binary), sind nach Compiler-Vorschlag auf
  `pub(super)` (140) bzw. `pub(crate)` (59) verengt — fuer Bibliotheksnutzer waren sie ohnehin
  unsichtbar; erreichbare `pub`-Items bleiben unangetastet (Semver/Studio, 1.7). `lib.rs` und
  `main.rs` setzen jetzt `#![warn(unreachable_pub)]`. Die 100 Vergleiche
  `i18n.locale() == "en"` laufen ueber den vorhandenen Helfer `output::localized::is_english`;
  dafuer ist `output::localized` nicht mehr an das Feature `pdf` gebunden (nur `pick` bleibt es).
  Verifiziert: Golden-Harness vor/nach identisch, Clippy mit beiden Feature-Sets.
- **Refactoring `audit/pipeline.rs` (Plan 66 WP7), 2026-09-29:** Reine
  Verschiebung ohne Aenderung an Ablauf oder Ausgabe. `src/audit/pipeline.rs` ist jetzt das
  Verzeichnis `src/audit/pipeline/`: `mod.rs` behaelt `PipelineConfig`, `run_single_audit`,
  `audit_page` mit beiden Viewport-Durchlaeufen, die Regellaeufe und die Artefakt-Persistenz;
  `assembly.rs` nimmt die synchrone Nachbearbeitung auf (WCAG-Zusammenfuehrung, Viewport-Scores,
  `aggregate_report`, Modul-Laeufe, Audit-Qualitaet); `throttled.rs` die gedrosselten
  Performance-Durchlaeufe samt `recover_after_throttled_failure` und der Uebernahme des
  LhMobile-Werts. In `audit_page` sind nur die wortgleichen Doppelungen herausgezogen (die drei
  `prepare_*_collection`-Aufrufe je Durchlauf, der Screenshot-Match); Reihenfolge von
  Ausschluessen (#645), Darstellungsmodi (#653) und Journeys unveraendert, ein gemeinsamer
  `run_viewport_pass` bewusst nicht. Oeffentliche Pfade (`auditmysite::audit::{audit_page,
  run_single_audit, PipelineConfig}`) bleiben. Verifiziert: Golden-Harness vor/nach identisch,
  `integration_test` und `detection_corpus_test` (`--ignored`) gegen echtes Chrome.
- **Refactoring: Modul-Details und Page Health als Verzeichnismodule (Plan 66 WP6), 2026-09-29:**
  Reiner Umbau ohne Verhaltensaenderung. `output/builder/single/module_details.rs` (2.618 Zeilen)
  ist jetzt ein Verzeichnismodul mit einer Datei je Modul, gespiegelt an `output/pdf/detail_modules/`:
  `performance.rs`, `seo.rs`, `platform.rs` (Security, Mobile), `html_conform.rs`, `commerce.rs`,
  `dark_mode.rs`, `design_quality.rs`, `ai_transparency.rs`, `experience.rs` (UX, Journey); die
  Orchestrierung (`build_module_details_from_normalized`) bleibt in `module_details.rs`. Die
  Inline-Berechnungen im `SeoPresentation`-Literal sind benannte Funktionen (`heading_summary`,
  `social_summary`, `technical_summary_rows`, `tracking_summary_rows`,
  `build_robots_presentation`, `build_image_efficiency_presentation`, `technical_issue_rows`).
  `seo/page_health.rs` (3.128 Zeilen) behaelt Typen, Textfunktionen, `analyze_page_health` und die
  URL-Analyse; neu sind `page_health/dom.rs` (DOM-Inspektion, lokale HTML-Validierung),
  `page_health/probes.rs` (HTTP-Proben) und `page_health/issues.rs` (`collect_issues`). Alle
  oeffentlichen Pfade bleiben (`seo::page_health::collect_issues` per `pub use`). Die Umbenennung
  `run_w3c_html_validation` → `run_local_html_validation` war bereits mit WP9 erledigt.
  Verifiziert mit dem Golden-Harness (WP0) gegen `main`: `diff -r` leer.

- **Darstellungsmodi (`data-display`-Konvention), 2026-09-29 (#653):** Neues Modul `src/display/`
  und neue Option `--display calm|text|visual|all`. Ohne Option bleibt alles wie bisher (Voreinstellung
  der Seite). Mit einem Modus schreibt ein vor der Navigation injiziertes Skript
  (`Page.addScriptToEvaluateOnNewDocument`) die Wahl nach `localStorage.display`; `calm` und `text`
  emulieren zusaetzlich `prefers-reduced-motion: reduce` (`Emulation.setEmulatedMedia`), damit auch
  Seiten ohne Konvention ihre reduzierte Variante zeigen. Weil `setEmulatedMedia` die ganze
  Feature-Liste ersetzt, reicht die Dark-Mode-Analyse (Druck, Forced Colors, Dunkel-Kontrast) die
  Grund-Features jetzt durch (`analyze_dark_mode_with_base_media`) — sonst haette sie die reduzierte
  Bewegung fuer den Rest des Audits geloescht. JSON: `audit_scope.display_mode`
  (`site_default|visual|calm|text`) und je Seite `pages[].display_modes` (angebotene Modi,
  dargestellter Modus, `set_before_body`, Umschalter, Visualisierungen je Art, reduzierte Bewegung);
  PDF: Zeilen „Darstellungsmodus"/„Darstellungsmodi der Seite" im Methodik-Anhang, im Batch auf dem
  Deckblatt. `--display all` prueft jeden Modus als eigenen Lauf mit eigenem Bericht (Suffix
  `-calm`/`-text`/`-visual`), nie ein gemischter Score; `visual` nur fuer Seiten mit
  `figure[data-viz="3d|interactive"]` aus dem `calm`-Lauf, mit doppeltem Seiten-Timeout. Befunde
  werden nicht als „in allen Modi" markiert — dafuer muessten die Laeufe zusammengefuehrt werden.

  Neue Regelgruppe `display/*` (Best Practice, keine WCAG-Anforderung, verankert an 2.2.2 bzw.
  1.1.1; eigene Taxonomie-Eintraege und Erklaerungen): `display/toggle-missing`,
  `display/init-missing` (ueber einen Beobachter, der beim Einfuegen von `<body>` festhaelt, ob
  `data-display` schon gesetzt war), `display/text-media-visible`, `display/text-not-visible`,
  `display/text-hidden`. Die Textmodus-Pruefungen richten sich nach dem tatsaechlich dargestellten
  Modus. Umzugskandidaten fuer barrierlab: die statischen Haelften von `toggle-missing` und
  `text-hidden` (Attribut-Teil = `viz/text-hidden` des Entwurfs). Korpus-Fixtures
  `display_modes_conforming`/`display_modes_violations`, Integrationstest
  `display_text_sets_stored_choice_and_reduced_motion` (localStorage und `matchMedia` vor dem
  ersten Skript und nach dem Audit).

  Live gegen geographia.eu: `--sitemap .../sitemap-index.xml --display calm -m 10` 266 s, 10/10
  Seiten, keine Pool-Timeouts (die ersten 10 URLs sind Textseiten ohne Visualisierung). Zehn Seiten
  mit Visualisierungen per URL-Liste: `calm` 1151 s, 7/10, 0 Pool-Timeouts, aber 3 Seiten mit
  `interactive` laufen in das 120-s-Audit-Limit; `text` 293 s, 10/10, 0 Timeouts. Befunde: auf allen
  Seiten `display/toggle-missing` (kein `[data-display-toggle]` ausgeliefert); in `calm`/`visual`
  `display/text-hidden`, weil `.viz-text { display: none }` die Textschicht ausserhalb von `text`
  auch fuer Screenreader entfernt (im Browser bestaetigt); in `text` `display/text-not-visible` auf
  Seiten ohne `[data-viz-text]` (Startseite 4, Erklaerungs-Labor 10).

- **Englischer Bericht: Fuellwoerter als Themen, irrefuehrender Validator-Titel, 2026-09-29:**
  Befund aus einem Scan der englischen Typst-Ausgaben des Golden-Sets (17 Seiten).
  (1) Der Batch-Bericht fuehrte unter „Dominant content topics" den Eintrag „nicht" („Present on
  2 audited pages"). Ursache: `seo/topics.rs` filtert Seitentitel, Beschreibung, Ueberschriften und
  Textauszug gegen `topic-stopwords` aus den FTL-Dateien, und diese Listen enthielten nur einen
  Teil der Funktionswoerter — „nicht" (barrierlab.eu, casoon.de), „weitere" (berlin.de),
  „dein"/„dich" (hornbach.de, otto.de) und „neue" (volkswagen.de) rutschten als Seitenthemen durch.
  Beide Listen um gaengige Funktionswoerter (Negation, Hilfs- und Modalverben, Pronomen,
  Konjunktionen, Praepositionen) mit mindestens vier Zeichen ergaenzt; kuerzere filtert der Code
  ohnehin. Echte Themenwoerter bleiben. (2) Der Callout zur HTML-Validierung hiess „W3C HTML
  validator"/„W3C HTML Validator", obwohl die Pruefung lokal mit html5ever laeuft und nie den
  W3C-Dienst aufruft. Jetzt „HTML validation (local, html5ever)" bzw. „HTML-Validierung (lokal,
  html5ever)"; der FTL-Schluessel heisst `pdf-ph-html-validator-title`. (3) Die deutschen Zeilen
  der HTML-Validierungstabelle im englischen PDF waren bereits mit #671 behoben. (4) Die
  Terminal-Ausgabe zeigte mit `--lang en` „Certificate: SEHR GUT". Das Zertifikat im JSON bleibt
  bewusst das kanonische deutsche Token (#449: Schluessel fuer Badge/Farbe, per Enum im
  Studio-Vertragsschema festgelegt, von report-lint geprueft und von rankinglab ausgewertet); nur
  die Anzeige wird lokalisiert. (5) Der Validator-Callout verglich den lokalisierten Status mit
  „Fehlgeschlagen"; im englischen PDF („Failed") wurde eine fehlgeschlagene Validierung deshalb
  als Info statt als Warnung gezeigt. Die Praesentation traegt jetzt den kanonischen Status
  (`executed`/`failed`/`skipped`), der Callout entscheidet darueber. `certificate_label_localized` liegt jetzt in `registry` statt im
  PDF-Cover, damit auch das Terminal es nutzt: mit `--lang en` steht dort „EXCELLENT". Ebenso
  zeigte die Zeile „Certificate" im Kasten „Audit scope" des englischen Batch-Berichts das
  deutsche Token („AUSBAUFÄHIG"); sie nutzt jetzt dieselbe Funktion („INADEQUATE").

  **Aendert die Ausgabe:** Seitenthemen (`topic_terms`, `top_terms`, Themen-Ueberschneidungen) im
  JSON und in beiden PDF-Sprachen, wo eine Seite ein Funktionswort unter ihren fuenf Themen hatte;
  Titel des Validator-Callouts in beiden Sprachen; Zertifikatszeile im englischen Terminal und im
  englischen Batch-Kasten „Audit scope". Das JSON-Feld `certificate` bleibt unveraendert.
  Verifiziert mit dem Golden-Harness gegen `main`: Unterschiede nur an diesen Stellen, deutsches
  Typst bis auf Validator-Titel und Themenliste wortgleich. Unit-Tests gegen Funktionswoerter als
  Themen und gegen Umlaute in den englischen Zertifikats-Labels.
- **`document-title` doppelt gezaehlt, 2026-09-29:** Ein fehlender oder leerer Seitentitel stand
  zweimal im Bericht, einmal aus der AX-Pruefung `check_page_titled` (Knoten `document`, ohne
  Selektor) und einmal aus der DOM-Pruefung `check_page_titled_with_page` (Selektor `head`).
  Massgeblich ist jetzt allein die DOM-Pruefung: Den Namen des Wurzelknotens fuellt Chrome bei
  fehlendem Titel mit der URL, die AX-Pruefung konnte diesen Fall deshalb selbst nicht
  entscheiden und verwies schon bisher auf die DOM-Pruefung. Alles, was sie als Verstoss meldete
  (leer, fehlend, generischer Titel), meldet die DOM-Pruefung ebenfalls; die AX-Pruefung ist
  samt Tests entfernt (oeffentliche Funktion `wcag::rules::check_page_titled` entfaellt).
  `empty_title` pinnt 1 Vorkommen, der Integrationstest auf `parity_gaps.html` ebenfalls.

- **Tabellen: ignorierte und noch nicht gerenderte Zeilen (#654, #659), 2026-09-29:**
  (1) `aria-roles` meldete auf og-vanilla.casoon.dev (`/accessibility`, `table.keys`) „role 'table'
  is missing required child roles: row, rowgroup" fuer eine Tabelle mit `tbody > tr`. Chrome
  markiert ein schlichtes `<tbody>` als ignoriert, die Zeilen haengen darunter; die Pruefung der
  geforderten Kind-Rollen sah nur direkte Kinder. Sie schaut jetzt durch ignorierte Knoten hindurch
  (gleiches Muster wie #638). Eine Tabelle ohne Zeile (nur `caption`) bleibt gemeldet.
  (2) `th-has-data-cells` meldete die Kopfzellen des `role="grid"`-Rasters in einer Shadow-Root
  (`/columns`, `/accessibility`), obwohl jede Spalte Daten hat. Im Nachstellen per CDP lag die
  Aufnahme vor dem Eintreffen der Daten: Das Raster haelt 40 Pool-Zeilen mit `display: none` im
  DOM, bis der Worker liefert (rund 300-500 ms nach `load`); Chrome fuehrt sie als ignoriert mit
  `notRendered`. Die Stabilitaetswartung sieht Mutationen in Shadow-Roots nicht und meldete die
  Seite nach 255 ms stabil. Stehen in einer Tabelle ohne Datenzellen solche nicht gerenderten Zeilen
  an Zeilenposition (direkt unter der Tabelle, unter einer rowgroup oder einem ignorierten
  `<tbody>`), gilt das Ergebnis jetzt als unbestimmt (`incomplete`) statt als Verstoss. Versteckte
  Inhalte innerhalb einer Kopfzelle zaehlen nicht; eine Tabelle nur mit Kopfzeile bleibt gemeldet.

  Neue Korpus-Fixtures `table_required_rows_tbody` und `table_grid_rows_unrendered` (Shadow-DOM-
  Raster mit wartenden bzw. gerenderten Zeilen, Tabelle nur mit Kopfzeile als Verstoss) und
  Unit-Tests in beiden Regeln.
- **Technician-Modus: Einzelseiten-JSON zum Beheben statt Bericht zum Lesen, 2026-09-29 (Plan 67):**
  Neu fuer Leute, die Befunde abarbeiten: `--sitemap URL --technician -o DIR/` (ebenso mit
  `--url-file`/`--crawl`) schreibt pro Seite den regulaeren Einzelbericht als JSON und daneben zwei
  flache Dateien. `index.json` fuehrt jede versuchte URL in Eingabereihenfolge — Dateiname oder
  `null`, Status `ok`/`blocked`/`failed` mit Grund, Gesamt- und Barrierefreiheits-Score, Zahl der
  Befunde und Vorkommen, `audit_quality`. Gesperrte (Bot-Wall, `AccessBlocked`) und
  fehlgeschlagene Seiten stehen nur dort, ohne Platzhalterdatei; dafuer traegt `BatchError` jetzt
  `blocked_reason` (nicht serialisiert, Batch-JSON unveraendert). `findings.jsonl` enthaelt eine
  Zeile pro Vorkommen ueber alle Seiten (`url`, `source` wcag/journey/seo/html_conform, `rule_id`,
  `wcag_criterion`, `level`, `severity`, `selector`, `location`, `message`, `fix_suggestion`,
  `viewport_tags`), vollstaendig — die Seitendatei kappt ihre Beispiel-Vorkommen pro Befund auf
  fuenf. Beide Dateien sind kanonisch Englisch (#406), Schemas unter
  `docs/technician-index.schema.json` und `docs/technician-finding.schema.json`. Jeder Lauf mit
  `--per-page-reports -f json` schreibt sie.

  `--technician` steht fuer `--per-page-reports -f json --no-screen-reader-report --seo
  --html-conform`: Barrierefreiheit (inkl. Tastatur-Journeys), HTML-Konformitaet und SEO, keine
  gedrosselten Performance-Durchlaeufe, kein Mobile/Security/Tech-Stack. Jede Seitendatei nennt
  diesen Umfang in `execution.scope`/`module_runs`. Ausdruecklich gesetzte Flags gewinnen: `-f`
  ersetzt das Format, `--full`/`--performance`/`--mobile`/`--security` schalten ihre Module wie
  gewohnt zu. Einzeln nutzbar sind die neuen Flags auch: `--include-path`/`--exclude-path`
  (wiederholbar, Glob auf den dekodierten URL-Pfad: `*`/`?` innerhalb eines Segments, `**`
  segmentuebergreifend, `/**/` auch als einzelnes `/`; wirkt vor `-m`; `sample.selection` wird
  dann `path_filter`), `--no-screen-reader-report` (kein `*-screen-reader-audit.json`, auch im
  Einzelmodus) und `--html-conform` (HTML-Konformitaet ohne `--full`).

  Nicht in `findings.jsonl`: `accessibility_assessments` (Hinweise/manuelle Pruefpunkte, keine
  Verstoesse). Die abgeleiteten Indikator-Module (UX, Journey, Commerce, Content Visibility, AI
  Visibility, Source Quality, Dark Mode) laufen weiter, weil sie an `check_seo` bzw. fest an
  `PipelineConfig` haengen; sie abzuschalten braeuchte neue `PipelineConfig`-Felder.
- **3.2.2 On Input: nur echte Kontextwechsel sind Verstoesse, 2026-09-29 (#657):** Auf
  og-vanilla.casoon.dev (`/filtering`, `/sorting`, `/localization`) meldete
  `a11y.on_input.risk` Selects fuer Filter-Preset, Sortierung und Sprache als „may trigger
  navigation", nur weil ihr Name „filter"/„sort"/„language" enthielt und die Seite keinen
  Absende-Button hat. Die Selects aktualisieren aber nur das Grid daneben — eine Inhaltsaenderung
  ist kein Kontextwechsel. Jetzt ist nur noch ein Verstoss, was der Inline-`onchange`-Handler
  (oder die globale Funktion, die er direkt aufruft) sichtbar tut: Navigation (`location…`),
  Formular absenden (`.submit(`/`.requestSubmit(`), neues Fenster (`window.open(`) oder
  Fokusverschiebung (`.focus(`). Ein Handler, dessen Wirkung sich nicht ablesen laesst, und ein
  Name, der nur nach Navigation klingt (language, country, region, navigate, redirect, go to),
  werden zur Pruefwarnung statt zum Verstoss; „sort" und „filter" fallen als Hinweis ganz weg.
  Die Einordnung liegt jetzt in Rust (`evaluate`) und ist unit-getestet. Neue Korpus-Fixture
  `on_input_context_change` (besteht: Filter ohne Handler; Pruefung: inhaltsaendernder Handler,
  Sprach-Select; Verstoss: Handler navigiert ueber aufgerufene Funktion, Handler sendet Formular).

- **1.3.5 Identify Input Purpose: „Name" einer Sache ist kein Personenname, 2026-09-29 (#658):**
  Auf og-vanilla.casoon.dev/saved-views galt `#view-name` (Label „Name", Name einer gespeicherten
  Grid-Ansicht) als Feld fuer den Namen der Nutzerin, weil das Label „name" als Teilstring
  enthielt. „name" wird jetzt wortweise eingeordnet (Trennung an Satzzeichen und camelCase): ein
  vorangestelltes Wort, das keine Person und kein technisches Id-Praefix ist („view name",
  „project name", „company name"), ein folgendes „of/for/der/des/für/von" („Name der Ansicht")
  und Komposita wie „Dateiname"/„Filename" schliessen das Feld aus; „Name", „Your name",
  „Full name", „Vorname", „Nachname", „Ihr Name" bleiben Treffer. Ein Sach-Qualifier in `id` oder
  `name`-Attribut schliesst ein blosses „Name"-Label aus, sofern das andere Attribut nicht die
  Person nennt. Die Erkennungswoerter sind Englisch und Deutsch zusammengefuehrt (wie bei
  `media_alternative`), unabhaengig von der Ausgabesprache. Neue Korpus-Fixture
  `input_purpose_name`.
- **Absichtlich kaputte Beispiele vom Audit ausnehmen, sichtbar im Bericht, 2026-09-29 (#645):**
  Seiten, die Barrierefreiheit lehren, zeigen bewusst fehlerhafte Beispiele (barrierlab.eu,
  `/tasks/ticket/`: unbeschriftete Felder, ein `div` als Button). auditmysite meldete sie als
  eigene Verstoesse der Seite. Neu, nach dem Vorbild von axe-cores `exclude`:
  `--exclude-selector <CSS>` (wiederholbar), `[audit] exclude_selectors` in `auditmysite.toml`,
  und das immer beachtete Attribut `[data-audit-exclude]`. Ausgenommen wird nie eine Seite und
  nie eine Regel, sondern nur Befunde, deren Element in einem ausgeschlossenen Teilbaum liegt —
  die Seite selbst wird vollstaendig geprueft, seitenweite Befunde bleiben immer.

  Umsetzung (`src/audit/exclusion.rs`): je Viewport-Durchgang zaehlt ein `Runtime.evaluate` die
  Treffer jedes Selektors (ungueltige Selektoren werden erkannt); nur bei Treffern holt
  `DOM.getDocument` + `DOM.querySelectorAll` die Backend-Node-IDs aller Knoten der getroffenen
  Teilbaeume (inkl. Shadow Roots, Frame- und Template-Inhalt). Gefiltert wird je Regel in
  `run_rules`, bevor ihr Ausfuehrungsvermerk gezaehlt wird, und vor Anreicherung und
  Element-Screenshots: Befunde aus dem AX-Baum ueber die
  Backend-ID ihres AX-Knotens, Befunde der geteilten DOM-Regeln ueber die neu mitgefuehrte
  Backend-ID (`Violation::backend_node_id`, nur im Speicher; gesetzt im Adapter
  `wcag/shared.rs`, am Regelcode von barrierlab aendert sich nichts), JavaScript-Seitenregeln,
  Kontrast und Journey-Befunde ueber ihren Selektor — ausgenommen nur, wenn er mindestens ein
  Element trifft und **jedes** davon im ausgeschlossenen Teilbaum liegt; ein mehrdeutiger oder
  nicht parsebarer Selektor behaelt den Befund. Muster-Befunde laufen durch denselben Filter.

  Sichtbarkeit: Das JSON fuehrt je Seite `pages[].exclusions` (angewandte Selektoren mit
  Trefferzahl — auch 0 und ungueltig —, ausgeschlossene Vorkommen und bestaetigte Verstoesse
  gesamt und je Regel, ausgeschlossene Journey-Befunde; je Regel der groessere Wert der beiden
  Viewports), der Batch zusaetzlich `summary.exclusions` als Summe. Das PDF nennt Selektoren und
  Zahlen im Methodik-Teil, der Batch im Audit-Rahmen des Deckblatts (de/en ueber Fluent). Ein
  Standardlauf ohne `data-audit-exclude` auf der Seite bleibt im PDF still, das JSON listet den
  eingebauten Selektor mit 0 Treffern. Cache-Format 19; die Selektoren gehen in die
  Audit-Signatur ein.

  Obergrenzen der JavaScript-Regeln: Viele Seitenregeln melden hoechstens N Fundstellen (10 bei
  `click-events-have-key-events`, 20 bei `link-as-button`, 250 bei den ARIA-Regeln, 5 bei
  Zielgroesse, verdeckt fokussierten Elementen, Sprachwechseln u. a.). Ohne Gegenmassnahme haetten
  Beispiel-Treffer die Obergrenze aufgebraucht und echte Treffer dahinter still verschwinden
  lassen — genau das, was die Funktion ausschliessen soll. Deshalb legt der Ausschluss-Schritt je
  Durchgang `window.__amsIsExcluded(el)` in die Seite (die getroffenen Wurzeln in einem
  `WeakSet` im Closure, kein DOM-Attribut, keine fuer andere Regeln sichtbare Aenderung). Der
  gemeinsame Baustein `CSS_SELECTOR_JS`, den jede lokalisierende Regel einbindet, bringt
  `__amsIsExcludedEl`, `__amsPush` und `__amsReal` mit: Echte und ausgeschlossene Treffer werden
  getrennt gedeckelt, die Schleifengrenze liest nur die echten. Ausgeschlossene Treffer kommen
  weiter zurueck und werden wie alle anderen gefiltert und gezaehlt (je Regel hoechstens bis zur
  selben Obergrenze). Umgestellt: `aria_allowed_attr`, `aria_hidden_focus` (deren Gesamtzahl
  zaehlt Ausgeschlossene nicht mehr mit), `aria_relationships`, `aria_roles` (2),
  `aria_required_attr`, `image_input_rules`, `on_focus`, `on_input`, `server_side_image_map`,
  `table_extended`, `widget_rules`, `click_handlers`, `content_on_hover`, `fake_navigation_link`,
  `redundant_role`, `use_of_color`, `language_of_parts`, `meaningful_sequence`,
  `target_size_minimum`/`_enhanced` (auch die Kandidatenlisten), `focus_not_obscured_minimum`/
  `_enhanced` (Fokus-Kandidaten, die 60 geprueften Bedienelemente, die 5 Befunde),
  `pause_stop_hide` (Widgets; die seitenweite Animations-Meldung ignoriert ausgeschlossene
  Elemente ganz) und `accessible_authentication` (Obergrenze von Rust ins Skript verlegt).
  `identify_purpose` meldet einen Feldnamen statt eines CSS-Pfads — das Skript markiert
  ausgeschlossene Felder selbst (`Violation::in_excluded_subtree`, nur im Speicher), die
  Obergrenze 5 gilt getrennt. Kontrast: Text in ausgeschlossenen Teilbaeumen belegt keinen der 60
  Plaetze fuer die Pixelprobe mehr.

  `rule_outcomes[].findings` zaehlt jetzt, was im Bericht steht: Die Baum-Regeln filtern je Regel
  in `wcag::check_all_excluding` (neu; `check_all_with_config` bleibt als Huelle), die
  Seitenregeln, Kontrast und HTML-Inhaltsmodell vor `page_rule_outcome`; bei den geteilten
  Regeln wird der aus `a11y-rules` uebernommene Zaehler um die entfallenen Befunde derselben
  Kennung verringert.

  Bleibt offen: Regeln, deren gemeldeter „Selektor" kein CSS-Pfad ist und die ihr Element nicht
  selbst markieren, koennen ausgeschlossene Treffer nicht verlieren (bekannt und behoben:
  `identify_purpose`). Obergrenzen, die nur Kandidaten fuer eine seitenweite Aussage deckeln
  (Overlay-Suche in `focus_not_obscured_*`, Felder je Formular in `redundant_entry`), sind
  unveraendert.

  Tests: Unit-Tests fuer Filter, Zaehlung und PDF-Text (EN ohne Umlaute), neue Korpus-Fixture
  `audit_exclude_specimen` (je ein Fehler pro Ortungsweg — AX-Knoten, JS-Seitenregel, geteilte
  DOM-Regel — innerhalb von `[data-audit-exclude]` nicht gemeldet, derselbe Fehler ausserhalb
  gemeldet) und ein Integrationstest fuer `--exclude-selector` samt 0-Treffer- und
  ungueltigem Selektor. Dazu Korpus-Fixture `audit_exclude_cap`: 12/22/6 ausgeschlossene Treffer
  vor je einem echten (Obergrenzen 10/20/5) — der echte wird gemeldet, und
  `excluded_hits_do_not_spend_a_capped_rules_budget` prueft dasselbe samt Ausschluss-Zaehlung und
  `rule_outcomes[].findings == 1`. Gegenprobe: mit abgeschaltetem `__amsIsExcludedEl` faellt der
  Test (`div#real-click` fehlt).

- **4.1.2/3.3.2: aufklappbare Treegrid-Zeilen und native Wertfelder, 2026-09-29 (#655, #656):**
  (1) Das Accordion-Muster hielt jedes Element mit `aria-expanded` fuer einen Accordion-Ausloeser
  und meldete auf og-vanilla.casoon.dev/grouping jede Gruppenzeile des `role="treegrid"` als
  „should be a button" (Medium, 12 Vorkommen). `aria-expanded` auf einer Zeile in `grid`/`treegrid`
  und auf einem `treeitem` ist der Zustand des eigenen zusammengesetzten Widgets, dessen
  Tastaturvertrag (Pfeiltasten, Enter) das Grid bzw. der Baum stellt; solche Knoten zaehlen nicht
  mehr als Ausloeser. Eine aufklappbare Zeile ausserhalb eines Grids wird weiter gemeldet.

  (2) `aria-required-attr` pruefte `aria-valuenow` ueber eine AX-Eigenschaft `valuenow`, die CDP
  nicht kennt — der aktuelle Wert steht im `value` des Knotens, und Chrome erfindet einen (50 beim
  Slider, 0 beim Spinbutton), wenn der Autor keinen setzt. Die Pruefung schlug deshalb bei jedem
  `slider`/`spinbutton`/`meter`/fokussierbaren `separator` an, auch bei nativem
  `<input type=range|number>` und den internen Tag/Monat/Jahr-Feldern von `<input type=date>`
  (Critical; auf /filtering, /columns, /grouping, /theming 16 Vorkommen). Aus demselben Grund
  meldete `check_slider_has_value` („Slider is missing accessible value") jeden Slider: Der
  Extraktor liest nur Text-Werte, die Zahl des Sliders fiel weg. Die `aria-valuenow`-Pflicht
  laeuft jetzt als DOM-Regel `check_value_now_with_page` (wie schon `aria-checked`): sie liest das
  Attribut, durchlaeuft offene Shadow Roots, nimmt natives `range`/`number` und `<meter>` aus und
  erreicht die User-Agent-Felder von Datumsfeldern gar nicht. Die Slider-Pruefung in
  `widget_rules.rs` entfaellt; der eigene `role="slider"` ohne `aria-valuenow` bleibt ein Verstoss.

  (3) Die 3.3.2-Heuristik „may require format instructions" uebergeht Knoten innerhalb der
  Chrome-Rollen `Date`/`DateTime`/`InputTime` — deren Felder und Format stellt der Browser. Die
  Rolle `spinbutton` allein loest den Hinweis nicht mehr aus: 3.3.2 verlangt Anleitungen, wo die
  Eingabe einem Format folgen muss, das man nicht erschliessen kann; ein Spinbutton — natives
  `<input type=number>` wie eigenes `role="spinbutton"` — haelt eine Zahl, die das Widget selbst
  begrenzt und mit den Pfeiltasten schrittweise aendert. Die Erkennung ueber die Beschriftung
  („Date", „Postal code") gilt weiter fuer jede Rolle.

  (4) Der AX-Extraktor las `value` nur als Zeichenkette; Chrome schickt den Wert von Slider,
  Spinbutton, Progressbar und Meter als Zahl, der damit verloren ging. Zahlen und Wahrheitswerte
  bleiben jetzt in Textform erhalten. Keine WCAG-Regel liest `AXNode.value` mehr (die einzige,
  `check_slider_has_value`, ist oben entfallen); Folge hat es nur fuer die Screenreader-Linearisierung
  (`a11y-perception`): Wertelemente ohne Namen tragen jetzt ihren Wert und zaehlen in der
  Ansage-Wuesten-Messung als angesagter Inhalt, was sie beim Vorlesen auch sind.

  Live nachgeprueft mit dem Release-Build gegen og-vanilla.casoon.dev: keine Accordion-, keine
  `aria-valuenow`-, keine Slider-Wert- und keine Spinbutton-Format-Befunde mehr. Neue Korpus-Fixtures `treegrid_expandable_rows`,
  `value_widgets_native` (bestehen), `value_widgets_custom` und `value_widgets_shadow` (Verstoss);
  `aria_and_widgets` erwartet den eigenen Slider jetzt unter `aria-required-attr`.

  Nachzug derselben Fehlerklasse: Auch die uebrigen 1.3.5-Stichwoerter wurden als Teilstring
  gesucht — „Hotel" galt wegen „tel" als Telefonfeld, „Sorting" haette „ort" enthalten. Sie gelten
  jetzt nur als ganze Woerter (tel, zip, plz, fax, city, first, last …) oder als bekannte
  Kompositum-Anfaenge bzw. -Enden (`telefon…`, `postleit…`, `birth…`, `geburts…`, `…adresse`,
  `…address`, `…strasse`); „E-Mail" zaehlt als ein Wort. Deutsche Felder wie „Telefonnummer",
  „Postleitzahl", „Straße", „Passwort" werden damit erstmals erkannt. Ebenso bei 3.2.2: die
  seitenweite Suche nach einem Absende-Button fand „go" in „Google", „Category" oder „Logo" und
  unterdrueckte dadurch die Pruefwarnung; die Button-Namen werden jetzt wortweise geprueft
  (submit, send, go, search, absenden, senden, suchen), die Entscheidung liegt in Rust und ist
  unit-getestet. Korpus: `#stay` („Hotel") besteht, „Telefonnummer"/„Postleitzahl" sind
  Verstoesse; die On-Input-Fixture hat Google/Category/Logo-Buttons, der Sprach-Select bleibt
  Pruefung.

- **`landmark-unique` doppelt gezaehlt, 2026-09-29:** Die Regel lief zweimal, einmal ueber den
  AX-Baum (`check_landmark_unique`) und einmal in der DOM-Ergaenzung (`check_landmarks_with_page`).
  Beide meldeten dieselben Elemente mit unterschiedlich gebildeten Selektoren, sodass nichts
  zusammengefuehrt wurde: Der Korpusfall `landmark_granular` kam auf 22 statt 11 Vorkommen,
  `parity_gaps.html` auf 4 statt 2. Massgeblich ist jetzt allein die AX-Pruefung; sie arbeitet
  mit Chromes berechneten Rollen und Namen und erfasst jede Landmark, die die DOM-Naeherung sieht
  (dazu `<section>` mit Namen, `form`, und korrekte Rollen fuer verschachtelte `header`/`footer`/
  `aside`). Dasselbe galt fuer `landmark-main-present`: Eine fehlende `main`-Landmark stand
  zweimal im Bericht (`root` und `document`). Die DOM-Ergaenzung hatte dabei keine eigene
  Abdeckung: Faellt der AX-Baum aus, bricht der ganze Audit ab; bei Mini-Seiten mit hoechstens
  zwei AX-Knoten schweigt die AX-Pruefung absichtlich; und ein `<main>` in einem Shadow Root sah
  nur Chrome, `querySelectorAll` nicht, sodass die DOM-Pruefung dort faelschlich meldete. Die
  DOM-Ergaenzung (`check_landmarks_with_page`, Page-Regel `1.3.1/landmark-dom`) ist deshalb samt
  Rollenableitung entfernt. Der Korpus kann jetzt per `occurrences` die genaue Anzahl pinnen:
  `landmark_granular` 11, der neue Fall `landmark_unique_one_per_element` 4 (1.6.0: 6),
  `landmark_aside_named_duplicate` 2, `missing_main_landmark` 1 (1.6.0: 2); der neue Fall
  `landmark_main_in_shadow_root` ist ein echtes Negativ. `parity_gaps.html` prueft die Anzahl im
  Integrationstest (1 fehlende `main`, 2 doppelte Navigationen).

- **Page Health: HTML-Validierung im JSON kanonisch Englisch (#406, Plan 66 WP9), 2026-09-29:**
  `seo/page_health.rs` schrieb deutschen Text ins kanonische JSON: `html_validator_detail`
  („HTML5-Validierung lokal via html5ever") sowie `check`/`detail` aller `html_issues`
  („Doppelte IDs", „3 (z.B. …)", „HTML5-Parsing-Fehler", „+37 weitere", „Leere Ueberschriften"
  usw.). `reconcile_image_alt_count` in `audit/pipeline.rs` verglich sogar gegen den deutschen
  String „Bilder ohne alt-Attribut". Jetzt nach dem #406-Muster: `HtmlValidationIssue` traegt ein
  `kind` (`HtmlValidationKind`) und die Rohwerte in `samples` (Beispiel-IDs bzw. die ersten
  Parser-Fehler); `html_validation_check_text`/`html_validation_detail_text`/
  `html_validator_executed_text` sind die einzige Textquelle, die Analyse ruft sie mit `en=true`,
  der PDF-Builder (`builder/single/serp.rs`) mit der Lauf-Sprache. Vergleiche laufen ueber `kind`.
  Die private `run_w3c_html_validation` heisst jetzt `run_local_html_validation` (sie ruft nie den
  W3C-Validator auf).

  **Aendert die JSON-Ausgabe:** `html_issues[].check`/`detail` und `html_validator_detail` sind
  englisch, jeder Eintrag hat neu `kind`, Eintraege mit Rohwerten zusaetzlich `samples`. Das
  deutsche PDF bleibt wortgleich; das englische PDF zeigt dort jetzt englischen Text statt des
  bisherigen deutschen. Cache-Eintraege derselben Version mit altem deutschem Text werden beim
  Einlesen auf `kind` + `samples` zurueckgefuehrt. Verifiziert mit dem Golden-Harness (WP0) gegen
  `main`: Unterschiede nur in diesen Feldern im JSON und im englischen Typst, deutsches Typst und
  Batch-Ausgaben unveraendert; Guard-Test gegen Umlaute/deutsche Woerter in der EN-Ausgabe.

- **2.5.8/2.5.5 Target Size: Ausnahme „Equivalent" fuer Links, 2026-09-29 (#652):** Beide
  Kriterien nehmen ein zu kleines Ziel aus, wenn dieselbe Funktion ueber ein anderes Bedienelement
  auf derselben Seite erreichbar ist, das die Groesse erfuellt. Das fehlte: auf geographia.eu
  (`/atmosphere/de/kapitel/hebel/`) galt die 39×10 px kleine Absenderzeile des Modul-Logos
  (`a.logo-main`, `href="/de/"`) als Verstoss, obwohl der Footer-Link „Alle Sphaeren im Ueberblick"
  (`/de/#modules`, 141×161 px) dieselbe Startseite oeffnet. Jetzt besteht ein zu kleiner Link, wenn
  ein anderer Link mit demselben Ziel mindestens 24×24 (2.5.5: 44×44) misst, gerendert und sichtbar
  ist (`checkVisibility` inkl. `visibility`/`opacity`, nicht links/oberhalb der Seite verschoben),
  Pointer-Events hat und nicht in einem `inert`- oder `aria-hidden="true"`-Teilbaum liegt.

  Das Ziel ist die aufgeloeste absolute URL. Fuehrt der Link in ein anderes Dokument, zaehlt das
  Fragment nicht — `/de/` und `/de/#modules` oeffnen dieselbe Seite, das Fragment setzt nur die
  Scrollposition. Fuehrt er in die aktuelle Seite, zaehlt es: dort ist das Springen an die Stelle
  die ganze Funktion, `#a` und `#b` sind verschieden. `href=""`, `href="#"` und `javascript:`
  zaehlen nie — sie nennen kein Ziel. Ein ebenfalls zu kleiner Link ist kein Aequivalent, auch wenn
  er selbst ueber die Abstands-Ausnahme besteht (bewusst konservativ). Buttons bleiben aussen vor,
  ihre Funktion steht nicht im Markup. Der Baustein `hasEquivalentLink` liegt in
  `TARGET_HELPERS_JS`, beide Regeln nutzen ihn. Die Regel fuehrt keine Pass-Liste, daher wird das
  ausgleichende Element nicht vermerkt.

  Neue Korpus-Fixture `target_size_equivalent` (besteht: kleiner Link mit grossem Aequivalent,
  auch mit anderem Fragment in ein anderes Dokument; Verstoss: nur versteckte/inerte/unsichtbare/
  verschobene Aequivalente, nur zu kleine, anderes Fragment in die eigene Seite, `href="#"`, ohne
  Aequivalent) und ein Integrationstest, der beide Regeln direkt aufruft.
- **Batch: Pool-Wartezeit und still fehlende Seiten (#651), 2026-09-29:** Ein Batch ueber
  geographia.eu (`-c 2 -t 120`, WebGL-Seiten) verlor 46 von 312 Seiten mit „Browser pool timeout:
  no page available after 60 seconds". Nachgestellt mit 1.6.0 auf 8 Kapitelseiten: 3 von 8
  fehlten. Zwei Ursachen: (1) `BrowserPool::acquire` zaehlt den Slot, bevor die neue Seite
  angelegt ist; schlug `new_page` fehl (Chrome unter der WebGL-Last, CDP-Anfrage laeuft ab),
  gingen Semaphor-Genehmigung und Slot dauerhaft verloren — der Pool hatte fuer den Rest des Laufs
  eine Seite weniger. Beides wird jetzt zurueckgegeben, und das Anlegen der Seite ist auf
  `--timeout` begrenzt: Im Nachstellen unter Last blieb `new_page` ohne Antwort stehen, was 1.6.0
  nur ueber das 480-s-Versuchsbudget abfing. (2) Die Wartezeit auf eine freie Seite war
  fest 60 s, eine Seite in Arbeit darf aber `max(timeout, 30) * 4` Sekunden brauchen (480 s bei
  `-t 120`); die wartende Seite gab auf, obwohl mit ihr nichts war, und verbrauchte dabei auch
  ihren Wiederholungsversuch. Die Wartezeit folgt jetzt `--timeout` (Anlegen der Seite plus
  Versuchsbudget plus 15 s fuer das Zuruecksetzen, 615 s bei `-t 120`), liegt ausserhalb des Versuchsbudgets der Seite
  und verbraucht keinen Versuch. Seiten, die trotzdem keine Browser-Seite bekommen, werden nach
  der parallelen Phase einzeln wiederholt, bevor der Bericht entsteht. Was dann noch scheitert,
  steht wie bisher unter `errors`; neu sagt der Bericht, wie viele URLs die Scores abdecken: JSON
  `summary.attempted_url_count` (additiv, neben `url_count`), PDF eine Zeile „Bewertungsbasis" im
  Audit-Rahmen und ein Hinweis mit den nicht auditierten URLs im Statusabschnitt (de/en),
  Terminal eine Scope-Zeile. Die Score-Berechnung ist unveraendert. Verifiziert mit Unit-Tests
  (Ableitung der Wartezeit, nur Pool-Timeouts werden zurueckgestellt, Bericht in JSON/PDF/Terminal)
  und dem Chrome-Integrationstest `batch_retries_pool_timeouts_serially` (ein Slot, zwei Worker,
  1 s Wartezeit: alle drei Seiten auditiert, jede genau einmal gemeldet). Live mit denselben 8
  Kapitelseiten und `-c 2 -t 120`: 1.6.0 verlor 3 (zwei Pool-Timeouts nach dem Slot-Verlust, ein
  Audit-Timeout nach 480 s), der Fix auditierte 8 von 8 — allerdings bei geringerer Rechnerlast,
  der Fehlerpfad selbst wurde dabei nicht beruehrt.

- **Skip-Link und `role="list"`, 2026-09-29 (#642, #644):** (1) `region` (1.3.1) meldete den
  Skip-Link `<a href="#main">Aller au contenu</a>` auf allen franzoesischen Seiten von
  barrierlab.eu, auf den englischen, deutschen und spanischen nicht. Die Ausnahme hing am Linktext:
  Die Pruefung auf `href="#…"` verglich mit `#`, Chrome liefert als `url` aber die aufgeloeste
  Adresse (`https://barrierlab.eu/fr/#main`), also griff nur die Phrasenliste, und der fehlte das
  Franzoesische. Jetzt zaehlt das Verhalten wie bei axe-cores `isSkipLink`: ein Link auf ein
  Fragment derselben Seite (Adresse ohne Fragment gleich der `url` des Wurzelknotens), der vor dem
  ersten gewoehnlichen Link steht. Die Phrasenliste ist entfallen; sie nahm auch jeden Link mit
  „springen" im Text aus. (2) `redundant-role` (4.1.2) meldete `role="list"` an `<ul>`/`<ol>` mit
  `list-style: none` (1.142 Vorkommen auf barrierlab.eu). WebKit/VoiceOver verwirft bei solchen
  Listen die Listensemantik, die Rolle stellt sie wieder her. Bei berechnetem
  `list-style-type: none` bleibt sie jetzt unerwaehnt; alle anderen redundanten Rollen werden
  weiter gemeldet. Geprueft mit Unit-Tests, zwei neuen Faellen im Detection-Korpus
  (`skip_link_language`, `redundant_role_list_style`, je mit weiter gemeldetem Gegenbeispiel) und
  Live-Laeufen gegen barrierlab.eu/fr/ und /en/.
- **Kontrast bei Verlaufstext, #640, 2026-09-29:** `a11y.contrast.weak` meldete fuer die
  Logo-Links auf geographia.eu (`background-clip: text` mit Verlauf, `color: transparent`) einen
  Verstoss mit 1,00:1. Das war keine Messung: Die transparente Textfarbe wurde auf den Hintergrund
  gerechnet und ergab den Hintergrund selbst. Die Stilerfassung setzt jetzt
  `foreground-uncertain`, wenn das Element oder ein Vorfahr `background-clip: text` (auch
  `-webkit-`) traegt oder `color`/`-webkit-text-fill-color` (nahezu) transparent ist. Solche Texte
  gehen als Pruefhinweis (Warnung) ohne `contrast_ratio`-Beleg in den Bericht und werden nicht per
  Pixelabtastung aufgeloest, weil diese die CSS-Textfarbe braucht. Zudem gilt eine gesetzte,
  deckende `-webkit-text-fill-color` jetzt als Textfarbe (Kontrastrechnung und Pixelabtastung),
  denn Browser malen die Glyphen damit, nicht mit `color` (Korpusfall `text_fill_color_contrast`:
  helle Fuellfarbe bei dunklem `color` ist Verstoss, umgekehrt bestanden). Belegt mit Unit-Tests und dem
  Korpusfall `gradient_text_contrast` (Verlaufstext Hinweis, grauer Fliesstext weiter Verstoss);
  Live-Lauf gegen geographia.eu/atmosphere/de/kapitel/hebel/: beide 1,00:1-Verstoesse sind
  Hinweise, weitere Kontrastbefunde gab es auf der Seite weder vorher noch nachher
  (Barrierefreiheit 75 → 86, gesamt 74 → 79).
- **Heuristische Formularbefunde, 2026-09-29 (#643):** Zwei Vermutungsregeln meldeten auf
  barrierlab.eu korrekt gebaute Formulare. (1) „Input may require format instructions" (3.3.2,
  `label`) liess eine Accessible Description nur gelten, wenn sie ein Formatwort wie „Format:"
  oder „z.B." enthielt; das Datumsfeld des Musterformulars mit `aria-describedby` („Day of
  travel") fiel durch. Ausserdem traf der Begriff „pass" (Reisepass) per Teilstring das Label „Was
  ist passiert?" der Kontakt-Textarea. Jetzt zaehlt jede Beschreibung als Anleitung, Begriffe unter
  fuenf Zeichen (`pass`, `tel`, `date`, `zip`, `plz`, `ssn`) zaehlen nur als ganzes Wort, und der
  Befund ist ein Pruefhinweis (`as_warning`) statt eines Verstosses, weil der Formatbedarf nur aus
  dem Labeltext geraten ist. (2) „Grouped form controls may be missing a fieldset/legend" (1.3.1,
  `form-field-group`) verlangte fuer jede Checkbox eine Gruppe, sobald die Seite irgendwo zwei
  Radio-/Checkbox-Elemente hatte — auf /tasks/bill/ und /tasks/appointment/ traf das die einzelne
  Checkbox neben einer korrekt gruppierten Radio-Umschaltung. Zusammengehoerig sind Checkboxen
  ueber ein gemeinsames `name` im selben Formular; das traegt der AX-Baum nicht. Die Pruefung fuer
  Checkboxen laeuft deshalb als DOM-Seitenregel (`check_checkbox_group_with_page`): erst ab zwei
  sichtbaren Checkboxen gleichen Namens wird ein `fieldset`/`role=group`-Vorfahr verlangt. Radios
  bleiben unveraendert im Baum (ein Radio braucht immer eine Gruppe). Neue Korpusfaelle
  `form_heuristics_clean` und `form_heuristics_flagged`.
- **`--per-page-reports`: eine Datei je Seite, 2026-09-29:** Die Dateinamen der Einzelberichte
  kamen nur aus dem Host (`casoon-de-<datum>-single-report.json`); jede Seite einer Website
  ueberschrieb die vorige, aus einer Sitemap mit drei Seiten blieb eine Datei. Der Name enthaelt
  jetzt Host und Pfad (`casoon-de-arbeitsweise-…`), bei Query-Strings einen kurzen stabilen Hash.
  Gilt fuer alle Formate; Einzelaudits ohne `--per-page-reports` behalten ihren Namen.
- **Zwei Fehlalarme auf geographia.eu (#638, #639), 2026-09-29:** Auf
  geographia.eu/atmosphere/de/kapitel/hebel/ meldete 1.6.0 zwei Verstoesse gegen 1.3.1, die keine
  sind. (1) `th-has-data-cells` (8 Vorkommen, hoch): Chrome legt `<thead>` als `rowgroup` in den
  AX-Baum, markiert das schlichte `<tbody>` aber als ignoriert; Zeilen und Zellen haengen trotzdem
  darunter. Die Regel liess ignorierte Knoten samt Teilbaum aus, fand so keine einzige Datenzelle
  und meldete jede Spaltenueberschrift. Leere Zellen und `aria-hidden`-Kinder spielten keine
  Rolle. Jetzt steigt die Suche durch ignorierte Knoten hindurch, zaehlt sie aber selbst nicht.
  (2) `landmark-unique` (2 Vorkommen, mittel): Die DOM-Ergaenzung zu den Landmark-Regeln leitete
  die Rolle aus dem Tag ab und machte jedes `<header>` zum `banner`, auch das Kapitel-`<header>`
  in `<main>`. Nach HTML-AAM (und in Chromes AX-Baum) sind `<header>`/`<footer>` in `article`,
  `aside`, `main`, `nav`, `section` oder den entsprechenden Rollen generisch; die DOM-Ergaenzung
  folgt dem jetzt. Dasselbe galt fuer `<aside>`: Innerhalb von `article`, `aside`, `nav` oder
  `section` (bzw. den Rollen `article`, `complementary`, `navigation`) ist es nur mit
  zugaenglichem Namen `complementary`, sonst generisch; `role=region` zaehlt dabei nicht, ein
  unbenanntes `<section>` schon (so rechnet Chrome). Die DOM-Ergaenzung beruecksichtigt dafuer
  auch `title` als Namen. Belegt am Live-Lauf: beide Befunde weg, die uebrigen vier unveraendert,
  Barrierefreiheitswert 75 auf 85. Neue Korpusfaelle `table_headers_tbody_ignored` und
  `landmark_header_in_main`, `landmark_aside_scoping` (echte Negative) und
  `landmark_aside_named_duplicate` (echtes Positiv); `table_headers_no_data` und
  `landmark_granular` sichern die bisherigen echten Positive.
- **Englische PDFs ohne deutsche Tabellenbeschriftungen, 2026-09-29:** Mit `--lang en` standen in
  den Tabellen zu Sicherheits-Headern („Vorhanden"/„Fehlt"), SSL („Gueltiges Zertifikat", „Laeuft
  ab in … Tage", „Chain-Laenge"), Touch-Targets („Zu klein", „Zu eng beieinander") und Schrift
  („Kleinste Schrift", „Lesbarer Text") deutsche Beschriftungen, gefunden an einem englischen
  casoon.de-Bericht. Die Beschriftungen folgen jetzt der Laufsprache; ein Guard-Test mit den
  Moduldaten dieses Laufs prueft die englische Ausgabe auf deutsche Begriffe.
- **Cache-Rundreise und deterministische Reihenfolge, 2026-09-29:** Beim Aufbau des
  Golden-Render-Harness (Plan 66) fielen zwei Fehler auf. (1) `GraphEntity.properties` wurde mit
  `skip_serializing_if` weggelassen, trug aber kein `default`: Ein gecachter `report.json` mit einer
  Entitaet ohne Eigenschaften liess sich nicht mehr lesen („missing field `properties`"), und die
  CLI auditierte stillschweigend neu. Betroffen waren 13 der 17 eingefrorenen Cache-Eintraege.
  Dieselbe Luecke hatten `BatchReport.errors`, die vier Listen in `SitemapDiagnostics` und die
  drei in `CrawlDiagnostics`; alle tragen jetzt `default`. Neue Rundreise-Tests serialisieren und
  lesen einen vollstaendigen `AuditReport` und einen `BatchReport` mit leeren Listen. (2) Zwei
  Laeufe ueber dieselbe Eingabe konnten verschiedene Berichte liefern, weil Ergebnisse aus
  `HashMap`s ohne vollstaendige Sortierung kamen: Bei Gleichstand konnten sich Reihenfolge und
  (nach dem Kuerzen auf 10) Inhalt von `top_recurring_rules` aendern; Schema-Konflikte im
  Batch, die interaktiven Kategorien samt PDF-Tabelle „Befunde nach Kategorie", die dominante
  Regel der Zusammenfassung, die Hauptrolle im Massnahmenplan, Template-Cluster, doppelte
  Inhalte, Minifizierungs-Abweichungen und die Sitemap-HTTP-Befunde (Abschlussreihenfolge der
  Anfragen) hatten dieselbe Luecke; die Strafpunkte des Accessibility-Scores wurden in
  `HashMap`-Reihenfolge summiert. Jede Sortierung hat jetzt einen letzten eindeutigen Schluessel
  (Regel-ID, Entitaets-ID, Kategorie, Selektor, URL), die Strafpunkte werden in Regel-ID-Reihenfolge
  summiert. Tests fixieren die Reihenfolge bei Gleichstand. Geprueft: drei Golden-Render-Laeufe
  ueber alle 17 Seiten, `diff -r` leer.

- **1.6.0, 2026-09-29:** Sammelversion der Stabilitaets- und Genauigkeitsarbeit seit 1.5.1
  (Plaene 58-65, Sperrseiten, Abgleich mit dem rankinglab-Korpus; Einzelheiten in den Eintraegen
  darunter). Abhaengigkeiten auf die dabei veroeffentlichten Stande gehoben: `a11y-rules`,
  `a11y-dom`, `a11y-report`, `accname` 0.12.2 (Listenregeln sehen durch `<slot>`,
  `<ul role=listbox>` ist keine Liste) und `html-conform` 0.3.1 (bringt die vnu-naeheren Pruefungen
  aus 0.3.0 mit: `img` ohne `alt`, interaktiver Inhalt in `a`, weitere Parserfehler; dazu `name`
  neben RDFa-`property` an `<meta>`). HTML-Konformitaetswerte koennen dadurch sinken.

- **Befunde der report-lint-Pruefung ueber 120 rankinglab-Berichte, 2026-09-29:** 8 Berichte fielen
  durch. (1) Das Quellenqualitaets-Signal „Bedienelemente benannt" zaehlte alle rohen 4.1.2- und
  1.1.1-Verstoesse, die Lint-Pruefung alle normalisierten; seit Plan 61 tragen
  `aria-hidden-focus` und `aria-prohibited-attr` 4.1.2 nur normalisiert, und beide Zaehlungen
  widersprachen sich (5 Berichte). Inhaltlich waren beide zu breit: Doppelte IDs oder verbotene
  ARIA-Attribute sind kein fehlender Name. `taxonomy::is_missing_name_or_role` ist jetzt die eine
  Definition fuer Blocker, Signal und Lint; das Signal bildet rohe Verstoesse wie die
  Normalisierung auf die Taxonomie ab. (2) `labels::check_link` pruefte nebenbei generische
  Linktexte (nur englisch, ohne die Kontext-Ausnahme aus #569) und meldete dieselben Links wie
  `link_purpose` als zweite Zeile derselben Taxonomie-Regel (deutschebahn.com 7 + 5 Vorkommen,
  `violated_rule_count` eins zu klein, 3 Berichte). Die Nebenpruefung ist entfernt.

- **Abbrueche im rankinglab-Lauf, 2026-09-29:** Von 122 rankinglab-Seiten scheiterten 7; vier
  davon liefen mit 1.3.1 problemlos. (1) Einzelaudits mit vollem Chrome unter macOS blieben mit
  der CDP-Fokus-Emulation aus Plan 62 zeitweise ganz stehen: www.deutschebahn.com hing in 3 von 6
  Laeufen (jeder CDP-Befehl lief 30 s in den Timeout, der Audit ueberschritt rankinglabs
  12-Minuten-Grenze), ohne Emulation liefen 6 von 6 in 103-128 s mit gleichen Ergebnissen. Die
  Emulation war fuer verborgene Tabs paralleler Batch-Seiten gedacht und gilt jetzt nur dort
  (`BrowserOptions::focus_emulation`, gesetzt bei `--concurrency` > 1). (2) Scheitert ein
  gedrosselter Performance-Aufruf, laedt die Seite unter der CPU-Drosselung weiter; die
  Aufraeumbefehle warteten dann je 30 s. Jetzt loest die Pipeline zuerst die CPU-Drosselung,
  stoppt das Laden, jeder Schritt mit 5 s, und laesst die uebrigen Profile aus, wenn der Tab nicht
  antwortet. (3) Die Vorpruefung vor dem Browserstart brach bei einem HEAD-Timeout ab:
  www.regierung-mv.de beantwortet HEAD gar nicht (GET in 0,3 s), www.ing.de braucht 9-10 s je
  Antwort bei fest 10 s Wartezeit. Jetzt entscheidet ein GET, und gewartet wird so lange wie
  `-t`. Keine Fehler bei uns: www.unito.it und www.douglas.de sperren (403), www.tu-berlin.de
  drosselte voruebergehend (429).

- **Berichtsgenauigkeit nach dem Abgleich mit dem rankinglab-Korpus, 2026-09-29:** Zwei
  Berichtskritiken (gov.uk, sachsen-anhalt.de) gegen 1.3.1 fanden Fehler, die Urteil und Zahlen
  verfaelschten; jeder ist am Code bzw. an den Daten nachgeprueft. (1) Die Zusammenfuehrung von
  Desktop und Mobile paarte jeden Mobile-Befund mit dem ersten Desktop-Befund gleichen Selektors,
  auch wenn der schon vergeben war: 18 leere Listen je Ansicht wurden 35 Vorkommen. Jetzt eins zu
  eins. (2) `role="status"`/`alert` ohne Namen galt als 4.1.2-Verstoss; ARIA verlangt fuer diese
  Rollen keinen Namen. Auf gov.uk trugen zwei leere Status-Regionen das „nicht bestanden".
  (3) Die Combobox-Regel suchte die Optionsliste nur im Teilbaum; `aria-controls` und der
  geschlossene Zustand zaehlen jetzt (APG-Autocomplete auf gov.uk). (4) „Bedienelemente ohne
  Namen" zaehlte alle 4.1.2-Befunde ab mittel, auch doppelte IDs und unbenannte Dialoge; jetzt nur
  fehlender Name/fehlende Rolle und 2.1.1. Der Satz „0 kritische und 0 hohe Befunde, davon 3 …"
  nennt die Blocker nicht mehr als Teilmenge. (5) Sicherheitskarte und Modultabelle folgen der
  korrigierten Wortstufe (Plan 33) auch in Farbe und Tabelle; die Quellenqualitaets-Karten nutzen
  eine Skala statt drei; der Aufwand einer Massnahme steigt mit der Komplexitaet des Befunds
  („Geringe Komplexitaet" neben „53 Vorkommen deuten auf ein Template-Problem"). (6) Kein
  doppelter Punkt im Anhang, deutsche Bereichsnamen bei Touch-Targets, und die Zeile unter den
  Zaehlern trennt Stellen zur Handpruefung von den nur manuell pruefbaren Kriterien. (7) JSON
  bleibt kanonisch Englisch (#406): Seit #633 standen die Texte der geteilten Regeln in der
  Laufsprache im JSON. Die Regeln laufen jetzt englisch; ein deutscher Lauf prueft zusaetzlich
  auf Deutsch und legt die Texte in `WcagResults::localized_texts` ab (nicht serialisiert), die
  PDF-Builder (Einzel und Batch) setzen sie ein. (8) Der DOM fuer die geteilten Regeln ist der
  flache Baum: Per Slot zugewiesene Light-DOM-Knoten haengen unter ihrem `<slot>`, nicht
  zugewiesene erscheinen nicht. Bisher hingen sie neben dem Shadow-Inhalt, und jede
  `<ul><slot>` einer Web-Komponente stand leer da (sachsen-anhalt.de). (9) Beim Beenden meldete
  `BrowserManager::close` „Failed to close page: Session with given id not found" fuer die schon
  geschlossene Audit-Seite (5 von 15 Einzelaudits); das ist jetzt eine Debug-Meldung.
  Nachkontrolle an neu erzeugten Berichten: (10) Blocker zaehlen Elemente, nicht Regeltreffer (drei
  Namensregeln am selben Menue-Element auf sachsen-anhalt.de waren „3 Bedienelemente"), mit
  Einzahl im Satz. (11) Die Checkliste „Alle Verstoesse" zeigt die geteilten Texte ebenfalls
  deutsch. (12) Quellenqualitaet: Kartenfarben nach denselben Schwellen wie die Wortstufe (40
  statt 60 fuer rot), und „alle Signale in Ordnung" nur, wenn jede Dimension gut ist (gov.uk: 80+
  gesamt, Substanz 70). (13) Eine Landmark-Massnahme mit `aria-label` bekommt nicht mehr den
  Titel „Interaktive Elemente (Buttons, Links) verstaendlich benennen".
- **Sperrseiten werden nicht mehr bewertet, 2026-09-29:** Beim Abgleich mit dem rankinglab-Korpus
  fiel auf, dass www.douglas.de automatisierten Browsern seit mindestens 1.3.1 eine Akamai-Seite
  „Access Denied" mit HTTP 403 und rund 100 Knoten liefert; sie wurde mit 75 bewertet und als
  vollstaendig gemeldet. Mit der headless-shell lieferte www.hornbach.de eine Fastly-„Client
  Challenge" mit CAPTCHA (355 statt 12.942 Knoten, 85 statt 56). Neues Modul
  `audit::access_block`: HTTP 401/403/407/429, eindeutige Challenge-Titel der Anbieter oder
  Challenge-Ressourcen auf einer duennen Seite (< 300 Elemente; Cloudflare laedt
  `/cdn-cgi/challenge-platform/` auch in regulaere Seiten) lassen das Audit der URL mit
  `AccessBlocked` scheitern: Einzelaudit mit Exit-Code 3 und Grund, im Batch unter `errors`, ohne
  Wiederholung. Die Pruefung laeuft direkt nach dem ersten Aufruf; Challenge-Seiten wechseln oft
  sofort ihr Dokument, daher bis zu drei Versuche und ein zweiter Blick auf den Dokumentstatus
  spaeter in der Pipeline. Geprueft: douglas.de 3 von 3 gesperrt, hornbach.de mit Chrome normal
  und mit headless-shell gesperrt, Batch ueber 15 Vergleichsseiten ohne Fehlalarm.
- **Batch unter macOS mit vollem Browser seriell gegen haengende Journeys, 2026-09-29 (Plan 65):**
  Im Referenz-Batch (gov.uk, bundesregierung.de, dm.de, berlin.de, `--full`, drei Worker) liefen
  mit dem System-Chrome 154 in 4 von 4 Laeufen Journeys ins 10-s-Budget, zweimal blieb der
  Browser ganz stehen (> 5 bzw. > 10 min). Alle offenen Seiten liefen in derselben Sekunde in den
  Timeout, unabhaengig von der Journey. Eine Stichprobe des Browser-Prozesses zeigte den
  Hauptthread in der AppKit-Tastaturverarbeitung (`performKeyEquivalent` → `SLSObscureCursor`,
  wartend auf den WindowServer): Auch `--headless=new` faehrt unter macOS die Ereignisschleife.
  Gegenproben: nur die Journeys seitenuebergreifend serialisieren half nicht (1-10 Timeouts); ohne
  Fokus-Emulation kein Totalhaenger, aber wieder Tab-Walk-Timeouts in verborgenen Tabs (Plan 62);
  ein Worker: 2 von 2 sauber (100-120 s statt 38 s); Einzelaudits 6 von 6 sauber;
  `chrome-headless-shell`: 5 von 5 sauber, aber www.hornbach.de lieferte ihr eine Bot-Sperrseite
  (355 statt 12.942 Knoten, bewertet mit 85 statt 56). Aenderung: Ohne `--concurrency` laufen
  Batches mit Journeys unter macOS mit vollem Browser seitenweise; der Laufstart und `doctor`
  sagen es. Nachher: 2 Referenz-Batches ohne `-c` in 97-111 s, einer sauber, im anderen ein
  einzelner Modal-Test auf dm.de ueber 10 s (als `partial` ausgewiesen, kein Haenger). Die
  Browserauswahl fand eine per `browser install --headless-shell` installierte Shell
  bisher gar nicht; sie ist jetzt Rueckfall nach Chrome for Testing, wenn kein System-Browser da
  ist. Messungen in `reports/p65/`.
- **JSON und PDF zeigen dieselben Befunde in derselben Reihenfolge, 2026-09-28 (Plaene 58-64):**
  Stabilitaetspruefung vor 1.6.0 an gov.uk, bundesregierung.de, dm.de und casoon.de, je zwei
  Laeufe. Scores waren stabil, die Berichte nicht deckungsgleich. (1) `top_actions` und die
  „5 wichtigsten Massnahmen" reihten nach zwei Logiken; jetzt gilt
  `audit::prioritization::action_order` fuer JSON und PDF: Schwere, darin BFSG-Pflicht (WCAG A/AA)
  zuerst, dann Hebel, Vorkommen und `rule_id`. Das PDF uebernimmt die Reihenfolge der
  Normalisierung, auch in der Befundmatrix; `top_actions` fuehrt `rule_id`. (2) Das PDF nahm den
  Titel aus der Erklaerung, die fuer Regeln ohne eigene auf den Kriteriumstext zurueckfaellt, und
  fasste gleiche Titel zusammen: vier 4.1.2-Regeln wurden „Fehlende Name/Rolle" mit summierter
  Anzahl. Titel kommen jetzt aus der Taxonomie; zusammengefasst wird nur dieselbe Stelle aus WCAG
  und SEO, ohne zu addieren. (3) Gleichstaende und die Seitenreihenfolge im Batch hingen von der
  Eingabe- bzw. Fertigstellungsreihenfolge ab. (4) `wcag_criterion` enthielt bei
  `aria-prohibited-attr`, `aria-hidden-focus` und `frame-tested` die Regel-Kennung statt 4.1.2.
  (5) Im Batch waren alle Seiten ausser einer verborgene Tabs: `requestAnimationFrame` feuerte
  nie, `document.hasFocus()` war false, der Tab-Walk lief auf 7 von 10 casoon.de-Seiten in den
  Timeout und Fokus-Pruefungen massen anders als im Einzelaudit. Jede Seite laeuft jetzt mit
  CDP-Fokus-Emulation, sichtbar und fokussiert (casoon.de-Batch 84 s -> rund 50 s, keine
  Timeouts). Ein eigenes Fenster je Seite leistete dasselbe, liess Chrome aber mit mehreren
  schweren Seiten gleichzeitig haengen (Referenz-Batch 215 s statt 57 s) und wurde verworfen.
  `run_single_audit` schliesst seine Seite.
  (6) `report-lint` pruefte das PDF-Zertifikat gegen `overall_score` statt gegen das gespeicherte
  Zertifikat. (7) Das Batch-JSON listete in `top_actions` dieselbe Regel je Seite einmal, das
  Batch-PDF fasste zusammen, sortierte aber eigen und nannte in „Wirkung" die Anzahl einer Seite;
  beide nutzen jetzt `aggregate_batch_findings` (je Regel summiert, Schwere als Maximum, abgeleitete
  Felder aus der Gesamtzahl), `top_actions` fuehrt `url_count`. Neue Tests: JSON-PDF-Paritaet fuer
  Einzel- und Batch-Bericht, Reihenfolge unabhaengig von der Eingabe, sichtbare und fokussierte
  Parallel-Seiten.
- **`a11y-*` 0.12: deutsche Befundtexte, `ids/duplicate` im Kern, 2026-09-28:** Die geteilten
  Regeln laufen jetzt in der Laufsprache des Berichts (`run_with_semantics_in`, `--lang de` ist die
  Vorgabe). Seit 0.11 kamen ihre `message`-Texte englisch und standen so in deutschen PDF-Berichten.
  Der WCAG-2.2-Filter fuer `ids/duplicate` (nur per IDREF referenzierte Duplikate, 4.1.2) liegt jetzt
  in `a11y-rules` selbst und gilt damit auch fuer astro-post-audit und LiveAudit; `IDREF_ATTRS`,
  `referenced_ids`, `duplicate_id_is_referenced` und die Nachzaehlung am Vermerk sind entfallen.
  Die Duplikat-Tests in `shared.rs` pruefen dasselbe Verhalten weiter, neu ist ein Test fuer deutsche
  Texte. Geprueft: 1749 Tests gruen, clippy und fmt sauber.
- **Strukturierte Daten aus `web-checks` 0.4, 2026-09-28:** Regeltabellen, JSON-LD-Normalisierung
  und Bewertung (`schema_rules.rs`, Normalisierungshaelfte von `schema.rs`) liegen jetzt in
  `web_checks::structured_data`, geteilt mit astro-post-audit (Plan barrierlab/02, Entscheidungen
  D1–D8). Hier bleiben CDP-Erhebung, Inhaltsabgleich (`schema_parity`), Mikrodaten/RDFa und die
  Berichtstexte (`feature_label`, `status_text`, `manual_review_text`). Aenderungen: Breadcrumb-Name
  darf aus `item.name` kommen (tagesschau.de: drei falsche Pflichtbefunde weniger); FAQPage verlangt
  `acceptedAnswer` je Frage; NewsArticle empfiehlt `publisher`, WebSite `potentialAction` (Qualitaet
  von WebSite jetzt ueber 5 statt 4 Angaben); `RULESET_VERSION` 2026-09-28. **JSON-Form:**
  `rule_assessments[].manual_review` enthaelt Kennungen (`merchant_listing_context`, …) statt
  englischer Saetze (#406-Muster; das PDF leitet die Texte ab). Strukturmeldungen im JSON nennen den
  Block, Kennungen (`jsonld_*`) unveraendert.
- **Meta-Laengen und OpenGraph aus `web-checks` 0.3, 2026-09-28:** Titel- und
  Description-Laenge kamen an vier Stellen (`meta.rs`, `serp.rs`, zweimal `profile.rs`) mit
  denselben Grenzen 30–60 / 120–160 und zaehlten **Bytes**: Jeder Umlaut zaehlte doppelt, ein
  deutscher Titel mit 60 Zeichen konnte „zu lang" sein. Jetzt `web_checks::meta` (Zeichen, Leerraum
  wie im Browser zusammengefasst), geteilt mit astro-post-audit. OpenGraph/Twitter: Tag mit leerem
  `content` gilt als fehlend (vorher vorhanden), eine Gruppe nur aus leeren Tags als nicht
  vorhanden; Vollstaendigkeit ueber `web_checks::social`. Neu in `meta_issues`: ungueltiger
  `twitter:card`-Wert und relatives `og:image` (je Medium), bisher nur in astro-post-audit geprueft.
  web-checks von 0.1 auf 0.3 (robots unveraendert).
- **accname-Differential mit berechnetem Stil, a11y-core 0.11.3, 2026-09-28:** `accname-diff`
  rechnet die eigene Seite jetzt mit `accname::name_rendered`: `display`/`visibility` kommen aus
  einem CDP-`DOMSnapshot` (`fetch_dom_document_with_layout`, `LayoutStyles`,
  `RenderedCdpDocument`), ebenso die Leerraum-Textknoten, die `DOM.getDocument` auslaesst. Die
  geteilten Regeln laufen unveraendert ueber `fetch_dom_document`. Korpus (35 Seiten): ein echtes
  Abweichungsmuster statt sechs (2 Vorkommen, per Skript eingefuegter Text auf
  bundesregierung.de) — „EU-Arktis", „Rechenpower", „abholen*", ein `display:none`- und ein
  `<br>`-Fall stimmen jetzt mit Chrome ueberein.
- **a11y-core 0.11.2, 2026-09-28 (Plan 52):** `accname`, `a11y-dom`, `a11y-report`, `a11y-rules`
  von 0.11.0 auf 0.11.2. 0.11.1 (Leerzeichen zwischen Inline-Elementen nach Tag) ist uebersprungen
  und zurueckgezogen: Im accname-Korpus fiel die Namensgleichheit von 97,7 % auf 92,9 %, weil per
  CSS zu Bloecken gemachte `<span>` zusammengeklebt wurden. 0.11.2 nimmt das zurueck und haelt
  `<dt>` (Rolle `term`) als namenlos fest. Korpus mit 0.11.2: 97,9 %, 6 echte Abweichungen wie
  zuvor. Die richtige Loesung (Trenner aus dem berechneten `display`) liegt in barrierlab.
- **Meta-Refresh-Zwischenseiten im JSON, 2026-09-28 (Plan 47):** Dass `navigate` eine
  Zwischenseite (kurzer `<meta http-equiv="refresh">`) verfolgt oder eine echte Weiterleitung per
  Refresh abgebrochen hat, stand bisher nur im Log. Jetzt fuehrt
  `accessibility.execution.navigation.meta_refresh` je Viewport-Durchgang `outcome`
  (`interstitial_followed` | `redirect_not_followed`) und `hops`; ohne Refresh fehlt das Feld.
- **Leere Ueberschrift mit eigenem Taxonomie-Eintrag, 2026-09-28 (Plan 56):** `headings/empty`
  fiel auf `a11y.headings.missing` zurueck — „Fehlende Ueberschriftenstruktur", Schwere High,
  pauschal 20 Punkte Abzug. Eine leere Ueberschrift ist ein einzelner Defekt, keine fehlende
  Gliederung. Jetzt `a11y.heading_empty.invalid` (Medium, 2,5–8 Punkte logarithmisch wie die
  anderen Ueberschriftenregeln, Bereich Ueberschriften) mit eigener Erklaerung DE/EN.
- **Fehlende Navigations- und Fussbereich-Landmark mit eigener Kennung, 2026-09-27 (Plan 56):**
  Die beiden Pruefungen in `landmarks.rs` vergaben keine `rule_id`, fielen damit auf ihr Kriterium
  zurueck und landeten im 1.3.1-Sammelbucket `a11y.structure.missing` („Missing semantic
  structure", allgemeine Erklaerung mit Tabellen-Beispiel). Jetzt `landmark-navigation-present`
  und `landmark-contentinfo-present` mit eigenen Taxonomie-Eintraegen
  (`a11y.landmark_navigation.missing`, `a11y.landmark_contentinfo.missing`, Bereich
  „Landmarks"), Erklaerung DE/EN und Korpus-Erwartungen (`missing_main_landmark`,
  `landmarks_and_lists`). Erkennung und Schwere unveraendert.
- **Eigene Erklaerungen fuer die geteilten Kennungen, 2026-09-27 (Plan 56):** Die geteilten
  Kennungen mit eigenem Taxonomie-Eintrag (Ueberschriftenebene, h1 fehlt/mehrfach, vier
  Listen-, drei Tabellenregeln, ungueltiger Sprachcode, positiver tabindex, begrenzter Zoom)
  hatten keinen eigenen Erklaerungstext. `resolve_explanation` fiel auf den Text ihres Kriteriums
  zurueck: Eine leere Liste las sich im PDF und in `fix_guidance` wie „Fehlende semantische
  Struktur" mit Tabellen-Beispiel, ein ungueltiger Sprachcode wie eine fehlende Sprachangabe. Jetzt
  hat jede einen Text (deutsch und englisch, mit Beispielcode), geschluesselt nach der
  Taxonomie-Kennung. Die beiden Texte fuer `list` und `meta-viewport-large` waren unter toten
  Kennungen abgelegt und nie erreichbar; sie sind in die Eintraege fuer
  `a11y.list_structure.missing` und `a11y.viewport_zoom.restricted` aufgegangen (der Listentext
  sprach noch von leeren Listen, die inzwischen einen eigenen Eintrag haben). Ein Test haelt fest,
  dass keine geteilte Kennung mit eigenem Eintrag mehr auf den Kriteriumstext zurueckfaellt.
  `headings/empty` hat keinen eigenen Taxonomie-Eintrag und bleibt beim Text zu 2.4.6.
- **Tote Katalog-Eintraege entfernt, 2026-09-27 (Plan 56):** `LEGACY_WCAG_MAP` fuehrte noch
  `list`, `meta-viewport-large` und `table-duplicate-name` — Kennungen, die keine Regel mehr
  vergibt (die ersten beiden sind an die geteilten Regeln `lists/*` bzw.
  `zoom/viewport-scale-limited` gegangen, die dritte hat es als Pruefung nie gegeben). Ebenso
  `a11y.table_structure.invalid`, nur ueber `table-duplicate-name` erreichbar, samt
  Bereichszuordnung und Erklaerung. Geprueft per Suche ueber Quelltext, Tests, Erklaerungen und
  PDF-Pfad. Die Eintraege `a11y.list_structure.missing` und `a11y.viewport_zoom.restricted`
  bleiben, sie tragen die geteilten Kennungen.
- **Eine Pruefung fuer die Gruppierung von Radio-Buttons und Checkboxen: `form-field-group`,
  2026-09-27 (Plan 56):** Zwei eigene Pruefungen meldeten denselben Radio-Button ausserhalb einer
  Gruppe: `form_rules::check_grouped_controls` unter `form-field-multiple-labels` (in axe-core
  heisst das „Feld mit mehreren Labels" — ein anderer Defekt) und `info_relationships` unter
  `radio-group`. Letztere pruefte nur den direkten Elternknoten und meldete damit zu Unrecht, sobald
  ein `<label>` zwischen Radio und `<fieldset>` lag. Jetzt gibt es eine Pruefung mit eigener
  Kennung `form-field-group` (`a11y.form_field_group.missing`): Ein Radio-Button braucht immer
  eine Gruppe als Vorfahren, eine Checkbox erst, wenn die Seite mehr als ein Radio-/Checkbox-Feld
  hat. `check_info_relationships` ist damit leer und geloescht; die Laufkennung
  `definition-list`/`radio-group` entfaellt, `form-field-multiple-labels` heisst als Lauf- und
  Filterkennung jetzt `form-field-group` (wer sie in `auditmysite.toml` abgeschaltet hatte, muss
  den Namen nachziehen). Taxonomie-Eintrag `a11y.radio_group.missing` entfernt, Erklaerung und
  Korpus-Erwartungen (`forms_and_misc`, `landmarks_and_lists`) umgestellt. Accessibility-Score:
  `forms_and_misc` 71 → 72 (die beiden Radio-Buttons zaehlten doppelt); ueber alle Plan-56-
  Aenderungen dieses Tages ist das die einzige bewegte Fixture-Seite, axe-Vergleich Spearman
  −0,536 unveraendert.
- **Tabellen- und Listenpruefung aus `info_relationships` geloescht, 2026-09-27 (Plan 56):** Die
  beiden Teilpruefungen meldeten unter der Laufkennung `definition-list` und doppelten die geteilten
  Regeln. Verglichen: „Datentabelle ohne Kopfzellen" (AX-Rolle `table` mit Zellen, aber ohne
  `columnheader`/`rowheader` unter Kindern und Enkeln) ist eine Teilmenge von
  `tables/header-missing` (jede `<table>` bzw. `role=table|grid` ohne `<th>`/Kopfzellenrolle im
  ganzen Teilbaum); „Liste ohne Listeneintraege" (AX-Rolle `list`, nur fremde Kinder) eine
  Teilmenge von `lists/invalid-structure` + `lists/empty`. Echte Duplikate, also nach dem
  `SHARED_RULES`-Grundsatz entfernt, samt der Zellen-Pruefung, die nur Bestanden-Zaehler hochzog.
  Einzige Luecke: `<menu>` zaehlt fuer Chrome als Liste, fuer die geteilte Regel nicht. Mit den
  Pruefungen fallen der Taxonomie-Eintrag `a11y.definition_list.invalid` und die Erklaerung, die
  von `<dl>` sprach, obwohl nie eine Beschreibungsliste gemeint war. Die Radio-Pruefung laeuft
  vorerst unter der Laufkennung `radio-group` weiter. Der Korpus-Fall `misc_content_checks`
  erwartet jetzt ausdruecklich `tables/header-missing` fuer seine Tabelle ohne Kopfzellen.
- **`valid-lang` geloescht: der ungueltige Sprachcode kam doppelt, 2026-09-27 (Plan 56):** Die
  eigene Regel `valid-lang` (`language_extended.rs`) las dasselbe `<html lang>` wie die geteilte
  `document/lang-invalid` und meldete denselben Defekt ein zweites Mal. Nach dem Grundsatz von
  `SHARED_RULES` (das eigene Gegenstueck faellt, sobald die geteilte Kennung uebernommen ist) ist
  die eigene Pruefung entfernt; die geteilte deckt sie ab (Primaerkennung 2-3 Buchstaben, dazu
  die Untertags, und auch ein leeres `lang`). `html-xml-lang-mismatch` bleibt eigen. Mit ihr
  fallen der Taxonomie-Eintrag `a11y.language_valid.invalid` und seine Zuordnungen; die
  Korpus-Erwartung `invalid_lang_code` erwartet jetzt `document/lang-invalid`.
- **accname-Differential: Namensquelle aus Chromes Detail, `image` = `img`, 2026-09-27 (Plan 52):**
  Die Quellachse war blind: Der AX-Extractor fasst `aria-label`, `alt` und `value` zu `Attribute`
  und `aria-labelledby` wie `<label>` zu `RelatedElement` zusammen, und das Differential meldete
  alle diese Namen als `unknown`. Der Extractor liest jetzt zusaetzlich die gewinnende Quelle aus
  `name.sources` (erste mit Wert; Kennung aus `attribute`/`nativeSource`), liefert sie ueber
  `extract_ax_tree_with_name_sources` neben dem Baum, und `CdpDocument::with_chrome_name_sources`
  reicht sie an `names_by_source` und `Semantics::name_source` weiter. Das grobe `NameSource` aus
  `a11y-perception`, das `text_alternatives`, `accessible_name`, `svg_rules` und `instructions`
  lesen, bleibt unveraendert (Test). Rollenvergleich: Chrome `image` gegen `accname` `img` zaehlt
  als gleich. Korpus neu gemessen (35 Seiten): `unknown` 56 → 0 (`aria-label` 50, `alt` 3,
  `label` 2, `aria-labelledby` 1), Rollenabweichungen 215 → 25. Nur Messseite, `accname` und
  `a11y-perception` unberuehrt.
- **Referenzset bewertet: sieben Live-Seiten mit Baendern, 2026-09-27 (Plan 47, Schritt 2):**
  Baender aus drei Quellen je Seite — aktueller Score, Pruefung im Browser (Skip-Link, Gliederung,
  Alternativtexte, Formularbeschriftungen, Umbruch bei 320 px) und die veroeffentlichte
  Barrierefreiheitserklaerung —, vom Nutzer bestaetigt: gov.uk 75–92, gov.ie 75–92, casoon.de
  90–100, mit.edu 70–89, bundesregierung.de 35–60, dm.de 15–40, berlin.de 15–40, basf.com 0–25.
  berlin.de schwankte zunaechst (Desktop 71 oder 27): Die 71 stammte von einer Bot-/Last-Zwischenseite,
  die das Werkzeug als Startseite bewertete — behoben durch das Folgen eines Meta-Refresh auf dieselbe
  URL; die echte Seite liegt stabil bei 27.

- **1.1.1: Icon in einem benannten Link ist kein fehlender Alternativtext, 2026-09-27 (Plan 47):**
  Beim Pruefen der Referenzseiten meldete auditmysite auf www.mit.edu 7 „Image is missing
  alternative text" (High, Level A) — alles Inline-`<svg>`-Icons ohne `role="img"` in Links mit
  eigenem Namen („MIT@twitter", „open search", das Logo in „Massachusetts Institute of Technology").
  Der Link traegt die Alternative; das Icon ist dekorativ, 1.1.1 erfuellt. axe laesst diesen Fall aus
  (`svg-img-alt` gilt nur fuer `role="img"`). Wegen der neuen Level-A-Kappung haette schon ein
  solcher Fehlalarm die Seite auf 89 begrenzt. `text_alternatives` nimmt jetzt namenlose Grafiken
  *ohne* `url`-Eigenschaft (also nicht `<img>`) in benannten Links/Buttons aus; ein `<img>` ohne
  `alt` bleibt ueberall ein Verstoss (F65). Bekannte Unschaerfe: ein explizites `role="img"`
  sieht im Baum gleich aus. mit.edu 68 → 78.

- **aria-prohibited-attr nennt die Rolle statt des Tags, 2026-09-27:** Fuer `div`/`span` ohne
  `role` meldete die Regel „prohibited on role 'span'". Die Regel erfasst in diesem Fall nur diese
  beiden Tags, deren implizite Rolle nach HTML-AAM `generic` ist — die Meldung nennt jetzt
  `generic`. Gefunden an den 30 `aria-label`-Spans auf casoon.de `/leistungskatalog/` (Plan 52).
  In `docs/accname-differential.md` ist `text_transform` jetzt als erwartete Abweichung gefuehrt
  (Entscheidung 2026-09-27: `accname` bleibt beim DOM-Text).

- **Menue-Buttons nur noch mit der Menue-Journey, 2026-09-27 (Plan 53):** Ein Button mit
  `aria-haspopup="menu"` wurde zweimal geprueft — als Menue (`MenuOpen`) und zusaetzlich vom
  Accordion-Erkenner als Aufklapp-Element (`AccordionToggle`, also mit der Disclosure-Journey).
  Nach den ARIA Authoring Practices ist das das Muster „Menu Button", nicht „Disclosure"; die
  Disclosure-Journey beurteilte ihn nach Erwartungen, die er nicht erfuellen muss. Der
  Accordion-Erkenner bietet jetzt keinen Ausloeser mehr an, den der Disclosure-/Menue-Erkenner schon
  angeboten hat; die Erkennung im Berichtstext bleibt.

- **Refresh-Zwischenseiten (Bot-/Lastpruefung) wurden als Seite auditiert, 2026-09-27 (Plan 47):**
  berlin.de liefert einem Teil der frischen Browser-Sitzungen statt der Startseite eine 453-Byte-
  Zwischenseite („Einen Augenblick bitte / Just a moment please", Varnish, `<meta http-equiv=
  "refresh" content="2; url=/~~delay/">`, `<dialog open>`), die nach 2 s auf die echte Seite
  zurueckfuehrt. HTTP 200, `readyState` `complete`, keine Mutationen — fuer `navigate` und die
  Stabilitaetspruefung fertig. Der Desktop-Pass auditierte in 8 von 12 Laeufen diese Zwischenseite
  (5 AX-Knoten, Barrierefreiheit 71 statt 27, Befunde fehlender main/banner-Landmark, bypass,
  `dialog-name`, `2.2.1/meta-refresh`); der Mobile-Pass sah danach die echte Seite. `navigate`
  folgt jetzt einem Meta-Refresh mit hoechstens 5 s Verzoegerung (max. 3 Spruenge), bis ein neues
  Dokument interaktiv ist. Endet die Kette wieder auf der angefragten URL, war es eine
  Zwischenseite; fuehrt sie woanders hin (echte zeitgesteuerte Weiterleitung), wird die angefragte
  Seite neu geladen und ihr Refresh nach dem Laden mit `window.stop()` abgebrochen — sie wird so
  auditiert wie ausgeliefert, samt `2.2.1/meta-refresh`, statt wie bisher je nach Verzoegerung mitten in der Analyse
  wegzunavigieren. Nachgemessen: berlin.de 12 Laeufe nach dem Fix, Desktop/Mobile stets 27/27
  (Zwischenseite in 7 davon aufgeloest); bundesregierung.de 4 Laeufe unveraendert 41/41; lokale
  Seite mit `1; url=b.html` bleibt auf a.html mit Meta-Refresh-Befund. Unit-Tests fuer das Parsen
  der Verzoegerung und den URL-Vergleich.
- **accname-Differential: Korpuslauf und `text_transform`-Klasse, 2026-09-27 (Plan 52):**
  `accname-diff` war auf eine URL beschraenkt; ein Korpus liess sich nur von Hand zusammenzaehlen.
  Das Kommando nimmt jetzt mehrere URLs und `--url-file` und schreibt ab zwei Seiten ein
  `AccnameCorpus`: Summen je Form und Namensquelle, Kreuztabelle beider Achsen, wiederkehrende
  Muster mit Vorkommen und Seitenzahl, die Einzelergebnisse und nicht geladene Seiten. Eine Seite,
  die nicht laedt, bricht den Lauf nicht mehr ab.

  Neue Form `text_transform`: Namen, die sich nur in der Gross-/Kleinschreibung unterscheiden,
  werden mit dem berechneten Stil des Elements und seiner Nachfahren abgeglichen (`DOM.resolveNode`
  + `getComputedStyle`, nur fuer diese Kandidaten). Hat eines davon `text-transform` ungleich
  `none`, wird der Fall getrennt gezaehlt — im ersten Befund war das die haeufigste Ursache und
  haette jeden Korpus dominiert. Messung auf der Seite dieses Werkzeugs, `accname` bleibt
  unberuehrt. Ohne Stil (`compare`) bleibt es `mismatch`. Unit-Tests fuer Klassifikation, Aggregat
  und CLI-Parsing.
- **Stabilitaetspruefung meldete immer „Budget ausgeschoepft": jeder Audit galt als partial,
  2026-09-27:** `wait_for_page_stability` wertet ein Promise aus, das zu einem Objekt `{ status,
  waited_ms, mutation_count }` aufloest — gebaut ohne `return_by_value`. CDP gab deshalb nur eine
  Objekt-Referenz zurueck, `value` blieb leer, und jeder Aufruf fiel auf die Vorgabewerte:
  `budget_exhausted`, volles Budget, 0 Mutationen, gleich was die Seite tat. Gewartet wurde richtig;
  falsch war nur, was darueber berichtet wurde. Folgen: `audit_quality` stand praktisch immer auf
  `partial` mit `page_stability_budget_exhausted:2` (so bei jedem casoon.de-Lauf), und die Journeys
  konnten nicht sehen, wie lange ein Klick tatsaechlich abgewartet wurde. Gefunden beim Umsetzen von
  Plan 53 (d). Nachgemessen: casoon.de und bundesregierung.de `complete`; Wartezeiten im Trace jetzt
  echt (z. B. „stable after 201 ms, 0 mutations").

- **Disclosure-Befunde mit Unsicherheits-Hinweis statt laengerer Wartezeit, 2026-09-27 (Plan 53):**
  Die Wartezeit nach einem Klick endet nach 200 ms ohne DOM-Aenderung; eine spaetere Reaktion wird
  nur erfasst, wenn zufaellig eine andere Aenderung das Fenster verlaengert — gemessen die Ursache der
  rund 8 % Lauf-zu-Lauf-Schwankung. Laenger zu warten kostete +57 % Disclosure-Zeit fuer ein
  geaendertes Urteil in rund 66. Stattdessen traegt der Trace jetzt je Klick Status, Wartezeit und
  Mutationen, und ein Befund aus einem Klick, dessen Warten vor dem Budget endete, bekommt
  `uncertainty: {kind: "late_reaction_possible", waited_ms}`. Nach dem #406-Muster: kanonisches
  `kind` + Rohwert, `finding_uncertainty_text` als einzige Textquelle, im PDF lokalisiert an den
  Befund gehaengt; unsichere und sichere Befunde werden dort nicht zusammengefasst.

- **Natives `<details>`: ein Journey-Lauf je Ausloeser, keine Fehlalarme mehr, 2026-09-27 (Plan 53):**
  Ein neuer Chrome-Fixture-Test (`natives_details_erreicht_die_disclosure_journey`) fand drei
  Luecken. Erstens bot `patterns::accordion` jeden `<summary>` und jeden ARIA-Button mit
  `aria-expanded` ausserhalb von Navigation/Banner zusaetzlich als `AccordionToggle` an — der
  dieselbe `disclosure_journey` startet wie der schon vorhandene `DisclosureToggle`-Kandidat. Jeder
  solche Ausloeser wurde zweimal durchgeklickt, seine Befunde doppelt gemeldet. Das Akkordeon
  ueberspringt jetzt Ausloeser, die bereits einen `DisclosureToggle`-Kandidaten haben; die Zahl der
  Disclosure-Journeys je Seite sinkt entsprechend. Zweitens traegt `<summary>` in `<details name>`
  die Rolle `DisclosureTriangleGrouped`: das Akkordeon meldete dafuer auf jedem exklusiven Akkordeon
  `accordion-trigger-not-button` (Medium, 4.1.2), und die Journey bestimmte keinen gesteuerten
  Bereich. Drittens bekam ein offenes natives `<summary>` `accordion-no-controls` (Low) — ein
  natives Element braucht kein `aria-controls`; die Pruefung gilt jetzt nur fuer die Rolle `button`.
  Die Laufzeitschwankung der Disclosure-Urteile ist an Fixtures vermessen und in Plan 53
  dokumentiert (Ursache: das Beruhigungsfenster endet 200 ms nach dem Klick, spaetere Reaktionen
  werden nur zufaellig erfasst); das Zeitverhalten bleibt unveraendert, weil die Abhilfe auf acht
  Live-Seiten +57 % Journey-Zeit kostet.
- **Geteilte Regeln bekommen eigene Taxonomie-Eintraege, 2026-09-27 (Plan 56):** Auf
  `landmarks_and_lists` stand ein Befund „Missing semantic structure" mit drei unverwandten
  Fundstellen — fehlende Navigations-Landmark, leere Liste, Listeneintrag ausserhalb einer Liste.
  Ursache: `wcag_group_key` gruppiert nach der Regelkennung nur, wenn `LEGACY_WCAG_MAP` sie kennt,
  sonst nach dem WCAG-Kriterium. Keine der 1.3.1-Kennungen aus `a11y-rules` (vier Listen-, drei
  Tabellen-, drei Ueberschriftenregeln) stand dort, alle fielen in `a11y.structure.missing`. Ebenso
  lief `document/lang-invalid` unter „Missing language declaration", und
  `keyboard/positive-tabindex` teilte sich `a11y.focus_order.weak` mit der 2.4.3-Gruppe, womit zwei
  Befunde dieselbe Kennung tragen konnten. Jetzt hat jede dieser Kennungen einen eigenen Eintrag
  (Titel deutsch und englisch, dasselbe Kriterium, Score-Bereich); `lists/item-outside-list` und
  `zoom/viewport-scale-limited` nutzen die Eintraege der abgeloesten eigenen Regeln
  (`a11y.list_structure.missing`, `a11y.viewport_zoom.restricted`). Ein Test haelt fest, dass keine
  geteilte Kennung mehr im 1.3.1-Sammelbucket landet.

  Der Radio-Button ausserhalb einer Gruppe trug die axe-Kennung `definition-list`: Die eigene Regel
  `info_relationships` stempelte alle drei Teilpruefungen (Tabelle, Liste, Radio-Gruppe) mit der
  Kennung ihres Laufs. Die Radio-Pruefung meldet jetzt als `radio-group`
  (`a11y.radio_group.missing`); die Korpus-Erwartung ist nachgezogen.

  Der Accessibility-Score bleibt gleich: `AccessibilityScorer` gruppiert nach `Violation.rule`, also
  dem Kriterium, nicht nach der Taxonomie. Der Abzug der neuen Eintraege ist der des Sammeleintrags,
  unter dem die Kennung vorher lief; nur die beiden wiederverwendeten Eintraege bringen ihren
  frueheren eigenen Abzug mit. Bewegen kann sich die Bereichsaufschluesselung
  (`accessibility_score_breakdown`), um ein bis zwei Punkte — Ueberschriftenbefunde zaehlen jetzt
  unter „Heading structure" statt „Semantics". axe-Vergleich: Spearman −0,536 unveraendert, keine Seite bewegt.
- **Eindeutige Selektoren in allen eigenen Seitenregeln, 2026-09-27 (Plan 57):** Etliche
  JS-Regeln bauten ihren Selektor als `tag` bzw. `tag#id` (teils mit Klasse). Fundstellen werden
  nach (Regel, Selektor) zusammengelegt — also fielen alle id-losen Elemente desselben Tags zu
  *einer* Fundstelle („a", „button", „div") zusammen: zu niedrige Zaehlung, nicht auffindbar.
  Betroffen waren 2.4.11/2.4.12 Focus Not Obscured (auch die Fokus-Walk-Pruefung), 1.3.2
  Meaningful Sequence, 3.3.8 Accessible Authentication (Feld, Formular, CAPTCHA-Widget), 2.2.2
  Pause/Stop/Hide, 2.5.3 Label in Name, 1.4.11 Non-text Contrast (CSS), 1.4.13 Content on Hover,
  2.1.1 Click-Handler, 1.4.1 Use of Color und 3.1.2 Language of Parts (einstufiges
  `nth-of-type`). Alle nutzen jetzt den vorhandenen `__amsCssSelector`
  (`js_helpers::CSS_SELECTOR_JS`); auch 2.5.5/2.5.8 geben ihre eigene Kopie (`targetSelector`)
  dafuer auf. Elemente mit id behalten die Form `tag#id`, die Korpus-Erwartungen bleiben gueltig.
  Die Kind-Bezeichner in der Meaningful-Sequence-Meldung bleiben kurz — sie sind Text, keine
  Fundstelle. Korpus, Score-Baender und Integrationstests gruen; axe-Vergleich unveraendert
  (Spearman −0,536), keine Seite bewegt.

- **2.5.8 Target Size: Abstands- und Inline-Ausnahme, 2026-09-26 (Plan 47, axe-Vergleich):**
  `target_size_minimum` mass nur das Rechteck eines Ziels. WCAG 2.5.8 nimmt aber zwei Faelle aus:
  ein zu kleines Ziel, um dessen Mitte ein 24-px-Kreis kein anderes Ziel (und keinen Kreis eines
  anderen zu kleinen Ziels) schneidet, und ein Link im Fliesstext. Beides fehlte — obwohl der
  Doc-Kommentar der Regel die Inline-Ausnahme nannte. Jeder einzeln stehende flache Button und jeder
  Link in einem Satz galt als Verstoss; so auch „Send Message" und der Skip-Link auf `perfect.html`.
  Jetzt umgesetzt; 2.5.5 (AAA) bekommt nur die Inline-Ausnahme, weil es keine Abstands-Ausnahme
  kennt. Der Inline-Baustein ist gemeinsam (`INLINE_TARGET_JS`).

  Die Korpus-Fixture `keyboard_and_targets` erwartete einen einzelnen 16-px-Button als Verstoss —
  die Grundwahrheit bildete den Fehler ab. Sie traegt jetzt zwei benachbarte 16-px-Buttons
  (Verstoss), einen einzelnen mit Abstand und einen Link im Fliesstext (beide bestanden, der Link
  auch fuer 2.5.5). axe-Vergleich: Spearman −0,492 → −0,536.

  Zwei Nachbesserungen, gefunden am Abgleich mit axe auf `perfect.html`: Die Inline-Pruefung zaehlt
  nur den Inline-Inhalt der eigenen Zeile ohne andere Ziele — mit dem ganzen Blocktext galt ein
  Skip-Link direkt unter `<body>` als „im Fliesstext". Und der Kreis darf das *Rechteck* jedes
  anderen Ziels nicht schneiden, auch wenn das selbst zu klein ist. Dazu eindeutige Selektoren
  (`tag#id` oder `nth-of-type`-Pfad): mit dem blossen Tag-Namen fielen alle Links einer Seite zu
  *einer* Fundstelle „a" zusammen. `perfect.html` meldet jetzt dieselben vier dicht stehenden Links
  wie axe (Skip-Link, Home, About Us, Contact) und laesst den frei stehenden Button aus.

- **Geteilte Regeln melden im JSON Englisch: a11y-Familie auf 0.11.0, 2026-09-26:** Beim Pruefen der
  axe-Abweichung von `landmarks_and_lists` fielen deutsche Saetze im JSON auf („Der Listeneintrag
  steht außerhalb einer Liste."). `run_shared_rules` uebernimmt die Meldung aus `a11y-rules`
  unveraendert, und bis 0.10 formulierte das Crate deutsch — betroffen waren alle 16 eingebundenen
  geteilten Regeln, ein Verstoss gegen die Regel „JSON ist kanonisch Englisch" (#406). `a11y-rules`
  0.11.0 formuliert Befundtexte und Hinweise englisch und stand bereits auf crates.io;
  `a11y-rules`, `a11y-report`, `a11y-dom` und `accname` sind von 0.10.0 auf 0.11.0 gehoben, ohne
  API-Anpassung. Das deutsche PDF zeigt die Fundstellen-Meldungen damit englisch — wie die der
  eigenen Regeln schon immer; Titel und Erklaerungen bleiben lokalisiert. Die zu grobe Einordnung
  der geteilten Listen-Regeln in die Taxonomie steht als Plan 56.

- **Player-Bedienelemente von `<video controls>` sind kein Autoreninhalt, 2026-09-26 (Plan 47,
  axe-Vergleich):** Die groesste Abweichung im neuen axe-Vergleich war `media_and_motion`: auditmysite
  49 (Critical), axe nichts. Der Critical kam aus Chromes eigener Player-Oberflaeche: Mit `controls`
  legt der Browser Wiedergabe-, Stumm- und Vollbild-Button und eine Zeitleiste als `slider` ohne
  `valuenow` aus seinem Shadow DOM in den Accessibility-Tree. Die ARIA-Regeln meldeten die
  Zeitleiste als „Required ARIA attribute missing" (Critical) und „Slider is missing accessible
  value" (High) — auf jeder Seite mit einem Video, gekappt auf 49, und von keinem Autor behebbar.
  Behoben in `a11y-perception` 0.2.1: `AXTree::iter()` laesst alles unterhalb von `Video`/`Audio`
  aus, fuer alle WCAG-Regeln. `media_and_motion` 49 → 85, die drei eingebauten Maengel bleiben
  gefunden. Die Fixture stand in der Critical-Klasse der Score-Baender — nur wegen des
  Fehlalarms; sie ist dort entfernt. axe-Basislinie neu: Spearman −0,462 → −0,492, einzige
  bewegte Seite ist diese.

- **Live-Referenzset und axe-Vergleich fuer die Score-Kalibrierung, 2026-09-26 (Plan 47, Schritte
  2 und 3):** *Referenzset:* `tests/fixtures/reference_sites.json` fuehrt acht oeffentliche Seiten;
  jede bekommt ein Band aus einer manuellen Pruefung (nicht aus dem Werkzeug), mit Datum.
  `tests/reference_sites_test.rs` (Chrome + Netz) meldet jede Seite ausserhalb ihres Bands und laeuft
  als neuer Release-Schritt 0 vor dem Tag (`CLAUDE.md`). Seiten ohne Pruefung werden uebersprungen.

  *axe-Vergleich:* `scripts/axe-divergence.py` laesst auditmysite und axe-core (WCAG-2.2-A/AA-Tags)
  ueber alle 45 Fixture-Seiten laufen und haelt je Seite Score und axe-Verstoesse nach Schwere fest.
  Kennzahl ist die Rangkorrelation (Spearman) zwischen Score und gewichteter axe-Last; die
  Basislinie liegt in `tests/fixtures/axe_divergence_baseline.json`: **−0,46** — gleiche Richtung,
  maessig stark. Ein spaeterer Lauf nennt die Veraenderung der Korrelation und jede Seite, deren
  Score sich um ≥ 5 bewegt, obwohl axe dasselbe sieht. Uebereinstimmung ist nicht das Ziel; ein
  Scoring-Umbau, der die Korrelation deutlich verschlechtert, soll auffallen.

- **Score-Baender und eine neue Kappung: Level-A-Verstoss ist nicht SEHR GUT, 2026-09-26 (Plan 47,
  Schritt 1b):** Zum ersten Mal steht neben dem Scorer eine Erwartung, die nicht aus ihm selbst
  kommt. Entschieden in Zertifikatsbegriffen: eine praktisch saubere Seite ≥ 95; genau ein
  High-Verstoss auf Level A, sonst (fast) sauber, 75–89; mindestens ein Critical ≤ 49.
  `tests/score_calibration_test.rs` prueft das an 21 Fixtures gegen echtes Chrome, auf Standardniveau
  AA wie ein Nutzer-Audit.

  Das mittlere Band scheiterte am bisherigen Scorer: Ein fehlender Alt-Text auf sonst sauberer Seite
  ergab 92, also SEHR GUT — fuer eine Seite, die Level A nicht erfuellt und damit nicht konform ist.
  Neue Kappung neben den bestehenden (Critical → 49, ≥ 5 Critical/High → 92): **ein High- oder
  Critical-Verstoss gegen ein Level-A-Kriterium begrenzt auf 89**. Dieselbe Kappung wirkt in der
  JSON-Aufschluesselung nach Bereichen, damit die nicht wieder vom Score abweicht (Plan 37).
  Betroffen: jede Seite mit einem Level-A-High-Befund und sonst hohem Score — sie faellt von
  SEHR GUT auf GUT. `test_score_with_errors` pinnte genau das alte Verhalten (95, Note A) und ist
  angepasst; der Invarianten-Generator mischt jetzt Level A und AA, damit auch die 92er-Kappung
  geprueft wird.

- **Invarianten fuer den Accessibility-Score, 2026-09-26 (Plan 47, Schritt 1a):** Der Score hatte
  keinerlei Pruefung, die nicht von seinen eigenen Konstanten abhing. Fuenf Eigenschaften, die keine
  Abstimmung brechen darf, laufen jetzt als Tests ueber je 2000 deterministisch erzeugte
  Befundmengen (katalogisierte und unbekannte Regeln gemischt): Score in 0–100; ein weiterer Befund
  hebt ihn nie; eine hoehere Severity hebt ihn nie; die Kappungen (Critical → ≤ 49, ≥ 5 Critical/High
  → ≤ 92) gelten fuer jede Menge; nur die leere Menge ergibt 100. Alle halten. Einzige Beobachtung:
  Die Strafen werden in `HashMap`-Reihenfolge summiert, dieselbe Menge kann in der sechsten
  Nachkommastelle abweichen — irrelevant fuer den ganzzahlig angezeigten Score, die Tests tolerieren
  1e-3. Was eine Seite *erreichen soll*, sagen die Invarianten nicht; dafuer folgen die Score-Baender.
- **Zwei Detection-Fehler, gefunden beim Kalibrieren der Scores, 2026-09-26 (Plan 47):** Bevor
  Score-Baender festgelegt werden, lief das Tool ueber alle 43 Fixtures. Zwei Befunde waren falsch.

  *1.3.5 Identify Input Purpose las die falsche Quelle.* Die Regel pruefte die AX-Eigenschaft
  `autocomplete` — das ist `aria-autocomplete` (`list`/`inline`/`both`); das HTML-Attribut
  `autocomplete` legt Chrome im Baum gar nicht ab. Folge: **jedes** korrekt ausgezeichnete
  Personendaten-Feld galt als „lacks autocomplete attribute" (Medium), sichtbar schon an
  `perfect.html` mit `autocomplete="name"`/`"email"`; eine Combobox mit `aria-autocomplete="list"`
  haette „Invalid autocomplete value" bekommen. Die Regel laeuft jetzt als DOM-Seitenregel
  (`check_input_purpose_with_page`, wie 1.3.6 und 3.3.7) und liest das Attribut; die Beschriftung
  naehert den Accessible Name an (aria-label → aria-labelledby → label → title → placeholder). Die
  Einordnung ist eine reine Funktion mit Unit-Tests.

  *2.4.7 Focus Visible meldete eine Seite ohne fokussierbare Elemente als Verstoss (High).* 2.4.7
  gilt fuer „any keyboard operable user interface" — gibt es keine, ist das Kriterium nicht
  anwendbar; axe-core meldet dort nichts. #568 hatte den Befund bewusst wieder erreichbar gemacht,
  der Korpus erwartete ihn. Er bleibt, aber als Review-Hinweis (Low) ohne Wirkung auf den Score;
  `no_focus_targets` erwartet jetzt `needs_review`. Betroffen waren 21 von 43 Fixtures.

  Detection-Korpus gegen echtes Chrome gruen. `perfect.html`: 97 → 99 (verbleibend 2.5.8, ein 21 px
  hoher Button — nach WCAG 2.2 zutreffend).

- **AAA-Liste im PDF ehrlich gezaehlt: 11 statt 17, 2026-09-26:** Dieselbe Ueberzeichnung wie
  bei der A/AA-Quote (30/55): Das PDF fuehrte 17 AAA-Kriterien als „automatisch geprueft", aber
  sechs davon koennen keinen Verstoss melden — 1.2.8, 2.2.3, 2.2.4, 2.2.5 und 3.1.3 liefern nur
  einen `untested`-Eintrag, 2.4.12 nur einen Review-Hinweis. Sie stehen jetzt in
  `HINT_ONLY_CRITERIA`; die Liste zeigt 41 Kriterien (30 A/AA + 11 AAA). Anders als die A/AA-Faelle
  kommen sie nicht auf die Liste der manuell zu pruefenden Kriterien, weil die wie die Quote auf
  A/AA bezogen ist. Der Guard-Test unterscheidet das jetzt. Geprueft ueber `--debug-typ` (DE/EN).
- **Journey-Reste aus Plan 53: Skip-Link-Ziel, eingegrenzter Inhalt, Lesefehler, 2026-09-26:**
  Drei Maengel aus der Offen-Liste von Plan 53.

  *Skip-Link.* Die Journey pruefte nur, ob der Fokus `body` verlassen hat. Das meldete ein nicht
  fokussierbares `<main>` als High-Befund, obwohl der Browser den Startpunkt der Tab-Navigation
  versetzt und der naechste Tab im Inhalt landet — und liess einen Fokus durchgehen, der
  irgendwohin sprang, nur nicht zum Ziel. Jetzt wird der Link zuerst fokussiert (wie bei
  Tastaturbedienung), das Ziel aus dem `href`-Fragment aufgeloest und geprueft, ob der Fokus darin
  oder dahinter steht; bleibt er auf Link oder `body`, entscheidet ein Tab. Ein Fragment ohne
  Element ist ein Befund. Fuenf Fixtures, gegen echtes Chrome getestet.

  *Disclosure.* „Inhalt erschienen" war seitenweit — ein nachgeladenes Bild am Seitenende machte
  aus „ausgeklappt gehoert, nichts zu lesen" ein Bestehen. Ist der gesteuerte Bereich bekannt
  (`aria-controls`, oder das `<details>` um ein `<summary>`), zaehlt nur noch, was darin erscheint
  oder verschwindet. Im Feld reicht das weniger weit als erhofft: auf 12 Seiten mit 138
  Disclosure-Journeys liess sich der Bereich nur 8-mal bestimmen. Der Trace vermerkt je Journey
  `scoped` oder `page_wide`.

  *quantity_stepper.* Scheiterte ein CDP-Aufruf, wurde aus `focus_and_confirm`/`is_native_input`
  `false` — und daraus ein High-Befund „nicht per Tastatur bedienbar". Nicht lesbare Werte stehen
  jetzt als `not_observable` im Trace, ohne Befund.

- **Journey-Budget 15 s, harte Grenze je Journey, 2026-09-25 (Plan 53):** Die Hoehe des Budgets
  war offen. Gemessen an 48 oeffentlichen Startseiten ohne Grenze und dann je Budgethoehe
  nachgerechnet (Deadline vor jedem Start geprueft, wie im Code): mit **5 s** liefen 22 % der Seiten
  ins Limit und nur **55 %** der Befunde wurden erfasst — allein der Tab-Walk braucht konstant rund
  2 s. **15 s** erfassen 92 % (10 s: 83 %, 20 s: 98 %) und kosten im Mittel +1 s je Seite, weil das
  Budget nur auf schweren Seiten greift; der Median der interaktiven Phase liegt bei 2,8 s.

  Derselbe Lauf zeigte, dass eine einzelne Journey unbegrenzt laufen kann: Die Deadline wird nur vor
  dem Start geprueft, und haengt die Seite, wartet jeder CDP-Aufruf seine 30 s ab — auf www.dm.de
  brauchte eine Accordion-Journey 180 s. Jede Journey (Tab-Walk, Muster-Journeys,
  SPA-Navigation) laeuft jetzt unter einer Grenze von 10 s und wird sonst mit `journey_timeout`
  als fehlgeschlagen gefuehrt; die langsamste Journey ohne Haenger lag bei 5,6 s (Modal).

  Nebenbei behoben: Lief das Budget ab, wurden die uebrigen Kandidaten aus der *unsortierten*
  Liste als `budget_exhausted` eingetragen, obwohl seit der Sortierung nach Konfidenz die
  sortierte abgearbeitet wird — der Report nannte die falschen Journeys als uebersprungen.

  Mit dem hoeheren Budget kamen auf www.dm.de erstmals die Modal-Journeys an die Reihe — und der
  Audit dauerte danach 243 s statt 73 s, alle Throttling-Profile der Performance-Messung liefen in
  Timeouts. Ursache (Plan 55): Escape schliesst keinen der drei Info-Dialoge, die Journeys lassen
  sie offen, und die Folgephasen auf demselben Tab konfigurieren Drosselung und Cache *vor* ihrem
  eigenen Neuladen — unter CPU-Drosselung antwortete der Renderer mit drei offenen Dialogen nicht
  mehr. Ein JavaScript-Dialog war es nicht (geprueft). Nach der interaktiven Phase wird der Tab
  jetzt auf `about:blank` gesetzt; alle spaeteren Phasen navigieren ohnehin selbst zur URL.
  dm.de: 87 s, Performance `completed`, alle drei Profile gemessen.
- **Ansage-Struktur nach a11y-perception, Woerter bleiben hier, 2026-09-25:** `announcer.rs` hat
  zwei Dinge in einem Zug getan: entschieden, *woraus* eine Ansage besteht, und gleich gesagt, *wie
  das auf Deutsch heisst*. Die erste Haelfte ist keine Eigenschaft von auditmysite -- dass die Ebene
  einer Ueberschrift in die Rolle gehoert, dass `expanded=false` "eingeklappt" zu sagen hat und
  `required=false` nichts, dass "fokussierbar" nur dort etwas hinzufuegt, wo die Rolle es nicht schon
  verraet: das gilt in jeder Sprache und fuer jede Oberflaeche. Sie liegt jetzt in
  `a11y-perception 0.2.0` als `announce() -> Announcement` (Name, Rolle, Zustaende als benannte
  Teile, `AnnouncedRole`/`AnnouncedState`). Hier bleibt die Zuordnung auf die Locale-Schluessel und
  das Trennzeichen -- 16 Rollen, 13 Zustaende, dieselben Schluessel wie vorher, keine Aenderung an
  `locales/`. Der Renderer ist von 118 auf 89 Zeilen Code geschrumpft; der Gewinn ist weniger die
  Zahl als das, was hier nicht mehr entschieden wird.

  Ein Verhaltensunterschied, klein und bewusst: `level` wird jetzt als `u8` gefuehrt statt als roher
  Zeichenkettenwert. Eine Ueberschrift mit einer nicht-numerischen Ebene wird damit als
  "Ueberschrift" angesagt statt als "Ueberschrift Ebene <muell>". Chrome liefert `level` als Zahl,
  der Fall ist also theoretisch.

  Neuer Waechter `every_announced_role_and_state_has_a_german_word`: Der Compiler erzwingt, dass
  jede Variante im Match steht, aber nicht, dass der i18n-Schluessel existiert -- `I18n::t` gibt
  einen fehlenden Schluessel unveraendert zurueck, und "Suche, sr-role-textbox" waere sonst
  durchgegangen. Geprueft: `cargo test --lib screen_reader` (44 ok), `cargo clippy --all-targets -D
  warnings`, `cargo fmt`. Die vier bestehenden Snapshot-Tests sind **unveraendert** gruen -- das ist
  der Beleg, dass die Ansagen Zeichen fuer Zeichen dieselben sind.

- **Gepinnte Toolchain und ein schlankeres Crate-Paket, 2026-09-25 (Plan 28):** Lokal, CI und
  Release bauten mit dem jeweils aktuellen `stable`. Jetzt pinnt `rust-toolchain.toml` Rust
  **1.98.1** (die aktuelle Stable-Version, eine Punktversion ueber dem im Plan genannten 1.98.0)
  mit `clippy` und `rustfmt`. Ein MSRV-Versprechen gibt es bewusst nicht, `Cargo.toml` bekommt
  kein `rust-version`. Das README-Badge `rust-1.75+` behauptete eine nie gepruefte Untergrenze und
  nennt jetzt die gepinnte Version.

  `dtolnay/rust-toolchain` liest die Datei nicht. Es installiert, was im Ref oder im Pflicht-Input
  `toolchain` steht, und setzt es per `rustup default`. Die Datei haette das zwar ueberstimmt, aber
  `components`/`targets` waeren auf der falschen Toolchain gelandet, und die Version haette doppelt
  gepflegt werden muessen. Die Workflows rufen deshalb `rustup toolchain install` ohne Argument
  auf; das installiert Version, Profil und Komponenten aus der Datei. Der Release-Build ergaenzt
  sein Matrix-Target per `rustup target add`. `CARGO_INCREMENTAL=0`, das die Action bisher
  setzte, steht jetzt im `env` beider Workflows.

  `Cargo.toml` hat eine `exclude`-Liste: `.claude/`, `.github/`, `.githooks/`, `docs/`,
  `scripts/`, `tests/`, die beiden Zusatzdokumente und `rust-toolchain.toml`. Das Paket schrumpft
  von 623 auf 386 Dateien. Die Muster sind mit `/` verankert, denn die Liste aus dem Plan haette
  mit einem nackten `tests/` still auch `src/output/tests/` aus dem Paket entfernt. Jeder Pfad,
  den `include_str!`/`include_bytes!` referenziert, ist im Paket; die einzige Ausnahme ist eine
  Test-Fixture unter `#[cfg(test)]`. `cargo package --locked` laeuft ohne `--allow-dirty` durch,
  und das Binary aus dem entpackten Paket installiert sich und antwortet auf `--help`. Den ersten
  CI-Lauf mit der gepinnten Toolchain gab es noch nicht.

- **Browser-Tests laufen im Release-Tor fuer den getaggten Commit, 2026-09-25 (Plan 30):**
  `release.yml` ruft `ci.yml` als wiederverwendbaren Workflow auf und baut erst, wenn dieser
  gruen ist. Der Kommentar in `ci.yml` behauptete, `browser-smoke` und `coverage` liefen dabei
  nicht mit, weil ihr `if:` nur `push`/`pull_request` zulaesst. Das stimmte nicht: In einem
  aufgerufenen Workflow gehoert der `github`-Kontext dem Aufrufer, `event_name` ist also das
  `push` des Tags, nie `workflow_call`. Beide Jobs liefen im Release-Aufruf mit, nur aus Zufall,
  und eine auf `workflow_call` gezielte Bedingung haette nichts unterschieden.

  Jetzt steht es ausdruecklich da: `browser-smoke` hat keine Job-Bedingung mehr und laeuft bei
  jedem Aufruf, also auch fuer den getaggten SHA. `coverage` ist eine Messung und kein Tor und
  wird ueber `!startsWith(github.ref, 'refs/tags/')` fuer Tags uebersprungen. Die Kommentare in
  `ci.yml` und `release.yml` beschreiben das tatsaechliche Verhalten. Ein Contract-Test in
  `tests/release_contract_tests.rs` prueft beides. Gegenprobe: Mit der alten Bedingung an
  `browser-smoke` oder ohne die Tag-Bedingung an `coverage` schlaegt er fehl. Einen echten
  Tag-Lauf gab es dafuer noch nicht, er steht beim naechsten Release aus.
- **Best-Effort-Aufraeumfehler nicht mehr unsichtbar, 2026-09-25 (Plan 31):** `cargo judge errors`
  meldete sechs Produktionsstellen in vier Dateien, die ein I/O-Ergebnis kommentarlos verwarfen.
  Keine davon darf den Audit abbrechen, aber zwei koennen unbemerkt Dateien liegen lassen. Das
  Loeschen des heruntergeladenen Browser-Archivs nach dem Entpacken und das Loeschen der
  `.partial`-Datei nach einem fehlgeschlagenen atomaren Schreiben protokollieren jetzt ein `warn!`
  mit Pfad; Installationsergebnis und urspruenglicher Schreibfehler bleiben unveraendert. Der Flush
  der Statuszeile „Checking for sitemap..." protokolliert auf Debug-Level, und nur wenn stdout ein
  Terminal ist — eine Pipe oder geschlossene Ausgabe ist eine gueltige Umgebung, kein Fehler. Das
  Aufraeumen von PDF-Screenshots und Evidence-Ausschnitten laeuft ueber einen Helper,
  `remove_temp_file`, der `NotFound` als Erfolg wertet (die meisten Evidence-Plaetze werden nie
  geschrieben) und andere Fehler auf Debug-Level meldet, damit ein normaler Lauf still bleibt.

  Geprueft mit einem Unit-Test fuer den Helper (fehlende Datei bleibt still; `remove_file` auf ein
  Verzeichnis liefert einen Fehler, der nicht `NotFound` ist), dem bestehenden Atomic-Write-Test,
  clippy fuer beide Feature-Sets, `cargo fmt --check` und `cargo test --no-default-features`.
  `cargo judge errors` sinkt von 17 auf 11 `swallowed-result`-Funde; der Rest liegt in Tests und
  bleibt bewusst stehen, ebenso absichtlich verlustbehaftete Parse-Konvertierungen.
- **Desktop- und Mobile-Score ueber dieselben Regeln, 2026-09-25 (Plan 46):** Plan 46 vermutete,
  der zweite Viewport-Durchlauf bewege den Barrierefreiheits-Score nie. Gemessen an 35 oeffentlichen
  Startseiten stimmt das nicht: bei **24 von 35** weichen Desktop und Mobile ab, teils stark
  (berlin.de 74/26, kit.edu 46/26). Ein Wiederholungslauf ueber acht davon war auf sechs exakt
  gleich — das Signal ist ueberwiegend echt: Seiten liefern je Breakpoint anderes Markup, und die
  abweichenden Befunde kommen aus gewoehnlichen Strukturregeln (`interactive_name`, `alt_text`,
  Landmarks, ARIA).

  Ein Teil des Abstands war aber ein Artefakt: `html_content_model` und `reflow` laufen nur im
  Mobile-Durchlauf und flossen nur in den Mobile-Score. Beide urteilen ueber eine
  viewport-unabhaengige Eigenschaft der Seite (Roh-Markup; Umbruch bei 320 CSS-Pixeln). Der
  Desktop-Score wird jetzt ueber die Desktop-Befunde plus die Befunde dieser beiden Regeln
  berechnet; Ausfuehrungsvermerke und die Viewport-Kennzeichnung der Befunde bleiben unveraendert,
  denn gelaufen sind die Regeln weiterhin nur mobil. Nachgemessen: wo `html_content_model` der
  einzige Unterschied war, sind die Scores jetzt gleich (commerzbank.de 36/36, gov.ie 73/73,
  hamburg.de 20/20); berlin.de behaelt seinen echten Abstand.
- **Automatisierungsquote ehrlich gezaehlt: 30/55 statt 39/55, 2026-09-25:** Als automatisiert galt
  jedes Kriterium, fuer das der Regelkatalog einen Eintrag hat — auch wenn die Regel nur
  Pruefhinweise oder einen `untested`-Eintrag erzeugen kann. Ein Kriterium, das das Werkzeug nicht
  als verletzt melden kann, prueft es nicht automatisch. Neun Kriterien betrifft das: 1.2.2, 1.3.2,
  2.1.2, 2.2.2, 2.4.11, 2.5.1, 2.5.2, 2.5.4 und 3.3.7. Sie stehen in `HINT_ONLY_CRITERIA`
  (`src/wcag/coverage.rs`), fallen aus `automated_criteria()` heraus und zaehlen jetzt als
  manuelle Pruefung; 1.3.2, 2.1.2, 2.4.11 und 3.3.7 sind dafuer neu in der Manuell-Liste. Die Regeln
  laufen unveraendert weiter, ihre Hinweise bleiben im Report. Neue Zahlen: 30 automatisiert,
  21 manuell, 4 weder noch (von 55). Mitbewegt: die Prinzip-Abdeckung (informativ, kein Score-Einfluss)
  und der EN-301-549-Anhang, der diese Klauseln nun als „manuelle Pruefung" statt „automatisch ohne
  Befund" fuehrt. Ein Guard-Test stellt sicher, dass jedes Hint-only-Kriterium eine Katalogregel hat,
  nicht als automatisiert zaehlt und in der Manuell-Liste steht. `docs/PARITY_CONTRACT.jsonc`,
  `docs/PARITY_CONTRACT.md` (stand noch auf WCAG 2.1, 50/36/10) und README nachgezogen. Geprueft:
  `--debug-typ`-Lauf gegen eine lokale Fixture (Pruefumfang „30 von ca. 55", Manuell-Wolke 21),
  `cargo test --lib`, `parity_contract`, `release_contract_tests`, Clippy mit allen Features.

- **Kriterien-Befunde auf blosse Technik-Praesenz entfernt, 2026-09-25 (Plan 45):** Vier Regeln
  meldeten ein WCAG-Kriterium, sobald eine Technik auf der Seite vorkam, die mit dem Kriterium
  zusammenhaengen *koennte* — ohne beobachteten Mangel. **2.2.3 No Timing** schlug auf jedes
  `setTimeout`/`setInterval` in einem Inline-Skript an (Debounce, Karussell-Takt, Analytics),
  **2.2.4 Interruptions** auf jedes `role="alertdialog"`, **2.5.1 Pointer Gestures** auf
  Zeichenketten wie `ontouchstart` (auch reine Feature-Erkennung) und **2.5.4 Motion Actuation**
  auf `deviceorientation`/`devicemotion`/`shake` im Skripttext — letzteres sogar als Verstoss statt
  als Pruefhinweis. Alle vier melden jetzt nur noch den `untested`-Eintrag, dessen Text bereits
  sagt, was manuell zu pruefen ist. Bei **2.5.2 Pointer Cancellation** entfaellt der zweite Zweig
  (irgendein `addEventListener('mousedown'|'touchstart')` im Inline-Skript); der Hinweis auf
  `onmousedown`/`ontouchstart` direkt an einem Bedienelement bleibt. Eine Stelle fuer
  kriterienlose Informationssignale gibt es nicht, die Erkennung ist daher ersatzlos gestrichen.
  Die Pruefhinweise flossen in die `review`-Zaehlung und von dort in Screenreader- und
  Risiko-Aggregation. Geprueft: `cargo test --lib`, Clippy mit allen Features, Detection-Corpus mit
  echtem Chrome gruen.

- **Pruefziel auf WCAG 2.2 AA umgestellt, 2026-09-23:** Das Werkzeug pruefte gegen WCAG 2.1 AA und
  zaehlte die 2.2-Kriterien, die es laengst pruefte, bewusst aus der Quote heraus. Bezugsgroesse ist
  jetzt WCAG 2.2 AA: 55 A/AA-Kriterien (2.1s 50, minus das gestrichene 4.1.1, plus die sechs neuen),
  davon 38 automatisiert. Die WCAG-2.1-Sicht, auf die sich das BFSG ueber EN 301 549 V3.2.1 beruft,
  bleibt als Anhang aus demselben Lauf erhalten; Kriterien, die erst 2.2 gebracht hat, sind dort
  weiterhin als solche gekennzeichnet, weil die Norm sie nicht abdeckt.

  **4.1.1 Parsing** ist in WCAG 2.2 gestrichen und wird nicht mehr als Kriterium gefuehrt. Die
  beiden Pruefungen, die daran hingen, melden jetzt 4.1.2 — dieselbe Zuordnung, die axe-core fuer
  `duplicate-id-aria` fuehrt. Die generische Dopplung (`ids/duplicate`, iframe-Variante) meldet nur
  noch, was auch ein Verstoss ist: eine doppelt vergebene ID, auf die ein IDREF zeigt (`for`,
  `headers`, `aria-labelledby` und die uebrigen ARIA-Verweise). Dann ist die Beziehung nicht mehr
  eindeutig aufloesbar. Eine Dopplung, auf die niemand zeigt, verletzt seit dem Wegfall von 4.1.1
  kein Kriterium mehr und erzeugt keinen Befund. axe-core hat seine Entsprechungen (`duplicate-id`,
  `duplicate-id-active`) aus demselben Grund entfernt.

  **Scores und Quoten verschieben sich dadurch** und sind mit aelteren Reports nicht direkt
  vergleichbar — ein Bezugswechsel, keine Regression. Geprueft an www.casoon.de (Single, PDF ueber
  `--debug-typ` und JSON): Quote 38/55, `wcag_coverage.level` jetzt „WCAG 2.2 AA", `report-lint`
  ohne Befund. Beim Gegenlesen fiel ein Zaehlfehler auf: Der Pruefumfang-Text zaehlte 2.4.12 (AAA
  und neu in 2.2) in die A/AA-Quote hinein; gezaehlt wird jetzt nur, was auch im Nenner steht.
  Alle `help_url`s zeigen jetzt auf `WCAG22/Understanding` bzw. `WCAG22/Techniques`; 2.5.5 dabei
  auf den 2.2-Namen `target-size-enhanced`.

  **3.3.8 Accessible Authentication (Minimum)** ist als eigene Regel dazugekommen, womit die Quote
  auf 39/55 steigt. Ein Verstoss ist nur, was gemessen ist: Auf jedes Passwort- und
  Einmalcode-Feld wird ein synthetisches `paste`-Ereignis mit einem Probewert gefeuert; blockiert
  gilt das Feld nur, wenn das Ereignis abgebrochen wurde **und** der Wert nicht im Feld landet. Ein
  Handler, der das native Einfuegen abbricht und den Text selbst bereinigt einsetzt, bleibt damit
  unbeanstandet. Ein interaktives CAPTCHA in einem Formular mit Passwort- oder Code-Feld ist ein
  Pruefhinweis, kein Verstoss, weil Objekterkennung und Alternativen aus dem DOM nicht
  entscheidbar sind; ein CAPTCHA im Kontaktformular gehoert nicht zu 3.3.8. `autocomplete="off"`
  am Passwortfeld wird bewusst nicht gemeldet — Browser ignorieren es dort.

  Beim Lauf des Detection-Corpus fiel auf, dass die `duplicate_id`-Fixture noch eine
  unreferenzierte Dublette als Verstoss erwartete. Sie prueft jetzt beide Seiten: referenziert
  (`label for`) ist ein Verstoss, unreferenziert keiner. Geprueft: Detection-Corpus mit echtem
  Chrome komplett gruen, `cargo test --all-features`, Clippy fuer beide Feature-Sets,
  www.casoon.de (Regel laeuft, kein Befund, Quote 39/55, `report-lint` ohne Befund).

  **3.2.6 Consistent Help** wird im Batch-/Sitemap-Modus seitenuebergreifend geprueft. Pro Seite
  erfasst `patterns::help_mechanisms` die Hilfe-Mechanismen ausserhalb von `main` in
  Dokumentreihenfolge: Kontakt-/Hilfe-/FAQ-Links, `mailto:`, `tel:` und bekannte Chat-Widgets, jeweils
  mit ihrem Seitenbereich (Kopf, Navigation, Seitenleiste, Fuss, schwebend). Inhalte in `main` und
  unsichtbare Elemente bleiben aussen vor, sonst wuerde jeder Kontakt-Link in einem Artikel als
  Inkonsistenz zaehlen. `audit::batch_consistency` vergleicht alle Mechanismen, die auf mindestens zwei
  Seiten vorkommen, mit der Platzierung der meisten Seiten. Eine Abweichung ist entweder ein
  Mechanismus in einem voellig anderen Bereich (disjunkte Bereichsmengen, ein zusaetzliches Vorkommen
  zaehlt nicht) oder eine vertauschte Reihenfolge zweier Mechanismen im selben Bereich. Ohne
  wiederkehrenden Mechanismus bleibt das Kriterium „manuell pruefen". Im Einzelseiten-Modus steht
  3.2.6 jetzt in der Liste der manuell zu pruefenden Kriterien (11 statt 10), mit deutschem Titel aus
  einer neuen Tabelle fuer die 2.2-Kriterien, die EN 301 549 V3.2.1 nicht enthaelt.

  Dabei fiel auf, dass das Batch-JSON die seitenuebergreifenden WCAG-Bewertungen (3.2.3, 3.2.4,
  3.2.6, 2.4.5) gar nicht ausgab, obwohl `OUTPUT_CONTRACT.md` sie zusagt — sie standen nur im PDF.
  `site_analysis.consistency` traegt jetzt `wcag_cross_page` und `help`. Geprueft: neuer
  Chrome-Integrationstest des Detektors (`tests/help_mechanisms_detection_test.rs`), Unit-Tests fuer
  Bereichs- und Reihenfolge-Abweichung, PDF-Test fuer die lokalisierte Abweichungsliste, Batch-Lauf
  www.casoon.de (12 Seiten): 2 wiederkehrende Mechanismen, keine Abweichung, deutscher Text in der
  Typst-Quelle korrekt.

  **2.5.7 Dragging Movements** steht in der Liste der manuell zu pruefenden Kriterien (jetzt 12).
  Ob eine Ziehbewegung eine Alternative mit einfachem Zeigen hat, ist Verhalten und nicht am Markup
  ablesbar. Ein Hinweis-Detektor (sortierbare Listen, `draggable`) wurde bewusst nicht gebaut: Jede
  Regel mit WCAG-Bezug zaehlt hier als automatisiert, 2.5.7 waere damit in die Quote gerutscht, ohne
  dass je ein Verstoss belegbar ist. Die Quote bleibt 39/55.

  **`summary.wcag_coverage.wcag_version`** (`"2.2"`) macht die Bezugsversion maschinenlesbar. Reports
  von vor der Umstellung haben das Feld nicht und waren auf WCAG 2.1 bezogen. Beide JSON-Schemas
  kennen das Feld, `schema_version` bleibt 2.0 (additiv).

  Beim Referenzlauf auf www.inros-lackner.de fiel ein Zaehlfehler aus dem 4.1.1-Umbau auf:
  `rule_outcomes` meldete fuer `ids/duplicate` 10 Befunde, `violations` enthielt keinen. Der Filter
  auf referenzierte IDs verwarf die Befunde, der Vermerk aus `a11y-rules` wurde aber unveraendert
  weitergereicht. Er wird jetzt um die verworfenen Befunde gekuerzt. Geprueft: Unit-Test,
  `cargo test --all-features`, Clippy fuer beide Feature-Sets, Referenzlaeufe casoon.de (Single +
  Batch) und inros-lackner.de mit `wcag_version` „2.2" und ohne `report-lint`-Befund. inros-lackner.de
  steht bei Barrierefreiheit 47 (am 2026-09-20: 20); die zehn unreferenzierten doppelten IDs zaehlen
  nicht mehr. Wie viel des Sprungs darauf und wie viel auf Aenderungen an der Seite entfaellt, wurde
  nicht isoliert gemessen.

- **Den docs.rs-Build in der CI nachstellen, 2026-09-23 (#588):** Der Melder von #588 verwies auf
  eine Action im Beta-Test, die den Doku-Build im *Sandbox-Image von docs.rs* ausfuehrt — mit
  deren Nightly, deren Ressourcengrenzen und auf dem, was `cargo package` veroeffentlichen wuerde.
  Das ist die Frage, die der bestehende Doku-Schritt nicht beantworten kann: Er baut auf dem
  Runner, mit Stable und ohne Speicherdeckel.

  Der Anlass steht im Build-Log von 1.5.1: **4,75 GB von 6,4 GB** verfuegbarem RAM, 1m35s von 15
  Minuten. Seit `all-features = true` dokumentiert docs.rs auch `ai-transparency`, was den
  Abhaengigkeitsgraphen von 454 auf 565 Crates hebt. Ein Build, der an dieser Decke stirbt, faellt
  erst nach `cargo publish` auf, und eine veroeffentlichte Version laesst sich nicht ersetzen.

  Der Job laeuft **nur auf Tags**: Die Frage lautet „schafft docs.rs diesen Build", und sie ist
  einmal zu beantworten, kurz bevor die Version publiziert wird. Auf jedem Pull Request kostete sie
  sieben Runner-Minuten und einen Cache-Platz fuer einen Commit, den niemand veroeffentlicht.
  `CLAUDE.md` fuehrt den Job jetzt als Pflichtschritt vor `cargo publish`.

  Die Action ist auf einen Commit gepinnt statt auf einen Tag — sie startet fremdes Docker in der
  CI — und der Job laeuft mit `continue-on-error: true`, solange sie Beta ist. Ein neues
  `dependabot.yml` haelt den Pin (und die uebrigen Actions) im Blick.

- **Die restlichen rustdoc-Warnungen abgeraeumt, 2026-09-23:** Nach #588 blieben sieben
  Warnungen stehen, die das Tor damals bewusst durchliess: zwei Verweise auf Namen, die es nicht
  (mehr) gibt (`NodeId` ohne Pfad, `RuleOutcome` statt `Outcome`), drei Verweise aus oeffentlicher
  Doku auf private Items (`defect_key`, `is_layout_only_role`, `score_intent_fit`) — die stehen
  jetzt als Code statt als Link, denn ein Link, dem der Leser nicht folgen kann, ist keiner — und
  zwei nackte URLs in `cli/args.rs`. Die letzten beiden bleiben nackt: Diese Doc-Kommentare sind
  clap's `--help`-Text, bevor sie rustdoc sind, und spitze Klammern oder Backticks stuenden dann
  vor jedem Nutzer im Terminal. Dort steht jetzt ein `#[allow(rustdoc::bare_urls)]` mit genau
  dieser Begruendung.

  Das CI-Tor zieht entsprechend nach: statt `-D rustdoc::invalid_html_tags` jetzt
  `RUSTDOCFLAGS: -D warnings`. Verifiziert mit `cargo doc --all-features --no-deps` (keine
  Warnung), clippy, `cargo fmt --check`, 1546 Unit-Tests und einem Blick auf `--help`, wo die
  Beispiel-URLs unveraendert nackt stehen.

- **docs.rs baut mit allen Features, 1.5.1, 2026-09-22:** Ohne `[package.metadata.docs.rs]` baut
  docs.rs nur mit `default = ["pdf"]`. Alles hinter einem optionalen Feature — etwa `c2pa` — fehlte
  damit in der veroeffentlichten Doku. `all-features = true` behebt das. Die Einstellung wirkt erst
  fuer Versionen, die danach publiziert werden, daher 1.5.1 statt eines Nachtrags zu 1.5.0.

- **Live-Regionen ueber die Zeit beobachtet, Journeys von Messartefakten befreit, 2026-09-22:**
  Die Formularfehler-Journey fragte, ob nach dem Absenden eine `[aria-live]`-Region existiert, die
  vorher nicht existierte. Damit war sie genau fuer die empfohlene Umsetzung blind: Wer den leeren
  Live-Container von Anfang an im Markup hat, bekam ein `FormErrorInvalidWithoutLiveRegion` mit
  Severity High. Die Add-to-Cart-Journey verglich zwei Stichproben und sah deshalb keine Meldung,
  die erscheint und nach zwei Sekunden wieder verschwindet — bei Warenkorb-Feedback der Normalfall.

  Neu ist `interaction::live_regions`: ein `MutationObserver` ueber Live-Regionen, auch in spaeter
  erzeugten Shadow Roots, zeichnet auf, *dass* eine Region Inhalt bekam, mit Zeitpunkt und
  Dringlichkeit. Beide Journeys werten diese Ereignisse aus. Laesst sich der Beobachter nicht
  installieren, entsteht kein Befund, sondern ein `live_observer_unavailable` im Trace — ohne
  Messung ist „nichts angekuendigt" keine Aussage ueber die Seite. Beobachtet heisst nicht
  vorgelesen: Das Modul belegt nur, dass der Browser den Anlass dazu hatte.

  Beim Nachpruefen am Korpus fielen weitere Journeys auf, die Messartefakte meldeten:
  - **Tabs:** `element.click()` verschiebt den Fokus nicht, die Pfeiltaste traf also nie die
    Tab-Liste. 20 von 22 Tabs-Journeys meldeten `TabsSelectionNotMoved` (High). Bewertet wird jetzt
    nur, wenn der Fokus auf dem betaetigten Tab steht.
  - **Menue:** Escape wird nur bewertet, wenn der Fokus im Menue steht, sonst traf die Taste die
    Seite und nicht das Menue.
  - **Modal:** Die Journey liest jetzt `modal` am geoeffneten Dialog. Nicht-modale Dialoge wurden
    bisher pauschal als High-Befund gemeldet; jetzt gilt der Fokus-Einschluss nur fuer modale
    Dialoge, und `FocusTrapBackgroundNotHidden` nur fuer Dialoge, die den Fokus einschliessen, ohne
    sich als modal auszuweisen. Stand der Fokus vor dem Oeffnen auf `body`, ist `body` nach dem
    Schliessen die richtige Wiederherstellung. Ausserdem erzeugte die Kandidatensuche nie einen
    Ausloeser, solange kein Dialog im Baum stand — ein geschlossenes `<dialog>` hat keine Rolle.
    Ueber 171 gelaufene Seiten lief deshalb keine einzige Modal-Journey.
  - **SPA-Navigation:** Titel, URL, Ueberschrift und Fokus kommen aus zwei `AXSnapshot`s statt aus
    `document.querySelector('h1')`; damit zaehlen auch Shadow Roots und `role="heading"`. Die
    Ausgangsaufnahme wird vor jedem Versuch neu genommen, damit ein Fehlversuch dem naechsten keine
    Fokusbewegung unterschiebt.
  - **Disclosure:** Die Groessenordnung der nativen `<details>`-Luecke ist nachgemessen: 22 von 238
    Domains, nicht „280 von 663 Seiten".

  Verifiziert mit `cargo test --all-features --lib` (1546 bestanden), drei neuen
  Chrome-Integrationstests (transiente Fehlermeldung, korrekter modaler Dialog, korrektes Menue —
  jeweils ohne Fehlalarm) sowie clippy `-D warnings` fuer beide Feature-Sets.

- **Rohe HTML-Tags in Doc-Kommentaren, 2026-09-22 (#588):** docs.rs meldete abgeschnittene
  Seiten, etwa `PageHealthAnalysis`: Doc-Kommentare nannten Elemente wie `<iframe>` ohne
  Backticks, rustdoc gab sie als echtes HTML aus, und der Streaming-Rewriter von docs.rs brach
  daran ab. 16 Stellen in sechs Dateien stehen jetzt als Inline-Code. Verifiziert mit
  `cargo doc --all-features --no-deps`: keine `invalid_html_tags`-Warnung mehr. Die CI baut die
  Doku jetzt mit `-D rustdoc::invalid_html_tags`, damit neue Faelle schon vor dem Release auffallen.

- **a11y-core 0.10.0 und die Ueberschriftenregeln abgegeben, 2026-09-21:** Das Projekt hing auf
  `a11y-core` 0.6.0, veroeffentlicht war 0.10.0. Der Sprung ueber vier Minor-Versionen brachte
  keinen einzigen API-Bruch — `dom_document.rs` und `wcag/shared.rs` uebersetzen unveraendert —,
  aber zwoelf zusaetzliche Kennungen: fuenf `landmarks/*`, `keyboard/skip-link-missing`,
  `zoom/viewport-missing`, `headings/h1-multiple`, `links/generic-name`,
  `aria/required-attribute-missing` und die beiden `contrast/*`. Damit stehen 42 geteilte
  Kennungen zur Verfuegung.

  Ein Test fiel dabei, und zwar richtig: Er verlangte, dass mit `Semantics` keine Regel mehr
  mangels Faehigkeit ausfaellt. Das galt bei 0.6.0, weil es dort kein Tier 3 gab. Seit 0.7.0
  melden die Kontrastregeln ohne gerenderte Stile `CapabilityMissing` — das Prinzip „nicht
  gelaufen ist nicht bestanden", angewandt auf auditmysite selbst. Die Zusicherung nennt jetzt
  die beiden Kennungen, statt eine Null zu verlangen, die nichts mehr bedeutet. `Rendering` ueber
  CDP zu bedienen bleibt ein eigener Schritt.

  Mit `headings/h1-multiple` fiel die letzte Luecke, an der `wcag::rules::headings` nur teilweise
  abloesbar war. Das Modul ist geloescht, alle vier Pruefungen kommen aus dem geteilten Bestand,
  `SHARED_RULES` fuehrt 16 statt 12 Kennungen. Beim Abraeumen kam eine Fehlmessung ans Licht: Die
  AX-Fassung sortierte Ueberschriften nach `node_id` *als Text*, womit „h10" vor „h2" stand und
  Spruenge in laengeren Seiten falsch bewertet wurden. Die geteilte Fassung laeuft in
  Dokumentreihenfolge ueber den DOM.

  Zwei gewollte Unterschiede bleiben: Mehrere `h1` sind geteilt `REVIEW` statt Verstoss und melden
  einmal statt je ueberzaehliger Ueberschrift — in HTML sind mehrere `h1` zulaessig. Und „leer"
  heisst geteilt „kein Text im Teilbaum, kein `aria-label`"; eine Ueberschrift, die nur ueber
  `aria-labelledby` oder ein `alt` im Bild benannt ist, meldet die geteilte Fassung zu Unrecht.
  Das ist als Befund fuer `a11y-core` vermerkt, nicht als Rueckausnahme hier.

  Verifiziert: 1.716 Tests gruen, `clippy` und `fmt` sauber, die Korpus-Erwartung `heading_order`
  auf `headings/skip-level` gehoben. Der Vollstaendigkeitstest des Korpus war erneut der Waechter,
  der eine stehengebliebene Kennung gefunden haette.

- **Wartbarkeits-Hotspot 1, 2026-09-20 (Plan 27):** `src/output/pdf/single_report.rs` war mit 3.717
  Zeilen und 21 Aenderungen in 60 Tagen die am haeufigsten angefasste Datei des Projekts und mischte
  Risiken/Staerken, Problem-Profil, Score-Treiber-Tabelle, Subkategorie-Breakdown, Anhang, Findings,
  Root-Cause-Analyse und Roadmap.

  Herausgeloest ist das Problem-Profil-Cluster nach `src/output/pdf/problem_profile.rs` (527
  Zeilen): neun Funktionen, zwei Konstanten und zwei Typen. Der Schnitt war risikoarm, weil nichts
  davon von ausserhalb `single_report.rs` referenziert wurde — nur drei Funktionen mussten von
  privat auf `pub(super)`. `single_report.rs` faellt auf 3.219 Zeilen; der Diff dort enthaelt ausser
  drei Importzeilen und zwei Sichtbarkeits-Anhebungen nur Loeschungen.

  Output-neutral verifiziert: der `--debug-typ`-Lauf von www.inros-lackner.de hat davor und danach
  dieselbe Zeilenzahl und einen identischen Problemprofil-Abschnitt.

  Der Test-Helfer `test_report_view_model` wurde nicht dupliziert, sondern ueber
  `pub(in crate::output::pdf)` sichtbar gemacht — eine zweite Kopie einer 70-Zeilen-Fixture waere
  genau die Art Duplikat, die dieser Plan abbaut.

  Neubewertung fuer Hotspot 2: `normalized.rs` ist mit 4.553 Zeilen und 20 Aenderungen/60 Tagen
  jetzt die groesste Datei und der naechste Kandidat. Details im Plan.

- **Darstellungsartefakte im PDF, 2026-09-20 (Plan 36):** Sechs kleine, unabhaengige Defekte aus
  einem Review der beiden `--debug-typ`-Quellen, jeder wenige Zeilen, keiner eine
  Produktentscheidung.

  - **Anhang behauptete 36 automatisch gepruefte Kriterien und listete 56.** Die Wolke trug den
    Titel der WCAG-2.1-A/AA-Quote, enthielt aber zusaetzlich AAA und die WCAG-2.2-Kriterien, ohne
    dass das irgendwo stand. Jetzt traegt sie ihre echte Zahl plus eine Zeile, die sie aufschluesselt.
    Beim Schreiben ging die Aufschluesselung erst nicht auf -- 36 + 4 + 17 = 57 gegen 56 gelistete --,
    weil ein AAA-Kriterium zugleich WCAG-2.2-only ist und in beiden Gruppen zaehlte. Die drei Gruppen
    sind jetzt disjunkt, und `the_three_appendix_groups_partition_the_automated_criteria` pinnt die
    Summe.
  - **`◐` (U+25D0) wurde still verschluckt.** Kein gebuendelter Font deckt das Zeichen; Typst meldete
    "Dropping U+25D0" und fuenf Zeilen mittlerer Konfidenz begannen mit einer Luecke, wo alle anderen
    ein Symbol hatten. Eine Legende gab es ohnehin nicht. Die Skala ist jetzt ein Wort
    (`[gemessen]`/`[wahrscheinlich]`/`[schwaches Signal]`) -- verifiziert: keine Typst-Warnung mehr,
    und die fuenf Zeilen tragen jetzt ihr Label.
  - **Modul-Takeaway stand zweimal.** Der `section-header-split`-Body wurde zwei Komponenten
    spaeter als eigenes Label wiederholt. Neu `interpretation_beyond_takeaway`: das Label zeigt nur
    noch, was der Kopf nicht schon gesagt hat.
  - **"Header 0/10" in der Erfolgsfarbe.** Der Akzent war auf Teal festgenagelt; er kommt jetzt aus
    dem Verhaeltnis, ueber dieselbe `score_color`-Funktion wie ueberall sonst. inros-lackner 0/10 ist
    rot, casoon 8/10 gruen.
  - **"Details im Methodik-Anhang" fuehrte ins Leere.** Der Anhang wiederholte denselben Satz;
    welche Messungen verworfen wurden, stand nirgends, obwohl
    `execution.navigation.stability` es weiss. Der Hinweis nennt sie jetzt selbst ("Betroffen:
    Desktop-Ansicht nach 1500 ms, Mobile-Ansicht nach 1500 ms"). `mutation_count` wird nur
    genannt, wenn es tatsaechlich gezaehlt wurde -- der Wert faellt sonst auf 0 zurueck, und "noch 0
    DOM-Aenderungen" waere ein Default, gedruckt als Messung.
  - **Widerspruechliche DOM-Einordnung.** "(Performance 75 Pkt stabil)" beruhigte ueber genau das,
    was die Massnahmenliste zwei Bloecke weiter oben priorisiert ("DOM-Struktur verschlanken").
    Jetzt "bei derzeit tragfaehiger Performance" -- beide Fakten, keiner widerspricht dem anderen.

- **„Wenig Text oben" mass Knoten- statt Seitenposition, 2026-09-20 (Plan 51):** Der Befund sagt
  dem Kunden „Wenig sichtbarer Text im oberen Seitenbereich — Nutzer erhalten keine sofortige
  Orientierung". Das ist eine Aussage ueber die *gerenderte* Seite; gemessen wurden die ersten 50
  Knoten des Accessibility-Baums in Dokumentreihenfolge.

  Nach dem Rollen-Fix aus Plan 50 blieben 93 von 2731 gecachten Seiten geflaggt, und diese Reste
  waren keine kleinere Version desselben Problems, sondern das Versagen der Naeherung. Auf
  www.inros-lackner.de sind die ersten 50 Knoten ein Skip-Link und vierzig Wrapper; die
  `article`-Elemente stehen als leere Huellen *vor* ihrem eigenen Text. Baumreihenfolge ist dort
  ueberhaupt keine vertikale Reihenfolge, also haette auch eine Wrapper-ueberspringende Variante
  (gemessen: 13 statt 93) nur eine unkalibrierte Konstante gegen eine andere getauscht.

  Jetzt misst `JourneyModule::collect` in der Seite: ein `TreeWalker` summiert den Text, dessen
  Box in der ersten Viewport-Hoehe liegt. In Dokumentkoordinaten (`rect.top + scrollY`), weil ein
  anderes Modul vorher gescrollt haben kann, und ohne Screenreader-only-Text — derselbe
  `__amsIsVisuallyHidden`-Helfer, den die In-Page-WCAG-Checks nutzen. Dasselbe Muster wie der
  bereits vorhandene `<main>`-DOM-Check des Moduls. Die Baum-Naeherung bleibt als Fallback fuer
  Aufrufe ohne Seite (`analyze_journey`, Tests), damit ein fehlender Messwert nicht als „kein Text"
  gelesen wird.

  Verifiziert in beide Richtungen mit drei Fixtures: Text hinter einem 3000px-Hero wird geflaggt,
  eine Seite mit Ueberschrift und Absatz oben nicht, und 79 Zeichen reiner Screenreader-Text oben
  zaehlen nicht als sichtbarer Text. www.inros-lackner.de wird nicht mehr geflaggt, Journey dort
  65 -> 68; www.casoon.de unveraendert 91. Zwei Fixtures und ein End-to-End-Test in
  `tests/integration_test.rs` halten beide Richtungen fest — die alte Naeherung konnte sie nicht
  auseinanderhalten, weil die Ueberschrift im Baum so oder so vorne steht.

- **Flaky Fixture-Server, 2026-09-20 (Plan 48):** `security_detection_corpus_matches_real_analyze_security_run`
  fiel sporadisch mit einer FALSE-NEGATIVE-Meldung um -- der schlimmstmoeglichen Form eines
  Flakes, weil sie sich wie eine echte Regression in der Security-Erkennung liest. Der Plan
  vermutete einen Port-Race; `bind` auf Port 0 war aber schon da.

  Die tatsaechliche Ursache: der Listener wird non-blocking gesetzt, damit die Accept-Schleife ein
  Shutdown-Flag pollen kann -- und unter macOS/BSD **erbt der akzeptierte Socket dieses Flag**.
  `read` und `write_all` konnten also `WouldBlock` liefern, bevor der Request ueberhaupt da war,
  und beide Ergebnisse wurden mit `let _ =` verworfen. Die Antwort ging ungeschrieben raus, der
  Client sah nichts, `analyze_security` fiel auf leere Header zurueck (was fuer eine Seite, die
  HEAD und GET verweigert, richtig ist) und der Corpus-Diff meldete einen fehlenden Header.

  Belegt statt vermutet: eine Probe zeigte `read -> Err(WouldBlock)` in **jedem** Lauf, und unter
  16-facher Parallellast fiel der alte Test in **29 von 48** Laeufen um, mit exakt der Meldung aus
  dem Plan (`cors_wildcard_credentials`). Nach dem Fix 48 von 48 gruen.

  Der Fix: akzeptierte Verbindung zurueck auf blocking, Request-Kopf bis zum Ende lesen, jede
  Verbindung in einem eigenen Thread (eine Accept-Schleife, die auf einen Client wartet, verhungert
  alle anderen), und Transportfehler werden mitgeschrieben statt verschluckt. Der Test prueft jetzt
  `fixture.served() > 0`, bevor er irgendetwas als Erkennungsfehler wertet -- verifiziert mit einem
  simulierten Transportfehler, der jetzt als solcher gemeldet wird statt als FALSE NEGATIVE.

  Derselbe Server steckte **sechsmal kopiert** in den Integrationstests, jede Kopie mit demselben
  Race. Er liegt jetzt einmal in `tests/common/fixture_server.rs` (298 Zeilen weniger). Die vier
  Chrome-gegateten Suiten laufen damit gruen.

  Dabei fiel ein zweiter, bereits vorhandener Flake auf:
  `test_concurrent_wait_for_stable_stays_within_its_timeout_budget` legte sein 4,65-s-Budget um die
  gesamte Task inklusive `new_page()`, obwohl nur `wait_for_stable` gemeint ist -- auf einer
  ausgelasteten Maschine fiel die volle `--ignored`-Runde deshalb auf dem **unveraenderten** Code
  2 von 2 Mal um. Die aeussere Schranke ist jetzt eine reine Hang-Sicherung; die eigentliche
  Zusicherung misst weiterhin nur `wait_for_stable`. Danach 3 von 3 volle Runden gruen.

- **`StaticText` war fuer jeden text-messenden Leser unsichtbar, 2026-09-20 (Plan 50):**
  `AXTree::iter()` filtert browser-generierte Rollen heraus, `StaticText` darunter. Fuer
  WCAG-Regeln ist das richtig -- ein Befund darf nicht gegen einen Knoten gemeldet werden, den
  niemand geschrieben hat. Fuer alles, was *Text misst*, ist es falsch: auf einer echten Seite
  stehen die sichtbaren Woerter genau dort. Gemessen an zwei gecachten Baeumen: 72-76 % aller
  Zeichen in einem `name` sitzen auf browser-generierten Rollen, und `paragraph`-Knoten tragen
  ueberhaupt nie einen eigenen `name`.

  Damit las `matches!(role, "StaticText" | "paragraph")` ueber `iter()` auf jeder realen Seite
  nichts. Zwei Stellen waren betroffen:

  - **Seitentyp-Klassifikation** (`page_intent.rs`): `text_len` war strukturell 0, also konnte
    `approx_words > 500` nie greifen. Dabei fiel eine zweite tote Konstante auf: der
    Marketing-Fallback verlangte zusaetzlich `approx_words < 300`, was mit einer Summe von 0
    immer zutraf. Mit echten Wortzahlen waeren 222 Seiten von „Marketing" auf „Nicht erkannt"
    gefallen -- ein Landing-Page-Default, der gegen eine nie gepruefte 300er-Schranke verliert.
    Die Klausel ist entfernt, weil sie nie in Kraft war.
  - **Journeys „fruehe Inhalte"** (`analysis.rs`): las `StaticText | heading` und sah damit nur
    Ueberschriften. `LittleEarlyText` -- 20 Punkte Abzug auf Entry Clarity -- feuerte auf
    **1595 von 2731** eindeutigen gecachten Seiten. Mehr als die Haelfte des Webs angeblich oben
    ohne Text.

  Eine Definition statt Rollenlisten pro Call-Site: `AXNode::is_text()` und
  `AXTree::text_nodes()`/`visible_text_len()` zaehlen den Text genau einmal -- `StaticText` und
  nichts sonst, weil Chrome denselben Lauf auf den `InlineTextBox`-Kindern wiederholt und der Name
  einer Ueberschrift oder eines Links aus denselben `StaticText`-Nachfahren berechnet wird
  (gemessen: 30 von 44 Ueberschriften-Namen und 71 von 87 Link-Namen sind wortgleich mit einem
  `StaticText`).

  Verifiziert ueber 2731 eindeutige gecachte Seiten mit dem neuen
  `tests/page_intent_corpus.rs` (`#[ignore]`-gated, liest einen lokalen Artefakt-Cache):
  `LittleEarlyText` faellt von 1595 auf 93, und 68 Seiten wechseln den Seitentyp -- alle Richtung
  Editorial, 48 davon auf einem tatsaechlichen Blog, der vorher als Corporate durchging.

  **Nicht mitgefixt:** das Zeitfenster selbst. `LittleEarlyText` verspricht „wenig Text im oberen
  Seitenbereich", misst aber die ersten 50 Knoten in Baumreihenfolge. Auf
  www.inros-lackner.de stehen dort der Skip-Link und vierzig Wrapper. Eine Variante, die reine
  Wrapper ueberspringt, wurde gemessen (13 statt 93 geflaggt) und verworfen: die `article`-Knoten
  dieser Seite stehen als leere Huellen vor ihrem eigenen Text, Baumreihenfolge ist dort also
  ueberhaupt keine vertikale Reihenfolge. Das braucht Geometrie statt einer neuen Konstante und
  steht als Plan 51.

- **`AXTree` ohne Dokumentreihenfolge, 2026-09-20 (Plan 49):** `AXTree.nodes` ist eine `HashMap`,
  und `iter()`/`headings()` gaben `values()` zurueck -- Hash-Reihenfolge, die Rust pro Prozess
  zufaellig setzt. Detektoren, die den Baum als Dokument lesen, widersprachen sich damit selbst:
  zwei aufeinanderfolgende Audits von www.inros-lackner.de aus demselben Binary, eines meldete
  UX 78, das andere UX 81, ohne dass sich an der Seite etwas geaendert hatte. Betroffen waren die
  Heading-Skip-Erkennung (`last_level` setzt Dokumentreihenfolge voraus) und Journeys "fruehe
  Inhalte" (`iter().take(50)` ueber eine `HashMap` sind fuenfzig beliebige Knoten, nicht der Anfang
  der Seite).

  CDP liefert `getFullAXTree` bereits in Dokumentreihenfolge -- `from_nodes` hat sie nur in die
  `HashMap` geworfen. Neues privates Feld `order: Vec<String>` haelt sie fest; `iter()` und
  `iter_all()` laufen darueber. Faellt die Reihenfolge weg (Snapshot aus dem Cache, der vor dem
  Feld geschrieben wurde, oder ein direkter Schreibzugriff auf das oeffentliche `nodes`), wird sie
  aus `child_ids` per Tiefensuche ab der Wurzel rekonstruiert, Unerreichbares nach Id sortiert
  angehaengt -- degradiert, aber deterministisch.

  Verifiziert: drei aufeinanderfolgende Laeufe ueber www.inros-lackner.de liefern identische
  UX-Befunde und Modulwerte. Journey faellt dabei von 68 auf 65, weil `take(50)` jetzt tatsaechlich
  den Seitenanfang liest. Der bei Plan 43 bewusst ausgelassene `HeadingSkips`-Fall steht jetzt in
  `every_penalty_states_its_own_reason_at_the_first_occurrence` -- er war nicht zu behaupten,
  solange h1 -> h3 nur je nach Lauf ein Skip war.

  **Nicht mitgefixt:** die `StaticText`-Blindheit der `iter()`-Leser, die im "Related"-Abschnitt von
  49 als zwei Call-Sites stand. Der Einzeiler (`iter()` -> `iter_all()`) wurde umgesetzt, gemessen
  und wieder verworfen: er aenderte auf keiner der beiden Testseiten die Klassifikation, und ein
  Guard-Test dafuer bestand auch mit dem Defekt, weil der Keyword-Scan darueber `StaticText`
  ebenfalls nicht sieht. Das ist das eigentliche Problem und steht jetzt als Plan 50.

- **Score-Hierarchie, 2026-09-20 (Plan 29):** Der Report trug rund 30 verschiedene 0-100-Werte,
  14 davon mit Note A-F. Der Gegenstand des Tools -- Barrierefreiheit -- war einer davon, mit 40 %
  Gewicht in einem anderen. Vier Entscheidungen, vier Commits, danach gilt eine Drei-Ebenen-Regel:
  Barrierefreiheit als Ueberschrift, die fuenf uebrigen gewichteten Module als bewerteter Kontext,
  alles andere als Indikator ohne Note.

  - **Note nur fuer Module, die den Gesamtwert tragen (D2):** Sieben der dreizehn Module haben
    `weight_pct: 0`, speisen also nichts, trugen aber dieselbe A-F-Note wie Accessibility. So stand
    "Dark Mode: F" -- eine Note fuer ein optionales Produktmerkmal, das der Code selbst als
    `measurement_type: "optional"` fuehrt -- neben einem F fuer den Ausschluss von
    Screenreader-Nutzern. `ModuleScoreEntry::new` leitet Gewicht, Note und Band aus einer Quelle ab:
    `grade` ist `Option` und nur bei Gewicht > 0 gesetzt, `band` traegt immer das kanonisch
    englische FIVE_BAND-Label. Gleiches fuer die `grade`-Felder von `AiVisibilityAnalysis` und
    `SourceQualityAnalysis` sowie fuer `dark_mode`. Das PDF war nicht betroffen -- es zeigte
    ohnehin Bandwoerter, die Noten steckten nur im JSON.
  - **Zaehler statt Quote, wo der Nenner beliebig ist (D4, schliesst Plan 43 §3):** Content
    Visibility war `(Signale - Probleme) / Signale`, also "Anteil der Checks, die nicht gemeckert
    haben" -- ein Check mehr, der meist besteht, hob den Wert jeder Seite. Der Score entfaellt
    ersatzlos; berichtet werden die Zaehler, die ohnehin schon danebenstanden. Dabei fiel auf, dass
    die Erfolgsmeldung "Alle Content-Visibility-Signale sind in Ordnung" ab `score >= 80` erschien,
    also auch bei bis zu einem Fuenftel Problemsignalen; sie haengt jetzt an `problem_count == 0`.
    Eine Ebene tiefer traegt `SignalCategory` `passed`/`total` statt `score_pct`, und der
    Tabellentitel nennt "17 von 24 Signalen erfuellt" statt eines mit sechs erfundenen Faktoren
    gewichteten Prozentwerts.
  - **Abgeleitete Indikatoren als solche ausweisen (D3, schliesst Plan 42):** Ein Leser sah SEO,
    KI-Sichtbarkeit, Quellenqualitaet, UX und Journey als fuenf getrennte Bewertungen, die zufaellig
    uebereinstimmen. `AiSignalKind::TechnicalTrust` faltete `a11y_score >= 80` und
    `security_score >= 70` in die Zitierbarkeit, `QualitySignalKind::Accessibility` zaehlte den
    Accessibility-Score noch einmal in der Authority-Dimension -- beide entfernt. Die elf
    erfundenen Citation-Gewichte (0.08, 0.15, 0.10, ...) suggerierten eine Kalibrierung, die es
    nicht gibt; jetzt gleichgewichtet unter benanntem `SIGNAL_WEIGHT`. Was bleibt, wird deklariert:
    neues Feld `derived_from`, gespeist aus `taxonomy::module_derived_from`, plus ein Satz im PDF
    unter jedem betroffenen Indikator. Der Accessibility-Abzug auf UX und Journey wird beziffert
    statt still eingerechnet.
  - **Die Ueberschrift ist die Barrierefreiheit (D1, schliesst Plan 41 §3):** Note und Zertifikat
    hingen am gewichteten Gesamtwert (#233, damit sie einander nicht widersprechen). Das loeste den
    internen Widerspruch, rueckte die Kennzahl aber vom Gegenstand des Berichts weg:
    inros-lackner.de mit Barrierefreiheit 20 erschien auf dem Deckblatt als 45, weil Performance,
    Mobile und SEO sie hochzogen; casoon.de mit Barrierefreiheit 100 als 92. Beide lesen jetzt den
    Barrierefreiheits-Score, koennen sich also weiterhin nicht widersprechen. Der gewichtete Wert
    bleibt als benannte ScoreCard "Kombinierter technischer Wert" mit Gewichtsbasis -- ohne sie
    waere er aus dem PDF verschwunden, er stand nur auf dem Deckblatt. `report-lint` prueft gegen
    `accessibility_score`; neue Fixture, weil in allen bisherigen beide Scores gleich sind und
    keine den Wechsel bemerkt haette. Nebenbei zwei weitere Bandtabellen beseitigt: der
    Batch-Builder rechnete Note und Zertifikat mit eigenen Literalen (95/90/85/80/70/60), das
    Batch-Deckblatt mit `BATCH_GRADE` -- derselbe Score war also je nach Bericht anders benotet.
    Beide nutzen jetzt dieselben zwei Funktionen wie der Single-Report; `BATCH_GRADE` entfaellt.

  Verifiziert an frischen Reports von www.casoon.de (Barrierefreiheit 100, kombiniert 92, Note A)
  und www.inros-lackner.de (Barrierefreiheit 20, kombiniert 45, Note F, Zertifikat NICHT
  BESTANDEN); `report-lint` auf beiden ohne Befund.

- **Report-Qualitaetsdurchlauf, 2026-09-19:** Ergebnis eines `report-critic`-Laufs ueber frische
  Single-URL-Reports von www.casoon.de und www.inros-lackner.de. `auditmysite report-lint` war auf
  beiden Reports ohne Befund — alle Punkte sind semantischer Natur und fuer den deterministischen
  Linter strukturell unsichtbar. Sechs Commits, je einer pro Befund.

  - **Erklaerungs-Lookup uebersprang die axe-Kennung:** JSON-Writer und Batch-Report loesten die
    kundenseitige Erklaerung ueber die Taxonomie-`rule_id` auf, `get_explanation` faellt von dort
    auf die blosse WCAG-Nummer zurueck, und die regelspezifischen Eintraege haengen an der
    axe-Kennung. **Befund:** 4 von 15 `detail.fix_guidance`-Eintraegen trugen fremden Text — die
    drei Landmark-Regeln erschienen als "Missing semantic structure" mit Tabellen-Markup als
    Codebeispiel. Das Single-PDF war korrekt, nur das JSON nicht. Dieselbe Fehlerklasse wie #571.
    Neu `resolve_explanation` als einziger Aufloeser an allen drei Aufrufstellen. Beim Fixen fiel
    auf, dass die nun erreichbaren Codebeispiele deutschen Text ins kanonisch englische JSON
    gebracht haetten; der Guard prueft jetzt nicht mehr nur auf Umlaute.
  - **`accessibility_score_breakdown` war ein zweites Score-Modell:** eigene Severity-Gewichte
    (20/14/8/4), Deckel bei 90, erfundene Wichtigkeits-Gewichte. **Befund:** ein
    `accessibility_score` von 20 neben Bereichen, deren gewichteter Schnitt 57 ergab. Ein
    gewichteter Schnitt kann den Score nicht reproduzieren — `diversity_factor`, sqrt-Kompression,
    `apply_soft_floor` und der Cap sind nicht-linear und haengen am gesamten Befundsatz. Die zwei
    vermischten Fragen sind jetzt zwei Felder: `score` ist der Bereich allein durch den Scorer
    (neu `score_from_penalties`), `estimated_lost_points` der Anteil am tatsaechlichen Verlust.
    `sum(estimated_lost_points) == 100 - accessibility_score` gilt exakt und wird von
    `report-lint` geprueft. Schema unveraendert.
  - **Score-Bereiche kamen aus Fliesstext:** `score_area_for_finding` suchte Woerter in
    `rule_id + title + description + subcategory`. **Befund:** `a11y.landmark_region.missing`
    landete unter "Images / alternative text", weil seine Beschreibung `role 'image'` erwaehnt;
    `seo.headings.multiple_h1` trieb einen Bereich des *Accessibility*-Breakdowns. Neu
    `taxonomy::score_area` mit einer totalen `rule_id -> ScoreArea`-Tabelle ueber alle 120
    Accessibility-Regeln und drei Inventar-Tests. `score_area_for_key_point` war eine zweite,
    wortgleiche Kopie derselben Suche und delegiert jetzt.
  - **"Erwartungswert fuer diesen Seitentyp" war ein Literal:** `intent_fit_score` ist eine von elf
    Konstanten und wurde gegen `seo.score` verglichen — andere Funktion, andere Skala. Fuer
    `ThinContent` (28) kippte die Aussage: eine duenne Seite "erfuellte die Erwartung". Der
    Vergleich ist gestrichen; die Zahl bleibt als Qualitaetswert erhalten. Zusaetzlich haengt das
    Gate `intent_fit_score < 65` fuer drei Seitentypen an einer reinen Typ-Konstante, eine
    Media-Heavy-Seite bekam den Ratschlag also unabhaengig von ihrer Qualitaet.
  - **Zwei Modelle fuer dieselben Risiken:** `build_management_risks` fuellte das JSON,
    `compute_dimension_rows` baute daneben das PDF-Panel. **Befund:** das JSON fuehrte 4x high,
    das Panel zeigte 3x bad und 2x warn, zwei high-Dimensionen fehlten ganz, und alle
    belegkraeftigen Begruendungen wurden verworfen. Neu `audit::management_risk` als einzige
    Ableitung nach dem #406-Muster. Dabei fielen drei Aussagen ohne Messgrundlage weg: die
    Conversion-Ketten (die ohnehin kein Ausgabeformat erreichten), der konstante
    Trust-Satz und der Template-Schluss aus einer einzelnen Seite. `RiskTier::Unknown` ist neu —
    ein nicht ausgefuehrtes Modul wurde vorher per `unwrap_or(100)` als *Staerke* gerendert.
  - **Security behauptete Unauffaelligkeit ueber offenen Findings:** Clean-Hinweis und Takeaway
    hingen am Score, nicht an der Befundliste, und `derive_security_recommendations` prueft nur auf
    *fehlende* Header — ein vorhandener, aber fehlkonfigurierter Header erzeugte nichts, und der
    Default behauptete, die Header seien sauber gesetzt. **Befund:** casoon.de, Score 95, drei
    offene CSP-Findings (eines High), und die Sektion las "Sehr gut — keine wesentlichen
    Sicherheitsauffaelligkeiten". Die Bandkorrektur ist jetzt symmetrisch, die Empfehlungen haengen
    an `SecurityIssueKind`, und die ScoreCard nimmt dasselbe korrigierte Band — sonst stand
    "95 — Sehr gut" ueber einem Takeaway "Verbesserungswuerdig". Derselbe Defekt steckte in der
    SEO-Sektion.

- **Heuristik-Hygiene: keine stillen Abzuege mehr, 2026-09-19:** Plan 43, Teile 1 und 2.

  - **Abzuege ohne genannten Grund:** In `ux/analysis.rs` senkten drei Pruefungen den Score voellig
    stumm (Wortzahl 50-100, lange Seite mit unter fuenf Ueberschriften, unter drei
    Vertrauens-Stichworten), vier weitere zogen ab dem ersten Vorkommen ab, meldeten aber erst ab
    einer deutlich hoeheren Schwelle. **Befund:** `reports/inros-lackner-audit.json` wies
    `content_clarity: 80` aus, ohne dass irgendein ContentClarity-Befund in der Liste stand — 80
    ist genau ein 20-Punkte-Abzug, der stille Wortzahl-Zweig. Die Regel stand schon im Code: der
    `LargeDom`-Zweig traegt den Kommentar, der Score duerfe nie ohne genannten Grund fallen —
    angewendet war sie an genau einer Stelle. Jetzt meldet jeder Abzug; damit das nicht von stumm
    auf alarmierend kippt, ist `ux_issue_severity` wertabhaengig: milde Baender melden `low`. In
    `journey/analysis.rs` hatten fuenf Abzuege keinen Friction-Point, dafuer gibt es fuenf neue
    `FrictionKind`-Varianten.
  - **Binaere Schwellen neben der Prozentzahl, die sie ignorieren:** Die
    Cross-Page-Konsistenzsignale waren Booleans, waehrend der Report den echten Prozentwert
    ausgab — 94 % HSTS-Abdeckung zaehlte exakt wie 0 %, sichtbar neben einer "94 %". `QualitySignal`
    hat jetzt ein `fulfilment` (0.0-1.0); `present` bleibt die Pass/Fail-Markierung, gerechnet wird
    proportional. `ErrorFreePages` benutzt das `clean_pct`, das es ohnehin schon berechnet und
    verworfen hatte. Echte Ja/Nein-Signale bleiben unveraendert.
  - **Dabei gefunden (Plan 49):** `AXTree` iteriert ueber eine `HashMap`, und die
    Heading-Skip-Erkennung liest das trotzdem — zwei Laeufe derselben Seite lieferten
    unterschiedliche Befunde. Ausserdem zaehlte `analyze_content_clarity` Text ueber `iter()` und
    fuehrte `StaticText` als erste Rolle auf, obwohl `iter()` genau diese Rolle herausfiltert: der
    Zweig konnte nie greifen, und der Abzug war ein **Falschbefund**. Mit `iter_all()` geht
    inros-lackner.de von `content_clarity` 80 auf 100. Hier behoben, weil sonst aus einem stillen
    Falschbefund ein sichtbarer geworden waere.

- **Verdict, Einstufung und Herleitung des Gesamtwerts, 2026-09-19:** Fortsetzung des
  Report-Qualitaetsdurchlaufs, Plan 34 zusammen mit Plan 44.

  - **Der Gesamtwert war nicht herleitbar:** Die Tabelle "Warum ist der Gesamtwert, was er ist"
    listete die Module mit ihren Gewichten; nachgerechnet ergab das 90.05 gegen ausgewiesene 92
    (casoon) und 47.05 gegen 45 (inros-lackner). **Befund:** es gibt zwei Rechenwege, und nur einer
    ist die Modulgewichtung. Im `viewport_weighted`-Modus entsteht der Wert aus einer
    Desktop/Mobile-Mischung mit eingemischter Security — das steht in `score_breakdown` und wurde
    nirgends gerendert. Die Tabelle heisst dort jetzt "Die gewichteten Module im Einzelnen", eine
    zweite Tabelle zeigt die echte Rechnung und reproduziert den Wert exakt. Im
    `module_weighted`-Modus bleibt alles, wie es war — dort *ist* die Gewichtung die Herleitung.
  - **Verdict und Einstufung fehlten im PDF vollstaendig:** `summary.certificate` wurde nur als
    Gate gelesen, `verdict`/`verdict_reasons` gar nicht gerendert. Das `SummaryBlock`-Feld namens
    `verdict` ist ein Prosa-Satz, ein anderer Wert unter gleichem Namen. **Befund:** die CLI
    druckte "FAIL — legal_flags: 5, blocking_issues: 36", im Dokument stand davon nichts, und das
    Deckblatt las "Ausbaufaehiger technischer Zustand". Entschieden: das PDF nennt beides, das
    Label heisst "Gesamteinstufung" statt "Zertifikat" — `calculate_certificate` bildet den Score
    auf ein Bandlabel ab und zertifiziert nichts. Ein Teil-Lauf darf eine Einstufung tragen, aber
    als vorlaeufige, und das steht jetzt vorn statt nur im Anhang.
  - **Die Verdict-Gruende waren Maschinen-Tokens:** `legal_flags: 5` unveraendert zu rendern haette
    kanonisches Englisch ins deutsche PDF gebracht. `VerdictReasonKind` traegt jetzt die Zahlen,
    `text(en)` ist die einzige leserseitige Quelle; `VerdictResult::reasons` behaelt seine exakten
    Strings fuer JSON und CLI, `reason_kinds` ist `serde(skip)`.

- **Umstellung auf die geteilten a11y-core-Crates, zweiter Abschnitt, 2026-09-19:** Alle drei
  Crates von 0.2.0 auf 0.3.0. Damit fällt der Blocker des ersten Abschnitts weg: `RuleRun` führt
  jetzt `viewport` und `wcag`, und `Finding` hat mit `rule_name`, `with_element(role, name)` und
  dem undurchsichtigen `Extra`-Slot die Felder, an denen die Umstellung von `Violation` zuvor
  scheiterte.

  - **`RuleOutcome` durch `a11y_report::RuleRun` ersetzt:** Der Ausführungsvermerk kommt aus dem
    geteilten Crate. Der Schlüssel eines Vermerks ist damit `(rule_id, viewport)` — dieselbe Regel
    läuft je Viewport einmal, und `rule_runs` und `findings` benutzen dieselbe Namensmenge.
    **Befund:** Von den sieben Werten der lokalen `RuleOutcomeStatus` sagten vier dasselbe aus —
    dass die Regel gelaufen ist. Kein Auswerter im Code hat `ViolationsFound`, `Warning`,
    `ManualReviewRequired` und `NoViolationDetected` je unterschieden; gelesen wurden ausschließlich
    `Failed` und `Skipped`. Die vier waren schreibend tot. Dass sie zusammenfallen, ist kein
    Verlust, sondern der Zwei-Achsen-Schnitt: *wie sicher* eine Aussage ist, steht am Befund
    (`Outcome`), nicht am Vermerk. **JSON ändert sich** in `pages[].detail.rule_outcomes[]`:
    `status` entfällt (`not_run` sagt jetzt, ob und warum eine Regel nicht lief),
    `wcag_criterion` → `wcag` (Array statt Einzelwert), `reason_code` → `reason`,
    `finding_count` → `findings`.
  - **Die Übersetzungsschicht in `wcag::shared` fällt weg:** Beide Seiten sprechen dasselbe Modell,
    der Vermerk aus `a11y-rules` wird durchgereicht statt übersetzt. Ergänzt wird nur das
    WCAG-Kriterium, das der geteilte Bestand am Vermerk nicht mitführt.
  - **Zwei weitere Regeln abgelöst und gelöscht, nicht danebengestellt:**
    `parsing::check_parsing_with_page` (axe-Kennung `duplicate-id`) läuft jetzt als
    `ids/duplicate`, `focus_order::check_positive_tabindex_with_page` als
    `keyboard/positive-tabindex`. Beide lasen den DOM per JavaScript, weil weder `id` noch
    `tabindex` AX-Eigenschaften sind (#QA-030); die geteilten Regeln lesen dieselben Attribute aus
    dem CDP-Abzug. **JSON ändert sich:** `rule_id` ist für diese Befunde `ids/duplicate` bzw.
    `keyboard/positive-tabindex`. **Severity sinkt** in beiden Fällen von `High` auf das `Medium`
    des geteilten Bestands — das wirkt auf die Bewertung und ist bewusst nicht lokal
    überschrieben, weil ein eigener Schweregrad die Zusicherung bräche, dass derselbe Befund
    überall gleich heißt.
  - **Was danebensteht und warum:** `parsing::check_parsing` (`duplicate-id-aria`, widersprüchliche
    `aria-owns`-Beziehungen im AX-Baum) und `focus_order::check_focus_order` (fokussierbar trotz
    `aria-hidden`) bleiben — sie prüfen etwas anderes als die abgelösten Hälften. `iframe_rules`
    erzeugt weiterhin `duplicate-id`, aber für das Dokument *im* iframe.
  - **Befund an der eigenen Testabdeckung:** Die Kennungsänderung an `keyboard/positive-tabindex`
    schlug in `patterns_disclosure.expected.json` durch — dieser Korpusfall erwartete
    `focus-order-semantics`, ausgelöst durch das `tabindex="5"` in seinem Fixture, nicht durch die
    ARIA-Hälfte derselben Kennung. Zwei Regeln hinter einer axe-Kennung lassen sich am Korpus also
    nicht auseinanderhalten.
  - **Befund an `docs/OUTPUT_CONTRACT.md`:** Der Vertrag behauptete, `rule_outcomes` benutze
    dieselben Ausführungszustände wie `module_runs` (`completed`, `partial`, `failed`, `skipped`,
    `not_applicable`). Das stimmte schon vor dieser Umstellung nicht — `RuleOutcomeStatus`
    schrieb `violations_found`, `no_violation_detected`, `warning`, `manual_review_required`.
    Aufgefallen ist es erst, weil `docs/json-report.schema.json` `rule_outcomes` als
    unbeschränktes `{"type": "array"}` führt und die Form der Einträge gar nicht prüft. Der
    Vertragstext ist nachgezogen.
  - **Noch offen:** `Violation` ist weiterhin auditmysites eigener Typ und nicht
    `a11y_report::Finding` — der Umbau ist jetzt zwar möglich, aber mit 295 Konstruktionsstellen
    und 155 betroffenen Dateien zu groß für diesen Abschnitt. Von 24 geteilten Kennungen sind
    jetzt 4 übernommen (`document/lang-missing`, `document/lang-invalid`, `ids/duplicate`,
    `keyboard/positive-tabindex`), 20 stehen aus.
  - **Warum nicht mehr Regeln:** Ein Fähigkeitsvergleich Regel für Regel zeigt ein systematisches
    Muster statt einzelner Lücken: auditmysites Regeln entscheiden über Chromes **berechnetem**
    Accessibility-Tree, die geteilten über **Tags und Attribute**. Wo beide dasselbe Kriterium
    bedienen, ist die AX-basierte Fassung deshalb meist die breitere. Drei belegte Fälle:
    `lists/invalid-structure` prüft nur `<ul>`/`<ol>` mit fremden Kindern, während
    `rules::list_structure` zusätzlich leere Listen und `<dt>` ohne `<dd>` kennt und über Rollen
    auch `role="list"` auf einem `<div>` erfasst; `tables/header-missing` sucht `<th>`, während
    `rules::table_rules` `columnheader`/`rowheader` als Rollen sucht und damit auch
    ARIA-ausgezeichnete Kopfzellen findet; `zoom/viewport-locked` vergleicht den
    `content`-Wert ohne vorheriges Kleinschreiben und übersieht damit `user-scalable=NO`, und es
    kennt die Abstufung nicht, mit der `rules::resize_text` `user-scalable=no` (`High`) von
    `maximum-scale<2` (`Medium`) trennt. Diese drei sind Kandidaten dafür, den geteilten Bestand
    zu **stärken** statt auditmysite zu schwächen — das gehört aber in einen eigenen Pull Request
    in a11y-core und muss dort veröffentlicht sein, bevor auditmysite es nutzen kann.

  - **Die drei Lücken sind in a11y-core behoben, aber noch nicht veröffentlicht:** Ein eigener
    Pull Request dort (Branch `feat/staerkere-strukturregeln`, vorgesehen als 0.4.0) schreibt den
    `content`-Wert des Viewports vor dem Vergleich klein, lässt `role="list"`/`role="listitem"`
    und `role="columnheader"`/`role="rowheader"`/`role="table"` gleichberechtigt neben den Tags
    gelten und ergänzt die Kennung `lists/empty`. Solange das nicht auf crates.io steht, kann
    auditmysite es nicht nutzen — die drei Regeln bleiben bis dahin auditmysite-eigen.

  *Verifiziert:* `cargo clippy --all-targets` ohne Befund; `cargo test --lib` 1404 Tests und
  `cargo test --tests` alle browserfreien Binaries grün. Diesmal **mit** Chrome gelaufen, was der
  erste Abschnitt schuldig blieb: `cargo test --test detection_corpus_test -- --ignored` grün
  (218 s) — dort zeigt sich, dass beide Kennungsänderungen wirklich tragen, und genau dort fiel
  die Fixture-Lücke bei `patterns_disclosure` auf. `cargo test --test integration_test --
  --ignored` liefert 22 bestanden, 1 fehlgeschlagen — **derselbe Stand wie auf `main`**, gegen den
  eigens gegengeprüft wurde: `test_concurrent_wait_for_stable_stays_within_its_timeout_budget`
  fällt auch ohne diese Änderungen durch, sobald die volle Suite nebenläufig läuft. Es ist ein
  lastabhängiger Zeitbudget-Test, kein Regressionsbefund — aber ein eigener, bisher nicht
  vermerkter.

- **Umstellung auf die geteilten a11y-core-Crates, erster Abschnitt, 2026-09-19:** auditmysite
  bezieht Befundmodell und einen ersten Teil des Regelbestands aus
  [a11y-core](https://github.com/casoon/a11y-core) (alle Crates 0.2.0), damit derselbe Befund in
  astro-post-audit, auditmysite und liveaudit dieselbe Kennung trägt. Bewusst schrittweise: Dieser
  Stand stellt das Modell und **eine** Regel um, nicht den ganzen Bestand.

  - **Doppelte Typdefinitionen entfernt:** `Severity` und `ViolationEvidence` waren strukturell
    identisch zu `a11y_report::Severity` bzw. `a11y_report::Evidence` — gleiche Varianten, gleiche
    Reihenfolge, gleiche serde-Darstellung. Beide sind durch die geteilten Typen ersetzt; die
    auditmysite-eigenen Zusätze an `Severity` (deutsche Labels, Umsetzer aus den Alt-Systemen)
    hängen als `SeverityExt` daran, weil an einem fremden Typ keine inhärenten Methoden ergänzt
    werden können. Das frühere `Display for Severity` fällt der Orphan-Regel zum Opfer; die eine
    Fundstelle nutzt `as_str()`, das exakt dieselben Strings liefert. **JSON unverändert.**
  - **`FindingKind` durch `a11y_report::Outcome` ersetzt:** Variante für Variante dasselbe Enum
    unter anderem Namen. **JSON ändert sich:** Das Feld `kind` schreibt jetzt
    `"fail"`/`"review"`/`"pass"`/`"untested"` statt
    `"violation"`/`"warning"`/`"positive"`/`"not_testable"`. Kein Fixture und kein Snapshot
    beobachtete das Feld — die Änderung wäre unbemerkt geblieben, deshalb pinnt ein neuer Test die
    Vokabel jetzt ausdrücklich.
  - **CDP-DOM als `a11y-dom`-Dokument (`accessibility::dom_document`):** Die geteilten Regeln sind
    generisch über ein bewusst **DOM-förmiges** Trait, weil die Mehrzahl von ihnen über Tags und
    Attribute entscheidet (`lang`, `tabindex`, `id`, `alt`). Der AXTree gibt die gar nicht her.
    Neu wird der DOM einmal per `DOM.getDocument` geholt und in eine Arena materialisiert;
    `Semantics` wird über Chromes **nativen** Accessibility-Tree erfüllt, nicht über `accname` —
    das ist der Ersatz für Hosts ohne nativen Tree. Beide Bäume finden über die Backend-Node-ID
    zueinander. Wurzel ist `<html>` (die Regeln prüfen `doc.root()`), Shadow-Root-Kinder hängen
    unter ihrem Host, iframe-Inhalte werden nicht betreten (eigenes `lang`/`title`).
  - **3.1.1 läuft als geteilte Regel, die eigene ist gelöscht:** auditmysite prüfte 3.1.1 zweimal
    und beide Male schlecht. `rules::language::check_language` las die AX-Eigenschaft `language`,
    die Chrome aber aus Locale und Kontext synthetisiert, auch wenn der Autor nie ein `lang`
    gesetzt hat — für den häufigsten Verstoß also blind. Ausgeglichen wurde das durch
    `pipeline::apply_lang_attribute_check`, eine zweite, per JavaScript nachgeschobene Prüfung, die
    den Befund der ersten zurücknahm oder nachtrug. Ungültige Sprachcodes kannte keine von beiden.
    Beide sind gelöscht; 3.1.1 läuft jetzt als `document/lang-missing` / `document/lang-invalid`
    direkt gegen das DOM-Attribut. **JSON ändert sich:** `rule_id` ist für diesen Befund
    `document/lang-missing` statt `html-has-lang`. `html-has-lang` bleibt bestehen, wird aber nur
    noch von `iframe_rules` erzeugt — für das Dokument *im* iframe, ein anderer Befund.
  - **„Nicht gelaufen" ist nicht „bestanden":** Die geteilten Kennungen, die auditmysite noch nicht
    führt, werden je Kennung als `shared_rule_not_yet_adopted` vermerkt statt stillschweigend
    verworfen. Fällt der DOM-Abruf aus, werden sie als `Failed` vermerkt, nicht als bestanden.
  - **Noch offen:** `Violation` ist weiterhin auditmysites eigener Typ und nicht
    `a11y_report::Finding` (es fehlen dort Felder, die auditmysite pro Befund mitführt:
    `rule_name`, `role`, `name`, `evidence_screenshot`, `evidence_viewport`); `RuleOutcome` ist
    nicht auf `RuleRun` umgestellt; von 24 geteilten Kennungen ist eine Regel mit 2 Kennungen
    übernommen.

  *Verifiziert:* `cargo clippy --all-features --all-targets` ohne Befund; 1667 Tests in 18
  browserfreien Testbinaries bestanden, 0 fehlgeschlagen. Die browsergestützten Korpus-Tests
  (`detection_corpus_test`) brauchen Chrome und liefen dabei nicht.

- **a11y-core 0.6.0, 2026-09-19 — drei Regeln abgelöst:** `list_structure`, `table_rules` und
  `meta_viewport_large` sind gelöscht; ihre Prüfungen kommen jetzt aus dem geteilten Bestand.
  `SHARED_RULES` führt damit 12 statt 4 Kennungen.

  Möglich wurde das erst, nachdem a11y-core die Lücken geschlossen hat, an denen die Ablösung
  vorher gescheitert wäre: Begriff ohne Definition, präsentationale Tabelle mit Kopfzellen,
  Tabellenname, die Trennung von Zoom-Verstoß und Zoom-Begrenzung — und beim Abräumen fiel ein
  vierter auf, der verwaiste Listeneintrag (`<li>` ohne Liste darüber). Die alte Regel lief über
  den AX-Tree und sah ihn; die geteilte lief über Listen und hätte ihn nie besucht. Ohne
  `lists/item-outside-list` in 0.6.0 hätte die Ablösung eine Prüfung verloren statt sie zu teilen.

  **Eine bewusste Abweichung:** Die fehlende Tabellenbenennung ist im geteilten Bestand
  `needs_review`, nicht `violation` — ob eine Tabelle einen Namen braucht, hängt vom Kontext ab,
  und ein Verstoß wäre eine Behauptung, die die Regel nicht decken kann. Das Korpus hält das fest.

  `meta_viewport_large` verschwindet aus `PAGE_RULES` (47 → 46); die dort liegenden Viewport-Helfer
  sind nach `resize_text.rs` gewandert, das als einzige Nutzerin bleibt und über die
  200-%-Schwelle hinaus die Textvergrößerung selbst prüft.

  *Verifiziert:* 1.430 browserfreie Tests, 27 Testbinaries, clippy `-D warnings` und `fmt` sauber,
  Browser-Korpus mit echtem Chrome (217 s). Der Vollständigkeitstest des Korpus war der Wächter,
  der die drei verbliebenen Fixture-Verweise auf die alten Kennungen gefunden hat.

- **a11y-core 0.4.0, 2026-09-19 — was noch nicht ablösbar ist:** Mit 0.4.0 lesen die geteilten
  Struktur­regeln auch `role`-Attribute (`role="list"`, `role="columnheader"`, `role="table"`), und
  `positive-tabindex` steht wieder auf `High`. Drei naheliegende Ablösungen bleiben trotzdem aus —
  sie wären ein Fähigkeitsverlust, nicht ein Tausch:

  - **Listen:** `lists/invalid-structure` und `lists/empty` decken zwei der drei Prüfungen aus
    `rules/list_structure.rs` ab. Es fehlt, ob ein `<dt>` eine zugehörige Definition hat.
  - **Tabellen:** `tables/header-missing` deckt nur die Kopfzellen ab. `rules/table_rules.rs` prüft
    zusätzlich Caption bzw. Accessible Name und ob präsentationale Tabellen fälschlich Kopfzellen
    führen.
  - **Viewport:** `zoom/viewport-locked` prüft `maximum-scale < 2.0` und entspricht damit
    `rules/resize_text.rs` (1.4.4, 200 %) — **nicht** `rules/meta_viewport_large.rs`, das dieselbe
    Auszeichnung bei der strengeren 500-%-Schwelle prüft. Eine Ablösung von `meta_viewport_large`
    wäre die falsche Zuordnung gewesen.

  *Verifiziert:* Bau und 1.442 browserfreie Tests gegen 0.4.0, clippy `-D warnings` sauber. Die
  drei Lücken gehören in die nächste a11y-core-Runde, nicht in eine lokale Sonderlocke.

- **1.5.0, 2026-09-19 — Lizenzwechsel auf MIT:** auditmysite steht ab dieser Version unter der
  MIT-Lizenz. Frühere Releases bleiben unter der Lizenz, die zum jeweiligen Zeitpunkt galt — bis
  0.25.x AGPL-3.0-or-later, 0.26.0 bis 1.4.0 Business Source License 1.1; das ist in `NOTICE`
  festgehalten und muss nicht rückwirkend geändert werden.

  *Warum:* Der Regelbestand wandert nach [a11y-core](https://github.com/casoon/a11y-core), damit
  derselbe Befund in astro-post-audit (Build-Zeit), auditmysite (CI/Crawl) und liveaudit (laufende
  Seite) gleich heißt. Läge auditmysite weiter unter BUSL, müsste bei jedem Modul, das dorthin
  wandert, einzeln über die Lizenz entschieden werden — eine Abgrenzung, die dauerhaft gepflegt
  werden müsste und quer zum Ziel steht.

  *Unumkehrbar:* Ein einmal unter MIT veröffentlichter Release lässt sich nicht zurückholen.
  Bewusst so entschieden.

- **1.4.0, 2026-09-18 — Upgrade-Hinweise:** Kein reines Patch-Release. Was Konsumenten des
  JSON-Reports und der Bibliothek beim Hochziehen prüfen sollten (Details jeweils in den Einträgen
  unten):
  - **Gesamtscores verschieben sich:** HTML-Conformance zählt seit dem Per-Ursache-Scoring-Fix mit
    5 % in den Gesamtscore, SEO sinkt von 20 % auf 15 %. Reports derselben Seite ergeben andere
    `overall_score`-Werte als unter 1.3.x.
  - **Geänderte JSON-Werte** (nicht nur ergänzte Felder): `rich_snippets_potential` nennt den
    bloßen Schema-Typ (`"FAQPage"` statt `"FAQ Rich Snippet"`); die Schlüssel in
    `mobile.issues[].values.small_by_context` sind kanonisch englisch (`other`/`form` statt
    `sonstige`/`formular`).
  - **Zwei SEO-Befunde entfallen:** `pagination_missing_rel_links` und
    `pwa_missing_service_worker` werden nicht mehr emittiert.
  - **Additiv:** `security.issues[].values` und `mobile.issues[].values` tragen die interpolierten
    Rohwerte; leer werden sie weggelassen. `header`, `issue_type` und `message` bleiben unverändert.
  - **Rust-API bricht:** `security::coop_corp_verification_text` ist entfallen (in
    `security_issue_text` aufgegangen), `SecurityIssue` und `MobileIssue` haben ein neues
    öffentliches Feld `values`, und `a11y_journey::link_inventory::analyse` nimmt jetzt Seiten- und
    Fallback-Sprache statt einer Locale.
  - Bekannter Konsument: Studio hing auf 1.3.1 (siehe Auto-Memory) — vor dem Upgrade prüfen.

- **Report-Aussagen: Widersprüche und Überbehauptungen behoben, 2026-09-18:** Ausgelöst durch eine
  externe Kritik am casoon.de-Report. Der Report behauptete an mehreren Stellen mehr, als die
  Messung hergibt, und widersprach sich dabei selbst.
  - **Linktext-Heuristik erzeugte falsche WCAG-2.4.4-Behauptungen:** `a11y_journey::link_inventory`s
    `is_generic` verglich per `lower.contains(stopword)`, also als Substring. Damit galt jeder
    beschreibende Linktext als generisch, der irgendwo ein Stoppwort enthielt — an casoon.de
    „Claude-Code-Kontingent: Der Verlauf kostet **mehr** als jede Konfiguration" (200 Zeichen
    Kartentext), „**Mehr** über Jörn Seidel", „**Mehr** Sichtbarkeit bei Google & KI"; die
    englische Liste traf über `here`/`more`/`open` zusätzlich mitten in deutsche Wörter. Alle vier
    gemeldeten Links waren False Positives. Jetzt exakter Vergleich auf dem normalisierten Namen
    (lowercase, umschließende Nicht-Alphanumerik entfernt, sodass „Mehr erfahren »" weiterhin
    trifft). Zusätzlich folgt die Detektionssprache jetzt der **Seiten**-Sprache (`<html lang>`,
    per `eval_string` im Journey-Hook gelesen) statt der Lauf-Sprache, mit expliziter
    `STOPWORD_LOCALES`-Prüfung — `I18n::new` liefert für unbekannte Locales stillschweigend das
    deutsche Bundle, eine französische Seite wäre sonst mit deutschen Stoppwörtern gescannt worden
    (#406-Trennung Detektions- vs. Message-Sprache). Vier Regressionstests, darunter die drei
    echten casoon.de-Namen. Der Befundtext behauptet außerdem nicht mehr „erfüllen WCAG 2.4.4
    nicht", sondern kennzeichnet sich als heuristische Warnung mit manuellem Prüfbedarf.
  - **Drei Evidenzklassen statt einer:** Neue `EvidenceClasses` (bestätigte WCAG-Verstöße /
    heuristische Warnungen / manuell zu prüfende Kriterien) auf dem `FindingsBlock`, einmal im
    Builder berechnet. **Scores bleiben unverändert** — sie werden weiterhin allein aus den
    bestätigten Verstößen abgeleitet. Die Klassen existieren, damit kein Zählsatz mehr „0 Befunde"
    sagen kann, während vier Seiten später Befunde stehen: Die Ursachenanalyse sagt jetzt „Keine
    bestätigten WCAG-Verstöße … Es bleiben N Warnung(en) und M Prüfhinweis(e)" statt „Es wurden
    keine Barrierefreiheits-Befunde erkannt", und unter dem Zählstrip der Management-Sicht steht
    explizit, dass dort ausschließlich bestätigte Verstöße gezählt werden. Der „Heuristische
    Warnungen"-Metrikwert zählt jetzt auch die Journey- und Screenreader-Signale, nicht mehr nur
    die Warnungen der WCAG-Engine.
  - **BFSG-Aussagen auf technische Kriterien zurückgeführt:** „BFSG-Prüfung: keine Verstöße im
    geprüften Umfang" (grün) und die Checklisten-Zeile „Rechtliche Konformität (BFSG) — Geringes
    Compliance-Risiko" lasen sich wie eine rechtliche Freigabe. Ein Scanner kann weder feststellen,
    ob ein Dienst überhaupt unter das BFSG fällt, noch ob sämtliche gesetzlichen Anforderungen
    erfüllt sind. Beide heißen jetzt „BFSG-relevante technische Kriterien" und sagen explizit
    „Dies ist keine rechtliche Konformitätsprüfung."
  - **Breadcrumb-Widerspruch (`seo::serp`):** Das SERP-Signal prüfte nur die Typ-Präsenz von
    `BreadcrumbList` und meldete dann „Google zeigt Pfadangabe im Listeneintrag" — während die
    Schema-Tabelle desselben Reports „Pflichtangaben fehlen: itemListElement (minimum 2 entries)"
    ausgab. Das Signal konsultiert jetzt die bereits vorhandenen `rule_assessments`; ein
    unvollständiges Schema wird zur Warnung mit Nennung der fehlenden Felder. Zwei
    Regressionstests.
  - **Maßnahmenliste modulübergreifend:** „Die 5 wichtigsten Maßnahmen" wurde ausschließlich aus
    WCAG-Findinggruppen gespeist und meldete bei 0 WCAG-Befunden „Keine dringenden Maßnahmen
    erforderlich" — obwohl derselbe Report Duplicate Host, fehlerhaftes Breadcrumb-Schema, 13.756
    DOM-Knoten und eine CSP-Fehlkonfiguration dokumentierte. Neu füllt `cross_module_measures` die
    Liste aus Security-, Mobile-, SEO- und Performance-Empfehlungen auf, sortiert nach Schwere und
    danach nach den Overall-Punkten, die das Modul kostet. Der Leertext des Maßnahmenplans
    verneint keine Befunde mehr, sondern verweist auf die Modulabschnitte.
  - **Terminologie und PDF-Artefakte:** TBT wird nicht mehr als Core Web Vital ausgewiesen
    (`perf-lab-data-body` nennt LCP/CLS als CWV und TBT als Lab-Näherung, weil INP headless nicht
    messbar ist); die Abschnitte „Desktop/Mobile — Core Web Vitals" heißen „Kennzahlen
    (Labormessung)", weil ihre Strips auch FCP und TTFB enthalten, die Zusatztabelle heißt „Weitere
    Lab-Metriken". Die Schwellwerte für DOM-Knoten, Ladezeit und DOMContentLoaded sind als
    „Richtwert" statt „Ziel" beschriftet (Lighthouse-Heuristik, keine Norm). Interne Journey-IDs
    (`tab_walk`, `disclosure_0`, `skip_link_0`) erscheinen nicht mehr roh im Kunden-PDF, sondern als
    Labels, mit Rückfall auf die rohe ID bei unbekannten Journeys. Der Callout über den
    interaktiven Befunden ist nicht mehr grün (der Erfolgszweig war ohnehin unerreichbar, weil der
    Abschnitt nur bei vorhandenen Befunden gerendert wird) und ist grammatikalisch korrekt im
    Singular. `rich_snippet_type` heißt nicht mehr „FAQ Rich Snippet", sondern nennt den bloßen
    Schema-Typ — vorhandenes Markup ist keine Zusage über die Suchdarstellung. Das Gesamturteil
    sagt „im automatisierten Accessibility-Prüfumfang" statt „im Accessibility-Audit".
  - Verifiziert: `cargo test --all-features` (1376 Lib- plus Integrationstests grün),
    `cargo clippy --all-features --all-targets` ohne Warnungen, `cargo fmt --check`, sowie ein
    echter Lauf gegen www.casoon.de — der Linktext-Befund ist verschwunden (2 → 1 interaktive
    Befunde), das Breadcrumb-Signal steht auf „Warnung" mit Begründung, die Maßnahmenliste führt
    fünf modulübergreifende Punkte, und `report-lint` meldet weiterhin keine Findings. Score
    unverändert (Accessibility 100, Overall 92).
  - **Zwei fachlich fragwürdige SEO-Regeln entfernt (Nachtrag desselben Tages):**
    `pagination_missing_rel_links` meldete fehlendes `rel="prev"`/`rel="next"`-Markup als
    SEO-Problem — Google nutzt das Paar seit 2019 nicht mehr zur Indexierung. Die übrigen
    Pagination-Prüfungen bleiben: sie greifen nur, wenn rel-Links tatsächlich vorhanden und falsch
    sind (Selbstreferenz, prev == next), und das ist in jedem Fall kaputtes Markup.
    `pwa_missing_service_worker` meldete eine fehlende Service-Worker-Registrierung, sobald ein
    Web-App-Manifest vorhanden war — ein Manifest wird aber routinemäßig allein für Icons und
    Theme-Color ausgeliefert (an casoon.de exakt dieser Fall: gültiges Manifest, `display:
    standalone`, keine Registrierung) und belegt keine Absicht, eine installierbare, offline-fähige
    PWA zu sein. Die `service_worker_*`-Felder bleiben als Information im JSON. Beide Tests durch
    Negativtests ersetzt. Ergebnis an casoon.de: „Technische SEO-Probleme" schrumpft von drei auf
    eine Zeile (den echten Duplicate-Host-Befund).
  - **Maßnahmenliste fasst gleichartige Befunde zusammen:** Nach dem Entfernen der beiden
    SEO-Regeln rückten drei separate CSP-Fehlkonfigurationen nach und belegten drei der fünf
    Plätze. `cross_module_measures` fasst jetzt Befunde mit gleichem Modul und gleichem Titel zu
    einer Maßnahme zusammen und hängt „(+N weitere gleicher Art)" an — drei CSP-Befunde sind eine
    Maßnahme „CSP korrigieren". `TopMeasure` trägt dafür Titel und Detail getrennt; Performance-
    und technische SEO-Empfehlungen haben keinen eigenen Titel und werden ohne Trenner gerendert.
    Drei Rendering-Tests. Dabei mitkorrigiert: die Mobile-Einträge trugen den rohen
    snake_case-Kategorieschlüssel (`touch_targets`, `fonts`) — sie nutzen jetzt dasselbe
    `mobile_category_label`, das der Mobile-Modulabschnitt schon verwendet („Touch-Targets",
    „Schriftgrößen").
  - **Security- und Mobile-Meldungen lokalisiert (Nachtrag desselben Tages, #406):** Beide Module
    backten englische Prosa in `message` und der PDF-Renderer gab sie unverändert im deutschen
    Report aus („CSP allows unsafe-inline scripts without nonce/hash protection", „1 touch targets
    are too small"). Beide folgen jetzt dem kind-Enum-Muster: `SecurityIssueKind` (19 Varianten)
    bzw. `MobileIssueKind` (6) plus `*IssueValues` für die interpolierten Rohwerte, und je eine
    reine `security_issue_text` / `mobile_issue_text` als EINZIGE Textquelle — die Analyse ruft sie
    mit `en=true`, die PDF-Präsentationsschicht über `localized_message()` mit der Lauf-Sprache.
    Die Konstruktoren `security_issue`/`mobile_issue` bauen `message` daraus ab, sodass an keiner
    Aufrufstelle mehr ein Meldungstext von Hand steht. `kind()` wird aus den bereits gespeicherten
    `header`/`issue_type`-Feldern abgeleitet statt als viertes Feld serialisiert, damit Reports und
    gecachte Artefakte älterer Builds unverändert weiterlokalisieren; unbekannte Typen fallen auf
    die gespeicherte kanonische Fassung zurück. JSON-Vertrag bleibt identisch (`header`,
    `issue_type`, `message` unverändert englisch; `values` ist additiv und wird leer weggelassen),
    abgesichert durch `security_issue_types_match_the_stored_contract`. Dabei mitkorrigiert: das
    Mobile-Seitenskript schrieb die deutschen Kontextschlüssel `sonstige`/`formular` ins
    eigentlich englisch-kanonische JSON — jetzt `other`/`form`, lokalisiert über
    `mobile_context_label` (unbekannte Schlüssel gehen unverändert durch). Die
    #578-Sonderlösung `coop_corp_verification_text` ist entfallen, sie ging in
    `security_issue_text` auf. Guard-Tests pro Modul (jede Variante lokalisiert, EN ohne Umlaute)
    plus Fallback- und Round-Trip-Tests.
  - **Check-Inventar aus dem Enum statt aus dem Quelltext:** `tests/common/nonwcag_rule_inventory.rs`
    ermittelte die kanonischen Security-Check-IDs, indem es `src/security/mod.rs` nach
    `SecurityIssue { header: "…", issue_type: "…" }`-Stringliteralen durchsuchte — durch den
    Umbau standen dort keine Literale mehr und der Corpus-Test schlug fehl. Die IDs kommen jetzt
    aus `security::all_security_check_ids()`, abgeleitet aus `SecurityIssueKind::ALL`; der
    Textscan samt hand-gepflegter `CSP_DYNAMIC_DIRECTIVE_IDS`-Liste entfällt, und die
    CSP-Pflichtdirektiven stehen als `CSP_REQUIRED_DIRECTIVES` an einer Stelle.
  - **Prüfumfang aufs Deckblatt (Nachtrag desselben Tages):** Die Scope-Zeile stand nur auf Seite 2,
    das Deckblatt zeigte „Website-Qualitätsbericht · casoon.de · 92" und einen nackten
    „Barrierefreiheit 100"-Gauge — wer dort aufhört, liest beides als Aussage über die ganze
    Website. Die Zeile kommt jetzt aus einer gemeinsamen Quelle (`single_report::audit_scope_line`)
    und steht im Cover-Untertitel; das Label über dem Gauge-Streifen lautet „MODULE · 0–100 IM
    AUTOMATISIERTEN PRÜFUMFANG · HÖHER IST BESSER".
  - **Dabei einen Layout-Bug erzeugt und behoben:** Der erste Versuch hängte die Scope-Zeile an den
    vollen Kicker an, der Untertitel umbrach auf zwei Zeilen und schob die Labels der unteren
    Gauge-Reihe (Mobile, UX, Journey) aus der Seite — das Typst-Template gibt jedem Gauge-Label
    eine feste 22pt-Box und die Coverseite wächst nicht, der Überlauf wurde still verworfen. Per
    gerendertem PDF entdeckt, nicht im Typst-Quelltext sichtbar. `narrative-cover-kicker` ist
    deshalb jetzt knapp („Technischer Website-Check" statt „… mit Fokus auf Accessibility, SEO und
    Performance") — die weggefallenen Bereiche benennt der Gauge-Streifen direkt darunter ohnehin.
    Neuer Wächter `test_cover_subtitle_stays_on_one_line` (Zeichenbudget 95, gemessen: ~83 passen,
    132 brachen um) plus `test_cover_carries_scope_line_and_gauge_qualifier`.
  - **Bewusst nicht geändert:** die Score-Inflation — siehe
    `plan/29-report-claim-integrity-followups.md`.

- **Produktdokumentation: Widersprüche zum tatsächlichen Verhalten korrigiert, 2026-09-17:** Drei
  bestätigte Abweichungen behoben. (1) `README.md`s HTML5-Conformance-Beschreibung sagte noch
  "score-neutral", obwohl das Modul seit dem Per-Ursache-Scoring-Fix vom 2026-09-16
  (`contributes_to_overall: true`, `weight_pct: 5`) real in den Gesamtscore einfließt. (2)
  `src/lib.rs`s Modulübersicht listete "HTML" als Report-Format, obwohl kein HTML-Export existiert.
  (3) Die Kurzbeschreibung sagte an mehreren Stellen (`Cargo.toml`, `src/lib.rs`, `src/cli/args.rs`,
  `src/cli/plan.rs`s Laufzeit-CLI-Banner, `CLAUDE.md`, `docs/ARCHITECTURE.md`) pauschal "WCAG 2.1",
  während `README.md` einzelne WCAG-2.2-Regeln (2.4.11/2.4.12, 3.3.7, 2.5.8) bereits korrekt
  einzeln kennzeichnet — überall auf "WCAG 2.1 AA (+ select 2.2 criteria)" vereinheitlicht. Dabei
  einen weiteren, direkt benachbarten Fund mitkorrigiert: `src/lib.rs`s "Comprehensive: Checks
  WCAG 2.1 Level A, AA, and AAA" widersprach `README.md`s eigener Aussage "AAA is not fully
  implemented yet" — jetzt präzise: 36 von 50 automatisierten A/AA-Kriterien
  (`docs/PARITY_CONTRACT.jsonc`), plus ausgewählte 2.2- und einzelne AAA-Kriterien.
  `docs/browser-architecture.md`s veraltetes Homebrew-Formula-Beispiel bewusst unangetastet
  gelassen — die Datei deklariert sich selbst bereits als alte, nicht vollständig umgesetzte
  Vorschlagssammlung, nicht als aktuelle Verhaltensdokumentation. Drei neue, maschinell prüfbare
  Regressionswächter in `scripts/release-check.sh` (unqualifizierte "WCAG 2.1 Accessibility
  Checker"-Phrase, HTML5-Conformance-"score-neutral", HTML als Report-Format), je mit manuellem
  Negativfall verifiziert (alten Text testweise wiederhergestellt → Check schlägt fehl;
  zurückgesetzt → wieder grün).
- **Release-Workflow: Qualitätsgates vor dem Artefakt-Build erzwungen, 2026-09-17:**
  `.github/workflows/release.yml`s `build`-Job hing bisher nur von `verify-version` ab — Tests,
  Formatierung, Clippy und das Dependency-Advisory-Gate waren keine Voraussetzung für einen
  `v*`-Tag-Release. `.github/workflows/ci.yml` bekommt einen zusätzlichen `workflow_call:`-Trigger
  (neben `push`/`pull_request`), `release.yml` ruft es jetzt als neuen Job `quality-gates` per
  `uses: ./.github/workflows/ci.yml` auf demselben Commit-SHA auf, den `build` jetzt zusätzlich zu
  `verify-version` als Voraussetzung hat — technisch über den SHA abgesichert (derselbe Checkout),
  nicht nur über den Branch-Namen. `browser-smoke`/`coverage` laufen dabei bewusst nicht erneut mit
  (ihr eigenes `if: github.event_name == 'push' || ... == 'pull_request'` schließt
  `workflow_call` bereits aus) — die Chrome-/Corpus-Tests liefen schon beim Merge auf `main` und
  laufen erneut lokal über den pre-push-Hook (`scripts/release-check.sh`), bevor überhaupt ein
  Release-Tag gepusht wird; sie im Release-Gate zu wiederholen hätte jeden Release unnötig
  verlangsamt. Beide Plattform-Build-Schritte bekommen `--locked`, damit Releases reproduzierbar
  aus dem eingecheckten `Cargo.lock` gebaut werden. Zwei neue `release_contract_tests`
  (`test_release_build_requires_quality_gates_to_pass_first`,
  `test_release_build_uses_locked_lockfile`) verhindern, dass der Release-Job künftig wieder ohne
  Gate auf Artefaktbau zuläuft — Negativfall verifiziert (`needs:` auf nur `verify-version`
  zurückgesetzt → Test schlägt fehl).
- **Dependency-Sicherheit: RustSec-Gate ergänzt, 2026-09-17:** `cargo audit` meldete 9
  Schwachstellen im `Cargo.lock`. Patch-Level-Updates ohne API-Auswirkung eingespielt:
  `rustls` 0.23.40 → 0.23.45 (RUSTSEC-2026-0285), `crossbeam-epoch` 0.9.18 → 0.9.21
  (RUSTSEC-2026-0204), `quinn-proto` 0.11.14 → 0.11.18 (RUSTSEC-2026-0185). `lopdf`
  (Dev-Dependency, PDF-Testhelfer) von 0.34 auf 0.45 angehoben (RUSTSEC-2026-0187) — die
  verwendete API (`Document::load_mem`, `.objects`, `.catalog()`, `.get_pages()`,
  `.extract_text()`) blieb über den Versionssprung unverändert, alle PDF-Tests weiterhin grün.
  Zwei Advisories bleiben offen und sind in `.cargo/audit.toml` einzeln mit Begründung
  dokumentiert statt pauschal ausgeblendet: `rsa` (RUSTSEC-2023-0071, nur über die optionale
  `ai-transparency`/`c2pa`-Kette, kein Upstream-Patch verfügbar, auditmysite hält keinen privaten
  RSA-Schlüssel) und `quick-xml` (RUSTSEC-2026-0195/-0194, zwei transitive Kopien über
  `hayagriva`/`syntect`, jeweils bereits am von den Elternpaketen erlaubten Versions-Deckel —
  Fix erfordert ein Upstream-Release der `typst`/`renderreport`-Kette; betroffene Eingaben sind
  gebündelte, nicht angreifer-kontrollierte Daten wie Zitationsstile und Theme-`.plist`-Dateien,
  nicht Inhalte geprüfter Webseiten). Neuer CI-Job `dependency-audit` (`.github/workflows/ci.yml`)
  führt `cargo audit` auf jedem PR aus.
- **Screenreader-Modul: Lesereihenfolge zählte Layout-Artefakte, 2026-09-17:** Ausgelöst durch die
  Prüfung eines RankingLab-Reports (Sachsen-Anhalt): Der Abschnitt „Screenreader-Lesereihenfolge"
  meldete 237 „angekündigte Knoten" und vier Befunde „Langer Abschnitt ohne Landmark, Überschrift
  oder Fokusziel" — bei gleichzeitig 100/100 Heading- und Landmark-Qualität. Lokal an
  www.sachsen-anhalt.de exakt reproduziert. Ursache: `linearize` emittierte jeden nicht-ignorierten
  AX-Knoten als `ReadingItem`, darunter 67 `InlineTextBox` — Blinks Zeilenfragmente, in die ein
  `StaticText` durch Zeilenumbruch zerlegt wird. Sie existieren für Caret-/Auswahl-Bounds, werden
  nie an eine assistive Technologie gereicht, und trugen denselben Text zwei- bis viermal. Dadurch
  war jede aus der Lesereihenfolge abgeleitete Zahl aufgebläht: Die Desert-Heuristik zählte eine
  News-Teaser-Karte aus vier Ankündigungen (Schlagzeile, Datum, Teaser, Bildnachweis) als
  23-Einträge-Wüste; alle vier Befunde der Seite waren False Positives, erzeugt allein vom
  Zeilenumbruch. Neu: `linearizer::is_layout_only_role` überspringt `InlineTextBox` beim Emittieren,
  traversiert aber weiter (Nachfahren bleiben erhalten); `analyzer::carries_announced_content`
  zählt für die Desert-Distanz nur noch Items mit eigenem Namen oder Wert, sodass leere Wrapper
  (`figure`, `paragraph`, `group`, `generic`, `Figcaption`, `list`) nicht mehr dieselbe Ankündigung
  doppelt zählen — rollen-agnostisch, ein `aria-label`ter `group` zählt weiterhin.
  `CONSENT_WALL_NODE_THRESHOLD` von 200 auf 145 nachkalibriert, weil die Knotenzahl um denselben
  Anteil sinkt. Ergebnis an sachsen-anhalt.de: 237 → 170 Knoten, vier False Positives verschwunden,
  der eine echte Befund (Button ohne zugänglichen Namen, 4.1.2 → § 12 Nr. 3 BFSGV) bleibt; Tab-Stopps
  und alle drei Qualitätswerte unverändert. Verifiziert per Vorher/Nachher-Lauf und 4 neuen
  Unit-Tests.
- **HTML-Konformität: Score zählte Fundstellen statt Ursachen, 2026-09-16:** Das Modul stand auf
  `weight_pct: 0` mit dem Report-Hinweis, sein HTML5-Schema kenne keine RDFa-/Open-Graph-Attribute
  und `og:`-Meta-Tags erzeugten deshalb False Positives. Beides war überholt: Die RDFa-Lücke ist
  seit `html-conform` 0.2.1 geschlossen (`schema/html5/meta.rnc`, Block „RDFa Lite Property
  Metadata"); an einer Seite mit `og:`-Tags direkt gegengeprüft — kein einziger solcher Befund.
  Der eigentliche Defekt war die Strafformel `error_count * 10`: Sie rechnete pro *Fundstelle*,
  nicht pro *Ursache*. Ein fehlerhaftes Template-Element erzeugt eine Fundstelle pro Rendering,
  also floorte jede reale Seite bei 0 — bei sachsen-anhalt.de gingen 201 der 208 gemeldeten Fehler
  auf ein einziges `<link as=…>` ohne `rel` zurück, insgesamt 6 distinkte Ursachen. Neu:
  `html_conform::defect_key` normalisiert Regel-ID plus Meldung (Backtick-Werte und
  `at Zeile:Spalte` entfernt) zu einer Ursache; `score_findings` bestraft pro Ursache, mit
  gedecktem Häufungszuschlag (max. das Doppelte der Basis) und Rangdämpfung (erste Ursache voll,
  zweite/dritte halb, Rest ein Viertel), damit der Score über die ganze Spanne aussagekräftig
  bleibt statt bei 0 zu sättigen. Sachsen-anhalt.de damit 0 → 64. Das Modul zählt wieder in die
  Gesamtwertung (5 %, aus SEO umgeschichtet: SEO 20 → 15) und meldet `measurement_type: "measured"`
  statt `"heuristic"`. Report-seitig: neues Feld `distinct_defect_count` im JSON, Kennzahl
  „Ursachen" im PDF, Findings-Tabelle eine Zeile pro Ursache mit Spalte „Vorkommen" (statt 20
  redundanter Fundstellen-Zeilen), `ScoreCard` wieder mit normalem Notenband, der veraltete
  RDFa-Hinweis durch eine Erklärung der Ursachen-Zählung ersetzt. Verifiziert an sachsen-anhalt.de
  (JSON + `--debug-typ`) und mit 8 neuen Unit-Tests.
- **PDF-Abbruch durch fremde Glyphen und Panic im HTML-Content-Model, 2026-09-16:** (1) Ein
  einzelnes Icon-Font-Zeichen auf der auditierten Seite brach die gesamte PDF-Erzeugung ab:
  Linktexte, Selektoren und Custom-Property-Namen tragen Private-Use-Codepoints in den Report,
  Typst fällt auf `LastResort` zurück und der PDF/UA-Export scheitert
  (`PDF/UA-1 error: the text "\u{e900}" could not be displayed`). Betraf 8 von 84 Domains eines
  RankingLab-Laufs (u. a. kit.edu, fu-berlin.de, uni-goettingen.de). Neu: `output/pdf/sanitize.rs`
  entfernt Private-Use-Zeichen aus allen Strings des `RenderRequest` (ein Chokepoint nach
  `builder.build()`, deshalb ohne Feld-für-Feld-Pflege) und lernt die übrigen nicht darstellbaren
  Zeichen aus dem Compile-Fehler: Zeichen verwerfen, neu rendern, max. 8 Runden — so fiel auf
  jyu.fi ein `↳` aus einem CSS-Variablennamen. Nur Präsentationsschicht; das kanonische JSON
  behält den Originaltext, damit Selektoren kopierbar bleiben. (2) `enclosing_tag_stack`
  (`wcag/rules/html_content_model.rs`) panickte auf ikea.com, weil der Byte-Offset des externen
  Parsers mitten in einem `\u{a0}` lag — entgegen der eigenen Zusage „never panics". Offset wird
  jetzt auf die Zeichengrenze zurückgesetzt, der Raw-Text-Close-Tag-Vergleich läuft über Bytes
  statt `str`-Slices. Verifiziert an den Live-Domains plus je zwei Regressionstests.
- **1.3.1-Fixes, 2026-09-16 (#581, #582, #583):** (1) Die Accessibility-Subkategorie- und
  Security-Kategorie-Scores aus `d6daf7b` erschienen nur im PDF, nicht im JSON-Report — der
  JSON-Builder (`build_page`/`PageDetail`) las sie nie. Jetzt als
  `pages[].detail.accessibility_subcategory_scores`/`security_category_scores` (`{ name, score }`,
  kanonisch Englisch) im Single-Report, mit Schema-, Registry- und `OUTPUT_CONTRACT.md`-Eintrag;
  Batch-Details bleiben kompakt und lassen beide weg. (2) Der CI-Schritt für die vier
  Detection-Corpus-Binaries brach beim ersten Fehlschlag ab, die übrigen liefen nie (so im ersten
  1.3.0-CI-Lauf passiert) — jetzt ein Schritt pro Binary mit `if: !cancelled()`, `release-check.sh`
  sammelt Fehlschläge und bricht erst am Ende ab. (3) `Cargo.lock` hielt das gelöschte
  `spin 0.9.8` (über `lazy_static`) — auf 0.9.9 gehoben.
- **Leerseite nach leerem Report-Teil behoben, renderreport 0.5.1, 2026-09-15:** der
  `pdf-smoke`-Test `test_single_pdf_technical_pages_are_not_blank_when_pdftoppm_is_available`
  schlug fehl: im Technical-Fixture folgte auf die Teil-3-Trennseite (seit plan/22 mit eigenem
  abschließendem `PageBreak`) kein Modulinhalt, sondern direkt der Anhang mit eigenem `PageBreak` —
  dazwischen eine leere Seite. Ursache lag in renderreport: die Engine schrieb für
  `LayoutHint::AlwaysNewPage` (nur `page-break` nutzt ihn) einen harten `#pagebreak()` vor jede
  eigentlich weiche Umbruch-Komponente, zwei benachbarte Umbrüche ergaben so eine Leerseite.
  renderreport 0.5.1 schreibt dort einen weichen Umbruch (mit Regressionstest, der ohne Fix rot
  war). Echte Reports waren in drei Live-Läufen (Standard, Technical, ohne `--full`) nicht
  betroffen, weil Teil 3 dort immer Inhalt hat.
- **Batch-Fortschritt vor dem Report abschließen, 2026-09-15 (#580):** `run_batch_mode`
  (`src/cli/runners.rs`) rief `finish_verdict` erst nach `output_batch_report` bzw.
  `output_batch_as_single_reports` auf — im TTY zeichnete der aktive Balken über den Report, in
  einer Pipe kam die Verdict-Zeile nach dem Report (Regression gegen das #530-Abnahmekriterium).
  Die Reihenfolge ist jetzt in `BatchLifecyclePresenter::finish_then_render` festgelegt, beide
  Zweige nutzen sie. Regressionstest `lifecycle_is_finished_before_report_renders`; Live per Pipe
  mit der Reproduktion aus dem Issue verifiziert (Verdict-Zeile vor dem Report, ANSI-frei).
- **`CLAUDE.md`s „Current State"-Historie nach `CHANGELOG.md` ausgelagert, 2026-09-06
  (plan/11-claude-md-version-drift.md):** die Datei war auf 1058 Zeilen gewachsen, davon 805
  reine chronologische Historie. `CLAUDE.md` behält jetzt nur Arbeitsregeln + einen kurzen
  Verweis hierher; neue Einträge kommen künftig hier rein, nicht mehr in `CLAUDE.md`. Reiner
  Cut-Paste, kein Eintrag inhaltlich verändert.
- **Plan-Review-Batch komplett abgeschlossen, 2026-09-06 (Punkte 7–11):** Punkt 7 (Management-
  Summary verdichten) auf Nutzerwunsch als „leichtere Verdichtung" statt exakt 2 Seiten
  umgesetzt: `render_module_split_dashboards`s Stärken/Schwächen-Karten zeigten dieselben 4-5
  gewichteten Module (Accessibility/Performance/Security/Mobile) bereits ein drittes Mal (nach
  Radar-Chart und der neuen Score-Driver-Tabelle aus Punkt 2) — jetzt auf `measurement_type !=
  "measured"` gefiltert, zeigt also nur noch die nicht-gewichteten Indikator-Module (UX,
  Journey, Search Experience, …), inklusive angepasster „alles gut"-Fallback-Nachricht (scoped
  auf „zusätzliche Indikator-Module" statt pauschal „alle Module"). Neuer Hebelwirkungs-Satz auf
  der Maßnahmenplan-Seite („Die N wichtigsten Ursachen erklären rund X % der erkannten
  Vorkommen — ihre Behebung hat die größte Hebelwirkung …") — teilt sich die Berechnung mit dem
  Diagnose-Satz aus Punkt 3 über eine neue gemeinsame `compute_problem_concentration`-Funktion
  (eine Zahl, zwei unterschiedlich formulierte Sätze auf zwei Seiten, kein Duplikat-Text). Beide
  Änderungen per `--debug-typ`-Smoke-Test verifiziert (Karten verschwinden korrekt, wenn keine
  Indikator-Module vorhanden sind; Hebelwirkungs-Satz rechnet korrekt „4 Ursachen, 50 %, 8
  Vorkommen" auf der Fixture). Punkte 8–11 (Detection-Corpus-Tests im Release-Skript, README-
  Genauigkeitsanspruch, Datei-Größen-Bestätigung, Versionsnummer + Modul-Liste) waren kleine,
  unabhängig verifizierte Fixes — siehe `plan/status.md` für die Details je Punkt.
- **Report-Review-Batch plan/1–6 umgesetzt, 2026-09-06 (externes Report-Review zu
  inros-lackner-de):** sieben destillierte Punkte, sechs umgesetzt (Punkt 7 — Management-Summary
  4→2 Seiten — bewusst zurückgestellt, Layout-Entscheidung braucht separate Nutzer-Bestätigung).
  **Punkt 1 (P0) legte einen deutlich größeren Bug frei als gemeldet:** der gemeldete Fall (ein
  `landmark-unique`-Finding zeigte den falschen, generischen WCAG-1.3.1-Titel "Fehlende
  semantische Struktur" statt "Landmarks nicht eindeutig benannt") war ein einfacher fehlender
  `explanations.rs`-Eintrag (#571-Muster). Die Root-Cause-Untersuchung deckte aber einen
  zweiten, strukturellen Bug in `wcag_group_key` (`src/audit/normalized.rs`) auf: jeder `axe_id`
  ohne eigenen `LEGACY_WCAG_MAP`-Eintrag fällt auf die rohe WCAG-Kriteriums-ID als Gruppierungs-
  schlüssel zurück — teilen sich zwei *unabhängige* Checks ein Kriterium (z. B. mehrere
  1.3.1-Landmark-Checks, mehrere 4.1.2-ARIA-Checks), vermischen sich ihre Violations zu einem
  einzigen Finding mit summierter Occurrence-Zahl und dem Titel des zufällig ersten Treffers. Ein
  systematischer Scan von `src/wcag/rules/*.rs` fand **43 betroffene axe_ids über 10 WCAG-
  Kriterien** (1.1.1, 1.3.1, 1.4.4, 2.1.1, 2.4.1, 2.4.4, 2.4.7, 3.3.1, 4.1.1, 4.1.2) — nicht nur
  die 14 axe_ids unter 1.3.1, die der gemeldete Fall vermuten ließ. Jeder betroffene axe_id bekam
  einen eigenen `taxonomy::rules::Rule`-Eintrag + `LEGACY_WCAG_MAP`-Eintrag; 32 davon zusätzlich
  eine neue zweisprachige `RuleExplanation`, 3 (`region`, `focusable-no-role`, `link-name-context`)
  hatten bereits eine passende Explanation, brauchten nur den fehlenden Rule/Map-Eintrag. Zwei
  axe_id-String-Kollisionen zwischen unabhängigen Checks entdeckt und aufgelöst (Umbenennung, da
  ein globaler `LEGACY_WCAG_MAP`-Eintrag sonst beide Checks fälschlich zusammengeführt hätte):
  `labels.rs`s 4.1.2-Check `"label"` → `"control-missing-label"` (Kollision mit `instructions.rs`/
  `form_rules.rs`s absichtlich geteiltem 3.3.2-`"label"`), `link_purpose_link_only.rs`s 2.4.9-Check
  `"link-name"` → `"link-name-only"` (Kollision mit `link_purpose.rs`s 2.4.4-`"link-name"`). Ein
  bereits bestehender Test (`tests/wcag_coverage.rs::no_undocumented_severity_collisions_in_group_key_mechanism`,
  Teil der QA-009-Nachverfolgung) bestätigte den Fix unabhängig: er verlangte, 5 der 6
  dokumentierten `ALLOWED_MIXED_SEVERITY_GROUPS`-Ausnahmen zu entfernen, weil ihr Severity-Mix
  nach dem Fix nicht mehr auftritt — die verbleibende `3.3.2`-Ausnahme ist ein bewusst geteilter
  axe_id (kein Grouping-Bug). **Punkt 2** (Score-Driver-Tabelle): neue `ModulesBlock.module_scores`
  (roh aus `NormalizedReport.module_scores`, nicht aus dem für die Radar-Karten umgeformten
  `dashboard`, da z. B. die "Sichtbarkeit & Nutzerverständnis"-Karte SEO mit heuristischen
  KI-Sichtbarkeits-Signalen zu einem Komposit-Score verschmilzt, der nicht der rohe SEO-Score mit
  SEO's echtem Gewicht ist) speist eine neue Tabelle direkt nach dem Qualitätsprofil-Radar:
  Modul/Score/Gewichtung/Einordnung ("Größter Schwachpunkt"/"Erheblicher Risikotreiber"/
  "Stabilisiert Gesamtwert"), Einordnung nach `(100-score)*weight_pct`-Drag statt rohem Score (ein
  per Live-Smoke-Test gefundener und gefixter Bug: die erste Fassung sortierte nach Score mit
  Gewichtung nur als Tie-Breaker und wählte fälschlich das niedrig gewichtete Security 30 statt
  des hoch gewichteten Accessibility 40 als "größten Schwachpunkt" — genau der Fall, den der
  Review als Beispiel nannte). **Punkt 3**: neuer generierter Diagnose-Satz ("N Vorkommen erkannt,
  rund X % auf 4 wiederkehrende Ursachen zurückführbar, besonders betroffen: …") direkt nach der
  Score-Driver-Tabelle, Scope bewusst Accessibility+SEO kombiniert (Nutzerentscheidung). **Punkt
  4**: "Root Cause"-Formulierungen vereinheitlicht auf vorsichtigeren Wortlaut ("X Vorkommen
  folgen demselben Muster — deutet stark auf … hin, bestätigt erst mit Belegen von weiteren
  Seiten"), dabei einen unabhängigen #406-Lokalisierungs-Leak gefunden und gefixt
  (`src/output/pdf/findings.rs`: das "Root Cause"-Karten-Label war fest Englisch, auch in
  deutschen Reports) sowie ein identisches hartcodiertes Duplikat in `builder/batch.rs`
  mitgezogen. Batch-Reports' `TemplateCluster::confidence` (`confirmed`/`likely`) erfüllte die im
  Plan gestellte Frage nach stärkerer, tatsächlich begründeter Formulierung bei Mehrseiten-Evidenz
  bereits — keine Änderung nötig. **Punkt 5**: neues Scoring-Subsystem, da entgegen der
  Plan-Annahme noch gar kein echter 0–100-Subkategorie-Score existierte (nur eine
  Taxonomie-Kategorisierung für Labels und ein heuristischer Top-3-Text). Accessibility:
  `compute_accessibility_subcategory_scores` partitioniert dieselben rohen `Violation`s nach
  `taxonomy::Subcategory` (via der schon vorhandenen `wcag_group_key`-Auflösung) und lässt den
  echten `AccessibilityScorer::calculate_score` pro Subcategory-Teilmenge laufen — kein neu
  erfundenes Penalty-Schema. Security hat keine WCAG-äquivalente Rule-Registry; neues
  `security::SecurityCategory` (Response-Header/Transportsicherheit/Zugriffs-Richtlinien/
  Drittanbieter-Exposition) gruppiert `SecurityIssue.header`-Strings und nutzt dieselbe
  Severity→Penalty-Zuordnung wie `calculate_security_score`. PDF zeigt „Kritische Schwächen: …
  · Vergleichsweise stabil: …" (Schwelle: bestehender `FIVE_BAND`-Cutoff bei 60, keine neue
  Schwelle erfunden), pro Modul nur wenn beide Seiten nicht leer sind. **Punkt 6**: neue
  `RoadmapItemData.leverage` ("Sehr hoch"/"Hoch"/"Niedrig", feste Regel aus `occurrence_count`
  ≥10 → Sehr hoch, sonst ≥5 Vorkommen oder Quick-Fix+hohe Priorität → Hoch, sonst Niedrig — die
  10er-Schwelle spiegelt bewusst die bestehende `FindingGroup::is_component_issue`-Schwelle) als
  neue Zeile im Maßnahmenplan-Karten-Text, Wortwahl "betroffen" statt "behoben" (konsistent mit
  Punkt 4). Alle sechs Punkte per `cargo test`/`--no-default-features`/`--all-features` +
  `cargo clippy --all-targets --all-features -D warnings` + mehreren `--debug-typ`-Smoke-Tests
  gegen angereicherte Fixtures verifiziert (1368 Tests im vollen `pdf_test`-Feature-Lauf, 0
  Fehlschläge).
- **CI seit mehreren Wochen durchgehend rot — echte Ursache gefunden und gefixt, 2026-09-06
  (Repo-Aufräum-Session):** beim Aufräumen `gh run list` geprüft (vorher nie gemacht — lokal
  wurde immer nur `cargo check --all-features` per Pre-Push-Hook verifiziert, nie der tatsächliche
  CI-Status) und festgestellt, dass praktisch jeder Push seit mindestens 2026-07-16 in CI
  fehlschlug. Ursache: `src/seo/schema.rs`s Test `astro_structured_data_exports_work_...` lädt
  seine Fixture per `include_str!("../../tests/fixtures/astro_structured_data_components.json")`
  — diese Datei lag lokal auf der Platte, war aber **nie in Git getrackt** (von `.gitignore`s
  pauschalem `*.json` erfasst, keine Whitelist-Ausnahme wie bei den anderen Fixture-Verzeichnissen).
  `cargo check` (der Pre-Push-Hook) kompiliert keine Test-Targets und hat das nie bemerkt; jeder
  `cargo test`/`cargo clippy --all-targets`-Lauf in CI brach dagegen mit einem harten
  `include_str!`-Compile-Fehler ab, bevor auch nur ein Test lief — betraf praktisch alle
  CI-Jobs (Check & Test, PDF Smoke Tests, Coverage). Gleiche Fehlerklasse wie der
  `detection_corpus_nonwcag/*/*.html`-Gitignore-Gap aus plan/11 (Symptom: lokal vorhandene,
  aber nie committete Fixture-Datei), hier aber mit echtem CI-Impact statt nur "würde beim
  nächsten Hinzufügen verlorengehen". `.gitignore` um `!tests/fixtures/astro_structured_data_components.json`
  ergänzt, Datei eingecheckt. Dabei zusätzlich `tests/wcag_fixtures/*.html` (7 Dateien, von
  `tests/coverage_matrix.rs` per Laufzeit-Verzeichnis-Scan genutzt, kein `include_str!` — daher
  kein CI-Blocker, aber derselbe Tracking-Gap) mitgefixt. Lokal mit den exakten CI-Befehlen
  verifiziert: `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo clippy --no-default-features -- -D warnings`, `cargo test --no-default-features` —
  alle grün.
- **Repo-Konsolidierung, 2026-09-06:** verwaisten Worktree + Branch `worktree-agent-*` entfernt
  (0 eigene Commits, die nicht schon in main waren — reiner Leftover eines alten
  `isolation: "worktree"`-Agent-Laufs). `archive/main-history` (lokal + origin) auf expliziten
  Nutzerwunsch gelöscht, nur noch `main` existiert. Streudateien im Projekt-Root entfernt
  (`delete.html`, ein liegengebliebener Shopify-Seiten-Dump; ein eigener Test-Rest-JSON aus einer
  vorherigen Session). `reports/` (68 MB) und `tmp/pdfs/` (lokale, gitignorte Testreport-Ausgaben)
  auf Nutzerwunsch komplett geleert — regenerierbar, kein Git-Tracking betroffen. Lokaler
  `target/`-Ordner-Rest bereinigt (`CARGO_TARGET_DIR` zeigt eigentlich nach `~/.cargo/target`,
  siehe `reference_cargo_target_dir.md`).
- **plan/11: Non-WCAG-Detection-Corpus für vulnerable_libs/schema_rules befüllt, 2026-09-06
  (Rest-Scope von #558 abgeschlossen — Security-Domain war bereits fertig):** beide Domänen
  brauchen echtes Chrome/CDP (anders als Security, das nur HTTP-Fetch ist), daher zwei neue
  `#[ignore]`-gegatete Tests analog `tests/detection_corpus_test.rs`.
  **vulnerable_libs (8/8 covered):** `tests/vulnerable_libs_detection_corpus_test.rs` diffed
  `analyze_vulnerable_libraries`'s echten Output gegen 4 kombinierte HTML-Fixtures (nicht mehr,
  da minimale globale Stubs reichen — `window.jQuery.fn.jquery` etc., keine echte
  Bibliotheksfunktionalität nötig). Auf 4 statt 2 Dateien aufgeteilt wegen eines echten,
  vom Modul selbst dokumentierten Constraints: Lodash und Underscore teilen sich das globale `_`
  und würden sich bei gemeinsamer Nutzung gegenseitig überschreiben. Prototype/MooTools sind im
  Code unconditional als vulnerable markiert (unmaintained), haben also nur einen Violation-,
  keinen Pass-Fixture. **Dabei einen echten, vorbestehenden Bug gefunden und gefixt:**
  `.gitignore`s `*.html`-Sperre hatte eine Ausnahme für `tests/fixtures/detection_corpus/*.html`
  (WCAG-Corpus), aber keine für das gleichrangige `detection_corpus_nonwcag/*/*.html` — neue
  Fixtures in dieser Domäne wären lokal vorhanden, aber nie eingecheckt worden, unbemerkt.
  **schema_rules (17/17 covered):** vor dem Bau des vollen Corpus geprüft und festgestellt, dass
  `schema_rules.rs`s reine `assess_node`-Regellogik bereits für 14 von 17 Features sehr
  ausführliche Pure-Rust-Unit-Tests hat (Grenzfälle wie EventRescheduled, Remote-Jobs,
  Merchant-vs-Editorial-Context) — dem Nutzer vorgelegt, der sich trotzdem für den vollen
  Chrome-Corpus entschieden (Tracking-Konsistenz mit der Security-/vulnerable_libs-Domäne wichtiger
  als Redundanzvermeidung). `tests/schema_rules_detection_corpus_test.rs` diffed
  `detect_structured_data`s echten Output gegen 2 kombinierte Fixtures (alle 17 Typen
  vollständig/gültig → erwartet "pass"; die 11 Typen, die überhaupt Pflichtfelder haben können,
  unvollständig → erwartet "violation"). `merchant_listing` erwartet bewusst `needs_review`, nicht
  pass/violation: `detect_structured_data` wertet Regeln immer mit
  `ProductRuleContext::Indeterminate`, der echte Seiten-Intent-Kontext wird erst später von
  `seo::module`s `derive`-Schritt über `schema_fit`/`refresh_rule_assessments` gesetzt — das ist
  reales, korrektes Verhalten dieses Einstiegspunkts, keine Testlücke. Fünf Features
  (Article/Organization/WebPage/WebSite/Person) haben in `assess_node` keine Pflichtfelder und
  können strukturell nie "violation" werden — im "complete"-Fixture als "pass" mitgetestet, kein
  eigener Violation-Fixture nötig. Beide neuen Tests per absichtlich verfälschtem
  `expected.json` verifiziert, dass sie einen echten Diff erkennen (danach zurückgesetzt).
- **plan/18: zwei neue Best-Practice-ARIA-Hygiene-Regeln, 2026-09-06 (letzter Punkt aus dem
  aktuellen Insights-Artikel-Batch):** `redundant_role.rs` (`4.1.2/redundant-role`, z. B.
  `<button role="button">`, `<a href="..." role="link">` — fixe, kontextunabhängige
  Tag-zu-implizite-Rolle-Tabelle; Tags mit vorfahrenabhängiger Rolle wie `<header>`/`<footer>`
  bewusst ausgeschlossen) und `fake_navigation_link.rs` (`4.1.2/link-as-button`, `<a href="#"
  onclick="...">`/`href="javascript:..."` als Button-Ersatz — nur inline `onclick`, dieselbe
  Grenze wie `click_handlers.rs`). Beide `Severity::Low`, `WcagLevel::A`, Tag `"best-practice"`,
  registriert im normalen `PageRuleEntry`-Mechanismus (`page_rules.rs`), keine neue
  Infrastruktur. **Zwei reale Bugs beim Live-Verifizieren gefunden und behoben, nicht nur
  Wording:** (1) erste Fassung baute Selektoren manuell aus Tag+id+class (wie `click_handlers.rs`
  es vormacht) — bei mehreren gleichnamigen Tags ohne id/class (z. B. zwei `<a>`-Elementen ohne
  Klasse) kollidierten beide auf denselben Selector-String `"a"`, nicht in der Fixture reproduziert
  zunächst unbemerkt, aber real: `js_helpers::CSS_SELECTOR_JS`'s eigener Doc-Kommentar warnt
  explizit davor ("Never falls back to a bare tag name"). Auf den robusteren, bereits
  vorhandenen `__amsCssSelector`-Helper umgestellt (wie `aria_prohibited_attr.rs`), live
  verifiziert: `body > a:nth-of-type(1)`/`:nth-of-type(3)`/`:nth-of-type(4)` bleiben jetzt
  unterscheidbar. (2) Naive String-Konkatenation von `CSS_SELECTOR_JS` (eine Top-Level
  `function`-Deklaration) direkt gefolgt vom eigenen, selbständigen `(function(){...})()` erzeugte
  einen echten Live-Fehler (`TypeError: (intermediate value)(intermediate value)... is not a
  function`) — `Runtime.evaluate` parst ein führendes `function`-Token an Ausdrucksposition als
  Funktionsausdruck statt als Deklaration, wodurch die folgende IIFE als Funktionsaufruf-Argument
  interpretiert wird statt als eigenständiges Statement. Behoben nach demselben Muster wie
  `aria_prohibited_attr.rs`: beides in ein explizites äußeres `(function() { ... })()` einbetten,
  statt zwei unabhängige Top-Level-Snippets zu verketten — beim ersten Versuch übersehen, weil
  `click_handlers.rs` (das ursprüngliche Vorbild) `CSS_SELECTOR_JS` gar nicht nutzt und dieses
  Konkatenationsproblem daher dort nie auftritt. Live gegen eine lokale `file://`-Fixture
  verifiziert (5 Testfälle: 3× redundante Rolle, 2× Fake-Navigation, 2× korrekt nicht geflaggte
  legitime Fälle) — beide Regeln feuern exakt wie erwartet, keine False Positives. Zwei
  hartcodierte Page-Rule-Zähler-Regressionstests (`level_a_filter_includes_only_level_a_rules`,
  `level_aa_filter_includes_aa_plus_a_rules`, `page_rules.rs`) entsprechend hochgezählt
  (36→38, 47→49). Kein neuer Detection-Corpus-Fixture-Eintrag (#556-Format) — passend zum
  bestehenden Präzedenzfall reiner DOM-JS-Regeln ohne Offline-Unit-Test in diesem Modul.
- **plan/17: 4.1.3-Vertiefung — Live-Region-Spät-Einfügung erkannt, 2026-09-06:** neuer
  `InteractiveFindingKind::FormErrorLiveRegionLateInsertion` (Severity::Low, Advisory) in
  `src/a11y_journey/form_error.rs` — Erweiterung der bestehenden Form-Error-Journey statt neuer
  Erkennungspfad, wie im Plan gefordert: `check_error_state` erfasste bereits `live_before`
  (vor Submit) und `live_after` (nach Submit), `new_live = live_after && !live_before` wurde
  bisher nur intern zur Ableitung von `FormErrorInvalidWithoutLiveRegion` genutzt, nie selbst als
  eigener Befund gemeldet. Neuer Check: wenn `new_live` wahr ist (Live-Region hat korrekt
  angekündigt, war aber initial nicht im DOM), wird das jetzt als eigenständiger Advisory-Befund
  gemeldet — klar getrennt vom bestehenden "gar keine Ankündigung"-Fall
  (`FormErrorInvalidWithoutLiveRegion`/Silent-Failure, die beide `!new_live` voraussetzen, also
  nie gleichzeitig mit dem neuen Befund feuern) und vom Politur-Konflikt-Check in
  `status_messages.rs` (unverändert). Bewusst Advisory/Low statt High: die Ankündigung hat in
  diesem Lauf funktioniert, es ist eine Robustheits-Empfehlung für andere Browser-/AT-Kombinationen,
  kein bestätigter Fehler. Live gegen casoon.de (Startseite + /kontakt) geprüft — dort wurde kein
  Form-Error-Journey-Kandidat erkannt (keine passende Formular-/Submit-Struktur auf diesen zwei
  Seiten), der neue Codepfad ist daher nur durch Build/Clippy/die volle Testsuite (1318 Tests) und
  den EN-Locale-Umlaut-Guard verifiziert, nicht durch einen live beobachteten Treffer — wie alle
  anderen `FormError*`-Befunde in dieser Datei hat `form_error.rs` keine eigene Unit-Test-Infra
  (Chrome-`Page`-Interaktion, kein Mocking-Präzedenzfall in diesem Modul).
- **plan/16: konkrete AT-Selbsttest-Anleitungen statt generischem "manuell prüfen", 2026-09-06:**
  neue `manual_recheck_instruction(wcag_criterion, en)` (`src/output/pdf/helpers.rs`), keyed nach
  WCAG-Kriterium, mit VoiceOver-Rotor-/NVDA-Elementliste-Tastenkürzeln für die vier Kriterien, bei
  denen ein Laie einen Verdacht in wenigen Minuten selbst nachprüfen kann: 1.3.6/1.3.1
  (Landmarken/Überschriften via Rotor VO+U bzw. Insert+F7), 3.3.2 (Formularfeld-Labels via
  VO+Cmd+J/NVDA F), 4.1.3 (Fehlermeldungs-Ansage — Meldung auslösen, prüfen ob automatisch ohne
  Fokus-Sprung angesagt). Bewusst keine neue JSON-Struktur/kein kind-Enum — reiner
  PDF-Präsentationstext (analog `render_manual_only_criteria_note`, plan/15), an drei bereits
  bestehenden Stellen angehängt statt eines neuen, vom Finding losgelösten Anhangs: der
  Screenreader-Befunde-Tabelle (`detail_modules/accessibility.rs`, keyed auf `issue.wcag_criterion`),
  der WCAG-Violation-Finding-Karte (`findings.rs`, neue "Selbsttest"-Zeile im
  Maßnahmenbewertungs-Block, keyed auf `group.wcag_criterion`) und der
  Manuelle-Prüfpunkte-Checklist (`single_report.rs`, an den `recommendation`/`text`-String
  angehängt, keyed auf `finding.rule`). `1.3.1` deckt bewusst sowohl Heading- als auch
  Landmark-Duplikat-Funde mit einem gemeinsamen Text ab (beide Sub-Fälle teilen dieselbe
  `wcag_criterion`-Konstante im Code und werden im selben VoiceOver-Rotor/NVDA-Elementliste-Tool
  geprüft, keine künstliche Trennung nötig). Live gegen casoon.de verifiziert (`--debug-typ`): die
  Landmarken-Anleitung erscheint korrekt an jedem 1.3.6-Screenreader-Befund.
- **plan/15: feste "Strukturell nur manuell prüfbare Kriterien"-Liste im PDF, 2026-09-06:** neue
  `render_manual_only_criteria_note` (`src/output/pdf/single_report.rs`), direkt nach der
  WCAG-Coverage-Sektion, unconditional (kein `--annex`-Flag, gleiches Report-Level-Gate wie die
  Coverage-Sektion selbst). Fünf feste, seiten-unabhängige Zeilen (2.4.3 Fokusreihenfolge, 3.3.1
  Fehlerkennzeichnung, 1.4.4/1.4.10 Zoom & Reflow, 1.1.1 Alt-Text-Korrektheit, 1.2.1/1.2.2
  Video-Inhaltszusammenfassung) — bewusst NICHT aus den Findings dieser Seite abgeleitet wie
  EN-301-549-/BIK-Annex, weil genau die dritte, "unsichtbare" Konfidenzkategorie aus dem
  Insights-Artikel (plan/13) sonst bei einem 0-Findings-Report komplett unsichtbar bliebe. Live
  gegen casoon.de verifiziert (`--debug-typ`): rendert korrekt, ohne Status-Punkt (kein
  good/warn/bad-Urteil, reine Scope-Aussage). Beim Umsetzen entdeckt: es gibt bereits eine
  separate, ähnlich benannte "So testen Sie manuell"-Sektion (`src/output/pdf/wcag_coverage.rs`)
  mit AT-Testanleitungen (Tastaturnavigation, Screenreader, 400%-Zoom, Reduced Motion, …) — andere
  Funktion (Anleitung "wie testen" statt Erklärung "warum strukturell nicht automatisierbar"),
  bewusst nicht zusammengelegt, aber relevanter Ankerpunkt für plan/16.
- **plan/14: "Prozess statt Zertifikat"-Satz im Disclaimer ergänzt, 2026-09-06:** Prüfung ergab,
  dass der Scope-Vorbehalt selbst ("kein vollständiger Konformitätsnachweis", "ersetzt keine
  manuelle Prüfung") bereits im bestehenden `disclaimer`-Text (`build_methodology`,
  `src/output/builder/single/methodology.rs`) und im Cover-Label ("WCAG-Vorkommen" statt
  pauschal "Accessibility-Befunde") vorhanden war — beide Fundstellen unverändert gelassen.
  Fehlend war die im Artikel betonte Prozess-Aussage: Barrierefreiheit als fortlaufende Aufgabe,
  nicht einmalig erreichbarer Zustand. Ein Satz an den bestehenden `disclaimer` angehängt (DE+EN,
  keine neue Callout-Komponente, keine Änderung an `gate_certificate_by_risk`/`CERTIFICATE`-
  Schwellen). Bewusst nicht auf dem Cover platziert — das ist eine feste, knappe Dashboard-Seite
  (renderreport `CoverPage`), Scope-/Rechtstexte leben in diesem Projekt durchgehend im
  Methodology-Kapitel, nicht auf dem Cover.
- **plan/13 (Drei-Stufen-Konfidenz-Kennzeichnung) als bereits erfüllt geschlossen, 2026-09-06:**
  der aus einem Insights-Artikel abgeleitete Plan-Punkt wollte ein neues `DetectionConfidence`-
  Enum (`Certain`/`HeuristicSuspicion`/`ManualOnly`) plus Registry-Klassifikation pro `rule_id`
  und ein neues PDF-Badge einführen. Bei der Umsetzung geprüft und verworfen: `NormalizedFinding`
  trägt bereits `confidence`/`false_positive_risk`/`verification` (heuristisch pro Regel via
  `derive_confidence`/`derive_false_positive_risk`/`derive_verification`,
  `src/audit/normalized.rs`), und jede Finding-Karte im PDF zeigt **unconditional**
  "Erkennungssicherheit"/"Falschpositiv-Risiko" im "Prüfmetadaten"-Block
  (`src/output/pdf/findings.rs`) — nicht nur für Verdachtsfälle. Ein erster Versuch, das
  bestehende (aus `false_positive_risk` 1:1 abgeleitete) `verification`-Feld zusätzlich
  symmetrisch als weitere Zeile zu zeigen, wurde wieder verworfen: das hätte dieselbe
  Information ein drittes Mal in anderen Worten auf derselben Karte gezeigt — echte Redundanz
  statt Erkenntniswert, das Gegenteil dessen, was `report-lint`/`report-critic` in diesem
  Projekt verhindern sollen. Die "nur kontextuell entscheidbare" dritte Stufe (Media-Alternative,
  Kontrast bei Bild-Hintergrund) läuft bereits sichtbar getrennt unter der PDF-Sektion
  "Manuelle Prüfpunkte und heuristische Hinweise" (`render_assessment_and_execution_notes`) mit
  eigenem "Manuelle Prüfung"-Label. Keine Code-Änderung — der Plan-Punkt war beim genauen
  Abgleich bereits durch bestehende, anders benannte Infrastruktur erfüllt. Der Plan-Text
  enthielt zudem einen inneren Widerspruch (Punkt 1 vs. Punkt 4 zur Einordnung von
  Media-Alternative als `ManualOnly` vs. `HeuristicSuspicion`), der beim Nutzer geklärt wurde
  (Punkt 1/`ManualOnly` ist korrekt).
- **`build_id` bekommt `-dirty`-Suffix bei uncommittetem Arbeitsstand (plan/21, 2026-09-05):**
  `build.rs` las den `build_id` bisher nur aus `git rev-parse --short HEAD` — solange (wie in
  diesem Projekt üblich) uncommittet gearbeitet wird, blieb der eingebettete SHA über mehrere
  tatsächlich unterschiedliche Builds hinweg identisch und machte zwei Live-Reports mit
  unterschiedlichem Fix-Stand nicht unterscheidbar (live an drei Reports vom 2026-09-05
  bestätigt, alle mit `build_id: "be9aed2"` trotz seither committeter Änderungen). Neue
  `is_dirty()`-Prüfung (`git status --porcelain`, gleicher No-Git-Fallback wie beim
  SHA-Lookup — Fehler/kein Git ⇒ nicht dirty, kein Hard-Fail) hängt bei uncommittetem Stand
  `-dirty` an, z. B. `44e7480-dirty`. Da der Dirty-Status sich bei jeder beliebigen
  Arbeitsbaum-Änderung ändern kann, nicht nur bei einem HEAD-/Ref-Wechsel, reicht das bestehende
  `rerun-if-changed=.git/HEAD` allein nicht — zusätzlicher `rerun-if-changed` auf einen
  garantiert nicht existierenden Pfad zwingt Cargo, das Build-Script bei jedem Build neu
  auszuführen (empirisch gegen ein Scratch-Projekt verifiziert: 3 Builds → 3 Skript-Läufe).
  Live gegen den eigenen, absichtlich uncommittet gehaltenen Arbeitsstand verifiziert (`strings`
  auf der gebauten Binary zeigt `44e7480-dirty`).
- **Verifiziert: 1.3.6-Fundzahl auf casoon.de kein Zählfehler (plan/22, 2026-09-05):** ein
  Live-Report hatte 200 Vorkommen für WCAG 1.3.6 auf einer einzigen Seite gezeigt, als niedrig
  priorisierte Beobachtung ohne Verdacht auf konkreten Bug angelegt. Verifiziert gegen einen
  frischen Live-Lauf (`--format json` gegen www.casoon.de): kein Doppelzählungs-Bug über
  Desktop-/Mobile-Viewport hinweg (`build_sr_audit_report` läuft nur einmal pro Report, auf einem
  einzigen AXTree) und keine echte Diskrepanz durch den `is_real_node_id`-Sanitizing-Filter
  (`analyzer.rs`s `analyze_reading_sequence` entfernt leere/synthetische IDs aus
  `affected_node_ids` **nach** der Analyse — Meldungstexte wie "18 entries" zählen bewusst die
  ungefilterte Rohmenge, während `affected_node_ids.len()` danach kleiner sein kann, z. B. 8).
  `derive_bik_chapters`s Structure-Kapitel summiert `issue.affected_node_ids.len()` über **alle**
  `1.3.6`-getaggten Funde einer Seite (identisches, bereits bestehendes Aggregationsmuster wie in
  jedem anderen Kapitel/Kriterium) — auf casoon.de sind das 14 separate, durch
  `detect_announcement_deserts` gefundene "long section ohne Landmark/Heading/Fokusziel"-Stellen,
  deren gefilterte `affected_node_ids`-Längen sich exakt zu 200 aufsummieren. Reale, verifizierte
  Struktureigenschaft der Seite (viele lange Content-Abschnitte ohne Orientierungspunkt dazwischen),
  kein Tool-Bug — keine Verhaltensänderung nötig. Klärender Kommentar in `bik_guide.rs` ergänzt,
  damit künftige Reviewer die Summenbildung nicht erneut als Zählfehler missverstehen.
- **#406-Fix: NotTestable/Warning-Findings zeigten rohen Englisch-Text im PDF (plan/20,
  2026-09-05):** live in DE-Reports bestätigter Lokalisierungs-Verstoß (casoon-de,
  satower-mosterei-de, inros-lackner-de) — `render_assessment_and_execution_notes`
  (`src/output/pdf/single_report.rs`) rief für `wcag.not_testables` nur
  `get_explanation(rule_id)` auf (axe_id-Lookup, z. B. `"video-caption"`) und fiel bei
  fehlendem Treffer auf den rohen kanonisch-englischen `finding.fix_suggestion` zurück statt
  auf den sicheren lokalisierten Fallback-Satz; `wcag.warnings` (Kontrast bei Bild-/Gradient-
  Hintergrund, 1.4.3) versuchte gar keinen `get_explanation`-Lookup. Fix: zweistufiger Lookup
  (`rule_id` zuerst, dann `rule`/WCAG-ID als Fallback — deckt sowohl axe-id-Override-Einträge
  wie `"region"` als auch die neuen WCAG-ID-Einträge ab) in beiden Schleifen, mit dem
  bestehenden lokalisierten Fallback-Satz als letztem Schritt statt `finding.fix_suggestion`.
  Vier neue `explanations.rs`-Einträge für die zuvor komplett fehlende 1.2.x-Video-Familie
  (1.2.1, 1.2.2, 1.2.3, 1.2.8). **Realer, dabei entdeckter Nebeneffekt behoben:**
  `wcag::bik_guide::derive_bik_chapters`s "Videos"-Kapitel zeigte `NoFindingsDetected`, obwohl
  ein echter, PDF-sichtbarer 1.2.2-Fund existierte — `findings[]` (`NormalizedFinding`) wird
  ausschließlich aus `wcag_results.violations` gebaut (`audit::normalized::normalize`), nie aus
  `warnings`/`not_testables`, und Video-Checks resolven fast immer zu genau diesen beiden Kinds.
  `derive_bik_chapters` nimmt jetzt zusätzlich `&[AccessibilityAssessment]`
  (`NormalizedReport.accessibility_assessments`, bereits vorhandener Träger für genau diese
  Daten, in jedem Report-Pfad verfügbar) und speist das Videos-Kapitel zusätzlich daraus;
  `normalize_assessments` dafür von privat auf `pub(crate)` angehoben. Neue Regressionstests:
  EN/DE-Fallback-Guard in `src/output/pdf/tests.rs` (fiktive Regel ohne `explanations.rs`-
  Eintrag) sowie zwei neue `bik_guide`-Tests (manual-review-only Fund erscheint im
  Videos-Kapitel; unrelated-Kriterium leakt nicht rein).
- **Neues `html_conform`-Modul: HTML5-Spezifikationskonformität via `html-conform`-Crate,
  2026-08-30:** neues, **standalone** Modul (`src/html_conform/`), das per `html-conform`
  (crates.io, pure Rust, kein Netzwerk/Subprocess) Browser-artige HTML5-Baumkonstruktion,
  volle W3C-RelaxNG-Schemavalidierung, Schematron-Co-Constraints sowie Import-Map-/
  Speculation-Rules-JSON- und CSP-Enforcement-Prüfung durchführt — deutlich tiefer als der
  bestehende, unverändert belassene `html5ever`-Parse-Fehler-Check in `seo::page_health`
  (`validate_html_locally`). **Läuft als Teil von `--full`** (`check_html_conform: full_audit`,
  kein eigener CLI-Flag), nur auf dem Mobile-Viewport-Pass (analog SEO/Mobile/DesignQuality),
  und **fließt in den Score ein** (nicht score-neutral wie `design_quality`/`ai_transparency`):
  eigenes `ModuleScoreEntry` mit 10 % Gewicht, finanziert durch eine proportionale Kürzung der
  fünf bisherigen Gewichte (Accessibility 40→36, Performance 20→18, SEO 20→18, Security 10→9,
  Mobile 10→9). Wie `security` als eigenständiges Top-Level-Feld auf `AuditReport`
  (`html_conform: Option<HtmlConformAnalysis>`, URL-/Seiten-Ebene statt Viewport-Experience) und
  außerhalb des Viewport-Blends auf `overall_score` aufgesetzt (`ScoreBreakdown.html_conform_score/
  _weight_pct`, analog `security_score`/`security_weight_pct`). `rule_id`/`message` bleiben
  opakes kanonisch-englisches Passthrough (`html-conform`s Regelmenge ist offen, nicht ein
  kleines Enum, und die Messages sind Drittanbieter-Fließtext) — bewusst **kein** #406-
  kind-Enum-Muster, gleiche Präzedenz wie `best_practices::console_errors`/`vulnerable_libs`.
  Auf einem technischen Setup-Fehler oder HTML-Extraktions-Fehler: `checked: false` / `score: 100`
  ("nicht gemessen", ausgeschlossen vom gewichteten Overall-Score) statt eines punitiven 0,
  analog Performance's `metrics_available == 0`-Behandlung. Volle JSON/PDF-Anbindung
  (`ModuleBlob.html_conform`, `HtmlConformPresentation`, eigener PDF-Renderer mit ScoreCard/
  MetricStrip/AuditTable). Batch-PDF-Narrativ-Arme (`batch_report/sections.rs`) und
  URL-Matrix-Spalte bewusst **nicht** Teil dieser Änderung (Score zählt bereits automatisch in
  jedes Batch-Aggregat ein) — gleiche Präzedenz wie der akzeptierte `commerce`-hat-kein-PDF-Gap.
- **Neues `ai_transparency`-Modul: C2PA-Bildherkunfts-Check (EU AI Act Art. 50), 2026-07-31:**
  neues, **opt-in** (`--ai-transparency`) und **score-neutrales** Modul, das eingebettete C2PA-
  ("Content Credentials"-)Manifeste auf `<img>`-Elementen der geprüften Seite liest und —
  falls das Manifest eine KI-/algorithmische Erzeugung ausweist (IPTC `digitalSourceType`
  `trainedAlgorithmicMedia`/`compositeWithTrainedAlgorithmicMedia`/`compositeSynthetic`/
  `virtualRecording`/`trainedAlgorithmicData`) — einen Manual-Review-Advisory-Fund erzeugt.
  Bewusst **nur Presence melden**, kein Versuch, eine sichtbare Kennzeichnung in Bild-Nähe
  automatisch zu erkennen (gleiches Fragilitäts-Problem wie Chat-Widget-Disclosure-Detection,
  daher aus dem Scope genommen), und bewusst **kein EXIF-Software-Tag-Fallback** (C2PA ist der
  vom AI Act referenzierte Standard-Mechanismus, EXIF wäre eine zweite, unsichere
  Erkennungslogik). **Nur Single-URL-Modus** (`PipelineConfig.check_ai_transparency =
  args.ai_transparency && args.url.is_some()`, exaktes Muster wie `capture_element_evidence`):
  ein Batch-Lauf würde pro Bild einen Netzwerk-Fetch + C2PA-Parse machen, nur damit
  `build_batch_detail()` den gesamten Modul-Blob pro Seite ohnehin verwirft (#256).
  **Doppeltes Opt-in:** neues Cargo-Feature `ai-transparency` (`c2pa = "0.90.3"`,
  `default-features = false, features = ["rust_native_crypto"]` — kein natives OpenSSL, keine
  HTTP-Client-Features, damit der Reader strukturell nie selbst nachlädt/telefoniert), nicht Teil
  von `default` (Precedent: `pdf`-Feature) — **plus** der Laufzeit-Flag. Fehlt das Feature beim
  Build, aber ist der Flag gesetzt, bricht `src/cli/runners.rs`s `check_ai_transparency_feature`
  vor Pipeline-Start laut mit `ConfigError` ab (Rebuild-Hinweis) statt still No-op zu bleiben.
  **SSRF-Härtung** (erster Fetch dieses Codebase auf fremde, potenziell durch die geprüfte Seite
  kontrollierte Origins — anders als `seo::robots`/`security` mit reinem Same-Origin-Fetch):
  eigene DNS-Auflösung + IP-Range-Check (privat/loopback/link-local/multicast/IPv4-mapped-IPv6,
  explizite Range-Checks statt neuerer, ggf. noch nicht überall stabiler `Ipv6Addr`-Methoden) VOR
  jedem Fetch, plus `reqwest::ClientBuilder::resolve()` pinnt die Verbindung auf exakt die
  geprüfte IP (verhindert einen zweiten, nicht erneut geprüften DNS-Lookup beim eigentlichen
  Connect — DNS-Rebinding-TOCTOU). Manifest-Auswertung nutzt die **typisierte** `c2pa`-API
  (`Manifest::find_assertion::<Actions>(labels::ACTIONS)` → `Action::source_type()`/
  `.software_agent()`, `Reader::validation_state()`), kein Blind-JSON-String-Walk — robuster
  gegen Schema-Drift und Fließtext-False-Positives. Lokalisierung folgt dem etablierten
  kind-Enum-Muster (#406): `finding_message_text(rule_id, en)` einzige Textquelle. Neue lokale
  Enums `AiProvenanceKind`/`ManifestValidation` spiegeln `c2pa::DigitalSourceType`/
  `ValidationState`, damit `src/ai_transparency/mod.rs` (Typen, JSON/PDF-Anbindung) **ohne** das
  Cargo-Feature kompiliert — nur `image_provenance.rs` (der eigentliche Fetch+Parse) ist
  `#[cfg(feature = "ai-transparency")]`-gated; Modul-Registrierung in `AuditCatalog`, `ModuleData`-
  Variante, `ExperienceSection.ai_transparency`, `ModuleBlob`/PDF-Dispatch bleiben unconditional
  (kleinerer Diff als ursprünglich geplant, keine `#[cfg]`-Streuung über Catalog/Report/Output).
  Test-Fixtures für den C2PA-Parser sind **synthetisch selbst erzeugt** zur Testzeit
  (`c2pa::Builder` + `EphemeralSigner`, in ein hartcodiertes 1×1-PNG signiert) statt aus den
  öffentlichen C2PA-Testdateien (`c2pa-org/public-testfiles`, CC-BY-SA-4.0) vendored — die
  enthalten keine eindeutig als KI-generiert gekennzeichneten Samples (nur Adobe-Editing-
  Konformitätstests von 2022), und eine synthetische Fixture vermeidet jede Lizenz-/
  Attributions-Frage. **Realer, dabei entdeckter Nebeneffekt:** `c2pa` aktiviert unconditional
  (fest in dessen eigener `Cargo.toml`, nicht hinter einem seiner optionalen Features)
  `serde_json`s `preserve_order`-Feature — durch Cargo-Feature-Unification ändert das
  `serde_json::Value::Object`s Backing-Map **projektweit** von sortiertem `BTreeMap` auf
  insertion-order `IndexMap`, sobald `ai-transparency` mitgebaut wird, unabhängig davon ob der
  Laufzeit-Flag je gesetzt wird. Betraf 4 bestehende Snapshot-Tests (`src/output/sr_audit_json.rs`,
  3× `tests/snapshot_tests.rs`), die stillschweigend alphabetische Schlüssel-Reihenfolge
  voraussetzten — behoben mit einer kleinen `sort_json_keys`-Normalisierungsfunktion vor dem
  jeweiligen `assert_json_snapshot!`, nicht durch Snapshot-Neuaufnahme (die Reihenfolge muss
  unabhängig vom Feature-Set stabil bleiben). Vollständig grün verifiziert:
  `cargo test --all-features --lib` (1157 passed), `cargo clippy --all-features -- -D warnings`
  auf lib+bins+den beiden betroffenen Test-Targets (ein separates, bereits vor dieser Session
  bestehendes Dead-Code-Problem in `tests/wcag_rule_id_inventory.rs`/`tests/common/` — unfertige,
  uncommittete Arbeit an anderer Stelle — via `git stash` als vorbestehend verifiziert, nicht
  Teil dieser Änderung). Batch-Report-Rollup (analog `template_clusters`) bewusst **nicht** Teil
  dieser Änderung — separates Folge-Issue.
- **Kanonische WCAG-Regel-ID-Inventur, 2026-07-31 (#552, tracking #559):** schließt eine
  vorher falsche Annahme — `src/taxonomy/rules.rs`s `RULES` (117 Einträge, 85
  `Dimension::Accessibility`) wurde fälschlich als vollständige Liste aller Regeln behandelt.
  Tatsächlich ist `RULES` eine kuratierte Scoring-/Report-Klassifikationstabelle, kein
  Laufzeit-Register; `PAGE_RULES`s `rule_id` (`page_rules.rs`) ist laut eigenem Doc-Kommentar
  "used for logging only", `run_if_allowed!`s axe_id (`engine.rs`) ist nur `RuleOutcome`-
  Telemetrie. Der tatsächliche Mechanismus hinter `Violation.rule_id`: eine `RuleMetadata`-
  Konstante pro Regeldatei. Neuer, browser-freier Test `tests/wcag_rule_id_inventory.rs`
  scannt `src/wcag/rules/*.rs` (RuleMetadata-Literale + die bare `*_AXE_ID`-Konstanten in
  `aria_roles.rs`/`widget_rules.rs`) und `src/patterns/*.rs` (literale `.with_rule_id("...")`-
  Aufrufe in den Pattern-Detection-Modulen Accordion/Modal/TabList/DisclosureMenu, die ganz
  ohne `RuleMetadata` auskommen) — **reale, verifizierte Zahl: 116 distinkte `rule_id`s**,
  nicht 117/85. Empirisch gegen einen vollen `--full --level aaa`-Lauf (casoon.de) abgeglichen;
  zwei reale, vorher unbemerkte Diskrepanzen dabei gefunden: (1) `NormalizedFinding.rule_id`
  führt einen zweiten, parallelen ID-Namensraum (`a11y.*`, aus `taxonomy::rules::RULES`) neben
  dem hier inventarisierten axe_id-Namensraum — beide sind für dieselbe Regel im JSON
  gleichzeitig sichtbar, je nach Report-Stelle; kein Bug, aber wichtig für zukünftige
  Regel-Vergleiche (welchen Namensraum meint man?). (2) Reflow trägt zwei abweichende IDs für
  dieselbe Prüfung — `RuleOutcome.rule_id = "reflow"` (Telemetrie-Label in `pipeline.rs`) vs.
  `Violation.rule_id = "css-overflow-hidden"` (`REFLOW_RULE.axe_id`, tatsächliches Finding).
  Dabei zusätzlich entdeckt und als eigenes Bug-Issue #560 angelegt (bewusst nicht in #552
  mitgefixt, unabhängiges Problem): Kontrast (`1.4.3`) und Reflow (`1.4.10`) laufen beide
  direkt inline in `pipeline.rs`s `run_rules`, ohne je `RuleFilterConfig.should_run(...)` zu
  prüfen — `[rules] disabled`/`enabled_only` in `auditmysite.toml` (dokumentiertes Feature,
  `src/cli/config.rs:75`) hat für diese zwei Regeln aktuell keine Wirkung. Diese Inventur ist
  die Grundlage für den geplanten Ground-Truth-Detection-Corpus (#553–#558, siehe
  `plans/contrast-a11y-detection-testbed.md`), nicht `RULES`.
- **Detection-Corpus-Format + Completeness-Check, 2026-07-31 (#553, #554, tracking #559):**
  baut auf der Regel-Inventur (#552) auf. Neues, browser-freies `tests/common/` (geteilter
  Test-Support, nicht selbst ein Testbinary — Standard-Rust-Idiom): `rule_inventory.rs` (die
  #552-Scan-Logik, jetzt wiederverwendbar) und `detection_corpus.rs` (`ExpectedCase`/
  `Expectation`/`Verdict`-Typen + Loader für `tests/fixtures/detection_corpus/<case>.expected.json`
  und `structurally_deferred.json` — beide Loader liefern eine leere Liste statt zu fehlern, wenn
  Verzeichnis/Datei noch nicht existiert, da der Corpus bewusst leer startet und erst #556 ihn
  befüllt). `tests/detection_corpus_format_test.rs` (7 Tests, Parsing/Loader gegen ein Tempdir,
  kein echtes Fixture nötig — vermeidet unverifizierte Ground-Truth-Werte vor #556).
  `tests/detection_corpus_completeness.rs` diffed die 116 Regel-IDs gegen die im Corpus
  referenzierten IDs + `structurally_deferred.json`, meldet covered/deferred/gap **ohne bei
  niedriger Abdeckung zu scheitern** (Startzustand aktuell: 0/0/116 — erwarteter Rückstand,
  kein Regressions-Fehlschlag), schlägt aber hart fehl bei einem echten Autoren-Fehler
  (referenzierte `rule_id` existiert gar nicht in der Inventur, oder eine Regel ist gleichzeitig
  abgedeckt und als `structurally_deferred` markiert).
- **Runemark-Terminalpräsentation + Batch-Progress-Adapter, 2026-07-30 (#529, #530):**
  #529 ersetzt den bisherigen `colored`/`comfy-table`-Renderer (`src/output/cli.rs`, gelöscht)
  für `--format table` durch eine dünne Präsentationsschicht auf `runemark` (eigenes,
  auf crates.io veröffentlichtes Schwester-Crate wie `renderreport`; **Version live gegen
  crates.io geprüft**: 0.2.0 ist die aktuell veröffentlichte Version, nicht geraten). Neues
  `src/output/terminal.rs` mappt die bereits existierende, von JSON/PDF geteilte
  Präsentationsschicht (`ReportViewModel`/`BatchPresentation` — bewusst wiederverwendet statt
  neu gebaut, beide sind **nicht** hinter `feature = "pdf"` gegated, also für `--format table`
  in jeder Feature-Kombination verfügbar) auf `runemark::report::Report` (`Console`,
  `Verdict`, `Metric`, `FindingGroup`, `NextStep`, `RenderOptions`). Neuer `--color
  {auto,always,never}`-Flag (`ColorPolicy`); Terminalbreite via `console::Term::stdout()`
  (bereits transitive Dependency von `dialoguer`, jetzt direkt). `console_for()` erzwingt
  `Never`, wenn das Ergebnis in eine Datei geschrieben wird (`--output`), außer bei
  explizitem `--color always` — verhindert ANSI-Bytes in gespeicherten Report-Dateien.
  `comfy-table` komplett entfernt (0 verbleibende Referenzen); `colored` bleibt, da an vielen
  anderen CLI-Stellen (Banner, doctor, sitemap-suggest) weiter genutzt — bewusst nicht im
  Scope dieses Issues, das Issue verlangt nur "remove only once nothing requires it".
  #530 ersetzt die direkte `indicatif::ProgressBar`-Nutzung in `run_batch_mode`
  (`src/cli/runners.rs`) durch `BatchLifecyclePresenter` (neu, `src/cli/batch_lifecycle.rs`),
  einen dünnen Adapter über `runemark::ProgressSink` (`TerminalProgress`/`SilentProgress`,
  `progress`-Feature von runemark, zieht `indicatif` jetzt transitiv statt direkt — eigene
  `indicatif`-Dependency entfernt). Neuer `--progress {auto,always,never}`-Flag
  (`ProgressPolicy`), unabhängig von `--quiet` (das weiterhin vollständig unterdrückt).
  `run_concurrent_batch`s Signatur/Concurrency-Algorithmus **unverändert** — nur der
  Callback-Body in `runners.rs` ruft jetzt `presenter.advance/notice_error` statt direkt
  die Progress-Bar zu berühren. **Realer, dabei entdeckter Bug behoben**: Banner-/Plan-/
  Diagnose-/Ergebnis-/Verdict-Zeilen in `src/cli/plan.rs` und `run_batch_mode` waren
  durchgehend `println!` (stdout), nur von `--quiet` gegated, nicht vom Output-Format — ein
  Batch-Lauf mit `--format json` ohne `--output`/`--quiet` mischte Klartext in den
  JSON-Payload auf stdout. Mechanischer Fix: alle diese Zeilen jetzt `eprintln!` (stderr)
  bzw. laufen für die eigentliche Batch-Lifecycle (Sitemap-/Crawl-Diagnose-Einzeiler,
  Results-Zeile, finaler Verdict) durch `presenter.notice()`/`presenter.finish_verdict()`.
  `print_verdict()` (auch von Single-Mode genutzt) bleibt als eigenständige Funktion
  bestehen, aber jetzt `eprintln!`-basiert — behebt dieselbe Bug-Klasse auch für
  Single-Mode-JSON-Läufe. **Zweiter, von der ersten Live-Verifikation aufgedeckter Leck
  gefunden und gefixt:** der globale `tracing_subscriber::fmt()` in `src/main.rs` hatte
  keinen expliziten `.with_writer(...)` und schrieb dadurch (tracing-subscribers eigener
  Default) selbst auf **stdout** — ein `tracing::warn!` während eines Batch-Laufs (z. B. ein
  Page-Timeout in `src/audit/batch.rs:164`) landete dadurch weiterhin, ANSI-gefärbt, vor dem
  JSON-Payload auf stdout und brach `jq`. Jetzt `.with_writer(std::io::stderr)` +
  `.with_ansi(...)`, Letzteres an dieselbe `--color`-Policy gekoppelt (nicht an
  tracing-subscribers eigene, unabhängige Auto-Erkennung). Nach diesem Fix erneut live
  verifiziert (Timeout-Fall gezielt reproduziert): stdout bei `--format json` bleibt bei
  einem Batch-Lauf ohne `--quiet` jetzt auch bei einem Page-Timeout sauberes, von `jq`
  valide geparstes JSON; `--progress never --quiet` erzeugt exakt 0 Byte auf stderr;
  `--color always` erzwingt ANSI auf stderr; `report-lint` auf dem Ergebnis-JSON liefert
  keine Findings.
- **Contrast pixel-sampling coverage + neues `design_quality`-Modul, 2026-07-30 (#527, #528):**
  #527 behebt eine reale Lücke bei WCAG 1.4.3: `build_sample_tasks` (`src/wcag/rules/contrast.rs`)
  sampelte bisher nur, wenn der Hintergrund unsicher WAR **und** das CSS-Ratio bereits durchfiel —
  ein Element mit optisch plausiblen CSS-Farben, dessen gerendertes Ergebnis durch Opacity-Stack,
  Blend-Mode oder ein echtes `<img>` hinter dem Text reduziert wird, wurde nie gesampelt und blieb
  komplett unsichtbar (live an casoon.de bestätigt: `finding_count: 0` trotz sichtbar fehlschlagendem
  Hero-Bereich). Fix: die Sampling-Bedingung sampelt jetzt bei jedem unsicheren Hintergrund
  unabhängig vom CSS-Ratio (mit `MAX_SAMPLE_TASKS = 60`-Deckel, größte Fläche zuerst);
  `styles.rs`s Ancestor-Walk erkennt „unsicher" jetzt zusätzlich bei `opacity < 1`,
  `mix-blend-mode`/`background-blend-mode` und einem überlappenden `<img>`/positionierten
  Overlay-Geschwister (begrenzt auf die ersten 3 Ancestor-Ebenen). Ein dabei gefundener echter
  Duplicate-Finding-Bug wurde mitgefixt: dieselbe Selector/Regel-Kombination konnte durch
  unterschiedliche Sampling-Ergebnisse zwischen Desktop- und Mobile-Pass gleichzeitig als
  bestätigter Violation UND als NeedsReview-Warning auftauchen — `merge_wcag_violations`
  (`src/audit/pipeline.rs`) unterdrückt jetzt Warnings für jede Selector/Regel-Kombination, die
  bereits als Violation vorliegt. Neue Fixture `tests/fixtures/opacity_overlay_contrast.html` +
  `test_opacity_overlay_contrast_pixel_sampling`.
  #528 fügt `design_quality` als neues, **opt-in** (`--design-quality`, bewusst noch nicht Teil von
  `--full`) und **score-neutrales** Modul hinzu: heuristische UX-/Lesbarkeits-Hinweise (Overflow-Clip
  bei positionierten/interaktiven Elementen, Zeilenlänge, Zeilenabstand, langer Großbuchstaben-Text,
  layoutwirksame CSS-Transitions) aus der bereits offenen CDP-Seite, ein einziger `page.evaluate()`-
  Durchlauf für alle Textregeln. Eigener `DesignQualityFinding`-Typ (nicht `wcag::types::Violation`)
  garantiert Score-Neutralität durch Konstruktion — es gibt nie einen `push_indicator`-Aufruf/
  `ModuleScoreEntry`, daher berührt kein Codepfad `AccessibilityScorer`/den gewichteten
  Overall-Score; per Integrationstest verifiziert (`report.accessibility.score/grade/certificate`
  byte-identisch mit und ohne Modul). Layout-Transition-Regel erkennt Transitions nicht selbst neu,
  sondern projiziert `performance::animations`-Funde (auf layoutwirksame Properties gefiltert) als
  Advisories — vermeidet einen zweiten unabhängigen Erkennungspfad und damit Duplicate Findings by
  construction. Lokalisierung folgt dem etablierten kind-Enum-Muster (#406):
  `finding_message_text(rule_id, level, en)` ist die einzige Textquelle, Analyse ruft mit `en=true`
  (JSON kanonisch Englisch), PDF-Builder mit der Lauf-Sprache. Neues `ExperienceSection.design_quality`-
  Feld, volle JSON/PDF-Anbindung (`ReportModule`, `ModuleBlob`, `ModuleDetailsBlock`,
  eigener PDF-Renderer ohne ScoreCard). Cache-Signatur auf `fmt=12` gebumpt.
- **Report Quality Layer v1.2 — Phase 5: visuelle Prüfpipeline, 2026-07-16 (#510, tracking #512):**
  schließt die zuvor als offen dokumentierte Lücke (siehe vorheriger Eintrag unten, "partial").
  Statt einer gespeicherten Pixel-Diff-Baseline (Font-Rendering/Anti-Aliasing würde das über
  verschiedene Maschinen/CI-Runner hinweg flaky machen) zwei neue, deterministische
  Same-Run-Prüfungen in `src/output/pdf/tests.rs`: (1)
  `test_dual_viewport_performance_renders_two_gauges_not_a_flat_strip` — Struktur-Regressionswächter
  für den in #510 konkret genannten historischen Fall (die "umgebrochene Dual-Viewport-Zelle"): prüft
  im Typst-Quelltext (nicht Pixeln), dass der Desktop/Mobile-Performance-Vergleich weiterhin als zwei
  eigenständige `Gauge`-Komponenten in einem 2-Spalten-`Grid` rendert (`type: "gauge"` ≥ 2×, eigene
  `label: "Desktop"`/`label: "Mobile"`) statt in die vormalige flache "Desktop 85 · Mobile 67"-Textzeile
  zurückzufallen — mit einem neuen `dual_viewport_performance(desktop_score, mobile_score)`-
  Test-Helper, der `report.performance` (mobil) + `report.dual_viewport.desktop.performance` (Desktop)
  konstruiert, dem einzigen Weg, `PerformancePresentation.desktop`/`.mobile` beide zu befüllen. (2)
  `test_representative_fixture_pages_are_not_blank_and_stay_within_page_budget` — rastert eine
  angereicherte Fixture (WCAG-Funde, Dual-Viewport-Gauges, Tech-Stack-Tabellen, gedrosselte
  Netzwerk-Tabelle; deckt damit Cover/Scorekarten/Tabellen/Findings/Methodik/technische Kennzahlen
  aus #510s Abnahmekriterium ab) auf Technical-Level und prüft Nicht-Leere UND ein Seiten-Budget
  (60 Seiten, erkennt einen Reflow-Ausreißer, ohne subjektive Layout-Qualität pixelbasiert zu
  beurteilen — das bleibt bewusst Aufgabe des `report-critic`-Skills, #509). Bei einem Fehlschlag
  werden alle rasterisierten Seiten nach `target/pdf-visual-debug/<grund>/` kopiert (Tempdir-Inhalte
  wären sonst vor jeder manuellen Inspektion gelöscht). **Realer, zuvor unbemerkter Gap gefunden und
  gefixt:** der `pdf-smoke`-CI-Job installierte `poppler-utils` nie — jeder `pdftoppm`/`pdftotext`-
  gated Test (alle Blank-Page-/Visual-Tests aus Phase 5 sowie die älteren `pdftotext`-basierten
  Content-Traceability-Tests) übersprang sich in CI seit ihrer Einführung selbst still
  (`find_executable("pdftoppm")` → `None`) statt zu laufen — nie als rotes CI sichtbar, weil ein
  übersprungener Test nicht fehlschlägt. `.github/workflows/ci.yml`'s `pdf-smoke`-Job installiert
  `poppler-utils` jetzt per `apt-get` und lädt `target/pdf-visual-debug/` bei einem Fehlschlag als
  Build-Artefakt hoch (`actions/upload-artifact@v4`, `if: failure()`) — schließt #510s
  Abnahmekriterium "erzeugte Artefakte sind in CI nachvollziehbar".
- **Report Quality Layer v1.2 — Phase 3: Feedback-Korpus, 2026-07-16 (#511, tracking #512):**
  neues `tests/regression_corpus/*.json` (16 Einträge, ein File pro bestätigtem Fall) — Format
  `{id, category: Invariant|Semantic|Completeness|Explanation|Visualization, problem, evidence,
  expected, regression, counter_examples[], status: resolved|known_gap}`. Deckt das Startkorpus aus
  Issue #511 ab (Score-Mismatch, kritisch-als-Label-für-kritisch+hoch, Score ohne Skala, Zähler
  ohne Scope, vorhandene Daten ohne Reportnutzung, Metrikstreifen-Umbruch, Batch-Klassifikation aus
  falscher Score-Basis) plus die in dieser Session bestätigten konkreten Fälle: die vier
  Lokalisierungs-Leaks (SEO-Details, Tech-Stack-Severity, TechCategory-Debug-Format,
  renderreport-Komponenten-Default-Labels) und den `violated_rule_count`-Scope-False-Positive aus
  Gruppe C — alle `status: resolved` mit Verweis auf ihren jeweiligen Regressionstest. Vier weitere,
  vom `report-critic`-Skill-Dry-Run gegen echte Batch-/Single-Reports gefundene, in dieser Session
  aber **nicht** behobene Fälle (`score_area_for_finding`-Substring-Fehlklassifikation,
  `is_generic()`-Linktext-Substring-False-Positive, Impact-vs.-Reach-Widerspruch in
  Batch-`top_actions`, `{:?}`-Debug-Leak + Truncating-Division in `management_risks[].rationale`)
  sind `status: known_gap` — bewusst dokumentiert statt stillschweigend fallengelassen. Neuer
  `tests/regression_corpus_contract.rs` validiert nur die Korpus-Form selbst (Pflichtfelder,
  Enum-Werte, eindeutige/dateiname-passende `id`, nicht-leere `counter_examples`, `resolved` ⇒
  nicht-null `regression`) — führt die referenzierten Regressionstests nicht erneut aus.
  `.gitignore`s pauschales `*.json` bekam ein `!tests/regression_corpus/*.json`-Whitelist-Eintrag
  (gleiches Muster wie `tests/lint_fixtures/`).
- **report-lint False-Positive-Fixes aus report-critic-Eval, 2026-07-16 (#507-Nachbesserung):**
  der Eval-Lauf fand zwei echte False Positives in `src/lint/checks.rs`, an einem realen
  Batch-Report bestätigt (`reports/casoon-batch-en301549.json` — vorher 2 Findings, jetzt 0):
  (1) `check_grade_and_certificate` kannte `gate_certificate_by_risk`
  (`src/audit/normalized.rs:515`) nicht — ein wegen Risk=High/Critical/legal_flags/
  blocking_issues absichtlich herabgestuftes Zertifikat ("EINGESCHRÄNKT"/"NICHT BESTANDEN")
  wurde fälschlich als Score-Mismatch gemeldet. Jetzt akzeptiert der Check jeden Wert, den
  `gate_certificate_by_risk` für die gegebenen Risk-Eingaben legitim produzieren könnte
  (`risk_gate_inputs`/`acceptable_certificates`, neue Helper). (2) `violated_rule_count` (global
  eindeutige Regeln) vs. `severity_counts.total` (Summe der pro-Seite-eindeutigen Regel-Zeilen)
  sind auf Batch-`summary`-Ebene strukturell verschiedene Aggregationen, die nur zufällig für
  Single-Reports übereinstimmen — der Check verglich sie fälschlich auf exakte Gleichheit. Neue
  `ScopeRelation`-Unterscheidung: `Exact` für Single-Reports und jede einzelne Seite, `AtMost`
  (global ≤ Summe) nur für `summary` eines Batch-Reports. 5 neue Regressionstests in
  `src/lint/checks.rs`.
- **Lokalisierungs-Nachbesserung aus report-critic-Eval, 2026-07-16:** der report-critic-Eval-Lauf
  (siehe Skill-Eintrag unten) fand reale Lokalisierungslecks — deutsche Wörter ohne Umlaut/ß, daher
  vom bestehenden Guard-Test unentdeckt. Gefixt: `build_seo_details`
  (`src/output/builder/single/module_details.rs`) komplett durchlokalisiert (meta_tags,
  identity_facts, page_profile_facts, heading/social/technical/tracking_summary, SchemaExtracted-
  Textbausteine, `signal_rows`-Rating — ~50 Stellen); `Severity::label()` (immer Deutsch) an zwei
  weiteren PDF-Stellen (`detail_modules/indicators.rs`, `detail_modules/overview.rs` — dort war ein
  drittes vermeintliches Vorkommen tatsächlich `BudgetSeverity` mit eigenen, bereits
  sprachneutralen "Error"/"Warning"-Labels, kein Bug); `single_report.rs` ("Absprungrate" im
  EN-Zweig selbst falsch übersetzt, "Vorkommen" hartcodiert im Format-String). Nebenbefund: `{:?}`-
  Debug-Leck bei `tech.category` (`TechCategory` hatte keine `label()`-Methode) — jetzt behoben.
  Neuer Regressionstest `test_seo_details_english_locale_has_no_known_german_leaks`
  (`src/output/pdf/tests.rs`) — die erste EN-Locale-Prüfung, die `build_seo_details` überhaupt mit
  echten SEO-Daten ausführt (`pdf_fixture_report()` allein trägt kein SEO). **Wichtige
  Methodik-Korrektur dabei entdeckt:** `--debug-typ`-Dumps enthalten die komplette
  renderreport-Komponentenbibliothek (`include_str!`), nicht nur den tatsächlich gerenderten
  Content — ein String-Vorkommen im Dump beweist nicht, dass er auf einer Seite erscheint (kann
  aus einer nie instanziierten Komponentendefinition stammen). Führte zu zwei renderreport-Fixes
  (**v0.2.36**, siehe unten) und einer Ergänzung im `report-critic`-Skill.
- **renderreport v0.2.36:** `dominant-issue-spotlight` und `severity-overview` (zwei von
  auditmysite aktuell ungenutzte Komponenten) hatten deutsche Labels fest im Typst-Template
  (`label-text("Empfehlung")` etc.) statt sie wie alle anderen Komponenten als Datenfeld vom
  Rust-Aufrufer zu bekommen — kein Live-Bug (unbenutzt), aber Inkonsistenz mit dem etablierten
  Muster (Rust übergibt immer schon lokalisierte Strings, Templates selbst haben keine
  Sprachlogik). Beide Templates nehmen jetzt optionale `label_*`-Datenfelder mit deutschen
  Defaults (`data.at(key, default: "...")`) an; `DominantIssueSpotlight`/`SeverityOverview` in
  renderreport haben neue `with_labels(...)`-Builder analog zu `CoverPage`. Rein additiv, keine
  Verhaltensänderung für bestehende Aufrufer. `cover_page.typ`'s deutscher Fallback-Wert für
  `modules_label` bewusst unverändert gelassen — auditmysite setzt dieses Feld immer explizit für
  beide Sprachen, der Fallback ist unerreichbar.
- **Neuer Skill `report-critic` (.claude/skills/report-critic/SKILL.md, #509):** evidenzgebundene
  KI-Kritik eines fertigen Reports (JSON + `--debug-typ`-Text) gegen Widersprüche, fehlenden
  Scope, unbelegte Schlussfolgerungen, fehlende Maßnahmen-Verknüpfung, textsichtbare
  Layout-Artefakte — ergänzt (ersetzt nicht) `report-lint`. Per skill-creator-Eval-Loop getestet
  (3 Testfälle, mit/ohne Skill, echter 1.1 MB Batch-Report von casoon.de): beide Konfigurationen
  fanden durchgehend reale Bugs (Substring-Klassifikations-Bug in zwei unabhängigen Modulen,
  `Some(84)`-Debug-Leck, Impact-vs-Reach-Widerspruch in einer Aggregations-Tabelle, u.a.) — der
  Skill bringt vor allem konsistente Ausgabestruktur, nicht zusätzliche Fähigkeit gegenüber einem
  bereits sehr kompetenten Baseline-Agenten. Eval-Workspace unter
  `.claude/skills/report-critic-workspace/` (nicht committet, lokal).
- **Report Quality Layer v1.2 — Phase 5 (partial): visual PDF smoke checks, 2026-07-16 (#510,
  tracking #512):** correction to the original Phase 5 plan — rasterization via `pdftoppm` was
  **not** greenfield; `src/output/pdf/tests.rs` already had two `pdftoppm`-gated smoke tests
  (`find_executable("pdftoppm")` skips gracefully when unavailable, matching CI's `pdf-smoke` job
  which runs `cargo test --features pdf_test`). Extended that existing infra rather than building a
  new `xtask`: a `png_luma_std_dev` helper (grayscale pixel std-dev via the already-present `image`
  crate) plus `rasterize_pages`, backing two new tests —
  `test_single_pdf_technical_pages_are_not_blank_when_pdftoppm_is_available` and
  `test_batch_pdf_pages_are_not_blank_when_pdftoppm_is_available` (batch PDF rasterization had no
  test at all before this) — flagging a near-solid-color page (missing font/asset → blank render)
  without exact pixel-diff, which is deliberately avoided: font hinting/anti-aliasing differs across
  machines/CI runners, so an exact-match baseline would be flaky by construction. **Not yet done,
  open design question**: the plan's "visual diffs for stable layout regions" (e.g. cover skeleton)
  needs a real decision on baseline storage/regeneration workflow and tolerance before implementing
  — deferred rather than guessed at. Overflow/cut-content/misalignment detection and the semantic/
  visual AI judgment pass both still route to #509 as originally planned.
  new `tests/coverage_matrix.rs` — an `#[ignore]`-gated `export_coverage_matrix` test (same pattern
  as `output::builder::tests::export_all_interpretations`) walks every `AuditCatalog::standard()`
  module and counts literal id() occurrences across four surfaces (`src/output/pdf/**`,
  `docs/OUTPUT_CONTRACT.md`, both JSON schemas, `tests/**` + fixture dirs), writing
  `reports/coverage_matrix.json` for human review. Deliberately **not a CI gate** — a substring
  count can false-negative on indirection and false-positive on a common word (confirmed: the tool's
  own doc-comment mentioning "commerce" counts as one of commerce's "fixture references"). Only one
  hard, always-true assertion ships (`every_catalog_module_has_a_unique_nonempty_id` — a tripwire on
  the catalog itself, not on coverage outcomes) rather than "every module has PDF coverage", which
  would immediately fail today. **Real gap the first run surfaced**: `commerce` has 0 PDF references
  and 0 references anywhere under `tests/` outside its own module directory — the module has JSON
  output (`src/output/module.rs`, `src/output/json.rs`) but no PDF rendering and no cross-cutting
  test coverage, despite CLAUDE.md recording it as "COMPLETE, alle Slices 1-4 gemergt". Not fixed as
  part of #508 (out of scope — #508 is the detection tool, not a mandate to close every gap it
  finds); flagged here for a deliberate decision on whether that's an intentional Studio/JSON-only
  scope or a real oversight. `#509`/`#510`/`#511` (AI critic, visual PDF pipeline, feedback corpus)
  are planned but not started.
- **Report Quality Layer v1.2 — Phase 2: deterministic report-lint, 2026-07-16 (#507, tracking #512):**
  new `src/lint/` — `lint(report: &serde_json::Value) -> LintReport` runs four registry-driven
  (#506) checks with zero network/Chrome dependency, each producing a `LintFinding{check_id,
  evidence_path, expected, actual, severity}`: (1) `summary.score`/`summary.overall_score` alias
  consistency plus single-report summary-vs-page cross-check (the "18/100 vs 20/100" corpus
  shape); (2) `grade`/`certificate` re-derivable from `overall_score` via the same shared
  `LETTER_GRADE`/`CERTIFICATE` `BandSet`s the production scorer uses, checked on both `summary`
  and every page; (3) `severity_counts`/`occurrence_counts.total` equal the sum of their four
  severity fields, and `violated_rule_count`/`violation_count` match the corresponding scoped
  total (the "Zähler ohne Scope" corpus shape); (4) the report's own `metric_context` block
  matches what `REGISTRY` currently generates (catches a stale/cached or hand-edited report).
  New CLI subcommand `auditmysite report-lint <file> [--fail-on low|medium|high|critical]`
  (default `high`) prints findings and returns a non-zero exit code (via `AuditError::ConfigError`,
  exit 3) when the worst finding meets or exceeds the threshold — verified end-to-end against a
  hand-built broken report before considering this feature done, not just via unit tests.
  `registry::json_path_candidates` (moved from a private test-only helper in
  `tests/registry_contract.rs` to `src/registry/paths.rs` so `lint` and the contract test share one
  implementation) had a latent bug caught while writing paths.rs's own dedicated unit tests:
  splitting on `" and "` before trimming turned prose like "...score and nested dimension scores"
  into a second bogus path candidate ("nested") — fixed by dropping the `" and "` split entirely
  (only `" / "` ever separates two *real* paths in this codebase's `json_path` text; `take_while`
  alone already stops at the first invalid character, which discards trailing " and ..." prose).
  Added a 5th check: every `REGISTRY` entry's `docs_url` is a well-formed `<path>#<anchor>`
  reference (shape only, no filesystem access — a released binary has no guarantee `docs/` exists
  alongside it; the deeper anchor-resolution check stays in `tests/registry_contract.rs`, which
  only needs to hold in the dev/CI checkout). `tests/lint_fixtures/*.json` (clean single, 3 broken
  variants covering score-alias/grade/batch-certificate mismatches) plus `tests/report_lint_tests.rs`
  spawn the compiled binary end-to-end via `CARGO_BIN_EXE_auditmysite` and assert on exit code +
  finding check-ids — these fixtures double as the seed for #511's regression corpus. No new CI job
  was added: since this test file has no network/Chrome/pdf-feature dependency, it's already
  exercised by the existing unscoped `cargo test` in the `check`/`check-all-features` jobs.
  Added a 6th, narrowly-scoped PDF-traceability check instead of the originally-sketched "does any
  number in the PDF appear anywhere in the JSON" scan (rejected: real false-positive risk from
  dates/page counts/unrelated percentages). New optional `--typst-source <path>` on `report-lint`
  (the `--debug-typ` Typst source for the same report); when given,
  `check_pdf_certificate_traceability` computes the certificate token the JSON's `overall_score`
  implies via the same shared `CERTIFICATE` `BandSet` the PDF itself uses, and checks that exact
  token is present in the Typst text — presence-only (not "no other certificate word may appear",
  since a legend explaining the band system may legitimately mention other tokens), `Severity::Low`
  (advisory, never breaches the default `--fail-on high` on its own). `lint()`'s signature grew a
  second `Option<&str>` parameter for the Typst text. Two `.typ` fixtures added under
  `tests/lint_fixtures/`; a first draft of the "broken" fixture accidentally spelled the certificate
  word out in its own comment ("SEHR GUT" contains "GUT" as a substring) and silently passed —
  caught only because the CLI integration test asserted on the actual finding appearing, not just
  the exit code, which is why report-lint's own test fixtures need their negative-case text
  double-checked for accidental substring self-matches. `#508`–`#511` (coverage matrix, visual PDF
  pipeline, AI critic, feedback corpus) are planned but not started.
- **Report Quality Layer v1.2 — Phase 1: canonical metric registry, 2026-07-16 (#506, tracking #512):**
  new `src/registry/` (`MetricSpec`/`BandSet`/`MetricKind`/`Direction`/`Scope`/`Aggregation`,
  `REGISTRY` const table) gives every specialized number one machine-readable definition instead of
  scattered renderer/doc logic. Seeded 1:1 from `src/output/json.rs`'s former hand-written
  `metric_context()` vec — `metric_context()` now derives `score_definitions`/`count_definitions`
  from `REGISTRY` instead of the other way around, with the same `field`/`unit`/`meaning` text
  (zero JSON output change, verified against snapshot/schema/consistency test suites).
  `docs/OUTPUT_CONTRACT.md` gained a `## Metrics` section with one `<a id>` anchor per registry
  entry; `tests/registry_contract.rs` (mirrors `tests/parity_contract.rs`'s "contract file + test"
  shape) checks unique ids, `docs_url` anchors resolve, `reviewed_at` parses as a date, and
  `json_path` resolves against `docs/json-report.schema.json`/`docs/json-batch-report.schema.json`.
  **Phase 1 complete (all 7 migration steps):** every one of the ~19 independent score→label/grade
  definitions found across taxonomy, PDF renderers, and module-specific label functions now
  references a named `BandSet` in `src/registry/bands.rs` instead of re-coding thresholds —
  `FIVE_BAND` (90/75/60/40 words), `FIVE_BAND_LETTERS` (same cutoffs, A–F), `LETTER_GRADE`
  (90/80/70/60, A–F), `SECURITY_GRADE` (90/80/70/60/50, A+–F), `BATCH_GRADE` (95/90/80/70/60,
  A+–F), `CERTIFICATE` (90/75/60/40, SEHR GUT…UNGENÜGEND — was independently re-implemented in
  both `audit::scoring::calculate_certificate` and the PDF's `cover::batch_certificate_label`
  before this migration), `COVER_PHRASE`/`SCORE_RANGE` (90/75/60/40, sentence variants), `MEDAL`
  (90/80/60, terminal-table GOLD/SILVER/BRONZE/FAILED), `BAR_COLOR_BAND` (90/80/70/50, terminal
  color only), and `SEO_BAND` (90/70/55/35 — SEO's own family, deliberately kept distinct, not
  collapsed into `FIVE_BAND`). No threshold values changed; `output::cli::colorize_grade` was
  deliberately left untouched (keys off an already-resolved grade letter, no threshold to
  register). Found and flagged but **not fixed** (out of scope for #506):
  `performance::scoring::PerformanceGrade::emoji()`/`.label()`/`Display` are dead code — never
  called anywhere reachable in `src/` (JSON serializes the enum variant name directly; the PDF
  renders scores via the now-migrated `score_band_label`/`score_range_label` instead), so the
  emoji never actually reaches a report despite existing in source.
  `#507`–`#511` (report-lint, coverage matrix, visual PDF pipeline, AI critic, feedback corpus)
  are planned but not started.
- **BFSG / EN 301 549 mapping annex, 2026-07-15 (#en301549):** `src/wcag/en301549.rs` — canonical
  50-entry WCAG 2.1 A/AA ↔ EN 301 549 (chapter 9, "Web") clause table, `derive_annex`/
  `derive_batch_rollup` as pure projections over `NormalizedFinding` (nothing new stored on
  `NormalizedReport`, no cache-signature change). Four-way scope split per clause: violations
  found / no violations in automated scope / manual review required / (chapter-level, not
  per-clause) out of audit scope. `screen_reader/bfsg.rs` reduced to a thin wrapper; the
  legally-unverified `"§12 Abs. 1"` citation stays local there, deliberately not propagated.
  JSON `en301549_annex` always emitted (`PageDetail`, single + batch) plus a batch
  `UnifiedSummary.en301549_rollup`; the PDF appendix only renders behind the new opt-in
  `--annex en301549` flag ("Zusatz", not default-on). Disclaimer text (DE/EN) is a
  scope-of-testing disclosure only — no statutory citation, no conformity claim — reusing this
  project's existing "manual audit with assistive technologies (screen reader, keyboard
  navigation)" wording rather than inventing new phrasing.
- **Plain-language content in the existing PDF, 2026-07-15:** no separate report variant — the
  Chapter 02 finding card gained a plain-language lead-in (`customer_description` + `user_impact`)
  between the header and the QA-meta block (previously not rendered there at all, not just
  misordered). `finding_group_from_normalized`'s no-`RuleExplanation` fallback no longer leaks raw
  canonical-English `f.description` into German reports. Part-1 divider reframed as dual-audience
  ("Inhaber, Entscheider und Entwickler").
- **Journey × Commerce deepening, 2026-07-14/15:** form-error journey now groups required fields
  into up to 3 per-form candidates (was one page-wide candidate) and a `PURCHASE_FINAL_HINTS`
  deny-list guarantees a purchase-final button (e.g. "Jetzt kaufen") is never a synthetic-click
  target. New commerce-aware journeys on a detected shop's product-detail page under
  `--interactive full`: add-to-cart feedback (SC 4.1.3) and quantity-stepper operability
  (SC 2.1.1/4.1.2). **`CommercePageKind::Cart`/`::Checkout` removed entirely** (breaking JSON
  change) — this tool has no cross-page session/cart state, so a cart/checkout URL reached cold
  is almost always empty or redirects before rendering anything a page-kind-gated heuristic could
  act on; confirmed no reference in the sibling `auditmysite_studio` repo before landing.
- **WCAG coverage + correctness sweep, 2026-07-14:** new rules 1.3.2 Meaningful Sequence, 3.3.7
  Redundant Entry, 2.4.11/2.4.12 Focus Not Obscured, 2.2.2 Pause/Stop/Hide (automated WCAG-AA
  count now 36/50, up from 33). Fixed three known-defective rules: `focus_visible_css.rs` (never
  fired in production — missing evidence selector demoted every finding to a warning),
  `focus_visible.rs` (dead AX-tree `tabindex` read, removed), `non_text_contrast.rs` (mistagged/
  dead, replaced by a real CDP-based `non_text_contrast_css.rs`). Closed remaining #406
  localization gaps (Dark Mode, Tastatur-Journey, `expected_impact`/`complexity_reason`) and
  several report-wording/readability fixes across Chapters 01–03 of the single report.
- **Evidence-Grade Findings (single report only) + Template-Root-Cause-Dedup (batch only), 2026-07-14:**
  Single-report finding cards now embed a cropped element screenshot (`src/accessibility/element_capture.rs`,
  gated on single-URL mode via `PipelineConfig.capture_element_evidence`, capped at 12 crops/report,
  contrast findings excluded by construction), a ≤3-level DOM path, and computed contrast-ratio evidence
  (`ViolationEvidence::computed`, `OccurrenceDetail.evidence: Vec<ViolationEvidence>` — new additive JSON
  field, `docs/json-report.schema.json` updated). Batch reports gain verified template-level clustering
  (`src/audit/template_dedup.rs`): findings sharing an identical `(rule_id, normalized selector)` fingerprint
  across ≥3 pages / ≥60% coverage become a `TemplateCluster` (`confirmed` when the HTML-snippet shape also
  matches, `likely` otherwise — decision-action wording only upgrades for `confirmed`), surfaced additively
  in `UnifiedSummary.template_clusters` and the batch PDF. Both features are additive/JSON-safe (screenshot
  bytes are `#[serde(skip)]`, never touch cached `report.json`). Fixed two pre-existing binary-test
  regressions surfaced by running the full `cargo test --features pdf`/`--no-default-features` suites
  (not covered by `cargo test --lib`): a stale `non_text_contrast`→`non_text_contrast_css` rename reference
  and a stale `KNOWN_EXCEPTIONS` entry in `tests/wcag_coverage.rs`.
- **Product-Grade PDF-Redesign (Single-Report, PR feat/report-product-redesign):** Cover als komponiertes Dashboard (dominanter Overall-Score + Notenband-Phrase + Modul-Gauge-Strip); Management-Sicht mit Severity-Zählern, Spider-Radar „Qualitätsprofil" und Stärken/Optimierungs-Cards; jedes Modul ein eigenes Level-2-Kapitel mit Magazin-Opener + Kernaussage-Zeile (#15); AI-Visibility + Content-Visibility + Source-Quality zu einem Kapitel „KI & Vertrauen" zusammengeführt; Maßnahmenplan als Action-Cards gruppiert nach Problem-Ebene (systemisch/lokal, ohne Zeit/Aufwand); Ursachen-Verteilung als Bar-Chart; ToC auf Top-Ebene (depth 2); moderne randlose Tabellen; durchgängiges 4-Farben-Gesetz in `src/output/pdf/design.rs` (`score_color`/`severity_color`, Schwellen 75/40); kein „/100", kein A–F-Grade (Band-Label via `score_band_label`), keine Emoji. **renderreport 0.2.26** (komponierte `cover-page`, echter Spider-Radar, randlose `audit-table`, de-emoji'te Callouts, sticky Headings/Komponenten-Titel gegen verwaiste Überschriften). **JSON-Fix:** Cache-Hit-JSON emittiert jetzt den vollen `detail.modules`-Blob (zuvor leer, da normalized-only-Pfad).
- **Semantic-Eval komplett entfernt:** Modul `src/semantic_eval/` (Fastembed + Mistral), CLI-Flag `--no-semantic-eval`/`--semantic-eval`, `[semantic_eval]`-TOML-Sektion, `fastembed`-Dependency + `semantic-eval`-Cargo-Feature, Typ `AdvisoryFinding` und das Feld `advisory_findings` (aus `NormalizedReport`/`AuditReport`/JSON sowie den PDF-Advisory-Sektionen). `audit_signature` enthält kein `semantic`-Segment mehr (Cache invalidiert einmalig).
- **Scoring-Korrektheit + Report-Lesbarkeit (PR fix/perf-relative-weight-cap):** relativer Weight-Penalty-Cap (≤70 % der Vitals-Basis, schützt Low-Base-Seiten vor 0); renderreport **0.2.23** (Progress-Arc-Gauges + feste Label-Box, keine Cover-Überlagerung); Customer-Passagen ohne Jargon-Duplikat/Meta-Prefixe; Cover-Label „N Accessibility-Befunde" (Scope explizit, WCAG-only); Vuln-Detektion Lodash↔Underscore via `_.runInContext`; #406-Leaks (search_experience-Komponenten + Warnungen re-derived); Pluralisierung „1 Schema"; `compact_html` (data-URIs → „data:…", Zeilenhöhen); leere „Befunde nach Ursache"-Trenn-Seite gefüllt; kurze Indikator-Module (Best Practices/Tech-Stack) per Divider gepackt statt je eigene Fast-Leerseite.
- **Cache-Korrektheit (PR #458, #404/#405):** voller `AuditReport` wird gecacht (`report.json`, Screenshots gestrippt), Cache-Hits rendern originalgetreu statt über das verlustbehaftete `to_audit_report`; `screen_reader_audit` (`#[serde(skip)]`) wird via `hydrate_cached_report` aus dem AXTree neu gebaut. `NormalizedReport`-Felder mit `skip_serializing_if` haben jetzt `#[serde(default)]` (Round-Trip-Blocker behoben — der Cache lud nie). Verdikt immer aus `cached.audit`. `persist_artifacts` läuft nach der Canonical-Perf-Adoption (`audit_page` gibt `SnapshotData` zurück). `audit_signature` enthält `lang`; korrupter Cache → Miss + Warnung.
- **Report-Qualität (PR #459, #446):** Security/SEO/Page-Health geben bei leerer Findings-Sammlung eine „keine Auffälligkeiten"-Bestätigungszeile aus (`pdf-section-clean`) — „geprüft & sauber" vs. „nicht geprüft" unterscheidbar.
- **Scoring-Korrekturen (PR #460, #455/#456/#457):** DOM-Größe als degressiver Penalty (max 35) statt hartem 59-Cap; Throttled-Profile bekommen die Headline-`content_weight` (keine Slow3G>Fast3G-Inversion); Risk-Breadth-Pfad von Critical-Occurrences entkoppelt (`legal_flags >= 3`), `driven_by`/Summary spiegeln den echten Auslöser (Breadth vs. Volumen).
- **Audit-Qualität (PR #454):** Lokalisierungs-Fixes (Security-CSP, WCAG-Findings, SEO-Heading kanonisch Englisch); Scoring-Entsättigung (DOM-Cap >6000/>10000, Accessibility-Wurzelkurve ab Penalty 70, Mobile-Soft-Floor, Risk=Critical nur bei systemischer Exposition #250); Core-Web-Vitals-Messkorrektheit (CLS Session-Window, LCP+TBT aufs Lade-Fenster begrenzt, `MeasurementContext::LabThrottledMobile` kennzeichnet gedrosselte Headline-Vitals im JSON).
- **Lokalisierungs-Architektur (#406):** JSON kanonisch Englisch, nur PDF mehrsprachig. Analyse backt Englisch, PDF-Präsentation leitet ab (kind-Enum-Muster). Siehe Abschnitt „Lokalisierungs-Architektur". Plus Audit-Finding-Fixes (#442–#452, #411, #447, #449) — PR #453.
- **Catalog-Refactoring** (Phase A+B): `trait AuditModule` + `AuditCatalog` Registry mit Topo-Sort; alle 12 Module migriert; table-driven WCAG-Page-Rule-Catalog; `audit/interpretation.rs` (pre-computed DE/EN-Texte); `audit/summary.rs` (Aggregations-Logik); Builder ist reiner Mapper (#330–#338)
- Branch: `main`
- Cache: `--reuse-cache` validiert `CacheMeta.audit_signature` (Tool-Version + WCAG-Level + aktive Module + Consent) gegen die aktuelle Konfiguration; bei Mismatch Cache-Miss + Warnung, Legacy-Cache ohne Signatur wird nie wiederverwendet (#260)
- Crawler: parserbasierte Linkextraktion via html5ever inkl. `<base href>` (#263)
- Batch-JSON: optionaler `sample`-Block (source, total_discovered, audited, sample_limit, selection, is_sample) + PDF-Prüfumfang-Zeile (#261)
- Performance: `VitalMetric.measurement` (`lab_headless`/`estimated_lab`); INP/TTI/Speed Index als Lab-Schätzung markiert, Lab-Disclaimer im Report (#262)
- Kontrast: Bild-/Gradient-Hintergründe werden zu Manual-Review-Warnungen demoted statt als bestätigte Verstöße (#264, Pixel-Sampling offen)
- **Accessibility Journey Layer** (`--interactive off|basic|full`): Tab-Walk, Skip-Link, Disclosure, Modal, TabList, Menu, Form-Error-Announcement, SPA-Navigation, Linktext-/Heading-/Landmark-Inventur (#297–#301). Ergebnisse in `interactive_findings` + `accessibility_journey` im JSON.
- **Snapshot Export** (`--export-snapshot <path>`): AXTree + Journey-Traces als YAML für CI-Regression (#301).
- Linktext-Stopwords in i18n FTL (`locales/de|en/report.ftl`, Schlüssel `linktext-generic-stopwords`) — erweiterbar ohne Code-Änderung (#299).
- 95+ WCAG rules implemented (Level A, AA, full AAA coverage)
- 2 output formats (json, pdf); table for quick terminal checks
- Batch processing with configurable concurrency
- Pattern Detection: MainNavigation, SkipLink, Accordion, Dialog, DisclosureMenu, TabList, Form
- Modules: Performance, SEO, Security, Mobile, Dark Mode, Design Quality (opt-in, score-neutral), UX, Journey, AI Visibility, Content Visibility, Source Quality, Tech Stack, Best Practices, Commerce, Accessibility Journey Layer
- Consent: `--dismiss-consent` Flag; CMP-Cookie-Injection + Banner-Click; `consent_banner` audit_flag im JSON
- `audit_flags` kinds: `conflicting_signal` (3.1.1 vs. SEO lang), `viewport_gap` (Desktop/Mobile ≥20 Punkte), `consent_banner`, `consent_wall_artifact`, `bypass_blocks_untested` (Skip-Link vorhanden aber funktional kaputt — statischer Check hat PASS, Journey FAIL)
- JSON: **Unified Report Envelope v2.0** — einheitliches Schema für single + batch (`schema_version`, `report_type`, `summary`, `pages[]`, `pages[i].detail`). Breaking Change ggü. v0.17.
- Scoring: Depth-Saturation (Zwei-Phasen), Diversity-Faktor, Soft Floor + logarithmische Kompression für extreme Penalties (≥85 Punkte), WCAG-Prinzip-Coverage; `score_breakdown` (nur bei `score_calculation_method = "viewport_weighted"`, sonst absent)
- Findings: `category`-Feld auf `NormalizedFinding` (`"wcag"` / `"seo"`); `severity_counts` zählt **Findings** (eine Zeile pro Regel/Severity, **nur WCAG-Kategorie** — bleibt risiko-/rechts-relevant). Im JSON-Report decken `occurrence_counts`, `violation_count` und `violated_rule_count` **alle Kategorien (WCAG + SEO)** ab — konsistent mit `findings[]` und `detail.fix_guidance` (#254/#255). `top_recurring_rules` bleibt WCAG-only. Achtung: `NormalizedReport.occurrence_counts` ist weiterhin WCAG-only (speist `SiteState`/Risk), der JSON-PageEntry berechnet die All-Category-Variante separat. `risk.severity` = schwerste Violation über alle Findings (kein eigenes `severity_max`-Feld)
- Risk Level: Score-basierter Fallback (score ≤ 20 → mindestens Medium); `legal_flags > 0` oder `blocking_issues ≥ 1` heben das Level mindestens auf Medium. `legal_flags` zählt **distinct WCAG-Level-A-Regeln** mit High/Critical-Severity (nicht Occurrences).
- History: `schema_version: "1.0"`, `report_type: "history"` in History-JSON-Dateien
- PDF: Throttled-Performance-Tabelle, Indikator-Kennzeichnung konsistent, leere Seite nach ToC behoben; Accessibility-Journey-Section in Single- und Batch-Reports
- Performance-Score: Lighthouse-v10/v11-Gewichtung (FCP 10 %, LCP 25 %, TBT 30 %, CLS 25 %), log-normale Score-Kurven mit p10/p50-Kalibrierung; CLS > 0.5 hart auf 0 gecappt
- `tool_version` als Top-Level-Feld im JSON-Report (parallel zu `schema_version`/`report_type`)
- Sitemap-Summary enthält `violated_rule_count` (dedupliziert über alle Pages) und `top_recurring_rules` (max. 10 häufigste WCAG-Verstöße)
- Pass-Kriterium (`passed_url_count`): accessibility_score ≥ 80, keine Critical-Findings und keine WCAG-Level-A High/Critical-Findings (also `legal_flags == 0`)
- `detail.fix_guidance` ist immer im JSON präsent (leeres Array bei 0 Findings) — auch in Batch-/Sitemap-Reports; dort trägt jede Page ein kompaktes `detail` (nur `fix_guidance`, ohne Modul-Blob), siehe #256
