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
