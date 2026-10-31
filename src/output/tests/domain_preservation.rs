//! Domain preservation (#780): one injected problem per domain the finding
//! contract collects must reach `NormalizedReport.findings` with its contract,
//! be accounted for in `finding_inventory`, and survive every projection
//! (JSON, Studio, compact summary, PDF view model, terminal).

use std::collections::BTreeSet;

use clap::Parser;

use crate::audit::finding_contract::{Disposition, FindingDomain};
use crate::audit::normalized::{normalize, InteractiveFinding, InteractiveFindingKind};
use crate::audit::AuditReport;
use crate::cli::{Args, WcagLevel};
use crate::output::report_model::ReportConfig;
use crate::taxonomy::Severity;
use crate::wcag::WcagResults;

/// One injected problem: the finding it must become.
struct Expected {
    domain: FindingDomain,
    rule_id: &'static str,
    count: usize,
}

const fn exp(domain: FindingDomain, rule_id: &'static str, count: usize) -> Expected {
    Expected {
        domain,
        rule_id,
        count,
    }
}

/// Every injection of `report_with_every_domain`.
fn expected() -> Vec<Expected> {
    use FindingDomain as D;
    vec![
        exp(D::Seo, "seo.headings.multiple_h1", 2),
        exp(D::Seo, "seo.meta.description", 1),
        exp(D::Seo, "seo.technical.no_canonical", 1),
        exp(D::Seo, "seo.schema.missing_offers", 1),
        exp(D::Seo, "seo.page_health.meta_refresh", 1),
        exp(D::Security, "security.headers.missing_csp", 1),
        exp(D::Mobile, "mobile.small_font", 1),
        exp(D::Performance, "performance.budget.LCP", 1),
        exp(D::HtmlConformance, "html_conform.duplicate-id", 1),
        exp(
            D::BestPractices,
            "best_practices.vulnerable_libraries.jquery",
            1,
        ),
        exp(D::BestPractices, "best_practices.console.error", 1),
        exp(D::DarkMode, "dark_mode.no_meta_color_scheme", 1),
        exp(D::DesignQuality, "design_quality.line_length", 1),
        exp(D::TechStack, "tech_stack.wordpress.exposed_version", 1),
        exp(D::Ux, "ux.competing_ctas", 1),
        exp(D::Journey, "journey.no_h1", 1),
        exp(D::Commerce, "commerce.missing_price", 1),
        exp(D::AiTransparency, "ai_transparency.unlabeled_ai_image", 1),
        exp(D::NetworkDns, "network_dns.no_dmarc", 1),
        exp(
            D::AccessibilityJourney,
            "a11y_journey.skip_link_focus_not_moved",
            1,
        ),
        exp(D::ScreenReader, "screen_reader.wcag_1_3_1", 1),
    ]
}

/// Every `FindingDomain` and whether this test injects a problem for it.
/// The exhaustive `match` makes a new variant a compile error here until it
/// gets an injection (or an explicit reason it has none).
fn injected(domain: FindingDomain) -> bool {
    use FindingDomain as D;
    match domain {
        // WCAG violations are grouped by `build_wcag_findings`, not the
        // domain collector; none is injected here.
        D::Accessibility => false,
        D::AccessibilityJourney
        | D::ScreenReader
        | D::Seo
        | D::Security
        | D::Mobile
        | D::Performance
        | D::HtmlConformance
        | D::BestPractices
        | D::DarkMode
        | D::DesignQuality
        | D::TechStack
        | D::Ux
        | D::Journey
        | D::Commerce
        | D::AiTransparency
        | D::NetworkDns => true,
    }
}

