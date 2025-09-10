// 🎯 SINGLE SOURCE OF TRUTH - All report types are now unified
export * from './types/report-export';
export * from './exporters/unified-export';

// Legacy exports for backwards compatibility (DEPRECATED)
export * from './detailed-issue-markdown';
export * from './report-utils';
export * from './performance-issue-markdown';
export * from './unified';

// ⚠️ IMPORTANT: New code should ONLY use types from './types/report-export'
// The legacy exports above will be removed in a future version
