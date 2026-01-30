# Chrome/Chromium Installation Guide

**AuditMySit** requires Chrome or Chromium to run accessibility audits. This guide explains all installation options without forcing system-level dependencies.

---

## 🎯 Quick Start (Recommended)

### Option 1: Auto-Download (Zero Configuration)

Just run AuditMySit - it will automatically handle Chrome:

```bash
auditmysit https://example.com
```

**What happens:**
1. ✅ Checks for system Chrome/Chromium
2. ✅ If found: Uses existing installation (no downloads!)
3. ✅ If not found: Prompts for auto-download to `~/.auditmysit/chromium/`

**Benefits:**
- 🟢 **No system dependencies affected**
- 🟢 **Isolated installation** (in ~/.auditmysit/)
- 🟢 **No Homebrew required**
- 🟢 **Easy to remove** (`rm -rf ~/.auditmysit/`)
- 🟢 **Managed by AuditMySit**

**Download size:** ~120 MB  
**Location:** `~/.auditmysit/chromium/`

---

## 📦 Manual Installation Options

### Option 2: Use Existing System Chrome

If you already have Chrome installed, AuditMySit will find it automatically:

**macOS:**
```bash
# Google Chrome (if installed)
/Applications/Google Chrome.app/Contents/MacOS/Google Chrome

# Chromium (if installed via Homebrew)
/Applications/Chromium.app/Contents/MacOS/Chromium
```

**Linux:**
```bash
# Debian/Ubuntu
/usr/bin/google-chrome
/usr/bin/chromium-browser

# Arch Linux
/usr/bin/chromium

# Snap
/snap/bin/chromium
```

**Windows:**
```powershell
# Google Chrome
C:\Program Files\Google\Chrome\Application\chrome.exe
C:\Program Files (x86)\Google\Chrome\Application\chrome.exe
```

---

### Option 3: Install via Package Manager

#### macOS (Homebrew)

```bash
# Chromium (open-source, lightweight)
brew install chromium

# OR Google Chrome (official)
brew install --cask google-chrome
```

#### Linux (Debian/Ubuntu)

```bash
# Chromium
sudo apt update
sudo apt install chromium-browser

# OR Google Chrome
wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
sudo dpkg -i google-chrome-stable_current_amd64.deb
```

#### Linux (Arch)

```bash
sudo pacman -S chromium
```

#### Linux (Fedora/RHEL)

```bash
sudo dnf install chromium
```

#### Windows

Download from: https://www.google.com/chrome/

---

### Option 4: Specify Custom Chrome Path

If Chrome is installed in a non-standard location:

```bash
auditmysit --chrome-path "/path/to/chrome" https://example.com

# macOS example
auditmysit --chrome-path "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" https://example.com

# Linux example
auditmysit --chrome-path /usr/bin/chromium https://example.com

# Windows example
auditmysit --chrome-path "C:\Program Files\Google\Chrome\Application\chrome.exe" https://example.com
```

**Or via environment variable:**

```bash
export CHROME_PATH="/path/to/chrome"
auditmysit https://example.com
```

---

## 🔍 Chrome Detection

AuditMySit searches for Chrome in this order:

1. **`--chrome-path` flag** (if specified)
2. **`CHROME_PATH` environment variable**
3. **System standard paths:**
   - macOS: `/Applications/Google Chrome.app/...`
   - macOS: `/Applications/Chromium.app/...`
   - Linux: `/usr/bin/google-chrome`, `/usr/bin/chromium`
   - Windows: `C:\Program Files\Google\Chrome\...`
4. **`which` command** (searches PATH)
5. **Auto-download prompt** (if none found)

### Test Detection

```bash
# See where Chrome is found
auditmysit --detect-chrome
```

**Example output:**
```
Detecting Chrome/Chromium...

Success: Chrome found!

  Path:    /Applications/Google Chrome.app/Contents/MacOS/Google Chrome
  Version: 144.0.7559.110
  Method:  StandardPath
```

---

## 🚫 What We DON'T Do

Unlike other tools, AuditMySit **does not**:

- ❌ Auto-install via Homebrew (avoids system dependencies)
- ❌ Force download without asking
- ❌ Affect existing Chrome installations
- ❌ Require Docker
- ❌ Bundle Chrome in the binary (keeps it lightweight: 8.5 MB)

**Philosophy:** Your system, your choice. AuditMySit adapts to your setup.

---

## 🗑️ Uninstalling Auto-Downloaded Chromium

If you used auto-download:

