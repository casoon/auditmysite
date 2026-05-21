#!/usr/bin/env node
/**
 * axe-compare.js — Cross-tool comparison: auditmysite vs. axe-core
 *
 * Runs both tools on the same URL and produces a side-by-side Markdown table
 * grouped by axe rule ID. Useful for calibrating our rules and spotting gaps.
 *
 * Usage:
 *   node scripts/axe-compare.js <URL> [options]
 *
 * Options:
 *   --output <file>      Write Markdown to file (default: stdout)
 *   --raw-output <file>  Also write the combined raw {auditmysite, axe} JSON
 *                        for reproducibility / offline re-analysis
 *   --bin <path>         Path to auditmysite binary (default: ./target/release/auditmysite)
 *   --level <A|AA>       WCAG level to pass to auditmysite (default: AA)
 *
 * Requirements (run once):
 *   npm install playwright axe-core
 *   npx playwright install chromium
 *
 * See docs/AXE_PARITY.md for the calibration workflow, rule categories, and the
 * policy that separates confirmed axe gaps from auditmysite-only heuristics.
 */

'use strict';

const { execSync } = require('child_process');
const { chromium } = require('playwright');
const axeSource = require('axe-core').source;
const fs = require('fs');
const path = require('path');
const url = require('url');

// ── CLI args ──────────────────────────────────────────────────────────────────

const args = process.argv.slice(2);
if (args.length === 0 || args[0] === '--help') {
  console.error('Usage: node scripts/axe-compare.js <URL> [--output <file>] [--bin <path>] [--level AA]');
  process.exit(1);
}

const targetUrl = args[0];
let outputFile = null;
let binPath = './target/release/auditmysite';
let wcagLevel = 'AA';
let rawOutputFile = null;

for (let i = 1; i < args.length; i++) {
  if (args[i] === '--output' && args[i + 1]) outputFile = args[++i];
  else if (args[i] === '--raw-output' && args[i + 1]) rawOutputFile = args[++i];
  else if (args[i] === '--bin' && args[i + 1]) binPath = args[++i];
  else if (args[i] === '--level' && args[i + 1]) wcagLevel = args[++i].toUpperCase();
}

// ── Step 1: Run auditmysite ───────────────────────────────────────────────────

function runAuditMySite(targetUrl, binPath, level) {
  console.error(`[1/3] Running auditmysite on ${targetUrl} ...`);
  try {
    const cmd = `${binPath} "${targetUrl}" --format json --level ${level}`;
    const raw = execSync(cmd, { encoding: 'utf8', maxBuffer: 10 * 1024 * 1024, stdio: ['pipe', 'pipe', 'pipe'] });
    // Strip any non-JSON prefix (e.g. ASCII banner written to stdout).
    // Prefer the schema marker over the first brace because banners/logs can
    // contain braces too.
    const marker = raw.indexOf('{\n  "schema_version"');
    const jsonStart = marker >= 0 ? marker : raw.indexOf('{');
    if (jsonStart < 0) throw new Error('No JSON found in output: ' + raw.slice(0, 200));
    return JSON.parse(raw.slice(jsonStart));
  } catch (err) {
    const msg = err.stdout || err.stderr || err.message;
    console.error('auditmysite failed:', msg.slice(0, 500));
    process.exit(1);
  }
}

// Extract findings from our JSON: violations + warnings
// Returns Map<axeId, { count, criterion, level, ourOnly: violations[] }>
function extractOurFindings(json) {
  // Support current single report schema, older report.findings, and raw WCAG.
  const currentFindings = Array.isArray(json?.pages)
    ? json.pages.flatMap((page) => page.findings || [])
    : [];
  const legacyFindings = json?.report?.findings || [];
  const rawViolations = json?.report?.raw_wcag?.violations || [];
  const rawWarnings = json?.report?.raw_wcag?.warnings || [];
  const findings = currentFindings.length > 0 ? currentFindings : legacyFindings.length > 0 ? legacyFindings : rawViolations;
  const warnings = rawWarnings;

  if (findings.length === 0 && warnings.length === 0) {
    console.error('No findings found in auditmysite JSON output.');
    process.exit(1);
  }

  const map = new Map(); // axeId → { count, criterion, wcagLevel, messages, samples }

  const add = (v, isWarning = false) => {
    // Prefer axe_id (mapped from taxonomy), fall back to rule_id, then synthesize
    const axeId = v.axe_id || v.rule_id || `(no-axe-id:${v.wcag_criterion || v.criterion || '?'})`;
    const criterion = v.wcag_criterion || v.criterion || v.rule || '?';
    const wcagLevel = v.wcag_level || v.level || '?';
    if (!map.has(axeId)) {
      map.set(axeId, { axeId, criterion, wcagLevel, count: 0, messages: [], samples: [], isWarning });
    }
    const entry = map.get(axeId);
    entry.isWarning = entry.isWarning || isWarning;
    entry.count += (v.occurrence_count || 1);
    const msg = v.description || v.message || '';
    if (msg && entry.messages.length < 3) entry.messages.push(msg.slice(0, 80));
    const occurrences = v.occurrences || [v];
    for (const occurrence of occurrences) {
      if (entry.samples.length >= 3) break;
      const selector = occurrence.selector || occurrence.node_id;
      if (!selector) continue;
      entry.samples.push({
        selector,
        message: occurrence.message || msg,
        html: occurrence.html_snippet || null,
      });
    }
  };

  findings.forEach((v) => add(v, false));
  warnings.forEach((v) => add(v, true));

  return map;
}

