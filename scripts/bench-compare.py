#!/usr/bin/env python3
"""Head-to-head benchmark (#1041): auditmysite against pa11y, @axe-core/cli
and Lighthouse on the same pages. Measures wall time, peak memory of the
whole process tree and the overlap of the reported WCAG success criteria.
Measures; judges nothing. Run locally before a release, not in CI.

    cargo build --release
    scripts/bench-compare.py [--runs 5] [--full-runs 3] [--out FILE.json] [--markdown FILE.md]

Tools: pa11y, @axe-core/cli and Lighthouse are installed at the pinned
versions below into a throwaway directory (`--tools-dir`, default a temp
directory) with `npm install --ignore-scripts`, so nothing is installed
globally and no package downloads a browser. Every tool is pointed at the
same Chrome binary (`--chrome`, default: the one `auditmysite browser path`
selects). @axe-core/cli also needs a chromedriver matching that Chrome
(`--chromedriver`); without one, or when the probe run fails, it is skipped
and the reason is recorded.

Variants (tool and scope):
  auditmysite a11y   --modules a11y --interactive off (WCAG rules only)
  auditmysite full   the default run (every module, journeys on)
  pa11y htmlcs       the default runner (HTML_CodeSniffer, WCAG2AA)
  pa11y axe          --runner axe
  axe-core cli       WCAG 2.x A/AA tags
  lighthouse a11y    --only-categories=accessibility
  lighthouse full    every default category

Pages: three repository fixtures served from a local static server (same
bytes for every tool, no network) and three stable public pages
(`--no-public` drops them).

Method: one discarded warm-up run of every variant on the first page, then
`--runs` rounds (the two full-scope variants only in the first `--full-runs`,
they are the slow ones); each round runs every variant on every page, round-robin,
so a change in machine load spreads over all tools instead of hitting one.
Every run is a new process with a new browser and profile (no tool keeps a
browser between runs); the warm-up only warms the OS file cache. Wall time
runs from process start to process exit. Peak memory is the largest sum of
RSS over the process tree (the tool, its browser and every helper; members
are the descendants of the tool process plus its process group), sampled
every 100 ms with `ps`. RSS counts shared pages once per process, so the sum
overstates unique memory; it is the same measure for every tool. A spike
shorter than the interval can be missed. The peak RSS of the tree without
the browser processes (executable name contains "chrom": the tool itself, node
and any version-manager shim) is recorded next to it. The load average is recorded at the
start of every run: a busy machine invalidates the comparison.

Findings: from the last round, every tool's reported violations are reduced
to WCAG success criteria (axe rule ids through the tags of the axe-core
version that tool ships, HTML_CodeSniffer codes through the criterion in the
code, auditmysite through `wcag_criterion`) and compared per page against
auditmysite (a11y scope): shared, only auditmysite, only the other tool.
Findings without a criterion (best-practice rules) are counted separately.
"""

import argparse
import functools
import http.server
import json
import os
import platform
import re
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
PA11Y = "10.0.0"
AXE_CLI = "4.14.0"
LIGHTHOUSE = "13.6.0"
LOCAL_PAGES = [
    "tests/fixtures/perfect.html",
    "tests/fixtures/many_violations.html",
    "tests/fixtures/detection_corpus/forms_embedded.html",
]
PUBLIC_PAGES = [
    "https://example.com/",
    "https://www.gov.uk/",
    "https://www.w3.org/WAI/",
]
# Same scope as the auditmysite default (WCAG 2.2 A/AA), no best-practice rules.
AXE_TAGS = "wcag2a,wcag2aa,wcag21a,wcag21aa,wcag22aa"
RUN_TIMEOUT_S = 240
SAMPLE_S = 0.1


# --- process tree measurement -------------------------------------------------


def ps_rows() -> dict[int, tuple[int, int, int, bool]]:
    """pid -> (ppid, pgid, rss kB, is a browser process)."""
    out = subprocess.run(["ps", "-axo", "pid=,ppid=,pgid=,rss=,comm="], capture_output=True, text=True).stdout
    rows = {}
    for line in out.splitlines():
        parts = line.split(None, 4)
        if len(parts) == 5:
            browser = "chrom" in parts[4].lower()
            rows[int(parts[0])] = (int(parts[1]), int(parts[2]), int(parts[3]), browser)
    return rows


