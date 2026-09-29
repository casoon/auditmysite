---
title: Browser setup
description: auditmysite drives Chrome or Chromium over the DevTools Protocol. It never downloads a browser on its own; this page shows how it finds one and how to install one.
order: 6
---

## Which browser is used

In this order:

1. `--browser-path <path>` on the command line
2. the environment variable `AUDITMYSITE_BROWSER` (`CHROME_PATH` is still read for compatibility)
3. a system browser at the usual locations for macOS, Linux and Windows: Chrome, then Edge,
   Ungoogled Chromium, Chromium
4. a managed install under `~/.auditmysite/browsers/`: Chrome for Testing, then the headless
   shell

```sh
auditmysite browser detect   # what is found, and where
auditmysite browser path     # the browser that will be used
auditmysite doctor           # browser, permissions, connectivity
```

`--remote-debugging-port <port>` connects to a Chrome you started yourself with
`--remote-debugging-port`.

## Installing a managed browser

```sh
auditmysite browser install                   # Chrome for Testing
auditmysite browser install --headless-shell  # smaller headless shell
auditmysite browser remove                    # delete the managed install
```

Both install into `~/.auditmysite/browsers/` and are used only when no system browser is found.
`--version` installs a specific version, `--force` reinstalls.

## macOS: full Chrome and the headless shell

On macOS, Chrome runs the AppKit event loop even in headless mode. Keyboard events a page does
not consume go through it on the browser's main thread, and with several pages at once that can
stall the whole browser. So batches with keyboard journeys audit one page at a time when a full
Chrome is used; `--concurrency` overrides this.

The headless shell is not affected, but sites with bot protection may serve it a challenge page
instead of their content. That is why it is only a fallback when no system browser exists.

## Containers and root

Chrome's sandbox does not start as root or in many containers. Use `--no-sandbox` there, and only
there: it reduces isolation.
