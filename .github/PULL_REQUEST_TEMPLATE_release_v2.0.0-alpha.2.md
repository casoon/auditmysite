# PR: Release v2.0.0-alpha.2

## Summary
- Root cleanup: moved architecture/release/process/maintenance/notes docs under docs/*, dev/test scripts under scripts/dev/, and sample files under examples/data/
- Report generator restored to original lightweight, high-contrast layout (pre-design state)
- Redirect handling:
  - Robust detection of HTTP redirects (even when Playwright follows to 200)
  - Skip redirect targets consistently and list under "Skipped (Redirects)"
  - Homepage-first sampling; parallel minimal checks; post-run top-up to ensure max non-redirect pages
- Detailed Accessibility block:
  - Scrollable Markdown + copy button restored for all non-skipped pages
- CLI logging:
  - Remaining DEBUG logs silenced; cleaner output
- README:
  - Version updated to 2.0.0-alpha.2; links to docs verified
- Publish:
  - Published @casoon/auditmysite@2.0.0-alpha.2 (tag: next and latest)

## Notable Changes
- feat(cli): homepage-first sampling, parallelized minimal checks, and top-up to ensure max non-redirect pages
- fix(redirects): robust detection; short-circuit when skipRedirects enabled
- fix(content-weight): isolate analysis on separate page to avoid context destruction
- fix(seo): DOM null-safety in evaluators
- style(report): reverted to original, lightweight design for readability/contrast
- chore(root): move docs/scripts/examples to proper locations; update README

## Docs & Links
- Changelog: docs/release/CHANGELOG.md
- Release Notes: docs/release/RELEASE-NOTES-2.0.0-alpha.2.md
- Upgrade Guide: docs/maintenance/UPGRADE-v2.0.0.md

## Checklist
- [x] Build passes (npm run build)
- [x] Reports generate (HTML + detailed issues MD)
- [x] Redirect pages skipped; filled up to max non-redirect pages
- [x] README updated (version, docs links)
- [x] npm publish done (alpha.2)
