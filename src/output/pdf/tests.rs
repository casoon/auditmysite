#[allow(clippy::module_inception)]
#[cfg(all(test, feature = "pdf_test"))]
mod tests {
    use super::super::*;
    use crate::audit::{AuditReport, BatchReport, PageScreenshots, ScreenshotStatus};
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

    /// Regression test for a class of localization leak the existing
    /// "no umlauts/ß" guard tests structurally cannot catch: hardcoded German
    /// table labels/words that happen to contain no umlaut or ß (e.g.
    /// "Vorkommen", "Sprache", "Interne Links"). `pdf_fixture_report()` alone
    /// carries no SEO data, so `build_seo_details`'s whole English-locale
    /// code path (identity_facts, page_profile_facts, technical_summary,
    /// tracking_summary) was previously never exercised by any EN-locale
    /// test at all.
    ///
    /// Note on methodology: a string's presence in `--debug-typ` output does
    /// NOT by itself prove it renders onto a page — the dump embeds every
    /// registered renderreport component template verbatim (`include_str!`),
    /// including ones this report never instantiates. Only assert against
    /// strings actually reachable through this report's own content-building
    /// code (as verified here against `build_seo_details`), not raw
    /// substring presence in the dump.
    #[test]
    fn test_seo_details_english_locale_has_no_known_german_leaks() {
        let report = pdf_fixture_report().with_seo(crate::seo::SeoAnalysis {
            score: 80,
            ..Default::default()
        });
        let typ = generate_typ(
            &report,
            &ReportConfig {
                level: ReportLevel::Technical,
                locale: "en".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("English Typst source should render");

        // Confirms the English-only branches actually fire (not just "no
        // German leaked", but "the intended English text is present").
        for expected in [
            "Page title",
            "Content type",
            "Topic hints",
            "Page type",
            "Characteristics",
            "Classification",
            "Recommendation",
            "Language tag",
            "Word count",
            "Internal links",
        ] {
            assert!(
                typ.contains(expected),
                "expected English label {expected:?} in EN-locale Typst source"
            );
        }

        // The specific hardcoded-German strings confirmed and fixed in this
        // session (#511-style regression corpus candidates). Deliberately
        // excludes "Empfehlung": it also happens to be the unrelated default
        // fallback text of the (unused) renderreport `dominant-issue-spotlight`
        // component, which is always present in any --debug-typ dump per the
        // methodology note above, regardless of this fix.
        for leaked in [
            "Seitentitel",
            "Inhaltstyp",
            "Themenhinweise",
            "Seitentyp",
            "Merkmale",
            "Einordnung",
            "Sprachangabe",
            "Wortanzahl",
            "Interne Links",
            "Externe Links",
            "Vollständigkeit",
        ] {
            assert!(
                !typ.contains(leaked),
                "German string {leaked:?} leaked into EN-locale Typst source"
            );
        }
    }

    #[test]
    fn test_single_pdf_places_json_ld_status_before_schema_inventory() {
        let structured_data = crate::seo::schema::analyze_structured_data_payloads(
            &[
                serde_json::json!({
                    "@context": "https://schema.org",
                    "@type": "WebPage",
                    "name": "Example",
                    "url": "https://example.com"
                })
                .to_string(),
                r#"{"@context":"https://schema.org","@type":"Product""#.to_string(),
            ],
            false,
            false,
        );
        let seo = crate::seo::SeoAnalysis {
            structured_data,
            score: 73,
            ..Default::default()
        };
        let report = pdf_fixture_report().with_seo(seo);
        let typ = generate_typ(
            &report,
            &ReportConfig {
                level: ReportLevel::Technical,
                locale: "de".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("Typst source should render");

        let status = typ.find("JSON-LD-Status").expect("JSON-LD status missing");
        let inventory = typ
            .find("Strukturierte Daten (1 Schema)")
            .expect("schema inventory missing");
        assert!(status < inventory, "status must precede schema inventory");
        assert!(typ.contains("ungültiges JSON"));
    }

    #[test]
    fn test_single_pdf_places_page_fit_and_feature_rules_before_inventory() {
        let mut structured_data = crate::seo::schema::analyze_structured_data_payloads(
            &[serde_json::json!({
                "@context": "https://schema.org",
                "@type": "Product",
                "name": "Example product",
                "offers": {"@type": "Offer", "price": "19.99"}
            })
            .to_string()],
            false,
            false,
        );
        let fit = crate::seo::schema_fit::assess_schema_fit(
            "https://example.com/produkt/example",
            crate::journey::PageIntent::Shop,
            &structured_data,
        );
        crate::seo::schema::refresh_rule_assessments(&mut structured_data, fit.product_context());
        structured_data.rule_assessments[0]
            .manual_review
            .push("Confirm that marked-up details are visible.".to_string());
        structured_data.content_parity = vec![crate::seo::schema_parity::ContentParityAssessment {
            node_index: 0,
            schema_type: "Product".to_string(),
            property: "name".to_string(),
            status: crate::seo::schema_parity::ContentParityStatus::Mismatch,
            schema_value: Some("Example product".to_string()),
            visible_value: Some("Different visible title".to_string()),
            evidence: "Schema and visible title differ".to_string(),
        }];
        structured_data.fit_assessment = Some(fit);

        let report = pdf_fixture_report().with_seo(crate::seo::SeoAnalysis {
            structured_data,
            score: 73,
            ..Default::default()
        });
        let typ = generate_typ(
            &report,
            &ReportConfig {
                level: ReportLevel::Technical,
                locale: "de".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("Typst source should render");

        let status = typ.find("JSON-LD-Status").expect("JSON-LD status missing");
        let fit = typ
            .find("Seitentyp und Schema-Eignung")
            .expect("schema fit missing");
        let rules = typ
            .find("Funktionsbezogene Schema-Anforderungen")
            .expect("feature rules missing");
        let inventory = typ
            .find("Strukturierte Daten (1 Schema)")
            .expect("schema inventory missing");
        let manual_review = typ
            .find("Kontext- und Inhaltsprüfung")
            .expect("manual-review table missing");
        let content_parity = typ
            .find("Abgleich mit sichtbaren Inhalten")
            .expect("content-parity table missing");

        assert!(
            status < fit
                && fit < rules
                && rules < manual_review
                && manual_review < content_parity
                && content_parity < inventory
        );
        assert!(typ.contains("Merchant Listing"));
        assert!(typ.contains("Pflichtangaben fehlen"));
        assert!(typ.contains("Schema: Example product; sichtbar: Different visible title"));
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

    /// Standard deviation of grayscale pixel values, as a cheap "is there
    /// real content here" signal. A blank/degenerate page (missing font or
    /// asset causing empty content) renders as a near-solid color and scores
    /// close to 0; real report content (text, tables, charts) always has
    /// substantial variance. Deliberately not an exact pixel-diff — those are
    /// too flaky across environments (font hinting/anti-aliasing differ by
    /// machine); this only asks "is content there", not "is it pixel-identical".
    fn png_luma_std_dev(png_path: &std::path::Path) -> f64 {
        let bytes = std::fs::read(png_path).expect("rasterized page PNG should be readable");
        let image =
            image::load_from_memory(&bytes).expect("rasterized page should be a valid image");
        let luma = image.to_luma8();
        let pixels: Vec<f64> = luma.pixels().map(|p| p.0[0] as f64).collect();
        let mean = pixels.iter().sum::<f64>() / pixels.len() as f64;
        let variance = pixels.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / pixels.len() as f64;
        variance.sqrt()
    }

    /// Rasterizes `pdf` to one PNG per page under `temp_dir` (via `pdftoppm`)
    /// and returns the produced paths in page order.
    fn rasterize_pages(
        pdftoppm: &std::path::Path,
        pdf: &[u8],
        temp_dir: &std::path::Path,
    ) -> Vec<PathBuf> {
        let pdf_path = temp_dir.join("auditmysite-visual.pdf");
        let png_prefix = temp_dir.join("auditmysite-visual");
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

        let mut pages: Vec<PathBuf> = std::fs::read_dir(temp_dir)
            .expect("temp dir should be readable")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("auditmysite-visual-") && n.ends_with(".png"))
                    .unwrap_or(false)
            })
            .collect();
        pages.sort();
        pages
    }

    #[test]
    fn test_single_pdf_technical_pages_are_not_blank_when_pdftoppm_is_available() {
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
        let pages = rasterize_pages(&pdftoppm, &pdf, temp_dir.path());
        assert!(
            pages.len() >= 3,
            "expected at least 3 pages, got {}",
            pages.len()
        );

        for page in &pages {
            let std_dev = png_luma_std_dev(page);
            assert!(
                std_dev > 5.0,
                "page {} looks blank/degenerate (grayscale std dev {std_dev:.2})",
                page.display()
            );
        }
    }

    #[test]
    fn test_batch_pdf_pages_are_not_blank_when_pdftoppm_is_available() {
        let Some(pdftoppm) = find_executable("pdftoppm") else {
            return;
        };

        let batch = BatchReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://example.com"),
                pdf_fixture_report_for_url("https://example.com/about"),
            ],
            vec![],
            2_400,
        );
        let pdf = generate_batch_pdf(&batch, &ReportConfig::default()).expect("batch PDF");

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pages = rasterize_pages(&pdftoppm, &pdf, temp_dir.path());
        assert!(
            pages.len() >= 3,
            "expected at least 3 pages, got {}",
            pages.len()
        );

        for page in &pages {
            let std_dev = png_luma_std_dev(page);
            assert!(
                std_dev > 5.0,
                "page {} looks blank/degenerate (grayscale std dev {std_dev:.2})",
                page.display()
            );
        }
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
            journey_candidates: vec![],
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
                    si_score: None,
                    metrics_available: 0,
                    size_penalty: None,
                    js_penalty: None,
                    request_penalty: None,
                    dom_penalty: None,
                    is_capped: None,
                },
                render_blocking: None,
                content_weight: None,
                third_party: None,
                critical_chain: None,
                minification: None,
                animations: None,
                coverage: None,
                measurement_warnings: vec![],
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
    fn test_pdf_interprets_performance_resources_and_bottlenecks() {
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };
        let report = pdf_fixture_report_rich().with_performance(crate::audit::PerformanceResults {
            vitals: crate::performance::WebVitals {
                dom_nodes: Some(12_485),
                load_time: Some(7_280.0),
                dom_content_loaded: Some(4_756.0),
                js_heap_size: Some(9_961_472),
                ..crate::performance::WebVitals::default()
            },
            score: crate::performance::PerformanceScore {
                overall: 38,
                grade: crate::performance::PerformanceGrade::NeedsImprovement,
                lcp_score: None,
                fcp_score: None,
                cls_score: None,
                interactivity_score: None,
                si_score: None,
                metrics_available: 0,
                size_penalty: None,
                js_penalty: None,
                request_penalty: None,
                dom_penalty: None,
                is_capped: None,
            },
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: Some(crate::performance::MinificationAnalysis {
                unminified_scripts: vec![crate::performance::UnminifiedAsset {
                    url: "https://www.inros-lackner.de/assets/app/build/index-noncritical.js"
                        .to_string(),
                    kind: "script".to_string(),
                    decoded_bytes: 305_818,
                    transfer_bytes: 80_000,
                    savings_bytes: 203_878,
                }],
                unminified_styles: vec![crate::performance::UnminifiedAsset {
                    url: "https://www.inros-lackner.de/assets/app/build/index.css?v=1".to_string(),
                    kind: "css".to_string(),
                    decoded_bytes: 454_810,
                    transfer_bytes: 110_000,
                    savings_bytes: 303_514,
                }],
                total_savings_bytes: 507_392,
                total_unminified_count: 2,
                legacy_scripts: vec![],
                total_legacy_wasted_bytes: 0,
            }),
            animations: None,
            coverage: Some(crate::performance::CoverageAnalysis {
                unused_js: crate::performance::UnusedJsAnalysis {
                    scripts: vec![],
                    total_bytes: 760_628,
                    unused_bytes: 0,
                    used_pct: 100.0,
                },
                unused_css: crate::performance::UnusedCssAnalysis {
                    total_rules: 0,
                    used_rules: 0,
                    used_pct: None,
                    measurement: "not_available".to_string(),
                },
                measurement_warnings: vec![],
            }),
            measurement_warnings: vec![],
        });
        let pdf = generate_pdf(
            &report,
            &ReportConfig {
                level: ReportLevel::Standard,
                ..ReportConfig::default()
            },
        )
        .expect("performance preview PDF");
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("performance-interpretation.pdf");
        let text_path = temp_dir.path().join("performance-interpretation.txt");
        std::fs::write(&pdf_path, pdf).expect("write performance PDF");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&text_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(text_path).expect("read PDF text");
        for expected in [
            "Größter direkt nutzbarer Hebel",
            "Code-Nutzung unauffällig",
            "Ziel: max. 800",
            "Priorisierte Maßnahmen",
        ] {
            assert!(
                text.contains(expected),
                "missing PDF interpretation: {expected}"
            );
        }
    }

    #[test]
    fn test_pdf_score_present_in_extracted_text() {
        // The overall score computed by normalize() must appear as a number in the rendered PDF.
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let report = pdf_fixture_report_rich();
        let normalized = crate::audit::normalize(&report);
        let expected_score = normalized.normalized.score.to_string();
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

    fn tiny_png_bytes() -> &'static [u8] {
        &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9c, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ]
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

    #[test]
    fn test_pdf_contains_no_raw_typst_syntax() {
        // Regression test for #239: raw Typst source code must never appear in the
        // rendered PDF text (e.g. "block( width: 100%, fill: accent, radius: 8pt )").
        // If renderreport fails to compile a section, it may emit the raw template
        // string into the output instead of a compiled result.
        //
        // Text extraction uses `lopdf` so no external tool (pdftotext) is required.
        for level in [
            ReportLevel::Executive,
            ReportLevel::Standard,
            ReportLevel::Technical,
        ] {
            let report = pdf_fixture_report_rich();
            let config = ReportConfig {
                level,
                ..ReportConfig::default()
            };
            let pdf = generate_pdf(&report, &config).expect("PDF should render");

            let doc = lopdf::Document::load_mem(&pdf).expect("lopdf parse");
            let page_ids: Vec<u32> = doc.get_pages().keys().copied().collect();
            let text = doc.extract_text(&page_ids).unwrap_or_default();

            for (pattern, description) in forbidden_typst_patterns() {
                assert!(
                    !text.contains(pattern),
                    "Raw Typst syntax found in {:?} PDF — {description} ({pattern:?} in extracted text). Issue #239.",
                    level,
                );
            }
        }
    }

    /// Fast smoke test for #239 using `Engine::render_typ()` — verifies that the
    /// intermediate Typst source for every report level assembles without errors
    /// and that templates we depend on are present in the source. Much faster
    /// than the full PDF round-trip; runs in <1s.
    #[test]
    fn test_typ_source_smoke_for_all_report_levels() {
        for level in [
            ReportLevel::Executive,
            ReportLevel::Standard,
            ReportLevel::Technical,
        ] {
            let report = pdf_fixture_report_rich();
            let config = ReportConfig {
                level,
                ..ReportConfig::default()
            };
            let typ = generate_typ(&report, &config).expect("typ source should assemble");

            assert!(
                typ.len() > 5_000,
                "typ source for {:?} suspiciously short ({} bytes)",
                level,
                typ.len()
            );

            // Sanity: source must contain the template token boundary marker.
            assert!(
                typ.contains("#let "),
                "typ source for {:?} must include at least one `#let` (template definitions)",
                level
            );
        }
    }

    #[test]
    fn test_typ_renders_device_preview_when_screenshots_are_available() {
        use crate::audit::{ViewportScoreSet, ViewportScores};

        let mut report = pdf_fixture_report_rich();
        report.page_screenshots = Some(PageScreenshots {
            desktop: tiny_png_bytes().to_vec(),
            mobile: tiny_png_bytes().to_vec(),
        });
        report.screenshot_status = ScreenshotStatus::Captured;
        report.viewport_scores = Some(ViewportScores {
            desktop: ViewportScoreSet {
                accessibility: 20,
                performance: None,
                overall: 20,
            },
            mobile: ViewportScoreSet {
                accessibility: 20,
                performance: None,
                overall: 20,
            },
            weighted_overall: 20,
        });

        let ts = report.timestamp.timestamp_nanos_opt().unwrap_or(0);
        let desktop_path = std::env::temp_dir().join(format!("ams-desktop-{}.png", ts));
        let mobile_path = std::env::temp_dir().join(format!("ams-mobile-{}.png", ts));

        let typ = unescape_typ(&generate_typ(&report, &ReportConfig::default()).expect("typ"));

        assert!(typ.contains("device-preview"));
        assert!(typ.contains(PAGE_DESKTOP_SCREENSHOT_ASSET));
        assert!(typ.contains(PAGE_MOBILE_SCREENSHOT_ASSET));
        // With viewport scores available, the compact strip contains the three
        // score values while the captured images provide the visual preview.
        assert!(typ.contains("Barrierefreiheit"));
        assert!(typ.contains("Desktop"));
        assert!(typ.contains("Mobile"));
        assert!(typ.contains("Barrierefreiheit - Gesamt"));
        assert!(typ.contains("70/30 gewichtet"));
        assert!(typ.contains("Barrierefreiheits-Gesamtwert"));
        assert!(
            !desktop_path.exists() && !mobile_path.exists(),
            "temporary screenshot assets should be removed after Typst rendering"
        );
    }

    // ── Typst ⇄ JSON consistency per report part (3 internal areas) ─────────
    //
    // The single report is split into three parts (TEIL 1/2/3). For each part we
    // decide which aggregated audit value belongs there and assert that the value
    // is (a) present in that part of the Typst source and (b) identical to the
    // value the JSON report exposes. We deliberately test the *number as a value*
    // (the way the figure is represented, e.g. `73` / `73/100`), NOT the visual
    // formatting (fonts, colors, spacing) — those are not part of the contract.
    //
    //   Teil 1 (Executive)      → overall score (headline aggregate)
    //   Teil 2 (Accessibility)  → accessibility score + every module score
    //                             (the aggregated module overview lives here)
    //   Teil 3 (Tech & Quality) → every non-accessibility module score (detail)

    /// Unescape the embedded component JSON so values read as plain `"value":"73"`.
    fn unescape_typ(typ: &str) -> String {
        typ.replace("\\\"", "\"")
    }

    /// True if `n` appears as a *value* (not a color/spacing/font literal).
    /// Matches both JSON-in-string (renderreport ≤0.2.20) and Typst-dict
    /// (renderreport ≥0.2.21) representations:
    ///   JSON: `"value":"73"`  `"value":"73/100"`  `"score":73`
    ///   Typst: `value: "73"`  `value: "73/100"`  `score: 73,`
    fn part_has_value(part: &str, n: u32) -> bool {
        let s = n.to_string();
        [
            // JSON-in-string format (renderreport ≤0.2.20)
            format!(":\"{s}\""),
            format!(":\"{s}/100\""),
            format!(":\"{s}/"),
            format!(":{s},"),
            format!(":{s}}}"),
            // Typst dict format (renderreport ≥0.2.21): `key: "value"` or `key: number`
            format!(": \"{s}\""),
            format!(": \"{s}/"),
            format!(": {s},"),
            format!(": {s})"),
            format!(": {s}.0,"),
            format!(": {s}.0)"),
        ]
        .iter()
        .any(|needle| part.contains(needle))
    }

    /// Split the (unescaped) Typst source into the 3 report parts on the
    /// "TEIL 2 / TEIL 3" dividers. Part 1 covers the cover + executive front matter.
    fn split_parts(typ: &str) -> (String, String, String) {
        let i2 = typ.find("TEIL 2 VON 3").expect("Teil 2 divider present");
        let i3 = typ.find("TEIL 3 VON 3").expect("Teil 3 divider present");
        assert!(i2 < i3, "part dividers must appear in order");
        (
            typ[..i2].to_string(),
            typ[i2..i3].to_string(),
            typ[i3..].to_string(),
        )
    }

    #[test]
    fn test_typ_aggregates_consistent_with_json_per_part() {
        // Rich WCAG fixture + one module (SEO = 73) so all three parts render.
        let seo = crate::seo::SeoAnalysis {
            score: 73,
            ..Default::default()
        };
        let report = pdf_fixture_report_rich().with_seo(seo);

        let config = ReportConfig {
            level: ReportLevel::Technical,
            locale: "de".to_string(),
            ..ReportConfig::default()
        };

        // JSON holds the aggregated values that MUST also appear in the PDF.
        let normalized = crate::audit::normalize(&report);
        let unified = crate::output::UnifiedReport::single(&normalized, &report);
        let json: serde_json::Value =
            serde_json::from_str(&unified.to_json(true).expect("json")).expect("parse json");
        let page = &json["pages"][0];

        let overall = page["overall_score"].as_u64().expect("overall_score") as u32;
        let a11y = page["accessibility_score"]
            .as_u64()
            .expect("accessibility_score") as u32;
        let module_scores: Vec<(String, u32)> = page["module_scores"]
            .as_array()
            .expect("module_scores")
            .iter()
            .map(|m| {
                (
                    m["name"].as_str().unwrap_or_default().to_string(),
                    m["score"].as_u64().unwrap_or_default() as u32,
                )
            })
            .collect();
        assert!(
            module_scores.iter().any(|(n, _)| n == "SEO"),
            "fixture must expose an SEO module score"
        );

        let typ = unescape_typ(&generate_typ(&report, &config).expect("typ"));
        let (p1, p2, p3) = split_parts(&typ);

        // Teil 1 — Executive: the overall score is the headline aggregate.
        assert!(
            part_has_value(&p1, overall),
            "overall score {overall} (JSON) must appear in Teil 1"
        );

        // Teil 2 — Accessibility: a11y score + every module score (overview).
        assert!(
            part_has_value(&p2, a11y),
            "accessibility score {a11y} (JSON) must appear in Teil 2"
        );
        for (name, score) in &module_scores {
            assert!(
                part_has_value(&p2, *score),
                "module '{name}' score {score} (JSON) must appear in the Teil 2 overview"
            );
        }

        // Teil 3 — Tech & Quality: non-accessibility module scores (detail).
        for (name, score) in &module_scores {
            if name == "Accessibility" {
                continue;
            }
            assert!(
                part_has_value(&p3, *score),
                "module '{name}' score {score} (JSON) must appear in the Teil 3 detail"
            );
        }
    }

    #[test]
    fn test_dual_viewport_accessibility_score_is_identical_in_json_and_pdf() {
        use crate::audit::{ViewportScoreSet, ViewportScores};

        let mut report = pdf_fixture_report_rich();
        // Simulate the former bug: a score recomputed from the merged finding
        // union differed from the two viewport scores shown later in the PDF.
        report.accessibility.score = 12.0;
        report.viewport_scores = Some(ViewportScores {
            desktop: ViewportScoreSet {
                accessibility: 80,
                performance: None,
                overall: 80,
            },
            mobile: ViewportScoreSet {
                accessibility: 20,
                performance: None,
                overall: 20,
            },
            weighted_overall: 38,
        });

        let normalized = crate::audit::normalize(&report);
        let unified = crate::output::UnifiedReport::single(&normalized, &report);
        assert_eq!(unified.summary.accessibility_score, 38);
        assert_eq!(unified.pages[0].accessibility_score, 38);
        assert_eq!(
            unified.pages[0]
                .module_scores
                .iter()
                .find(|module| module.name == "Accessibility")
                .map(|module| module.score),
            Some(38)
        );

        let typ =
            unescape_typ(&generate_typ(&report, &ReportConfig::default()).expect("Typst source"));
        assert!(typ.contains("Barrierefreiheit - Gesamt"));
        assert!(part_has_value(&typ, 38));
        assert!(typ.contains("70/30 gewichtet"));
    }

    #[test]
    fn test_screen_reader_quality_numbers_explain_scale_and_counts() {
        let mut report = pdf_fixture_report_rich();
        report.screen_reader_audit = Some(crate::screen_reader::build_sr_audit_report(
            &report.url,
            report.timestamp,
            &crate::AXTree::new(),
            "de",
            None,
        ));

        let typ =
            unescape_typ(&generate_typ(&report, &ReportConfig::default()).expect("Typst source"));
        assert!(typ.contains("Heading-Qualität"));
        assert!(typ.contains("Qualitätswerte nutzen eine Skala von 0–100"));
        assert!(typ.contains("reine Anzahlen, keine Qualitätswerte"));
    }

    fn forbidden_typst_patterns() -> &'static [(&'static str, &'static str)] {
        &[
            ("block( width:", "Typst block() call with params"),
            ("block(width:", "Typst block() call (no space)"),
            ("v(spacing-", "Typst v() vertical-space call"),
            ("box(height:", "Typst box() call with height"),
            ("fill: accent", "Typst fill: accent token"),
            ("#pagebreak()", "Typst page-break call"),
            ("#colbreak()", "Typst column-break call"),
        ]
    }
}
