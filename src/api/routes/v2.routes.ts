import { Router, Request, Response } from 'express';
import { SitemapService } from '../services/sitemap.service';
import { AccessibilityService } from '../services/accessibility.service';
import { PerformanceService } from '../services/performance.service';
import { SEOService } from '../services/seo.service';
import { version as toolVersion } from '../../../package.json';

// Services - detect test mode
const isTestMode = process.env.NODE_ENV === 'test';
const sitemapService = new SitemapService();
const accessibilityService = new AccessibilityService();
const performanceService = new PerformanceService(isTestMode);
const seoService = new SEOService(isTestMode);

// Cleanup handler for graceful shutdown
let isShuttingDown = false;
const cleanupServices = async () => {
  if (isShuttingDown) return;
  isShuttingDown = true;
  
  try {
    await accessibilityService.cleanup();
  } catch (error) {
    console.error('Error during service cleanup:', error);
  }
};

process.on('SIGTERM', cleanupServices);
process.on('SIGINT', cleanupServices);

export function createV2Router(): Router {
  const router = Router();

  // GET /api/v2/sitemap/:domain - Returns SitemapResult
  router.get('/sitemap/:domain', async (req: Request, res: Response) => {
    try {
      const { domain } = req.params;
      
      if (!domain) {
        return res.status(400).json({
          success: false,
          error: 'Domain parameter is required'
        });
      }
      
      const result = await sitemapService.getUrlsFromDomain(domain);
      
      res.json({
        success: true,
        data: result,
        meta: {
          timestamp: new Date().toISOString(),
          version: '2.0',
          toolVersion
        }
      });
      
    } catch (error) {
      res.status(500).json({
        success: false,
        error: (error as Error).message || 'Failed to parse sitemap'
      });
    }
  });

  // POST /api/v2/page/accessibility - Returns AccessibilityResult
  router.post('/page/accessibility', async (req: Request, res: Response) => {
    try {
      const { url, options = {} } = req.body;
      
      if (!url) {
        return res.status(400).json({
          success: false,
          error: 'URL is required in request body'
        });
      }
      
      const result = await accessibilityService.analyzeUrl(url, options);
      
      res.json({
        success: true,
        data: result,
        meta: {
          timestamp: new Date().toISOString(),
          version: '2.0',
          toolVersion,
          analyzedUrl: url
        }
      });
      
    } catch (error) {
      res.status(500).json({
        success: false,
        error: (error as Error).message || 'Failed to analyze accessibility'
      });
    }
  });

  // POST /api/v2/page/performance - Returns PerformanceResult
  router.post('/page/performance', async (req: Request, res: Response) => {
    try {
      const { url, options = {} } = req.body;
      
      if (!url) {
        return res.status(400).json({
          success: false,
          error: 'URL is required in request body'
        });
      }
      
      const result = await performanceService.analyzeUrl(url, options);
      
      res.json({
        success: true,
        data: result,
        meta: {
          timestamp: new Date().toISOString(),
          version: '2.0',
          toolVersion,
          analyzedUrl: url
        }
      });
      
    } catch (error) {
      const err = error as Error;
      // Check if it's our "not implemented" error
      if (err.message.includes('not yet implemented')) {
        res.status(501).json({
          success: false,
          error: err.message,
          available: 'Use POST /api/v1/audit/performance for full site performance analysis'
        });
      } else {
        res.status(500).json({
          success: false,
          error: err.message || 'Failed to analyze performance'
        });
      }
    }
  });

  // POST /api/v2/page/seo - Returns SEOResult
  router.post('/page/seo', async (req: Request, res: Response) => {
    try {
      const { url, options = {} } = req.body;
      
      if (!url) {
        return res.status(400).json({
          success: false,
          error: 'URL is required in request body'
        });
      }
      
      const result = await seoService.analyzeUrl(url, options);
      
      res.json({
        success: true,
        data: result,
        meta: {
          timestamp: new Date().toISOString(),
          version: '2.0',
          toolVersion,
          analyzedUrl: url
        }
      });
      
    } catch (error) {
      const err = error as Error;
      // Check if it's our "not implemented" error
      if (err.message.includes('not yet implemented')) {
        res.status(501).json({
          success: false,
          error: err.message,
          available: 'Use POST /api/v1/audit/seo for full site SEO analysis'
        });
      } else {
        res.status(500).json({
          success: false,
          error: err.message || 'Failed to analyze SEO'
        });
      }
    }
  });

  // GET /api/v2/schema - Introspection for Electron app discovery
  router.get('/schema', (req: Request, res: Response) => {
    res.json({
      success: true,
      data: {
        version: '2.0',
        toolVersion,
        description: 'AuditMySite v2.0 API - Modular endpoints using shared TypeScript types',
        endpoints: [
          {
            method: 'GET',
            path: '/api/v2/sitemap/:domain',
            returns: 'SitemapResult',
            description: 'Get sitemap URLs for domain with filtering'
          },
          {
            method: 'POST',
            path: '/api/v2/page/accessibility',
            returns: 'AccessibilityResult',
            description: 'Analyze accessibility for single URL',
            requestBody: { url: 'string', options: 'object' }
          },
          {
            method: 'POST',
            path: '/api/v2/page/performance',
            returns: 'PerformanceResult',
            description: 'Analyze performance for single URL (experimental)',
            requestBody: { url: 'string', options: 'object' }
          },
          {
            method: 'POST',
            path: '/api/v2/page/seo',
            returns: 'SEOResult',
            description: 'Analyze SEO for single URL (experimental)',
            requestBody: { url: 'string', options: 'object' }
          }
        ],
        types: {
          SitemapResult: {
            properties: ['sourceUrl', 'urls', 'parsedAt', 'totalUrls', 'filteredUrls', 'filterPatterns']
          },
          AccessibilityResult: {
            properties: ['passed', 'wcagLevel', 'score', 'errors', 'warnings', 'pa11yResults']
          },
          PerformanceResult: {
            properties: ['score', 'grade', 'coreWebVitals', 'metrics', 'issues']
          },
          SEOResult: {
            properties: ['score', 'grade', 'metaTags', 'headings', 'images', 'issues']
          }
        },
        compatibility: {
          v1: 'Available at /api/v1/* for full site analysis',
          jsonTypes: 'Same types used in CLI JSON export (FullAuditResult)',
          electronApp: 'Designed for modular access from Electron desktop app'
        }
      }
    });
  });

  return router;
}
