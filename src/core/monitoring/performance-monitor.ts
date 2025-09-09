/**
 * Performance Monitoring and Benchmarking System
 */

export interface PerformanceMetrics {
    timestamp: number;
    operation: string;
    duration: number;
    memoryUsage: {
        used: number;
        total: number;
        external: number;
        heapUsed: number;
        heapTotal: number;
    };
    cpuUsage?: {
        user: number;
        system: number;
    };
    metadata?: Record<string, any>;
}

export interface BenchmarkResult {
    operation: string;
    iterations: number;
    totalTime: number;
    averageTime: number;
    minTime: number;
    maxTime: number;
    standardDeviation: number;
    throughput: number; // operations per second
    memoryProfile: {
        initial: number;
        peak: number;
        final: number;
        leaked: number;
    };
}

export class PerformanceMonitor {
    private metrics: PerformanceMetrics[] = [];
    private activeOperations: Map<string, { startTime: number; startMemory: number }> = new Map();
    private benchmarks: Map<string, BenchmarkResult> = new Map();

    /**
     * Start monitoring an operation
     */
    startOperation(operationId: string, operation: string, metadata?: Record<string, any>): void {
        const memoryUsage = process.memoryUsage();
        
        this.activeOperations.set(operationId, {
            startTime: Date.now(),
            startMemory: memoryUsage.heapUsed
        });

        console.log(`⏱️  Started monitoring: ${operation} (ID: ${operationId})`);
    }

    /**
     * End monitoring an operation and record metrics
     */
    endOperation(operationId: string, operation: string, metadata?: Record<string, any>): PerformanceMetrics | null {
        const activeOp = this.activeOperations.get(operationId);
        if (!activeOp) {
            console.warn(`⚠️ No active operation found for ID: ${operationId}`);
            return null;
        }

        const endTime = Date.now();
        const duration = endTime - activeOp.startTime;
        const memoryUsage = process.memoryUsage();
        
        // Get CPU usage if available
        let cpuUsage;
        try {
            cpuUsage = process.cpuUsage();
        } catch (error) {
            // CPU usage not available on all platforms
        }

        const metric: PerformanceMetrics = {
            timestamp: endTime,
            operation,
            duration,
            memoryUsage: {
                used: memoryUsage.rss,
                total: memoryUsage.rss + memoryUsage.external,
                external: memoryUsage.external,
                heapUsed: memoryUsage.heapUsed,
                heapTotal: memoryUsage.heapTotal
            },
            cpuUsage: cpuUsage ? {
                user: cpuUsage.user / 1000, // Convert to milliseconds
                system: cpuUsage.system / 1000
            } : undefined,
            metadata
        };

        this.metrics.push(metric);
        this.activeOperations.delete(operationId);

        const memoryDelta = memoryUsage.heapUsed - activeOp.startMemory;
        console.log(`✅ Completed: ${operation} (${duration}ms, ${this.formatBytes(memoryDelta)} memory)`);

        return metric;
    }

