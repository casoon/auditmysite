/**
 * Browser Pool Manager for optimized resource usage and performance
 */

import { Browser, BrowserContext, chromium, firefox, webkit } from 'playwright';

export interface BrowserPoolOptions {
    maxConcurrent: number;
    maxIdleTime: number; // milliseconds
    browserType: 'chromium' | 'firefox' | 'webkit';
    launchOptions?: any;
    enableResourceOptimization: boolean;
}

export interface PooledBrowser {
    browser: Browser;
    context: BrowserContext;
    lastUsed: number;
    inUse: boolean;
    id: string;
}

export class BrowserPoolManager {
    private pool: Map<string, PooledBrowser> = new Map();
    private queue: string[] = [];
    private options: BrowserPoolOptions;
    private cleanupInterval: NodeJS.Timeout | null = null;
    private metrics = {
        created: 0,
        reused: 0,
        destroyed: 0,
        errors: 0,
        activeConnections: 0,
        totalRequests: 0
    };

    constructor(options: Partial<BrowserPoolOptions> = {}) {
        this.options = {
            maxConcurrent: options.maxConcurrent || 3,
            maxIdleTime: options.maxIdleTime || 30000, // 30 seconds
            browserType: options.browserType || 'chromium',
            launchOptions: options.launchOptions || {
                headless: true,
                args: [
                    '--no-sandbox',
                    '--disable-setuid-sandbox',
                    '--disable-dev-shm-usage',
                    '--disable-gpu',
                    '--disable-web-security',
                    '--disable-features=VizDisplayCompositor',
                    '--memory-pressure-off',
                    '--max_old_space_size=4096'
                ]
            },
            enableResourceOptimization: options.enableResourceOptimization !== false
        };

        // Start cleanup interval
        this.startCleanup();
    }

    /**
     * Get a browser instance from the pool
     */
    async acquire(): Promise<{ browser: Browser; context: BrowserContext; release: () => Promise<void> }> {
        this.metrics.totalRequests++;

        // Try to reuse an existing browser
        let pooledBrowser = this.findAvailableBrowser();

        if (!pooledBrowser && this.pool.size < this.options.maxConcurrent) {
            // Create new browser if under limit
            pooledBrowser = await this.createBrowser();
        } else if (!pooledBrowser) {
            // Wait for available browser
            pooledBrowser = await this.waitForAvailableBrowser();
        }

        if (!pooledBrowser) {
            throw new Error('Unable to acquire browser from pool');
        }

        // Mark as in use
        pooledBrowser.inUse = true;
        pooledBrowser.lastUsed = Date.now();
        this.metrics.activeConnections++;

        if (this.metrics.reused > 0 || this.pool.size > 1) {
            this.metrics.reused++;
        }

        const release = async () => {
            pooledBrowser!.inUse = false;
            pooledBrowser!.lastUsed = Date.now();
            this.metrics.activeConnections--;
            
            // Add back to queue
            this.queue.push(pooledBrowser!.id);
        };

        return {
            browser: pooledBrowser.browser,
            context: pooledBrowser.context,
            release
        };
    }

    /**
     * Find an available browser in the pool
     */
    private findAvailableBrowser(): PooledBrowser | null {
        for (const [id, pooledBrowser] of this.pool.entries()) {
            if (!pooledBrowser.inUse && pooledBrowser.browser.isConnected()) {
                return pooledBrowser;
            }
        }
        return null;
    }

