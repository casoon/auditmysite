# AuditMySite v2.0.0 - Major Upgrade Guide

## 🚀 What's New

### 1. Performance Optimizations
- **Browser Pool Manager**: Efficient resource management with connection pooling
- **Performance Monitoring**: Real-time metrics tracking and memory leak detection
- **Optimized Parallel Processing**: Up to 3x faster analysis with smart resource allocation

### 2. Enhanced Report Features
- **Interactive Charts**: Beautiful visualizations powered by Chart.js
- **Multiple Export Formats**: HTML, PDF, JSON, CSV, and Markdown
- **Trend Analysis**: Historical performance tracking
- **Comparison Views**: Desktop vs Mobile analysis
- **Print-friendly Reports**: Optimized for sharing and presentations

### 3. API Extensions
- **Webhook System**: Real-time notifications for audit completion
- **Streaming Responses**: Real-time progress updates
- **Rate Limiting**: Improved stability and resource management
- **Job Management**: Priority queues and retry mechanisms

### 4. New Analysis Features
- **Schema Markup Validator**: Complete structured data analysis
- **Rich Snippet Opportunities**: Identify SEO enhancement possibilities
- **Enhanced Mobile Analysis**: Desktop vs Mobile comparison
- **Advanced Performance Metrics**: Core Web Vitals, resource timing
- **Content Weight Analysis**: Detailed resource breakdown

### 5. Developer Experience
- **Comprehensive Monitoring**: Built-in performance tracking
- **Error Recovery**: Automatic retries and graceful degradation
- **Memory Management**: Automatic cleanup and optimization
- **TypeScript Support**: Full type safety throughout

### 6. Adaptive Backpressure & Resource Management 🆕
- **Smart Queue Management**: Automatic backpressure when system resources are constrained
- **Resource Monitoring**: Real-time CPU, memory, and event loop monitoring
- **Dynamic Worker Scaling**: Automatically adjust concurrency based on system health
- **Garbage Collection Control**: Proactive memory management with forced GC
- **System Health Scoring**: 0-100 resource health indicators
- **Configurable Thresholds**: Fine-tune memory and CPU limits for your environment

## 🔧 Breaking Changes

### CLI Changes
- Enhanced analysis is now **enabled by default**
- New options for granular control:
  - `--no-performance`: Disable performance analysis
  - `--no-seo`: Disable SEO analysis
  - `--no-mobile`: Disable mobile-friendliness analysis
  - `--no-content-weight`: Disable content weight analysis
- **New resource management options**:
  - `--enable-backpressure`: Enable adaptive backpressure control
  - `--max-memory-mb <size>`: Set memory limit (default: auto-detected)
  - `--enable-resource-monitoring`: Enable system resource monitoring
  - `--resource-monitoring-interval <ms>`: Set monitoring frequency

### API Changes
- All endpoints now return enhanced data by default
- New webhook endpoints for real-time notifications

## 📦 Installation & Upgrade

### From npm
```bash
npm install -g @casoon/auditmysite@latest
```

### From Source
```bash
git pull origin main
npm install
npm run build
```

## 🧪 Testing the New Features

### 1. Basic Enhanced Audit
```bash
auditmysite https://example.com --format html --verbose
```

### 2. Multiple Pages Processing
```bash
auditmysite https://example.com/sitemap.xml --max-pages 5 --format json
```

### 3. Performance Monitoring
```javascript
import { performanceMonitor } from '@casoon/auditmysite';

const result = await performanceMonitor.monitor('audit-operation', async () => {
    // Your audit code here
});

console.log(performanceMonitor.generateReport());
```

### 4. Webhook Integration
```javascript
import { webhookManager } from '@casoon/auditmysite';

webhookManager.register('my-webhook', {
    url: 'https://mysite.com/webhook',
    events: ['audit.completed'],
    secret: 'my-secret-key',
    active: true
});
```

### 5. Interactive Reports
The new HTML reports include:
- Interactive charts and graphs
- Export functionality
- Responsive design
- Print optimization

### 6. 🆕 Adaptive Backpressure Testing
```bash
# Test with backpressure enabled
auditmysite https://example.com/sitemap.xml --max-pages 10 --enable-backpressure --verbose

# Test with custom memory limits
auditmysite https://large-site.com/sitemap.xml --max-memory-mb 1024 --enable-resource-monitoring

# Stress test with large sitemap
auditmysite https://example.com/sitemap.xml --max-pages 50 --enable-backpressure
```

