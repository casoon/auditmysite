# Development Setup

## Prerequisites

This project uses [Volta](https://volta.sh/) for Node.js version management and pnpm as the package manager.

### 1. Install Volta

**macOS/Linux:**
```bash
curl https://get.volta.sh | bash
```

**Windows:**
```powershell
# Download and run the Windows installer from https://volta.sh/
```

After installation, restart your terminal.

### 2. Install Node.js and pnpm

Volta will automatically install the correct Node.js and pnpm versions specified in `package.json`:

```bash
# Clone the repository
git clone https://github.com/casoon/AuditMySite.git
cd auditmysite

# Volta automatically installs Node 20.18.0 and pnpm 9.15.0
# when you enter the directory
volta install node pnpm
```

### 3. Install Dependencies

```bash
pnpm install
```

### 4. Build the Project

```bash
pnpm run build
```

### 5. Run Tests

```bash
pnpm test
```

## Development Workflow

### Watch Mode
```bash
pnpm run dev
```

### Type Checking
```bash
pnpm run type-check
```

### Run Local CLI
```bash
node bin/audit.js https://example.com/sitemap.xml
```

## Verifying Your Setup

Check your installed versions:
```bash
node --version   # Should be v20.18.0
pnpm --version   # Should be 9.15.0
```

## Troubleshooting

### Volta not found
Make sure Volta's bin directory is in your PATH. Restart your terminal after installation.

### pnpm command not found
Run `volta install pnpm` manually.

### Build errors
1. Remove `node_modules` and `pnpm-lock.yaml`
2. Run `pnpm install` again
3. Run `pnpm run build`
