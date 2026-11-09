#!/usr/bin/env bash
# Run a cargo test selection N times without retries and classify every test:
#   stable-pass  passed in all runs
#   stable-fail  failed in all runs   -> drift (re-rate the site), not flakiness
#   flaky        passed and failed    -> unstable test or environment
#
# Usage: scripts/flake-hunt.sh [-n RUNS] [-o DIR] [-- <cargo test args>]
# Default selection: the live reference sites (needs Chrome and network).
#   scripts/flake-hunt.sh -n 5
#   scripts/flake-hunt.sh -n 3 -- --test browser_pool_test -- --ignored
# Logs of every run go to DIR (default reports/flake-hunt, gitignored).
# Exit code: 1 if any test is flaky, else 0 (stable failures are reported, not judged).
set -euo pipefail

runs=5
out="reports/flake-hunt"
while getopts "n:o:" opt; do
  case "$opt" in
    n) runs="$OPTARG" ;;
    o) out="$OPTARG" ;;
    *) exit 2 ;;
  esac
done
shift $((OPTIND - 1))
[ "${1:-}" = "--" ] && shift
if [ "$#" -eq 0 ]; then
  set -- --test reference_sites_test -- --ignored
fi

mkdir -p "$out"
stamp=$(date +%Y%m%d-%H%M%S)
results="$out/$stamp.results"
: >"$results"

for i in $(seq 1 "$runs"); do
  log="$out/$stamp-run$i.log"
  echo "run $i/$runs -> $log" >&2
  # --no-fail-fast: one failing test must not hide the others.
  cargo test --no-fail-fast "$@" >"$log" 2>&1 || true
  # libtest line: "test <name> ... ok|FAILED"
  sed -nE 's/^test (.+) \.\.\. (ok|FAILED)$/\1 \2/p' "$log" >>"$results"
done

echo
awk -v runs="$runs" '
  { name = $1; res = $2; if (res == "ok") pass[name]++; else fail[name]++; seen[name] = 1 }
  END {
    flaky = 0
    for (n in seen) {
      p = pass[n] + 0; f = fail[n] + 0
      cls = (f == 0) ? "stable-pass" : (p == 0) ? "stable-fail" : "flaky"
      if (cls == "flaky") flaky++
      printf "%-12s pass=%d fail=%d  %s\n", cls, p, f, n
    }
    if (length(seen) == 0) { print "no test results parsed - check the cargo test arguments"; exit 3 }
    exit (flaky > 0) ? 1 : 0
  }' "$results" | sort
