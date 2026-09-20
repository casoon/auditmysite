# accname-Differential

`auditmysite accname-diff <URL>` stellt die eigene Accessible-Name-Berechnung aus
dem [`accname`](https://crates.io/crates/accname)-Crate gegen die native Berechnung
von Blink und meldet, wo beide auseinandergehen.

```bash
auditmysite accname-diff https://example.com
auditmysite accname-diff https://example.com --output reports/example-accname.json
auditmysite accname-diff https://example.com --max-samples 500
```

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

**Namensquelle laut Chrome** (`names_by_source`): `aria-label`,
`aria-labelledby`, `label`, `title`, `alt`, `placeholder`, `contents`, `value`,
`unknown`. Die nützlichere Achse, weil sie direkt auf den Abschnitt der
Spezifikation zeigt, der zu prüfen ist.

## Rollenvergleich

Sekundär und heuristisch. Chrome liefert im AX-Tree teils interne Bezeichnungen
statt ARIA-Rollennamen (`RootWebArea`, `StaticText`, `LineBreak`). Diese werden am
Großbuchstaben erkannt und als `roles_not_comparable` gezählt statt als Abweichung.
Die Heuristik ist als solche ausgewiesen; die Zahl gehört mitgelesen.

## JSON-Ausgabe

`--output` schreibt das vollständige Ergebnis. Zählungen sind immer vollständig,
`--max-samples` deckelt nur die mitgeführten Beispiele je Abweichungsart.

Jedes Beispiel trägt `backend_node_id` und `ax_node_id` — der Rückweg auf das
Element in der Seite und in den vorhandenen Anreicherungspfad
(`enrichment`, `element_capture`).

## Grenzen

- Der Vergleich läuft gegen **eine** Engine. Eine Übereinstimmung mit Chrome ist
  kein Nachweis von Spezifikationstreue, nur Abwesenheit eines Unterschieds.
- Ein einzelner Seitenlauf ist eine Stichprobe. Aussagekraft entsteht erst über
  einen Korpus — siehe `plan/52-accname-differential-korpus.md`.
- Der Rollenvergleich ist heuristisch abgegrenzt, siehe oben.

## Code

| Datei | Inhalt |
|---|---|
| `src/accessibility/accname_diff.rs` | Vergleich und Klassifikation, reine Funktion über `CdpDocument`, unit-getestet ohne Browser |
| `src/cli/args.rs` | `Command::AccnameDiff` |
| `src/cli/commands.rs` | `run_accname_diff_command`, Terminalausgabe |
