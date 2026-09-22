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
const INSTALL_JS: &str = r#"
(function () {
    if (window.__ams_live_log) { return true; }
    window.__ams_live_log = [];
    var lastText = new WeakMap();

    function roleOf(el) {
        var r = (el.getAttribute('role') || '').toLowerCase();
        return (r === 'alert' || r === 'status' || r === 'log') ? r : null;
    }
    function isLive(el) {
        if (!el || el.nodeType !== 1) return false;
        if (el.hasAttribute('aria-live')) return true;
        return roleOf(el) !== null;
    }
    function politenessOf(el) {
        var declared = (el.getAttribute('aria-live') || '').toLowerCase();
        if (declared) return declared;
        var r = roleOf(el);
        if (r === 'alert') return 'assertive';
        return 'polite';
    }
    // Aufwärts bis zur nächsten Live-Region, über Shadow-Grenzen hinweg.
    function regionFor(node) {
        var el = (node && node.nodeType === 1) ? node : (node ? node.parentElement : null);
        var guard = 0;
        while (el && guard++ < 200) {
            if (isLive(el)) return el;
            if (el.parentElement) { el = el.parentElement; continue; }
            var root = el.getRootNode && el.getRootNode();
            el = (root && root.host) ? root.host : null;
        }
        return null;
    }

    var observer = new MutationObserver(function (records) {
        for (var i = 0; i < records.length; i++) {
            var region = regionFor(records[i].target);
            if (!region) continue;
            var text = (region.textContent || '').trim().slice(0, 300);
            if (lastText.get(region) === text) continue;
            lastText.set(region, text);
            window.__ams_live_log.push({
                at: Math.round((window.performance && performance.now()) || 0),
                politeness: politenessOf(region),
                role: roleOf(region),
                text: text
            });
            if (window.__ams_live_log.length > 200) window.__ams_live_log.shift();
        }
    });
    var opts = { subtree: true, childList: true, characterData: true, attributes: false };

    function observeRoot(root) {
        try { observer.observe(root, opts); } catch (e) { /* abgelöste Wurzel */ }
    }
    observeRoot(document.documentElement);

    // Vorhandene Shadow Roots: der Observer durchdringt sie nicht von selbst.
    (function walk(root, depth) {
        if (depth > 12) return;
        var all = root.querySelectorAll('*');
        for (var i = 0; i < all.length; i++) {
            if (all[i].shadowRoot) { observeRoot(all[i].shadowRoot); walk(all[i].shadowRoot, depth + 1); }
        }
    })(document, 0);

    // Später erzeugte Shadow Roots.
    var origAttach = Element.prototype.attachShadow;
    Element.prototype.attachShadow = function () {
        var root = origAttach.apply(this, arguments);
        observeRoot(root);
        return root;
    };
    return true;
})()
"#;

/// Liest die aufgezeichneten Ereignisse und leert den Puffer.
const DRAIN_JS: &str = r#"
(function () {
    var log = window.__ams_live_log || [];
    window.__ams_live_log = [];
    return log;
})()
"#;

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

/// Holt die seit dem letzten Aufruf aufgezeichneten Ereignisse.
pub async fn drain(page: &Page) -> Vec<LiveEvent> {
    let Some(value) = eval_value(page, DRAIN_JS).await else {
        return Vec::new();
    };
    parse_events(&value)
}

/// Die Umwandlung der JS-Aufzeichnung in Ereignisse — getrennt, damit sie
/// ohne Browser geprüft werden kann.
fn parse_events(value: &serde_json::Value) -> Vec<LiveEvent> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
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
        .collect()
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
        ]));
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].at_ms, 1200);
        assert_eq!(events[0].politeness, "assertive");
        assert_eq!(events[0].role.as_deref(), Some("alert"));
        assert!(events[0].is_announcement());
    }

    /// Eine geleerte Region erzeugt eine Mutation, kündigt aber nichts an.
    #[test]
    fn leere_region_ist_keine_ankuendigung() {
        let events = parse_events(&json!([{"at": 1, "politeness": "polite", "text": "   "}]));
        assert_eq!(events.len(), 1);
        assert!(!events[0].is_announcement());
    }

    #[test]
    fn unbrauchbare_aufzeichnung_ergibt_nichts() {
        assert!(parse_events(&json!(null)).is_empty());
        assert!(parse_events(&json!({"at": 1})).is_empty());
        // Ein Eintrag ohne Text ist kein Ereignis, die übrigen bleiben.
        let events = parse_events(&json!([{"at": 1}, {"at": 2, "text": "da"}]));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].text, "da");
    }
}
