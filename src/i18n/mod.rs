pub mod number;

#[cfg(test)]
pub(crate) mod detail_texts_export;
#[cfg(test)]
pub(crate) mod label_texts_export;
#[cfg(test)]
mod label_texts_guard;

use std::sync::Arc;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

pub struct I18n {
    locale: String,
    bundle: FluentBundle<Arc<FluentResource>>,
}

impl I18n {
    pub fn new(locale: &str) -> anyhow::Result<Self> {
        let lang: LanguageIdentifier = locale.parse()?;
        let mut bundle = FluentBundle::new(vec![lang]);
        // Disable Unicode bidi isolates so format_pattern doesn't wrap
        // interpolated values in U+2068 / U+2069 isolate marks.
        bundle.set_use_isolating(false);
        // Report texts, the rule texts (the catalogue of `taxonomy::rules`
        // and the explanations of `output::explanations`) the page-health
        // issue texts and HTML validation texts (`seo::page_health`), the SEO
        // issue texts and labels (`seo::technical`, `seo::meta`,
        // `seo::schema`, `seo::robots`, `seo::profile`, `seo::serp`,
        // `seo::schema_fit`, `seo::schema_parity`), the
        // security finding texts and category labels (`security`), the
        // mobile finding texts (`mobile`) and the labels of cross-cutting
        // enums (`labels.ftl`: page intent, execution priority, issue class,
        // subcategory, evidence source and field), the titles of the
        // findings outside the WCAG taxonomy and their fixes
        // (`audit::finding_titles`), the interactive (journey) finding
        // texts (`audit::interactive_finding`), the texts of the batch PDF's
        // consistency and management sections (`batch-consistency.ftl`),
        // the texts of the single PDF's page sections (`single-report.ftl`,
        // `output::pdf::single_report`), the texts of the single report's
        // methodology section (`methodology.ftl`,
        // `output::builder::single::methodology`), the texts of the SEO
        // module detail presentation (`seo-details.ftl`), the texts of the
        // single report's findings overview and system diagnosis
        // (`diagnosis.ftl`, `output::builder::single::diagnosis`,
        // `output::pdf::diagnosis`), the texts of the accessibility detail
        // sections (`accessibility-details.ftl`,
        // `output::pdf::detail_modules::accessibility`), the texts of the
        // management summary's score drivers (`score-drivers.ftl`,
        // `output::pdf::score_drivers`), the texts of the PDF finding
        // cards and shared module detail helpers (`pdf-findings.ftl`,
        // `output::pdf::findings`, `output::pdf::detail_modules`), the texts
        // of the single PDF's frame (`pdf-frame.ftl`, `output::pdf`), of the
        // action plan block (`actions-block.ftl`,
        // `output::builder::single::actions_block`) and of the batch PDF's
        // journey, module, impact, commerce, EN 301 549 and overview sections
        // (`batch-report.ftl`, `output::pdf::batch_report::{interactive,
        // portfolio, appendix, overview}`), the texts of the audit
        // (analysis) layer (`audit-texts.ftl`, `audit::normalized`,
        // `audit::batch_consistency`, `audit::management_risk`) and of the
        // report builder helpers, the batch builder, the PDF detail modules
        // for performance, SEO, tech stack and best practices and the display
        // modes (`output-texts.ftl`, `output::builder::{helpers, batch}`,
        // `output::pdf::detail_modules::{indicators, seo, performance}`,
        // `display`), of the privacy findings (`privacy-findings.ftl`,
        // `privacy::findings`) and of the accessibility analysis layer
        // (`a11y-analysis.ftl`, `screen_reader::analyzer`,
        // `patterns::pattern_message_text`), of the PDF annexes and section
        // helpers (`pdf-sections.ftl`, `output::pdf::{en301549, bik_guide,
        // cover, helpers, appendix}`, `output::pdf::detail_modules::overview`)
        // and of the search experience and the PDF's root cause, problem
        // profile, action plan and WCAG coverage sections (`pdf-analysis.ftl`,
        // `output::search_experience`, `output::pdf::{root_causes,
        // problem_profile, wcag_coverage}`), of the single report's builder
        // helpers (positive signals and aspects, key points, quality note,
        // overall score explanation, page health, performance, security and
        // mobile detail rows, and the terminal table's experimental lines;
        // `report-builder.ftl`, `output::builder::single`, `output::terminal`)
        // and of the interpretation layer (`audit-interpretation.ftl`,
        // `seo::interpretation`, `audit::{interpretation,
        // performance_interpretation, verdict, finding_derive, summary,
        // baseline}`, `seo::headings`, `taxonomy::score_area`) are
        // twenty-nine resources of the one bundle.
        let sources = match locale {
            "en" => [
                include_str!("../../locales/en/report.ftl"),
                include_str!("../../locales/en/rules.ftl"),
                include_str!("../../locales/en/page-health.ftl"),
                include_str!("../../locales/en/seo-issues.ftl"),
                include_str!("../../locales/en/security-issues.ftl"),
                include_str!("../../locales/en/mobile-issues.ftl"),
                include_str!("../../locales/en/labels.ftl"),
                include_str!("../../locales/en/finding-titles.ftl"),
                include_str!("../../locales/en/finding-fixes.ftl"),
                include_str!("../../locales/en/interactive.ftl"),
                include_str!("../../locales/en/batch-consistency.ftl"),
                include_str!("../../locales/en/single-report.ftl"),
                include_str!("../../locales/en/methodology.ftl"),
                include_str!("../../locales/en/seo-details.ftl"),
                include_str!("../../locales/en/diagnosis.ftl"),
                include_str!("../../locales/en/accessibility-details.ftl"),
                include_str!("../../locales/en/score-drivers.ftl"),
                include_str!("../../locales/en/pdf-findings.ftl"),
                include_str!("../../locales/en/pdf-frame.ftl"),
                include_str!("../../locales/en/actions-block.ftl"),
                include_str!("../../locales/en/batch-report.ftl"),
                include_str!("../../locales/en/audit-texts.ftl"),
                include_str!("../../locales/en/a11y-analysis.ftl"),
                include_str!("../../locales/en/output-texts.ftl"),
                include_str!("../../locales/en/privacy-findings.ftl"),
                include_str!("../../locales/en/pdf-sections.ftl"),
                include_str!("../../locales/en/pdf-analysis.ftl"),
                include_str!("../../locales/en/audit-interpretation.ftl"),
                include_str!("../../locales/en/report-builder.ftl"),
            ],
            _ => [
                include_str!("../../locales/de/report.ftl"),
                include_str!("../../locales/de/rules.ftl"),
                include_str!("../../locales/de/page-health.ftl"),
                include_str!("../../locales/de/seo-issues.ftl"),
                include_str!("../../locales/de/security-issues.ftl"),
                include_str!("../../locales/de/mobile-issues.ftl"),
                include_str!("../../locales/de/labels.ftl"),
                include_str!("../../locales/de/finding-titles.ftl"),
                include_str!("../../locales/de/finding-fixes.ftl"),
                include_str!("../../locales/de/interactive.ftl"),
                include_str!("../../locales/de/batch-consistency.ftl"),
                include_str!("../../locales/de/single-report.ftl"),
                include_str!("../../locales/de/methodology.ftl"),
                include_str!("../../locales/de/seo-details.ftl"),
                include_str!("../../locales/de/diagnosis.ftl"),
                include_str!("../../locales/de/accessibility-details.ftl"),
                include_str!("../../locales/de/score-drivers.ftl"),
                include_str!("../../locales/de/pdf-findings.ftl"),
                include_str!("../../locales/de/pdf-frame.ftl"),
                include_str!("../../locales/de/actions-block.ftl"),
                include_str!("../../locales/de/batch-report.ftl"),
                include_str!("../../locales/de/audit-texts.ftl"),
                include_str!("../../locales/de/a11y-analysis.ftl"),
                include_str!("../../locales/de/output-texts.ftl"),
                include_str!("../../locales/de/privacy-findings.ftl"),
                include_str!("../../locales/de/pdf-sections.ftl"),
                include_str!("../../locales/de/pdf-analysis.ftl"),
                include_str!("../../locales/de/audit-interpretation.ftl"),
                include_str!("../../locales/de/report-builder.ftl"),
            ],
        };
        for ftl in sources {
            let resource = Arc::new(
                FluentResource::try_new(ftl.to_string())
                    .map_err(|e| anyhow::anyhow!("Failed to parse FTL: {:?}", e))?,
            );
            bundle
                .add_resource(resource)
                .map_err(|e| anyhow::anyhow!("Failed to add FTL resource: {:?}", e))?;
        }
        Ok(Self {
            locale: locale.to_string(),
            bundle,
        })
    }

