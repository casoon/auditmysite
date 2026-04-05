# Softwarearchitektur — AuditMySite

Technische Struktur und Designentscheidungen des Projekts.

**Sprache:** Rust (async, tokio)
**Größe:** ~39.000 Zeilen Rust, 80+ Module
**Version:** 0.6.3

---

## Modulstruktur

```
src/
├── main.rs                    # CLI-Einstiegspunkt, Modus-Routing
├── lib.rs                     # Öffentliche API, Modul-Exports
├── error.rs                   # Zentrale Fehlertypen (AuditError)
├── util.rs                    # Hilfsfunktionen
│
├── cli/
│   ├── args.rs                # Clap-Argumente (Args, WcagLevel, OutputFormat)
│   ├── config.rs              # Konfigurationsdatei (auditmysite.toml)
│   └── doctor.rs              # Diagnose-Werkzeug
│
├── audit/
│   ├── pipeline.rs            # Kern-Audit-Pipeline (audit_page, run_single_audit)
│   ├── report.rs              # AuditReport, BatchReport, PerformanceResults
│   ├── batch.rs               # Parallele Stapelverarbeitung (run_concurrent_batch)
│   ├── scoring.rs             # AccessibilityScorer, ViolationStatistics
│   ├── normalized.rs          # Report-Normalisierung
│   ├── artifacts.rs           # Artefakt-Persistenz und Caching
│   ├── history.rs             # Verlauf-Tracking für Trend-Analyse
│   ├── crawl.rs               # Seitenentdeckung via BFS-Crawl
│   ├── comparison.rs          # Multi-Domain-Vergleich
│   ├── baseline.rs            # Violation-Waivers / Baseline
│   ├── budget.rs              # Performance-Budget-Prüfungen
│   ├── duplicate.rs           # Near-Duplicate-Erkennung (SimHash)
│   └── summary.rs             # Zusammenfassungs-Generierung
│
├── browser/
│   ├── types.rs               # BrowserKind, DetectedBrowser
│   ├── detection.rs           # System-Scan für Chrome/Edge/Chromium
│   ├── registry.rs            # Plattformspezifische Browser-Pfade
│   ├── resolver.rs            # Prioritätsbasierte Browser-Auswahl
│   ├── manager.rs             # BrowserManager (Launch, CDP-Verbindung)
│   ├── pool.rs                # BrowserPool (concurrent Page-Management)
│   └── installer.rs           # Chrome for Testing / Headless Shell
│
├── accessibility/
│   ├── tree.rs                # AXTree, AXNode, AXProperty
│   ├── extractor.rs           # CDP-basierte AXTree-Extraktion
│   ├── enrichment.rs          # DOM-Selektoren und HTML-Snippets
│   ├── styles.rs              # CSS-Style-Extraktion
│   └── code_gen.rs            # Code-Generierung
│
├── wcag/
│   ├── engine.rs              # Rule-Dispatcher (check_all, RuleFilterConfig)
│   ├── types.rs               # Violation, Severity, WcagResults, RuleMetadata
│   └── rules/                 # 40+ WCAG-Regelimplementierungen (je eine Datei)
│       ├── mod.rs             # Regel-Exports und Dispatcher-Registrierung
│       ├── text_alternatives.rs   # 1.1.1
│       ├── contrast.rs            # 1.4.3 (benötigt CDP)
│       ├── info_relationships.rs  # 1.3.1
│       ├── keyboard.rs            # 2.1.1
│       ├── language.rs            # 3.1.1
│       └── ... (weitere Regelfiles)
│
├── output/
│   ├── cli.rs                 # Tabellen-Formatter für Terminal
│   ├── json.rs                # JSON-Formatter
│   ├── report_model.rs        # ViewModel (ReportViewModel und alle Blöcke)
│   ├── explanations.rs        # Regel-Erklärungen (i18n)
│   ├── builder/               # Rohdaten → ViewModel
│   │   ├── mod.rs             # Exports (build_view_model, build_batch_presentation)
│   │   ├── single.rs          # Single-Report-Builder
│   │   ├── batch.rs           # Batch-Report-Builder
│   │   ├── modules.rs         # Modul-Score-Berechnungen, Lever/Context
│   │   ├── actions.rs         # Maßnahmenplan (Aufwand, Priorität, Rolle)
│   │   ├── seo.rs             # SEO-Aufbereitung
│   │   └── helpers.rs         # Gemeinsame Hilfsfunktionen
│   └── pdf/                   # PDF-Rendering via renderreport (Typst)
│       ├── mod.rs             # Einstiegspunkt (generate_pdf, generate_batch_pdf)
│       ├── cover.rs           # Cover-Page
│       ├── findings.rs        # Findings-Sektion
│       ├── modules.rs         # Modul-Übersicht und Tabellen
│       ├── detail_modules.rs  # Performance, SEO, Security, Mobile-Detail
│       ├── batch.rs           # Batch-Bericht-Struktur
│       ├── history.rs         # Verlauf- und Methodik-Sektion
│       └── helpers.rs         # Engine-Setup, i18n, Hilfsfunktionen
│
├── performance/               # Core Web Vitals, Render-Blocking, Content-Weight
├── seo/                       # Meta, Headings, Schema.org, Social, Technical
├── security/                  # Security-Header-Analyse
├── mobile/                    # Mobile-Friendliness, UX-Heuristiken
├── ux/                        # UX-Analyse (5 Dimensionen, Sättigungskurven)
│   ├── mod.rs                 # Modul-Exports
│   ├── analysis.rs            # 5-Dimensionen-Analyse auf AXTree-Basis
│   └── scoring.rs             # Sättigungskurven, Dimensions-Score, gewichteter Durchschnitt
├── journey/                   # Journey-Analyse (Nutzerfluss, Seitentyp-Erkennung)
│   ├── mod.rs                 # Modul-Exports
│   ├── analysis.rs            # 5-Dimensionen-Journey-Analyse auf AXTree-Basis
│   ├── page_intent.rs         # Seitentyp-Erkennung (Shop, LeadGen, Editorial, etc.)
│   └── scoring.rs             # Dimensions-Score, gewichteter Durchschnitt mit Intent-Gewichten
├── dark_mode/                 # Dark-Mode-Support-Analyse
├── i18n/                      # Project Fluent (.ftl), Standard-Sprache: Deutsch
└── taxonomy/                  # Severity, Dimensions, IssueClass, Score-Enums
```

