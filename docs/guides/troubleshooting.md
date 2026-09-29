---
title: Troubleshooting
description: Common errors, what causes them and what to do.
order: 4
---

## No browser found

auditmysite exits with installation hints when it finds no browser.

```sh
auditmysite browser detect
auditmysite browser install
# or point it at one
auditmysite --browser-path "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" https://example.com
```

See [Browser setup](../browser-setup/).

## “Access to … was blocked”

The site answered HTTP 401, 403, 407 or 429, or served a bot challenge (Cloudflare, Fastly,
Akamai, DataDome, Imperva, HUMAN). That page is not the site's content, so the URL is not scored;
in a batch it is listed under `errors`. Ask the site owner to allow the auditing machine. If you
use the headless shell, try a full Chrome first.

## Timeouts

`--timeout` sets the page-load timeout in seconds (default 30). Slow pages may also need a larger
`--stability-budget-ms` for late hydration (default 1500). `--disable-images` loads faster but
skips the contrast check.

## Docker or root

Use `--no-sandbox`, only in containers you trust.

## More detail

```sh
auditmysite --verbose https://example.com
RUST_LOG=debug auditmysite https://example.com
```

## Still stuck

Open an [issue](https://github.com/casoon/auditmysite/issues) with the command, the full error
message, the output of `auditmysite --version` and your operating system.
