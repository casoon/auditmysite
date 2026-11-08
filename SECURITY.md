# Security Policy

## Reporting a vulnerability

Please report security issues privately by email to **joern.seidel@casoon.de**
rather than in a public issue or pull request. Include the affected version
(`auditmysite --version`), the command line or input that triggers the problem,
and what you expected to happen.

You can expect an acknowledgement within a few days. After a fix is released,
the report is credited in the changelog unless you prefer otherwise.

## Supported versions

Only the latest released version on [crates.io](https://crates.io/crates/auditmysite)
and in the [GitHub releases](https://github.com/casoon/auditmysite/releases) receives
security fixes. Please check that the problem still occurs there before reporting.

## Scope

auditmysite fetches the URLs it is given and drives a local Chrome/Chromium, so the
following are in scope:

- Server-side request forgery or access to local/private addresses through URL
  handling, sitemaps, redirects, image or resource fetches.
- Unsafe handling of untrusted page content, sitemap or URL-file input, or of
  report output (for example injection into the generated PDF or JSON).
- Problems in how the tool downloads, verifies or launches the managed browser.
- Vulnerable or malicious dependencies that affect a released version.

Out of scope: findings the audit reports about the sites you scan (those are the
tool's output, not vulnerabilities in it), issues that need an attacker who already
controls your machine or your Chrome installation, and denial of service by
auditing an extremely large or slow site.
