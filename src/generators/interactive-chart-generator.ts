/**
 * Interactive Chart Generator for Enhanced Reports
 */

export interface ChartData {
    labels: string[];
    datasets: {
        label: string;
        data: number[];
        backgroundColor?: string | string[];
        borderColor?: string | string[];
        borderWidth?: number;
        fill?: boolean;
    }[];
}

export interface ChartConfig {
    type: 'bar' | 'line' | 'doughnut' | 'radar' | 'scatter' | 'bubble';
    title: string;
    subtitle?: string;
    width?: number;
    height?: number;
    responsive?: boolean;
    options?: any;
}

export class InteractiveChartGenerator {
    private chartCounter = 0;

    /**
     * Generate Chart.js configuration
     */
    generateChartJS(data: ChartData, config: ChartConfig): string {
        const chartId = `chart-${++this.chartCounter}`;
        
        const chartConfig = {
            type: config.type,
            data,
            options: {
                responsive: config.responsive !== false,
                maintainAspectRatio: false,
                plugins: {
                    title: {
                        display: true,
                        text: config.title,
                        font: { size: 16, weight: 'bold' }
                    },
                    subtitle: config.subtitle ? {
                        display: true,
                        text: config.subtitle,
                        font: { size: 12 }
                    } : undefined,
                    legend: {
                        display: true,
                        position: 'top'
                    },
                    tooltip: {
                        enabled: true,
                        mode: 'index',
                        intersect: false
                    }
                },
                scales: config.type === 'radar' || config.type === 'doughnut' ? undefined : {
                    y: {
                        beginAtZero: true,
                        grid: {
                            color: 'rgba(0,0,0,0.1)'
                        }
                    },
                    x: {
                        grid: {
                            color: 'rgba(0,0,0,0.1)'
                        }
                    }
                },
                ...config.options
            }
        };

        return `
            <div class="chart-container" style="position: relative; height: ${config.height || 400}px; width: ${config.width || '100%'};">
                <canvas id="${chartId}"></canvas>
            </div>
            <script>
                const ctx_${chartId} = document.getElementById('${chartId}').getContext('2d');
                const chart_${chartId} = new Chart(ctx_${chartId}, ${JSON.stringify(chartConfig, null, 2)});
            </script>
        `;
    }

    /**
     * Generate performance metrics chart
     */
    generatePerformanceChart(performanceData: any): string {
        const data: ChartData = {
            labels: ['LCP', 'FID', 'CLS', 'TTFB', 'Speed Index'],
            datasets: [{
                label: 'Performance Score',
                data: [
                    performanceData.lcp?.score || 0,
                    performanceData.fid?.score || 0,
                    performanceData.cls?.score || 0,
                    performanceData.ttfb?.score || 0,
                    performanceData.speedIndex?.score || 0
                ],
                backgroundColor: [
                    this.getScoreColor(performanceData.lcp?.score || 0),
                    this.getScoreColor(performanceData.fid?.score || 0),
                    this.getScoreColor(performanceData.cls?.score || 0),
                    this.getScoreColor(performanceData.ttfb?.score || 0),
                    this.getScoreColor(performanceData.speedIndex?.score || 0)
                ],
                borderWidth: 1
            }]
        };

        const config: ChartConfig = {
            type: 'bar',
            title: 'Core Web Vitals Performance',
            subtitle: 'Higher scores are better',
            height: 300
        };

        return this.generateChartJS(data, config);
    }

    /**
     * Generate accessibility issues breakdown chart
     */
    generateAccessibilityChart(accessibilityData: any): string {
        const errorCount = accessibilityData.errors?.length || 0;
        const warningCount = accessibilityData.warnings?.length || 0;
        const passCount = accessibilityData.passes?.length || 0;

        const data: ChartData = {
            labels: ['Errors', 'Warnings', 'Passes'],
            datasets: [{
                label: 'Accessibility Issues',
                data: [errorCount, warningCount, passCount],
                backgroundColor: ['#ef4444', '#f97316', '#10b981'],
                borderWidth: 2,
                borderColor: '#fff'
            }]
        };

        const config: ChartConfig = {
            type: 'doughnut',
            title: 'Accessibility Issues Breakdown',
            height: 300
        };

        return this.generateChartJS(data, config);
    }