```bash
# Remove downloaded Chromium
rm -rf ~/.auditmysit/chromium/

# Or remove everything
rm -rf ~/.auditmysit/
```

**No system files affected!**

---

## 🐳 Docker Usage (Optional)

For CI/CD or containerized environments:

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    chromium \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/auditmysit /usr/local/bin/
ENV CHROME_PATH=/usr/bin/chromium

ENTRYPOINT ["auditmysit"]
```

### Usage

```bash
# Build
docker build -t auditmysit .

# Run
docker run --rm auditmysit https://example.com
```

---

## 🔧 Troubleshooting

### "Chrome not found"

```bash
# Check detection
auditmysit --detect-chrome

# If not found, specify manually
auditmysit --chrome-path /path/to/chrome https://example.com
```

### "Permission denied"

```bash
# macOS: Chrome needs execute permission
chmod +x "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"

# Linux
chmod +x /usr/bin/chromium
```

### "Browser process exited with status 9"

This usually means Chrome crashed. Try:

```bash
# Use --no-sandbox flag (Docker/root environments)
auditmysit --no-sandbox https://example.com
```

### Auto-download stuck

```bash
# Check network connection
curl -I https://storage.googleapis.com/chrome-for-testing-public/

# Download manually and specify path
auditmysit --chrome-path /path/to/downloaded/chrome https://example.com
```

---

## 📋 Chrome Versions

### Recommended Versions

- **Chrome 120-144:** Fully compatible
- **Chromium 120+:** Works great
- **Chrome for Testing:** Official headless Chrome builds

### Auto-Download Version

AuditMySit downloads **Chrome for Testing 131.0.6778.108** (stable):
- macOS: arm64 (Apple Silicon) or x64 (Intel)
- Linux: x64
- Windows: x64

---

## 🌐 Playwright/Puppeteer Integration

If you have Playwright or Puppeteer installed, AuditMySit can use their Chromium:

```bash
# Playwright cache
~/.cache/ms-playwright/chromium-*/chrome-mac/Chromium.app  # macOS
~/.cache/ms-playwright/chromium-*/chrome-linux/chrome      # Linux

# Puppeteer cache
~/.cache/puppeteer/chrome/mac-*/chrome-mac/Chromium.app    # macOS
~/.cache/puppeteer/chrome/linux-*/chrome-linux/chrome      # Linux
```

AuditMySit automatically checks these locations.

---

## ⚙️ CI/CD Examples

### GitHub Actions

```yaml
name: Accessibility Audit

on: [push]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Chromium
        run: sudo apt-get install -y chromium-browser
      
      - name: Install AuditMySit
        run: cargo install auditmysit
      
      - name: Run Audit
        run: auditmysit https://example.com -f json -o report.json
      
      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: accessibility-report
          path: report.json
```

### GitLab CI

```yaml
audit:
  image: rust:latest
  before_script:
    - apt-get update && apt-get install -y chromium
  script:
    - cargo install auditmysit
    - auditmysit https://example.com -f json -o report.json
  artifacts:
    paths:
      - report.json
```

---

## 💡 Best Practices

### Development

```bash
# Use system Chrome (fast, no downloads)
auditmysit https://localhost:3000
```

### CI/CD

```bash
# Install Chromium in container (reproducible)
apt-get install chromium-browser
auditmysit https://staging.example.com
```

### Production Monitoring

```bash
# Use auto-download (isolated, no admin rights needed)
auditmysit https://production.example.com
```

---

## 📊 Comparison

| Method | Size | System Impact | Speed | Reproducible |
|--------|------|---------------|-------|--------------|
| **Auto-Download** | 120 MB | None | Fast | ✅ Yes |
| **System Chrome** | 0 MB | Uses existing | Fastest | ⚠️ Version varies |
| **Homebrew** | 150 MB | System-level | Fast | ✅ Yes |
| **Docker** | 200 MB | Isolated | Medium | ✅ Yes |

---

## 🆘 Support

### Check Chrome Status

```bash
# Detect Chrome
auditmysit --detect-chrome

# Test with verbose output
auditmysit --verbose https://example.com

# Check version
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --version
```

### Common Issues

1. **Chrome version too old:** Update to 120+
2. **Headless mode issues:** Try `--no-sandbox` flag
3. **macOS Gatekeeper:** `xattr -cr /path/to/chrome`

---

**Last Updated:** 2024-01-30  
**Tool Version:** AuditMySit v0.1.0  
**Supported Chrome:** 120-144+
