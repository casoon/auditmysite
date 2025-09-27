# Changelog

All notable changes to AuditMySite will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0-alpha.2] - 2025-09-24

### 🎉 Added
- **WordPress Sitemap Index Support** - Full support for WordPress `wp-sitemap.xml` files with recursive sub-sitemap processing
- **Enhanced Sitemap Parser** - Automatically detects and processes sitemap index files with up to 10 sub-sitemaps
- **Unified Event System** - Modern event handling architecture replacing deprecated callback system
- **Professional CLI Output** - Clean, deprecation-warning-free interface suitable for production use

### 🔧 Changed
- **Event System Migration** - Replaced deprecated `eventCallbacks` with `setUnifiedEventCallbacks()`
- **Improved Error Handling** - Better error messages for sitemap parsing failures
- **Enhanced Resource Management** - More efficient memory usage and cleanup
- **Performance Optimization** - Event-driven parallel processing for better throughput

### 🐛 Fixed
- **WordPress Sitemap Parsing** - Fixed "No URLs found in sitemap" error for WordPress sites
- **Deprecation Warnings** - Eliminated all CLI deprecation warnings for cleaner output
- **Global Installation Sync** - Fixed issue where global CLI didn't reflect latest source changes
- **Memory Leaks** - Improved resource cleanup and browser context management

### ✅ Tested
- **WordPress Sites** - Verified with complex sitemap structures (125+ URLs from 6 sub-sitemaps)
- **Standard Sitemaps** - Maintained compatibility with regular XML sitemaps (527+ URLs)
- **Performance Benchmarks** - Achieved 30.0 pages/minute processing speed
- **Real-World Scenarios** - Tested with production websites and CI/CD environments

### 📈 Performance
- **30.0 pages/minute** - Processing speed for standard sitemaps
- **6.0 pages/minute** - Processing speed for comprehensive analysis
- **100% Success Rate** - Reliable URL discovery and processing
- **Reduced Memory Footprint** - More efficient resource utilization

## [2.0.0-alpha.1] - Previous Release

### Added
- Initial alpha release with comprehensive accessibility testing
- Core Web Vitals performance monitoring
- SEO analysis capabilities
- Content weight optimization insights
- Professional HTML report generation
- CLI interface with expert mode
- API server functionality
- Browser pooling and parallel processing

---

## Release Numbering

- **Major.Minor.Patch-PreRelease.Build**
- **2.0.0** - Major rewrite with enhanced architecture
- **alpha** - Pre-release for early testing
- **Build number** - Incremental improvements within alpha phase

## Support

For questions, bugs, or feature requests:
- **GitHub Issues**: https://github.com/casoon/AuditMySite/issues
- **Email**: joern.seidel@casoon.de
- **Documentation**: https://github.com/casoon/AuditMySite#readme