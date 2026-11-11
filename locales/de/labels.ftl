# Labels of cross-cutting enums (#936): the page intent (`page_flow::PageIntent`),
# the execution priority (`audit::prioritization`), the issue class
# (`taxonomy::IssueClass`) and the subcategory (`taxonomy::Subcategory`), and
# the evidence labels of content signals (`assessment`). One message per
# variant; which variant applies stays in Rust. The action priority, effort
# and responsible role have their one message per variant in report.ftl
# (`priority-*`, `effort-*`, `effort-short-*`, `role-*`).

labels-page-intent-shop = Shop / E-Commerce
labels-page-intent-lead-gen = Lead-Generierung
labels-page-intent-editorial = Redaktionell / Blog
labels-page-intent-marketing = Marketing / Landing Page
labels-page-intent-corporate = Unternehmensseite
labels-page-intent-hub = Hub / Portal
labels-page-intent-unknown = Nicht erkannt

labels-execution-priority-immediate = Dringend
labels-execution-priority-important = Hoch
labels-execution-priority-optional = Standard

labels-issue-class-missing = Fehlend
labels-issue-class-invalid = Ungültig
labels-issue-class-weak = Schwach
labels-issue-class-risk = Risiko
labels-issue-class-opportunity = Optimierungspotenzial
labels-issue-class-informational = Hinweis

labels-subcategory-content-alternatives = Inhalte & Alternativen
labels-subcategory-structure-semantics = Struktur & Semantik
labels-subcategory-navigation-interaction = Navigation & Bedienung
labels-subcategory-forms-interaction = Formulare & Interaktion
labels-subcategory-language-clarity = Sprache & Verständlichkeit
labels-subcategory-technical-robustness = Technische Robustheit
labels-subcategory-visual-presentation = Visuelle Darstellung
labels-subcategory-load-behavior = Ladeverhalten
labels-subcategory-interactivity = Interaktivität
labels-subcategory-visual-stability = Stabilität
labels-subcategory-resource-usage = Ressourcenverbrauch
labels-subcategory-technical-complexity = Technische Komplexität
labels-subcategory-snippet-metadata = Snippet & Metadaten
labels-subcategory-content-structure = Inhaltsstruktur
labels-subcategory-indexability = Indexierbarkeit
labels-subcategory-linking = Verlinkung
labels-subcategory-semantic-signals = Semantische Signale
labels-subcategory-transport = Transport
labels-subcategory-headers = Security Headers
labels-subcategory-browser-protection = Browser-Schutz
labels-subcategory-server-configuration = Server-Konfiguration
labels-subcategory-viewport = Viewport
labels-subcategory-touch-usability = Touch & Bedienbarkeit
labels-subcategory-readability = Lesbarkeit
labels-subcategory-responsive-layout = Responsive Layout
labels-subcategory-content-sizing = Content Sizing

# Labels of the evidence of a content signal (`assessment`): the source
# (`EvidenceSource::label`) and the readable names of the field paths
# `evidence_field_label` knows; which path maps to which message, and that
# other paths are shown as they are or dropped, stays in Rust.
labels-evidence-source-meta = Meta-Tag
labels-evidence-source-json-ld = JSON-LD
labels-evidence-source-visible-text = Sichtbarer Text
labels-evidence-source-link = Link
labels-evidence-source-http-header = HTTP-Header
labels-evidence-source-ax-tree = Accessibility-Tree
labels-evidence-source-dom-attribute = DOM-Attribut
labels-evidence-source-css-property = CSS-Eigenschaft
labels-evidence-source-computed = Abgeleiteter Wert
labels-evidence-field-heading-h1 = H1
labels-evidence-field-meta-title = Titel
labels-evidence-field-meta-description = Meta-Beschreibung
labels-evidence-field-ssr-content-gap = Server-Rendering-Lücke
labels-evidence-field-canonical = Canonical
labels-evidence-field-patterns-recognized = Erkannte Muster
labels-evidence-field-structured-data-type = Typ der strukturierten Daten
labels-evidence-field-structured-data-rich-snippets = Rich-Snippet-Potenzial
labels-evidence-field-structured-data = Strukturierte Daten
labels-evidence-field-headings-h2-plus = Überschriften ab H2
labels-evidence-field-internal-links = Interne Links
labels-evidence-field-word-count = Wortanzahl
