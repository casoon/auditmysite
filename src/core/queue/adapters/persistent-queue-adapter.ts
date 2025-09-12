/**
 * 💾 Persistent Queue Adapter
 * 
 * Queue adapter that provides persistence functionality for queue state.
 * Can save and restore queue items across application restarts.
 */

import { QueueAdapter } from '../queue-adapter';
import { QueueConfig, QueueEventCallbacks, QueueItem, QueueProcessor, QueueResult } from '../types';
import { ParallelQueueAdapter } from './parallel-queue-adapter';
import { promises as fs } from 'fs';
import path from 'path';

export interface PersistentQueueConfig extends QueueConfig {
  persistencePath?: string;
  enableAutoSave?: boolean;
  autoSaveInterval?: number;
}

export class PersistentQueueAdapter<T = any> extends ParallelQueueAdapter<T> {
  private persistencePath: string;
  private enableAutoSave: boolean;
  private autoSaveInterval: number;
  private autoSaveTimer?: NodeJS.Timeout;
  
  constructor(
    config: PersistentQueueConfig = {},
    callbacks?: QueueEventCallbacks<T>
  ) {
    super(config, callbacks);
    
    this.persistencePath = config.persistencePath || path.join(process.cwd(), '.auditmysite-queue-state.json');
    this.enableAutoSave = config.enableAutoSave !== false;
    this.autoSaveInterval = config.autoSaveInterval || 30000; // 30 seconds
    
    this.initializeAutoSave();
  }
  
  /**
   * Initialize auto-save functionality
   */
  private initializeAutoSave(): void {
    if (this.enableAutoSave) {
      this.autoSaveTimer = setInterval(() => {
        this.saveState().catch(error => {
          console.warn('Auto-save failed:', error);
        });
      }, this.autoSaveInterval);
    }
  }
  
  /**
   * Save current queue state to disk
   */
  async saveState(): Promise<void> {
    try {
      const state = {
        timestamp: new Date().toISOString(),
        config: {
          maxConcurrent: this.config.maxConcurrent,
          maxRetries: this.config.maxRetries,
          retryDelay: this.config.retryDelay,
          timeout: this.config.timeout
        },
        items: Array.from(this.items.values()).map(item => ({
          id: item.id,
          data: item.data,
          priority: item.priority,
          status: item.status,
          attempts: item.attempts,
          error: item.error,
          timestamp: item.timestamp.toISOString(),
          startedAt: item.startedAt?.toISOString(),
          completedAt: item.completedAt?.toISOString()
        })),
        statistics: this.getStatistics()
      };
      
      await fs.writeFile(this.persistencePath, JSON.stringify(state, null, 2), 'utf-8');
      
      // State saved successfully
    } catch (error) {
      console.error('Failed to save queue state:', error);
      throw new Error(`Queue state persistence failed: ${(error as Error).message}`);
    }
  }
  
  /**
   * Load queue state from disk
   */
  async loadState(): Promise<void> {
    try {
      const stateData = await fs.readFile(this.persistencePath, 'utf-8');
      const state = JSON.parse(stateData);
      
      // Restore configuration
      if (state.config) {
        this.configure({
          maxConcurrent: state.config.maxConcurrent,
          maxRetries: state.config.maxRetries,
          retryDelay: state.config.retryDelay,
          timeout: state.config.timeout
        });
      }
      
      // Restore queue items
      if (state.items && Array.isArray(state.items)) {
        this.items.clear();
        for (const itemData of state.items) {
          const item = {
            id: itemData.id,
            data: itemData.data,
            priority: itemData.priority || 5,
            status: itemData.status || 'pending',
            attempts: itemData.attempts || 0,
            maxAttempts: this.config.maxRetries || 3,
            error: itemData.error,
            timestamp: new Date(itemData.timestamp || new Date().toISOString()),
            startedAt: itemData.startedAt ? new Date(itemData.startedAt) : undefined,
            completedAt: itemData.completedAt ? new Date(itemData.completedAt) : undefined
          };
          this.items.set(item.id, item);
        }
      }
      
      console.log(`📦 Restored ${this.items.size} queue items from persistence`);
      
      // State loaded successfully
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        // File doesn't exist, start with empty state
        console.log('💾 No persistent state found, starting with empty queue');
        return;
      }
      
      console.error('Failed to load queue state:', error);
      throw new Error(`Queue state loading failed: ${(error as Error).message}`);
    }
  }
  
  /**
   * Clear persistent state file
   */
  async clearPersistedState(): Promise<void> {
    try {
      await fs.unlink(this.persistencePath);
      console.log('🗑️ Persistent state cleared');
      
      // State cleared successfully
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
        console.warn('Failed to clear persistent state:', error);
      }
    }
  }
  
  /**
   * Process queue with persistence
   */
  async process(processor: QueueProcessor<T>): Promise<QueueResult<T>> {
    // Load state before processing
    await this.loadState();
    
    // Save state before starting processing
    await this.saveState();
    
    // Run the actual processing
    const result = await super.process(processor);
    
    // Save final state
    await this.saveState();
    
    return result;
  }
  
  /**
   * Add items with persistence
   */
  enqueue(data: T[], options?: { priority?: number }): string[] {
    const ids = super.enqueue(data, options);
    
    // Trigger auto-save after adding items
    if (this.enableAutoSave) {
      this.saveState().catch(error => {
        console.warn('Failed to save state after enqueue:', error);
      });
    }
    
    return ids;
  }
  
  /**
   * Clear queue with persistence
   */
  clear(): void {
    super.clear();
    
    // Clear persistent state
    this.clearPersistedState().catch(error => {
      console.warn('Failed to clear persistent state:', error);
    });
  }
  
  /**
   * Get persistence status
   */
  getPersistenceInfo(): {
    path: string;
    autoSave: boolean;
    lastSave?: string;
  } {
    return {
      path: this.persistencePath,
      autoSave: this.enableAutoSave,
      lastSave: undefined // Could track this if needed
    };
  }
  
  /**
   * Cleanup with persistence
   */
  async cleanup(): Promise<void> {
    // Stop auto-save
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
      this.autoSaveTimer = undefined;
    }
    
    // Final save before cleanup
    if (this.items.size > 0) {
      await this.saveState();
    } else {
      // Clear persistent state if queue is empty
      await this.clearPersistedState();
    }
    
    console.log('💾 PersistentQueueAdapter cleanup completed');
  }
}
