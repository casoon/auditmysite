#[allow(clippy::module_inception)]
#[cfg(all(test, feature = "pdf_test"))]
mod tests {
    use super::super::*;
    use crate::audit::{AuditReport, BatchReport, PageScreenshots, ScreenshotStatus};
    use crate::cli::{AnnexKind, ReportLevel, WcagLevel};
    use crate::util::truncate_url;
    use crate::wcag::{Severity, Violation, WcagResults};
    use std::path::PathBuf;
    use std::process::Command;

    #[test]
    fn remove_file_if_exists_ignores_missing_file_but_reports_real_errors() {
        let directory = tempfile::tempdir().unwrap();
        let missing = directory.path().join("never-created.png");
        assert!(remove_file_if_exists(&missing).is_ok());

        let file = directory.path().join("shot.png");
        std::fs::write(&file, b"png").unwrap();
        assert!(remove_file_if_exists(&file).is_ok());
        assert!(!file.exists());

        // `remove_file` on a directory fails on every platform, with a kind other than NotFound.
        let error = remove_file_if_exists(directory.path()).unwrap_err();
        assert_ne!(error.kind(), std::io::ErrorKind::NotFound);
    }

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
    fn test_single_pdf_smoke_renders_with_bik_guide_annex() {
        // Opt-in "BIK für Alle" chapter mapping (--annex bik). The fixture's
        // 1.1.1 violation should land in the annex's "Images & alt text"
        // chapter; this is a rendering smoke test only (structure/content is
        // covered by `wcag::bik_guide`'s own unit tests).
        let report = pdf_fixture_report();
        let config = ReportConfig {
            level: ReportLevel::Technical,
            annex: Some(AnnexKind::Bik),
            ..ReportConfig::default()
        };

        let pdf = generate_pdf(&report, &config).expect("PDF with BIK annex should render");
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

    /// Regression test for the #406-violating fallback in
    /// `render_assessment_and_execution_notes` (`single_report.rs`): a
    /// NotTestable/Warning finding whose `rule_id` has no `explanations.rs`
    /// entry used to fall straight through to the raw, canonical-English
    /// `fix_suggestion` text instead of the localized generic fallback
    /// sentence — confirmed live in DE-locale reports (1.2.2 video-caption
    /// and 1.4.3 image-background-contrast findings). A fixture rule with a
    /// deliberately unknown `rule_id`/`rule` (no `explanations.rs` entry
    /// exists for either) must render the safe localized fallback, never the
    /// English `fix_suggestion` text, in both loops (`not_testables` and
    /// `warnings`).
    #[test]
    fn test_assessment_notes_unknown_rule_uses_localized_fallback_not_raw_english_fix() {
        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "9.9.9",
                "Fictional Rule",
                WcagLevel::A,
                Severity::Medium,
                "Fictional not-testable finding",
                "node-1",
            )
            .with_fix("This raw English fix text must never leak into a German report.")
            .with_rule_id("fictional-not-testable-rule")
            .with_kind(crate::wcag::Outcome::Untested),
        );
        results.add_violation(
            Violation::new(
                "9.9.8",
                "Fictional Warning Rule",
                WcagLevel::A,
                Severity::Low,
                "Fictional warning finding",
                "node-2",
            )
            .with_fix("This raw English warning text must never leak into a German report.")
            .with_rule_id("fictional-warning-rule")
            .with_kind(crate::wcag::Outcome::Review),
        );
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1_200,
        );

        for (locale, expected_not_testable_fallback, expected_warning_fallback) in [
            (
                "de",
                "Dieses Kriterium manuell an der gerenderten Seite prüfen.",
                "Dieses heuristische Signal manuell bestätigen.",
            ),
            (
                "en",
                "Verify this criterion manually on the rendered page.",
                "Confirm this heuristic signal manually.",
            ),
        ] {
            let typ = generate_typ(
                &report,
                &ReportConfig {
                    level: ReportLevel::Standard,
                    locale: locale.to_string(),
                    ..ReportConfig::default()
                },
            )
            .expect("Typst source should render");

            assert!(
                typ.contains(expected_not_testable_fallback),
                "[{locale}] expected localized not-testable fallback in Typst source"
            );
            assert!(
                typ.contains(expected_warning_fallback),
                "[{locale}] expected localized warning fallback in Typst source"
            );
            assert!(
                !typ.contains("must never leak into a German report"),
                "[{locale}] raw English fix_suggestion text leaked into Typst source"
            );
        }
    }

    /// Build a report where a single WCAG A/AA rule recurs `count` times on
    /// the one audited page — enough to cross the `occurrence_count >= 10`
    /// threshold that promotes a finding into the "systemic"
    /// (`is_component_issue`) category
    /// (`src/output/builder/single/findings.rs`).
    fn pdf_fixture_report_with_repeated_rule(count: usize) -> AuditReport {
        let mut results = WcagResults::new();
        for idx in 0..count {
            results.add_violation(
                Violation::new(
                    "4.1.2",
                    "Name, Role, Value",
                    WcagLevel::AA,
                    Severity::High,
                    format!("Button {idx} has no accessible name"),
                    format!("node-{idx}"),
                )
                .with_selector(format!("#item-{idx}"))
                .with_fix("Add an accessible name"),
            );
        }
        AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            1_500,
        )
    }

    /// #575: single-report "systemic" finding-category text must not assert
    /// unverified cross-page coverage ("resolves the issue ... across all
    /// pages") — a single-URL audit never observed any other page. It must
    /// instead read as an explicit hypothesis pending evidence from
    /// additional pages.
    #[test]
    fn test_systemic_finding_category_text_is_hypothesis_not_domain_wide_assertion() {
        let report = pdf_fixture_report_with_repeated_rule(12);

        for (locale, unqualified_claim, hypothesis_marker) in [
            (
                "de",
                "auf allen betroffenen Seiten",
                "Belegen von weiteren Seiten",
            ),
            ("en", "across all pages", "evidence from additional pages"),
        ] {
            let typ = generate_typ(
                &report,
                &ReportConfig {
                    level: ReportLevel::Standard,
                    locale: locale.to_string(),
                    ..ReportConfig::default()
                },
            )
            .expect("Typst source should render");

            assert!(
                !typ.contains(unqualified_claim),
                "[{locale}] unqualified domain-wide claim {unqualified_claim:?} should not appear in Typst source"
            );
            assert!(
                typ.contains(hypothesis_marker),
                "[{locale}] expected hypothesis-framing marker {hypothesis_marker:?} in Typst source"
            );
        }
    }

    /// #575: the management-summary page must state the audit's actual scope
    /// (exactly one URL, both viewports, no site-wide crawl) up front, so the
    /// page-scoped language used elsewhere in the report can't be misread as
    /// a domain-wide claim.
    #[test]
    fn test_management_summary_scope_line_states_single_url_no_crawl() {
        let report = pdf_fixture_report();

        let de =
            generate_typ(&report, &ReportConfig::default()).expect("DE Typst source should render");
        assert!(
            de.contains("Geprüft: 1 URL · Desktop und Mobile · kein Website-Crawl"),
            "expected DE scope line in Typst source"
        );

        let en = generate_typ(
            &report,
            &ReportConfig {
                locale: "en".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("EN Typst source should render");
        assert!(
            en.contains("Audited: 1 URL · Desktop and Mobile · no website crawl"),
            "expected EN scope line in Typst source"
        );
    }

    /// The cover must carry the scope too — a reader who stops at
    /// "Website-Qualitätsbericht · 92" and a "Barrierefreiheit 100" gauge
    /// otherwise takes both for statements about the whole site.
    #[test]
    fn test_cover_carries_scope_line_and_gauge_qualifier() {
        let report = pdf_fixture_report();

        let de =
            generate_typ(&report, &ReportConfig::default()).expect("DE Typst source should render");
        let cover_start = de.find("#cover-page(").expect("cover page component");
        let cover = &de[cover_start..cover_start + 2000];
        assert!(
            cover.contains("Geprüft: 1 URL · Desktop und Mobile · kein Website-Crawl"),
            "expected DE scope line inside the cover component: {cover}"
        );
        assert!(
            cover.contains("IM AUTOMATISIERTEN PRÜFUMFANG"),
            "expected DE gauge-strip scope qualifier on the cover: {cover}"
        );

        let en = generate_typ(
            &report,
            &ReportConfig {
                locale: "en".to_string(),
                ..ReportConfig::default()
            },
        )
        .expect("EN Typst source should render");
        let cover_start = en.find("#cover-page(").expect("cover page component");
        let cover = &en[cover_start..cover_start + 2000];
        assert!(
            cover.contains("Audited: 1 URL · Desktop and Mobile · no website crawl"),
            "expected EN scope line inside the cover component: {cover}"
        );
        assert!(
            cover.contains("WITHIN THE AUTOMATED SCOPE"),
            "expected EN gauge-strip scope qualifier on the cover: {cover}"
        );
    }

    /// Regression test for the tech-stack findings-severity localization fix
    /// in `detail_modules/indicators.rs::render_tech_stack` — the severity
    /// column used to call `finding.severity.label()` unconditionally, always
    /// emitting the German label ("Hoch"/"Kritisch") even in the EN-locale
    /// report.
    #[test]
    fn test_tech_stack_findings_english_locale_uses_english_severity_label() {
        let report = pdf_fixture_report().with_tech_stack(crate::tech_stack::TechStackAnalysis {
            detected: vec![],
            findings: vec![crate::tech_stack::StackFinding {
                tech: "jQuery".to_string(),
                title: "Outdated jQuery version".to_string(),
                detail: "jQuery 1.x has known vulnerabilities.".to_string(),
                severity: Severity::Critical,
                fix: None,
                url_checked: None,
            }],
            score: 60,
            grade: "C".to_string(),
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

        assert!(
            typ.contains("CRITICAL"),
            "expected English severity label \"CRITICAL\" in EN-locale Typst source"
        );
        assert!(
            !typ.contains("Kritisch"),
            "German severity label \"Kritisch\" leaked into EN-locale Typst source"
        );
    }

    /// Regression test for #577: Dark Mode is an optional product feature,
    /// not a WCAG conformance criterion, so its chapter must not present a
    /// low score with the same Excellent/Good/…/Critical compliance-band
    /// language every other module's ScoreCard uses. Asserts on a window
    /// starting at the chapter's own heading rather than the whole
    /// document — the executive dashboard's severity counter strip always
    /// renders the literal word "Critical"/"Kritisch" as a metric label
    /// regardless of this fix, so a whole-document search would false-fail.
    #[test]
    fn test_dark_mode_chapter_uses_optional_feature_framing_not_compliance_band() {
        for (locale, chapter_marker, optional_word, band_critical_word) in [
            ("de", "Dark Mode", "Optionales Merkmal", "Kritisch"),
            ("en", "Dark mode", "Optional feature", "Critical"),
        ] {
            let report = pdf_fixture_report().with_dark_mode(crate::dark_mode::DarkModeAnalysis {
                supported: false,
                class_based_dark_mode: false,
                score: 20,
                detection_methods: vec![],
                color_scheme_css: false,
                meta_color_scheme: None,
                meta_theme_color_dark: false,
                css_custom_properties: 0,
                dark_contrast_violations: 0,
                light_only_violations: 0,
                dark_only_violations: 0,
                contrast_violations: vec![],
                print: Default::default(),
                forced_colors: Default::default(),
                vision_deficiency: Default::default(),
                issues: vec![],
            });
            let typ = generate_typ(
                &report,
                &ReportConfig {
                    level: ReportLevel::Technical,
                    locale: locale.to_string(),
                    ..ReportConfig::default()
                },
            )
            .expect("Typst source should render");

            // The first occurrence of the chapter title is the actual
            // level-2 chapter heading (`section-header-split(... title:
            // "Dark Mode")`) — this fixture's report has no other content
            // mentioning "Dark Mode" before it. Later occurrences are the
            // chapter's own score card / metric strip / key-value list and,
            // further on, an unrelated methodology sentence listing
            // indicator module names — bounding the window to a modest
            // range after the heading keeps the assertions scoped to the
            // chapter itself.
            let chapter_start = typ.find(chapter_marker).unwrap_or_else(|| {
                panic!(
                    "Dark Mode chapter marker {chapter_marker:?} not found in {locale}-locale Typst source"
                )
            });
            let chapter_end = (chapter_start + 1500).min(typ.len());
            let chapter_text = &typ[chapter_start..chapter_end];

            assert!(
                chapter_text.contains(optional_word),
                "expected optional-feature qualifier {optional_word:?} at/after the Dark Mode chapter heading in {locale}-locale Typst source"
            );
            assert!(
                !chapter_text.contains(band_critical_word),
                "Dark Mode chapter still shows the compliance-band word {band_critical_word:?} in {locale}-locale Typst source"
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

    /// Builds the mobile `PerformanceResults` (for `report.performance`) and
    /// the desktop counterpart wrapped in `DualViewportResults` (for
    /// `report.dual_viewport`), so `build_performance_details` populates both
    /// `PerformancePresentation.desktop` and `.mobile` and `render_performance`
    /// takes the two-gauge branch (`score_gauge_grid`) instead of the
    /// single-viewport fallback.
    fn dual_viewport_performance(
        desktop_score: u32,
        mobile_score: u32,
    ) -> (
        crate::audit::PerformanceResults,
        crate::audit::DualViewportResults,
    ) {
        use crate::audit::{DualViewportResults, PerformanceResults, ViewportAuditData};
        use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

        fn perf(score: u32) -> PerformanceResults {
            PerformanceResults {
                vitals: WebVitals::default(),
                score: PerformanceScore {
                    overall: score,
                    grade: if score >= 75 {
                        PerformanceGrade::Gold
                    } else {
                        PerformanceGrade::NeedsImprovement
                    },
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
            }
        }

        let empty_viewport = |performance: Option<PerformanceResults>| ViewportAuditData {
            wcag_results: WcagResults::new(),
            accessibility_score: 0.0,
            performance,
            seo: None,
            mobile: None,
            ux: None,
            journey: None,
            screenshot: None,
            module_runs: vec![],
        };

        (
            perf(mobile_score),
            DualViewportResults {
                desktop: empty_viewport(Some(perf(desktop_score))),
                mobile: empty_viewport(None),
            },
        )
    }

    /// Regression guard for the historical "flat metric strip" bug named in
    /// issue #510 (the wrapped dual-viewport cell): the Desktop/Mobile
    /// performance comparison must render as two distinct gauge components
    /// inside a 2-column `Grid`, not collapse back into a single combined
    /// text line. This is a structural check on the Typst source (reliable,
    /// no rasterization needed) rather than a pixel judgment — it can't
    /// detect *every* possible bad-wrap regression, but it directly locks in
    /// the fix for the one concrete case #510 names. `to_typst_dict`
    /// (renderreport) turns each component's JSON into a Typst dict literal
    /// (`type: "gauge"`, `label: "Desktop"`), not raw JSON text.
    #[test]
    fn test_dual_viewport_performance_renders_two_gauges_not_a_flat_strip() {
        let (mobile_perf, dual_viewport) = dual_viewport_performance(85, 40);
        let mut report = pdf_fixture_report_rich().with_performance(mobile_perf);
        report.dual_viewport = Some(dual_viewport);

        let typ =
            unescape_typ(&generate_typ(&report, &ReportConfig::default()).expect("Typst source"));

        let gauge_occurrences = typ.matches("type: \"gauge\"").count();
        assert!(
            gauge_occurrences >= 2,
            "expected at least 2 gauge components (Desktop + Mobile) in the Typst source, found {gauge_occurrences}"
        );
        assert!(
            typ.contains("label: \"Desktop\"") && typ.contains("label: \"Mobile\""),
            "expected distinct \"Desktop\" and \"Mobile\" gauge labels in the Typst source"
        );
    }

    /// Renders a fixture that exercises every fixture category issue #510's
    /// acceptance criteria names (cover, scorecards, tables, findings,
    /// methodology, technical metrics) in one report — WCAG findings table,
    /// dual-viewport performance gauges, detected-technology/findings tables,
    /// throttled-network table, lab-data methodology callout — rasterizes
    /// every page, and checks each is non-blank (as the existing blank-page
    /// tests do) plus that the page count stays within a generous, explicit
    /// budget. The budget catches genuine Typst-reflow blowups (a component
    /// stuck re-wrapping runaway content across dozens of pages); it does not
    /// attempt to judge subjective layout quality, which is out of reach for
    /// a pixel/page-count heuristic and stays the `report-critic` skill's job
    /// (#509).
    ///
    /// On failure, rasterized pages are copied to a stable, gitignored
    /// `target/pdf-visual-debug/` directory (tempdir contents are otherwise
    /// deleted before a human could inspect them) — CI uploads that
    /// directory as a build artifact on failure (see `.github/workflows/ci.yml`'s
    /// `pdf-smoke` job) so a red run is diagnosable without a local re-run.
    #[test]
    fn test_representative_fixture_pages_are_not_blank_and_stay_within_page_budget() {
        let Some(pdftoppm) = find_executable("pdftoppm") else {
            return;
        };

        let (mobile_perf, dual_viewport) = dual_viewport_performance(85, 40);
        let mut report = pdf_fixture_report_rich().with_performance(mobile_perf);
        report.dual_viewport = Some(dual_viewport);
        report.throttled_performance = vec![crate::audit::ThrottledPerfResult {
            profile: crate::browser::ThrottleProfile::Slow3G,
            lcp_ms: Some(3200.0),
            tbt_ms: Some(180.0),
            cls: Some(0.03),
            score: 72,
        }];
        report = report.with_tech_stack(crate::tech_stack::TechStackAnalysis {
            detected: vec![crate::tech_stack::DetectedTech {
                name: "jQuery".to_string(),
                category: crate::tech_stack::TechCategory::JsLibrary,
                version: Some("1.12.4".to_string()),
                confidence: crate::tech_stack::Confidence::High,
                signals: vec!["window.jQuery".to_string()],
            }],
            findings: vec![crate::tech_stack::StackFinding {
                tech: "jQuery".to_string(),
                title: "Outdated jQuery version".to_string(),
                detail: "jQuery 1.x has known vulnerabilities.".to_string(),
                severity: Severity::Critical,
                fix: None,
                url_checked: None,
            }],
            score: 60,
            grade: "C".to_string(),
        });

        let config = ReportConfig {
            level: ReportLevel::Technical,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pages = rasterize_pages(&pdftoppm, &pdf, temp_dir.path());

        const MAX_EXPECTED_PAGES: usize = 60;
        if pages.len() > MAX_EXPECTED_PAGES {
            let debug_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target/pdf-visual-debug/page-budget-exceeded");
            let _ = std::fs::create_dir_all(&debug_dir);
            for page in &pages {
                if let Some(name) = page.file_name() {
                    let _ = std::fs::copy(page, debug_dir.join(name));
                }
            }
            panic!(
                "representative fixture rendered {} pages (budget: {MAX_EXPECTED_PAGES}) — \
                 likely a reflow regression; rasterized pages copied to {}",
                pages.len(),
                debug_dir.display()
            );
        }

        let mut blank_pages = Vec::new();
        for page in &pages {
            if png_luma_std_dev(page) <= 5.0 {
                blank_pages.push(page.clone());
            }
        }
        if !blank_pages.is_empty() {
            let debug_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target/pdf-visual-debug/blank-pages");
            let _ = std::fs::create_dir_all(&debug_dir);
            for page in &pages {
                if let Some(name) = page.file_name() {
                    let _ = std::fs::copy(page, debug_dir.join(name));
                }
            }
            panic!(
                "{} of {} pages look blank/degenerate: {:?} — all rasterized pages copied to {}",
                blank_pages.len(),
                pages.len(),
                blank_pages,
                debug_dir.display()
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
            help_mechanisms: vec![],
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
                duplicate_assets: vec![],
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
            "Richtwert: max. 800",
            "Priorisierte Maßnahmen",
        ] {
            assert!(
                text.contains(expected),
                "missing PDF interpretation: {expected}"
            );
        }
    }

    #[test]
    fn test_pdf_renders_html_conform_findings() {
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let report =
            pdf_fixture_report_rich().with_html_conform(crate::html_conform::HtmlConformAnalysis {
                score: 74,
                checked: true,
                error_count: 2,
                warning_count: 1,
                info_count: 0,
                distinct_defect_count: 2,
                findings: vec![
                    crate::html_conform::HtmlConformFinding {
                        rule_id: "schema.html5".to_string(),
                        severity: "error".to_string(),
                        message: "Element <div> not allowed as child of <head>".to_string(),
                        location: Some("12:34".to_string()),
                        byte_offset: None,
                    },
                    crate::html_conform::HtmlConformFinding {
                        rule_id: "parser.html5".to_string(),
                        severity: "warning".to_string(),
                        message: "Unknown entity reference".to_string(),
                        location: None,
                        byte_offset: None,
                    },
                ],
                raw_html: None,
            });
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("html-conform-check.pdf");
        let txt_path = temp_dir.path().join("html-conform-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read extracted text");
        // Collapse the layout's line breaks: the findings table wraps a long
        // message across several lines, so a raw `contains` would assert on
        // column widths rather than on the rendered content.
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

        for expected in [
            "HTML-Konformität",
            "schema.html5",
            "parser.html5",
            "Element <div> not allowed as child of <head>",
            "12:34",
        ] {
            assert!(
                text.contains(expected),
                "missing HTML conformance PDF content: {expected}"
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

    /// The cover has room for exactly one subtitle line. When the scope line
    /// was first appended to the kicker the subtitle wrapped, everything below
    /// shifted down, and the lower gauge row's labels (Mobile/UX/Journey) were
    /// silently dropped — the template gives each label a fixed-height box and
    /// the cover page cannot grow. Measured: ~83 characters render on one line
    /// at this font size and page width, ~132 wrapped.
    #[test]
    fn test_cover_subtitle_stays_on_one_line() {
        const BUDGET: usize = 95;
        for locale in ["de", "en"] {
            let typ = generate_typ(
                &pdf_fixture_report(),
                &ReportConfig {
                    locale: locale.to_string(),
                    ..ReportConfig::default()
                },
            )
            .expect("Typst source should render");
            let subtitle = cover_field(&typ, "subtitle");
            assert!(
                subtitle.chars().count() <= BUDGET,
                "[{locale}] cover subtitle is {} characters and will wrap to a second line, \
                 which pushes the lower gauge row's labels off the page: {subtitle}",
                subtitle.chars().count()
            );
        }
    }

    /// One `name: "value"` field of the cover component in a rendered Typst
    /// source.
    fn cover_field(typ: &str, field: &str) -> String {
        let cover_start = typ.find("#cover-page((").expect("cover page component");
        let marker = format!("{field}: \"");
        let value_start = cover_start
            + typ[cover_start..]
                .find(&marker)
                .unwrap_or_else(|| panic!("cover field '{field}' not found"))
            + marker.len();
        let value_end = value_start + typ[value_start..].find('"').expect("unterminated value");
        typ[value_start..value_end].to_string()
    }

    fn find_executable(name: &str) -> Option<PathBuf> {
        let paths = std::env::var_os("PATH")?;
        std::env::split_paths(&paths)
            .map(|path| path.join(name))
            .find(|path| path.is_file())
    }

    /// Count PDF pages by scanning for `/Type /Page` objects (not `/Type /Pages`).
    /// Count PDF `/Type /Page` objects (leaf pages, not the `/Pages` tree
    /// node). Uses `lopdf` to parse the actual object graph rather than a raw
    /// byte-string scan: Typst 0.15 (renderreport's Typst upgrade for
    /// PDF/UA-1 tagging, #573) can place page objects inside compressed
    /// object streams for larger documents, where the literal ASCII bytes
    /// `/Type /Page` never appear uncompressed — a raw scan silently found 0
    /// pages for exactly the larger (Standard/Batch level) fixtures while
    /// still finding some for small ones, which made this look like a
    /// fixture-specific bug rather than the real cause. `lopdf::Document`
    /// decompresses object streams into `doc.objects` on load, so this
    /// mirrors `count_pdf_annotations` below and stays correct either way.
    fn count_pdf_pages(pdf: &[u8]) -> usize {
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
                        .map(|n| n == b"Page")
                        .unwrap_or(false);
                }
                false
            })
            .count()
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
    fn test_batch_template_cluster_with_long_selector_uses_list_not_key_value_grid() {
        // Regression test for #518: a KeyValueList row with the raw CSS
        // selector as its `key` blew up the row layout (Typst's `auto`
        // column sizing has no cap), squeezing the headline into a
        // one-word-per-line sliver spanning dozens of rows and corrupting
        // the following page break. Fixed by rendering template clusters as
        // a plain bullet List instead, where the selector only ever appears
        // once (embedded in the headline's `{ $selector }` interpolation).
        let long_selector = "section.zone-header:nth-of-type(1) > div.ngl-block.ngl-column:nth-of-type(2) > div.ngl-block.ngl-twig_block > div.container.container-wide > div.breadcrumb-wrapper";
        let snippet = r#"<div class="breadcrumb-wrapper"><a href="/">Start</a></div>"#;
        let make_report = |url: &str| {
            let mut results = WcagResults::new();
            results.nodes_checked = 42;
            results.passes = 8;
            results.add_violation(
                Violation::new(
                    "1.3.1",
                    "Info and Relationships",
                    WcagLevel::A,
                    Severity::Medium,
                    "Heading structure is not semantic",
                    "node-breadcrumb",
                )
                .with_selector(long_selector)
                .with_html_snippet(snippet),
            );
            AuditReport::new(url.to_string(), WcagLevel::AA, results, 1_200)
        };

        let batch = BatchReport::from_reports(
            vec![
                make_report("https://example.com/a"),
                make_report("https://example.com/b"),
                make_report("https://example.com/c"),
            ],
            vec![],
            2_400,
        );
        let typ = unescape_typ(
            &generate_batch_typ(&batch, &ReportConfig::default()).expect("batch Typst source"),
        );

        assert!(
            typ.contains(long_selector),
            "expected the template cluster's selector to appear in the Typst source at all"
        );
        let occurrences = typ.matches(long_selector).count();
        assert_eq!(
            occurrences, 1,
            "expected the selector to appear exactly once (embedded in the List item's \
             headline sentence), found {occurrences} — a KeyValueList-style separate key \
             column would render it a second time and risks the #518 layout blowup"
        );
    }

    /// Plan 54 §3: a 3.2.6 deviation reaches the batch PDF as a localized
    /// sentence next to the affected URL, and the cross-page table carries
    /// the German assessment basis instead of the English JSON text.
    #[test]
    fn batch_pdf_lists_consistent_help_deviations_localized() {
        use crate::patterns::help_mechanisms::{HelpKind, HelpMechanism, HelpRegion};
        let make_report = |url: &str, region: HelpRegion| {
            let mut report =
                AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100);
            report.patterns = Some(crate::patterns::PatternAnalysis {
                help_mechanisms: vec![HelpMechanism {
                    kind: HelpKind::Phone,
                    key: "tel:+4930123".to_string(),
                    region,
                }],
                ..Default::default()
            });
            report
        };
        let batch = BatchReport::from_reports(
            vec![
                make_report("https://example.com/a", HelpRegion::Footer),
                make_report("https://example.com/b", HelpRegion::Footer),
                make_report("https://example.com/c", HelpRegion::Header),
            ],
            vec![],
            300,
        );
        let typ = unescape_typ(
            &generate_batch_typ(&batch, &ReportConfig::default()).expect("batch Typst source"),
        );

        assert!(typ.contains("Konsistente Hilfe (WCAG 3.2.6): Abweichungen"));
        assert!(typ.contains(
            "„tel:+4930123“ steht im Bereich Kopfbereich statt wie auf den meisten Seiten im Bereich Fußbereich."
        ));
        assert!(typ.contains("https://example.com/c"));
        assert!(typ.contains("1 wiederkehrender Hilfemechanismus nach Seitenbereich und relativer Reihenfolge verglichen; 1 Abweichung auf 1 Seite."));
        assert!(!typ.contains("recurring help mechanisms"));
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

    /// Asserts the PDF/UA-1 structural export checklist from auditmysite#573
    /// directly on real, end-to-end generated report bytes: a structure tree
    /// (tagged), a declared document language, a document title, and at
    /// least one outline/bookmark entry. renderreport 0.4.0 (Typst 0.15)
    /// enforces `PdfStandard::Ua_1` at compile time — a report that fails any
    /// of these would already fail to render at all — but this asserts
    /// directly on the emitted bytes rather than relying on that enforcement
    /// alone, per the issue's own "automatisierbaren Export-Check" criterion.
    fn assert_pdf_ua_structure(pdf: &[u8], expected_lang: &str) {
        let doc = lopdf::Document::load_mem(pdf).expect("PDF must parse via lopdf");
        let catalog = doc.catalog().expect("PDF must have a document catalog");

        assert!(
            catalog.has(b"StructTreeRoot"),
            "catalog is missing /StructTreeRoot — the PDF is not tagged"
        );

        let lang = catalog
            .get(b"Lang")
            .ok()
            .and_then(|o| o.as_str().ok())
            .map(|b| String::from_utf8_lossy(b).to_string());
        assert_eq!(
            lang.as_deref(),
            Some(expected_lang),
            "catalog /Lang did not match the report's locale"
        );

        let title = doc
            .trailer
            .get(b"Info")
            .ok()
            .and_then(|o| o.as_reference().ok())
            .and_then(|id| doc.get_dictionary(id).ok())
            .and_then(|info| info.get(b"Title").ok())
            .and_then(|o| o.as_str().ok());
        assert!(
            title.is_some_and(|t| !t.is_empty()),
            "/Info dictionary is missing a non-empty /Title"
        );

        assert!(
            !pdf_outline_titles(pdf).is_empty(),
            "PDF has no outline/bookmark entries"
        );
    }

    #[test]
    fn test_single_pdf_with_findings_is_pdf_ua_tagged() {
        let report = pdf_fixture_report_rich();
        let pdf = generate_pdf(&report, &ReportConfig::default()).expect("standard PDF");
        assert_pdf_ua_structure(&pdf, "de");
    }

    #[test]
    fn test_single_pdf_with_zero_findings_is_pdf_ua_tagged() {
        // #573 acceptance criterion: verify both a report with findings and a
        // clean (0-violation) report — the zero-finding path renders
        // different content (the #572/#576 clean-run callouts) and must stay
        // just as conformant.
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            1_000,
        );
        let pdf = generate_pdf(&report, &ReportConfig::default()).expect("standard PDF");
        assert_pdf_ua_structure(&pdf, "de");
    }

    #[test]
    fn test_batch_pdf_is_pdf_ua_tagged() {
        let batch = BatchReport::from_reports(
            vec![
                pdf_fixture_report_for_url("https://example.com"),
                pdf_fixture_report_for_url("https://example.com/about"),
            ],
            vec![],
            2_400,
        );
        let pdf = generate_batch_pdf(&batch, &ReportConfig::default()).expect("batch PDF");
        assert_pdf_ua_structure(&pdf, "de");
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
    fn test_management_summary_risks_and_strengths_split_for_mixed_report() {
        // #576: the management summary must no longer render a fixed
        // "Die 5 wichtigsten Risiken" block with every dimension — only the
        // ones that are actually a risk, with the good ones in their own
        // strengths panel.
        //
        // The SEO score is set explicitly here. `pdf_fixture_report_rich` has
        // no SEO module, and the panel used to invent one: it looked the score
        // up with `unwrap_or(100)` and rendered "Sehr gute Auffindbarkeit" as
        // a strength for a module that never ran (plan/29, plan 35). An unrun
        // module is now excluded from both panels, so a test about a *good*
        // SEO score has to supply one.
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let mut report = pdf_fixture_report_rich();
        report.discoverability.seo = Some(crate::seo::SeoAnalysis {
            score: 95,
            ..Default::default()
        });
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("risks-strengths-check.pdf");
        let txt_path = temp_dir.path().join("risks-strengths-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read text");

        assert!(
            !text.contains("Die 5 wichtigsten Risiken") && !text.contains("5 Key Risks"),
            "the old fixed-5 risks title must be gone"
        );
        assert!(
            text.contains("Wichtigste Risiken"),
            "expected the dynamic risks panel title in the PDF text"
        );
        assert!(
            text.contains("Stärken im geprüften Umfang"),
            "expected a separate strengths panel for the good dimensions"
        );
        assert_eq!(
            panel_of(&text, "SEO & Sichtbarkeit"),
            "strengths",
            "a good SEO score must be rendered as a strength, not a risk",
        );
    }

    /// Which of the two management-summary panels a dimension label was
    /// rendered under. The risks panel is rendered first, the strengths panel
    /// directly below it, so position in the extracted text settles it — the
    /// row text itself is now the same in both (plan 35 replaced the old
    /// status-specific prose with the rationale that names the numbers).
    fn panel_of(text: &str, label: &str) -> &'static str {
        let risks = text.find("Wichtigste Risiken").expect("risks panel title");
        let strengths = text
            .find("Stärken im geprüften Umfang")
            .expect("strengths panel title");
        let at = text
            .find(label)
            .unwrap_or_else(|| panic!("{label} not in PDF text"));
        assert!(at > risks, "{label} rendered above the risks panel");
        if at < strengths {
            "risks"
        } else {
            "strengths"
        }
    }

    #[test]
    fn test_management_summary_low_seo_score_is_not_shown_as_a_strength() {
        // plan/29-pdf-dashboard-module-lookup-by-name.md: `render_risks_and_strengths`
        // used to look up the SEO/Performance/Mobile score by matching a
        // *display label* substring ("SEO") against `modules.dashboard` —
        // whose cards are relabeled for narrative presentation and no longer
        // necessarily contain that substring. A failed match silently fell
        // back to score 100 ("good"), so a real SEO score of 20 was rendered
        // as a strength ("Sehr gute Auffindbarkeit ...") instead of a risk.
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let mut report = pdf_fixture_report_rich();
        report.discoverability.seo = Some(crate::seo::SeoAnalysis {
            score: 20,
            ..Default::default()
        });
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("seo-not-a-strength-check.pdf");
        let txt_path = temp_dir.path().join("seo-not-a-strength-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read text");

        assert_eq!(
            panel_of(&text, "SEO & Sichtbarkeit"),
            "risks",
            "an SEO score of 20 must be rendered as a risk, not a strength",
        );
        assert!(
            text.contains("SEO-Score 20/100"),
            "the risk row must name the score that set it: {text}",
        );
    }

    #[test]
    fn test_management_summary_renders_zero_risk_empty_state_when_all_dimensions_are_good() {
        // #576: a report with no WCAG violations at all must render the
        // specific zero-risk empty-state wording instead of an empty/missing
        // panel or a padded-out fake risk.
        let Some(pdftotext) = find_executable("pdftotext") else {
            return;
        };

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            1_000,
        );
        let config = ReportConfig {
            level: ReportLevel::Standard,
            ..ReportConfig::default()
        };
        let pdf = generate_pdf(&report, &config).expect("PDF should render");

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let pdf_path = temp_dir.path().join("zero-risk-check.pdf");
        let txt_path = temp_dir.path().join("zero-risk-check.txt");
        std::fs::write(&pdf_path, &pdf).expect("write pdf");
        Command::new(pdftotext)
            .arg(&pdf_path)
            .arg(&txt_path)
            .status()
            .expect("pdftotext should run");
        let text = std::fs::read_to_string(&txt_path).expect("read text");

        assert!(
            text.contains("Keine prioritären Risiken im automatisierten Prüfumfang erkannt"),
            "expected the specific zero-risk empty-state wording"
        );
        assert!(
            !text.contains("Die 5 wichtigsten Risiken") && !text.contains("5 Key Risks"),
            "the old fixed-5 risks title must be gone"
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

        // Teil 1 — Executive: the accessibility score is the headline (plan
        // 29, D1). The weighted overall value is a secondary figure and is
        // derived further in, so it is only required to appear somewhere.
        assert!(
            part_has_value(&p1, a11y),
            "accessibility score {a11y} (JSON) must be the Teil 1 headline"
        );
        assert!(
            part_has_value(&typ, overall),
            "overall score {overall} (JSON) must still appear in the report"
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
