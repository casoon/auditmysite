# Roadmap 0.6.0

## Ziel

`0.6.0` soll `auditmysite` von einem starken Audit-CLI mit Sitemap-Support zu einem Tool weiterziehen, das Seiten auch selbst entdecken und SEO-/Site-Probleme domainweit belastbarer erkennen kann.

## Scope

### 1. Crawler + URL-Discovery ✅

Ziel:
- aus einer Seed-URL intern verlinkte Seiten selbst entdecken
- same-domain
- BFS mit Begrenzung

Ergebnis:
- `--crawl` + `--crawl-depth` CLI-Flags
- `crawl_site` / `CrawlResult` / `CrawlNode` mit `links_out` + `external_links_out`
- Basis für Link-Graph und Broken Links

### 2. Broken Links ✅

Ziel:
- interne und externe kaputte Links unterscheiden
- Redirect-Ketten erkennen

Ergebnis:
- `BrokenLink` mit `is_external`, `redirect_hops`, `severity` (High/Medium/Low)
- `CrawlDiagnostics` mit `broken_internal_links`, `broken_external_links`, `redirect_chains`
- `check_link`: folgt Redirects manuell bis 6 Hops, HEAD→GET-Fallback
- `RedirectChain` für Ketten > 1 Hop
- PDF: separate Tabellen intern/extern + Redirect-Ketten-Sektion
- Severity: intern 4xx → High, intern 5xx → Medium, extern 4xx → Medium, extern 5xx → Low

### 3. Duplicate / Near-Duplicate Content ✅

Ziel:
- doppelte und fast doppelte Seiten auf Domain-Ebene erkennen

Ergebnis:
- `src/audit/duplicate.rs`: SimHash (64-bit, 2-Wort-Shingles)
- Boilerplate-Filter (Zeilen < 4 Wörter werden entfernt)
- Schwellenwert 80 % Ähnlichkeit, min. 80 Wörter
- Unterscheidung „Duplikat" (≥ 95 %) vs. „Near-Duplicate" (80–94 %)
- Batch-PDF: Sektion „Near-Duplicate Content" mit Paar-Tabelle

### 4. Render Blocking + Asset-Größen ✅

Ziel:
- CSS/JS-Signale ergänzen, ohne Lighthouse zu spiegeln

Ergebnis:
- `src/performance/render_blocking.rs`: Erkennt `<head>`-Scripts ohne `defer`/`async` und blockierendes CSS
- First-Party vs. Third-Party Ressourcenaufteilung mit Origin-Zählung
- `ContentWeight` (vorhandene Implementierung) jetzt tatsächlich in der Pipeline aufgerufen
- Render-Blocking-Sektion im PDF-Performance-Kapitel
- Actionable Suggestions auf Deutsch

### 5. Performance Budgets ✅

Ziel:
- einfache Budget-Regeln für Requests, JS, CSS, Third-Party

Ergebnis:
- `[budgets]` Sektion in `auditmysite.toml` (10 konfigurierbare Limits)
- `src/audit/budget.rs`: `evaluate_budgets()` prüft LCP, FCP, CLS, TBT, JS-KB, CSS-KB, Seitengröße, Requests, Blocking-Scripts, Third-Party-KB
- Severity: Error (> 50 % überschritten), Warning (bis 50 %)
- Ausgabe in JSON, CLI-Tabelle und PDF-Sektion
- Inline-Summary nach Audit: „N Budget-Verletzungen: X Errors, Y Warnings"

### 6. Wettbewerbsvergleich ✅

Ziel:
- mehrere Domains in einem Vergleichslauf gegenüberstellen

Ergebnis:
- `--compare https://a.com https://b.com https://c.com` (2–10 Domains)
- `src/audit/comparison.rs`: `ComparisonReport` + `ComparisonEntry`
- PDF: Domain-Ranking via `BenchmarkTable`, Modul-Vergleich, Top-Findings je Domain
- JSON und CSV-Tabelle als weitere Ausgabeformate
- Hoher Produktwert für Agentur-/Beratungsfälle

## Nicht Teil von 0.6.0

- API-Layer
- LLM-/AI-Layer
- Kanban-/Team-Workflow

## Reihenfolge

1. ✅ Crawler + URL-Discovery
2. ✅ Broken Links
3. ✅ Duplicate / Near-Duplicate Content
4. ✅ Render Blocking + Asset-Größen
5. ✅ Performance Budgets
6. ✅ Wettbewerbsvergleich

## Release-Hinweis

Wenn Punkte aus dieser Roadmap umgesetzt werden, müssen README und Docs immer mitgezogen werden, damit der reale Funktionsstand dokumentiert bleibt.
