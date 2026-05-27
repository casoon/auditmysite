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
    // English
    "cart",
    "shop",
    "product",
    "price",
    "order",
    "buy",
    "add to cart",
    "checkout",
    "quantity",
    "stock",
    "in stock",
    // German
    "warenkorb",
    "produkt",
    "preis",
    "bestellen",
    "kaufen",
    "in den warenkorb",
    "kasse",
    "artikel",
    "menge",
    "vorrat",
    "auf lager",
    // French
    "panier",
    "produit",
    "prix",
    "commander",
    "acheter",
    "ajouter au panier",
    "passer la commande",
    "quantité",
    "en stock",
    // Spanish
    "carrito",
    "producto",
    "precio",
    "pedir",
    "comprar",
    "añadir al carrito",
    "pagar",
    "cantidad",
    "en stock",
    // Italian
    "carrello",
    "prodotto",
    "prezzo",
    "ordinare",
    "acquistare",
    "aggiungi al carrello",
    "cassa",
    "quantità",
    "disponibile",
    // Portuguese
    "carrinho",
    "produto",
    "preço",
    "pedir",
    "comprar",
    "adicionar ao carrinho",
    "pagamento",
    "quantidade",
    "em estoque",
    // Dutch
    "winkelwagen",
    "product",
    "prijs",
    "bestellen",
    "kopen",
    "in winkelwagen",
    "afrekenen",
    "hoeveelheid",
    "op voorraad",
    // Swedish
    "kundvagn",
    "produkt",
    "pris",
    "beställa",
    "köpa",
    "lägg i kundvagn",
    "kassa",
    "antal",
    "i lager",
    // Polish
    "koszyk",
    "produkt",
    "cena",
    "zamówić",
    "kupić",
    "dodaj do koszyka",
    "kasa",
    "ilość",
    // Turkish
    "sepet",
    "ürün",
    "fiyat",
    "sipariş ver",
    "satın al",
    "sepete ekle",
    "ödeme",
    "adet",
    "stokta",
];

const LEADGEN_KEYWORDS: &[&str] = &[
    // English
    "contact form",
    "request",
    "quote",
    "appointment",
    "consultation",
    "newsletter",
    "subscribe",
    "sign up",
    "demo",
    "free trial",
    "get in touch",
    // German
    "kontaktformular",
    "anfrage",
    "angebot",
    "termin",
    "beratung",
    "registrieren",
    "kostenlos testen",
    "rückruf",
    // French
    "formulaire de contact",
    "demande",
    "devis",
    "rendez-vous",
    "consultation",
    "s'abonner",
    "s'inscrire",
    "essai gratuit",
    "nous contacter",
    // Spanish
    "formulario de contacto",
    "solicitud",
    "presupuesto",
    "cita",
    "asesoramiento",
    "suscribirse",
    "registrarse",
    "prueba gratuita",
    "contactar",
    // Italian
    "modulo di contatto",
    "richiesta",
    "preventivo",
    "appuntamento",
    "consulenza",
    "iscriversi",
    "registrarsi",
    "prova gratuita",
    "contattaci",
    // Portuguese
    "formulário de contato",
    "solicitação",
    "orçamento",
    "consulta",
    "assinar",
    "registrar",
    "teste gratuito",
    "entrar em contato",
    // Dutch
    "contactformulier",
    "aanvraag",
    "offerte",
    "afspraak",
    "advies",
    "abonneren",
    "registreren",
    "gratis proberen",
    "contact opnemen",
    // Swedish
    "kontaktformulär",
    "förfrågan",
    "offert",
    "möte",
    "rådgivning",
    "prenumerera",
    "registrera",
    "gratis provperiod",
    // Polish
    "formularz kontaktowy",
    "zapytanie",
    "wycena",
    "spotkanie",
    "konsultacja",
    "subskrybować",
    "rejestracja",
    "bezpłatna wersja próbna",
    // Turkish
    "iletişim formu",
    "talep",
    "teklif",
    "randevu",
    "danışmanlık",
    "abone ol",
    "kayıt ol",
    "ücretsiz deneme",
];