### 7. Resource Monitoring
```javascript
import { ResourceMonitor } from '@casoon/auditmysite';

const monitor = new ResourceMonitor({
  enabled: true,
  memoryWarningThresholdMB: 1536,
  memoryCriticalThresholdMB: 2048
});

monitor.on('resourceAlert', (alert) => {
  console.log(`${alert.level}: ${alert.message}`);
  if (alert.level === 'critical') {
    // Take action - reduce concurrency, trigger GC, etc.
  }
});

monitor.start();
```

### 8. Queue Management with Backpressure
```javascript
import { EnhancedParallelQueueAdapter } from '@casoon/auditmysite';

const queue = new EnhancedParallelQueueAdapter({
  maxConcurrent: 5,
  enableBackpressure: true,
  maxMemoryUsage: 2048,
  enableResourceMonitoring: true
}, {
  onBackpressureActivated: (metrics) => {
    console.log('Backpressure activated:', metrics.currentDelay + 'ms delay');
  },
  onResourceWarning: (snapshot) => {
    console.log('Resource warning:', snapshot.rssMemoryMB + 'MB used');
  }
});
```

## 🐛 Bug Fixes

- Fixed Pa11y score calculation
- Resolved technical SEO data mapping
- Improved mobile-friendliness recommendations
- Enhanced error handling throughout
- Fixed memory leaks in browser management

## ⚡ Performance Improvements

- **3x faster** parallel processing
- **50% less** memory usage with browser pooling
- **Real-time** progress monitoring
- **Automatic** resource cleanup
- **Smart** retry mechanisms

## 🔧 Configuration

### New Configuration Options
```javascript
{
  // Browser pool configuration
  browserPool: {
    maxConcurrent: 3,
    maxIdleTime: 30000,
    enableResourceOptimization: true
  },
  
  // Performance monitoring
  performanceMonitoring: {
    enabled: true,
    memoryLeakThreshold: 50 * 1024 * 1024
  },
  
  // Webhook configuration
  webhooks: {
    retries: 3,
    timeout: 30000
  },
  
  // 🆕 Adaptive Backpressure Configuration
  queue: {
    enableBackpressure: true,
    maxQueueSize: 1000,
    maxMemoryUsage: 2048, // MB
    backpressureThreshold: 0.8,
    adaptiveDelay: true,
    enableResourceMonitoring: true,
    enableGarbageCollection: true
  }
}
```

## 🧪 Testing Your Upgrade

Run the comprehensive test:

```bash
# Test basic functionality
auditmysite https://example.com --format html

# Test multiple pages processing
auditmysite https://example.com/sitemap.xml --max-pages 3

# Test performance monitoring
node -e "
const { performanceMonitor } = require('@casoon/auditmysite');
performanceMonitor.benchmark('test', () => {
  return new Promise(resolve => setTimeout(resolve, 100));
}, { iterations: 10 }).then(() => {
  console.log(performanceMonitor.generateReport());
});
"
```

## 📊 Monitoring & Analytics

### Performance Metrics
```javascript
import { performanceMonitor } from '@casoon/auditmysite';

// Get comprehensive stats
const stats = performanceMonitor.getStats();
console.log(`Average duration: ${stats.averageDuration}ms`);
console.log(`Memory usage: ${stats.peakMemoryUsage} bytes`);

// Export metrics
const report = performanceMonitor.export();
```

### Browser Pool Status
```javascript
import { BrowserPoolManager } from '@casoon/auditmysite';

const poolManager = new BrowserPoolManager();
const status = poolManager.getStatus();
console.log(`Active browsers: ${status.activeBrowsers}`);
console.log(`Pool efficiency: ${status.metrics.efficiency}%`);
```

### Webhook Delivery Stats
```javascript
import { webhookManager } from '@casoon/auditmysite';

const stats = webhookManager.getDeliveryStats();
console.log(`Success rate: ${stats.successRate}%`);
```

## 🆕 Environment Variables for Resource Management

You can configure backpressure and resource monitoring via environment variables:

### Queue Configuration
```bash
# Core queue settings
export QUEUE_MAX_CONCURRENT=5
export QUEUE_MAX_RETRIES=3
export QUEUE_TIMEOUT=30000
export QUEUE_MAX_SIZE=1000

# Backpressure settings
export QUEUE_ENABLE_BACKPRESSURE=true
export QUEUE_BACKPRESSURE_THRESHOLD=0.8
export QUEUE_MAX_MEMORY_MB=2048
export QUEUE_MIN_DELAY_MS=10
export QUEUE_MAX_DELAY_MS=5000

# Resource monitoring
export QUEUE_ENABLE_RESOURCE_MONITORING=true
export QUEUE_MEMORY_WARNING_MB=1536
export QUEUE_MEMORY_CRITICAL_MB=2048
export QUEUE_SAMPLING_INTERVAL_MS=2000

# Performance tuning
export QUEUE_ENABLE_ADAPTIVE_DELAY=true
export QUEUE_ENABLE_GC=true
export QUEUE_GC_INTERVAL=30000
```

