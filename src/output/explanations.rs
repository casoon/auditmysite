//! Customer-facing explanations for WCAG rules
//!
//! Maps technical WCAG rule IDs to human-readable explanations in German and English,
//! suitable for non-technical stakeholders.

use crate::output::report_model::{Effort, ExampleBlock, Role};

/// Complete explanation for a WCAG rule, bilingual (German default, English alternative).
pub struct RuleExplanation {
    /// Customer-facing title in German
    pub customer_title: &'static str,
    /// Customer-facing title in English
    pub customer_title_en: &'static str,
    /// Layperson description of the issue (German)
    pub customer_description: &'static str,
    /// Layperson description of the issue (English)
    pub customer_description_en: &'static str,
    /// Who is affected and how (German)
    pub user_impact: &'static str,
    /// Who is affected and how (English)
    pub user_impact_en: &'static str,
    /// Why this typically happens (German)
    pub typical_cause: &'static str,
    /// Why this typically happens (English)
    pub typical_cause_en: &'static str,
    /// Recommendation in customer language (German)
    pub recommendation: &'static str,
    /// Recommendation in customer language (English)
    pub recommendation_en: &'static str,
    /// Technical note for developers (German)
    pub technical_note: &'static str,
    /// Technical note for developers (English)
    pub technical_note_en: &'static str,
    /// Primary responsible role
    pub responsible_role: Role,
    /// Estimated effort to fix
    pub effort_estimate: Effort,
    /// Optional code examples (bad, good, decorative)
    pub example_bad: Option<&'static str>,
    pub example_good: Option<&'static str>,
    pub example_decorative: Option<&'static str>,
}

impl RuleExplanation {
    /// Build example blocks from the static data
    pub fn examples(&self) -> Vec<ExampleBlock> {
        match (self.example_bad, self.example_good) {
            (Some(bad), Some(good)) => vec![ExampleBlock {
                bad: bad.to_string(),
                good: good.to_string(),
                decorative: self.example_decorative.map(|s| s.to_string()),
            }],
            _ => vec![],
        }
    }

    /// Customer-facing title for the given locale ("en" -> English, otherwise German).
    pub fn customer_title_for(&self, locale: &str) -> &'static str {
        if locale == "en" {
            self.customer_title_en
        } else {
            self.customer_title
        }
    }

    /// Customer description for the given locale.
    pub fn customer_description_for(&self, locale: &str) -> &'static str {
        if locale == "en" {
            self.customer_description_en
        } else {
            self.customer_description
        }
    }

    /// User impact text for the given locale.
    pub fn user_impact_for(&self, locale: &str) -> &'static str {
        if locale == "en" {
            self.user_impact_en
        } else {
            self.user_impact
        }
    }

    /// Typical cause for the given locale.
    pub fn typical_cause_for(&self, locale: &str) -> &'static str {
        if locale == "en" {
            self.typical_cause_en
        } else {
            self.typical_cause
        }
    }

    /// Recommendation for the given locale.
    pub fn recommendation_for(&self, locale: &str) -> &'static str {
        if locale == "en" {
            self.recommendation_en
        } else {
            self.recommendation
        }
    }

    /// Technical note for the given locale.
    pub fn technical_note_for(&self, locale: &str) -> &'static str {
        if locale == "en" {
            self.technical_note_en
        } else {
            self.technical_note
        }
    }
}

/// Look up the explanation for a rule by its WCAG ID (e.g., "1.1.1")
/// or taxonomy rule ID (e.g., "a11y.alt_text.missing")
pub fn get_explanation(rule_id: &str) -> Option<&'static RuleExplanation> {
    // Direct lookup by WCAG ID
    if let Some(expl) = EXPLANATIONS
        .iter()
        .find(|(id, _)| *id == rule_id)
        .map(|(_, e)| e)
    {
        return Some(expl);
    }
    // Fallback: if a taxonomy rule_id was passed, resolve to WCAG ID via legacy map
    if rule_id.contains('.') {
        use crate::taxonomy::rules::RULES;
        if let Some(rule) = RULES.iter().find(|r| r.id == rule_id) {
            if let Some(ext_ref) = rule.external_ref {
                // external_ref is "WCAG 1.1.1" — extract the number
                let wcag_id = ext_ref.strip_prefix("WCAG ").unwrap_or(ext_ref);
                return EXPLANATIONS
                    .iter()
                    .find(|(id, _)| *id == wcag_id)
                    .map(|(_, e)| e);
            }
        }
    }
    None
}

