/**
 * Event-Driven Queue Integration Tests
 * 
 * Tests the core event-driven queue system with mocked processors to avoid browser dependencies.
 */

import { EventDrivenQueue } from '../../src/core/pipeline/event-driven-queue';
import { AccessibilityResult } from '../../src/types';

describe('Event-Driven Queue Integration Tests', () => {
  let queue: EventDrivenQueue;
  let mockEventCallbacks: any;
  let mockProcessor: jest.MockedFunction<(url: string) => Promise<AccessibilityResult>>;
  
  const VALID_TEST_URLS = [
    'https://example.com',
    'https://example.com/about', 
    'https://example.com/contact',
  ];
  
  const INVALID_TEST_URLS = [
    'https://invalid-domain-12345.com',
    'https://httpbin.org/status/500',
  ];

  beforeEach(() => {
    // Reset event callback tracking
    mockEventCallbacks = {
      onUrlStarted: jest.fn(),
      onUrlCompleted: jest.fn(),
      onUrlFailed: jest.fn(),
      onProgressUpdate: jest.fn(),
      onQueueEmpty: jest.fn(),
    };

    // Create mock processor that simulates accessibility testing
    mockProcessor = jest.fn().mockImplementation(async (url: string): Promise<AccessibilityResult> => {
      // Simulate processing delay
      await new Promise(resolve => setTimeout(resolve, 10 + Math.random() * 50));
      
      // Simulate failure for invalid URLs
      const isValidUrl = VALID_TEST_URLS.includes(url);
      if (!isValidUrl) {
        throw new Error(`Failed to process ${url}`);
      }
      
      // Return mock AccessibilityResult
      return {
        url,
        title: `Mock page title for ${url}`,
        imagesWithoutAlt: Math.floor(Math.random() * 5),
        buttonsWithoutLabel: Math.floor(Math.random() * 3),
        headingsCount: Math.floor(Math.random() * 10) + 1,
        errors: [],
        warnings: [`Mock warning for ${url}`],
        passed: true,
        duration: 50 + Math.random() * 100
      };
    });

    // Initialize queue with event callbacks
    queue = new EventDrivenQueue({
      maxConcurrent: 3,
      enableEvents: true,
      enableShortStatus: true
    });
    
    // Set up event listeners using the QueueEvent interface
    queue.onUrlStarted((event: any) => {
      mockEventCallbacks.onUrlStarted(event.url);
    });
    queue.onUrlCompleted((event: any) => {
      mockEventCallbacks.onUrlCompleted(event.url, event.result, event.duration || 0);
    });
    queue.onUrlFailed((event: any) => {
      mockEventCallbacks.onUrlFailed(event.url, event.error, event.attempts);
    });
    queue.onProgressUpdate((event: any) => {
      mockEventCallbacks.onProgressUpdate(event.stats);
    });
    queue.onQueueEmpty((event: any) => {
      mockEventCallbacks.onQueueEmpty();
    });
  });

  afterEach(async () => {
    if (queue) {
      queue.cleanup();
    }
  });

  describe('Success Scenarios', () => {
    test('should handle multiple URLs with complete event flow', async () => {
      const results = await queue.processUrls(VALID_TEST_URLS, {
        processor: mockProcessor
      });

      // Validate results structure
      expect(results).toBeInstanceOf(Array);
      expect(results).toHaveLength(VALID_TEST_URLS.length);

      // Validate event callbacks were triggered
      expect(mockEventCallbacks.onUrlStarted).toHaveBeenCalledTimes(VALID_TEST_URLS.length);
      expect(mockEventCallbacks.onUrlCompleted).toHaveBeenCalled();
      expect(mockEventCallbacks.onProgressUpdate).toHaveBeenCalled();
      expect(mockEventCallbacks.onQueueEmpty).toHaveBeenCalledTimes(1);

      // Check that all URLs were processed
      const processedUrls = results.map(r => r.url);
      VALID_TEST_URLS.forEach(url => {
        expect(processedUrls).toContain(url);
      });

      // Validate mock processor was called for each URL
      expect(mockProcessor).toHaveBeenCalledTimes(VALID_TEST_URLS.length);
    });

    test('should maintain event callback order and data consistency', async () => {
      const eventLog: Array<{type: string, url?: string, timestamp: number}> = [];
      
      // Create a separate queue with sequential processing for this test
      const sequentialQueue = new EventDrivenQueue({
        maxConcurrent: 1, // Force sequential to test order
        enableEvents: true,
        enableShortStatus: true
      });
      
      // Set up tracking callbacks
      sequentialQueue.onUrlStarted((event: any) => {
        eventLog.push({type: 'started', url: event.url, timestamp: Date.now()});
      });
      
      sequentialQueue.onUrlCompleted((event: any) => {
        eventLog.push({type: 'completed', url: event.url, timestamp: Date.now()});
        expect(event.result).toBeDefined();
        expect(event.result.url).toBe(event.url);
      });
      
      sequentialQueue.onProgressUpdate((event: any) => {
        eventLog.push({type: 'progress', timestamp: Date.now()});
        const progress = event.stats;
        expect(progress.completed).toBeGreaterThanOrEqual(0);
        expect(progress.total).toBeGreaterThan(0);
        expect(progress.completed).toBeLessThanOrEqual(progress.total);
      });
      
      sequentialQueue.onQueueEmpty((event: any) => {
        eventLog.push({type: 'empty', timestamp: Date.now()});
      });

      try {
        await sequentialQueue.processUrls(VALID_TEST_URLS.slice(0, 2), {
          processor: mockProcessor
        });

        // Validate event order
        expect(eventLog.length).toBeGreaterThan(0);
        expect(eventLog.filter(e => e.type === 'started')).toHaveLength(2);
        expect(eventLog.filter(e => e.type === 'completed')).toHaveLength(2);
        expect(eventLog.filter(e => e.type === 'empty')).toHaveLength(1);
        
        // Queue empty should be last
        const lastEvent = eventLog[eventLog.length - 1];
        expect(lastEvent.type).toBe('empty');
      } finally {
        sequentialQueue.cleanup();
      }
    });
  });

  describe('Failure Handling', () => {
    test('should handle URL failures with proper event callbacks', async () => {
      const mixedUrls = [...VALID_TEST_URLS.slice(0, 1), ...INVALID_TEST_URLS.slice(0, 1)];
      
      const results = await queue.processUrls(mixedUrls, {
        processor: mockProcessor
      });

      // Should still get results structure (successful ones only)
      expect(results).toBeInstanceOf(Array);
      expect(results.length).toBeLessThanOrEqual(mixedUrls.length);
      
      // Check that failures were tracked
      expect(mockEventCallbacks.onUrlFailed).toHaveBeenCalled();
      // URL started may be called more times due to retries
      expect(mockEventCallbacks.onUrlStarted).toHaveBeenCalledWith(
        expect.stringContaining('example.com')
      );
      
      // Queue should still complete
      expect(mockEventCallbacks.onQueueEmpty).toHaveBeenCalledTimes(1);
    });

    test('should continue processing after individual URL failures', async () => {
      const mixedUrls = [
        'https://example.com',
        'https://invalid-domain-12345.com',
        'https://example.com/about'
      ];

      const results = await queue.processUrls(mixedUrls, {
        processor: mockProcessor
      });

      // Should complete processing despite failures
      expect(mockEventCallbacks.onQueueEmpty).toHaveBeenCalledTimes(1);
      expect(results.length).toBe(2); // Only valid URLs should return results
      expect(results.filter(r => r.passed).length).toBe(2);
    });
  });

  describe('Resource Management', () => {
    test('should respect concurrency limits', async () => {
      const concurrentUrls: string[] = [];
      const maxConcurrent = 2;

      const concurrencyQueue = new EventDrivenQueue({
        maxConcurrent: maxConcurrent,
        enableEvents: true,
        enableShortStatus: true
      });

      concurrencyQueue.onUrlStarted((event: any) => {
        concurrentUrls.push(event.url);
      });
      
      concurrencyQueue.onUrlCompleted((event: any) => {
        const index = concurrentUrls.indexOf(event.url);
        if (index > -1) {
          concurrentUrls.splice(index, 1);
        }
        // At any point, concurrent URLs should not exceed limit
        expect(concurrentUrls.length).toBeLessThanOrEqual(maxConcurrent);
      });

      try {
        await concurrencyQueue.processUrls(VALID_TEST_URLS, {
          processor: mockProcessor
        });

        expect(concurrentUrls.length).toBe(0); // All should be completed
      } finally {
        concurrencyQueue.cleanup();
      }
    });

    test('should handle memory pressure gracefully', async () => {
      // Create a large number of URLs to test memory handling
      const manyUrls = Array(10).fill(0).map((_, i) => `https://example.com/page${i}`);
      
      // Update mock to handle new URLs
      mockProcessor.mockImplementation(async (url: string): Promise<AccessibilityResult> => {
        await new Promise(resolve => setTimeout(resolve, 10));
        return {
          url,
          title: `Mock title for ${url}`,
          imagesWithoutAlt: 0,
          buttonsWithoutLabel: 0,
          headingsCount: 1,
          errors: [],
          warnings: [],
          passed: true,
          duration: 10
        };
      });
      
      const results = await queue.processUrls(manyUrls, {
        processor: mockProcessor
      });

      expect(results.length).toBe(manyUrls.length);
      expect(mockEventCallbacks.onQueueEmpty).toHaveBeenCalledTimes(1);
    });
  });

  describe('Data Quality and Structure', () => {
    test('should produce consistent JSON structure across events and final results', async () => {
      const completedResults: any[] = [];
      
      queue.onUrlCompleted((event: any) => {
        completedResults.push(event.result);
        
        // Validate individual result structure (AccessibilityResult interface)
        expect(event.result).toHaveProperty('url');
        expect(event.result).toHaveProperty('passed');
        expect(event.result).toHaveProperty('errors');
        expect(event.result).toHaveProperty('warnings');
        expect(event.result).toHaveProperty('duration');
        expect(event.result).toHaveProperty('title');
        expect(event.result).toHaveProperty('imagesWithoutAlt');
        expect(event.result).toHaveProperty('buttonsWithoutLabel');
        expect(event.result).toHaveProperty('headingsCount');
      });

      const finalResults = await queue.processUrls(VALID_TEST_URLS.slice(0, 2), {
        processor: mockProcessor
      });

      // Compare event results with final results
      expect(completedResults.length).toBe(finalResults.length);
      
      completedResults.forEach(eventResult => {
        const finalResult = finalResults.find(r => r.url === eventResult.url);
        expect(finalResult).toBeDefined();
        
        // Key fields should match
        expect(finalResult!.url).toBe(eventResult.url);
        expect(finalResult!.passed).toBe(eventResult.passed);
        expect(finalResult!.duration).toBe(eventResult.duration);
      });
    });
  });

  describe('Performance Metrics', () => {
    test('should track processing metrics accurately', async () => {
      const startTime = Date.now();
      
      const results = await queue.processUrls(VALID_TEST_URLS, {
        processor: mockProcessor
      });
      
      const endTime = Date.now();
      const totalDuration = endTime - startTime;

      expect(results.length).toBe(VALID_TEST_URLS.length);
      expect(totalDuration).toBeGreaterThan(0);
      
      // Each result should have a realistic duration
      results.forEach(result => {
        expect(result.duration).toBeGreaterThan(0);
        expect(result.duration).toBeLessThan(1000); // Should be under 1 second for mock
      });
      
      console.log(`✅ Processed ${results.length} URLs in ${totalDuration}ms`);
      console.log(`📊 Average time per URL: ${(totalDuration / results.length).toFixed(1)}ms`);
    });
  });
});
