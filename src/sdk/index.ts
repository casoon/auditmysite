/**
 * 🔧 AuditMySite SDK - Main Entry Point
 * 
 * This is the main entry point for the AuditMySite SDK.
 * Provides everything needed for programmatic accessibility testing.
 */

// Main SDK class
export { AuditSDK } from './audit-sdk';

// API Server for remote access
export { AuditAPIServer } from '../api/server';

// Import for default export
import { AuditSDK } from './audit-sdk';
import { AuditAPIServer } from '../api/server';

// Complete type definitions
export * from './types';

// Configuration management
export { ConfigManager } from '../core/config/config-manager';

// Core components for advanced users
export { StandardPipeline } from '../core/pipeline/standard-pipeline';
// export { AccessibilityChecker } from '../core/accessibility/accessibility-checker';
// export { UnifiedQueue, QueueType } from '../core/queue/unified-queue';

// Default export for simple require/import
const AuditMySiteSDK = {
  AuditSDK,
  AuditAPIServer
};

export default AuditMySiteSDK;
