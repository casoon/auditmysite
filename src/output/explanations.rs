//! Customer-facing explanations for WCAG rules
//!
//! Maps technical WCAG rule IDs to human-readable explanations in German and English,
//! suitable for non-technical stakeholders.
//!
//! The prose of an explanation (title, description, user impact, typical
//! cause, recommendation, technical note) is not in this table: it lives in
//! `locales/<lang>/rules.ftl` and is read through the text methods of
//! [`RuleExplanation`], under the key [`explanation_text_key`] (#935). An entry
//! whose key is a taxonomy rule id has no message of its own: its texts are
//! attributes of the rule's message (`rule-<id>`), and its title is the rule's
//! title. Which entry a rule id uses is `RULE_ENTRIES`, an explicit table.

use crate::output::report_model::{Effort, ExampleBlock, Role};

/// Complete explanation for a WCAG rule. The texts are bilingual (German
/// default, English alternative) and come from `locales/<lang>/rules.ftl`.
pub struct RuleExplanation {
    /// Key of the entry in the explanation table (WCAG ID, axe ID or taxonomy
    /// rule ID); [`explanation_text_key`] turns it into the Fluent message.
    pub key: &'static str,
    /// Primary responsible role
    pub responsible_role: Role,
    /// Estimated effort to fix
    pub effort_estimate: Effort,
    /// Optional code examples (bad, good, decorative)
    pub example_bad: Option<&'static str>,
    pub example_good: Option<&'static str>,
    pub example_decorative: Option<&'static str>,
}

/// The Fluent message of an explanation's texts in `locales/<lang>/rules.ftl`.
/// For an entry keyed by a taxonomy rule id it is the rule's message
/// ([`crate::taxonomy::rules::rule_text_key`]); for an entry keyed by a WCAG
/// criterion number or an axe id it is `explanation-` plus the table key, with
/// `.`, `/` and `_` written as `-` (a Fluent identifier allows only letters,
/// digits, `_` and `-`).
pub fn explanation_text_key(key: &str) -> String {
    if is_rule_entry(key) {
        crate::taxonomy::rules::rule_text_key(key)
    } else {
        format!("explanation-{}", key.replace(['.', '/', '_'], "-"))
    }
}

/// Whether the entry is keyed by a taxonomy rule id.
fn is_rule_entry(key: &str) -> bool {
    crate::taxonomy::RuleLookup::by_id(key).is_some()
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

    /// Customer-facing title (`en = true` canonical English, as in the JSON;
    /// otherwise German). For an entry keyed by a rule id it is the rule's
    /// title.
    pub fn customer_title(&self, en: bool) -> String {
        crate::i18n::text(&explanation_text_key(self.key), en)
    }

    /// Layperson description of the issue (`en = true` canonical English,
    /// otherwise German).
    pub fn customer_description(&self, en: bool) -> String {
        crate::i18n::attr_text(&explanation_text_key(self.key), "description", en)
    }

    /// Who is affected and how (`en = true` canonical English, otherwise
    /// German).
    pub fn user_impact(&self, en: bool) -> String {
        // On a rule message `.user-impact` is the shorter catalogue text.
        let attribute = if is_rule_entry(self.key) {
            "explanation-user-impact"
        } else {
            "user-impact"
        };
        crate::i18n::attr_text(&explanation_text_key(self.key), attribute, en)
    }

    /// Why this typically happens (`en = true` canonical English, otherwise
    /// German).
    pub fn typical_cause(&self, en: bool) -> String {
        crate::i18n::attr_text(&explanation_text_key(self.key), "typical-cause", en)
    }

    /// Recommendation in customer language (`en = true` canonical English,
    /// otherwise German).
    pub fn recommendation(&self, en: bool) -> String {
        crate::i18n::attr_text(&explanation_text_key(self.key), "recommendation", en)
    }

    /// Technical note for developers (`en = true` canonical English,
    /// otherwise German).
    pub fn technical_note(&self, en: bool) -> String {
        crate::i18n::attr_text(&explanation_text_key(self.key), "technical-note", en)
    }
}

/// All rule explanations, keyed by WCAG ID, axe ID or taxonomy rule ID — for review
/// surfaces that need to enumerate every customer text (e.g.
/// `export_all_interpretations`). Test-only: no production caller needs the
/// full list.
#[cfg(test)]
pub(crate) fn all() -> &'static [(&'static str, RuleExplanation)] {
    EXPLANATIONS
}

/// Look up an explanation by a taxonomy rule id (e.g. "images/alt-missing"),
/// an axe id or a WCAG criterion number (e.g. "1.1.1").
///
/// A key of `EXPLANATIONS` finds its own entry; a taxonomy rule id finds the
/// entry `RULE_ENTRIES` lists for it, and a rule that is not listed has no
/// explanation. Nothing else is derived: in particular a rule does not fall
/// back to the entry of its WCAG criterion unless the table says so.
pub fn get_explanation(rule_id: &str) -> Option<&'static RuleExplanation> {
    let key = RULE_ENTRIES
        .iter()
        .find(|(id, _)| *id == rule_id)
        .map_or(rule_id, |(_, key)| *key);
    EXPLANATIONS
        .iter()
        .find(|(id, _)| *id == key)
        .map(|(_, e)| e)
}

/// Resolve a finding's explanation in the only order that is safe: the
/// finding's own axe id first, then the taxonomy rule id, then the WCAG
/// criterion.
///
/// Several distinct checks share one taxonomy rule id or WCAG criterion — the
/// many "1.3.1" checks (landmarks, lists, tables, form groups) are the usual
/// case. A rule id whose `RULE_ENTRIES` row lists a criterion entry, like the
/// criterion itself, returns that shared text, so looking up by rule id or
/// criterion *alone* silently returns an explanation written for a different
/// check (#571). The rule-specific overrides that prevent this are keyed by
/// axe id (e.g. "landmark-unique"), and are only reachable when the axe id is
/// tried first.
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

