/**
 * Webhook Management System for API notifications
 */

import axios from 'axios';
import crypto from 'crypto';

export interface WebhookConfig {
    url: string;
    secret?: string;
    events: WebhookEvent[];
    headers?: Record<string, string>;
    retries?: number;
    timeout?: number;
    active: boolean;
}

export type WebhookEvent = 
    | 'audit.started'
    | 'audit.completed'
    | 'audit.failed'
    | 'report.generated'
    | 'analysis.progress';

export interface WebhookPayload {
    event: WebhookEvent;
    timestamp: number;
    data: any;
    signature?: string;
}

export interface WebhookDelivery {
    id: string;
    webhook_id: string;
    event: WebhookEvent;
    payload: WebhookPayload;
    status: 'pending' | 'delivered' | 'failed' | 'retrying';
    attempts: number;
    last_attempt: number;
    response_code?: number;
    response_body?: string;
    error?: string;
}

export class WebhookManager {
    private webhooks: Map<string, WebhookConfig> = new Map();
    private deliveries: Map<string, WebhookDelivery> = new Map();
    private retryQueue: Set<string> = new Set();
    private retryInterval: NodeJS.Timeout | null = null;

    constructor() {
        this.startRetryProcessor();
    }

    /**
     * Register a new webhook
     */
    register(id: string, config: WebhookConfig): void {
        this.webhooks.set(id, {
            ...config,
            retries: config.retries || 3,
            timeout: config.timeout || 30000
        });
        
        console.log(`🎣 Webhook registered: ${id} -> ${config.url}`);
    }

    /**
     * Unregister a webhook
     */
    unregister(id: string): boolean {
        const removed = this.webhooks.delete(id);
        if (removed) {
            console.log(`🗑️ Webhook unregistered: ${id}`);
        }
        return removed;
    }

    /**
     * Update webhook configuration
     */
    update(id: string, config: Partial<WebhookConfig>): boolean {
        const existing = this.webhooks.get(id);
        if (!existing) {
            return false;
        }

        this.webhooks.set(id, { ...existing, ...config });
        console.log(`🔄 Webhook updated: ${id}`);
        return true;
    }

    /**
     * Get webhook configuration
     */
    getWebhook(id: string): WebhookConfig | undefined {
        return this.webhooks.get(id);
    }

    /**
     * List all webhooks
     */
    listWebhooks(): Map<string, WebhookConfig> {
        return new Map(this.webhooks);
    }

    /**
     * Trigger webhook for a specific event
     */
    async trigger(event: WebhookEvent, data: any): Promise<WebhookDelivery[]> {
        const relevantWebhooks = Array.from(this.webhooks.entries())
            .filter(([_, config]) => config.active && config.events.includes(event));

        if (relevantWebhooks.length === 0) {
            console.log(`📭 No active webhooks for event: ${event}`);
            return [];
        }

        console.log(`📨 Triggering ${relevantWebhooks.length} webhooks for event: ${event}`);

        const deliveries: WebhookDelivery[] = [];

        for (const [webhookId, config] of relevantWebhooks) {
            const delivery = await this.deliver(webhookId, config, event, data);
            deliveries.push(delivery);
        }

        return deliveries;
    }

