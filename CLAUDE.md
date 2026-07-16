# AuditMySite - Project Instructions

## Project Overview
Resource-efficient WCAG 2.1 Accessibility Checker written in Rust. Audits web pages using Chrome DevTools Protocol (CDP) and the browser's native Accessibility Tree. Supports single URL, sitemap batch, and URL file batch modes.

## Architecture
- **Language:** Rust (async with tokio)
- **Browser:** Chrome/Chromium via `chromiumoxide` (CDP)
- **CLI:** `clap` with derive macros
- **PDF:** `renderreport` (Typst-based, optional `pdf` feature) — lokales Repo unter `../renderreport`
- **Config:** Optional `auditmysite.toml` files

## Module Structure
```
src/
├── main.rs              # CLI entry point + test module
├── lib.rs               # Library exports
├── error.rs             # Centralized error types (AuditError)
├── util.rs              # Utility functions
│
├── cli/                 # CLI layer (args, config, orchestration)
│   ├── args.rs          # Clap args (Args, WcagLevel, OutputFormat)
│   ├── config.rs        # auditmysite.toml config file support
│   ├── commands.rs      # Subcommand handlers (browser, doctor, plan)
│   ├── runners.rs       # Mode runners (single, batch, compare)
│   ├── report_writers.rs# Output dispatch (single/batch/comparison)
│   ├── output_paths.rs  # File path generation for reports
│   ├── plan.rs          # Pre-audit plan/banner printing
│   └── sitemap_suggest.rs # Sitemap discovery + interactive prompt
│
├── audit/               # Pipeline, batch, scoring, normalization
├── browser/             # Chrome detection, launch, pooling
├── accessibility/       # AXTree extraction via CDP
├── wcag/                # WCAG rule engine + 50+ rule files
├── best_practices/      # Console errors and vulnerable JS library detection
│   ├── console_errors.rs # CDP-based console error/warning collection
│   └── vulnerable_libs.rs # Known-CVE JS library detection (jQuery, Bootstrap, …)
│
├── performance/         # Core Web Vitals, render-blocking, content weight
│   ├── animations.rs    # Non-composited animation detection
│   ├── coverage.rs      # Unused JS/CSS detection via CDP Coverage API
│   ├── critical_chain.rs # Critical request chain analysis
│   ├── minification.rs  # Unminified JS/CSS asset detection
│   └── third_party.rs   # Third-party resource attribution per origin
├── seo/                 # Meta, headings, schema, social, technical SEO
│   ├── image_efficiency.rs # Image format and resolution analysis
│   ├── schema.rs        # JSON-LD extraction and normalization
│   ├── schema_rules.rs  # Central feature-specific structured-data rules
│   ├── schema_fit.rs    # Visible page-type to primary-schema fit
│   └── schema_parity.rs # Visible-content to JSON-LD consistency checks
├── security/            # Security header analysis
├── mobile/              # Mobile friendliness analysis
├── dark_mode/           # Dark mode support detection and contrast
├── ux/                  # UX analysis (5 dimensions, saturation curves)
├── journey/             # User journey analysis, page intent detection
├── screen_reader/       # Screen-reader reading-order primitives
├── ai_visibility/       # AI/LLM discoverability analysis
├── content_visibility/  # Cross-module signal aggregation (SEO+AI+Quality)
├── commerce/            # Shop audit (derive-only, shop-gated): product schema-completeness, mandatory/trust-page links, page-kind (PDP/Category only — Cart/Checkout removed, unreachable in this tool's stateless single-page model), batch roll-up
├── source_quality/      # Source quality signals (headers, schema, HTTPS)
├── tech_stack/          # CMS/framework detection from in-page signals
├── patterns/            # UI pattern detection (nav, accordion, modal, …)
├── assessment/          # Shared assessment types and evidence model
├── studio/              # Studio contract types (GUI data contract)
│
├── output/              # Formatters: table, json, pdf
├── taxonomy/            # Severity, Dimension, IssueClass enums
├── registry/            # Canonical metric registry (#506): MetricSpec/BandSet per specialized JSON/PDF/docs number
├── lint/                # Deterministic report-lint (#507): score/grade/certificate/count-sum checks over JSON reports
└── i18n/                # Project Fluent (.ftl), default language: German
```

