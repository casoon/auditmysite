//! Page Intent Detection
//!
//! Heuristic classification of a page's primary purpose based on AXTree signals.
//! The detected intent shifts dimension weights so that, e.g., a shop page weighs
//! trust and conversion higher while an editorial page weighs content clarity.

use serde::{Deserialize, Serialize};

use crate::accessibility::AXTree;

/// Detected page intent / type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageIntent {
    /// Product / e-commerce page
    Shop,
    /// Lead generation / contact / signup
    LeadGen,
    /// Blog, article, documentation
    Editorial,
    /// Marketing / landing page
    Marketing,
    /// Corporate info page (about, team, career)
    Corporate,
    /// Hub / portal / dashboard with many links
    Hub,
    /// Could not determine intent
    Unknown,
}

impl PageIntent {
    /// Dimension weights tuned for this page intent:
    /// (entry, orientation, navigation, interaction, conversion)
    pub fn weights(&self) -> (f64, f64, f64, f64, f64) {
        match self {
            PageIntent::Shop => (0.15, 0.15, 0.20, 0.20, 0.30),
            PageIntent::LeadGen => (0.15, 0.15, 0.15, 0.25, 0.30),
            PageIntent::Editorial => (0.20, 0.20, 0.25, 0.15, 0.20),
            PageIntent::Marketing => (0.25, 0.15, 0.15, 0.15, 0.30),
            PageIntent::Corporate => (0.20, 0.20, 0.25, 0.20, 0.15),
            PageIntent::Hub => (0.15, 0.25, 0.30, 0.15, 0.15),
            PageIntent::Unknown => (0.20, 0.20, 0.25, 0.20, 0.15),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PageIntent::Shop => "Shop / E-Commerce",
            PageIntent::LeadGen => "Lead-Generierung",
            PageIntent::Editorial => "Redaktionell / Blog",
            PageIntent::Marketing => "Marketing / Landing Page",
            PageIntent::Corporate => "Unternehmensseite",
            PageIntent::Hub => "Hub / Portal",
            PageIntent::Unknown => "Nicht erkannt",
        }
    }
}

// ── Detection keywords ─────────────────────────────────────────────

const SHOP_KEYWORDS: &[&str] = &[
    "warenkorb",
    "cart",
    "shop",
    "produkt",
    "product",
    "preis",
    "price",
    "bestellen",
    "order",
    "kaufen",
    "buy",
    "in den warenkorb",
    "add to cart",
    "checkout",
    "kasse",
    "artikel",
    "menge",
    "quantity",
];

const LEADGEN_KEYWORDS: &[&str] = &[
    "kontaktformular",
    "contact form",
    "anfrage",
    "request",
    "angebot",
    "quote",
    "termin",
    "appointment",
    "beratung",
    "consultation",
    "newsletter",
    "subscribe",
    "registrieren",
    "sign up",
    "demo",
];

const EDITORIAL_KEYWORDS: &[&str] = &[
    "artikel",
    "article",
    "blog",
    "beitrag",
    "post",
    "autor",
    "author",
    "veröffentlicht",
    "published",
    "lesezeit",
    "reading time",
    "kommentar",
    "comment",
    "kategorie",
    "category",
    "tag",
];

const CORPORATE_KEYWORDS: &[&str] = &[
    "über uns",
    "about us",
    "team",
    "karriere",
    "career",
    "unternehmen",
    "company",
    "mission",
    "vision",
    "standort",
    "location",
    "partner",
    "referenz",
    "reference",
    "geschichte",
    "history",
];

/// Detect the primary intent of a page from AXTree signals.
pub fn detect_page_intent(tree: &AXTree) -> PageIntent {
    let mut shop_score = 0u32;
    let mut leadgen_score = 0u32;
    let mut editorial_score = 0u32;
    let mut corporate_score = 0u32;

    let links = tree.links();
    let buttons = tree.nodes_with_role("button");
    let headings = tree.headings();
    let form_controls = tree.form_controls();

    // Scan all text-bearing nodes
    for node in tree.iter() {
        let name = match node.name.as_deref() {
            Some(n) if !n.is_empty() => n.to_lowercase(),
            _ => continue,
        };

        for kw in SHOP_KEYWORDS {
            if name.contains(kw) {
                shop_score += 1;
            }
        }
        for kw in LEADGEN_KEYWORDS {
            if name.contains(kw) {
                leadgen_score += 1;
            }
        }
        for kw in EDITORIAL_KEYWORDS {
            if name.contains(kw) {
                editorial_score += 1;
            }
        }
        for kw in CORPORATE_KEYWORDS {
            if name.contains(kw) {
                corporate_score += 1;
            }
        }
    }

    // Form controls boost leadgen
    if form_controls.len() >= 3 {
        leadgen_score += 3;
    }

    // Many links = hub
    let link_count = links.len();
    if link_count > 80 {
        return PageIntent::Hub;
    }

    // Long text content = editorial
    let text_len: usize = tree
        .iter()
        .filter(|n| matches!(n.role.as_deref(), Some("StaticText" | "paragraph")))
        .filter_map(|n| n.name.as_ref())
        .map(|n| n.len())
        .sum();
    let approx_words = text_len / 6;
    if approx_words > 500 && editorial_score >= 2 {
        editorial_score += 5;
    }

    // Few buttons + few form controls + many headings = editorial
    if buttons.len() < 3 && form_controls.is_empty() && headings.len() > 4 {
        editorial_score += 3;
    }

    let scores = [
        (shop_score, PageIntent::Shop),
        (leadgen_score, PageIntent::LeadGen),
        (editorial_score, PageIntent::Editorial),
        (corporate_score, PageIntent::Corporate),
    ];

    let (best_score, best_intent) = scores
        .iter()
        .max_by_key(|(s, _)| *s)
        .copied()
        .unwrap_or((0, PageIntent::Unknown));

    if best_score >= 3 {
        best_intent
    } else {
        // Marketing is the default for landing-page-like structure
        if link_count < 30 && !buttons.is_empty() && approx_words < 300 {
            PageIntent::Marketing
        } else {
            PageIntent::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXTree;

    #[test]
    fn test_unknown_for_empty_tree() {
        let tree = AXTree::new();
        let intent = detect_page_intent(&tree);
        // Empty tree -> Unknown or Marketing depending on heuristic
        assert!(matches!(
            intent,
            PageIntent::Unknown | PageIntent::Marketing
        ));
    }

    #[test]
    fn test_weights_sum_to_one() {
        for intent in &[
            PageIntent::Shop,
            PageIntent::LeadGen,
            PageIntent::Editorial,
            PageIntent::Marketing,
            PageIntent::Corporate,
            PageIntent::Hub,
            PageIntent::Unknown,
        ] {
            let (a, b, c, d, e) = intent.weights();
            let sum = a + b + c + d + e;
            assert!(
                (sum - 1.0).abs() < 0.001,
                "{:?} weights sum to {} instead of 1.0",
                intent,
                sum
            );
        }
    }
}
