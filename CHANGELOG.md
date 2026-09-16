# Changelog

Detailed, chronological development history for auditmysite — each entry documents a finding,
the fix, and how it was verified. Extracted from `CLAUDE.md`'s former "Current State" section
(plan/11-claude-md-version-drift.md) so `CLAUDE.md` itself stays focused on working rules and a
short current-state summary. Newest entries first (unchanged order from before the extraction).

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
