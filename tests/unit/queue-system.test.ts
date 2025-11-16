/**
 * 🧪 Queue System Unit Tests
 * 
 * Tests the unified queue system with focus on core business logic.
 * Fast, isolated tests without I/O operations.
 * 
 * Updated for v2.0 architecture with current queue implementations.
 */

// Removing reference to legacy UnifiedQueue
import { SimpleQueueAdapter } from '../../src/core/queue/adapters/simple-queue-adapter';
import { ParallelQueueAdapter } from '../../src/core/queue/adapters/parallel-queue-adapter';
import { QueueConfig, QueueProcessor, QueueType } from '../../src/core/queue/types';
import { QueueFactory } from '../../src/core/queue/queue-factory';

// Legacy UnifiedQueue tests removed in refactoring

describe('QueueFactory', () => {
  describe('Queue Creation', () => {
    it('should create simple queue adapter', () => {
      const queue = QueueFactory.create('simple', { maxConcurrent: 1 });
      expect(queue).toBeInstanceOf(SimpleQueueAdapter);
    });

    it('should create parallel queue adapter', () => {
      const queue = QueueFactory.create('parallel', { maxConcurrent: 3 });
      expect(queue).toBeInstanceOf(ParallelQueueAdapter);
    });

    it('should create priority queue (using parallel adapter)', () => {
      const queue = QueueFactory.create('priority');
      expect(queue).toBeInstanceOf(ParallelQueueAdapter);
    });

    it('should throw error for unsupported queue type', () => {
      expect(() => {
        QueueFactory.create('invalid-type' as QueueType);
      }).toThrow('Unsupported queue type');
    });
  });

  describe('Configuration Validation', () => {
    it('should validate valid configuration', () => {
      const config: QueueConfig = {
        maxConcurrent: 3,
        maxRetries: 2,
        retryDelay: 1000,
        timeout: 30000
      };
      
      const validation = QueueFactory.validateConfig(config);
      expect(validation.valid).toBe(true);
      expect(validation.errors).toHaveLength(0);
    });

    it('should invalidate configuration with invalid maxConcurrent', () => {
      const config: QueueConfig = {
        maxConcurrent: -1
      };
      
      const validation = QueueFactory.validateConfig(config);
      expect(validation.valid).toBe(false);
      expect(validation.errors).toContain('maxConcurrent must be between 1 and 10');
    });

    it('should invalidate configuration with invalid timeout', () => {
      const config: QueueConfig = {
        timeout: 500 // Too low
      };
      
      const validation = QueueFactory.validateConfig(config);
      expect(validation.valid).toBe(false);
      expect(validation.errors).toContain('timeout must be between 1000 and 300000ms');
    });
  });

  describe('Accessibility Testing Factory', () => {
    it('should create queue optimized for accessibility testing', () => {
      const queue = QueueFactory.createForAccessibilityTesting();
      expect(queue).toBeInstanceOf(ParallelQueueAdapter);
    });

    it('should apply accessibility testing configurations', () => {
      const customConfig = { maxConcurrent: 1 };
      const queue = QueueFactory.createForAccessibilityTesting('simple', customConfig);
      expect(queue).toBeInstanceOf(SimpleQueueAdapter);
    });
  });

  describe('Utility Methods', () => {
    it('should return supported queue types', () => {
      const types = QueueFactory.getSupportedTypes();
      expect(types).toContain('simple');
      expect(types).toContain('parallel');
      expect(types).toContain('priority');
      expect(types).toContain('persistent');
    });

    it('should get default configuration for each type', () => {
      const simpleConfig = QueueFactory.getDefaultConfig('simple');
      expect(simpleConfig.maxConcurrent).toBe(1);

      const parallelConfig = QueueFactory.getDefaultConfig('parallel');
      expect(parallelConfig.maxConcurrent).toBe(3);

      const priorityConfig = QueueFactory.getDefaultConfig('priority');
      expect(priorityConfig.priorityPatterns).toBeDefined();
    });
  });
});

describe('SimpleQueueAdapter', () => {
  let adapter: SimpleQueueAdapter;

  beforeEach(() => {
    adapter = new SimpleQueueAdapter({ maxConcurrent: 1, timeout: 5000 });
  });

  it('should process elements sequentially', async () => {
    const mockProcessor: QueueProcessor<any> = jest.fn().mockResolvedValue({ success: true });
    const elements = [
      { url: 'https://example.com/1' },
      { url: 'https://example.com/2' }
    ];

    adapter.enqueue(elements);
    const result = await adapter.process(mockProcessor);
    
    expect(mockProcessor).toHaveBeenCalledTimes(2);
    expect(result.completed).toHaveLength(2);
  });

  it('should respect priority order', async () => {
    const processingOrder: string[] = [];
    const mockProcessor: QueueProcessor<any> = jest.fn().mockImplementation((data) => {
      processingOrder.push(data.url);
      return Promise.resolve({ success: true });
    });

    const elements = [
      { url: 'https://example.com/low' },
      { url: 'https://example.com/high' },
      { url: 'https://example.com/medium' }
    ];

    // Add with different priorities
    adapter.enqueue([elements[0]], { priority: 1 }); // low priority
    adapter.enqueue([elements[1]], { priority: 3 }); // high priority  
    adapter.enqueue([elements[2]], { priority: 2 }); // medium priority
    
    await adapter.process(mockProcessor);
    
    expect(processingOrder).toEqual([
      'https://example.com/high',
      'https://example.com/medium', 
      'https://example.com/low'
    ]);
  }, 10000);
});

describe('ParallelQueueAdapter', () => {
  let adapter: ParallelQueueAdapter;

  beforeEach(() => {
    adapter = new ParallelQueueAdapter({ maxConcurrent: 2, timeout: 15000 });
  });

  it('should limit concurrent processing', async () => {
    let activeProcesses = 0;
    let maxConcurrent = 0;

    const mockProcessor: QueueProcessor<any> = jest.fn().mockImplementation(() => {
      activeProcesses++;
      maxConcurrent = Math.max(maxConcurrent, activeProcesses);
      
      return new Promise(resolve => {
        setTimeout(() => {
          activeProcesses--;
          resolve({ success: true });
        }, 50);
      });
    });

    const elements = Array.from({ length: 5 }, (_, i) => ({
      url: `https://example.com/${i}`
    }));

    adapter.enqueue(elements);
    const result = await adapter.process(mockProcessor);
    
    expect(maxConcurrent).toBe(2); // Should not exceed concurrency limit
    expect(mockProcessor).toHaveBeenCalledTimes(5);
    expect(result.completed.length).toBe(5);
  }, 20000);

  it('should handle mixed success and failure in parallel processing', async () => {
    const mockProcessor: QueueProcessor<any> = jest.fn()
      .mockImplementation((data) => {
        if (data.url === 'https://example.com/2') {
          return Promise.reject(new Error('Parallel failure'));
        }
        return Promise.resolve({ success: true });
      });

    const elements = [
      { url: 'https://example.com/1' },
      { url: 'https://example.com/2' },
      { url: 'https://example.com/3' }
    ];

    adapter.enqueue(elements);
    const result = await adapter.process(mockProcessor);
    
    // Check total items processed
    const totalProcessed = result.completed.length + result.failed.length;
    expect(totalProcessed).toBe(3);
    
    // Should have at least one failure
    expect(result.failed.length).toBeGreaterThan(0);
  }, 20000);
});
