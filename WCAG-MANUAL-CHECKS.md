# WCAG Rules: Not Automatable

These rules cannot be reliably checked through static DOM/AX-tree analysis or CDP
evaluation. They require manual inspection or specialised tools (OCR, video analysis,
behavioural observation).

No known automated tool (axe-core, Pa11y, Lighthouse) implements these criteria fully.

---

## 1.4.9 Images of Text (No Exception) — Level AAA

**Criterion:** Text may only appear in images if it is purely decorative or if a
specific visual presentation is essential to the information conveyed.

**Why not automatable:** Requires OCR / computer vision to detect text inside images.
Heuristics based on image dimensions or file names produce too many false positives.

**Manual check:** Visually inspect all `<img>`, `<canvas>`, and CSS background images;
`<img>` elements with `role="img"` and text-like alt attributes are the primary
candidates.

---

## 2.3.2 Three Flashes (No Exception) — Level AAA

**Criterion:** Content must not flash more than three times per second, with no
exception for the "safe area" threshold that applies under 2.3.1 Level A.

**Why not automatable:** Requires frame-accurate visual rendering analysis (video
capture at ≥ 60 fps). Even the weaker 2.3.1 (Level A) is unsupported by nearly every
tool. CSS animation duration is not a proxy for the actual flash rate on screen.

**Manual check:** Analyse animations and videos with a photosensitivity tool such as
PEAT (Photosensitive Epilepsy Analysis Tool).

---

## 3.2.5 Change on Request — Level AAA

**Criterion:** Context changes are initiated only by explicit user request; no
automatic redirects, no auto-advance.

**Why not automatable:** Requires behavioural observation over time (e.g. whether a
slideshow advances on its own, or whether `onchange` triggers navigation).
`<meta http-equiv="refresh">` is detectable, but the criterion is broader. Interaction
sequences cannot be assessed statically.

**Manual check:** Observe the page without interaction (does anything navigate
automatically?); test all form fields for unexpected context changes on `onchange` /
`onblur`.

---

## 3.3.6 Error Prevention (All) — Level AAA

**Criterion:** For all pages that accept user input, the user must be able to review,
confirm, or correct entries — not only for legal or financial transactions (that is
Level AA 3.3.4).

**Why not automatable:** Whether a confirmation step exists cannot be inferred from the
DOM alone. Requires real form-submission simulation with state tracking across multiple
steps.

**Manual check:** Complete all forms through to submission; verify that a summary,
confirmation dialog, or correction opportunity is offered before the final action is
committed.
