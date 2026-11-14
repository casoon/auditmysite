# Texts of the single report's action plan block
# (`output::builder::single::actions_block`, `actions-block-*`), #936: the
# systemic and local column labels and descriptions, the block title and intro
# for a good site and for the default case, and the risk effect of an action
# per priority. Which message applies and the thresholds stay in Rust. The
# remediation leverage tiers are `pdf-findings-code-very-high`,
# `severity-high` and `severity-low`; the role, priority, execution priority
# and effort labels are messages of their own enums.

## Roadmap columns
actions-block-systemic-label = Systemische Maßnahmen
# Reworded from an assertion into a hypothesis: this report audits exactly one
# page, so "affects all pages" is never observed; the verification task is part
# of the text (#575).
actions-block-systemic-desc = Einmal in Vorlage oder Komponente beheben — das Muster deutet darauf hin, dass auch andere Seiten mit dieser Komponente betroffen sind. Hinweis: auf weiteren Seitentypen gegenprüfen, ob dasselbe Muster dort ebenfalls auftritt.
actions-block-local-label = Lokale Maßnahmen & Einzelfälle
actions-block-local-desc = Punktuelle Korrekturen an einzelnen Seiten, Bildern oder redaktionellen Inhalten.

## Block title and intro
actions-block-title-good = Letzte Optimierungsschritte
actions-block-title-default = Maßnahmenplan nach Priorität
actions-block-intro-good = Die Seite ist technisch stark aufgestellt. Die folgenden Punkte sind letzte Optimierungshebel ohne strukturellen Druck.
actions-block-intro-default = Befunde der Priorität 1 tragen das höchste Risiko und sollten bevorzugt behandelt werden. Danach folgen Priorität 2 und 3. Priorität 4 sind ergänzende Empfehlungen.

## Risk effect per priority
actions-block-risk-critical = Reduziert kritisches WCAG-Verstoßrisiko direkt
actions-block-risk-high = Reduziert hohes Barrierefreiheitsrisiko
actions-block-risk-medium = Verringert mittleres Barrierefreiheitsrisiko
actions-block-risk-low = Verbessert WCAG-Konformität im Detail
