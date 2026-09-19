#!/bin/bash

# Suche nach key-Begriffen in den Dateien
declare -A SEARCH_PATTERNS

# Für document/title
grep -l "title" /tmp/am-migration-2/src/wcag/rules/*.rs | while read f; do
  echo "=== TITLE: $(basename "$f") ===" >&2
  grep -E "axe_id|check_.*title|\"title" "$f" | head -5
done 2>&1 | head -20

# Für viewport
echo "===VIEWPORT===" >&2
grep -l "viewport\|zoom" /tmp/am-migration-2/src/wcag/rules/*.rs | while read f; do
  echo "$(basename "$f")"
  grep "axe_id" "$f" | head -1
done

# Für headings
echo "===HEADINGS===" >&2
grep -l "heading" /tmp/am-migration-2/src/wcag/rules/*.rs | while read f; do
  echo "$(basename "$f")"
  grep "axe_id" "$f" | head -1
done

