import { ansiToHtml } from '@casoon/pages-theme/ansi';
import type { ShowcaseExample } from '@casoon/pages-theme/showcase';

// Real output of auditmysite, recorded with examples/record.sh and rendered at build time.
// Text fixtures go through the theme's ANSI renderer; the terminal demos are castwright
// recordings (<name>.terminal.yaml) compiled to an animated SVG and an asciicast (<name>.cast),
// whose text is the demo's accessible and reduced-motion version.
const files = import.meta.glob<string>('../../examples/*.{txt,json,cast}', {
  query: '?raw',
  import: 'default',
  eager: true,
});
const svgs = import.meta.glob<string>('../../examples/*.svg', {
  query: '?url',
  import: 'default',
  eager: true,
});
const svgSources = import.meta.glob<string>('../../examples/*.svg', {
  query: '?raw',
  import: 'default',
  eager: true,
});

export function fixture(file: string): string {
  const source = files[`../../examples/${file}`];
  if (source === undefined) throw new Error(`examples/${file} not found`);
  return source;
}

/** Everything the recording printed, as one ANSI stream: the output events of the asciicast. */
function castText(name: string): string {
  const [, ...events] = fixture(`${name}.cast`).trim().split('\n');
  return events
    .map((line) => JSON.parse(line) as [number, string, string])
    .filter(([, type]) => type === 'o')
    .map(([, , data]) => data)
    .join('');
}

const escapeAttr = (s: string) => s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;');

// Scoped to .cw-demo. It travels with the markup because the showcase panel takes HTML only.
// The animated SVG has no pause of its own, so the checkbox swaps it for the text version
// (WCAG 2.2.2); screen readers always get the text, reduced motion shows only the text.
const style = `<style>
.cw-demo{position:relative;margin:0;overflow:hidden;border:1px solid var(--border);border-radius:14px;background:var(--term-bg);color:var(--term-text)}
.cw-demo figcaption{padding:10px 16px;border-bottom:1px solid var(--term-border);color:var(--term-dim);font-family:var(--font-mono);font-size:12px}
.cw-anim img{display:block;width:100%;height:auto}
.cw-text{margin:0;padding:18px;font-size:12.5px;line-height:1.7;white-space:pre-wrap;overflow-wrap:anywhere}
.cw-controls{display:flex;align-items:center;gap:8px;padding:8px 16px;border-top:1px solid var(--term-border);color:var(--term-dim);font-size:13px}
.cw-controls input{margin:0;accent-color:var(--brand)}
.cw-demo:has(.cw-toggle:checked) .cw-anim{display:none}
@media (prefers-reduced-motion:no-preference){.cw-demo:not(:has(.cw-toggle:checked)) .cw-text{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}}
@media (prefers-reduced-motion:reduce){.cw-anim,.cw-controls{display:none}}
</style>`;

/**
 * A castwright terminal demo: the animated SVG, its text, and a control that stops the
 * animation by showing the text instead. `id` must be unique on the page.
 */
export function castDemo(name: string, id: string, caption?: string): string {
  const src = svgs[`../../examples/${name}.svg`];
  const svg = svgSources[`../../examples/${name}.svg`];
  if (src === undefined || svg === undefined) throw new Error(`examples/${name}.svg not found`);
  const [, width, height] = /viewBox="0 0 ([\d.]+) ([\d.]+)"/.exec(svg) ?? [];
  return `<figure class="cw-demo">${style}${
    caption ? `<figcaption>${escapeAttr(caption)}</figcaption>` : ''
  }` +
    `<div class="cw-anim"><img src="${src}" width="${width}" height="${height}" alt="" decoding="async"></div>` +
    `<pre class="cw-text">${ansiToHtml(castText(name))}</pre>` +
    `<div class="cw-controls"><input type="checkbox" class="cw-toggle" id="${id}"><label for="${id}">Stop animation, show as text</label></div></figure>`;
}

const examples_ = [
  {
    slug: 'single-url',
    title: 'Single URL, terminal summary',
    demo: 'single-table',
    command: 'auditmysite https://www.casoon.de/ --format table --no-sitemap-suggest --lang en --request-mode browser',
    tags: ['single', 'table'],
    description:
      'A full single-page audit of casoon.de with the compact terminal summary. The screen-reader sidecar JSON is written next to it. --request-mode skips the prompt an interactive terminal gets.',
  },
  {
    slug: 'technician-mode',
    title: 'Technician mode',
    demo: 'technician',
    command:
      'auditmysite --sitemap https://www.casoon.de/sitemap.xml --technician -m 3 -o tech/ --request-mode browser\njq -c .totals tech/index.json\njq -r \'"\\(.severity)\\t\\(.rule_id)\\t\\(.url)"\' tech/findings.jsonl',
    tags: ['batch', 'json', 'technician'],
    description:
      'The first three sitemap URLs of casoon.de as one JSON file per page, plus index.json and findings.jsonl to script against.',
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
    demo: 'report-lint',
    command: 'auditmysite https://www.casoon.de/ -f json -o report.json --quiet --no-sitemap-suggest\nauditmysite report-lint report.json; echo "exit code $?"',
    tags: ['json', 'lint'],
    description:
      'Deterministic consistency checks on a finished JSON report, without network or Chrome. Exit code 3 when a finding reaches --fail-on (default: high).',
  },
];

export const examples: ShowcaseExample[] = examples_.map(({ file, demo, command, ...meta }) => ({
  ...meta,
  file: demo ? `examples/${demo}.terminal.yaml` : `examples/${file}`,
  input: { code: `$ ${command.split('\n').join('\n$ ')}`, lang: 'sh' },
  output: demo
    ? { html: castDemo(demo, `cw-${meta.slug}`), kind: 'panel' }
    : { html: ansiToHtml(fixture(file as string)), kind: 'terminal' },
}));
