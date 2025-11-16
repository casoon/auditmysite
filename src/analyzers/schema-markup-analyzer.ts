/**
 * Schema Markup Analysis and Validation
 */

import { Page } from 'playwright';

export interface SchemaMarkup {
    type: string;
    context?: string;
    properties: Record<string, any>;
    errors: string[];
    warnings: string[];
    isValid: boolean;
    location: {
        selector: string;
        line?: number;
        column?: number;
    };
}

export interface SchemaAnalysisResult {
    hasStructuredData: boolean;
    totalSchemas: number;
    validSchemas: number;
    invalidSchemas: number;
    schemas: SchemaMarkup[];
    recommendations: string[];
    score: number;
    grade: 'A' | 'B' | 'C' | 'D' | 'F';
    supportedTypes: {
        found: string[];
        recommended: string[];
        missing: string[];
    };
    richSnippetOpportunities: Array<{
        type: string;
        description: string;
        priority: 'high' | 'medium' | 'low';
        implementation: string;
    }>;
}

export class SchemaMarkupAnalyzer {
    private commonSchemaTypes = [
        'Organization',
        'LocalBusiness',
        'Product',
        'Article',
        'BlogPosting',
        'NewsArticle',
        'Recipe',
        'Event',
        'Person',
        'WebPage',
        'WebSite',
        'BreadcrumbList',
        'Review',
        'Rating',
        'FAQ',
        'HowTo',
        'Service',
        'ContactPoint'
    ];

    private requiredProperties: Record<string, string[]> = {
        'Organization': ['name', 'url'],
        'LocalBusiness': ['name', 'address', 'telephone'],
        'Product': ['name', 'image', 'description'],
        'Article': ['headline', 'author', 'datePublished'],
        'BlogPosting': ['headline', 'author', 'datePublished'],
        'Recipe': ['name', 'author', 'recipeIngredient', 'recipeInstructions'],
        'Event': ['name', 'startDate', 'location'],
        'Person': ['name'],
        'Review': ['reviewBody', 'author', 'itemReviewed'],
        'FAQ': ['mainEntity'],
        'HowTo': ['name', 'step']
    };

    /**
     * Analyze schema markup on a page
     */
    async analyze(page: Page): Promise<SchemaAnalysisResult> {
        console.log('🏷️ Analyzing schema markup...');

        const schemas = await this.extractSchemaMarkup(page);
        const validatedSchemas = schemas.map(schema => this.validateSchema(schema));
        
        const validSchemas = validatedSchemas.filter(s => s.isValid);
        const invalidSchemas = validatedSchemas.filter(s => !s.isValid);
        
        const supportedTypes = this.analyzeSupportedTypes(validatedSchemas);
        const richSnippetOpportunities = await this.identifyRichSnippetOpportunities(page, validatedSchemas);
        const recommendations = this.generateRecommendations(validatedSchemas, supportedTypes, richSnippetOpportunities);
        
        const score = this.calculateScore(validatedSchemas, supportedTypes);
        const grade = this.calculateGrade(score);

        return {
            hasStructuredData: validatedSchemas.length > 0,
            totalSchemas: validatedSchemas.length,
            validSchemas: validSchemas.length,
            invalidSchemas: invalidSchemas.length,
            schemas: validatedSchemas,
            recommendations,
            score,
            grade,
            supportedTypes,
            richSnippetOpportunities
        };
    }

