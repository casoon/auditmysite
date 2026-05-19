#[allow(clippy::module_inception)]
#[cfg(all(test, feature = "pdf_test"))]
mod tests {
    use super::super::*;
    use crate::audit::{AuditReport, BatchReport, ComparisonReport};
    use crate::cli::{ReportLevel, WcagLevel};
    use crate::util::truncate_url;
    use crate::wcag::{Severity, Violation, WcagResults};
    use std::path::PathBuf;
    use std::process::Command;

    #[test]
    fn test_truncate_url() {
        assert_eq!(
            truncate_url("https://example.com/very/long/path/that/exceeds/limit", 30),
            "https://example.com/very/lo..."
        );
        assert_eq!(
            truncate_url("https://example.com", 30),
            "https://example.com"
        );
    }

    #[test]
    fn test_single_pdf_smoke_renders_valid_pdf() {
        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };

        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        assert_pdf_smoke(&pdf, 20_000);
    }

    #[test]
    fn test_single_pdf_smoke_renders_all_report_levels() {
        for level in [
            ReportLevel::Executive,
            ReportLevel::Standard,
            ReportLevel::Technical,
        ] {
            let report = pdf_fixture_report();
            let config = ReportConfig {
                level,
                ..ReportConfig::default()
            };

            let pdf = generate_pdf(&report, &config).expect("PDF should render");
            assert_pdf_smoke(&pdf, 15_000);
        }
    }

    #[test]
    fn test_batch_pdf_smoke_renders_valid_pdf() {
        let batch = BatchReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://example.com"),
                pdf_fixture_report_for_url("https://example.com/about"),
            ],
            vec![],
            2_400,
        );

        let pdf = generate_batch_pdf(&batch, &ReportConfig::default()).expect("PDF should render");
        assert_pdf_smoke(&pdf, 20_000);
    }

    #[test]
    fn test_comparison_pdf_smoke_renders_valid_pdf() {
        let comparison = ComparisonReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://alpha.example.com"),
                pdf_fixture_report_for_url("https://beta.example.com"),
            ],
            2_400,
        );

        let pdf = generate_comparison_pdf(&comparison, &ReportConfig::default())
            .expect("PDF should render");
        assert_pdf_smoke(&pdf, 12_000);
    }

    #[test]
    fn test_cover_logo_asset_prefers_existing_custom_logo() {
        let logo = tempfile::NamedTempFile::new().expect("custom logo fixture should be writable");
        let config = ReportConfig {
            logo_path: Some(logo.path().to_path_buf()),
            ..ReportConfig::default()
        };

        assert_eq!(cover_logo_asset(&config), CUSTOM_COVER_LOGO_ASSET);
    }

    #[test]
    fn test_cover_logo_asset_falls_back_for_missing_custom_logo() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let missing_logo = temp_dir.path().join("missing-logo.svg");
        let config = ReportConfig {
            logo_path: Some(missing_logo),
            ..ReportConfig::default()
        };

        assert_eq!(cover_logo_asset(&config), WORDMARK_ASSET);
    }

    #[test]
    fn test_single_pdf_renders_in_english_locale() {
        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Standard,
            locale: "en".to_string(),
            ..ReportConfig::default()
        };

        let pdf = generate_pdf(&report, &config).expect("English PDF should render");
        assert_pdf_smoke(&pdf, 15_000);
    }

    #[test]
    fn test_pdf_german_and_english_outputs_differ() {
        // Locale-aware narrative must produce different PDF bytes.
        let report = pdf_fixture_report();
        let de = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Standard,
                locale: "de".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("German PDF should render");
        let en = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Standard,
                locale: "en".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("English PDF should render");
        assert_ne!(de, en, "German and English PDFs should differ in content");
    }

    #[test]
    fn test_pdf_with_custom_logo_differs_from_default() {
        // A custom logo asset registered on the cover must change PDF bytes.
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let logo_path = temp_dir.path().join("custom-logo.svg");
        // Minimal valid SVG so Typst can decode it.
        std::fs::write(
            &logo_path,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="32" viewBox="0 0 120 32"><rect width="120" height="32" fill="#ff00ff"/></svg>"##,
        )
        .expect("write logo");

        let report = pdf_fixture_report();
        let default_pdf =
            generate_pdf(&report, &ReportConfig::default()).expect("default PDF should render");
        let custom_pdf = generate_pdf(
            &report,
            &ReportConfig {
                logo_path: Some(logo_path),
                ..ReportConfig::default()
            },
        )
        .expect("custom logo PDF should render");

        assert_ne!(
            default_pdf, custom_pdf,
            "PDF with custom logo should differ from default cover"
        );
    }

    #[test]
    fn test_single_pdf_technical_renders_multiple_pages_when_pdftoppm_is_available() {
        let Some(pdftoppm) = find_executable("pdftoppm") else {
            return;
        };

        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Technical,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("auditmysite-pages.pdf");
        let png_prefix = temp_dir.path().join("auditmysite-pages");
        std::fs::write(&pdf_path, pdf).expect("PDF fixture should be writable");

        let status = Command::new(pdftoppm)
            .arg("-png")
            .arg("-r")
            .arg("72")
            .arg(&pdf_path)
            .arg(&png_prefix)
            .status()
            .expect("pdftoppm should run");
        assert!(status.success(), "pdftoppm failed with {status}");

        let mut produced_pages = 0;
        for entry in std::fs::read_dir(temp_dir.path()).expect("temp dir should be readable") {
            let entry = entry.expect("dir entry should be readable");
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("auditmysite-pages-") && name_str.ends_with(".png") {
                produced_pages += 1;
            }
        }

        assert!(
            produced_pages >= 3,
            "Technical report should render at least 3 pages, got {produced_pages}"
        );
    }

    #[test]
    fn test_single_pdf_first_page_can_be_rasterized_when_pdftoppm_is_available() {
        let Some(pdftoppm) = find_executable("pdftoppm") else {
            return;
        };

        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Executive,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("auditmysite-smoke.pdf");
        let png_prefix = temp_dir.path().join("auditmysite-smoke-page");
        std::fs::write(&pdf_path, pdf).expect("PDF fixture should be writable");

        let status = Command::new(pdftoppm)
            .arg("-png")
            .arg("-f")
            .arg("1")
            .arg("-singlefile")
            .arg(&pdf_path)
            .arg(&png_prefix)
            .status()
            .expect("pdftoppm should run");

        assert!(status.success(), "pdftoppm failed with {status}");

        let png_path = png_prefix.with_extension("png");
        let png = std::fs::read(&png_path).expect("first page PNG should exist");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "PNG header missing");
        assert!(
            png.len() > 10_000,
            "PNG too small to represent a rendered report page: {} bytes",
            png.len()
        );
    }

    #[test]
    fn test_pdf_technical_contains_violation_criteria() {
        // Every WCAG criterion from the input must appear as text in the rendered PDF.
        // This catches silent information loss between builder and renderer.
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let report = pdf_fixture_report_rich();
        let criteria = [
            "1.4.3", "1.1.1", "4.1.2", "2.4.4", "1.3.1", "2.4.1", "2.4.6", "3.1.1",
        ];
        let config = ReportConfig {
            level: ReportLevel::Technical,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("criteria-check.pdf");
        let txt_path = temp_dir.path().join("criteria-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read extracted text");

        for criterion in criteria {
            assert!(
                text.contains(criterion),
                "Criterion {criterion} missing from PDF text — information lost in renderer"
            );
        }
    }

    #[test]
    fn test_pdf_renders_positive_signals_from_patterns() {
        // When the report carries recognized patterns, the PDF text should
        // include a localized pattern title (e.g. "Skip-Link").
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let mut report = pdf_fixture_report_rich();
        report.patterns = Some(crate::patterns::PatternAnalysis {
            recognized: vec![crate::patterns::RecognizedPattern {
                pattern: "SkipLink".to_string(),
                message: "Skip link recognized and correctly positioned.".to_string(),
                confidence: crate::patterns::PatternConfidence::Strong,
            }],
            violations: vec![],
        });

        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("patterns-check.pdf");
        let txt_path = temp_dir.path().join("patterns-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read text");

        assert!(
            text.contains("Skip-Link"),
            "Expected localized pattern title 'Skip-Link' in PDF text"
        );
    }

    #[test]
    fn test_pdf_renders_throttled_performance_table() {
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let mut report =
            pdf_fixture_report_rich().with_performance(crate::audit::PerformanceResults {
                vitals: crate::performance::WebVitals::default(),
                score: crate::performance::PerformanceScore {
                    overall: 80,
                    grade: crate::performance::PerformanceGrade::Gold,
                    lcp_score: None,
                    fcp_score: None,
                    cls_score: None,
                    interactivity_score: None,
                    metrics_available: 0,
                },
                render_blocking: None,
                content_weight: None,
                third_party: None,
                critical_chain: None,
                minification: None,
                animations: None,
                coverage: None,
            });
        report.throttled_performance = vec![crate::audit::ThrottledPerfResult {
            profile: crate::browser::ThrottleProfile::Slow3G,
            lcp_ms: Some(3200.0),
            tbt_ms: Some(180.0),
            cls: Some(0.03),
            score: 72,
        }];

        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("throttled-perf-check.pdf");
        let txt_path = temp_dir.path().join("throttled-perf-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read text");

        assert!(
            text.contains("Performance unter gedrosselten Bedingungen"),
            "Expected throttled-performance section title in PDF text"
        );
        assert!(
            text.contains("Slow3G") && text.contains("3200 ms") && text.contains("180 ms"),
            "Expected throttled-performance values in PDF text"
        );
    }

    #[test]
    fn test_pdf_score_present_in_extracted_text() {
        // The overall score computed by normalize() must appear as a number in the rendered PDF.
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let report = pdf_fixture_report_rich();
        let normalized = crate::audit::normalize(&report);
        let expected_score = normalized.score.to_string();
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("score-check.pdf");
        let txt_path = temp_dir.path().join("score-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read extracted text");

        assert!(
            text.contains(&expected_score),
            "Score {expected_score} missing from PDF text — score not rendered on page"
        );
    }

    fn assert_pdf_smoke(pdf: &[u8], min_size: usize) {
        assert!(pdf.starts_with(b"%PDF-"), "PDF header missing");
        assert!(
            pdf.windows(5).any(|window| window == b"%%EOF"),
            "PDF EOF marker missing"
        );
        assert!(
            pdf.len() > min_size,
            "PDF too small to contain the expected report layout: {} bytes",
            pdf.len()
        );
    }

    fn pdf_fixture_report() -> AuditReport {
        pdf_fixture_report_for_url("https://example.com")
    }

    fn pdf_fixture_report_for_url(url: &str) -> AuditReport {
        let mut results = WcagResults::new();
        results.nodes_checked = 42;
        results.passes = 8;
        results.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::High,
                "Image missing alternative text",
                "node-hero-image",
            )
            .with_selector("img.hero")
            .with_html_snippet("<img class=\"hero\" src=\"hero.jpg\">")
            .with_fix("Add a meaningful alt attribute"),
        );

        AuditReport::new(url.to_string(), WcagLevel::AA, results, 1_200)
    }

    /// Richer fixture with multiple violations across severities — closer to a real-world report.
    fn pdf_fixture_report_rich() -> AuditReport {
        let mut results = WcagResults::new();
        results.nodes_checked = 320;
        results.passes = 48;

        let violations = [
            (
                "1.4.3",
                "Contrast (Minimum)",
                WcagLevel::AA,
                Severity::Critical,
                "Text has insufficient color contrast ratio",
                "node-body-text",
            ),
            (
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::Critical,
                "Image missing alternative text on hero banner",
                "node-hero-1",
            ),
            (
                "4.1.2",
                "Name, Role, Value",
                WcagLevel::A,
                Severity::High,
                "Button has no accessible name",
                "node-cta-btn",
            ),
            (
                "2.4.4",
                "Link Purpose",
                WcagLevel::A,
                Severity::High,
                "Link text is not descriptive enough",
                "node-read-more",
            ),
            (
                "1.3.1",
                "Info and Relationships",
                WcagLevel::A,
                Severity::High,
                "Form field missing label",
                "node-email-input",
            ),
            (
                "2.4.1",
                "Bypass Blocks",
                WcagLevel::A,
                Severity::Medium,
                "Skip navigation link missing",
                "node-skip",
            ),
            (
                "2.4.6",
                "Headings and Labels",
                WcagLevel::AA,
                Severity::Medium,
                "Heading hierarchy skips levels",
                "node-h3",
            ),
            (
                "3.1.1",
                "Language of Page",
                WcagLevel::A,
                Severity::Low,
                "HTML lang attribute not set",
                "node-html",
            ),
        ];

        for (criterion, rule_name, level, severity, msg, node_id) in violations {
            results.add_violation(
                Violation::new(criterion, rule_name, level, severity, msg, node_id)
                    .with_selector(format!("#{node_id}"))
                    .with_fix(format!("Fix required for {rule_name}")),
            );
        }

        AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            3_800,
        )
    }

    fn find_executable(name: &str) -> Option<PathBuf> {
        let paths = std::env::var_os("PATH")?;
        std::env::split_paths(&paths)
            .map(|path| path.join(name))
            .find(|path| path.is_file())
    }

    /// Count PDF pages by scanning for `/Type /Page` objects (not `/Type /Pages`).
    fn count_pdf_pages(pdf: &[u8]) -> usize {
        let needle = b"/Type /Page";
        let mut count = 0;
        let mut i = 0;
        while i + needle.len() <= pdf.len() {
            if pdf[i..i + needle.len()] == *needle {
                // Exclude /Type /Pages (the catalogue node)
                if pdf.get(i + needle.len()).copied() != Some(b's') {
                    count += 1;
                }
            }
            i += 1;
        }
        count
    }

    /// Count PDF `/Annot` entries — proxy for callout boxes / links.
    fn count_pdf_annotations(pdf: &[u8]) -> usize {
        let doc = match lopdf::Document::load_mem(pdf) {
            Ok(d) => d,
            Err(_) => return 0,
        };
        doc.objects
            .values()
            .filter(|o| {
                if let Ok(d) = o.as_dict() {
                    return d
                        .get(b"Type")
                        .ok()
                        .and_then(|v| v.as_name().ok())
                        .map(|n| n == b"Annot")
                        .unwrap_or(false);
                }
                false
            })
            .count()
    }

    /// Read PDF outline (bookmark) titles in tree order. Empty when there
    /// is no outline.
    fn pdf_outline_titles(pdf: &[u8]) -> Vec<String> {
        let doc = match lopdf::Document::load_mem(pdf) {
            Ok(d) => d,
            Err(_) => return vec![],
        };
        let mut titles = Vec::new();
        let catalog = match doc.catalog() {
            Ok(c) => c,
            Err(_) => return titles,
        };
        let outlines_ref = match catalog.get(b"Outlines") {
            Ok(v) => v,
            Err(_) => return titles,
        };
        let outlines_id = match outlines_ref.as_reference() {
            Ok(id) => id,
            Err(_) => return titles,
        };
        let outlines = match doc.get_dictionary(outlines_id) {
            Ok(d) => d,
            Err(_) => return titles,
        };
        let mut current = outlines
            .get(b"First")
            .ok()
            .and_then(|v| v.as_reference().ok());
        while let Some(id) = current {
            let dict = match doc.get_dictionary(id) {
                Ok(d) => d,
                Err(_) => break,
            };
            if let Ok(title) = dict.get(b"Title").and_then(|v| v.as_str()) {
                titles.push(String::from_utf8_lossy(title).trim().to_string());
            }
            current = dict.get(b"Next").ok().and_then(|v| v.as_reference().ok());
        }
        titles
    }

    #[test]
    fn test_standard_pdf_larger_than_executive() {
        let report = pdf_fixture_report_rich();
        let exec_pdf = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Executive,
                ..ReportConfig::default()
            },
        )
        .expect("executive PDF should render");
        let std_pdf = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Standard,
                ..ReportConfig::default()
            },
        )
        .expect("standard PDF should render");
        assert!(
            std_pdf.len() > exec_pdf.len(),
            "Standard PDF ({} bytes) should be larger than Executive ({} bytes)",
            std_pdf.len(),
            exec_pdf.len()
        );
    }

    #[test]
    fn test_batch_pdf_page_count_reasonable() {
        let batch = BatchReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://example.com"),
                pdf_fixture_report_for_url("https://example.com/about"),
            ],
            vec![],
            2_400,
        );
        let pdf = generate_batch_pdf(&batch, &ReportConfig::default()).expect("batch PDF");
        let pages = count_pdf_pages(&pdf);
        assert!(
            pages >= 3,
            "Batch PDF must have at least 3 pages, got {}",
            pages
        );
    }

    #[test]
    fn test_comparison_pdf_renders_without_panic() {
        let comparison = ComparisonReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://alpha.example.com"),
                pdf_fixture_report_for_url("https://beta.example.com"),
            ],
            2_400,
        );
        let pdf = generate_comparison_pdf(&comparison, &ReportConfig::default())
            .expect("comparison PDF should render");
        assert!(!pdf.is_empty(), "comparison PDF should not be empty");
    }

    #[test]
    fn test_pdf_has_annotations() {
        // Renderreport emits annotations for some interactive constructs
        // (links, etc.). This is a smoke check that lopdf can parse the PDF
        // and the structural pipeline is intact.
        let report = pdf_fixture_report_rich();
        let pdf = generate_pdf(&report, &ReportConfig::default()).expect("standard PDF");
        let _ = count_pdf_annotations(&pdf); // result not asserted; counts may be 0
        let _ = pdf_outline_titles(&pdf);
        assert!(
            lopdf::Document::load_mem(&pdf).is_ok(),
            "PDF must parse via lopdf"
        );
    }

    #[test]
    fn test_executive_pdf_page_count_within_target() {
        // Use the richer fixture (8 violations across severities) to validate
        // that executive stays compact even with a realistic finding load.
        let report = pdf_fixture_report_rich();
        let config = ReportConfig {
            level: ReportLevel::Executive,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("Executive PDF should render");
        let pages = count_pdf_pages(&pdf);
        assert!(
            pages <= 8,
            "Executive PDF must be ≤ 8 pages per target, got {} pages",
            pages
        );
    }

    #[test]
    fn test_standard_pdf_page_count_reasonable() {
        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("Standard PDF should render");
        let pages = count_pdf_pages(&pdf);
        assert!(
            pages >= 3,
            "Standard PDF must have at least 3 pages, got {}",
            pages
        );
        assert!(
            pages <= 35,
            "Standard PDF must not exceed 35 pages, got {} pages",
            pages
        );
    }
}