/// All WCAG rule explanations indexed by rule ID
static EXPLANATIONS: &[(&str, RuleExplanation)] = &[
    // ── 1. Perceivable ──────────────────────────────────────────────────────
    (
        "1.1.1",
        RuleExplanation {
            customer_title: "Fehlende Alternativtexte bei Bildern",
            customer_title_en: "Missing alternative text for images",
            customer_description:
                "Bilder auf der Website haben keinen beschreibenden Alternativtext. \
                 Dadurch können Screenreader den Bildinhalt nicht an blinde oder \
                 sehbeeinträchtigte Nutzer vermitteln.",
            customer_description_en:
                "Images on the website have no descriptive alternative text. \
                 As a result, screen readers cannot convey the image content to \
                 blind or visually impaired users.",
            user_impact:
                "Menschen mit Sehbeeinträchtigung erhalten an diesen Stellen keine oder \
                 nur unvollständige Information. Der Bildinhalt geht für sie vollständig verloren.",
            user_impact_en:
                "People with visual impairments receive no or only incomplete information \
                 at these points. The image content is entirely lost to them.",
            typical_cause:
                "Teaserbilder, Slider, redaktionell eingepflegte Medien ohne Pflichtfeld \
                 im CMS, oder dekorative Bilder, die nicht als solche markiert sind.",
            typical_cause_en:
                "Teaser images, sliders, or editorially uploaded media without a required \
                 field in the CMS, or decorative images that are not marked as such.",
            recommendation:
                "Für informative Bilder einen beschreibenden Alt-Text hinterlegen, der den \
                 Bildinhalt oder -zweck vermittelt. Rein dekorative Bilder mit einem leeren \
                 Alt-Attribut markieren (alt=\"\").",
            recommendation_en:
                "Provide a descriptive alt text for informative images that conveys the \
                 image content or purpose. Mark purely decorative images with an empty \
                 alt attribute (alt=\"\").",
            technical_note:
                "Informative Bilder: <img alt=\"Beschreibung\">. \
                 Dekorative Bilder: <img alt=\"\"> oder role=\"presentation\". \
                 CMS-Felder für Alt-Texte als Pflichtfeld konfigurieren.",
            technical_note_en:
                "Informative images: <img alt=\"description\">. \
                 Decorative images: <img alt=\"\"> or role=\"presentation\". \
                 Configure alt-text fields in the CMS as required.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "1.3.1",
        RuleExplanation {
            customer_title: "Fehlende semantische Struktur",
            customer_title_en: "Missing semantic structure",
            customer_description:
                "Inhalte sind visuell strukturiert (z. B. durch Größe oder Farbe), aber \
                 die Struktur ist nicht im HTML-Code hinterlegt. Screenreader und andere \
                 Hilfstechnologien können die Beziehungen zwischen Inhalten nicht erkennen.",
            customer_description_en:
                "Content is visually structured (e.g. by size or color), but the structure \
                 is not encoded in the HTML. Screen readers and other assistive technologies \
                 cannot recognize the relationships between pieces of content.",
            user_impact:
                "Nutzer mit Screenreader können Tabellen, Listen und Formulargruppen nicht \
                 korrekt navigieren. Die logische Struktur der Seite geht verloren.",
            user_impact_en:
                "Screen reader users cannot navigate tables, lists, and form groups \
                 correctly. The logical structure of the page is lost.",
            typical_cause:
                "Tabellen ohne korrekte Tabellenauszeichnung, fehlende Fieldsets bei \
                 Formularen, Listen als div-Elemente statt ul/ol, fehlende Landmarks.",
            typical_cause_en:
                "Tables without proper table markup, missing fieldsets in forms, lists \
                 implemented as divs instead of ul/ol, missing landmarks.",
            recommendation:
                "Inhalte semantisch korrekt auszeichnen: Tabellen mit <table>, <th>, <td>; \
                 Listen mit <ul>/<ol>; Formulare mit <fieldset> und <legend> gruppieren.",
            recommendation_en:
                "Mark up content with correct semantics: tables with <table>, <th>, <td>; \
                 lists with <ul>/<ol>; group form fields with <fieldset> and <legend>.",
            technical_note:
                "HTML5-Semantik nutzen: <nav>, <main>, <aside>, <header>, <footer>. \
                 Tabellen: scope-Attribute für Kopfzellen. ARIA-Rollen nur als Ergänzung.",
            technical_note_en:
                "Use HTML5 semantics: <nav>, <main>, <aside>, <header>, <footer>. \
                 Tables: use scope attributes on header cells. Use ARIA roles only as a supplement.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<div class=\"table\">...</div>"),
            example_good: Some("<table><thead><tr><th scope=\"col\">...</th></tr></thead>...</table>"),
            example_decorative: None,
        },
    ),
    (
        "1.3.5",
        RuleExplanation {
            customer_title: "Fehlende Eingabezweck-Kennzeichnung",
            customer_title_en: "Missing input purpose identification",
            customer_description:
                "Formularfelder haben keine maschinenlesbare Kennzeichnung ihres Zwecks. \
                 Browser und Hilfstechnologien können dadurch keine Autofill-Vorschläge \
                 machen und Nutzern nicht helfen, Formulare schneller auszufüllen.",
            customer_description_en:
                "Form fields lack machine-readable identification of their purpose. \
                 As a result, browsers and assistive technologies cannot offer autofill \
                 suggestions or help users complete forms faster.",
            user_impact:
                "Menschen mit motorischen oder kognitiven Einschränkungen können nicht von \
                 automatischer Formularausfüllung profitieren. Das Ausfüllen dauert länger \
                 und ist fehleranfälliger.",
            user_impact_en:
                "People with motor or cognitive impairments cannot benefit from automatic \
                 form filling. Completing forms takes longer and is more error-prone.",
            typical_cause:
                "Fehlende autocomplete-Attribute in Formularen für persönliche Daten \
                 (Name, E-Mail, Adresse, Telefon).",
            typical_cause_en:
                "Missing autocomplete attributes on form fields for personal data \
                 (name, email, address, phone).",
            recommendation:
                "Alle Formularfelder für persönliche Daten mit dem passenden \
                 autocomplete-Attribut versehen (z. B. autocomplete=\"email\", \
                 autocomplete=\"given-name\").",
            recommendation_en:
                "Add the appropriate autocomplete attribute to all form fields for \
                 personal data (e.g. autocomplete=\"email\", autocomplete=\"given-name\").",
            technical_note:
                "autocomplete-Werte gemäß HTML-Spezifikation verwenden: \
                 name, email, tel, street-address, postal-code, country, etc.",
            technical_note_en:
                "Use autocomplete values per the HTML specification: \
                 name, email, tel, street-address, postal-code, country, etc.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<input type=\"email\" name=\"email\">"),
            example_good: Some("<input type=\"email\" name=\"email\" autocomplete=\"email\">"),
            example_decorative: None,
        },
    ),
    (
        "1.4.1",
        RuleExplanation {
            customer_title: "Links nur durch Farbe erkennbar",
            customer_title_en: "Links distinguishable by color alone",
            customer_description:
                "Links im Fließtext lassen sich ausschließlich durch ihre Farbe von normalem \
                 Text unterscheiden. Für Menschen mit Farbsehschwäche sind diese Links \
                 nicht als klickbar erkennbar.",
            customer_description_en:
                "Links within body text are distinguishable from regular text only by their \
                 color. For people with color vision deficiency, these links are not \
                 recognizable as clickable.",
            user_impact:
                "Nutzer mit Rot-Grün-Schwäche oder anderen Farbsehschwächen können Links \
                 im Text nicht erkennen und verpassen so wichtige Navigationsmöglichkeiten.",
            user_impact_en:
                "Users with red-green deficiency or other color vision impairments cannot \
                 recognize links in text and miss important navigation options.",
            typical_cause:
                "CSS setzt `text-decoration: none` auf Links im Fließtext ohne ein \
                 alternatives nicht-farbliches Unterscheidungsmerkmal wie Unterstrich, \
                 Fettschrift oder ein Icon.",
            typical_cause_en:
                "CSS sets `text-decoration: none` on inline links without an alternative \
                 non-color distinguishing feature such as underline, bold, or an icon.",
            recommendation:
                "Links im Fließtext mit einem nicht-farblichen Merkmal kennzeichnen: \
                 Unterstrich (Standard und empfohlen), Fettschrift oder ein kleines Icon. \
                 Der Unterstrich ist die stärkste Konvention.",
            recommendation_en:
                "Mark inline links with a non-color cue: underline (standard and recommended), \
                 bold text, or a small icon. Underline is the strongest convention.",
            technical_note:
                "CSS: `a { text-decoration: underline; }` oder bei `text-decoration: none` \
                 mindestens `font-weight: bold` oder `border-bottom`. \
                 Gilt nur für Links im Fließtext, nicht für Links in Navigationselementen.",
            technical_note_en:
                "CSS: `a { text-decoration: underline; }` or, if `text-decoration: none`, \
                 at least `font-weight: bold` or `border-bottom`. \
                 Applies only to inline links in body text, not to navigation links.",
            responsible_role: Role::DesignUx,
            effort_estimate: Effort::Quick,
            example_bad: Some("a { color: #0057b8; text-decoration: none; }"),
            example_good: Some("a { color: #0057b8; text-decoration: underline; }"),
            example_decorative: None,
        },
    ),
    (
        "1.4.3",
        RuleExplanation {
            customer_title: "Unzureichender Farbkontrast",
            customer_title_en: "Insufficient color contrast",
            customer_description:
                "Text auf der Website hat nicht genügend Kontrast zum Hintergrund. \
                 Bei ungünstigen Lichtverhältnissen oder für Menschen mit \
                 Sehschwäche ist der Text schwer lesbar.",
            customer_description_en:
                "Text on the website has insufficient contrast against its background. \
                 Under poor lighting conditions or for people with low vision, the text \
                 is hard to read.",
            user_impact:
                "Menschen mit Sehbeeinträchtigung, ältere Nutzer und alle Nutzer bei \
                 ungünstigen Bildschirmbedingungen (Sonnenlicht, schlechte Displays) \
                 können Texte schlecht oder gar nicht lesen.",
            user_impact_en:
                "People with visual impairments, older users, and any user under poor \
                 screen conditions (sunlight, low-quality displays) can read the text \
                 only with difficulty or not at all.",
            typical_cause:
                "Helle Schriftfarbe auf hellem Hintergrund, graue Texte auf weißem Grund, \
                 Designentscheidungen ohne Kontrastprüfung.",
            typical_cause_en:
                "Light font color on a light background, gray text on white, \
                 design decisions made without contrast checking.",
            recommendation:
                "Schriftfarben anpassen, sodass normaler Text mindestens ein Kontrastverhältnis \
                 von 4,5:1 und großer Text von 3:1 zum Hintergrund hat. \
                 Kontrastwerte mit Tools wie dem WebAIM Contrast Checker prüfen.",
            recommendation_en:
                "Adjust font colors so normal text has a contrast ratio of at least 4.5:1 \
                 and large text at least 3:1 against the background. \
                 Verify contrast values with tools such as the WebAIM Contrast Checker.",
            technical_note:
                "WCAG 2.1 Level AA: 4.5:1 für normalen Text (<18pt / <14pt bold), \
                 3:1 für großen Text (≥18pt / ≥14pt bold). \
                 CSS-Variablen für konsistente Farbpalette verwenden.",
            technical_note_en:
                "WCAG 2.1 Level AA: 4.5:1 for normal text (<18pt / <14pt bold), \
                 3:1 for large text (≥18pt / ≥14pt bold). \
                 Use CSS variables for a consistent color palette.",
            responsible_role: Role::DesignUx,
            effort_estimate: Effort::Medium,
            example_bad: Some("color: #999999; background: #ffffff; /* Kontrast 2.8:1 */"),
            example_good: Some("color: #595959; background: #ffffff; /* Kontrast 7:1 */"),
            example_decorative: None,
        },
    ),
    (
        "1.4.4",
        RuleExplanation {
            customer_title: "Text nicht ausreichend skalierbar",
            customer_title_en: "Text cannot be sufficiently resized",
            customer_description:
                "Texte auf der Website können nicht ohne Verlust von Inhalt oder Funktion \
                 auf 200% vergrößert werden. Nutzer, die auf größere Schrift angewiesen sind, \
                 verlieren dadurch Inhalte.",
            customer_description_en:
                "Text on the website cannot be enlarged to 200% without loss of content \
                 or functionality. Users who depend on larger text lose information.",
            user_impact:
                "Menschen mit Sehbeeinträchtigung, die die Schriftgröße im Browser vergrößern, \
                 stoßen auf abgeschnittene Texte, überlagerte Elemente oder nicht \
                 scrollbare Bereiche.",
            user_impact_en:
                "People with visual impairments who increase the browser font size \
                 encounter clipped text, overlapping elements, or non-scrollable areas.",
            typical_cause:
                "Feste Pixelwerte für Schriftgrößen und Container, overflow: hidden \
                 bei Textcontainern, fehlende responsive Anpassungen.",
            typical_cause_en:
                "Fixed pixel values for font sizes and containers, overflow: hidden on \
                 text containers, missing responsive adjustments.",
            recommendation:
                "Schriftgrößen in relativen Einheiten (rem/em) angeben. Container so gestalten, \
                 dass sie bei vergrößertem Text mitwachsen. Browser-Zoom auf 200% testen.",
            recommendation_en:
                "Specify font sizes in relative units (rem/em). Design containers so they \
                 grow with enlarged text. Test browser zoom at 200%.",
            technical_note:
                "font-size in rem/em statt px. Container mit min-height statt height. \
                 overflow: auto statt hidden. @media-Queries für verschiedene Zoom-Stufen.",
            technical_note_en:
                "Use font-size in rem/em instead of px. Use min-height on containers \
                 instead of height. Prefer overflow: auto over hidden. Use @media queries \
                 for different zoom levels.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("font-size: 12px; height: 40px; overflow: hidden;"),
            example_good: Some("font-size: 0.75rem; min-height: 2.5rem; overflow: auto;"),
            example_decorative: None,
        },
    ),
    (
        "1.4.11",
        RuleExplanation {
            customer_title: "Unzureichender Kontrast bei UI-Elementen",
            customer_title_en: "Insufficient contrast on UI elements",
            customer_description:
                "Bedienelemente wie Buttons, Eingabefelder oder Icons haben nicht genügend \
                 Kontrast zum Hintergrund. Sie sind dadurch für manche Nutzer schwer erkennbar.",
            customer_description_en:
                "Interactive elements such as buttons, input fields, or icons have \
                 insufficient contrast against the background. They are therefore hard \
                 to recognize for some users.",
            user_impact:
                "Nutzer mit eingeschränktem Sehvermögen können interaktive Elemente nicht \
                 zuverlässig erkennen und bedienen.",
            user_impact_en:
                "Users with limited vision cannot reliably perceive and operate \
                 interactive elements.",
            typical_cause:
                "Subtile Rahmenfarben bei Formularelementen, Icons mit geringem Kontrast, \
                 Fokus-Indikatoren, die zu unauffällig sind.",
            typical_cause_en:
                "Subtle border colors on form elements, low-contrast icons, focus \
                 indicators that are too inconspicuous.",
            recommendation:
                "Alle interaktiven Elemente und ihre Zustände (Normal, Hover, Fokus) mit \
                 mindestens 3:1 Kontrast zum Hintergrund gestalten.",
            recommendation_en:
                "Design all interactive elements and their states (default, hover, focus) \
                 with a contrast ratio of at least 3:1 against the background.",
            technical_note:
                "3:1 Kontrastverhältnis für UI-Komponenten und grafische Objekte. \
                 Gilt für Rahmen, Icons, Slider, Checkboxen, etc.",
            technical_note_en:
                "3:1 contrast ratio for UI components and graphical objects. \
                 Applies to borders, icons, sliders, checkboxes, etc.",
            responsible_role: Role::DesignUx,
            effort_estimate: Effort::Medium,
            example_bad: Some("border: 1px solid #cccccc; /* auf #ffffff = 1.6:1 */"),
            example_good: Some("border: 1px solid #767676; /* auf #ffffff = 4.5:1 */"),
            example_decorative: None,
        },
    ),
    (
        "1.4.13",
        RuleExplanation {
            customer_title: "Inhalte bei Hover/Fokus nicht steuerbar",
            customer_title_en: "Content on hover or focus not controllable",
            customer_description:
                "Inhalte, die bei Hover oder Fokus eingeblendet werden (z. B. Tooltips, \
                 Dropdowns), verschwinden sofort, wenn die Maus bewegt wird, oder können \
                 nicht per Tastatur geschlossen werden.",
            customer_description_en:
                "Content that appears on hover or focus (e.g. tooltips, dropdowns) \
                 disappears immediately when the mouse is moved, or cannot be dismissed \
                 with the keyboard.",
            user_impact:
                "Nutzer mit motorischen Einschränkungen können Tooltip-Inhalte nicht \
                 vollständig lesen, bevor sie verschwinden. Screenreader-Nutzer können \
                 eingeblendete Inhalte möglicherweise nicht erreichen.",
            user_impact_en:
                "Users with motor impairments cannot fully read tooltip content before it \
                 disappears. Screen reader users may not be able to reach the revealed content.",
            typical_cause:
                "Tooltips oder Overlays, die `onmouseleave` sofort schließen, ohne \
                 dem Nutzer Zeit zu lassen, den Zeiger in den Tooltip zu bewegen. \
                 Fehlende Escape-Taste-Unterstützung.",
            typical_cause_en:
                "Tooltips or overlays that close on `onmouseleave` immediately, without \
                 giving the user time to move the pointer into the tooltip. \
                 Missing Escape-key support.",
            recommendation:
                "Hover-Inhalte so implementieren, dass sie bestehen bleiben, wenn der \
                 Zeiger in den Inhalt bewegt wird, per Escape-Taste schließbar sind und \
                 ausreichend lange sichtbar bleiben.",
            recommendation_en:
                "Implement hover content so it persists when the pointer moves into it, \
                 is dismissible with the Escape key, and remains visible long enough to read.",
            technical_note:
                "WCAG 2.1 Level AA: Hover-Inhalt muss (1) hoverbar sein, (2) dismissible \
                 (Escape), (3) persistent bleiben bis der Nutzer es schließt oder den \
                 Trigger verlässt. CSS-only-Tooltips via :hover reichen nicht.",
            technical_note_en:
                "WCAG 2.1 Level AA: Hover content must be (1) hoverable, (2) dismissible \
                 (Escape), (3) persistent until the user closes it or leaves the trigger. \
                 CSS-only tooltips via :hover are insufficient.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<div onmouseleave=\"hide()\">Tooltip</div>"),
            example_good: Some(
                "<div onmouseleave=\"scheduleHide()\" onmouseenter=\"cancelHide()\">\
                 Tooltip — schließbar mit Escape</div>",
            ),
            example_decorative: None,
        },
    ),
    // ── 2. Operable ─────────────────────────────────────────────────────────
    (
        "2.1.1",
        RuleExplanation {
            customer_title: "Inhalte nicht per Tastatur bedienbar",
            customer_title_en: "Content not operable by keyboard",
            customer_description:
                "Bestimmte Funktionen der Website können nur mit der Maus bedient werden. \
                 Nutzer, die auf die Tastatur angewiesen sind, können diese Inhalte \
                 nicht erreichen oder nutzen.",
            customer_description_en:
                "Certain features of the website can only be operated with a mouse. \
                 Users who rely on the keyboard cannot reach or use these features.",
            user_impact:
                "Menschen mit motorischen Einschränkungen, die keine Maus verwenden können, \
                 sowie Screenreader-Nutzer sind von diesen Funktionen ausgeschlossen.",
            user_impact_en:
                "People with motor impairments who cannot use a mouse, as well as \
                 screen reader users, are excluded from these features.",
            typical_cause:
                "Click-Handler nur auf div/span statt auf interaktive Elemente, \
                 fehlende tabindex-Attribute, JavaScript-Widgets ohne Tastaturunterstützung.",
            typical_cause_en:
                "Click handlers attached to div/span instead of interactive elements, \
                 missing tabindex attributes, JavaScript widgets without keyboard support.",
            recommendation:
                "Alle interaktiven Elemente mit nativen HTML-Elementen (button, a, input) \
                 umsetzen. Bei Custom-Widgets Tastaturnavigation (Tab, Enter, Escape, Pfeiltasten) \
                 implementieren.",
            recommendation_en:
                "Implement all interactive elements with native HTML elements \
                 (button, a, input). For custom widgets, implement keyboard navigation \
                 (Tab, Enter, Escape, arrow keys).",
            technical_note:
                "Native HTML-Elemente bevorzugen. Bei Custom-Widgets: tabindex=\"0\", \
                 keydown/keyup-Handler, ARIA-Rollen und -Zustände.",
            technical_note_en:
                "Prefer native HTML elements. For custom widgets: tabindex=\"0\", \
                 keydown/keyup handlers, ARIA roles and states.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<div onclick=\"doAction()\">Klick mich</div>"),
            example_good: Some("<button type=\"button\" onclick=\"doAction()\">Klick mich</button>"),
            example_decorative: None,
        },
    ),
    (
        "2.1.2",
        RuleExplanation {
            customer_title: "Tastaturfalle — Fokus kann nicht verlassen werden",
            customer_title_en: "Keyboard trap — focus cannot leave the element",
            customer_description:
                "An bestimmten Stellen der Website bleibt der Tastaturfokus hängen. \
                 Nutzer können mit der Tastatur nicht mehr vor- oder zurücknavigieren.",
            customer_description_en:
                "At certain points on the website, keyboard focus gets stuck. \
                 Users can no longer navigate forward or backward with the keyboard.",
            user_impact:
                "Tastaturnutzer sind an dieser Stelle gefangen und können den Rest \
                 der Seite nicht mehr erreichen — die Seite wird faktisch unbenutzbar.",
            user_impact_en:
                "Keyboard users are trapped at this point and cannot reach the rest of \
                 the page — the page effectively becomes unusable.",
            typical_cause:
                "Modale Dialoge ohne korrekte Fokus-Verwaltung, Widgets, die den Fokus \
                 abfangen, fehlende Escape-Taste zum Verlassen.",
            typical_cause_en:
                "Modal dialogs without proper focus management, widgets that capture \
                 focus, missing Escape key to dismiss.",
            recommendation:
                "Sicherstellen, dass der Fokus jedes Element mit Tab und Shift+Tab \
                 verlassen kann. Modale Dialoge mit Escape schließbar machen und den \
                 Fokus danach korrekt zurücksetzen.",
            recommendation_en:
                "Ensure focus can leave every element using Tab and Shift+Tab. \
                 Make modal dialogs dismissible with Escape and restore focus correctly \
                 afterwards.",
            technical_note:
                "Focus trapping nur in Modals mit korrekter Implementierung. \
                 Escape zum Schließen. Fokus-Rückgabe an das auslösende Element.",
            technical_note_en:
                "Use focus trapping only in modals with a correct implementation. \
                 Allow Escape to close. Return focus to the triggering element.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "2.4.1",
        RuleExplanation {
            customer_title: "Fehlende Sprungnavigation",
            customer_title_en: "Missing skip navigation",
            customer_description:
                "Die Website bietet keinen Mechanismus, um wiederkehrende Inhaltsblöcke \
                 (z. B. Navigation, Header) zu überspringen und direkt zum Hauptinhalt \
                 zu gelangen.",
            customer_description_en:
                "The website provides no mechanism to skip recurring blocks of content \
                 (e.g. navigation, header) and jump directly to the main content.",
            user_impact:
                "Tastaturnutzer und Screenreader-Nutzer müssen bei jedem Seitenwechsel \
                 erneut durch die gesamte Navigation tabben, bevor sie den Inhalt erreichen.",
            user_impact_en:
                "Keyboard and screen reader users must tab through the entire navigation \
                 again on every page change before they reach the content.",
            typical_cause:
                "Fehlender Skip-Link als erstes Element der Seite. \
                 WCAG 2.4.1 verlangt einen Mechanismus zum Überspringen wiederkehrender Blöcke — \
                 nicht zwingend Landmarks, aber ein sichtbarer Skip-Link ist die direkteste Lösung.",
            typical_cause_en:
                "Missing skip link as the first element on the page. \
                 WCAG 2.4.1 requires a mechanism to bypass repeated blocks — \
                 not necessarily landmarks, but a visible skip link is the most direct solution.",
            recommendation:
                "Einen sichtbaren Skip-Link ('Zum Inhalt springen') als erstes interaktives Element \
                 der Seite einbauen. HTML5-Landmarks (<nav>, <main>) sind ergänzend sinnvoll, \
                 ersetzen aber den Skip-Link nicht.",
            recommendation_en:
                "Add a visible skip link ('Skip to main content') as the first interactive \
                 element on the page. HTML5 landmarks (<nav>, <main>) are a useful addition \
                 but do not replace the skip link.",
            technical_note:
                "Skip-Link: <a href=\"#main\" class=\"skip-link\">Zum Inhalt springen</a>. \
                 Bei Fokus sichtbar machen via CSS :focus { clip: auto; position: static; }. \
                 Landmarks (<nav>, <main id=\"main\">) verbessern zusätzlich die Screenreader-Navigation.",
            technical_note_en:
                "Skip link: <a href=\"#main\" class=\"skip-link\">Skip to main content</a>. \
                 Make it visible on focus via CSS :focus { clip: auto; position: static; }. \
                 Landmarks (<nav>, <main id=\"main\">) additionally improve screen reader navigation.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<body><div class=\"nav\">...</div><div class=\"content\">...</div></body>"),
            example_good: Some("<body><a href=\"#main\" class=\"skip-link\">Zum Inhalt</a><nav>...</nav><main id=\"main\">...</main></body>"),
            example_decorative: None,
        },
    ),
    (
        "2.4.2",
        RuleExplanation {
            customer_title: "Fehlender oder unzureichender Seitentitel",
            customer_title_en: "Missing or insufficient page title",
            customer_description:
                "Die Seite hat keinen aussagekräftigen Titel im Browser-Tab. \
                 Nutzer können nicht erkennen, auf welcher Seite sie sich befinden.",
            customer_description_en:
                "The page has no meaningful title in the browser tab. \
                 Users cannot tell which page they are on.",
            user_impact:
                "Screenreader-Nutzer hören den Seitentitel als Erstes — ohne klaren Titel \
                 fehlt die Orientierung. Auch bei vielen offenen Tabs ist die Seite \
                 nicht zuzuordnen.",
            user_impact_en:
                "Screen reader users hear the page title first — without a clear title, \
                 they have no orientation. With many tabs open, the page is also hard \
                 to identify.",
            typical_cause:
                "Leeres <title>-Tag, generischer Titel ('Home', 'Untitled'), \
                 identischer Titel auf allen Seiten.",
            typical_cause_en:
                "Empty <title> tag, generic title ('Home', 'Untitled'), \
                 identical title on every page.",
            recommendation:
                "Jeder Seite einen eindeutigen, beschreibenden Titel geben, der den \
                 Seiteninhalt und die Website-Zugehörigkeit klar macht \
                 (z. B. 'Kontakt — Firmenname').",
            recommendation_en:
                "Give each page a unique, descriptive title that clearly states the page \
                 content and site context (e.g. 'Contact — Company Name').",
            technical_note:
                "<title>Seiteninhalt — Website</title>. Titel im CMS als Pflichtfeld. \
                 Pattern: Seitenspezifisch + Seitenname.",
            technical_note_en:
                "<title>Page content — Site</title>. Make the title a required field in \
                 the CMS. Pattern: page-specific + site name.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: Some("<title>Home</title>"),
            example_good: Some("<title>Kontakt — Casoon Digital Solutions</title>"),
            example_decorative: None,
        },
    ),
    (
        "2.4.3",
        RuleExplanation {
            customer_title: "Unlogische Fokus-Reihenfolge",
            customer_title_en: "Illogical focus order",
            customer_description:
                "Die Reihenfolge, in der interaktive Elemente per Tastatur erreicht werden, \
                 entspricht nicht der visuellen oder logischen Reihenfolge der Seite.",
            customer_description_en:
                "The order in which interactive elements are reached via keyboard does \
                 not match the visual or logical order of the page.",
            user_impact:
                "Tastaturnutzer erleben eine verwirrende Navigation — der Fokus springt \
                 zwischen unzusammenhängenden Bereichen hin und her.",
            user_impact_en:
                "Keyboard users experience confusing navigation — focus jumps back and \
                 forth between unrelated areas.",
            typical_cause:
                "Visuelles Layout per CSS umgeordnet ohne DOM-Reihenfolge anzupassen, \
                 positive tabindex-Werte, dynamisch eingefügte Elemente an falscher Stelle.",
            typical_cause_en:
                "Visual layout reordered via CSS without adjusting DOM order, positive \
                 tabindex values, dynamically inserted elements placed in the wrong spot.",
            recommendation:
                "Die DOM-Reihenfolge an die visuelle Reihenfolge anpassen. \
                 Auf positive tabindex-Werte verzichten. Bei dynamischen Inhalten \
                 den Fokus programmatisch steuern.",
            recommendation_en:
                "Align DOM order with the visual order. Avoid positive tabindex values. \
                 Manage focus programmatically for dynamic content.",
            technical_note:
                "tabindex nur 0 oder -1 verwenden. CSS-Order nicht für inhaltlich \
                 relevante Umordnungen nutzen. DOM-Reihenfolge = Lesereihenfolge.",
            technical_note_en:
                "Use only tabindex 0 or -1. Do not use CSS order for content-relevant \
                 reordering. DOM order = reading order.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "2.4.4",
        RuleExplanation {
            customer_title: "Unklare oder generische Linktexte",
            customer_title_en: "Unclear or generic link text",
            customer_description:
                "Links auf der Seite haben Texte wie 'hier', 'mehr', 'weiterlesen', \
                 die ohne Kontext nicht verständlich sind. Nutzer können nicht erkennen, \
                 wohin ein Link führt.",
            customer_description_en:
                "Links on the page use text like 'here', 'more', 'read more' that is \
                 not understandable out of context. Users cannot tell where a link leads.",
            user_impact:
                "Screenreader-Nutzer navigieren häufig über eine Linkliste — generische \
                 Texte wie 'mehr' sind dort völlig nichtssagend. Auch für alle anderen \
                 Nutzer ist die Orientierung erschwert.",
            user_impact_en:
                "Screen reader users often navigate via a list of links — generic text \
                 like 'more' is meaningless there. Orientation is also harder for all \
                 other users.",
            typical_cause:
                "Redaktionelle Gewohnheit, 'hier klicken' oder 'mehr erfahren' als Linktext \
                 zu verwenden. Teaser-Komponenten mit generischem 'Weiterlesen'.",
            typical_cause_en:
                "Editorial habit of using 'click here' or 'learn more' as link text. \
                 Teaser components with a generic 'Read more'.",
            recommendation:
                "Linktexte so formulieren, dass sie auch ohne umgebenden Text verständlich \
                 sind (z. B. 'Leistungen im Bereich Webentwicklung' statt 'mehr erfahren').",
            recommendation_en:
                "Phrase link text so it is understandable without surrounding context \
                 (e.g. 'Web development services' instead of 'learn more').",
            technical_note:
                "Sprechende Linktexte verwenden. Falls visuell knapper Text gewünscht: \
                 aria-label oder aria-labelledby für erweiterte Beschreibung.",
            technical_note_en:
                "Use descriptive link text. If a visually short text is required, use \
                 aria-label or aria-labelledby for an extended description.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: Some("<a href=\"/leistungen\">mehr erfahren</a>"),
            example_good: Some("<a href=\"/leistungen\">Unsere Leistungen im Bereich Webentwicklung</a>"),
            example_decorative: None,
        },
    ),
    (
        "2.4.6",
        RuleExplanation {
            customer_title: "Fehlende oder unzureichende Überschriftenstruktur",
            customer_title_en: "Missing or insufficient heading structure",
            customer_description:
                "Die Seite verwendet keine oder unlogische Überschriften. \
                 Die inhaltliche Gliederung ist für assistive Technologien nicht erkennbar.",
            customer_description_en:
                "The page uses no headings or has an illogical heading structure. \
                 The content outline is not recognizable to assistive technologies.",
            user_impact:
                "Screenreader-Nutzer können nicht per Überschrift durch die Seite navigieren. \
                 Die Seite wird zu einem undifferenzierten Textblock — schnelles Finden \
                 relevanter Abschnitte ist unmöglich.",
            user_impact_en:
                "Screen reader users cannot navigate the page by headings. \
                 The page becomes an undifferentiated block of text — quickly finding \
                 relevant sections is impossible.",
            typical_cause:
                "Keine Überschriften verwendet, nur visuelle Formatierung (fett, groß). \
                 Überschriftenhierarchie übersprungen (z. B. H1 → H3). \
                 Mehrere oder fehlende H1.",
            typical_cause_en:
                "No headings used, only visual formatting (bold, large). \
                 Heading hierarchy skipped (e.g. H1 → H3). \
                 Multiple or missing H1.",
            recommendation:
                "Eine logische Überschriftenhierarchie aufbauen: genau eine H1 pro Seite, \
                 darunter H2 für Hauptabschnitte, H3 für Unterabschnitte. \
                 Keine Ebenen überspringen.",
            recommendation_en:
                "Build a logical heading hierarchy: exactly one H1 per page, H2 for \
                 main sections, H3 for sub-sections. Do not skip levels.",
            technical_note:
                "H1 = Seitentitel (einmal). H2-H6 hierarchisch verschachtelt. \
                 Keine Ebenen überspringen. Heading-Outline mit Browser-Extension prüfen.",
            technical_note_en:
                "H1 = page title (once). H2–H6 nested hierarchically. \
                 Do not skip levels. Verify the heading outline with a browser extension.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div class=\"big-title\">Überschrift</div>"),
            example_good: Some("<h1>Hauptüberschrift</h1>\n<h2>Abschnitt</h2>\n<h3>Unterabschnitt</h3>"),
            example_decorative: None,
        },
    ),
    (
        "2.4.7",
        RuleExplanation {
            customer_title: "Fokus-Indikator nicht sichtbar",
            customer_title_en: "Focus indicator not visible",
            customer_description:
                "Beim Navigieren mit der Tastatur ist nicht erkennbar, welches Element \
                 gerade den Fokus hat. Der visuelle Fokus-Rahmen fehlt oder ist unsichtbar.",
            customer_description_en:
                "When navigating with the keyboard, it is not visible which element \
                 currently has focus. The visual focus outline is missing or invisible.",
            user_impact:
                "Tastaturnutzer verlieren die Orientierung — sie wissen nicht, wo sie sich \
                 auf der Seite befinden und welches Element sie gerade aktivieren würden.",
            user_impact_en:
                "Keyboard users lose orientation — they cannot tell where they are on \
                 the page or which element they would activate.",
            typical_cause:
                "CSS-Reset entfernt outline (outline: none / outline: 0). \
                 Kein eigener Fokus-Stil definiert. Fokus-Stil zu unauffällig.",
            typical_cause_en:
                "CSS reset removes the outline (outline: none / outline: 0). \
                 No custom focus style defined. Focus style too inconspicuous.",
            recommendation:
                "Einen gut sichtbaren Fokus-Indikator für alle interaktiven Elemente gestalten. \
                 Mindestens 2px Umrandung mit ausreichendem Kontrast.",
            recommendation_en:
                "Design a clearly visible focus indicator for all interactive elements. \
                 At least a 2px outline with sufficient contrast.",
            technical_note:
                ":focus-visible statt :focus für Tastatur-only Fokus. \
                 outline: 2px solid #005fcc; outline-offset: 2px; \
                 Niemals outline: none ohne Alternative.",
            technical_note_en:
                "Use :focus-visible instead of :focus for keyboard-only focus. \
                 outline: 2px solid #005fcc; outline-offset: 2px; \
                 Never use outline: none without an alternative.",
            responsible_role: Role::DesignUx,
            effort_estimate: Effort::Quick,
            example_bad: Some("*:focus { outline: none; }"),
            example_good: Some("*:focus-visible { outline: 2px solid #005fcc; outline-offset: 2px; }"),
            example_decorative: None,
        },
    ),
    (
        "2.4.10",
        RuleExplanation {
            customer_title: "Fehlende Abschnittsüberschriften",
            customer_title_en: "Missing section headings",
            customer_description:
                "Längere Inhalte sind nicht durch Abschnittsüberschriften gegliedert. \
                 Die Seite wirkt als ein zusammenhängender Block ohne erkennbare Struktur.",
            customer_description_en:
                "Long content is not broken up by section headings. \
                 The page reads as one continuous block with no recognizable structure.",
            user_impact:
                "Nutzer können Inhalte nicht gezielt ansteuern und müssen lange Textpassagen \
                 komplett durchlesen, um relevante Informationen zu finden.",
            user_impact_en:
                "Users cannot jump to specific content and have to read long passages in \
                 full to find relevant information.",
            typical_cause:
                "Lange Inhaltsseiten ohne Zwischenüberschriften. Redaktionelle Inhalte \
                 als Fließtext ohne Gliederung.",
            typical_cause_en:
                "Long content pages without sub-headings. Editorial content presented as \
                 running text with no structure.",
            recommendation:
                "Längere Inhalte mit aussagekräftigen Zwischenüberschriften gliedern. \
                 Alle 2-3 Absätze eine Überschrift einsetzen.",
            recommendation_en:
                "Break up long content with meaningful sub-headings. Add a heading every \
                 2–3 paragraphs.",
            technical_note:
                "Korrekte Heading-Hierarchie (H2, H3...) innerhalb von Abschnitten. \
                 WAI-ARIA: role=\"heading\" nur als Fallback, native HTML bevorzugen.",
            technical_note_en:
                "Use correct heading hierarchy (H2, H3...) within sections. \
                 WAI-ARIA: use role=\"heading\" only as a fallback; prefer native HTML.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "2.5.3",
        RuleExplanation {
            customer_title: "Sichtbarer Text stimmt nicht mit dem zugänglichen Namen überein",
            customer_title_en: "Visible text does not match the accessible name",
            customer_description:
                "Der sichtbare Text eines Bedienelements stimmt nicht mit dem \
                 Namen überein, den assistive Technologien vorlesen. \
                 Sprachsteuerungsnutzer können das Element nicht ansprechen.",
            customer_description_en:
                "The visible text of a control does not match the name announced by \
                 assistive technologies. Voice-control users cannot address the element.",
            user_impact:
                "Nutzer, die Sprachsteuerung verwenden, können Elemente nicht per \
                 Sprachbefehl aktivieren, weil der vorgelesene Name nicht dem \
                 sichtbaren Text entspricht.",
            user_impact_en:
                "Users relying on voice control cannot activate elements by voice \
                 command because the announced name does not match the visible text.",
            typical_cause:
                "aria-label überschreibt sichtbaren Text mit abweichendem Wortlaut. \
                 Sichtbarer Text und zugänglicher Name sind unterschiedlich formuliert.",
            typical_cause_en:
                "aria-label overrides the visible text with different wording. \
                 Visible text and accessible name are phrased differently.",
            recommendation:
                "Sicherstellen, dass der zugängliche Name (aria-label) den sichtbaren \
                 Text enthält oder identisch ist.",
            recommendation_en:
                "Make sure the accessible name (aria-label) contains the visible text \
                 or is identical to it.",
            technical_note:
                "aria-label muss den sichtbaren Text als Teilstring enthalten. \
                 Besser: aria-label ganz weglassen, wenn der sichtbare Text ausreicht.",
            technical_note_en:
                "aria-label must include the visible text as a substring. \
                 Better: omit aria-label entirely when the visible text is sufficient.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button aria-label=\"Menü schließen\">X</button>"),
            example_good: Some("<button aria-label=\"Schließen\">Schließen</button>"),
            example_decorative: None,
        },
    ),
    // ── 3. Understandable ───────────────────────────────────────────────────
    (
        "3.1.1",
        RuleExplanation {
            customer_title: "Fehlende Sprachangabe der Seite",
            customer_title_en: "Missing page language declaration",
            customer_description:
                "Die Hauptsprache der Seite ist nicht im HTML-Code angegeben. \
                 Screenreader und Übersetzungstools können die Sprache nicht \
                 automatisch erkennen.",
            customer_description_en:
                "The primary language of the page is not declared in the HTML. \
                 Screen readers and translation tools cannot detect the language \
                 automatically.",
            user_impact:
                "Screenreader lesen den Text mit falscher Aussprache vor — \
                 deutsche Inhalte werden z. B. mit englischer Phonetik gelesen, \
                 was unverständlich ist.",
            user_impact_en:
                "Screen readers read out the text with the wrong pronunciation — \
                 e.g. German content is read with English phonetics, which is \
                 unintelligible.",
            typical_cause:
                "Fehlendes lang-Attribut im <html>-Tag. Häufig bei Templates vergessen.",
            typical_cause_en:
                "Missing lang attribute on the <html> tag. Often forgotten in templates.",
            recommendation:
                "Das lang-Attribut im <html>-Tag setzen (z. B. lang=\"de\" für Deutsch). \
                 Bei mehrsprachigen Seiten zusätzlich Abschnitte mit lang-Attribut \
                 kennzeichnen.",
            recommendation_en:
                "Set the lang attribute on the <html> tag (e.g. lang=\"en\" for English). \
                 On multilingual pages, additionally mark sections with a lang attribute.",
            technical_note:
                "<html lang=\"de\"> für deutschsprachige Seiten. \
                 ISO 639-1 Sprachcodes verwenden. \
                 Für fremdsprachige Abschnitte: <span lang=\"en\">...</span>.",
            technical_note_en:
                "<html lang=\"en\"> for English-language pages. \
                 Use ISO 639-1 language codes. \
                 For foreign-language sections: <span lang=\"en\">...</span>.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<html>"),
            example_good: Some("<html lang=\"de\">"),
            example_decorative: None,
        },
    ),
    (
        "3.2.1",
        RuleExplanation {
            customer_title: "Unerwartete Kontextänderung bei Fokus",
            customer_title_en: "Unexpected change of context on focus",
            customer_description:
                "Wenn ein Element den Tastaturfokus erhält, ändert sich unerwartet \
                 der Kontext (z. B. Seite wird gewechselt, neues Fenster öffnet sich).",
            customer_description_en:
                "When an element receives keyboard focus, the context changes \
                 unexpectedly (e.g. the page changes, a new window opens).",
            user_impact:
                "Tastaturnutzer und Screenreader-Nutzer erleben unvorhersehbare \
                 Seitenveränderungen. Das ist verwirrend und stört den Arbeitsfluss.",
            user_impact_en:
                "Keyboard and screen reader users experience unpredictable page changes. \
                 This is confusing and disrupts their workflow.",
            typical_cause:
                "JavaScript-Events auf onfocus, die Navigation oder DOM-Änderungen auslösen. \
                 Select-Elemente, die bei Fokus bereits eine Aktion auslösen.",
            typical_cause_en:
                "JavaScript events on onfocus that trigger navigation or DOM changes. \
                 Select elements that perform an action on focus.",
            recommendation:
                "Kontextänderungen nur durch explizite Benutzeraktionen auslösen \
                 (Klick, Enter), nicht durch bloßen Fokuswechsel.",
            recommendation_en:
                "Trigger context changes only via explicit user actions (click, Enter), \
                 not by focus change alone.",
            technical_note:
                "Keine onchange/onfocus-Handler für Navigation. \
                 Select-Menüs mit Submit-Button statt auto-submit.",
            technical_note_en:
                "No onchange/onfocus handlers for navigation. \
                 Use a submit button with select menus instead of auto-submit.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "3.2.2",
        RuleExplanation {
            customer_title: "Unerwartete Kontextänderung bei Eingabe",
            customer_title_en: "Unexpected change of context on input",
            customer_description:
                "Wenn ein Nutzer Eingaben in ein Formularelement macht oder eine \
                 Auswahl trifft, ändert sich unerwartet der Kontext der Seite.",
            customer_description_en:
                "When a user enters data into a form element or makes a selection, \
                 the page context changes unexpectedly.",
            user_impact:
                "Nutzer erleben verwirrende Änderungen beim Ausfüllen von Formularen. \
                 Besonders für Screenreader-Nutzer sind unangekündigte Änderungen problematisch.",
            user_impact_en:
                "Users experience confusing changes while filling out forms. \
                 Unannounced changes are especially problematic for screen reader users.",
            typical_cause:
                "Auto-Submit bei Formularänderungen. Seitenweiterleitung bei \
                 Select-Auswahl ohne Bestätigung.",
            typical_cause_en:
                "Auto-submit on form changes. Page redirect on select choice without \
                 confirmation.",
            recommendation:
                "Formularänderungen erst nach expliziter Bestätigung (Submit-Button) \
                 verarbeiten. Vorab ankündigen, wenn eine Eingabe sofortige Änderungen auslöst.",
            recommendation_en:
                "Process form changes only after an explicit confirmation (submit button). \
                 Announce up front when an input triggers immediate changes.",
            technical_note:
                "Kein auto-submit bei onchange. Explizite Submit-Buttons verwenden. \
                 Alternativ: ARIA-Live-Region für Vorab-Hinweis.",
            technical_note_en:
                "No auto-submit on onchange. Use explicit submit buttons. \
                 Alternatively: an ARIA live region for an advance notice.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "3.3.2",
        RuleExplanation {
            customer_title: "Fehlende Beschriftungen und Anleitungen bei Formularen",
            customer_title_en: "Missing labels and instructions on forms",
            customer_description:
                "Formularfelder haben keine sichtbare Beschriftung oder Anleitung. \
                 Nutzer wissen nicht, welche Eingaben erwartet werden.",
            customer_description_en:
                "Form fields have no visible label or instruction. \
                 Users do not know what input is expected.",
            user_impact:
                "Alle Nutzer, besonders Menschen mit kognitiven Einschränkungen und \
                 Screenreader-Nutzer, können Formulare nicht korrekt ausfüllen.",
            user_impact_en:
                "All users, especially people with cognitive impairments and screen \
                 reader users, cannot complete forms correctly.",
            typical_cause:
                "Fehlende <label>-Elemente, nur Platzhaltertext statt sichtbarer Labels, \
                 fehlende Hinweise auf Pflichtfelder oder Eingabeformat.",
            typical_cause_en:
                "Missing <label> elements, placeholder text used instead of visible labels, \
                 no indication of required fields or expected input format.",
            recommendation:
                "Jedes Formularfeld mit einem sichtbaren, per <label> verknüpften \
                 Label versehen. Pflichtfelder kennzeichnen. Bei speziellen Formaten \
                 (z. B. Datum) das erwartete Format angeben.",
            recommendation_en:
                "Provide every form field with a visible label linked via <label>. \
                 Mark required fields. For special formats (e.g. date), state the \
                 expected format.",
            technical_note:
                "<label for=\"id\"> für alle Formularfelder. \
                 Pflichtfelder: required + aria-required=\"true\". \
                 Platzhalter nicht als einzige Beschriftung verwenden.",
            technical_note_en:
                "<label for=\"id\"> on all form fields. \
                 Required fields: required + aria-required=\"true\". \
                 Do not use placeholders as the only label.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<input type=\"text\" placeholder=\"Name\">"),
            example_good: Some("<label for=\"name\">Name *</label>\n<input type=\"text\" id=\"name\" required aria-required=\"true\">"),
            example_decorative: None,
        },
    ),
    // ── 4. Robust ───────────────────────────────────────────────────────────
    (
        "4.1.2",
        RuleExplanation {
            customer_title: "Fehlende Name/Rolle bei Bedienelementen",
            customer_title_en: "Missing name/role on controls",
            customer_description:
                "Interaktive Elemente (Buttons, Links, Formularfelder) haben keinen \
                 zugänglichen Namen oder keine erkennbare Rolle. Assistive Technologien \
                 können nicht vermitteln, worum es sich handelt.",
            customer_description_en:
                "Interactive elements (buttons, links, form fields) have no accessible \
                 name or recognizable role. Assistive technologies cannot convey what \
                 the element is.",
            user_impact:
                "Screenreader-Nutzer hören z. B. nur 'Button' ohne Beschreibung der Funktion, \
                 oder ein klickbares Element wird gar nicht als interaktiv erkannt.",
            user_impact_en:
                "Screen reader users hear, for example, only 'button' with no description \
                 of its function, or a clickable element is not recognized as interactive \
                 at all.",
            typical_cause:
                "Buttons oder Links ohne Text oder aria-label. Icon-only-Buttons ohne \
                 zugänglichen Namen. Custom-Widgets ohne ARIA-Rollen.",
            typical_cause_en:
                "Buttons or links with no text or aria-label. Icon-only buttons without \
                 an accessible name. Custom widgets without ARIA roles.",
            recommendation:
                "Alle interaktiven Elemente mit einem verständlichen, zugänglichen Namen \
                 versehen. Native HTML-Elemente bevorzugen. Bei Icon-Buttons: \
                 aria-label verwenden.",
            recommendation_en:
                "Give every interactive element a clear, accessible name. Prefer native \
                 HTML elements. For icon buttons: use aria-label.",
            technical_note:
                "Buttons: sichtbarer Text oder aria-label. \
                 Links: sprechender Linktext. \
                 Inputs: verknüpftes <label>. \
                 Custom-Widgets: role + aria-label + aria-Zustände.",
            technical_note_en:
                "Buttons: visible text or aria-label. \
                 Links: descriptive link text. \
                 Inputs: associated <label>. \
                 Custom widgets: role + aria-label + ARIA states.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button><svg>...</svg></button>"),
            example_good: Some("<button aria-label=\"Menü öffnen\"><svg aria-hidden=\"true\">...</svg></button>"),
            example_decorative: None,
        },
    ),
    // ── SEO rules (no WCAG criterion) ────────────────────────────────────────
    (
        "seo.headings.long_heading",
        RuleExplanation {
            customer_title: "Überschrift zu lang",
            customer_title_en: "Heading too long",
            customer_description:
                "Überschriften auf der Seite überschreiten die empfohlene Länge von \
                 70 Zeichen. Lange Überschriften werden in Suchergebnissen abgeschnitten \
                 und sind für Leser schwerer zu erfassen.",
            customer_description_en:
                "Headings on the page exceed the recommended length of 70 characters. \
                 Long headings are truncated in search results and harder for readers to scan.",
            user_impact:
                "In Suchergebnissen und sozialen Medien werden die Überschriften \
                 abgeschnitten, was den Klickanreiz verringert. Leser müssen mehr \
                 kognitive Arbeit leisten, um den Inhalt einzuordnen.",
            user_impact_en:
                "In search results and social media the headings are cut off, reducing \
                 click-through appeal. Readers need more cognitive effort to understand \
                 the content at a glance.",
            typical_cause:
                "Redaktionelle Texte werden ungekürzt als Überschrift eingesetzt. \
                 CMS-Felder für Überschriften haben keine Längenbeschränkung.",
            typical_cause_en:
                "Editorial text is used as a heading without trimming. \
                 CMS heading fields have no character limit enforced.",
            recommendation:
                "Überschriften auf unter 70 Zeichen kürzen und den Kern der Aussage \
                 voranstellen. Längere Beschreibungen gehören in den Fließtext.",
            recommendation_en:
                "Shorten headings to under 70 characters and lead with the core message. \
                 Longer descriptions belong in the body text.",
            technical_note:
                "Google zeigt typischerweise 50–60 Zeichen im Title-Tag an. \
                 Für H2/H3 gilt keine feste Grenze, aber über 70 Zeichen sinkt \
                 die Lesbarkeit deutlich. CMS-Validierung empfohlen.",
            technical_note_en:
                "Google typically displays 50–60 characters in the title tag. \
                 There is no hard limit for H2/H3, but readability drops noticeably \
                 beyond 70 characters. CMS-level validation is recommended.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
];
