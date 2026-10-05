//! Die geteilten Regeln aus `a11y-rules` in auditmysites Befundfluss.
//!
//! # Wozu
//!
//! Ein Regelbestand soll drei Oberflächen bedienen — astro-post-audit zur
//! Build-Zeit, auditmysite in CI und Crawl, LiveAudit in der laufenden Seite —
//! und derselbe Befund soll überall **dieselbe Kennung** tragen. Solange
//! auditmysite eigene Regeln mit eigenen (axe-core-nahen) Kennungen führt,
//! heißt derselbe Befund eben nicht überall gleich.
//!
//! # Warum eine Liste statt „alles an"
//!
//! [`a11y_rules::run_with_semantics`] lässt den gesamten geteilten Bestand
//! laufen. Würde auditmysite alle Befunde daraus übernehmen, gäbe es jeden
//! doppelt, solange die entsprechende auditmysite-Regel noch existiert — und
//! die lassen sich nicht alle in einem Zug ablösen.
//!
//! [`SHARED_RULES`] führt deshalb genau die Kennungen, deren auditmysite-
//! Gegenstück **bereits gelöscht** ist. Alles andere aus dem geteilten Bestand
//! wird nicht etwa stillschweigend verworfen, sondern als
//! [`NotRun::Disabled`] vermerkt: Der Bericht sagt damit aus, was er nicht
//! geprüft hat, statt Schweigen wie ein Bestehen aussehen zu lassen.
//!
//! [`NotRun::Disabled`]: a11y_report::NotRun::Disabled

use a11y_dom::{Node, NodeId};
use a11y_report::{Finding, NotRun, Outcome};
use a11y_rules::Locale;

use crate::accessibility::CdpDocument;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleRun, Violation, WcagResults};
use crate::wcag::RuleFilterConfig;

/// Was auditmysite über eine geteilte Regel zusätzlich wissen muss.
///
/// `a11y-rules` führt Kennung, WCAG-Kriterium und Schwere, aber weder die
/// Konformitätsstufe noch eine Fundstellen-URL — beides braucht auditmysites
/// Ausgabe. Diese Tabelle ergänzt genau das und nichts weiter.
pub struct SharedRule {
    /// Die geteilte Kennung, identisch auf allen drei Oberflächen.
    pub id: &'static str,
    /// WCAG-Erfolgskriterium, z. B. `"3.1.1"`.
    pub criterion: &'static str,
    pub level: WcagLevel,
    /// Anzeigename, entspricht dem vormaligen `RuleMetadata::name`.
    pub name: &'static str,
    pub help_url: &'static str,
}

