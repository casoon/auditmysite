# Labels of cross-cutting enums (#936): the page intent (`page_flow::PageIntent`),
# the action priority, effort and execution priority
# (`audit::prioritization`), the responsible role
# (`output::report_model::Role`), the issue class (`taxonomy::IssueClass`) and
# the subcategory (`taxonomy::Subcategory`). One message per variant; which
# variant applies stays in Rust. The prefix `labels-` keeps them apart from the
# PDF table labels of the same enums in report.ftl, which word some of them
# differently.

labels-page-intent-shop = Shop / E-Commerce
labels-page-intent-lead-gen = Lead generation
labels-page-intent-editorial = Editorial / Blog
labels-page-intent-marketing = Marketing / Landing Page
labels-page-intent-corporate = Corporate page
labels-page-intent-hub = Hub / Portal
labels-page-intent-unknown = Not detected

labels-priority-critical = Critical
labels-priority-high = High
labels-priority-medium = Medium
labels-priority-low = Low

labels-effort-quick = Low complexity
labels-effort-medium = Medium effort
labels-effort-structural = High complexity

labels-execution-priority-immediate = Urgent
labels-execution-priority-important = High
labels-execution-priority-optional = Standard

labels-role-development = Development
labels-role-editorial = Content
labels-role-design-ux = Design / UX
labels-role-project-management = Project management

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
