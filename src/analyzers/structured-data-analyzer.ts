/**
 * 📊 Structured Data Analyzer
 * 
 * Comprehensive structured data validation including:
 * - JSON-LD, Microdata, and RDFa detection
 * - Schema.org compliance validation
 * - Rich snippets potential analysis
 * - Knowledge Graph readiness assessment
 * - SEO impact evaluation
 */

import { Page } from 'playwright';
import { 
  StructuredDataMetrics,
  StructuredDataItem,
  SchemaTypeAnalysis,
  RichSnippetsAnalysis,
  KnowledgeGraphAnalysis
} from '../types/enhanced-metrics';
import {
  BaseAnalyzer,
  BaseAnalysisResult,
  BaseAnalysisOptions,
  BaseRecommendation,
  Grade,
  CertificateLevel,
  calculateGrade,
  calculateCertificateLevel
} from '../types/base-types';

// Structured Data specific result interface
interface StructuredDataAnalysisResult extends BaseAnalysisResult {
  structuredData: StructuredDataMetrics;
  recommendations: BaseRecommendation[];
}

// Structured Data specific options interface
interface StructuredDataAnalysisOptions extends BaseAnalysisOptions {
  /** Include rich snippets analysis */
  includeRichSnippetsAnalysis?: boolean;
  /** Include Knowledge Graph analysis */
  includeKnowledgeGraphAnalysis?: boolean;
  /** Validate against specific schema types */
  targetSchemaTypes?: string[];
  /** Enable deep schema validation */
  deepValidation?: boolean;
}

export class StructuredDataAnalyzer implements BaseAnalyzer<StructuredDataAnalysisResult, StructuredDataAnalysisOptions> {
  
  // Schema.org required properties mapping
  private readonly schemaRequiredProperties: Record<string, string[]> = {
    'Organization': ['name'],
    'LocalBusiness': ['name', 'address'],
    'Article': ['headline', 'author', 'datePublished'],
    'BlogPosting': ['headline', 'author', 'datePublished'],
    'Product': ['name', 'image', 'description'],
    'Offer': ['price', 'availability'],
    'Person': ['name'],
    'Event': ['name', 'startDate', 'location'],
    'Recipe': ['name', 'author', 'description'],
    'VideoObject': ['name', 'description', 'thumbnailUrl', 'uploadDate'],
    'ImageObject': ['contentUrl'],
    'WebPage': ['name'],
    'WebSite': ['name', 'url'],
    'BreadcrumbList': ['itemListElement'],
    'FAQPage': ['mainEntity'],
    'HowTo': ['name', 'step']
  };

  // Schema.org recommended properties mapping
  private readonly schemaRecommendedProperties: Record<string, Array<{property: string, benefit: string}>> = {
    'Organization': [
      { property: 'logo', benefit: 'Displays organization logo in search results' },
      { property: 'url', benefit: 'Links to official website' },
      { property: 'sameAs', benefit: 'Social media profile connections' },
      { property: 'contactPoint', benefit: 'Contact information for customers' }
    ],
    'Article': [
      { property: 'image', benefit: 'Article thumbnail in search results' },
      { property: 'dateModified', benefit: 'Shows content freshness' },
      { property: 'publisher', benefit: 'Publisher information for trust' },
      { property: 'articleSection', benefit: 'Content categorization' }
    ],
    'Product': [
      { property: 'brand', benefit: 'Brand recognition in search' },
      { property: 'sku', benefit: 'Product identification' },
      { property: 'offers', benefit: 'Price and availability information' },
      { property: 'aggregateRating', benefit: 'Star ratings in search results' }
    ]
  };

  constructor() {}

  // BaseAnalyzer interface implementations
  getName(): string {
    return 'StructuredDataAnalyzer';
  }

  getVersion(): string {
    return '1.0.0';
  }

  getScore(result: StructuredDataAnalysisResult): number {
    return result.overallScore;
  }

  getGrade(score: number): Grade {
    return calculateGrade(score);
  }

  getCertificateLevel(score: number): CertificateLevel {
    return calculateCertificateLevel(score);
  }

  getRecommendations(result: StructuredDataAnalysisResult): BaseRecommendation[] {
    return result.recommendations;
  }

