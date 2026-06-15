//! Topic extraction and semantic topic helpers for batch SEO reporting.

use std::collections::{HashMap, HashSet};

use crate::audit::AuditReport;
use crate::i18n::I18n;
use crate::output::report_model::CompactUrlSummary;

const SUPPORTED_LOCALES: &[&str] = &["de", "en"];

fn normalize_topic_token(token: &str) -> String {
    token
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
        .replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ü", "ue")
        .replace("ß", "ss")
}

fn stopwords_for_locale(locale: &str) -> Vec<String> {
    let Ok(i18n) = I18n::new(locale) else {
        return Vec::new();
    };
    let raw = i18n.t("topic-stopwords");
    if raw == "topic-stopwords" {
        return Vec::new();
    }
    raw.split(',')
        .map(normalize_topic_token)
        .filter(|word| !word.is_empty())
        .collect()
}

fn topic_stopwords() -> HashSet<String> {
    SUPPORTED_LOCALES
        .iter()
        .flat_map(|locale| stopwords_for_locale(locale))
        .collect()
}

pub fn extract_page_topics(report: &AuditReport) -> Vec<String> {
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

pub fn derive_domain_topics(url_details: &[CompactUrlSummary]) -> Vec<(String, usize)> {
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

pub fn derive_topic_overlap_pairs(url_details: &[CompactUrlSummary]) -> Vec<(String, String, u32)> {
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

pub fn average_page_semantic_score(
    classification: &crate::seo::profile::PageClassification,
) -> u32 {
    let total = classification.content_depth_score
        + classification.structural_richness_score
        + classification.media_text_balance_score
        + classification.intent_fit_score;
    total / 4
}

fn top_terms_from_segments(segments: &[(String, usize)], limit: usize) -> Vec<String> {
    let stopwords = topic_stopwords();
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
