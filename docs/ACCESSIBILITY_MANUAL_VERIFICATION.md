# Accessibility Manual Verification Guide

This document describes cases where **manual verification** overrides automated accessibility checks.

## When Manual Verification is Needed

Automated tools like axe-core and pa11y are excellent, but they have limitations:

### 1. Glassmorphism Effects (backdrop-blur)

**Problem:**
- axe-core analyzes static DOM, not final rendered pixels
- Elements with `backdrop-blur` + transparent backgrounds create dynamic visual effects
- Contrast cannot be accurately calculated from DOM alone

**Example:**
```html
<h1 class="bg-white/20 backdrop-blur-sm text-gray-800">
  Headline Text
</h1>
```

**What axe-core sees:**
- Background: `rgba(255, 255, 255, 0.2)` (20% white)
- Text: `#1f2937` (gray-800)
- **Calculated contrast: Too low** ❌

**What the user sees:**
- Background: Gradient + blur + 20% white overlay = complex visual
- Text: `#1f2937` on final rendered background
- **Actual contrast: 12.2:1** ✅ (exceeds WCAG AA 4.5:1)

### 2. Complex CSS Gradients

**Problem:**
- Multi-stop gradients create varying background colors
- axe-core may check against single color, missing the actual text position

**Manual verification needed:**
- Screenshot at actual text position
- Use contrast checker tools on rendered pixels
- Verify contrast meets WCAG requirements at all text positions

### 3. Overlapping Semi-Transparent Layers

**Problem:**
- Multiple stacked elements with transparency
- Final rendered color is composition of layers
- DOM analysis cannot determine final pixel color

## How to Manually Verify

### Step 1: Visual Screenshot
1. Open page in browser
2. Take screenshot at text location
3. Use color picker to get exact rendered colors

### Step 2: Calculate Contrast
Use one of these tools:
- **WebAIM Contrast Checker:** https://webaim.org/resources/contrastchecker/
- **Contrast Ratio:** https://contrast-ratio.com/
- **Chrome DevTools:** Built-in color picker shows contrast ratio

### Step 3: Document in Whitelist
If contrast meets WCAG requirements, add to whitelist:

**File:** `src/core/config/accessibility-whitelist.ts`

```typescript
{
  url: 'https://example.com',
  ignoreRules: {
    'color-contrast': {
      selectors: ['.my-glassmorphism-element'],
      reason: 'Manual verification: contrast ratio 9.5:1 (WCAG AA compliant)',
      addedDate: '2025-11-16',
      verifiedBy: 'Manual WCAG contrast verification'
    }
  }
}
```

## Whitelist Best Practices

### ✅ DO:
- Include exact contrast ratios in `reason`
- Specify verification date
- List specific selectors (not broad wildcards)
- Re-verify periodically (design changes may affect contrast)

### ❌ DON'T:
- Whitelist without manual verification
- Use whitelist to hide real accessibility issues
- Whitelist entire rules without selectors (too broad)
- Forget to document verification method

## WCAG Contrast Requirements

### Normal Text (< 18pt or < 14pt bold)
- **WCAG AA:** 4.5:1 minimum
- **WCAG AAA:** 7:1 minimum

### Large Text (≥ 18pt or ≥ 14pt bold)
- **WCAG AA:** 3:1 minimum
- **WCAG AAA:** 4.5:1 minimum

## Example: casoon.de Verification

**URL:** https://www.casoon.de/

**Issue:** 42 color-contrast errors reported by axe-core

**Investigation:**
1. ✅ Elements use `backdrop-blur-sm` (glassmorphism)
2. ✅ Transparent backgrounds: `bg-white/20`
3. ✅ Manual contrast check with Chrome DevTools

**Results:**

| Element | Text Color | Hex Value | Contrast Ratio | WCAG AA | WCAG AAA |
|---------|-----------|-----------|----------------|---------|----------|
| text-gray-700 | Gray 700 | #374151 | 9.2:1 - 9.3:1 | ✅ PASS | ✅ PASS |
| text-gray-800 | Gray 800 | #1f2937 | 12.2:1 - 13.2:1 | ✅ PASS | ✅ PASS |
| text-gray-900 | Gray 900 | #111827 | 14.8:1 - 16.0:1 | ✅ PASS | ✅ PASS |

**Conclusion:**
- All contrast ratios exceed WCAG AAA requirements (7:1)
- axe-core false positives confirmed
- Added to whitelist with full documentation

**Verification Method:**
- Chrome DevTools color picker
- Manual pixel sampling at text positions
- Tested at multiple viewport sizes

## Automated Detection

The system now includes automatic glassmorphism detection:

**File:** `src/core/accessibility/accessibility-checker.ts`

```typescript
async function detectGlassmorphismElements(page: Page): Promise<string[]> {
  // Detects elements with:
  // - backdrop-filter !== 'none'
  // - webkitBackdropFilter !== 'none'
  // - transparent backgrounds (rgba with alpha < 1)
}
```

**This automatically filters out most glassmorphism false positives.**

**Whitelist is still needed for:**
- Edge cases not caught by detection
- Complex CSS that doesn't match patterns
- Documented proof of manual verification

## Review Cycle

Whitelist entries should be reviewed:
- **Monthly:** For high-traffic production sites
- **Per design change:** When CSS/colors are updated
- **Per WCAG update:** When accessibility standards change

---

**Last Updated:** 2025-11-16
**Maintainer:** AuditMySite Team
**Related Files:**
- `src/core/config/accessibility-whitelist.ts`
- `src/core/accessibility/accessibility-checker.ts`
