const { AccessibilityChecker } = require('./dist/core/accessibility/accessibility-checker');
const { BrowserPoolManager } = require('./dist/core/browser/browser-pool-manager');
const { SilentLogger } = require('./dist/core/logging/structured-logger');

async function testComprehensive() {
  console.log('🔍 Testing comprehensive analysis...\n');
  
  const poolManager = new BrowserPoolManager({
    maxBrowsers: 1,
    maxPagesPerBrowser: 5,
    verbose: true
  });
  
  const qualityOptions = {
    includeResourceAnalysis: true,
    includeSocialAnalysis: false,
    includeReadabilityAnalysis: true,
    includeTechnicalSEO: true,
    includeMobileFriendliness: true,
    analysisTimeout: 30000,
    psiProfile: true,
    psiCPUThrottlingRate: 4,
    psiNetwork: { latencyMs: 150, downloadKbps: 1600, uploadKbps: 750 }
  };
  
  const checker = new AccessibilityChecker({
    poolManager: poolManager,
    logger: new SilentLogger(),
    enableComprehensiveAnalysis: true,
    analyzerTypes: ['performance', 'seo', 'content-weight', 'mobile-friendliness'],
    qualityAnalysisOptions: qualityOptions
  });
  
  await checker.initialize();
  console.log('✅ Checker initialized\n');
  
  try {
    console.log('🚀 Testing https://www.casoon.de\n');
    const result = await checker.testPage('https://www.casoon.de', {
      enableComprehensiveAnalysis: true,
      timeout: 30000
    });
    
    console.log('\n📊 Results:');
    console.log(`URL: ${result.url}`);
    console.log(`Title: ${result.title}`);
    console.log(`Duration: ${result.duration}ms`);
    console.log(`Accessibility: ${result.accessibilityResult.passed ? 'PASSED' : 'FAILED'}`);
    console.log(`\nComprehensive Analysis:`);
    
    if (result.comprehensiveAnalysis) {
      console.log(`  ✅ Present`);
      console.log(`  Results count: ${result.comprehensiveAnalysis.results ? result.comprehensiveAnalysis.results.length : 0}`);
      
      if (result.comprehensiveAnalysis.results) {
        console.log(`\n  Analyzer Results:`);
        result.comprehensiveAnalysis.results.forEach(r => {
          console.log(`    - ${r.metadata?.analyzerType || 'unknown'}: ${r.success ? 'SUCCESS' : 'FAILED'}`);
          if (r.metadata?.analyzerType === 'seo') {
            console.log(`\n      SEO Result Structure:`);
            console.log(`      H1 count: ${r.headingStructure?.h1Count || 0}`);
            console.log(`      H2 count: ${r.headingStructure?.h2Count || 0}`);
            console.log(`      Structure valid: ${r.headingStructure?.structureValid}`);
            console.log(`      Issues: ${JSON.stringify(r.headingStructure?.issues || [])}`);
            console.log(`      Score: ${r.score || 0}`);
            console.log(`      Grade: ${r.grade || 'F'}`);
          }
        });
      }
    } else {
      console.log(`  ❌ Missing!`);
    }
    
    console.log(`\npa11y Score: ${result.accessibilityResult.pa11yScore || 0}`);
    console.log(`pa11y Issues: ${result.accessibilityResult.pa11yIssues?.length || 0}`);
    
  } catch (error) {
    console.error('❌ Error:', error);
  } finally {
    await checker.cleanup();
    await poolManager.cleanup();
  }
}

testComprehensive().catch(console.error);
