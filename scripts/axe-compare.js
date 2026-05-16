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
 *   --output <file>   Write Markdown to file (default: stdout)
 *   --bin <path>      Path to auditmysite binary (default: ./target/release/auditmysite)
 *   --level <A|AA>    WCAG level to pass to auditmysite (default: AA)
 *
 * Requirements (run once):
 *   npm install playwright axe-core
 *   npx playwright install chromium
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

for (let i = 1; i < args.length; i++) {
  if (args[i] === '--output' && args[i + 1]) outputFile = args[++i];
  else if (args[i] === '--bin' && args[i + 1]) binPath = args[++i];
  else if (args[i] === '--level' && args[i + 1]) wcagLevel = args[++i].toUpperCase();
}

// ── Step 1: Run auditmysite ───────────────────────────────────────────────────

function runAuditMySite(targetUrl, binPath, level) {
  console.error(`[1/3] Running auditmysite on ${targetUrl} ...`);
  try {
    const cmd = `${binPath} "${targetUrl}" --format json --level ${level}`;
    const raw = execSync(cmd, { encoding: 'utf8', maxBuffer: 10 * 1024 * 1024, stdio: ['pipe', 'pipe', 'pipe'] });
    // Strip any non-JSON prefix (e.g. ASCII banner written to stdout)
    const jsonStart = raw.indexOf('{');
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
  // Support both old (report.raw_wcag) and new (report.findings) JSON schema
  const findings = json?.report?.findings || json?.report?.raw_wcag?.violations || [];
  const warnings = json?.report?.raw_wcag?.warnings || [];
  if (!json?.report) {
    console.error('No report field in auditmysite JSON output. Re-run with a current binary.');
    process.exit(1);
  }

  const map = new Map(); // axeId → { count, criterion, wcagLevel, messages }

  const add = (v) => {
    // Prefer axe_id (mapped from taxonomy), fall back to rule_id, then synthesize
    const axeId = v.axe_id || v.rule_id || `(no-axe-id:${v.wcag_criterion || v.criterion || '?'})`;
    const criterion = v.wcag_criterion || v.criterion || v.rule || '?';
    const wcagLevel = v.wcag_level || v.level || '?';
    if (!map.has(axeId)) {
      map.set(axeId, { axeId, criterion, wcagLevel, count: 0, messages: [] });
    }
    const entry = map.get(axeId);
    entry.count += (v.occurrence_count || 1);
    const msg = v.description || v.message || '';
    if (msg && entry.messages.length < 3) entry.messages.push(msg.slice(0, 80));
  };

  findings.forEach(add);
  warnings.forEach((v) => {
    const axeId = v.axe_id || v.rule_id || `(warning:${v.wcag_criterion || v.criterion || '?'})`;
    const criterion = v.wcag_criterion || v.criterion || v.rule || '?';
    const wcagLevel = v.wcag_level || v.level || '?';
    if (!map.has(axeId)) {
      map.set(axeId, { axeId, criterion, wcagLevel, count: 0, messages: [], isWarning: true });
    }
    const entry = map.get(axeId);
    entry.count++;
    entry.isWarning = true;
  });

  return map;
}

// ── Step 2: Run axe-core via Playwright ──────────────────────────────────────

async function runAxeCore(targetUrl) {
  console.error('[2/3] Running axe-core via Playwright ...');
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

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

    rows.push({ axeId, criterion, wcagLevel, ourResult, axeResult, note });
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
