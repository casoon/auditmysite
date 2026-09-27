# accname-Differential

`auditmysite accname-diff <URL>` stellt die eigene Accessible-Name-Berechnung aus
dem [`accname`](https://crates.io/crates/accname)-Crate gegen die native Berechnung
von Blink und meldet, wo beide auseinandergehen.

```bash
auditmysite accname-diff https://example.com
auditmysite accname-diff https://example.com --output reports/example-accname.json
auditmysite accname-diff https://example.com --max-samples 500

# Korpus: mehrere URLs und/oder eine URL-Datei (eine URL je Zeile, `#` für Kommentare)
auditmysite accname-diff https://a.example https://b.example --output reports/accname-corpus.json
auditmysite accname-diff --url-file urls.txt --output reports/accname-corpus.json
```

Eine URL ergibt das Einzelergebnis wie bisher. Mehr als eine URL — Argumente und
`--url-file` zusammengezählt — ergibt ein Korpus-Aggregat (siehe unten). Alle
Seiten laufen im selben Browser, jede in einem eigenen Tab; eine Seite, die nicht
lädt, landet in `failures`, statt den Lauf abzubrechen.

## Warum das ohne zusätzlichen Aufbau geht

`CdpDocument` (`src/accessibility/dom_document.rs`) trägt beides nebeneinander: den
über `DOM.getDocument` geholten DOM als Arena samt Attributen, und die nativen
Accessibility-Werte aus `Accessibility.getFullAXTree`, über die Backend-Node-ID
verbunden. Damit liegt eine unabhängige zweite Implementierung derselben
Spezifikation bereits im Prozess — kein zweiter Browserlauf, kein Screenreader,
keine VM.

Im Normalbetrieb erfüllt `CdpDocument` das `Semantics`-Trait bewusst über die
nativen Werte; `accname` ist dort der Ersatz für Hosts *ohne* nativen Tree
(astro-post-audit, LiveAudit). Der Differentialmodus ist der einzige Ort, an dem
beide Wege gleichzeitig laufen.

## Chrome ist nicht die Spezifikation

Eine Abweichung ist ein **Hinweis, kein Urteil**. Sie kann sein:

- ein Fehler in `accname`,
- eine bekannte Chrome-Eigenheit,
- eine Stelle, an der [accname 1.2](https://w3c.github.io/accname/) mehrdeutig ist.

Entschieden wird eine Abweichung gegen die Spezifikation oder gegen
[WPT](https://github.com/web-platform-tests/wpt) (`accname/`, `html-aam/`), nicht
gegen Chrome. Deshalb gibt das Kommando **keine Trefferquote** aus und **keinen
Exit-Code ungleich 0**: es misst, es urteilt nicht.

## Was verglichen wird

Nur Elemente, die im Accessibility-Tree ein Gegenstück haben und dort nicht als
`ignored` geführt werden. Für alle anderen hat Chrome keinen Namen berechnet; ein
Vergleich wäre Rauschen. Beide übersprungenen Mengen werden gezählt und
ausgewiesen, damit der nicht verglichene Anteil sichtbar bleibt.

## Klassifikation

Zwei Achsen, weil eine Zahl allein nicht sagt, wo hinzuschauen ist.

**Form der Abweichung** (`names_by_shape`):

| Wert | Bedeutung |
|---|---|
| `whitespace` | Nach Normalisierung gleich, im Rohtext verschieden. `accname` zieht Leerraum selbst zusammen, dieser Fall entsteht also nur, wenn Chrome ihn stehen lässt — etwa bei geschütztem Leerzeichen. Wird getrennt geführt, damit er die Statistik nicht dominiert. |
| `missing_locally` | Chrome hat einen Namen, `accname` keinen. |
| `missing_in_chrome` | `accname` hat einen Namen, Chrome keinen. |
| `mismatch` | Beide haben einen Namen, die Texte unterscheiden sich. |
| `text_transform` | Beide Namen sind bis auf Groß-/Kleinschreibung gleich (Vergleich über Großschreibung, damit „ß“ → „SS“ erfasst wird), und das Element oder ein Nachfahre hat ein berechnetes `text-transform` ungleich `none`. Chrome wendet die CSS-Transformation auf den Namen an, `accname` rechnet über den DOM-Text. **Erwartete Abweichung, kein Fehler** (Entscheidung 2026-09-27): accname 1.2 regelt CSS-Transformationen nicht, `accname` bleibt beim DOM-Text. Getrennt geführt, weil die Klasse sonst jeden Korpus dominiert. |

Der Stil wird nur für die Kandidaten geholt — Elemente, deren Namen sich nur in
der Groß-/Kleinschreibung unterscheiden — über `DOM.resolveNode` und
`getComputedStyle` im laufenden Tab. Das ist Messung auf der Seite dieses
Werkzeugs; `accname` bleibt unberührt. Ein Kandidat ohne `text-transform` bleibt
`mismatch` und damit sichtbar.

**Namensquelle laut Chrome** (`names_by_source`): die Quelle, die Chrome in
`name.sources` von `Accessibility.getFullAXTree` als erste mit Wert meldet (spätere
sind `superseded`). Die Kennung folgt CDP: bei `attribute` das Attribut
(`aria-label`, `alt`, `title`, `value`, …), bei `relatedElement` `aria-labelledby`
oder die native Quelle — `labelfor`/`labelwrapped` werden `label`, ein
SVG-`<title>`-Kind wird `title-element`, andere (`legend`, `tablecaption`,
`figcaption`, …) behalten Chromes Namen —, dazu `placeholder` und `contents`.
`unknown` bleibt nur, wo Chrome gar keine Quelle mit Wert nennt. Die nützlichere
Achse, weil sie direkt auf den Abschnitt der Spezifikation zeigt, der zu prüfen ist.

Die feine Quelle reist neben dem AXTree (`extract_ax_tree_with_name_sources`,
`CdpDocument::with_chrome_name_sources`). Das grobe `NameSource` aus
`a11y-perception`, das die WCAG-Regeln lesen, bleibt unverändert.

`names_by_shape_and_source` führt beide Achsen als Kreuztabelle.

## Rollenvergleich

Sekundär und heuristisch. Chrome liefert im AX-Tree teils interne Bezeichnungen
statt ARIA-Rollennamen (`RootWebArea`, `StaticText`, `LineBreak`). Diese werden am
Großbuchstaben erkannt und als `roles_not_comparable` gezählt statt als Abweichung.
Die Heuristik ist als solche ausgewiesen; die Zahl gehört mitgelesen.

Bekannte Schreibvarianten derselben Rolle zählen als gleich: Chrome `image`
gegen `accname` `img` (ARIA 1.3 führt `image` als Synonym). Das ist Normalisierung
der Messung, keine Änderung an `accname`.

## JSON-Ausgabe

`--output` schreibt das vollständige Ergebnis. Zählungen sind immer vollständig,
`--max-samples` deckelt nur die mitgeführten Beispiele je Abweichungsart.

Jedes Beispiel trägt `backend_node_id` und `ax_node_id` — der Rückweg auf das
Element in der Seite und in den vorhandenen Anreicherungspfad
(`enrichment`, `element_capture`).

## Korpus-Aggregat

Bei mehr als einer URL schreibt `--output` ein `AccnameCorpus`:

| Feld | Inhalt |
|---|---|
| `pages_total`, `pages_compared`, `failures` | Umfang und nicht geladene Seiten |
| `elements_*`, `skipped_*`, `names_equal`, `roles_*` | Summen über alle Seiten |
| `names_by_shape`, `names_by_source`, `names_by_shape_and_source` | Summen der Klassifikation |
| `name_patterns` | Wiederkehrende Muster (Form, Quelle, Tag, Namenspaar) mit `occurrences`, `pages`, `first_url`; je Form nach Vorkommen sortiert, gedeckelt durch `--max-samples` |
| `patterns_complete` | `false`, wenn eine Seite an ihre Beispielgrenze gestoßen ist — dann sind die Vorkommen je Muster Untergrenzen |
| `pages` | Die vollständigen Einzelergebnisse, jeweils mit `url` |

Die Muster sind die Leseeinheit für einen Korpus: dieselbe Navigation auf dreißig
Unterseiten ist ein Befund, nicht dreißig.

## Grenzen

- Der Vergleich läuft gegen **eine** Engine. Eine Übereinstimmung mit Chrome ist
  kein Nachweis von Spezifikationstreue, nur Abwesenheit eines Unterschieds.
- Ein einzelner Seitenlauf ist eine Stichprobe. Aussagekraft entsteht erst über
  einen Korpus — siehe `plan/52-accname-differential-korpus.md`.
- Der Rollenvergleich ist heuristisch abgegrenzt, siehe oben.

## Code

| Datei | Inhalt |
|---|---|
| `src/accessibility/accname_diff.rs` | Vergleich, Klassifikation und Korpus-Aggregat, reine Funktionen über `CdpDocument`, unit-getestet ohne Browser |
| `src/cli/args.rs` | `Command::AccnameDiff` |
| `src/cli/commands.rs` | `run_accname_diff_command`, Stilabfrage für `text_transform`, Terminalausgabe |