  /**
   * Main analyze method implementing BaseAnalyzer interface
   */
  async analyze(page: Page, url: string | { loc: string }, options: StructuredDataAnalysisOptions = {}): Promise<StructuredDataAnalysisResult> {
    // Extract URL string from URL object if needed
    const urlString = (typeof url === 'object' && url.loc ? url.loc : url) as string;

    const startTime = Date.now();
    
    try {
      // Analyze structured data
      const structuredData = await this.analyzeStructuredData(page, urlString, options);
      
      const duration = Date.now() - startTime;
      
      // Calculate overall score
      const overallScore = this.calculateOverallScore(structuredData);
      const grade = calculateGrade(overallScore);
      const certificate = calculateCertificateLevel(overallScore);
      
      // Generate recommendations
      const recommendations = this.generateRecommendations(structuredData);

      return {
        overallScore,
        grade,
        certificate,
        analyzedAt: new Date().toISOString(),
        duration,
        status: 'completed' as const,
        structuredData,
        recommendations
      };

    } catch (error) {
      console.error('❌ Structured data analysis failed:', error);
      throw new Error(`Structured data analysis failed: ${error}`);
    }
  }

  /**
   * Analyze structured data comprehensively
   */
  private async analyzeStructuredData(
    page: Page, 
    url: string, 
    options: StructuredDataAnalysisOptions
  ): Promise<StructuredDataMetrics> {
    
    // Extract all structured data from the page
    const items = await this.extractStructuredDataItems(page);
    
    // Analyze by schema type
    const schemaTypes = this.analyzeSchemaTypes(items);
    
    // Rich snippets analysis
    const richSnippets = options.includeRichSnippetsAnalysis 
      ? this.analyzeRichSnippets(items, schemaTypes)
      : this.getDefaultRichSnippetsAnalysis();
    
    // Knowledge Graph analysis
    const knowledgeGraph = options.includeKnowledgeGraphAnalysis 
      ? this.analyzeKnowledgeGraph(items, schemaTypes)
      : this.getDefaultKnowledgeGraphAnalysis();
    
    // SEO impact analysis
    const seoImpact = this.analyzeSEOImpact(items, schemaTypes, richSnippets);
    
    // Collect validation issues
    const issues = this.collectValidationIssues(items, schemaTypes);
    
    // Calculate summary statistics
    const summary = {
      totalItems: items.length,
      validItems: items.filter(item => item.valid).length,
      invalidItems: items.filter(item => !item.valid).length,
      jsonLdCount: items.filter(item => item.format === 'JSON-LD').length,
      microdataCount: items.filter(item => item.format === 'Microdata').length,
      rdfaCount: items.filter(item => item.format === 'RDFa').length,
      uniqueTypes: [...new Set(items.map(item => item.type))]
    };

    // Calculate overall score
    const overallScore = this.calculateStructuredDataScore(
      items, schemaTypes, richSnippets, knowledgeGraph
    );
    
    const structuredDataGrade = calculateGrade(overallScore);
    
    // Generate recommendations
    const recommendations = this.generateStructuredDataRecommendations(
      items, schemaTypes, richSnippets, knowledgeGraph, issues
    );

    // Generate testing URLs
    const testingUrls = {
      googleRichResultsTest: `https://search.google.com/test/rich-results?url=${encodeURIComponent(url)}`,
      googleStructuredDataTest: `https://validator.schema.org/#url=${encodeURIComponent(url)}`,
      schemaMarkupValidator: `https://validator.schema.org/#url=${encodeURIComponent(url)}`
    };

    return {
      overallScore,
      structuredDataGrade,
      summary,
      items,
      schemaTypes,
      richSnippets,
      knowledgeGraph,
      seoImpact,
      issues,
      recommendations,
      testingUrls
    };
  }

