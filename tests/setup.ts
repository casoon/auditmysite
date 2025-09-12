/**
 * 🧪 Test Setup & Global Mocks
 * 
 * Global test configuration and mocks to keep tests fast and reliable.
 * Mocks external dependencies that cause slow I/O operations.
 */

// Mock external dependencies that cause slow/unreliable tests
jest.mock('playwright', () => ({
  chromium: {
    launch: jest.fn(() => Promise.resolve({
      // Real browser interface: browser.newContext() -> context -> context.newPage()
      newContext: jest.fn(() => Promise.resolve({
        newPage: jest.fn(() => Promise.resolve({
          goto: jest.fn(),
          content: jest.fn(() => Promise.resolve('<html><body>Mock page</body></html>')),
          evaluate: jest.fn(),
          close: jest.fn(),
          title: jest.fn(() => Promise.resolve('Mock Title')),
          locator: jest.fn(() => ({
            count: jest.fn(() => Promise.resolve(0)),
            filter: jest.fn(() => ({
              count: jest.fn(() => Promise.resolve(0))
            }))
          })),
          setDefaultTimeout: jest.fn(),
          setDefaultNavigationTimeout: jest.fn()
        })),
        route: jest.fn(() => Promise.resolve()),
        close: jest.fn()
      })),
      close: jest.fn(),
      isConnected: jest.fn(() => true)
    }))
  },
  firefox: {
    launch: jest.fn(() => Promise.resolve({
      newContext: jest.fn(() => Promise.resolve({
        newPage: jest.fn(() => Promise.resolve({
          goto: jest.fn(),
          close: jest.fn(),
          title: jest.fn(() => Promise.resolve('Mock Title'))
        })),
        route: jest.fn(() => Promise.resolve()),
        close: jest.fn()
      })),
      close: jest.fn(),
      isConnected: jest.fn(() => true)
    }))
  },
  webkit: {
    launch: jest.fn(() => Promise.resolve({
      newContext: jest.fn(() => Promise.resolve({
        newPage: jest.fn(() => Promise.resolve({
          goto: jest.fn(),
          close: jest.fn(),
          title: jest.fn(() => Promise.resolve('Mock Title'))
        })),
        route: jest.fn(() => Promise.resolve()),
        close: jest.fn()
      })),
      close: jest.fn(),
      isConnected: jest.fn(() => true)
    }))
  }
}));

jest.mock('pa11y', () => jest.fn(() => Promise.resolve([])));

jest.mock('fast-xml-parser', () => ({
  XMLParser: jest.fn().mockImplementation(() => ({
    parse: jest.fn(() => ({
      urlset: {
        url: [
          { loc: 'https://example.com/' },
          { loc: 'https://example.com/page1' },
          { loc: 'https://example.com/page2' }
        ]
      }
    }))
  }))
}));

// Mock file system operations for report generation tests
jest.mock('fs', () => ({
  ...jest.requireActual('fs'),
  writeFileSync: jest.fn(),
  existsSync: jest.fn(() => true),
  mkdirSync: jest.fn(),
  createReadStream: jest.fn(() => ({
    pipe: jest.fn()
  }))
}));

// Mock HTTP requests
global.fetch = jest.fn(() =>
  Promise.resolve({
    ok: true,
    status: 200,
    text: () => Promise.resolve(`<?xml version="1.0"?>
      <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url><loc>https://example.com/</loc></url>
        <url><loc>https://example.com/page1</loc></url>
      </urlset>`)
  })
) as jest.Mock;

// Test utilities
export const createMockAuditResult = (overrides = {}) => ({
  sessionId: 'test-session-123',
  sitemapUrl: 'https://example.com/sitemap.xml',
  startTime: new Date('2023-01-01T00:00:00Z'),
  endTime: new Date('2023-01-01T00:01:00Z'),
  duration: 60000,
  summary: {
    testedPages: 3,
    passedPages: 2,
    failedPages: 1,
    crashedPages: 0,
    totalErrors: 5,
    totalWarnings: 2,
    totalDuration: 60000,
    results: []
  },
  results: [],
  reports: [],
  metadata: {
    version: '1.6.1',
    environment: 'test',
    userAgent: 'AuditMySite-Test',
    configuration: {}
  },
  ...overrides
});

export const createMockPageResult = (overrides = {}) => ({
  url: 'https://example.com/test-page',
  title: 'Test Page',
  passed: true,
  crashed: false,
  errors: [],
  warnings: [],
  duration: 1000,
  timestamp: '2023-01-01T00:00:00Z',
  performanceMetrics: {
    largestContentfulPaint: 2500,
    cumulativeLayoutShift: 0.1,
    firstInputDelay: 100,
    timeToInteractive: 3000
  },
  ...overrides
});

export const createMockGeneratedReport = (format = 'html', overrides = {}) => ({
  format,
  path: `/mock/reports/report.${format}`,
  size: 1024,
  metadata: {
    generatedAt: new Date('2023-01-01T00:00:00Z'),
    duration: 100
  },
  ...overrides
});

// Suppress console logs in tests unless explicitly testing them
const originalConsole = { ...console };
beforeEach(() => {
  if (!process.env.TEST_VERBOSE) {
    console.log = jest.fn();
    console.warn = jest.fn();
    console.error = jest.fn();
  }
});

afterEach(() => {
  if (!process.env.TEST_VERBOSE) {
    console.log = originalConsole.log;
    console.warn = originalConsole.warn;
    console.error = originalConsole.error;
  }
});

// Cleanup after each test
afterEach(() => {
  jest.clearAllMocks();
});