    /**
     * Wait for an available browser
     */
    private async waitForAvailableBrowser(): Promise<PooledBrowser | null> {
        return new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                reject(new Error('Timeout waiting for available browser'));
            }, 10000); // 10 second timeout

            const check = () => {
                const available = this.findAvailableBrowser();
                if (available) {
                    clearTimeout(timeout);
                    resolve(available);
                } else {
                    setTimeout(check, 100);
                }
            };

            check();
        });
    }

    /**
     * Create a new browser instance
     */
    private async createBrowser(): Promise<PooledBrowser> {
        const id = `browser-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        
        try {
            let browser: Browser;
            
            switch (this.options.browserType) {
                case 'firefox':
                    browser = await firefox.launch(this.options.launchOptions);
                    break;
                case 'webkit':
                    browser = await webkit.launch(this.options.launchOptions);
                    break;
                default:
                    browser = await chromium.launch(this.options.launchOptions);
            }

            // Create optimized browser context
            const context = await browser.newContext({
                ignoreHTTPSErrors: true,
                bypassCSP: true,
                ...(this.options.enableResourceOptimization && {
                    // Block unnecessary resources for performance  
                })
            });

            // Optimize context for performance
            if (this.options.enableResourceOptimization) {
                await context.route('**/*', (route) => {
                    const resourceType = route.request().resourceType();
                    
                    // Block non-essential resources
                    if (['image', 'font', 'media'].includes(resourceType)) {
                        const url = route.request().url();
                        if (url.includes('tracking') || url.includes('analytics') || url.includes('ads')) {
                            route.abort();
                            return;
                        }
                    }
                    
                    route.continue();
                });
            }

            const pooledBrowser: PooledBrowser = {
                id,
                browser,
                context,
                lastUsed: Date.now(),
                inUse: false
            };

            this.pool.set(id, pooledBrowser);
            this.queue.push(id);
            this.metrics.created++;

            console.log(`🌐 Created browser ${id} (Pool size: ${this.pool.size})`);
            return pooledBrowser;

        } catch (error) {
            this.metrics.errors++;
            console.error(`❌ Failed to create browser:`, error);
            throw error;
        }
    }

    /**
     * Start cleanup interval to remove idle browsers
     */
    private startCleanup(): void {
        this.cleanupInterval = setInterval(() => {
            this.cleanup();
        }, this.options.maxIdleTime / 2);
    }

    /**
     * Clean up idle browsers
     */
    private async cleanup(): Promise<void> {
        const now = Date.now();
        const toRemove: string[] = [];

        for (const [id, pooledBrowser] of this.pool.entries()) {
            const isIdle = !pooledBrowser.inUse && 
                          (now - pooledBrowser.lastUsed) > this.options.maxIdleTime;
            const isDisconnected = !pooledBrowser.browser.isConnected();

            if (isIdle || isDisconnected) {
                toRemove.push(id);
            }
        }

        for (const id of toRemove) {
            await this.destroyBrowser(id);
        }

        if (toRemove.length > 0) {
            console.log(`🧹 Cleaned up ${toRemove.length} idle browsers`);
        }
    }

    /**
     * Destroy a specific browser
     */
    private async destroyBrowser(id: string): Promise<void> {
        const pooledBrowser = this.pool.get(id);
        if (!pooledBrowser) return;

        try {
            await pooledBrowser.context.close();
            await pooledBrowser.browser.close();
            this.metrics.destroyed++;
        } catch (error) {
            console.error(`Error closing browser ${id}:`, error);
        }

        this.pool.delete(id);
        this.queue = this.queue.filter(queueId => queueId !== id);
        
        console.log(`🗑️ Destroyed browser ${id} (Pool size: ${this.pool.size})`);
    }

    /**
     * Get pool metrics
     */
    getMetrics() {
        return {
            ...this.metrics,
            poolSize: this.pool.size,
            queueLength: this.queue.length,
            efficiency: this.metrics.totalRequests > 0 ? 
                        (this.metrics.reused / this.metrics.totalRequests) * 100 : 0
        };
    }

    /**
     * Warm up the pool by creating initial browsers
     */
    async warmUp(count: number = 1): Promise<void> {
        console.log(`🔥 Warming up browser pool with ${count} browsers...`);
        
        const warmupPromises: Promise<PooledBrowser>[] = [];
        for (let i = 0; i < Math.min(count, this.options.maxConcurrent); i++) {
            warmupPromises.push(this.createBrowser());
        }

        await Promise.all(warmupPromises);
        console.log(`✅ Browser pool warmed up with ${this.pool.size} browsers`);
    }

    /**
     * Gracefully shutdown all browsers
     */
    async shutdown(): Promise<void> {
        console.log('🔄 Shutting down browser pool...');
        
        if (this.cleanupInterval) {
            clearInterval(this.cleanupInterval);
        }

        const shutdownPromises: Promise<void>[] = [];
        for (const id of this.pool.keys()) {
            shutdownPromises.push(this.destroyBrowser(id));
        }

        await Promise.all(shutdownPromises);
        console.log('✅ Browser pool shutdown complete');
    }

    /**
     * Get pool status
     */
    getStatus() {
        return {
            totalBrowsers: this.pool.size,
            activeBrowsers: Array.from(this.pool.values()).filter(b => b.inUse).length,
            idleBrowsers: Array.from(this.pool.values()).filter(b => !b.inUse).length,
            queueLength: this.queue.length,
            metrics: this.getMetrics()
        };
    }
}

export default BrowserPoolManager;