const EDITORIAL_KEYWORDS: &[&str] = &[
    // English
    "article",
    "blog",
    "post",
    "author",
    "published",
    "reading time",
    "comment",
    "category",
    "tag",
    "min read",
    "related articles",
    // German
    "artikel",
    "beitrag",
    "autor",
    "veröffentlicht",
    "lesezeit",
    "kommentar",
    "kategorie",
    "ähnliche artikel",
    "weiterlesen",
    // French
    "article",
    "billet",
    "auteur",
    "publié",
    "temps de lecture",
    "commentaire",
    "catégorie",
    "articles similaires",
    "lire la suite",
    // Spanish
    "artículo",
    "entrada",
    "autor",
    "publicado",
    "tiempo de lectura",
    "comentario",
    "categoría",
    "artículos relacionados",
    "leer más",
    // Italian
    "articolo",
    "post",
    "autore",
    "pubblicato",
    "tempo di lettura",
    "commento",
    "categoria",
    "articoli correlati",
    "leggi di più",
    // Portuguese
    "artigo",
    "postagem",
    "autor",
    "publicado",
    "tempo de leitura",
    "comentário",
    "categoria",
    "artigos relacionados",
    "leia mais",
    // Dutch
    "artikel",
    "bericht",
    "auteur",
    "gepubliceerd",
    "leestijd",
    "reactie",
    "categorie",
    "gerelateerde artikelen",
    "lees meer",
    // Swedish
    "artikel",
    "inlägg",
    "författare",
    "publicerad",
    "lästid",
    "kommentar",
    "kategori",
    "relaterade artiklar",
    "läs mer",
    // Polish
    "artykuł",
    "wpis",
    "autor",
    "opublikowano",
    "czas czytania",
    "komentarz",
    "kategoria",
    "powiązane artykuły",
    "czytaj więcej",
    // Turkish
    "makale",
    "gönderi",
    "yazar",
    "yayınlandı",
    "okuma süresi",
    "yorum",
    "kategori",
    "ilgili makaleler",
    "devamını oku",
];

const CORPORATE_KEYWORDS: &[&str] = &[
    // English
    "about us",
    "team",
    "career",
    "company",
    "mission",
    "vision",
    "location",
    "partner",
    "reference",
    "history",
    "values",
    "leadership",
    "press",
    "investor",
    // German — public sector / government
    "rathaus",
    "gemeinde",
    "verwaltung",
    "bürgermeister",
    "bürgermeisterin",
    "stadtrat",
    "stadtvertretung",
    "stadtgebiet",
    "bürgeramt",
    "einwohnermeldeamt",
    "standesamt",
    "bauamt",
    "hauptamt",
    "finanzamt",
    "landratsamt",
    "kreistag",
    "behörde",
    "behörden",
    "amt",
    "ämter",
    "amtsblatt",
    "bekanntmachung",
    "bürgerinformation",
    "bürgerservice",
    "ostseebad",
    // German — corporate
    "über uns",
    "karriere",
    "unternehmen",
    "standort",
    "referenz",
    "geschichte",
    "werte",
    "führung",
    "presse",
    "investor",
    // French
    "à propos",
    "équipe",
    "carrière",
    "entreprise",
    "mission",
    "vision",
    "emplacement",
    "partenaire",
    "référence",
    "histoire",
    "valeurs",
    "direction",
    "presse",
    // Spanish
    "sobre nosotros",
    "equipo",
    "carrera",
    "empresa",
    "misión",
    "visión",
    "ubicación",
    "socio",
    "referencia",
    "historia",
    "valores",
    "dirección",
    "prensa",
    // Italian
    "chi siamo",
    "team",
    "carriera",
    "azienda",
    "missione",
    "visione",
    "sede",
    "partner",
    "riferimento",
    "storia",
    "valori",
    "leadership",
    "stampa",
    // Portuguese
    "sobre nós",
    "equipe",
    "carreira",
    "empresa",
    "missão",
    "visão",
    "localização",
    "parceiro",
    "referência",
    "história",
    "valores",
    "imprensa",
    // Dutch
    "over ons",
    "team",
    "carrière",
    "bedrijf",
    "missie",
    "visie",
    "locatie",
    "partner",
    "referentie",
    "geschiedenis",
    "waarden",
    "leiderschap",
    "pers",
    // Swedish
    "om oss",
    "team",
    "karriär",
    "företag",
    "mission",
    "vision",
    "plats",
    "partner",
    "referens",
    "historia",
    "värderingar",
    "press",
    // Polish
    "o nas",
    "zespół",
    "kariera",
    "firma",
    "misja",
    "wizja",
    "lokalizacja",
    "partner",
    "referencje",
    "historia",
    "wartości",
    "prasa",
    // Turkish
    "hakkımızda",
    "ekip",
    "kariyer",
    "şirket",
    "misyon",
    "vizyon",
    "konum",
    "ortak",
    "referans",
    "tarihçe",
    "değerler",
    "basın",
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

    // Many links = hub. Threshold of 40 catches portal/navigation-heavy pages
    // where the AX tree may suppress some links (collapsed nav, hidden elements),
    // causing the raw link_count to be lower than the actual DOM link count.
    let link_count = links.len();
    if link_count > 40 {
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
