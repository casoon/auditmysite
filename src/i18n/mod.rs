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
        // issue texts (`seo::page_health`), the SEO issue texts and labels
        // (`seo::technical`, `seo::meta`, `seo::schema`, `seo::robots`,
        // `seo::profile`, `seo::serp`), the
        // security finding texts and category labels (`security`), the
        // mobile finding texts (`mobile`) and the labels of cross-cutting
        // enums (`labels.ftl`: page intent, action priority, effort and role,
        // issue class, subcategory) are seven resources of the one bundle.
        let sources = match locale {
            "en" => [
                include_str!("../../locales/en/report.ftl"),
                include_str!("../../locales/en/rules.ftl"),
                include_str!("../../locales/en/page-health.ftl"),
                include_str!("../../locales/en/seo-issues.ftl"),
                include_str!("../../locales/en/security-issues.ftl"),
                include_str!("../../locales/en/mobile-issues.ftl"),
                include_str!("../../locales/en/labels.ftl"),
            ],
            _ => [
                include_str!("../../locales/de/report.ftl"),
                include_str!("../../locales/de/rules.ftl"),
                include_str!("../../locales/de/page-health.ftl"),
                include_str!("../../locales/de/seo-issues.ftl"),
                include_str!("../../locales/de/security-issues.ftl"),
                include_str!("../../locales/de/mobile-issues.ftl"),
                include_str!("../../locales/de/labels.ftl"),
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
            "count-h1-headings",
            "1 H1-Überschrift",
            "3 H1-Überschriften",
            "1 H1 heading",
            "3 H1 headings",
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
                "pdf-ux-more-issues",
                "1 weiterer Befund",
                "2 weitere Befunde",
                "1 additional issue",
            ),
            (
                "pdf-journey-more-issues",
                "1 weiterer Befund",
                "2 weitere Befunde",
                "1 additional issue",
            ),
            (
                "pdf-seo-more-signals",
                "1 weiteres Signal",
                "2 weitere Signale",
                "1 additional signal",
            ),
            (
                "pdf-ai-more-signals",
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
}
