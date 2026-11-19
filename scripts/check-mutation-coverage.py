#!/usr/bin/env python3
"""Tell the tag run whether an earlier full QA already ran the detector mutation check.

The check rebuilds once per mutation and sets the wall time of a release. The
release process dispatches the full QA on the candidate before the tag, so the
same mutations would run twice on the same code. This script finds a successful
`ci.yml` dispatch whose mutation shards all passed and whose commit differs from
the tagged one only in files no detector test reads (CHANGELOG, README,
CLAUDE.md). It writes `covered=true` to `$GITHUB_OUTPUT` then; in every other
case, and on every error, `covered=false`, and the tag run runs the mutations
itself. It never blocks a release: a missing or unreadable QA only costs time.
"""

import argparse
import json
import os
import re
import subprocess
import sys
from urllib.request import Request, urlopen

# Files that may differ between the QA'd commit and the tag. Everything else
# (sources, tests, fixtures, Cargo files, scripts, workflows) invalidates the QA.
ALLOWED_CHANGES = ("CHANGELOG.md", "README.md", "CLAUDE.md")
MUTATION_JOB = re.compile(r"Detector mutation check (\d+)/(\d+)$")


def complete_and_green(jobs):
    """True when the run's mutation jobs cover shards 1..N exactly once and all succeeded."""
    found = {}
    for job in jobs:
        match = MUTATION_JOB.search(job.get("name", ""))
        if not match:
            continue
        k, n = int(match.group(1)), int(match.group(2))
        if job.get("conclusion") != "success" or found.setdefault(n, {}).get(k):
            return False
        found[n][k] = True
    return len(found) == 1 and all(
        set(shards) == set(range(1, n + 1)) for n, shards in found.items()
    )


def only_allowed(paths):
    return all(path in ALLOWED_CHANGES for path in paths)


def api(path):
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "auditmysite-release-check"}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    with urlopen(Request(f"https://api.github.com/{path}", headers=headers), timeout=20) as response:
        return json.load(response)


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def changed_files(qa_sha):
    git("fetch", "--no-tags", "--depth=1", "origin", qa_sha)
    return git("diff", "--name-only", qa_sha, "HEAD").split()


def find_covering_run(repo, workflow, limit):
    runs = api(
        f"repos/{repo}/actions/workflows/{workflow}/runs"
        f"?event=workflow_dispatch&status=success&per_page={limit}"
    )["workflow_runs"]
    for run in runs:
        try:
            jobs = api(f"repos/{repo}/actions/runs/{run['id']}/jobs?per_page=100")["jobs"]
            if not complete_and_green(jobs):
                continue
            changed = changed_files(run["head_sha"])
        except Exception as error:  # an unreadable run is simply not evidence
            print(f"run {run.get('id')}: skipped ({error})")
            continue
        if only_allowed(changed):
            return run, changed
        print(f"run {run['id']} ({run['head_sha'][:9]}): code differs from the tag: "
              + ", ".join(changed[:5]) + (" …" if len(changed) > 5 else ""))
    return None, []


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY"))
    parser.add_argument("--workflow", default="ci.yml")
    parser.add_argument("--limit", type=int, default=20)
    args = parser.parse_args()

    covered = False
    try:
        run, changed = find_covering_run(args.repo, args.workflow, args.limit)
        if run:
            covered = True
            print(f"covered: run {run['id']} on {run['head_sha'][:9]} passed every mutation "
                  f"shard; changes since: {', '.join(changed) or 'none'}")
        else:
            print("not covered: no successful QA dispatch with all mutation shards green "
                  "on this code; the mutation shards run in this tag run")
    except Exception as error:
        print(f"not covered: {error}; the mutation shards run in this tag run")

    output = os.environ.get("GITHUB_OUTPUT")
    if output:
        with open(output, "a") as handle:
            handle.write(f"covered={'true' if covered else 'false'}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
