# 🚨 DEPRECATED SYSTEMS - AuditMySite v2.0.0

This document lists all deprecated systems and provides migration guides to the unified event system.

## 📋 Summary

As of AuditMySite v2.0.0, multiple parallel event systems have been **consolidated into a single, unified PageAnalysisEmitter system**. This provides:

- ✅ **Better Performance**: Reduced system complexity and overhead
- ✅ **Consistent APIs**: Single interface for all event types
- ✅ **Enhanced Features**: Integrated resource monitoring, backpressure control
- ✅ **Backward Compatibility**: Existing code continues to work via adapters

## 🗂️ Deprecated Systems

### 1. EventDrivenQueue (`src/core/pipeline/event-driven-queue.ts`)
**Status**: 🚨 Deprecated in v2.0.0, will be removed in v3.0.0

**Migration**:
```typescript
// OLD (deprecated)
const queue = new EventDrivenQueue({
  eventCallbacks: {
    onUrlStarted: (url) => console.log(`Starting: ${url}`),
    onUrlCompleted: (url, result, duration) => console.log(`Completed: ${url}`)
  }
});

// NEW (recommended)  
const checker = new AccessibilityChecker({ 
  enableUnifiedEvents: true,
  enableComprehensiveAnalysis: true
});
checker.setUnifiedEventCallbacks({
  onUrlStarted: (url) => console.log(`Starting: ${url}`),
  onUrlCompleted: (url, result, duration) => console.log(`Completed: ${url}`)
});
```

### 2. ParallelTestManager (`src/core/pipeline/parallel-test-manager.ts`)
**Status**: 🚨 Deprecated in v2.0.0, will be removed in v3.0.0

**Migration**:
```typescript
// OLD (deprecated)
const manager = new ParallelTestManager({
  maxConcurrent: 3,
  onTestComplete: (url, result) => { ... }
});
await manager.runTests(urls);

// NEW (recommended)
const checker = new AccessibilityChecker({ 
  enableUnifiedEvents: true,
  enableComprehensiveAnalysis: true 
});
checker.setUnifiedEventCallbacks({ 
  onUrlCompleted: (url, result, duration) => { ... } 
});
await checker.testMultiplePagesParallel(urls, { maxConcurrent: 3 });
```

### 3. TestOptions.eventCallbacks (`src/types.ts`)
**Status**: 🟡 Deprecated in v2.0.0, maintained for compatibility

**Migration**:
```typescript
// OLD (still works but deprecated)
const results = await checker.testMultiplePagesParallel(urls, {
  eventCallbacks: {
    onUrlStarted: (url) => { ... },
    onUrlCompleted: (url, result, duration) => { ... }
  }
});

// NEW (recommended)
checker.setUnifiedEventCallbacks({
  onUrlStarted: (url) => { ... },
  onUrlCompleted: (url, result, duration) => { ... }
});
const results = await checker.testMultiplePagesParallel(urls);
```

### 4. Direct bin/audit.js Callback Patterns
**Status**: 🟡 Deprecated in v2.0.0, maintained for compatibility

**Migration**: The direct callback patterns in `bin/audit.js` are automatically adapted to use the unified system internally. No changes required for CLI usage.

## 🎯 NEW UNIFIED SYSTEM

### PageAnalysisEmitter (`src/core/events/page-analysis-emitter.ts`)
The new unified event system that consolidates all previous event patterns.

**Key Features**:
- 🔄 **Unified Interface**: Single callback interface for all events
- 📊 **Resource Monitoring**: Built-in memory/CPU monitoring
- 🏃 **Backpressure Control**: Automatic resource management
- 🔁 **Retry Logic**: Smart retry mechanisms with exponential backoff
- 📈 **Progress Tracking**: Detailed progress and statistics
- 🧪 **State Management**: Support for pause/resume functionality

**Usage**:
```typescript
// Direct usage (advanced)
const emitter = new PageAnalysisEmitter({
  verbose: true,
  enableResourceMonitoring: true,
  enableBackpressure: true,
  callbacks: {
    onUrlStarted: (url) => console.log(`Starting: ${url}`),
    onUrlCompleted: (url, result, duration) => console.log(`Completed: ${url}`),
    onProgressUpdate: (stats) => console.log(`Progress: ${stats.progress}%`),
    onResourceWarning: (usage, limit, type) => console.warn(`${type} usage: ${usage}/${limit}`)
  }
});

// Via AccessibilityChecker (recommended)
const checker = new AccessibilityChecker({ 
  enableUnifiedEvents: true,
  enableComprehensiveAnalysis: true
});
```

## 📅 Deprecation Timeline

| Version | Status | Action |
|---------|--------|--------|
| **v2.0.0** | 🟡 Deprecated | All old systems marked as deprecated, adapters provided |
| **v2.1.0** | 🟡 Maintained | Compatibility maintained, warnings shown |
| **v2.5.0** | 🚨 Final Warning | Last version with full compatibility |
| **v3.0.0** | ❌ Removed | All deprecated systems removed |

## 🔧 Compatibility Mode

During the transition period (v2.x), all deprecated systems continue to work via adapter layers:

- **TestOptionsEventAdapter**: Converts `TestOptions.eventCallbacks` to `UnifiedEventCallbacks`
- **EventDrivenQueueAdapter**: Converts `EventDrivenQueueOptions.eventCallbacks`
- **ParallelTestManagerAdapter**: Converts `ParallelTestManager` callbacks

These adapters show deprecation warnings (unless disabled) and guide users toward the new system.

## 🛠️ Migration Tools

### 1. Disable Deprecation Warnings
```typescript
const checker = new AccessibilityChecker({
  showDeprecationWarnings: false
});
```

### 2. Check Active Warnings
```typescript
import { DeprecationManager } from './src/core/events/event-system-adapters';

// See which systems showed warnings
const warnings = DeprecationManager.getWarnings();
console.log('Deprecated systems used:', warnings);
```

### 3. Migration Validation
```typescript
// Verify unified system is active
const checker = new AccessibilityChecker({ enableUnifiedEvents: true });
const emitter = checker.getUnifiedEmitter();
console.log('Unified system active:', !!emitter);
```

## 📚 Support

- **Documentation**: [Unified Events Guide](https://auditmysite.com/docs/unified-events)
- **Migration Guide**: [v2.0.0 Migration](https://auditmysite.com/docs/v2-migration)
- **GitHub Issues**: [Report migration issues](https://github.com/your-org/auditmysite/issues)

## ⚠️ Breaking Changes in v3.0.0

The following will be **completely removed** in v3.0.0:

- `EventDrivenQueue` class and all related interfaces
- `ParallelTestManager` class and all related interfaces  
- Event adapter classes (TestOptionsEventAdapter, etc.)
- Legacy callback patterns in TestOptions
- Compatibility layers and deprecation warnings

**Action Required**: Migrate to `PageAnalysisEmitter` and `UnifiedEventCallbacks` before v3.0.0.
