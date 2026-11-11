# auditmysite Architecture

This document describes the current implementation. For older browser-design explorations and proposals that are not fully implemented, see [browser-architecture.md](browser-architecture.md).

## Overview

auditmysite is a WCAG 2.2 AA accessibility checker written in Rust. It uses Chrome DevTools Protocol (CDP) to extract the browser's native Accessibility Tree and analyze it for violations.

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI (main.rs)                           │
├─────────────────────────────────────────────────────────────────┤
│                      Audit Pipeline                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Browser  │→ │ AXTree   │→ │  WCAG    │→ │  Report  │        │
│  │ Manager  │  │ Extract  │  │  Engine  │  │  Output  │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── error.rs             # Error types (AuditError)
├── annotations.rs       # --annotations (#961): reads the barrierlab annotation file, matches it to findings; outputs render the result apart from the findings
├── util.rs              # Utility functions
│
├── bin/
│   └── auditmysite-dev.rs # Maintainer binary, `dev-tools` feature (not installed by `cargo install`, #912)
│
├── model/               # Shared value types (#934): named by cli, audit, analysis modules and output; imports nothing else from the crate
│   ├── mod.rs
│   ├── wcag_level.rs    # `WcagLevel` (re-exported by `wcag` and `cli`)
│   ├── interactive_mode.rs # `InteractiveMode` (`--interactive`, `audit_scope.interactive_mode`)
│   ├── report_level.rs  # `ReportLevel` (`--report-level`)
│   ├── annex_kind.rs    # `AnnexKind` (`--annex`)
│   └── color_policy.rs  # `ColorPolicy` (`--color`)
│
├── cli/                 # Command-line interface, the entry layer: nothing outside `cli/` and `main.rs` names `crate::cli` (#934, strict in `tests/architecture_test.rs`)
│   ├── args.rs          # Clap argument parsing (Args, OutputFormat; re-exports the `model` value types)
│   ├── config.rs        # auditmysite.toml config support (re-exports `BudgetConfig` from `audit::budget`)
│   ├── from_args.rs     # The configuration of the layers below, built from `Args` and the TOML `Config`: `PipelineConfig::from_args_and_config`, `From<&Args>` for `PipelineConfig`, `BatchConfig` and `output::terminal::TerminalOptions` (#934)
│   ├── app.rs           # Shared by both binaries: logging, signals, exit code, the audit run
│   ├── commands.rs      # Subcommand handlers (browser, doctor)
│   ├── dev_commands.rs  # Maintainer commands of `auditmysite-dev` only (plan, replay, accname-diff, report-lint, --debug-typ, --export-snapshot, #912)
│   ├── report_lint.rs   # `report-lint` command logic (library, so the release-gate tests run without the dev binary)
│   ├── runners.rs       # Mode runners (single, batch, compare)
│   ├── report_writers.rs # Output dispatch (single/batch/comparison)
│   ├── output_paths.rs  # File path generation for reports
│   ├── plan.rs          # Pre-audit plan/banner printing
│   ├── doctor.rs        # `auditmysite doctor` diagnostics
│   ├── url_filter.rs    # --include-path/--exclude-path glob selection of batch URLs
│   └── sitemap_suggest.rs # Sitemap discovery (hint for a base URL; no prompt)
│
├── browser/             # Chrome/Chromium management
│   ├── mod.rs
│   ├── platform.rs      # Platform contract (macOS ARM64, Linux x64/ARM64, Windows Edge only)
│   ├── detection.rs     # Candidate scan: dedup by canonical path, verdict per candidate
│   ├── probe.rs         # Bounded `--version` and headless CDP probe (launch, connect, cleanup)
│   ├── provenance.rs    # Browser provenance record of the measurement profile (categories, no paths)
│   ├── installer.rs     # Facade: `browser install|remove`, managed binary paths
│   ├── install_plan.rs  # Publisher-manifest selection, URL trust policy
│   ├── install_txn.rs   # Transactional install: lock, bounded download, safe extraction, CDP smoke, atomic promote/rollback
│   ├── bootstrap.rs     # Consent policy for the one-time managed install (`browser ensure`, interactive prompt)
│   ├── resolver.rs      # Selection, failover chain, browser diagnosis (detect/doctor)
│   ├── manager.rs       # Browser lifecycle (launch, navigate, close)
│   ├── pool.rs          # Page pool for concurrent audits
│   ├── consent.rs       # CMP cookie injection + consent-banner dismissal
│   ├── throttle.rs      # CPU/network throttling profiles
│   ├── registry.rs      # Known install locations and PATH names per platform
│   └── mock.rs          # Test-only mock browser
│
├── accessibility/       # Accessibility Tree handling
│   ├── mod.rs
│   ├── extractor.rs     # CDP AXTree extraction
│   ├── tree.rs          # AXNode, AXTree structures
│   ├── enrichment.rs    # AXNode enrichment (roles, computed properties)
│   ├── code_gen.rs      # Selector/snippet generation for findings
│   ├── element_capture.rs # CDP screenshot capture + element highlighting for evidence
│   ├── snapshot.rs      # AXSnapshot: tree + focus + url + title at one point in time
│   ├── diff.rs          # AXTreeDiff between two snapshots, matched by backend node id
│   └── accname_diff.rs  # Differential: own accname crate vs. Chrome's native computation
│
├── wcag/                # WCAG rule engine
│   ├── mod.rs
│   ├── engine.rs        # Rule orchestration; `TREE_RULES` is the AX-tree rule registration
│   ├── expectation.rs   # Expected rule outcomes per level/filter/viewport, derived from the registrations; counting unit of checks (#778)
│   ├── types.rs         # Violation, Severity, WcagResults
│   ├── coverage.rs      # WCAG success-criterion coverage accounting
│   ├── en301549.rs      # EN 301 549 (chapter 9, Web) clause mapping + annex derivation
│   └── rules/           # Local WCAG rules that need the browser or have no shared id (17 files; the rest runs via wcag::shared)
│       ├── text_alternatives.rs  # 1.1.1
│       ├── keyboard.rs           # 2.1.2 keyboard-trap note (2.1.1 is shared since #694)
│       ├── headings.rs           # 2.4.6
│       ├── target_size_minimum.rs # 2.5.8
│       ├── display_modes.rs      # display/* checks of the data-display convention that need the running page (#653, best-practice); the static ones are shared (viz/*, #699)
│       └── ... (registered in rules/mod.rs)
│
├── display/             # Display modes (#653): --display pre-navigation setup (localStorage.display,
│   └── mod.rs           # prefers-reduced-motion emulation, body observer) and per-page detection
│
├── taxonomy/            # Rule taxonomy & classification
│   ├── mod.rs
│   ├── rules.rs         # Canonical rule list (RULES) with metadata; a rule's texts come from `locales/<lang>/rules.ftl` via `Rule::title`/`user_impact`/`technical_impact(en)`, key = `rule_text_key(id)`: one message per rule id, which also carries the explanation texts of a rule that has an explanation entry under its own id (#935)
│   ├── rule_docs.rs     # rule_doc_url(): rule id → page on the project site (help_url, #929)
│   ├── criteria.rs      # WCAG success-criterion definitions
│   ├── dimensions.rs    # Dimension enum (Barrierefreiheit, Usability, ...)
│   ├── issue_class.rs   # IssueClass enum (Fehlend, Falsch, Unvollständig)
│   ├── score.rs         # Taxonomy-based score impact helpers
│   ├── score_area.rs    # rule_id → ScoreArea map for the accessibility breakdown
│   └── severity.rs      # Severity mapping helpers
│
├── registry/            # Canonical metric registry (#506, Report Quality Layer)
│   ├── mod.rs           # MetricSpec/BandSet/MetricKind/Direction/Scope/Aggregation
│   ├── metrics.rs       # REGISTRY — one entry per specialized JSON/PDF/docs number
│   ├── bands.rs         # Named score→label band families (FIVE_BAND, ...)
│   └── paths.rs         # json_path_candidates() — interprets MetricSpec.json_path text
│
├── lint/                # Deterministic report-lint (#507, Report Quality Layer)
│   ├── mod.rs           # LintFinding/LintReport, lint() entry point
│   ├── checks.rs        # Score/grade/count-sum/metric_context checks
│   └── invariants.rs    # Independent exported-weight, execution and finding-conservation checks
│
├── audit/               # Audit orchestration (`experimental.rs`: `--experimental` indicators and their `ExperimentalIndicators` result block, #916)
│   ├── mod.rs
│   ├── validation.rs    # Format-independent page/batch contracts and capture-gap policy
│   ├── baseline.rs      # Findings baseline (#930): file format, digest, known/new/fixed classification and the gate inputs the verdict reads; `BatchReport::with_baseline`
│   ├── collection.rs    # Collection contract: Collected<T> states, per-page CollectionLog, COLLECTORS inventory
│   ├── dark_scheme.rs   # Second pass of the shared `contrast/*` rules under `prefers-color-scheme: dark` emulation when the page declares a dark scheme; findings keep the rule id and carry `color_scheme: dark` evidence (#917)
│   ├── capture_collectors.rs # Capture-layer collectors (navigation, AX tree, consent, exclusion, frames, stability, rule execution, journeys) projected onto the contract
│   ├── pipeline/        # Single page audit flow
│   │   ├── mod.rs       # PipelineConfig (`#[non_exhaustive]`, `Default` = the default CLI run, `with_*` setters), run_single_audit/audit_page (desktop + mobile pass), rule runs, artifact persistence
│   │   ├── assembly.rs  # Sync post-processing: WCAG merge, viewport scores, report aggregation, module runs, audit quality
│   │   └── throttled.rs # Throttled performance passes (single URL), tab recovery, canonical LhMobile adoption
│   ├── batch.rs         # Sitemap/batch processing
│   ├── access_block.rs  # Bot walls/access denials: blocked pages fail instead of being scored
│   ├── module.rs        # AuditModule trait + AuditCatalog registry (topo-sorted)
│   ├── catalog.rs       # Module registration/wiring
│   ├── report.rs        # AuditReport structure (raw data)
│   ├── normalized.rs    # NormalizedReport (enriched, score-corrected); re-exports the two modules below
│   ├── interactive_finding.rs # Journey/trace types, InteractiveFinding + interactive_finding_text (#406 en/de)
│   ├── finding_contract.rs # One finding contract for every domain: domain adapters, cause/scoring responsibility, finding inventory, indexability expectation
│   ├── finding_text.rs  # MessageText: a finding message as a function of the language (module `*_text(kind, values, en)` bound to kind + raw values); `localized_report` / `localized_context` return the run-language copy of a report the PDF builders read (shared-rule texts, then `localize_domain_texts` for non-WCAG findings; #898, #406, #937)
│   ├── finding_titles.rs # German titles of non-WCAG findings by rule_id (JSON keeps the English check title)
│   ├── finding_derive.rs # Derived finding fields: priority score, assurance (confidence/verification from detection method + evidence quality), complexity/expected-impact kinds + texts, BFSG relevance
│   ├── signal_contract.rs # Versioned contracts of the heuristic signals (target, method, applicability, evidence, uncertainty, permitted interpretation, validation status); see docs/calibration.md
│   ├── module_set.rs    # `Module` and `ModuleSet`: the modules a run selects next to the accessibility checks (`PipelineConfig.modules`, #933)
│   ├── module_scores.rs # Per-module score entries (build_module_scores, UX/Journey a11y penalties)
│   ├── risk_assessment.rs # RiskAssessment of the normalized report (level, thresholds, legal/blocking counts)
│   ├── scoring.rs       # Score calculation
│   ├── interpretation.rs # Pre-computed DE/EN interpretation texts
│   ├── summary.rs       # Cross-page aggregation logic
│   ├── template_dedup.rs # Template-level root-cause deduplication (batch)
│   ├── occurrence_analysis.rs # Occurrence counting across categories
│   ├── prioritization.rs # Fix-guidance prioritization
│   ├── verdict.rs       # Decision contract: versioned profile, the only verdict policy (single + batch); stored in NormalizedReport/BatchSummary
│   ├── management_risk.rs # Risk dimensions shared by the JSON and PDF outputs
│   ├── batch_consistency.rs # Cross-page consistency checks (incl. WCAG 3.2.6 help comparison)
│   ├── budget.rs        # Performance budgets: `BudgetConfig` (the `[budgets]` section of auditmysite.toml) and its evaluation against a report
│   ├── crawl.rs         # Link crawling (html5ever-based)
│   ├── duplicate.rs     # Duplicate-page detection
│   ├── exclusion.rs     # Audit exclusions (--exclude-selector, [data-audit-exclude]) and their report block
│   ├── frames.rs        # Element-level WCAG rules inside same-process iframes and the frame coverage block (#715)
│   ├── artifacts.rs     # Cache artifact persistence (--reuse-cache); replayable snapshot (collected report, versions), `replay`
│   └── performance_interpretation.rs # Performance-specific interpretation texts
│
├── output/              # Report generation
│   ├── mod.rs
│   ├── terminal.rs      # Terminal table output; the renderers take `TerminalOptions` (report level, language, `ColorPolicy`, file target), which `cli` builds from `Args`; the `runemark` colour mode stays internal (#934)
│   ├── json.rs          # JSON reports (via NormalizedReport)
│   ├── json/            # JSON detail/helper builders
│   ├── sr_audit_json.rs # Standalone screen-reader audit JSON sidecar
│   ├── technician.rs    # index.json + findings.jsonl next to per-page JSON reports
│   ├── sarif.rs         # SARIF output format
│   ├── builder/         # AuditReport → PDF ViewModel transformation
│   │   ├── mod.rs, actions.rs, modules.rs, helpers.rs, batch.rs
│   │   ├── finding_group.rs # The one `report_model::FindingGroup` builder, shared by single and batch reports (#937); the batch caller only adds `affected_urls` to a cross-page aggregate finding
│   │   └── single/      # Single-report builder (findings, etc.)
│   │       └── module_details.rs + module_details/ # Per-module detail presentations,
│   │                        # one file per module (mirrors pdf/detail_modules/)
│   ├── pdf/             # PDF reports (via renderreport/Typst)
│   │   ├── mod.rs, single_report.rs, batch_report.rs
│   │   ├── batch_report/ # Batch PDF sections: overview (cover, status, overview grid),
│   │   │                 # management, portfolio, interactive, actions, seo,
│   │   │                 # consistency, appendix (crawl links, commerce)
│   │   ├── cover.rs, findings.rs, wcag_coverage.rs, en301549.rs
│   │   ├── diagnosis.rs, appendix.rs, design.rs (4-color law)
│   │   ├── problem_profile.rs # Problem concentration/breadth diagnosis
│   │   ├── score_drivers.rs   # Score-driver table, score derivation, subcategory breakdown
│   │   ├── root_causes.rs     # Root-cause analysis (A, B, C…) + timeframe roadmap
│   │   ├── sanitize.rs  # Drops glyphs no report font can display (PDF/UA)
│   │   └── detail_modules/ # Per-module chapter renderers
│   ├── report_model.rs  # ViewModel structs for PDF
│   ├── explanations.rs  # Explanation table (EXPLANATIONS: owner role, effort, code examples per entry key), the explicit table `RULE_ENTRIES` (which entry each taxonomy rule id uses) and the lookup; the prose comes from `locales/<lang>/rules.ftl` via `RuleExplanation::customer_title`/`customer_description`/`user_impact`/`typical_cause`/`recommendation`/`technical_note(en)`, key = `explanation_text_key(key)`: the rule's message for an entry keyed by a rule id, `explanation-<key>` for an entry keyed by a WCAG criterion or an axe id (#935)
│   ├── search_experience.rs # Search/AI-visibility presentation helpers
│   ├── localized.rs     # Locale-aware text helpers
│   ├── renderer.rs      # Top-level format dispatch
│   └── snapshot_export.rs # AXTree + journey traces → YAML (`auditmysite-dev --export-snapshot`)
│
├── a11y_journey/        # Accessibility Journey Layer (--interactive): phase 2 of detect → exercise → primitives, drives the candidates of `patterns` with the primitives of `interaction`
│   ├── mod.rs           # Orchestrator: run() single entry point, commerce gating
│   ├── tab_walk.rs      # Tab-order recording + focus evaluation
│   ├── skip_link.rs     # Skip-link activation journey
│   ├── disclosure_journey.rs # Accordion/disclosure toggle
│   ├── modal_journey.rs # Modal focus-trap verification
│   ├── tabs_journey.rs  # TabList navigation
│   ├── menu_journey.rs  # DisclosureMenu journey
│   ├── form_error.rs    # Form-error announcement (per-form clustering, multi-form pages)
│   ├── spa_navigation.rs # SPA single-page navigation detection
│   ├── link_inventory.rs # Linktext/heading/landmark inventory (pure AXTree)
│   ├── add_to_cart.rs   # Add-to-cart status/feedback journey (commerce-gated, PDP only)
│   ├── quantity_stepper.rs # Quantity-stepper keyboard operability (commerce-gated, PDP only)
│   └── evaluate.rs      # Tab-walk finding evaluation
│
├── screen_reader/       # Screen-reader reading-order primitives
│   ├── mod.rs
│   ├── types.rs         # ReadingItem and ignored-node diagnostics
│   ├── announcer.rs     # Localized screen-reader announcement strings
│   ├── navigator.rs     # Virtual SR navigation lists
│   ├── analyzer.rs      # SR-specific issue detection
│   └── linearizer.rs    # AXTree DFS reading-order linearization
│
├── best_practices/      # Console errors and vulnerable JS library detection
│   ├── mod.rs, module.rs
│   ├── console_errors.rs # CDP-based console error/warning collection
│   ├── vulnerable_libs.rs # Known-CVE JS library detection (jQuery, Bootstrap, ...)
│   └── advisories.json  # Versioned advisory data read by vulnerable_libs (validated on load)
│
├── performance/         # Core Web Vitals, render-blocking, content weight
│   ├── mod.rs, vitals.rs
│   ├── animations.rs    # Non-composited animation detection
│   ├── coverage.rs      # Unused JS/CSS detection via CDP Coverage API
│   ├── critical_chain.rs # Critical request chain analysis
│   ├── minification.rs  # Unminified JS/CSS asset detection
│   └── third_party.rs   # Third-party resource attribution per origin
│
├── seo/                 # Meta, headings, schema, social, technical SEO
│   ├── mod.rs, module.rs
│   ├── meta.rs          # Meta tags (the finding texts: `seo-meta-*` in `locales/<lang>/seo-issues.ftl`)
│   ├── headings.rs      # Heading structure
│   ├── profile.rs       # SEO content profile
│   ├── schema.rs        # Structured data (JSON-LD/schema.org; the issue texts and table labels: `seo-schema-*` in `locales/<lang>/seo-issues.ftl`; `label_texts_export.rs` beside it is the ignored text-export test of these labels and of the robots.txt labels)
│   ├── schema_rules.rs  # Feature-specific required/recommended property rules
│   ├── schema_fit.rs    # Page-intent and URL based primary-schema fit
│   ├── schema_parity.rs # Conservative visible-content vs. JSON-LD comparison
│   ├── social.rs        # Open Graph / Twitter Card
│   ├── technical.rs     # robots/canonical/hreflang (the issue texts: `seo-technical-*` in `locales/<lang>/seo-issues.ftl`, #936; `issue_texts_export.rs` and `issue_texts_guard.rs` beside it are the ignored text-export test and the guards of that file)
│   ├── robots.rs        # robots.txt audit (bot-class and policy labels: `seo-robots-*` in `locales/<lang>/seo-issues.ftl`, #936)
│   ├── page_health.rs   # Page-health types, analyze_page_health, URL analysis
│   ├── page_health/     # dom.rs (DOM inspection; HTML validation lives in html_conform), probes.rs
│   │                    # (HTTP probes), issues.rs (collect_issues: thresholds, severities and arguments; the texts are the `page-health-*` messages of `locales/<lang>/page-health.ftl`, #936), text_export.rs (ignored test: exports every issue text in both languages) and text_guard.rs (guards of page-health.ftl)
│   ├── image_efficiency.rs # Image format and resolution analysis
│   └── ...
│
├── security/            # Security header analysis
│   ├── mod.rs, module.rs  # finding texts: `security-*` in `locales/<lang>/security-issues.ftl` (#936)
│   └── issue_texts_export.rs, issue_texts_guard.rs  # ignored text-export test and guards of security-issues.ftl
│
├── mobile/              # Mobile friendliness analysis
│   ├── mod.rs, module.rs  # finding texts and touch-target context labels: `mobile-*` in `locales/<lang>/mobile-issues.ftl` (#936)
│   ├── issue_texts_export.rs, issue_texts_guard.rs  # ignored text-export test and guards of mobile-issues.ftl
│   └── ux_heuristics.rs
│
├── html_conform/        # HTML5 spec-conformance checking via the html-conform crate (scored, part of --full)
│   ├── recommendation.rs # written fix hint per checker rule family (action plan, key measures)
│   ├── message.rs        # run-language text of checker messages (mapped by rule id, family title as fallback)
│   └── mod.rs, module.rs
│
├── privacy/             # Privacy indicators (tracking #875): technical findings, no score, no legal verdict
│   ├── first_visit.rs   # First-visit capture in a clean browser context: requests, cookies, storage keys, consent/legal-link traces of the document (raw, EN, no values); collector `privacy.first_visit`
│   ├── module.rs        # Catalog entry (`PrivacyModule`, enabled by `--privacy-check`); the capture itself runs in `pipeline::run_extra_page_checks`
│   ├── findings.rs      # `analyze_privacy` (#871): six checks, confidence classes, `not_measured`; classification via web-checks `third_party`; `finding_text(kind, .., en)` is the only text source. Runs from `run_extra_page_checks` (needs the BrowserManager), result in `experience.privacy`, JSON `detail.modules.privacy`
│   └── aggregate.rs     # `aggregate_privacy` (#872): batch roll-up (indicators/providers by affected pages, check distribution, per-page matrix); pure data, no text. PDF: single chapter `output/pdf/detail_modules/privacy.rs`, batch section `output/pdf/batch_report/privacy.rs`
│
├── ux/                  # UX analysis (5 dimensions, saturation curves)
│   ├── mod.rs, module.rs
│   ├── analysis.rs
│   └── scoring.rs
│
├── page_flow/           # AXTree user-flow heuristic (page intent, friction points; indicator `journey`, #938). Not `a11y_journey`
│   ├── mod.rs, module.rs
│   ├── analysis.rs
│   ├── page_intent.rs
│   ├── scoring.rs
│   └── js/early_text.js # Visible text in the first viewport (embedded script)
│
├── ai_visibility/       # AI/LLM discoverability analysis
│   ├── mod.rs, module.rs
│   ├── chunks.rs, citation.rs, knowledge_graph.rs, readability.rs
│
├── content_visibility/  # Cross-module signal aggregation (SEO+AI+Quality)
│   └── mod.rs, module.rs
│
├── commerce/            # Shop audit (derive-only, shop-gated): product schema-completeness,
│   └── mod.rs, module.rs # mandatory/trust-page links, page-kind (PDP/Category only), batch roll-up
│
├── source_quality/      # Technical preparation indicator (publisher/security signals, on-page consistency)
│   └── mod.rs, module.rs
│
├── tech_stack/          # CMS/framework detection from in-page signals
│   └── mod.rs, module.rs
│
├── patterns/            # UI pattern detection (nav, accordion, modal, ..., help-mechanism inventory): phase 1 (detect), imports neither `a11y_journey` nor `interaction`
│   ├── mod.rs           # PatternKind/JourneyKind registry, analyze()
│   ├── main_navigation.rs, accordion.rs, modal_dialog.rs
│   ├── disclosure_menu.rs, tab_list.rs, skip_link.rs
│   ├── form.rs          # Per-form clustering (ancestor_chain_from_root)
│   ├── add_to_cart.rs   # Add-to-cart button/feedback detection
│   └── quantity_stepper.rs # Quantity-stepper control detection
│
├── interaction/         # Browser interaction primitives (actions only, no evaluation): phase 3 (primitives), imports neither `patterns` nor `a11y_journey`
│   ├── pointer.rs       # CDP mouse events + synthetic click fallback
│   ├── keyboard.rs      # CDP key events
│   ├── focus.rs         # FocusSnapshot, incl. backend-node-id resolution (pierces shadow roots)
│   ├── live_regions.rs  # MutationObserver over aria-live regions — records events, not state
│   └── stability.rs     # Bounded DOM/app-ready settling before each capture
│
├── assessment/          # Shared assessment types and evidence model
│   └── mod.rs
│
├── studio/              # Studio contract types (GUI data contract)
│   └── mod.rs
│
└── i18n/                # Project Fluent (.ftl) loader, library default language: German (the CLI default is English); one bundle per language from `locales/<lang>/report.ftl` (report texts), `page-health.ftl` (the page-health issue texts of `seo::page_health`, #936), `seo-issues.ftl` (the technical, meta-tag and structured-data issue texts of `seo::technical`, `seo::meta` and `seo::schema`, and the structured-data and robots.txt table labels of `seo::schema` and `seo::robots`, #936), `security-issues.ftl` (the security finding texts of `security`, #936), `mobile-issues.ftl` (the mobile finding texts and touch-target context labels of `mobile`, #936) and `rules.ftl` (rule texts, one message per rule id with attributes, including the explanation texts of a rule with an entry under its own id, and explanation texts for the entries keyed by a WCAG criterion or an axe id, one message per entry with attributes, #935); `number.rs` = the one decimal formatter, `I18n::count` = plural-aware counts, `I18n::attr` = message attributes, `text_args` = a message with named arguments for the pure `*_text(…, en)` functions
    └── mod.rs

