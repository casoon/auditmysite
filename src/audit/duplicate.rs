//! Near-duplicate content detection using SimHash fingerprinting.
//!
//! Algorithm: Charikar SimHash over 2-word shingles.
//! Each page's text excerpt is fingerprinted as a 64-bit hash.
//! Pages with Hamming distance ≤ threshold are near-duplicates.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Computed SimHash fingerprint for a page.
#[derive(Debug, Clone)]
struct ContentFingerprint {
    url: String,
    hash: u64,
    word_count: usize,
}

/// A pair of near-duplicate pages.
#[derive(Debug, Clone)]
pub struct DuplicatePair {
    pub url_a: String,
    pub url_b: String,
    /// Similarity percentage (0–100), derived from Hamming distance
    pub similarity: u8,
    /// Hamming distance between the two fingerprints (0 = identical content)
    pub hamming_distance: u32,
}

impl DuplicatePair {
    /// True if content is essentially identical (≥ 95% similarity)
    pub fn is_exact_duplicate(&self) -> bool {
        self.similarity >= 95
    }
}

/// Detect near-duplicate page pairs from a list of (url, text_excerpt) inputs.
///
/// Returns pairs whose SimHash similarity is ≥ `threshold_pct`, sorted by
/// similarity descending. Pages with fewer than `min_words` content words are
/// excluded (thin-content pages produce unreliable fingerprints).
pub fn detect_near_duplicates(
    pages: &[(String, String)],
    threshold_pct: u8,
    min_words: usize,
) -> Vec<DuplicatePair> {
    let fingerprints: Vec<ContentFingerprint> = pages
        .iter()
        .map(|(url, text)| {
            let cleaned = strip_boilerplate(text);
            let word_count = cleaned.split_whitespace().count();
            ContentFingerprint {
                url: url.clone(),
                hash: simhash(&cleaned),
                word_count,
            }
        })
        .collect();

    let mut pairs = Vec::new();

    for (i, a) in fingerprints.iter().enumerate() {
        if a.word_count < min_words {
            continue;
        }
        for b in fingerprints.iter().skip(i + 1) {
            if b.word_count < min_words {
                continue;
            }
            let dist = hamming_distance(a.hash, b.hash);
            let sim = similarity_pct(dist);
            if sim >= threshold_pct {
                pairs.push(DuplicatePair {
                    url_a: a.url.clone(),
                    url_b: b.url.clone(),
                    similarity: sim,
                    hamming_distance: dist,
                });
            }
        }
    }

    pairs.sort_by(|a, b| b.similarity.cmp(&a.similarity));
    pairs
}

// ─── SimHash ────────────────────────────────────────────────────────────────

fn simhash(text: &str) -> u64 {
    let tokens = tokenize(text);
    if tokens.len() < 2 {
        return 0;
    }

    let mut v = [0i32; 64];

    // Use 2-word shingles for content-sensitive fingerprinting
    for window in tokens.windows(2) {
        let shingle = format!("{} {}", window[0], window[1]);
        let h = hash_str(&shingle);
        for bit in 0u64..64 {
            if (h >> bit) & 1 == 1 {
                v[bit as usize] += 1;
            } else {
                v[bit as usize] -= 1;
            }
        }
    }

    let mut fingerprint = 0u64;
    for bit in 0..64 {
        if v[bit] > 0 {
            fingerprint |= 1u64 << bit;
        }
    }
    fingerprint
}

fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

fn similarity_pct(hamming: u32) -> u8 {
    let sim = (1.0 - hamming as f64 / 64.0) * 100.0;
    sim.round().clamp(0.0, 100.0) as u8
}

// ─── Text preprocessing ─────────────────────────────────────────────────────

/// Strip boilerplate: remove short lines (likely navigation / button labels)
/// and normalize whitespace. Returns cleaned text.
fn strip_boilerplate(text: &str) -> String {
    let filtered: Vec<&str> = text
        .lines()
        .filter(|line| line.split_whitespace().count() >= 4)
        .collect();
    filtered.join(" ")
}