// ── Step 2: Run axe-core via Playwright ──────────────────────────────────────

async function runAxeCore(targetUrl) {
  console.error('[2/3] Running axe-core via Playwright ...');
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ bypassCSP: true });
  const page = await context.newPage();

  try {
    await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
    // Small settle time for dynamic content
    await page.waitForTimeout(1500);

    // Inject axe-core and run
    await page.addScriptTag({ content: axeSource });
    const results = await page.evaluate(async () => {
      return await window.axe.run(document, {
        runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'] },
        resultTypes: ['violations', 'incomplete'],
      });
    });
    return results;
  } finally {
    await context.close();
    await browser.close();
  }
}

// Returns Map<axeId, { count, criterion, impact, description }>
function extractAxeFindings(axeResults) {
  const map = new Map();

  const addGroup = (violations, kind) => {
    for (const v of violations) {
      const wcagTags = v.tags.filter((t) => /^wcag\d/.test(t));
      const criterion = wcagTagsToCriterion(wcagTags);
      map.set(v.id, {
        axeId: v.id,
        criterion,
        impact: v.impact || '?',
        description: v.description?.slice(0, 100),
        count: v.nodes?.length ?? 1,
        kind,
        samples: (v.nodes || []).slice(0, 3).map((n) => ({
          target: (n.target || []).join(' '),
          html: n.html || '',
          failure: n.failureSummary || '',
        })),
      });
    }
  };

  addGroup(axeResults.violations || [], 'violation');
  addGroup(axeResults.incomplete || [], 'incomplete');
  return map;
}

// Map axe tag like "wcag111" → "1.1.1", "wcag143" → "1.4.3" etc.
function wcagTagsToCriterion(tags) {
  for (const tag of tags) {
    const m = tag.match(/^wcag(\d)(\d)(\d+)$/);
    if (m) return `${m[1]}.${m[2]}.${m[3]}`;
  }
  return '?';
}

// ── Step 3: Build comparison table ───────────────────────────────────────────

function buildTable(ourMap, axeMap) {
  const allIds = new Set([...ourMap.keys(), ...axeMap.keys()]);

  const rows = [];
  for (const axeId of allIds) {
    const our = ourMap.get(axeId);
    const axe = axeMap.get(axeId);

    const criterion = our?.criterion || axe?.criterion || '?';
    const wcagLevel = our?.wcagLevel || axe?.impact || '?';

    let ourResult, axeResult, note;

    if (our && axe) {
      ourResult = `${our.count} finding${our.count !== 1 ? 's' : ''}${our.isWarning ? ' ⚠' : ''}`;
      axeResult = axe.kind === 'incomplete'
        ? `${axe.count} (needs review)`
        : `${axe.count} violation${axe.count !== 1 ? 's' : ''}`;
      note = our.count > 0 && axe.count > 0 ? '✓ both' : '~ partial';
    } else if (our && !axe) {
      ourResult = `${our.count} finding${our.count !== 1 ? 's' : ''}${our.isWarning ? ' ⚠' : ''}`;
      axeResult = '–';
      note = our.isWarning ? 'only-us (heuristic)' : 'only-us';
    } else {
      ourResult = '–';
      axeResult = axe.kind === 'incomplete'
        ? `${axe.count} (needs review)`
        : `${axe.count} violation${axe.count !== 1 ? 's' : ''}`;
      note = axe.kind === 'incomplete' ? 'only-axe (incomplete)' : 'gap ← axe only';
    }

    rows.push({ axeId, criterion, wcagLevel, ourResult, axeResult, note, our, axe });
  }

  // Sort: gaps first (actionable), then both, then only-us
  rows.sort((a, b) => {
    const priority = (n) => (n.startsWith('gap') ? 0 : n.startsWith('only-axe') ? 1 : n === '✓ both' ? 2 : 3);
    return priority(a.note) - priority(b.note) || a.axeId.localeCompare(b.axeId);
  });

  return rows;
}

