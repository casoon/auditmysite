---
paths:
  - "src/output/**/*.rs"
  - "src/audit/interpretation.rs"
  - "src/seo/interpretation.rs"
  - "locales/**/*.ftl"
---

## Report Wording Style
Gilt für alle Interpretations-/Erklärungstexte (`interpret_score_localized`, `seo_interpretation_text`,
Overall-Erklärung, Modul-Dashboard). Quelle: `src/audit/interpretation.rs` (`interpret_score_localized`),
`src/seo/interpretation.rs` (`seo_interpretation_text`). Review-Surface: `reports/interpretations.json` (regenerieren mit
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