    /**
     * Extract schema markup from page
     */
    private async extractSchemaMarkup(page: Page): Promise<SchemaMarkup[]> {
        return await page.evaluate(() => {
            const schemas: SchemaMarkup[] = [];
            
            // Extract JSON-LD schemas
            const jsonLdScripts = document.querySelectorAll('script[type="application/ld+json"]');
            jsonLdScripts.forEach((script, index) => {
                try {
                    const content = script.textContent?.trim();
                    if (content) {
                        const data = JSON.parse(content);
                        const schemaArray = Array.isArray(data) ? data : [data];
                        
                        schemaArray.forEach((schemaData, subIndex) => {
                            schemas.push({
                                type: schemaData['@type'] || 'Unknown',
                                context: schemaData['@context'] || undefined,
                                properties: schemaData,
                                errors: [],
                                warnings: [],
                                isValid: true,
                                location: {
                                    selector: `script[type="application/ld+json"]:nth-child(${index + 1})`,
                                    line: undefined,
                                    column: undefined
                                }
                            });
                        });
                    }
                } catch (error: any) {
                    schemas.push({
                        type: 'Invalid JSON-LD',
                        properties: {},
                        errors: [`JSON parsing error: ${error?.message || 'Unknown error'}`],
                        warnings: [],
                        isValid: false,
                        location: {
                            selector: `script[type="application/ld+json"]:nth-child(${index + 1})`
                        }
                    });
                }
            });

            // Extract Microdata
            const microdataElements = document.querySelectorAll('[itemscope]');
            microdataElements.forEach((element, index) => {
                const itemType = element.getAttribute('itemtype');
                const properties: Record<string, any> = {};
                
                // Extract properties
                const propertyElements = element.querySelectorAll('[itemprop]');
                propertyElements.forEach(propElement => {
                    const propName = propElement.getAttribute('itemprop');
                    const propValue = propElement.getAttribute('content') || 
                                    propElement.textContent?.trim() || 
                                    propElement.getAttribute('href') ||
                                    propElement.getAttribute('src');
                    
                    if (propName && propValue) {
                        if (properties[propName]) {
                            if (Array.isArray(properties[propName])) {
                                properties[propName].push(propValue);
                            } else {
                                properties[propName] = [properties[propName], propValue];
                            }
                        } else {
                            properties[propName] = propValue;
                        }
                    }
                });

                if (itemType) {
                    const typeName = itemType.split('/').pop() || itemType;
                    schemas.push({
                        type: typeName,
                        properties,
                        errors: [],
                        warnings: [],
                        isValid: true,
                        location: {
                            selector: `[itemscope]:nth-child(${index + 1})`
                        }
                    });
                }
            });

            // Extract RDFa (basic support)
            const rdfaElements = document.querySelectorAll('[typeof]');
            rdfaElements.forEach((element, index) => {
                const typeOf = element.getAttribute('typeof');
                const properties: Record<string, any> = {};
                
                // Basic RDFa property extraction
                const propElements = element.querySelectorAll('[property]');
                propElements.forEach(propElement => {
                    const propName = propElement.getAttribute('property');
                    const propValue = propElement.getAttribute('content') || 
                                    propElement.textContent?.trim();
                    
                    if (propName && propValue) {
                        properties[propName] = propValue;
                    }
                });

                if (typeOf) {
                    schemas.push({
                        type: typeOf,
                        properties,
                        errors: [],
                        warnings: [],
                        isValid: true,
                        location: {
                            selector: `[typeof]:nth-child(${index + 1})`
                        }
                    });
                }
            });

            return schemas;
        });
    }

    /**
     * Validate a schema against Schema.org requirements
     */
    private validateSchema(schema: SchemaMarkup): SchemaMarkup {
        const errors: string[] = [...schema.errors];
        const warnings: string[] = [...schema.warnings];

        // Check for required context
        if (!schema.context && schema.type !== 'Invalid JSON-LD') {
            warnings.push('Missing @context property (recommended: "https://schema.org")');
        }

        // Validate against required properties
        const requiredProps = this.requiredProperties[schema.type];
        if (requiredProps) {
            for (const prop of requiredProps) {
                if (!schema.properties[prop]) {
                    errors.push(`Missing required property: ${prop}`);
                }
            }
        }

        // Validate common properties
        if (schema.properties.url && !this.isValidUrl(schema.properties.url)) {
            errors.push('Invalid URL format');
        }

        if (schema.properties.email && !this.isValidEmail(schema.properties.email)) {
            errors.push('Invalid email format');
        }

        if (schema.properties.telephone && !this.isValidPhoneNumber(schema.properties.telephone)) {
            warnings.push('Phone number format may not be optimal');
        }

        // Validate date formats
        const dateFields = ['datePublished', 'dateModified', 'startDate', 'endDate'];
        for (const field of dateFields) {
            if (schema.properties[field] && !this.isValidDate(schema.properties[field])) {
                errors.push(`Invalid date format for ${field} (use ISO 8601)`);
            }
        }

        // Validate image properties
        if (schema.properties.image) {
            const images = Array.isArray(schema.properties.image) ? schema.properties.image : [schema.properties.image];
            for (const image of images) {
                if (typeof image === 'string' && !this.isValidUrl(image)) {
                    errors.push('Invalid image URL');
                } else if (typeof image === 'object' && !image.url) {
                    errors.push('Image object missing URL property');
                }
            }
        }

        return {
            ...schema,
            errors,
            warnings,
            isValid: errors.length === 0
        };
    }