    /// [`I18n::t_args`], or `None` when the bundle has no message `key`.
    pub fn try_t_args(&self, key: &str, args: &[(&str, FluentValue<'_>)]) -> Option<String> {
        self.bundle.get_message(key)?.value()?;
        Some(self.t_args(key, args))
    }

    pub fn t(&self, key: &str) -> String {
        let Some(msg) = self.bundle.get_message(key) else {
            return key.to_string();
        };
        let Some(pattern) = msg.value() else {
            return key.to_string();
        };
        let mut errors = vec![];
        self.bundle
            .format_pattern(pattern, None, &mut errors)
            .to_string()
    }

    /// The attribute `attr` of the message `key` (`.attr = …` below
    /// `key = …` in the locale file). Falls back to `key.attr` on missing
    /// entries.
    pub fn attr(&self, key: &str, attr: &str) -> String {
        let Some(attribute) = self
            .bundle
            .get_message(key)
            .and_then(|msg| msg.get_attribute(attr))
        else {
            return format!("{key}.{attr}");
        };
        let mut errors = vec![];
        self.bundle
            .format_pattern(attribute.value(), None, &mut errors)
            .to_string()
    }

    /// Translate `key` and substitute Fluent placeables using the supplied
    /// `(name, value)` pairs. Falls back to the key on missing entries.
    pub fn t_args<'a, V>(&self, key: &str, args: &[(&'a str, V)]) -> String
    where
        V: Into<FluentValue<'a>> + Clone,
    {
        let Some(msg) = self.bundle.get_message(key) else {
            return key.to_string();
        };
        let Some(pattern) = msg.value() else {
            return key.to_string();
        };
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, value.clone().into());
        }
        let mut errors = vec![];
        self.bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors)
            .to_string()
    }

    /// A message whose wording depends on a count: the entry gets `$count`
    /// and selects singular or plural with a Fluent plural selector, so
    /// agreement ("1 Hinweis" / "2 Hinweise", "Es wurde 1" / "Es wurden 2")
    /// lives in the locale file and never in Rust (#902).
    pub fn count(&self, key: &str, n: usize) -> String {
        self.t_args(key, &[("count", n as i64)])
    }

    /// [`I18n::count`] for messages that also take plain text placeholders
    /// (`$count` stays numeric so the plural selector works).
    pub fn count_with(&self, key: &str, n: usize, extra: &[(&str, String)]) -> String {
        let Some(msg) = self.bundle.get_message(key) else {
            return key.to_string();
        };
        let Some(pattern) = msg.value() else {
            return key.to_string();
        };
        let mut fluent_args = FluentArgs::new();
        fluent_args.set("count", FluentValue::from(n as i64));
        for (name, value) in extra {
            fluent_args.set(*name, FluentValue::from(value.clone()));
        }
        let mut errors = vec![];
        self.bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors)
            .to_string()
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }
}

