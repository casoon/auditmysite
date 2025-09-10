import { AccessibilityResult } from '../../types/audit-results';
import { EnhancedReportSummary } from './enhanced-report-generator';

/**
 * Stream Event Types for Tauri Integration
 */
export type StreamEventType = 'init' | 'progress' | 'page_result' | 'summary' | 'error' | 'complete';

export interface StreamEvent {
  type: StreamEventType;
  sessionId: string;
  timestamp: string;
  data: unknown;
}

export interface InitEvent extends StreamEvent {
  type: 'init';
  data: {
    sessionId: string;
    config: StreamingConfiguration;
    totalPages: number;
  };
}

export interface ProgressEvent extends StreamEvent {
  type: 'progress';
  data: {
    current: number;
    total: number;
    currentUrl: string;
    timeElapsed: number;
    estimatedRemaining?: number;
    stage: 'parsing_sitemap' | 'testing_pages' | 'generating_report';
    percentage: number;
  };
}

export interface PageResultEvent extends StreamEvent {
  type: 'page_result';
  data: AccessibilityResult & {
    streamingMeta: {
      processedAt: string;
      processingTime: number;
      chunkId: string;
      sequenceNumber: number;
    };
  };
}

export interface SummaryEvent extends StreamEvent {
  type: 'summary';
  data: EnhancedReportSummary;
}

export interface ErrorEvent extends StreamEvent {
  type: 'error';
  data: {
    error: string;
    url?: string;
    recoverable: boolean;
    stage: string;
  };
}

export interface CompleteEvent extends StreamEvent {
  type: 'complete';
  data: {
    totalTime: number;
    totalPages: number;
    successfulPages: number;
    failedPages: number;
    summary: EnhancedReportSummary;
  };
}

export interface StreamingConfiguration {
  enabled: boolean;
  chunkSize: number;
  bufferTimeout: number;
  includeDetailedResults: boolean;
  compressResults: boolean;
}

/**
 * Streaming Reporter for Real-time Tauri Integration
 * Outputs NDJSON (Newline Delimited JSON) for efficient parsing
 */
export class StreamingReporter {
  private outputStream: NodeJS.WritableStream;
  private sessionId: string;
  private config: StreamingConfiguration;
  private startTime: number;
  private sequenceNumber: number = 0;
  private resultBuffer: AccessibilityResult[] = [];
  private bufferTimeout?: NodeJS.Timeout;

  constructor(
    outputStream: NodeJS.WritableStream,
    sessionId: string,
    config: StreamingConfiguration = {
      enabled: true,
      chunkSize: 10,
      bufferTimeout: 1000,
      includeDetailedResults: true,
      compressResults: false
    }
  ) {
    this.outputStream = outputStream;
    this.sessionId = sessionId;
    this.config = config;
    this.startTime = Date.now();
  }

  /**
   * Initialize streaming session
   */
  init(totalPages: number, config: any): void {
    const event: InitEvent = {
      type: 'init',
      sessionId: this.sessionId,
      timestamp: new Date().toISOString(),
      data: {
        sessionId: this.sessionId,
        config: this.config,
        totalPages
      }
    };
    
    this.writeEvent(event);
  }

  /**
   * Report progress updates
   */
  reportProgress(data: {
    current: number;
    total: number;
    currentUrl: string;
    stage: 'parsing_sitemap' | 'testing_pages' | 'generating_report';
  }): void {
    const timeElapsed = Date.now() - this.startTime;
    const percentage = Math.round((data.current / data.total) * 100);
    
    // Calculate estimated remaining time
    let estimatedRemaining: number | undefined;
    if (data.current > 0 && data.stage === 'testing_pages') {
      const avgTimePerPage = timeElapsed / data.current;
      estimatedRemaining = Math.round(avgTimePerPage * (data.total - data.current));
    }

    const event: ProgressEvent = {
      type: 'progress',
      sessionId: this.sessionId,
      timestamp: new Date().toISOString(),
      data: {
        ...data,
        timeElapsed,
        estimatedRemaining,
        percentage
      }
    };

    this.writeEvent(event);
  }