/// Keep in sync with `FindingDomain`; `injected` fails to compile for a
/// variant it does not know, and this list is asserted against it.
const ALL_DOMAINS: &[FindingDomain] = &[
    FindingDomain::Accessibility,
    FindingDomain::AccessibilityJourney,
    FindingDomain::ScreenReader,
    FindingDomain::Seo,
    FindingDomain::Security,
    FindingDomain::Mobile,
    FindingDomain::Performance,
    FindingDomain::HtmlConformance,
    FindingDomain::BestPractices,
    FindingDomain::DarkMode,
    FindingDomain::DesignQuality,
    FindingDomain::TechStack,
    FindingDomain::Ux,
    FindingDomain::Journey,
    FindingDomain::Commerce,
    FindingDomain::AiTransparency,
    FindingDomain::NetworkDns,
];

fn report_with_every_domain() -> AuditReport {
    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );

    // ── SEO: headings (2×), meta, technical, schema, page health ──────────
    let mut seo = crate::seo::SeoAnalysis::default();
    for _ in 0..2 {
        seo.headings
            .issues
            .push(crate::seo::headings::HeadingIssue {
                issue_type: "multiple_h1".to_string(),
                message: "The page has more than one H1.".to_string(),
                severity: Severity::Medium,
            });
    }
    seo.meta_issues.push(crate::seo::meta::MetaValidation {
        field: "description".to_string(),
        message: "Meta description is missing.".to_string(),
        severity: Severity::Medium,
        suggestion: None,
    });
    seo.technical
        .issues
        .push(crate::seo::technical::TechnicalIssue {
            issue_type: "no_canonical".to_string(),
            message: "No canonical link.".to_string(),
            severity: Severity::High,
        });
    seo.structured_data
        .schema_issues
        .push(crate::seo::schema::SchemaIssue {
            schema_type: "Product".to_string(),
            severity: crate::seo::schema::SchemaIssueSeverity::Required,
            issue_type: "missing_offers".to_string(),
            message: "Product schema lacks offers.".to_string(),
        });
    seo.page_health = Some(crate::seo::page_health::PageHealthAnalysis {
        issues: vec![crate::seo::page_health::PageHealthIssue {
            issue_type: "meta_refresh".to_string(),
            message: "The page uses a meta refresh.".to_string(),
            severity: "warning".to_string(),
        }],
        ..Default::default()
    });
    report = report.with_seo(seo);

    // ── Security ──────────────────────────────────────────────────────────
    report = report.with_security(crate::security::SecurityAnalysis {
        score: 80,
        grade: "B".into(),
        headers: Default::default(),
        ssl: Default::default(),
        issues: vec![crate::security::SecurityIssue {
            header: "Content-Security-Policy".to_string(),
            issue_type: "missing_csp".to_string(),
            message: "No Content-Security-Policy header.".to_string(),
            severity: Severity::High,
            tier: crate::security::HeaderTier::Baseline,
            values: Default::default(),
            score_deduction: 10,
        }],
        recommendations: vec![],
        protection: Default::default(),
        sourcemap_leaks: Default::default(),
        fetch: Default::default(),
    });

    // ── Mobile ────────────────────────────────────────────────────────────
    report = report.with_mobile(crate::mobile::MobileFriendliness {
        score: 80,
        viewport: Default::default(),
        touch_targets: Default::default(),
        font_sizes: Default::default(),
        content_sizing: Default::default(),
        issues: vec![crate::mobile::MobileIssue {
            category: "font".to_string(),
            issue_type: "small_font".to_string(),
            message: "Text is too small on mobile.".to_string(),
            severity: Severity::Medium,
            impact: "Hard to read.".to_string(),
            values: Default::default(),
        }],
    });

    // ── Performance budget ────────────────────────────────────────────────
    report
        .experience
        .budget_violations
        .push(crate::audit::BudgetViolation {
            metric: "LCP".to_string(),
            budget_label: "≤ 2500 ms".to_string(),
            actual_label: "4000 ms".to_string(),
            budget_value: 2500.0,
            actual_value: 4000.0,
            exceeded_by_pct: 60.0,
            severity: crate::audit::BudgetSeverity::Error,
        });

    // ── HTML conformance ──────────────────────────────────────────────────
    report = report.with_html_conform(crate::html_conform::HtmlConformAnalysis {
        score: 90,
        checked: true,
        error_count: 1,
        warning_count: 0,
        info_count: 0,
        distinct_defect_count: 1,
        findings: vec![crate::html_conform::HtmlConformFinding {
            rule_id: "duplicate-id".to_string(),
            severity: "error".to_string(),
            message: "Duplicate id `main`.".to_string(),
            location: Some("#main".to_string()),
            byte_offset: None,
        }],
        raw_html: None,
    });

    // ── Best practices: one vulnerable library, one console error ─────────
    report = report.with_best_practices(crate::best_practices::BestPracticesAnalysis {
        console_errors: crate::best_practices::ConsoleErrorsAnalysis {
            errors: vec![crate::best_practices::ConsoleMessage {
                level: "error".to_string(),
                message: "Uncaught TypeError".to_string(),
            }],
            warnings: vec![],
            error_count: 1,
            warning_count: 0,
            collected: true,
        },
        vulnerable_libraries: crate::best_practices::VulnerableLibrariesAnalysis {
            detected: vec![],
            vulnerable: vec![crate::best_practices::VulnerableLibrary {
                name: "jQuery".to_string(),
                version: "1.12.4".to_string(),
                severity: "medium".to_string(),
                description: "XSS in htmlPrefilter".to_string(),
                safe_version: "3.5.0".to_string(),
                advisories: vec![],
                exploitability: Default::default(),
            }],
            has_vulnerabilities: true,
            duplicate_libraries: vec![],
            checked: true,
        },
        score: 80,
    });

    // ── Dark mode ─────────────────────────────────────────────────────────
    report = report.with_dark_mode(crate::dark_mode::DarkModeAnalysis {
        supported: false,
        class_based_dark_mode: false,
        score: 50,
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
        issues: vec![crate::dark_mode::DarkModeIssue {
            kind: crate::dark_mode::DarkModeIssueKind::NoMetaColorScheme,
            description: "No color-scheme meta tag.".to_string(),
            severity: "low".to_string(),
            selectors: vec![],
        }],
    });

    // ── Design quality ────────────────────────────────────────────────────
    report = report.with_design_quality(crate::design_quality::DesignQualityAnalysis {
        findings: vec![crate::design_quality::DesignQualityFinding {
            rule_id: "design.line_length".to_string(),
            level: crate::design_quality::FindingLevel::Advisory,
            confidence: crate::design_quality::Confidence::Medium,
            measurement_type: "heuristic".to_string(),
            selector: "p.body-text".to_string(),
            evidence: "~110 characters per line.".to_string(),
            message: "Long lines may reduce readability.".to_string(),
        }],
        rules_run: vec!["design.line_length".to_string()],
    });

    // ── Tech stack ────────────────────────────────────────────────────────
    report = report.with_tech_stack(crate::tech_stack::TechStackAnalysis {
        detected: vec![],
        findings: vec![crate::tech_stack::StackFinding {
            tech: "WordPress".to_string(),
            title: "Exposed version".to_string(),
            detail: "The generator meta tag reveals the version.".to_string(),
            severity: Severity::High,
            fix: None,
            url_checked: Some("https://example.com/".to_string()),
        }],
        score: 80,
        grade: "B".into(),
        audited_stacks: Vec::new(),
    });

    // ── UX ────────────────────────────────────────────────────────────────
    let mut ux = crate::ux::analyze_ux(&crate::AXTree::new());
    ux.issues = vec![crate::ux::UxIssue {
        kind: crate::ux::UxIssueKind::CompetingCtas,
        dimension: "CTA Clarity".to_string(),
        severity: "medium".to_string(),
        problem: "Several primary calls to action compete.".to_string(),
        impact: "Users hesitate.".to_string(),
        recommendation: "Keep one primary CTA.".to_string(),
        values: Default::default(),
    }];
    report = report.with_ux(ux);

    // ── Journey ───────────────────────────────────────────────────────────
    let mut journey = crate::journey::analyze_journey(&crate::AXTree::new());
    journey.friction_points = vec![crate::journey::FrictionPoint {
        kind: crate::journey::FrictionKind::NoH1,
        step: "entry".to_string(),
        severity: "high".to_string(),
        problem: "The page has no H1.".to_string(),
        impact: "Orientation suffers.".to_string(),
        recommendation: "Add an H1.".to_string(),
        values: Default::default(),
    }];
    report = report.with_journey(journey);

    // ── Commerce ──────────────────────────────────────────────────────────
    report.commerce = Some(crate::commerce::CommerceAnalysis {
        page_kind: crate::commerce::CommercePageKind::ProductDetail,
        product: None,
        trust_pages: Default::default(),
        identification_function_detected: false,
        identification_function_bfsg_reference: None,
        findings: vec![crate::commerce::CommerceFinding {
            kind: crate::commerce::CommerceFindingKind::MissingPrice,
            severity: Severity::High,
            message: "The product has no price.".to_string(),
        }],
    });

    // ── AI transparency ───────────────────────────────────────────────────
    report.experience.ai_transparency = Some(crate::ai_transparency::AiTransparencyAnalysis {
        findings: vec![crate::ai_transparency::ImageProvenanceFinding {
            rule_id: "ai_transparency.unlabeled_ai_image".to_string(),
            image_url: "https://example.com/hero.png".to_string(),
            provenance: crate::ai_transparency::AiProvenanceKind::TrainedAlgorithmicMedia,
            validation: crate::ai_transparency::ManifestValidation::Valid,
            generator: None,
            measurement_type: "metadata".to_string(),
            evidence: "IPTC DigitalSourceType".to_string(),
            message: "AI-generated image without a visible label.".to_string(),
        }],
        ..Default::default()
    });

    // ── Network DNS ───────────────────────────────────────────────────────
    report.experience.network_dns = Some(crate::network::dns::NetworkDnsAnalysis {
        findings: vec![crate::network::dns::DnsFinding {
            rule_id: "dns.no_dmarc".to_string(),
            message: "No DMARC record.".to_string(),
        }],
        ..Default::default()
    });

    // ── Accessibility journey (interactive) ───────────────────────────────
    report.interactive_findings.push(InteractiveFinding {
        category: "SkipLink".to_string(),
        kind: InteractiveFindingKind::SkipLinkFocusNotMoved,
        maps_to_finding: None,
        severity: Severity::High,
        journey: "skip_link".to_string(),
        before_snapshot_label: None,
        after_snapshot_label: None,
        message: "Skip link does not move focus".to_string(),
        fix_suggestion: None,
        values: Default::default(),
        uncertainty: None,
    });

    // ── Screen reader ─────────────────────────────────────────────────────
    let mut sr = crate::screen_reader::build_sr_audit_report(
        "https://example.com",
        chrono::Utc::now(),
        &crate::AXTree::new(),
        "en",
        None,
    );
    sr.issues = vec![crate::screen_reader::SrAuditIssue {
        wcag_criterion: Some("1.3.1".to_string()),
        severity: "medium".to_string(),
        affected_node_ids: vec!["node-1".to_string()],
        message: "Landmark structure is incomplete.".to_string(),
        review: false,
    }];
    report.screen_reader_audit = Some(sr);

    report
}

