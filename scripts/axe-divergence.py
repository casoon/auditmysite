#!/usr/bin/env python3
"""Divergence between auditmysite's accessibility score and axe-core (plan 47, step 3).

Runs auditmysite and axe-core over every fixture page (detection corpus + WCAG
fixtures) and records, per page, the auditmysite score next to axe's violations
by impact. The goal is not agreement -- the rule sets differ on purpose -- but
a visible signal when a scoring change moves auditmysite away from an outside
reference on the same markup.

One number summarises it: the Spearman rank correlation between the
auditmysite score and axe's impact-weighted violation load (expected to be
negative: more axe load, lower score).

    scripts/axe-divergence.py            # run, compare against the baseline
    scripts/axe-divergence.py --update   # run, write the baseline

Needs a release binary (cargo build --release), Chrome, node/npx.
"""

import argparse
import functools
import http.server
import json
import os
import subprocess
import sys
import tempfile
import threading
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
FIXTURE_DIRS = [REPO / "tests/fixtures/detection_corpus", REPO / "tests/wcag_fixtures"]
BASELINE = REPO / "tests/fixtures/axe_divergence_baseline.json"
BINARY = REPO / "target/release/auditmysite"
# Same scope as the auditmysite default (WCAG 2.2 A/AA), no best-practice rules.
AXE_TAGS = "wcag2a,wcag2aa,wcag21a,wcag21aa,wcag22aa"
IMPACT_WEIGHT = {"critical": 4, "serious": 3, "moderate": 2, "minor": 1}
# A score move of at least this many points while axe saw the same thing is listed.
MOVE_THRESHOLD = 5


def serve(root: Path) -> tuple[http.server.ThreadingHTTPServer, int]:
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(root))
    handler.log_message = lambda *a: None
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, server.server_address[1]


def auditmysite_score(url: str, out: Path) -> int | None:
    subprocess.run(
        [str(BINARY), url, "-f", "json", "-o", str(out), "--interactive", "off"],
        capture_output=True,
    )
    try:
        return json.loads(out.read_text())["pages"][0]["accessibility_score"]
    except (OSError, KeyError, IndexError, ValueError):
        return None


def axe_results(urls: list[str]) -> dict[str, dict[str, int]]:
    proc = subprocess.run(
        ["npx", "--yes", "@axe-core/cli", *urls, "--tags", AXE_TAGS, "--stdout"],
        capture_output=True,
        text=True,
    )
    results = {}
    for page in json.loads(proc.stdout):
        counts = {impact: 0 for impact in IMPACT_WEIGHT}
        for violation in page["violations"]:
            counts[violation["impact"]] = counts.get(violation["impact"], 0) + 1
        results[page["url"]] = counts
    return results


def ranks(values: list[float]) -> list[float]:
    order = sorted(range(len(values)), key=lambda i: values[i])
    out = [0.0] * len(values)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and values[order[j + 1]] == values[order[i]]:
            j += 1
        for k in range(i, j + 1):
            out[order[k]] = (i + j) / 2  # average rank for ties
        i = j + 1
    return out


def spearman(xs: list[float], ys: list[float]) -> float:
    n = len(xs)
    if n < 2:
        return 0.0
    rx, ry = ranks(xs), ranks(ys)
    mx, my = sum(rx) / n, sum(ry) / n
    cov = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    sx = sum((a - mx) ** 2 for a in rx) ** 0.5
    sy = sum((b - my) ** 2 for b in ry) ** 0.5
    return cov / (sx * sy) if sx and sy else 0.0


def axe_load(counts: dict[str, int]) -> int:
    return sum(IMPACT_WEIGHT[k] * v for k, v in counts.items())


def run() -> dict:
    if not BINARY.exists():
        sys.exit(f"missing {BINARY} — run: cargo build --release")
    pages = {}
    with tempfile.TemporaryDirectory() as tmp:
        for root in FIXTURE_DIRS:
            server, port = serve(root)
            try:
                files = sorted(p.name for p in root.glob("*.html"))
                urls = {f: f"http://127.0.0.1:{port}/{f}" for f in files}
                axe = axe_results(list(urls.values()))
                for name, url in urls.items():
                    key = f"{root.name}/{name}"
                    pages[key] = {
                        "auditmysite_score": auditmysite_score(url, Path(tmp) / f"{name}.json"),
                        "axe": axe.get(url),
                    }
                    print(f"  {key}: {pages[key]['auditmysite_score']} / axe {pages[key]['axe']}")
            finally:
                server.shutdown()
    usable = [p for p in pages.values() if p["auditmysite_score"] is not None and p["axe"]]
    rho = spearman([p["auditmysite_score"] for p in usable], [axe_load(p["axe"]) for p in usable])
    return {"axe_tags": AXE_TAGS, "spearman_score_vs_axe_load": round(rho, 3), "pages": pages}


def compare(current: dict, baseline: dict) -> None:
    print(
        f"\nSpearman(score, axe load): baseline {baseline['spearman_score_vs_axe_load']} "
        f"-> now {current['spearman_score_vs_axe_load']}  (more negative = closer to axe)"
    )
    moved = []
    for key, now in current["pages"].items():
        before = baseline["pages"].get(key)
        if not before or now["auditmysite_score"] is None or before["auditmysite_score"] is None:
            continue
        delta = now["auditmysite_score"] - before["auditmysite_score"]
        if abs(delta) >= MOVE_THRESHOLD and now["axe"] == before["axe"]:
            moved.append((abs(delta), key, before["auditmysite_score"], now["auditmysite_score"]))
    if moved:
        print(f"\nScore moved by >= {MOVE_THRESHOLD} while axe saw the same page:")
        for _, key, a, b in sorted(moved, reverse=True):
            print(f"  {key}: {a} -> {b}")
    else:
        print(f"\nNo page moved by >= {MOVE_THRESHOLD} points while axe saw the same.")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--update", action="store_true", help="write the baseline")
    args = parser.parse_args()
    current = run()
    if args.update or not BASELINE.exists():
        BASELINE.write_text(json.dumps(current, indent=2, sort_keys=True) + "\n")
        print(f"\nbaseline written: {BASELINE.relative_to(REPO)} "
              f"(Spearman {current['spearman_score_vs_axe_load']})")
        return
    compare(current, json.loads(BASELINE.read_text()))


if __name__ == "__main__":
    main()
