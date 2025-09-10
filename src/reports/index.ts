// 🎯 SINGLE SOURCE OF TRUTH - All report types are now unified
export * from './types/report-export';
export * from './exporters/unified-export';

// Core components for backward compatibility only
export * from './detailed-issue-markdown';
export * from './report-utils';
export * from './unified';

// ⚠️ IMPORTANT: New code should ONLY use:
// - UnifiedHTMLGenerator from './unified/unified-html-generator'
// - Types from './types/report-export'