def tree_members(rows: dict, root: int) -> set[int]:
    members = {pid for pid, (_, pgid, _, _) in rows.items() if pgid == root} | {root}
    grew = True
    while grew:
        grew = False
        for pid, (ppid, _, _, _) in rows.items():
            if ppid in members and pid not in members:
                members.add(pid)
                grew = True
    return members & rows.keys()


def measure(args: list[str], env: dict, stdout_path: Path) -> dict:
    """Run one command in its own process group; wall time and peak tree RSS."""
    load = os.getloadavg()[0]
    with stdout_path.open("wb") as out:
        started = time.monotonic()
        proc = subprocess.Popen(
            args, stdout=out, stderr=subprocess.PIPE, env=env, start_new_session=True
        )
        stderr = []
        reader = threading.Thread(target=lambda: stderr.append(proc.stderr.read()), daemon=True)
        reader.start()
        peak_kb = peak_tool_kb = peak_procs = 0
        timed_out = False
        while proc.poll() is None:
            rows = ps_rows()
            members = tree_members(rows, proc.pid)
            peak_kb = max(peak_kb, sum(rows[p][2] for p in members))
            peak_tool_kb = max(peak_tool_kb, sum(rows[p][2] for p in members if not rows[p][3]))
            peak_procs = max(peak_procs, len(members))
            if time.monotonic() - started > RUN_TIMEOUT_S:
                timed_out = True
                os.killpg(proc.pid, signal.SIGKILL)
                break
            time.sleep(SAMPLE_S)
        proc.wait()
        wall = time.monotonic() - started
    reader.join(timeout=5)
    # A browser the tool left behind in its process group is killed and counted.
    leftovers = [pid for pid, (_, pgid, _, _) in ps_rows().items() if pgid == proc.pid]
    for pid in leftovers:
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    err = (stderr[0] if stderr else b"").decode(errors="replace").strip().splitlines()
    return {
        "wall_s": round(wall, 3),
        "peak_tree_rss_mb": round(peak_kb / 1024),
        "peak_tool_rss_mb": round(peak_tool_kb / 1024),
        "peak_procs": peak_procs,
        "exit_code": proc.returncode,
        "timed_out": timed_out,
        "leftover_procs": len(leftovers),
        "load_1m": round(load, 2),
        "stderr_tail": err[-2:],
    }


# --- findings -> WCAG success criteria ---------------------------------------

AXE_SC_TAG = re.compile(r"^wcag(\d)(\d)(\d+)$")
HTMLCS_SC = re.compile(r"(?<![\d_])(\d)_(\d+)_(\d+)(?![\d_])")


def axe_rule_map(axe_dir: Path) -> dict[str, list[str]]:
    """axe rule id -> WCAG success criteria, from that axe-core's own rule tags."""
    js = (
        f"const a=require({json.dumps(str(axe_dir))});"
        "console.log(JSON.stringify(Object.fromEntries(a.getRules().map(r=>[r.ruleId,r.tags]))))"
    )
    tags = json.loads(subprocess.run(["node", "-e", js], capture_output=True, text=True, check=True).stdout)
    return {rule: sc_from_axe_tags(t) for rule, t in tags.items()}


def sc_from_axe_tags(tags: list[str]) -> list[str]:
    out = []
    for tag in tags:
        m = AXE_SC_TAG.match(tag)
        if m:
            out.append(".".join(m.groups()))
    return out


def criteria(ids_to_sc: list[list[str]]) -> dict:
    scs, unmapped = set(), 0
    for sc in ids_to_sc:
        if sc:
            scs.update(sc)
        else:
            unmapped += 1
    return {"criteria": sorted(scs, key=sc_key), "without_criterion": unmapped}


def sc_key(sc: str) -> tuple:
    return tuple(int(p) for p in sc.split("."))


def findings_auditmysite(path: Path, _maps) -> dict:
    page = json.loads(path.read_text())["pages"][0]
    return criteria([[f["wcag_criterion"]] if f.get("wcag_criterion") else [] for f in page["findings"]])


