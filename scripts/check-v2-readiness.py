#!/usr/bin/env python3
"""Fail closed on open or missing 2.x release prerequisites in GitHub."""

import argparse
import json
import os
from pathlib import Path
import re
import sys
import tomllib
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parent.parent


def fetch_issues(manifest):
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "auditmysite-release-check"}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    issues = []
    for page in range(1, 101):
        url = (f"https://api.github.com/repos/{manifest['repository']}/issues"
               f"?milestone={manifest['milestone']}&state=all&per_page=100&page={page}")
        with urlopen(Request(url, headers=headers), timeout=15) as response:
            batch = json.load(response)
        if not isinstance(batch, list) or not all(isinstance(issue, dict) for issue in batch):
            raise ValueError("GitHub returned an invalid issue list")
        issues.extend(issue for issue in batch if "pull_request" not in issue)
        if len(batch) < 100:
            return issues
    raise ValueError("GitHub pagination exceeded the release-check bound")


def blockers(manifest, issues):
    requirements = manifest["requirements"]
    expected = {f"U{i:02}" for i in range(1, 27)}
    if (len(requirements) != 26 or {r["audit_id"] for r in requirements} != expected
            or len({r["issue"] for r in requirements}) != 26):
        raise ValueError("release manifest must preserve all 26 distinct audit requirements")
    by_number = {issue["number"]: issue for issue in issues}
    required = {r["issue"] for r in requirements} | {manifest["tracker"]}
    problems = []
    for number in sorted(required):
        issue = by_number.get(number)
        if issue is None:
            problems.append(f"#{number}: missing from the required milestone")
        elif issue.get("milestone", {}).get("number") != manifest["milestone"]:
            problems.append(f"#{number}: wrong milestone")
        elif issue.get("state") != "closed" or issue.get("state_reason") != "completed":
            problems.append(f"#{number}: not completed ({issue.get('state')}/{issue.get('state_reason')})")
    for issue in issues:
        if issue["number"] not in required and issue.get("state") != "closed":
            problems.append(f"#{issue['number']}: additional open milestone prerequisite")
    return problems


def check(version, manifest, fetch=fetch_issues):
    match = re.fullmatch(r"v?(\d+)\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", version)
    if not match:
        raise ValueError(f"invalid release version: {version}")
    if int(match.group(1)) < 2:
        return []
    return blockers(manifest, fetch(manifest))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", default=os.environ.get("GITHUB_REF_NAME"))
    args = parser.parse_args()
    try:
        version = args.version or tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
        manifest = json.loads((ROOT / "scripts/v2-readiness.json").read_text())
        problems = check(version, manifest)
        if problems:
            print("Release blocked by unfinished 2.0 prerequisites:", file=sys.stderr)
            print("\n".join(problems), file=sys.stderr)
            return 1
        print(f"Release prerequisite check passed for {version}")
        return 0
    except Exception as error:
        print(f"Cannot verify release prerequisites: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
