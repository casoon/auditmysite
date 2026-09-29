import { docsCollection } from '@casoon/pages-theme/content';
import { glob } from 'astro/loaders';

// Sources: ../docs (docs/ in the project repository). The top level of docs/ also holds the
// repository's own reference files (ARCHITECTURE.md, OUTPUT_CONTRACT.md, …), which are not site
// pages. Only docs/index.md and the group folders are read; the schema stays the theme's.
export const collections = {
  docs: {
    ...docsCollection(),
    loader: glob({ base: '../docs', pattern: ['index.md', '*/**/*.{md,mdx}'] }),
  },
};