/// Die geteilten Kennungen, die auditmysite bereits führt.
///
/// Eine Kennung darf hier erst stehen, wenn die auditmysite-eigene Regel
/// dazu gelöscht ist — sonst stünde derselbe Befund zweimal im Bericht.
pub const SHARED_RULES: &[SharedRule] = &[
    // Ersetzt `wcag::rules::language::check_language` samt der
    // DOM-Nachbesserung `audit::pipeline::apply_lang_attribute_check`.
    // Die AX-Eigenschaft `language` synthetisiert Chrome aus Locale und
    // Kontext, auch wenn der Autor nie ein `lang` gesetzt hat — die
    // AX-basierte Prüfung war für den häufigsten Fall also blind, und
    // auditmysite hat das mit einer zweiten, per JavaScript nachgeschobenen
    // Prüfung ausgeglichen. Die geteilte Regel liest das `lang`-Attribut
    // direkt aus dem DOM und braucht beides nicht.
    SharedRule {
        id: "document/lang-missing",
        criterion: "3.1.1",
        level: WcagLevel::A,
        name: "Language of Page",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/language-of-page.html",
    },
    // Neu gegenüber der abgelösten Regel: Sie kannte nur „da oder nicht da".
    SharedRule {
        id: "document/lang-invalid",
        criterion: "3.1.1",
        level: WcagLevel::A,
        name: "Language of Page",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/language-of-page.html",
    },
    // Ersetzt `wcag::rules::parsing::check_parsing_with_page` (axe-Kennung
    // `duplicate-id`). Beide lesen dieselbe Quelle -- die eigene Regel wertete
    // `document.querySelectorAll('[id]')` per JavaScript aus, die geteilte
    // läuft über denselben DOM aus dem CDP-Abzug. Gleiche Erkennung; gemeldet
    // wird davon seit Plan 54 §2 nur noch die referenzierte Dublette.
    //
    // Nicht abgelöst ist `check_parsing` (axe-Kennung `duplicate-id-aria`):
    // Die prüft widersprüchliche `aria-owns`-Beziehungen im AX-Baum, also
    // etwas anderes als doppelte IDs, und bleibt.
    // Ersetzt `wcag::rules::headings` vollständig — alle vier Prüfungen haben
    // seit a11y-rules 0.8.0 ein geteiltes Gegenstück; `headings/h1-multiple`
    // war die letzte Lücke.
    //
    // Drei Unterschiede, alle gewollt:
    //
    // - Die geteilte Fassung läuft in Dokumentreihenfolge über den DOM. Die
    //   AX-basierte sortierte nach `node_id` als Text, womit „h10" vor „h2"
    //   kam und Sprünge in langen Seiten falsch bewertet wurden.
    // - Mehrere `h1` sind dort `REVIEW` statt `Violation` und melden einmal
    //   statt je überzähliger Überschrift. In HTML sind mehrere `h1`
    //   zulässig; das ist eine Erwartung, kein Verstoß.
    // - „Leer" heißt dort: kein Text im Teilbaum und kein `aria-label`. Die
    //   AX-Fassung fragte den Accessible Name und übersah damit nichts, was
    //   über `aria-labelledby` oder ein `alt` im Bild benannt ist. Diese
    //   beiden Fälle meldet die geteilte Fassung zu Unrecht — ein Befund für
    //   `a11y-core`, keine Rückausnahme hier.
    SharedRule {
        id: "headings/empty",
        // Die geteilte Regel führt 1.3.1 und 2.4.6; auditmysite meldete leere
        // Überschriften bisher unter 2.4.6, und dabei bleibt es.
        criterion: "2.4.6",
        level: WcagLevel::AA,
        name: "Headings and Labels (Empty Heading)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/headings-and-labels.html",
    },
    SharedRule {
        id: "headings/skip-level",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Heading Hierarchy)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "headings/h1-missing",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Missing Main Heading)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "headings/h1-multiple",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Multiple Main Headings)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // WCAG 2.2 hat 4.1.1 (Parsing) gestrichen — eine doppelte ID ist für sich
    // genommen kein Erfolgskriterium mehr. Ein Verstoß bleibt sie dort, wo ein
    // IDREF auf sie zeigt: Dann ist nicht mehr bestimmbar, welches Element
    // gemeint ist, und Name/Rolle/Wert der referenzierenden Beziehung bricht.
    // Genau dieser Fall wird gemeldet, und zwar als 4.1.2 — dieselbe
    // Zuordnung, die axe-core für `duplicate-id-aria` führt (Plan 54 §2).
    // Seit a11y-rules 0.12 entscheidet das die geteilte Regel selbst; bis
    // dahin filterte auditmysite hier nach, und die Kennung hieß in den
    // anderen Hosts etwas anderes.
    SharedRule {
        id: "ids/duplicate",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Referenced Duplicate ID)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::focus_order::check_positive_tabindex_with_page`.
    // Auch hier las die eigene Regel den DOM per JavaScript; die geteilte
    // liest dasselbe Attribut aus dem Abzug.
    //
    // Die eigene Regel deckelte die Zahl der Befunde bei 250
    // (`POSITIVE_TABINDEX_CAP`) -- die geteilte tut das nicht und meldet
    // damit eher mehr als weniger.
    //
    // `check_focus_order` (aria-hidden und trotzdem fokussierbar) ist seit
    // #690 ebenfalls abgelöst, siehe `keyboard/hidden-focusable`.
    SharedRule {
        id: "keyboard/positive-tabindex",
        criterion: "2.4.3",
        level: WcagLevel::A,
        name: "Focus Order",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/focus-order.html",
    },
    // Ersetzt `wcag::rules::list_structure` vollständig. Die geteilte Fassung
    // deckt seit a11y-rules 0.5.0 alle drei Prüfungen ab — Fremdkinder, leere
    // Listen und Begriffe ohne Definition — und erkennt zusätzlich
    // `role="list"`/`role="listitem"`, wofür die AX-basierte Regel blind war.
    SharedRule {
        id: "lists/invalid-structure",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (List Structure)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "lists/empty",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Empty List)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Der Fall, den die geteilte Fassung bis 0.6.0 nicht kannte: ein <li>
    // ganz ohne Liste darüber. Ohne ihn hätte die Ablösung eine Prüfung
    // verloren statt sie zu teilen.
    SharedRule {
        id: "lists/item-outside-list",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Orphan List Item)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "lists/term-without-definition",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Definition Term)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `wcag::rules::table_rules`. Kopfzellen, Name und die
    // widersprüchlich ausgezeichnete Layouttabelle sind seit 0.5.0 alle
    // abgedeckt. Ein Unterschied bleibt und ist gewollt: Die fehlende
    // Tabellenbenennung ist dort `REVIEW`, nicht `Violation` — ob eine
    // Tabelle einen Namen braucht, hängt vom Kontext ab.
    SharedRule {
        id: "tables/header-missing",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Table Headers)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "tables/name-missing",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Table Name)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "tables/presentational-with-headers",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Presentational Table)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `wcag::rules::meta_viewport_large`. Die geteilte Regel trennt
    // seit 0.5.0, was auditmysite auf zwei Regeln verteilt hatte:
    // `zoom/viewport-locked` ist der Verstoß unter 200 %,
    // `zoom/viewport-scale-limited` die Begrenzung zwischen 200 % und 500 %.
    SharedRule {
        id: "zoom/viewport-scale-limited",
        criterion: "1.4.4",
        level: WcagLevel::AA,
        name: "Resize Text (Viewport Scale)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/resize-text.html",
    },
    // Ersetzt `wcag::rules::resize_text` (axe-Kennung `meta-viewport`)
    // vollständig -- die Regel prüfte trotz ihres Namens nur den Viewport,
    // nicht die Textvergrößerung selbst.
    //
    // Zwei Unterschiede, beide gewollt:
    //
    // - Die eigene Regel suchte `user-scalable=no` und `maximum-scale=` als
    //   Teilzeichenkette. Die geteilte zerlegt `content` nach CSS Viewport
    //   (Komma, Semikolon, Leerraum als Trenner, Leerraum um `=`) und erkennt
    //   damit auch `maximum-scale = 1` oder `user-scalable=0`.
    // - `maximum-scale` unter 2 war dort `Medium`, `user-scalable=no`
    //   `High`. Die geteilte Regel meldet beides als `High`: Beides sperrt
    //   die Vergrößerung auf 200 %, einmal ganz und einmal teilweise.
    SharedRule {
        id: "zoom/viewport-locked",
        criterion: "1.4.4",
        level: WcagLevel::AA,
        name: "Resize Text (Viewport Zoom Locked)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/resize-text.html",
    },
    // Neu, und bewusst jetzt übernommen: Ohne Viewport-Angabe legen mobile
    // Browser eine Desktop-Breite zugrunde und verkleinern die Seite, der
    // Text landet unter jeder lesbaren Größe. `resize_text` sah darin keinen
    // Befund -- ohne Meta-Tag gab es nichts zu prüfen. Die geteilte Regel
    // führt 1.4.4 und 1.4.10; auditmysite meldet sie unter 1.4.4, wie den
    // Rest der Viewport-Befunde.
    SharedRule {
        id: "zoom/viewport-missing",
        criterion: "1.4.4",
        level: WcagLevel::AA,
        name: "Resize Text (Viewport Missing)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/resize-text.html",
    },
    // Ersetzt den Fall „fehlt oder leer" aus
    // `wcag::rules::page_titled::check_page_titled_with_page`. Beide lesen
    // den DOM; die eigene Regel meldete fehlenden, leeren und nichtssagenden
    // Titel unter einer Kennung und einem Text. Jetzt sind es drei Aussagen:
    // `document/title-missing`, `document/title-empty` und die eigene
    // Heuristik für den nichtssagenden Titel („Untitled", „Home"), die
    // weiter unter `document-title` läuft -- für fehlende und leere Titel
    // aber nicht mehr anschlägt.
    SharedRule {
        id: "document/title-missing",
        criterion: "2.4.2",
        level: WcagLevel::A,
        name: "Page Titled",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/page-titled.html",
    },
    SharedRule {
        id: "document/title-empty",
        criterion: "2.4.2",
        level: WcagLevel::A,
        name: "Page Titled",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/page-titled.html",
    },
    // Ersetzt `wcag::rules::aria_relationships` vollständig (axe-Kennung
    // `aria-valid-attr`), die AX-Prüfung und die DOM-Ergänzung.
    //
    // Die DOM-Ergänzung prüfte `aria-controls`, `aria-owns` und
    // `aria-activedescendant` auf leere Werte und auf IDs, die es nicht gibt.
    // Genau das tut die geteilte Regel auch, und zusätzlich
    // `aria-labelledby`/`aria-describedby` auf nicht vorhandene IDs -- ein
    // leeres `aria-labelledby` lässt sie durch, weil die Namensberechnung
    // dann auf die übrigen Quellen zurückfällt.
    //
    // Nicht übernommen, und gewollt: Die AX-Prüfung meldete eine Beziehung,
    // deren Ziel im DOM steht, aber im AX-Baum fehlt (versteckt). Auf ein
    // verstecktes Element zu verweisen ist zulässig -- ein ausgeklapptes
    // Menü zeigt per `aria-controls` auf seinen eingeklappten Zustand.
    // Ebenso entfällt der Deckel von 250 Befunden.
    SharedRule {
        id: "aria/reference-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (ARIA Reference)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::aria_required_attr` vollständig (axe-Kennung
    // `aria-required-attr`): die AX-Prüfung (combobox, heading, scrollbar)
    // und die DOM-Prüfungen `check_checked_state_with_page` (checkbox, radio,
    // switch) und `check_value_now_with_page` (meter, scrollbar, separator,
    // slider, spinbutton). Die geteilte Regel führt dieselbe Tabelle, nimmt
    // dieselben nativen Elemente aus (`<input type=checkbox|radio|range|
    // number>`, `<meter>`, `<progress>`), verlangt `aria-valuenow` am
    // `separator` ebenso nur, wenn er fokussierbar ist, und sieht wie
    // `check_value_now_with_page` in offene Shadow-Roots.
    //
    // Ein Unterschied, gewollt: `heading` ohne `aria-level` ist kein Befund
    // mehr. ARIA 1.2 gibt der Rolle die Vorgabe 2, das Attribut ist nicht
    // mehr erforderlich -- und Chrome füllt `level` im AX-Baum ohnehin, die
    // Prüfung schlug in der Praxis nie an.
    SharedRule {
        id: "aria/required-attribute-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Required ARIA Attribute)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::aria_hidden_focus` (DOM, axe-Kennung
    // `aria-hidden-focus`) und `wcag::rules::focus_order::check_focus_order`
    // (AX-Baum, `focus-order-semantics`). Beide meldeten denselben Fall --
    // fokussierbar unter `aria-hidden` --, einmal unter 4.1.2 und einmal
    // unter 2.4.3; jetzt steht er einmal im Bericht, unter 4.1.2 wie bei axe.
    //
    // Wie die DOM-Regel nimmt die geteilte aus, was `inert`, per
    // `tabindex="-1"` aus der Tabfolge genommen oder nicht dargestellt ist;
    // Letzteres über die berechneten Stile (`run_full_in`). Fehlen die --
    // der Layout-Abzug ist gescheitert --, sieht sie nur das
    // `hidden`-Attribut und meldet ein per CSS verstecktes Element mit.
    SharedRule {
        id: "keyboard/hidden-focusable",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Focusable in aria-hidden)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::landmarks` vollständig und aus
    // `wcag::rules::landmark_granular` die Prüfungen `landmark-main-present`,
    // `landmark-banner-present` und `landmark-no-duplicate-main`. Die übrigen
    // granularen Prüfungen (eindeutige Namen, Verschachtelung, doppelte
    // banner/contentinfo) sind seit #694 ebenfalls geteilt, siehe unten.
    //
    // Die fehlende main-Landmark meldeten bisher zwei Regeln, einmal unter
    // 2.4.1 (`landmark-one-main`) und einmal unter 1.3.1
    // (`landmark-main-present`). Jetzt einmal, unter 2.4.1 wie bei axe
    // `landmark-one-main`: Die main-Landmark ist das Sprungziel.
    //
    // Ein Unterschied, gewollt: Fehlende navigation-, banner- und
    // contentinfo-Landmark sind dort `REVIEW`, nicht `Violation`. Eine Seite
    // darf ohne Navigation oder Fußzeile auskommen -- das ist eine
    // Erwartung, kein beweisbarer Verstoß.
    SharedRule {
        id: "landmarks/main-missing",
        criterion: "2.4.1",
        level: WcagLevel::A,
        name: "Bypass Blocks (Main Landmark)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/bypass-blocks.html",
    },
    SharedRule {
        id: "landmarks/main-duplicate",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Multiple Main Landmarks)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "landmarks/banner-missing",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Banner Landmark)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "landmarks/contentinfo-missing",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Contentinfo Landmark)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "landmarks/navigation-missing",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Navigation Landmark)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `wcag::rules::svg_rules` (axe-Kennung `svg-img-alt`) und den
    // `<svg>`-Teil von `wcag::rules::text_alternatives`. Im AX-Baum ist ein
    // `<svg>` vom `<img>` und vom `<div role="img">` nicht zu unterscheiden,
    // deshalb meldete `image-alt` es bisher mit; die geteilte Regel prüft
    // das Element selbst. Die Pipeline nimmt die `image-alt`-Befunde an
    // `<svg>` heraus (`wcag::rules::is_svg_finding`).
    //
    // Dieselbe Ausnahme wie bisher: das Icon in einem benannten Link oder
    // Button braucht keinen eigenen Namen. Nicht mehr erfasst ist ein
    // Nicht-`<svg>`-Element mit `role="graphics-document"` oder
    // `"graphics-symbol"` -- etwa ein `<g>` im SVG. Die Rollen kommen in der
    // Praxis am `<svg>` selbst vor, und dort prüft die geteilte Regel.
    SharedRule {
        id: "svg/name-missing",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Non-text Content (SVG)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    // Ersetzt den `<img>`-Teil von `wcag::rules::text_alternatives`
    // (axe-Kennung `image-alt`). Dort blieben Elemente mit `role="img"`, die
    // weder `<img>` noch `<svg>` sind, und die Icon-Heuristik.
    //
    // Die geteilte Regel liest das `alt`-Attribut statt des Accessible Name:
    // Ein `<img>` ohne `alt`, aber mit `aria-label`, `aria-labelledby` oder
    // `title`, ist benannt und kein Befund (ARIA6, ARIA10, H67); eines mit
    // `role="presentation"` oder `aria-hidden="true"` ist erklärt dekorativ.
    // Das deckt sich mit der AX-Prüfung.
    SharedRule {
        id: "images/alt-missing",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Non-text Content",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    // Neu: ein `alt`, das das Bild vermutlich nicht beschreibt -- ein
    // Dateiname, „Bild", „Logo", ein oder zwei Zeichen. Heuristisch und
    // deshalb `REVIEW`. Die AX-Prüfung kannte nur „Name da oder nicht da".
    SharedRule {
        id: "images/alt-suspicious",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Non-text Content (Suspicious Alt Text)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    // Ersetzt `check_link` und `check_button` aus `wcag::rules::labels`
    // (axe-Kennung `control-missing-label`); `check_form_control` ist seit
    // #693 `forms/label-missing` und `names/required-missing`.
    // Beide Seiten fragen den Accessible Name: die eigene Regel den aus
    // Chromes AX-Baum, die geteilte den aus `CdpDocument`, der auf denselben
    // AX-Baum zurückgreift.
    //
    // Zwei Unterschiede, beide gewollt:
    //
    // - Als Link zählt dort nur `<a href>`, nicht jedes Element mit der
    //   AX-Rolle `link`. Ein unbenanntes `<span role="link">` meldet
    //   `names/required-missing` (#692).
    // - Beide sind dort `Critical` statt `High`: Ein unbenanntes
    //   Bedienelement ist für Screenreader-Nutzer nicht bedienbar.
    SharedRule {
        id: "links/name-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Link Name)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    SharedRule {
        id: "buttons/name-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Button Name)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt die Sprunglink-Erkennung aus `wcag::rules::bypass_blocks`.
    // Die eigene Regel suchte im Linknamen eine Liste von Wendungen in
    // vierzehn Sprachen; die geteilte erkennt den Sprunglink wie axe am Ziel
    // -- Links vor dem ersten, der die Seite verlässt, springen innerhalb
    // der Seite, in jeder Sprache (auditmysite#642) -- und nur ergänzend am
    // Text. Sie ist `REVIEW`: Kein Merkmal weist einen Sprunglink sicher aus.
    //
    // Die bisherige Sammelmeldung „weder Sprunglink noch main" entfällt: Sie
    // war die Verknüpfung dieser Prüfung mit `landmarks/main-missing`.
    SharedRule {
        id: "keyboard/skip-link-missing",
        criterion: "2.4.1",
        level: WcagLevel::A,
        name: "Bypass Blocks (Skip Link)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/bypass-blocks.html",
    }, // ── ARIA-Regeln aus #691 (B1) ──
    //
    // Alle unter 4.1.2 A wie die abgelösten Regeln, auch wo `a11y-rules`
    // zusätzlich 1.3.1 oder 2.1.1 führt. Kontext und Bestandteile prüft die
    // geteilte Fassung am DOM statt am AX-Baum; die Rolle eines Elements ohne
    // `role` kommt dabei aus Chromes AX-Baum, übersetzt in
    // `accessibility::dom_document` (`LayoutTable*` und andere
    // Chrome-Namen → keine Rolle, `image` → `img`).

    // Ersetzt `wcag::rules::aria_roles::check_invalid_role_with_page`
    // (axe-Kennung `aria-roles`, DOM per JavaScript). Zwei Unterschiede,
    // beide gewollt: Die geteilte Regel meldet jede ungültige Rolle der
    // Liste, nicht nur die erste, und vergleicht ohne Kleinschreibung --
    // `role="Button"` ist keine ARIA-Rolle. Neu ist die abstrakte Rolle
    // (`role="widget"`) als eigene Aussage; die eigene Regel hielt sie
    // ebenfalls für ungültig, nannte aber den Grund nicht.
    SharedRule {
        id: "aria/role-invalid",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Invalid ARIA Role)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    SharedRule {
        id: "aria/role-abstract",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Abstract ARIA Role)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `aria_roles::check_invalid_aria_attribute_name_with_page`
    // (`aria-attr-name-invalid`). Dieselbe Liste, ARIA 1.2 und die
    // Entwurfsattribute aus 1.3, die Browser schon umsetzen. Der Deckel von
    // 250 Befunden entfällt.
    SharedRule {
        id: "aria/attribute-unknown",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Unknown ARIA Attribute)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt die Wertebereiche aus `wcag::rules::aria_valid_attr_value`
    // (`aria-valid-attr-value`); die IDREF-Hälfte der Regel ist
    // `aria/reference-missing`. Drei Unterschiede, alle gewollt:
    //
    // - `aria-current` und `aria-invalid` mit unbekanntem Wert sind kein
    //   Befund: ARIA 1.2 legt fest, dass er als `true` gilt.
    // - Geprüft werden auch Ganzzahlen und Zahlen (`aria-level`,
    //   `aria-valuenow`, …) und `undefined` bei `aria-expanded`/`-selected`/
    //   `-hidden`/`-checked`/`-pressed` ist gültig.
    // - Vergleich ohne Groß-/Kleinschreibung, leere Werte ohne Befund.
    SharedRule {
        id: "aria/attribute-value-invalid",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Invalid ARIA Attribute Value)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::parsing::check_parsing` (`duplicate-id-aria`):
    // dieselbe ID in mehreren `aria-owns`. Die eigene Regel las die
    // Beziehung aus dem AX-Baum, die geteilte aus dem DOM, und meldet den
    // zweiten Besitzer statt des ersten.
    SharedRule {
        id: "aria/owns-conflict",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Conflicting aria-owns)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `widget_rules::check_tab_selected_state_with_page`
    // (`aria-tab-selected-state`) und die Tab-Prüfung aus
    // `patterns::tab_list` (`tab-no-aria-selected`) -- beide meldeten
    // denselben Fall, je Tab und zweimal. Ein Unterschied, gewollt: Die
    // geteilte Regel ist `REVIEW` und steht einmal an der Tabliste, wenn
    // **kein** Tab `aria-selected="true"` trägt; ein Tab ohne das Attribut
    // hat mit der Vorgabe `false` den richtigen Zustand (ARIA 1.2: SHOULD).
    SharedRule {
        id: "aria/tab-selected-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Selected Tab)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `widget_rules::check_tablist_has_tabpanel`
    // (`aria-tablist-tabpanel`). Jetzt `REVIEW` statt Verstoß: ARIA 1.2
    // beschreibt das Panel nur als üblich.
    SharedRule {
        id: "aria/tabpanel-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Tab Panel)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `widget_rules::check_combobox_has_options`
    // (`aria-combobox-options`). Gleiche Ausnahmen -- zugeklappt,
    // `aria-controls`, Popup im Teilbaum --, dazu `aria-owns`. Geprüft wird
    // nur eine explizite `role="combobox"`; ein natives `<select>` hat sein
    // Popup immer.
    SharedRule {
        id: "aria/combobox-popup-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Combobox Popup)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzen die Popover-Prüfung aus `wcag::rules::modern_attributes`
    // (`modern-attribute-misuse`, `popover_target_missing` und
    // `popover_target_invalid`) ohne Unterschied.
    SharedRule {
        id: "popover/target-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Popover Target)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    SharedRule {
        id: "popover/target-invalid",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Popover Target)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt aus `modern_attributes` den Fall `active_surface_inert` für
    // Dialoge. Zwei Unterschiede, beide gewollt: Die geteilte Regel sieht
    // auch ein `inert` an einem Vorfahren, und `role="dialog"` ist `REVIEW`
    // (ohne Stile ist ein geschlossener Dialog nicht von einem offenen zu
    // unterscheiden), nur `<dialog open>` ein Verstoß. Ein sichtbares
    // `role="menu"` mit `inert` ist kein Befund mehr -- ein aus dem Bild
    // geschobenes Menü ist so richtig gebaut. Das offene Popover mit
    // `inert`, den fehlenden Namen offener Dialoge und Popover und den Fokus
    // in einem inerten Teilbaum prüft `modern_attributes` weiter.
    SharedRule {
        id: "inert/dialog-inert",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Inert Dialog)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::aria_allowed_attr` (`aria-allowed-attr`). Zwei
    // Unterschiede, beide gewollt: Die geteilte Regel urteilt über die Rolle
    // des Elements, explizit oder implizit -- `<div aria-expanded>` ist ein
    // Befund, die eigene Regel sah nur explizite `role`-Angaben. Und sie
    // kennt die „MUST NOT"-Fälle aus ARIA in HTML (`aria-checked` an einer
    // nativen Checkbox, `aria-valuemax` neben `max`, …). Die implizite Rolle
    // kommt aus Chromes AX-Baum; Chrome-eigene Rollen bleiben ohne Urteil.
    SharedRule {
        id: "aria/attribute-not-allowed",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (ARIA Attribute Not Allowed)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::aria_prohibited_attr` (`aria-prohibited-attr`).
    // Dieselbe Tabelle; die eigene Regel urteilte über explizite Rollen und
    // `div`/`span`, die geteilte über jede Rolle, die der Host meldet.
    SharedRule {
        id: "aria/attribute-prohibited",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Prohibited ARIA Attribute)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::aria_required_parent` (`aria-required-parent`).
    // Die eigene Regel ging im AX-Baum alle Vorfahren durch, die geteilte
    // nimmt den nächsten Vorfahren mit einer Rolle (durchlässig sind
    // `generic`, `none`/`presentation` und Ignoriertes). Damit meldet sie wie
    // axe den Fall aus #715: `ul[role=tablist] > li > a[role=tab]` -- Chrome
    // glättet das `<li>` im AX-Baum, im DOM steht es als `listitem` zwischen
    // Tabliste und Tab. Stößt der Weg auf eine Rolle außerhalb von ARIA 1.2,
    // entsteht kein Befund.
    SharedRule {
        id: "aria/required-parent-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Required Parent Role)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt die Bestandteilprüfung aus `wcag::rules::aria_roles`
    // (`aria-roles`). Ein Unterschied, gewollt: Geprüft werden nur Behälter
    // mit expliziter Rolle; native Tabellen und Listen geben ihre
    // Bestandteile per HTML vor (#659, #674). Eine native Tabelle nur mit
    // `<caption>` ist deshalb kein Befund mehr (Korpus
    // `table_required_rows_tbody`).
    SharedRule {
        id: "aria/required-children-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Required Owned Elements)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // ── Namensregeln aus #692 (B2) ──
    //
    // Kriterium und Stufe wie bei den abgelösten Regeln, die Schwere setzt
    // `a11y-rules` gleich der bisherigen. Die geteilte Fassung fragt den
    // Accessible Name aus `CdpDocument`, der auf Chromes AX-Baum zurückgreift
    // -- dieselbe Quelle wie bisher.

    // Ersetzt `wcag::rules::aria_naming_rules` ohne den Dialog
    // (`aria-command-name` ohne `button` und `a[href]`,
    // `aria-input-field-name`, `aria-meter-name`, `aria-progressbar-name`,
    // `aria-toggle-field-name`, `aria-treeitem-name`) und aus
    // `wcag::rules::accessible_name` den Fall „kein Name" (`aria-label`).
    // Beide meldeten dasselbe Element bisher doppelt. Unterschiede, alle
    // gewollt:
    //
    // - Unbenannte Buttons und Links melden `buttons/name-missing` und
    //   `links/name-missing`, ein Formularfeld ganz ohne Beschriftung
    //   `forms/label-missing`; hier steht nur das Feld, dessen Beschriftung
    //   behauptet ist, aber leer ausgeht (`<label for>` ohne Text).
    // - `menu` und `tab` sind kein Befund mehr: ARIA 1.2 verlangt für beide
    //   keinen Namen. Ebenso die native `<option>`.
    // - Ein unbenanntes, nicht fokussierbares Element ist ein Verstoß wie in
    //   `aria_naming_rules`, nicht die Warnung aus `accessible_name`.
    // - `meter` und `progressbar` stehen unter 4.1.2 statt 1.1.1: Die Tabelle
    //   führt ein Kriterium je Kennung. `a11y-rules` gibt dem Befund 1.1.1
    //   mit, auditmysite meldet ihn wie alle Namensbefunde unter 4.1.2.
    SharedRule {
        id: "names/required-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Accessible Name)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt aus `accessible_name` den Fall „Icon Only" (`aria-label`): ein
    // Name aus einem Attribut, der nur ein Symbol ist. Jetzt `REVIEW` statt
    // Verstoß -- 4.1.2 verlangt einen Namen, über seine Güte sagt es nichts.
    // Buchstaben zählen nach Unicode, „Ä" ist kein Symbol mehr.
    SharedRule {
        id: "names/symbol-only",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Symbol-Only Name)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::dialog_rules` (`dialog-name`), den Dialog aus
    // `aria_naming_rules` (`aria-dialog-name`) und die Namensprüfung aus
    // `patterns::modal_dialog` (ebenfalls `aria-dialog-name`) -- derselbe
    // Fall stand bisher bis zu dreimal im Bericht. Ein geschlossenes
    // `<dialog>` ohne `open` wird nicht geprüft.
    SharedRule {
        id: "dialog/name-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Dialog Name)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt aus `dialog_rules` den Fall „Dialog Modal". Zwei Unterschiede,
    // beide gewollt: `REVIEW` statt Verstoß -- ARIA verlangt `aria-modal`
    // nicht, ein nicht modaler Dialog trägt es zu Recht nicht --, und nur
    // `role="dialog"`: Ob ein natives `<dialog>` per `showModal()` offen ist,
    // sieht der Abzug nicht.
    SharedRule {
        id: "dialog/modal-unmarked",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Dialog Modal)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::summary_name` (`summary-name`). Gemeldet wird nur
    // die `<summary>`, die ihr `<details>` bedient; die abgelöste Regel
    // meldete zusätzlich das `<details>`, dessen Name aus derselben
    // `<summary>` stammt. `buttons/name-missing` meldet die `<summary>` mit
    // derselben a11y-rules-Version nicht mehr mit.
    SharedRule {
        id: "summary/name-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Summary Name)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::status_messages` (`aria-live-region-role`).
    // Schwere wie bisher (`alert` hoch, `status` und `log` mittel). Ein
    // Unterschied, gewollt: Verstoß ist nur noch `aria-live="off"`, die
    // Region wird dann nicht angesagt. Eine andere Dringlichkeit (`alert` mit
    // `polite`, `status` mit `assertive`) ist `REVIEW` -- ARIA erlaubt das
    // Überschreiben, angesagt wird weiterhin. Implizite Rollen zählen mit
    // (`<output>` ist `status`).
    SharedRule {
        id: "status/live-overridden",
        criterion: "4.1.3",
        level: WcagLevel::AA,
        name: "Status Messages",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html",
    },
    // Ersetzt `wcag::rules::label_in_name` (`label-content-name-mismatch`,
    // DOM per JavaScript, höchstens 50 Buttons). Gleich geblieben: kein
    // Befund, wenn der Name im sichtbaren Text steht, und `REVIEW`, wenn der
    // sichtbare Text mehr als doppelt so lang ist wie der Name (#513).
    // Unterschiede, alle gewollt: geprüft werden alle Rollen mit Namen aus
    // dem Inhalt (Link, Menüeintrag, Tab, Checkbox, …) und Namen aus
    // `aria-labelledby`, nicht nur Buttons mit `aria-label`; verglichen wird
    // der berechnete Name, Buchstaben nach Unicode statt nur ASCII; ohne
    // Deckel. Visuell versteckter Text (`.sr-only`) zählt als sichtbar.
    SharedRule {
        id: "label-in-name/mismatch",
        criterion: "2.5.3",
        level: WcagLevel::A,
        name: "Label in Name",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/label-in-name.html",
    },
    // ── Formularregeln aus #693 (B3) ──
    //
    // Kriterium und Stufe wie bei den abgelösten Regeln, auch wo
    // `a11y-rules` mehrere Kriterien führt; die Schwere setzt `a11y-rules`.
    // Einzelheiten und alle Abweichungen im Changelog von `a11y-rules`
    // (casoon/barrierlab#16).

    // Ersetzt aus `wcag::rules::instructions` den Fall „no accessible label"
    // (`label`, 3.3.2, kritisch) und `wcag::rules::labels::check_form_control`
    // (`control-missing-label`, 4.1.2, hoch) -- dasselbe Feld stand bisher
    // zweimal im Bericht, jetzt einmal unter 3.3.2. Ein Unterschied, gewollt:
    // Die geteilte Regel prüft native Felder auf eine Beschriftungsquelle
    // (`<label>`, `aria-label`, `aria-labelledby`, `title`). ARIA-Widgets
    // ohne Namen, ein leeres oder ins Leere zeigendes `aria-labelledby` und
    // ein leeres `<label for>` meldet `names/required-missing`.
    SharedRule {
        id: "forms/label-missing",
        criterion: "3.3.2",
        level: WcagLevel::A,
        name: "Labels or Instructions (Missing Label)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html",
    },
    // Ersetzt aus `instructions` den Fall „Placeholder used as only label".
    // Die eigene Regel las Chromes `name_source`, die geteilte das Markup:
    // `placeholder` ohne `<label>`, `aria-label` oder `aria-labelledby`. Ein
    // Feld mit `title` und `placeholder` meldet sie hier, nicht unter
    // `forms/title-only-label`.
    SharedRule {
        id: "forms/placeholder-as-label",
        criterion: "3.3.2",
        level: WcagLevel::A,
        name: "Labels or Instructions (Placeholder as Label)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html",
    },
    // Ersetzt aus `wcag::rules::input_purpose` (`autocomplete-valid`) den
    // ungültigen Wert. Gewollt anders: Geprüft wird die ganze Grammatik des
    // HTML-Standards (`section-*`, `shipping`/`billing`, Kontaktart,
    // `webauthn`), nicht nur das letzte Token, und die Schwere ist niedrig
    // statt mittel.
    SharedRule {
        id: "forms/autocomplete-invalid",
        criterion: "1.3.5",
        level: WcagLevel::AA,
        name: "Identify Input Purpose (Invalid Autocomplete)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/identify-input-purpose.html",
    },
    // Ersetzt aus `input_purpose` das fehlende `autocomplete`
    // (`autocomplete-valid`, Verstoß) und `wcag::rules::identify_purpose`
    // (`identify-purpose`, 1.3.6 AAA, niedrig), die dieselben Felder nach
    // `id`/`name` ein zweites Mal meldete. Gewollt anders: `REVIEW` statt
    // Verstoß -- ob ein Feld die Person betrifft, ist aus Beschriftung, `id`
    // und `name` geraten --, und einmal unter 1.3.5 AA. Der seitenweite
    // `UNTESTED`-Vermerk aus `identify_purpose` (1.3.6 für Symbole und
    // Bereiche) entfällt.
    SharedRule {
        id: "forms/purpose-missing",
        criterion: "1.3.5",
        level: WcagLevel::AA,
        name: "Identify Input Purpose",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/identify-input-purpose.html",
    },
    // Ersetzt `input-error-message` aus `wcag::rules::form_rules` und
    // `wcag::rules::error_identification` (`aria-invalid-without-describedby`),
    // die dasselbe Feld zweimal meldeten. Als Beschreibung zählen
    // `aria-describedby` und `aria-errormessage` mit Text; ein verstecktes
    // oder fehlendes Ziel gilt als Beschreibung (das Fehlen meldet
    // `aria/reference-missing`).
    SharedRule {
        id: "forms/error-unidentified",
        criterion: "3.3.1",
        level: WcagLevel::A,
        name: "Error Identification",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/error-identification.html",
    },
    // Ersetzt `form-field-group` aus `form_rules`, den AX-Teil (Optionsfelder)
    // und den DOM-Teil (gleichnamige Kontrollkästchen, #643). Gruppe ist ein
    // `<fieldset>`, `<details>` oder `role="group"`/`"radiogroup"` als
    // Vorfahre.
    SharedRule {
        id: "forms/group-missing",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Form Field Group)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt aus `instructions` den Fall „Form group has no legend or label"
    // (`label`). Ein unbenanntes `radiogroup` meldet nicht mehr diese Regel,
    // sondern `names/required-missing`.
    SharedRule {
        id: "forms/group-name-missing",
        criterion: "3.3.2",
        level: WcagLevel::A,
        name: "Labels or Instructions (Group Name)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html",
    },
    // Ersetzt „Required field not clearly indicated" aus `instructions` und
    // „may not indicate required status" aus `form_rules` (beide `label`),
    // die dasselbe Feld zweimal meldeten. Gewollt anders: `REVIEW` statt
    // Verstoß und einmal -- der Screenreader sagt das Pflichtfeld an; ob es
    // sichtbar gekennzeichnet ist, sieht nur ein Mensch.
    SharedRule {
        id: "forms/required-unmarked",
        criterion: "3.3.2",
        level: WcagLevel::A,
        name: "Labels or Instructions (Required Field)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html",
    },
    // Ersetzt „may require format instructions" aus `instructions` (`label`,
    // `REVIEW`) samt den Korrekturen aus #643 und #656.
    SharedRule {
        id: "forms/instructions-missing",
        criterion: "3.3.2",
        level: WcagLevel::A,
        name: "Labels or Instructions (Format Instructions)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html",
    },
    // Ersetzt `wcag::rules::label_title_only` (`label-title-only`). Gewollt
    // anders: `REVIEW` statt Verstoß -- `title` ist eine zulässige Technik
    // (H65).
    SharedRule {
        id: "forms/title-only-label",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Title-Only Label)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `form-no-submit` aus `form_rules`, samt der Korrektur aus #728
    // (ohne `action` und ohne Textfeld nur `REVIEW`, niedrig). Neu: Felder
    // und Buttons, die per `form`-Attribut außerhalb stehen, zählen mit.
    SharedRule {
        id: "forms/no-submit",
        criterion: "3.2.2",
        level: WcagLevel::A,
        name: "On Input (Form Without Submit)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/on-input.html",
    },
    // Ersetzt `wcag::rules::redundant_entry` (`redundant-entry`), `REVIEW`
    // wie bisher.
    SharedRule {
        id: "forms/redundant-entry",
        criterion: "3.3.7",
        level: WcagLevel::A,
        name: "Redundant Entry",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/redundant-entry.html",
    },
    // Ersetzt aus `wcag::rules::on_input` (`input-no-context-change`) das
    // Urteil über den Handlertext im Markup. Neu: auch Optionsfelder mit
    // `onchange` (F37). In `on_input` bleiben, weil sie die laufende Seite
    // brauchen: der Quelltext einer aufgerufenen Funktion über `window` und
    // die Namensvermutung („Language") ohne Absende-Button.
    SharedRule {
        id: "context/on-input",
        criterion: "3.2.2",
        level: WcagLevel::A,
        name: "On Input",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/on-input.html",
    },
    // Ersetzen `wcag::rules::on_focus` (`focus-no-context-change`), das
    // `onfocus` und das `autofocus` jetzt unter je eigener Kennung. Gewollt
    // anders: beide `REVIEW` statt Verstoß -- ob ein `onfocus` den Kontext
    // wechselt, steht nicht im Markup, und `autofocus` wechselt ihn für sich
    // genommen nicht. Schwere wie bisher (hoch, mittel).
    SharedRule {
        id: "context/on-focus",
        criterion: "3.2.1",
        level: WcagLevel::A,
        name: "On Focus",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/on-focus.html",
    },
    SharedRule {
        id: "context/autofocus",
        criterion: "3.2.1",
        level: WcagLevel::A,
        name: "On Focus (Autofocus)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/on-focus.html",
    },
    // Ersetzt das Captcha aus `wcag::rules::accessible_authentication`
    // (`accessible-auth-captcha`, `REVIEW`). Der Einfüge-Test an Passwort-
    // und Einmalcode-Feldern braucht die laufende Seite und bleibt dort.
    SharedRule {
        id: "auth/captcha",
        criterion: "3.3.8",
        level: WcagLevel::AA,
        name: "Accessible Authentication (Minimum)",
        help_url:
            "https://www.w3.org/WAI/WCAG22/Understanding/accessible-authentication-minimum.html",
    },
    // ── Landmark-, Tastatur- und Strukturregeln aus #694 (B4) ──
    //
    // Kriterium und Stufe wie bei den abgelösten Regeln, die Schwere setzt
    // `a11y-rules`. Einzelheiten und alle Abweichungen im Changelog von
    // `a11y-rules` (casoon/barrierlab#17). Die Landmark-Rolle bestimmt die
    // geteilte Fassung aus dem Markup, nicht aus Chromes Rolle: Ein
    // unbenanntes `<form>`/`<section>` ist keine Landmark (#727), ein
    // `<header>`/`<footer>` in `main`, `article` oder unter `role="main"`
    // kein banner/contentinfo (#639).

    // Ersetzt `landmark-unique` aus `wcag::rules::landmark_granular`. Gewollt
    // anders: `REVIEW` statt Verstoß -- WAI-ARIA und die APG verlangen
    // unterscheidbare Namen nur als SHOULD. Ein `<aside>` in einem Abschnitt
    // zählt nur mit Namen als `complementary`.
    SharedRule {
        id: "landmarks/not-unique",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Landmark Unique)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `landmark-banner-is-top-level`,
    // `landmark-contentinfo-is-top-level` und `landmark-main-is-top-level`
    // aus `landmark_granular` -- eine Kennung, die Rolle steht im Text.
    SharedRule {
        id: "landmarks/not-top-level",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Landmark Not Top Level)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzen `landmark-no-duplicate-banner` und
    // `landmark-no-duplicate-contentinfo` aus `landmark_granular`. Gewollt
    // anders: Der Befund steht wie bei `landmarks/main-duplicate` an der
    // zweiten Landmark, nicht an der ersten.
    SharedRule {
        id: "landmarks/banner-duplicate",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Multiple Banner Landmarks)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "landmarks/contentinfo-duplicate",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Multiple Contentinfo Landmarks)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `wcag::rules::region` (`region`). Gewollt anders: gemeldet wird
    // das äußerste Element ohne Landmark darin, einmal je Block wie axe
    // `region` -- nicht jeder Textknoten und jedes benannte Element einzeln.
    // Die Befundzahl je Seite sinkt damit. Sprunglinks erkennt die geteilte
    // Fassung wie bisher am Ziel (#642).
    SharedRule {
        id: "landmarks/content-outside",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Content Outside Landmarks)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `wcag::rules::bypass_blocks` (`bypass`, „No headings found")
    // samt der Ausnahme aus #709: hinter einem offenen Dialog `REVIEW`,
    // niedrig. Neu: `role="heading"` zählt mit, versteckte Überschriften
    // nicht.
    SharedRule {
        id: "headings/none",
        criterion: "2.4.1",
        level: WcagLevel::A,
        name: "Bypass Blocks (No Headings)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/bypass-blocks.html",
    },
    // Ersetzt aus `wcag::rules::keyboard` `focusable-no-role`. Gewollt
    // anders: nur die Tabfolge (`tabindex` ≥ 0) zählt -- ein `tabindex="-1"`
    // am Ziel eines Sprunglinks erreicht niemand per Tab --, und ein
    // benannter Bereich mit `tabindex="0"` (scrollbarer Bereich) ist kein
    // Befund.
    SharedRule {
        id: "keyboard/focusable-no-role",
        criterion: "2.1.1",
        level: WcagLevel::A,
        name: "Keyboard (Focusable Without Role)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html",
    },
    // Ersetzt aus `keyboard` „appears not keyboard-focusable" (`keyboard`).
    // `REVIEW`, hoch wie bisher. Gewollt anders: Fokussierbarkeit kommt aus
    // dem Markup statt aus Chromes `focusable`; ausgenommen sind
    // deaktivierte Felder, native `<option>`, Elemente unter
    // `aria-activedescendant` und Inertes. Die Tastaturfalle (2.1.2) braucht
    // echte Bedienung und bleibt in `keyboard`.
    SharedRule {
        id: "keyboard/interactive-not-focusable",
        criterion: "2.1.1",
        level: WcagLevel::A,
        name: "Keyboard (Interactive Not Focusable)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html",
    },
    // Ersetzt `dialog-no-focusable` aus `patterns::modal_dialog`. Gewollt
    // anders: Gesucht wird in allen Nachfahren, nicht nur in den direkten
    // Kindern, und ein geschlossenes `<dialog>` zählt nicht.
    SharedRule {
        id: "dialog/focusable-missing",
        criterion: "2.4.3",
        level: WcagLevel::A,
        name: "Focus Order (Dialog Without Focusable Element)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/focus-order.html",
    },
    // Ersetzt `accordion-no-controls` aus `patterns::accordion`. Gewollt
    // anders: `REVIEW` statt Verstoß -- WAI-ARIA verlangt `aria-controls` am
    // Button nicht, die APG nennt es beim Disclosure-Muster optional.
    // Ausgenommen wie bisher: zugeklappte Buttons, `<summary>` und Buttons in
    // `navigation`/`banner`.
    //
    // Nicht übernommen und ersatzlos gelöscht: `accordion-trigger-not-button`
    // (meldet an Rollen ohne `aria-expanded` schon
    // `aria/attribute-not-allowed`, an `link`/`tab`/`treeitem` erlaubt
    // WAI-ARIA den Zustand, fehlender Fokus ist
    // `keyboard/interactive-not-focusable`) und `aria-expanded-required`
    // aus `patterns::disclosure_menu` (riet ein Aufklappmenü aus dem Wort
    // „menu"/„Menü" im Namen, ohne Norm dahinter).
    SharedRule {
        id: "patterns/accordion-controls-missing",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Accordion Controls)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // ── Links- und Zeigerregeln aus #695 (B5) ──
    //
    // Einzelheiten und Abweichungen im Changelog von `a11y-rules` 0.18.0
    // (casoon/barrierlab#18). Gewollt anders für alle drei: keine Obergrenze
    // je Seite mehr (bisher 10 bzw. 20 Befunde).

    // Ersetzt `wcag::rules::click_handlers` (`click-events-have-key-events`).
    // Gewollt anders: Ein `<a onclick>` ohne `href` landet hier statt bei
    // `links/used-as-button` -- ohne `href` ist es kein Link und nicht
    // fokussierbar.
    SharedRule {
        id: "keyboard/click-handler-not-focusable",
        criterion: "2.1.1",
        level: WcagLevel::A,
        name: "Keyboard",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html",
    },
    // Ersetzt `wcag::rules::fake_navigation_link` (`link-as-button`).
    SharedRule {
        id: "links/used-as-button",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::location` (`location`). Gewollt anders: `REVIEW`
    // statt Verstoß -- 2.4.8 lässt sich auch mit Titel, Überschriften oder
    // einer Sitemap erfüllen. AAA: läuft wie bisher nur mit `--level aaa`
    // ([`retain_up_to_level`]).
    //
    // `wcag::rules::pointer_cancellation` (2.5.2) ist nicht abgelöst:
    // `a11y-rules` hat den statischen Teil (`onmousedown`/`ontouchstart`)
    // mangels Beleg nicht übernommen, er ist hier gelöscht. Der seitenweite
    // `UNTESTED`-Vermerk bleibt im Host, bis die manuelle Checkliste
    // (casoon/barrierlab#39) einen Punkt für 2.5.2 hat.
    SharedRule {
        id: "navigation/location-missing",
        criterion: "2.4.8",
        level: WcagLevel::AAA,
        name: "Location",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/location.html",
    },
    // ── Bild- und Medienregeln aus #696 (B6) ──
    //
    // Einzelheiten und Abweichungen im Changelog von `a11y-rules` 0.18.0
    // (casoon/barrierlab#19). Im Host bleiben, weil nur der Browser es weiß:
    // ob eine `<track>`-Datei lädt (`video-caption`, Netzabruf) und welche
    // Rahmen fremd sind (`frame-tested`), dazu der Hinweis `media-alt`
    // (1.2.8) -- `a11y-rules` verweist dafür auf `manual/media-alternatives`,
    // und die manuelle Checkliste führt auditmysite noch nicht.

    // Ersetzen `area-alt`, `input-image-alt` und `object-alt` aus
    // `wcag::rules::image_input_rules`. Gewollt anders: `aria-labelledby`
    // zählt an allen dreien, `aria-label` auch an `<area>`; `<embed>` prüft
    // `objects/alt-missing` nicht (kein Beleg). `images/area-alt-missing`
    // erst ab a11y-rules 0.19: davor nahm die Sicht mit berechneten Stilen
    // `<area>` heraus (UA `display: none`, Korpus `misc_content_checks`).
    SharedRule {
        id: "images/area-alt-missing",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Non-text Content (Area)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    SharedRule {
        id: "images/input-alt-missing",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Non-text Content (Image Button)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    SharedRule {
        id: "objects/alt-missing",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Non-text Content (Object)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    // Ersetzt `wcag::rules::server_side_image_map`.
    SharedRule {
        id: "images/server-side-map",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Non-text Content (Server-side Image Map)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    // Ersetzt `wcag::rules::background_audio` (`background-audio`, 1.4.7
    // AAA), ab a11y-rules 0.19 (davor fehlte `<audio>` ohne `controls` in
    // der Sicht, Korpus `media_and_visual`). Gewollt anders: `REVIEW` und
    // 1.4.2 (Audio Control, A) statt 1.4.7 -- selbststartender Ton ist
    // Gegenstand von 1.4.2, ob er länger als drei Sekunden läuft, steht
    // nicht im Markup. Läuft damit schon ab Stufe A.
    SharedRule {
        id: "media/audio-autoplay",
        criterion: "1.4.2",
        level: WcagLevel::A,
        name: "Audio Control",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/audio-control.html",
    },
    // Ersetzt `frame-title` aus `wcag::rules::media_rules`. `a11y-rules`
    // führt 2.4.1 und 4.1.2 (H64); auditmysite meldete unter 2.4.1 und
    // bleibt dabei. Gewollt anders: nur `<iframe>`; unsichtbar heißt
    // `hidden`/`aria-hidden`, per Stil ausgeblendet (mit Layout-Stilen) oder
    // Breite und Höhe als Attribut 0/1 -- die gerenderte Größe ≤ 1 px kennt
    // die Regel ohne Geometrie nicht.
    SharedRule {
        id: "frames/name-missing",
        criterion: "2.4.1",
        level: WcagLevel::A,
        name: "Frame title",
        help_url: "https://www.w3.org/WAI/WCAG22/Techniques/html/H64",
    },
    // ── Tabellen-, Dokument-, Sprach- und Rollenregeln aus #697 (B7) ──
    //
    // Einzelheiten und Abweichungen im Changelog von `a11y-rules` 0.18.0
    // (casoon/barrierlab#20). Für alle gilt: keine Obergrenze je Seite mehr
    // (bisher 5, 10 bzw. 20 Befunde).
    //
    // Nicht übernommen und gelöscht: `presentation-semantic-children`
    // (`info_relationships`) -- nach WAI-ARIA 1.2 nimmt
    // `role="presentation"`/`"none"` nur dem Element selbst die Semantik,
    // nicht seinen Nachfahren; die Regel hätte das APG-Menüleistenmuster
    // (`<li role="none">` um Menülinks) als Fehler gemeldet. Den
    // Tabellenfall meldet `tables/presentational-with-headers`. Aus
    // `section_headings` die Gliederungslücken (meldet
    // `headings/skip-level`) und „mehr als 10 Absätze, weniger als 3
    // Überschriften" (kein Beleg).
    //
    // Im Host bleiben: aus `timing_adjustable` die seitenweiten
    // `UNTESTED`-Vermerke für Skript-Fristen (2.2.1) und `timeouts` (2.2.6),
    // die `a11y-rules` der manuellen Checkliste zuweist.

    // Ersetzt `th-has-data-cells` aus `wcag::rules::table_extended` samt
    // #638, #654 und #659. Ein noch nicht dargestellter Zeilenvorrat ist
    // `tables/data-undetermined` (`UNTESTED`), nicht `FAIL`.
    SharedRule {
        id: "tables/header-without-data",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Header Without Data)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    SharedRule {
        id: "tables/data-undetermined",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Header Without Data)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `td-headers-attr` aus `table_extended`. Gewollt anders: nur an
    // Zellen geprüft, wo HTML das Attribut definiert.
    SharedRule {
        id: "tables/headers-attr-invalid",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Info and Relationships (Headers Attribute)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
    // Ersetzt `wcag::rules::language_extended` (`html-xml-lang-mismatch`).
    SharedRule {
        id: "document/lang-mismatch",
        criterion: "3.1.1",
        level: WcagLevel::A,
        name: "Language of Page (lang/xml:lang Mismatch)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/language-of-page.html",
    },
    // Ersetzt `wcag::rules::language_of_parts`. Gewollt anders: der innerste
    // Textblock, ohne `<script>`/`<style>`/`<template>`; auf Seiten, die
    // weder Deutsch noch Englisch sind, `language/part-undetermined`
    // (`UNTESTED`) statt nichts.
    SharedRule {
        id: "language/part-unmarked",
        criterion: "3.1.2",
        level: WcagLevel::AA,
        name: "Language of Parts",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/language-of-parts.html",
    },
    SharedRule {
        id: "language/part-undetermined",
        criterion: "3.1.2",
        level: WcagLevel::AA,
        name: "Language of Parts",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/language-of-parts.html",
    },
    // Ersetzt `wcag::rules::abbreviations`. Gewollt anders: `REVIEW` statt
    // Verstoß, ein leeres `title` zählt als fehlend.
    SharedRule {
        id: "language/abbreviation-unexpanded",
        criterion: "3.1.4",
        level: WcagLevel::AAA,
        name: "Abbreviations",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/abbreviations.html",
    },
    // Ersetzt aus `wcag::rules::timing_adjustable` die Erkennung von
    // `<meta http-equiv="refresh">`. Gewollt anders: `0` s ist keine Frist
    // (H76), eine Anweisung ohne Ziffern führt der Browser nicht aus.
    SharedRule {
        id: "timing/meta-refresh",
        criterion: "2.2.1",
        level: WcagLevel::A,
        name: "Timing Adjustable",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/timing-adjustable.html",
    },
    // Ersetzt die Abschnittszählung aus `wcag::rules::section_headings`
    // (`heading-order`, 2.4.10). Gewollt anders: Artikel und benannte
    // Abschnitte ohne eigene Überschrift statt aller Abschnitte gegen alle
    // Überschriften; `REVIEW` statt Verstoß.
    SharedRule {
        id: "headings/section-without-heading",
        criterion: "2.4.10",
        level: WcagLevel::AAA,
        name: "Section Headings",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/section-headings.html",
    },
    // Ersetzt `wcag::rules::redundant_role` bis auf `<ul>`/`<ol>` (dafür
    // `lists/role-redundant`, siehe unten). `<li role="listitem">` meldet es
    // nur in einer Liste ohne eigene Rolle.
    SharedRule {
        id: "aria/role-redundant",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzen `title-only-description` und `content-on-hover-focus` aus
    // `wcag::rules::content_on_hover`. Gewollt anders: `names/title-only` ist
    // `REVIEW` (`title` ist eine gültige Namensquelle, H65), Textfelder
    // meldet `forms/title-only-label`; `patterns/tooltip-unreferenced` lässt
    // auch `aria-labelledby` als Verweis gelten.
    SharedRule {
        id: "names/title-only",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value (Title Only)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    SharedRule {
        id: "patterns/tooltip-unreferenced",
        criterion: "1.4.13",
        level: WcagLevel::AA,
        name: "Content on Hover or Focus",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/content-on-hover-or-focus.html",
    },
    // ── Regeln über Stylesheets (a11y-rules 0.19, barrierlab#21 erster Teil,
    // Host-Seite von #698) ──
    //
    // Laufen nur mit den Sheets der Seite (`run_shared_rules_with_stylesheets`,
    // gelesen über `document.styleSheets` wie die abgelösten
    // JavaScript-Regeln); ohne sie `NotRun::CapabilityMissing`. In Frames
    // laufen sie nicht -- sie urteilen über die Seite, die abgelösten Regeln
    // liefen ebenfalls nur im obersten Dokument. Abweichungen im Changelog
    // von `a11y-rules` 0.19.0.

    // Darstellungsregeln mit berechneten Stilen und Geometrie aus dem
    // DOMSnapshot (`RenderedCdpDocument`, #698). Fehlt einer Seite ein Feld
    // -- etwa weil der Snapshot scheiterte --, meldet die Regel das als
    // `UNTESTED` für die Seite statt zu schweigen.
    //
    // Ersetzt aus `wcag::rules::redundant_role` das Paar `<ul>`/`<ol>` mit
    // `role="list"`: überflüssig nur, wenn `list-style-type` nicht `none` ist
    // (#644).
    SharedRule {
        id: "lists/role-redundant",
        criterion: "4.1.2",
        level: WcagLevel::A,
        name: "Name, Role, Value",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    },
    // Ersetzt `wcag::rules::use_of_color` (`link-in-text-block`) samt #710:
    // nur Links im Fließtext, ohne Navigation, Menüs, Logo-Links und
    // Listeneinträge aus nur dem Link. Gewollt anders: auch Schriftfamilie,
    // Unterkante und Hintergrund zählen als Unterscheidung.
    SharedRule {
        id: "color/link-indistinct",
        criterion: "1.4.1",
        level: WcagLevel::A,
        name: "Use of Color",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html",
    },
    // Ersetzt `wcag::rules::scrollable_region` (`scrollable-region-focusable`,
    // #717): Überhang aus den DOM-Rechtecken des Snapshots, 13 px Puffer wie
    // axe.
    SharedRule {
        id: "keyboard/scrollable-region-not-focusable",
        criterion: "2.1.1",
        level: WcagLevel::A,
        name: "Keyboard",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html",
    },
    // Ersetzen `wcag::rules::contrast` (`color-contrast`, #698): Farben,
    // effektiver Hintergrund, optisch verborgener und überdeckter Text aus
    // dem DOMSnapshot (`RenderedCdpDocument`). Verglichen am Korpus und an
    // fünf Live-Seiten, AA und AAA (barrierlab#47). Ein nicht bestimmbarer
    // Hintergrund (Bild, Verlauf, positioniertes Bild darunter) wird wie
    // zuvor am Bildschirmfoto abgetastet (`Rendering::sampled_backdrop`).
    // Gewollt anders: Text, den ein fixiertes oder klebendes Element verdeckt
    // (Cookie-Banner), ist `UNTESTED` (#716 Fall 7); Text unter `aria-hidden` wird
    // gemessen (#395 nahm ihn aus — 1.4.3 gilt für sichtbaren Text). AAA
    // (1.4.6) steht unter eigener Kennung und meldet nur, was 1.4.3 besteht.
    SharedRule {
        id: "contrast/text-insufficient",
        criterion: "1.4.3",
        level: WcagLevel::AA,
        name: "Contrast (Minimum)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html",
    },
    SharedRule {
        id: "contrast/text-undetermined",
        criterion: "1.4.3",
        level: WcagLevel::AA,
        name: "Contrast (Minimum)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html",
    },
    SharedRule {
        id: "contrast/text-enhanced",
        criterion: "1.4.6",
        level: WcagLevel::AAA,
        name: "Contrast (Enhanced)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/contrast-enhanced.html",
    },
    // Ersetzt `wcag::rules::focus_visible_css` (`focus-visible-outline-none`).
    // Gewollt anders: Ein in einer `:focus`-Regel wieder gesetzter Rahmen
    // zählt als Ersatz (sueddeutsche.de setzt ihn für die Tastatur neu).
    SharedRule {
        id: "focus/outline-removed",
        criterion: "2.4.7",
        level: WcagLevel::AA,
        name: "Focus Visible (CSS outline suppression)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/focus-visible.html",
    },
    // Ersetzt `wcag::rules::reduced_motion` (`prefers-reduced-motion`), ab
    // a11y-rules 0.19.1: 0.19.0 las aus Chromes serialisierter Kurzform
    // `animation` (`2s linear 0s infinite normal none running spin`) `none`
    // als Namen. Gewollt anders: `REVIEW` statt Verstoß; eine allein
    // stehende `transition-duration` ist kein Übergang. AAA: nur mit
    // `--level aaa`.
    SharedRule {
        id: "motion/reduced-motion-ignored",
        criterion: "2.3.3",
        level: WcagLevel::AAA,
        name: "Animation from Interactions",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html",
    },
    // Ersetzt den Stylesheet-Teil von `wcag::rules::orientation`
    // (`css-orientation-lock`). Gewollt anders: `REVIEW` statt Verstoß, nur
    // Selektoren, die ein Element der Seite treffen, ohne Pseudo-Elemente
    // (Breakpoint-Marker `body:before` auf bundesregierung.de). Der
    // berechnete `transform: rotate` an `body`/`html` bleibt in
    // `orientation`.
    SharedRule {
        id: "orientation/content-hidden",
        criterion: "1.3.4",
        level: WcagLevel::AA,
        name: "Orientation",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/orientation.html",
    },
    // Ersetzen Blocksatz und Zeilenabstand aus
    // `wcag::rules::visual_presentation` (`visual-presentation`). Gewollt
    // anders: gemessen an den `<p>` der Seite statt an Selektoren, die mit
    // `body`, `p`, `div` … beginnen (sonst meldete normalize.css jede
    // Seite); `REVIEW` statt Verstoß. Der `UNTESTED`-Vermerk zu Farbwahl und
    // Spaltenbreite bleibt in `visual_presentation`. AAA.
    SharedRule {
        id: "text/justified",
        criterion: "1.4.8",
        level: WcagLevel::AAA,
        name: "Visual Presentation (Justified Text)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/visual-presentation.html",
    },
    SharedRule {
        id: "text/line-height-tight",
        criterion: "1.4.8",
        level: WcagLevel::AAA,
        name: "Visual Presentation (Line Height)",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/visual-presentation.html",
    },
    // Die statischen Regeln der Darstellungskonvention (barrierlab#22, #699):
    // Best Practice, keine WCAG-Anforderung, am nächsten Kriterium verankert
    // wie die lokalen `display/*` in `wcag::rules::display_modes`. Dort
    // bleibt, was die laufende Seite misst (`display/init-missing` per
    // Beobachter, `display/text-hidden` samt berechneter Sichtbarkeit, die
    // Textmodus-Prüfungen); diese Kennungen aus `a11y-rules` übernimmt
    // auditmysite nicht.
    // Ersetzt `display/toggle-missing` aus `display_modes`: derselbe Text,
    // dieselbe Prüfung am DOM nach JavaScript.
    SharedRule {
        id: "display/toggle-missing",
        criterion: "2.2.2",
        level: WcagLevel::A,
        name: "Display modes: toggle",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html",
    },
    // Ersetzt den `missing`-Fall von `display/text-not-visible`: eine
    // Visualisierung ohne `[data-viz-text]` oder nur mit leeren, in jedem
    // Modus. Ob eine vorhandene Textschicht im Textmodus auch erscheint,
    // misst `display_modes` weiter an der laufenden Seite.
    SharedRule {
        id: "viz/text-missing",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Display modes: text layer",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    // Neu mit der Konvention: `<figcaption>` als Kind der `figure[data-viz]`.
    SharedRule {
        id: "viz/caption-missing",
        criterion: "1.1.1",
        level: WcagLevel::A,
        name: "Display modes: caption",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    },
    // Neu: `data-viz="3d"`/`"interactive"` ohne Standbild `[data-viz-static]`
    // -- im ruhigen Modus ersetzt nichts die Bewegung.
    SharedRule {
        id: "viz/static-missing",
        criterion: "2.2.2",
        level: WcagLevel::A,
        name: "Display modes: still image",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html",
    },
    // Neu: `data-viz="chart"` ohne `<table>`; `REVIEW`, weil wenige Werte
    // auch ein Satz trägt.
    SharedRule {
        id: "viz/table-missing",
        criterion: "1.3.1",
        level: WcagLevel::A,
        name: "Display modes: values as table",
        help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    },
];

/// Nimmt Befunde der geteilten Kennungen oberhalb der geprüften Stufe heraus
/// und vermerkt sie als nicht gelaufen.
///
/// Die abgelösten Regeln liefen nur ab ihrer Stufe (`PageRuleEntry::min_level`,
/// die AAA-Durchgänge in `wcag::engine`); eine AAA-Kennung wie
/// `navigation/location-missing` gehört nicht in einen AA-Bericht. Der
/// Vermerk sagt, warum sie fehlt, statt sie wie bestanden aussehen zu lassen.
pub fn retain_up_to_level(results: &mut WcagResults, level: WcagLevel) {
    let above = |id: &str| shared_rule(id).is_some_and(|r| r.level > level);
    let finding_above = |v: &Violation| v.rule_id.as_deref().is_some_and(above);
    for list in [
        &mut results.violations,
        &mut results.warnings,
        &mut results.positives,
        &mut results.not_testables,
    ] {
        list.retain(|v| !finding_above(v));
    }
    for outcome in &mut results.rule_outcomes {
        if let Some(rule) = shared_rule(&outcome.rule_id).filter(|r| r.level > level) {
            *outcome = RuleRun::not_run(rule.id, NotRun::Disabled)
                .with_wcag([rule.criterion])
                .with_reason("above_wcag_level");
        }
    }
}

/// Kennungen abgelöster auditmysite-Regeln, unter denen ein `[rules]`-Filter
/// (`--disable-rule`, `enabled_only`) die geteilte Regel weiter trifft — wer
/// `color-contrast` abschaltete, schaltet auch `contrast/text-*` ab. Nur
/// vollständig abgelöste Regeln: Läuft unter der alten Kennung noch ein
/// lokaler Teil (`css-orientation-lock`, `visual-presentation`), steht sie
/// nicht hier.
pub const LEGACY_RULE_IDS: &[(&str, &str)] = &[
    ("contrast/text-insufficient", "color-contrast"),
    ("contrast/text-undetermined", "color-contrast"),
    ("contrast/text-enhanced", "color-contrast"),
    ("color/link-indistinct", "link-in-text-block"),
    (
        "keyboard/scrollable-region-not-focusable",
        "scrollable-region-focusable",
    ),
    ("lists/role-redundant", "redundant-role"),
    ("focus/outline-removed", "focus-visible-outline-none"),
    ("motion/reduced-motion-ignored", "prefers-reduced-motion"),
    ("links/used-as-button", "link-as-button"),
    (
        "keyboard/click-handler-not-focusable",
        "click-events-have-key-events",
    ),
    ("navigation/location-missing", "location"),
    ("landmarks/main-missing", "landmark-one-main"),
    ("landmarks/main-duplicate", "landmark-no-duplicate-main"),
    ("landmarks/banner-missing", "landmark-banner-present"),
    (
        "landmarks/contentinfo-missing",
        "landmark-contentinfo-present",
    ),
    (
        "landmarks/navigation-missing",
        "landmark-navigation-present",
    ),
    ("svg/name-missing", "svg-img-alt"),
    ("keyboard/skip-link-missing", "skip-link"),
    ("dialog/name-missing", "dialog-name"),
    ("summary/name-missing", "summary-name"),
    ("label-in-name/mismatch", "label-content-name-mismatch"),
    ("forms/autocomplete-invalid", "autocomplete-valid"),
    ("forms/title-only-label", "label-title-only"),
    ("forms/redundant-entry", "redundant-entry"),
    ("auth/captcha", "accessible-auth-captcha"),
    ("aria/attribute-prohibited", "aria-prohibited-attr"),
    ("aria/required-parent-missing", "aria-required-parent"),
    ("document/lang-mismatch", "html-xml-lang-mismatch"),
];

/// Ob der `[rules]`-Filter die geteilte Kennung laufen lässt — unter ihrem
/// eigenen Namen oder einer alten Kennung aus [`LEGACY_RULE_IDS`].
/// Abgeschaltet ist sie, sobald einer der Namen abgeschaltet ist; mit
/// `enabled_only` läuft sie, sobald einer der Namen darin steht.
fn filter_allows(filter: &RuleFilterConfig, id: &str) -> bool {
    let names = || {
        std::iter::once(id).chain(
            LEGACY_RULE_IDS
                .iter()
                .filter(move |(shared, _)| *shared == id)
                .map(|(_, old)| *old),
        )
    };
    if !filter.enabled_only_rules.is_empty() {
        names().any(|n| filter.enabled_only_rules.iter().any(|r| r == n))
    } else {
        !names().any(|n| filter.disabled_rules.iter().any(|r| r == n))
    }
}

/// Nimmt Befunde der geteilten Kennungen heraus, die der `[rules]`-Filter
/// abschaltet, und vermerkt sie als nicht gelaufen — wie die Baum-Regeln,
/// die der Filter gar nicht erst startet. Vor #698 galt der Filter für die
/// geteilten Regeln nicht: `--disable-rule color-contrast` schaltete nach
/// der Umstellung nichts mehr ab.
pub fn retain_allowed(results: &mut WcagResults, filter: &RuleFilterConfig) {
    let off = |id: &str| shared_rule(id).is_some() && !filter_allows(filter, id);
    let finding_off = |v: &Violation| v.rule_id.as_deref().is_some_and(off);
    for list in [
        &mut results.violations,
        &mut results.warnings,
        &mut results.positives,
        &mut results.not_testables,
    ] {
        list.retain(|v| !finding_off(v));
    }
    for outcome in &mut results.rule_outcomes {
        if let Some(rule) = shared_rule(&outcome.rule_id).filter(|r| off(r.id)) {
            *outcome = RuleRun::not_run(rule.id, NotRun::Disabled)
                .with_wcag([rule.criterion])
                .with_reason("disabled_by_rule_filter");
        }
    }
}

/// Geteilte Kennungen, die Landmarks der ganzen Seite zählen oder
/// vergleichen. Ein ausgeschlossener Teilbaum (`--exclude-selector`, #645)
/// darf darin nicht mitzählen (#726): Die Musterseite im Teilbaum hätte sonst
/// eine zweite banner-Landmark und machte die der Seite zum Duplikat. Für
/// diese Kennungen gilt deshalb der Lauf über das Dokument ohne die
/// ausgeschlossenen Teilbäume ([`adopt_page_counts`]).
pub const PAGE_COUNT_RULES: &[&str] = &[
    "landmarks/not-unique",
    "landmarks/main-duplicate",
    "landmarks/banner-duplicate",
    "landmarks/contentinfo-duplicate",
];

/// Ersetzt in `results` Befunde und Vermerke der [`PAGE_COUNT_RULES`] durch
/// die aus `counted`, dem Lauf über das Dokument ohne ausgeschlossene
/// Teilbäume (#726).
pub fn adopt_page_counts(results: &mut WcagResults, counted: WcagResults) {
    let counts = |id: &str| PAGE_COUNT_RULES.contains(&id);
    let finding_counts = |v: &Violation| v.rule_id.as_deref().is_some_and(counts);
    results.violations.retain(|v| !finding_counts(v));
    results.warnings.retain(|v| !finding_counts(v));
    results.rule_outcomes.retain(|o| !counts(&o.rule_id));
    results
        .violations
        .extend(counted.violations.into_iter().filter(|v| finding_counts(v)));
    results
        .warnings
        .extend(counted.warnings.into_iter().filter(|v| finding_counts(v)));
    results.rule_outcomes.extend(
        counted
            .rule_outcomes
            .into_iter()
            .filter(|o| counts(&o.rule_id)),
    );
    results.localized_texts.extend(counted.localized_texts);
}

fn shared_rule(id: &str) -> Option<&'static SharedRule> {
    SHARED_RULES.iter().find(|r| r.id == id)
}

/// Die AX-Kennungen aller `<svg>`-Elemente im Dokument.
///
/// Im AX-Baum ist ein `<svg>` von einem `<div role="img">` nicht zu
/// unterscheiden; `svg/name-missing` prüft es aber schon. Die Pipeline nimmt
/// darüber die `image-alt`-Befunde an `<svg>` heraus
/// (`wcag::rules::is_svg_finding`), damit derselbe Fall nicht zweimal im
/// Bericht steht.
pub fn svg_ax_node_ids(doc: &CdpDocument) -> std::collections::HashSet<String> {
    a11y_dom::elements(doc)
        .filter(|n| n.is_element("svg"))
        .filter_map(|n| doc.ax_node_id(n).map(str::to_string))
        .collect()
}

/// Ein kurzer, im Devtools-Suchfeld benutzbarer Selektor für ein Element.
///
/// Wird gesetzt, damit `enrich_violations_with_page` den Befund überspringt:
/// Die Anreicherung schlägt sonst über die AXTree-Knotenkennung nach, und ein
/// Element, das im Accessibility-Tree gar nicht vorkommt (ignoriert,
/// präsentational), würde dort als „Geisterelement" zu einer Warnung
/// herabgestuft — obwohl der Befund aus dem DOM sicher belegt ist.
fn selector_for(node: a11y_dom::ArenaNode<'_>) -> String {
    let mut sel = node.local_name().to_string();
    if let Some(id) = node.attr("id").filter(|v| !v.trim().is_empty()) {
        sel.push('#');
        sel.push_str(id.trim());
        return sel;
    }
    if let Some(class) = node.attr("class").filter(|v| !v.trim().is_empty()) {
        if let Some(first) = class.split_whitespace().next() {
            sel.push('.');
            sel.push_str(first);
        }
    }
    sel
}

/// Das Element zu einem Befund. `location.node` trägt den Arena-Index als
/// Text; darüber geht es zurück auf das Element und von dort auf Selektor,
/// Attribute und AXTree-Kennung.
fn finding_node<'d>(doc: &'d CdpDocument, finding: &Finding) -> Option<a11y_dom::ArenaNode<'d>> {
    finding
        .location
        .node
        .as_deref()
        .and_then(|s| s.parse::<u32>().ok())
        .and_then(|idx| doc.node_at(NodeId(idx)))
}

/// Übersetzt einen geteilten [`Finding`] in auditmysites [`Violation`].
fn to_violation(doc: &CdpDocument, finding: &Finding, rule: &SharedRule) -> Violation {
    let node = finding_node(doc, finding);

    // "document" ist eine der Platzhalter-Kennungen, die die Anreicherung
    // ausdrücklich nicht als Geisterelement wertet.
    let node_id = node
        .and_then(|n| doc.ax_node_id(n))
        .unwrap_or("document")
        .to_string();

    let mut violation = Violation::new(
        rule.criterion,
        rule.name,
        rule.level,
        finding.severity,
        finding.message.clone(),
        node_id,
    )
    .with_rule_id(rule.id)
    .with_help_url(rule.help_url)
    .with_kind(finding.outcome)
    .with_tags(finding.tags.clone());

    if let Some(n) = node {
        violation = violation.with_selector(selector_for(n));
        violation.backend_node_id = doc.backend_node_id(n);
    }
    if let Some(help) = &finding.help {
        violation = violation.with_fix(help.clone());
    }
    if let Some(code) = &finding.suggested_code {
        violation = violation.with_suggested_code(code.clone());
    }
    if let Some(snippet) = &finding.snippet {
        violation = violation.with_html_snippet(snippet.clone());
    }
    violation.evidence = finding.evidence.clone();
    violation
}

/// Vermerk für eine geteilte Kennung, die auditmysite noch nicht führt.
///
/// [`NotRun::Disabled`] und nicht etwa Schweigen: Der Bericht sagt damit
/// ausdrücklich, dass diese Kennung nicht geprüft wurde.
fn not_yet_migrated(id: &str) -> RuleRun {
    RuleRun::not_run(id, NotRun::Disabled).with_reason("shared_rule_not_yet_adopted")
}

/// Die Sprache der Befundtexte zur Laufsprache des Berichts.
///
/// Die Laufsprache kennt nur `de` und `en`; alles andere fällt auf Englisch,
/// die Vorgabe des geteilten Bestands.
fn locale_for(lang: &str) -> Locale {
    if lang == "de" {
        Locale::De
    } else {
        Locale::En
    }
}

/// Lässt den geteilten Regelbestand laufen und übernimmt die Befunde der
/// Kennungen aus [`SHARED_RULES`]. Ohne Stylesheets: Deren Regeln stehen
/// dann als `NotRun::CapabilityMissing` im Bericht.
pub fn run_shared_rules(doc: &CdpDocument, lang: &str) -> WcagResults {
    run_shared_rules_inner(doc, lang, None)
}

/// [`run_shared_rules`] samt den Regeln über Stylesheets
/// (`a11y_rules::run_stylesheets`). `sheets` sind die lesbaren Sheets der
/// Seite in Dokumentreihenfolge (`fetch_stylesheets`); fremde Sheets ohne
/// lesbare `cssRules` fehlen darin wie bisher in den JavaScript-Regeln.
pub fn run_shared_rules_with_stylesheets(
    doc: &CdpDocument,
    lang: &str,
    sheets: &[stylesheet_parse::Stylesheet],
) -> WcagResults {
    run_shared_rules_inner(doc, lang, Some(sheets))
}

fn run_shared_rules_inner(
    doc: &CdpDocument,
    lang: &str,
    sheets: Option<&[stylesheet_parse::Stylesheet]>,
) -> WcagResults {
    let mut results = WcagResults::new();
    // `nodes_checked` bleibt bewusst unberuehrt: Der Zaehler fuehrt
    // AXTree-Knoten, und die geteilten Regeln laufen ueber den DOM. Beide
    // Baeume beschreiben dieselben Elemente -- sie zu addieren zaehlte jedes
    // Element doppelt und machte die Zahl im Bericht unbrauchbar.

    // Findings carry canonical English, like every other analysis result
    // (#406). a11y-rules formats its texts per locale and exposes no message
    // keys, so a German run checks the document a second time and pairs the
    // two reports finding by finding. The rules are deterministic over the
    // same document; a pair whose rule ids differ is skipped, not guessed.
    // Mit Stilen, wo das Dokument sie hat: Dann sieht der Geltungsbereich der
    // Regeln per CSS Verstecktes (`display: none`, `visibility: hidden`) als
    // verborgen. Ohne sie nur das `hidden`-Attribut, und etwa ein per Klasse
    // ausgeblendetes Menü unter `aria-hidden` fiele als fokussierbar auf.
    let lauf = |locale: Locale| {
        let report = match doc.rendered() {
            Some(rendered) => a11y_rules::run_full_in(&rendered, locale),
            None => a11y_rules::run_with_semantics_in(doc, locale),
        };
        match sheets {
            Some(sheets) => a11y_rules::run_stylesheets_in(report, doc, sheets, locale),
            None => report,
        }
    };
    let report = lauf(Locale::En);
    if locale_for(lang) != Locale::En {
        let localized = lauf(locale_for(lang));
        if localized.findings.len() == report.findings.len() {
            for (en, loc) in report.findings.iter().zip(&localized.findings) {
                if en.rule_id != loc.rule_id {
                    continue;
                }
                if en.message != loc.message {
                    results
                        .localized_texts
                        .insert(en.message.clone(), loc.message.clone());
                }
                if let (Some(en_help), Some(loc_help)) = (&en.help, &loc.help) {
                    if en_help != loc_help {
                        results
                            .localized_texts
                            .insert(en_help.clone(), loc_help.clone());
                    }
                }
            }
        }
    }

    for finding in &report.findings {
        let Some(rule) = shared_rule(&finding.rule_id) else {
            continue;
        };
        results.add_violation(to_violation(doc, finding, rule));
    }

    // Ein Vermerk je deklarierter Kennung — dieselbe Namensmenge wie die
    // Befunde, damit ein Join über `rule_id` aufgeht.
    for run in &report.rule_runs {
        match shared_rule(&run.rule_id) {
            None => results.rule_outcomes.push(not_yet_migrated(&run.rule_id)),
            // Seit der Vermerk selbst aus dem geteilten Crate kommt, sprechen
            // beide Seiten dasselbe Modell -- er wird durchgereicht statt
            // uebersetzt. Ergaenzt wird nur das Kriterium, das `a11y-rules`
            // am Vermerk nicht mitfuehrt.
            Some(rule) => results
                .rule_outcomes
                .push(run.clone().with_wcag([rule.criterion])),
        }
    }

    // Befunde, die der geteilte Bestand als bestanden führt, zählen wie
    // bisher als Pass und erscheinen nicht als Problem.
    results.passes += report
        .findings
        .iter()
        .filter(|f| f.outcome == Outcome::Pass)
        .count();

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{build_document, AXTree};
    use crate::wcag::types::Severity;
    use chromiumoxide::cdp::browser_protocol::dom::Node as CdpNode;

    fn seite(html_attrs: &[&str]) -> CdpNode {
        serde_json::from_value(serde_json::json!({
            "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
            "nodeName": "#document", "localName": "", "nodeValue": "",
            "children": [{
                "nodeId": 2, "backendNodeId": 2, "nodeType": 1,
                "nodeName": "HTML", "localName": "html", "nodeValue": "",
                "attributes": html_attrs,
                "children": [{
                    "nodeId": 3, "backendNodeId": 3, "nodeType": 1,
                    "nodeName": "BODY", "localName": "body", "nodeValue": "",
                    "attributes": [], "children": []
                }]
            }]
        }))
        .expect("CDP-Knoten")
    }

    fn ergebnis(html_attrs: &[&str]) -> WcagResults {
        let doc = build_document(&seite(html_attrs), &AXTree::new()).unwrap();
        run_shared_rules(&doc, "en")
    }

    #[test]
    fn fehlendes_lang_faellt_unter_der_geteilten_kennung() {
        let r = ergebnis(&[]);
        let ids: Vec<_> = r
            .violations
            .iter()
            .filter_map(|v| v.rule_id.as_deref())
            .collect();
        assert!(ids.contains(&"document/lang-missing"), "{ids:?}");
    }

    /// Der Befund bleibt in jeder Laufsprache englisch (#406); ein deutscher
    /// Lauf legt die deutsche Fassung zum englischen Text ab, fuer das PDF.
    #[test]
    fn deutscher_lauf_legt_deutsche_befundtexte_daneben() {
        let doc = build_document(&seite(&[]), &AXTree::new()).unwrap();
        let en_text = "The <html> element has no lang attribute.";
        let lauf = |lang: &str| run_shared_rules(&doc, lang);
        let meldung = |r: &WcagResults| {
            r.violations
                .iter()
                .find(|v| v.rule_id.as_deref() == Some("document/lang-missing"))
                .map(|v| v.message.clone())
        };

        let de = lauf("de");
        assert_eq!(meldung(&de).as_deref(), Some(en_text));
        assert_eq!(
            de.localized_texts.get(en_text).map(String::as_str),
            Some("Das <html>-Element hat kein lang-Attribut.")
        );

        let en = lauf("en");
        assert_eq!(meldung(&en).as_deref(), Some(en_text));
        assert!(en.localized_texts.is_empty());
    }

    /// Der Befund muss auditmysites Felder fuellen, sonst faellt er in der
    /// Ausgabe durch: ohne Kriterium keine Zuordnung, ohne Selektor stuft
    /// die Anreicherung ihn zur Warnung herab.
    #[test]
    fn geteilter_befund_traegt_kriterium_stufe_und_selektor() {
        let r = ergebnis(&[]);
        let v = r
            .violations
            .iter()
            .find(|v| v.rule_id.as_deref() == Some("document/lang-missing"))
            .expect("Befund");

        assert_eq!(v.rule, "3.1.1");
        assert_eq!(v.level, WcagLevel::A);
        assert_eq!(v.severity, Severity::High);
        assert_eq!(v.selector.as_deref(), Some("html"));
        assert_eq!(v.kind, Outcome::Fail);
    }

    /// Das konnte die abgeloeste Regel nicht: Sie kannte nur "da oder nicht
    /// da" und haette `lang="x"` durchgewinkt.
    #[test]
    fn ungueltiger_sprachcode_wird_erkannt() {
        let r = ergebnis(&["lang", "x"]);
        let ids: Vec<_> = r
            .violations
            .iter()
            .filter_map(|v| v.rule_id.as_deref())
            .collect();
        assert!(ids.contains(&"document/lang-invalid"), "{ids:?}");
    }

    #[test]
    fn gueltiges_lang_erzeugt_keinen_befund() {
        let r = ergebnis(&["lang", "de"]);
        assert!(
            !r.violations.iter().any(|v| v
                .rule_id
                .as_deref()
                .is_some_and(|id| id.starts_with("document/lang"))),
            "unerwartet: {:?}",
            r.violations
        );
    }

    /// Noch nicht uebernommene Kennungen duerfen nicht als Befund
    /// durchschlagen -- sonst staende jeder Befund doppelt im Bericht,
    /// solange die auditmysite-eigene Regel noch existiert.
    #[test]
    fn nicht_uebernommene_kennungen_liefern_keine_befunde() {
        // Diese Seite hat kaum etwas: kein <title>, keinen Viewport, keine
        // Landmarks. Der geteilte Bestand findet dazu einiges -- durchkommen
        // darf davon nur, was in `SHARED_RULES` steht.
        let r = ergebnis(&["lang", "de"]);
        assert!(!r.violations.is_empty());
        for v in r.violations.iter().chain(&r.warnings) {
            let id = v.rule_id.as_deref().expect("rule_id");
            assert!(shared_rule(id).is_some(), "nicht uebernommen: {id}");
        }
    }

    /// ... sie werden aber vermerkt. "Nicht gelaufen" ist nicht "bestanden".
    #[test]
    fn nicht_uebernommene_kennungen_werden_vermerkt() {
        let r = ergebnis(&["lang", "de"]);

        let label = r
            .rule_outcomes
            .iter()
            .find(|o| o.rule_id == "links/generic-name")
            .expect("Vermerk zu links/generic-name");
        assert!(crate::wcag::rule_run_skipped(label));
        assert_eq!(label.reason.as_deref(), Some("shared_rule_not_yet_adopted"));
    }

    /// Der Schluessel eines Vermerks ist `(rule_id, viewport)`. Die Pipeline
    /// stempelt den Viewport nachtraeglich auf jeden Vermerk eines
    /// Durchgangs -- das traegt nur, wenn die Kennung **innerhalb** eines
    /// Durchgangs schon eindeutig ist. Sonst kollidieren zwei Vermerke
    /// derselben Regel im selben Viewport und der Join wird mehrdeutig.
    #[test]
    fn jede_kennung_kommt_je_durchgang_genau_einmal_vor() {
        let r = ergebnis(&[]);
        let mut gesehen = std::collections::BTreeSet::new();
        for o in &r.rule_outcomes {
            assert!(
                gesehen.insert(o.rule_id.clone()),
                "Kennung {} doppelt im selben Durchgang",
                o.rule_id
            );
        }
        // Und der Bestand ist vollstaendig vermerkt, nicht nur das Uebernommene.
        assert_eq!(gesehen.len(), r.rule_outcomes.len());
        assert!(gesehen.contains("ids/duplicate"), "{gesehen:?}");
        assert!(
            gesehen.contains("keyboard/positive-tabindex"),
            "{gesehen:?}"
        );
    }

    /// `image-alt` sieht ein `<svg>` wie ein `<div role="img">`; nur ueber
    /// den DOM laesst sich das `<svg>` herausnehmen, das `svg/name-missing`
    /// schon meldet. Die Kennungen muessen die des AX-Baums sein, denn daran
    /// haengt der `image-alt`-Befund.
    #[test]
    fn svg_kennungen_kommen_aus_dem_ax_baum() {
        let kind = |id: i64, tag: &str| {
            serde_json::json!({
                "nodeId": id, "backendNodeId": id, "nodeType": 1,
                "nodeName": tag.to_uppercase(), "localName": tag, "nodeValue": "",
                "attributes": [], "children": []
            })
        };
        let dom: CdpNode = serde_json::from_value(serde_json::json!({
            "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
            "nodeName": "#document", "localName": "", "nodeValue": "",
            "children": [{
                "nodeId": 2, "backendNodeId": 2, "nodeType": 1,
                "nodeName": "HTML", "localName": "html", "nodeValue": "",
                "attributes": ["lang", "de"],
                "children": [{
                    "nodeId": 3, "backendNodeId": 3, "nodeType": 1,
                    "nodeName": "BODY", "localName": "body", "nodeValue": "",
                    "attributes": [],
                    "children": [kind(4, "svg"), kind(5, "div")]
                }]
            }]
        }))
        .expect("CDP-Knoten");
        let ax_knoten = |backend: i64| crate::accessibility::AXNode {
            node_id: format!("ax-{backend}"),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: Some(backend),
        };
        let ax = AXTree::from_nodes(vec![ax_knoten(4), ax_knoten(5)]);
        let doc = build_document(&dom, &ax).unwrap();

        let ids = svg_ax_node_ids(&doc);
        assert_eq!(ids, ["ax-4".to_string()].into());
    }

    /// Eine Seite mit zwei `<div id="dopplung">` im Koerper. `verweis` haengt,
    /// wenn gesetzt, ein `<label for=...>` davor.
    fn seite_mit_doppelter_id(verweis: Option<&str>) -> CdpNode {
        let mut kinder = vec![];
        if let Some(ziel) = verweis {
            kinder.push(serde_json::json!({
                "nodeId": 4, "backendNodeId": 4, "nodeType": 1,
                "nodeName": "LABEL", "localName": "label", "nodeValue": "",
                "attributes": ["for", ziel], "children": []
            }));
        }
        for (i, node_id) in [5, 6].iter().enumerate() {
            kinder.push(serde_json::json!({
                "nodeId": node_id, "backendNodeId": node_id, "nodeType": 1,
                "nodeName": "DIV", "localName": "div", "nodeValue": "",
                "attributes": ["id", "dopplung", "data-nr", i.to_string()],
                "children": []
            }));
        }
        serde_json::from_value(serde_json::json!({
            "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
            "nodeName": "#document", "localName": "", "nodeValue": "",
            "children": [{
                "nodeId": 2, "backendNodeId": 2, "nodeType": 1,
                "nodeName": "HTML", "localName": "html", "nodeValue": "",
                "attributes": ["lang", "de"],
                "children": [{
                    "nodeId": 3, "backendNodeId": 3, "nodeType": 1,
                    "nodeName": "BODY", "localName": "body", "nodeValue": "",
                    "attributes": [], "children": kinder
                }]
            }]
        }))
        .expect("CDP-Knoten")
    }

    fn duplikat_befunde(verweis: Option<&str>) -> Vec<Violation> {
        let doc = build_document(&seite_mit_doppelter_id(verweis), &AXTree::new()).unwrap();
        run_shared_rules(&doc, "en")
            .violations
            .into_iter()
            .filter(|v| v.rule_id.as_deref() == Some("ids/duplicate"))
            .collect()
    }

    /// Der Filter greift über eine Konstante, die Tabelle schreibt die
    /// Kennung aus — beide müssen dieselbe meinen, sonst liefe der Filter ins
    /// Leere.
    /// Plan 54 §2: Zeigt ein IDREF auf die doppelt vergebene ID, ist die
    /// Beziehung mehrdeutig -- das ist ein Verstoss gegen 4.1.2, nicht mehr
    /// gegen das gestrichene 4.1.1.
    #[test]
    fn referenzierte_doppelte_id_faellt_unter_4_1_2() {
        let befunde = duplikat_befunde(Some("dopplung"));
        assert_eq!(befunde.len(), 1, "{befunde:?}");
        assert_eq!(befunde[0].rule, "4.1.2");
    }

    /// Und ohne Verweis darauf gibt es seit der Streichung von 4.1.1 kein
    /// Kriterium mehr, das die Dopplung verletzt -- also auch keinen Befund.
    #[test]
    fn unreferenzierte_doppelte_id_ist_kein_befund_mehr() {
        assert!(duplikat_befunde(None).is_empty());
        // Ein Verweis auf eine andere ID macht sie nicht referenziert.
        assert!(duplikat_befunde(Some("etwas-anderes")).is_empty());
    }

    /// Der Vermerk zählt nur, was als Befund übrig bleibt. Vorher trug er die
    /// Zählung des Crates weiter, und ein Report meldete für `ids/duplicate`
    /// zehn Befunde, die es in `violations` nicht gab (inros-lackner.de,
    /// 2026-09-24).
    #[test]
    fn der_vermerk_zaehlt_verworfene_duplikate_nicht_mit() {
        let vermerk = |verweis| {
            let doc = build_document(&seite_mit_doppelter_id(verweis), &AXTree::new()).unwrap();
            run_shared_rules(&doc, "en")
                .rule_outcomes
                .into_iter()
                .find(|o| o.rule_id == "ids/duplicate")
                .expect("Vermerk")
                .findings
        };
        assert_eq!(vermerk(None), 0);
        assert_eq!(vermerk(Some("dopplung")), 1);
    }

    /// Ein Element im CDP-Format; `nodeId` und `backendNodeId` sind gleich.
    fn el(
        id: i64,
        tag: &str,
        attrs: &[&str],
        children: Vec<serde_json::Value>,
    ) -> serde_json::Value {
        serde_json::json!({
            "nodeId": id, "backendNodeId": id, "nodeType": 1,
            "nodeName": tag.to_uppercase(), "localName": tag, "nodeValue": "",
            "attributes": attrs, "children": children
        })
    }

    /// Befunde der geteilten Regeln zu einem `<body>`-Inhalt; `rollen` ist
    /// Chromes AX-Rolle je Backend-ID (`None`: ignoriert).
    fn aria_befunde(
        body: Vec<serde_json::Value>,
        rollen: &[(i64, Option<&str>)],
    ) -> Vec<(String, Option<String>)> {
        let dom: CdpNode = serde_json::from_value(serde_json::json!({
            "nodeId": 1, "backendNodeId": 1, "nodeType": 9,
            "nodeName": "#document", "localName": "", "nodeValue": "",
            "children": [el(2, "html", &["lang", "de"], vec![el(3, "body", &[], body)])]
        }))
        .expect("CDP-Knoten");
        let ax = AXTree::from_nodes(
            rollen
                .iter()
                .map(|(backend, role)| crate::accessibility::AXNode {
                    node_id: format!("ax-{backend}"),
                    ignored: role.is_none(),
                    ignored_reasons: vec![],
                    role: Some(role.unwrap_or("none").to_string()),
                    name: None,
                    name_source: None,
                    description: None,
                    value: None,
                    properties: vec![],
                    child_ids: vec![],
                    parent_id: None,
                    backend_dom_node_id: Some(*backend),
                })
                .collect(),
        );
        let doc = build_document(&dom, &ax).unwrap();
        let r = run_shared_rules(&doc, "en");
        r.violations
            .iter()
            .chain(&r.warnings)
            .filter(|v| {
                v.rule_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("aria/"))
            })
            .map(|v| (v.rule_id.clone().unwrap(), v.selector.clone()))
            .collect()
    }

    /// #715 (magyarorszag.hu): Chrome glaettet das `<li>` zwischen Tabliste
    /// und Tab, der AX-Baum zeigt `tablist -> tab`. Die geteilte Regel prueft
    /// den DOM und meldet wie axe beide Seiten (#691).
    #[test]
    fn tabliste_aus_listeneintraegen_meldet_kontext_und_bestandteile() {
        let befunde = aria_befunde(
            vec![el(
                4,
                "ul",
                &["role", "tablist", "id", "tabs"],
                vec![el(
                    5,
                    "li",
                    &[],
                    vec![el(
                        6,
                        "a",
                        &["role", "tab", "href", "#a", "id", "t1"],
                        vec![],
                    )],
                )],
            )],
            &[(4, Some("tablist")), (5, None), (6, Some("tab"))],
        );
        let hat = |id: &str, sel: &str| {
            befunde
                .iter()
                .any(|(i, s)| i == id && s.as_deref() == Some(sel))
        };
        assert!(
            hat("aria/required-children-missing", "ul#tabs"),
            "{befunde:?}"
        );
        assert!(hat("aria/required-parent-missing", "a#t1"), "{befunde:?}");
    }

    /// Chromes `LayoutTable*` ist keine ARIA-Rolle. Als unbekannte Rolle
    /// braeche sie die Kontextpruefung ab, und ein verwaister Tab in einer
    /// Layouttabelle bliebe ohne Befund; uebersetzt ist sie durchlaessig.
    #[test]
    fn layouttabelle_verdeckt_den_fehlenden_kontext_nicht() {
        let befunde = aria_befunde(
            vec![el(
                4,
                "table",
                &[],
                vec![el(
                    5,
                    "tbody",
                    &[],
                    vec![el(
                        6,
                        "tr",
                        &[],
                        vec![el(
                            7,
                            "td",
                            &[],
                            vec![el(8, "div", &["role", "tab", "id", "t1"], vec![])],
                        )],
                    )],
                )],
            )],
            &[
                (4, Some("LayoutTable")),
                (5, None),
                (6, Some("LayoutTableRow")),
                (7, Some("LayoutTableCell")),
                (8, Some("tab")),
            ],
        );
        assert!(
            befunde
                .iter()
                .any(|(id, sel)| id == "aria/required-parent-missing"
                    && sel.as_deref() == Some("div#t1")),
            "{befunde:?}"
        );
    }

    /// `role="img"` meldet Chrome als `image`. Ohne Uebersetzung bliebe die
    /// Rolle ohne Urteil und `aria-checked` daran unbemerkt (Korpus
    /// `aria_attribute_validation`).
    #[test]
    fn chromes_image_wird_als_img_beurteilt() {
        let befunde = aria_befunde(
            vec![el(
                4,
                "div",
                &[
                    "role",
                    "img",
                    "aria-checked",
                    "true",
                    "aria-label",
                    "Icon",
                    "id",
                    "i",
                ],
                vec![],
            )],
            &[(4, Some("image"))],
        );
        assert!(
            befunde
                .iter()
                .any(|(id, sel)| id == "aria/attribute-not-allowed"
                    && sel.as_deref() == Some("div#i")),
            "{befunde:?}"
        );
    }

    /// Eine AAA-Kennung erscheint nur in einem AAA-Lauf, wie die abgeloeste
    /// Regel `location`; sonst wird sie als nicht gelaufen vermerkt (#695).
    #[test]
    fn kennung_oberhalb_der_stufe_wird_vermerkt_statt_gemeldet() {
        let ergebnis = || {
            let mut r = WcagResults::new();
            r.add_violation(
                Violation::new("2.4.8", "Location", WcagLevel::AAA, Severity::Low, "m", "x")
                    .with_rule_id("navigation/location-missing")
                    .with_kind(Outcome::Review),
            );
            r.rule_outcomes
                .push(RuleRun::ran("navigation/location-missing", 1));
            r
        };

        let mut aa = ergebnis();
        retain_up_to_level(&mut aa, WcagLevel::AA);
        assert!(aa.warnings.is_empty());
        let vermerk = &aa.rule_outcomes[0];
        assert!(crate::wcag::rule_run_skipped(vermerk));
        assert_eq!(vermerk.reason.as_deref(), Some("above_wcag_level"));

        let mut aaa = ergebnis();
        retain_up_to_level(&mut aaa, WcagLevel::AAA);
        assert_eq!(aaa.warnings.len(), 1);
        assert_eq!(aaa.rule_outcomes[0].findings, 1);
    }

    /// `--disable-rule color-contrast` schaltet die geteilten
    /// `contrast/text-*` ab, die neue Kennung ebenso; `enabled_only` mit der
    /// alten Kennung laesst sie laufen und alle anderen geteilten nicht.
    #[test]
    fn regelfilter_greift_unter_alter_und_neuer_kennung() {
        let ergebnis = || {
            let mut r = WcagResults::new();
            for id in ["contrast/text-insufficient", "links/name-missing"] {
                r.add_violation(
                    Violation::new("1.4.3", "n", WcagLevel::AA, Severity::High, "m", "x")
                        .with_rule_id(id),
                );
                r.rule_outcomes.push(RuleRun::ran(id, 1));
            }
            r
        };
        let ids = |r: &WcagResults| -> Vec<String> {
            r.violations
                .iter()
                .filter_map(|v| v.rule_id.clone())
                .collect()
        };
        let filter = |disabled: &[&str], only: &[&str]| RuleFilterConfig {
            disabled_rules: disabled.iter().map(|s| s.to_string()).collect(),
            enabled_only_rules: only.iter().map(|s| s.to_string()).collect(),
        };

        for disabled in ["color-contrast", "contrast/text-insufficient"] {
            let mut r = ergebnis();
            retain_allowed(&mut r, &filter(&[disabled], &[]));
            assert_eq!(ids(&r), ["links/name-missing"], "{disabled}");
            let vermerk = &r.rule_outcomes[0];
            assert!(crate::wcag::rule_run_skipped(vermerk));
            assert_eq!(vermerk.reason.as_deref(), Some("disabled_by_rule_filter"));
        }

        let mut nur = ergebnis();
        retain_allowed(&mut nur, &filter(&[], &["color-contrast"]));
        assert_eq!(ids(&nur), ["contrast/text-insufficient"]);

        let mut alles = ergebnis();
        retain_allowed(&mut alles, &RuleFilterConfig::default());
        assert_eq!(alles.violations.len(), 2);
    }

    /// Jede alte Kennung zeigt auf eine geteilte, die auditmysite fuehrt.
    #[test]
    fn alte_kennungen_zeigen_auf_gefuehrte_regeln() {
        for (shared, old) in LEGACY_RULE_IDS {
            assert!(shared_rule(shared).is_some(), "{shared} ({old})");
        }
    }

    /// `rule_outcomes` und `violations` muessen dieselbe Namensmenge
    /// benutzen, sonst laesst sich der Bericht nicht verbinden.
    #[test]
    fn jeder_befund_hat_einen_vermerk_gleicher_kennung() {
        let r = ergebnis(&[]);
        for v in &r.violations {
            let id = v.rule_id.as_deref().expect("rule_id");
            assert!(
                r.rule_outcomes.iter().any(|o| o.rule_id == id),
                "kein Vermerk zu {id}"
            );
        }
    }
}
