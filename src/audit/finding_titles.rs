//! Titles of the findings outside the WCAG taxonomy (#898, #1039, #406).
//!
//! The key is the finding's `rule_id` (`<source>.<check>`); the text is the
//! `finding-title-*` message of `locales/{en,de}/finding-titles.ftl`.
//! [`finding_title`] is the only source: the analysis bakes its English into
//! the JSON `title`, the PDF and the table call it with the run language.

/// `rule_id` with every character other than an ASCII letter or digit written
/// as `-`: the key part of its title and its fix.
fn key_id(rule_id: &str) -> String {
    rule_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Fluent key of the title of `rule_id`.
fn title_key(rule_id: &str) -> String {
    format!("finding-title-{}", key_id(rule_id))
}

/// The written fix of the finding with `rule_id` in the requested language
/// (`finding-fix-*` of `locales/{en,de}/finding-fixes.ftl`), or `None` for a
/// check without one (#1039).
pub fn finding_fix(rule_id: &str, en: bool) -> Option<String> {
    crate::i18n::try_text_args(&format!("finding-fix-{}", key_id(rule_id)), &[], en)
}

/// Title of the finding with `rule_id` in the requested language, or `None`
/// for checks whose title is page content or open-ended (budget metrics,
/// library names, the SEO heading issues with their own text function): those
/// keep the stored title.
pub fn finding_title(rule_id: &str, en: bool) -> Option<String> {
    use fluent_bundle::FluentValue;
    let text =
        |key: &str, args: &[(&str, FluentValue<'_>)]| crate::i18n::try_text_args(key, args, en);

    // A missing CSP directive is named, so findings for different directives
    // do not share one title.
    if let Some(directive) = rule_id
        .strip_prefix("security.headers.missing_")
        .filter(|directive| crate::security::CSP_REQUIRED_DIRECTIVES.contains(directive))
    {
        return text(
            "finding-title-security-csp-missing-directive",
            &[("directive", FluentValue::from(directive.to_string()))],
        );
    }
    // A missing required property names the structured-data feature.
    if let Some(rest) = rule_id.strip_prefix("seo.schema.schema_rule_missing_") {
        let feature = crate::seo::schema_rules::SchemaFeature::ALL
            .into_iter()
            .filter(|feature| {
                rest.strip_prefix(feature.key())
                    .is_some_and(|tail| tail.starts_with('_'))
            })
            .max_by_key(|feature| feature.key().len())?;
        return text(
            "finding-title-seo-schema-rule-missing",
            &[(
                "feature",
                FluentValue::from(crate::seo::schema_rules::feature_label(feature, en)),
            )],
        );
    }
    // HTML conformance: the checker's rule set is open-ended, so the title is
    // that of the rule family; an assertion also names its rule id, so two
    // rules of one family never share a title.
    if rule_id.starts_with("html_conform.") {
        let family = crate::html_conform::recommendation::family(rule_id).unwrap_or("generic");
        let family_title = text(&format!("finding-title-html-conform-{family}"), &[])?;
        return match rule_id.strip_prefix("html_conform.assertion.") {
            Some(rule) => text(
                "finding-title-html-conform-rule",
                &[
                    ("family", FluentValue::from(family_title)),
                    ("rule", FluentValue::from(rule.to_string())),
                ],
            ),
            None => Some(family_title),
        };
    }
    text(&title_key(rule_id), &[])
}

/// The title a report shows as a heading or table cell: [`finding_title`]
/// without the checker's rule id an HTML conformance assertion carries in
/// parentheses ("Redundant ARIA role (roles.unnecessary-list)", #1039). The
/// stored title keeps the id, so two rules of one family stay apart in the
/// JSON; a reader gets the family's title. `None` like [`finding_title`].
pub fn display_title(rule_id: &str, en: bool) -> Option<String> {
    if rule_id.starts_with("html_conform.assertion.") {
        let family = crate::html_conform::recommendation::family(rule_id).unwrap_or("generic");
        return crate::i18n::try_text_args(
            &format!("finding-title-html-conform-{family}"),
            &[],
            en,
        );
    }
    finding_title(rule_id, en)
}

/// The element kinds a name or label finding names in its display title.
/// Only roles Chrome reports for the elements those rules check; any other
/// role leaves the rule title.
const NAMED_KINDS: [&str; 6] = ["link", "button", "textbox", "combobox", "checkbox", "radio"];

/// The kind of element a Chrome AX role stands for, if the display title names
/// it: a search box is a text field.
fn element_kind(role: &str) -> Option<&'static str> {
    let role = if role == "searchbox" { "textbox" } else { role };
    NAMED_KINDS.iter().copied().find(|kind| *kind == role)
}

/// Which family of rules carries a kind-specific display title: a missing
/// accessible name (`name`) or a missing label (`label`); `None` for every
/// other rule, whose title stays as it is.
fn display_family(rule_id: &str) -> Option<&'static str> {
    match rule_id {
        "links/name-missing" | "buttons/name-missing" | "names/required-missing" => Some("name"),
        "forms/label-missing" => Some("label"),
        _ => None,
    }
}

/// Whether the findings of `rule_id` get their element roles tallied for the
/// display title ([`finding_display_title`]).
pub fn tallies_roles(rule_id: &str) -> bool {
    display_family(rule_id).is_some()
}

/// How far a finding reaches: the pages it occurs on of the pages audited, and
/// its occurrences (on every page together).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindingReach {
    pub pages_affected: usize,
    pub pages_audited: usize,
    pub occurrences: usize,
}

