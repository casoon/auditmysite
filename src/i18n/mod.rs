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
        let ftl = match locale {
            "en" => include_str!("../../locales/en/report.ftl"),
            _ => include_str!("../../locales/de/report.ftl"),
        };
        let resource = Arc::new(
            FluentResource::try_new(ftl.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to parse FTL: {:?}", e))?,
        );
        bundle
            .add_resource(resource)
            .map_err(|e| anyhow::anyhow!("Failed to add FTL resource: {:?}", e))?;
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

    pub fn locale(&self) -> &str {
        &self.locale
    }
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
}