---

## Kerndatentypen

### `AuditReport` — Audit-Ergebnis einer URL
**Datei:** `src/audit/report.rs`

```rust
pub struct AuditReport {
    pub url: String,
    pub wcag_level: WcagLevel,
    pub timestamp: DateTime<Utc>,
    pub wcag_results: WcagResults,
    pub score: f32,                          // 0–100
    pub grade: String,                       // A–F
    pub certificate: String,                 // PLATINUM / GOLD / SILVER / BRONZE / FAILED
    pub statistics: ViolationStatistics,
    pub nodes_analyzed: usize,
    pub duration_ms: u64,
    pub performance: Option<PerformanceResults>,
    pub seo: Option<SeoAnalysis>,
    pub security: Option<SecurityAnalysis>,
    pub mobile: Option<MobileFriendliness>,
    pub ux: Option<UxAnalysis>,
    pub journey: Option<JourneyAnalysis>,
    pub dark_mode: Option<DarkModeAnalysis>,
    pub budget_violations: Vec<BudgetViolation>,
}
```

### `Violation` — Einzelne WCAG-Verletzung
**Datei:** `src/wcag/types.rs`

```rust
pub struct Violation {
    pub rule: String,              // z.B. "1.1.1"
    pub rule_name: String,         // z.B. "Non-text Content"
    pub level: WcagLevel,          // A / AA / AAA
    pub severity: Severity,        // Critical / High / Medium / Low
    pub message: String,
    pub node_id: String,
    pub role: Option<String>,
    pub name: Option<String>,
    pub selector: Option<String>,  // CSS-Selektor oder Positionshinweis
    pub fix_suggestion: Option<String>,
    pub html_snippet: Option<String>,
    pub suggested_code: Option<String>,
    pub tags: Vec<String>,         // ["wcag2a", "wcag412"]
    pub impact: Option<String>,    // "critical" / "serious" / "moderate" / "minor"
}
```

### `AXTree` / `AXNode` — Accessibility-Tree
**Datei:** `src/accessibility/tree.rs`

```rust
pub struct AXTree {
    pub nodes: HashMap<String, AXNode>,
    pub root_id: Option<String>,
}

pub struct AXNode {
    pub node_id: String,
    pub ignored: bool,
    pub role: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub properties: Vec<AXProperty>,
    pub child_ids: Vec<String>,
    pub parent_id: Option<String>,
    pub backend_dom_node_id: Option<i64>,
}
```

### `ReportViewModel` — Präsentationsschicht
**Datei:** `src/output/report_model.rs`