    /**
     * Deliver webhook to a specific endpoint
     */
    private async deliver(
        webhookId: string, 
        config: WebhookConfig, 
        event: WebhookEvent, 
        data: any
    ): Promise<WebhookDelivery> {
        const deliveryId = `delivery-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        
        const payload: WebhookPayload = {
            event,
            timestamp: Date.now(),
            data
        };

        // Add signature if secret is provided
        if (config.secret) {
            payload.signature = this.generateSignature(payload, config.secret);
        }

        const delivery: WebhookDelivery = {
            id: deliveryId,
            webhook_id: webhookId,
            event,
            payload,
            status: 'pending',
            attempts: 0,
            last_attempt: Date.now()
        };

        this.deliveries.set(deliveryId, delivery);

        try {
            await this.attemptDelivery(delivery, config);
        } catch (error: any) {
            delivery.status = 'failed';
            delivery.error = error?.message || String(error);
            
            // Add to retry queue if retries are configured
            if (config.retries && config.retries > 0) {
                delivery.status = 'retrying';
                this.retryQueue.add(deliveryId);
            }
        }

        this.deliveries.set(deliveryId, delivery);
        return delivery;
    }

    /**
     * Attempt to deliver a webhook
     */
    private async attemptDelivery(delivery: WebhookDelivery, config: WebhookConfig): Promise<void> {
        delivery.attempts++;
        delivery.last_attempt = Date.now();

        const headers: Record<string, string> = {
            'Content-Type': 'application/json',
            'User-Agent': 'AuditMySite-Webhook/1.0',
            'X-Webhook-Event': delivery.event,
            'X-Webhook-Delivery': delivery.id,
            ...config.headers
        };

        if (delivery.payload.signature) {
            headers['X-Hub-Signature-256'] = `sha256=${delivery.payload.signature}`;
        }

        const response = await axios.post(config.url, delivery.payload, {
            headers,
            timeout: config.timeout || 30000,
            validateStatus: (status: number) => status >= 200 && status < 300
        });

        delivery.response_code = response.status;
        delivery.response_body = typeof response.data === 'string' 
            ? response.data.substring(0, 1000) // Limit response body size
            : JSON.stringify(response.data).substring(0, 1000);

        delivery.status = 'delivered';
        
        console.log(`✅ Webhook delivered: ${delivery.id} -> ${config.url} (${response.status})`);
    }

    /**
     * Generate HMAC signature for webhook payload
     */
    private generateSignature(payload: WebhookPayload, secret: string): string {
        const payloadString = JSON.stringify(payload);
        return crypto.createHmac('sha256', secret).update(payloadString).digest('hex');
    }

    /**
     * Verify webhook signature
     */
    static verifySignature(payload: string, signature: string, secret: string): boolean {
        const expectedSignature = crypto.createHmac('sha256', secret).update(payload).digest('hex');
        const providedSignature = signature.replace('sha256=', '');
        
        return crypto.timingSafeEqual(
            Buffer.from(expectedSignature, 'hex'),
            Buffer.from(providedSignature, 'hex')
        );
    }

    /**
     * Start retry processor for failed deliveries
     */
    private startRetryProcessor(): void {
        this.retryInterval = setInterval(() => {
            this.processRetries();
        }, 60000); // Check every minute

        console.log('🔄 Webhook retry processor started');
    }

    /**
     * Process retry queue
     */
    private async processRetries(): Promise<void> {
        if (this.retryQueue.size === 0) {
            return;
        }

        console.log(`🔄 Processing ${this.retryQueue.size} webhook retries...`);

        const retries = Array.from(this.retryQueue);
        this.retryQueue.clear();

        for (const deliveryId of retries) {
            const delivery = this.deliveries.get(deliveryId);
            if (!delivery) {
                continue;
            }

            const webhook = this.webhooks.get(delivery.webhook_id);
            if (!webhook || !webhook.active) {
                delivery.status = 'failed';
                delivery.error = 'Webhook no longer active';
                continue;
            }

            const maxRetries = webhook.retries || 3;
            if (delivery.attempts >= maxRetries) {
                delivery.status = 'failed';
                delivery.error = `Max retries (${maxRetries}) exceeded`;
                console.log(`❌ Webhook delivery failed after ${maxRetries} attempts: ${deliveryId}`);
                continue;
            }

            // Exponential backoff: wait longer between retries
            const backoffTime = Math.pow(2, delivery.attempts - 1) * 60000; // 1min, 2min, 4min, etc.
            const timeSinceLastAttempt = Date.now() - delivery.last_attempt;
            
            if (timeSinceLastAttempt < backoffTime) {
                this.retryQueue.add(deliveryId); // Re-queue for later
                continue;
            }

            try {
                await this.attemptDelivery(delivery, webhook);
            } catch (error: any) {
                delivery.error = error?.message || String(error);
                this.retryQueue.add(deliveryId); // Re-queue for next retry
            }

            this.deliveries.set(deliveryId, delivery);
        }
    }

    /**
     * Get delivery information
     */
    getDelivery(deliveryId: string): WebhookDelivery | undefined {
        return this.deliveries.get(deliveryId);
    }

    /**
     * Get deliveries for a webhook
     */
    getWebhookDeliveries(webhookId: string, limit: number = 50): WebhookDelivery[] {
        return Array.from(this.deliveries.values())
            .filter(delivery => delivery.webhook_id === webhookId)
            .sort((a, b) => b.last_attempt - a.last_attempt)
            .slice(0, limit);
    }

    /**
     * Get delivery statistics
     */
    getDeliveryStats(webhookId?: string): {
        total: number;
        delivered: number;
        failed: number;
        pending: number;
        retrying: number;
        successRate: number;
    } {
        const deliveries = webhookId 
            ? Array.from(this.deliveries.values()).filter(d => d.webhook_id === webhookId)
            : Array.from(this.deliveries.values());

        const stats = {
            total: deliveries.length,
            delivered: deliveries.filter(d => d.status === 'delivered').length,
            failed: deliveries.filter(d => d.status === 'failed').length,
            pending: deliveries.filter(d => d.status === 'pending').length,
            retrying: deliveries.filter(d => d.status === 'retrying').length,
            successRate: 0
        };

        if (stats.total > 0) {
            stats.successRate = (stats.delivered / stats.total) * 100;
        }

        return stats;
    }

    /**
     * Clean up old deliveries
     */
    cleanup(olderThanMs: number = 7 * 24 * 60 * 60 * 1000): number { // Default: 7 days
        const cutoff = Date.now() - olderThanMs;
        let cleaned = 0;

        for (const [id, delivery] of this.deliveries.entries()) {
            if (delivery.last_attempt < cutoff) {
                this.deliveries.delete(id);
                this.retryQueue.delete(id);
                cleaned++;
            }
        }

        console.log(`🧹 Cleaned up ${cleaned} old webhook deliveries`);
        return cleaned;
    }

    /**
     * Shutdown webhook manager
     */
    shutdown(): void {
        if (this.retryInterval) {
            clearInterval(this.retryInterval);
            this.retryInterval = null;
        }

        console.log('🛑 Webhook manager shutdown');
    }

    /**
     * Export webhook configuration and deliveries
     */
    export(): {
        webhooks: Record<string, WebhookConfig>;
        deliveries: WebhookDelivery[];
        stats: {
            total: number;
            delivered: number;
            failed: number;
            pending: number;
            retrying: number;
            successRate: number;
        };
    } {
        return {
            webhooks: Object.fromEntries(this.webhooks),
            deliveries: Array.from(this.deliveries.values()),
            stats: this.getDeliveryStats()
        };
    }
}

// Global webhook manager instance
export const webhookManager = new WebhookManager();
