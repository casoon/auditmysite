#!/bin/bash
# Installation script for audit CLI
# Usage: curl -fsSL https://raw.githubusercontent.com/casoon/auditmysite/main/install.sh | bash

set -e

REPO="casoon/auditmysite"
BINARY_NAME="audit"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os arch

    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)
            case "$arch" in
                x86_64|amd64)
                    echo "x86_64-unknown-linux-gnu"
                    ;;
                aarch64|arm64)
                    echo "aarch64-unknown-linux-gnu"
                    ;;
                *)
                    error "Unsupported architecture: $arch"
                    ;;
            esac
            ;;
        darwin)
            case "$arch" in
                x86_64|amd64)
                    echo "x86_64-apple-darwin"
                    ;;
                aarch64|arm64)
                    echo "aarch64-apple-darwin"
                    ;;
                *)
                    error "Unsupported architecture: $arch"
                    ;;
            esac
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac
}

# Get latest version from GitHub
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
install() {
    local platform version url tmp_dir

    platform=$(detect_platform)
    version=$(get_latest_version)

    if [ -z "$version" ]; then
        error "Could not determine latest version"
    fi

    info "Installing $BINARY_NAME $version for $platform"

    url="https://github.com/$REPO/releases/download/$version/${BINARY_NAME}-${platform}.tar.gz"

    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    info "Downloading from $url"
    curl -fsSL "$url" -o "$tmp_dir/audit.tar.gz"

    info "Extracting..."
    tar -xzf "$tmp_dir/audit.tar.gz" -C "$tmp_dir"

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    info "Installing to $INSTALL_DIR"
    mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add the following to your shell profile (.bashrc, .zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
    fi

    info "Successfully installed $BINARY_NAME $version"
    echo ""
    echo "Run 'audit --help' to get started"
}

# Check dependencies
check_dependencies() {
    if ! command -v curl &> /dev/null; then
        error "curl is required but not installed"
    fi

    if ! command -v tar &> /dev/null; then
        error "tar is required but not installed"
    fi

    # Check for Chrome (optional but recommended)
    if ! command -v google-chrome &> /dev/null && ! command -v chromium &> /dev/null; then
        if [ ! -f "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" ]; then
            warn "Chrome/Chromium not found. audit requires Chrome for accessibility checks."
            echo "  Install Chrome: https://www.google.com/chrome/"
            echo "  Or use: audit --detect-chrome to check available browsers"
            echo ""
        fi
    fi
}

main() {
    echo ""
    echo "  ╔═══════════════════════════════════════╗"
    echo "  ║     audit CLI Installer               ║"
    echo "  ║     WCAG 2.1 Accessibility Checker    ║"
    echo "  ╚═══════════════════════════════════════╝"
    echo ""

    check_dependencies
    install
}

main "$@"
