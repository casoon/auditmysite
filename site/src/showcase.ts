import { ansiToHtml } from '@casoon/pages-theme/ansi';
import type { ShowcaseExample } from '@casoon/pages-theme/showcase';

// Real output of auditmysite, recorded with examples/record.sh and rendered at build time.
// auditmysite writes terminal text, not HTML, so the theme's ANSI renderer displays it.
const files = import.meta.glob<string>('../../examples/*.{ansi,txt,json}', {
  query: '?raw',
  import: 'default',
  eager: true,
});

export function fixture(file: string): string {
  const source = files[`../../examples/${file}`];
  if (source === undefined) throw new Error(`examples/${file} not found`);
  return source;
}

const examples_ = [
  {
    slug: 'single-url',
    title: 'Single URL, terminal summary',
    file: 'single-table.ansi',
    command: 'auditmysite https://www.casoon.de/ --format table --color always --no-sitemap-suggest --lang en',
    tags: ['single', 'table'],
    description:
      'A full single-page audit of casoon.de with the compact terminal summary. The screen-reader sidecar JSON is written next to it.',
  },
  {
    slug: 'audit-plan',
    title: 'Audit plan',
    file: 'plan.txt',
    command: 'auditmysite plan https://www.casoon.de/ --lang en',
    tags: ['plan'],
    description: 'Mode, modules and output files an audit would use, printed without starting a browser.',
  },
  {
    slug: 'wcag-coverage',
    title: 'WCAG coverage in the JSON report',
    file: 'wcag-coverage.json',
    command: "auditmysite https://www.casoon.de/ -f json -o report.json\njq '.summary.wcag_coverage' report.json",
    tags: ['json', 'wcag'],
    description:
      'Every report states how many WCAG 2.2 A/AA criteria the automated checks cover and how many need manual review.',
  },
  {
    slug: 'report-lint',
    title: 'report-lint',
    file: 'report-lint.txt',
    command: 'auditmysite report-lint report.json',
    tags: ['json', 'lint'],
    description:
      'Deterministic consistency checks on a finished JSON report, without network or Chrome. Exit code 3 when a finding reaches --fail-on (default: high).',
  },
];

export const examples: ShowcaseExample[] = examples_.map(({ file, command, ...meta }) => ({
  ...meta,
  file: `examples/${file}`,
  input: { code: `$ ${command.split('\n').join('\n$ ')}`, lang: 'sh' },
  output: { html: ansiToHtml(fixture(file)), kind: 'terminal' },
}));