  /**
   * Extract all structured data items from the page
   */
  private async extractStructuredDataItems(page: Page): Promise<StructuredDataItem[]> {
    return await page.evaluate(() => {
      // Helper functions defined in page context
      function validateJsonLd(data: any): boolean {
        return data && typeof data === 'object' && data['@type'];
      }

      function validateMicrodata(data: any, type: string): boolean {
        return data && typeof data === 'object' && Object.keys(data).length > 0;
      }

      function validateRdfa(data: any, type: string): boolean {
        return data && typeof data === 'object' && Object.keys(data).length > 0;
      }

      function calculateComplianceScore(data: any, type: string): number {
        if (!data || typeof data !== 'object') return 0;
        
        // Basic scoring based on presence of common properties
        let score = 50; // Base score
        
        if (data.name) score += 20;
        if (data.description) score += 10;
        if (data.image) score += 10;
        if (data.url) score += 10;
        
        return Math.min(100, score);
      }

      function extractMicrodataFromElement(element: HTMLElement): any {
        const data: any = {};
        
        // Extract itemprops
        const itemProps = element.querySelectorAll('[itemprop]');
        itemProps.forEach(prop => {
          const propName = prop.getAttribute('itemprop');
          if (propName) {
            const content = prop.getAttribute('content') || 
                           prop.textContent?.trim() || 
                           prop.getAttribute('href') ||
                           prop.getAttribute('src');
            if (content) {
              data[propName] = content;
            }
          }
        });
        
        return data;
      }

      function extractRdfaFromElement(element: HTMLElement): any {
        const data: any = {};
        
        // Extract RDFa properties
        const properties = element.querySelectorAll('[property]');
        properties.forEach(prop => {
          const propName = prop.getAttribute('property');
          if (propName) {
            const content = prop.getAttribute('content') || 
                           prop.textContent?.trim() ||
                           prop.getAttribute('href') ||
                           prop.getAttribute('src');
            if (content) {
              data[propName] = content;
            }
          }
        });
        
        return data;
      }

      function generateUniqueSelector(element: Element): string {
        const tagName = element.tagName.toLowerCase();
        const className = element.className ? `.${element.className.replace(/\s+/g, '.')}` : '';
        const id = element.id ? `#${element.id}` : '';
        
        return `${tagName}${id}${className}`;
      }
      
      const items: StructuredDataItem[] = [];

      // Extract JSON-LD
      const jsonLdScripts = document.querySelectorAll('script[type="application/ld+json"]');
      jsonLdScripts.forEach((script, index) => {
        try {
          const data = JSON.parse(script.textContent || '{}');
          const dataArray = Array.isArray(data) ? data : [data];
          
          dataArray.forEach((item, subIndex) => {
            if (item['@type']) {
              items.push({
                format: 'JSON-LD',
                type: item['@type'],
                location: script.closest('head') ? 'head' : 'body',
                selector: `script[type="application/ld+json"]:nth-of-type(${index + 1})`,
                data: item,
                valid: validateJsonLd(item),
                errors: [],
                warnings: [],
                complianceScore: calculateComplianceScore(item, item['@type'])
              } as StructuredDataItem);
            }
          });
        } catch (error) {
          items.push({
            format: 'JSON-LD',
            type: 'Invalid',
            location: script.closest('head') ? 'head' : 'body',
            selector: `script[type="application/ld+json"]:nth-of-type(${index + 1})`,
            data: null,
            valid: false,
            errors: [`JSON parsing error: ${error}`],
            warnings: [],
            complianceScore: 0
          } as StructuredDataItem);
        }
      });

      // Extract Microdata
      const microdataElements = document.querySelectorAll('[itemscope]');
      microdataElements.forEach((element, index) => {
        const itemType = element.getAttribute('itemtype') || 'Unknown';
        const schemaType = itemType.split('/').pop() || 'Unknown';
        
        const data = extractMicrodataFromElement(element as HTMLElement);
        
        items.push({
          format: 'Microdata',
          type: schemaType,
          location: element.closest('head') ? 'head' : 'body',
          selector: generateUniqueSelector(element),
          data,
          valid: validateMicrodata(data, schemaType),
          errors: [],
          warnings: [],
          complianceScore: calculateComplianceScore(data, schemaType)
        } as StructuredDataItem);
      });

      // Extract RDFa (basic detection)
      const rdfaElements = document.querySelectorAll('[typeof]');
      rdfaElements.forEach((element, index) => {
        const typeOf = element.getAttribute('typeof') || 'Unknown';
        
        const data = extractRdfaFromElement(element as HTMLElement);
        
        items.push({
          format: 'RDFa',
          type: typeOf,
          location: element.closest('head') ? 'head' : 'body',
          selector: generateUniqueSelector(element),
          data,
          valid: validateRdfa(data, typeOf),
          errors: [],
          warnings: [],
          complianceScore: calculateComplianceScore(data, typeOf)
        } as StructuredDataItem);
      });

      return items;
    });
  }

