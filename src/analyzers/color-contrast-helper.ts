/**
 * 🎨 Color Contrast Helper
 * 
 * Analyzes color contrast issues and provides actionable recommendations
 * with specific color suggestions that meet WCAG standards.
 */

export interface ColorPair {
  foreground: string;
  background: string;
  contrastRatio: number;
}

export interface ColorRecommendation {
  originalForeground: string;
  originalBackground: string;
  currentRatio: number;
  requiredRatio: number;
  level: 'AA' | 'AAA';
  suggestions: {
    foreground?: string[];
    background?: string[];
    explanation: string;
  };
}

/**
 * Calculate relative luminance of a color
 * Formula from WCAG 2.0: https://www.w3.org/TR/WCAG20/#relativeluminancedef
 */
function getLuminance(r: number, g: number, b: number): number {
  const [rs, gs, bs] = [r, g, b].map(c => {
    c = c / 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

/**
 * Calculate contrast ratio between two colors
 * Formula from WCAG 2.0: https://www.w3.org/TR/WCAG20/#contrast-ratiodef
 */
export function calculateContrastRatio(fg: string, bg: string): number {
  const fgRgb = parseColor(fg);
  const bgRgb = parseColor(bg);
  
  if (!fgRgb || !bgRgb) return 0;
  
  const l1 = getLuminance(fgRgb.r, fgRgb.g, fgRgb.b);
  const l2 = getLuminance(bgRgb.r, bgRgb.g, bgRgb.b);
  
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Parse CSS color to RGB
 */
function parseColor(color: string): { r: number; g: number; b: number } | null {
  // Handle hex colors
  if (color.startsWith('#')) {
    const hex = color.replace('#', '');
    if (hex.length === 3) {
      return {
        r: parseInt(hex[0] + hex[0], 16),
        g: parseInt(hex[1] + hex[1], 16),
        b: parseInt(hex[2] + hex[2], 16)
      };
    }
    if (hex.length === 6) {
      return {
        r: parseInt(hex.substring(0, 2), 16),
        g: parseInt(hex.substring(2, 4), 16),
        b: parseInt(hex.substring(4, 6), 16)
      };
    }
  }
  
  // Handle rgb/rgba
  const rgbMatch = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
  if (rgbMatch) {
    return {
      r: parseInt(rgbMatch[1]),
      g: parseInt(rgbMatch[2]),
      b: parseInt(rgbMatch[3])
    };
  }
  
  // Handle common color names
  const colorNames: Record<string, string> = {
    'white': '#FFFFFF',
    'black': '#000000',
    'red': '#FF0000',
    'green': '#008000',
    'blue': '#0000FF',
    'gray': '#808080',
    'grey': '#808080'
  };
  
  if (colorNames[color.toLowerCase()]) {
    return parseColor(colorNames[color.toLowerCase()]);
  }
  
  return null;
}

/**
 * Get color recommendations for contrast issues
 */
export function getColorRecommendations(
  foreground: string,
  background: string,
  textSize: 'normal' | 'large' = 'normal',
  targetLevel: 'AA' | 'AAA' = 'AA'
): ColorRecommendation {
  const currentRatio = calculateContrastRatio(foreground, background);
  
  // WCAG 2.0 requirements
  const requiredRatio = targetLevel === 'AAA' 
    ? (textSize === 'large' ? 4.5 : 7) 
    : (textSize === 'large' ? 3 : 4.5);
  
  const recommendation: ColorRecommendation = {
    originalForeground: foreground,
    originalBackground: background,
    currentRatio,
    requiredRatio,
    level: targetLevel,
    suggestions: {
      explanation: ''
    }
  };
  
  if (currentRatio >= requiredRatio) {
    recommendation.suggestions.explanation = `Current contrast ratio ${currentRatio.toFixed(2)}:1 meets ${targetLevel} standards`;
    return recommendation;
  }
  
  // Generate suggestions
  const fgRgb = parseColor(foreground);
  const bgRgb = parseColor(background);
  
  if (!fgRgb || !bgRgb) {
    recommendation.suggestions.explanation = 'Could not parse colors for recommendations';
    return recommendation;
  }
  
  // Determine if we should darken foreground or lighten background
  const bgLuminance = getLuminance(bgRgb.r, bgRgb.g, bgRgb.b);
  
  if (bgLuminance > 0.5) {
    // Light background - suggest darker foreground
    recommendation.suggestions.foreground = [
      '#1F2937', // gray-800
      '#111827', // gray-900
      '#000000', // black
      '#374151'  // gray-700
    ];
    recommendation.suggestions.explanation = 
      `Current ratio ${currentRatio.toFixed(2)}:1 is below ${targetLevel} standard (${requiredRatio}:1). ` +
      `With a light background, use darker text colors. Suggested: #1F2937 (gray-800) or #111827 (gray-900).`;
  } else {
    // Dark background - suggest lighter foreground
    recommendation.suggestions.foreground = [
      '#FFFFFF', // white
      '#F9FAFB', // gray-50
      '#F3F4F6', // gray-100
      '#E5E7EB'  // gray-200
    ];
    recommendation.suggestions.explanation = 
      `Current ratio ${currentRatio.toFixed(2)}:1 is below ${targetLevel} standard (${requiredRatio}:1). ` +
      `With a dark background, use lighter text colors. Suggested: #FFFFFF (white) or #F9FAFB (gray-50).`;
  }
  
  return recommendation;
}

/**
 * Analyze common Tailwind CSS color combinations
 */
export function analyzeTailwindContrast(className: string, background: string = '#FFFFFF'): {
  meetsAA: boolean;
  meetsAAA: boolean;
  ratio: number;
  recommendation?: string;
} {
  const tailwindColors: Record<string, string> = {
    'text-gray-50': '#F9FAFB',
    'text-gray-100': '#F3F4F6',
    'text-gray-200': '#E5E7EB',
    'text-gray-300': '#D1D5DB',
    'text-gray-400': '#9CA3AF',
    'text-gray-500': '#6B7280',
    'text-gray-600': '#4B5563',
    'text-gray-700': '#374151',
    'text-gray-800': '#1F2937',
    'text-gray-900': '#111827',
    'text-black': '#000000',
    'text-white': '#FFFFFF'
  };
  
  const color = tailwindColors[className];
  if (!color) {
    return {
      meetsAA: false,
      meetsAAA: false,
      ratio: 0,
      recommendation: 'Unknown Tailwind class'
    };
  }
  
  const ratio = calculateContrastRatio(color, background);
  const meetsAA = ratio >= 4.5;
  const meetsAAA = ratio >= 7;
  
  let recommendation: string | undefined;
  if (!meetsAA) {
    if (background === '#FFFFFF' || background.toLowerCase() === 'white') {
      recommendation = 'Use text-gray-800 or text-gray-900 for better contrast on white backgrounds';
    } else {
      recommendation = 'Adjust text color to meet WCAG AA standards (4.5:1 ratio)';
    }
  }
  
  return { meetsAA, meetsAAA, ratio, recommendation };
}

/**
 * Generate contrast fix CSS
 */
export function generateContrastFixCSS(
  selector: string,
  originalColor: string,
  background: string
): string {
  const recommendation = getColorRecommendations(originalColor, background);
  
  if (!recommendation.suggestions.foreground || recommendation.suggestions.foreground.length === 0) {
    return `/* No specific recommendation available for ${selector} */`;
  }
  
  const suggestedColor = recommendation.suggestions.foreground[0];
  
  return `
/* Original: ${originalColor} (contrast ratio: ${recommendation.currentRatio.toFixed(2)}:1) */
/* Required: ${recommendation.requiredRatio}:1 for WCAG ${recommendation.level} */
${selector} {
  color: ${suggestedColor}; /* Meets ${recommendation.level} standards */
}`;
}

/**
 * Batch analyze colors from accessibility report
 */
export function analyzeContrastIssues(issues: Array<{
  selector: string;
  context: string;
  code: string;
}>): Array<{
  selector: string;
  issue: string;
  recommendation: ColorRecommendation;
  fixCSS: string;
}> {
  const results = [];
  
  for (const issue of issues) {
    if (issue.code !== 'color-contrast') continue;
    
    // Try to extract colors from context
    const classMatch = issue.context.match(/class="[^"]*text-gray-(\d+)/);
    const bgMatch = issue.context.match(/bg-(\w+)/);
    
    if (classMatch) {
      const textClass = `text-gray-${classMatch[1]}`;
      const background = bgMatch ? `#FFFFFF` : '#FFFFFF'; // Default to white
      
      const recommendation = getColorRecommendations(
        getTailwindColor(textClass) || '#000000',
        background
      );
      
      const fixCSS = generateContrastFixCSS(
        issue.selector,
        getTailwindColor(textClass) || '#000000',
        background
      );
      
      results.push({
        selector: issue.selector,
        issue: `Text class ${textClass} has insufficient contrast`,
        recommendation,
        fixCSS
      });
    }
  }
  
  return results;
}

/**
 * Helper to get Tailwind color value
 */
function getTailwindColor(className: string): string | null {
  const colors: Record<string, string> = {
    'text-gray-700': '#374151',
    'text-gray-800': '#1F2937',
    'text-gray-900': '#111827'
  };
  return colors[className] || null;
}