thread_local! {
    static SHARED: [I18n; 2] = [
        I18n::new("de").expect("German bundle parses"),
        I18n::new("en").expect("English bundle parses"),
    ];
}

/// [`I18n::count`] for pure text functions that take `en: bool` instead of an
/// [`I18n`] (the `*_text(kind, values, en)` pattern of #406).
pub fn count_text(key: &str, n: usize, en: bool) -> String {
    SHARED.with(|bundles| bundles[usize::from(en)].count(key, n))
}

/// A plain message of the shared bundles by key, for the same pure text
/// functions as [`count_text`].
pub fn text(key: &str, en: bool) -> String {
    SHARED.with(|bundles| bundles[usize::from(en)].t(key))
}

/// A message of the shared bundles with named arguments ([`I18n::t_args`]),
/// for the same pure text functions as [`count_text`]. Numbers go in as
/// `FluentValue::from(i64)` so a plural selector can use them; text as
/// `FluentValue::from(String)`.
pub fn text_args(key: &str, args: &[(&str, FluentValue<'_>)], en: bool) -> String {
    SHARED.with(|bundles| bundles[usize::from(en)].t_args(key, args))
}

/// [`text_args`], or `None` when the shared bundles have no message `key`.
pub fn try_text_args(key: &str, args: &[(&str, FluentValue<'_>)], en: bool) -> Option<String> {
    SHARED.with(|bundles| bundles[usize::from(en)].try_t_args(key, args))
}

/// An attribute of a message of the shared bundles ([`I18n::attr`]), for the
/// same pure text functions as [`count_text`].
pub fn attr_text(key: &str, attr: &str, en: bool) -> String {
    SHARED.with(|bundles| bundles[usize::from(en)].attr(key, attr))
}

/// What a `rules.ftl` defines: every message with a value as `key`, every
/// attribute as `key.attribute`. The file holds one text per line, so any
/// other line than a comment or `name = text` fails here, and so does a
/// message or attribute that is defined twice.
#[cfg(test)]
pub(crate) fn ftl_entries(ftl: &str) -> std::collections::BTreeSet<String> {
    let mut entries = std::collections::BTreeSet::new();
    let mut message = "";
    for line in ftl.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, text) = line
            .split_once(" = ")
            .unwrap_or_else(|| panic!("not a `name = text` line: {line}"));
        assert!(!text.trim().is_empty(), "no text: {line}");
        let is_new = match name.strip_prefix("    .") {
            Some(attribute) => entries.insert(format!("{message}.{attribute}")),
            None => {
                message = name;
                entries.insert(name.to_string())
            }
        };
        assert!(is_new, "defined twice: {line}");
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collect FTL message identifiers (column-0 `id = ...` lines).
    fn ftl_keys(ftl: &str) -> std::collections::BTreeSet<String> {
        ftl.lines()
            .filter_map(|line| {
                // Message identifiers start at column 0; continuations are indented
                // and comments start with '#'.
                if line.is_empty() || line.starts_with([' ', '\t', '#']) {
                    return None;
                }
                let (id, _) = line.split_once('=')?;
                let id = id.trim();
                if id.is_empty() || id.contains(char::is_whitespace) {
                    return None;
                }
                Some(id.to_string())
            })
            .collect()
    }

    #[test]
    fn de_and_en_locales_have_identical_key_sets() {
        let de = ftl_keys(include_str!("../../locales/de/report.ftl"));
        let en = ftl_keys(include_str!("../../locales/en/report.ftl"));
        let de_only: Vec<_> = de.difference(&en).collect();
        let en_only: Vec<_> = en.difference(&de).collect();
        assert!(
            de_only.is_empty(),
            "keys only in de/report.ftl: {de_only:?}"
        );
        assert!(
            en_only.is_empty(),
            "keys only in en/report.ftl: {en_only:?}"
        );
    }

    /// `(key, de one, de many, en one, en many)` of the shared count messages.
    const PLURAL_CASES: &[(&str, &str, &str, &str, &str)] = &[
        (
            "count-occurrences-found",
            "Es wurde 1 Vorkommen erkannt",
            "Es wurden 3 Vorkommen erkannt",
            "1 occurrence was found",
            "3 occurrences were found",
        ),
        (
            "count-problem-groups",
            "1 Problemgruppe",
            "3 Problemgruppen",
            "1 problem group",
            "3 problem groups",
        ),
        (
            "count-more-findings",
            "1 weiterer Befund",
            "3 weitere Befunde",
            "1 more finding",
            "3 more findings",
        ),
        (
            "count-notes",
            "1 Hinweis",
            "3 Hinweise",
            "1 note",
            "3 notes",
        ),
        (
            "count-warnings",
            "1 Warnung",
            "3 Warnungen",
            "1 warning",
            "3 warnings",
        ),
        (
            "count-link-texts-repeated",
            "1 Linktext kommt",
            "3 Linktexte kommen",
            "1 link text appears",
            "3 link texts appear",
        ),
        (
            "count-links-generic",
            "1 Link trägt",
            "3 Links tragen",
            "1 link carries",
            "3 links carry",
        ),
        (
            "count-heuristic-warnings",
            "1 heuristische Barrierefreiheits-Warnung",
            "3 heuristische Barrierefreiheits-Warnungen",
            "1 heuristic accessibility warning",
            "3 heuristic accessibility warnings",
        ),
        (
            "count-manual-points",
            "1 Stelle",
            "3 Stellen",
            "1 point",
            "3 points",
        ),
        (
            "count-a11y-warnings",
            "1 Barrierefreiheits-Warnung",
            "3 Barrierefreiheits-Warnungen",
            "1 accessibility warning",
            "3 accessibility warnings",
        ),
        (
            "count-manual-hints",
            "1 manueller Prüfhinweis",
            "3 manuelle Prüfhinweise",
            "1 manual check",
            "3 manual checks",
        ),
        (
            "count-recommendations",
            "1 Empfehlung",
            "3 Empfehlungen",
            "1 recommendation",
            "3 recommendations",
        ),
        (
            "count-images-scaled-down",
            "1 Bild wird",
            "3 Bilder werden",
            "1 image is",
            "3 images are",
        ),
        (
            "output-ts-takeaway-count",
            "1 Technologie erkannt.",
            "3 Technologien erkannt.",
            "1 technology detected.",
            "3 technologies detected.",
        ),
        (
            "output-batch-cover-urls",
            "1 URL geprüft",
            "3 URLs geprüft",
            "1 URL audited",
            "3 URLs audited",
        ),
        (
            "a11y-details-journey-steps",
            "1 Schritt",
            "3 Schritte",
            "1 step",
            "3 steps",
        ),
        (
            "a11y-details-journey-overview-high",
            "1 interaktiver Befund mit hoher Schwere gefunden.",
            "3 interaktive Befunde mit hoher Schwere gefunden.",
            "1 high-severity interactive issue found.",
            "3 high-severity interactive issues found.",
        ),
        (
            "a11y-details-journey-more",
            "1 weiterer interaktiver Befund im JSON-Report.",
            "3 weitere interaktive Befunde im JSON-Report.",
            "1 additional interactive finding in the JSON report.",
            "3 additional interactive findings in the JSON report.",
        ),
        (
            "seo-details-schema-questions",
            "1 Frage",
            "3 Fragen",
            "1 question",
            "3 questions",
        ),
        (
            "seo-details-schema-levels",
            "1 Ebene",
            "3 Ebenen",
            "1 level",
            "3 levels",
        ),
        (
            "seo-details-amp-issues",
            "Erkannt, 1 Basisproblem",
            "Erkannt, 3 Basisprobleme",
            "Detected, 1 basic issue",
            "Detected, 3 basic issues",
        ),
        (
            "seo-profile-detail-hreflang-languages",
            "1 Sprache",
            "3 Sprachen",
            "1 language",
            "3 languages",
        ),
        (
            "seo-profile-generic-array-entries",
            "[1 Eintrag]",
            "[3 Einträge]",
            "[1 entry]",
            "[3 entries]",
        ),
        (
            "page-health-hreflang-invalid",
            "1 hreflang-Eintrag mit ungültigem Sprachcode",
            "3 hreflang-Einträge mit ungültigem Sprachcode",
            "1 hreflang entry with an invalid language code",
            "3 hreflang entries with an invalid language code",
        ),
    ];

    #[test]
    fn count_selects_singular_and_plural_in_both_locales() {
        let de = I18n::new("de").unwrap();
        let en = I18n::new("en").unwrap();
        for (key, de_one, de_many, en_one, en_many) in PLURAL_CASES {
            assert_eq!(de.count(key, 1), *de_one, "{key} de 1");
            assert_eq!(de.count(key, 3), *de_many, "{key} de 3");
            assert_eq!(en.count(key, 1), *en_one, "{key} en 1");
            assert_eq!(en.count(key, 3), *en_many, "{key} en 3");
        }
    }

    /// Messages with several arguments: the count words agree with their
    /// own number, and the displayed numbers keep their thousands grouping
    /// while the selector takes the plain number under another name.
    #[test]
    fn multi_argument_messages_agree_with_their_numbers() {
        use fluent_bundle::FluentValue as V;
        let de = I18n::new("de").unwrap();
        let en = I18n::new("en").unwrap();
        let n = |value: i64| V::from(value);
        let t = |value: &str| V::from(value.to_string());

        let regions = |i18n: &I18n, count: i64| {
            i18n.t_args(
                "seo-details-schema-regions",
                &[("base", t("Org")), ("count", n(count))],
            )
        };
        assert_eq!(regions(&en, 1), "Org · 1 region");
        assert_eq!(regions(&en, 3), "Org · 3 regions");
        assert_eq!(regions(&de, 1), "Org · 1 Region");
        assert_eq!(regions(&de, 3), "Org · 3 Regionen");

        let headings = |i18n: &I18n, h1: i64, total: i64, issues: i64| {
            i18n.t_args(
                "seo-details-heading-summary",
                &[("h1", n(h1)), ("total", n(total)), ("issues", n(issues))],
            )
        };
        assert_eq!(
            headings(&en, 1, 1, 1),
            "1 H1 heading, 1 heading total, 1 issue"
        );
        assert_eq!(
            headings(&en, 0, 12, 3),
            "0 H1 headings, 12 headings total, 3 issues"
        );
        assert_eq!(
            headings(&de, 1, 1, 1),
            "1 H1-Überschrift, 1 Überschrift gesamt, 1 Problem"
        );
        assert_eq!(
            headings(&de, 0, 12, 3),
            "0 H1-Überschriften, 12 Überschriften gesamt, 3 Probleme"
        );

        let method = |i18n: &I18n, shown: &str, count: i64| {
            i18n.t_args(
                "methodology-method",
                &[("rules", t(shown)), ("count", n(count))],
            )
        };
        assert!(method(&en, "1", 1).contains("1 automated rule execution was recorded"));
        assert!(
            method(&en, "1,234", 1234).contains("1,234 automated rule executions were recorded")
        );
        assert!(method(&de, "1", 1).contains("wurde 1 automatisierte Regelausführung erfasst"));
        assert!(method(&de, "1.234", 1234)
            .contains("wurden 1.234 automatisierte Regelausführungen erfasst"));

        let counting = |i18n: &I18n, shown: [&str; 4], plain: [i64; 4]| {
            i18n.t_args(
                "methodology-counting",
                &[
                    ("groups", t(shown[0])),
                    ("groups_n", n(plain[0])),
                    ("occurrences", t(shown[1])),
                    ("occurrences_n", n(plain[1])),
                    ("rows", t(shown[2])),
                    ("rows_n", n(plain[2])),
                    ("all", t(shown[3])),
                    ("all_n", n(plain[3])),
                ],
            )
        };
        assert_eq!(
            counting(&en, ["1", "3", "1", "3"], [1, 3, 1, 3]),
            "1 WCAG finding group, 3 WCAG occurrences; 1 finding row and 3 occurrences across all categories"
        );
        assert_eq!(
            counting(&en, ["2", "1,234", "5", "1,300"], [2, 1234, 5, 1300]),
            "2 distinct WCAG finding groups, 1,234 WCAG occurrences; 5 finding rows and 1,300 occurrences across all categories"
        );
        assert_eq!(
            counting(&de, ["1", "3", "1", "3"], [1, 3, 1, 3]),
            "1 WCAG-Regelgruppe, 3 WCAG-Vorkommen; 1 Befundzeile und 3 Vorkommen über alle Kategorien"
        );
        let recurring = |i18n: &I18n, count: i64| {
            i18n.t_args(
                "audit-risk-project-recurring",
                &[("count", n(count)), ("pages", n(7))],
            )
        };
        assert!(recurring(&en, 1).starts_with("1 rule recurs on two or more of the 7"));
        assert!(recurring(&en, 3).starts_with("3 rules recur on two or more of the 7"));
        assert!(recurring(&de, 1).starts_with("1 Regel tritt auf zwei oder mehr der 7"));
        assert!(recurring(&de, 3).starts_with("3 Regeln treten auf zwei oder mehr der 7"));

        assert_eq!(
            counting(&de, ["2", "1.234", "5", "1.300"], [2, 1234, 5, 1300]),
            "2 unterschiedliche WCAG-Regelgruppen, 1.234 WCAG-Vorkommen; 5 Befundzeilen und 1.300 Vorkommen über alle Kategorien"
        );
    }

    #[test]
    fn schema_status_uses_numeric_plural_selection() {
        let de = I18n::new("de").expect("German bundle parses");
        let text = de.t_args(
            "pdf-seo-schema-status-issues",
            &[("nodes", 2_i64), ("issues", 1_i64)],
        );
        assert!(text.contains("2 auswertbare Schema-Knoten"));
        assert!(text.contains("Ein Syntax-/Strukturproblem liegt vor"));

        let en = I18n::new("en").expect("English bundle parses");
        let text = en.t_args(
            "pdf-seo-schema-status-issues",
            &[("nodes", 1_i64), ("issues", 2_i64)],
        );
        assert!(text.contains("One evaluable schema node"));
        assert!(text.contains("2 syntax or structure issues are present"));
    }

    #[test]
    fn batch_scope_keys_render_with_args_in_both_locales() {
        for locale in ["de", "en"] {
            let i18n = I18n::new(locale).expect("bundle parses");
            let rendered = i18n.t_args(
                "batch-scope-sample",
                &[
                    ("audited", "20".to_string()),
                    ("total", "487".to_string()),
                    ("source", i18n.t("batch-source-sitemap")),
                ],
            );
            // A missing key falls back to the key itself — assert real text + the
            // interpolated numbers came through.
            assert_ne!(rendered, "batch-scope-sample", "key must exist in {locale}");
            assert!(rendered.contains("20") && rendered.contains("487"));
        }
    }

    /// Plural agreement of the 2.0.1 leftovers (#952): the count reaches the
    /// message as a number, so "1 Datei" / "1 weiterer Befund" select the
    /// singular instead of printing "1 Dateien" / "1 weitere Befunde".
    #[test]
    fn counted_messages_agree_in_number() {
        let de = I18n::new("de").unwrap();
        let en = I18n::new("en").unwrap();
        for (key, singular_de, plural_de, singular_en) in [
            (
                "pdf-seo-more-signals",
                "1 weiteres Signal",
                "2 weitere Signale",
                "1 additional signal",
            ),
            ("seo-cookie-inventory", "1 Cookie", "2 Cookies", "1 cookie"),
        ] {
            assert!(
                de.count(key, 1).starts_with(singular_de),
                "{key}: {}",
                de.count(key, 1)
            );
            assert!(
                de.count(key, 2).starts_with(plural_de),
                "{key}: {}",
                de.count(key, 2)
            );
            assert!(
                en.count(key, 1).starts_with(singular_en),
                "{key}: {}",
                en.count(key, 1)
            );
        }
        let one = de.count_with("pdf-perf-resource-focus", 1, &[("savings", "175,5".into())]);
        assert!(one.contains("von 1 Datei lassen"), "{one}");
        let many = de.count_with("pdf-perf-resource-focus", 2, &[("savings", "175,5".into())]);
        assert!(many.contains("von 2 Dateien lassen"), "{many}");
        let dup = de.count_with("pdf-perf-dup-title", 1, &[("wasted", "3,0".into())]);
        assert!(dup.starts_with("1 doppelte Asset-Gruppe erkannt"), "{dup}");
    }
    /// Wording round 3: messages whose count words, verbs and pronouns must
    /// follow their number, and the clause count of the roll-up title. Each
    /// assertion fails on the text it replaced.
    #[test]
    fn wording_round_3_messages_agree_with_their_numbers() {
        use fluent_bundle::FluentValue as V;
        let de = I18n::new("de").unwrap();
        let en = I18n::new("en").unwrap();
        let n = |value: i64| V::from(value);
        let t = |value: &str| V::from(value.to_string());

        let intro = |i18n: &I18n, key: &str, urgent: i64| {
            i18n.t_args(key, &[("urgent", n(urgent)), ("cross", t(""))])
        };
        let polished = "audit-interpretation-summary-intro-polished-topics";
        assert!(intro(&de, polished, 1).contains("1 priorisiertes Thema — gezielt"));
        assert!(intro(&de, polished, 3).contains("3 priorisierte Themen — gezielt"));
        assert!(intro(&en, polished, 1)
            .contains("1 prioritized topic — fix it deliberately before it piles up."));
        assert!(intro(&en, polished, 3)
            .contains("3 prioritized topics — fix them deliberately before they pile up."));
        let urgent = "audit-interpretation-summary-intro-needs-work-urgent";
        assert!(intro(&de, urgent, 1).contains("1 priorisiertes Thema braucht sofortige"));
        assert!(intro(&de, urgent, 3).contains("3 priorisierte Themen brauchen sofortige"));

        let blocking = |i18n: &I18n, count: i64| {
            i18n.t_args(
                "audit-interpretation-perf-render-blocking",
                &[("count", n(count))],
            )
        };
        assert_eq!(blocking(&de, 1), "1 render-blockierende Ressource");
        assert_eq!(blocking(&de, 4), "4 render-blockierende Ressourcen");
        assert_eq!(blocking(&en, 1), "1 render-blocking resource");
        assert_eq!(blocking(&en, 4), "4 render-blocking resources");

        let band = |i18n: &I18n, key: &str, count: i64| {
            i18n.t_args(
                key,
                &[
                    ("score", t("62")),
                    ("numeric", t("Mittel")),
                    ("count", n(count)),
                ],
            )
        };
        let open = "report-builder-security-band-note-open";
        let severe = "report-builder-security-band-note-severe";
        assert!(band(&de, open, 1).contains("1 offener Befund steht weiter unten"));
        assert!(band(&de, open, 2).contains("2 offene Befunde stehen weiter unten"));
        assert!(band(&de, severe, 1).contains("1 offener Befund hoher Schwere steht weiter unten"));
        assert!(band(&en, open, 1).contains("1 open finding is listed below"));
        assert!(band(&en, severe, 2).contains("2 open findings of high severity are listed below"));

        let outside = |i18n: &I18n, count: i64| {
            i18n.t_args(
                "pdf-sections-en301549-out-of-standard",
                &[("count", n(count))],
            )
        };
        assert!(outside(&de, 1).starts_with("1 weiterer WCAG-Befund"));
        assert!(
            outside(&de, 1).contains("hat in EN 301 549 V4.1.1 keine normative Klausel und wird")
        );
        assert!(outside(&de, 2).starts_with("2 weitere WCAG-Befunde"));
        assert!(outside(&en, 1).starts_with("1 additional WCAG finding outside"));
        assert!(outside(&en, 1).contains("has no normative clause in EN 301 549 V4.1.1 and is not"));
        assert!(outside(&en, 2).starts_with("2 additional WCAG findings"));

        let leverage = |i18n: &I18n, count: i64| {
            i18n.t_args(
                "pdf-analysis-roadmap-leverage",
                &[
                    ("leverage", t("high")),
                    ("count", n(count)),
                    ("effort", t("low")),
                ],
            )
        };
        assert!(leverage(&en, 1).contains("1 occurrence affected"));
        assert!(leverage(&en, 3).contains("3 occurrences affected"));
        assert!(leverage(&de, 1).contains("1 Vorkommen betroffen"));

        // The EN texts take the count words the German ones already used.
        let evidence = en.t_args(
            "pdf-analysis-causes-none-evidence",
            &[
                ("warnings_text", t("1 accessibility warning")),
                ("manual_text", t("1 manual check")),
            ],
        );
        assert!(evidence.contains("1 accessibility warning and 1 manual check remain open"));
        let few = en.t_args(
            "pdf-analysis-profile-few",
            &[("found", t("1 occurrence was found"))],
        );
        assert!(few.starts_with("1 occurrence was found — too few"));
        let modules = en.t_args(
            "pdf-analysis-roadmap-empty-modules",
            &[("recommendations", t("1 recommendation"))],
        );
        assert!(modules.contains("(performance, SEO, security, mobile): 1 recommendation —"));
        let note = en.t_args(
            "single-report-evidence-note",
            &[
                ("warnings_text", t("1 heuristic accessibility warning")),
                ("manual_text", t("1 point")),
                ("manual_only", n(12)),
            ],
        );
        assert!(note
            .contains("In addition: 1 heuristic accessibility warning and 1 point on this page"));

        // The roll-up title names the real clause count (WCAG 2.2: 55 A/AA clauses).
        let title =
            |i18n: &I18n| i18n.t_args("batch-report-en301549-table-title", &[("count", n(55))]);
        assert_eq!(
            title(&de),
            "Klausel-Übersicht (alle 55 WCAG-2.2-A/AA-Klauseln)"
        );
        assert_eq!(title(&en), "Clause roll-up (all 55 WCAG 2.2 A/AA clauses)");
    }

    /// Typography and spelling that one rendered string cannot show: every
    /// locale file is scanned for plural suffixes, German quotation marks
    /// closed by ASCII, transliterated umlauts and double hyphens.
    #[test]
    fn locale_files_keep_quotes_dashes_and_umlauts_consistent() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        let transliterated = [
            "unvollstaendig",
            "verlaesslich",
            "koennen",
            "Theman",
            "brauchtn",
        ];
        for locale in ["de", "en"] {
            for entry in std::fs::read_dir(root.join(locale)).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|e| e != "ftl") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap();
                for (number, line) in text.lines().enumerate() {
                    if line.trim_start().starts_with('#') {
                        continue;
                    }
                    let at = format!("{}:{}", path.display(), number + 1);
                    // A count word is chosen by a plural select, never by a "(s)" suffix.
                    for hack in ["(s)", "(e)", "(r)", "(n)"] {
                        assert!(!line.contains(hack), "{at}: plural suffix {hack}");
                    }
                    if locale == "de" {
                        // „ opens a German quotation; it closes with “, not with ASCII ".
                        let mut rest = line;
                        while let Some(open) = rest.find('„') {
                            let after = &rest[open + '„'.len_utf8()..];
                            let close = after.find(['“', '"']).unwrap_or(after.len());
                            assert_eq!(
                                after[close..].chars().next(),
                                Some('“'),
                                "{at}: „ is not closed by “"
                            );
                            rest = &after[close..];
                        }
                        for word in transliterated {
                            assert!(!line.contains(word), "{at}: {word}");
                        }
                    }
                    if line.starts_with("audit-interpretation-summary-intro") {
                        assert!(!line.contains(" -- "), "{at}: double hyphen instead of —");
                    }
                }
            }
        }
    }
}