  /**
   * Analyze structured data by schema type
   */
  private analyzeSchemaTypes(items: StructuredDataItem[]): SchemaTypeAnalysis[] {
    const typeGroups = items.reduce((groups, item) => {
      if (!groups[item.type]) {
        groups[item.type] = [];
      }
      groups[item.type].push(item);
      return groups;
    }, {} as Record<string, StructuredDataItem[]>);

    return Object.entries(typeGroups).map(([type, typeItems]) => {
      const requiredProps = this.schemaRequiredProperties[type] || [];
      const recommendedProps = this.schemaRecommendedProperties[type] || [];

      // Check required properties across all instances
      const requiredProperties = requiredProps.map(property => {
        const presentInAll = typeItems.every(item => 
          item.data && typeof item.data === 'object' && property in item.data
        );
        const validInAll = typeItems.every(item =>
          item.data && item.data[property] && String(item.data[property]).trim() !== ''
        );

        return {
          property,
          present: presentInAll,
          valid: validInAll,
          value: typeItems[0]?.data?.[property] || undefined
        };
      });

      // Check recommended properties
      const recommendedProperties = recommendedProps.map(({ property, benefit }) => ({
        property,
        present: typeItems.some(item => 
          item.data && typeof item.data === 'object' && property in item.data
        ),
        benefit
      }));

      // Calculate completeness score
      const requiredScore = requiredProperties.length > 0 
        ? (requiredProperties.filter(p => p.present).length / requiredProperties.length) * 70
        : 70;
      const recommendedScore = recommendedProperties.length > 0
        ? (recommendedProperties.filter(p => p.present).length / recommendedProperties.length) * 30
        : 30;
      
      const completenessScore = Math.round(requiredScore + recommendedScore);

      // Collect issues
      const issues: string[] = [];
      requiredProperties.forEach(prop => {
        if (!prop.present) {
          issues.push(`Missing required property: ${prop.property}`);
        } else if (!prop.valid) {
          issues.push(`Invalid value for required property: ${prop.property}`);
        }
      });

      return {
        type,
        count: typeItems.length,
        requiredProperties,
        recommendedProperties,
        completenessScore,
        issues
      };
    });
  }

  /**
   * Analyze rich snippets potential
   */
  private analyzeRichSnippets(items: StructuredDataItem[], schemaTypes: SchemaTypeAnalysis[]): RichSnippetsAnalysis {
    const supportedSnippetTypes = [
      'Article', 'BlogPosting', 'NewsArticle',
      'Product', 'Offer',
      'Recipe',
      'Event',
      'Organization', 'LocalBusiness',
      'Person',
      'Review', 'AggregateRating',
      'VideoObject',
      'FAQPage',
      'HowTo',
      'BreadcrumbList'
    ];

    const foundTypes = items.map(item => item.type);
    const supportedTypes = foundTypes.filter(type => supportedSnippetTypes.includes(type));
    const potentialTypes = supportedSnippetTypes.filter(type => !foundTypes.includes(type));

    // Calculate rich snippets score
    let richSnippetsScore = 0;
    
    if (supportedTypes.length > 0) {
      richSnippetsScore += 40; // Base score for having supported types
      
      // Bonus for complete implementations
      schemaTypes.forEach(schema => {
        if (supportedSnippetTypes.includes(schema.type)) {
          richSnippetsScore += Math.min(15, schema.completenessScore * 0.15);
        }
      });
    }

    const recommendations: string[] = [];
    
    if (supportedTypes.length === 0) {
      recommendations.push('Add structured data for content types that support rich snippets');
      recommendations.push('Consider implementing Article, Product, or Organization markup');
    } else {
      const incompleteTypes = schemaTypes.filter(s => 
        supportedSnippetTypes.includes(s.type) && s.completenessScore < 80
      );
      
      if (incompleteTypes.length > 0) {
        recommendations.push('Complete missing required properties for better rich snippet display');
      }
    }

    potentialTypes.slice(0, 3).forEach(type => {
      recommendations.push(`Consider adding ${type} markup if applicable to your content`);
    });

    return {
      eligible: supportedTypes.length > 0,
      supportedTypes,
      potentialTypes: potentialTypes.slice(0, 5),
      richSnippetsScore: Math.min(100, richSnippetsScore),
      recommendations
    };
  }