locales/                 # Fluent sources, compiled in with include_str!
├── de/                  # report.ftl, rules.ftl, page-health.ftl, seo-issues.ftl, security-issues.ftl, mobile-issues.ftl
└── en/                  # same key sets as de/ (guard tests in i18n, taxonomy::rules and output::explanations)
```

## Data Flow

### Single URL Audit

```
1. CLI parses arguments
   │
2. validate_url() checks for SSRF
   │
3. BrowserManager launches Chrome
   │
4. Navigate to URL, wait for load
   │
5. Extract AXTree via CDP
   │  └── GetFullAXTreeParams
   │
6. Run WCAG rules against AXTree
   │  ├── Level A rules
   │  ├── Level AA rules (if requested)
   │  └── Level AAA rules (if requested)
   │
6b. Run optional modules (`--full`, or individually enabled): performance, seo,
    security, mobile, ux, journey, ai-visibility, content-visibility,
    source-quality, tech-stack, best-practices, commerce (shop-gated)
   │
6c. Run Accessibility Journey Layer (`--interactive off|basic|full`): tab-walk,
    skip-link, disclosure/modal/tablist/menu journeys, form-error announcement,
    SPA navigation, commerce-gated add-to-cart/quantity-stepper journeys
   │
7. Calculate raw score → AuditReport
   │  └── Preserve requested scope, module/subcheck runs, rule outcomes,
   │      navigation, consent state, environment and audit quality
   │
