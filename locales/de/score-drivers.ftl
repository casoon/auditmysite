# Texts of the score drivers of the single PDF's management summary
# (`output::pdf::score_drivers`, `score-drivers-*`), #936: the combined
# technical score card, the score-driver table, the weight-basis note, the
# viewport-weighted score derivation, the additional score drivers and the
# per-module subcategory breakdown. Which message applies (weakest module,
# corrected band, renormalised basis), the module lists and the formula
# cells stay in Rust. The module column heading is `label-module` of
# report.ftl.

## Combined technical score card
score-drivers-card-title = Kombinierter technischer Wert · 0–100
score-drivers-card-description = Gewichtet über sechs Module, Gewichtsbasis { $basis }
score-drivers-card-note = Eine andere Frage als der Barrierefreiheitswert auf dem Deckblatt, um den es in diesem Bericht geht.

## Score-driver table
score-drivers-table-title-derivation = Wie sich der kombinierte technische Wert zusammensetzt
score-drivers-table-title-weighted = Die gewichteten Module im Einzelnen
score-drivers-col-weight = Gewichtung
score-drivers-col-assessment = Einordnung
score-drivers-class-weakest = Größter Schwachpunkt
score-drivers-class-risk-driver = Erheblicher Risikotreiber
score-drivers-class-band-corrected = Offene schwere Befunde begrenzen die Einordnung
score-drivers-class-stabilizing = Stabilisiert den kombinierten Wert

## Weight basis
score-drivers-basis-title = Gewichtsbasis
score-drivers-basis-note = Der kombinierte technische Wert ist auf die gemessenen Module normiert — Gewichtsbasis { $basis } von 100, nicht der volle Satz.
score-drivers-basis-missing = Nicht enthalten: { $modules }.
score-drivers-basis-not-comparable = Ein Wert aus einem anderen Modulsatz ist damit nicht direkt vergleichbar.

## Viewport-weighted score derivation
score-drivers-derivation-title = Wie der kombinierte technische Wert zustande kommt
score-drivers-derivation-col-step = Schritt
score-drivers-derivation-col-input = Eingang
score-drivers-derivation-col-result = Ergebnis
score-drivers-derivation-desktop = Desktop-Wert: Barrierefreiheit und Performance, Modulgewichte neu normiert
score-drivers-derivation-desktop-input = auf Desktop gemessen
score-drivers-derivation-mobile = Mobile-Wert: alle gewichteten Module außer Sicherheit
score-drivers-derivation-mobile-input = Modulgewichte oben, auf 90 bezogen
score-drivers-derivation-blend = Viewport-Mischung (Mobile zählt mehr)
score-drivers-derivation-security = Sicherheit eingemischt
score-drivers-derivation-no-security = Sicherheit nicht gemessen — die Mischung trägt voll
score-drivers-derivation-note = Die Modulgewichte oben zeigen, was wie stark zählt; diese Tabelle ist die Rechnung, aus der der kombinierte technische Wert entsteht.

## Additional score drivers
score-drivers-additional-title = Weitere Score-Treiber

## Subcategory breakdown
score-drivers-subcategory-weak = Kritische Schwächen: { $items }
score-drivers-subcategory-partial = Weitere Teilbereiche der { $title }: { $items }
score-drivers-subcategory-clean = Ohne zugeordnete Befunde (100): { $items }. Das heißt nur, dass dort nichts gezählt wurde — nicht, dass der Bereich geprüft und mängelfrei ist.
score-drivers-subcategory-critical = Kritische Befunde und der Teilbereich, in dem sie zählen: { $items }
