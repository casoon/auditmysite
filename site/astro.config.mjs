// @ts-check
import { readFileSync } from 'node:fs';
import casoonPages from '@casoon/pages-theme';
import { defineConfig } from 'astro/config';

// The crate version is the single source; the header badge follows Cargo.toml.
const cargoToml = readFileSync(new URL('../Cargo.toml', import.meta.url), 'utf8');
const version = cargoToml.match(/^version = "([^"]+)"/m)?.[1];
if (!version) throw new Error('version not found in ../Cargo.toml');

// Project page: https://casoon.github.io/auditmysite/ — `base` is the GitHub Pages path.
export default defineConfig({
  site: 'https://casoon.github.io',
  base: '/auditmysite/',
  integrations: [
    casoonPages({
      name: 'auditmysite',
      description:
        'WCAG 2.2 AA accessibility and website audits of rendered pages via the Chrome DevTools Protocol. JSON and PDF reports, PDF in German and English.',
      repo: 'casoon/auditmysite',
      version,
      license: 'MIT',
      packages: [
        { label: 'crates.io', href: 'https://crates.io/crates/auditmysite' },
        { label: 'docs.rs', href: 'https://docs.rs/auditmysite' },
        { label: 'GitHub Releases', href: 'https://github.com/casoon/auditmysite/releases' },
      ],
      docsGroups: {
        'getting-started': 'Getting started',
        guides: 'Guides',
        concepts: 'Concepts',
        reference: 'Reference',
      },
      // CHANGELOG.md is a chronological development log, not Keep a Changelog;
      // release notes live on GitHub Releases (linked in the footer and the docs).
      changelog: false,
    }),
  ],
});
