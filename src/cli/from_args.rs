//! The configuration of the layers below, built from the arguments and the
//! config file (#934).
//!
//! `cli` is the entry layer: `audit` and `output` do not know [`Args`] or the
//! TOML [`Config`](super::config::Config). The constructors that read them are
//! impl blocks here, on the types of the layers they configure.

use super::{Args, ColorPolicy, ModuleChoice, RequestMode};
use crate::audit::{pool_acquire_timeout_secs, BatchConfig, ExperimentalIndicator, PipelineConfig};
use crate::browser::{BrowserOptions, PoolConfig};
use crate::output::terminal::{ColorMode, TerminalOptions};

/// The requested indicators, sorted and without duplicates.
fn normalized_experimental(requested: &[ExperimentalIndicator]) -> Vec<ExperimentalIndicator> {
    let mut list = requested.to_vec();
    list.sort();
    list.dedup();
    list
}

/// Whether `--experimental` asks for `indicator` or something derived from it.
fn experimental_requested(args: &Args, indicator: ExperimentalIndicator) -> bool {
    ExperimentalIndicator::closure(&args.experimental).contains(&indicator)
}

impl From<&Args> for PipelineConfig {
    fn from(args: &Args) -> Self {
        let toml_cfg = crate::cli::config::Config::load();
        Self::from_args_and_config(args, toml_cfg.as_ref())
    }
}

impl PipelineConfig {
    pub fn from_args_and_config(
        args: &Args,
        toml_cfg: Option<&crate::cli::config::Config>,
    ) -> Self {
        // The default run is defined once, in `Default`: what neither an
        // argument nor the config file sets comes from there.
        let defaults = Self::default();
        let journey_budget_ms = toml_cfg
            .and_then(|c| c.interactive.journey_budget_ms)
            .unwrap_or(defaults.journey_budget_ms);
        let rule_filter = crate::wcag::RuleFilterConfig {
            // `ignore` was a previously-dead, undocumented synonym for
            // `disabled` (#561, follow-up to #560) — folded in here rather
            // than removed, since any existing config already setting it now
            // starts working instead of staying silently inert.
            disabled_rules: toml_cfg
                .map(|c| {
                    let mut disabled = c.rules.disabled.clone();
                    if let Some(ignored) = &c.rules.ignore {
                        disabled.extend(ignored.iter().cloned());
                    }
                    disabled
                })
                .unwrap_or_default(),
            enabled_only_rules: toml_cfg
                .map(|c| c.rules.enabled_only.clone())
                .unwrap_or_default(),
        };
        Self {
            wcag_level: args.level,
            timeout_secs: args.effective_timeout(),
            stability_budget_ms: args.stability_budget_ms,
            verbose: args.verbose,
            modules: args.module_set(),
            perf_runs: args.perf_runs,
            // Requires performance checking itself active — an isolation
            // pass without a baseline TBT/third-party attribution to diff
            // against is meaningless (see doc comment on the field).
            check_isolate_third_party_impact: args.isolate_third_party_impact
                && args.module_enabled(ModuleChoice::Performance),
            // Requires SEO checking itself active — the resulting finding
            // is attached to `content_visibility`, which is only derived
            // with the SEO module (see `ContentVisibilityModule::is_enabled`).
            check_ssr_content: args.check_ssr_content
                && args.module_enabled(ModuleChoice::Seo)
                && experimental_requested(args, ExperimentalIndicator::ContentVisibility),
            experimental: normalized_experimental(&args.experimental),
            rule_filter,
            capture_screenshots: args.url.is_some()
                && args.effective_format() == crate::cli::OutputFormat::Pdf,
            capture_element_evidence: args.url.is_some()
                && args.effective_format() == crate::cli::OutputFormat::Pdf,
            dismiss_consent: args.dismiss_consent,
            exclude_selectors: toml_cfg
                .map(|c| c.audit.exclude_selectors.clone())
                .unwrap_or_default()
                .into_iter()
                .chain(args.exclude_selector.iter().cloned())
                .collect(),
            interactive: args.interactive,
            allow_state_changes: args.allow_state_changes,
            journey_budget_ms,
            session: args.session,
            storage_state: args
                .load_storage_state()
                .expect("--storage-state is checked by Args::validate"),
            state_id: args.state_id.clone(),
            lang: args.lang.clone(),
            display_mode: args.display.and_then(|d| d.mode()),
            indexability: {
                use crate::audit::finding_contract::IndexabilityExpectation;
                let declared = args
                    .expect_indexable
                    .or_else(|| toml_cfg.and_then(|c| c.audit.expect_indexable))
                    .map(IndexabilityExpectation::declared)
                    .unwrap_or_default();
                // A sitemap lists the URLs a site wants indexed.
                if args.sitemap.is_some() {
                    declared.or(IndexabilityExpectation::sitemap())
                } else {
                    declared
                }
            },
            decision_profile: toml_cfg
                .map(|c| c.effective_verdict_config().profile())
                .unwrap_or(defaults.decision_profile),
            // Not a CLI option: `persist_artifacts`.
            ..defaults
        }
    }
}

