/**
 * 🚀 AuditMySite REST API Server
 * 
 * Express.js-based REST API for remote accessibility auditing.
 * Clean implementation for v1.8.0 with full TypeScript compatibility.
 */

import express, { Express, Request, Response, NextFunction, RequestHandler, ErrorRequestHandler } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import swaggerUi from 'swagger-ui-express';
import { v4 as uuidv4 } from 'uuid';
import { AuditSDK } from '../sdk/audit-sdk';
import {
  APIResponse,
  AuditJob,
  AuditJobRequest,
  AuditOptions,
  AuditResult,
  SDKConfig,
  ProgressData
} from '../sdk/types';

// =============================================================================
// Type Definitions
// =============================================================================

// Extend Express Request globally
declare global {
  namespace Express {
    interface Request {
      requestId: string;
      apiKey?: string;
    }
  }
}

type AuthenticatedRequest = Request;

interface APIConfig {
  port: number;
  host: string;
  apiKeyRequired: boolean;
  validApiKeys: string[];
  maxConcurrentJobs: number;
  jobTimeout: number;
  enableSwagger: boolean;
  corsOrigins: string[];
}

interface JobManager {
  jobs: Map<string, AuditJob>;
  runningJobs: Set<string>;
  maxConcurrent: number;
}

// =============================================================================
// API Server Class
// =============================================================================

export class AuditAPIServer {
  private app: Express;
  private config: APIConfig;
  private jobManager: JobManager;
  private sdk: AuditSDK;
  private server: any = null;

  constructor(config: Partial<APIConfig> = {}) {
    this.config = this.mergeConfig(config);
    this.jobManager = {
      jobs: new Map(),
      runningJobs: new Set(),
      maxConcurrent: this.config.maxConcurrentJobs
    };
    this.sdk = new AuditSDK();
    this.app = express();
    
    this.setupMiddleware();
    this.setupRoutes();
    this.setupErrorHandling();
  }

