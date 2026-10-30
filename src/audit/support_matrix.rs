//! Support matrix of the opt-in extra checks (#788).
//!
//! One table says, for every extra check a user can request, in which mode it
//! runs (single URL, batch, cache hit, replay), what it needs, and what the
//! run does where it cannot execute. `Args::validate` reads it to reject an
//! unsupported combination before any work starts, so a switch that is set
//! but never executed — and leaves no outcome — cannot reach the pipeline.
//!
//! The table holds plain booleans, not CLI types: analysis code must not
//! import `cli`.

/// An opt-in check beyond the base accessibility audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraCheck {
    /// `--isolate-third-party-impact` (#531): one reload per third-party origin.
    ThirdPartyIsolation,
    /// `--check-ssr-content` (#534): one reload with JavaScript disabled.
    SsrContent,
    /// `--ai-transparency`: C2PA image provenance.
    AiTransparency,
    /// `--dns-check` (#545): once per unique host.
    Dns,
}

/// How a run reaches the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// One URL, measured live.
    Single,
    /// Several URLs (`--sitemap`, `--url-file`, `--crawl`), measured live.
    Batch,
    /// `--reuse-cache` hit on a single URL: the stored report is returned.
    Cache,
    /// Re-derivation from a stored snapshot (#790); there is no live page.
    Replay,
}

impl Mode {
    pub const ALL: [Mode; 4] = [Mode::Single, Mode::Batch, Mode::Cache, Mode::Replay];
}

/// What a mode does with a requested check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Support {
    /// The check executes (or, for a cache hit, was executed by the stored run
    /// — the cache signature contains the flag) and records its outcome.
    Runs,
    /// The CLI refuses the combination up front with this reason.
    Rejected(&'static str),
    /// The mode cannot measure it; the report records a not-run outcome with
    /// this reason code instead of leaving the check silent.
    NotRun(&'static str),
}

/// What must be active for the check to have anything to attach to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prerequisite {
    None,
    /// Performance checking: the check diffs against a baseline TBT.
    Performance,
    /// SEO checking: the finding is attached to `content_visibility`.
    Seo,
}

