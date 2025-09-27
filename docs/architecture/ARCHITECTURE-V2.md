# AuditMySite v2 - Clean Architecture

## 🎯 Overview

This document describes the completely refactored architecture of AuditMySite, focusing on **clean separation of concerns**, **proper dependency injection**, **type safety**, and **logical interfaces**.

## 🏗️ Architecture Principles

### ✅ What We Fixed
1. **No more fallback code** - Single code path, no legacy compatibility
2. **Proper dependency injection** - All dependencies are explicit and injected
3. **Clear interfaces** - Every component implements well-defined contracts
4. **Separation of concerns** - Each class has a single, clear responsibility
5. **Type safety** - Full TypeScript coverage with proper interfaces
6. **Structured logging** - Consistent logging interface throughout
7. **Error handling** - Standardized error handling patterns

### ❌ What We Removed
- Legacy BrowserManager fallbacks
- Unified Event System (over-engineered)
- usePooling flags and optional behaviors
- Deprecated methods and compatibility layers
- Console.log calls (replaced with structured logging)
- Complex option interfaces with too many optionals

## 📂 New File Structure

```
src/core/
├── analyzers/
│   ├── interfaces.ts              # Core analyzer contracts
│   └── analyzer-factory.ts        # Dependency injection factory
├── analysis/
│   └── analysis-orchestrator.ts   # Coordinates multiple analyzers
├── accessibility/
│   ├── accessibility-checker.ts   # Legacy (to be replaced)
│   └── accessibility-checker-v2.ts # New clean implementation
└── logging/
    └── structured-logger.ts       # Structured logging implementation
```

## 🔌 Core Interfaces

### IAnalyzer Interface
```typescript
export interface IAnalyzer<TResult = any, TOptions = any> {
  readonly type: AnalyzerType;
  readonly name: string;
  initialize?(): Promise<void>;
  analyze(page: Page, url: string, options?: TOptions): Promise<TResult>;
  cleanup?(): Promise<void>;
}
```

**Benefits:**
- **Consistent API** across all analyzers
- **Type safety** with generic result types
- **Optional lifecycle methods** (initialize/cleanup)
- **Clear responsibility** - single analyze method

### ILogger Interface
```typescript
export interface ILogger {
  debug(message: string, data?: any): void;
  info(message: string, data?: any): void;
  warn(message: string, data?: any): void;
  error(message: string, error?: Error | any): void;
  success(message: string, data?: any): void;
  child?(prefix: string): ILogger;
}
```

**Benefits:**
- **Structured logging** with data context
- **Level-based filtering**
- **Child loggers** for component-specific logging
- **Easy testing** with silent logger implementation

## 🏭 Dependency Injection

### AnalyzerFactory
The factory pattern provides clean dependency injection:

```typescript
const factory = new AnalyzerFactory({
  logger: createLogger('analyzer'),
  qualityAnalysisOptions: config.quality,
  enabledAnalyzers: ['performance', 'seo', 'content-weight']
});

const performanceAnalyzer = factory.createAnalyzer<PerformanceAnalyzer>('performance');
```

**Benefits:**
- **Singleton behavior** - analyzers are cached
- **Configuration consistency** - same config for all analyzers
- **Type safety** - generic return types
- **Easy testing** - mock factory for unit tests

### AnalysisOrchestrator
Coordinates multiple analyzers with proper resource management:

```typescript
const orchestrator = new AnalysisOrchestrator({
  analyzerFactory: factory,
  logger: createLogger('orchestrator'),
  defaultTimeout: 30000,
  failFast: false
});

const results = await orchestrator.runComprehensiveAnalysis(page, url, {
  concurrency: true,
  timeout: 30000
});
```

**Benefits:**
- **Resource management** - proper cleanup and timeouts
- **Concurrent execution** - parallel analyzer execution
- **Error isolation** - failed analyzers don't break others
- **Standardized results** - consistent result format

## 🔍 AccessibilityChecker v2

### Clean Configuration
```typescript
const checker = new AccessibilityChecker({
  poolManager: browserPoolManager,        // Required - no fallbacks!
  logger: createLogger('a11y-checker'),   // Optional - defaults provided
  enableComprehensiveAnalysis: true,      // Optional - clear default
  analyzerTypes: ['performance', 'seo']   // Optional - specific analyzers
});
```

