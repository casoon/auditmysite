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

use crate::accessibility::CdpDocument;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleOutcome, RuleOutcomeStatus, Violation, WcagResults};

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
        help_url: "https://www.w3.org/WAI/WCAG21/Understanding/language-of-page.html",
    },
    // Neu gegenüber der abgelösten Regel: Sie kannte nur „da oder nicht da".
    SharedRule {
        id: "document/lang-invalid",
        criterion: "3.1.1",
        level: WcagLevel::A,
        name: "Language of Page",
        help_url: "https://www.w3.org/WAI/WCAG21/Understanding/language-of-page.html",
    },
];

fn shared_rule(id: &str) -> Option<&'static SharedRule> {
    SHARED_RULES.iter().find(|r| r.id == id)
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

/// Übersetzt einen geteilten [`Finding`] in auditmysites [`Violation`].
fn to_violation(doc: &CdpDocument, finding: &Finding, rule: &SharedRule) -> Violation {
    // `location.node` trägt den Arena-Index als Text; darüber geht es zurück
    // auf das Element und von dort auf Selektor und AXTree-Kennung.
    let node = finding
        .location
        .node
        .as_deref()
        .and_then(|s| s.parse::<u32>().ok())
        .and_then(|idx| doc.node_at(NodeId(idx)));

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
fn not_yet_migrated(id: &str) -> RuleOutcome {
    RuleOutcome {
        rule_id: id.to_string(),
        status: RuleOutcomeStatus::Skipped,
        wcag_criterion: None,
        viewport: None,
        reason_code: Some("shared_rule_not_yet_adopted".to_string()),
        finding_count: 0,
    }
}

/// Lässt den geteilten Regelbestand laufen und übernimmt die Befunde der
/// Kennungen aus [`SHARED_RULES`].
pub fn run_shared_rules(doc: &CdpDocument) -> WcagResults {
    let mut results = WcagResults::new();
    results.nodes_checked += doc.len();

    let report = a11y_rules::run_with_semantics(doc);

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
            Some(rule) => results.rule_outcomes.push(RuleOutcome {
                rule_id: rule.id.to_string(),
                status: match run.not_run {
                    // Sollte hier nicht vorkommen: Der Host erfüllt
                    // `Semantics`. Ein Vermerk ist trotzdem ehrlicher als
                    // eine stille Null.
                    Some(NotRun::CapabilityMissing) => RuleOutcomeStatus::Skipped,
                    Some(NotRun::Disabled) => RuleOutcomeStatus::Skipped,
                    Some(NotRun::NotApplicable) => RuleOutcomeStatus::NotApplicable,
                    Some(NotRun::Errored) => RuleOutcomeStatus::Failed,
                    None if run.findings > 0 => RuleOutcomeStatus::ViolationsFound,
                    None => RuleOutcomeStatus::NoViolationDetected,
                },
                wcag_criterion: Some(rule.criterion.to_string()),
                viewport: None,
                reason_code: run.reason.clone(),
                finding_count: run.findings,
            }),
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
        run_shared_rules(&doc)
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
        // Diese Seite hat weder <title> noch <h1>; der geteilte Bestand
        // faende dazu etwas, auditmysite fuehrt die Kennungen aber noch nicht.
        let r = ergebnis(&["lang", "de"]);
        assert!(r.violations.is_empty(), "{:?}", r.violations);
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }

    /// ... sie werden aber vermerkt. "Nicht gelaufen" ist nicht "bestanden".
    #[test]
    fn nicht_uebernommene_kennungen_werden_vermerkt() {
        let r = ergebnis(&["lang", "de"]);

        let titel = r
            .rule_outcomes
            .iter()
            .find(|o| o.rule_id == "document/title-missing")
            .expect("Vermerk zu document/title-missing");
        assert_eq!(titel.status, RuleOutcomeStatus::Skipped);
        assert_eq!(
            titel.reason_code.as_deref(),
            Some("shared_rule_not_yet_adopted")
        );
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