/// The reach as a phrase ("on all 5 audited pages", "on 2 of 5 pages",
/// "20 occurrences on this page"), or `None` where it says nothing: a single
/// occurrence on the one page audited.
fn reach_text(reach: FindingReach, en: bool) -> Option<String> {
    use fluent_bundle::FluentValue;
    let n = |value: usize| FluentValue::from(value as i64);
    let (key, args): (&str, Vec<(&str, FluentValue<'_>)>) = if reach.pages_audited > 1 {
        if reach.pages_affected >= reach.pages_audited {
            (
                "finding-reach-all-pages",
                vec![("pages", n(reach.pages_audited))],
            )
        } else {
            (
                "finding-reach-some-pages",
                vec![
                    ("affected", n(reach.pages_affected)),
                    ("audited", n(reach.pages_audited)),
                ],
            )
        }
    } else if reach.occurrences > 1 {
        (
            "finding-reach-this-page",
            vec![("count", n(reach.occurrences))],
        )
    } else {
        return None;
    };
    crate::i18n::try_text_args(key, &args, en)
}

/// The title a decision maker reads for a finding (#1170): the kind of
/// element and the reach instead of the rule title alone, from measured data.
///
/// For the rules of a missing name or label, when every observed element is of
/// one kind (`roles`, the tally of their AX roles), the title names that kind
/// ("Link without an accessible name"); with mixed kinds, none or an unnamed
/// one it is `rule_title`. The reach follows after a comma. Every other rule
/// keeps `rule_title`. The JSON `title` is not this one: it stays the English
/// rule title.
pub fn finding_display_title(
    rule_id: &str,
    rule_title: &str,
    roles: &std::collections::BTreeMap<String, usize>,
    reach: FindingReach,
    en: bool,
) -> String {
    use fluent_bundle::FluentValue;
    let Some(family) = display_family(rule_id) else {
        return rule_title.to_string();
    };
    let one_kind = roles
        .keys()
        .next()
        .and_then(|role| element_kind(role))
        .filter(|kind| roles.keys().all(|role| element_kind(role) == Some(*kind)));
    let title = one_kind
        .and_then(|kind| {
            crate::i18n::try_text_args(&format!("finding-kind-title-{family}-{kind}"), &[], en)
        })
        .unwrap_or_else(|| rule_title.to_string());
    match reach_text(reach, en) {
        Some(reach) => crate::i18n::try_text_args(
            "finding-title-with-reach",
            &[
                ("title", FluentValue::from(title.clone())),
                ("reach", FluentValue::from(reach)),
            ],
            en,
        )
        .unwrap_or(title),
        None => title,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::finding_contract::kind_code;

    /// Every variant of an enum, as an array. The match has no wildcard arm,
    /// so a new variant fails to compile here until it is listed.
    macro_rules! all_variants {
        ($ty:path: $($variant:ident),+ $(,)?) => {{
            use $ty as Kind;
            let listed = |kind: Kind| match kind {
                $(Kind::$variant)|+ => (),
            };
            let all = [$(Kind::$variant),+];
            all.iter().copied().for_each(listed);
            all
        }};
    }

    /// The first string literal after each `call` in `source`: the issue type
    /// argument of the collectors that build their issues from literals.
    fn first_literals_after(source: &str, call: &str) -> Vec<String> {
        source
            .split(call)
            .skip(1)
            .filter_map(|rest| rest.split('"').nth(1).map(str::to_string))
            .collect()
    }

    /// Every rule id outside the WCAG taxonomy this build can emit and that is
    /// titled from `finding-titles.ftl` (the collectors of
    /// `finding_contract::collect`).
    fn emitted_rule_ids() -> Vec<String> {
        let mut ids = vec!["best_practices.console.error".to_string()];
        // seo.meta: one field per kind.
        {
            use crate::seo::meta::MetaIssueKind as K;
            let field = |kind: K| match kind {
                K::MissingTitle | K::TitleTooShort | K::TitleTooLong => "title",
                K::MissingDescription | K::DescriptionTooShort | K::DescriptionTooLong => {
                    "description"
                }
                K::MissingViewport => "viewport",
                K::InvalidTwitterCard => "twitter:card",
                K::RelativeOgImage => "og:image",
            };
            for kind in all_variants!(crate::seo::meta::MetaIssueKind:
                MissingTitle, TitleTooShort, TitleTooLong, MissingDescription,
                DescriptionTooShort, DescriptionTooLong, MissingViewport,
                InvalidTwitterCard, RelativeOgImage)
            {
                ids.push(format!("seo.meta.{}", field(kind)));
            }
        }
        // seo.technical and seo.page_health build their issues from literals.
        let technical = first_literals_after(
            include_str!("../seo/technical.rs")
                .split("#[cfg(test)]")
                .next()
                .unwrap(),
            "push_issue(",
        );
        assert!(technical.len() >= 20, "{technical:?}");
        ids.extend(
            technical
                .iter()
                .filter(|t| !t.contains('{'))
                .map(|t| format!("seo.technical.{t}")),
        );
        // AMP issues are `amp_<issue>` for each issue the collector matches.
        let amp = include_str!("../seo/technical.rs")
            .split("fn collect_amp_issues")
            .nth(1)
            .and_then(|rest| rest.split("\nfn ").next())
            .unwrap();
        let amp: Vec<&str> = amp
            .split(" => ")
            .filter_map(|before| before.trim_end().strip_suffix('"')?.rsplit('"').next())
            .collect();
        assert!(amp.len() >= 5, "{amp:?}");
        ids.extend(amp.iter().map(|issue| format!("seo.technical.amp_{issue}")));
        let page_health = first_literals_after(
            include_str!("../seo/page_health/issues.rs")
                .split("#[cfg(test)]")
                .next()
                .unwrap(),
            "add(",
        );
        assert!(page_health.len() >= 40, "{page_health:?}");
        ids.extend(page_health.iter().map(|t| format!("seo.page_health.{t}")));
        // seo.schema: the structural issues and one missing property per feature.
        for issue_type in [
            "jsonld_parse_error",
            "jsonld_invalid_root",
            "jsonld_missing_context",
            "jsonld_missing_type",
            "jsonld_graph_not_array",
            "jsonld_empty_document",
        ] {
            ids.push(format!("seo.schema.{issue_type}"));
        }
        for feature in crate::seo::schema_rules::SchemaFeature::ALL {
            ids.push(format!(
                "seo.schema.schema_rule_missing_{}_name",
                feature.key()
            ));
        }
        for check in crate::security::all_security_check_ids() {
            let (_, issue_type) = check.split_once(':').unwrap();
            ids.push(format!("security.headers.{issue_type}"));
        }
        {
            use crate::mobile::MobileIssueKind as K;
            for kind in all_variants!(crate::mobile::MobileIssueKind:
                MissingViewport, ImproperViewport, NotScalable, SmallTargets, SmallFonts,
                HorizontalScroll)
            {
                let issue_type = match kind {
                    K::MissingViewport => "missing_viewport",
                    K::ImproperViewport => "improper_viewport",
                    K::NotScalable => "not_scalable",
                    K::SmallTargets => "small_targets",
                    K::SmallFonts => "small_fonts",
                    K::HorizontalScroll => "horizontal_scroll",
                };
                ids.push(format!("mobile.{issue_type}"));
            }
        }
        #[cfg(feature = "experimental")]
        for kind in all_variants!(crate::ux::UxIssueKind:
            NoCta, CompetingCtas, GenericLinks, NoH1, MultipleH1, HeadingSkips, LargeDom,
            LittleText, NoSubheadings, NoContact, NoImprint, NoPrivacy, WeakTrustSignals,
            TooManyLinks, TooManyInteractive)
        {
            ids.push(format!("ux.{}", kind_code(&kind)));
        }
        #[cfg(feature = "experimental")]
        for kind in all_variants!(crate::page_flow::FrictionKind:
            NoH1, MultipleH1, VagueH1, NoPageTitle, LittleEarlyText, NavigationMissing,
            MainHiddenFromAx, MainMissing, FewSubheadings, NoBreadcrumb, NoFooter,
            GenericLinks, EmptyLinks, DuplicateLinks, TooManyLinks, UnnamedButtons,
            UnlabeledForms, GenericButtons, NoCallToAction, CompetingCtas, BlockingDialog,
            ComplexForm)
        {
            ids.push(format!("journey.{}", kind_code(&kind)));
        }
        #[cfg(feature = "experimental")]
        for kind in all_variants!(crate::commerce::CommerceFindingKind:
            MissingPrice, MissingAvailability, MissingShippingDetails, MissingReturnPolicy,
            MissingReviews, MissingImpressumLink, MissingAgbLink, MissingWiderrufLink,
            MissingShippingPageLink, MissingPaymentLink, MissingContactLink)
        {
            ids.push(format!("commerce.{}", kind_code(&kind)));
        }
        for kind in all_variants!(crate::privacy::findings::PrivacyFindingKind:
            TrackingRequestBeforeConsent, NonEssentialStorageBeforeConsent, VisitorIpTransfer,
            ThirdCountryProvider, NoConsentPlatformDetected, PolicyLinkMissing,
            PolicyLinkUnusable)
        {
            ids.push(format!("privacy.{}", kind_code(&kind)));
        }
        for kind in all_variants!(crate::audit::interactive_finding::InteractiveFindingKind:
            HiddenFocusableAriaHidden, HiddenFocusableInert, HiddenFocusableStyle,
            FocusIndicatorNotDetected, TabOrderBackwardJumps, FocusTrapNotEntered,
            FocusTrapBackgroundNotHidden, FocusTrapEscaped, FocusTrapEscapeNotClosing,
            FocusRestorationLostToBody, ModalNotOpened, MenuNotOpened, MenuFocusNotMoved,
            MenuEscapeNotClosing, TabsSelectionNotMoved, TabsFocusNotOnTab,
            DisclosureNotOpened, DisclosureNotClosed, DisclosureStateWithoutContent,
            DisclosureContentWithoutState, SpaNoAnnouncementSignal, SpaTitleUnchanged,
            SpaFocusNotMoved, SkipLinkFocusNotMoved, FormErrorSilentFailure,
            FormErrorInvalidWithoutLiveRegion, FormErrorUnlinkedFields,
            FormErrorFocusNotManaged, FormErrorLiveRegionLateInsertion,
            AddToCartNoStatusAnnouncement, AddToCartNoFeedbackDetected,
            QuantityStepperKeyboardInoperable, QuantityStepperValueNotExposed,
            LinkTextGeneric, LinkTextDuplicate, HeadingMissingH1, HeadingMultipleH1,
            HeadingLevelSkip, LandmarkMissingMain, LandmarkNavWithoutLabels,
            LandmarkDuplicateUnique, MediaControlsMissingName, MediaControlsNotReachable)
        {
            ids.push(format!("a11y_journey.{}", kind_code(&kind)));
        }
        // screen_reader: one check per criterion the analyzer tags, plus the
        // untagged reading-sequence issue.
        let criteria = first_literals_after(
            include_str!("../screen_reader/analyzer.rs")
                .split("#[cfg(test)]")
                .next()
                .unwrap(),
            "wcag_criterion: Some(",
        );
        assert!(!criteria.is_empty());
        ids.extend(
            criteria
                .iter()
                .map(|c| format!("screen_reader.wcag_{}", c.replace('.', "_"))),
        );
        ids.push("screen_reader.reading_sequence".to_string());
        // html_conform: every family is covered by
        // `every_html_conform_family_has_a_title`.
        ids.extend(
            [
                "html_conform.parser.html5",
                "html_conform.schema.html5",
                "html_conform.assertion.roles.unnecessary-list",
                "html_conform.assertion.future.rule",
            ]
            .map(str::to_string),
        );
        ids.sort();
        ids.dedup();
        ids
    }

    #[test]
    fn every_emitted_rule_id_has_an_english_and_a_german_title() {
        for rule_id in emitted_rule_ids() {
            let en = finding_title(&rule_id, true)
                .unwrap_or_else(|| panic!("{rule_id}: no English title"));
            let de = finding_title(&rule_id, false)
                .unwrap_or_else(|| panic!("{rule_id}: no German title"));
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "{rule_id}: German in the English title {en:?}"
            );
            assert_ne!(en, de, "{rule_id}: German title is the English one");
        }
    }

    /// #1039: every page-health and mobile check has a written fix in both
    /// languages, so it reaches the action plan like the security checks.
    #[test]
    fn every_page_health_and_mobile_check_has_a_fix() {
        let ids: Vec<String> = emitted_rule_ids()
            .into_iter()
            .filter(|id| id.starts_with("seo.page_health.") || id.starts_with("mobile."))
            .collect();
        assert!(ids.len() >= 46, "{ids:?}");
        for rule_id in ids {
            let en =
                finding_fix(&rule_id, true).unwrap_or_else(|| panic!("{rule_id}: no English fix"));
            let de =
                finding_fix(&rule_id, false).unwrap_or_else(|| panic!("{rule_id}: no German fix"));
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "{rule_id}: German in the English fix {en:?}"
            );
            assert_ne!(en, de, "{rule_id}: German fix is the English one");
        }
    }

    /// Every fix belongs to a rule id this build emits, and both locales
    /// define the same fixes.
    #[test]
    fn fixes_belong_to_emitted_rule_ids() {
        let en = crate::i18n::ftl_entries(include_str!("../../locales/en/finding-fixes.ftl"));
        let de = crate::i18n::ftl_entries(include_str!("../../locales/de/finding-fixes.ftl"));
        assert_eq!(en, de);
        let keys: Vec<String> = emitted_rule_ids()
            .iter()
            .map(|id| format!("finding-fix-{}", key_id(id)))
            .collect();
        for entry in &en {
            let key = entry.split(['.', ' ']).next().unwrap_or(entry);
            assert!(keys.iter().any(|k| k == key), "{entry}: no such rule id");
        }
    }

    #[test]
    fn every_html_conform_family_has_a_title() {
        for family in crate::html_conform::recommendation::FAMILIES
            .iter()
            .chain(&["generic"])
        {
            for en in [true, false] {
                let key = format!("finding-title-html-conform-{family}");
                assert!(crate::i18n::try_text_args(&key, &[], en).is_some(), "{key}");
            }
        }
    }

    #[test]
    fn both_locales_define_the_same_titles() {
        let en = crate::i18n::ftl_entries(include_str!("../../locales/en/finding-titles.ftl"));
        let de = crate::i18n::ftl_entries(include_str!("../../locales/de/finding-titles.ftl"));
        assert_eq!(en, de);
    }

    /// Open-ended checks keep their stored title.
    #[test]
    fn open_ended_checks_have_no_title() {
        for rule_id in [
            "performance.budget.lcp",
            "best_practices.vulnerable_libraries.jquery",
            "seo.headings.missing_h1",
        ] {
            assert_eq!(finding_title(rule_id, true), None, "{rule_id}");
        }
    }

    /// Two rules of one family, and two missing CSP directives, never share a
    /// title (#1039).
    #[test]
    fn titles_distinguish_rules_of_one_family() {
        for en in [true, false] {
            let a = finding_title("html_conform.assertion.roles.unnecessary-list", en).unwrap();
            let b =
                finding_title("html_conform.assertion.roles.unnecessary-navigation", en).unwrap();
            assert_ne!(a, b);
            let titles: std::collections::BTreeSet<String> =
                crate::security::CSP_REQUIRED_DIRECTIVES
                    .iter()
                    .map(|directive| {
                        let title =
                            finding_title(&format!("security.headers.missing_{directive}"), en)
                                .unwrap();
                        assert!(title.contains(directive), "{title}");
                        title
                    })
                    .collect();
            assert_eq!(titles.len(), crate::security::CSP_REQUIRED_DIRECTIVES.len());
        }
    }

    /// The JSON title is the English one: the readable check code is gone
    /// (#1039: "Wcag 1 3 6", "Http1 only", "Large dom", "Description").
    #[test]
    fn english_titles_replace_the_check_code() {
        for (rule_id, expected) in [
            (
                "screen_reader.wcag_1_3_6",
                "Screen reader: page regions not identifiable",
            ),
            ("seo.page_health.http1_only", "HTTP/1.1 only"),
            ("seo.page_health.large_dom", "Large DOM"),
            ("seo.meta.description", "Meta description"),
            (
                "html_conform.schema.html5",
                "Element or attribute not allowed here",
            ),
        ] {
            assert_eq!(finding_title(rule_id, true).as_deref(), Some(expected));
        }
    }
    fn roles(entries: &[(&str, usize)]) -> std::collections::BTreeMap<String, usize> {
        entries.iter().map(|(r, n)| (r.to_string(), *n)).collect()
    }

    fn reach(affected: usize, audited: usize, occurrences: usize) -> FindingReach {
        FindingReach {
            pages_affected: affected,
            pages_audited: audited,
            occurrences,
        }
    }

    const RULE_TITLE: &str = "Form control, link, or button without an accessible name";

    /// #1170: one kind names the element, with the reach of the finding.
    #[test]
    fn display_title_names_one_kind_and_the_reach_en() {
        let title = |roles: &_, reach| {
            finding_display_title("links/name-missing", RULE_TITLE, roles, reach, true)
        };
        let links = roles(&[("link", 20)]);
        assert_eq!(
            title(&links, reach(5, 5, 20)),
            "Link without an accessible name, on all 5 audited pages"
        );
        assert_eq!(
            title(&links, reach(2, 5, 20)),
            "Link without an accessible name, on 2 of 5 pages"
        );
        assert_eq!(
            title(&links, reach(1, 1, 20)),
            "Link without an accessible name, 20 occurrences on this page"
        );
        assert_eq!(
            title(&links, reach(1, 1, 1)),
            "Link without an accessible name"
        );
        assert_eq!(
            finding_display_title(
                "names/required-missing",
                "Interactive element without a meaningful name",
                &roles(&[("textbox", 2), ("searchbox", 1)]),
                reach(1, 1, 3),
                true
            ),
            "Text field without an accessible name, 3 occurrences on this page"
        );
        assert_eq!(
            finding_display_title(
                "forms/label-missing",
                "Missing form labels",
                &roles(&[("checkbox", 4)]),
                reach(3, 3, 4),
                true
            ),
            "Checkbox without a label, on all 3 audited pages"
        );
    }

    #[test]
    fn display_title_names_one_kind_and_the_reach_de() {
        assert_eq!(
            finding_display_title(
                "buttons/name-missing",
                "Formularelement, Link oder Button ohne zugänglichen Namen",
                &roles(&[("button", 7)]),
                reach(2, 5, 7),
                false
            ),
            "Button ohne zugänglichen Namen, auf 2 von 5 Seiten"
        );
        assert_eq!(
            finding_display_title(
                "links/name-missing",
                "x",
                &roles(&[("link", 20)]),
                reach(4, 4, 20),
                false
            ),
            "Link ohne zugänglichen Namen, auf allen 4 geprüften Seiten"
        );
        assert_eq!(
            finding_display_title(
                "forms/label-missing",
                "x",
                &roles(&[("textbox", 6)]),
                reach(1, 1, 6),
                false
            ),
            "Textfeld ohne Beschriftung, 6 Vorkommen auf dieser Seite"
        );
    }

    /// Mixed, unknown or unmeasured kinds keep the rule title, and a rule
    /// outside the name and label rules is never touched.
    #[test]
    fn display_title_never_guesses_a_kind() {
        for en in [true, false] {
            let one = reach(1, 1, 1);
            let rule = |roles: &_| {
                finding_display_title("names/required-missing", "Rule title", roles, one, en)
            };
            assert_eq!(rule(&roles(&[("link", 1), ("button", 1)])), "Rule title");
            assert_eq!(rule(&roles(&[("slider", 3)])), "Rule title");
            assert_eq!(rule(&roles(&[("link", 3), ("slider", 1)])), "Rule title");
            assert_eq!(rule(&roles(&[])), "Rule title");
            assert_eq!(
                finding_display_title(
                    "images/alt-missing",
                    "Rule title",
                    &roles(&[("image", 4)]),
                    reach(5, 5, 4),
                    en
                ),
                "Rule title"
            );
        }
    }

    /// The English texts of the display title carry no German umlaut.
    #[test]
    fn english_display_titles_have_no_umlauts() {
        let en = include_str!("../../locales/en/finding-titles.ftl");
        assert!(!en.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
        for family in ["name", "label"] {
            for kind in NAMED_KINDS {
                if family == "label" && matches!(kind, "link" | "button") {
                    continue;
                }
                for en in [true, false] {
                    let key = format!("finding-kind-title-{family}-{kind}");
                    assert!(crate::i18n::try_text_args(&key, &[], en).is_some(), "{key}");
                }
            }
        }
    }
}