### Production Configuration
```bash
# Recommended production settings
export QUEUE_ENABLE_BACKPRESSURE=true
export QUEUE_ENABLE_RESOURCE_MONITORING=true
export QUEUE_MAX_MEMORY_MB=3072
export QUEUE_MAX_CONCURRENT=4
export QUEUE_ENABLE_GC=true
export QUEUE_BACKPRESSURE_THRESHOLD=0.75
```

### CI/Test Configuration  
```bash
# Minimal settings for CI environments
export NODE_ENV=test
export QUEUE_ENABLE_BACKPRESSURE=false
export QUEUE_ENABLE_RESOURCE_MONITORING=false
export QUEUE_MAX_CONCURRENT=2
export QUEUE_MAX_SIZE=50
```

## 📊 Backpressure Metrics

### Understanding the Metrics
```javascript
import { AdaptiveBackpressureController } from '@casoon/auditmysite';

const controller = new AdaptiveBackpressureController({ enabled: true });
controller.updateQueueState(queueLength, concurrency, hasError);

const metrics = controller.getMetrics();
console.log({
  isActive: metrics.isActive,              // Backpressure currently active
  currentDelay: metrics.currentDelay,      // Current adaptive delay (ms)
  memoryUsageMB: metrics.memoryUsageMB,    // Current memory usage
  cpuUsagePercent: metrics.cpuUsagePercent,// Current CPU usage
  errorRate: metrics.errorRate,            // Recent error rate (%)
  activationCount: metrics.activationCount,// Total activations
  peakMemoryMB: metrics.peakMemoryMB,      // Peak memory seen
  gcCount: metrics.gcCount                 // Garbage collections triggered
});
```

### Health Score Interpretation
- **90-100**: Excellent system health
- **70-89**: Good performance, minor pressure
- **50-69**: Moderate pressure, backpressure may activate
- **30-49**: High pressure, performance degraded
- **0-29**: Critical state, aggressive backpressure active

## 🔧 Troubleshooting Resource Issues

### Memory Warnings
```
⚠️ RSS memory usage high: 1800.5MB
```
**Solution**: Enable backpressure or reduce maxConcurrent

### Backpressure Activation
```
🔄 Backpressure activated: Memory/CPU pressure detected (250ms delay)
```
**Normal**: System is automatically managing load

### Critical Memory Alerts
```
🔴 Heap usage critical: 92.3%
```
**Action**: Reduce queue size, enable GC, or increase memory limit

### Worker Health Issues
```
⚠️ Worker health score low: 45% (worker_2)
```
**Solution**: Worker will be automatically replaced or scaled down
console.log(`Success rate: ${stats.successRate}%`);
```

## 🔄 Migration Guide

### From v1.x to v2.0

1. **Update CLI usage**: Enhanced analysis is now default
2. **Update API calls**: New response format with additional data
3. **Update webhook handlers**: New event types available
4. **Update report parsing**: New HTML structure and interactive features

### Breaking Changes Details

#### CLI Options
- `--enhanced` flag is **deprecated** (now default)
- New granular control flags (`--no-performance`, etc.)

#### API Response Format
```javascript
// Old format
{
  "accessibility": { ... },
  "qualityScore": 85
}

// New format
{
  "accessibility": { ... },
  "performance": { ... },
  "seo": { ... },
  "mobileFriendliness": { ... },
  "contentWeight": { ... },
  "qualityScore": 85,
  "detailedIssues": [ ... ]
}
```

## 🆘 Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Check browser pool configuration
   - Monitor with performance tools
   - Adjust `maxConcurrent` settings

2. **Webhook Delivery Failures**
   - Verify endpoint URLs
   - Check webhook signatures
   - Review retry configuration

3. **Slow Performance**
   - Enable resource optimization
   - Adjust concurrency limits
   - Monitor system resources

### Getting Help
- Check the [documentation](https://github.com/your-repo/wiki)
- Report issues on [GitHub](https://github.com/your-repo/issues)
- Join our [community](https://discord.gg/auditmysite)

---

## 📈 Next Steps

After upgrading, consider:
1. Setting up webhook notifications
2. Implementing batch processing for large sites
3. Using performance monitoring for optimization
4. Exploring the new interactive reports
5. Integrating with your CI/CD pipeline

Happy auditing with AuditMySite v2.0! 🎉