  /**
   * Analyze Knowledge Graph readiness
   */
  private analyzeKnowledgeGraph(items: StructuredDataItem[], schemaTypes: SchemaTypeAnalysis[]): KnowledgeGraphAnalysis {
    const organizationSchema = schemaTypes.find(s => s.type === 'Organization');
    const localBusinessSchema = schemaTypes.find(s => s.type === 'LocalBusiness');
    const contentSchemas = schemaTypes.filter(s => 
      ['Article', 'BlogPosting', 'NewsArticle', 'WebPage'].includes(s.type)
    );

    const organization = {
      present: !!organizationSchema,
      completeness: organizationSchema?.completenessScore || 0,
      missingProperties: organizationSchema?.issues.filter(i => i.includes('Missing')).map(i => 
        i.replace('Missing required property: ', '')
      ) || []
    };

    const localBusiness = {
      present: !!localBusinessSchema,
      completeness: localBusinessSchema?.completenessScore || 0,
      missingProperties: localBusinessSchema?.issues.filter(i => i.includes('Missing')).map(i => 
        i.replace('Missing required property: ', '')
      ) || []
    };

    const content = {
      present: contentSchemas.length > 0,
      completeness: contentSchemas.length > 0 
        ? Math.round(contentSchemas.reduce((sum, s) => sum + s.completenessScore, 0) / contentSchemas.length)
        : 0,
      missingProperties: contentSchemas.flatMap(s => 
        s.issues.filter(i => i.includes('Missing')).map(i => 
          i.replace('Missing required property: ', '')
        )
      )
    };

    // Calculate readiness score
    let readinessScore = 0;
    
    if (organization.present) {
      readinessScore += organization.completeness * 0.4;
    }
    
    if (localBusiness.present) {
      readinessScore += localBusiness.completeness * 0.3;
    }
    
    if (content.present) {
      readinessScore += content.completeness * 0.3;
    }
    
    // If no specific schemas, give base score for any structured data
    if (!organization.present && !localBusiness.present && !content.present && items.length > 0) {
      readinessScore = 20;
    }

    return {
      organization,
      localBusiness,
      content,
      readinessScore: Math.round(Math.min(100, readinessScore))
    };
  }

  /**
   * Analyze SEO impact of structured data
   */
  private analyzeSEOImpact(
    items: StructuredDataItem[], 
    schemaTypes: SchemaTypeAnalysis[], 
    richSnippets: RichSnippetsAnalysis
  ): StructuredDataMetrics['seoImpact'] {
    
    let searchVisibilityBoost = 0;
    let clickThroughRateImpact = 0;
    let rankingFactorScore = 0;

    if (items.length > 0) {
      // Base boost for having structured data
      searchVisibilityBoost = 10;
      rankingFactorScore = 15;
      
      if (richSnippets.eligible) {
        searchVisibilityBoost += 25; // Rich snippets significantly increase visibility
        clickThroughRateImpact = 30; // Rich snippets typically improve CTR by 20-30%
      }

      // Additional boosts for specific schema types
      const hasOrganization = schemaTypes.some(s => s.type === 'Organization');
      const hasArticles = schemaTypes.some(s => ['Article', 'BlogPosting'].includes(s.type));
      const hasProducts = schemaTypes.some(s => s.type === 'Product');

      if (hasOrganization) {
        searchVisibilityBoost += 10;
        rankingFactorScore += 10;
      }

      if (hasArticles) {
        searchVisibilityBoost += 15;
        clickThroughRateImpact += 10;
      }

      if (hasProducts) {
        clickThroughRateImpact += 20; // Product rich snippets have high CTR impact
      }

      // Quality bonus
      const avgCompleteness = schemaTypes.length > 0 
        ? schemaTypes.reduce((sum, s) => sum + s.completenessScore, 0) / schemaTypes.length
        : 0;
      
      const qualityBonus = Math.round(avgCompleteness * 0.2);
      searchVisibilityBoost += qualityBonus;
      rankingFactorScore += qualityBonus;
    }

    return {
      searchVisibilityBoost: Math.min(100, searchVisibilityBoost),
      clickThroughRateImpact: Math.min(100, clickThroughRateImpact),
      rankingFactorScore: Math.min(100, rankingFactorScore)
    };
  }

