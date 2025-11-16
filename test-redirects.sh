#!/bin/bash

# Test-Skript für CASOON.DE Redirects

echo "🔍 CASOON.DE Redirect Analysis with curl"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

urls=(
  "https://www.casoon.de/"
  "https://www.casoon.de/arbeitsweise"
  "https://www.casoon.de/cloud-entwicklung"
  "https://www.casoon.de/datenschutz"
  "https://www.casoon.de/e-commerce"
  "https://www.casoon.de/impressum"
  "https://www.casoon.de/kollaboration"
  "https://www.casoon.de/kontakt"
  "https://www.casoon.de/leistungskatalog"
  "https://www.casoon.de/plattform-apps"
  "https://www.casoon.de/projekte"
  "https://www.casoon.de/seo-marketing"
  "https://www.casoon.de/technologien"
  "https://www.casoon.de/usp"
  "https://www.casoon.de/webentwicklung"
)

ok_count=0
redirect_count=0
error_count=0

declare -a ok_urls
declare -a redirect_details

for url in "${urls[@]}"; do
  echo -n "Testing: $url ... "

  # Get only the first response (without following redirects)
  response=$(curl -I -s --max-time 10 "$url" 2>&1 | head -1)
  status=$(echo "$response" | grep -oP 'HTTP/[0-9.]+ \K[0-9]+' | head -1)

  if [ -z "$status" ]; then
    echo "❌ ERROR (No response)"
    ((error_count++))
  elif [ "$status" = "200" ]; then
    echo "✅ OK (200)"
    ((ok_count++))
    ok_urls+=("$url")
  elif [ "$status" -ge 300 ] && [ "$status" -lt 400 ]; then
    # Get redirect location
    location=$(curl -I -s --max-time 10 "$url" 2>&1 | grep -i "^location:" | cut -d' ' -f2 | tr -d '\r\n')
    echo "🔀 REDIRECT ($status) → $location"
    ((redirect_count++))
    redirect_details+=("$url|$status|$location")
  else
    echo "⚠️  Status: $status"
    ((error_count++))
  fi

  sleep 0.5
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Total URLs tested: ${#urls[@]}"
echo "✅ OK (200): $ok_count"
echo "🔀 Redirects (3xx): $redirect_count"
echo "❌ Errors/Other: $error_count"
echo ""

if [ $redirect_count -gt 0 ]; then
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "🔀 REDIRECT DETAILS"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  for detail in "${redirect_details[@]}"; do
    IFS='|' read -r source status target <<< "$detail"
    echo "Source: $source"
    echo "Status: $status"
    echo "Target: $target"
    echo ""
  done
fi

if [ $ok_count -gt 0 ]; then
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "✅ WORKING URLs (No Redirects)"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  for url in "${ok_urls[@]}"; do
    echo "- $url"
  done
  echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Analysis complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
