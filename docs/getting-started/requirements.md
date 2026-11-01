---
title: "Requirements"
description: "What auditmysite needs at run time: a browser, and how to install a managed one."
order: 4
---

- Rust as pinned in `rust-toolchain.toml` for local builds
- Chrome, Chromium, or a managed browser install (`auditmysite browser install`)
- macOS, Linux, or Windows for released binaries

`auditmysite` requires a browser to be present at run time. It does **not** download or install one automatically — it reports an error and exits if none is found. To install a managed Chrome for Testing into `~/.auditmysite/browsers/`:

```bash
auditmysite browser detect                    # show what's found
auditmysite browser install --headless-shell  # smaller; fallback only (see below)
auditmysite browser install                   # download Chrome for Testing (opt-in)
```

On macOS, batches with keyboard journeys audit one page at a time when a full browser is used:
Chrome runs the AppKit event loop even in headless mode, and keyboard events a page does not
consume go through it on the browser's main thread; with several pages at once that can stall
the whole browser. `--concurrency` overrides this. The headless shell is not affected, but sites
with bot protection may serve it a challenge page, so it is only used when no system browser is
found.
