#!/usr/bin/env node
/**
 * pa11y-compare.js — Cross-tool comparison: auditmysite vs. pa11y
 *
 * Runs both tools on the same URL and produces a side-by-side Markdown table
 * grouped by WCAG criterion. Pa11y wraps HTML_CodeSniffer (HTMLCS) and checks
 * rules that axe-core doesn't — useful for spotting different coverage gaps.
 *
 * Usage:
 *   node scripts/pa11y-compare.js <URL> [options]
 *
 * Options:
 *   --output <file>   Write Markdown to file (default: stdout)
 *   --bin <path>      Path to auditmysite binary (default: ./target/release/auditmysite)
 *   --level <A|AA>    WCAG level to pass to auditmysite (default: AA)
 *
 * Requirements (run once from scripts/):
 *   npm install
 */

'use strict';

const { execSync } = require('child_process');
const pa11y = require('pa11y');
const fs = require('fs');
const path = require('path');

// ── CLI args ──────────────────────────────────────────────────────────────────

const args = process.argv.slice(2);
if (args.length === 0 || args[0] === '--help') {
  console.error('Usage: node scripts/pa11y-compare.js <URL> [--output <file>] [--bin <path>] [--level AA]');
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
    const jsonStart = raw.indexOf('{');
    if (jsonStart < 0) throw new Error('No JSON found in output: ' + raw.slice(0, 200));
    return JSON.parse(raw.slice(jsonStart));
  } catch (err) {
    const msg = err.stdout || err.stderr || err.message;
    console.error('auditmysite failed:', msg.slice(0, 500));
    process.exit(1);
  }
}

// Returns Map<criterion, { count, ourAxeId, messages }>
function extractOurFindings(json) {
  const findings = json?.report?.findings || json?.report?.raw_wcag?.violations || [];
  const warnings = json?.report?.raw_wcag?.warnings || [];
  if (!json?.report) {
    console.error('No report field in auditmysite JSON output.');
    process.exit(1);
  }

  const map = new Map(); // criterion → { count, axeId, ourOnly }

  const add = (v, isWarning) => {
    const criterion = v.wcag_criterion || v.criterion || v.rule || '?';
    const axeId = v.axe_id || v.rule_id || null;
    if (!map.has(criterion)) {
      map.set(criterion, { criterion, axeId, count: 0, isWarning: false });
    }
    const entry = map.get(criterion);
    entry.count += (v.occurrence_count || 1);
    if (isWarning) entry.isWarning = true;
    if (axeId && !entry.axeId) entry.axeId = axeId;
  };

  findings.forEach((v) => add(v, false));
  warnings.forEach((v) => add(v, true));
  return map;
}

// ── Step 2: Run Pa11y ─────────────────────────────────────────────────────────

async function runPa11y(targetUrl, level) {
  console.error('[2/3] Running pa11y ...');
  const standard = level === 'AAA' ? 'WCAG2AAA' : level === 'A' ? 'WCAG2A' : 'WCAG2AA';
  try {
    const results = await pa11y(targetUrl, {
      standard,
      timeout: 30000,
      wait: 1500,
      includeNotices: false,
      includeWarnings: false, // only errors (confirmed violations)
      runners: ['htmlcs'],
    });
    return results;
  } catch (err) {
    console.error('pa11y failed:', err.message);
    process.exit(1);
  }
}

// Returns Map<criterion, { count, kind, codes }>
function extractPa11yFindings(results) {
  const map = new Map();

  for (const issue of results.issues || []) {
    const criterion = pa11yCodeToCriterion(issue.code);
    if (!map.has(criterion)) {
      map.set(criterion, { criterion, count: 0, kind: issue.type, codes: [] });
    }
    const entry = map.get(criterion);
    entry.count++;
    if (!entry.codes.includes(issue.code)) entry.codes.push(issue.code);
  }

  return map;
}

// Extract criterion from Pa11y issue code.
// Example: "WCAG2AA.Principle4.Guideline4_1.4_1_2.H91.InputText.Name" → "4.1.2"
function pa11yCodeToCriterion(code) {
  const m = code.match(/\.(\d+_\d+_\d+)\./);
  if (m) return m[1].replace(/_/g, '.');
  return code;
}

// ── Step 3: Build comparison table ───────────────────────────────────────────

