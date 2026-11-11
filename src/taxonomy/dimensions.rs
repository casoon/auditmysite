//! Audit-Dimensionen und Subkategorien

use serde::{Deserialize, Serialize};

/// Die 5 Audit-Dimensionen (Top-Level Produktkategorien)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dimension {
    #[default]
    Accessibility,
    Performance,
    Seo,
    Security,
    Mobile,
}

impl Dimension {
    /// Nutzerfreundlicher Label (`en = true` englisch, sonst deutsch).
    /// Die Dimensions-Namen sind in beiden Sprachen identisch.
    pub fn label(&self, _en: bool) -> &'static str {
        match self {
            Dimension::Accessibility => "Accessibility",
            Dimension::Performance => "Performance",
            Dimension::Seo => "SEO",
            Dimension::Security => "Security",
            Dimension::Mobile => "Mobile",
        }
    }
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label(false))
    }
}

/// Subkategorien innerhalb jeder Dimension
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subcategory {
    // ── Accessibility ──
    /// Inhalte & Alternativen (WCAG 1.1.x, 1.2.x)
    #[default]
    ContentAlternatives,
    /// Struktur & Semantik (WCAG 1.3.x)
    StructureSemantics,
    /// Navigation & Bedienung (WCAG 2.1.x, 2.4.x)
    NavigationInteraction,
    /// Formulare & Interaktion (WCAG 1.3.5, 2.5.x, 3.3.x)
    FormsInteraction,
    /// Sprache & Verständlichkeit (WCAG 3.1.x, 3.2.x)
    LanguageClarity,
    /// Technische Robustheit (WCAG 4.1.x)
    TechnicalRobustness,
    /// Wahrnehmbarkeit / Kontrast (WCAG 1.4.x)
    VisualPresentation,

    // ── Performance ──
    /// Ladeverhalten (FCP, LCP, TTFB)
    LoadBehavior,
    /// Interaktivität (TBT, INP)
    Interactivity,
    /// Visuelle Stabilität (CLS)
    VisualStability,
    /// Ressourcenverbrauch (JS-Heap, Requests)
    ResourceUsage,
    /// Technische Komplexität (DOM-Größe)
    TechnicalComplexity,

    // ── SEO ──
    /// Snippet & Metadaten (Title, Description)
    SnippetMetadata,
    /// Inhaltsstruktur (Headings)
    ContentStructure,
    /// Indexierbarkeit (Canonical, Robots)
    Indexability,
    /// Verlinkung (intern/extern)
    Linking,
    /// Semantische Signale (structured data, Open Graph)
    SemanticSignals,

    // ── Security ──
    /// Transport (HTTPS, Zertifikate)
    Transport,
    /// Security Headers (HSTS, CSP, X-Frame-Options)
    Headers,
    /// Browser-Schutzmechanismen (XSS Protection, etc.)
    BrowserProtection,
    /// Server-Konfiguration
    ServerConfiguration,

    // ── Mobile ──
    /// Viewport-Konfiguration
    Viewport,
    /// Touch & Bedienbarkeit
    TouchUsability,
    /// Lesbarkeit (Font-Größen)
    Readability,
    /// Responsive Layout
    ResponsiveLayout,
    /// Content Sizing
    ContentSizing,
}

impl Subcategory {
    /// Zugehörige Dimension
    pub fn dimension(&self) -> Dimension {
        match self {
            // Accessibility
            Self::ContentAlternatives
            | Self::StructureSemantics
            | Self::NavigationInteraction
            | Self::FormsInteraction
            | Self::LanguageClarity
            | Self::TechnicalRobustness
            | Self::VisualPresentation => Dimension::Accessibility,

            // Performance
            Self::LoadBehavior
            | Self::Interactivity
            | Self::VisualStability
            | Self::ResourceUsage
            | Self::TechnicalComplexity => Dimension::Performance,

            // SEO
            Self::SnippetMetadata
            | Self::ContentStructure
            | Self::Indexability
            | Self::Linking
            | Self::SemanticSignals => Dimension::Seo,

            // Security
            Self::Transport
            | Self::Headers
            | Self::BrowserProtection
            | Self::ServerConfiguration => Dimension::Security,

            // Mobile
            Self::Viewport
            | Self::TouchUsability
            | Self::Readability
            | Self::ResponsiveLayout
            | Self::ContentSizing => Dimension::Mobile,
        }
    }

    /// Nutzerfreundlicher Label (`en = true` englisch, sonst deutsch): die
    /// `labels-subcategory-*`-Message aus `locales/{en,de}/labels.ftl`.
    pub fn label(&self, en: bool) -> String {
        let key = match self {
            Self::ContentAlternatives => "labels-subcategory-content-alternatives",
            Self::StructureSemantics => "labels-subcategory-structure-semantics",
            Self::NavigationInteraction => "labels-subcategory-navigation-interaction",
            Self::FormsInteraction => "labels-subcategory-forms-interaction",
            Self::LanguageClarity => "labels-subcategory-language-clarity",
            Self::TechnicalRobustness => "labels-subcategory-technical-robustness",
            Self::VisualPresentation => "labels-subcategory-visual-presentation",
            Self::LoadBehavior => "labels-subcategory-load-behavior",
            Self::Interactivity => "labels-subcategory-interactivity",
            Self::VisualStability => "labels-subcategory-visual-stability",
            Self::ResourceUsage => "labels-subcategory-resource-usage",
            Self::TechnicalComplexity => "labels-subcategory-technical-complexity",
            Self::SnippetMetadata => "labels-subcategory-snippet-metadata",
            Self::ContentStructure => "labels-subcategory-content-structure",
            Self::Indexability => "labels-subcategory-indexability",
            Self::Linking => "labels-subcategory-linking",
            Self::SemanticSignals => "labels-subcategory-semantic-signals",
            Self::Transport => "labels-subcategory-transport",
            Self::Headers => "labels-subcategory-headers",
            Self::BrowserProtection => "labels-subcategory-browser-protection",
            Self::ServerConfiguration => "labels-subcategory-server-configuration",
            Self::Viewport => "labels-subcategory-viewport",
            Self::TouchUsability => "labels-subcategory-touch-usability",
            Self::Readability => "labels-subcategory-readability",
            Self::ResponsiveLayout => "labels-subcategory-responsive-layout",
            Self::ContentSizing => "labels-subcategory-content-sizing",
        };
        crate::i18n::text(key, en)
    }
}
