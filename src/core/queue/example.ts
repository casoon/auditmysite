/**
 * 🔧 Queue Example & Test
 * 
 * Example usage of the queue system.
 * Shows different queue types and configurations.
 */

import { Queue, QueueEventCallbacks } from './index';

// Example: Simple sequential processing
async function exampleSimpleQueue() {
  console.log('\n📋 Simple Queue Example:');
  
  const queue = new Queue<string>('simple', {
    maxRetries: 2,
    timeout: 5000
  });

  // Add URLs to process
  const urls = [
    'https://example.com',
    'https://example.com/about',
    'https://example.com/contact'
  ];

  queue.enqueue(urls);

  // Process with mock accessibility checker
  const result = await queue.process(async (url: string) => {
    console.log(`  🔍 Testing: ${url}`);
    await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate processing
    return { url, passed: true, errors: [] };
  });

  console.log(`  ✅ Completed: ${result.completed.length}/${result.statistics.total}`);
  return result;
}

// Example: Parallel processing with progress
async function exampleParallelQueue() {
  console.log('\n🚀 Parallel Queue Example:');
  
  const callbacks: QueueEventCallbacks<string> = {
    onProgressUpdate: (stats) => {
      if (stats.progress % 25 === 0) { // Report every 25%
        console.log(`  📊 Progress: ${stats.progress.toFixed(1)}% (${stats.completed}/${stats.total})`);
      }
    },
    onItemCompleted: (item, result) => {
      console.log(`  ✅ ${item.data} completed`);
    },
    onItemFailed: (item, error) => {
      console.log(`  ❌ ${item.data} failed: ${error}`);
    }
  };

  const queue = new Queue<string>('parallel', {
    maxConcurrent: 2,
    maxRetries: 3,
    timeout: 8000,
    enableProgressReporting: true
  }, callbacks);

  // Larger set of URLs
  const urls = [
    'https://example.com',
    'https://example.com/about',
    'https://example.com/contact', 
    'https://example.com/services',
    'https://example.com/blog',
    'https://example.com/pricing'
  ];

  const result = await queue.processWithProgress(urls, async (url: string) => {
    const duration = Math.random() * 3000 + 1000; // Random 1-4 seconds
    await new Promise(resolve => setTimeout(resolve, duration));
    
    // Simulate occasional failures
    if (Math.random() < 0.2) {
      throw new Error('Simulated test failure');
    }
    
    return { 
      url, 
      passed: true, 
      errors: [], 
      duration: Math.round(duration) 
    };
  });

  const metrics = queue.getPerformanceMetrics();
  console.log(`  📈 Performance: ${metrics.efficiency.toFixed(1)}% efficiency, ${metrics.throughput.toFixed(2)} items/sec`);
  
  return result;
}

// Example: Priority-based processing
async function examplePriorityQueue() {
  console.log('\n⭐ Priority Queue Example:');
  
  const queue = new Queue<string>('priority', {
    maxConcurrent: 2,
    priorityPatterns: [
      { pattern: '/home', priority: 10 },
      { pattern: '/', priority: 9 },
      { pattern: '/about', priority: 8 },
      { pattern: '/contact', priority: 7 },
      { pattern: '/blog', priority: 5 }
    ]
  });

  // URLs with different priorities
  const urls = [
    'https://example.com/blog/post1',     // Priority 5
    'https://example.com/home',           // Priority 10 
    'https://example.com',                // Priority 9
    'https://example.com/about',          // Priority 8
    'https://example.com/blog/post2',     // Priority 5
    'https://example.com/contact'         // Priority 7
  ];

  queue.enqueue(urls);
  
  console.log('  📋 Processing order (by priority):');
  const result = await queue.process(async (url: string) => {
    console.log(`  🔍 Testing: ${url}`);
    await new Promise(resolve => setTimeout(resolve, 500));
    return { url, passed: true };
  });

  return result;
}

// Example: Accessibility testing optimized queue
async function exampleAccessibilityQueue() {
  console.log('\n♿ Accessibility Testing Queue:');
  
  // Use factory method for accessibility-optimized settings
  const queue = Queue.forAccessibilityTesting<string>('parallel', {
    maxConcurrent: 2, // Conservative for browser testing
    timeout: 30000    // Longer timeout for accessibility scans
  });

  const urls = [
    'https://example.com',
    'https://example.com/about',
    'https://example.com/products'
  ];

  const result = await queue.processWithProgress(urls, async (url: string) => {
    console.log(`  🔍 Accessibility scan: ${url}`);
    
    // Simulate accessibility testing
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    return {
      url,
      title: 'Example Page',
      passed: Math.random() > 0.3,
      errors: Math.random() > 0.5 ? [] : ['Missing alt text', 'Low contrast'],
      warnings: ['Consider adding ARIA labels'],
      duration: 2000
    };
  });

  console.log(`  📊 Accessibility Results: ${result.completed.length} pages scanned`);
  return result;
}

// Run all examples
export async function runQueueExamples() {
  console.log('🔧 Queue System Examples\\n');
  
  try {
    await exampleSimpleQueue();
    await exampleParallelQueue();
    await examplePriorityQueue();
    await exampleAccessibilityQueue();
    
    console.log('\n✅ All queue examples completed successfully!');
    console.log('🎯 The Queue system provides:');
    console.log('   - Consistent API across all queue types');
    console.log('   - Automatic retry and error handling');
    console.log('   - Real-time progress reporting');
    console.log('   - Performance metrics and optimization');
    console.log('   - Built-in accessibility testing support');
    
  } catch (error) {
    console.error('❌ Queue example failed:', error);
  }
}

// Export for use in tests
export {
  exampleSimpleQueue,
  exampleParallelQueue, 
  examplePriorityQueue,
  exampleAccessibilityQueue
};