  /**
   * Report individual page result
   */
  reportPageResult(result: AccessibilityResult, processingTime: number): void {
    const chunkId = this.generateChunkId();
    const streamingMeta = {
      processedAt: new Date().toISOString(),
      processingTime,
      chunkId,
      sequenceNumber: ++this.sequenceNumber
    };

    const enhancedResult = {
      ...result,
      streamingMeta
    };

    if (this.config.includeDetailedResults) {
      const event: PageResultEvent = {
        type: 'page_result',
        sessionId: this.sessionId,
        timestamp: new Date().toISOString(),
        data: enhancedResult
      };

      this.writeEvent(event);
    }

    // Add to buffer for batch processing if needed
    this.resultBuffer.push(enhancedResult);
    this.maybeFlushBuffer();
  }

  /**
   * Report summary data
   */
  reportSummary(summary: EnhancedReportSummary): void {
    const event: SummaryEvent = {
      type: 'summary',
      sessionId: this.sessionId,
      timestamp: new Date().toISOString(),
      data: summary
    };

    this.writeEvent(event);
  }

  /**
   * Report errors
   */
  reportError(error: string, url?: string, stage: string = 'unknown', recoverable: boolean = false): void {
    const event: ErrorEvent = {
      type: 'error',
      sessionId: this.sessionId,
      timestamp: new Date().toISOString(),
      data: {
        error,
        url,
        recoverable,
        stage
      }
    };

    this.writeEvent(event);
  }

  /**
   * Report completion
   */
  complete(summary: EnhancedReportSummary, totalPages: number, successfulPages: number): void {
    // Flush any remaining buffer
    this.flushBuffer();

    const totalTime = Date.now() - this.startTime;
    const event: CompleteEvent = {
      type: 'complete',
      sessionId: this.sessionId,
      timestamp: new Date().toISOString(),
      data: {
        totalTime,
        totalPages,
        successfulPages,
        failedPages: totalPages - successfulPages,
        summary
      }
    };

    this.writeEvent(event);
  }

  /**
   * Get real-time metrics
   */
  getRealTimeMetrics(): {
    pagesPerSecond: number;
    memoryUsage: number;
    timeElapsed: number;
  } {
    const timeElapsed = Date.now() - this.startTime;
    const pagesPerSecond = this.sequenceNumber / (timeElapsed / 1000);
    const memoryUsage = process.memoryUsage().heapUsed;

    return {
      pagesPerSecond,
      memoryUsage,
      timeElapsed
    };
  }

  /**
   * Write event to output stream in NDJSON format
   */
  private writeEvent(event: StreamEvent): void {
    try {
      const jsonLine = JSON.stringify(event) + '\n';
      
      // Compress if configured and data is large
      if (this.config.compressResults && jsonLine.length > 1024 * 10) { // > 10KB
        // Could implement compression here if needed
      }

      this.outputStream.write(jsonLine, 'utf8');
    } catch (error) {
      console.error('Failed to write stream event:', error);
    }
  }

  /**
   * Generate unique chunk ID
   */
  private generateChunkId(): string {
    return `${this.sessionId}-${Date.now()}-${this.sequenceNumber}`;
  }

  /**
   * Maybe flush buffer based on configuration
   */
  private maybeFlushBuffer(): void {
    if (this.resultBuffer.length >= this.config.chunkSize) {
      this.flushBuffer();
    } else if (!this.bufferTimeout) {
      this.bufferTimeout = setTimeout(() => {
        this.flushBuffer();
      }, this.config.bufferTimeout);
    }
  }

  /**
   * Flush result buffer
   */
  private flushBuffer(): void {
    if (this.bufferTimeout) {
      clearTimeout(this.bufferTimeout);
      this.bufferTimeout = undefined;
    }

    if (this.resultBuffer.length === 0) return;

    // Could emit a batch event if needed for efficiency
    // For now, individual page results are already emitted
    this.resultBuffer = [];
  }

  /**
   * Create streaming reporter from options
   */
  static create(
    sessionId: string,
    outputStream: NodeJS.WritableStream = process.stdout,
    options: Partial<StreamingConfiguration> = {}
  ): StreamingReporter {
    const config: StreamingConfiguration = {
      enabled: true,
      chunkSize: 10,
      bufferTimeout: 1000,
      includeDetailedResults: true,
      compressResults: false,
      ...options
    };

    return new StreamingReporter(outputStream, sessionId, config);
  }

  /**
   * Cleanup resources
   */
  cleanup(): void {
    if (this.bufferTimeout) {
      clearTimeout(this.bufferTimeout);
      this.bufferTimeout = undefined;
    }
    this.flushBuffer();
  }
}
