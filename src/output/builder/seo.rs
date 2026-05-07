//! SEO-related interpretation and topic extraction helpers.

use std::collections::HashMap;

use super::helpers::{german_stopwords, normalize_topic_token};
use crate::audit::AuditReport;
use crate::output::report_model::CompactUrlSummary;

pub(super) fn build_seo_interpretation(locale: &str, seo: &crate::seo::SeoAnalysis) -> String {
    let en = locale == "en";
    if seo.score >= 90 {
        if en {
            "Technical SEO foundations largely met — relevant ranking signals are in place."
                .to_string()
        } else {
            "Technische SEO-Grundlagen weitgehend erfüllt — relevante Ranking-Signale vorhanden."
                .to_string()
        }
    } else if seo.score >= 70 {
        if en {
            "Solid SEO base with targeted optimization potential.".to_string()
        } else {
            "Solide SEO-Basis mit gezieltem Optimierungspotenzial.".to_string()
        }
    } else if seo.score >= 55 {
        if en {
            "SEO base patchy — relevant ranking signals missing, visibility clearly limited."
                .to_string()
        } else {
            "SEO-Basis lückenhaft — relevante Ranking-Signale fehlen, Sichtbarkeit deutlich eingeschränkt.".to_string()
        }
    } else if seo.score >= 35 {
        if en {
            "SEO insufficient — essential foundations missing. Ranking in competitive areas is almost impossible.".to_string()
        } else {
            "SEO unzureichend — wesentliche Grundlagen fehlen. Ranking in kompetitiven Bereichen quasi unmöglich.".to_string()
        }
    } else if en {
        "SEO critical — the site is barely indexable for search engines. Not competitive."
            .to_string()
    } else {
        "SEO kritisch — Seite ist für Suchmaschinen kaum indexierbar. Nicht wettbewerbsfähig."
            .to_string()
    }
}

pub(super) fn extract_page_topics(report: &AuditReport) -> Vec<String> {
    let mut weighted_segments: Vec<(String, usize)> = Vec::new();
    if let Some(ref seo) = report.seo {
        if let Some(ref title) = seo.meta.title {
            weighted_segments.push((title.clone(), 4));
        }
        if let Some(ref description) = seo.meta.description {
            weighted_segments.push((description.clone(), 2));
        }
        for heading in &seo.headings.headings {
            weighted_segments.push((heading.text.clone(), if heading.level <= 2 { 3 } else { 2 }));
        }
        weighted_segments.push((seo.technical.text_excerpt.clone(), 1));
    }

    top_terms_from_segments(&weighted_segments, 5)
}

pub(super) fn derive_domain_topics(url_details: &[CompactUrlSummary]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for detail in url_details {
        for term in &detail.topic_terms {
            *counts.entry(term.clone()).or_default() += 1;
        }
    }

    let mut topics: Vec<(String, usize)> = counts.into_iter().collect();
    topics.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    topics.into_iter().take(8).collect()
}

pub(super) fn derive_topic_overlap_pairs(
    url_details: &[CompactUrlSummary],
) -> Vec<(String, String, u32)> {
    use std::collections::HashSet;

    let mut pairs = Vec::new();
    for (idx, left) in url_details.iter().enumerate() {
        let left_terms: HashSet<&str> = left.topic_terms.iter().map(String::as_str).collect();
        if left_terms.len() < 3 {
            continue;
        }

        for right in url_details.iter().skip(idx + 1) {
            let right_terms: HashSet<&str> = right.topic_terms.iter().map(String::as_str).collect();
            if right_terms.len() < 3 {
                continue;
            }

            let intersection = left_terms.intersection(&right_terms).count();
            if intersection < 2 {
                continue;
            }

            let overlap_ratio =
                intersection as f64 / left_terms.len().min(right_terms.len()) as f64;
            let union = left_terms.union(&right_terms).count();
            let jaccard = intersection as f64 / union as f64;
            let similarity = ((jaccard * 0.55 + overlap_ratio * 0.45) * 100.0).round() as u32;
            if similarity >= 45 {
                pairs.push((left.url.clone(), right.url.clone(), similarity));
            }
        }
    }

    pairs.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
    });
    pairs.into_iter().take(6).collect()
}

pub(super) fn top_terms_from_segments(segments: &[(String, usize)], limit: usize) -> Vec<String> {
    let stopwords = german_stopwords();
    let mut counts: HashMap<String, usize> = HashMap::new();

    for (segment, weight) in segments {
        for token in segment
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .filter(|token| !token.is_empty())
        {
            let normalized = normalize_topic_token(token);
            if normalized.len() < 4
                || normalized.chars().all(|ch| ch.is_ascii_digit())
                || stopwords.contains(normalized.as_str())
            {
                continue;
            }
            *counts.entry(normalized).or_default() += *weight;
        }
    }

    let mut terms: Vec<(String, usize)> = counts.into_iter().collect();
    terms.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    terms
        .into_iter()
        .take(limit)
        .map(|(term, _)| term)
        .collect()
}

pub(super) fn average_page_semantic_score(
    classification: &crate::seo::profile::PageClassification,
) -> u32 {
    let total = classification.content_depth_score
        + classification.structural_richness_score
        + classification.media_text_balance_score
        + classification.intent_fit_score;
    total / 4
}

pub(super) fn summarize_page_profile(
    locale: &str,
    profile: &crate::seo::profile::SeoContentProfile,
) -> String {
    let en = locale == "en";
    let classification = &profile.page_classification;
    let avg = average_page_semantic_score(classification);
    let quality = if en {
        match avg {
            85..=100 => "very coherently structured",
            70..=84 => "solidly built in terms of content",
            50..=69 => "only partly clearly structured",
            _ => "currently weak in content and structure",
        }
    } else {
        match avg {
            85..=100 => "sehr stimmig aufgebaut",
            70..=84 => "inhaltlich solide aufgestellt",
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
            classification.primary_type.label(),
            quality,
            traits.join(", ")
        )
    } else {
        format!(
            "Die Seite wirkt wie \u{201E}{}\u{201C} und ist {}. Auffällig sind {}.",
            classification.primary_type.label(),
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
