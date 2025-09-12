import { chromium, Browser, BrowserContext } from 'playwright';
import { TestOptions } from '@core/types';
import { log } from '@core/logging';

export interface BrowserConfig {
  headless?: boolean;
  slowMo?: number;
  devtools?: boolean;
  args?: string[];
  port?: number;
  verbose?: boolean; // Controls browser initialization logging
}

export class BrowserManager {
  private browser: Browser | null = null;
  private context: BrowserContext | null = null;
  private wsEndpoint: string | null = null;
  private port: number;
  private userDataDir: string | null = null;

  constructor(private config: BrowserConfig = {}) {
    this.port = config.port || 9222;
  }

  async initialize(): Promise<void> {
    if (this.config.verbose) {
      console.log('🚀 Initializing shared browser instance...');
    }
    
    // Environment detection for better error handling
    const platform = process.platform;
    const isRoot = process.getuid?.() === 0;
    
    if (isRoot && this.config.verbose) {
      console.log('⚠️  Running as root user - will use no-sandbox mode');
    }
    
    if (this.config.verbose) {
      console.log(`ℹ️  Platform: ${platform}, User: ${process.env.USER || 'unknown'}`);
    }
    
    
    // Browser mit Remote Debugging starten
    const browserArgs = [
      '--disable-web-security',
      '--disable-features=VizDisplayCompositor',
      '--remote-debugging-port=' + this.port,
      '--remote-debugging-address=127.0.0.1',
      '--disable-dev-shm-usage',
      '--disable-gpu',
      '--disable-background-timer-throttling',
      '--disable-backgrounding-occluded-windows',
      '--disable-renderer-backgrounding',
      '--no-first-run',
      '--no-default-browser-check',
      ...(this.config.args || [])
    ];
    
    // Add sandbox flags conditionally based on environment
    const isDocker = process.env.DOCKER_CONTAINER === 'true';
    const needsSandboxDisable = isDocker || process.getuid?.() === 0;
    
    if (needsSandboxDisable) {
      browserArgs.push('--no-sandbox', '--disable-setuid-sandbox');
    }
    
    // Create user data directory in accessible location
    const os = require('os');
    const path = require('path');
    this.userDataDir = path.join(os.tmpdir(), 'auditmysite-browser-' + Date.now());
    
    try {
      // Use launchPersistentContext for user data directory support
      if (!this.userDataDir) {
        throw new Error('User data directory not initialized');
      }
      
      const context = await chromium.launchPersistentContext(this.userDataDir, {
        headless: this.config.headless !== false,
        slowMo: this.config.slowMo || 0,
        devtools: this.config.devtools || false,
        viewport: { width: 1920, height: 1080 },
        userAgent: 'auditmysite/1.5.0 (+https://github.com/casoon/AuditMySite)',
        args: browserArgs
      });
      
      // Extract browser from context
      this.browser = context.browser()!;
      this.context = context;
      
    } catch (error: any) {
      // Handle permission errors with fallback configuration
      if (error.message?.includes('permission') || error.message?.includes('Operation not permitted') || error.message?.includes('userDataDir')) {
        // Always show this fallback - indicates browser permission issues
        log.fallback('Browser Launch', 'launch failed due to permissions', 'using fallback without persistent context', error.message);
        
        const fallbackArgs = [
          '--headless=new',
          '--disable-gpu',
          '--no-sandbox',
          '--disable-setuid-sandbox',
          '--disable-dev-shm-usage',
          '--remote-debugging-port=' + this.port
        ];
        
        this.browser = await chromium.launch({
          headless: true,
          args: fallbackArgs
        });
        
        // Create context manually without user data dir
        this.context = await this.browser.newContext({
          viewport: { width: 1920, height: 1080 },
          userAgent: 'auditmysite/1.5.0 (+https://github.com/casoon/AuditMySite)'
        });
        
        // Always show successful fallback recovery
        log.success('Browser launched with fallback configuration (no persistent context)');
      } else {
        throw error;
      }
    }
    
    // Context was already created above, check if we need to create WebSocket endpoint

    // WebSocket Endpoint für pa11y/Lighthouse
    this.wsEndpoint = `ws://127.0.0.1:${this.port}`;
    
    if (this.config.verbose) {
      console.log(`✅ Shared browser ready on port ${this.port}`);
      console.log(`ℹ️  WebSocket endpoint for internal use: ${this.wsEndpoint}`);
    }
  }

  async getPage() {
    if (!this.context) {
      throw new Error('Browser not initialized');
    }
    return await this.context.newPage();
  }

  getWsEndpoint(): string {
    if (!this.wsEndpoint) {
      throw new Error('Browser not initialized');
    }
    return this.wsEndpoint;
  }

  getPort(): number {
    return this.port;
  }

  async cleanup(): Promise<void> {
    if (this.context) {
      await this.context.close();
    }
    if (this.browser) {
      await this.browser.close();
    }
    
    // Clean up user data directory (async)
    if (this.userDataDir) {
      try {
        const fs = require('fs/promises');
        const { existsSync } = require('fs'); // Keep sync check for existence
        if (existsSync(this.userDataDir)) {
          await fs.rm(this.userDataDir, { recursive: true, force: true });
        }
      } catch (error) {
        console.warn('⚠️  Could not clean up browser user data directory:', error);
      }
    }
    
    if (this.config.verbose) {
      console.log('🧹 Shared browser cleaned up');
    }
  }

  isInitialized(): boolean {
    return this.browser !== null && this.context !== null;
  }
} 