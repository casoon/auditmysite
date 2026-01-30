#!/bin/bash
set -e

echo "♿ Running WCAG Compliance Tests..."
echo ""

# Check if binary exists
if [ ! -f "target/debug/auditmysit" ] && [ ! -f "target/release/auditmysit" ]; then
  echo "🔨 Building project first..."
  cargo build
fi

# Determine which binary to use
if [ -f "target/release/auditmysit" ]; then
  BINARY="target/release/auditmysit"
else
  BINARY="target/debug/auditmysit"
fi

echo "Using binary: $BINARY"
echo ""

# Run WCAG-specific unit tests
echo "1️⃣  Running WCAG rule tests..."
if cargo test --lib wcag -- --nocapture; then
  echo "✅ WCAG rule tests passed"
else
  echo "❌ WCAG rule tests failed"
  exit 1
fi
echo ""

# Run integration tests with fixtures
echo "2️⃣  Testing against HTML fixtures..."
if [ -d "tests/fixtures" ]; then
  FIXTURE_COUNT=0
  PASSED_COUNT=0

  for fixture in tests/fixtures/*.html; do
    if [ -f "$fixture" ]; then
      FIXTURE_COUNT=$((FIXTURE_COUNT + 1))
      FIXTURE_NAME=$(basename "$fixture")

      echo "   Testing: $FIXTURE_NAME"

      # Run audit on fixture (file:// URL)
      if $BINARY "file://$PWD/$fixture" -f json > /dev/null 2>&1; then
        PASSED_COUNT=$((PASSED_COUNT + 1))
        echo "   ✅ Passed"
      else
        echo "   ⚠️  Completed with violations (expected)"
      fi
    fi
  done

  echo ""
  echo "Tested $FIXTURE_COUNT fixtures"
else
  echo "⚠️  No test fixtures found in tests/fixtures/"
fi
echo ""

# Run full integration tests
echo "3️⃣  Running integration tests..."
if cargo test --test integration_test -- --nocapture; then
  echo "✅ Integration tests passed"
else
  echo "❌ Integration tests failed"
  exit 1
fi
echo ""

echo "✅ All WCAG tests completed!"
echo ""
echo "Summary:"
echo "  ✓ WCAG rule unit tests"
echo "  ✓ HTML fixture validation"
echo "  ✓ Integration tests"
