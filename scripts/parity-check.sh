#!/usr/bin/env bash
# On-demand parity check: auditmysite vs axe-core vs pa11y
# Usage: ./scripts/parity-check.sh <URL>
set -euo pipefail

URL="${1:?Usage: $0 <URL>}"
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
REPORTS="$REPO_ROOT/reports"
CONTRACT="$REPO_ROOT/docs/PARITY_CONTRACT.jsonc"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

DOMAIN=$(echo "$URL" | sed -E 's|https?://||;s|/.*||;s|^www\.||')
DATE=$(date +%Y-%m-%d)
OUT="$REPORTS/parity-${DOMAIN}-${DATE}.md"

CHROMEDRIVER="/Users/jseidel/.browser-driver-manager/chromedriver/mac_arm-149.0.7827.54/chromedriver-mac-arm64/chromedriver"
BINARY=""
for candidate in \
  "$REPO_ROOT/target/release/auditmysite" \
  "$REPO_ROOT/target/debug/auditmysite" \
  "$HOME/.local/bin/auditmysite" \
  "$(which auditmysite 2>/dev/null)"; do
  if [ -f "$candidate" ] && [ -x "$candidate" ]; then
    BINARY="$candidate"
    break
  fi
done

echo "Parity check: $URL"
echo

# --- auditmysite ---
echo "[1/3] auditmysite..."
if [ -z "$BINARY" ]; then
  echo "  skipped (binary not found — run: cargo build)"
else
  echo "  using: $BINARY"
  "$BINARY" "$URL" --format json --output "$TMP/ams.json" 2>/dev/null || true
fi

# --- axe-core ---
echo "[2/3] axe-core..."
if [ -f "$CHROMEDRIVER" ]; then
  npx --yes @axe-core/cli "$URL" \
    --chromedriver-path "$CHROMEDRIVER" \
    --dir "$TMP/axe" 2>/dev/null || true
else
  npx --yes @axe-core/cli "$URL" --dir "$TMP/axe" 2>/dev/null || true
fi

# --- pa11y ---
echo "[3/3] pa11y..."
npx --yes pa11y "$URL" --reporter json > "$TMP/pa11y.json" 2>/dev/null || true

echo "Comparing..."

python3 - "$TMP" "$URL" "$OUT" "$CONTRACT" << 'PYEOF'
import json, sys, glob, os
from collections import defaultdict

tmp, url, out, contract_path = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
with open(contract_path) as f:
    contract = json.load(f)

# ── load auditmysite ─────────────────────────────────────────────────────────
ams_violations = []
ams_path = os.path.join(tmp, "ams.json")
if os.path.exists(ams_path):
    with open(ams_path) as f:
        ams = json.load(f)
    pages = ams.get("pages", [ams])
    for page in pages:
        findings = page.get("detail", page).get("findings", []) or page.get("findings", [])
        for fi in findings:
            ams_violations.append({
                "rule": fi.get("rule_id", "?"),
                "axe_id": fi.get("axe_id", ""),
                "wcag": fi.get("wcag_criterion", ""),
                "severity": fi.get("severity", "?"),
                "count": fi.get("occurrence_count", 1),
                "title": fi.get("title", ""),
            })

# ── load axe-core ────────────────────────────────────────────────────────────
axe_violations, axe_passes, axe_incomplete = [], [], []
axe_files = glob.glob(os.path.join(tmp, "axe", "*.json"))
if axe_files:
    with open(axe_files[0]) as f:
        axe_raw = json.load(f)
    results = axe_raw if isinstance(axe_raw, list) else [axe_raw]
    for r in results:
        for v in r.get("violations", []):
            axe_violations.append({
                "rule": v["id"],
                "impact": v.get("impact", "?"),
                "count": len(v.get("nodes", [])),
                "tags": [t for t in v.get("tags", []) if "wcag" in t],
            })
        for p in r.get("passes", []):
            axe_passes.append(p["id"])
        for i in r.get("incomplete", []):
            axe_incomplete.append({
                "rule": i["id"],
                "impact": i.get("impact", "?"),
                "count": len(i.get("nodes", [])),
            })

# ── load pa11y ───────────────────────────────────────────────────────────────
pa11y_issues = []
pa11y_path = os.path.join(tmp, "pa11y.json")
if os.path.exists(pa11y_path):
    with open(pa11y_path) as f:
        raw = json.load(f)
    items = raw if isinstance(raw, list) else raw.get("issues", [])
    by_code = defaultdict(list)
    for i in items:
        by_code[i["code"]].append(i)
    for code, items in by_code.items():
        pa11y_issues.append({
            "code": code,
            "type": items[0]["type"],
            "count": len(items),
            "msg": items[0]["message"][:80],
        })

# ── build report ─────────────────────────────────────────────────────────────
known_pairs = contract.get("axe_rule_mappings", {})
frozen = contract.get("frozen_numbers", {})
methodology = contract.get("methodology", {})

def ams_matches_axe(axe_rule, ams_finding):
    ams_rule = ams_finding.get("rule", "")
    ams_axe_id = ams_finding.get("axe_id", "")
    if axe_rule == ams_rule or axe_rule == ams_axe_id:
        return True
    return any(pattern in ams_rule for pattern in known_pairs.get(axe_rule, []))

def ams_matches_for_axe(axe_rule):
    return [v for v in ams_violations if ams_matches_axe(axe_rule, v)]