  /**
   * Start the API server
   */
  async start(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.server = this.app.listen(this.config.port, this.config.host, () => {
          console.log(`🚀 AuditMySite API Server running at http://${this.config.host}:${this.config.port}`);
          resolve();
        });

        this.server.on('error', reject);
      } catch (error) {
        reject(error);
      }
    });
  }

  /**
   * Get Express app instance (for testing)
   */
  getApp(): Express {
    return this.app;
  }
  
  /**
   * Shutdown server and clean up resources
   */
  async shutdown(): Promise<void> {
    if (this.server) {
      return new Promise((resolve) => {
        this.server.close(() => {
          this.server = null;
          resolve();
        });
      });
    }
  }

  private mergeConfig(config: Partial<APIConfig>): APIConfig {
    return {
      port: 3000,
      host: '0.0.0.0',
      apiKeyRequired: process.env.NODE_ENV === 'production',
      validApiKeys: process.env.API_KEYS?.split(',') || [],
      maxConcurrentJobs: 5,
      jobTimeout: 300000, // 5 minutes
      enableSwagger: true,
      corsOrigins: ['*'],
      ...config
    };
  }

  private setupMiddleware(): void {
    // Security middleware
    this.app.use(helmet());
    
    // CORS
    this.app.use(cors({
      origin: this.config.corsOrigins,
      credentials: true
    }));

    // Body parsing
    this.app.use(express.json({ limit: '10mb' }));
    this.app.use(express.urlencoded({ extended: true }));

    // Rate limiting
    const limiter = rateLimit({
      windowMs: 15 * 60 * 1000, // 15 minutes
      max: 100, // limit each IP to 100 requests per windowMs
      message: this.createErrorResponse('RATE_LIMIT_EXCEEDED', 'Too many requests'),
      standardHeaders: true,
      legacyHeaders: false
    });
    this.app.use(limiter);

    // Request ID middleware
    const requestIdHandler: RequestHandler = (req: Request, res: Response, next: NextFunction) => {
      req.requestId = uuidv4();
      res.setHeader('X-Request-ID', req.requestId);
      next();
    };
    this.app.use(requestIdHandler);
  }

  private setupRoutes(): void {
    // Health check (no auth required)
    this.app.get('/health', (req: Request, res: Response) => {
      res.json({
        success: true,
        data: {
          status: 'healthy',
          timestamp: new Date().toISOString(),
          version: '2.0.0-alpha.1', // Updated for v2.0
          uptime: process.uptime(),
          jobs: {
            total: this.jobManager.jobs.size,
            running: this.jobManager.runningJobs.size
          }
        }
      });
    });
    
    // v2.0 API Routes (no auth required for now)
    const { createV2Router } = require('./routes/v2.routes');
    this.app.use('/api/v2', createV2Router());
    
    // Swagger UI for v2.0 API documentation
    if (this.config.enableSwagger) {
      const swaggerSpec = {
        openapi: '3.0.0',
        info: {
          title: 'AuditMySite API v2.0',
          version: '2.0.0-alpha.1',
          description: 'Modular API for website analysis using shared TypeScript types. Designed for Electron app integration.'
        },
        servers: [
          { url: '/api/v2', description: 'v2.0 API (modular)' },
          { url: '/api/v1', description: 'v1.0 API (full site analysis)' }
        ],
        paths: {
          '/sitemap/{domain}': {
            get: {
              summary: 'Get sitemap URLs for domain',
              parameters: [{ name: 'domain', in: 'path', required: true, schema: { type: 'string' } }],
              responses: { 
                '200': { description: 'SitemapResult with URLs and metadata' },
                '500': { description: 'Sitemap parsing failed' }
              }
            }
          },
          '/page/accessibility': {
            post: {
              summary: 'Analyze accessibility for single URL',
              requestBody: {
                required: true,
                content: {
                  'application/json': {
                    schema: {
                      type: 'object',
                      properties: {
                        url: { type: 'string', description: 'URL to analyze' },
                        options: {
                          type: 'object',
                          properties: {
                            pa11yStandard: { type: 'string', enum: ['WCAG2A', 'WCAG2AA', 'WCAG2AAA'] },
                            includeWarnings: { type: 'boolean' }
                          }
                        }
                      },
                      required: ['url']
                    }
                  }
                }
              },
              responses: { 
                '200': { description: 'AccessibilityResult with score and issues' },
                '400': { description: 'Invalid request' },
                '500': { description: 'Analysis failed' }
              }
            }
          },
          '/schema': {
            get: {
              summary: 'API introspection for Electron app discovery',
              responses: {
                '200': { description: 'Available endpoints and type definitions' }
              }
            }
          }
        }
      };
      
      this.app.use('/api-docs', swaggerUi.serve, swaggerUi.setup(swaggerSpec));
      console.log('📚 API documentation available at /api-docs');
    }
    
    // Apply API Key authentication to API endpoints only
    if (this.config.apiKeyRequired) {
      this.app.use('/api/v1/*', this.authenticateApiKey.bind(this));
    }
    
    // API info
    this.app.get('/api/v1/info', (req: Request, res: Response) => {
      res.json({
        success: true,
        data: {
          name: 'AuditMySite API',
          version: '1.8.8',
          description: 'REST API for comprehensive website analysis with enhanced accessibility, performance, SEO, and content weight testing',
          features: [
            'Enhanced Accessibility Analysis (ARIA, Focus, Color Contrast)',
            'Core Web Vitals Performance Metrics',
            'Advanced SEO Analysis',
            'Content Weight Assessment',
            'Multiple Report Formats (HTML, JSON, CSV)'
          ],
          endpoints: [
            '/health', 
            '/api/v1/audit', 
            '/api/v1/audit/quick',
            '/api/v1/audit/performance',
            '/api/v1/audit/seo',
            '/api/v1/audit/content-weight'
          ],
          options: {
            'accessibility': 'Enable enhanced accessibility analysis (default: true)',
            'performance': 'Enable Core Web Vitals collection (default: true)',
            'seo': 'Enable SEO analysis (default: true)',
            'contentWeight': 'Enable content weight assessment (default: true)',
            'reduced': 'Use reduced analysis mode (default: false)',
            'includeRecommendations': 'Include actionable recommendations (default: true)',
            'outputFormat': 'Output format: json, html, csv (default: json)'
          },
          maxConcurrentJobs: this.config.maxConcurrentJobs
        }
      });
    });
    
    // Audit endpoints
    this.app.post('/api/v1/audit', this.handleCreateAudit.bind(this));
    this.app.get('/api/v1/audit/:jobId', this.handleGetAudit.bind(this));
    this.app.delete('/api/v1/audit/:jobId', this.handleCancelAudit.bind(this));
    this.app.get('/api/v1/audits', this.handleListAudits.bind(this));
    
    // Quick audit endpoint
    this.app.post('/api/v1/audit/quick', this.handleQuickAudit.bind(this));
    
    // Specialized analysis endpoints
    this.app.post('/api/v1/audit/performance', this.handlePerformanceAudit.bind(this));
    this.app.post('/api/v1/audit/seo', this.handleSeoAudit.bind(this));
    this.app.post('/api/v1/audit/content-weight', this.handleContentWeightAudit.bind(this));
    this.app.post('/api/v1/audit/accessibility', this.handleAccessibilityAudit.bind(this));
    
    // Test connection endpoint
    this.app.post('/api/v1/test-connection', this.handleTestConnection.bind(this));
    
    // Reports endpoints
    this.app.get('/api/v1/audit/:jobId/reports', this.handleGetReports.bind(this));
    this.app.get('/api/v1/audit/:jobId/reports/:format', this.handleDownloadReport.bind(this));

    // 404 handler
    this.app.use('*', (req: Request, res: Response) => {
      res.status(404).json(this.createErrorResponse('NOT_FOUND', 'Endpoint not found'));
    });
  }

  private setupErrorHandling(): void {
    this.app.use((error: Error, req: Request, res: Response, next: NextFunction) => {
      console.error(`API Error [${req.requestId}]:`, error);
      
      const statusCode = (error as any).statusCode || 500;
      const response = this.createErrorResponse(
        (error as any).code || 'INTERNAL_ERROR',
        error.message || 'Internal server error'
      );
      
      res.status(statusCode).json(response);
    });
  }

  private async authenticateApiKey(req: Request, res: Response, next: NextFunction): Promise<void> {
    const apiKey = req.headers['x-api-key'] as string || req.query.apiKey as string;
    
    if (!apiKey) {
      res.status(401).json(this.createErrorResponse('AUTH_REQUIRED', 'API key required'));
      return;
    }
    
    if (!this.config.validApiKeys.includes(apiKey)) {
      res.status(401).json(this.createErrorResponse('INVALID_API_KEY', 'Invalid API key'));
      return;
    }
    
    req.apiKey = apiKey;
    next();
  }

  private async handleCreateAudit(req: Request, res: Response): Promise<void> {
    try {
      const jobRequest: AuditJobRequest = req.body;
      const jobId = uuidv4();
      
      // Validate input
      if (!jobRequest.sitemapUrl) {
        res.status(400).json(
          this.createErrorResponse('INVALID_INPUT', 'sitemapUrl is required')
        );
        return;
      }
      
      // Check concurrent job limit
      if (this.jobManager.runningJobs.size >= this.config.maxConcurrentJobs) {
        res.status(429).json(
          this.createErrorResponse('TOO_MANY_JOBS', 'Maximum concurrent jobs reached')
        );
        return;
      }
      
      // Create job with enhanced defaults
      const enhancedOptions: AuditOptions = {
        accessibility: true,
        performance: true,
        seo: true,
        contentWeight: true,
        includeRecommendations: true,
        reduced: false,
        ...(jobRequest.options || {})
      };
      
      const job: AuditJob = {
        id: jobId,
        status: 'pending',
        sitemapUrl: jobRequest.sitemapUrl,
        options: enhancedOptions,
        createdAt: new Date(),
        progress: { current: 0, total: 100, percentage: 0 }
      };
      
      this.jobManager.jobs.set(jobId, job);
      
      // Start audit in background
      this.startAuditJob(jobId).catch(error => {
        console.error(`Job ${jobId} failed:`, error);
        const failedJob = this.jobManager.jobs.get(jobId);
        if (failedJob) {
          failedJob.status = 'failed';
          failedJob.error = error.message;
          failedJob.completedAt = new Date();
        }
      });
      
      res.status(201).json({
        success: true,
        data: { jobId, status: 'pending' }
      });
      
    } catch (error) {
      res.status(500).json(this.createErrorResponse('INTERNAL_ERROR', 'Failed to create audit'));
    }
  }

  private async handleGetAudit(req: Request, res: Response): Promise<void> {
    const { jobId } = req.params;
    const job = this.jobManager.jobs.get(jobId);
    
    if (!job) {
      res.status(404).json(
        this.createErrorResponse('JOB_NOT_FOUND', 'Audit job not found')
      );
      return;
    }
    
    res.json({
      success: true,
      data: job
    });
  }

  private async handleCancelAudit(req: Request, res: Response): Promise<void> {
    const { jobId } = req.params;
    const job = this.jobManager.jobs.get(jobId);
    
    if (!job) {
      res.status(404).json(
        this.createErrorResponse('JOB_NOT_FOUND', 'Audit job not found')
      );
      return;
    }
    
    job.status = 'cancelled';
    job.completedAt = new Date();
    this.jobManager.runningJobs.delete(jobId);
    
    res.json({
      success: true,
      data: { jobId, status: 'cancelled' }
    });
  }

  private async handleListAudits(req: Request, res: Response): Promise<void> {
    const { status, limit = '10', offset = '0' } = req.query;
    
    let jobs = Array.from(this.jobManager.jobs.values());
    
    if (status) {
      jobs = jobs.filter(job => job.status === status);
    }
    
    const startIndex = parseInt(offset as string);
    const endIndex = startIndex + parseInt(limit as string);
    const paginatedJobs = jobs.slice(startIndex, endIndex);
    
    res.json({
      success: true,
      data: {
        jobs: paginatedJobs,
        total: jobs.length,
        offset: startIndex,
        limit: parseInt(limit as string)
      }
    });
  }

  private async handleQuickAudit(req: Request, res: Response): Promise<void> {
    try {
      const { sitemapUrl, options } = req.body;
      
      if (!sitemapUrl) {
        res.status(400).json(
          this.createErrorResponse('INVALID_INPUT', 'sitemapUrl is required')
        );
        return;
      }
      
      // Merge options with new defaults: enhanced features are on by default
      const mergedOptions: AuditOptions = {
        accessibility: true,
        performance: true,
        seo: true,
        contentWeight: true,
        includeRecommendations: true,
        reduced: false,
        ...(options || {})
      };
      
      const result = await this.sdk.quickAudit(sitemapUrl, mergedOptions);
      
      res.json({
        success: true,
        data: result
      });
      
    } catch (error) {
      res.status(500).json(this.createErrorResponse('AUDIT_ERROR', 'Audit failed'));
    }
  }

  private async handlePerformanceAudit(req: Request, res: Response): Promise<void> {
    try {
      const { sitemapUrl, options } = req.body;
      
      if (!sitemapUrl) {
        res.status(400).json(
          this.createErrorResponse('INVALID_INPUT', 'sitemapUrl is required')
        );
        return;
      }
      
      const performanceOptions: AuditOptions = {
        accessibility: false,
        performance: true,
        seo: false,
        contentWeight: false,
        includeRecommendations: true,
        reduced: false,
        ...(options || {})
      };
      
      const result = await this.sdk.quickAudit(sitemapUrl, performanceOptions);
      
      res.json({
        success: true,
        data: {
          ...result,
          analysisType: 'performance',
          focus: 'Core Web Vitals and performance metrics'
        }
      });
      
    } catch (error) {
      res.status(500).json(this.createErrorResponse('PERFORMANCE_AUDIT_ERROR', 'Performance audit failed'));
    }
  }
  
  private async handleSeoAudit(req: Request, res: Response): Promise<void> {
    try {
      const { sitemapUrl, options } = req.body;
      
      if (!sitemapUrl) {
        res.status(400).json(
          this.createErrorResponse('INVALID_INPUT', 'sitemapUrl is required')
        );
        return;
      }
      
      const seoOptions: AuditOptions = {
        accessibility: false,
        performance: false,
        seo: true,
        contentWeight: false,
        includeRecommendations: true,
        reduced: false,
        ...(options || {})
      };
      
      const result = await this.sdk.quickAudit(sitemapUrl, seoOptions);
      
      res.json({
        success: true,
        data: {
          ...result,
          analysisType: 'seo',
          focus: 'Search engine optimization analysis'
        }
      });
      
    } catch (error) {
      res.status(500).json(this.createErrorResponse('SEO_AUDIT_ERROR', 'SEO audit failed'));
    }
  }
  
  private async handleContentWeightAudit(req: Request, res: Response): Promise<void> {
    try {
      const { sitemapUrl, options } = req.body;
      
      if (!sitemapUrl) {
        res.status(400).json(
          this.createErrorResponse('INVALID_INPUT', 'sitemapUrl is required')
        );
        return;
      }
      
      const contentOptions: AuditOptions = {
        accessibility: false,
        performance: false,
        seo: false,
        contentWeight: true,
        includeRecommendations: true,
        reduced: false,
        ...(options || {})
      };
      
      const result = await this.sdk.quickAudit(sitemapUrl, contentOptions);
      
      res.json({
        success: true,
        data: {
          ...result,
          analysisType: 'content-weight',
          focus: 'Content weight and optimization analysis'
        }
      });
      
    } catch (error) {
      res.status(500).json(this.createErrorResponse('CONTENT_WEIGHT_AUDIT_ERROR', 'Content weight audit failed'));
    }
  }
  
  private async handleAccessibilityAudit(req: Request, res: Response): Promise<void> {
    try {
      const { sitemapUrl, options } = req.body;
      
      if (!sitemapUrl) {
        res.status(400).json(
          this.createErrorResponse('INVALID_INPUT', 'sitemapUrl is required')
        );
        return;
      }
      
      const accessibilityOptions: AuditOptions = {
        accessibility: true,
        performance: false,
        seo: false,
        contentWeight: false,
        includeRecommendations: true,
        reduced: false,
        ...(options || {})
      };
      
      const result = await this.sdk.quickAudit(sitemapUrl, accessibilityOptions);
      
      res.json({
        success: true,
        data: {
          ...result,
          analysisType: 'accessibility',
          focus: 'Enhanced accessibility and WCAG compliance analysis'
        }
      });
      
    } catch (error) {
      res.status(500).json(this.createErrorResponse('ACCESSIBILITY_AUDIT_ERROR', 'Accessibility audit failed'));
    }
  }
  
  private async handleTestConnection(req: Request, res: Response): Promise<void> {
    try {
      const { sitemapUrl } = req.body;
      
      if (!sitemapUrl) {
        res.status(400).json(
          this.createErrorResponse('INVALID_INPUT', 'sitemapUrl is required')
        );
        return;
      }
      
      const result = await this.sdk.testConnection(sitemapUrl);
      
      res.json({
        success: true,
        data: result
      });
      
    } catch (error) {
      res.status(500).json(this.createErrorResponse('CONNECTION_ERROR', 'Connection test failed'));
    }
  }

  private async handleGetReports(req: Request, res: Response): Promise<void> {
    const { jobId } = req.params;
    const job = this.jobManager.jobs.get(jobId);
    
    if (!job) {
      res.status(404).json(
        this.createErrorResponse('JOB_NOT_FOUND', 'Audit job not found')
      );
      return;
    }
    
    if (job.status !== 'completed' || !job.result) {
      res.status(400).json(
        this.createErrorResponse('JOB_NOT_COMPLETE', 'Job not completed yet')
      );
      return;
    }
    
    res.json({
      success: true,
      data: {
        reports: job.result.reports || []
      }
    });
  }

  private async handleDownloadReport(req: Request, res: Response): Promise<void> {
    const { jobId, format } = req.params;
    const job = this.jobManager.jobs.get(jobId);
    
    if (!job) {
      res.status(404).json(
        this.createErrorResponse('JOB_NOT_FOUND', 'Audit job not found')
      );
      return;
    }
    
    if (job.status !== 'completed' || !job.result) {
      res.status(404).json(
        this.createErrorResponse('REPORT_NOT_FOUND', 'Report not available')
      );
      return;
    }
    
    const report = job.result.reports?.find(r => r.format === format);
    if (!report) {
      res.status(404).json(
        this.createErrorResponse('REPORT_NOT_FOUND', `Report in ${format} format not found`)
      );
      return;
    }
    
    res.json({
      success: true,
      data: report
    });
  }

  private async startAuditJob(jobId: string): Promise<void> {
    const job = this.jobManager.jobs.get(jobId);
    if (!job) return;
    
    job.status = 'running';
    job.startedAt = new Date();
    this.jobManager.runningJobs.add(jobId);
    
    try {
      const result = await this.sdk.quickAudit(job.sitemapUrl, job.options);
      
      job.status = 'completed';
      job.result = result;
      job.completedAt = new Date();
      job.progress = { current: 100, total: 100, percentage: 100 };
      
    } catch (error) {
      job.status = 'failed';
      job.error = (error as Error).message;
      job.completedAt = new Date();
    } finally {
      this.jobManager.runningJobs.delete(jobId);
    }
  }

  private createErrorResponse(code: string, message: string, details?: any): APIResponse<null> {
    return {
      success: false,
      error: {
        code,
        message,
        details
      },
      data: null
    };
  }
}