## Key CLI Modes
- Single: `auditmysite <URL>`
- Sitemap: `auditmysite --sitemap <SITEMAP_URL>` (batch from XML sitemap)
- URL file: `auditmysite --url-file <FILE>` (batch from text file)
- Full audit: `--full` (enables performance, seo, security, mobile)
- Browser: `auditmysite browser {detect|install|remove|path}`, `auditmysite doctor`
- Report lint: `auditmysite report-lint <JSON_FILE> [--fail-on low|medium|high|critical]` (#507, deterministic checks, no network/Chrome; default fail-on: high; exit code 3 if breached)
- Output formats: `--format {json|table|pdf}`

## Report Intent
- **Single URL audit** is intentionally detailed and page-specific.
- Use it when one concrete page should be reviewed deeply, with findings, explanations, module detail, and implementation guidance for that page.
- **Sitemap / batch audit** is intentionally aggregated and domain-wide.
- Use it when multiple URLs should be compared, averaged, and prioritized across the site.
- Batch reports must focus on cross-page information such as:
  - average scores
  - strongest / weakest URLs
  - recurring issues
  - URL ranking and compact URL matrices
  - distribution patterns across the scanned set
- Batch reports must **not** devolve into a stack of single-page reports. Per-URL detail should stay compressed unless a dedicated technical appendix is explicitly intended.

## Reports Directory
- **All manually generated test reports MUST be saved to `reports/`**
- Use `--output reports/<filename>` when running audits
- The `reports/` directory is gitignored (except `reports/README.md`)
- Naming convention: `<domain>-audit.<format>` (e.g., `casoon-audit.html`)
- Batch reports: `<domain>-batch-audit.<format>`

## Build & Test
```bash
cargo build --release          # Build optimized binary
cargo check                    # Fast compile check
cargo check --all-features     # PFLICHT vor jedem Push — was CI prüft
cargo test                     # Run all tests
cargo test --lib               # Unit tests only
```

**Vor jedem Push `cargo check --all-features` ausführen.** CI prüft immer mit allen Features und Clippy.
Ein pre-push Hook ist unter `.git/hooks/pre-push` eingerichtet und läuft automatisch.

Häufige Falle: neue Felder in `NormalizedReport` brechen Struct-Initialisierer in
`src/audit/normalized.rs` und `src/audit/summary.rs`. Immer beide prüfen.

## Testing Against Live Sites
```bash
# 1. Single page audit (all modules) — tiefe Analyse einer konkreten Seite
./target/release/auditmysite https://example.com --full --format pdf --output reports/example-audit.pdf

# 2. Sample batch audit — 20 Seiten als repräsentativer Durchschnitt
# Ideal um template-weite Probleme (fehlendes ARIA, Struktur, SEO-Muster)
# von seitenspezifischen Fehlern zu trennen. Liefert stabile Durchschnittswerte.
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format pdf --output reports/example-sample-audit.pdf --max-pages 20

# 3. Full sitemap batch audit — alle Seiten, domain-weit
./target/release/auditmysite --sitemap https://example.com/sitemap.xml --full --format pdf --output reports/example-batch-audit.pdf

# Quick CLI check
./target/release/auditmysite https://example.com --format table
```

## renderreport-Workflow

`renderreport` ist eine eigene Typst-basierte PDF-Library unter `/Users/jseidel/GitHub/renderreport`.

**Dependency-Regel:** Immer als **crates.io-Dependency mit exakter Version** — niemals als `path`- oder `git`-Dep:
```toml
renderreport = { version = "0.2.19", optional = true }
```

**Neue Komponente oder Bugfix in renderreport:**
1. Änderungen in `/Users/jseidel/GitHub/renderreport` vornehmen
2. Version in `renderreport/Cargo.toml` bumpen (z. B. `0.2.19` → `0.2.20`)
3. In renderreport committen und pushen: `git push origin main`
4. Tag setzen und pushen: `git tag v0.2.20 && git push origin v0.2.20`
5. Auf crates.io veröffentlichen: `cargo publish --allow-dirty`
6. In `auditmysite/Cargo.toml` die Version aktualisieren
7. `cargo check --features pdf` zur Verifikation
8. `Cargo.lock` committen

**Komponenten** (Rust-Struct + Typst-Template + Registry-Eintrag):
- Rust-Struct: `src/components/standard.rs` oder `advanced.rs`
- Typst-Template: `templates/components/<name>.typ`
- Registry: `src/components/registry.rs` → `self.register(ComponentId::new("name"), include_str!(...))`
- Bei Verwendung in FlowGroup: Eintrag in `templates/components/flow_group.typ`
- Export über `pub use standard::*` in `src/components/mod.rs` — kein separater Re-export nötig

**Spacing-Tokens:** spacing-1=4pt, spacing-2=6pt, spacing-3=10pt, spacing-4=14pt, spacing-5=20pt
**Font-Tokens:** xs=8.5pt, sm=8.8pt, base=10.5pt, lg=13pt, xl=18pt, 2xl=24pt

## Report Format Rules
- **Always use PDF format** (`--format pdf`) when generating test reports
- Never use HTML export for reports
- PDF reports use the `renderreport` Typst engine with full module detail sections

## Lokalisierungs-Architektur (#406 — PFLICHT bei neuen report-sichtbaren Texten)
**JSON ist kanonisch Englisch, nur das PDF ist mehrsprachig.** Regel:
- Die **Analyse-/Derive-Schicht backt kanonisches Englisch** in die gespeicherten Structs
  (`AuditReport`/`NormalizedReport` → JSON). Niemals die Lauf-Sprache in die Analyse-Ergebnisse
  backen — das JSON muss sprach-unabhängig englisch bleiben.
- Die **PDF-Präsentationsschicht leitet lokalisierte Texte zur Laufzeit ab** (`i18n.locale()`),
  über reine Funktionen/`build_*_presentation`/`render_*`.
- **Muster für message-baked Structs:** das Struct trägt ein kanonisches `kind`-Enum (+ Rohwerte
  in einem `#[serde(skip)]`/`skip_serializing_if`-Feld); eine reine `pub fn *_text(kind, .., en)`
  ist die EINZIGE Textquelle — die Analyse ruft sie mit `en=true`, der PDF-Builder mit der
  Lauf-Sprache. Beispiele: `source_quality`, `content_visibility`, `ai_visibility`, `journey`, `ux`,
  `seo/page_health` (`collect_issues`), `screen_reader`.
- **Erkennungs-Sprache ≠ Message-Sprache:** sprachabhängige DETEKTION (z. B. Stopword-Matching für
  generische Linktexte) folgt der **Seiten-Sprache**, nicht der Ausgabe-Sprache. Beide getrennt
  durchreichen (siehe `screen_reader::analyze_reading_sequence(detect_locale, message_en)`).
- Guard-Test pro lokalisiertem Modul: EN-Ausgabe enthält keine deutschen Umlaute/ß.

## Report Wording Style
Gilt für alle Interpretations-/Erklärungstexte (`interpret_score`, `build_seo_interpretation`,
Overall-Erklärung, Modul-Dashboard). Quelle: `src/output/builder/helpers.rs` (`interpret_score`),
`src/output/builder/seo.rs`. Review-Surface: `reports/interpretations.json` (regenerieren mit
`cargo test --lib export_all_interpretations -- --ignored --nocapture`).

- **Lokalisierung ist Pflicht.** Bei `locale = "en"` echte englische Sätze ausgeben — nie deutschen
  Satzbau mit eingesetztem englischem Nomen ("die accessibility ist…"). Jeder Text existiert in `de` **und** `en`.
- **Module unterscheiden sich.** Keine geteilte Satzschablone über alle Module. Pro Modul eigene
  Betonung: Accessibility = rechtlich/Zugänglichkeit, Performance/UX/Journey = Nutzerwirkung,
  Security = vorsichtig/juristisch ("im geprüften Umfang", keine Sicherheitsgarantie),
  Mobile = "Nutzung auf Mobilgeräten" (nicht "mobile Nutzbarkeit").
- **Beschreibe Wirkung, nicht nur Zustand.** Gute Audit-Sätze decken Zustand + Auswirkung (+ ggf.
  Risiko/Priorität) ab — nicht nur "ist solide".
- **Aufwand nicht als Zeit ausdrücken.** Keine Zeitfenster oder Fristen für Aufwandsschätzungen
  nennen ("einige Tage", "1–4 Wochen", "mittelfristig", "innerhalb von Tagen"). Stattdessen nach
  Aufwand/Priorität formulieren: `geringer Aufwand`, `mittlerer Aufwand`, `strukturelle Änderung`.
- **Notenbänder (Label-Präfix):** `Sehr gut` (≥90) · `Gut` (≥75) · `Verbesserungswürdig` (≥60) ·
  `Ausbaufähig` (≥40) · `Kritisch` (<40). EN: `Excellent` · `Good` · `Needs improvement` ·
  `Inadequate` · `Critical`. **"Befriedigend" ist verboten** (klingt nach Schulnote).
- **Verbotene Füllphrasen:** "auf einem hohen Niveau", "einzelne Verbesserungen sind möglich",
  "weist (relevante/einzelne) Schwächen auf", "solide" als Allzweckwort. Bei SEO Endkunden-Jargon
  vermeiden ("Ranking-Signale" → "Sichtbarkeit in Suchmaschinen").
- **Bevorzugtes Vokabular:** beeinträchtigt, erschwert, stabil, konsistent, technisch sauber,
  zuverlässig, robust, eingeschränkt, fehlend, unvollständig, nachvollziehbar.

## Architecture Documentation
Whenever a new module is added, renamed, or removed, update the Module Structure section above **and** `ARCHITECTURE.md` in the same commit. Also update the `Current State` version and module list when bumping the version.

## Code Conventions
- Use `thiserror` for error types, `anyhow` for propagation
- WCAG rules go in `src/wcag/rules/` as individual files, register in `mod.rs`
- Output formatters go in `src/output/`, support both single and batch reports
- Keep async operations in audit pipeline and browser modules
- Use `tracing` for structured logging (INFO, WARN, ERROR)

## Report Quality Layer v1.2 — Release-Gate-Policy (#512)
Bewusste, über alle Phasen hinweg getroffene Entscheidung statt einer nachträglichen Lücke:
- **Blockierend, auf jedem PR:** `report-lint` (`tests/report_lint_tests.rs`), der Registry-Contract
  (`tests/registry_contract.rs`) und der Regressionskorpus-Contract
  (`tests/regression_corpus_contract.rs`) laufen ohne eigenen CI-Job — sie sind netzwerk-/Chrome-/
  pdf-feature-frei und werden dadurch bereits vom unscoped `cargo test` in den bestehenden Jobs
  `check` (`--no-default-features`) und `check-all-features` (`--features pdf`) mitgeprüft. Die
  Blank-Page-Visualprüfung, der Page-Budget-Check und der Dual-Viewport-Gauge-Regressionstest
  (Phase 5/#510, `src/output/pdf/tests.rs`) laufen im bestehenden `pdf-smoke`-Job, `pdftoppm`-gated
  — der Job installiert `poppler-utils` jetzt explizit (vorher fehlte das, wodurch alle
  `pdftoppm`/`pdftotext`-gated Tests in CI **immer** still übersprungen wurden, nie tatsächlich
  liefen), und lädt bei einem Fehlschlag `target/pdf-visual-debug/` als Build-Artefakt hoch.
- **Nicht blockierend, manuell/Release-only:** `reports/coverage_matrix.json` (Phase 4/#508) und
  der `report-critic`-Skill (Phase 6/#509) sind absichtlich keine Gates — ein Substring-Coverage-Scan
  kann false-negativ/-positiv sein, ein KI-Kritiker braucht eine Modell-Invokation, die kein
  netzwerk-/API-freier CI-Job leisten kann, ohne genau die Infrastruktur (API-Keys, Kosten,
  Nicht-Determinismus) wieder einzuführen, die mit der Entfernung von `semantic_eval` bewusst
  abgebaut wurde. Beide bleiben von Menschen/Agenten auf Anfrage ausgeführte Review-Werkzeuge.
  Eine vollständige Pixel-Diff-Baseline-Pipeline (gespeicherte Referenzbilder pro stabiler
  Layout-Region) ist weiterhin bewusst **nicht** gebaut — das ursprünglich offene Design-Problem
  (Baseline-Speicherung/-Regenerierung über Font-Hinting/Anti-Aliasing-Unterschiede zwischen
  Maschinen hinweg) wurde stattdessen strukturell umgangen: #510s konkrete Fälle (Blank-Page,
  Seiten-Explosion, die "umgebrochene Dual-Viewport-Zelle") werden über **Struktur-/Same-Run-
  Differenzprüfungen** statt gespeicherter Pixel-Baselines erkannt, siehe Phase-5-Eintrag unten.

## Current State (v1.1.0)
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
- Modules: Performance, SEO, Security, Mobile, Dark Mode, UX, Journey, AI Visibility, Content Visibility, Source Quality, Tech Stack, Best Practices, Commerce, Accessibility Journey Layer
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
