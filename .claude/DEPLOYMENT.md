# Deployment Guide for audit

## Release Process

### 1. Version Bump

Update version in `Cargo.toml`:

```toml
[package]
version = "X.Y.Z"
```

### 2. Build Release Binary

```bash
# Build optimized release binary
cargo build --release

# Verify binary
./target/release/audit --version
```

### 3. Run Tests

```bash
cargo test
```

### 4. Create Git Tag

```bash
git add -A
git commit -m "chore: Release vX.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main --tags
```

### 5. GitHub Release

Create a new release on GitHub:

1. Go to https://github.com/casoon/auditmysite/releases
2. Click "Draft a new release"
3. Select the tag `vX.Y.Z`
4. Title: `vX.Y.Z`
5. Add release notes
6. Attach binaries (see below)

### 6. Build Platform Binaries

#### macOS (Apple Silicon)

```bash
cargo build --release
# Binary: target/release/audit
```

#### macOS (Intel)

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
# Binary: target/x86_64-apple-darwin/release/audit
```

#### Linux (x86_64)

```bash
# Using cross-compilation or on Linux machine
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
# Binary: target/x86_64-unknown-linux-gnu/release/audit
```

#### Windows

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
# Binary: target/x86_64-pc-windows-msvc/release/audit.exe
```

### 7. Homebrew Formula Update

After releasing binaries, update the Homebrew formula:

1. Calculate SHA256 checksums for each binary:

```bash
shasum -a 256 audit-macos-arm64.tar.gz
shasum -a 256 audit-macos-x86_64.tar.gz
shasum -a 256 audit-linux-x86_64.tar.gz
```

2. Update `Formula/audit.rb`:

```ruby
class Audit < Formula
  desc "Lightning-fast WCAG accessibility auditing written in Rust"
  homepage "https://github.com/casoon/auditmysite"
  version "X.Y.Z"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/casoon/auditmysite/releases/download/vX.Y.Z/audit-macos-arm64.tar.gz"
      sha256 "<SHA256_ARM64>"
    else
      url "https://github.com/casoon/auditmysite/releases/download/vX.Y.Z/audit-macos-x86_64.tar.gz"
      sha256 "<SHA256_X86_64>"
    end
  end

  on_linux do
    url "https://github.com/casoon/auditmysite/releases/download/vX.Y.Z/audit-linux-x86_64.tar.gz"
    sha256 "<SHA256_LINUX>"
  end

  def install
    bin.install "audit"
  end

  test do
    system "#{bin}/audit", "--version"
  end
end
```

3. Commit and push formula update:

```bash
git add Formula/audit.rb
git commit -m "chore: Update Formula with vX.Y.Z SHA256 checksums"
git push origin main
```

### 8. Publish to crates.io (Optional)

```bash
cargo login
cargo publish
```

## Quick Release Checklist

- [ ] Version bumped in `Cargo.toml`
- [ ] All tests pass (`cargo test`)
- [ ] Commit and tag created
- [ ] Tag pushed to GitHub
- [ ] GitHub release created with binaries
- [ ] Homebrew formula updated with new SHA256 checksums
- [ ] README version updated if needed

## Troubleshooting

### Binary not found after Homebrew install

```bash
brew uninstall audit
brew install casoon/tap/audit
```

### Permission denied on macOS

```bash
xattr -d com.apple.quarantine /usr/local/bin/audit
```

### Chrome not found

The binary will auto-download Chromium to `~/.cache/chromiumoxide/` on first run. Alternatively, specify Chrome path:

```bash
audit --chrome-path /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome https://example.com
```