impl From<&Args> for BatchConfig {
    fn from(args: &Args) -> Self {
        let pool_config = PoolConfig {
            max_pages: args.effective_concurrency(),
            browser_options: BrowserOptions {
                // Parallel pages share one window; only then are tabs hidden.
                focus_emulation: args.effective_concurrency() > 1,
                session: args.session,
                chrome_path: args.chrome_path.clone(),
                no_sandbox: args.no_sandbox,
                disable_images: args.disable_images,
                timeout_secs: args.effective_timeout(),
                verbose: args.verbose,
                user_agent_override: (args.request_mode == RequestMode::Bot).then(|| {
                    concat!(
                        "auditmysite/",
                        env!("CARGO_PKG_VERSION"),
                        " (+https://github.com/casoon/auditmysite)"
                    )
                    .to_string()
                }),
                ..BrowserOptions::default()
            },
            acquire_timeout_secs: pool_acquire_timeout_secs(args.effective_timeout()),
        };

        Self {
            pipeline: PipelineConfig::from(args),
            concurrency: args.effective_concurrency(),
            max_urls: args.max_pages,
            pool_config,
        }
    }
}

impl From<ColorPolicy> for ColorMode {
    fn from(policy: ColorPolicy) -> Self {
        match policy {
            ColorPolicy::Auto => ColorMode::Auto,
            ColorPolicy::Always => ColorMode::Always,
            ColorPolicy::Never => ColorMode::Never,
        }
    }
}

impl From<&Args> for TerminalOptions {
    fn from(args: &Args) -> Self {
        TerminalOptions {
            report_level: args.report_level,
            lang: args.lang.clone(),
            color: ColorMode::from(args.color),
            writing_to_file: args.output.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{Module, ModuleSet, Viewport};
    use crate::model::WcagLevel;
    use clap::Parser;

    #[test]
    fn test_pipeline_config_from_args() {
        let args = Args {
            expect_indexable: None,
            baseline: None,
            write_baseline: None,
            command: None,
            url: Some("https://example.com".to_string()),
            sitemap: None,
            url_file: None,
            crawl: false,
            level: WcagLevel::AA,
            format: None,
            output: None,
            chrome_path: None,
            remote_debugging_port: None,
            max_pages: 0,
            crawl_depth: 2,
            concurrency: None,
            timeout: None,
            stability_budget_ms: 1500,
            no_sandbox: false,
            disable_images: false,
            verbose: true,
            quiet: false,
            color: crate::cli::ColorPolicy::Auto,
            progress: crate::cli::ProgressPolicy::Auto,
            full: false,
            modules: Vec::new(),
            experimental: Vec::new(),
            privacy_check: false,
            isolate_third_party_impact: false,
            perf_runs: 1,
            check_ssr_content: false,
            reuse_cache: false,
            prefer_sitemap: false,
            per_page_reports: false,
            include_path: Vec::new(),
            exclude_path: Vec::new(),
            sample_strategy: Default::default(),
            screen_reader_report: false,
            dismiss_consent: false,
            exclude_selector: Vec::new(),
            interactive: crate::model::InteractiveMode::Off,
            allow_state_changes: false,
            session: Default::default(),
            storage_state: None,
            state_id: None,
            report_level: crate::model::ReportLevel::Standard,
            lang: "en".to_string(),
            display: None,
            also_json: false,
            logo: None,
            dev: Default::default(),
            annex: None,
            request_mode: crate::cli::RequestMode::Browser,
            report_mode: false,
            annotations: None,
        };

        let config = PipelineConfig::from(&args);
        assert_eq!(config.wcag_level, WcagLevel::AA);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.verbose);
        assert!(config.modules.contains(Module::Performance));
        assert!(config.modules.contains(Module::Mobile));
        assert!(config.modules.contains(Module::Seo));
        assert!(config.modules.contains(Module::Security));
    }

    /// #916: the experimental indicators are in no default run, and an
    /// indicator brings the ones it is derived from.
    #[test]
    fn experimental_indicators_run_only_when_asked_for() {
        use crate::audit::ExperimentalIndicator as I;
        let config =
            PipelineConfig::from(&Args::parse_from(["auditmysite", "https://example.com"]));
        for indicator in I::ALL {
            assert!(!config.runs_indicator(indicator), "{indicator:?}");
        }

        let config = PipelineConfig::from(&Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--experimental",
            "content-visibility,commerce",
        ]));
        for indicator in [
            I::ContentVisibility,
            I::SourceQuality,
            I::AiVisibility,
            I::Ux,
            I::Commerce,
        ] {
            assert!(config.runs_indicator(indicator), "{indicator:?}");
        }
        assert!(!config.runs_indicator(I::Journey));
        let labels = config.active_module_labels();
        assert!(labels.contains(&"Content Visibility") && labels.contains(&"Commerce"));
        assert!(!labels.contains(&"Journey"));

