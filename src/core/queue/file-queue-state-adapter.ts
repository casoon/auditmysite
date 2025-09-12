import * as fs from 'fs/promises';
import * as path from 'path';
import { QueueState, QueueStateAdapter, QueueStateError } from '../../types/queue-state';

export class FileQueueStateAdapter implements QueueStateAdapter {
  private stateDir: string;

  constructor(stateDir: string = './.queue-states') {
    this.stateDir = path.resolve(stateDir);
  }

  async save(state: QueueState): Promise<void> {
    try {
      // Ensure state directory exists
      await fs.mkdir(this.stateDir, { recursive: true });
      
      const statePath = this.getStatePath(state.id);
      const stateData = JSON.stringify(state, null, 2);
      
      // Write state atomically using a temporary file
      const tempPath = `${statePath}.tmp`;
      await fs.writeFile(tempPath, stateData, 'utf8');
      await fs.rename(tempPath, statePath);
      
    } catch (error) {
      throw new QueueStateError(
        `Failed to save queue state ${state.id}: ${error instanceof Error ? error.message : String(error)}`,
        error instanceof Error ? error : new Error(String(error))
      );
    }
  }

  async load(id: string): Promise<QueueState | null> {
    try {
      const statePath = this.getStatePath(id);
      
      // Check if state file exists
      try {
        await fs.access(statePath);
      } catch {
        return null; // State doesn't exist
      }
      
      const stateData = await fs.readFile(statePath, 'utf8');
      const state = JSON.parse(stateData) as QueueState;
      
      // Validate basic structure
      if (!state.id || !Array.isArray(state.urls)) {
        throw new Error('Invalid state structure');
      }
      
      return state;
      
    } catch (error) {
      throw new QueueStateError(
        `Failed to load queue state ${id}: ${error instanceof Error ? error.message : String(error)}`,
        error instanceof Error ? error : new Error(String(error))
      );
    }
  }

  async exists(id: string): Promise<boolean> {
    try {
      const statePath = this.getStatePath(id);
      await fs.access(statePath);
      return true;
    } catch {
      return false;
    }
  }

  async delete(id: string): Promise<void> {
    try {
      const statePath = this.getStatePath(id);
      await fs.unlink(statePath);
    } catch (error) {
      if (error instanceof Error && 'code' in error && error.code !== 'ENOENT') {
        throw new QueueStateError(
          `Failed to delete queue state ${id}: ${error.message}`,
          error
        );
      } else if (!(error instanceof Error) || !('code' in error) || error.code !== 'ENOENT') {
        throw new QueueStateError(
          `Failed to delete queue state ${id}: ${error instanceof Error ? error.message : String(error)}`,
          error instanceof Error ? error : new Error(String(error))
        );
      }
    }
  }

  async list(): Promise<string[]> {
    try {
      // Check if state directory exists
      try {
        await fs.access(this.stateDir);
      } catch {
        return []; // Directory doesn't exist, no states
      }
      
      const files = await fs.readdir(this.stateDir);
      const stateFiles = files
        .filter(file => file.endsWith('.json'))
        .map(file => path.basename(file, '.json'));
      
      return stateFiles;
      
    } catch (error) {
      throw new QueueStateError(
        `Failed to list queue states: ${error instanceof Error ? error.message : String(error)}`,
        error instanceof Error ? error : new Error(String(error))
      );
    }
  }

  async cleanup(maxAge: number = 7 * 24 * 60 * 60 * 1000): Promise<void> {
    try {
      // Check if state directory exists
      try {
        await fs.access(this.stateDir);
      } catch {
        return; // Directory doesn't exist, nothing to clean
      }
      
      const files = await fs.readdir(this.stateDir);
      const now = Date.now();
      
      for (const file of files) {
        if (!file.endsWith('.json')) continue;
        
        const filePath = path.join(this.stateDir, file);
        const stats = await fs.stat(filePath);
        
        if (now - stats.mtime.getTime() > maxAge) {
          await fs.unlink(filePath);
        }
      }
      
    } catch (error) {
      throw new QueueStateError(
        `Failed to cleanup old queue states: ${error instanceof Error ? error.message : String(error)}`,
        error instanceof Error ? error : new Error(String(error))
      );
    }
  }

  private getStatePath(id: string): string {
    // Sanitize the ID to create a safe filename
    const safeId = id.replace(/[^a-zA-Z0-9-_]/g, '_');
    return path.join(this.stateDir, `${safeId}.json`);
  }

  /**
   * Get information about a specific state without loading the full data
   */
  async getStateInfo(id: string): Promise<{
    id: string;
    status: string;
    totalUrls: number;
    processedUrls: number;
    lastUpdateTime: number;
    size: number;
  } | null> {
    try {
      const statePath = this.getStatePath(id);
      const stats = await fs.stat(statePath);
      const stateData = await fs.readFile(statePath, 'utf8');
      const state = JSON.parse(stateData) as QueueState;
      
      return {
        id: state.id,
        status: state.status,
        totalUrls: state.totalUrls,
        processedUrls: state.processedUrls.length,
        lastUpdateTime: state.lastUpdateTime,
        size: stats.size
      };
      
    } catch {
      return null;
    }
  }
}