Das ViewModel ist die einzige Schnittstelle zwischen Builder und Renderer. Es enthält ausschließlich aufbereitete Präsentationsdaten — keine Logik.

```rust
pub struct ReportViewModel {
    pub meta: MetaBlock,               // Titel, Version, Datum
    pub cover: CoverBlock,             // Cover-Page-Daten (Score, Zertifikat)
    pub summary: SummaryBlock,         // Hero-Zusammenfassung (Metriken, Empfehlung)
    pub history: Option<HistoryTrendBlock>,
    pub methodology: MethodologyBlock, // Scope, Methode, Einschränkungen
    pub modules: ModulesBlock,         // Modul-Scores (WCAG, Performance, SEO…)
    pub severity: SeverityBlock,       // Violations nach Schweregrad
    pub findings: FindingsBlock,       // Gruppierte Findings mit Kundenbeschreibung
    pub module_details: ModuleDetailsBlock,
    pub actions: ActionsBlock,         // Maßnahmenplan mit Roadmap
    pub appendix: AppendixBlock,       // Technische Violation-Liste
}
```

---

## Datenfluss: URL → PDF

```
CLI-Args (Args)
  │
  ├─ Browser-Erkennung (detect/find Chrome)
  │   └─ Priorität: CLI --browser > Konfiguration > System-Scan > Installer
  │
  ├─ [Einzel-Modus]
  │   └─ BrowserManager::launch() → CDP-Verbindung
  │       └─ audit_page(page, url, config)
  │
  └─ [Stapel-Modus]
      └─ BrowserPool (n Pages) + FuturesUnordered (bounded Work-Queue)
          └─ audit_page() je URL, max. `concurrency` gleichzeitig

audit_page():
  1. AXTree-Extraktion via CDP
  2. [--full] Performance-Metriken, SEO, Security, Mobile parallel
  3. [--full] UX-Analyse auf AXTree (kein CDP nötig)
  4. [--full] Journey-Analyse auf AXTree (kein CDP nötig)
  5. wcag::check_all(&ax_tree, level) → Vec<Violation>
  5. [AA/AAA] ContrastRule::check_with_page() → CDP-Styles + Kontrastberechnung
  6. enrich_violations_with_page() → CSS-Selektoren, HTML-Snippets
  7. AccessibilityScorer::calculate_score() → f32
  8. → AuditReport

AuditReport
  │
  ├─ [JSON] format_json_normalized() → JsonReport
  ├─ [Table] print_report()
  └─ [PDF]
      │
      ├─ normalize() → NormalizedReport
      ├─ build_view_model() → ReportViewModel
      │   ├─ Violations gruppieren nach rule_id
      │   ├─ Kundenbeschreibungen ableiten
      │   ├─ Modul-Scores und Interpretationen berechnen
      │   └─ Maßnahmenplan mit Aufwand/Priorität/Rolle erzeugen
      │
      └─ generate_pdf(vm, config)
          ├─ renderreport-Engine instanziieren (Typst)
          ├─ add_component() je ViewModel-Block
          │   Cover → Summary → Methodology → Modules →
          │   Findings → Detail-Module (inkl. UX) → Actions → Appendix
          └─ Typst kompiliert → PDF-Bytes → Datei
```

---

## UX-Modul

**Dateien:** `src/ux/analysis.rs`, `src/ux/scoring.rs`

Das UX-Modul analysiert die User Experience einer Seite anhand von 5 Dimensionen, vollständig auf Basis des bereits extrahierten AXTree — ohne zusätzliche CDP-Aufrufe.

### 5 Dimensionen

| Dimension | Gewicht | Was wird geprüft |
|-----------|---------|-------------------|
| CTA Clarity | 30% | CTAs vorhanden, aussagekräftig benannt (DE/EN-Keywords), keine generischen Labels |
| Visual Hierarchy | 20% | H1-Präsenz, Heading-Reihenfolge, Heading-Verteilung |
| Content Clarity | 20% | Textumfang, Subheading-Dichte, Lesbarkeit |
| Trust Signals | 15% | Kontakt/Impressum/Datenschutz-Links, Vertrauenselemente |
| Cognitive Load | 15% | Link-Anzahl, interaktive Elemente, DOM-Größe |

### Sättigungskurven

Jede Dimension verwendet Sättigungskurven statt linearer Abzüge:

```
penalty = max_penalty × (1 - e^(-count / pivot))
```