  /**
   * Collect validation issues
   */
  private collectValidationIssues(
    items: StructuredDataItem[], 
    schemaTypes: SchemaTypeAnalysis[]
  ): StructuredDataMetrics['issues'] {
    
    const issues: StructuredDataMetrics['issues'] = [];

    // Issues from invalid items
    items.forEach(item => {
      if (!item.valid) {
        issues.push({
          severity: 'error',
          type: item.type,
          location: item.selector || 'unknown',
          message: 'Invalid structured data format or syntax',
          recommendation: 'Fix JSON-LD syntax errors or microdata structure'
        });
      }

      item.errors.forEach(error => {
        issues.push({
          severity: 'error',
          type: item.type,
          location: item.selector || 'unknown',
          message: error,
          recommendation: 'Fix the syntax error in the structured data'
        });
      });

      item.warnings.forEach(warning => {
        issues.push({
          severity: 'warning',
          type: item.type,
          location: item.selector || 'unknown',
          message: warning,
          recommendation: 'Review and improve the structured data implementation'
        });
      });
    });

    // Issues from schema type analysis
    schemaTypes.forEach(schema => {
      schema.issues.forEach(issue => {
        issues.push({
          severity: issue.includes('required') ? 'error' : 'warning',
          type: schema.type,
          location: 'schema structure',
          message: issue,
          recommendation: issue.includes('Missing') 
            ? `Add the required property to improve ${schema.type} markup`
            : 'Review and fix the property value'
        });
      });
    });

    return issues;
  }

  /**
   * Calculate overall structured data score
   */
  private calculateStructuredDataScore(
    items: StructuredDataItem[],
    schemaTypes: SchemaTypeAnalysis[],
    richSnippets: RichSnippetsAnalysis,
    knowledgeGraph: KnowledgeGraphAnalysis
  ): number {
    
    if (items.length === 0) {
      return 0; // No structured data at all
    }

    const weights = {
      presence: 0.2,        // 20% - Having structured data at all
      validity: 0.25,       // 25% - Data is valid and well-formed
      completeness: 0.25,   // 25% - Required properties are present
      richSnippets: 0.20,   // 20% - Rich snippets eligibility
      knowledgeGraph: 0.10  // 10% - Knowledge Graph readiness
    };

    let score = 0;

    // Presence score (20%)
    score += 100 * weights.presence; // Full points for having structured data

    // Validity score (25%)
    const validItems = items.filter(item => item.valid);
    const validityScore = items.length > 0 ? (validItems.length / items.length) * 100 : 0;
    score += validityScore * weights.validity;

    // Completeness score (25%)
    const avgCompleteness = schemaTypes.length > 0 
      ? schemaTypes.reduce((sum, s) => sum + s.completenessScore, 0) / schemaTypes.length
      : 0;
    score += avgCompleteness * weights.completeness;

    // Rich snippets score (20%)
    score += richSnippets.richSnippetsScore * weights.richSnippets;

    // Knowledge Graph score (10%)
    score += knowledgeGraph.readinessScore * weights.knowledgeGraph;

    return Math.round(Math.min(100, score));
  }

  /**
   * Calculate overall score for BaseAnalyzer interface
   */
  private calculateOverallScore(structuredData: StructuredDataMetrics): number {
    return structuredData.overallScore;
  }