def findings_pa11y(path: Path, maps, runner: str) -> dict:
    issues = [i for i in json.loads(path.read_text()) if i.get("type") == "error"]
    if runner == "axe":
        return criteria([maps["pa11y"].get(i["code"], []) for i in issues])
    return criteria([[".".join(m.groups())] if (m := HTMLCS_SC.search(i["code"])) else [] for i in issues])


def findings_axe_cli(path: Path, _maps) -> dict:
    pages = json.loads(path.read_text())
    return criteria([sc_from_axe_tags(v["tags"]) for v in pages[0]["violations"]])


def findings_lighthouse(path: Path, maps) -> dict:
    report = json.loads(path.read_text())
    refs = report["categories"]["accessibility"]["auditRefs"]
    failed = [
        r["id"]
        for r in refs
        if report["audits"][r["id"]].get("scoreDisplayMode") == "binary" and report["audits"][r["id"]]["score"] == 0
    ]
    return criteria([maps["lighthouse"].get(a, []) for a in failed])


# --- variants -----------------------------------------------------------------


def variants(opts, bins: dict, workdir: Path) -> list[dict]:
    pa11y_config = workdir / "pa11y.json"
    pa11y_config.write_text(json.dumps({"chromeLaunchConfig": {"executablePath": opts.chrome}}))
    ams = [opts.binary, "{url}", "--browser-path", opts.chrome, "-f", "json", "-o", "{out}", "--report-mode", "--quiet"]
    lh = [
        bins["lighthouse"], "{url}", "--output=json", "--output-path={out}", "--quiet",
        "--chrome-flags=--headless=new",
    ]
    pa11y = [bins["pa11y"], "--config", str(pa11y_config), "--reporter", "json"]
    axe = [
        bins["axe"], "{url}", "--tags", AXE_TAGS, "--stdout",
        "--chrome-path", opts.chrome, "--chromedriver-path", opts.chromedriver or "",
    ]
    return [
        {"name": "auditmysite a11y", "tool": "auditmysite", "scope": "a11y",
         "cmd": ams + ["--modules", "a11y", "--interactive", "off"], "out": "file",
         "findings": findings_auditmysite},
        {"name": "auditmysite full", "tool": "auditmysite", "scope": "full", "cmd": ams, "out": "file"},
        {"name": "pa11y htmlcs", "tool": "pa11y", "scope": "a11y", "cmd": pa11y + ["{url}"], "out": "stdout",
         "findings": functools.partial(findings_pa11y, runner="htmlcs")},
        {"name": "pa11y axe", "tool": "pa11y", "scope": "a11y", "cmd": pa11y + ["--runner", "axe", "{url}"],
         "out": "stdout", "findings": functools.partial(findings_pa11y, runner="axe")},
        {"name": "axe-core cli", "tool": "axe", "scope": "a11y", "cmd": axe, "out": "stdout",
         "findings": findings_axe_cli},
        {"name": "lighthouse a11y", "tool": "lighthouse", "scope": "a11y",
         "cmd": lh + ["--only-categories=accessibility"], "out": "file", "findings": findings_lighthouse},
        {"name": "lighthouse full", "tool": "lighthouse", "scope": "full", "cmd": lh, "out": "file"},
    ]


def run_variant(variant: dict, url: str, env: dict, workdir: Path, tag: str) -> tuple[dict, Path]:
    out = workdir / f"{tag}.json"
    out.unlink(missing_ok=True)
    stdout = out if variant["out"] == "stdout" else workdir / f"{tag}.stdout"
    args = [a.replace("{url}", url).replace("{out}", str(out)) for a in variant["cmd"]]
    result = measure(args, env, stdout)
    try:
        json.loads(out.read_text())
        result["ok"] = not result["timed_out"]
    except (OSError, ValueError):
        result["ok"] = False
    return result, out


# --- environment ----------------------------------------------------------------


def cmd_out(args: list[str], **kw) -> str:
    try:
        return subprocess.run(args, capture_output=True, text=True, timeout=60, **kw).stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        return ""


