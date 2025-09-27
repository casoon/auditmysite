#!/usr/bin/env node

/**
 * API Server Startup Script
 * 
 * Starts the AuditMySite v2.0 API server with proper configuration.
 * Supports both development and production modes.
 */

const { AuditAPIServer } = require('./dist/api/server');

const config = {
  port: process.env.PORT || 3000,
  host: process.env.HOST || '0.0.0.0',
  apiKeyRequired: process.env.NODE_ENV === 'production',
  maxConcurrentJobs: parseInt(process.env.MAX_CONCURRENT_JOBS || '5'),
  enableSwagger: process.env.NODE_ENV !== 'production' // Only in dev/staging
};

async function startServer() {
  console.log('🚀 Starting AuditMySite API Server v2.0...');
  console.log(`   Environment: ${process.env.NODE_ENV || 'development'}`);
  console.log(`   Host: ${config.host}:${config.port}`);
  console.log(`   API Key Required: ${config.apiKeyRequired ? 'Yes' : 'No'}`);
  
  const server = new AuditAPIServer(config);
  
  try {
    await server.start();
    console.log('\n✅ Server started successfully!');
    console.log(`🌐 API v1: http://${config.host === '0.0.0.0' ? 'localhost' : config.host}:${config.port}/api/v1/info`);
    console.log(`🚀 API v2: http://${config.host === '0.0.0.0' ? 'localhost' : config.host}:${config.port}/api/v2/schema`);
    
    if (config.enableSwagger) {
      console.log(`📚 Swagger: http://${config.host === '0.0.0.0' ? 'localhost' : config.host}:${config.port}/api-docs`);
    }
    
    console.log('\n📊 Available v2.0 endpoints:');
    console.log('   GET  /api/v2/sitemap/:domain     → SitemapResult');
    console.log('   POST /api/v2/page/accessibility  → AccessibilityResult'); 
    console.log('   POST /api/v2/page/performance    → PerformanceResult (experimental)');
    console.log('   POST /api/v2/page/seo            → SEOResult (experimental)');
    console.log('   GET  /api/v2/schema              → API introspection');
    
    // Graceful shutdown
    const shutdown = () => {
      console.log('\n🛑 Graceful shutdown initiated...');
      process.exit(0);
    };
    
    process.on('SIGTERM', shutdown);
    process.on('SIGINT', shutdown);
    
  } catch (error) {
    console.error('❌ Failed to start server:', error.message);
    process.exit(1);
  }
}

if (require.main === module) {
  startServer();
}

module.exports = { startServer };
