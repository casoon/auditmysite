#!/usr/bin/env python3
"""Pool benchmark (#856): wall time, peak RSS and failures of the real CLI
against a local fixture server. Measures; judges nothing.

    cargo build --release
    scripts/bench-pool.py [--binary PATH] [--repeats 3] [--out FILE.json]

Scenarios: one URL, a small batch (6 pages) and a larger batch (30 pages),
each at the concurrencies in SCENARIOS. The pages are generated here (clean
HTML, 5 to 55 kB, different per URL), so the numbers describe the browser
pool and the audit pipeline, not network or a third-party site.

Peak RSS is the sum over the whole process tree (auditmysite, Chrome and its
helpers), sampled every 100 ms with `ps`; a spike shorter than that can be
missed, so it is a lower bound on the true peak. A page counts as failed when
the batch report has fewer `pages` than `summary.attempted_url_count`
(`failed_url_count` in the report is the certification verdict, not an
error count).
"""

import argparse
import json
import platform
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

# (name, number of URLs, concurrency)
SCENARIOS = [
    ("single", 1, 1),
    ("small-c1", 6, 1),
    ("small-c3", 6, 3),
    ("large-c1", 30, 1),
    ("large-c3", 30, 3),
    ("large-c6", 30, 6),
]


def page(i: int) -> bytes:
    paragraphs = "".join(
        f"<p>Paragraph {n} of page {i}. Plain text for the audit to read.</p>"
        f'<a href="/p{(i + n) % 30}.html">Next page {n}</a>'
        for n in range(40 * (1 + i % 10))
    )
    return (
        f"<!doctype html><html lang=en><head><meta charset=utf-8>"
        f"<title>Bench page {i}</title></head><body><main><h1>Bench page {i}</h1>"
        f"{paragraphs}</main></body></html>"
    ).encode()


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        name = self.path.lstrip("/").removesuffix(".html").removeprefix("p")
        body = page(int(name)) if name.isdigit() else b"not found"
        self.send_response(200 if name.isdigit() else 404)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


def tree_rss_kb(root_pid: int) -> tuple[int, int]:
    """(sum of RSS over root and all descendants, RSS of root), in kB."""
    out = subprocess.run(
        ["ps", "-axo", "pid=,ppid=,rss="], capture_output=True, text=True
    ).stdout
    rows = {}
    for line in out.splitlines():
        parts = line.split()
        if len(parts) == 3:
            rows[int(parts[0])] = (int(parts[1]), int(parts[2]))
    members, grew = {root_pid}, True
    while grew:
        grew = False
        for pid, (ppid, _) in rows.items():
            if ppid in members and pid not in members:
                members.add(pid)
                grew = True
    return (
        sum(rows[p][1] for p in members if p in rows),
        rows.get(root_pid, (0, 0))[1],
    )


def run(binary: str, urls: list[str], concurrency: int, workdir: Path) -> dict:
    report = workdir / "report.json"
    report.unlink(missing_ok=True)
    if len(urls) == 1:
        args = [binary, urls[0]]
    else:
        url_file = workdir / "urls.txt"
        url_file.write_text("\n".join(urls) + "\n")
        args = [binary, "--url-file", str(url_file)]
    # --report-mode: exit 0 whenever the report was written, so a non-zero exit
    # here is a technical failure, not the verdict on the fixture pages.
    args += ["--report-mode", "--concurrency", str(concurrency), "--format", "json", "--output", str(report)]

    started = time.monotonic()
    proc = subprocess.Popen(args, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    peak_tree = peak_self = 0
    while proc.poll() is None:
        tree, own = tree_rss_kb(proc.pid)
        peak_tree, peak_self = max(peak_tree, tree), max(peak_self, own)
        time.sleep(0.1)
    wall = time.monotonic() - started
    stderr = proc.stderr.read()

    audited = None
    if report.exists():
        data = json.loads(report.read_text())
        audited = len(data["pages"]) if "pages" in data else 1
    return {
        "exit_code": proc.returncode,
        "wall_s": round(wall, 2),
        "peak_tree_rss_mb": round(peak_tree / 1024),
        "peak_cli_rss_mb": round(peak_self / 1024),
        "audited": audited,
        "failed": None if audited is None else len(urls) - audited,
        "stderr_tail": stderr.strip().splitlines()[-1:] if proc.returncode else [],
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--binary", default="target/release/auditmysite")
    ap.add_argument("--repeats", type=int, default=3)
    ap.add_argument("--out", help="write the results as JSON")
    ap.add_argument("--only", help="run only the scenario with this name")
    opts = ap.parse_args()

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}"

    version = subprocess.run([opts.binary, "--version"], capture_output=True, text=True).stdout.strip()
    commit = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    results = {
        "host": {"system": platform.system(), "machine": platform.machine(), "release": platform.release()},
        "tool": version,
        "commit": commit,
        "runs": [],
    }
    print(f"{version} @ {commit}, {platform.system()} {platform.machine()}", file=sys.stderr)

    with tempfile.TemporaryDirectory() as tmp:
        for name, count, concurrency in SCENARIOS:
            if opts.only and opts.only != name:
                continue
            urls = [f"{base}/p{i}.html" for i in range(count)]
            for n in range(opts.repeats):
                r = run(opts.binary, urls, concurrency, Path(tmp))
                r.update(scenario=name, urls=count, concurrency=concurrency, repeat=n + 1)
                results["runs"].append(r)
                print(
                    f"{name:9} run {n + 1}: {r['wall_s']:7.2f} s  tree RSS {r['peak_tree_rss_mb']:5} MB  "
                    f"failed {r['failed']}  exit {r['exit_code']}",
                    file=sys.stderr,
                )
    server.shutdown()
    if opts.out:
        Path(opts.out).write_text(json.dumps(results, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
