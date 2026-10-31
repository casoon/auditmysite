//! `AuditModule` implementation for SEO collection (Phase A3 / #332).
//!
//! Wraps `analyze_seo` and surfaces the result as `ModuleData::Seo`.
//! Failure mirrors the previous inline behavior: log a warning and yield
//! `ModuleData::None` so downstream consumers see absence-of-data rather
//! than a propagated error.

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{AuditReport, PipelineConfig};
use crate::error::Result;

use super::analyze_seo;

pub struct SeoModule;

#[async_trait]
impl AuditModule for SeoModule {
    fn id(&self) -> &'static str {
        "seo"
    }

    fn label(&self) -> &'static str {
        "SEO"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_seo
    }

    fn depends_on(&self) -> &'static [&'static str] {
        &["journey"]
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_seo(ctx.page, ctx.url, ctx.collection).await {
            Ok(seo) => Ok(ModuleData::Seo(Box::new(seo))),
            Err(e) => {
                warn!("SEO analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }

    fn derive(&self, report: &mut AuditReport, _locale: &str) -> Result<()> {
        // Heading structure findings come from the shared rules (#724); the
        // score, content profile and SERP view built at collection time read
        // the heading issues, so they are rebuilt from the replaced list.
        let url = report.url.clone();
        // The transport is the browser document's, taken from the shared
        // navigation capture, not from `window.location` (#775).
        let https = report.document_is_https();
        if let Some(seo) = report.discoverability.seo.as_mut() {
            if seo.technical.https != https {
                seo.technical.https = https;
                seo.technical.issues =
                    super::technical::collect_technical_issues(&seo.technical, true);
            }
            super::headings::apply_shared_findings(
                &mut seo.headings,
                &report.accessibility.wcag_results,
            );
            super::rescore(seo, report.run_context.indexability);
            seo.content_profile = Some(crate::seo::profile::build_content_profile(seo, "en"));
            seo.serp = Some(crate::seo::serp::build_serp_analysis(seo, &url, "en"));
        }

        let Some(page_intent) = report.journey.as_ref().map(|journey| journey.page_intent) else {
            return Ok(());
        };
        let Some(seo) = report.discoverability.seo.as_mut() else {
            return Ok(());
        };

        let fit = crate::seo::schema_fit::assess_schema_fit_with_facts(
            &url,
            page_intent,
            &seo.structured_data,
            &seo.structured_data.visible_facts,
        );
        crate::seo::schema::refresh_rule_assessments(
            &mut seo.structured_data,
            fit.product_context(),
        );
        seo.structured_data.fit_assessment = Some(fit);
        seo.content_profile = Some(crate::seo::profile::build_content_profile(seo, "en"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{NavigationSnapshot, RedirectHop};
    use crate::seo::SeoAnalysis;
    use crate::wcag::WcagLevel;
    use crate::wcag::WcagResults;

    fn report_with(entry: &str, chain: &[(&str, u16)], served: &str) -> Box<AuditReport> {
        let mut report = Box::new(AuditReport::new(
            entry.to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            1,
        ));
        report.accessibility.execution.navigation = NavigationSnapshot {
            final_url: Some(served.to_string()),
            redirect_chain: chain
                .iter()
                .map(|(url, status)| RedirectHop {
                    url: (*url).to_string(),
                    status: *status,
                })
                .collect(),
            ..Default::default()
        };
        // What `window.location` reports after a redirect that ended on TLS.
        let mut seo = SeoAnalysis::default();
        seo.technical.https = true;
        report.discoverability.seo = Some(seo);
        report
    }

    fn derived(entry: &str, chain: &[(&str, u16)], served: &str) -> (bool, bool) {
        let mut report = report_with(entry, chain, served);
        SeoModule.derive(&mut report, "en").unwrap();
        let technical = &report.discoverability.seo.as_ref().unwrap().technical;
        (
            technical.https,
            technical.issues.iter().any(|i| i.issue_type == "no_https"),
        )
    }

    /// An `http://` entry that the server upgrades is served securely: the
    /// requested URL must not decide (#775).
    #[test]
    fn an_upgrade_to_https_is_secure() {
        assert_eq!(
            derived(
                "http://a.test/",
                &[("http://a.test/", 301)],
                "https://a.test/"
            ),
            (true, false)
        );
    }

    /// The other direction: an `https://` entry that ends on `http://` is not,
    /// and neither is one that dropped to cleartext and came back.
    #[test]
    fn a_downgrade_to_http_is_not_secure() {
        let flagged = (false, true);
        assert_eq!(
            derived(
                "https://a.test/",
                &[("https://a.test/", 302)],
                "http://a.test/"
            ),
            flagged
        );
        assert_eq!(
            derived(
                "https://a.test/",
                &[("https://a.test/", 302), ("http://a.test/x", 301)],
                "https://a.test/"
            ),
            flagged
        );
    }
}
