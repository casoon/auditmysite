import ora, { Ora } from 'ora';

/**
 * Enhanced Progress Tracker with ETA, Speed, and Detailed Info
 */
export class EnhancedProgress {
  private spinner: Ora;
  private startTime: number;
  private total: number;
  private current: number = 0;
  private lastUpdate: number;
  private processingTimes: number[] = [];
  
  constructor(total: number, initialMessage: string = 'Processing...') {
    this.total = total;
    this.startTime = Date.now();
    this.lastUpdate = this.startTime;
    
    this.spinner = ora({
      text: initialMessage,
      spinner: {
        interval: 100,
        frames: ['🚀', '⚡', '🎯', '🔍', '📊', '✨']
      }
    }).start();
  }

  /**
   * Update progress with current page info
   */
  update(currentPage: number, currentUrl?: string, stage?: string): void {
    this.current = currentPage;
    const now = Date.now();
    
    // Track processing time for this page
    if (this.current > 1) {
      const pageTime = now - this.lastUpdate;
      this.processingTimes.push(pageTime);
      
      // Keep only last 10 measurements for better accuracy
      if (this.processingTimes.length > 10) {
        this.processingTimes.shift();
      }
    }
    this.lastUpdate = now;
    
    const percentage = Math.round((this.current / this.total) * 100);
    const timeElapsed = Math.round((now - this.startTime) / 1000);
    const eta = this.calculateETA();
    const speed = this.calculateSpeed();
    
    // Create progress bar
    const progressBarLength = 20;
    const filledLength = Math.round((this.current / this.total) * progressBarLength);
    const progressBar = '█'.repeat(filledLength) + '░'.repeat(progressBarLength - filledLength);
    
    let text = `${stage ? stage + ' - ' : ''}Testing pages... ${progressBar} ${percentage}% (${this.current}/${this.total})`;
    
    if (currentUrl) {
      const url = this.truncateUrl(currentUrl);
      text += `\n   Current: ${url}`;
    }
    
    if (eta > 0) {
      text += `\n   ETA: ${this.formatTime(eta)}`;
    }
    
    if (speed > 0) {
      text += ` | Speed: ${speed.toFixed(1)} pages/min`;
    }
    
    text += ` | Elapsed: ${this.formatTime(timeElapsed)}`;
    
    this.spinner.text = text;
  }

  /**
   * Mark as completed with success
   */
  succeed(message: string): void {
    this.spinner.succeed(message);
  }

  /**
   * Mark as failed with error
   */
  fail(message: string): void {
    this.spinner.fail(message);
  }

  /**
   * Stop the spinner
   */
  stop(): void {
    this.spinner.stop();
  }

  /**
   * Calculate estimated time to completion
   */
  private calculateETA(): number {
    if (this.current <= 1 || this.processingTimes.length === 0) {
      return 0;
    }
    
    // Average processing time per page (in milliseconds)
    const avgTimePerPage = this.processingTimes.reduce((a, b) => a + b, 0) / this.processingTimes.length;
    
    // Remaining pages
    const remainingPages = this.total - this.current;
    
    // ETA in seconds
    return Math.round((remainingPages * avgTimePerPage) / 1000);
  }

  /**
   * Calculate processing speed in pages per minute
   */
  private calculateSpeed(): number {
    if (this.current <= 1) {
      return 0;
    }
    
    const timeElapsed = Date.now() - this.startTime;
    const timeElapsedMinutes = timeElapsed / (1000 * 60);
    
    return this.current / timeElapsedMinutes;
  }

  /**
   * Format time in seconds to human readable format
   */
  private formatTime(seconds: number): string {
    if (seconds < 60) {
      return `${seconds}s`;
    } else if (seconds < 3600) {
      const minutes = Math.floor(seconds / 60);
      const remainingSeconds = seconds % 60;
      return `${minutes}m ${remainingSeconds}s`;
    } else {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      return `${hours}h ${minutes}m`;
    }
  }

  /**
   * Truncate URL for display
   */
  private truncateUrl(url: string, maxLength: number = 60): string {
    if (url.length <= maxLength) {
      return url;
    }
    
    try {
      const urlObj = new URL(url);
      const pathname = urlObj.pathname;
      
      if (pathname.length > maxLength - 10) {
        return `${urlObj.hostname}...${pathname.slice(-30)}`;
      }
      
      return `${urlObj.hostname}${pathname}`;
    } catch {
      // If URL parsing fails, just truncate
      return url.length > maxLength ? url.substring(0, maxLength - 3) + '...' : url;
    }
  }

  /**
   * Update just the stage without changing other info
   */
  updateStage(stage: string): void {
    // Just update the stage and refresh display
    this.update(this.current, undefined, stage);
  }
}

/**
 * Simple progress tracker for backwards compatibility
 */
export class SimpleProgress {
  private spinner: Ora;
  
  constructor(message: string) {
    this.spinner = ora(message).start();
  }
  
  updateText(text: string): void {
    this.spinner.text = text;
  }
  
  succeed(message?: string): void {
    this.spinner.succeed(message);
  }
  
  fail(message?: string): void {
    this.spinner.fail(message);
  }
  
  stop(): void {
    this.spinner.stop();
  }
}
