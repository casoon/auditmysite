# Labels of cross-cutting enums (#936): the page intent (`page_flow::PageIntent`),
# the action priority, effort and execution priority
# (`audit::prioritization`), the responsible role
# (`output::report_model::Role`), the issue class (`taxonomy::IssueClass`) and
# the subcategory (`taxonomy::Subcategory`). One message per variant; which
# variant applies stays in Rust. The prefix `labels-` keeps them apart from the
# PDF table labels of the same enums in report.ftl, which word some of them
# differently.

labels-page-intent-shop = Shop / E-Commerce
labels-page-intent-lead-gen = Lead-Generierung
labels-page-intent-editorial = Redaktionell / Blog
labels-page-intent-marketing = Marketing / Landing Page
labels-page-intent-corporate = Unternehmensseite
labels-page-intent-hub = Hub / Portal
labels-page-intent-unknown = Nicht erkannt

labels-priority-critical = Kritisch
labels-priority-high = Hoch
labels-priority-medium = Mittel
labels-priority-low = Niedrig

labels-effort-quick = Geringe Komplexität
labels-effort-medium = Mittlerer Aufwand
labels-effort-structural = Hohe Komplexität

labels-execution-priority-immediate = Dringend
labels-execution-priority-important = Hoch
labels-execution-priority-optional = Standard

labels-role-development = Entwicklung
labels-role-editorial = Redaktion
labels-role-design-ux = Design / UX
labels-role-project-management = Projektleitung

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
