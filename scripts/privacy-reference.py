#!/usr/bin/env python3
"""Reference pages for the privacy findings (#873). Method: docs/privacy-findings.md.

  check   verify the seals: reference_pages.json against its .sha256 file, and
          every frozen capture against the capture_sha256 its labels carry
  seal    after a deliberate label edit: recompute each capture_sha256 from
          reference_captures.json and rewrite reference_pages.json.sha256
  live    run the live measurement (Chrome and network): per-finding-kind
          precision and recall against the labels; fails on a false positive

This script never produces a label. Labels are written by a person from a raw
capture (`dump`, below) and checked independently of the tool.

  dump URL...   write the raw first-visit capture of each URL to
                reports/privacy-reference/<host>.json (labelling starts here)
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "tests/fixtures/privacy"
LABELS = FIXTURES / "reference_pages.json"
SEAL = FIXTURES / "reference_pages.json.sha256"
CAPTURES = FIXTURES / "reference_captures.json"


def canon(value):
    # Same canonical form the test computes: sorted keys, no whitespace.
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def load():
    return json.loads(LABELS.read_text()), json.loads(CAPTURES.read_text())


def check():
    labels, captures = load()
    bad = []
    if sha(LABELS.read_text()) != SEAL.read_text().strip():
        bad.append("reference_pages.json differs from its .sha256 seal (labels are frozen; edit deliberately, then `seal`)")
    ids = set()
    for site in labels["sites"]:
        ids.add(site["id"])
        capture = captures.get(site["id"])
        if capture is None:
            bad.append(f"{site['id']}: no frozen capture")
        elif sha(canon(capture)) != site["capture_sha256"]:
            bad.append(f"{site['id']}: capture differs from the one the labels were written from")
        if not site["expected"] and not site.get("outside_ruleset") and not site.get("notes"):
            bad.append(f"{site['id']}: a page without expectations needs a note")
        for entry in site["expected"]:
            if not entry.get("provenance") or not entry.get("evidence"):
                bad.append(f"{site['id']}: label {entry['key']} lacks evidence or provenance")
    for extra in set(captures) - ids:
        bad.append(f"{extra}: capture without labels")
    for line in bad:
        print("FAIL", line)
    print("seals hold" if not bad else f"{len(bad)} problem(s)")
    return 1 if bad else 0


def seal():
    labels, captures = load()
    for site in labels["sites"]:
        site["capture_sha256"] = sha(canon(captures[site["id"]]))
    LABELS.write_text(json.dumps(labels, indent=2, ensure_ascii=False) + "\n")
    SEAL.write_text(sha(LABELS.read_text()) + "\n")
    print("sealed", len(labels["sites"]), "sites")
    return 0


def cargo(test, env_extra=None, *extra):
    env = dict(os.environ, **(env_extra or {}))
    cmd = ["cargo", "test", "--test", "privacy_reference_pages", test, "--", "--ignored", "--nocapture", *extra]
    return subprocess.call(cmd, cwd=ROOT, env=env)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("check")
    sub.add_parser("seal")
    sub.add_parser("live")
    dump = sub.add_parser("dump")
    dump.add_argument("urls", nargs="+")
    args = parser.parse_args()
    if args.cmd == "check":
        return check()
    if args.cmd == "seal":
        return seal()
    if args.cmd == "live":
        return cargo("live_reference_pages")
    out = ROOT / "reports/privacy-reference"
    return cargo("dump_raw_first_visit_capture", {"PRIVACY_DUMP_URLS": ",".join(args.urls), "PRIVACY_DUMP_DIR": str(out)})


if __name__ == "__main__":
    sys.exit(main())