/// How a taxonomy rule id reaches its explanation: for every rule that has
/// one, the key of the entry in `EXPLANATIONS` it uses. This table is the
/// only way a rule id resolves to an entry; nothing derives an entry from the
/// rule's WCAG reference or its text. A rule that is not listed has no
/// explanation. `RULE_ENTRIES` and `taxonomy::RULES` are held against each
/// other by the tests below.
static RULE_ENTRIES: &[(&str, &str)] = &[
    // Own entry: the rule id is the entry key, and the entry's texts are
    // attributes of the rule's message in `rules.ftl`.
    ("images/alt-suspicious", "images/alt-suspicious"),
    ("orientation/root-rotated", "orientation/root-rotated"),
    ("keyboard-trap", "keyboard-trap"),
    ("focus/obscured", "focus/obscured"),
    ("focus/obscured-enhanced", "focus/obscured-enhanced"),
    (
        "links/new-window-unannounced",
        "links/new-window-unannounced",
    ),
    ("auth/paste-blocked", "auth/paste-blocked"),
    ("auth/captcha", "auth/captcha"),
    ("forms/no-submit", "forms/no-submit"),
    ("language/part-unmarked", "language/part-unmarked"),
    ("zoom/viewport-locked", "zoom/viewport-locked"),
    ("zoom/viewport-missing", "zoom/viewport-missing"),
    ("media/audio-autoplay", "media/audio-autoplay"),
    ("keyboard/hidden-focusable", "keyboard/hidden-focusable"),
    (
        "aria/required-attribute-missing",
        "aria/required-attribute-missing",
    ),
    (
        "aria/required-parent-missing",
        "aria/required-parent-missing",
    ),
    ("modern-attribute-misuse", "modern-attribute-misuse"),
    ("aria/attribute-prohibited", "aria/attribute-prohibited"),
    ("aria/attribute-not-allowed", "aria/attribute-not-allowed"),
    (
        "aria/attribute-value-invalid",
        "aria/attribute-value-invalid",
    ),
    ("aria/reference-missing", "aria/reference-missing"),
    ("aria/role-invalid", "aria/role-invalid"),
    ("aria/attribute-unknown", "aria/attribute-unknown"),
    ("document/lang-mismatch", "document/lang-mismatch"),
    ("tables/headers-attr-invalid", "tables/headers-attr-invalid"),
    ("tables/header-without-data", "tables/header-without-data"),
    ("aria/tab-selected-missing", "aria/tab-selected-missing"),
    (
        "patterns/accordion-controls-missing",
        "patterns/accordion-controls-missing",
    ),
    ("dialog/modal-unmarked", "dialog/modal-unmarked"),
    ("aria/combobox-popup-missing", "aria/combobox-popup-missing"),
    ("frames/name-missing", "frames/name-missing"),
    ("frames/cross-origin", "frames/cross-origin"),
    ("ids/duplicate", "ids/duplicate"),
    ("html-content-model", "html-content-model"),
    ("landmarks/not-top-level", "landmarks/not-top-level"),
    ("lists/item-outside-list", "lists/item-outside-list"),
    ("headings/skip-level", "headings/skip-level"),
    ("headings/empty", "headings/empty"),
    ("headings/h1-missing", "headings/h1-missing"),
    ("headings/h1-multiple", "headings/h1-multiple"),
    ("lists/invalid-structure", "lists/invalid-structure"),
    ("lists/empty", "lists/empty"),
    (
        "lists/term-without-definition",
        "lists/term-without-definition",
    ),
    ("tables/header-missing", "tables/header-missing"),
    ("tables/name-missing", "tables/name-missing"),
    (
        "tables/presentational-with-headers",
        "tables/presentational-with-headers",
    ),
    ("document/lang-invalid", "document/lang-invalid"),
    ("keyboard/positive-tabindex", "keyboard/positive-tabindex"),
    ("zoom/viewport-scale-limited", "zoom/viewport-scale-limited"),
    ("aria/owns-conflict", "aria/owns-conflict"),
    ("description-duplicates-name", "description-duplicates-name"),
    (
        "aria/required-children-missing",
        "aria/required-children-missing",
    ),
    ("display/toggle-missing", "display/toggle-missing"),
    ("display/init-missing", "display/init-missing"),
    ("display/text-media-visible", "display/text-media-visible"),
    ("display/text-not-visible", "display/text-not-visible"),
    ("viz/text-missing", "viz/text-missing"),
    ("display/text-hidden", "display/text-hidden"),
    ("viz/caption-missing", "viz/caption-missing"),
    ("viz/static-missing", "viz/static-missing"),
    ("viz/table-missing", "viz/table-missing"),
    // Shared entry: another entry describes the same finding. A shared id
    // that replaced a former auditmysite rule (#690-#697) uses that rule's
    // text; the `aria/tabpanel-missing` text is the one `widget_rules`
    // reported under that id; the popover and inert checks share the
    // dialog/popover control text; the link-purpose ids use the 2.4.4
    // explanation of the former `link-name`.
    ("landmarks/not-unique", "landmark-unique"),
    ("links/generic-name", "2.4.4"),
    ("links/single-character-name", "2.4.4"),
    ("links/url-name", "2.4.4"),
    ("language/part-undetermined", "language/part-unmarked"),
    ("inert/dialog-inert", "modern-attribute-misuse"),
    ("popover/target-invalid", "modern-attribute-misuse"),
    ("popover/target-missing", "modern-attribute-misuse"),
    ("aria/role-abstract", "aria/role-invalid"),
    ("tables/data-undetermined", "tables/header-without-data"),
    ("aria/tabpanel-missing", "aria-required-children"),
    ("images/server-side-map", "server-side-image-map"),
    ("svg/name-missing", "svg-img-alt"),
    ("images/area-alt-missing", "area-alt"),
    ("images/input-alt-missing", "input-image-alt"),
    ("objects/alt-missing", "object-alt"),
    ("forms/group-missing", "form-field-group"),
    ("landmarks/banner-duplicate", "landmark-no-duplicate-banner"),
    (
        "landmarks/contentinfo-duplicate",
        "landmark-no-duplicate-contentinfo",
    ),
    ("landmarks/main-duplicate", "landmark-no-duplicate-main"),
    ("landmarks/banner-missing", "landmark-banner-present"),
    (
        "landmarks/navigation-missing",
        "landmark-navigation-present",
    ),
    (
        "landmarks/contentinfo-missing",
        "landmark-contentinfo-present",
    ),
    ("forms/title-only-label", "label-title-only"),
    (
        "keyboard/click-handler-not-focusable",
        "click-events-have-key-events",
    ),
    ("keyboard/pointer-only", "click-events-have-key-events"),
    ("landmarks/main-missing", "landmark-one-main"),
    ("keyboard/skip-link-missing", "skip-link"),
    ("focus/outline-removed", "focus-visible-outline-none"),
    (
        "keyboard/scrollable-region-not-focusable",
        "scrollable-region-focusable",
    ),
    ("forms/error-unidentified", "input-error-message"),
    ("names/required-missing", "aria-label"),
    ("names/symbol-only", "aria-label"),
    ("names/title-only", "title-only-description"),
    ("dialog/name-missing", "dialog-name"),
    ("buttons/name-missing", "control-missing-label"),
    ("links/name-missing", "control-missing-label"),
    ("links/used-as-button", "link-as-button"),
    ("aria/role-redundant", "redundant-role"),
    ("lists/role-redundant", "redundant-role"),
    ("summary/name-missing", "summary-name"),
    ("landmarks/content-outside", "region"),
    ("keyboard/focusable-no-role", "focusable-no-role"),
    // Criterion entry: the explanation is written for the rule's WCAG
    // criterion, not for the rule.
    ("image-alt", "1.1.1"),
    ("images/alt-missing", "1.1.1"),
    ("video-caption", "1.2.2"),
    ("media-alt", "1.2.8"),
    ("headings/section-without-heading", "2.4.10"),
    ("keyboard/interactive-not-focusable", "2.1.1"),
    ("headings/none", "2.4.1"),
    ("document-title", "2.4.2"),
    ("document/title-empty", "2.4.2"),
    ("document/title-missing", "2.4.2"),
    ("dialog/focusable-missing", "2.4.3"),
    ("focus-visible", "2.4.7"),
    ("label-in-name/mismatch", "2.5.3"),
    ("targets/size", "2.5.8"),
    ("forms/autocomplete-invalid", "1.3.5"),
    ("forms/purpose-missing", "1.3.5"),
    ("forms/group-name-missing", "3.3.2"),
    ("forms/instructions-missing", "3.3.2"),
    ("forms/label-missing", "3.3.2"),
    ("forms/placeholder-as-label", "3.3.2"),
    ("forms/required-unmarked", "3.3.2"),
    ("document/lang-missing", "3.1.1"),
    ("html-has-lang", "3.1.1"),
    ("context/autofocus", "3.2.1"),
    ("context/on-focus", "3.2.1"),
    ("context/on-input", "3.2.2"),
    ("contrast/text-insufficient", "1.4.3"),
    ("contrast/text-undetermined", "1.4.3"),
    ("css-overflow-hidden", "1.4.10"),
    ("contrast/control-insufficient", "1.4.11"),
    ("contrast/control-undetermined", "1.4.11"),
    ("text-spacing/clipped", "1.4.12"),
    ("color/link-indistinct", "1.4.1"),
    ("patterns/tooltip-unreferenced", "1.4.13"),
    ("focus/indicator-missing", "2.4.7"),
];

