# Labels of cross-cutting enums (#936): the page intent (`page_flow::PageIntent`),
# the execution priority (`audit::prioritization`), the issue class
# (`taxonomy::IssueClass`) and the subcategory (`taxonomy::Subcategory`), and
# the evidence labels of content signals (`assessment`). One message per
# variant; which variant applies stays in Rust. The action priority, effort
# and responsible role have their one message per variant in report.ftl
# (`priority-*`, `effort-*`, `effort-short-*`, `role-*`).

labels-page-intent-shop = Shop / E-Commerce
labels-page-intent-lead-gen = Lead generation
labels-page-intent-editorial = Editorial / Blog
labels-page-intent-marketing = Marketing / Landing Page
labels-page-intent-corporate = Corporate page
labels-page-intent-hub = Hub / Portal
labels-page-intent-unknown = Not detected

labels-execution-priority-immediate = Urgent
labels-execution-priority-important = High
labels-execution-priority-optional = Standard

labels-issue-class-missing = Missing
labels-issue-class-invalid = Invalid
labels-issue-class-weak = Weak
labels-issue-class-risk = Risk
labels-issue-class-opportunity = Opportunity
labels-issue-class-informational = Informational

labels-subcategory-content-alternatives = Content & Alternatives
labels-subcategory-structure-semantics = Structure & Semantics
labels-subcategory-navigation-interaction = Navigation & Operation
labels-subcategory-forms-interaction = Forms & Interaction
labels-subcategory-language-clarity = Language & Clarity
labels-subcategory-technical-robustness = Technical Robustness
labels-subcategory-visual-presentation = Visual Presentation
labels-subcategory-load-behavior = Load Behavior
labels-subcategory-interactivity = Interactivity
labels-subcategory-visual-stability = Stability
labels-subcategory-resource-usage = Resource Usage
labels-subcategory-technical-complexity = Technical Complexity
labels-subcategory-snippet-metadata = Snippet & Metadata
labels-subcategory-content-structure = Content Structure
labels-subcategory-indexability = Indexability
labels-subcategory-linking = Linking
labels-subcategory-semantic-signals = Semantic Signals
labels-subcategory-transport = Transport
labels-subcategory-headers = Security Headers
labels-subcategory-browser-protection = Browser Protection
labels-subcategory-server-configuration = Server Configuration
labels-subcategory-viewport = Viewport
labels-subcategory-touch-usability = Touch & Usability
labels-subcategory-readability = Readability
labels-subcategory-responsive-layout = Responsive Layout
labels-subcategory-content-sizing = Content Sizing

# Labels of the evidence of a content signal (`assessment`): the source
# (`EvidenceSource::label`) and the readable names of the field paths
# `evidence_field_label` knows; which path maps to which message, and that
# other paths are shown as they are or dropped, stays in Rust.
labels-evidence-source-meta = Meta tag
labels-evidence-source-json-ld = JSON-LD
labels-evidence-source-visible-text = Visible text
labels-evidence-source-link = Link
labels-evidence-source-http-header = HTTP header
labels-evidence-source-ax-tree = Accessibility tree
labels-evidence-source-dom-attribute = DOM attribute
labels-evidence-source-css-property = CSS property
labels-evidence-source-computed = Derived value
labels-evidence-field-heading-h1 = H1
labels-evidence-field-meta-title = Title
labels-evidence-field-meta-description = Meta description
labels-evidence-field-ssr-content-gap = Server-rendering gap
labels-evidence-field-canonical = Canonical
labels-evidence-field-patterns-recognized = Recognized patterns
labels-evidence-field-structured-data-type = Structured data type
labels-evidence-field-structured-data-rich-snippets = Rich snippet potential
labels-evidence-field-structured-data = Structured data
labels-evidence-field-headings-h2-plus = Headings from H2
labels-evidence-field-internal-links = Internal links
labels-evidence-field-word-count = Word count
