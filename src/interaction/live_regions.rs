//! Beobachtung von Live-Regionen **über die Zeit**.
//!
//! # Warum zwei Aufnahmen hier nicht genügen
//!
//! Der Zwei-Aufnahmen-Vergleich (`AXSnapshot` → Handlung → `AXSnapshot`) sieht
//! nur, was zum Stichprobenzeitpunkt noch dasteht. Eine Statusmeldung, die
//! eingefügt, vorgelesen und nach kurzer Zeit wieder entfernt wird, kann
//! zwischen zwei Aufnahmen nie sichtbar sein — und genau das ist bei
//! Formularfehlern und Warenkorb-Rückmeldungen der Normalfall.
//!
//! Eine reine Existenzprüfung (`document.querySelector('[aria-live]') !== null`)
//! trifft es ebenso wenig: die Region steht meist schon vor der Handlung leer
//! im DOM. Was zählt, ist **ob sie gefüllt wurde**.
//!
//! Dieses Modul zeichnet deshalb Änderungen in Live-Regionen mit Zeitpunkt
//! auf, statt einen Zustand abzufragen. Es beantwortet nicht, ob ein
//! Screenreader die Meldung tatsächlich angesagt hat — nur, dass der Browser
//! den Anlass dazu hatte.
//!
//! # Eingriff in die Seite
//!
//! [`install`] setzt einen `MutationObserver` und ersetzt
//! `Element.prototype.attachShadow`, damit auch später erzeugte Shadow Roots
//! beobachtet werden. Beides bleibt für die Lebensdauer der Seite bestehen.
//! Der Eingriff ist derselbe in der Art wie der History-API-Ersatz in
//! `a11y_journey::spa_navigation` und aus demselben Grund nötig: ohne ihn ist
//! das Ereignis nicht beobachtbar.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};

/// Eine beobachtete Änderung an einer Live-Region.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveEvent {
    /// Millisekunden seit Seitenstart (`performance.now`).
    pub at_ms: u64,
    /// `assertive`, `polite` oder `off` — aus `aria-live` bzw. der Rolle.
    pub politeness: String,
    /// Die Rolle der Region, sofern sie eine trägt (`alert`, `status`, `log`).
    pub role: Option<String>,
    /// Der Textinhalt der Region nach der Änderung, gekürzt.
    pub text: String,
}

impl LiveEvent {
    /// Ob dieses Ereignis eine wahrnehmbare Meldung trägt. Eine Region, die
    /// geleert wird, erzeugt ebenfalls eine Mutation — sie kündigt aber nichts an.
    pub fn is_announcement(&self) -> bool {
        !self.text.trim().is_empty()
    }
}

/// Setzt den Beobachter. Muss **vor** der Handlung laufen, deren Wirkung
/// gemessen werden soll. Mehrfaches Aufrufen ist folgenlos.
const INSTALL_JS: &str = include_str!("js/live_regions_install.js");

/// Liest die aufgezeichneten Ereignisse und leert den Puffer.
const DRAIN_JS: &str = include_str!("js/live_regions_drain.js");

async fn eval_value(page: &Page, js: &str) -> Option<serde_json::Value> {
    let params = EvaluateParams::builder()
        .expression(js.to_string())
        .return_by_value(true)
        .build()
        .ok()?;
    page.execute(params).await.ok()?.result.result.value.clone()
}

/// Setzt den Beobachter. `false`, wenn er nicht gesetzt werden konnte — dann
/// ist „keine Meldung beobachtet" **keine** Aussage über die Seite.
pub async fn install(page: &Page) -> bool {
    eval_value(page, INSTALL_JS)
        .await
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Holt die seit dem letzten Aufruf aufgezeichneten Ereignisse. `None`, wenn
/// die Aufzeichnung nicht gelesen werden konnte — dann ist „keine Meldung"
/// **keine** Aussage über die Seite (#777).
pub async fn drain(page: &Page) -> Option<Vec<LiveEvent>> {
    let value = eval_value(page, DRAIN_JS).await?;
    parse_events(&value)
}

/// Die Umwandlung der JS-Aufzeichnung in Ereignisse — getrennt, damit sie
/// ohne Browser geprüft werden kann. Etwas anderes als eine Liste ist keine
/// leere Aufzeichnung, sondern keine.
fn parse_events(value: &serde_json::Value) -> Option<Vec<LiveEvent>> {
    let items = value.as_array()?;
    Some(
        items
            .iter()
            .filter_map(|item| {
                Some(LiveEvent {
                    at_ms: item.get("at").and_then(|v| v.as_u64()).unwrap_or(0),
                    politeness: item
                        .get("politeness")
                        .and_then(|v| v.as_str())
                        .unwrap_or("polite")
                        .to_string(),
                    role: item
                        .get("role")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    text: item.get("text").and_then(|v| v.as_str())?.to_string(),
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ereignisse_werden_gelesen() {
        let events = parse_events(&json!([
            {"at": 1200, "politeness": "assertive", "role": "alert", "text": "Bitte Namen angeben"},
            {"at": 1400, "politeness": "polite", "role": null, "text": ""}
        ]))
        .unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].at_ms, 1200);
        assert_eq!(events[0].politeness, "assertive");
        assert_eq!(events[0].role.as_deref(), Some("alert"));
        assert!(events[0].is_announcement());
    }

    /// Eine geleerte Region erzeugt eine Mutation, kündigt aber nichts an.
    #[test]
    fn leere_region_ist_keine_ankuendigung() {
        let events =
            parse_events(&json!([{"at": 1, "politeness": "polite", "text": "   "}])).unwrap();
        assert_eq!(events.len(), 1);
        assert!(!events[0].is_announcement());
    }

    #[test]
    fn unbrauchbare_aufzeichnung_ist_keine_leere_aufzeichnung() {
        assert!(parse_events(&json!(null)).is_none());
        assert!(parse_events(&json!({"at": 1})).is_none());
        assert_eq!(parse_events(&json!([])), Some(Vec::new()));
        // Ein Eintrag ohne Text ist kein Ereignis, die übrigen bleiben.
        let events = parse_events(&json!([{"at": 1}, {"at": 2, "text": "da"}])).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].text, "da");
    }
}