/// Tokenize text into lowercase alphanumeric tokens of length ≥ 3.
fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .filter(|w| w.len() >= 3)
        .collect()
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_texts_have_hamming_zero() {
        let text = "The quick brown fox jumps over the lazy dog. \
                    This is a longer sentence to provide enough content for the fingerprint.";
        assert_eq!(hamming_distance(simhash(text), simhash(text)), 0);
    }

    #[test]
    fn test_completely_different_texts_have_high_hamming() {
        let a = "Accessibility audit web WCAG compliance screen reader navigation keyboard focus labels forms images contrast";
        let b = "Finanzplanung Altersvorsorge Aktien Fonds Anleihen Depot Rendite Portfolio Diversifikation Risikostreuung";
        // Very different content should yield high Hamming distance
        let dist = hamming_distance(simhash(a), simhash(b));
        assert!(dist > 10, "Expected high hamming distance, got {}", dist);
    }

    #[test]
    fn test_near_duplicate_texts_are_detected() {
        let base = "Wir bieten professionelle Webentwicklung für kleine und mittelständische Unternehmen. \
                    Unsere Leistungen umfassen Design Entwicklung und SEO-Optimierung. \
                    Kontaktieren Sie uns für ein kostenloses Beratungsgespräch.";
        let variant = "Wir bieten professionelle Webentwicklung für kleine und mittelständische Unternehmen. \
                    Unsere Leistungen umfassen Design Entwicklung und SEO-Optimierung. \
                    Kontaktieren Sie uns noch heute für ein kostenloses Erstgespräch.";
        let dist = hamming_distance(simhash(base), simhash(variant));
        // Very similar texts should have low Hamming distance
        assert!(dist <= 10, "Expected low hamming distance for near-duplicates, got {}", dist);
    }

    #[test]
    fn test_strip_boilerplate_removes_short_lines() {
        let text = "Home\nAbout\nContact\nThis is a full sentence with enough words to pass the filter.";
        let cleaned = strip_boilerplate(text);
        assert!(!cleaned.contains("Home"));
        assert!(cleaned.contains("This is a full sentence"));
    }

    #[test]
    fn test_detect_near_duplicates_finds_similar_pair() {
        let pages = vec![
            (
                "https://example.com/page-a".to_string(),
                "Wir bieten professionelle Webentwicklung fuer kleine und mittelstaendische Unternehmen. \
                 Unsere Leistungen umfassen Design Entwicklung und SEO Optimierung. \
                 Kontaktieren Sie uns fuer ein kostenloses Beratungsgespraech. \
                 Weitere Informationen finden Sie auf unserer Webseite unter den Leistungsseiten.".to_string(),
            ),
            (
                "https://example.com/page-b".to_string(),
                "Wir bieten professionelle Webentwicklung fuer kleine und mittelstaendische Unternehmen. \
                 Unsere Leistungen umfassen Design Entwicklung und SEO Optimierung. \
                 Kontaktieren Sie uns fuer ein kostenloses Erstgespraech. \
                 Weitere Informationen finden Sie auf unserer Webseite unter den Leistungsseiten.".to_string(),
            ),
            (
                "https://example.com/blog".to_string(),
                "Kubernetes deployment strategies for microservices architecture. \
                 Container orchestration and load balancing explained in detail. \
                 DevOps best practices for continuous integration and delivery pipelines.".to_string(),
            ),
        ];

        let pairs = detect_near_duplicates(&pages, 75, 10);
        assert!(!pairs.is_empty(), "Should detect near-duplicate pair");
        assert_eq!(pairs[0].url_a, "https://example.com/page-a");
        assert_eq!(pairs[0].url_b, "https://example.com/page-b");
        assert!(pairs[0].similarity >= 75);
    }

    #[test]
    fn test_detect_near_duplicates_skips_thin_content() {
        let pages = vec![
            ("https://example.com/a".to_string(), "Short".to_string()),
            ("https://example.com/b".to_string(), "Short".to_string()),
        ];
        // Both pages have too few words → no pairs returned
        let pairs = detect_near_duplicates(&pages, 80, 50);
        assert!(pairs.is_empty());
    }
}
