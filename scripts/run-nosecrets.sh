#!/bin/sh

set -eu

if command -v nosecrets >/dev/null 2>&1; then
  exec nosecrets scan --staged "$@"
fi

cat >&2 <<EOF
nosecrets is not installed or not available in PATH.

Install an actual nosecrets binary first, for example:

- npm install -g @casoon/nosecrets
- cargo install nosecrets-cli

Then make sure the binary is reachable via:
  command -v nosecrets
EOF
exit 2
