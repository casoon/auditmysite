# Texts of the single report's findings overview and system diagnosis
# (`output::builder::single::diagnosis`) and of the diagnosis section of the
# single PDF (`output::pdf::diagnosis`), #936. The builders keep which
# message applies to an issue pattern, a criticality tier and a finding
# category. The severity labels reuse `severity-*` of report.ftl, the finding
# count `count-findings`, the table headings the `diagnosis-col-*` and
# `diagnosis-table-*` messages of report.ftl.

## Issue pattern (findings overview, dominant-issue spotlight)
diagnosis-pattern-minimal = Keine Befunde
diagnosis-pattern-single-dominant = Einzelnes dominantes Problem
diagnosis-pattern-clustered = Geclusterte Probleme
diagnosis-pattern-scattered = Verteilte Einzelprobleme

## System diagnosis
diagnosis-section-title = Systemdiagnose
diagnosis-pattern-single-dominant-problem = Einzelnes dominantes Problem
diagnosis-pattern-distributed = Verteilte Einzelprobleme
diagnosis-pattern-minimal-description = Keine messbaren Barrierefreiheits-Barrieren erkannt.
diagnosis-pattern-single-dominant-description = Ein Regeltyp verursacht den Großteil aller kritischen/hohen Findings. Die Behebung dieser einen Ursache hat den größten Einzeleffekt.
diagnosis-pattern-clustered-description = Probleme konzentrieren sich in zusammenhängenden Bereichen. Die Behebung eines Clusters reduziert mehrere Findings gleichzeitig.
diagnosis-pattern-scattered-description = Probleme verteilen sich über viele unabhängige Regeln. Keine einzelne Ursache dominiert — jedes Finding erfordert individuelle Aufmerksamkeit.
diagnosis-dominant-spotlight = "{ $issue }" verursacht den Großteil der kritischen/hohen Findings.

## Criticality tiers
diagnosis-tier-mandatory = Gesetzlich relevante Barrieren (WCAG A/AA)
diagnosis-tier-mandatory-eyebrow = BEREICH 1 · WCAG-KONFORMITÄT
diagnosis-tier-mandatory-intro = Barrierefreiheits-Verstöße nach WCAG Level A/AA. Diese Befunde beschreiben Barrieren, deren Behebung für die Erfüllung gesetzlicher Vorgaben (wie dem BFSG) empfohlen ist.
diagnosis-tier-optimization = Ergänzende Qualitäts- & SEO-Empfehlungen
diagnosis-tier-optimization-eyebrow = BEREICH 2 · OPTIMIERUNG
diagnosis-tier-optimization-intro = Suchmaschinenoptimierung (SEO), KI-Sichtbarkeit, UX-Signale und weitere Qualitätskriterien. Diese verbessern die Auffindbarkeit und das allgemeine Nutzererlebnis.

## Thematic clusters ($findings: the finding count, `count-findings`)
diagnosis-cluster-accessibility = Barrierefreiheits-Barrieren ({ $findings })
diagnosis-cluster-seo = SEO-Probleme ({ $findings })
diagnosis-cluster-performance = Performance-Probleme ({ $findings })
diagnosis-cluster-security = Sicherheitslücken ({ $findings })
diagnosis-cluster-mobile = Mobile-Probleme ({ $findings })
diagnosis-cluster-other = { $category } ({ $findings })
