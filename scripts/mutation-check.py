#!/usr/bin/env python3
"""Prove detector test strength with the targeted mutations of the rule
contracts (#783).

Every local mutation in tests/fixtures/rule_contracts/*.toml (`find` /
`replace` on a file of this repository) is applied in turn, the test named
in `killed_by` is run, and the mutation must turn it red. The file is
restored afterwards, also on interrupt. Mutations whose target lives in
barrierlab are verified there, by barrierlab's own catalog, and skipped.

    scripts/mutation-check.py                 # all local mutations
    scripts/mutation-check.py --rule image-alt
    scripts/mutation-check.py --list

A mutation only counts as killed when the named test is green on the
unmutated code first and the mutated code still compiles: a test that is
red anyway, or a mutation that breaks the build, proves nothing and is
reported as invalid. Mutations run one at a time, in place, each under the
global build lock the interactive `cargo` wrapper uses.

Exit code 1 if any mutation survives or is invalid. Needs Chrome for
`corpus:` mutations, like the corpus test itself.
"""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import tomllib

ROOT = Path(__file__).resolve().parent.parent
CONTRACTS = ROOT / "tests" / "fixtures" / "rule_contracts"


def mutations(rule_filter):
    for path in sorted(CONTRACTS.rglob("*.toml")):
        with path.open("rb") as f:
            data = tomllib.load(f)
        for rule in data.get("rule", []):
            if rule_filter and rule["id"] not in rule_filter:
                continue
            for m in rule.get("test_strength", {}).get("mutations", []):
                if m["target"].startswith("barrierlab:"):
                    continue
                yield path.stem, rule["id"], m


def command(killed_by):
    kind, _, arg = killed_by.partition(":")
    env = dict(os.environ)
    if kind == "corpus":
        env["CORPUS_CASES"] = arg
        return ["cargo", "test", "--test", "detection_corpus_test", "--",
                "--ignored", "--test-threads=1"], env
    if kind == "test":
        binary, _, name = arg.partition("::")
        cmd = ["cargo", "test", "--test", binary]
        if name:
            cmd += [name]
        return cmd + ["--", "--include-ignored", "--test-threads=1"], env
    if kind == "lib":
        return ["cargo", "test", "--lib", arg], env
    raise ValueError(f"unknown killed_by kind: {killed_by}")


LOCK = Path.home() / ".cache" / "cargo-heavy-build.lock"


class BuildLock:
    """The `mkdir` lock of the interactive cargo wrapper, when present."""

    def __enter__(self):
        LOCK.parent.mkdir(parents=True, exist_ok=True)
        while True:
            try:
                LOCK.mkdir()
                break
            except FileExistsError:
                pid_file = LOCK / "pid"
                try:
                    os.kill(int(pid_file.read_text().strip()), 0)
                except (OSError, ValueError):
                    shutil.rmtree(LOCK, ignore_errors=True)
                    continue
                time.sleep(2)
        (LOCK / "pid").write_text(str(os.getpid()))
        return self

    def __exit__(self, *exc):
        shutil.rmtree(LOCK, ignore_errors=True)


def build_command(cmd):
    """The same cargo invocation, compiling only."""
    head = cmd[: cmd.index("--")] if "--" in cmd else cmd
    if head[:2] == ["cargo", "test"]:
        # A test-name filter after the target is meaningless for --no-run.
        head = [a for i, a in enumerate(head)
                if i < 2 or a.startswith("-") or head[i - 1] in ("--test", "--lib")
                or a == "--lib"]
    return head + ["--no-run"]


class TimedOut(Exception):
    pass


def passes(cmd, env, timeout=1800):
    try:
        return subprocess.run(cmd, cwd=ROOT, env=env, stdout=subprocess.DEVNULL,
                              stderr=subprocess.DEVNULL, timeout=timeout).returncode == 0
    except subprocess.TimeoutExpired as e:
        raise TimedOut(" ".join(cmd)) from e


baseline_cache = {}


def run_one(domain, rule_id, m):
    target = ROOT / m["target"]
    original = target.read_text()
    label = f"{domain}/{rule_id}"
    if original.count(m["find"]) != 1:
        return f"{label}: INVALID — `find` does not occur exactly once in {m['target']}"
    cmd, env = command(m["killed_by"])
    key = m["killed_by"]
    if key not in baseline_cache:
        baseline_cache[key] = passes(cmd, env)
    if not baseline_cache[key]:
        return f"{label}: INVALID — {key} is red without the mutation, so it proves nothing"
    try:
        target.write_text(original.replace(m["find"], m["replace"], 1))
        if not passes(build_command(cmd), env):
            return f"{label}: INVALID — mutated code does not compile ({m['change']})"
        if passes(cmd, env):
            return f"{label}: SURVIVED ({m['change']}) — {key} stayed green"
    finally:
        target.write_text(original)
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--rule", action="append", default=[])
    parser.add_argument("--list", action="store_true")
    args = parser.parse_args()

    todo = list(mutations(set(args.rule)))
    if args.list:
        for domain, rule_id, m in todo:
            print(f"{domain}/{rule_id}: {m['change']} -> {m['killed_by']}")
        return 0
    if not todo:
        print("mutation-check: no local mutations selected", file=sys.stderr)
        return 1

    problems = []
    for i, (domain, rule_id, m) in enumerate(todo, 1):
        print(f"[{i}/{len(todo)}] {domain}/{rule_id}: {m['change']}", flush=True)
        # One mutation per lock hold, so other sessions' builds interleave.
        with BuildLock():
            try:
                problem = run_one(domain, rule_id, m)
            except TimedOut as e:
                # A hang is no verdict: neither killed nor survived.
                problem = f"{domain}/{rule_id}: INVALID — timed out ({e})"
        if problem:
            print(f"  {problem}", flush=True)
            problems.append(problem)
        else:
            print("  killed", flush=True)
    survived = sum("SURVIVED" in p for p in problems)
    print(f"mutation-check: {len(todo) - len(problems)} killed, {survived} survived, "
          f"{len(problems) - survived} invalid")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