def machine() -> dict:
    info = {"os": f"{platform.system()} {platform.release()}", "arch": platform.machine(), "cpus": os.cpu_count()}
    if platform.system() == "Darwin":
        info["os"] = f"macOS {cmd_out(['sw_vers', '-productVersion'])} ({platform.release()})"
        info["cpu"] = cmd_out(["sysctl", "-n", "machdep.cpu.brand_string"])
        info["ram_gb"] = round(int(cmd_out(["sysctl", "-n", "hw.memsize"]) or 0) / 2**30)
    elif Path("/proc/meminfo").exists():
        cpuinfo = Path("/proc/cpuinfo").read_text()
        info["cpu"] = next((l.split(":", 1)[1].strip() for l in cpuinfo.splitlines() if l.startswith("model name")), "")
        mem_kb = int(Path("/proc/meminfo").read_text().split()[1])
        info["ram_gb"] = round(mem_kb / 2**20)
    return info


def install_tools(tools_dir: Path) -> dict:
    bins = {name: tools_dir / "node_modules/.bin" / name for name in ("pa11y", "axe", "lighthouse")}
    if not all(b.exists() for b in bins.values()):
        print(f"installing tools into {tools_dir} ...", file=sys.stderr)
        subprocess.run(
            ["npm", "install", "--prefix", str(tools_dir), "--no-audit", "--no-fund", "--ignore-scripts",
             f"pa11y@{PA11Y}", f"@axe-core/cli@{AXE_CLI}", f"lighthouse@{LIGHTHOUSE}"],
            check=True, stdout=subprocess.DEVNULL,
        )
    return {k: str(v) for k, v in bins.items()}


def axe_core_dir(tools_dir: Path, owner: str) -> Path:
    nested = tools_dir / "node_modules" / owner / "node_modules/axe-core"
    return nested if nested.exists() else tools_dir / "node_modules/axe-core"


def package_version(path: Path) -> str:
    return json.loads((path / "package.json").read_text())["version"]


# --- summary --------------------------------------------------------------------


def summarize(runs: list[dict], names: list[str]) -> list[dict]:
    rows = []
    for name in names:
        mine = [r for r in runs if r["variant"] == name]
        ok = [r for r in mine if r["ok"]]
        if not ok:
            rows.append({"variant": name, "runs": len(mine), "ok": 0})
            continue
        walls = [r["wall_s"] for r in ok]
        mems = [r["peak_tree_rss_mb"] for r in ok]
        rows.append({
            "variant": name,
            "runs": len(mine),
            "ok": len(ok),
            "median_s": round(statistics.median(walls), 2),
            "min_s": round(min(walls), 2),
            "max_s": round(max(walls), 2),
            "median_peak_mb": round(statistics.median(mems)),
            "max_peak_mb": max(mems),
            "median_tool_mb": round(statistics.median(r["peak_tool_rss_mb"] for r in ok)),
            "per_url_median_s": {
                url: round(statistics.median([r["wall_s"] for r in ok if r["url"] == url]), 2)
                for url in dict.fromkeys(r["url"] for r in ok)
            },
        })
    fastest = min((r["median_s"] for r in rows if r.get("ok")), default=None)
    for r in rows:
        if r.get("ok"):
            r["factor"] = round(r["median_s"] / fastest, 1)
    return rows


def overlap(findings: dict, names: list[str], urls: list[str]) -> list[dict]:
    base = "auditmysite a11y"
    out = []
    for name in names:
        if name == base or name not in findings:
            continue
        row = {"variant": name, "pages": 0, "shared": 0, "only_auditmysite": 0, "only_other": 0,
               "other_without_criterion": 0, "per_url": {}}
        for url in urls:
            a, b = findings[base].get(url), findings[name].get(url)
            if a is None or b is None:
                continue
            sa, sb = set(a["criteria"]), set(b["criteria"])
            row["pages"] += 1
            row["shared"] += len(sa & sb)
            row["only_auditmysite"] += len(sa - sb)
            row["only_other"] += len(sb - sa)
            row["other_without_criterion"] += b["without_criterion"]
            row["per_url"][url] = {
                "shared": sorted(sa & sb, key=sc_key),
                "only_auditmysite": sorted(sa - sb, key=sc_key),
                "only_other": sorted(sb - sa, key=sc_key),
            }
        out.append(row)
    return out