  /**
   * Generate structured data specific recommendations
   */
  private generateStructuredDataRecommendations(
    items: StructuredDataItem[],
    schemaTypes: SchemaTypeAnalysis[],
    richSnippets: RichSnippetsAnalysis,
    knowledgeGraph: KnowledgeGraphAnalysis,
    issues: StructuredDataMetrics['issues']
  ): StructuredDataMetrics['recommendations'] {
    
    const recommendations: StructuredDataMetrics['recommendations'] = [];

    // Basic structured data recommendations
    if (items.length === 0) {
      recommendations.push({
        priority: 'high',
        category: 'Structured Data Implementation',
        issue: 'No structured data found on the page',
        recommendation: 'Add JSON-LD structured data to improve search engine understanding',
        impact: 'Better search result appearance and potential rich snippets',
        implementation: 'Add JSON-LD script tags in the page head with relevant schema.org markup'
      });
    }

    // Rich snippets recommendations
    if (!richSnippets.eligible && items.length > 0) {
      recommendations.push({
        priority: 'medium',
        category: 'Rich Snippets',
        issue: 'Current structured data does not support rich snippets',
        recommendation: 'Implement schema types that support rich snippets (Article, Product, Organization, etc.)',
        impact: 'Enhanced search result appearance with rich snippets',
        implementation: 'Add appropriate schema.org types based on your content'
      });
    }

    // Completeness recommendations
    const incompleteSchemas = schemaTypes.filter(s => s.completenessScore < 80);
    if (incompleteSchemas.length > 0) {
      recommendations.push({
        priority: 'medium',
        category: 'Schema Completeness',
        issue: `${incompleteSchemas.length} schema types are missing required properties`,
        recommendation: 'Complete missing required and recommended properties for better compliance',
        impact: 'Higher chance of rich snippet qualification and better search understanding',
        implementation: 'Review schema requirements and add missing properties to existing markup'
      });
    }

    // Knowledge Graph recommendations
    if (knowledgeGraph.readinessScore < 60) {
      if (!knowledgeGraph.organization.present) {
        recommendations.push({
          priority: 'medium',
          category: 'Knowledge Graph',
          issue: 'Missing Organization schema for Knowledge Graph eligibility',
          recommendation: 'Add Organization schema with complete business information',
          impact: 'Eligibility for Knowledge Graph panels in search results',
          implementation: 'Implement Organization schema with name, logo, url, and contact information'
        });
      }
    }

    // Validation error recommendations
    const errorIssues = issues.filter(i => i.severity === 'error');
    if (errorIssues.length > 0) {
      recommendations.push({
        priority: 'high',
        category: 'Data Validation',
        issue: `${errorIssues.length} validation errors found in structured data`,
        recommendation: 'Fix syntax errors and invalid schema properties',
        impact: 'Proper recognition and processing by search engines',
        implementation: 'Use Google\'s Rich Results Test tool to identify and fix validation errors'
      });
    }

    // Format diversity recommendations
    if (items.every(item => item.format === 'Microdata') || items.every(item => item.format === 'RDFa')) {
      recommendations.push({
        priority: 'low',
        category: 'Implementation Format',
        issue: 'Consider using JSON-LD for better maintainability',
        recommendation: 'Migrate to JSON-LD format for easier management and better Google support',
        impact: 'Easier maintenance and better search engine support',
        implementation: 'Convert existing Microdata/RDFa to JSON-LD script tags'
      });
    }

    return recommendations;
  }

  /**
   * Generate general recommendations for BaseAnalyzer interface
   */
  private generateRecommendations(structuredData: StructuredDataMetrics): BaseRecommendation[] {
    return structuredData.recommendations.map((rec, index) => ({
      id: `structured-data-${index}`,
      priority: rec.priority as any,
      category: rec.category,
      issue: rec.issue,
      recommendation: rec.recommendation,
      impact: rec.impact,
      effort: this.estimateEffort(rec.priority),
      scoreImprovement: this.estimateScoreImprovement(rec.priority)
    }));
  }

  /**
   * Get default rich snippets analysis when not requested
   */
  private getDefaultRichSnippetsAnalysis(): RichSnippetsAnalysis {
    return {
      eligible: false,
      supportedTypes: [],
      potentialTypes: [],
      richSnippetsScore: 0,
      recommendations: ['Enable rich snippets analysis for detailed recommendations']
    };
  }

  /**
   * Get default knowledge graph analysis when not requested
   */
  private getDefaultKnowledgeGraphAnalysis(): KnowledgeGraphAnalysis {
    return {
      organization: { present: false, completeness: 0, missingProperties: [] },
      localBusiness: { present: false, completeness: 0, missingProperties: [] },
      content: { present: false, completeness: 0, missingProperties: [] },
      readinessScore: 0
    };
  }

  /**
   * Estimate implementation effort based on priority
   */
  private estimateEffort(priority: string): number {
    switch (priority) {
      case 'high': return 6;
      case 'medium': return 4;
      case 'low': return 2;
      default: return 3;
    }
  }

  /**
   * Estimate score improvement based on priority
   */
  private estimateScoreImprovement(priority: string): number {
    switch (priority) {
      case 'high': return 20;
      case 'medium': return 12;
      case 'low': return 6;
      default: return 10;
    }
  }
}