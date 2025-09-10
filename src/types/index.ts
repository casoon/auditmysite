// Main types barrel export - selective exports to avoid conflicts
export { 
  Pa11yIssue, 
  TestOptions, 
  LighthouseScores, 
  LighthouseMetrics, 
  SitemapUrl, 
  TestSummary,
  AuditIssue,
  DetailedIssue
} from '../types';

// Legacy result types (keep main AccessibilityResult from types.ts)
export type { AccessibilityResult } from '../types';

// Enhanced metrics
export * from './enhanced-metrics';

// Base types
export * from './base-types';

// New audit results (rename conflicting types)
export {
  FullAuditResult,
  AuditMetadata,
  SitemapResult,
  PageAuditResult,
  PerformanceResult,
  PerformanceIssue,
  SEOResult,
  SEOIssue,
  ContentWeightResult,
  ContentOptimization,
  AuditSummary,
  StructuredIssue,
  MobileFriendlinessResult,
  MobileFriendlinessRecommendation,
  AuditResultTypes
} from './audit-results';

// Rename conflicting types from audit-results with prefix
export {
  AuditConfig as V2AuditConfig,
  AccessibilityResult as V2AccessibilityResult,
  AccessibilityIssue as V2AccessibilityIssue,
  Grade as V2Grade,
  calculateGrade as v2CalculateGrade,
  calculateOverallScore as v2CalculateOverallScore
} from './audit-results';

// Queue types
export * from '../core/queue/types';