lines = []
lines.append(f"# Parity Check — {url}")
lines.append(f"\n## Summary\n")
lines.append(f"| Tool | Violations | Notes |")
lines.append(f"|---|---|---|")
lines.append(f"| auditmysite | {len(ams_violations)} findings | AX-tree based |")
lines.append(f"| axe-core | {len(axe_violations)} confirmed + {len(axe_incomplete)} incomplete | DOM based |")
lines.append(f"| pa11y | {len(pa11y_issues)} issues | htmlcs runner |")

lines.append("\n## Methodology Contract\n")
lines.append(f"- Contract schema: `{contract.get('schema_version', '?')}`")
lines.append(f"- Primary audit engine: {methodology.get('primary_audit_engine', 'n/a')}")
tools = methodology.get("comparison_tools", [])
if tools:
    lines.append(f"- Comparison tools: {', '.join(tools)}")
policy = methodology.get("candidate_policy", {})
for key in ["auditmysite_only", "axe_core_only", "axe_incomplete"]:
    if key in policy:
        lines.append(f"- {key}: {policy[key]}")

lines.append("\n## Frozen Landing Page Numbers\n")
lines.append("| Metric | Value |")
lines.append("|---|---:|")
for key in [
    "wcag_aa_total_criteria",
    "automated_wcag_aa_criteria",
    "manual_review_criteria",
]:
    if key in frozen:
        lines.append(f"| {key} | {frozen[key]} |")
if frozen.get("stable_parity_fixture"):
    lines.append(f"\nStable fixture: `{frozen['stable_parity_fixture']}`")
expected = frozen.get("stable_parity_fixture_expected_axe_ids", [])
if expected:
    lines.append(f"Expected fixture axe IDs: {', '.join(f'`{item}`' for item in expected)}")

# ── axe-core violations ──────────────────────────────────────────────────────
lines.append(f"\n## axe-core — Confirmed Violations ({len(axe_violations)})\n")
if axe_violations:
    lines.append("| Impact | Rule | Nodes | WCAG |")
    lines.append("|---|---|---|---|")
    for v in sorted(axe_violations, key=lambda x: ["critical","serious","moderate","minor"].index(x["impact"]) if x["impact"] in ["critical","serious","moderate","minor"] else 9):
        lines.append(f"| {v['impact']} | `{v['rule']}` | {v['count']} | {', '.join(v['tags'])} |")
else:
    lines.append("_None_")

# ── axe-core incomplete ──────────────────────────────────────────────────────
lines.append(f"\n## axe-core — Incomplete / Manual Review Needed ({len(axe_incomplete)})\n")
if axe_incomplete:
    lines.append("| Impact | Rule | Nodes |")
    lines.append("|---|---|---|")
    for v in axe_incomplete:
        lines.append(f"| {v['impact']} | `{v['rule']}` | {v['count']} |")

# ── pa11y ────────────────────────────────────────────────────────────────────
lines.append(f"\n## pa11y — Issues ({len(pa11y_issues)})\n")
if pa11y_issues:
    lines.append("| Type | Code | Count | Message |")
    lines.append("|---|---|---|---|")
    for i in pa11y_issues:
        lines.append(f"| {i['type']} | `{i['code']}` | {i['count']} | {i['msg']} |")
else:
    lines.append("_None_")

# ── auditmysite findings ─────────────────────────────────────────────────────
lines.append(f"\n## auditmysite — Findings ({len(ams_violations)})\n")
axe_pass_set = set(axe_passes)
axe_viol_set = {v["rule"] for v in axe_violations}
if ams_violations:
    lines.append("| Severity | Rule | axe ID | WCAG | Count | axe-core status |")
    lines.append("|---|---|---|---|---|---|")
    for v in sorted(ams_violations, key=lambda x: ["critical","high","medium","low"].index(x["severity"]) if x["severity"] in ["critical","high","medium","low"] else 9):
        axe_note = ""
        if any(ams_matches_axe(axe_rule, v) for axe_rule in axe_viol_set):
            axe_note = "⚠️ also violation"
        elif any(ams_matches_axe(axe_rule, v) for axe_rule in axe_pass_set):
            axe_note = "✅ PASS"
        lines.append(f"| {v['severity']} | `{v['rule']}` | `{v['axe_id']}` | {v['wcag']} | {v['count']} | {axe_note} |")

# ── divergence section ───────────────────────────────────────────────────────
lines.append(f"\n## Divergence Notes\n")

false_positive_candidates = []
for axe_rule, ams_patterns in known_pairs.items():
    if axe_rule in axe_pass_set:
        matching = ams_matches_for_axe(axe_rule)
        if matching:
            total = sum(v["count"] for v in matching)
            false_positive_candidates.append(f"- **`{axe_rule}` PASS in axe-core** but auditmysite reports {total} occurrence(s) via {[v['rule'] for v in matching]} — investigate for false positives")

false_negative_candidates = []
for axe_violation in axe_violations:
    axe_rule = axe_violation["rule"]
    if not ams_matches_for_axe(axe_rule):
        false_negative_candidates.append(
            f"- **`{axe_rule}` violation in axe-core** ({axe_violation['impact']}, {axe_violation['count']} node(s)) has no mapped auditmysite finding — investigate for false negatives"
        )

lines.append(f"### auditmysite-only candidates ({len(false_positive_candidates)})\n")
if false_positive_candidates:
    lines.extend(false_positive_candidates)
else:
    lines.append("_None_")

lines.append(f"\n### axe-core-only candidates ({len(false_negative_candidates)})\n")
if false_negative_candidates:
    lines.extend(false_negative_candidates)
else:
    lines.append("_None_")

report = "\n".join(lines)
with open(out, "w") as f:
    f.write(report)
print(report)
PYEOF

echo
echo "Saved: $OUT"
