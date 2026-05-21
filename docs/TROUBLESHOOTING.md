# Troubleshooting

## Common Issues

### Chrome/Chromium Not Found

**Error:** `No Chrome/Chromium installation found`

**Solutions:**

1. **Install via auditmysite** (recommended):
   ```bash
   auditmysite browser install
   # Downloads Chrome for Testing to ~/.auditmysite/browsers/
   ```

2. **Check browser status:**
   ```bash
   auditmysite browser detect
   # Shows which browser is found and where

   auditmysite doctor
   # Full system check (browser, permissions, connectivity)
   ```

3. **Specify path manually:**
   ```bash
   # macOS
   auditmysite --browser-path "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" https://example.com

   # Linux
   auditmysite --browser-path /usr/bin/chromium-browser https://example.com

   # Windows
   auditmysite --browser-path "C:\Program Files\Google\Chrome\Application\chrome.exe" https://example.com
   ```

4. **Install via package manager:**
   ```bash
   # macOS
   brew install chromium

   # Ubuntu/Debian
   sudo apt install chromium-browser

   # Fedora
   sudo dnf install chromium
   ```

### Timeout Errors

**Error:** `Page load timeout after 30 seconds`

**Solutions:**

1. Increase timeout:
   ```bash
   auditmysite --timeout 60 https://slow-site.com
   ```

2. Check if the site is accessible from your network

3. Try disabling images for faster loading:
   ```bash
   auditmysite --disable-images https://example.com
   ```

### Permission Denied (Docker/Root)

**Error:** `Failed to launch browser` or sandbox errors

**Solution:** Use `--no-sandbox` flag (only in trusted environments):
```bash
auditmysite --no-sandbox https://example.com
```

### WebSocket Errors on Close

**Error:** `WS Connection error: Ws(Protocol(ResetWithoutClosingHandshake))`

This warning is harmless and can be ignored. It occurs when Chrome's WebSocket connection closes during shutdown.

### PDF Generation Fails

**Error:** `Failed to generate PDF`

**Solutions:**

1. Ensure you have write permissions to the output directory
2. Check disk space
3. Try a different output path:
   ```bash
   auditmysite https://example.com -f pdf -o /tmp/report.pdf
   ```

### Private IP Blocked (SSRF Protection)

**Error:** `Private IP addresses are not allowed for security reasons`

This is a security feature. If you need to audit internal sites:

1. Use a public URL if available
2. For development, consider exposing via ngrok or similar

### Sitemap Parsing Fails

**Error:** `Failed to parse sitemap`

**Solutions:**

1. Verify the sitemap URL is accessible:
   ```bash
   curl https://example.com/sitemap.xml
   ```

2. Check if it's a sitemap index (will be processed recursively)

3. Use a URL file instead:
   ```bash
   echo "https://example.com/page1" > urls.txt
   echo "https://example.com/page2" >> urls.txt
   auditmysite --url-file urls.txt
   ```

## Browser Management

auditmysite provides built-in browser management commands:

```bash
# Detect installed browsers
auditmysite browser detect

# Install Chrome for Testing
auditmysite browser install

# Show browser binary path
auditmysite browser path

# Remove managed browser installation
auditmysite browser remove

# Full system diagnostic
auditmysite doctor
```

## Debug Mode

For detailed logging:
```bash
auditmysite --verbose https://example.com
```

For even more details:
```bash
RUST_LOG=debug auditmysite https://example.com
```

## Getting Help

1. Check the [GitHub Issues](https://github.com/casoon/auditmysite/issues)
2. Run `auditmysite --help` for CLI options
3. Open a new issue with:
   - Your command
   - Full error message
   - `auditmysite --version` output
   - Operating system
