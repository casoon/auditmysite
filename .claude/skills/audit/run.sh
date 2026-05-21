#!/bin/bash
set -e

echo "🔍 Running Rust Audit Pipeline..."
echo ""

# Check if in Rust project
if [ ! -f "Cargo.toml" ]; then
  echo "❌ Error: Not in a Rust project (Cargo.toml not found)"
  exit 1
fi

echo "1️⃣  Checking code formatting..."
if cargo fmt --check; then
  echo "✅ Format check passed"
else
  echo "❌ Format check failed - run 'cargo fmt' to fix"
  exit 1
fi
echo ""

echo "2️⃣  Running Clippy (linter)..."
if cargo clippy --all-targets --all-features -- -D warnings; then
  echo "✅ Clippy passed"
else
  echo "❌ Clippy found issues"
  exit 1
fi
echo ""

echo "3️⃣  Running tests..."
if cargo test --all-features; then
  echo "✅ Tests passed"
else
  echo "❌ Tests failed"
  exit 1
fi
echo ""

echo "4️⃣  Building release binary..."
if cargo build --release; then
  echo "✅ Release build successful"

  # Show binary info
  BINARY_PATH="target/release/auditmysit"
  if [ -f "$BINARY_PATH" ]; then
    BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
    echo "   Binary size: $BINARY_SIZE"
    echo "   Location: $BINARY_PATH"
  fi
else
  echo "❌ Build failed"
  exit 1
fi
echo ""

echo "✅ All audit checks passed!"
echo ""
echo "Summary:"
echo "  ✓ Code formatting"
echo "  ✓ Clippy lints"
echo "  ✓ Unit & integration tests"
echo "  ✓ Release build"
