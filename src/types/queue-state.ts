export interface QueueState {
  id: string;
  urls: string[];
  processedUrls: string[];
  failedUrls: string[];
  currentIndex: number;
  totalUrls: number;
  results: any[];
  startTime: number;
  lastUpdateTime: number;
  options: {
    concurrency: number;
    retryLimit: number;
    [key: string]: any;
  };
  status: 'pending' | 'processing' | 'paused' | 'completed' | 'failed';
  metadata: {
    projectName?: string;
    sitemapUrl?: string;
    version: string;
    [key: string]: any;
  };
}

export interface QueueStateAdapter {
  save(state: QueueState): Promise<void>;
  load(id: string): Promise<QueueState | null>;
  exists(id: string): Promise<boolean>;
  delete(id: string): Promise<void>;
  list(): Promise<string[]>;
  cleanup(maxAge?: number): Promise<void>;
}

export interface QueueStateOptions {
  adapter?: QueueStateAdapter;
  autoSave?: boolean;
  saveInterval?: number;
  stateId?: string;
  resumable?: boolean;
}

export interface ResumeOptions {
  stateId: string;
  adapter?: QueueStateAdapter;
  skipCompleted?: boolean;
}

export class QueueStateError extends Error {
  constructor(message: string, public cause?: Error) {
    super(message);
    this.name = 'QueueStateError';
  }
}