    /**
     * Monitor a function execution
     */
    async monitor<T>(operation: string, fn: () => Promise<T> | T, metadata?: Record<string, any>): Promise<T> {
        const operationId = `${operation}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        
        this.startOperation(operationId, operation, metadata);
        
        try {
            const result = await fn();
            this.endOperation(operationId, operation, metadata);
            return result;
        } catch (error: any) {
            this.endOperation(operationId, operation, { ...metadata, error: error?.message || String(error) });
            throw error;
        }
    }

    /**
     * Run benchmark tests
     */
    async benchmark(
        name: string,
        fn: () => Promise<any> | any,
        options: { iterations?: number; warmup?: number } = {}
    ): Promise<BenchmarkResult> {
        const { iterations = 100, warmup = 10 } = options;
        
        console.log(`🏁 Starting benchmark: ${name} (${iterations} iterations, ${warmup} warmup)`);
        
        // Warmup runs
        for (let i = 0; i < warmup; i++) {
            await fn();
        }

        // Force garbage collection if available
        if (global.gc) {
            global.gc();
        }

        const times: number[] = [];
        const initialMemory = process.memoryUsage().heapUsed;
        let peakMemory = initialMemory;

        // Benchmark runs
        for (let i = 0; i < iterations; i++) {
            const startTime = process.hrtime.bigint();
            await fn();
            const endTime = process.hrtime.bigint();
            
            const duration = Number(endTime - startTime) / 1000000; // Convert to milliseconds
            times.push(duration);

            const currentMemory = process.memoryUsage().heapUsed;
            peakMemory = Math.max(peakMemory, currentMemory);
        }

        const finalMemory = process.memoryUsage().heapUsed;
        const totalTime = times.reduce((sum, time) => sum + time, 0);
        const averageTime = totalTime / iterations;
        const minTime = Math.min(...times);
        const maxTime = Math.max(...times);
        
        // Calculate standard deviation
        const variance = times.reduce((sum, time) => sum + Math.pow(time - averageTime, 2), 0) / iterations;
        const standardDeviation = Math.sqrt(variance);

        const result: BenchmarkResult = {
            operation: name,
            iterations,
            totalTime,
            averageTime,
            minTime,
            maxTime,
            standardDeviation,
            throughput: 1000 / averageTime, // ops per second
            memoryProfile: {
                initial: initialMemory,
                peak: peakMemory,
                final: finalMemory,
                leaked: finalMemory - initialMemory
            }
        };

        this.benchmarks.set(name, result);

        console.log(`🏆 Benchmark completed: ${name}`);
        console.log(`   Average: ${averageTime.toFixed(2)}ms`);
        console.log(`   Throughput: ${result.throughput.toFixed(2)} ops/sec`);
        console.log(`   Memory leaked: ${this.formatBytes(result.memoryProfile.leaked)}`);

        return result;
    }

    /**
     * Get performance statistics
     */
    getStats(operation?: string): {
        count: number;
        averageDuration: number;
        totalDuration: number;
        minDuration: number;
        maxDuration: number;
        averageMemoryUsage: number;
        peakMemoryUsage: number;
    } {
        const filteredMetrics = operation 
            ? this.metrics.filter(m => m.operation === operation)
            : this.metrics;

        if (filteredMetrics.length === 0) {
            return {
                count: 0,
                averageDuration: 0,
                totalDuration: 0,
                minDuration: 0,
                maxDuration: 0,
                averageMemoryUsage: 0,
                peakMemoryUsage: 0
            };
        }

        const durations = filteredMetrics.map(m => m.duration);
        const memoryUsages = filteredMetrics.map(m => m.memoryUsage.heapUsed);

        return {
            count: filteredMetrics.length,
            averageDuration: durations.reduce((sum, d) => sum + d, 0) / durations.length,
            totalDuration: durations.reduce((sum, d) => sum + d, 0),
            minDuration: Math.min(...durations),
            maxDuration: Math.max(...durations),
            averageMemoryUsage: memoryUsages.reduce((sum, m) => sum + m, 0) / memoryUsages.length,
            peakMemoryUsage: Math.max(...memoryUsages)
        };
    }

    /**
     * Get all metrics
     */
    getMetrics(limit?: number): PerformanceMetrics[] {
        const sorted = [...this.metrics].sort((a, b) => b.timestamp - a.timestamp);
        return limit ? sorted.slice(0, limit) : sorted;
    }

    /**
     * Get all benchmarks
     */
    getBenchmarks(): Map<string, BenchmarkResult> {
        return new Map(this.benchmarks);
    }

    /**
     * Generate performance report
     */
    generateReport(): string {
        const stats = this.getStats();
        const operations = [...new Set(this.metrics.map(m => m.operation))];
        
        let report = `\n📊 Performance Report\n`;
        report += `=====================================\n`;
        report += `Total Operations: ${stats.count}\n`;
        report += `Average Duration: ${stats.averageDuration.toFixed(2)}ms\n`;
        report += `Total Duration: ${stats.totalDuration.toFixed(2)}ms\n`;
        report += `Peak Memory Usage: ${this.formatBytes(stats.peakMemoryUsage)}\n\n`;

        if (operations.length > 0) {
            report += `Operation Breakdown:\n`;
            report += `--------------------\n`;
            
            for (const operation of operations) {
                const opStats = this.getStats(operation);
                report += `${operation}:\n`;
                report += `  Count: ${opStats.count}\n`;
                report += `  Average: ${opStats.averageDuration.toFixed(2)}ms\n`;
                report += `  Min/Max: ${opStats.minDuration.toFixed(2)}/${opStats.maxDuration.toFixed(2)}ms\n`;
                report += `  Memory: ${this.formatBytes(opStats.averageMemoryUsage)}\n\n`;
            }
        }

        if (this.benchmarks.size > 0) {
            report += `Benchmarks:\n`;
            report += `-----------\n`;
            
            for (const [name, benchmark] of this.benchmarks) {
                report += `${name}:\n`;
                report += `  Iterations: ${benchmark.iterations}\n`;
                report += `  Average: ${benchmark.averageTime.toFixed(2)}ms\n`;
                report += `  Throughput: ${benchmark.throughput.toFixed(2)} ops/sec\n`;
                report += `  Memory Leaked: ${this.formatBytes(benchmark.memoryProfile.leaked)}\n\n`;
            }
        }

        return report;
    }

    /**
     * Clear all metrics and benchmarks
     */
    clear(): void {
        this.metrics = [];
        this.benchmarks.clear();
        this.activeOperations.clear();
        console.log('🧹 Performance monitor cleared');
    }

    /**
     * Export metrics to JSON
     */
    export(): {
        metrics: PerformanceMetrics[];
        benchmarks: Record<string, BenchmarkResult>;
        stats: {
            count: number;
            averageDuration: number;
            totalDuration: number;
            minDuration: number;
            maxDuration: number;
            averageMemoryUsage: number;
            peakMemoryUsage: number;
        };
        timestamp: number;
    } {
        return {
            metrics: this.metrics,
            benchmarks: Object.fromEntries(this.benchmarks),
            stats: this.getStats(),
            timestamp: Date.now()
        };
    }

    /**
     * Format bytes to human readable format
     */
    private formatBytes(bytes: number): string {
        const sizes = ['B', 'KB', 'MB', 'GB'];
        if (bytes === 0) return '0 B';
        const i = Math.floor(Math.log(Math.abs(bytes)) / Math.log(1024));
        const size = bytes / Math.pow(1024, i);
        return `${size.toFixed(2)} ${sizes[i]}`;
    }

    /**
     * Memory leak detection
     */
    detectMemoryLeaks(threshold: number = 50 * 1024 * 1024): {
        hasLeak: boolean;
        leakSize: number;
        operations: string[];
    } {
        const suspiciousOperations: string[] = [];
        let totalLeak = 0;

        for (const [name, benchmark] of this.benchmarks) {
            if (benchmark.memoryProfile.leaked > threshold) {
                suspiciousOperations.push(name);
                totalLeak += benchmark.memoryProfile.leaked;
            }
        }

        return {
            hasLeak: suspiciousOperations.length > 0,
            leakSize: totalLeak,
            operations: suspiciousOperations
        };
    }
}

// Global performance monitor instance
export const performanceMonitor = new PerformanceMonitor();