impl Prerequisite {
    fn unmet_reason(self) -> Option<&'static str> {
        match self {
            Prerequisite::None => None,
            Prerequisite::Performance => Some(
                "needs performance checking (use --full or --performance, without --skip-performance)",
            ),
            Prerequisite::Seo => Some("needs SEO checking (use --full or --seo)"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExtraCheckSpec {
    pub check: ExtraCheck,
    pub flag: &'static str,
    /// Id of the subcheck in `scope.requested_subchecks` / `module_runs`;
    /// `None` when the check is a module of its own.
    pub subcheck: Option<&'static str>,
    pub prerequisite: Prerequisite,
    pub single: Support,
    pub batch: Support,
    pub cache: Support,
    pub replay: Support,
}

impl ExtraCheckSpec {
    pub fn support(&self, mode: Mode) -> Support {
        match mode {
            Mode::Single => self.single,
            Mode::Batch => self.batch,
            Mode::Cache => self.cache,
            Mode::Replay => self.replay,
        }
    }
}

const REPLAY_DOES_NOT_MEASURE: Support = Support::NotRun("replay_does_not_measure");

pub const MATRIX: &[ExtraCheckSpec] = &[
    ExtraCheckSpec {
        check: ExtraCheck::ThirdPartyIsolation,
        flag: "--isolate-third-party-impact",
        subcheck: Some("performance.third_party_isolation"),
        prerequisite: Prerequisite::Performance,
        single: Support::Runs,
        batch: Support::Runs,
        cache: Support::Runs,
        replay: REPLAY_DOES_NOT_MEASURE,
    },
    ExtraCheckSpec {
        check: ExtraCheck::SsrContent,
        flag: "--check-ssr-content",
        subcheck: Some("content_visibility.ssr_gap"),
        prerequisite: Prerequisite::Seo,
        single: Support::Runs,
        batch: Support::Runs,
        cache: Support::Runs,
        replay: REPLAY_DOES_NOT_MEASURE,
    },
    ExtraCheckSpec {
        check: ExtraCheck::AiTransparency,
        flag: "--ai-transparency",
        subcheck: None,
        prerequisite: Prerequisite::None,
        single: Support::Runs,
        batch: Support::Rejected(
            "is single-URL only: fetching and parsing every image of every page is not run in \
             batch mode",
        ),
        cache: Support::Runs,
        replay: REPLAY_DOES_NOT_MEASURE,
    },
    ExtraCheckSpec {
        check: ExtraCheck::Dns,
        flag: "--dns-check",
        subcheck: None,
        prerequisite: Prerequisite::None,
        single: Support::Runs,
        batch: Support::Runs,
        cache: Support::Runs,
        replay: REPLAY_DOES_NOT_MEASURE,
    },
];

/// The extra checks a run asks for, and the context that decides support.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtraCheckRequest {
    pub isolate_third_party_impact: bool,
    pub check_ssr_content: bool,
    pub ai_transparency: bool,
    pub dns_check: bool,
    pub performance_active: bool,
    pub seo_active: bool,
    pub batch: bool,
    /// A single-URL run that may be answered from the cache.
    pub reuse_cache: bool,
}

impl ExtraCheckRequest {
    fn requested(&self, check: ExtraCheck) -> bool {
        match check {
            ExtraCheck::ThirdPartyIsolation => self.isolate_third_party_impact,
            ExtraCheck::SsrContent => self.check_ssr_content,
            ExtraCheck::AiTransparency => self.ai_transparency,
            ExtraCheck::Dns => self.dns_check,
        }
    }

    fn prerequisite_met(&self, prerequisite: Prerequisite) -> bool {
        match prerequisite {
            Prerequisite::None => true,
            Prerequisite::Performance => self.performance_active,
            Prerequisite::Seo => self.seo_active,
        }
    }

    /// Reject the first requested check this run cannot execute.
    pub fn validate(&self) -> Result<(), String> {
        for spec in MATRIX.iter().filter(|spec| self.requested(spec.check)) {
            if !self.prerequisite_met(spec.prerequisite) {
                if let Some(reason) = spec.prerequisite.unmet_reason() {
                    return Err(format!("{} {reason}", spec.flag));
                }
            }
            let mut modes = vec![if self.batch {
                Mode::Batch
            } else {
                Mode::Single
            }];
            if self.reuse_cache && !self.batch {
                modes.push(Mode::Cache);
            }
            for mode in modes {
                if let Support::Rejected(reason) = spec.support(mode) {
                    return Err(format!("{} {reason}", spec.flag));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(f: impl FnOnce(&mut ExtraCheckRequest)) -> ExtraCheckRequest {
        let mut r = ExtraCheckRequest {
            performance_active: true,
            seo_active: true,
            ..Default::default()
        };
        f(&mut r);
        r
    }

    #[test]
    fn every_check_is_in_the_matrix_once_with_a_defined_outcome_per_mode() {
        for check in [
            ExtraCheck::ThirdPartyIsolation,
            ExtraCheck::SsrContent,
            ExtraCheck::AiTransparency,
            ExtraCheck::Dns,
        ] {
            let rows: Vec<_> = MATRIX.iter().filter(|s| s.check == check).collect();
            assert_eq!(rows.len(), 1, "{check:?}");
            for mode in Mode::ALL {
                // A reason is never empty: a not-run or rejection is explained.
                match rows[0].support(mode) {
                    Support::Rejected(r) | Support::NotRun(r) => assert!(!r.is_empty()),
                    Support::Runs => {}
                }
            }
        }
    }

    #[test]
    fn a_replay_never_claims_to_have_measured() {
        assert!(MATRIX
            .iter()
            .all(|s| matches!(s.replay, Support::NotRun(_))));
    }

    #[test]
    fn unmet_prerequisites_are_rejected_not_dropped() {
        let e = request(|r| {
            r.isolate_third_party_impact = true;
            r.performance_active = false;
        })
        .validate()
        .unwrap_err();
        assert!(e.contains("--isolate-third-party-impact") && e.contains("performance"));
        let e = request(|r| {
            r.check_ssr_content = true;
            r.seo_active = false;
        })
        .validate()
        .unwrap_err();
        assert!(e.contains("--check-ssr-content") && e.contains("SEO"));
    }

    #[test]
    fn ai_transparency_is_rejected_in_batch_only() {
        assert!(request(|r| r.ai_transparency = true).validate().is_ok());
        let e = request(|r| {
            r.ai_transparency = true;
            r.batch = true;
        })
        .validate()
        .unwrap_err();
        assert!(e.contains("--ai-transparency") && e.contains("single-URL"));
    }

    #[test]
    fn supported_combinations_pass() {
        assert!(request(|r| {
            r.isolate_third_party_impact = true;
            r.check_ssr_content = true;
            r.dns_check = true;
            r.batch = true;
        })
        .validate()
        .is_ok());
        assert!(request(|r| {
            r.isolate_third_party_impact = true;
            r.reuse_cache = true;
        })
        .validate()
        .is_ok());
    }
}
