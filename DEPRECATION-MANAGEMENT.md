# Deprecation Warning Management

## Philosophy

Deprecation warnings are **internal software maintenance tools** and should not be visible to end users in production environments. They serve purely for developer guidance during refactoring periods.

## Automatic Suppression

Deprecation warnings are automatically suppressed in:

### Environment-Based Detection
- **CI Environments**: `CI=true` (standard CI environment variable)
- **Production**: `NODE_ENV=production`
- **Tests**: `NODE_ENV=test`

### Manual Override
- **CLI Flag**: `--quiet-deprecations` (for specific use cases)

## Implementation

The suppression logic is implemented in:
- `DeprecationManager.warnOnce()` - Central deprecation warning manager
- Event system adapters - Legacy system compatibility warnings
- CLI argument processing - User-facing deprecation suppression

## Usage Examples

```bash
# Automatic suppression in CI
CI=true auditmysite https://example.com/sitemap.xml

# Automatic suppression in production
NODE_ENV=production auditmysite https://example.com/sitemap.xml

# Manual suppression
auditmysite https://example.com/sitemap.xml --quiet-deprecations
```

## Development Workflow

1. **Development**: Deprecation warnings are shown to help with code migration
2. **CI/CD**: Warnings are automatically suppressed to avoid noise
3. **Production**: Warnings are automatically suppressed for end users
4. **Release**: Deprecated systems are removed entirely

## No User Configuration Needed

Users should **never** need to configure deprecation warning suppression manually. The system automatically detects the environment and behaves appropriately:

- **Development**: Shows warnings to help developers migrate code
- **Production/CI**: Suppresses warnings as they serve no purpose for end users

This approach ensures deprecation warnings fulfill their purpose (guiding refactoring) without impacting the user experience.
