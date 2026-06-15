//! SEO-related interpretation helpers.

pub(super) fn build_seo_interpretation(locale: &str, seo: &crate::seo::SeoAnalysis) -> String {
    let en = locale == "en";
    let lead = if seo.score >= 90 {
        if en {
            "The technical SEO foundations are largely in place and support visibility in search engines."
        } else {
            "Die technischen SEO-Grundlagen sind weitgehend erfüllt und unterstützen die Sichtbarkeit in Suchmaschinen."
        }
    } else if seo.score >= 70 {
        if en {
            "Good SEO base with targeted potential to improve visibility."
        } else {
            "Gute SEO-Grundlage mit gezieltem Optimierungspotenzial für mehr Sichtbarkeit."
        }
    } else if seo.score >= 55 {
        if en {
            "Some important SEO foundations are missing, which can limit visibility in search engines."
        } else {
            "Wichtige SEO-Grundlagen fehlen teilweise, wodurch die Sichtbarkeit in Suchmaschinen eingeschränkt sein kann."
        }
    } else if seo.score >= 35 {
        if en {
            "SEO insufficient — essential foundations are missing and noticeably limit discoverability."
        } else {
            "SEO unzureichend — wesentliche Grundlagen fehlen und begrenzen die Auffindbarkeit deutlich."
        }
    } else if en {
        "SEO critical — fundamental prerequisites are missing; indexing and discoverability are at risk."
    } else {
        "SEO kritisch — grundlegende Voraussetzungen fehlen; Indexierung und Auffindbarkeit sind gefährdet."
    };

    if let Some(profile) = &seo.content_profile {
        let page_type = profile.page_classification.primary_type.label(en);
        let reference = profile.page_classification.intent_fit_score;
        let content_depth = profile.page_classification.content_depth_score;
        let score = seo.score;
        let context = if score >= reference {
            if en {
                format!(
                    "Classified as \u{201C}{page_type}\u{201D} — score {score} meets the reference value for this page type ({reference})."
                )
            } else {
                format!(
                    "Seitentyp: \u{201E}{page_type}\u{201C} — Score {score} liegt im erwarteten Bereich für diesen Seitentyp (Referenz: {reference})."
                )
            }
        } else if reference.saturating_sub(score) <= 10 {
            if en {
                format!(
                    "Classified as \u{201C}{page_type}\u{201D} — score {score} is slightly below the reference for this page type ({reference}); a few signals are still missing."
                )
            } else {
                format!(
                    "Seitentyp: \u{201E}{page_type}\u{201C} — Score {score} liegt knapp unter dem Erwartungswert für diesen Seitentyp ({reference}); einzelne Signale fehlen noch."
                )
            }
        } else if en {
            format!(
                "Classified as \u{201C}{page_type}\u{201D} — score {score} is notably below the reference for this page type ({reference})."
            )
        } else {
            format!(
                "Seitentyp: \u{201E}{page_type}\u{201C} — Score {score} liegt deutlich unter dem Erwartungswert für diesen Seitentyp ({reference})."
            )
        };
        // When technical SEO is strong but content depth is weak, make the gap explicit
        // so readers don't interpret a high SEO score as endorsing the content quality.
        let depth_note = if seo.score >= 80 && content_depth < 55 {
            if en {
                format!(
                    " Technical SEO complete — content depth ({content_depth}/100) still has room to grow."
                )
            } else {
                format!(
                    " Technisches SEO vollständig — inhaltliche Tiefe ({content_depth}/100) noch ausbaufähig."
                )
            }
        } else {
            String::new()
        };
        format!("{lead} {context}{depth_note}")
    } else {
        lead.to_string()
    }
}

pub(super) fn summarize_page_profile(
    locale: &str,
    profile: &crate::seo::profile::SeoContentProfile,
) -> String {
    let en = locale == "en";
    let classification = &profile.page_classification;
    let avg = crate::seo::average_page_semantic_score(classification);
    let quality = if en {
        match avg {
            85..=100 => "very coherently structured",
            70..=84 => "technically and structurally well supported",
            50..=69 => "only partly clearly structured",
            _ => "currently weak in content and structure",
        }
    } else {
        match avg {
            85..=100 => "sehr stimmig aufgebaut",
            70..=84 => "technisch und strukturell gut gestützt",
            50..=69 => "nur teilweise klar strukturiert",
            _ => "aktuell inhaltlich und strukturell schwach ausgeprägt",
        }
    };

    let mut traits = classification.attributes.clone();
    if traits.is_empty() {
        traits.push(
            if en {
                "no clear extra attributes"
            } else {
                "ohne klare Zusatzmerkmale"
            }
            .to_string(),
        );
    }

    if en {
        format!(
            "The page reads as \u{201C}{}\u{201D} and is {}. Notable: {}.",
            classification.primary_type.label(true),
            quality,
            traits.join(", ")
        )
    } else {
        format!(
            "Die Seite wirkt wie \u{201E}{}\u{201C} und ist {}. Auffällig sind {}.",
            classification.primary_type.label(false),
            quality,
            traits.join(", ")
        )
    }
}

pub(super) fn page_profile_optimization_note(
    locale: &str,
    profile: &crate::seo::profile::SeoContentProfile,
) -> String {
    let en = locale == "en";
    let classification = &profile.page_classification;
    if classification.content_depth_score < 45 {
        return if en {
            "More content depth and clearly structured sections would raise utility.".to_string()
        } else {
            "Mehr inhaltliche Tiefe und klar gegliederte Abschnitte würden den Nutzwert erhöhen."
                .to_string()
        };
    }
    if classification.structural_richness_score < 55 {
        return if en {
            "More subheadings and a clearer content structure would make the page easier to scan."
                .to_string()
        } else {
            "Mehr Zwischenüberschriften und eine klarere Inhaltsstruktur würden die Seite besser scannbar machen.".to_string()
        };
    }
    if classification.media_text_balance_score < 55 {
        return if en {
            "The page is heavily visual. More explanatory text and clearer context would improve utility and orientation.".to_string()
        } else {
            "Die Seite wirkt stark visuell. Mehr erklärender Text und klarer Kontext würden Nutzen und Orientierung verbessern.".to_string()
        };
    }
    if classification.intent_fit_score < 65 {
        return if en {
            "The page does not yet serve its page type cleanly; structure and content should align more strongly with the actual user goal.".to_string()
        } else {
            "Die Seite bedient ihren Seitentyp noch nicht sauber; Aufbau und Inhalte sollten stärker auf das eigentliche Nutzerziel einzahlen.".to_string()
        };
    }
    if en {
        "The page fits its page type well overall. The biggest lever is further sharpening content rather than fundamental rebuilds.".to_string()
    } else {
        "Die Seite passt insgesamt gut zu ihrem Seitentyp. Der größte Hebel liegt in weiterer inhaltlicher Schärfung statt in Grundsatzumbauten.".to_string()
    }
}