8. Normalize: AuditReport → NormalizedReport
   │  ├── Apply score corrections (e.g. 3.1.1 lang suppression)
   │  ├── Enrich findings with taxonomy fields
   │  ├── Normalize warnings/manual-review items separately from violations
   │  ├── Retain Journey execution coverage and compact focus evidence
   │  └── Compute grade from corrected score
   │
9. Generate report (JSON/PDF/Table)
   │  └── All formats use NormalizedReport as single source of truth
   │
10. Close browser
```

### Batch Audit (Sitemap)

```
1. Fetch and parse sitemap.xml
   │
2. Create BrowserPool with N pages
   │
3. For each URL (concurrent):
   │  ├── Acquire page from pool
   │  ├── Run single audit
   │  ├── Return page to pool
   │  └── Report progress
   │
4. Aggregate results
   │
5. Generate batch report
```

## Key Components

### NormalizedReport

Central data layer between raw `AuditReport` and output formats:
- Applies score corrections (e.g. 3.1.1 language suppression)
- Enriches violations with taxonomy metadata (dimension, subcategory, issue_class, etc.)
- Carries explicit audit quality, module/rule execution status, accessibility assessments, and Journey coverage
- Computes grade from corrected score
- Single source of truth for JSON, PDF, and CLI output — ensures consistent scores
- `NormalizedFinding` is built in `audit/` only (`normalized.rs`, `finding_contract.rs`). `output/` reads it and derives its view models (`report_model::FindingGroup`) from it, in one builder (`output/builder/finding_group.rs`) for the single and the batch report; `output_never_constructs_a_normalized_finding` in `tests/architecture_test.rs` fails on any struct literal under `src/output/` outside `#[cfg(test)]` items, and `output_never_takes_a_normalized_finding_by_mutable_reference` on any `&mut` to one. The run-language copy of a report for the German PDF is made in `audit/finding_text.rs`; `output/` only reads it (#937)

