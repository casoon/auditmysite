# Texts of the score drivers of the single PDF's management summary
# (`output::pdf::score_drivers`, `score-drivers-*`), #936: the combined
# technical score card, the score-driver table, the weight-basis note, the
# viewport-weighted score derivation, the additional score drivers and the
# per-module subcategory breakdown. Which message applies (weakest module,
# corrected band, renormalised basis), the module lists and the formula
# cells stay in Rust. The module column heading is `label-module` of
# report.ftl.

## Combined technical score card
score-drivers-card-title = Combined technical score · 0–100
score-drivers-card-description = Weighted across six modules, weight basis { $basis }
score-drivers-card-note = A different question from the accessibility score on the cover, which is what this report assesses.

## Score-driver table
score-drivers-table-title-derivation = Why the combined technical score is what it is
score-drivers-table-title-weighted = The modules that carry weight
score-drivers-col-weight = Weight
score-drivers-col-assessment = Assessment
score-drivers-class-weakest = Biggest weak point
score-drivers-class-risk-driver = Significant risk driver
score-drivers-class-band-corrected = Open severe findings limit the rating
score-drivers-class-stabilizing = Stabilizes the combined technical score

## Weight basis
score-drivers-basis-title = Weight basis
score-drivers-basis-note = The combined technical score is normalised over the modules that were measured — a weight basis of { $basis } of 100, not the full set.
score-drivers-basis-missing = Not included: { $modules }.
score-drivers-basis-not-comparable = A score from a different module set is not directly comparable.

## Viewport-weighted score derivation
score-drivers-derivation-title = How the combined technical score was calculated
score-drivers-derivation-col-step = Step
score-drivers-derivation-col-input = Input
score-drivers-derivation-col-result = Result
score-drivers-derivation-desktop = Desktop value: accessibility and performance, module weights renormalized
score-drivers-derivation-desktop-input = measured on desktop
score-drivers-derivation-mobile = Mobile value: all weighted modules except security
score-drivers-derivation-mobile-input = module weights above, over 90
score-drivers-derivation-blend = Viewport blend (mobile weighs more)
score-drivers-derivation-security = Security mixed in
score-drivers-derivation-no-security = Security not measured — blend carries the full weight
score-drivers-derivation-note = The module weights above rank what carries how much; this table is the calculation that produced the combined technical score.

## Additional score drivers
score-drivers-additional-title = Additional score drivers

## Subcategory breakdown
score-drivers-subcategory-weak = Critical weaknesses: { $items }
score-drivers-subcategory-partial = Other sub-areas within { $title }: { $items }
score-drivers-subcategory-clean = No findings assigned (100): { $items }. That says nothing was counted there, not that the area was checked and found free of defects.
score-drivers-subcategory-critical = Critical findings and the sub-area they count in: { $items }
