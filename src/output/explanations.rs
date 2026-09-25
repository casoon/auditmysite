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

/// All rule explanations, keyed by WCAG ID or taxonomy rule ID — for review
/// surfaces that need to enumerate every customer text (e.g.
/// `export_all_interpretations`). Test-only: no production caller needs the
/// full list.
#[cfg(test)]
pub(crate) fn all() -> &'static [(&'static str, RuleExplanation)] {
    EXPLANATIONS
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

/// Resolve a finding's explanation in the only order that is safe: the
/// finding's own axe id first, then the taxonomy rule id, then the WCAG
/// criterion.
///
/// Several distinct checks share one taxonomy rule id or WCAG criterion — the
/// many "1.3.1" checks (landmarks, lists, tables, form groups) are the usual
/// case. `get_explanation` resolves a taxonomy id through `external_ref` down
/// to the bare WCAG number, so looking up by rule id or criterion *alone*
/// silently returns an explanation written for a different check (#571). The
/// rule-specific overrides that prevent this are keyed by axe id (e.g.
/// "landmark-unique"), and are only reachable when the axe id is tried first.
///
/// Every caller that turns a `NormalizedFinding` into customer-facing text
/// must go through here so the order cannot drift apart again (plan 32).
pub fn resolve_explanation(
    axe_id: Option<&str>,
    rule_id: &str,
    wcag_criterion: &str,
) -> Option<&'static RuleExplanation> {
    axe_id
        .and_then(get_explanation)
        .or_else(|| get_explanation(rule_id))
        .or_else(|| get_explanation(wcag_criterion))
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
        "1.2.1",
        RuleExplanation {
            customer_title: "Video/Audio ohne Alternative nicht automatisch geprüft",
            customer_title_en: "Video/audio without an alternative not automatically verified",
            customer_description:
                "Auf der Seite wurde ein reines Video- oder Audio-Element gefunden. Ob dafür eine \
                 gleichwertige Textalternative (z. B. ein Transkript) existiert, kann ein Scanner \
                 nicht zuverlässig aus dem HTML ableiten — das erfordert eine inhaltliche Prüfung.",
            customer_description_en:
                "A video-only or audio-only element was found on the page. Whether an equivalent \
                 text alternative (e.g. a transcript) exists cannot be reliably determined from \
                 the HTML alone — this requires a manual content check.",
            user_impact:
                "Menschen, die das Video nicht sehen oder das Audio nicht hören können, erhalten \
                 ohne Textalternative keinen Zugang zum vermittelten Inhalt.",
            user_impact_en:
                "People who cannot see the video or hear the audio have no access to the \
                 conveyed content without a text alternative.",
            typical_cause:
                "Eingebettete Video-/Audio-Player (nativ oder als Drittanbieter-Embed), für die \
                 keine Transkript-Seite oder kein Textäquivalent verlinkt ist.",
            typical_cause_en:
                "Embedded video/audio players (native or third-party) with no linked transcript \
                 page or text equivalent.",
            recommendation:
                "Für jedes reine Video-/Audio-Element manuell prüfen, ob ein Transkript oder eine \
                 gleichwertige Textalternative vorhanden und vollständig ist.",
            recommendation_en:
                "For every video-only or audio-only element, manually verify that a transcript or \
                 equivalent text alternative exists and is complete.",
            technical_note:
                "Ein Textlink zum Transkript in der Nähe des Medienelements platzieren, z. B. \
                 direkt unterhalb des Players.",
            technical_note_en:
                "Place a text link to the transcript near the media element, e.g. directly below \
                 the player.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "1.2.2",
        RuleExplanation {
            customer_title: "Untertitel bei Video nicht bestätigt",
            customer_title_en: "Captions on video not confirmed",
            customer_description:
                "Für ein Video auf der Seite konnte keine auflösbare Untertitel-/Caption-Datei \
                 (`<track kind=\"captions\">`) technisch bestätigt werden, oder das Video ist ein \
                 eingebetteter Drittanbieter-Player, dessen Untertitel-Einstellungen sich nicht \
                 von außen prüfen lassen.",
            customer_description_en:
                "For a video on the page, a resolving caption/subtitle file \
                 (`<track kind=\"captions\">`) could not be technically confirmed, or the video is \
                 a third-party embedded player whose caption settings cannot be checked from the \
                 outside.",
            user_impact:
                "Gehörlose und schwerhörige Nutzer erhalten ohne Untertitel keinen Zugang zum \
                 gesprochenen Inhalt des Videos.",
            user_impact_en:
                "Deaf and hard-of-hearing users have no access to the video's spoken content \
                 without captions.",
            typical_cause:
                "Video ohne `<track kind=\"captions\">`-Element, ein Track, dessen Datei nicht \
                 erreichbar ist, oder ein eingebetteter Player (YouTube, Vimeo u. ä.), dessen \
                 Untertitel-Status vom eigentlichen Betreiber der Seite konfiguriert wird.",
            typical_cause_en:
                "Video without a `<track kind=\"captions\">` element, a track whose file is \
                 unreachable, or an embedded player (YouTube, Vimeo, etc.) whose caption status is \
                 configured by the platform, not the audited page.",
            recommendation:
                "Sicherstellen, dass jedes vorab aufgezeichnete Video mit Ton synchronisierte \
                 Untertitel hat. Bei eingebetteten Playern die Untertitel-Einstellungen manuell im \
                 Player prüfen.",
            recommendation_en:
                "Ensure every prerecorded video with audio has synchronized captions. For embedded \
                 players, manually verify the caption settings within the player itself.",
            technical_note:
                "Natives `<video>`: `<track kind=\"captions\" src=\"...\" srclang=\"de\">` \
                 hinzufügen, Datei muss erreichbar sein. Bei Embeds die Caption-Funktion der \
                 Plattform aktivieren.",
            technical_note_en:
                "Native `<video>`: add `<track kind=\"captions\" src=\"...\" srclang=\"en\">`, the \
                 file must resolve. For embeds, enable the platform's caption feature.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<video src=\"clip.mp4\"></video>"),
            example_good: Some(
                "<video src=\"clip.mp4\"><track kind=\"captions\" src=\"clip.vtt\" srclang=\"de\"></video>",
            ),
            example_decorative: None,
        },
    ),
    (
        "1.2.3",
        RuleExplanation {
            customer_title: "Audiodeskription oder Textalternative nicht automatisch geprüft",
            customer_title_en: "Audio description or text alternative not automatically verified",
            customer_description:
                "Ob ein vorab aufgezeichnetes Video mit visuell vermittelten Informationen \
                 (z. B. Handlungen, Ortswechsel) eine Audiodeskription oder eine vollständige \
                 Textalternative besitzt, kann ein Scanner nicht aus dem HTML allein feststellen.",
            customer_description_en:
                "Whether a prerecorded video with visually conveyed information (e.g. actions, \
                 scene changes) has an audio description or a complete text alternative cannot be \
                 determined from the HTML alone.",
            user_impact:
                "Blinde und sehbeeinträchtigte Nutzer verpassen visuell vermittelte Informationen, \
                 die nicht im normalen Ton des Videos genannt werden.",
            user_impact_en:
                "Blind and visually impaired users miss visually conveyed information that is not \
                 mentioned in the video's regular audio track.",
            typical_cause:
                "Video mit wichtigen visuellen Inhalten (Diagramme, Handlungsabläufe, Textein-\
                 blendungen), ohne separate Audiodeskriptionsspur und ohne Textalternative.",
            typical_cause_en:
                "Video with important visual content (diagrams, actions, on-screen text) without a \
                 separate audio description track and without a text alternative.",
            recommendation:
                "Videos mit relevanten visuellen Inhalten manuell prüfen: Existiert eine \
                 Audiodeskription oder eine vollständige Textalternative, die diese Inhalte \
                 abdeckt?",
            recommendation_en:
                "Manually check videos with relevant visual content: does an audio description or \
                 a complete text alternative exist that covers this content?",
            technical_note:
                "Audiodeskription per `<track kind=\"descriptions\">` oder separater Tonspur \
                 einbinden, alternativ eine vollständige Textalternative verlinken.",
            technical_note_en:
                "Add an audio description via `<track kind=\"descriptions\">` or a separate audio \
                 track, or alternatively link a complete text alternative.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "1.2.8",
        RuleExplanation {
            customer_title: "Vollständige Textalternative für Video nicht bestätigt",
            customer_title_en: "Full text alternative for video not confirmed",
            customer_description:
                "Für vorab aufgezeichnete Video-/Audio-Inhalte konnte nicht automatisch bestätigt \
                 werden, dass eine vollständige Textalternative (Transkript inkl. relevanter \
                 visueller Informationen) existiert.",
            customer_description_en:
                "For prerecorded video/audio content, it could not be automatically confirmed that \
                 a complete text alternative (a transcript including relevant visual information) \
                 exists.",
            user_impact:
                "Taubblinde Nutzer und Nutzer, die weder Bild noch Ton wahrnehmen können, sind auf \
                 eine vollständige Textalternative angewiesen, um auf den Inhalt zuzugreifen.",
            user_impact_en:
                "Deafblind users and users who can perceive neither image nor sound depend on a \
                 complete text alternative to access the content.",
            typical_cause:
                "Video-/Audio-Element ohne verlinktes Transkript, oder ein in der Nähe gefundener \
                 Link deutet zwar auf ein Transkript hin, dessen Vollständigkeit ist aber nicht \
                 automatisch prüfbar.",
            typical_cause_en:
                "Video/audio element with no linked transcript, or a nearby link suggests a \
                 transcript exists but its completeness cannot be verified automatically.",
            recommendation:
                "Für jedes vorab aufgezeichnete Video-/Audio-Element ein vollständiges Transkript \
                 bereitstellen und manuell auf Vollständigkeit prüfen (inkl. gesprochenem Dialog \
                 und relevanten visuellen Informationen).",
            recommendation_en:
                "Provide a complete transcript for every prerecorded video/audio element and \
                 manually verify its completeness (including spoken dialogue and relevant visual \
                 information).",
            technical_note:
                "Transkript als Text auf der Seite oder verlinkt bereitstellen, `<track \
                 kind=\"descriptions\">` für Audiodeskription ergänzen, wo zutreffend.",
            technical_note_en:
                "Provide the transcript as text on the page or via a link, add `<track \
                 kind=\"descriptions\">` for audio description where applicable.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Medium,
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
    // Rule-specific override for `region.rs` findings (axe_id "region").
    // The WCAG 1.3.1 fallback above is written for tables/lists/forms and
    // doesn't fit this check — it flags content (often a link) that sits
    // outside every landmark region, not missing table/list/fieldset markup
    // (#571).
    (
        "region",
        RuleExplanation {
            customer_title: "Inhalt außerhalb einer Landmark-Region",
            customer_title_en: "Content outside a landmark region",
            customer_description:
                "Ein Element mit sichtbarem Inhalt — häufig ein Link — liegt außerhalb aller \
                 Landmark-Bereiche der Seite (main, nav, header, footer, aside). Screenreader-\
                 Nutzer, die per Landmark zwischen Seitenbereichen springen, finden dieses \
                 Element dabei nicht.",
            customer_description_en:
                "An element with visible content — often a link — sits outside every landmark \
                 region of the page (main, nav, header, footer, aside). Screen reader users who \
                 jump between page regions via landmark navigation will not encounter this \
                 element.",
            user_impact:
                "Screenreader-Nutzer, die per Landmark-Navigation zwischen Hauptinhalt, \
                 Navigation und Fußzeile wechseln, überspringen dieses Element unbemerkt — \
                 es erscheint in keiner der angesteuerten Regionen.",
            user_impact_en:
                "Screen reader users who switch between main content, navigation, and footer \
                 via landmark navigation skip over this element without noticing — it does not \
                 appear in any of the regions they navigate to.",
            typical_cause:
                "Markup, das per JavaScript nachträglich außerhalb des Haupt-Layouts \
                 eingefügt wird, Reste von Drittanbieter-Widgets, oder Inhalte, die vor dem \
                 ersten Landmark stehen, ohne selbst in main/nav/header/footer eingebettet zu sein.",
            typical_cause_en:
                "Markup inserted outside the main layout via JavaScript, leftover third-party \
                 widget content, or content placed before the first landmark without being \
                 embedded inside main/nav/header/footer itself.",
            recommendation:
                "Das Element in die passende Landmark-Region verschieben (z. B. einen Link in \
                 <nav> oder <main>). Passt keine bestehende Region, einen zusätzlichen Bereich \
                 mit role=\"region\" und einem beschreibenden aria-label anlegen.",
            recommendation_en:
                "Move the element into the appropriate landmark region (e.g. a link into <nav> \
                 or <main>). If no existing region fits, add an additional region with \
                 role=\"region\" and a descriptive aria-label.",
            technical_note:
                "HTML5-Landmarks konsequent nutzen: <header>, <nav>, <main>, <aside>, <footer>. \
                 Skip-Links vor dem ersten Landmark sind bewusst ausgenommen. Für Bereiche ohne \
                 passendes semantisches Element role=\"region\" plus aria-label verwenden.",
            technical_note_en:
                "Use HTML5 landmarks consistently: <header>, <nav>, <main>, <aside>, <footer>. \
                 Skip links placed before the first landmark are deliberately exempt. For areas \
                 without a matching semantic element, use role=\"region\" plus aria-label.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<body><nav>...</nav><a href=\"/contact\">Contact</a><main>...</main></body>"),
            example_good: Some("<body><nav><a href=\"/contact\">Contact</a></nav><main>...</main></body>"),
            example_decorative: None,
        },
    ),
    // Rule-specific override for `landmark_granular.rs`'s `check_landmark_unique`
    // findings (axe_id "landmark-unique"). Without this entry, get_explanation()
    // falls through to the generic WCAG 1.3.1 explanation above (written for
    // tables/lists/forms) via its internal taxonomy-rule_id → WCAG-id fallback —
    // confirmed live (plan/1-root-cause-title-occurrence-mismatch.md,
    // inros-lackner-de report, 2026-09-06): a 31-occurrence landmark-uniqueness
    // finding was titled "Fehlende semantische Struktur" instead of its own,
    // correct "Landmarks nicht eindeutig benannt" — same bug class as #571.
    (
        "landmark-unique",
        RuleExplanation {
            customer_title: "Landmarks nicht eindeutig benannt",
            customer_title_en: "Landmarks are not uniquely named",
            customer_description:
                "Mehrere Bereiche der Seite mit derselben Rolle (z. B. zwei Navigationen oder \
                 zwei Aside-Bereiche) tragen keinen unterscheidbaren Namen. Screenreader \
                 kündigen beide Bereiche mit demselben generischen Namen an.",
            customer_description_en:
                "Multiple regions of the page with the same role (e.g. two navigation areas \
                 or two aside regions) share no distinguishing name. Screen readers announce \
                 both regions with the same generic name.",
            user_impact:
                "Screenreader-Nutzer, die per Landmark-Navigation zwischen Bereichen springen, \
                 hören z. B. zweimal 'Navigation' und können ohne weitere Erkundung nicht \
                 unterscheiden, welcher Bereich gemeint ist.",
            user_impact_en:
                "Screen reader users who jump between regions via landmark navigation hear, \
                 for example, 'Navigation' announced twice and cannot tell the regions apart \
                 without further exploration.",
            typical_cause:
                "Mehrere <nav>-, <aside>- oder role=\"region\"-Elemente auf derselben Seite \
                 (z. B. Hauptnavigation und Breadcrumb-Navigation), denen kein aria-label oder \
                 aria-labelledby zur Unterscheidung mitgegeben wurde.",
            typical_cause_en:
                "Multiple <nav>, <aside>, or role=\"region\" elements on the same page (e.g. \
                 main navigation and breadcrumb navigation) without an aria-label or \
                 aria-labelledby to distinguish them.",
            recommendation:
                "Jedem Landmark derselben Rolle einen eigenen, beschreibenden aria-label geben, \
                 z. B. aria-label=\"Hauptnavigation\" und aria-label=\"Breadcrumb\".",
            recommendation_en:
                "Give each landmark of the same role its own descriptive aria-label, e.g. \
                 aria-label=\"Main navigation\" and aria-label=\"Breadcrumb\".",
            technical_note:
                "aria-label oder aria-labelledby auf jedem der mehreren gleichrangigen \
                 Landmarks ergänzen. Die Namen müssen sich tatsächlich unterscheiden — reine \
                 Wiederholung derselben Bezeichnung erfüllt die Regel nicht.",
            technical_note_en:
                "Add aria-label or aria-labelledby to each of the repeated same-role \
                 landmarks. The names must actually differ — repeating the identical label \
                 does not satisfy this check.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<nav>...</nav>\n<nav>...</nav>"),
            example_good: Some(
                "<nav aria-label=\"Main navigation\">...</nav>\n<nav aria-label=\"Breadcrumb\">...</nav>",
            ),
            example_decorative: None,
        },
    ),
    // Rule-specific entry for `keyboard.rs`'s NotTestable finding (axe_id
    // "keyboard-trap", WCAG 2.1.2). Without this entry, get_explanation()
    // returns None and the PDF fell back to the raw, English-only
    // Violation.fix_suggestion text unlocalized — confirmed live leaking
    // into German reports (shop.satower-mosterei.de, säfte.com, 2026-08-31;
    // report-quality review, 2026-09-01).
    (
        "keyboard-trap",
        RuleExplanation {
            customer_title: "Tastaturfalle nicht automatisch prüfbar",
            customer_title_en: "Keyboard trap cannot be automatically tested",
            customer_description:
                "Ob der Tastaturfokus in einem Dialog, Karussell oder individuellen \
                 JavaScript-Bedienelement hängen bleiben kann, lässt sich nicht automatisiert \
                 feststellen. Das erfordert manuelles Navigieren ausschließlich mit der \
                 Tab-Taste.",
            customer_description_en:
                "Whether keyboard focus can become permanently trapped inside a dialog, \
                 carousel, or custom JavaScript widget cannot be determined automatically. \
                 It requires manually navigating the page using only the Tab key.",
            user_impact:
                "Personen, die ausschließlich mit der Tastatur oder einem Screenreader \
                 navigieren, können in einem Bedienelement stecken bleiben und die Seite \
                 nicht mehr per Tastatur verlassen.",
            user_impact_en:
                "People who navigate exclusively via keyboard or screen reader can become \
                 stuck inside a component and lose the ability to leave the page by keyboard.",
            typical_cause:
                "Individuelle Dialoge, Karussells oder Widgets mit eigener JavaScript-\
                 Fokusverwaltung, die keinen Weg zurück per Escape-Taste, erreichbarem \
                 Schließen-Button oder dokumentiertem Tastaturkürzel vorsehen.",
            typical_cause_en:
                "Custom dialogs, carousels, or widgets with their own JavaScript focus \
                 management that provide no way back out via the Escape key, a reachable \
                 close button, or a documented keyboard shortcut.",
            recommendation:
                "Für jeden fokussierbaren Bereich einen Weg zum Verlassen per Tastatur \
                 sicherstellen: Escape-Taste, ein per Tab erreichbarer, sichtbarer \
                 Schließen-Button, oder ein dokumentiertes Tastaturkürzel.",
            recommendation_en:
                "Ensure every focusable region has a way to leave it by keyboard: the \
                 Escape key, a visible close button reachable by Tab, or a documented \
                 keyboard shortcut.",
            technical_note:
                "Nur mit Tab/Shift+Tab durch die Seite navigieren und prüfen, ob jeder \
                 Dialog/jedes Widget verlassen werden kann. Bei eigener Fokusverwaltung: \
                 Escape-Handler und einen erreichbaren Schließen-Button implementieren.",
            technical_note_en:
                "Navigate the page using only Tab/Shift+Tab and verify every dialog/widget \
                 can be exited. For custom focus management, implement an Escape handler and \
                 a reachable close button.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    // Rule-specific override for presentation-semantic-children violations.
    // The WCAG 1.3.1 fallback is too generic (tables/lists/forms) for this case —
    // the actual issue is an ARIA role conflict in navigation markup.
    (
        "a11y.presentation_semantic_children.invalid",
        RuleExplanation {
            customer_title: "ARIA-Rollen-Konflikt in Navigation",
            customer_title_en: "ARIA role conflict in navigation",
            customer_description:
                "Ein Element ist mit role=\"none\" als semantisch leer markiert, \
                 enthält aber Kindelemente mit eigenen ARIA-Rollen. Das verwirrt \
                 Screenreader: Der Container hat keine Semantik, aber die enthaltenen \
                 Elemente referenzieren Rollentypen, die einen bestimmten Container \
                 voraussetzen.",
            customer_description_en:
                "An element is marked as semantically empty with role=\"none\", but \
                 contains child elements with their own ARIA roles. This confuses screen \
                 readers: the container has no semantics, but the contained elements \
                 reference role types that require a specific parent container.",
            user_impact:
                "Screenreader können Navigationsmenüs nicht korrekt interpretieren. \
                 Tastaturnutzer erhalten möglicherweise falsche oder fehlende \
                 Navigationshinweise.",
            user_impact_en:
                "Screen readers cannot correctly interpret navigation menus. Keyboard \
                 users may receive incorrect or missing navigation cues.",
            typical_cause:
                "Verwendung von ARIA-Mustern aus Desktop-Anwendungen (z. B. \
                 role=\"menuitem\") für Website-Navigation. role=\"none\" auf \
                 List-Elementen, die ARIA-Kinder mit Rollenanforderungen enthalten.",
            typical_cause_en:
                "Using ARIA patterns from desktop applications (e.g. role=\"menuitem\") \
                 for website navigation. role=\"none\" on list items that contain ARIA \
                 children with role requirements.",
            recommendation:
                "Website-Navigation ohne ARIA-Menubar-Muster aufbauen: \
                 <nav><ul><li><a> reicht für Screenreader aus. role=\"menuitem\" und \
                 role=\"menubar\" sind für Desktop-App-Menüs reserviert, nicht für \
                 Seitennavigation. role=\"none\" nur auf Elementen verwenden, die \
                 wirklich keine Kinder mit Eigenrollen haben.",
            recommendation_en:
                "Build website navigation without the ARIA Menubar pattern: \
                 <nav><ul><li><a> is sufficient for screen readers. role=\"menuitem\" \
                 and role=\"menubar\" are reserved for desktop application menus, not \
                 website navigation. Only use role=\"none\" on elements that truly have \
                 no children with their own roles.",
            technical_note:
                "ARIA APG: Navigation-Menubar ist für App-ähnliche Menüs mit \
                 Pfeilnavigation. Website-Navigation verwendet <nav> + natürliche \
                 Listenstruktur. role=\"none\" hebt nicht die Rollen von Nachfahren auf.",
            technical_note_en:
                "ARIA APG: Navigation Menubar is for app-like menus with arrow key \
                 navigation. Website navigation uses <nav> + natural list structure. \
                 role=\"none\" does not remove the roles of descendant elements.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<li role=\"none\"><a href=\"/home\" role=\"menuitem\">Home</a></li>"),
            example_good: Some("<li><a href=\"/home\">Home</a></li>"),
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
            example_bad: Some("color: #999999; background: #ffffff; /* contrast 2.8:1 */"),
            example_good: Some("color: #595959; background: #ffffff; /* contrast 7:1 */"),
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
        "1.4.10",
        RuleExplanation {
            customer_title: "Kein automatischer Textumbruch (Reflow)",
            customer_title_en: "No automatic text wrapping (Reflow)",
            customer_description:
                "Die Website passt sich bei starker Vergrößerung oder schmalen Bildschirmen \
                 (320 CSS-Pixeln) nicht flexibel an. Nutzer müssen horizontal scrollen, um Text zu lesen.",
            customer_description_en:
                "The website does not adapt flexibly to high zoom levels or small screens \
                 (320 CSS pixels). Users have to scroll horizontally to read the text.",
            user_impact:
                "Menschen mit Sehbehinderungen, die stark zoomen (bis 400%), müssen \
                 in zwei Dimensionen scrollen. Das Lesen langer Texte wird extrem mühsam oder unmöglich.",
            user_impact_en:
                "People with visual impairments who zoom in heavily (up to 400%) must scroll \
                 in two directions. Reading text becomes extremely tedious or impossible.",
            typical_cause:
                "Verwendung von festen Pixelbreiten (width: 960px) oder overflow: hidden, \
                 was den Umbruch von Texten verhindert.",
            typical_cause_en:
                "Using fixed pixel widths (width: 960px) or overflow: hidden, which \
                 prevents text from wrapping properly.",
            recommendation:
                "Nutze responsives Design mit relativen Einheiten (%, vw, rem). Vermeide feste \
                 Breiten auf Containern, die Text enthalten. Teste Zoom auf 400%.",
            recommendation_en:
                "Use responsive design with relative units (%, vw, rem). Avoid fixed widths \
                 on containers holding text. Test zooming up to 400%.",
            technical_note:
                "WCAG 2.1 Level AA: No loss of content or function at width 320px (horizontal scroll \
                 must be avoided for text-flow containers). Use max-width: 100% and flexbox/grid.",
            technical_note_en:
                "WCAG 2.1 Level AA: No loss of content or function at width 320px (horizontal scroll \
                 must be avoided for text-flow containers). Use max-width: 100% and flexbox/grid.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("div.container { width: 960px; }"),
            example_good: Some("div.container { max-width: 100%; width: 100%; }"),
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
        "1.4.12",
        RuleExplanation {
            customer_title: "Inhalt wird bei größerem Zeilen-/Wortabstand abgeschnitten",
            customer_title_en: "Content is clipped when text spacing is increased",
            customer_description:
                "Wenn Nutzer den Zeilen-, Wort- oder Buchstabenabstand für bessere Lesbarkeit \
                 erhöhen, schneidet ein Container mit fester Höhe oder overflow:hidden Teile \
                 des Textes ab.",
            customer_description_en:
                "When users increase line, word, or letter spacing for better readability, a \
                 container with a fixed height or overflow:hidden clips parts of the text.",
            user_impact:
                "Nutzer mit Leseschwäche oder Sehbeeinträchtigung, die größere Abstände zur \
                 besseren Lesbarkeit einstellen, verlieren Inhalte oder können sie nicht mehr \
                 vollständig lesen.",
            user_impact_en:
                "Users with reading difficulties or visual impairments who increase spacing \
                 for readability lose content or can no longer read it fully.",
            typical_cause:
                "Fixe Höhen oder Breiten mit overflow:hidden auf Textcontainern, etwa bei \
                 Karten, Buttons oder Navigationselementen mit knapp bemessenem Platz.",
            typical_cause_en:
                "Fixed heights or widths with overflow:hidden on text containers, e.g. cards, \
                 buttons, or navigation elements with tightly sized space.",
            recommendation:
                "Textcontainer flexibel gestalten (min-height statt fester Höhe, kein \
                 overflow:hidden auf Textbereichen) und mit den WCAG-Mindestabständen testen.",
            recommendation_en:
                "Design text containers flexibly (min-height instead of a fixed height, no \
                 overflow:hidden on text areas) and test against the WCAG minimum spacing values.",
            technical_note:
                "WCAG-1.4.12-Mindestwerte simulieren (Zeilenhöhe 1.5x, Absatzabstand 2x, \
                 Buchstabenabstand 0.12x, Wortabstand 0.16x der Schriftgröße) und prüfen, ob \
                 Container Inhalte abschneiden statt zu wachsen.",
            technical_note_en:
                "Simulate the WCAG 1.4.12 minimum values (line height 1.5x, paragraph spacing \
                 2x, letter spacing 0.12x, word spacing 0.16x of font size) and check whether \
                 containers clip content instead of growing.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<div style=\"height: 40px; overflow: hidden;\">Langer Textinhalt...</div>"),
            example_good: Some("<div style=\"min-height: 40px; overflow: visible;\">Langer Textinhalt...</div>"),
            example_decorative: None,
        },
    ),
    (
        "1.4.13",
        RuleExplanation {
            customer_title: "Fehlende Barrierefreiheit bei Zusatzinhalten (Tooltips)",
            customer_title_en: "Suboptimal or inaccessible hover/focus content (tooltips)",
            customer_description:
                "Zusatzinhalte, die erst bei Hover (Mauszeiger) oder Fokus (Tastatur) erscheinen \
                 (wie Tooltips oder native title-Attribute), sind nicht barrierefrei zugänglich oder unvollständig verknüpft.",
            customer_description_en:
                "Additional content appearing on hover or focus (such as tooltips or native title attributes) \
                 is not accessible or not properly linked in the HTML code.",
            user_impact:
                "Nutzer von Screenreadern hören diese Inhalte gar nicht, Tastaturnutzer können sie oft nicht schließen, \
                 und auf Touchgeräten (Mobilgeräten) sind sie in der Regel unbrauchbar.",
            user_impact_en:
                "Screen reader users do not hear tooltips, keyboard users cannot dismiss them, \
                 and they are often completely unusable on touch devices.",
            typical_cause:
                "Verwendung des nativen HTML title-Attributs als einzige Beschriftung, unvollständige \
                 Zuweisung per aria-describedby oder fehlende Tastaturbedienbarkeit bei Custom-Tooltips.",
            typical_cause_en:
                "Using the native HTML title attribute for important information, missing aria-describedby \
                 associations, or lack of keyboard control on custom tooltips.",
            recommendation:
                "Verwende eine sichtbare Beschriftung oder moderne, barrierefreie Tooltips (schließbar mit Escape, \
                 verknüpft über aria-describedby und hoverbar).",
            recommendation_en:
                "Use visible labels or modern, accessible tooltips (dismissible with Escape, linked via \
                 aria-describedby, and hoverable).",
            technical_note:
                "Zusatzinhalte müssen (1) hoverbar sein (Maus kann in den Tooltip bewegt werden), (2) dismissible \
                 sein (Esc-Taste blendet ihn aus), (3) persistent sein. Tooltips mit role=\"tooltip\" per aria-describedby verknüpfen.",
            technical_note_en:
                "Additional content must be (1) hoverable, (2) dismissible (Esc), and (3) persistent. \
                 Associate tooltips carrying role=\"tooltip\" with their trigger via aria-describedby.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<button title=\"More info\">...</button>\n<div role=\"tooltip\">Help text</div>"),
            example_good: Some(
                "<button aria-describedby=\"tip1\">More info</button>\n<div id=\"tip1\" role=\"tooltip\">Help text</div>"
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
    // Rule-specific override for `keyboard.rs`'s "focusable without
    // interactive role" check (axe_id "focusable-no-role"). The 2.1.1
    // fallback above is written for elements that are NOT reachable by
    // keyboard at all ("add tabindex + a keydown handler") — this check
    // detects the opposite: the element is already focusable, but has no
    // role telling assistive technology what it does. Suggesting extra
    // focusability/keydown handling here would be wrong and can worsen
    // focus order (#571).
    (
        "focusable-no-role",
        RuleExplanation {
            customer_title: "Fokussierbares Element ohne interaktive Rolle",
            customer_title_en: "Focusable element without an interactive role",
            customer_description:
                "Ein Element kann per Tastatur fokussiert werden, trägt aber keine (oder eine \
                 nicht-interaktive) ARIA-Rolle. Assistive Technologien wissen dadurch nicht, \
                 was beim Fokussieren dieses Elements möglich ist.",
            customer_description_en:
                "An element can receive keyboard focus, but carries no (or a non-interactive) \
                 ARIA role. Assistive technologies therefore cannot tell what is possible once \
                 the element receives focus.",
            user_impact:
                "Screenreader-Nutzer hören beim Fokussieren keine oder eine irreführende \
                 Rollenangabe (z. B. \"Bereich\" statt \"Dialog\" oder \"Schaltfläche\") und \
                 können nicht einschätzen, welche Interaktion erwartet wird.",
            user_impact_en:
                "Screen reader users hear no role, or a misleading one (e.g. \"region\" \
                 instead of \"dialog\" or \"button\"), when the element receives focus, and \
                 cannot tell what interaction is expected.",
            typical_cause:
                "Ein Container — etwa ein Consent-Banner oder ein Drittanbieter-Widget — \
                 erhält technisch bedingt Tastaturfokus, ohne dass ihm eine passende ARIA-Rolle \
                 oder ein natives interaktives Element zugrunde liegt.",
            typical_cause_en:
                "A container — such as a consent banner or a third-party widget — technically \
                 receives keyboard focus, without a matching ARIA role or native interactive \
                 element behind it.",
            recommendation:
                "Dem Element eine passende ARIA-Rolle geben, die seine tatsächliche Funktion \
                 beschreibt, oder es durch ein natives interaktives HTML-Element ersetzen. \
                 Ein Element lediglich fokussierbar zu machen, ohne seine Funktion für \
                 Screenreader erkennbar zu machen, behebt das Problem nicht.",
            recommendation_en:
                "Give the element an ARIA role that matches its actual function, or replace it \
                 with a native interactive HTML element. Making an element merely focusable \
                 without exposing its function to screen readers does not fix the underlying \
                 issue.",
            technical_note:
                "Prüfen, warum das Element fokussierbar ist (tabindex, Drittanbieter-Skript). \
                 Ist es tatsächlich interaktiv: passende Rolle (z. B. role=\"dialog\", \
                 role=\"button\") und zugänglichen Namen ergänzen. Hat es keine eigene \
                 Interaktion: Fokussierbarkeit entfernen (tabindex=\"-1\" oder tabindex-Attribut löschen).",
            technical_note_en:
                "Check why the element is focusable (tabindex, third-party script). If it is \
                 genuinely interactive: add a matching role (e.g. role=\"dialog\", \
                 role=\"button\") and an accessible name. If it has no interaction of its own: \
                 remove focusability (tabindex=\"-1\" or delete the tabindex attribute).",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div tabindex=\"0\">...</div>"),
            example_good: Some("<div tabindex=\"0\" role=\"dialog\" aria-label=\"Cookie-Einstellungen\">...</div>"),
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
            example_good: Some("<body><a href=\"#main\" class=\"skip-link\">Skip to content</a><nav>...</nav><main id=\"main\">...</main></body>"),
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
            example_good: Some("<title>Contact — Example Company</title>"),
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
            example_bad: Some("<a href=\"/services\">read more</a>"),
            example_good: Some("<a href=\"/services\">Our web development services</a>"),
            example_decorative: None,
        },
    ),
    // Rule-specific override for `link_purpose.rs`'s "generic link text with
    // detected context" case (axe_id "link-name-context"). This branch is
    // demoted to a Warning (#569) because WCAG 2.4.4 explicitly allows a
    // link's purpose to be derived from its context, not just its own text —
    // the 2.4.4 fallback above is written for the no-context case and reads
    // as an unconditional rewrite instruction, which overclaims what an
    // automated check can confirm here (#571).
    (
        "link-name-context",
        RuleExplanation {
            customer_title: "Generischer Linktext mit möglichem Kontextbezug",
            customer_title_en: "Generic link text with possible contextual meaning",
            customer_description:
                "Der Linktext allein ist unspezifisch (z. B. \"Start\", \"weiter\"), steht \
                 aber neben beschreibendem Text, der den Linkzweck erklären könnte. WCAG 2.4.4 \
                 erlaubt es ausdrücklich, den Linkzweck aus dem Linktext zusammen mit seinem \
                 Kontext abzuleiten — ob das hier zutrifft, kann nur eine manuelle Prüfung \
                 bestätigen.",
            customer_description_en:
                "The link text alone is unspecific (e.g. \"Start\", \"next\"), but sits next \
                 to descriptive text that may explain the link's purpose. WCAG 2.4.4 \
                 explicitly allows a link's purpose to be derived from its text together with \
                 its context — whether that is the case here can only be confirmed by manual \
                 review.",
            user_impact:
                "Ob der Linkzweck für Screenreader-Nutzer verständlich ist, hängt davon ab, ob \
                 der umgebende Text tatsächlich vorgelesen wird, bevor der Link erreicht wird, \
                 und ob die Verbindung eindeutig ist. Ohne manuelle Prüfung ist unklar, ob \
                 Nutzer davon betroffen sind.",
            user_impact_en:
                "Whether the link's purpose is understandable to screen reader users depends \
                 on whether the surrounding text is actually read out before the link is \
                 reached, and whether the connection is unambiguous. Without manual review it \
                 is unclear whether users are actually affected.",
            typical_cause:
                "Teaser- oder Karten-Komponenten, in denen ein generischer Button- oder \
                 Linktext (\"Start\", \"weiter\") direkt auf eine Überschrift oder einen \
                 Beschreibungstext folgt.",
            typical_cause_en:
                "Teaser or card components where a generic button or link text (\"Start\", \
                 \"next\") directly follows a heading or description text.",
            recommendation:
                "Manuell prüfen, ob der umgebende Text den Linkzweck visuell und für \
                 Screenreader eindeutig macht. Ist das der Fall, ist keine Änderung nötig. \
                 Andernfalls den Linktext selbst aussagekräftig formulieren oder per \
                 aria-label/aria-labelledby ergänzen.",
            recommendation_en:
                "Manually verify whether the surrounding text makes the link's purpose \
                 unambiguous both visually and for screen readers. If so, no change is \
                 needed. Otherwise, phrase the link text itself descriptively, or supplement \
                 it via aria-label/aria-labelledby.",
            technical_note:
                "Kontext zählt nur, wenn er programmatisch mit dem Link verknüpft ist oder ihm \
                 unmittelbar vorausgeht (gleicher Satz, gleiche Listenzelle, gleiche \
                 Überschrift). Bei Unsicherheit aria-labelledby auf das Kontext-Element setzen, \
                 um die Verbindung eindeutig zu machen.",
            technical_note_en:
                "Context only counts if it is programmatically associated with the link or \
                 immediately precedes it (same sentence, same list cell, same heading). When \
                 in doubt, use aria-labelledby pointing at the context element to make the \
                 association explicit.",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
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
            example_bad: Some("<div class=\"big-title\">Heading</div>"),
            example_good: Some("<h1>Page title</h1>\n<h2>Section</h2>\n<h3>Subsection</h3>"),
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
            example_bad: Some("<button aria-label=\"Close menu\">X</button>"),
            example_good: Some("<button aria-label=\"Close\">Close</button>"),
            example_decorative: None,
        },
    ),
    (
        "2.5.8",
        RuleExplanation {
            customer_title: "Unzureichende Klickzielgröße",
            customer_title_en: "Insufficient click target size",
            customer_description:
                "Bedienelemente wie Buttons, Links oder Icons haben eine Klickfläche von \
                 weniger als 24×24 CSS-Pixeln.",
            customer_description_en:
                "Interactive elements such as buttons, links, or icons have a clickable area \
                 smaller than 24×24 CSS pixels.",
            user_impact:
                "Nutzer mit motorischen Einschränkungen oder auf Touchscreens treffen kleine \
                 Zielflächen leicht daneben und lösen versehentlich falsche Aktionen aus.",
            user_impact_en:
                "Users with motor impairments or on touchscreens easily miss small target \
                 areas and accidentally trigger the wrong action.",
            typical_cause:
                "Icon-Buttons ohne zusätzliches Padding, dicht gepackte Link- oder \
                 Icon-Listen, kleine Formularsteuerelemente im mobilen Layout.",
            typical_cause_en:
                "Icon buttons without extra padding, tightly packed link or icon lists, small \
                 form controls in the mobile layout.",
            recommendation:
                "Klickflächen auf mindestens 24×24 CSS-Pixel vergrößern, z. B. durch Padding, \
                 oder ausreichend Abstand zu benachbarten Zielen einhalten.",
            recommendation_en:
                "Increase clickable areas to at least 24×24 CSS pixels, e.g. via padding, or \
                 ensure sufficient spacing to neighboring targets.",
            technical_note:
                "Ausnahmen gelten u. a. für Inline-Links im Fließtext und wenn ein \
                 gleichwertiges, größeres Ziel für dieselbe Funktion existiert.",
            technical_note_en:
                "Exceptions apply to, among others, inline links within body text and cases \
                 where an equivalent, larger target for the same function exists.",
            responsible_role: Role::DesignUx,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button style=\"width: 16px; height: 16px; padding: 0;\">×</button>"),
            example_good: Some("<button style=\"width: 24px; height: 24px; padding: 4px;\">×</button>"),
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
    // Über die geteilte Kennung erreichbar, nicht über das Kriterium: Der
    // Befund fällt seit dem Wegfall von 4.1.1 unter 4.1.2, und die dortige
    // Erklärung (Name, Rolle, Wert allgemein) träfe die Dopplung nicht
    // (Plan 54 §2).
    (
        "ids/duplicate",
        RuleExplanation {
            customer_title: "Mehrfach vergebene ID wird referenziert",
            customer_title_en: "Referenced ID is assigned more than once",
            customer_description:
                "Mehrere Elemente auf der Seite tragen dieselbe id, und ein anderes \
                 Element verweist darauf (z. B. label for, aria-labelledby). Welches \
                 der Elemente gemeint ist, ist damit nicht mehr bestimmbar.",
            customer_description_en:
                "Multiple elements on the page share the same id, and another element \
                 references it (e.g. label for, aria-labelledby). Which of them is meant \
                 can no longer be determined.",
            user_impact:
                "Screenreader lösen die Beziehung auf das falsche Element auf und geben \
                 einen falschen Namen oder eine falsche Beschreibung aus.",
            user_impact_en:
                "Screen readers resolve the relationship to the wrong element and \
                 announce the wrong name or description.",
            typical_cause:
                "Wiederholt eingebundene Komponenten oder Templates (z. B. Menüpunkte, \
                 Widgets), die eine statische ID statt einer pro Instanz eindeutigen ID \
                 verwenden.",
            typical_cause_en:
                "Repeatedly embedded components or templates (e.g. menu items, widgets) \
                 that use a static ID instead of a per-instance unique ID.",
            recommendation:
                "Jede ID nur einmal im Dokument vergeben. Bei wiederverwendeten \
                 Komponenten die ID dynamisch generieren (z. B. mit Index oder Slug).",
            recommendation_en:
                "Assign each ID only once per document. For reused components, generate \
                 the ID dynamically (e.g. with an index or slug).",
            technical_note:
                "IDs eindeutig machen, etwa durch Anhängen eines Index oder einer \
                 eindeutigen Kennung pro Komponenteninstanz.",
            technical_note_en:
                "Make IDs unique, e.g. by appending an index or a unique identifier per \
                 component instance.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
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
            example_good: Some("<button aria-label=\"Open menu\"><svg aria-hidden=\"true\">...</svg></button>"),
            example_decorative: None,
        },
    ),
    (
        "a11y.aria_hidden_focus.invalid",
        RuleExplanation {
            customer_title: "Tastaturfokus auf unsichtbaren Elementen",
            customer_title_en: "Keyboard focus on hidden elements",
            customer_description:
                "Ein interaktives Element ist über aria-hidden=\"true\" vor Screenreadern versteckt, \
                 kann aber dennoch mit der Tastatur (Tabulator) fokussiert werden.",
            customer_description_en:
                "An interactive element is hidden from screen readers via aria-hidden=\"true\", \
                 but can still be focused using the keyboard (tab key).",
            user_impact:
                "Tastaturnutzer steuern auf unsichtbare Elemente (Fokus-Stolperfallen), \
                 während Screenreader-Nutzer keinerlei Rückmeldung über dieses Element erhalten.",
            user_impact_en:
                "Keyboard users navigate to hidden elements (focus traps), \
                 while screen reader users receive no feedback about the element.",
            typical_cause:
                "Es wurde aria-hidden=\"true\" verwendet, um ein Element visuell auszublenden, \
                 ohne tabindex=\"-1\" zu setzen oder das Element per display:none zu verstecken.",
            typical_cause_en:
                "Using aria-hidden=\"true\" to hide an element visually without setting \
                 tabindex=\"-1\" or hiding the element via display:none.",
            recommendation:
                "Wenn ein Element vollständig versteckt sein soll, display:none oder visibility:hidden verwenden. \
                 Soll es nur vor Screenreadern versteckt sein, muss es unfokussierbar sein (tabindex=\"-1\").",
            recommendation_en:
                "To hide an element completely, use display:none or visibility:hidden. \
                 If it should only be hidden from screen readers, make it unfocusable (tabindex=\"-1\").",
            technical_note:
                "Focusable elements (a, button, input) inside an aria-hidden=\"true\" tree must \
                 either be removed from tab order or hidden completely.",
            technical_note_en:
                "Focusable elements (a, button, input) inside an aria-hidden=\"true\" tree must \
                 either be removed from tab order or hidden completely.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button aria-hidden=\"true\">Menu</button>"),
            example_good: Some("<button aria-hidden=\"true\" tabindex=\"-1\">Menu</button>"),
            example_decorative: None,
        },
    ),
    (
        "a11y.aria_prohibited_attr.invalid",
        RuleExplanation {
            customer_title: "Ungültige oder verbotene ARIA-Attribute",
            customer_title_en: "Prohibited ARIA attributes",
            customer_description:
                "Für bestimmte HTML-Elemente oder ARIA-Rollen werden Attribute verwendet, die für \
                 diese Rolle nicht zulässig oder sinnvoll sind.",
            customer_description_en:
                "Attributes are used on HTML elements or ARIA roles that are not permitted or \
                 meaningful for that specific role.",
            user_impact:
                "Screenreader erhalten widersprüchliche oder fehlerhafte Signale, was die \
                 Bedienbarkeit der Website beeinträchtigen kann.",
            user_impact_en:
                "Screen readers receive conflicting or incorrect markup signals, which can \
                 impair accessibility.",
            typical_cause:
                "Verwendung von ARIA-Attributen (wie aria-label oder aria-expanded) auf Elementen, \
                 die keine interaktive Rolle haben (z. B. span oder div ohne role).",
            typical_cause_en:
                "Using ARIA attributes (like aria-label or aria-expanded) on elements that \
                 do not have an interactive role (e.g. span or div without role).",
            recommendation:
                "Prüfe die ARIA-Spezifikation für die jeweilige Rolle. Verwende ARIA-Attribute \
                 nur auf passenden Elementen und Rollen.",
            recommendation_en:
                "Check the ARIA specification for the respective role. Only use ARIA attributes \
                 on appropriate elements and roles.",
            technical_note:
                "Prohibited attributes on generic elements: <span aria-label=\"text\"> is invalid. \
                 Use native elements or valid roles (e.g. role=\"button\").",
            technical_note_en:
                "Prohibited attributes on generic elements: <span aria-label=\"text\"> is invalid. \
                 Use native elements or valid roles (e.g. role=\"button\").",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<span aria-label=\"Menu\">...</span>"),
            example_good: Some("<button aria-label=\"Menu\">...</button>"),
            example_decorative: None,
        },
    ),
    (
        "a11y.modern_attributes.invalid",
        RuleExplanation {
            customer_title: "Fehlerhafte Dialog- oder Popover-Steuerung",
            customer_title_en: "Broken dialog or popover control",
            customer_description:
                "Moderne HTML-Attribute für Popover, Dialoge oder gesperrte Bereiche sind \
                 unvollständig oder widersprüchlich eingesetzt.",
            customer_description_en:
                "Modern HTML attributes for popovers, dialogs, or disabled regions are used \
                 incompletely or inconsistently.",
            user_impact:
                "Tastatur- und Screenreader-Nutzer können Oberflächen öffnen, fokussieren oder \
                 verlassen, ohne eine verständliche Rückmeldung zu erhalten.",
            user_impact_en:
                "Keyboard and screen reader users may open, focus, or leave surfaces without \
                 understandable feedback.",
            typical_cause:
                "popovertarget verweist auf ein fehlendes Element, ein sichtbarer Dialog hat keinen \
                 zugänglichen Namen, oder ein aktiver Bereich ist gleichzeitig inert markiert.",
            typical_cause_en:
                "popovertarget points to a missing element, a visible dialog has no accessible \
                 name, or an active region is marked inert at the same time.",
            recommendation:
                "Popover-Ziele prüfen, sichtbare Dialoge und Menüs benennen und inert nur für \
                 tatsächlich inaktive Bereiche verwenden.",
            recommendation_en:
                "Check popover targets, name visible dialogs and menus, and use inert only for \
                 regions that are truly inactive.",
            technical_note:
                "popovertarget muss auf ein Element mit popover zeigen. Offene Dialoge, Menüs und \
                 Popover brauchen aria-label, aria-labelledby oder einen geeigneten Namen.",
            technical_note_en:
                "popovertarget must reference an element with popover. Open dialogs, menus, and \
                 popovers need aria-label, aria-labelledby, or another suitable name.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button popovertarget=\"menu\">Menu</button>\n<div id=\"menu\">...</div>"),
            example_good: Some("<button popovertarget=\"menu\">Menu</button>\n<div id=\"menu\" popover aria-label=\"Menu\">...</div>"),
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
    (
        "seo.headings.skipped_level",
        RuleExplanation {
            customer_title: "Übersprungene Überschriftenebene",
            customer_title_en: "Skipped heading level",
            customer_description:
                "Die Überschriften auf der Seite überspringen eine Ebene — zum Beispiel \
                 folgt auf eine H1 direkt eine H3, ohne dass eine H2 dazwischen steht. \
                 Das zerstört die logische Hierarchie, die Suchmaschinen und \
                 Screenreader zur Navigation nutzen.",
            customer_description_en:
                "The headings on the page skip a level — for example, an H1 is \
                 directly followed by an H3 without an H2 in between. This breaks the \
                 logical hierarchy that search engines and screen readers use for navigation.",
            user_impact:
                "Screenreader-Nutzer navigieren Seiten über Überschriften. Fehlende \
                 Ebenen unterbrechen diesen Pfad und erschweren die Orientierung. \
                 Suchmaschinen bewerten Seiten mit konsistenter Überschriftenstruktur \
                 als inhaltlich hochwertiger.",
            user_impact_en:
                "Screen reader users navigate pages via headings. Missing levels \
                 interrupt this path and make orientation harder. Search engines \
                 rate pages with consistent heading structure as higher quality content.",
            typical_cause:
                "CMS-Templates oder Komponenten setzen fest kodierte Überschriftenebenen \
                 (z. B. immer H3 in Footer-Spalten), unabhängig davon, was der Seitenkontext \
                 erfordert. Entwickler wählen Ebenen nach visuellem Stil statt nach Hierarchie.",
            typical_cause_en:
                "CMS templates or components hard-code heading levels (e.g. always H3 \
                 in footer columns) regardless of page context. Developers choose levels \
                 for visual style rather than hierarchy.",
            recommendation:
                "Überschriftenhierarchie von H1 abwärts lückenlos einhalten: H1 → H2 → H3. \
                 Wenn eine Überschrift visuell kleiner wirken soll, CSS-Klassen verwenden \
                 statt die HTML-Ebene zu ändern.",
            recommendation_en:
                "Maintain heading hierarchy from H1 downward without gaps: H1 → H2 → H3. \
                 To make a heading look smaller visually, use CSS classes rather than \
                 changing the HTML level.",
            technical_note:
                "WCAG 1.3.1 (Info and Relationships, Level A) erfordert, dass Struktur \
                 programmatisch erkennbar ist. Übersprungene Ebenen verletzen diese Anforderung. \
                 Screenreader wie NVDA/JAWS springen mit H-Tasten — Lücken erzeugen tote Punkte.",
            technical_note_en:
                "WCAG 1.3.1 (Info and Relationships, Level A) requires structure to be \
                 programmatically determinable. Skipped levels violate this requirement. \
                 Screen readers like NVDA/JAWS jump with H-keys — gaps create dead spots.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<h1>Page title</h1>\n<h3>Section</h3>"),
            example_good: Some("<h1>Page title</h1>\n<h2>Section</h2>"),
            example_decorative: None,
        },
    ),
    (
        "seo.headings.missing_h1",
        RuleExplanation {
            customer_title: "Fehlende H1-Überschrift",
            customer_title_en: "Missing H1 heading",
            customer_description:
                "Der Seite fehlt eine H1-Überschrift. Ohne sie ist der Seitenzweck für \
                 Suchmaschinen und Screenreader-Nutzer nicht auf den ersten Blick erkennbar.",
            customer_description_en:
                "The page has no H1 heading. Without one, its purpose isn't immediately \
                 recognizable to search engines or screen reader users.",
            user_impact:
                "Screenreader-Nutzer, die sich per Überschriftenliste orientieren, finden \
                 keinen Einstiegspunkt, der das Hauptthema der Seite benennt.",
            user_impact_en:
                "Screen reader users who orient via the heading list find no entry point \
                 naming the page's main topic.",
            typical_cause:
                "Das Seitentemplate setzt keine H1, oder ein Baustein, der normalerweise \
                 die H1 liefert, wurde entfernt oder falsch konfiguriert.",
            typical_cause_en:
                "The page template doesn't emit an H1, or a component that normally \
                 supplies it was removed or misconfigured.",
            recommendation:
                "Jeder Seite genau eine H1 geben, die das Hauptthema benennt und möglichst \
                 weit oben im Inhalt steht.",
            recommendation_en:
                "Give every page exactly one H1 naming the main topic, placed near the top \
                 of the content.",
            technical_note:
                "WCAG 1.3.1 (Info and Relationships, Level A) und gängige SEO-Praxis \
                 verlangen eine eindeutige, programmatisch erkennbare H1 pro Seite.",
            technical_note_en:
                "WCAG 1.3.1 (Info and Relationships, Level A) and standard SEO practice \
                 both expect one unambiguous, programmatically determinable H1 per page.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div class=\"page-title\">Page title</div>"),
            example_good: Some("<h1>Page title</h1>"),
            example_decorative: None,
        },
    ),
    (
        "seo.headings.multiple_h1",
        RuleExplanation {
            customer_title: "Mehrere H1-Überschriften",
            customer_title_en: "Multiple H1 headings",
            customer_description:
                "Die Seite enthält mehr als eine H1-Überschrift. Das untergräbt die \
                 inhaltliche Hierarchie — Suchmaschinen und Screenreader können keinen \
                 eindeutigen Hauptfokus der Seite ableiten.",
            customer_description_en:
                "The page contains more than one H1 heading. This undermines the content \
                 hierarchy — search engines and screen readers cannot derive a single main \
                 focus for the page.",
            user_impact:
                "Screenreader-Nutzer, die per Überschriftenliste navigieren, sehen mehrere \
                 gleichrangige Haupttitel und können das eigentliche Seitenthema nicht \
                 eindeutig zuordnen.",
            user_impact_en:
                "Screen reader users navigating via the heading list see several \
                 equally-ranked top-level titles and can't reliably identify the page's \
                 actual topic.",
            typical_cause:
                "Ein wiederverwendbarer Baustein (z. B. ein Hero- oder Logo-Bereich) setzt \
                 selbst eine H1, unabhängig davon, ob die Seite bereits eine eigene hat.",
            typical_cause_en:
                "A reusable component (e.g. a hero or logo block) emits its own H1 \
                 regardless of whether the page already has one.",
            recommendation:
                "Genau eine H1 pro Seite verwenden. Wiederverwendbare Bausteine sollten \
                 ihre Überschriftenebene konfigurierbar machen statt sie fest auf H1 zu setzen.",
            recommendation_en:
                "Use exactly one H1 per page. Reusable components should make their \
                 heading level configurable rather than hard-coding H1.",
            technical_note:
                "WCAG 1.3.1 (Info and Relationships, Level A) und SEO-Praxis erwarten eine \
                 eindeutige H1 pro Seite; mehrere H1 gelten als schwaches Strukturmerkmal.",
            technical_note_en:
                "WCAG 1.3.1 (Info and Relationships, Level A) and SEO practice both expect \
                 a single H1 per page; multiple H1s are treated as a weak structural signal.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<h1>Site name</h1>\n...\n<h1>Article title</h1>"),
            example_good: Some("<h1>Article title</h1>\n<p class=\"site-name\">Site name</p>"),
            example_decorative: None,
        },
    ),
    (
        "seo.headings.empty_heading",
        RuleExplanation {
            customer_title: "Leere Überschrift",
            customer_title_en: "Empty heading",
            customer_description:
                "Eine Überschrift auf der Seite enthält keinen Text. Leere Überschriften \
                 erzeugen Navigationsprobleme für Screenreader-Nutzer und werden von \
                 Suchmaschinen als schwaches Signal gewertet.",
            customer_description_en:
                "A heading on the page contains no text. Empty headings cause navigation \
                 problems for screen reader users and are treated as a weak signal by \
                 search engines.",
            user_impact:
                "Screenreader-Nutzer, die per Überschrift springen, landen auf einem \
                 unbenannten Punkt ohne erkennbaren Inhalt.",
            user_impact_en:
                "Screen reader users jumping by heading land on an unnamed point with no \
                 recognizable content.",
            typical_cause:
                "Eine Komponente rendert ihr Überschriften-Element unabhängig davon, ob ein \
                 Titel-Feld im CMS befüllt wurde, oder der Text wird ausschließlich über ein \
                 Hintergrundbild statt echten Text vermittelt.",
            typical_cause_en:
                "A component renders its heading element regardless of whether the CMS \
                 title field was filled in, or the text is conveyed only via a background \
                 image instead of real text.",
            recommendation:
                "Leere Überschriften-Elemente entfernen oder mit echtem, beschreibendem \
                 Text befüllen. Komponenten sollten das Überschriften-Element nur rendern, \
                 wenn ein Titel vorhanden ist.",
            recommendation_en:
                "Remove empty heading elements or fill them with real, descriptive text. \
                 Components should only render the heading element when a title is present.",
            technical_note:
                "WCAG 1.3.1 (Info and Relationships, Level A) und 2.4.6 (Headings and \
                 Labels, Level AA) erfordern, dass Überschriften einen erkennbaren, \
                 programmatisch verfügbaren Text tragen.",
            technical_note_en:
                "WCAG 1.3.1 (Info and Relationships, Level A) and 2.4.6 (Headings and \
                 Labels, Level AA) require headings to carry recognizable, programmatically \
                 available text.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<h2></h2>"),
            example_good: Some("<h2>Section title</h2>"),
            example_decorative: None,
        },
    ),
    // ── Added to fix the WCAG-criterion grouping fallback (plan/1
    // root-cause-title-occurrence-mismatch.md, "Mechanism 2"): each axe_id
    // below previously had no dedicated RuleExplanation and, once given its
    // own taxonomy Rule + LEGACY_WCAG_MAP entry, would otherwise fall back
    // to get_explanation()'s internal WCAG-criterion lookup and surface
    // text written for a different, unrelated check under the same
    // criterion (same failure mode as #571).
    (
        "server-side-image-map",
        RuleExplanation {
            customer_title: "Server-seitige Image-Map verwendet",
            customer_title_en: "Server-side image map used",
            customer_description: "Ein Bild nutzt eine server-seitige Image-Map (ismap-Attribut) statt einer client-seitigen Variante.",
            customer_description_en: "Server-side image map used. Keyboard and screen reader users cannot target the individual image regions.",
            user_impact: "Tastatur- und Screenreader-Nutzer können die einzelnen Bildbereiche nicht ansteuern.",
            user_impact_en: "Keyboard and screen reader users cannot target the individual image regions.",
            typical_cause: "ismap-Attribut auf einem Bild ohne äquivalente clientseitige Alternative.",
            typical_cause_en: "ismap attribute on an image without an equivalent client-side alternative.",
            recommendation: "Die server-seitige Image-Map durch eine client-seitige Image-Map (<map>/<area>) oder einfache Textlinks ersetzen.",
            recommendation_en: "Replace the server-side image map with a client-side image map (<map>/<area>) or plain text links.",
            technical_note: "ismap-Attribut auf einem Bild ohne äquivalente clientseitige Alternative.",
            technical_note_en: "ismap attribute on an image without an equivalent client-side alternative.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "svg-img-alt",
        RuleExplanation {
            customer_title: "Fehlender Alternativtext bei SVG-Grafiken",
            customer_title_en: "Missing alternative text on SVG graphics",
            customer_description: "Eine SVG-Grafik mit informativem Inhalt hat keinen Alternativtext (title, aria-label).",
            customer_description_en: "Missing alternative text on SVG graphics. Screen reader users receive no information about the graphic's content.",
            user_impact: "Screenreader-Nutzer erhalten keine Information über den Inhalt der Grafik.",
            user_impact_en: "Screen reader users receive no information about the graphic's content.",
            typical_cause: "SVG ohne <title>-Kindelement, aria-label oder aria-labelledby.",
            typical_cause_en: "SVG without a <title> child element, aria-label, or aria-labelledby.",
            recommendation: "Ein <title> als erstes Kindelement des <svg> ergänzen, aria-label verwenden, oder rein dekorative SVGs mit aria-hidden=\"true\" verstecken.",
            recommendation_en: "Add a <title> as the first child of the <svg>, use aria-label, or hide purely decorative SVGs with aria-hidden=\"true\".",
            technical_note: "SVG ohne <title>-Kindelement, aria-label oder aria-labelledby.",
            technical_note_en: "SVG without a <title> child element, aria-label, or aria-labelledby.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "area-alt",
        RuleExplanation {
            customer_title: "Fehlender Alternativtext bei Image-Map-Bereichen",
            customer_title_en: "Missing alternative text on image map areas",
            customer_description: "Ein aktiver <area>-Bereich einer Image-Map hat keinen Alternativtext.",
            customer_description_en: "Missing alternative text on image map areas. Screen reader users are not told where a clickable image region leads.",
            user_impact: "Screenreader-Nutzer erfahren nicht, wohin ein anklickbarer Bildbereich führt.",
            user_impact_en: "Screen reader users are not told where a clickable image region leads.",
            typical_cause: "<area>-Element ohne alt-Attribut.",
            typical_cause_en: "<area> element without an alt attribute.",
            recommendation: "Ein alt-Attribut auf dem <area>-Element ergänzen, das das Ziel beschreibt.",
            recommendation_en: "Add an alt attribute to the <area> element describing its destination.",
            technical_note: "<area>-Element ohne alt-Attribut.",
            technical_note_en: "<area> element without an alt attribute.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "input-image-alt",
        RuleExplanation {
            customer_title: "Fehlender Alternativtext bei Bild-Buttons",
            customer_title_en: "Missing alternative text on image buttons",
            customer_description: "Ein <input type=\"image\">-Button (z. B. eine Bild-Absende-Schaltfläche) hat keinen Alternativtext.",
            customer_description_en: "Missing alternative text on image buttons. Screen reader users are not told what action the button triggers.",
            user_impact: "Screenreader-Nutzer erfahren nicht, welche Aktion der Button auslöst.",
            user_impact_en: "Screen reader users are not told what action the button triggers.",
            typical_cause: "<input type=\"image\"> ohne alt-Attribut.",
            typical_cause_en: "<input type=\"image\"> without an alt attribute.",
            recommendation: "Ein alt-Attribut auf dem <input type=\"image\">-Element ergänzen.",
            recommendation_en: "Add an alt attribute to the <input type=\"image\"> element.",
            technical_note: "<input type=\"image\"> ohne alt-Attribut.",
            technical_note_en: "<input type=\"image\"> without an alt attribute.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "object-alt",
        RuleExplanation {
            customer_title: "Fehlende Textalternative bei eingebetteten Objekten",
            customer_title_en: "Missing text alternative on embedded objects",
            customer_description: "Ein <object>-Element (eingebettetes Bild/Dokument) hat keine Textalternative.",
            customer_description_en: "Missing text alternative on embedded objects. Screen reader users receive no information about the embedded object's content.",
            user_impact: "Screenreader-Nutzer erhalten keine Information über den Inhalt des eingebetteten Objekts.",
            user_impact_en: "Screen reader users receive no information about the embedded object's content.",
            typical_cause: "<object> ohne Textinhalt oder aria-label.",
            typical_cause_en: "<object> without text content or aria-label.",
            recommendation: "Eine Textalternative innerhalb des <object>-Elements oder via aria-label bereitstellen.",
            recommendation_en: "Provide a text alternative inside the <object> element or via aria-label.",
            technical_note: "<object> ohne Textinhalt oder aria-label.",
            technical_note_en: "<object> without text content or aria-label.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "definition-list",
        RuleExplanation {
            customer_title: "Fehlerhafte Definitionsliste",
            customer_title_en: "Malformed definition list",
            customer_description: "Eine Definitionsliste (<dl>) enthält Begriffe (<dt>) ohne zugehörige Definition (<dd>) oder umgekehrt.",
            customer_description_en: "Malformed definition list. Screen reader users cannot recognize the term and its definition as belonging together.",
            user_impact: "Screenreader-Nutzer können Begriff und Definition nicht als zusammengehörig erkennen.",
            user_impact_en: "Screen reader users cannot recognize the term and its definition as belonging together.",
            typical_cause: "<dt>/<dd>-Paare in der <dl> unvollständig oder falsch verschachtelt.",
            typical_cause_en: "Incomplete or incorrectly nested <dt>/<dd> pairs inside the <dl>.",
            recommendation: "<th>-Elemente für Tabellenüberschriften ergänzen, Listenelemente in <ul>/<ol> einbetten, oder zusammengehörige Radio-Buttons mit <fieldset> und <legend> gruppieren.",
            recommendation_en: "Add <th> elements for table headers, wrap list items in <ul>/<ol>, or group related radio buttons with <fieldset> and <legend>.",
            technical_note: "<dt>/<dd>-Paare in der <dl> unvollständig oder falsch verschachtelt.",
            technical_note_en: "Incomplete or incorrectly nested <dt>/<dd> pairs inside the <dl>.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "html-content-model",
        RuleExplanation {
            customer_title: "Verletzung des HTML5-Content-Models",
            customer_title_en: "HTML5 content model violation",
            customer_description: "Ein strukturelles Element (Liste, Tabelle, Formular, Select) enthält Inhalt, der sein HTML5-Content-Model verletzt — das kann Screenreader-Semantik brechen, selbst wenn der resultierende Accessibility-Tree oberflächlich gültig aussieht.",
            customer_description_en: "HTML5 content model violation. Screen readers cannot reliably parse the structure, even when it appears superficially valid.",
            user_impact: "Screenreader können die Struktur nicht zuverlässig auswerten, auch wenn sie augenscheinlich korrekt wirkt.",
            user_impact_en: "Screen readers cannot reliably parse the structure, even when it appears superficially valid.",
            typical_cause: "Unerwartetes Kindelement innerhalb eines strukturellen Containers (z. B. <ul>, <table>, <select>).",
            typical_cause_en: "Unexpected child element inside a structural container (e.g. <ul>, <table>, <select>).",
            recommendation: "Das Markup an das HTML5-Content-Model des Elements anpassen (z. B. darf ein <dl> nur direkt <dt>/<dd>-Gruppen, <div>, <script> oder <template> enthalten). Der Browser kann ungültige Kindelemente beim Aufbau des Accessibility-Trees still 'heilen' — der Defekt bleibt für rollenbasierte Prüfungen unsichtbar, verwirrt assistierende Technologie aber trotzdem.",
            recommendation_en: "Fix the markup so it matches the HTML5 content model for the element (e.g. a <dl> may only directly contain <dt>/<dd> groups, <div>, <script>, or <template>). Invalid children can be silently \"healed\" by the browser when building the accessibility tree, hiding the defect from role-based checks while still confusing assistive technology.",
            technical_note: "Unerwartetes Kindelement innerhalb eines strukturellen Containers (z. B. <ul>, <table>, <select>).",
            technical_note_en: "Unexpected child element inside a structural container (e.g. <ul>, <table>, <select>).",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "form-field-multiple-labels",
        RuleExplanation {
            customer_title: "Fehlende Gruppierung verwandter Formularfelder",
            customer_title_en: "Missing grouping for related form fields",
            customer_description: "Zusammengehörige Radio-Buttons oder Checkboxen sind nicht mit <fieldset>/<legend> oder role=\"group\" gruppiert.",
            customer_description_en: "Missing grouping for related form fields. Screen reader users cannot tell that several fields belong to one question/group.",
            user_impact: "Screenreader-Nutzer erkennen nicht, dass mehrere Felder zu einer Frage/Gruppe gehören.",
            user_impact_en: "Screen reader users cannot tell that several fields belong to one question/group.",
            typical_cause: "Radio-Buttons/Checkboxen ohne umschließendes <fieldset> mit <legend>.",
            typical_cause_en: "Radio buttons/checkboxes without an enclosing <fieldset> with <legend>.",
            recommendation: "Zusammengehörige Radio-Buttons oder Checkboxen in ein <fieldset> mit <legend> einbetten, oder role=\"group\" mit aria-labelledby verwenden.",
            recommendation_en: "Wrap related radio buttons or checkboxes in a <fieldset> with a <legend>, or use role=\"group\" with aria-labelledby.",
            technical_note: "Radio-Buttons/Checkboxen ohne umschließendes <fieldset> mit <legend>.",
            technical_note_en: "Radio buttons/checkboxes without an enclosing <fieldset> with <legend>.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "table-duplicate-name",
        RuleExplanation {
            customer_title: "Unvollständige Tabellenstruktur",
            customer_title_en: "Incomplete table structure",
            customer_description: "Eine Datentabelle hat keine <caption> oder keine Kopfzellen, oder eine rein präsentationelle Tabelle enthält fälschlich Kopfzellen.",
            customer_description_en: "Incomplete table structure. Screen reader users cannot associate table cells with their column/row headers.",
            user_impact: "Screenreader-Nutzer können Tabellenzellen nicht ihren Spalten-/Zeilenüberschriften zuordnen.",
            user_impact_en: "Screen reader users cannot associate table cells with their column/row headers.",
            typical_cause: "Fehlende <caption>, fehlende <th>-Elemente, oder Zellen außerhalb einer <tr>.",
            typical_cause_en: "Missing <caption>, missing <th> elements, or cells outside a <tr>.",
            recommendation: "Eine <caption> innerhalb der Tabelle (oder aria-label/aria-labelledby) sowie <th>-Elemente für Spalten-/Zeilenüberschriften ergänzen.",
            recommendation_en: "Add a <caption> element inside the table (or aria-label/aria-labelledby) and add <th> elements for column/row headers.",
            technical_note: "Fehlende <caption>, fehlende <th>-Elemente, oder Zellen außerhalb einer <tr>.",
            technical_note_en: "Missing <caption>, missing <th> elements, or cells outside a <tr>.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-banner-is-top-level",
        RuleExplanation {
            customer_title: "Banner-Landmark ist verschachtelt",
            customer_title_en: "Banner landmark is nested",
            customer_description: "Der Banner-Bereich (<header>/role=\"banner\") liegt innerhalb eines anderen Landmarks statt auf oberster Ebene.",
            customer_description_en: "Banner landmark is nested. Screen reader users who jump to the header via landmark navigation land in an unexpected context.",
            user_impact: "Screenreader-Nutzer, die per Landmark-Navigation zum Kopfbereich springen, landen in einem unerwarteten Kontext.",
            user_impact_en: "Screen reader users who jump to the header via landmark navigation land in an unexpected context.",
            typical_cause: "<header>/role=\"banner\" ist Nachfahre eines anderen Landmarks (z. B. <main>).",
            typical_cause_en: "<header>/role=\"banner\" is a descendant of another landmark (e.g. <main>).",
            recommendation: "Das <header>-/role=\"banner\"-Element auf die oberste Ebene der Seite verschieben, außerhalb jedes anderen Landmarks.",
            recommendation_en: "Move the <header> / role=\"banner\" element to the top level of the page, outside any other landmark.",
            technical_note: "<header>/role=\"banner\" ist Nachfahre eines anderen Landmarks (z. B. <main>).",
            technical_note_en: "<header>/role=\"banner\" is a descendant of another landmark (e.g. <main>).",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-contentinfo-is-top-level",
        RuleExplanation {
            customer_title: "Contentinfo-Landmark ist verschachtelt",
            customer_title_en: "Contentinfo landmark is nested",
            customer_description: "Der Fußbereich (<footer>/role=\"contentinfo\") liegt innerhalb eines anderen Landmarks statt auf oberster Ebene.",
            customer_description_en: "Contentinfo landmark is nested. Screen reader users who jump to the footer via landmark navigation land in an unexpected context.",
            user_impact: "Screenreader-Nutzer, die per Landmark-Navigation zum Fußbereich springen, landen in einem unerwarteten Kontext.",
            user_impact_en: "Screen reader users who jump to the footer via landmark navigation land in an unexpected context.",
            typical_cause: "<footer>/role=\"contentinfo\" ist Nachfahre eines anderen Landmarks.",
            typical_cause_en: "<footer>/role=\"contentinfo\" is a descendant of another landmark.",
            recommendation: "Das <footer>-/role=\"contentinfo\"-Element auf die oberste Ebene der Seite verschieben, außerhalb jedes anderen Landmarks.",
            recommendation_en: "Move the <footer> / role=\"contentinfo\" element to the top level of the page, outside any other landmark.",
            technical_note: "<footer>/role=\"contentinfo\" ist Nachfahre eines anderen Landmarks.",
            technical_note_en: "<footer>/role=\"contentinfo\" is a descendant of another landmark.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-main-is-top-level",
        RuleExplanation {
            customer_title: "Main-Landmark ist verschachtelt",
            customer_title_en: "Main landmark is nested",
            customer_description: "Der Hauptinhaltsbereich (<main>/role=\"main\") liegt innerhalb eines anderen Landmarks statt auf oberster Ebene.",
            customer_description_en: "Main landmark is nested. Screen reader users who jump to the main content via landmark navigation land in an unexpected context.",
            user_impact: "Screenreader-Nutzer, die per Landmark-Navigation zum Hauptinhalt springen, landen in einem unerwarteten Kontext.",
            user_impact_en: "Screen reader users who jump to the main content via landmark navigation land in an unexpected context.",
            typical_cause: "<main>/role=\"main\" ist Nachfahre eines anderen Landmarks.",
            typical_cause_en: "<main>/role=\"main\" is a descendant of another landmark.",
            recommendation: "Das <main>-/role=\"main\"-Element auf die oberste Ebene der Seite verschieben, außerhalb jedes anderen Landmarks.",
            recommendation_en: "Move the <main> / role=\"main\" element to the top level of the page, outside any other landmark.",
            technical_note: "<main>/role=\"main\" ist Nachfahre eines anderen Landmarks.",
            technical_note_en: "<main>/role=\"main\" is a descendant of another landmark.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-no-duplicate-banner",
        RuleExplanation {
            customer_title: "Mehrere Banner-Landmarks",
            customer_title_en: "Multiple banner landmarks",
            customer_description: "Die Seite hat mehr als einen Banner-Bereich (<header>/role=\"banner\"); erlaubt ist höchstens einer.",
            customer_description_en: "Multiple banner landmarks. Screen reader users cannot uniquely identify the page's 'header' region.",
            user_impact: "Screenreader-Nutzer können den 'Kopfbereich' der Seite nicht eindeutig zuordnen.",
            user_impact_en: "Screen reader users cannot uniquely identify the page's 'header' region.",
            typical_cause: "Mehr als ein Element mit role=\"banner\" (implizit oder explizit).",
            typical_cause_en: "More than one element with role=\"banner\" (implicit or explicit).",
            recommendation: "Sicherstellen, dass die Seite höchstens ein <header>-/role=\"banner\"-Element hat.",
            recommendation_en: "Ensure the page has at most one <header> / role=\"banner\" element.",
            technical_note: "Mehr als ein Element mit role=\"banner\" (implizit oder explizit).",
            technical_note_en: "More than one element with role=\"banner\" (implicit or explicit).",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-no-duplicate-contentinfo",
        RuleExplanation {
            customer_title: "Mehrere Contentinfo-Landmarks",
            customer_title_en: "Multiple contentinfo landmarks",
            customer_description: "Die Seite hat mehr als einen Fußbereich (<footer>/role=\"contentinfo\"); erlaubt ist höchstens einer.",
            customer_description_en: "Multiple contentinfo landmarks. Screen reader users cannot uniquely identify the page's 'footer' region.",
            user_impact: "Screenreader-Nutzer können den 'Fußbereich' der Seite nicht eindeutig zuordnen.",
            user_impact_en: "Screen reader users cannot uniquely identify the page's 'footer' region.",
            typical_cause: "Mehr als ein Element mit role=\"contentinfo\" (implizit oder explizit).",
            typical_cause_en: "More than one element with role=\"contentinfo\" (implicit or explicit).",
            recommendation: "Sicherstellen, dass die Seite höchstens ein <footer>-/role=\"contentinfo\"-Element hat.",
            recommendation_en: "Ensure the page has at most one <footer> / role=\"contentinfo\" element.",
            technical_note: "Mehr als ein Element mit role=\"contentinfo\" (implizit oder explizit).",
            technical_note_en: "More than one element with role=\"contentinfo\" (implicit or explicit).",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-no-duplicate-main",
        RuleExplanation {
            customer_title: "Mehrere Main-Landmarks",
            customer_title_en: "Multiple main landmarks",
            customer_description: "Die Seite hat mehr als einen Hauptinhaltsbereich (<main>/role=\"main\"); erlaubt ist genau einer.",
            customer_description_en: "Multiple main landmarks. Screen reader users who jump to the main content cannot tell which region is meant.",
            user_impact: "Screenreader-Nutzer, die zum Hauptinhalt springen, wissen nicht, welcher Bereich gemeint ist.",
            user_impact_en: "Screen reader users who jump to the main content cannot tell which region is meant.",
            typical_cause: "Mehr als ein Element mit role=\"main\" (implizit oder explizit).",
            typical_cause_en: "More than one element with role=\"main\" (implicit or explicit).",
            recommendation: "Sicherstellen, dass die Seite genau ein <main>-/role=\"main\"-Element hat.",
            recommendation_en: "Ensure the page has exactly one <main> / role=\"main\" element.",
            technical_note: "Mehr als ein Element mit role=\"main\" (implizit oder explizit).",
            technical_note_en: "More than one element with role=\"main\" (implicit or explicit).",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-banner-present",
        RuleExplanation {
            customer_title: "Fehlender Banner-Landmark",
            customer_title_en: "Missing banner landmark",
            customer_description: "Die Seite hat keinen Banner-Bereich (<header> bzw. role=\"banner\").",
            customer_description_en: "Missing banner landmark. Assistive technologies cannot identify the site header.",
            user_impact: "Assistierende Technologien können den Kopfbereich der Seite nicht identifizieren.",
            user_impact_en: "Assistive technologies cannot identify the site header.",
            typical_cause: "Kein Element mit role=\"banner\" (implizit über <header> oder explizit) vorhanden.",
            typical_cause_en: "No element with role=\"banner\" (implicit via <header> or explicit) present.",
            recommendation: "Ein <header>-Element auf oberster Ebene (oder ein Element mit role=\"banner\") ergänzen, das den Seitenkopf umschließt.",
            recommendation_en: "Add a top-level <header> element (or an element with role=\"banner\") that wraps the site header content.",
            technical_note: "Kein Element mit role=\"banner\" (implizit über <header> oder explizit) vorhanden.",
            technical_note_en: "No element with role=\"banner\" (implicit via <header> or explicit) present.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "list",
        RuleExplanation {
            customer_title: "Listenelemente ohne Listen-Container",
            customer_title_en: "List items outside a list container",
            customer_description: "Listenelemente (role=\"listitem\") liegen nicht innerhalb eines Listen-Containers (role=\"list\"), oder eine Liste ist leer.",
            customer_description_en: "List items outside a list container. Screen reader users receive no announcement of the list's length/structure.",
            user_impact: "Screenreader-Nutzer erhalten keine Ansage der Listenlänge/-struktur.",
            user_impact_en: "Screen reader users receive no announcement of the list's length/structure.",
            typical_cause: "<li> außerhalb von <ul>/<ol>, oder leere Liste ohne Listenelemente.",
            typical_cause_en: "<li> outside <ul>/<ol>, or an empty list with no list items.",
            recommendation: "Sicherstellen, dass Listenelemente (role=\"listitem\") innerhalb eines Listen-Containers (role=\"list\") liegen; leere Listen entfernen oder mit Listenelementen füllen.",
            recommendation_en: "Ensure list items (role=\"listitem\") are inside a list (role=\"list\") element; remove empty lists or add list items to them.",
            technical_note: "<li> außerhalb von <ul>/<ol>, oder leere Liste ohne Listenelemente.",
            technical_note_en: "<li> outside <ul>/<ol>, or an empty list with no list items.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "label-title-only",
        RuleExplanation {
            customer_title: "Formularfeld-Beschriftung nur über title-Attribut",
            customer_title_en: "Form field labeled only via the title attribute",
            customer_description: "Ein Formularfeld bezieht seinen zugänglichen Namen ausschließlich aus dem title-Attribut statt aus einem sichtbaren Label.",
            customer_description_en: "Form field labeled only via the title attribute. The title attribute is read inconsistently or not at all by many screen readers; sighted users only see it on hover.",
            user_impact: "Das title-Attribut wird von vielen Screenreadern inkonsistent oder gar nicht vorgelesen; sehende Nutzer sehen es nur bei Hover.",
            user_impact_en: "The title attribute is read inconsistently or not at all by many screen readers; sighted users only see it on hover.",
            typical_cause: "Formularelement ohne <label>/aria-label, Name kommt nur aus title.",
            typical_cause_en: "Form element without <label>/aria-label, name derived only from title.",
            recommendation: "Ein sichtbares <label>-Element oder aria-label statt des title-Attributs verwenden.",
            recommendation_en: "Add a visible <label> element or aria-label instead of relying on the title attribute.",
            technical_note: "Formularelement ohne <label>/aria-label, Name kommt nur aus title.",
            technical_note_en: "Form element without <label>/aria-label, name derived only from title.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "meta-viewport-large",
        RuleExplanation {
            customer_title: "Zoom auf 500% durch Viewport-Meta-Tag verhindert",
            customer_title_en: "Viewport meta tag prevents zooming to 500%",
            customer_description: "Das viewport-Meta-Tag begrenzt maximum-scale so stark, dass Nutzer nicht auf mindestens 500% zoomen können.",
            customer_description_en: "Viewport meta tag prevents zooming to 500%. Users with low vision cannot magnify the page content enough.",
            user_impact: "Sehbehinderte Nutzer können den Seiteninhalt nicht ausreichend vergrößern.",
            user_impact_en: "Users with low vision cannot magnify the page content enough.",
            typical_cause: "maximum-scale im viewport-Meta-Tag unter 5 gesetzt.",
            typical_cause_en: "maximum-scale in the viewport meta tag set below 5.",
            recommendation: "maximum-scale im viewport-Meta-Tag auf mindestens 5 setzen oder die maximum-scale-Beschränkung ganz entfernen.",
            recommendation_en: "Set maximum-scale to at least 5 in the viewport meta tag, or remove the maximum-scale restriction entirely.",
            technical_note: "maximum-scale im viewport-Meta-Tag unter 5 gesetzt.",
            technical_note_en: "maximum-scale in the viewport meta tag set below 5.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "click-events-have-key-events",
        RuleExplanation {
            customer_title: "Klick-Handler ohne Tastatur-Äquivalent",
            customer_title_en: "Click handler without keyboard equivalent",
            customer_description: "Ein nicht-interaktives Element (z. B. <div>) hat einen Klick-Handler, aber keinen entsprechenden Tastatur-Event-Handler.",
            customer_description_en: "Click handler without keyboard equivalent. Keyboard users cannot trigger the function.",
            user_impact: "Tastaturnutzer können die Funktion nicht auslösen.",
            user_impact_en: "Keyboard users cannot trigger the function.",
            typical_cause: "onclick auf einem <div>/<span> ohne onkeydown/onkeyup, tabindex oder passende Rolle.",
            typical_cause_en: "onclick on a <div>/<span> without onkeydown/onkeyup, tabindex, or a matching role.",
            recommendation: "Einen keydown/keyup-Handler ergänzen, der auf Enter/Leertaste reagiert, tabindex=\"0\" und eine passende ARIA-Rolle setzen, oder stattdessen ein natives <button>-Element verwenden.",
            recommendation_en: "Add a keydown/keyup handler that responds to Enter/Space, add tabindex=\"0\" and an appropriate ARIA role, or use a native <button> element instead.",
            technical_note: "onclick auf einem <div>/<span> ohne onkeydown/onkeyup, tabindex oder passende Rolle.",
            technical_note_en: "onclick on a <div>/<span> without onkeydown/onkeyup, tabindex, or a matching role.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-one-main",
        RuleExplanation {
            customer_title: "Kein Hauptinhaltsbereich als Sprungziel",
            customer_title_en: "No main-content region as a bypass target",
            customer_description: "Die Seite hat keinen <main>-Bereich, der als Sprungziel zum Überspringen wiederkehrender Blöcke dienen könnte.",
            customer_description_en: "No main-content region as a bypass target. Keyboard users have no reliable target to bypass repeated navigation blocks.",
            user_impact: "Tastaturnutzer haben kein verlässliches Ziel, um wiederkehrende Navigation zu überspringen.",
            user_impact_en: "Keyboard users have no reliable target to bypass repeated navigation blocks.",
            typical_cause: "Kein Element mit role=\"main\" (implizit über <main> oder explizit) vorhanden.",
            typical_cause_en: "No element with role=\"main\" (implicit via <main> or explicit) present.",
            recommendation: "Ein <main>-Element oder ein Element mit role=\"main\" ergänzen, damit Tastaturnutzer ein verlässliches Sprungziel haben.",
            recommendation_en: "Add a <main> element or an element with role=\"main\" so keyboard users have a reliable bypass target.",
            technical_note: "Kein Element mit role=\"main\" (implizit über <main> oder explizit) vorhanden.",
            technical_note_en: "No element with role=\"main\" (implicit via <main> or explicit) present.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "skip-link",
        RuleExplanation {
            customer_title: "Fehlender Skip-Link",
            customer_title_en: "Missing skip link",
            customer_description: "Die Seite hat eine Navigation, aber keinen Skip-Link, um sie zu überspringen und direkt zum Inhalt zu springen.",
            customer_description_en: "Missing skip link. Keyboard users must tab through the entire navigation on every page change.",
            user_impact: "Tastaturnutzer müssen bei jedem Seitenwechsel durch die gesamte Navigation tabben.",
            user_impact_en: "Keyboard users must tab through the entire navigation on every page change.",
            typical_cause: "Kein Link mit erkennbarem 'skip'/'zum Inhalt'-Text oder Anker auf #main/#content am Seitenanfang.",
            typical_cause_en: "No link with a recognizable 'skip'/'to content' text or anchor to #main/#content at the top of the page.",
            recommendation: "Einen visuell versteckten oder sichtbaren Link am Seitenanfang ergänzen, der auf den Hauptinhalt verweist, z. B. <a href=\"#main\">Zum Inhalt springen</a>.",
            recommendation_en: "Add a visually hidden or visible link at the top of the page pointing to the main content anchor, e.g. <a href=\"#main\">Skip to main content</a>.",
            technical_note: "Kein Link mit erkennbarem 'skip'/'zum Inhalt'-Text oder Anker auf #main/#content am Seitenanfang.",
            technical_note_en: "No link with a recognizable 'skip'/'to content' text or anchor to #main/#content at the top of the page.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "focus-visible-outline-none",
        RuleExplanation {
            customer_title: "Fokus-Indikator per CSS entfernt",
            customer_title_en: "Focus indicator removed via CSS",
            customer_description: "CSS entfernt den sichtbaren Fokusrahmen (outline: none/0) ohne einen gleichwertigen Ersatz bereitzustellen.",
            customer_description_en: "Focus indicator removed via CSS. Keyboard users cannot see which element currently has focus.",
            user_impact: "Tastaturnutzer sehen nicht, welches Element gerade fokussiert ist.",
            user_impact_en: "Keyboard users cannot see which element currently has focus.",
            typical_cause: "outline: none/0 in :focus-Regeln ohne alternative sichtbare Fokusmarkierung.",
            typical_cause_en: "outline: none/0 in :focus rules without an alternative visible focus indicator.",
            recommendation: "Den Standard-Fokusrahmen (outline: none/0) nicht entfernen, ohne einen gleichwertigen sichtbaren Ersatz bereitzustellen, z. B. einen eigenen Outline/Box-Shadow auf :focus-visible.",
            recommendation_en: "Do not remove the default focus outline (outline: none/0) without providing an equally visible replacement, e.g. a custom outline or box-shadow on :focus-visible.",
            technical_note: "outline: none/0 in :focus-Regeln ohne alternative sichtbare Fokusmarkierung.",
            technical_note_en: "outline: none/0 in :focus rules without an alternative visible focus indicator.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "input-error-message",
        RuleExplanation {
            customer_title: "Ungültiges Feld ohne zugängliche Fehlerbeschreibung",
            customer_title_en: "Invalid field without an accessible error description",
            customer_description: "Ein als ungültig markiertes Formularfeld hat keine zugängliche Beschreibung des Fehlers.",
            customer_description_en: "Invalid field without an accessible error description. Screen reader users learn that an error exists, but not what it is.",
            user_impact: "Screenreader-Nutzer erfahren, dass ein Fehler vorliegt, aber nicht welcher.",
            user_impact_en: "Screen reader users learn that an error exists, but not what it is.",
            typical_cause: "aria-invalid=\"true\" ohne description/aria-describedby auf ein sichtbares Fehlerelement.",
            typical_cause_en: "aria-invalid=\"true\" without a description/aria-describedby pointing to a visible error element.",
            recommendation: "aria-describedby auf ein Element mit der Fehlermeldung setzen, oder aria-errormessage verwenden.",
            recommendation_en: "Add aria-describedby pointing to an element containing the error message, or use aria-errormessage.",
            technical_note: "aria-invalid=\"true\" ohne description/aria-describedby auf ein sichtbares Fehlerelement.",
            technical_note_en: "aria-invalid=\"true\" without a description/aria-describedby pointing to a visible error element.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "duplicate-id-aria",
        RuleExplanation {
            customer_title: "ARIA-Referenz zeigt auf mehrdeutige ID",
            customer_title_en: "ARIA reference points to an ambiguous ID",
            customer_description: "Ein ARIA-Attribut wie aria-owns/aria-describedby/aria-labelledby verweist auf eine ID, die im Dokument mehrfach vergeben ist.",
            customer_description_en: "ARIA reference points to an ambiguous ID. Assistive technologies cannot reliably determine which element is referenced.",
            user_impact: "Assistierende Technologien können nicht zuverlässig bestimmen, welches Element gemeint ist.",
            user_impact_en: "Assistive technologies cannot reliably determine which element is referenced.",
            typical_cause: "Referenzierte ID kommt im DOM mehr als einmal vor.",
            typical_cause_en: "The referenced ID occurs more than once in the DOM.",
            recommendation: "Sicherstellen, dass die von aria-owns/aria-describedby/aria-labelledby referenzierte ID im Dokument eindeutig ist, oder ein anderes, eindeutig identifiziertes Element referenzieren.",
            recommendation_en: "Ensure the ID referenced by aria-owns/aria-describedby/aria-labelledby is unique in the document, or reference a different, uniquely identified element.",
            technical_note: "Referenzierte ID kommt im DOM mehr als einmal vor.",
            technical_note_en: "The referenced ID occurs more than once in the DOM.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "aria-label",
        RuleExplanation {
            customer_title: "Interaktives Element ohne aussagekräftigen Namen",
            customer_title_en: "Interactive element without a meaningful name",
            customer_description: "Ein interaktives Element hat keinen zugänglichen Namen, oder der Name besteht nur aus einem Icon/Symbol ohne Beschreibung.",
            customer_description_en: "Interactive element without a meaningful name. Screen reader users are not told what the element does.",
            user_impact: "Screenreader-Nutzer erfahren nicht, wozu das Element dient.",
            user_impact_en: "Screen reader users are not told what the element does.",
            typical_cause: "Fehlendes oder rein symbolisches aria-label/aria-labelledby/Textinhalt.",
            typical_cause_en: "Missing or purely symbolic aria-label/aria-labelledby/text content.",
            recommendation: "aria-label, aria-labelledby oder sichtbaren Textinhalt ergänzen, der den Zweck des Elements beschreibt (nicht nur sein Icon).",
            recommendation_en: "Add aria-label, aria-labelledby, or visible text content that describes the element's purpose (not just its icon).",
            technical_note: "Fehlendes oder rein symbolisches aria-label/aria-labelledby/Textinhalt.",
            technical_note_en: "Missing or purely symbolic aria-label/aria-labelledby/text content.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "aria-required-children",
        RuleExplanation {
            customer_title: "Erforderliche Kindelemente einer ARIA-Rolle fehlen",
            customer_title_en: "Required child elements of an ARIA role are missing",
            customer_description: "Ein Element mit einer Composite-ARIA-Rolle (z. B. tablist) fehlt eines der laut ARIA-Spezifikation erforderlichen Kindelemente (z. B. zugehörige tabpanels).",
            customer_description_en: "Required child elements of an ARIA role are missing. Assistive technologies cannot resolve the relationship between the widget's parts.",
            user_impact: "Assistierende Technologien können die Beziehung zwischen den Widget-Teilen nicht auswerten.",
            user_impact_en: "Assistive technologies cannot resolve the relationship between the widget's parts.",
            typical_cause: "z. B. role=\"tablist\" ohne zugehörige role=\"tabpanel\"-Elemente.",
            typical_cause_en: "e.g. role=\"tablist\" without corresponding role=\"tabpanel\" elements.",
            recommendation: "Die laut ARIA-Rolle erforderlichen Kindelemente ergänzen, z. B. role=\"tabpanel\"-Elemente für jeden Tab in einem role=\"tablist\".",
            recommendation_en: "Add the child elements required by the ARIA role, e.g. role=\"tabpanel\" elements for every tab in a role=\"tablist\".",
            technical_note: "z. B. role=\"tablist\" ohne zugehörige role=\"tabpanel\"-Elemente.",
            technical_note_en: "e.g. role=\"tablist\" without corresponding role=\"tabpanel\" elements.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "dialog-name",
        RuleExplanation {
            customer_title: "Dialog ohne zugänglichen Namen",
            customer_title_en: "Dialog without an accessible name",
            customer_description: "Ein Dialog (role=\"dialog\"/\"alertdialog\") hat keinen zugänglichen Namen.",
            customer_description_en: "Dialog without an accessible name. Screen reader users are not told what the dialog is about when it opens.",
            user_impact: "Screenreader-Nutzer erfahren beim Öffnen des Dialogs nicht, worum es geht.",
            user_impact_en: "Screen reader users are not told what the dialog is about when it opens.",
            typical_cause: "Dialog-Element ohne aria-label oder aria-labelledby auf den Dialogtitel.",
            typical_cause_en: "Dialog element without aria-label or aria-labelledby pointing to the dialog title.",
            recommendation: "aria-labelledby auf den sichtbaren Dialogtitel setzen, oder aria-label verwenden.",
            recommendation_en: "Add aria-labelledby pointing to the dialog's visible title, or use aria-label.",
            technical_note: "Dialog-Element ohne aria-label oder aria-labelledby auf den Dialogtitel.",
            technical_note_en: "Dialog element without aria-label or aria-labelledby pointing to the dialog title.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "control-missing-label",
        RuleExplanation {
            customer_title: "Formularelement, Link oder Button ohne zugänglichen Namen",
            customer_title_en: "Form control, link, or button without an accessible name",
            customer_description: "Ein Formularfeld, Link oder Button hat keinen zugänglichen Namen (weder sichtbares Label noch aria-label/aria-labelledby).",
            customer_description_en: "Form control, link, or button without an accessible name. Screen reader users are not told what input is expected or where a link/button leads.",
            user_impact: "Screenreader-Nutzer erfahren nicht, welche Eingabe erwartet wird oder wohin ein Link/Button führt.",
            user_impact_en: "Screen reader users are not told what input is expected or where a link/button leads.",
            typical_cause: "Formularelement/Link/Button ohne <label>, aria-label, aria-labelledby oder Textinhalt.",
            typical_cause_en: "Form control/link/button without <label>, aria-label, aria-labelledby, or text content.",
            recommendation: "Ein <label>-Element mit passendem for-Attribut (Formularfelder) bzw. Textinhalt (Links/Buttons) ergänzen, oder aria-label/aria-labelledby verwenden.",
            recommendation_en: "Add a <label> element with a matching 'for' attribute (form controls) or text content (links/buttons), or use aria-label/aria-labelledby.",
            technical_note: "Formularelement/Link/Button ohne <label>, aria-label, aria-labelledby oder Textinhalt.",
            technical_note_en: "Form control/link/button without <label>, aria-label, aria-labelledby, or text content.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "link-as-button",
        RuleExplanation {
            customer_title: "Link als Button-Ersatz ohne Tastenrolle",
            customer_title_en: "Link used as a button substitute without a matching role",
            customer_description: "Ein Link (<a>) mit href=\"#\" oder javascript:-Pseudo-URL und onclick-Handler wird wie ein Button verwendet, ohne role=\"button\" zu tragen.",
            customer_description_en: "Link used as a button substitute without a matching role. Assistive technologies announce the element as a link even though it behaves like a button — keyboard users expect the wrong interaction.",
            user_impact: "Assistierende Technologien kündigen das Element als Link an, obwohl es sich wie ein Button verhält — Tastaturnutzer erwarten das falsche Bedienverhalten.",
            user_impact_en: "Assistive technologies announce the element as a link even though it behaves like a button — keyboard users expect the wrong interaction.",
            typical_cause: "<a href=\"#\">/<a href=\"javascript:...\"> mit onclick, ohne role=\"button\".",
            typical_cause_en: "<a href=\"#\">/<a href=\"javascript:...\"> with onclick, without role=\"button\".",
            recommendation: "role=\"button\" auf den Link setzen (plus Tastaturbehandlung für die Leertaste), oder stattdessen ein echtes <button>-Element verwenden.",
            recommendation_en: "Add role=\"button\" to the link (and keyboard handling for Space), or use a real <button> element instead.",
            technical_note: "<a href=\"#\">/<a href=\"javascript:...\"> mit onclick, ohne role=\"button\".",
            technical_note_en: "<a href=\"#\">/<a href=\"javascript:...\"> with onclick, without role=\"button\".",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "redundant-role",
        RuleExplanation {
            customer_title: "Redundante ARIA-Rolle",
            customer_title_en: "Redundant ARIA role",
            customer_description: "Ein Element trägt ein role-Attribut, das exakt der bereits impliziten Rolle des HTML-Elements entspricht (z. B. <button role=\"button\">).",
            customer_description_en: "Redundant ARIA role. No immediate user harm, but an unnecessary risk of inconsistency in future changes.",
            user_impact: "Kein unmittelbarer Nutzerschaden, aber unnötiges Risiko für Inkonsistenzen bei zukünftigen Änderungen.",
            user_impact_en: "No immediate user harm, but an unnecessary risk of inconsistency in future changes.",
            typical_cause: "role-Attribut dupliziert die native implizite Rolle des Tags.",
            typical_cause_en: "role attribute duplicates the tag's native implicit role.",
            recommendation: "Das redundante role-Attribut entfernen; das native HTML-Element bringt diese Rolle bereits implizit mit.",
            recommendation_en: "Remove the redundant role attribute; the native HTML element already exposes this role implicitly.",
            technical_note: "role-Attribut dupliziert die native implizite Rolle des Tags.",
            technical_note_en: "role attribute duplicates the tag's native implicit role.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "summary-name",
        RuleExplanation {
            customer_title: "Summary-Element ohne zugänglichen Namen",
            customer_title_en: "Summary element without an accessible name",
            customer_description: "Ein <summary>-Element (Aufklapp-Auslöser in <details>) hat keinen zugänglichen Namen.",
            customer_description_en: "Summary element without an accessible name. Screen reader users are not told what the disclosure section contains before opening it.",
            user_impact: "Screenreader-Nutzer erfahren nicht, was der Aufklapp-Bereich enthält, bevor sie ihn öffnen.",
            user_impact_en: "Screen reader users are not told what the disclosure section contains before opening it.",
            typical_cause: "<summary> ohne Textinhalt oder aria-label.",
            typical_cause_en: "<summary> without text content or aria-label.",
            recommendation: "Textinhalt im <summary>-Element ergänzen, der beschreibt, was der Aufklapp-Bereich enthält.",
            recommendation_en: "Add text content to the <summary> element describing what the disclosure section contains.",
            technical_note: "<summary> ohne Textinhalt oder aria-label.",
            technical_note_en: "<summary> without text content or aria-label.",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression for plan 32: several distinct checks share WCAG 1.3.1, so
    /// resolving a landmark finding by `rule_id` alone returns the generic
    /// "missing semantic structure" explanation written for tables and lists.
    /// The axe-id override must win. Confirmed live on the 2026-09-19
    /// inros-lackner report, where `detail.fix_guidance` published three
    /// landmark rules as "Missing semantic structure" with `<table><thead>`
    /// as their fix example.
    #[test]
    fn axe_id_override_wins_over_shared_wcag_criterion() {
        let generic = get_explanation("1.3.1").expect("WCAG 1.3.1 explanation");

        for (axe_id, rule_id) in [
            ("landmark-unique", "a11y.landmark_unique.invalid"),
            ("region", "a11y.landmark_region.missing"),
            (
                "landmark-banner-is-top-level",
                "a11y.landmark_banner.missing",
            ),
        ] {
            // Precondition: the bug this guards against is only meaningful
            // while the rule_id really does fall through to the generic entry.
            assert_eq!(
                get_explanation(rule_id).map(|e| e.customer_title),
                Some(generic.customer_title),
                "{rule_id}: expected rule_id-only lookup to fall through to 1.3.1",
            );

            let resolved = resolve_explanation(Some(axe_id), rule_id, "1.3.1")
                .unwrap_or_else(|| panic!("no explanation resolved for {axe_id}"));
            assert_ne!(
                resolved.customer_title, generic.customer_title,
                "{axe_id}: resolved to the generic 1.3.1 explanation instead of its own",
            );
        }
    }

    /// The fallback chain still has to work for findings without an axe id,
    /// and for axe ids that have no override of their own.
    #[test]
    fn resolve_explanation_falls_back_past_missing_axe_id() {
        let by_rule = resolve_explanation(None, "a11y.aria_hidden_focus.invalid", "4.1.2");
        assert!(by_rule.is_some(), "rule_id fallback lost");

        let unknown_axe = resolve_explanation(
            Some("no-such-axe-rule"),
            "a11y.aria_hidden_focus.invalid",
            "4.1.2",
        );
        assert_eq!(
            unknown_axe.map(|e| e.customer_title),
            by_rule.map(|e| e.customer_title),
            "an unknown axe id must not shadow the rule_id match",
        );

        let by_criterion = resolve_explanation(None, "a11y.no.such.rule", "1.1.1");
        assert!(by_criterion.is_some(), "wcag_criterion fallback lost");
    }

    /// Regression for #357: these rules carry their explanation under the
    /// taxonomy key, so a lookup must resolve them and return a localized
    /// (non-empty, distinct DE/EN) recommendation rather than a raw English fix.
    #[test]
    fn taxonomy_keyed_rules_have_localized_recommendation() {
        for rule_id in [
            "a11y.aria_hidden_focus.invalid",
            "a11y.aria_prohibited_attr.invalid",
            "a11y.modern_attributes.invalid",
            "a11y.parsing.invalid",
            // Regression (2026-08-31/09-01): keyboard.rs's NotTestable
            // finding had no explanations.rs entry at all, so the PDF fell
            // back to the raw English Violation.fix_suggestion unlocalized
            // -- confirmed leaking into German reports.
            "keyboard-trap",
        ] {
            let expl = get_explanation(rule_id)
                .unwrap_or_else(|| panic!("missing explanation for {rule_id}"));
            let de = expl.recommendation_for("de");
            let en = expl.recommendation_for("en");
            assert!(!de.is_empty(), "{rule_id}: empty DE recommendation");
            assert!(!en.is_empty(), "{rule_id}: empty EN recommendation");
            assert_ne!(de, en, "{rule_id}: DE and EN recommendation identical");
        }
    }

    /// Code examples are single-source and land in the canonical-English JSON
    /// (`fix_guidance[].code_example`), so they must not contain German text
    /// (#406). Guards against re-introducing leaks like "Menü öffnen".
    #[test]
    fn code_examples_are_canonical_english() {
        let has_umlaut = |s: &str| s.chars().any(|c| "äöüßÄÖÜ".contains(c));
        // An umlaut check alone is not enough: plenty of German slipped
        // through as umlaut-free words ("Hauptnavigation", "Kontakt",
        // "Hilfetext", "Zum Inhalt") and only surfaced in the canonical
        // English JSON once plan 32 made those overrides reachable from the
        // JSON path. Case-insensitive substrings, so "kontakt" in a href is
        // caught together with "Kontakt" in link text.
        const GERMAN_MARKERS: &[&str] = &[
            "haupt",
            "kontakt",
            "hilfe",
            "zum inhalt",
            "leistungen",
            "mehr erfahren",
            "mehr info",
            "seite",
            "suche",
            "startseite",
            "impressum",
            "datenschutz",
            "schließen",
            "absenden",
            "anmelden",
            "weiterlesen",
        ];
        for (rule_id, expl) in EXPLANATIONS {
            for example in [expl.example_bad, expl.example_good, expl.example_decorative]
                .into_iter()
                .flatten()
            {
                assert!(
                    !has_umlaut(example),
                    "{rule_id}: code example contains German text: {example}"
                );
                let lower = example.to_lowercase();
                for marker in GERMAN_MARKERS {
                    assert!(
                        !lower.contains(marker),
                        "{rule_id}: code example contains German word '{marker}': {example}"
                    );
                }
            }
        }
    }

    /// Regression for #571 (example 1): a `region.rs` finding (axe_id
    /// "region", content outside a landmark) must resolve to its own
    /// landmark-specific explanation, not the generic 1.3.1 fallback written
    /// for tables/lists/fieldsets.
    #[test]
    fn region_finding_gets_landmark_explanation_not_table_fallback() {
        let expl = get_explanation("region").expect("missing explanation for 'region'");
        let de = expl.recommendation_for("de");
        let en = expl.recommendation_for("en");
        assert!(
            de.contains("Landmark") || de.contains("landmark"),
            "region explanation should mention landmarks: {de}"
        );
        for text in [de, en] {
            assert!(
                !text.contains("<table>") && !text.contains("<fieldset>"),
                "region explanation still reads like the generic table/list/form fallback: {text}"
            );
        }
    }

    /// Regression for #571 (example 2): `keyboard.rs`'s "focusable without
    /// interactive role" finding (axe_id "focusable-no-role") must resolve
    /// to an explanation recommending an appropriate role, not the generic
    /// 2.1.1 fallback that suggests adding tabindex + a keydown handler to
    /// an arbitrary container.
    #[test]
    fn focusable_no_role_finding_recommends_a_role_not_tabindex_keydown() {
        let expl = get_explanation("focusable-no-role")
            .expect("missing explanation for 'focusable-no-role'");
        let de = expl.recommendation_for("de");
        let en = expl.recommendation_for("en");
        assert!(
            de.contains("Rolle") && en.contains("role"),
            "focusable-no-role explanation should recommend adding a role: de={de} en={en}"
        );
        for text in [de, en] {
            assert!(
                !text.to_lowercase().contains("keydown"),
                "focusable-no-role explanation should not unconditionally push keydown handling: {text}"
            );
        }
    }

    /// Regression for #571 (example 3): `link_purpose.rs`'s generic-link-text-
    /// with-context finding (axe_id "link-name-context") must resolve to an
    /// explanation that asks for manual verification (WCAG 2.4.4's context
    /// allowance), not the generic 2.4.4 fallback's unconditional rewrite
    /// instruction.
    #[test]
    fn link_purpose_context_finding_asks_for_verification() {
        let expl = get_explanation("link-name-context")
            .expect("missing explanation for 'link-name-context'");
        let de = expl.recommendation_for("de");
        let en = expl.recommendation_for("en");
        assert!(
            de.contains("prüfen") && en.to_lowercase().contains("verify"),
            "link-name-context explanation should ask for manual verification: de={de} en={en}"
        );
    }
}
