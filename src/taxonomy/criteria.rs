//! Mapping von Regel-IDs auf WCAG-Erfolgskriterien und -Prinzipien.
//!
//! Das Scoring gruppiert Violations nach Regel-ID. Für Diversity-Bewertung
//! (#97) und Prinzip-Coverage (#99) wird stattdessen die Ebene der WCAG
//! Success Criteria benötigt — eine Criterion kann mehrere Regeln umfassen.

use super::RuleLookup;

/// Die vier WCAG-Prinzipien.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WcagPrinciple {
    Perceivable,
    Operable,
    Understandable,
    Robust,
}

impl WcagPrinciple {
    /// Alle vier Prinzipien in Reihenfolge.
    pub const ALL: [WcagPrinciple; 4] = [
        WcagPrinciple::Perceivable,
        WcagPrinciple::Operable,
        WcagPrinciple::Understandable,
        WcagPrinciple::Robust,
    ];
}

/// Ermittelt das WCAG-Erfolgskriterium ("1.1.1", "4.1.2", …) für eine Regel-ID.
///
/// Akzeptiert direkte WCAG-IDs ebenso wie Regel-Slugs (z.B. `aria-hidden-focus`).
/// Gibt `None` zurück, wenn keine WCAG-Zuordnung existiert.
pub fn criterion_for_rule(rule_id: &str) -> Option<String> {
    if is_wcag_criterion(rule_id) {
        return Some(rule_id.to_string());
    }
    let rule = RuleLookup::by_legacy_wcag_id(rule_id).or_else(|| RuleLookup::by_id(rule_id))?;
    rule.external_ref
        .and_then(|r| r.strip_prefix("WCAG "))
        .filter(|c| is_wcag_criterion(c))
        .map(|c| c.to_string())
}

/// WCAG-Prinzip für ein Kriterium ("1.x.x" → Perceivable, …).
pub fn principle_for_criterion(criterion: &str) -> Option<WcagPrinciple> {
    match criterion.split('.').next()?.parse::<u8>().ok()? {
        1 => Some(WcagPrinciple::Perceivable),
        2 => Some(WcagPrinciple::Operable),
        3 => Some(WcagPrinciple::Understandable),
        4 => Some(WcagPrinciple::Robust),
        _ => None,
    }
}

/// Prüft, ob ein String die Form `n.n.n` (WCAG-Kriterium) hat.
fn is_wcag_criterion(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_wcag_id_is_its_own_criterion() {
        assert_eq!(criterion_for_rule("1.4.3"), Some("1.4.3".to_string()));
        assert_eq!(criterion_for_rule("4.1.2"), Some("4.1.2".to_string()));
    }

    #[test]
    fn slug_rule_resolves_to_criterion() {
        assert_eq!(
            criterion_for_rule("aria-hidden-focus"),
            Some("4.1.2".to_string())
        );
    }

    #[test]
    fn non_wcag_rule_returns_none() {
        assert_eq!(criterion_for_rule("totally-unknown-rule"), None);
    }

    #[test]
    fn principle_mapping() {
        assert_eq!(
            principle_for_criterion("1.1.1"),
            Some(WcagPrinciple::Perceivable)
        );
        assert_eq!(
            principle_for_criterion("2.4.7"),
            Some(WcagPrinciple::Operable)
        );
        assert_eq!(
            principle_for_criterion("3.1.1"),
            Some(WcagPrinciple::Understandable)
        );
        assert_eq!(
            principle_for_criterion("4.1.2"),
            Some(WcagPrinciple::Robust)
        );
    }
}
