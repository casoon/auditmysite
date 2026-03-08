//! Customer-facing explanations for WCAG rules
//!
//! Maps technical WCAG rule IDs to human-readable explanations in German,
//! suitable for non-technical stakeholders.

use crate::output::report_model::{Effort, ExampleBlock, Role};

/// Complete explanation for a WCAG rule
pub struct RuleExplanation {
    /// Customer-facing title in German
    pub customer_title: &'static str,
    /// Layperson description of the issue
    pub customer_description: &'static str,
    /// Who is affected and how
    pub user_impact: &'static str,
    /// Why this typically happens
    pub typical_cause: &'static str,
    /// Recommendation in customer language
    pub recommendation: &'static str,
    /// Technical note for developers
    pub technical_note: &'static str,
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
}

/// Look up the explanation for a rule by its WCAG ID (e.g., "1.1.1")
/// or taxonomy rule ID (e.g., "a11y.alt_text.missing")
pub fn get_explanation(rule_id: &str) -> Option<&'static RuleExplanation> {
    // Direct lookup by WCAG ID
    if let Some(expl) = EXPLANATIONS.iter().find(|(id, _)| *id == rule_id).map(|(_, e)| e) {
        return Some(expl);
    }
    // Fallback: if a taxonomy rule_id was passed, resolve to WCAG ID via legacy map
    if rule_id.contains('.') {
        use crate::taxonomy::rules::RULES;
        if let Some(rule) = RULES.iter().find(|r| r.id == rule_id) {
            if let Some(ext_ref) = rule.external_ref {
                // external_ref is "WCAG 1.1.1" — extract the number
                let wcag_id = ext_ref.strip_prefix("WCAG ").unwrap_or(ext_ref);
                return EXPLANATIONS.iter().find(|(id, _)| *id == wcag_id).map(|(_, e)| e);
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
            customer_description:
                "Bilder auf der Website haben keinen beschreibenden Alternativtext. \
                 Dadurch können Screenreader den Bildinhalt nicht an blinde oder \
                 sehbeeinträchtigte Nutzer vermitteln.",
            user_impact:
                "Menschen mit Sehbeeinträchtigung erhalten an diesen Stellen keine oder \
                 nur unvollständige Information. Der Bildinhalt geht für sie vollständig verloren.",
            typical_cause:
                "Teaserbilder, Slider, redaktionell eingepflegte Medien ohne Pflichtfeld \
                 im CMS, oder dekorative Bilder, die nicht als solche markiert sind.",
            recommendation:
                "Für informative Bilder einen beschreibenden Alt-Text hinterlegen, der den \
                 Bildinhalt oder -zweck vermittelt. Rein dekorative Bilder mit einem leeren \
                 Alt-Attribut markieren (alt=\"\").",
            technical_note:
                "Informative Bilder: <img alt=\"Beschreibung\">. \
                 Dekorative Bilder: <img alt=\"\"> oder role=\"presentation\". \
                 CMS-Felder für Alt-Texte als Pflichtfeld konfigurieren.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: Some("<img src=\"hero.jpg\">"),
            example_good: Some("<img src=\"hero.jpg\" alt=\"Team von Casoon im Workshop\">"),
            example_decorative: Some("<img src=\"ornament.svg\" alt=\"\">"),
        },
    ),
    (
        "1.3.1",
        RuleExplanation {
            customer_title: "Fehlende semantische Struktur",
            customer_description:
                "Inhalte sind visuell strukturiert (z. B. durch Größe oder Farbe), aber \
                 die Struktur ist nicht im HTML-Code hinterlegt. Screenreader und andere \
                 Hilfstechnologien können die Beziehungen zwischen Inhalten nicht erkennen.",
            user_impact:
                "Nutzer mit Screenreader können Tabellen, Listen und Formulargruppen nicht \
                 korrekt navigieren. Die logische Struktur der Seite geht verloren.",
            typical_cause:
                "Tabellen ohne korrekte Tabellenauszeichnung, fehlende Fieldsets bei \
                 Formularen, Listen als div-Elemente statt ul/ol, fehlende Landmarks.",
            recommendation:
                "Inhalte semantisch korrekt auszeichnen: Tabellen mit <table>, <th>, <td>; \
                 Listen mit <ul>/<ol>; Formulare mit <fieldset> und <legend> gruppieren.",
            technical_note:
                "HTML5-Semantik nutzen: <nav>, <main>, <aside>, <header>, <footer>. \
                 Tabellen: scope-Attribute für Kopfzellen. ARIA-Rollen nur als Ergänzung.",
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
            customer_description:
                "Formularfelder haben keine maschinenlesbare Kennzeichnung ihres Zwecks. \
                 Browser und Hilfstechnologien können dadurch keine Autofill-Vorschläge \
                 machen und Nutzern nicht helfen, Formulare schneller auszufüllen.",
            user_impact:
                "Menschen mit motorischen oder kognitiven Einschränkungen können nicht von \
                 automatischer Formularausfüllung profitieren. Das Ausfüllen dauert länger \
                 und ist fehleranfälliger.",
            typical_cause:
                "Fehlende autocomplete-Attribute in Formularen für persönliche Daten \
                 (Name, E-Mail, Adresse, Telefon).",
            recommendation:
                "Alle Formularfelder für persönliche Daten mit dem passenden \
                 autocomplete-Attribut versehen (z. B. autocomplete=\"email\", \
                 autocomplete=\"given-name\").",
            technical_note:
                "autocomplete-Werte gemäß HTML-Spezifikation verwenden: \
                 name, email, tel, street-address, postal-code, country, etc.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<input type=\"email\" name=\"email\">"),
            example_good: Some("<input type=\"email\" name=\"email\" autocomplete=\"email\">"),
            example_decorative: None,
        },
    ),
    (
        "1.4.3",
        RuleExplanation {
            customer_title: "Unzureichender Farbkontrast",
            customer_description:
                "Text auf der Website hat nicht genügend Kontrast zum Hintergrund. \
                 Bei ungünstigen Lichtverhältnissen oder für Menschen mit \
                 Sehschwäche ist der Text schwer lesbar.",
            user_impact:
                "Menschen mit Sehbeeinträchtigung, ältere Nutzer und alle Nutzer bei \
                 ungünstigen Bildschirmbedingungen (Sonnenlicht, schlechte Displays) \
                 können Texte schlecht oder gar nicht lesen.",
            typical_cause:
                "Helle Schriftfarbe auf hellem Hintergrund, graue Texte auf weißem Grund, \
                 Designentscheidungen ohne Kontrastprüfung.",
            recommendation:
                "Schriftfarben anpassen, sodass normaler Text mindestens ein Kontrastverhältnis \
                 von 4,5:1 und großer Text von 3:1 zum Hintergrund hat. \
                 Kontrastwerte mit Tools wie dem WebAIM Contrast Checker prüfen.",
            technical_note:
                "WCAG 2.1 Level AA: 4.5:1 für normalen Text (<18pt / <14pt bold), \
                 3:1 für großen Text (≥18pt / ≥14pt bold). \
                 CSS-Variablen für konsistente Farbpalette verwenden.",
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
            customer_description:
                "Texte auf der Website können nicht ohne Verlust von Inhalt oder Funktion \
                 auf 200% vergrößert werden. Nutzer, die auf größere Schrift angewiesen sind, \
                 verlieren dadurch Inhalte.",
            user_impact:
                "Menschen mit Sehbeeinträchtigung, die die Schriftgröße im Browser vergrößern, \
                 stoßen auf abgeschnittene Texte, überlagerte Elemente oder nicht \
                 scrollbare Bereiche.",
            typical_cause:
                "Feste Pixelwerte für Schriftgrößen und Container, overflow: hidden \
                 bei Textcontainern, fehlende responsive Anpassungen.",
            recommendation:
                "Schriftgrößen in relativen Einheiten (rem/em) angeben. Container so gestalten, \
                 dass sie bei vergrößertem Text mitwachsen. Browser-Zoom auf 200% testen.",
            technical_note:
                "font-size in rem/em statt px. Container mit min-height statt height. \
                 overflow: auto statt hidden. @media-Queries für verschiedene Zoom-Stufen.",
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
            customer_description:
                "Bedienelemente wie Buttons, Eingabefelder oder Icons haben nicht genügend \
                 Kontrast zum Hintergrund. Sie sind dadurch für manche Nutzer schwer erkennbar.",
            user_impact:
                "Nutzer mit eingeschränktem Sehvermögen können interaktive Elemente nicht \
                 zuverlässig erkennen und bedienen.",
            typical_cause:
                "Subtile Rahmenfarben bei Formularelementen, Icons mit geringem Kontrast, \
                 Fokus-Indikatoren, die zu unauffällig sind.",
            recommendation:
                "Alle interaktiven Elemente und ihre Zustände (Normal, Hover, Fokus) mit \
                 mindestens 3:1 Kontrast zum Hintergrund gestalten.",
            technical_note:
                "3:1 Kontrastverhältnis für UI-Komponenten und grafische Objekte. \
                 Gilt für Rahmen, Icons, Slider, Checkboxen, etc.",
            responsible_role: Role::DesignUx,
            effort_estimate: Effort::Medium,
            example_bad: Some("border: 1px solid #cccccc; /* auf #ffffff = 1.6:1 */"),
            example_good: Some("border: 1px solid #767676; /* auf #ffffff = 4.5:1 */"),
            example_decorative: None,
        },
    ),
    // ── 2. Operable ─────────────────────────────────────────────────────────
    (
        "2.1.1",
        RuleExplanation {
            customer_title: "Inhalte nicht per Tastatur bedienbar",
            customer_description:
                "Bestimmte Funktionen der Website können nur mit der Maus bedient werden. \
                 Nutzer, die auf die Tastatur angewiesen sind, können diese Inhalte \
                 nicht erreichen oder nutzen.",
            user_impact:
                "Menschen mit motorischen Einschränkungen, die keine Maus verwenden können, \
                 sowie Screenreader-Nutzer sind von diesen Funktionen ausgeschlossen.",
            typical_cause:
                "Click-Handler nur auf div/span statt auf interaktive Elemente, \
                 fehlende tabindex-Attribute, JavaScript-Widgets ohne Tastaturunterstützung.",
            recommendation:
                "Alle interaktiven Elemente mit nativen HTML-Elementen (button, a, input) \
                 umsetzen. Bei Custom-Widgets Tastaturnavigation (Tab, Enter, Escape, Pfeiltasten) \
                 implementieren.",
            technical_note:
                "Native HTML-Elemente bevorzugen. Bei Custom-Widgets: tabindex=\"0\", \
                 keydown/keyup-Handler, ARIA-Rollen und -Zustände.",
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
            customer_description:
                "An bestimmten Stellen der Website bleibt der Tastaturfokus hängen. \
                 Nutzer können mit der Tastatur nicht mehr vor- oder zurücknavigieren.",
            user_impact:
                "Tastaturnutzer sind an dieser Stelle gefangen und können den Rest \
                 der Seite nicht mehr erreichen — die Seite wird faktisch unbenutzbar.",
            typical_cause:
                "Modale Dialoge ohne korrekte Fokus-Verwaltung, Widgets, die den Fokus \
                 abfangen, fehlende Escape-Taste zum Verlassen.",
            recommendation:
                "Sicherstellen, dass der Fokus jedes Element mit Tab und Shift+Tab \
                 verlassen kann. Modale Dialoge mit Escape schließbar machen und den \
                 Fokus danach korrekt zurücksetzen.",
            technical_note:
                "Focus trapping nur in Modals mit korrekter Implementierung. \
                 Escape zum Schließen. Fokus-Rückgabe an das auslösende Element.",
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
            customer_description:
                "Die Website bietet keinen Mechanismus, um wiederkehrende Inhaltsblöcke \
                 (z. B. Navigation, Header) zu überspringen und direkt zum Hauptinhalt \
                 zu gelangen.",
            user_impact:
                "Tastaturnutzer und Screenreader-Nutzer müssen bei jedem Seitenwechsel \
                 erneut durch die gesamte Navigation tabben, bevor sie den Inhalt erreichen.",
            typical_cause:
                "Fehlender Skip-Link, fehlende HTML5-Landmarks (nav, main, aside), \
                 Navigation nicht als Landmark ausgezeichnet.",
            recommendation:
                "Einen sichtbaren Skip-Link ('Zum Inhalt springen') als erstes Element \
                 der Seite einbauen. Zusätzlich HTML5-Landmarks korrekt verwenden: \
                 <nav>, <main>, <aside>, <header>, <footer>.",
            technical_note:
                "Skip-Link: <a href=\"#main\" class=\"skip-link\">Zum Inhalt springen</a>. \
                 Landmarks: <nav role=\"navigation\">, <main id=\"main\">, etc. \
                 Skip-Link bei Fokus sichtbar machen via CSS :focus.",
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
            customer_description:
                "Die Seite hat keinen aussagekräftigen Titel im Browser-Tab. \
                 Nutzer können nicht erkennen, auf welcher Seite sie sich befinden.",
            user_impact:
                "Screenreader-Nutzer hören den Seitentitel als Erstes — ohne klaren Titel \
                 fehlt die Orientierung. Auch bei vielen offenen Tabs ist die Seite \
                 nicht zuzuordnen.",
            typical_cause:
                "Leeres <title>-Tag, generischer Titel ('Home', 'Untitled'), \
                 identischer Titel auf allen Seiten.",
            recommendation:
                "Jeder Seite einen eindeutigen, beschreibenden Titel geben, der den \
                 Seiteninhalt und die Website-Zugehörigkeit klar macht \
                 (z. B. 'Kontakt — Firmenname').",
            technical_note:
                "<title>Seiteninhalt — Website</title>. Titel im CMS als Pflichtfeld. \
                 Pattern: Seitenspezifisch + Seitenname.",
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
            customer_description:
                "Die Reihenfolge, in der interaktive Elemente per Tastatur erreicht werden, \
                 entspricht nicht der visuellen oder logischen Reihenfolge der Seite.",
            user_impact:
                "Tastaturnutzer erleben eine verwirrende Navigation — der Fokus springt \
                 zwischen unzusammenhängenden Bereichen hin und her.",
            typical_cause:
                "Visuelles Layout per CSS umgeordnet ohne DOM-Reihenfolge anzupassen, \
                 positive tabindex-Werte, dynamisch eingefügte Elemente an falscher Stelle.",
            recommendation:
                "Die DOM-Reihenfolge an die visuelle Reihenfolge anpassen. \
                 Auf positive tabindex-Werte verzichten. Bei dynamischen Inhalten \
                 den Fokus programmatisch steuern.",
            technical_note:
                "tabindex nur 0 oder -1 verwenden. CSS-Order nicht für inhaltlich \
                 relevante Umordnungen nutzen. DOM-Reihenfolge = Lesereihenfolge.",
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
            customer_description:
                "Links auf der Seite haben Texte wie 'hier', 'mehr', 'weiterlesen', \
                 die ohne Kontext nicht verständlich sind. Nutzer können nicht erkennen, \
                 wohin ein Link führt.",
            user_impact:
                "Screenreader-Nutzer navigieren häufig über eine Linkliste — generische \
                 Texte wie 'mehr' sind dort völlig nichtssagend. Auch für alle anderen \
                 Nutzer ist die Orientierung erschwert.",
            typical_cause:
                "Redaktionelle Gewohnheit, 'hier klicken' oder 'mehr erfahren' als Linktext \
                 zu verwenden. Teaser-Komponenten mit generischem 'Weiterlesen'.",
            recommendation:
                "Linktexte so formulieren, dass sie auch ohne umgebenden Text verständlich \
                 sind (z. B. 'Leistungen im Bereich Webentwicklung' statt 'mehr erfahren').",
            technical_note:
                "Sprechende Linktexte verwenden. Falls visuell knapper Text gewünscht: \
                 aria-label oder aria-labelledby für erweiterte Beschreibung.",
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
            customer_description:
                "Die Seite verwendet keine oder unlogische Überschriften. \
                 Die inhaltliche Gliederung ist für assistive Technologien nicht erkennbar.",
            user_impact:
                "Screenreader-Nutzer können nicht per Überschrift durch die Seite navigieren. \
                 Die Seite wird zu einem undifferenzierten Textblock — schnelles Finden \
                 relevanter Abschnitte ist unmöglich.",
            typical_cause:
                "Keine Überschriften verwendet, nur visuelle Formatierung (fett, groß). \
                 Überschriftenhierarchie übersprungen (z. B. H1 → H3). \
                 Mehrere oder fehlende H1.",
            recommendation:
                "Eine logische Überschriftenhierarchie aufbauen: genau eine H1 pro Seite, \
                 darunter H2 für Hauptabschnitte, H3 für Unterabschnitte. \
                 Keine Ebenen überspringen.",
            technical_note:
                "H1 = Seitentitel (einmal). H2-H6 hierarchisch verschachtelt. \
                 Keine Ebenen überspringen. Heading-Outline mit Browser-Extension prüfen.",
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
            customer_description:
                "Beim Navigieren mit der Tastatur ist nicht erkennbar, welches Element \
                 gerade den Fokus hat. Der visuelle Fokus-Rahmen fehlt oder ist unsichtbar.",
            user_impact:
                "Tastaturnutzer verlieren die Orientierung — sie wissen nicht, wo sie sich \
                 auf der Seite befinden und welches Element sie gerade aktivieren würden.",
            typical_cause:
                "CSS-Reset entfernt outline (outline: none / outline: 0). \
                 Kein eigener Fokus-Stil definiert. Fokus-Stil zu unauffällig.",
            recommendation:
                "Einen gut sichtbaren Fokus-Indikator für alle interaktiven Elemente gestalten. \
                 Mindestens 2px Umrandung mit ausreichendem Kontrast.",
            technical_note:
                ":focus-visible statt :focus für Tastatur-only Fokus. \
                 outline: 2px solid #005fcc; outline-offset: 2px; \
                 Niemals outline: none ohne Alternative.",
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
            customer_description:
                "Längere Inhalte sind nicht durch Abschnittsüberschriften gegliedert. \
                 Die Seite wirkt als ein zusammenhängender Block ohne erkennbare Struktur.",
            user_impact:
                "Nutzer können Inhalte nicht gezielt ansteuern und müssen lange Textpassagen \
                 komplett durchlesen, um relevante Informationen zu finden.",
            typical_cause:
                "Lange Inhaltsseiten ohne Zwischenüberschriften. Redaktionelle Inhalte \
                 als Fließtext ohne Gliederung.",
            recommendation:
                "Längere Inhalte mit aussagekräftigen Zwischenüberschriften gliedern. \
                 Alle 2-3 Absätze eine Überschrift einsetzen.",
            technical_note:
                "Korrekte Heading-Hierarchie (H2, H3...) innerhalb von Abschnitten. \
                 WAI-ARIA: role=\"heading\" nur als Fallback, native HTML bevorzugen.",
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
            customer_description:
                "Der sichtbare Text eines Bedienelements stimmt nicht mit dem \
                 Namen überein, den assistive Technologien vorlesen. \
                 Sprachsteuerungsnutzer können das Element nicht ansprechen.",
            user_impact:
                "Nutzer, die Sprachsteuerung verwenden, können Elemente nicht per \
                 Sprachbefehl aktivieren, weil der vorgelesene Name nicht dem \
                 sichtbaren Text entspricht.",
            typical_cause:
                "aria-label überschreibt sichtbaren Text mit abweichendem Wortlaut. \
                 Sichtbarer Text und zugänglicher Name sind unterschiedlich formuliert.",
            recommendation:
                "Sicherstellen, dass der zugängliche Name (aria-label) den sichtbaren \
                 Text enthält oder identisch ist.",
            technical_note:
                "aria-label muss den sichtbaren Text als Teilstring enthalten. \
                 Besser: aria-label ganz weglassen, wenn der sichtbare Text ausreicht.",
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
            customer_description:
                "Die Hauptsprache der Seite ist nicht im HTML-Code angegeben. \
                 Screenreader und Übersetzungstools können die Sprache nicht \
                 automatisch erkennen.",
            user_impact:
                "Screenreader lesen den Text mit falscher Aussprache vor — \
                 deutsche Inhalte werden z. B. mit englischer Phonetik gelesen, \
                 was unverständlich ist.",
            typical_cause:
                "Fehlendes lang-Attribut im <html>-Tag. Häufig bei Templates vergessen.",
            recommendation:
                "Das lang-Attribut im <html>-Tag setzen (z. B. lang=\"de\" für Deutsch). \
                 Bei mehrsprachigen Seiten zusätzlich Abschnitte mit lang-Attribut \
                 kennzeichnen.",
            technical_note:
                "<html lang=\"de\"> für deutschsprachige Seiten. \
                 ISO 639-1 Sprachcodes verwenden. \
                 Für fremdsprachige Abschnitte: <span lang=\"en\">...</span>.",
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
            customer_description:
                "Wenn ein Element den Tastaturfokus erhält, ändert sich unerwartet \
                 der Kontext (z. B. Seite wird gewechselt, neues Fenster öffnet sich).",
            user_impact:
                "Tastaturnutzer und Screenreader-Nutzer erleben unvorhersehbare \
                 Seitenveränderungen. Das ist verwirrend und stört den Arbeitsfluss.",
            typical_cause:
                "JavaScript-Events auf onfocus, die Navigation oder DOM-Änderungen auslösen. \
                 Select-Elemente, die bei Fokus bereits eine Aktion auslösen.",
            recommendation:
                "Kontextänderungen nur durch explizite Benutzeraktionen auslösen \
                 (Klick, Enter), nicht durch bloßen Fokuswechsel.",
            technical_note:
                "Keine onchange/onfocus-Handler für Navigation. \
                 Select-Menüs mit Submit-Button statt auto-submit.",
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
            customer_description:
                "Wenn ein Nutzer Eingaben in ein Formularelement macht oder eine \
                 Auswahl trifft, ändert sich unerwartet der Kontext der Seite.",
            user_impact:
                "Nutzer erleben verwirrende Änderungen beim Ausfüllen von Formularen. \
                 Besonders für Screenreader-Nutzer sind unangekündigte Änderungen problematisch.",
            typical_cause:
                "Auto-Submit bei Formularänderungen. Seitenweiterleitung bei \
                 Select-Auswahl ohne Bestätigung.",
            recommendation:
                "Formularänderungen erst nach expliziter Bestätigung (Submit-Button) \
                 verarbeiten. Vorab ankündigen, wenn eine Eingabe sofortige Änderungen auslöst.",
            technical_note:
                "Kein auto-submit bei onchange. Explizite Submit-Buttons verwenden. \
                 Alternativ: ARIA-Live-Region für Vorab-Hinweis.",
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
            customer_description:
                "Formularfelder haben keine sichtbare Beschriftung oder Anleitung. \
                 Nutzer wissen nicht, welche Eingaben erwartet werden.",
            user_impact:
                "Alle Nutzer, besonders Menschen mit kognitiven Einschränkungen und \
                 Screenreader-Nutzer, können Formulare nicht korrekt ausfüllen.",
            typical_cause:
                "Fehlende <label>-Elemente, nur Platzhaltertext statt sichtbarer Labels, \
                 fehlende Hinweise auf Pflichtfelder oder Eingabeformat.",
            recommendation:
                "Jedes Formularfeld mit einem sichtbaren, per <label> verknüpften \
                 Label versehen. Pflichtfelder kennzeichnen. Bei speziellen Formaten \
                 (z. B. Datum) das erwartete Format angeben.",
            technical_note:
                "<label for=\"id\"> für alle Formularfelder. \
                 Pflichtfelder: required + aria-required=\"true\". \
                 Platzhalter nicht als einzige Beschriftung verwenden.",
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
            customer_description:
                "Interaktive Elemente (Buttons, Links, Formularfelder) haben keinen \
                 zugänglichen Namen oder keine erkennbare Rolle. Assistive Technologien \
                 können nicht vermitteln, worum es sich handelt.",
            user_impact:
                "Screenreader-Nutzer hören z. B. nur 'Button' ohne Beschreibung der Funktion, \
                 oder ein klickbares Element wird gar nicht als interaktiv erkannt.",
            typical_cause:
                "Buttons oder Links ohne Text oder aria-label. Icon-only-Buttons ohne \
                 zugänglichen Namen. Custom-Widgets ohne ARIA-Rollen.",
            recommendation:
                "Alle interaktiven Elemente mit einem verständlichen, zugänglichen Namen \
                 versehen. Native HTML-Elemente bevorzugen. Bei Icon-Buttons: \
                 aria-label verwenden.",
            technical_note:
                "Buttons: sichtbarer Text oder aria-label. \
                 Links: sprechender Linktext. \
                 Inputs: verknüpftes <label>. \
                 Custom-Widgets: role + aria-label + aria-Zustände.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button><svg>...</svg></button>"),
            example_good: Some("<button aria-label=\"Menü öffnen\"><svg aria-hidden=\"true\">...</svg></button>"),
            example_decorative: None,
        },
    ),
];
