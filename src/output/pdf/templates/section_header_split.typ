// Project-specific SectionHeaderSplit treatment.
// The eyebrow is intentionally light and widely tracked so it reads as a
// quiet orientation label rather than competing with the heading below.

#let section-header-split(data) = {
  block(width: 100%, breakable: false)[
    #if data.eyebrow != none [
      #text(
        size: font-size-xs,
        weight: "regular",
        fill: color-secondary,
        tracking: 0.20em,
        upper(data.eyebrow),
      )
      #v(spacing-3)
    ]

    // This is the complete eyebrow-to-heading gap. Suppressing the heading's
    // default top spacing keeps the pair predictable at every heading level.
    #show heading: set block(above: if data.eyebrow != none { 0pt } else { auto })
    #heading(level: data.level, outlined: data.outlined)[#data.title]

    #v(spacing-2)

    #par(justify: true)[
      #text(size: font-size-base, fill: color-text)[#data.body]
    ]

    #if data.divider_below [
      #v(spacing-3)
      #line(length: 100%, stroke: 0.5pt + rgb("#cbd5e1"))
      #v(spacing-2)
    ] else [
      #v(spacing-3)
    ]
  ]
}