#[test]
fn every_domain_reaches_findings_inventory_and_projections() {
    let report = report_with_every_domain();
    let ctx = normalize(&report);
    let normalized = &ctx.normalized;
    normalized
        .validate()
        .expect("normalized report must satisfy its contract");

    let expected = expected();

    // ── Canonical findings + inventory ────────────────────────────────────
    for e in &expected {
        let finding = normalized
            .findings
            .iter()
            .find(|f| f.rule_id == e.rule_id)
            .unwrap_or_else(|| {
                panic!(
                    "no finding `{}`; got {:?}",
                    e.rule_id,
                    normalized
                        .findings
                        .iter()
                        .map(|f| &f.rule_id)
                        .collect::<Vec<_>>()
                )
            });
        assert_eq!(finding.contract.domain, e.domain, "{}", e.rule_id);
        assert_eq!(
            format!("{}.{}", finding.contract.source, finding.contract.check),
            e.rule_id
        );
        assert_eq!(finding.occurrence_count, e.count, "{}", e.rule_id);
        assert!(
            normalized.finding_inventory.iter().any(|entry| {
                entry.domain == e.domain
                    && format!("{}.{}", entry.source, entry.check) == e.rule_id
                    && entry.collected == e.count
                    && matches!(&entry.disposition,
                        Disposition::Finding { finding_id } if finding_id == &finding.id)
            }),
            "no inventory entry for {}",
            e.rule_id
        );
    }

    // All five SEO sources are represented.
    let seo_sources: BTreeSet<&str> = normalized
        .findings
        .iter()
        .filter(|f| f.contract.domain == FindingDomain::Seo)
        .map(|f| f.contract.source.as_str())
        .collect();
    assert_eq!(
        seo_sources,
        BTreeSet::from([
            "seo.headings",
            "seo.meta",
            "seo.technical",
            "seo.schema",
            "seo.page_health"
        ])
    );

    // Domain coverage: every variant except Accessibility.
    let present: BTreeSet<FindingDomain> = normalized
        .findings
        .iter()
        .map(|f| f.contract.domain)
        .collect();
    let required: BTreeSet<FindingDomain> = ALL_DOMAINS
        .iter()
        .copied()
        .filter(|d| injected(*d))
        .collect();
    let injected_domains: BTreeSet<FindingDomain> = expected.iter().map(|e| e.domain).collect();
    assert_eq!(
        injected_domains, required,
        "every domain `injected` names needs an injection in `expected()`"
    );
    assert_eq!(present, required, "domains present in findings");

    let canonical: BTreeSet<&str> = normalized
        .findings
        .iter()
        .map(|f| f.rule_id.as_str())
        .collect();

    // ── JSON ──────────────────────────────────────────────────────────────
    let json = crate::output::json::UnifiedReport::single(&ctx, &report)
        .to_json(true)
        .expect("JSON serializes");
    let json: serde_json::Value = serde_json::from_str(&json).expect("JSON parses");
    let page = &json["pages"][0];
    let json_rule_ids: BTreeSet<&str> = page["findings"]
        .as_array()
        .expect("pages[0].findings")
        .iter()
        .filter_map(|f| f["rule_id"].as_str())
        .collect();
    assert_eq!(json_rule_ids, canonical, "JSON findings");
    let json_inventory = page["finding_inventory"]
        .as_array()
        .expect("pages[0].finding_inventory");
    assert_eq!(json_inventory.len(), normalized.finding_inventory.len());
    for e in &expected {
        assert!(
            json_inventory.iter().any(|entry| {
                format!(
                    "{}.{}",
                    entry["source"].as_str().unwrap_or_default(),
                    entry["check"].as_str().unwrap_or_default()
                ) == e.rule_id
                    && entry["collected"].as_u64() == Some(e.count as u64)
                    && entry["disposition"]["kind"] == "finding"
            }),
            "JSON inventory lacks {}",
            e.rule_id
        );
    }

    // ── Studio ────────────────────────────────────────────────────────────
    let studio = crate::studio::StudioAuditResponse::from_normalized(&ctx, &report, String::new());
    for e in &expected {
        let f = studio
            .findings
            .iter()
            .find(|f| f.rule_id == e.rule_id)
            .unwrap_or_else(|| panic!("Studio lacks {}", e.rule_id));
        assert_eq!(f.occurrences, e.count, "Studio count of {}", e.rule_id);
    }

    // ── Compact summary feed ──────────────────────────────────────────────
    let summary = crate::output::summary::format_summary(normalized).expect("summary renders");
    let summary: serde_json::Value = serde_json::from_str(&summary).expect("summary parses");
    let top = summary["topFindings"].as_array().expect("topFindings");
    assert_eq!(top.len(), normalized.findings.len().min(10));
    for t in top {
        let id = t["id"].as_str().expect("top finding id");
        let f = normalized
            .findings
            .iter()
            .find(|f| f.rule_id == id)
            .unwrap_or_else(|| panic!("summary lists non-canonical finding {id}"));
        assert_eq!(t["count"].as_u64(), Some(f.occurrence_count as u64), "{id}");
    }
    assert_eq!(
        summary["totalIssues"].as_u64(),
        Some(normalized.severity_counts.total as u64)
    );
    assert_eq!(
        summary["criticalIssues"].as_u64(),
        Some(normalized.severity_counts.critical as u64)
    );
    assert_eq!(
        summary["highIssues"].as_u64(),
        Some(normalized.severity_counts.high as u64)
    );
    assert_eq!(
        summary["verdict"],
        serde_json::to_value(normalized.decision.verdict).unwrap()
    );

    // ── PDF input model ───────────────────────────────────────────────────
    let vm = crate::output::builder::build_view_model(&ctx, &ReportConfig::default());
    let vm_rule_ids: BTreeSet<&str> = vm
        .findings
        .all_findings
        .iter()
        .map(|f| f.rule_id.as_str())
        .collect();
    let visible: BTreeSet<&str> = normalized
        .findings
        .iter()
        .filter(|f| f.report_visibility.standard)
        .map(|f| f.rule_id.as_str())
        .collect();
    assert!(
        !visible.is_empty(),
        "fixture must contain High findings to exercise the PDF path"
    );
    for f in normalized.findings.iter().filter(|f| f.category != "wcag") {
        assert_eq!(
            f.report_visibility.standard,
            (f.severity >= Severity::High && f.contract.is_primary())
                || f.contract.scoring.effect == crate::audit::finding_contract::ScoringEffect::Gate,
            "visibility rule for {}",
            f.rule_id
        );
    }
    assert_eq!(vm_rule_ids, visible, "PDF view model findings");

    // ── Terminal ──────────────────────────────────────────────────────────
    let args = Args::parse_from(["auditmysite", "https://example.com"]);
    let terminal = crate::output::terminal::render_single_report(&report, &args);
    // The headline carries the verdict symbol (ASCII or Unicode theme).
    let symbols: [&str; 2] = match normalized.decision.verdict {
        crate::audit::Verdict::Pass => ["[OK]", "✔"],
        crate::audit::Verdict::Warn => ["[WARN]", "⚠"],
        crate::audit::Verdict::Fail => ["[FAIL]", "✖"],
    };
    let headline = terminal.lines().next().unwrap_or_default();
    assert!(
        symbols.iter().any(|s| headline.contains(s)),
        "terminal headline must show the stored decision {:?}:\n{terminal}",
        normalized.decision.verdict
    );
}