    /**
     * Generate SEO metrics radar chart
     */
    generateSEORadarChart(seoData: any): string {
        const data: ChartData = {
            labels: [
                'Meta Tags',
                'Headings',
                'Images Alt Text',
                'Internal Links',
                'Page Speed',
                'Mobile Friendly',
                'Schema Markup',
                'Social Media'
            ],
            datasets: [{
                label: 'SEO Score',
                data: [
                    seoData.metaTags?.score || 0,
                    seoData.headings?.score || 0,
                    seoData.images?.score || 0,
                    seoData.links?.score || 0,
                    seoData.performance?.score || 0,
                    seoData.mobile?.score || 0,
                    seoData.schema?.score || 0,
                    seoData.social?.score || 0
                ],
                backgroundColor: 'rgba(59, 130, 246, 0.2)',
                borderColor: 'rgba(59, 130, 246, 1)',
                borderWidth: 2,
                fill: true
            }]
        };

        const config: ChartConfig = {
            type: 'radar',
            title: 'SEO Analysis Radar',
            subtitle: 'Comprehensive SEO metrics overview',
            height: 350
        };

        return this.generateChartJS(data, config);
    }

    /**
     * Generate mobile-friendliness comparison chart
     */
    generateMobileComparisonChart(mobileData: any): string {
        const desktopData = mobileData.desktopComparison?.desktop || {};
        const mobileDataPoints = mobileData.desktopComparison?.mobile || {};

        const data: ChartData = {
            labels: ['Touch Targets', 'Font Size', 'Performance', 'Navigation', 'Overall'],
            datasets: [
                {
                    label: 'Desktop',
                    data: [
                        desktopData.touchTargets?.score || 0,
                        desktopData.typography?.score || 0,
                        desktopData.performance?.score || 0,
                        desktopData.navigation?.score || 0,
                        desktopData.usabilityScore || 0
                    ],
                    backgroundColor: 'rgba(34, 197, 94, 0.7)',
                    borderColor: 'rgba(34, 197, 94, 1)',
                    borderWidth: 2
                },
                {
                    label: 'Mobile',
                    data: [
                        mobileDataPoints.touchTargets?.score || 0,
                        mobileDataPoints.typography?.score || 0,
                        mobileDataPoints.performance?.score || 0,
                        mobileDataPoints.navigation?.score || 0,
                        mobileDataPoints.usabilityScore || 0
                    ],
                    backgroundColor: 'rgba(168, 85, 247, 0.7)',
                    borderColor: 'rgba(168, 85, 247, 1)',
                    borderWidth: 2
                }
            ]
        };

        const config: ChartConfig = {
            type: 'bar',
            title: 'Desktop vs Mobile Comparison',
            subtitle: 'Usability scores across different metrics',
            height: 350
        };

        return this.generateChartJS(data, config);
    }

    /**
     * Generate content weight breakdown chart
     */
    generateContentWeightChart(contentData: any): string {
        const weights = contentData.resourceBreakdown || {};
        
        const data: ChartData = {
            labels: ['HTML', 'CSS', 'JavaScript', 'Images', 'Fonts', 'Other'],
            datasets: [{
                label: 'Size (KB)',
                data: [
                    Math.round((weights.html || 0) / 1024),
                    Math.round((weights.css || 0) / 1024),
                    Math.round((weights.javascript || 0) / 1024),
                    Math.round((weights.images || 0) / 1024),
                    Math.round((weights.fonts || 0) / 1024),
                    Math.round((weights.other || 0) / 1024)
                ],
                backgroundColor: [
                    '#f97316', // HTML - orange
                    '#3b82f6', // CSS - blue
                    '#eab308', // JS - yellow
                    '#10b981', // Images - green
                    '#8b5cf6', // Fonts - purple
                    '#6b7280'  // Other - gray
                ],
                borderWidth: 1
            }]
        };

        const config: ChartConfig = {
            type: 'doughnut',
            title: 'Content Weight Breakdown',
            subtitle: 'Resource sizes in KB',
            height: 300
        };

        return this.generateChartJS(data, config);
    }