The overall score is reconstructed from contributing `module_scores` and their
published weights, for single- and dual-viewport runs. Unchecked HTML conformance
and incomplete performance measurements carry no weight. Accessibility caps are
applied after viewport aggregation as well as during individual scoring.

`audit::normalize_checked` is the checked domain boundary. It calls the existing
normalizer and validates the resulting `NormalizedReport` independently of any
output format. `NormalizedReport::validate` collects all detected contradictions
as serializable `ContractViolation` entries (stable code, path, explanation).
It checks score limits, canonical grades, finding and occurrence
counts, module identity/weights/measurement state, the reconstructed overall
score, requested module outcomes and execution completeness. `validate_batch`
also verifies the measured-page denominator, accessibility average and canonical
pass/fail counts. These checks validate consistency; they do not certify that a
rule's underlying method is correct.

Live `audit_page` results cross this boundary before returning to callers. The
single-page wrapper refreshes audit quality and checks again after optional
post-processing, before persistence. CLI dispatch checks every output mode;
direct PDF/Typst, compact summary, SARIF and cached JSON entry points also check
their inputs. The existing infallible `normalize` and view-model builders remain
available for compatibility; library callers choosing them must explicitly call
`validate` before consuming constructed or mutated data. Studio callers using
the live pipeline inherit its validation; constructing a Studio response by
hand does not itself validate the supplied pre-rendered JSON.