#[test]
fn losing_a_domain_finding_breaks_the_inventory() {
    let report = report_with_every_domain();
    let ctx = normalize(&report);
    let mut normalized = ctx.normalized.clone();
    let idx = normalized
        .findings
        .iter()
        .position(|f| f.rule_id == "seo.technical.no_canonical")
        .expect("injected finding exists");
    normalized.findings.remove(idx);
    let err = normalized
        .validate()
        .expect_err("a lost finding must fail validation");
    assert!(
        err.violations.iter().any(|v| v.code == "finding_inventory"),
        "{err:?}"
    );
}

#[test]
fn changed_occurrence_count_breaks_the_inventory() {
    let report = report_with_every_domain();
    let ctx = normalize(&report);
    let mut normalized = ctx.normalized.clone();
    let finding = normalized
        .findings
        .iter_mut()
        .find(|f| f.rule_id == "seo.headings.multiple_h1")
        .expect("injected finding exists");
    finding.occurrence_count = 1;
    let err = normalized
        .validate()
        .expect_err("a changed occurrence count must fail validation");
    assert!(
        err.violations.iter().any(|v| v.code == "finding_inventory"),
        "{err:?}"
    );
}

/// Indicator modules publish signals, never problems (#780). Attaching them
/// to a report full of problems must not add, change or remove a finding, and
/// no finding may name one of them as its source.
#[test]
fn indicator_modules_create_no_findings_of_their_own() {
    use crate::audit::finding_contract::{DomainPolicy, DOMAIN_POLICIES};

    let plain = report_with_every_domain();
    let mut with_indicators = report_with_every_domain();
    with_indicators.discoverability.source_quality =
        Some(crate::source_quality::analyze_source_quality(&plain));
    with_indicators.discoverability.ai_visibility =
        Some(crate::ai_visibility::analyze_ai_visibility(&plain));
    with_indicators.discoverability.content_visibility = Some(
        crate::content_visibility::analyze_content_visibility(&plain),
    );

    let rule_ids = |report: &AuditReport| -> Vec<(String, usize)> {
        let mut ids: Vec<_> = normalize(report)
            .normalized
            .findings
            .iter()
            .map(|f| (f.rule_id.clone(), f.occurrence_count))
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(rule_ids(&plain), rule_ids(&with_indicators));

    let indicators: Vec<&str> = DOMAIN_POLICIES
        .iter()
        .filter(|(_, policy)| matches!(policy, DomainPolicy::SignalsOnly(_)))
        .map(|(module, _)| *module)
        .collect();
    assert!(indicators.len() >= 3, "{indicators:?}");
    let normalized = normalize(&with_indicators).normalized;
    for finding in &normalized.findings {
        assert!(
            !indicators
                .iter()
                .any(|module| finding.contract.source.starts_with(module)),
            "{} is a finding of an indicator module",
            finding.rule_id
        );
    }
    assert!(normalized
        .finding_inventory
        .iter()
        .all(|entry| !indicators.iter().any(|m| entry.source.starts_with(m))));
}

/// The WCAG domain (`Accessibility`) is built by `build_wcag_findings`, not by
/// the domain collector, so it needs its own injection: the violation must
/// reach the findings, the inventory-checked contract and the JSON.
#[test]
fn a_wcag_violation_reaches_findings_and_json() {
    let mut results = WcagResults::new();
    for node in ["n1", "n2"] {
        results.add_violation(crate::wcag::Violation::new(
            "3.1.1",
            "Language of Page",
            WcagLevel::A,
            Severity::High,
            "The page has no lang attribute.",
            node,
        ));
    }
    let report = AuditReport::new("https://example.com".to_string(), WcagLevel::AA, results, 1);
    let ctx = normalize(&report);
    ctx.normalized
        .validate()
        .expect("normalized report must satisfy its contract");
    let finding = ctx
        .normalized
        .findings
        .iter()
        .find(|f| f.contract.domain == FindingDomain::Accessibility)
        .expect("the WCAG violation is a finding");
    assert_eq!(finding.occurrence_count, 2);

    let json = crate::output::json::UnifiedReport::single(&ctx, &report)
        .to_json(true)
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(&json).unwrap();
    let ids: Vec<&str> = json["pages"][0]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["rule_id"].as_str())
        .collect();
    assert!(ids.contains(&finding.rule_id.as_str()), "{ids:?}");
}

/// A module that was not measured contributes no findings and no inventory
/// entries: "not measured" is not "measured and clean" (#780).
#[test]
fn an_unmeasured_module_leaves_no_findings() {
    let mut report = report_with_every_domain();
    let html = report.html_conform.as_mut().expect("html module injected");
    html.checked = false;
    let ctx = normalize(&report);
    ctx.normalized
        .validate()
        .expect("normalized report must satisfy its contract");
    assert!(ctx
        .normalized
        .findings
        .iter()
        .all(|f| f.contract.domain != FindingDomain::HtmlConformance));
    assert!(ctx
        .normalized
        .finding_inventory
        .iter()
        .all(|e| e.domain != FindingDomain::HtmlConformance));
    let entry = ctx
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "HTML Conformance")
        .unwrap();
    assert_eq!(entry.measurement_type, "not_measured");
}