function renderMarkdown(rows, targetUrl, ourVersion) {
  const now = new Date().toISOString().slice(0, 19).replace('T', ' ');
  const gaps = rows.filter((r) => r.note.startsWith('gap')).length;
  const both = rows.filter((r) => r.note === '✓ both').length;
  const onlyUs = rows.filter((r) => r.note.startsWith('only-us')).length;
  const onlyAxeIncomplete = rows.filter((r) => r.note.startsWith('only-axe (incomplete)')).length;
  const axeOnlyRows = rows.filter((r) => r.note.startsWith('gap') || r.note.startsWith('only-axe'));

  const lines = [
    `# axe-core Comparison — ${targetUrl}`,
    ``,
    `Generated: ${now}  `,
    `Tool: auditmysite ${ourVersion} · axe-core (wcag2a/aa/21a/21aa/22aa tags)`,
    ``,
    `## Summary`,
    ``,
    `| | Count |`,
    `|---|---|`,
    `| ✓ Both tools flagged | ${both} |`,
    `| gap ← axe-core only | ${gaps} |`,
    `| only-us (we flag, axe doesn't) | ${onlyUs} |`,
    `| only-axe (incomplete/needs-review) | ${onlyAxeIncomplete} |`,
    ``,
    `## Rule-by-rule comparison`,
    ``,
    `| axe-id | criterion | our result | axe-core result | note |`,
    `|--------|-----------|-----------|-----------------|------|`,
    ...rows.map(
      (r) =>
        `| \`${r.axeId}\` | ${r.criterion} | ${r.ourResult} | ${r.axeResult} | ${r.note} |`
    ),
    ``,
    `## axe-core only details`,
    ``,
    ...(axeOnlyRows.length === 0
      ? [`No axe-core-only findings.`]
      : axeOnlyRows.flatMap((r) => [
          `### ${r.axeId}`,
          ``,
          `Criterion: ${r.criterion}  `,
          `axe-core: ${r.axeResult}  `,
          `Status: ${r.note}`,
          ``,
          ...(r.axe?.samples || []).flatMap((sample, index) => [
            `Sample ${index + 1}: \`${sample.target || '(no target)'}\``,
            sample.html ? `` : null,
            sample.html ? '```html' : null,
            sample.html ? sample.html.slice(0, 500) : null,
            sample.html ? '```' : null,
            sample.failure ? `` : null,
            sample.failure ? '```text' : null,
            sample.failure ? sample.failure.slice(0, 800) : null,
            sample.failure ? '```' : null,
            ``,
          ].filter(Boolean)),
        ])),
    ``,
    `### Legend`,
    ``,
    `- **✓ both** — both tools flagged this rule on the page`,
    `- **gap ← axe only** — axe-core found violations we missed (coverage gap)`,
    `- **only-us** — we flag this, axe-core does not (possible false positive or intentional extension)`,
    `- **⚠** — our finding is a heuristic warning, not a confirmed violation`,
    `- **(needs review)** — axe-core returned this as \`incomplete\` (requires manual confirmation)`,
  ];

  return lines.join('\n');
}

// ── Main ──────────────────────────────────────────────────────────────────────

(async () => {
  const ourJson = runAuditMySite(targetUrl, binPath, wcagLevel);
  const ourVersion = ourJson?.metadata?.tool || 'unknown';
  const ourMap = extractOurFindings(ourJson);

  const axeResults = await runAxeCore(targetUrl);
  if (rawOutputFile) {
    fs.mkdirSync(path.dirname(path.resolve(rawOutputFile)), { recursive: true });
    fs.writeFileSync(rawOutputFile, JSON.stringify({ auditmysite: ourJson, axe: axeResults }, null, 2), 'utf8');
    console.error(`Raw data written to ${rawOutputFile}`);
  }
  const axeMap = extractAxeFindings(axeResults);

  console.error(`[3/3] Comparing ${ourMap.size} our rules vs ${axeMap.size} axe rules ...`);

  const rows = buildTable(ourMap, axeMap);
  const markdown = renderMarkdown(rows, targetUrl, ourVersion);

  if (outputFile) {
    fs.mkdirSync(path.dirname(path.resolve(outputFile)), { recursive: true });
    fs.writeFileSync(outputFile, markdown, 'utf8');
    console.error(`Written to ${outputFile}`);
  } else {
    process.stdout.write(markdown + '\n');
  }
})();