def markdown(result: dict) -> str:
    lines = [
        "| Tool and scope | Median s/page | Factor | Range s | Peak tree RSS MB (median / max) "
        "| Without browser RSS MB (median) | Runs ok |",
        "|---|---|---|---|---|---|---|",
    ]
    for r in result["summary"]:
        if not r.get("ok"):
            reason = result["skipped"].get(r["variant"], "no successful run")
            lines.append(f"| {r['variant']} | — | — | — | — | — | 0 / {r['runs']} ({reason}) |")
            continue
        lines.append(
            f"| {r['variant']} | {r['median_s']:.2f} | {r['factor']:.1f}× | {r['min_s']:.2f} – {r['max_s']:.2f} "
            f"| {r['median_peak_mb']} / {r['max_peak_mb']} | {r['median_tool_mb']} | {r['ok']} / {r['runs']} |"
        )
    for name, reason in result["skipped"].items():
        if not any(r["variant"] == name for r in result["summary"]):
            lines.append(f"| {name} | — | — | — | — | — | skipped: {reason[:90]} |")
    lines += ["", "Median seconds per page and URL:", ""]
    measured = [r for r in result["summary"] if r.get("ok")]
    urls = result["urls"]
    lines.append("| URL | " + " | ".join(r["variant"] for r in measured) + " |")
    lines.append("|---" * (len(measured) + 1) + "|")
    for url in urls:
        cells = [f"{r['per_url_median_s'][url]:.2f}" if url in r["per_url_median_s"] else "—" for r in measured]
        lines.append(f"| {short_url(url)} | " + " | ".join(cells) + " |")
    if result["overlap"]:
        lines += [
            "",
            "WCAG success criteria reported against auditmysite a11y (sum over pages):",
            "",
            "| Tool | Pages | Shared | Only auditmysite | Only tool | Tool findings without criterion |",
            "|---|---|---|---|---|---|",
        ]
        for o in result["overlap"]:
            lines.append(
                f"| {o['variant']} | {o['pages']} | {o['shared']} | {o['only_auditmysite']} | {o['only_other']} "
                f"| {o['other_without_criterion']} |"
            )
    return "\n".join(lines) + "\n"


def short_url(url: str) -> str:
    return re.sub(r"^http://127\.0\.0\.1:\d+/", "local: ", url)