/// All WCAG rule explanations indexed by rule ID
static EXPLANATIONS: &[(&str, RuleExplanation)] = &[
    // ── 1. Perceivable ──────────────────────────────────────────────────────
    (
        "1.1.1",
        RuleExplanation {
            key: "1.1.1",
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
            key: "1.2.1",
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
            key: "1.2.2",
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
            key: "1.2.3",
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
            key: "1.2.8",
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
            key: "1.3.1",
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
            key: "region",
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
            key: "landmark-unique",
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
            key: "keyboard-trap",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "1.3.5",
        RuleExplanation {
            key: "1.3.5",
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
            key: "1.4.1",
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
            key: "1.4.3",
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
            key: "1.4.4",
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
            key: "1.4.10",
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
            key: "1.4.11",
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
            key: "1.4.12",
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
            key: "1.4.13",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<button>More info</button>\n<div role=\"tooltip\">Help text</div>"),
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
            key: "2.1.1",
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
            key: "focusable-no-role",
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
            key: "2.1.2",
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
            key: "2.4.1",
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
            key: "2.4.2",
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
            key: "2.4.3",
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
            key: "2.4.4",
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
    // Plan 98: `links/new-window-unannounced` (G201, 3.2.5 AAA).
    (
        "links/new-window-unannounced",
        RuleExplanation {
            key: "links/new-window-unannounced",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: Some("<a href=\"/prices.pdf\" target=\"_blank\">Price list</a>"),
            example_good: Some(
                "<a href=\"/prices.pdf\" target=\"_blank\">Price list (PDF, opens in a new window)</a>",
            ),
            example_decorative: None,
        },
    ),
    (
        "link-name-context",
        RuleExplanation {
            key: "link-name-context",
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
            key: "2.4.6",
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
            key: "2.4.7",
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
            key: "2.4.10",
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
            key: "2.5.3",
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
            key: "2.5.8",
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
            key: "3.1.1",
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
            key: "3.2.1",
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
            key: "3.2.2",
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
            key: "3.3.2",
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
            key: "ids/duplicate",
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
            key: "4.1.2",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button><svg>...</svg></button>"),
            example_good: Some("<button aria-label=\"Open menu\"><svg aria-hidden=\"true\">...</svg></button>"),
            example_decorative: None,
        },
    ),
    (
        "keyboard/hidden-focusable",
        RuleExplanation {
            key: "keyboard/hidden-focusable",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button aria-hidden=\"true\">Menu</button>"),
            example_good: Some("<button aria-hidden=\"true\" tabindex=\"-1\">Menu</button>"),
            example_decorative: None,
        },
    ),
    (
        "aria/attribute-prohibited",
        RuleExplanation {
            key: "aria/attribute-prohibited",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<span aria-label=\"Menu\">...</span>"),
            example_good: Some("<button aria-label=\"Menu\">...</button>"),
            example_decorative: None,
        },
    ),
    (
        "modern-attribute-misuse",
        RuleExplanation {
            key: "modern-attribute-misuse",
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
            key: "seo.headings.long_heading",
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
            key: "seo.headings.skipped_level",
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
            key: "seo.headings.missing_h1",
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
            key: "seo.headings.multiple_h1",
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
            key: "seo.headings.empty_heading",
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
    // own taxonomy Rule, would otherwise fall back
    // to get_explanation()'s internal WCAG-criterion lookup and surface
    // text written for a different, unrelated check under the same
    // criterion (same failure mode as #571).
    (
        "server-side-image-map",
        RuleExplanation {
            key: "server-side-image-map",
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
            key: "svg-img-alt",
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
            key: "area-alt",
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
            key: "input-image-alt",
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
            key: "object-alt",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "html-content-model",
        RuleExplanation {
            key: "html-content-model",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "form-field-group",
        RuleExplanation {
            key: "form-field-group",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "landmark-banner-is-top-level",
        RuleExplanation {
            key: "landmark-banner-is-top-level",
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
            key: "landmark-contentinfo-is-top-level",
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
            key: "landmark-main-is-top-level",
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
            key: "landmark-no-duplicate-banner",
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
            key: "landmark-no-duplicate-contentinfo",
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
            key: "landmark-no-duplicate-main",
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
            key: "landmark-banner-present",
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
            key: "label-title-only",
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
            key: "click-events-have-key-events",
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
            key: "landmark-one-main",
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
            key: "skip-link",
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
            key: "focus-visible-outline-none",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "scrollable-region-focusable",
        RuleExplanation {
            key: "scrollable-region-focusable",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div style=\"height: 8em; overflow: auto\">Long text …</div>"),
            example_good: Some("<div role=\"region\" aria-label=\"Terms\" tabindex=\"0\" style=\"height: 8em; overflow: auto\">Long text …</div>"),
            example_decorative: None,
        },
    ),
    (
        "input-error-message",
        RuleExplanation {
            key: "input-error-message",
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
            key: "duplicate-id-aria",
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
            key: "aria-label",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "title-only-description",
        RuleExplanation {
            key: "title-only-description",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button title=\"Search\"><svg aria-hidden=\"true\"></svg></button>"),
            example_good: Some("<button aria-label=\"Search\"><svg aria-hidden=\"true\"></svg></button>"),
            example_decorative: None,
        },
    ),
    (
        "description-duplicates-name",
        RuleExplanation {
            key: "description-duplicates-name",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button title=\"Close\">Close</button>"),
            example_good: Some("<button>Close</button>"),
            example_decorative: None,
        },
    ),
    (
        "aria-required-children",
        RuleExplanation {
            key: "aria-required-children",
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
            key: "dialog-name",
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
            key: "control-missing-label",
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
            key: "link-as-button",
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
            key: "redundant-role",
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
            key: "summary-name",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    // ── Geteilte Kennungen mit eigenem Taxonomie-Eintrag (Plan 56) ──
    // Ohne eigenen Text fielen diese Befunde auf die Erklaerung ihres
    // Kriteriums zurueck: eine leere Liste las sich wie „Fehlende semantische
    // Struktur", ein ungueltiger Sprachcode wie eine fehlende Sprachangabe.
    // Geschluesselt nach der Taxonomie-Kennung, die `resolve_explanation`
    // nach der (hier nie hinterlegten) geteilten Kennung versucht.
    (
        "headings/skip-level",
        RuleExplanation {
            key: "headings/skip-level",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<h2>Products</h2>\n<h4>Pricing</h4>"),
            example_good: Some("<h2>Products</h2>\n<h3>Pricing</h3>"),
            example_decorative: None,
        },
    ),
    (
        "headings/empty",
        RuleExplanation {
            key: "headings/empty",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<h2></h2>\n<p>Our services</p>"),
            example_good: Some("<h2>Our services</h2>"),
            example_decorative: None,
        },
    ),
    (
        "headings/h1-missing",
        RuleExplanation {
            key: "headings/h1-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<main>\n  <p class=\"title\">Our services</p>\n</main>"),
            example_good: Some("<main>\n  <h1>Our services</h1>\n</main>"),
            example_decorative: None,
        },
    ),
    (
        "headings/h1-multiple",
        RuleExplanation {
            key: "headings/h1-multiple",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "lists/invalid-structure",
        RuleExplanation {
            key: "lists/invalid-structure",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<ul>\n  <div><li>First</li></div>\n  <li>Second</li>\n</ul>"),
            example_good: Some("<ul>\n  <li>First</li>\n  <li>Second</li>\n</ul>"),
            example_decorative: None,
        },
    ),
    (
        "lists/empty",
        RuleExplanation {
            key: "lists/empty",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<ul class=\"results\"></ul>"),
            example_good: Some("<p class=\"results\">No results.</p>"),
            example_decorative: None,
        },
    ),
    (
        "lists/item-outside-list",
        RuleExplanation {
            key: "lists/item-outside-list",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div>\n  <li>First</li>\n  <li>Second</li>\n</div>"),
            example_good: Some("<ul>\n  <li>First</li>\n  <li>Second</li>\n</ul>"),
            example_decorative: None,
        },
    ),
    (
        "lists/term-without-definition",
        RuleExplanation {
            key: "lists/term-without-definition",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<dl>\n  <dt>Delivery</dt>\n</dl>"),
            example_good: Some("<dl>\n  <dt>Delivery</dt>\n  <dd>2 to 3 working days</dd>\n</dl>"),
            example_decorative: None,
        },
    ),
    (
        "tables/header-missing",
        RuleExplanation {
            key: "tables/header-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<table>\n  <tr><td><b>Name</b></td><td><b>Price</b></td></tr>\n  <tr><td>Basic</td><td>10 EUR</td></tr>\n</table>"),
            example_good: Some("<table>\n  <tr><th scope=\"col\">Name</th><th scope=\"col\">Price</th></tr>\n  <tr><td>Basic</td><td>10 EUR</td></tr>\n</table>"),
            example_decorative: None,
        },
    ),
    (
        "tables/name-missing",
        RuleExplanation {
            key: "tables/name-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<h3>Opening hours</h3>\n<table>...</table>"),
            example_good: Some("<table>\n  <caption>Opening hours</caption>\n  ...\n</table>"),
            example_decorative: None,
        },
    ),
    (
        "tables/presentational-with-headers",
        RuleExplanation {
            key: "tables/presentational-with-headers",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<table role=\"presentation\">\n  <tr><th>Name</th><th>Price</th></tr>\n</table>"),
            example_good: Some("<table>\n  <tr><th scope=\"col\">Name</th><th scope=\"col\">Price</th></tr>\n</table>"),
            example_decorative: None,
        },
    ),
    (
        "document/lang-invalid",
        RuleExplanation {
            key: "document/lang-invalid",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<html lang=\"english\">"),
            example_good: Some("<html lang=\"en\">"),
            example_decorative: None,
        },
    ),
    (
        "keyboard/positive-tabindex",
        RuleExplanation {
            key: "keyboard/positive-tabindex",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<input type=\"search\" tabindex=\"1\">"),
            example_good: Some("<input type=\"search\">"),
            example_decorative: None,
        },
    ),
    (
        "zoom/viewport-scale-limited",
        RuleExplanation {
            key: "zoom/viewport-scale-limited",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<meta name=\"viewport\" content=\"width=device-width, maximum-scale=3\">"),
            example_good: Some("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"),
            example_decorative: None,
        },
    ),
    // ── Fehlende Navigations-/Fußbereich-Landmark (Plan 56) ──
    // Eigene Befunde aus `landmarks.rs`; ohne eigenen Text liefen sie unter
    // der allgemeinen 1.3.1-Erklaerung mit Tabellen-Beispiel.
    (
        "landmark-navigation-present",
        RuleExplanation {
            key: "landmark-navigation-present",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div class=\"menu\">\n  <ul>...</ul>\n</div>"),
            example_good: Some("<nav aria-label=\"Main\">\n  <ul>...</ul>\n</nav>"),
            example_decorative: None,
        },
    ),
    (
        "landmark-contentinfo-present",
        RuleExplanation {
            key: "landmark-contentinfo-present",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div class=\"footer\">...</div>"),
            example_good: Some("<footer>...</footer>"),
            example_decorative: None,
        },
    ),
    // ── Messungen an der laufenden Seite (#1002–#1007) ──
    (
        "frames/cross-origin",
        RuleExplanation {
            key: "frames/cross-origin",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<iframe src=\"https://other.example/booking\"></iframe>"),
            example_good: Some("<iframe src=\"https://other.example/booking\" title=\"Room booking\"></iframe> <!-- test the content separately -->"),
            example_decorative: None,
        },
    ),
    (
        "orientation/root-rotated",
        RuleExplanation {
            key: "orientation/root-rotated",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("@media (orientation: portrait) { body { transform: rotate(90deg); } }"),
            example_good: Some("/* no rotation; the layout adapts to both orientations */"),
            example_decorative: None,
        },
    ),
    (
        "auth/paste-blocked",
        RuleExplanation {
            key: "auth/paste-blocked",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<input type=\"password\" onpaste=\"return false\">"),
            example_good: Some("<input type=\"password\" autocomplete=\"current-password\">"),
            example_decorative: None,
        },
    ),
    (
        "focus/obscured",
        RuleExplanation {
            key: "focus/obscured",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("header { position: sticky; top: 0; height: 80px; }"),
            example_good: Some("html { scroll-padding-top: 80px; } header { position: sticky; top: 0; height: 80px; }"),
            example_decorative: None,
        },
    ),
    (
        "focus/obscured-enhanced",
        RuleExplanation {
            key: "focus/obscured-enhanced",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: None,
            example_decorative: None,
        },
    ),
    // ── Shared ids from #690 without a fitting text ──
    // The other shared ids take the text of the former rule or of their
    // criterion, as `RULE_ENTRIES` lists.
    (
        "zoom/viewport-locked",
        RuleExplanation {
            key: "zoom/viewport-locked",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<meta name=\"viewport\" content=\"width=device-width, user-scalable=no\">"),
            example_good: Some("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"),
            example_decorative: None,
        },
    ),
    (
        "zoom/viewport-missing",
        RuleExplanation {
            key: "zoom/viewport-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: Some("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"),
            example_decorative: None,
        },
    ),
    (
        "aria/reference-missing",
        RuleExplanation {
            key: "aria/reference-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button aria-controls=\"menu\">Menu</button>"),
            example_good: Some("<button aria-controls=\"menu\">Menu</button>\n<ul id=\"menu\">...</ul>"),
            example_decorative: None,
        },
    ),
    (
        "aria/required-attribute-missing",
        RuleExplanation {
            key: "aria/required-attribute-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<div role=\"switch\" tabindex=\"0\">Notifications</div>"),
            example_good: Some("<div role=\"switch\" tabindex=\"0\" aria-checked=\"false\">Notifications</div>"),
            example_decorative: None,
        },
    ),
    (
        "images/alt-suspicious",
        RuleExplanation {
            key: "images/alt-suspicious",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Quick,
            example_bad: Some("<img src=\"team.jpg\" alt=\"IMG_4711.jpg\">"),
            example_good: Some("<img src=\"team.jpg\" alt=\"The five members of the support team\">"),
            example_decorative: Some("<img src=\"divider.png\" alt=\"\">"),
        },
    ),
    // ── Geteilte Kennungen aus #691 (B1) ohne passenden Text ──
    // `aria/attribute-prohibited`, `popover/*` und `inert/dialog-inert`
    // finden ihren Text ueber den Taxonomie-Eintrag der abgeloesten Regel.
    (
        "aria/role-invalid",
        RuleExplanation {
            key: "aria/role-invalid",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div role=\"buton\" tabindex=\"0\">Save</div>"),
            example_good: Some("<button type=\"button\">Save</button>"),
            example_decorative: None,
        },
    ),
    (
        "aria/attribute-unknown",
        RuleExplanation {
            key: "aria/attribute-unknown",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<input aria-labeledby=\"name-label\">"),
            example_good: Some("<input aria-labelledby=\"name-label\">"),
            example_decorative: None,
        },
    ),
    (
        "aria/attribute-value-invalid",
        RuleExplanation {
            key: "aria/attribute-value-invalid",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button aria-expanded=\"yes\">Menu</button>"),
            example_good: Some("<button aria-expanded=\"true\">Menu</button>"),
            example_decorative: None,
        },
    ),
    (
        "aria/owns-conflict",
        RuleExplanation {
            key: "aria/owns-conflict",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<div role=\"combobox\" aria-owns=\"list\"></div>\n<div role=\"combobox\" aria-owns=\"list\"></div>"),
            example_good: None,
            example_decorative: None,
        },
    ),
    (
        "aria/tab-selected-missing",
        RuleExplanation {
            key: "aria/tab-selected-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div role=\"tablist\">\n  <button role=\"tab\" class=\"active\">One</button>\n  <button role=\"tab\">Two</button>\n</div>"),
            example_good: Some("<div role=\"tablist\">\n  <button role=\"tab\" aria-selected=\"true\">One</button>\n  <button role=\"tab\" aria-selected=\"false\">Two</button>\n</div>"),
            example_decorative: None,
        },
    ),
    (
        "aria/combobox-popup-missing",
        RuleExplanation {
            key: "aria/combobox-popup-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<input role=\"combobox\" aria-expanded=\"true\">\n<ul id=\"fruits\" role=\"listbox\">...</ul>"),
            example_good: Some("<input role=\"combobox\" aria-expanded=\"true\" aria-controls=\"fruits\">\n<ul id=\"fruits\" role=\"listbox\">...</ul>"),
            example_decorative: None,
        },
    ),
    (
        "aria/attribute-not-allowed",
        RuleExplanation {
            key: "aria/attribute-not-allowed",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div aria-expanded=\"false\">Details</div>"),
            example_good: Some("<button aria-expanded=\"false\">Details</button>"),
            example_decorative: None,
        },
    ),
    (
        "aria/required-parent-missing",
        RuleExplanation {
            key: "aria/required-parent-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<ul role=\"tablist\">\n  <li><a role=\"tab\" href=\"#a\">A</a></li>\n</ul>"),
            example_good: Some("<ul role=\"tablist\">\n  <li role=\"presentation\"><a role=\"tab\" href=\"#a\">A</a></li>\n</ul>"),
            example_decorative: None,
        },
    ),
    (
        "aria/required-children-missing",
        RuleExplanation {
            key: "aria/required-children-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<div role=\"tablist\">\n  <button>One</button>\n  <button>Two</button>\n</div>"),
            example_good: Some("<div role=\"tablist\">\n  <button role=\"tab\" aria-selected=\"true\">One</button>\n  <button role=\"tab\" aria-selected=\"false\">Two</button>\n</div>"),
            example_decorative: None,
        },
    ),
    // ── Geteilte Kennungen aus #692 (B2) ohne passenden Text ──
    // `dialog-name` beschreibt nur den fehlenden Namen; das fehlende
    // `aria-modal` hatte keinen eigenen Text.
    (
        "dialog/modal-unmarked",
        RuleExplanation {
            key: "dialog/modal-unmarked",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div role=\"dialog\" aria-labelledby=\"t\">\n  <h2 id=\"t\">Confirm</h2>\n</div>"),
            example_good: Some("<div role=\"dialog\" aria-modal=\"true\" aria-labelledby=\"t\">\n  <h2 id=\"t\">Confirm</h2>\n</div>"),
            example_decorative: None,
        },
    ),
    // ── Geteilte Kennungen aus #693 (B3) ohne passenden Text ──
    // `form-no-submit` hatte keinen eigenen Text und nahm den von 3.2.2, der
    // den Kontextwechsel bei Eingabe beschreibt, nicht das fehlende
    // Absende-Element.
    (
        "forms/no-submit",
        RuleExplanation {
            key: "forms/no-submit",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<form action=\"/results\">\n  <input type=\"checkbox\" name=\"open\" id=\"o\"><label for=\"o\">Open only</label>\n</form>"),
            example_good: Some("<form action=\"/results\">\n  <input type=\"checkbox\" name=\"open\" id=\"o\"><label for=\"o\">Open only</label>\n  <button type=\"submit\">Apply filter</button>\n</form>"),
            example_decorative: None,
        },
    ),
    // `accessible-auth-captcha` hatte gar keinen Text.
    (
        "auth/captcha",
        RuleExplanation {
            key: "auth/captcha",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<form action=\"/login\">\n  <input type=\"password\" autocomplete=\"current-password\">\n  <img src=\"captcha.png\" alt=\"captcha\">\n</form>"),
            example_good: None,
            example_decorative: None,
        },
    ),
    // ── Geteilte Kennungen aus #694 ohne passenden Text ──
    (
        "landmarks/not-top-level",
        RuleExplanation {
            key: "landmarks/not-top-level",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<main>\n  <div role=\"banner\">…</div>\n</main>"),
            example_good: Some("<header>…</header>\n<main>…</main>"),
            example_decorative: None,
        },
    ),
    (
        "patterns/accordion-controls-missing",
        RuleExplanation {
            key: "patterns/accordion-controls-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<button aria-expanded=\"true\">Details</button>\n<div id=\"panel\">…</div>"),
            example_good: Some("<button aria-expanded=\"true\" aria-controls=\"panel\">Details</button>\n<div id=\"panel\">…</div>"),
            example_decorative: None,
        },
    ),
    // ── Geteilte Kennungen aus #697 ohne passenden Text ──
    (
        "tables/header-without-data",
        RuleExplanation {
            key: "tables/header-without-data",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<table>
  <tr><th>Name</th><th>Preis</th></tr>
</table>"),
            example_good: Some("<table>
  <tr><th>Name</th><th>Preis</th></tr>
  <tr><td>Apfel</td><td>1 €</td></tr>
</table>"),
            example_decorative: None,
        },
    ),
    (
        "tables/headers-attr-invalid",
        RuleExplanation {
            key: "tables/headers-attr-invalid",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<td headers=\"preis-alt\">1 €</td>"),
            example_good: Some("<th id=\"preis\">Preis</th> … <td headers=\"preis\">1 €</td>"),
            example_decorative: None,
        },
    ),
    (
        "document/lang-mismatch",
        RuleExplanation {
            key: "document/lang-mismatch",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<html lang=\"de\" xml:lang=\"en\">"),
            example_good: Some("<html lang=\"de\">"),
            example_decorative: None,
        },
    ),
    (
        "language/part-unmarked",
        RuleExplanation {
            key: "language/part-unmarked",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<p>The quick brown fox jumps over the lazy dog.</p>"),
            example_good: Some("<p lang=\"en\">The quick brown fox jumps over the lazy dog.</p>"),
            example_decorative: None,
        },
    ),
    // ── Geteilte Kennungen aus #696 ohne passenden Text ──
    (
        "media/audio-autoplay",
        RuleExplanation {
            key: "media/audio-autoplay",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<audio src=\"intro.mp3\" autoplay></audio>"),
            example_good: Some("<audio src=\"intro.mp3\" controls></audio>"),
            example_decorative: None,
        },
    ),
    // `frame-title` fiel bisher auf den 2.4.1-Text („Fehlende
    // Sprungnavigation") zurueck.
    (
        "frames/name-missing",
        RuleExplanation {
            key: "frames/name-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<iframe src=\"map.html\"></iframe>"),
            example_good: Some("<iframe src=\"map.html\" title=\"Lageplan des Standorts\"></iframe>"),
            example_decorative: None,
        },
    ),
    (
        "display/toggle-missing",
        RuleExplanation {
            key: "display/toggle-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: Some("<button type=\"button\" data-display-toggle aria-haspopup=\"menu\">Display: calm</button>"),
            example_decorative: None,
        },
    ),
    (
        "display/init-missing",
        RuleExplanation {
            key: "display/init-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<script type=\"module\" src=\"/display-init.js\"></script>"),
            example_good: Some("<script src=\"/display-init.js\"></script> <!-- blocking, in <head> -->"),
            example_decorative: None,
        },
    ),
    (
        "display/text-media-visible",
        RuleExplanation {
            key: "display/text-media-visible",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: None,
            example_good: Some("html[data-display=\"text\"] [data-viz-live], html[data-display=\"text\"] [data-viz-static] { display: none; }"),
            example_decorative: None,
        },
    ),
    (
        "display/text-not-visible",
        RuleExplanation {
            key: "display/text-not-visible",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: Some("<figure data-viz=\"chart\"><div data-viz-text><p>Statement</p><table>...</table></div></figure>"),
            example_decorative: None,
        },
    ),
    (
        "display/text-hidden",
        RuleExplanation {
            key: "display/text-hidden",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<div data-viz-text aria-hidden=\"true\">...</div>"),
            example_good: Some("<div data-viz-text class=\"visually-hidden\">...</div>"),
            example_decorative: None,
        },
    ),
    (
        "viz/text-missing",
        RuleExplanation {
            key: "viz/text-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<figure data-viz=\"chart\"><svg>...</svg></figure>"),
            example_good: Some("<figure data-viz=\"chart\"><svg>...</svg><div data-viz-text>...</div></figure>"),
            example_decorative: None,
        },
    ),
    (
        "viz/caption-missing",
        RuleExplanation {
            key: "viz/caption-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Quick,
            example_bad: Some("<figure data-viz=\"chart\">...</figure>"),
            example_good: Some("<figure data-viz=\"chart\">... <figcaption>Global temperature anomaly</figcaption></figure>"),
            example_decorative: None,
        },
    ),
    (
        "viz/static-missing",
        RuleExplanation {
            key: "viz/static-missing",
            responsible_role: Role::Development,
            effort_estimate: Effort::Medium,
            example_bad: Some("<figure data-viz=\"3d\"><canvas data-viz-live></canvas></figure>"),
            example_good: Some("<figure data-viz=\"3d\"><img data-viz-static src=\"globe.png\" alt=\"\"><canvas data-viz-live></canvas></figure>"),
            example_decorative: None,
        },
    ),
    (
        "viz/table-missing",
        RuleExplanation {
            key: "viz/table-missing",
            responsible_role: Role::Editorial,
            effort_estimate: Effort::Medium,
            example_bad: None,
            example_good: Some("<div data-viz-text><table><caption>Values</caption>...</table></div>"),
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

        for rule_id in [
            "landmarks/not-unique",
            "landmarks/content-outside",
            "landmarks/banner-missing",
            "landmarks/navigation-missing",
            "landmarks/contentinfo-missing",
            // Plan 56: an empty list must not read as "Missing semantic
            // structure", an invalid lang code not as a missing one.
            "lists/empty",
            "tables/header-missing",
            "headings/skip-level",
        ] {
            let resolved = resolve_explanation(Some(rule_id), rule_id, "1.3.1")
                .unwrap_or_else(|| panic!("no explanation resolved for {rule_id}"));
            assert_ne!(
                resolved.customer_title(false),
                generic.customer_title(false),
                "{rule_id}: resolved to the generic 1.3.1 explanation instead of its own",
            );
        }
        let lang = get_explanation("3.1.1").expect("WCAG 3.1.1 explanation");
        let invalid = resolve_explanation(
            Some("document/lang-invalid"),
            "document/lang-invalid",
            "3.1.1",
        )
        .expect("document/lang-invalid");
        assert_ne!(invalid.customer_title(false), lang.customer_title(false));
    }

    /// The fallback chain still has to work for findings without an axe id,
    /// and for axe ids that have no override of their own.
    #[test]
    fn resolve_explanation_falls_back_past_missing_axe_id() {
        let by_rule = resolve_explanation(None, "keyboard/hidden-focusable", "4.1.2");
        assert!(by_rule.is_some(), "rule_id fallback lost");

        let unknown_axe = resolve_explanation(
            Some("no-such-axe-rule"),
            "keyboard/hidden-focusable",
            "4.1.2",
        );
        assert_eq!(
            unknown_axe.map(|e| e.customer_title(false)),
            by_rule.map(|e| e.customer_title(false)),
            "an unknown axe id must not shadow the rule_id match",
        );

        let by_criterion = resolve_explanation(None, "no/such-rule", "1.1.1");
        assert!(by_criterion.is_some(), "wcag_criterion fallback lost");
    }

    /// Regression for #357: these rules carry their explanation under the
    /// taxonomy key, so a lookup must resolve them and return a localized
    /// (non-empty, distinct DE/EN) recommendation rather than a raw English fix.
    #[test]
    fn taxonomy_keyed_rules_have_localized_recommendation() {
        for rule_id in [
            "keyboard/hidden-focusable",
            "aria/attribute-prohibited",
            "modern-attribute-misuse",
            "ids/duplicate",
            // Regression (2026-08-31/09-01): keyboard.rs's NotTestable
            // finding had no explanations.rs entry at all, so the PDF fell
            // back to the raw English Violation.fix_suggestion unlocalized
            // -- confirmed leaking into German reports.
            "keyboard-trap",
        ] {
            let expl = get_explanation(rule_id)
                .unwrap_or_else(|| panic!("missing explanation for {rule_id}"));
            let de = expl.recommendation(false);
            let en = expl.recommendation(true);
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
        let de = expl.recommendation(false);
        let en = expl.recommendation(true);
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
        let de = expl.recommendation(false);
        let en = expl.recommendation(true);
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
        let de = expl.recommendation(false);
        let en = expl.recommendation(true);
        assert!(
            de.contains("prüfen") && en.to_lowercase().contains("verify"),
            "link-name-context explanation should ask for manual verification: de={de} en={en}"
        );
    }

    const RULES_FTL_DE: &str = include_str!("../../locales/de/rules.ftl");
    const RULES_FTL_EN: &str = include_str!("../../locales/en/rules.ftl");

    /// The attributes of an explanation message; its value is the title.
    const ATTRIBUTES: [&str; 5] = [
        "description",
        "user-impact",
        "typical-cause",
        "recommendation",
        "technical-note",
    ];

    /// The same five texts as attributes of a rule message, where
    /// `.user-impact` already holds the shorter catalogue text.
    const RULE_MESSAGE_ATTRIBUTES: [&str; 5] = [
        "description",
        "explanation-user-impact",
        "typical-cause",
        "recommendation",
        "technical-note",
    ];

    /// The six texts of an explanation in one language, by field name.
    fn texts(expl: &RuleExplanation, en: bool) -> [(&'static str, String); 6] {
        [
            ("customer_title", expl.customer_title(en)),
            ("customer_description", expl.customer_description(en)),
            ("user_impact", expl.user_impact(en)),
            ("typical_cause", expl.typical_cause(en)),
            ("recommendation", expl.recommendation(en)),
            ("technical_note", expl.technical_note(en)),
        ]
    }

    /// An entry reads its texts under the key the lookup finds it by.
    #[test]
    fn every_entry_carries_its_table_key() {
        for (key, expl) in EXPLANATIONS {
            assert_eq!(*key, expl.key);
        }
    }

    /// `.`, `/` and `_` all become `-`, so two table keys could meet in one
    /// message; none do.
    #[test]
    fn explanation_text_keys_are_distinct_fluent_identifiers() {
        let mut keys_by_text_key = std::collections::BTreeMap::new();
        for (key, _) in EXPLANATIONS {
            let text_key = explanation_text_key(key);
            assert!(
                text_key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "{text_key} (entry {key}) is not a Fluent identifier"
            );
            if let Some(other) = keys_by_text_key.insert(text_key.clone(), key) {
                panic!("entries {other} and {key} share the key {text_key}");
            }
        }
    }

    /// `rules.ftl` holds exactly the texts of both tables and nothing else. A
    /// rule has its title, `.user-impact` and `.technical-impact`; a rule that
    /// has an entry under its own id carries the five explanation texts as
    /// further attributes of the same message. An entry keyed by a criterion
    /// number or an axe id has its own `explanation-` message with the title
    /// and the five attributes. No `explanation-` message exists for a key
    /// that is a rule id.
    #[test]
    fn rules_ftl_holds_exactly_the_rule_texts_and_the_explanation_texts() {
        let mut expected = std::collections::BTreeSet::new();
        for rule in crate::taxonomy::rules::RULES {
            let key = crate::taxonomy::rules::rule_text_key(rule.id);
            expected.insert(format!("{key}.user-impact"));
            expected.insert(format!("{key}.technical-impact"));
            expected.insert(key);
        }
        for (key, _) in EXPLANATIONS {
            let text_key = explanation_text_key(key);
            if is_rule_entry(key) {
                assert!(
                    text_key.starts_with("rule-"),
                    "entry {key} is a rule id but reads {text_key}"
                );
                for attribute in RULE_MESSAGE_ATTRIBUTES {
                    expected.insert(format!("{text_key}.{attribute}"));
                }
            } else {
                assert!(
                    text_key.starts_with("explanation-"),
                    "entry {key} is no rule id but reads {text_key}"
                );
                for attribute in ATTRIBUTES {
                    expected.insert(format!("{text_key}.{attribute}"));
                }
                expected.insert(text_key);
            }
        }
        for (locale, ftl) in [("de", RULES_FTL_DE), ("en", RULES_FTL_EN)] {
            let entries = crate::i18n::ftl_entries(ftl);
            let missing: Vec<_> = expected.difference(&entries).collect();
            let unused: Vec<_> = entries.difference(&expected).collect();
            assert!(
                missing.is_empty(),
                "missing in {locale}/rules.ftl: {missing:?}"
            );
            assert!(
                unused.is_empty(),
                "in {locale}/rules.ftl without a rule or an explanation: {unused:?}"
            );
        }
    }

    /// Every row of `RULE_ENTRIES` names a taxonomy rule and an entry that
    /// exists, no rule is listed twice, and a rule that has an entry under its
    /// own id is listed with it.
    #[test]
    fn rule_entries_name_existing_rules_and_entries() {
        let mut listed = std::collections::BTreeSet::new();
        for (rule_id, key) in RULE_ENTRIES {
            assert!(
                crate::taxonomy::RuleLookup::by_id(rule_id).is_some(),
                "{rule_id} is not a taxonomy rule id"
            );
            assert!(
                EXPLANATIONS.iter().any(|(k, _)| k == key),
                "{rule_id} names the entry {key}, which does not exist"
            );
            assert!(listed.insert(*rule_id), "{rule_id} is listed twice");
        }
        for (key, _) in EXPLANATIONS {
            if is_rule_entry(key) {
                assert!(
                    RULE_ENTRIES.contains(&(*key, *key)),
                    "the entry {key} is keyed by a rule id but not listed for it"
                );
            }
        }
    }

    /// The registry covers the rules that have an explanation and no other:
    /// a rule resolves to exactly the entry its row names, and a rule without
    /// a row has none, whatever its WCAG reference (no derivation through
    /// `external_ref`). The counts are those measured when the table replaced
    /// the alias table and the criterion fallback (#935): 61 own entries, 43
    /// shared entries and 35 criterion entries; 25 rules have no explanation.
    #[test]
    fn rule_entries_cover_exactly_the_rules_with_an_explanation() {
        let (mut own, mut other, mut none) = (0, 0, 0);
        for rule in crate::taxonomy::rules::RULES {
            let listed = RULE_ENTRIES
                .iter()
                .find(|(id, _)| *id == rule.id)
                .map(|(_, key)| *key);
            let resolved = get_explanation(rule.id).map(|e| e.key);
            assert_eq!(resolved, listed, "rule {}", rule.id);
            match listed {
                Some(key) if key == rule.id => own += 1,
                Some(_) => other += 1,
                None => none += 1,
            }
        }
        assert_eq!((own, other, none), (61, 78, 25));
        assert_eq!(RULE_ENTRIES.len(), 61 + 78);
    }

    /// The lookup the reports use resolves for every entry: a missing message
    /// would come back as its key.
    #[test]
    fn every_explanation_text_resolves_in_both_languages() {
        for (key, expl) in EXPLANATIONS {
            let text_key = explanation_text_key(key);
            for en in [false, true] {
                for (field, text) in texts(expl, en) {
                    assert!(
                        !text.is_empty() && !text.starts_with(&text_key),
                        "{key} {field} (en = {en}): {text:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn english_explanation_texts_have_no_german_umlauts() {
        for (key, expl) in EXPLANATIONS {
            for (field, text) in texts(expl, true) {
                assert!(
                    !text.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
                    "{key} {field}: EN text has an umlaut: {text}"
                );
            }
        }
    }

    /// Export every explanation text into one data file for review: one
    /// record per table key, field and language, sorted.
    ///
    /// Ignored by default; run on demand to (re)generate the file:
    ///   cargo test --lib export_all_explanation_texts -- --ignored --nocapture
    #[test]
    #[ignore = "run on demand to regenerate reports/explanation-texts.json"]
    fn export_all_explanation_texts() {
        let mut records: Vec<(&str, &str, &str, String)> = Vec::new();
        for (key, expl) in EXPLANATIONS {
            for (lang, en) in [("de", false), ("en", true)] {
                for (field, text) in texts(expl, en) {
                    records.push((key, field, lang, text));
                }
            }
        }
        records.sort();
        let records: Vec<serde_json::Value> = records
            .into_iter()
            .map(|(key, field, lang, text)| {
                serde_json::json!({
                    "key": key,
                    "field": field,
                    "lang": lang,
                    "text": text,
                })
            })
            .collect();

        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("reports");
        std::fs::create_dir_all(&out_dir).expect("create reports/");
        let out_path = out_dir.join("explanation-texts.json");
        let out = serde_json::to_string_pretty(&records).expect("serialize explanation texts");
        std::fs::write(&out_path, out).expect("write reports/explanation-texts.json");
        eprintln!(
            "Wrote {} explanation text records to {}",
            records.len(),
            out_path.display()
        );
    }

    /// Export which table entry every lookup resolves to, for review: the
    /// result of `get_explanation` and of `resolve_explanation` (axe id, rule
    /// id, criterion) for every taxonomy rule, and of `get_explanation` for
    /// every table key.
    ///
    /// Ignored by default; run on demand to (re)generate the file:
    ///   cargo test --lib export_explanation_resolution -- --ignored --nocapture
    #[test]
    #[ignore = "run on demand to regenerate reports/explanation-resolution.json"]
    fn export_explanation_resolution() {
        use std::collections::BTreeMap;
        let key_of = |e: Option<&RuleExplanation>| e.map(|e| e.key);
        let mut by_rule_id = BTreeMap::new();
        let mut by_resolve = BTreeMap::new();
        for rule in crate::taxonomy::rules::RULES {
            let criterion = rule
                .external_ref
                .and_then(|r| r.strip_prefix("WCAG "))
                .unwrap_or("");
            by_rule_id.insert(rule.id, key_of(get_explanation(rule.id)));
            by_resolve.insert(
                rule.id,
                key_of(resolve_explanation(rule.axe_id, rule.id, criterion)),
            );
        }
        let by_table_key: BTreeMap<_, _> = EXPLANATIONS
            .iter()
            .map(|(key, _)| (*key, key_of(get_explanation(key))))
            .collect();

        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("reports");
        std::fs::create_dir_all(&out_dir).expect("create reports/");
        let out_path = out_dir.join("explanation-resolution.json");
        let out = serde_json::to_string_pretty(&serde_json::json!({
            "get_explanation_by_rule_id": by_rule_id,
            "resolve_explanation_by_rule": by_resolve,
            "get_explanation_by_table_key": by_table_key,
        }))
        .expect("serialize explanation resolution");
        std::fs::write(&out_path, out).expect("write reports/explanation-resolution.json");
        eprintln!(
            "Wrote the resolution of {} rules and {} table keys to {}",
            by_rule_id.len(),
            by_table_key.len(),
            out_path.display()
        );
    }
}
