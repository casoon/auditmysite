#!/bin/bash

echo "🔍 Detecting Chrome/Chromium binary..."
echo ""

# Define platform-specific paths
CHROME_PATHS_MACOS=(
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
  "/Applications/Chromium.app/Contents/MacOS/Chromium"
  "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary"
  "/opt/homebrew/bin/chromium"
  "/usr/local/bin/chromium"
)

CHROME_PATHS_LINUX=(
  "/usr/bin/google-chrome"
  "/usr/bin/google-chrome-stable"
  "/usr/bin/chromium"
  "/usr/bin/chromium-browser"
  "/snap/bin/chromium"
  "/var/lib/flatpak/exports/bin/org.chromium.Chromium"
  "/usr/bin/chrome"
)

CHROME_PATHS_WINDOWS=(
  "/mnt/c/Program Files/Google/Chrome/Application/chrome.exe"
  "/mnt/c/Program Files (x86)/Google/Chrome/Application/chrome.exe"
)

# Detect OS
OS=$(uname -s)
case "$OS" in
  Darwin)
    echo "Platform: macOS"
    PATHS=("${CHROME_PATHS_MACOS[@]}")
    INSTALL_CMD="brew install --cask google-chrome"
    ;;
  Linux)
    echo "Platform: Linux"
    PATHS=("${CHROME_PATHS_LINUX[@]}")
    if command -v apt >/dev/null 2>&1; then
      INSTALL_CMD="sudo apt install chromium-browser"
    elif command -v pacman >/dev/null 2>&1; then
      INSTALL_CMD="sudo pacman -S chromium"
    elif command -v dnf >/dev/null 2>&1; then
      INSTALL_CMD="sudo dnf install chromium"
    else
      INSTALL_CMD="Install via your package manager"
    fi
    ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "Platform: Windows"
    PATHS=("${CHROME_PATHS_WINDOWS[@]}")
    INSTALL_CMD="Download from https://www.google.com/chrome/"
    ;;
  *)
    echo "❌ Unknown platform: $OS"
    exit 1
    ;;
esac
echo ""

# Search for Chrome
FOUND=false
for path in "${PATHS[@]}"; do
  if [ -f "$path" ]; then
    echo "✅ Found Chrome/Chromium:"
    echo "   Path: $path"

    # Try to get version
    if [[ "$path" == *".app"* ]]; then
      # macOS app bundle
      VERSION=$("$path" --version 2>/dev/null || echo "unknown")
    else
      VERSION=$("$path" --version 2>/dev/null || echo "unknown")
    fi
    echo "   Version: $VERSION"

    # Export for use in scripts
    echo ""
    echo "Export this variable:"
    echo "  export CHROME_PATH=\"$path\""

    FOUND=true
    break
  fi
done

if [ "$FOUND" = false ]; then
  echo "❌ Chrome/Chromium not found in standard locations"
  echo ""
  echo "Searched paths:"
  for path in "${PATHS[@]}"; do
    echo "  - $path"
  done
  echo ""
  echo "Installation:"
  echo "  $INSTALL_CMD"
  echo ""
  echo "Or specify manually with:"
  echo "  auditmysit --chrome-path /path/to/chrome <url>"
  exit 1
fi

exit 0
