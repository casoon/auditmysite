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
└── i18n/                # Project Fluent (.ftl), default language: German
```

## Key CLI Modes
- Single: `auditmysite <URL>`
- Sitemap: `auditmysite --sitemap <SITEMAP_URL>` (batch from XML sitemap)
- URL file: `auditmysite --url-file <FILE>` (batch from text file)
- Full audit: `--full` (enables performance, seo, security, mobile)
- Browser: `auditmysite browser {detect|install|remove|path}`, `auditmysite doctor`
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

## Current State (v1.1.0)
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