    /**
     * Analyze supported schema types
     */
    private analyzeSupportedTypes(schemas: SchemaMarkup[]): SchemaAnalysisResult['supportedTypes'] {
        const found = [...new Set(schemas.filter(s => s.isValid).map(s => s.type))];
        const missing = this.commonSchemaTypes.filter(type => !found.includes(type));
        
        // Recommend based on page content analysis
        const recommended = missing.filter(type => {
            // This would be enhanced with actual page content analysis
            return ['Organization', 'WebPage', 'BreadcrumbList'].includes(type);
        });

        return { found, recommended, missing };
    }

    /**
     * Identify rich snippet opportunities
     */
    private async identifyRichSnippetOpportunities(page: Page, schemas: SchemaMarkup[]): Promise<SchemaAnalysisResult['richSnippetOpportunities']> {
        const opportunities: SchemaAnalysisResult['richSnippetOpportunities'] = [];

        const pageContent = await page.evaluate(() => {
            return {
                hasProducts: document.querySelectorAll('[class*="product"], .product-item, .product-card').length > 0,
                hasReviews: document.querySelectorAll('[class*="review"], .review, .rating').length > 0,
                hasRecipes: document.querySelectorAll('[class*="recipe"], .recipe').length > 0,
                hasEvents: document.querySelectorAll('[class*="event"], .event').length > 0,
                hasFAQ: document.querySelectorAll('[class*="faq"], .faq, .accordion').length > 0,
                hasHowTo: document.querySelectorAll('[class*="tutorial"], [class*="guide"], .step').length > 0,
                hasArticles: document.querySelectorAll('article, [class*="post"], [class*="article"]').length > 0,
                hasBreadcrumbs: document.querySelectorAll('[class*="breadcrumb"], .breadcrumb').length > 0
            };
        });

        const existingTypes = schemas.map(s => s.type);

        if (pageContent.hasProducts && !existingTypes.includes('Product')) {
            opportunities.push({
                type: 'Product',
                description: 'Add Product schema to enable rich snippets with price, availability, and ratings',
                priority: 'high',
                implementation: 'Add JSON-LD with @type: "Product" including name, image, description, offers'
            });
        }

        if (pageContent.hasReviews && !existingTypes.includes('Review')) {
            opportunities.push({
                type: 'Review',
                description: 'Add Review schema to display star ratings in search results',
                priority: 'high',
                implementation: 'Add JSON-LD with @type: "Review" including reviewBody, author, reviewRating'
            });
        }

        if (pageContent.hasRecipes && !existingTypes.includes('Recipe')) {
            opportunities.push({
                type: 'Recipe',
                description: 'Add Recipe schema for enhanced recipe search results with cooking time and ingredients',
                priority: 'medium',
                implementation: 'Add JSON-LD with @type: "Recipe" including ingredients, instructions, nutrition'
            });
        }

        if (pageContent.hasFAQ && !existingTypes.includes('FAQPage')) {
            opportunities.push({
                type: 'FAQPage',
                description: 'Add FAQ schema to display questions and answers directly in search results',
                priority: 'medium',
                implementation: 'Add JSON-LD with @type: "FAQPage" and mainEntity array of questions'
            });
        }

        if (pageContent.hasHowTo && !existingTypes.includes('HowTo')) {
            opportunities.push({
                type: 'HowTo',
                description: 'Add HowTo schema for step-by-step instruction rich snippets',
                priority: 'medium',
                implementation: 'Add JSON-LD with @type: "HowTo" including step array with instructions'
            });
        }

        if (pageContent.hasBreadcrumbs && !existingTypes.includes('BreadcrumbList')) {
            opportunities.push({
                type: 'BreadcrumbList',
                description: 'Add Breadcrumb schema to display navigation path in search results',
                priority: 'low',
                implementation: 'Add JSON-LD with @type: "BreadcrumbList" and itemListElement array'
            });
        }

        return opportunities;
    }