Wenige Verstöße werden stark bestraft, weitere Verstöße desselben Typs haben abnehmenden Einfluss. Jede Dimension hat einen **Group Cap**, der verhindert dass ein einzelner Bereich den Gesamtscore dominiert.

### Integration

UX läuft im `audit_page()`-Pipeline wenn `--full` aktiv ist. Der Score fließt mit Gewicht 15% in den `overall_score` ein.

---

## Journey-Modul

**Dateien:** `src/journey/analysis.rs`, `src/journey/page_intent.rs`, `src/journey/scoring.rs`

Das Journey-Modul analysiert, wie gut eine Seite einen typischen Nutzerfluss unterstützt — vollständig auf Basis des AXTree, ohne zusätzliche CDP-Aufrufe.

### 5 Dimensionen

| Dimension | Beschreibung |
|-----------|-------------|
| Entry Clarity | Ist der Seitenzweck sofort erkennbar? (H1, Titel, Above-the-fold-Content) |
| Orientation | Kann der Nutzer sich orientieren? (Navigation, Landmarks, Heading-Struktur) |
| Navigation | Sind Links verständlich, eindeutig und strukturiert? |
| Interaction | Können Nutzer mit Controls effektiv interagieren? (Button-Labels, Form-Labels) |
| Conversion | Kann der Nutzer das Seitenziel erreichen? (CTA-Präsenz, Dialog-Blocker, Formular-Komplexität) |

### Seitentyp-Erkennung (PageIntent)

```rust
pub enum PageIntent { Shop, LeadGen, Editorial, Marketing, Corporate, Hub, Unknown }
```

Der erkannte Seitentyp steuert die Gewichtung der Dimensionen. Ein Shop gewichtet Conversion und Trust höher, eine Editorial-Seite Content Clarity und Navigation.

### Friction Points

Automatisch abgeleitete Reibungspunkte im Nutzerpfad, jedem Journey-Schritt zugeordnet und nach Severity priorisiert.

### Integration

Journey läuft im `audit_page()`-Pipeline wenn `--full` aktiv ist. Der Score fließt mit Gewicht 10% in den `overall_score` ein.

---

## Risikobewertung (Score ≠ Risk)

**Datei:** `src/audit/normalized.rs`

Die Risikobewertung ist konzeptionell unabhängig vom Score. Ein Score von 81 kann trotzdem Risikostufe „Kritisch" tragen, wenn z.B. Level-A-Verletzungen vorliegen die unter BFSG/EAA rechtlich relevant sind.

### RiskLevel

```rust
pub enum RiskLevel { Low, Medium, High, Critical }
```

### Berechnung

| Bedingung | Stufe |
|-----------|-------|
| `legal_flags > 0 && critical_issues > 0` | Critical |
| `critical_issues >= 3 \|\| blocking_issues >= 10` | High |
| `high_issues >= 3 \|\| critical_issues >= 1` | Medium |
| sonst | Low |

- **legal_flags**: WCAG Level-A-Verletzungen mit rechtlicher Relevanz (BFSG/EAA)
- **blocking_issues**: Verletzungen von 4.1.2/2.1.1 (interaktive Elemente ohne Namen, fehlende Tastaturzugänglichkeit)
- **critical_issues / high_issues**: Violations nach Severity

### Darstellung

Im PDF erscheint ein farbcodierter Risiko-Callout auf der Zusammenfassungsseite. Im CLI wird die Risikostufe farbig nach dem WCAG-Summary angezeigt. Im JSON liegt `risk` als eigenständiges Objekt im `normalizedReport`.

---

## WCAG-Regelsystem

### Dispatcher-Muster
**Datei:** `src/wcag/engine.rs`

Alle Regeln werden zentral über ein Makro ausgelöst:

```rust
macro_rules! run_if_allowed {
    ($filter:expr, $axe_id:expr, $check_fn:expr, $results:expr, $tree:expr) => {
        if $filter.should_run($axe_id) {
            $results.merge($check_fn($tree));
        }
    };
}
```

`check_all()` ruft `run_level_a_rules()`, `run_level_aa_rules()` und ggf. `run_level_aaa_rules()` auf. Jede Regel-Funktion erhält den `AXTree` und gibt `Vec<Violation>` zurück.

### Sonderfall: Kontrast-Regel
Die Kontrast-Prüfung (1.4.3) benötigt berechnete CSS-Stile und läuft asynchron direkt über CDP — sie ist nicht in den synchronen Dispatcher integriert, sondern wird separat nach `check_all()` aufgerufen.