        // --full says nothing about them.
        let config = PipelineConfig::from(&Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--full",
        ]));
        assert!(I::ALL.iter().all(|i| !config.runs_indicator(*i)));
    }

    #[test]
    fn the_experimental_selection_is_part_of_the_cache_signature() {
        let plain = PipelineConfig::from(&Args::parse_from(["auditmysite", "https://example.com"]));
        let ux = PipelineConfig::from(&Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--experimental",
            "ux",
        ]));
        let ux_twice = PipelineConfig::from(&Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--experimental",
            "ux,ux",
        ]));
        assert_ne!(plain.audit_signature(), ux.audit_signature());
        assert_eq!(ux.audit_signature(), ux_twice.audit_signature());
    }

    #[test]
    fn active_module_labels_respect_disabled_full_modules() {
        let config = PipelineConfig::from(&Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--modules",
            "a11y",
        ]));
        let expected = vec!["Accessibility", "Accessibility Journey"];
        assert_eq!(config.active_module_labels(), expected);
    }

    #[test]
    fn full_audit_follows_the_modules_that_run() {
        let config_for = |extra: &[&str]| {
            let mut argv = vec!["auditmysite", "https://example.com"];
            argv.extend_from_slice(extra);
            PipelineConfig::from_args_and_config(&Args::parse_from(argv), None)
        };
        assert!(config_for(&[]).is_full_audit());
        assert!(config_for(&["--full"]).is_full_audit());
        assert!(
            config_for(&["--privacy-check"]).is_full_audit(),
            "the opt-in privacy check does not make a run less than full"
        );
        assert!(
            config_for(&[
                "--modules",
                "performance,seo,security,mobile,html-conform,stack"
            ])
            .is_full_audit(),
            "all six modules listed by name are a full audit"
        );
        assert!(!config_for(&["--modules", "a11y"]).is_full_audit());
        assert!(
            !config_for(&["--modules", "performance,seo,security,mobile,html-conform"])
                .is_full_audit(),
            "one module short is not a full audit"
        );
        assert!(PipelineConfig::default().is_full_audit());
        assert!(!PipelineConfig::default()
            .with_modules(ModuleSet::full().without(Module::Stack))
            .is_full_audit());
    }

    #[test]
    fn seo_and_html_conform_selection_runs_only_fixable_finding_modules() {
        let args = Args::parse_from([
            "auditmysite",
            "--sitemap",
            "https://example.com/sitemap.xml",
            "--modules",
            "seo,html-conform",
        ]);
        let config = PipelineConfig::from_args_and_config(&args, None);
        assert!(!config.is_full_audit());
        assert!(
            !config.modules.contains(Module::Performance),
            "no throttled performance passes"
        );
        assert!(!config.modules.contains(Module::Mobile));
        assert!(!config.modules.contains(Module::Security));
        assert!(!config.modules.contains(Module::Stack));
        assert!(config.modules.contains(Module::Seo));
        assert!(config.modules.contains(Module::HtmlConform));
        let labels = config.active_module_labels();
        assert!(labels.contains(&"Accessibility"));
        assert!(labels.contains(&"HTML Conformance"));
        assert!(labels.contains(&"SEO"));
        assert!(!labels.contains(&"Performance"));
        assert!(!labels.contains(&"Best Practices"));
        assert!(!labels.contains(&"Tech Stack"));
    }

    /// The privacy first visit is opt-in (#871): `--full` alone neither runs it
    /// nor lists the module in the requested scope; `--privacy-check` does,
    /// and only the single capture pass (not a viewport pass) carries it.
    #[test]
    fn privacy_check_is_opt_in_and_not_part_of_full() {
        use crate::audit::audit_scope_from_config;
        let config =
            |argv: &[&str]| PipelineConfig::from_args_and_config(&Args::parse_from(argv), None);
        let full = config(&["auditmysite", "https://example.com", "--full"]);
        assert!(!full.modules.contains(Module::Privacy));
        assert!(!audit_scope_from_config(&full)
            .requested_modules
            .contains(&"privacy".to_string()));
        assert!(!full.active_module_labels().contains(&"Privacy"));

        let opted_in = config(&["auditmysite", "https://example.com", "--privacy-check"]);
        assert!(opted_in.modules.contains(Module::Privacy));
        assert!(audit_scope_from_config(&opted_in)
            .requested_modules
            .contains(&"privacy".to_string()));
        assert!(opted_in.active_module_labels().contains(&"Privacy"));
        assert!(!opted_in
            .for_viewport(Viewport::Desktop)
            .modules
            .contains(Module::Privacy));
        assert!(!opted_in
            .for_viewport(Viewport::Mobile)
            .modules
            .contains(Module::Privacy));
        // The removed pre-release spelling is not accepted.
        assert!(Args::try_parse_from(["auditmysite", "https://example.com", "--privacy"]).is_err());
    }

    /// #788: the extra-reload checks are part of the requested scope in single
    /// and batch invocations alike, and absent when not requested or when
    /// their prerequisite module is off (the flag is then a no-op by contract).
    #[test]
    fn extra_checks_are_requested_scope_in_single_and_batch() {
        use crate::audit::audit_scope_from_config;
        let scope = |argv: &[&str]| {
            let args = Args::parse_from(argv);
            audit_scope_from_config(&PipelineConfig::from_args_and_config(&args, None))
                .requested_subchecks
        };
        let both = [
            "--full",
            "--isolate-third-party-impact",
            "--check-ssr-content",
            "--experimental",
            "content-visibility",
            "--privacy-check",
        ];
        let expected = vec![
            "performance.third_party_isolation".to_string(),
            "content_visibility.ssr_gap".to_string(),
            // Opt-in `--privacy-check` (#871); `--full` alone does not request it.
            "privacy.first_visit".to_string(),
        ];
        for target in [
            ["https://example.com"].as_slice(),
            ["--url-file", "urls.txt"].as_slice(),
            ["--sitemap", "https://example.com/sitemap.xml"].as_slice(),
        ] {
            let argv: Vec<&str> = ["auditmysite"]
                .iter()
                .chain(target)
                .chain(both.iter())
                .copied()
                .collect();
            assert_eq!(scope(&argv), expected, "{target:?}");
        }
        // `--full` does not include the privacy first visit; the flag does.
        assert!(scope(&["auditmysite", "--url-file", "urls.txt", "--full"]).is_empty());
        assert_eq!(
            scope(&["auditmysite", "--url-file", "urls.txt", "--privacy-check"]),
            ["privacy.first_visit"]
        );
        // SEO off: the SSR check has no module to attach to and is not requested.
        // (`Args::validate` rejects this combination before it gets here; the
        // config stays defensive — see the support-matrix test below.)
        assert_eq!(
            scope(&[
                "auditmysite",
                "--url-file",
                "urls.txt",
                "--modules",
                "performance",
                "--check-ssr-content",
                "--isolate-third-party-impact"
            ]),
            vec!["performance.third_party_isolation".to_string()]
        );
    }

    /// #788: no flag combination ends as a silent `false`. For every
    /// combination of mode and module switches, either `Args::validate`
    /// rejects it or the pipeline config carries each requested check on.
    #[test]
    fn requested_extra_checks_are_executed_or_rejected_never_dropped() {
        let targets: [&[&str]; 2] = [&["https://example.com"], &["--url-file", "Cargo.toml"]];
        let module_sets: [&[&str]; 6] = [
            &[],
            &["--full"],
            &["--modules", "performance"],
            &["--modules", "seo"],
            &["--modules", "a11y"],
            &["--modules", "performance,seo"],
        ];
        let checks = ["--isolate-third-party-impact", "--check-ssr-content"];
        let (mut accepted, mut rejected) = (0, 0);
        for target in targets {
            for modules in module_sets {
                for check in checks {
                    let argv: Vec<&str> = ["auditmysite"]
                        .into_iter()
                        .chain(target.iter().copied())
                        .chain(modules.iter().copied())
                        .chain(["--experimental", "content-visibility"])
                        .chain([check])
                        .collect();
                    let args = Args::parse_from(&argv);
                    if args.validate().is_err() {
                        rejected += 1;
                        continue;
                    }
                    accepted += 1;
                    let cfg = PipelineConfig::from_args_and_config(&args, None);
                    let carried = match check {
                        "--isolate-third-party-impact" => cfg.check_isolate_third_party_impact,
                        _ => cfg.check_ssr_content,
                    };
                    assert!(carried, "{argv:?} was accepted but the check is off");
                }
            }
        }
        assert!(accepted > 0 && rejected > 0, "{accepted} / {rejected}");
    }

    #[test]
    fn unsupported_combinations_are_rejected_with_the_reason() {
        let reject = |argv: &[&str]| {
            Args::parse_from(argv)
                .validate()
                .expect_err("must be rejected")
        };
        let e = reject(&[
            "auditmysite",
            "https://example.com",
            "--modules",
            "seo",
            "--isolate-third-party-impact",
        ]);
        assert!(
            e.contains("--isolate-third-party-impact") && e.contains("performance"),
            "{e}"
        );
        let e = reject(&[
            "auditmysite",
            "--url-file",
            "Cargo.toml",
            "--modules",
            "performance",
            "--check-ssr-content",
        ]);
        assert!(
            e.contains("--check-ssr-content") && e.contains("SEO"),
            "{e}"
        );
    }

    /// Single and batch of the same flags carry the same requested scope and
    /// the same measurement profile (#788, #787).
    #[test]
    fn single_and_batch_share_scope_and_measurement_profile() {
        use crate::audit::audit_scope_from_config;
        let flags = [
            "--full",
            "--isolate-third-party-impact",
            "--check-ssr-content",
            "--experimental",
            "content-visibility",
            "--privacy-check",
        ];
        let scope = |target: &[&str]| {
            let argv: Vec<&str> = ["auditmysite"]
                .into_iter()
                .chain(target.iter().copied())
                .chain(flags)
                .collect();
            let args = Args::parse_from(argv);
            args.validate().unwrap();
            audit_scope_from_config(&PipelineConfig::from_args_and_config(&args, None))
        };
        let single = scope(&["https://example.com"]);
        let batch = scope(&["--url-file", "Cargo.toml"]);
        assert_eq!(single.requested_subchecks, batch.requested_subchecks);
        assert_eq!(single.requested_subchecks.len(), 3);
        assert_eq!(
            single.measurement_profile.extra_checks,
            [
                "--isolate-third-party-impact",
                "--check-ssr-content",
                "--privacy-check"
            ]
        );
        assert_eq!(
            serde_json::to_value(&single.measurement_profile).unwrap(),
            serde_json::to_value(&batch.measurement_profile).unwrap()
        );
    }

    #[test]
    fn html_conform_module_runs_by_default_and_when_selected() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        assert!(PipelineConfig::from_args_and_config(&args, None)
            .modules
            .contains(Module::HtmlConform));
        let args = Args::parse_from(["auditmysite", "https://example.com", "--modules", "seo"]);
        assert!(!PipelineConfig::from_args_and_config(&args, None)
            .modules
            .contains(Module::HtmlConform));
        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--modules",
            "seo,html-conform",
        ]);
        assert!(PipelineConfig::from_args_and_config(&args, None)
            .modules
            .contains(Module::HtmlConform));
    }

    /// Scope of the library check (#773): `--modules security` alone collects
    /// it, independent of performance; with neither it stays off.
    #[test]
    fn security_only_audit_runs_the_library_check() {
        use crate::audit::module::AuditModule;
        use crate::best_practices::{module::collects_best_practices, BestPracticesModule};

        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--modules",
            "security",
        ]);
        let cfg = PipelineConfig::from_args_and_config(&args, None);
        assert!(
            !cfg.modules.contains(Module::Performance),
            "fixture must be security-only"
        );
        assert!(BestPracticesModule.is_enabled(&cfg));
        assert!(collects_best_practices(&cfg));

        let mut off = cfg;
        off.modules = off.modules.without(Module::Security);
        assert!(!BestPracticesModule.is_enabled(&off));
        assert!(!collects_best_practices(&off));
    }

    /// One definition of the default run (#933): the configuration of a plain
    /// `auditmysite <URL>` and `PipelineConfig::default()` are the same value,
    /// in every field. `Debug` prints all of them, so a new field is covered
    /// without touching this test.
    #[test]
    fn default_is_the_default_cli_run() {
        let cli = PipelineConfig::from_args_and_config(
            &Args::parse_from(["auditmysite", "https://example.com"]),
            None,
        );
        let default = PipelineConfig::default();
        assert_eq!(cli.audit_signature(), default.audit_signature());
        assert_eq!(format!("{cli:#?}"), format!("{default:#?}"));
    }

    #[test]
    fn module_choices_map_to_the_module_set() {
        let modules = |extra: &[&str]| {
            let argv = ["auditmysite", "https://example.com"]
                .iter()
                .chain(extra)
                .copied();
            PipelineConfig::from_args_and_config(&Args::parse_from(argv), None).modules
        };
        assert_eq!(modules(&[]), ModuleSet::full());
        assert_eq!(modules(&["--full"]), ModuleSet::full());
        assert_eq!(modules(&["--modules", "a11y"]), ModuleSet::none());
        assert_eq!(
            modules(&["--modules", "a11y,seo,html-conform,stack"]),
            ModuleSet::none()
                .with(Module::Seo)
                .with(Module::HtmlConform)
                .with(Module::Stack)
        );
        assert_eq!(
            modules(&["--modules", "performance,security,mobile"]),
            ModuleSet::none()
                .with(Module::Performance)
                .with(Module::Security)
                .with(Module::Mobile)
        );
        assert_eq!(
            modules(&["--privacy-check"]),
            ModuleSet::full().with(Module::Privacy)
        );
        assert_eq!(
            modules(&["--modules", "a11y", "--privacy-check"]),
            ModuleSet::none().with(Module::Privacy)
        );
    }

    #[test]
    fn from_args_and_config_threads_rule_filter_from_toml_config() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let toml_cfg = crate::cli::config::Config {
            rules: crate::cli::config::RulesConfig {
                disabled: vec![
                    "color-contrast".to_string(),
                    "css-overflow-hidden".to_string(),
                ],
                enabled_only: vec![],
                ..Default::default()
            },
            ..Default::default()
        };
        let config = PipelineConfig::from_args_and_config(&args, Some(&toml_cfg));

        assert!(!config.rule_filter.should_run("color-contrast"));
        assert!(!config.rule_filter.should_run("css-overflow-hidden"));
        assert!(config.rule_filter.should_run("image-alt"));
    }

    #[test]
    fn from_args_and_config_merges_exclude_selectors_from_toml_and_cli() {
        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--exclude-selector",
            "[data-specimen]",
            "--exclude-selector",
            ".demo",
        ]);
        let toml_cfg: crate::cli::config::Config = toml::from_str(
            r#"
[audit]
exclude_selectors = [".from-config"]
"#,
        )
        .unwrap();
        let config = PipelineConfig::from_args_and_config(&args, Some(&toml_cfg));
        assert_eq!(
            config.exclude_selectors,
            vec![".from-config", "[data-specimen]", ".demo"]
        );

        let without = PipelineConfig::from_args_and_config(
            &Args::parse_from(["auditmysite", "https://example.com"]),
            None,
        );
        assert!(without.exclude_selectors.is_empty());
        assert_ne!(config.audit_signature(), without.audit_signature());
    }

    #[test]
    fn from_args_and_config_defaults_rule_filter_to_allow_all_without_toml_config() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let config = PipelineConfig::from_args_and_config(&args, None);

        assert!(config.rule_filter.should_run("color-contrast"));
        assert!(config.rule_filter.should_run("css-overflow-hidden"));
    }

    #[test]
    fn from_args_and_config_folds_legacy_ignore_field_into_disabled_rules() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let toml_cfg = crate::cli::config::Config {
            rules: crate::cli::config::RulesConfig {
                disabled: vec!["color-contrast".to_string()],
                ignore: Some(vec!["image-alt".to_string()]),
                enabled_only: vec![],
            },
            ..Default::default()
        };
        let config = PipelineConfig::from_args_and_config(&args, Some(&toml_cfg));

        assert!(!config.rule_filter.should_run("color-contrast"));
        assert!(!config.rule_filter.should_run("image-alt"));
        assert!(config.rule_filter.should_run("landmark-one-main"));
    }

    #[test]
    fn pipeline_config_uses_interactive_budget_from_config() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let config: crate::cli::config::Config = toml::from_str(
            r#"
[interactive]
journey_budget_ms = 1234
"#,
        )
        .unwrap();

        let pipeline = PipelineConfig::from_args_and_config(&args, Some(&config));

        assert_eq!(pipeline.journey_budget_ms, 1234);
    }

    #[test]
    fn pipeline_config_uses_default_interactive_budget_without_config() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);

        let pipeline = PipelineConfig::from_args_and_config(&args, None);

        assert_eq!(
            pipeline.journey_budget_ms,
            crate::a11y_journey::DEFAULT_BUDGET_MS
        );
    }

    /// `TerminalOptions::default()` is documented as the plain
    /// `auditmysite <URL>` run; this holds the two together.
    #[test]
    fn terminal_options_default_is_the_default_cli_run() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let from_args = TerminalOptions::from(&args);
        let default = TerminalOptions::default();
        assert_eq!(from_args.report_level, default.report_level);
        assert_eq!(from_args.lang, default.lang);
        assert_eq!(from_args.color, default.color);
        assert_eq!(from_args.writing_to_file, default.writing_to_file);
    }

    #[test]
    fn terminal_options_follow_the_arguments() {
        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--lang",
            "de",
            "--color",
            "never",
            "--report-level",
            "executive",
            "-o",
            "report.txt",
        ]);
        let options = TerminalOptions::from(&args);
        assert_eq!(options.report_level, args.report_level);
        assert_ne!(
            options.report_level,
            TerminalOptions::default().report_level
        );
        assert_eq!(options.lang, "de");
        assert_eq!(options.color, ColorMode::Never);
        assert!(options.writing_to_file);
        assert_eq!(ColorMode::from(ColorPolicy::Always), ColorMode::Always);
    }

    #[test]
    fn batch_config_derives_pool_wait_from_timeout() {
        use clap::Parser;
        let args = Args::parse_from(["auditmysite", "https://example.com", "-c", "2", "-t", "120"]);
        let config = BatchConfig::from(&args);
        assert_eq!(config.pool_config.acquire_timeout_secs, 615);

        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let config = BatchConfig::from(&args);
        assert_eq!(
            config.pool_config.acquire_timeout_secs,
            pool_acquire_timeout_secs(30)
        );
    }
}