function buildTable(ourMap, pa11yMap) {
  const allCriteria = new Set([...ourMap.keys(), ...pa11yMap.keys()]);

  const rows = [];
  for (const criterion of allCriteria) {
    const our = ourMap.get(criterion);
    const pa = pa11yMap.get(criterion);

    let ourResult, pa11yResult, note;

    if (our && pa) {
      ourResult = `${our.count} finding${our.count !== 1 ? 's' : ''}${our.isWarning ? ' ⚠' : ''}`;
      pa11yResult = `${pa.count} error${pa.count !== 1 ? 's' : ''}`;
      note = '✓ both';
    } else if (our && !pa) {
      ourResult = `${our.count} finding${our.count !== 1 ? 's' : ''}${our.isWarning ? ' ⚠' : ''}`;
      pa11yResult = '–';
      note = our.isWarning ? 'only-us (heuristic)' : 'only-us';
    } else {
      ourResult = '–';
      pa11yResult = `${pa.count} error${pa.count !== 1 ? 's' : ''}`;
      note = 'gap ← pa11y only';
    }

    const axeId = our?.axeId || null;
    rows.push({ criterion, axeId, ourResult, pa11yResult, note });
  }

  rows.sort((a, b) => {
    const priority = (n) => (n.startsWith('gap') ? 0 : n === '✓ both' ? 1 : 2);
    return priority(a.note) - priority(b.note) || a.criterion.localeCompare(b.criterion);
  });

  return rows;
}

function renderMarkdown(rows, targetUrl, ourVersion) {
  const now = new Date().toISOString().slice(0, 19).replace('T', ' ');
  const gaps = rows.filter((r) => r.note.startsWith('gap')).length;
  const both = rows.filter((r) => r.note === '✓ both').length;
  const onlyUs = rows.filter((r) => r.note.startsWith('only-us')).length;

  const lines = [
    `# Pa11y Comparison — ${targetUrl}`,
    ``,
    `Generated: ${now}  `,
    `Tool: auditmysite ${ourVersion} · pa11y/HTMLCS (WCAG2AA)`,
    ``,
    `## Summary`,
    ``,
    `| | Count |`,
    `|---|---|`,
    `| ✓ Both tools flagged | ${both} |`,
    `| gap ← pa11y only | ${gaps} |`,
    `| only-us (we flag, pa11y doesn't) | ${onlyUs} |`,
    ``,
    `## Rule-by-rule comparison`,
    ``,
    `| criterion | axe-id | our result | pa11y result | note |`,
    `|-----------|--------|-----------|--------------|------|`,
    ...rows.map(
      (r) =>
        `| ${r.criterion} | ${r.axeId ? `\`${r.axeId}\`` : '–'} | ${r.ourResult} | ${r.pa11yResult} | ${r.note} |`
    ),
    ``,
    `### Legend`,
    ``,
    `- **✓ both** — both tools flagged this criterion on the page`,
    `- **gap ← pa11y only** — pa11y found violations we missed (coverage gap)`,
    `- **only-us** — we flag this, pa11y does not (possible extension or pa11y gap)`,
    `- **⚠** — our finding is a heuristic warning, not a confirmed violation`,
  ];

  return lines.join('\n');
}

// ── Main ──────────────────────────────────────────────────────────────────────

(async () => {
  const ourJson = runAuditMySite(targetUrl, binPath, wcagLevel);
  const ourVersion = ourJson?.metadata?.tool || 'unknown';
  const ourMap = extractOurFindings(ourJson);

  const pa11yResults = await runPa11y(targetUrl, wcagLevel);
  const pa11yMap = extractPa11yFindings(pa11yResults);

  console.error(`[3/3] Comparing ${ourMap.size} our criteria vs ${pa11yMap.size} pa11y criteria ...`);

  const rows = buildTable(ourMap, pa11yMap);
  const markdown = renderMarkdown(rows, targetUrl, ourVersion);

  if (outputFile) {
    fs.mkdirSync(path.dirname(path.resolve(outputFile)), { recursive: true });
    fs.writeFileSync(outputFile, markdown, 'utf8');
    console.error(`Written to ${outputFile}`);
  } else {
    process.stdout.write(markdown + '\n');
  }
})();