    /**
     * Generate recommendations
     */
    private generateRecommendations(
        schemas: SchemaMarkup[], 
        supportedTypes: SchemaAnalysisResult['supportedTypes'],
        opportunities: SchemaAnalysisResult['richSnippetOpportunities']
    ): string[] {
        const recommendations: string[] = [];

        if (schemas.length === 0) {
            recommendations.push('Add structured data markup to improve SEO and enable rich snippets');
            recommendations.push('Start with basic Organization or WebPage schema markup');
        }

        const invalidSchemas = schemas.filter(s => !s.isValid);
        if (invalidSchemas.length > 0) {
            recommendations.push(`Fix ${invalidSchemas.length} invalid schema markup(s) to ensure proper indexing`);
        }

        const schemasWithWarnings = schemas.filter(s => s.warnings.length > 0);
        if (schemasWithWarnings.length > 0) {
            recommendations.push('Address schema warnings to improve markup quality');
        }

        if (supportedTypes.recommended.length > 0) {
            recommendations.push(`Consider adding ${supportedTypes.recommended.slice(0, 3).join(', ')} schema types`);
        }

        const highPriorityOpportunities = opportunities.filter(o => o.priority === 'high');
        if (highPriorityOpportunities.length > 0) {
            recommendations.push(`Implement ${highPriorityOpportunities[0].type} schema for enhanced search visibility`);
        }

        if (schemas.some(s => s.context !== 'https://schema.org')) {
            recommendations.push('Use "https://schema.org" as @context for better compatibility');
        }

        return recommendations.slice(0, 5);
    }

    /**
     * Calculate schema score
     */
    private calculateScore(schemas: SchemaMarkup[], supportedTypes: SchemaAnalysisResult['supportedTypes']): number {
        if (schemas.length === 0) return 0;

        let score = 0;

        // Base score for having schemas
        score += Math.min(schemas.length * 10, 40);

        // Bonus for valid schemas
        const validSchemas = schemas.filter(s => s.isValid);
        score += (validSchemas.length / schemas.length) * 30;

        // Bonus for variety of schema types
        score += Math.min(supportedTypes.found.length * 5, 20);

        // Penalty for errors
        const totalErrors = schemas.reduce((sum, s) => sum + s.errors.length, 0);
        score -= Math.min(totalErrors * 5, 30);

        // Bonus for common important schemas
        const importantSchemas = ['Organization', 'WebPage', 'BreadcrumbList'];
        const hasImportantSchemas = importantSchemas.filter(type => supportedTypes.found.includes(type)).length;
        score += hasImportantSchemas * 3;

        return Math.max(0, Math.min(100, Math.round(score)));
    }

    /**
     * Calculate grade based on score
     */
    private calculateGrade(score: number): 'A' | 'B' | 'C' | 'D' | 'F' {
        if (score >= 90) return 'A';
        if (score >= 80) return 'B';
        if (score >= 70) return 'C';
        if (score >= 60) return 'D';
        return 'F';
    }

    // Utility validation methods
    private isValidUrl(url: string): boolean {
        try {
            new URL(url);
            return true;
        } catch {
            return false;
        }
    }

    private isValidEmail(email: string): boolean {
        return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
    }

    private isValidPhoneNumber(phone: string): boolean {
        // Basic phone validation - could be enhanced
        return /^[+]?[\d\s\-()]{10,}$/.test(phone);
    }

    private isValidDate(date: string): boolean {
        // ISO 8601 date validation
        const isoDateRegex = /^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}:\d{2}(\.\d{3})?Z?)?$/;
        return isoDateRegex.test(date) && !isNaN(Date.parse(date));
    }
}

export default SchemaMarkupAnalyzer;
