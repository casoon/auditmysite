// Project-specific MetricStrip treatment.
// KPI values and their qualitative range label share a visual baseline.

#let metric-strip(data) = {
  let n = data.items.len()
  let cols = (1fr,) * n
  let padding = if data.compact { spacing-3 } else { spacing-4 }

  block(
    width: 100%,
    fill: color-surface,
    stroke: (paint: color-border, thickness: component-card-border-width),
    radius: 10pt,
  )[
    #grid(
      columns: cols,
      gutter: 0pt,
      ..data.items.enumerate().map(((i, item)) => {
        let accent = if item.accent_color != none { rgb(item.accent_color) }
          else if item.status == "good" { color-ok }
          else if item.status == "warn" { color-warn }
          else if item.status == "bad" { color-bad }
          else { color-primary }

        block(
          width: 100%,
          inset: (x: padding, y: padding),
          stroke: if i < n - 1 { (right: 0.5pt + color-border) } else { none },
        )[
          #label-text(item.label)
          #v(spacing-2)
          #grid(
            columns: (auto, auto),
            gutter: 5pt,
            align: bottom + left,
            text(
              size: if data.compact { font-size-sm } else { font-size-xl },
              weight: "bold",
              fill: accent,
            )[#item.value],
            if item.unit != none [#small-text(item.unit)],
          )
        ]
      })
    )
  ]
}
