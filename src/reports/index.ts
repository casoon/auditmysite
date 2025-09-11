// 🎯 SINGLE SOURCE OF TRUTH - Clean report system
export * from './types/report-export';
export * from './exporters/unified-export';

// Core components for backward compatibility only
export * from './detailed-issue-markdown';
export * from './report-utils';

// ⚠️ IMPORTANT: New code should use:
// - HTMLGenerator from '../generators/html-generator'
// - JsonGenerator from '../generators/json-generator'
// - MarkdownGenerator from '../generators/markdown-generator'
