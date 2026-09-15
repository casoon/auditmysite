---
paths:
  - "tests/report_lint_tests.rs"
  - "tests/registry_contract.rs"
  - "tests/regression_corpus_contract.rs"
  - "src/output/pdf/tests.rs"
  - "src/lint/**/*.rs"
  - "src/registry/**/*.rs"
  - ".github/workflows/*.yml"
  - "scripts/release-check.sh"
---

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
  Differenzprüfungen** statt gespeicherter Pixel-Baselines erkannt, siehe Phase-5-Eintrag in `CHANGELOG.md`.
