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
    // banner/contentinfo, Sprunglink bei Navigation) bleiben eigen.
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
    // (axe-Kennung `control-missing-label`); `check_form_control` bleibt dort.
    // Beide Seiten fragen den Accessible Name: die eigene Regel den aus
    // Chromes AX-Baum, die geteilte den aus `CdpDocument`, der auf denselben
    // AX-Baum zurückgreift.
    //
    // Zwei Unterschiede, beide gewollt:
    //
    // - Als Link zählt dort nur `<a href>`, nicht jedes Element mit der
    //   AX-Rolle `link`. Ein unbenanntes `<span role="link">` melden weiter
    //   `aria-command-name` (`aria_naming_rules`) und `aria-label`
    //   (`accessible_name`), die das schon bisher neben `labels` taten.
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
    },
];

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
/// Kennungen aus [`SHARED_RULES`].
pub fn run_shared_rules(doc: &CdpDocument, lang: &str) -> WcagResults {
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
    let lauf = |locale: Locale| match doc.rendered() {
        Some(rendered) => a11y_rules::run_full_in(&rendered, locale),
        None => a11y_rules::run_with_semantics_in(doc, locale),
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
            .find(|o| o.rule_id == "forms/label-missing")
            .expect("Vermerk zu forms/label-missing");
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