    /**
     * Generate trend line chart for historical data
     */
    generateTrendChart(historicalData: Array<{ date: string; score: number; }>): string {
        const data: ChartData = {
            labels: historicalData.map(d => d.date),
        datasets: [{
            label: 'Performance Score',
            data: historicalData.map(d => d.score),
            borderColor: 'rgba(59, 130, 246, 1)',
            backgroundColor: 'rgba(59, 130, 246, 0.1)',
            borderWidth: 3,
            fill: true
        }]
        };

        const config: ChartConfig = {
            type: 'line',
            title: 'Performance Trend',
            subtitle: 'Score changes over time',
            height: 250
        };

        return this.generateChartJS(data, config);
    }

    /**
     * Generate comprehensive dashboard with multiple charts
     */
    generateDashboard(reportData: any): string {
        let dashboard = `
            <div class="charts-dashboard">
                <style>
                    .charts-dashboard {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
                        gap: 20px;
                        margin: 20px 0;
                    }
                    .chart-container {
                        background: #fff;
                        border-radius: 8px;
                        padding: 20px;
                        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                        border: 1px solid #e5e7eb;
                    }
                    .chart-row {
                        grid-column: 1 / -1;
                    }
                    @media (max-width: 768px) {
                        .charts-dashboard {
                            grid-template-columns: 1fr;
                        }
                    }
                </style>
        `;

        // Performance chart
        if (reportData.performance) {
            dashboard += this.generatePerformanceChart(reportData.performance);
        }

        // Accessibility chart
        if (reportData.accessibility) {
            dashboard += this.generateAccessibilityChart(reportData.accessibility);
        }

        // SEO radar chart
        if (reportData.seo) {
            dashboard += `<div class="chart-row">${this.generateSEORadarChart(reportData.seo)}</div>`;
        }

        // Mobile comparison chart
        if (reportData.mobileFriendliness?.desktopComparison) {
            dashboard += this.generateMobileComparisonChart(reportData.mobileFriendliness);
        }

        // Content weight chart
        if (reportData.contentWeight) {
            dashboard += this.generateContentWeightChart(reportData.contentWeight);
        }

        dashboard += '</div>';

        return dashboard;
    }

    /**
     * Get chart.js library inclusion
     */
    getChartJSLibrary(): string {
        return `
            <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.js"></script>
            <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns@2.0.0/dist/chartjs-adapter-date-fns.bundle.min.js"></script>
        `;
    }

    /**
     * Generate export functionality
     */
    generateExportFunctionality(): string {
        return `
            <script>
                function exportChart(chartId, filename) {
                    const canvas = document.getElementById(chartId);
                    const url = canvas.toDataURL('image/png');
                    const link = document.createElement('a');
                    link.download = filename + '.png';
                    link.href = url;
                    link.click();
                }
                
                function exportAllCharts() {
                    const charts = document.querySelectorAll('canvas[id^="chart-"]');
                    charts.forEach((canvas, index) => {
                        setTimeout(() => {
                            const url = canvas.toDataURL('image/png');
                            const link = document.createElement('a');
                            link.download = \`chart-\${index + 1}.png\`;
                            link.href = url;
                            link.click();
                        }, index * 500);
                    });
                }
            </script>
            
            <div style="margin: 20px 0; text-align: center;">
                <button onclick="exportAllCharts()" 
                        style="background: #3b82f6; color: white; border: none; padding: 10px 20px; border-radius: 6px; cursor: pointer; font-size: 14px;">
                    📊 Export All Charts
                </button>
            </div>
        `;
    }

    /**
     * Get color based on score (0-100)
     */
    private getScoreColor(score: number): string {
        if (score >= 90) return '#10b981'; // Green
        if (score >= 70) return '#f97316'; // Orange
        return '#ef4444'; // Red
    }

    /**
     * Generate print-friendly version
     */
    generatePrintVersion(reportData: any): string {
        return `
            <style media="print">
                .chart-container {
                    break-inside: avoid;
                    page-break-inside: avoid;
                }
                .charts-dashboard {
                    display: block;
                }
            </style>
        `;
    }
}

export default InteractiveChartGenerator;