# --- main -----------------------------------------------------------------------


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--runs", type=int, default=5, help="measured rounds (default 5)")
    ap.add_argument("--full-runs", type=int, default=3, help="rounds of the full-scope variants (default 3)")
    ap.add_argument("--binary", default=None, help="auditmysite binary (default: release build in the cargo target dir)")
    ap.add_argument("--chrome", default=None, help="Chrome binary for every tool (default: auditmysite browser path)")
    ap.add_argument("--chromedriver", default=shutil.which("chromedriver"), help="chromedriver for @axe-core/cli")
    ap.add_argument("--tools-dir", default=None, help="npm prefix for the pinned tools (default: temp dir)")
    ap.add_argument("--no-public", action="store_true", help="local fixtures only")
    ap.add_argument("--only", help="comma-separated variant names to run")
    ap.add_argument("--out", help="write the full result as JSON")
    ap.add_argument("--markdown", help="write the tables as Markdown")
    opts = ap.parse_args()

    if opts.binary is None:
        meta = json.loads(cmd_out(["cargo", "metadata", "--format-version", "1", "--no-deps"], cwd=REPO))
        opts.binary = str(Path(meta["target_directory"]) / "release/auditmysite")
    if opts.chrome is None:
        opts.chrome = cmd_out([opts.binary, "browser", "path"]).splitlines()[-1].strip()

    tmp = tempfile.TemporaryDirectory()
    workdir = Path(tmp.name)
    tools_dir = Path(opts.tools_dir) if opts.tools_dir else workdir / "tools"
    bins = install_tools(tools_dir)
    env = dict(os.environ, CHROME_PATH=opts.chrome, PUPPETEER_SKIP_DOWNLOAD="1")

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(QuietHandler, directory=str(REPO)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}"
    urls = [f"{base}/{p}" for p in LOCAL_PAGES] + ([] if opts.no_public else PUBLIC_PAGES)

    all_variants = variants(opts, bins, workdir)
    if opts.only:
        wanted = {n.strip() for n in opts.only.split(",")}
        all_variants = [v for v in all_variants if v["name"] in wanted]

    maps = {
        "pa11y": axe_rule_map(axe_core_dir(tools_dir, "pa11y")),
        "lighthouse": axe_rule_map(axe_core_dir(tools_dir, "lighthouse")),
    }
    result = {
        "machine": machine(),
        "versions": {
            "auditmysite": cmd_out([opts.binary, "--version"]),
            "commit": cmd_out(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"]),
            "chrome": cmd_out([opts.chrome, "--version"]),
            "chrome_path": opts.chrome,
            "node": cmd_out(["node", "--version"]),
            "pa11y": cmd_out([bins["pa11y"], "--version"]),
            "axe_cli": cmd_out([bins["axe"], "--version"]),
            "lighthouse": cmd_out([bins["lighthouse"], "--version"]),
            "axe_core_in_pa11y": package_version(axe_core_dir(tools_dir, "pa11y")),
            "axe_core_in_lighthouse": package_version(axe_core_dir(tools_dir, "lighthouse")),
            "axe_core_in_axe_cli": package_version(axe_core_dir(tools_dir, "@axe-core/cli")),
            "chromedriver": cmd_out([opts.chromedriver, "--version"]) if opts.chromedriver else "",
        },
        "settings": {"runs": opts.runs, "full_runs": min(opts.full_runs, opts.runs), "warmup": 1, "sample_interval_s": SAMPLE_S, "timeout_s": RUN_TIMEOUT_S},
        "urls": urls,
        "skipped": {},
        "runs": [],
    }
    print(json.dumps(result["machine"]), file=sys.stderr)
    print(json.dumps(result["versions"], indent=1), file=sys.stderr)
    started = time.monotonic()

    # Warm-up on the first page; it also probes whether a tool runs at all here.
    active = []
    for v in all_variants:
        if v["tool"] == "axe" and not opts.chromedriver:
            result["skipped"][v["name"]] = "no chromedriver"
            continue
        r, _ = run_variant(v, urls[0], env, workdir, "warmup")
        if not r["ok"]:
            result["skipped"][v["name"]] = " ".join(r["stderr_tail"])[:200] or f"exit {r['exit_code']}"
            print(f"skip {v['name']}: {result['skipped'][v['name']]}", file=sys.stderr)
            continue
        active.append(v)

    findings: dict[str, dict] = {}
    for n in range(opts.runs):
        last = n == opts.runs - 1
        for url in urls:
            for v in active:
                if v["scope"] == "full" and n >= opts.full_runs:
                    continue
                r, out = run_variant(v, url, env, workdir, "run")
                r.update(variant=v["name"], url=url, round=n + 1)
                result["runs"].append(r)
                print(
                    f"round {n + 1} {v['name']:17} {short_url(url):45} {r['wall_s']:7.2f} s "
                    f"{r['peak_tree_rss_mb']:6} MB load {r['load_1m']:5.2f} {'ok' if r['ok'] else 'FAILED'}",
                    file=sys.stderr,
                )
                if last and r["ok"] and "findings" in v:
                    findings.setdefault(v["name"], {})[url] = v["findings"](out, maps)
    server.shutdown()

    names = [v["name"] for v in active]
    result["elapsed_min"] = round((time.monotonic() - started) / 60, 1)
    result["summary"] = summarize(result["runs"], names)
    result["findings"] = findings
    result["overlap"] = overlap(findings, names, urls)
    table = markdown(result)
    print(table)
    if opts.out:
        Path(opts.out).write_text(json.dumps(result, indent=1) + "\n")
    if opts.markdown:
        Path(opts.markdown).write_text(table)
    tmp.cleanup()
    return 0


if __name__ == "__main__":
    sys.exit(main())
