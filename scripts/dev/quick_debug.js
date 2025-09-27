const { AccessibilityChecker } = require('./dist/core/accessibility');
const { BrowserPoolManager } = require('./dist/core/browser/browser-pool-manager');

async function quickDebug() {
  const poolManager = new BrowserPoolManager({ maxBrowsers: 1, verbose: false });
  const checker = new AccessibilityChecker({
    usePooling: true,
    poolManager,
    enableComprehensiveAnalysis: true
  });
  
  await checker.initialize();
  
  checker.setUnifiedEventCallbacks({
    onUrlCompleted: (url, result) => {
      console.log('Event Result keys:', Object.keys(result));
      console.log('Has securityHeaders:', !!result.securityHeaders);
      console.log('Has structuredData:', !!result.structuredData);
      console.log('Has contentWeight:', !!result.contentWeight);
      console.log('Has enhancedPerformance:', !!result.enhancedPerformance);
    }
  });
  
  const results = await checker.testMultiplePagesWithQueue(['https://www.inros-lackner.de'], { verbose: false });
  
  console.log('Direct Result keys:', Object.keys(results[0]));
  console.log('Direct has securityHeaders:', !!results[0].securityHeaders);
  
  await checker.cleanup();
  await poolManager.cleanup();
}

quickDebug().catch(console.error);