The capture-gap policy distinguishes an intentional exclusion or hidden frame
from a visible frame that could not be inspected. The latter, stability
fallbacks and exhausted stability budgets qualify the audit. A partial or
insufficient result is valid when honestly marked; contradictions are rejected
before presentation. The JSON linter below separately checks the serialized
projection, so mistakes introduced during export are also detected.

`UnifiedReport::to_json` runs report lint against the serialized facts before
emitting JSON and rejects high/critical contradictions. `lint/invariants.rs`
independently reconstructs the weighted score, requires an outcome for requested
modules, checks audit completeness against failures, and accounts for rule
findings through findings, assessments or exclusions. Collector decoding in
`util.rs` distinguishes technical failure from a measured empty collection.

The strict `release_` tests in the WCAG and non-WCAG corpus completeness binaries
require both positive and negative evidence for every inventoried check.
`scripts/release-check.sh` and the release-tag path of `ci.yml` run these gates;
the regular gap reports remain diagnostic. A deferred check is a visible release
gap, not verification. Passing these inventory checks does not prove manual
WCAG conformance or the correctness of every heuristic.

For version 2.x, `scripts/check-v2-readiness.py` also checks the live GitHub
milestone against `scripts/v2-readiness.json`. All 26 audit requirements and
tracker [#792](https://github.com/casoon/auditmysite/issues/792) must be closed as
completed in milestone 2; additional open milestone issues also block release.
Missing/moved issues, invalid responses and network errors fail closed. The
release workflow runs this in `verify-version`, which is a prerequisite of every
artifact build. Offline regression tests exercise each blocker independently.

### Foundation contracts (state as of 2.0)

The validation boundary is the consistency gate; the contracts behind it are in
place. One directed processing chain inside the application, no service split and
no additional score model. Details per contract live in `docs/OUTPUT_CONTRACT.md`.

- **Collection contract** (`audit/collection.rs`): every collector is declared once
  in `COLLECTORS` and reports `measured`, `empty`, `partial`, `failed`,
  `unsupported` or `not_applicable` in `module_runs[].subchecks[]`. Missing data
  never becomes an empty success. *Side checks* (`SIDE_CHECKS`: the other host
  variant and the TLS certificate details) are the one deliberate exception to
  "a lost collection makes its module partial": their loss is a gap of that single
  subcheck and an audit caveat, not a qualification of the audited page.
- **Finding and decision model** (`audit/finding_contract.rs`,
  `audit/verdict.rs`): findings carry stable identity, applicability, evidence and
  scoring effect; verdict and batch counts are computed once during
  normalization and read by every output.
- **Measurement profile** (`audit_scope`): document/redirect, sample, viewport,
  session, consent and interaction state are recorded; unsupported flag
  combinations fail explicitly; a batch records its denominator and selection.
- **Replay** (`versions` in the JSON): evidence snapshot plus rule/scoring versions
  reproduce the same domain model without a live browser; fresh, replay, cached
  and Studio paths are parity-tested.
- **Executable rule contracts** (`tests/fixtures/rule_contracts/`,
  `rule_contract_gate`): every active rule id needs positive, negative, N/A and
  error fixtures; a rule without a contract fails verification.
- **Rule documentation pages** (#929): one page per WCAG rule, security header
  issue type, structured-data feature and checked library, generated from the
  taxonomy, the contracts and the corpus (`tests/common/rule_docs.rs`), committed as
  JSON under `site/src/data/rules/` and rendered by `site/src/pages/rules/`.
  `audit/rule_docs.rs` maps a finding's rule id to its page (`help_url`).
  `tests/rule_docs.rs` fails when they drift from their sources or a `help_url`
  has no page; `UPDATE_RULE_DOCS=1 cargo test --test rule_docs` regenerates them.
- **Checked boundary:** `normalize_checked` is the supported entry;
  `audit::normalize` is a deprecated compatibility API for callers outside the
  crate.

### BrowserManager

Handles Chrome lifecycle:
- Auto-detects Chrome/Chromium/Edge installations in a fixed order: per kind (Chrome, Edge,
  Ungoogled Chromium, Chromium) system install, user-level install, `PATH`; duplicates are
  listed once (canonical path). Managed installs come after every existing browser, full
  Chrome for Testing before the headless shell
- Platform contract (`browser::platform`): macOS ARM64, Linux x86_64/ARM64, Windows x86_64 with
  Microsoft Edge only (also for `--browser-path`/env vars); no managed browser on Windows
- An automatically selected browser that fails to start hands over to the next viable candidate
  (`ResolvedBrowser::launch_chain`), keeping one diagnosis per attempt; an explicit path never
  falls back unless `BrowserResolveOptions::fallback_to_auto` is set
- `browser detect` and `doctor` run `diagnose_browsers`: version query and headless CDP launch are
  time-bounded; every candidate shows its verdict (selected / alternative / reason)
- Managed Chrome for Testing only on request: `auditmysite browser install`, `browser ensure --yes`, or after the consent prompt of an interactive audit on macOS ARM64 / Linux x86_64 / Linux ARM64 (never in non-interactive, CI or Windows runs; Windows uses Edge only). Transactional and unauthenticated-by-upstream: see [chrome-dependency.md](chrome-dependency.md#2-install-chrome-srcbrowserinstallerrs)
- Configures headless mode with security flags
- Manages page navigation with timeouts

### BrowserPool

For concurrent audits:
- Pre-creates N pages
- Reuses pages between audits (reset to about:blank)
- Handles page failures gracefully
- Timeout protection on page reset

### AXTree Extractor

Uses CDP `Accessibility.getFullAXTree`:
- Converts CDP response to internal AXNode structure
- Preserves parent/child relationships
- Extracts properties (role, name, focusable, etc.)

### Interaction phases: detect, exercise, primitives (#938)

Three modules, one per phase, not one per UI concept:

| Phase | Module | Needs a browser | Imports |
| --- | --- | --- | --- |
| Detect | `patterns` | `analyze()` no (AXTree only); the help-mechanism and easy-language inventories read the page once | neither of the others |
| Exercise | `a11y_journey` | yes, drives the candidates and judges the state change | `patterns`, `interaction` |
| Primitives | `interaction` | yes, CDP pointer/keyboard/focus/live-region/settling calls, no evaluation | neither of the others |

A UI concept (modal, tabs, menu, accordion, skip link, form, add-to-cart,
quantity stepper) appears once per phase: `patterns/modal_dialog.rs` detects
the dialog, `a11y_journey/modal_journey.rs` operates it. The phases change for
different reasons (what counts as a pattern; what counts as a failure; how
Chrome is driven), and `a11y_journey` is the only one of the three that knows
about findings and journey budgets. A split per concept would put all three
reasons into every module. The direction is enforced by
`tests/architecture_test.rs` (`patterns_interaction_a11y_journey_depend_downwards_only`).

`page_flow` (until #938 `journey`) is unrelated to these three: a static
AXTree heuristic (page intent, friction points) behind the experimental
`journey` indicator. The indicator name, JSON key and rule ids kept the word;
only the Rust module path changed. `screen_reader::bfsg`, a thin wrapper over
`wcag::en301549`, is gone: `screen_reader` reads `EN301549_WEB_CLAUSES`
directly.

### Journey Layer: snapshot, act, snapshot, diff

Interactive journeys do not probe the page with JavaScript. They take an
`AXSnapshot` (tree + focus + URL + title), perform the action, take another,
and judge the `AXTreeDiff` between them.

Nodes are matched across snapshots by **backend DOM node id**, not by AXTree
node id — Chrome re-assigns `nodeId` on every `getFullAXTree` call. That id is
also what ties a state change to the element the journey actually operated:
`diff.property_change_for(trigger, "expanded")` answers "the clicked trigger
changed", not "something on the page changed".

Reading the accessibility tree instead of the DOM removes three blind spots
that page-side `document.querySelector` probes have, all three measured on
real pages:

- **Shadow DOM** — `querySelectorAll` does not pierce shadow roots;
  `getFullAXTree` does.
- **iframes** — same, for document boundaries.
- **Attribute vs. role** — `[role="dialog"]` matches the attribute. A native
  `<dialog>` carries the role without the attribute.

Two tree helpers carry the journeys: `AXTree::node_by_backend_id` and
`AXTree::is_within`, which answers "is focus inside this dialog?" along the
tree's parent chain rather than via `element.contains()`.

A capture failure yields no finding and records its cause in the trace. Silent
passing on a read error is the one outcome this layer must not produce.

**Two snapshots are not enough for everything.** A live region that is filled,
announced and emptied again can be invisible between two samples — the normal
case for form errors and add-to-cart feedback. `interaction::live_regions`
covers that with a `MutationObserver` that records *that* a live region
received content, with timestamp and politeness. It also observes existing
shadow roots and replaces `Element.prototype.attachShadow` to catch later ones.
Observed is not announced: the browser having cause to announce is not proof a
screen reader read it out.

Journeys on this footing: `disclosure`, `modal`, `menu`, `tabs`,
`spa_navigation`, `form_error`, `add_to_cart`. `tab_walk` and `link_inventory`
are not diff cases.

### WCAG Engine

Orchestrates rule checking:
- Filters rules by WCAG level (A, AA, AAA)
- Runs each rule against relevant nodes
- Collects violations with severity
- Counts passes for scoring

### Rule Structure

Each rule implements checking logic:

```rust
pub fn check(tree: &AXTree, _styles: &[NodeStyle], level: WcagLevel) -> Vec<Violation> {
    let mut violations = Vec::new();

    for node in tree.iter() {
        if should_check(node) && has_violation(node) {
            violations.push(Violation::new(
                "1.1.1",
                "Non-text Content",
                level,
                Severity::High,
                "Image missing alt text",
                &node.node_id,
            ));
        }
    }

    violations
}
```

### Taxonomy

Maps WCAG rule IDs to rich metadata:
- **Dimension**: Barrierefreiheit, Usability, etc.
- **Subcategory**: Inhalte & Alternativen, Navigation, etc.
- **IssueClass**: Fehlend, Falsch, Unvollständig
- **ScoreImpact**: Base penalty, max penalty, scaling curve
- **ReportVisibility**: Which report levels (Executive/Standard/Technical) include each finding

## Security Considerations

### SSRF Protection

`validate_url()` blocks:
- Private IPs (10.x, 172.16-31.x, 192.168.x)
- Localhost (127.x, ::1)
- Link-local (169.254.x)
- Non-HTTP schemes

### Path Traversal

`read_url_file()` uses `canonicalize()` to prevent `../` attacks.

### Managed Browser Downloads

- Nothing is downloaded without consent: `auditmysite browser install`, `browser ensure --yes`, or the consent prompt of an interactive audit
- The version and the per-platform URL come from the publisher's Chrome for Testing manifest; there is no hard-coded fallback version
- HTTPS only, and only to `googlechromelabs.github.io` (manifest) and `storage.googleapis.com/chrome-for-testing-public/` (archive), including redirects
- No checksum or signature is published upstream, so publisher integrity is not claimed; details in [chrome-dependency.md](chrome-dependency.md)

## Testing Strategy

```
tests/
├── output_format_tests.rs        # Report generation
├── error_handling_tests.rs       # Error paths
├── parity_contract.rs            # Frozen-count guard vs. docs/PARITY_CONTRACT.jsonc
├── act_mapping.rs                # Rule ids in docs/act-mapping.json (ACT mapping) exist
├── rule_docs.rs                  # Generated per-rule pages (site/src/data/rules/) match catalogue, contracts, corpus (#929)
├── act_testcases.rs              # ACT test case harness (#837): parser/judging unit tests; `#[ignore]` Chrome run via scripts/act-fetch.sh + tests/common/act_harness.rs
├── wcag_coverage.rs               # Level A/AA/AAA rule-set coverage checks
├── wcag_unit_tests.rs             # Per-rule unit tests (fixtures in wcag_fixtures/)
├── integration_test.rs / integration_mocked.rs # End-to-end pipeline tests
├── report_consistency_tests.rs   # Score/finding consistency across formats
├── batch_consistency (see src/audit/batch_consistency.rs)
├── release_contract_tests.rs     # Release-gate invariants
├── golden_render.rs              # Golden-render harness (ignored; scripts/golden-diff.sh)
└── snapshot_tests.rs              # insta snapshot tests (snapshots/)

src/*/tests (inline)               # Unit tests per module
```

Run all tests:
```bash
cargo test
```

Golden-render harness (`tests/golden_render.rs`, `#[ignore]`): renders single JSON, single
Typst (de/en), batch JSON and batch Typst (de/en) from a frozen set of cached audits
(`reports/golden/<site>/`, gitignored) through the same library calls the CLI uses on a cache
hit. Refactoring PRs render on `main` and on the branch and require an empty diff:
```bash
git switch main      && scripts/golden-diff.sh /tmp/golden-main
git switch <branch>  && scripts/golden-diff.sh /tmp/golden-branch /tmp/golden-main
```

## Performance

- Single audit: ~2-5 seconds (depends on page complexity)
- Batch audit: Linear scaling with concurrency
- Memory: ~100MB per Chrome page

## Dependencies

Key crates:
- `chromiumoxide` - CDP client
- `tokio` - Async runtime
- `clap` - CLI parsing
- `serde` / `serde_json` - Serialization
- `tracing` - Logging
- `html5ever` / `markup5ever_rcdom` - Local HTML5 parsing (link crawling, no remote validator)
- `html-conform` - Deep HTML5 spec-conformance checking (RelaxNG + Schematron) for the html_conform module
- `fluent-bundle` / `unic-langid` - i18n (Project Fluent, default language German)
- `reqwest` (rustls) - Sitemap fetching, managed browser download
- `renderreport` (optional, `pdf` feature) - PDF generation (Typst-based)