### Neue Regeln hinzufügen
1. Neue Datei in `src/wcag/rules/` anlegen
2. `check_*`-Funktion implementieren: `fn check_xyz(tree: &AXTree) -> Vec<Violation>`
3. In `src/wcag/rules/mod.rs` exportieren
4. In `src/wcag/engine.rs` mit `run_if_allowed!` registrieren

---

## Async- und Parallelitätsmodell

**Runtime:** Tokio Multi-Threaded Executor

### BrowserPool
**Datei:** `src/browser/pool.rs`

```rust
pub struct BrowserPool {
    browser: BrowserManager,
    pages: Arc<Mutex<VecDeque<Arc<Page>>>>,
    semaphore: Arc<Semaphore>,
    closed: Arc<AtomicBool>,
}
```

- Pages werden wiederverwendet (kein Launch-Overhead je URL)
- `PooledPage` gibt Page beim Drop automatisch zurück
- Konfigurierbare Größe via `--concurrency` (Standard: 3)

### Stapelverarbeitung
**Datei:** `src/audit/batch.rs`

- `FuturesUnordered` als bounded Work-Queue: exakt `concurrency` Tasks gleichzeitig in-flight
- Kein unbegrenztes `tokio::spawn()` — neuer Task wird erst gestartet wenn ein laufender fertig ist
- Atomarer Progress-Counter für Fortschrittsanzeige
- 2 Wiederholungsversuche je URL bei Fehler

---

## Fehlerbehandlung

**Typ:** `AuditError` enum
**Datei:** `src/error.rs`
**Propagation:** `anyhow` für Kontext-Ketten, `thiserror` für Typen

Wichtige Varianten:
- `ChromeNotFound` — kein Browser gefunden
- `NavigationFailed { url, reason }` — Seite nicht ladbar
- `PageLoadTimeout` — Timeout überschritten
- `AXTreeExtractionFailed` — CDP-Query fehlgeschlagen
- `PoolTimeout / PoolExhausted` — Browser-Pool-Fehler
- `ReportGenerationFailed` — PDF/JSON-Generierung fehlgeschlagen

**Strategie:** Frühzeitige Validierung der CLI-Args; graceful Degradation bei optionalen Modulen (Performance/SEO/Security schlagen nicht den WCAG-Audit fehl).

---

## Schlüssel-Abhängigkeiten

| Crate | Zweck |
|-------|-------|
| `chromiumoxide` | Chrome DevTools Protocol |
| `tokio` | Async-Runtime (rt-multi-thread, sync, time, macros) |
| `clap` | CLI-Argumente |
| `serde` / `serde_json` | Serialisierung |
| `anyhow` / `thiserror` | Fehlerbehandlung |
| `reqwest` | HTTP-Client (rustls-tls) |
| `chrono` | Datum/Uhrzeit |
| `tracing` | Strukturiertes Logging |
| `fluent-bundle` | i18n (Project Fluent) |
| `renderreport` | PDF-Generierung (Typst-basiert, lokale Path-Dep) |

---

## Bewusste Designentscheidungen

**Nur JSON und PDF als Output-Formate** — kein Output-Formatter-Trait, kein drittes Format geplant. Fokus auf wenige, hochwertige Ausgaben statt generischer Abstraktion.

**renderreport als lokale Path-Dependency** — renderreport ist ein eigenständiges Projekt, bereits auf crates.io veröffentlicht. Die lokale Abhängigkeit ist Entwicklungsbequemlichkeit während beide Projekte parallel weiterentwickelt werden. Wird auf Registry-Version umgestellt sobald renderreport stabil ist.

**ViewModel ohne Logik** — `report_model.rs` enthält ausschließlich Datenstrukturen. Alle Berechnungen, Gruppierungen und Ableitungen finden im Builder statt. Der PDF-Renderer transformiert nur noch.

**Regelfiles je Datei** — Jede WCAG-Regel lebt in einer eigenen Datei. Ermöglicht unabhängige Weiterentwicklung, einfaches Code-Review und minimale Merge-Konflikte.

**Artefakt-Caching** — AXTree und Metriken werden unter `~/.auditmysite/cache/{domain}/{url_hash}/v{VERSION}/` gespeichert. Die Versions-Unterverzeichnis sorgt für automatische Invalidierung bei Binary-Upgrades. FNV-1a-Hash (deterministisch über Prozesse/Plattformen) dient als Content-Fingerprint für Delta-Erkennung. `--reuse-cache` lädt gespeicherte Artefakte bei erneutem Audit derselben URL. `--force-refresh` umgeht den Cache.
