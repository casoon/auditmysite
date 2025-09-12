const { HTMLGenerator } = require('./dist/generators/html-generator');
const fs = require('fs');
const path = require('path');

async function testEnhancedGenerator() {
  console.log('🧪 Testing Enhanced HTML Generator...');
  
  try {
    const generator = new HTMLGenerator();
    const jsonPath = path.join(__dirname, 'test-enhanced-reports', 'test-data.json');
    
    console.log('📄 Loading test data from:', jsonPath);
    const htmlContent = await generator.generateFromJSON(jsonPath);
    
    const outputPath = path.join(__dirname, 'test-enhanced-reports', 'enhanced-report.html');
    fs.writeFileSync(outputPath, htmlContent, 'utf8');
    
    console.log('✅ Enhanced HTML report generated successfully!');
    console.log('📄 Report saved to:', outputPath);
    
    // Check if the HTML contains expected elements
    if (htmlContent.includes('certificate-badge')) {
      console.log('✅ Certificate badge found in HTML');
    } else {
      console.log('❌ Certificate badge NOT found in HTML');
    }
    
    if (htmlContent.includes('sticky-nav')) {
      console.log('✅ Sticky navigation found in HTML');
    } else {
      console.log('❌ Sticky navigation NOT found in HTML');
    }
    
    if (htmlContent.includes('Grade C')) {
      console.log('✅ Grade display found in HTML');
    } else {
      console.log('❌ Grade display NOT found in HTML');
    }
    
    if (htmlContent.includes('Overall Score: 75/100')) {
      console.log('✅ Overall score found in HTML');
    } else {
      console.log('❌ Overall score NOT found in HTML');
    }
    
    console.log('🔗 Open the HTML file in your browser to see the enhanced report with certificate badges!');
    
  } catch (error) {
    console.error('❌ Error testing enhanced generator:', error);
  }
}

testEnhancedGenerator();
