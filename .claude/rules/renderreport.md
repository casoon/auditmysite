---
paths:
  - "Cargo.toml"
  - "src/output/pdf/**/*.rs"
---

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