### Clear Responsibilities
1. **Core accessibility testing** via pa11y
2. **Basic page analysis** (images, buttons, headings) 
3. **Browser pool coordination** 
4. **Comprehensive analysis orchestration** (when enabled)

### Type-Safe Results
```typescript
interface PageTestResult {
  readonly url: string;
  readonly title: string;
  readonly accessibilityResult: AccessibilityResult;
  readonly comprehensiveAnalysis?: AnalysisResults;
  readonly duration: number;
  readonly timestamp: Date;
}
```

## 🚦 Usage Examples

### Basic Accessibility Testing
```typescript
const poolManager = new BrowserPoolManager({
  maxInstances: 3,
  headless: true
});
await poolManager.initialize();

const checker = new AccessibilityChecker({
  poolManager
});

const result = await checker.testPage('https://example.com');
console.log(`Accessibility score: ${result.accessibilityResult.pa11yScore}`);
```

### Comprehensive Analysis
```typescript
const checker = new AccessibilityChecker({
  poolManager,
  enableComprehensiveAnalysis: true,
  analyzerTypes: ['performance', 'seo', 'content-weight'],
  logger: createLogger('my-app')
});

const result = await checker.testPage('https://example.com');
console.log('Accessibility:', result.accessibilityResult.pa11yScore);
console.log('Performance:', result.comprehensiveAnalysis?.results);
```

### Multiple Pages with Queue
```typescript
const results = await checker.testMultiplePages(
  ['https://example.com', 'https://example.com/about'],
  {
    maxConcurrent: 3,
    timeout: 30000,
    enableComprehensiveAnalysis: true
  }
);
```

## 📊 Benefits of New Architecture

### For Developers
- **Predictable APIs** - no hidden fallbacks or legacy paths
- **Easy testing** - dependency injection makes mocking simple  
- **Type safety** - full IntelliSense support
- **Clear errors** - structured error handling with context

### For Maintenance
- **Single responsibility** - each class does one thing well
- **Loose coupling** - components depend on interfaces, not implementations
- **Easy extension** - new analyzers implement IAnalyzer interface
- **Configuration driven** - behavior controlled by explicit config

### For Performance  
- **Browser pooling always** - no single-browser fallbacks
- **Resource management** - proper cleanup and timeout handling
- **Concurrent analysis** - parallel analyzer execution
- **Efficient caching** - analyzer instances are reused

## 🔧 Migration Guide

### From Old AccessibilityChecker
```typescript
// OLD (with fallbacks and optional pooling)
const checker = new AccessibilityChecker({
  usePooling: true,
  enableUnifiedEvents: true,
  showDeprecationWarnings: false
});
await checker.initialize();

// NEW (explicit dependencies, no fallbacks)
const poolManager = new BrowserPoolManager({ maxInstances: 3 });
await poolManager.initialize();

const checker = new AccessibilityChecker({
  poolManager,
  enableComprehensiveAnalysis: true
});
await checker.initialize();
```

### Error Handling Changes
```typescript
// OLD (mixed error handling)
try {
  const result = await checker.testPage(url);
  if (result.crashed) {
    // Handle crash
  }
} catch (error) {
  // Handle other errors
}

// NEW (consistent error handling)
try {
  const result = await checker.testPage(url);
  if (!result.accessibilityResult.passed) {
    console.log('Accessibility issues:', result.accessibilityResult.errors);
  }
} catch (error) {
  // All errors are thrown consistently
  logger.error('Test failed', error);
}
```

## 🎯 Summary

The new architecture provides:

✅ **Clean code** - no legacy fallbacks or over-engineered systems
✅ **Type safety** - full TypeScript coverage with proper interfaces  
✅ **Dependency injection** - explicit, testable dependencies
✅ **Separation of concerns** - each component has a single responsibility
✅ **Structured logging** - consistent, contextual logging throughout
✅ **Error handling** - standardized error patterns
✅ **Performance** - optimized resource management and concurrency

This is a foundation that can be easily maintained, extended, and tested without the complexity and confusion of the previous implementation.