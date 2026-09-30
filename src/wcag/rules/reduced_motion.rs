//! WCAG 2.3.3 Animation from Interactions / Best Practice: prefers-reduced-motion
//!
//! Pages with animations should honor the user's `prefers-reduced-motion`
//! preference. People with vestibular disorders can experience nausea or
//! seizures from motion.
//!
//! Check (#712): nur Bewegung zaehlt, nicht jede Animation. Belegt ist
//! Bewegung, wenn eine Stilregel
//! - eine `animation` auf `@keyframes` setzt, die eine Bewegungs-Eigenschaft
//!   aendern (transform/translate/scale/rotate, top/left/right/bottom,
//!   inset, margin, position, offset-path/-distance), oder
//! - eine `transition` mit Dauer > 0 auf eine solche Eigenschaft setzt;
//!   `transition: all` zaehlt nur, wenn eine Zustandsregel (`:hover`,
//!   `:focus`, …) eine Bewegungs-Eigenschaft setzt.
//!
//! Farb- und Deckkraft-Uebergaenge zaehlen nie. Regeln in einem Block
//! `@media (prefers-reduced-motion: no-preference)` laufen nur ohne Wunsch
//! nach weniger Bewegung und zaehlen daher nicht. Ein Beleg gilt als
//! neutralisiert, wenn ein Block `@media (prefers-reduced-motion: reduce)`
//! die Animation bzw. Transition fuer denselben Selektor oder per
//! Universalselektor (`*`) abschaltet (Name `none`, Dauer nahe 0,
//! `animation-play-state: paused`, `transition-property: none`).
//! Nur ein nicht neutralisierter Beleg ergibt einen Befund.

use chromiumoxide::Page;
use serde::Deserialize;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const REDUCED_MOTION_RULE: RuleMetadata = RuleMetadata {
    id: "2.3.3",
    name: "Animation from Interactions",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Pages with animation should honor prefers-reduced-motion",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html",
    axe_id: "prefers-reduced-motion",
    tags: &[
        "wcag2aaa",
        "wcag233",
        "best-practice",
        "cat.sensory-and-visual-cues",
    ],
};

/// Dauer (ms), bis zu der eine Animation/Transition im Reduce-Block als
/// abgeschaltet gilt — deckt die ueblichen Resets `0.01ms`/`1ms` ab.
const NEUTRALISED_MAX_MS: f64 = 50.0;

/// Hoechstzahl der Belege, die in der Meldung genannt werden.
const MAX_EVIDENCE_IN_MESSAGE: usize = 3;

// Liest die zugaenglichen Stylesheets und liefert Rohdaten; die Entscheidung
// faellt in Rust (`unneutralised_motion`).
const REDUCED_MOTION_JS: &str = r#"
(function() {
  const LIMIT = 500;
  const keyframes = [];
  const animations = [];
  const transitions = [];
  const stateProps = new Set();
  const overrides = [];
  const STATE_RE = /:(hover|focus|focus-within|focus-visible|active|checked|target)\b/i;
  const maxMs = (value) => {
    let max = 0;
    for (const part of String(value || '').split(',')) {
      const t = part.trim().toLowerCase();
      const n = parseFloat(t);
      if (isNaN(n)) continue;
      const ms = t.endsWith('ms') ? n : t.endsWith('s') ? n * 1000 : n;
      if (ms > max) max = ms;
    }
    return max;
  };
  const propsOf = (style) => {
    const out = [];
    for (let i = 0; i < style.length; i++) out.push(style.item(i));
    return out;
  };
  const splitList = (value) => String(value || '').split(',').map(s => s.trim()).filter(Boolean);
  try {
    for (const sheet of Array.from(document.styleSheets)) {
      let rules;
      try { rules = Array.from(sheet.cssRules || []); }
      catch(e) { continue; } // cross-origin sheet
      // context: 'normal' | 'reduce' | 'gated'
      // Nur Bewegung, die auf dieser Seite laufen kann: Der Selektor muss ein
      // dargestelltes Element treffen. Zustands-Pseudoklassen und
      // Pseudo-Elemente werden dafuer abgestreift (`.card:hover` trifft
      // `.card`). Ohne das zaehlten Keyframes fuer Elemente, die es gar nicht
      // gibt — Lade-Spinner eines Players, der nie eingebunden ist
      // (bundesregierung.de, #712).
      const present = (selector) => {
        const plain = selector
          .replace(/::?(before|after|marker|placeholder|selection|backdrop|first-line|first-letter)\b/gi, '')
          .replace(/:(hover|focus|focus-visible|focus-within|active|checked|target|visited|link)\b/gi, '');
        try {
          for (const el of document.querySelectorAll(plain)) {
            if (typeof el.checkVisibility !== 'function' || el.checkVisibility()) return true;
          }
        } catch (_) { return true; }
        return false;
      };
      const walk = (rs, context) => {
        for (const r of rs) {
          if (r.type === CSSRule.MEDIA_RULE) {
            const condition = (r.conditionText || r.media?.mediaText || '').toLowerCase();
            let next = context;
            if (condition.includes('prefers-reduced-motion')) {
              // `reduce` und die boolesche Form `(prefers-reduced-motion)`
              // treffen beide nur bei gewuenschter Reduktion.
              next = (condition.includes('no-preference') || /\bnot\b/.test(condition)) ? 'gated' : 'reduce';
            }
            if (r.cssRules) walk(Array.from(r.cssRules), next);
            continue;
          }
          if (r.type === CSSRule.KEYFRAMES_RULE) {
            const props = new Set();
            for (const kf of Array.from(r.cssRules || [])) {
              if (kf.style) for (const p of propsOf(kf.style)) props.add(p);
            }
            if (keyframes.length < LIMIT) keyframes.push({ name: r.name, props: Array.from(props) });
            continue;
          }
          if (r.type === CSSRule.STYLE_RULE && r.style) {
            const s = r.style;
            const selector = r.selectorText || '';
            if (context === 'reduce') {
              if (overrides.length < LIMIT) overrides.push({
                selector,
                animation_name: s.animationName || '',
                animation_duration_ms: s.animationDuration ? maxMs(s.animationDuration) : null,
                animation_play_state: s.animationPlayState || '',
                transition_property: s.transitionProperty || '',
                transition_duration_ms: s.transitionDuration ? maxMs(s.transitionDuration) : null,
              });
            } else if (context === 'normal') {
              const names = splitList(s.animationName).filter(n => n.toLowerCase() !== 'none');
              if (names.length && animations.length < LIMIT && present(selector)) animations.push({ selector, names });
              const tprops = splitList(s.transitionProperty);
              if (tprops.length && maxMs(s.transitionDuration) > 0 && transitions.length < LIMIT &&
                  present(selector)) {
                transitions.push({ selector, props: tprops });
              }
              if (STATE_RE.test(selector)) for (const p of propsOf(s)) stateProps.add(p);
            }
          }
          if (r.cssRules) walk(Array.from(r.cssRules), context);
        }
      };
      walk(rules, 'normal');
    }
  } catch(e) {}
  return { keyframes, animations, transitions, state_props: Array.from(stateProps), overrides };
})()
"#;

#[derive(Debug, Default, Deserialize)]
struct MotionScan {
    #[serde(default)]
    keyframes: Vec<KeyframesScan>,
    #[serde(default)]
    animations: Vec<AnimationScan>,
    #[serde(default)]
    transitions: Vec<TransitionScan>,
    #[serde(default)]
    state_props: Vec<String>,
    #[serde(default)]
    overrides: Vec<ReduceOverride>,
}

#[derive(Debug, Deserialize)]
struct KeyframesScan {
    name: String,
    props: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AnimationScan {
    selector: String,
    names: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TransitionScan {
    selector: String,
    props: Vec<String>,
}

/// Eine Stilregel innerhalb von `@media (prefers-reduced-motion: reduce)`.
#[derive(Debug, Default, Deserialize)]
struct ReduceOverride {
    selector: String,
    #[serde(default)]
    animation_name: String,
    #[serde(default)]
    animation_duration_ms: Option<f64>,
    #[serde(default)]
    animation_play_state: String,
    #[serde(default)]
    transition_property: String,
    #[serde(default)]
    transition_duration_ms: Option<f64>,
}

impl ReduceOverride {
    fn disables_animation(&self) -> bool {
        self.animation_name.trim().eq_ignore_ascii_case("none")
            || self
                .animation_duration_ms
                .is_some_and(|ms| ms <= NEUTRALISED_MAX_MS)
            || self
                .animation_play_state
                .trim()
                .eq_ignore_ascii_case("paused")
    }

    fn disables_transition(&self) -> bool {
        self.transition_property.trim().eq_ignore_ascii_case("none")
            || self
                .transition_duration_ms
                .is_some_and(|ms| ms <= NEUTRALISED_MAX_MS)
    }

    /// Trifft die Override-Regel den Selektor des Belegs? Universalselektor
    /// oder ein gemeinsamer Teil der Selektorliste.
    fn covers(&self, selector: &str) -> bool {
        let parts = selector_parts(selector);
        selector_parts(&self.selector)
            .iter()
            .any(|own| own.contains('*') || parts.contains(own))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MotionKind {
    Animation { keyframes: String },
    Transition,
}

/// Beleg fuer Bewegung: Selektor der Stilregel und bewegte Eigenschaft.
#[derive(Debug, Clone, PartialEq)]
struct MotionEvidence {
    selector: String,
    kind: MotionKind,
    property: String,
}

impl MotionEvidence {
    fn describe(&self) -> String {
        match &self.kind {
            MotionKind::Animation { keyframes } => format!(
                "`{}` animates `{}` via @keyframes {}",
                self.selector, self.property, keyframes
            ),
            MotionKind::Transition => {
                format!("`{}` transitions `{}`", self.selector, self.property)
            }
        }
    }
}

fn selector_parts(selector: &str) -> Vec<String> {
    selector
        .split(',')
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Bewegungs-Eigenschaft im Sinne von 2.3.3: Lage, Verschiebung, Drehung,
/// Skalierung. Farbe und Deckkraft gehoeren nicht dazu.
fn is_motion_property(prop: &str) -> bool {
    let p = prop.trim().to_ascii_lowercase();
    matches!(
        p.as_str(),
        "transform"
            | "translate"
            | "rotate"
            | "scale"
            | "top"
            | "left"
            | "right"
            | "bottom"
            | "position"
            | "offset-path"
            | "offset-distance"
    ) || p.starts_with("inset")
        || p.starts_with("margin")
}

/// Reine Entscheidung: alle Bewegungsbelege, die kein Reduce-Block abschaltet.
fn unneutralised_motion(scan: &MotionScan) -> Vec<MotionEvidence> {
    let mut evidence = Vec::new();

    for anim in &scan.animations {
        for name in &anim.names {
            let motion_prop = scan
                .keyframes
                .iter()
                .filter(|k| &k.name == name)
                .flat_map(|k| k.props.iter())
                .find(|p| is_motion_property(p));
            if let Some(prop) = motion_prop {
                evidence.push(MotionEvidence {
                    selector: anim.selector.clone(),
                    kind: MotionKind::Animation {
                        keyframes: name.clone(),
                    },
                    property: prop.clone(),
                });
                break;
            }
        }
    }

    let state_motion = scan.state_props.iter().find(|p| is_motion_property(p));
    for transition in &scan.transitions {
        let has_all = transition
            .props
            .iter()
            .any(|p| p.trim().eq_ignore_ascii_case("all"));
        let property = transition
            .props
            .iter()
            .find(|p| is_motion_property(p))
            .cloned()
            .or_else(|| {
                state_motion
                    .filter(|_| has_all)
                    .map(|p| format!("all ({p})"))
            });
        if let Some(property) = property {
            evidence.push(MotionEvidence {
                selector: transition.selector.clone(),
                kind: MotionKind::Transition,
                property,
            });
        }
    }

    evidence.retain(|ev| {
        !scan.overrides.iter().any(|o| {
            o.covers(&ev.selector)
                && match ev.kind {
                    MotionKind::Animation { .. } => o.disables_animation(),
                    MotionKind::Transition => o.disables_transition(),
                }
        })
    });
    evidence
}

pub async fn check_reduced_motion_with_page(page: &Page) -> Vec<Violation> {
    let val =
        match crate::wcag::types::evaluate_or_fail(page, &REDUCED_MOTION_RULE, REDUCED_MOTION_JS)
            .await
        {
            Ok(v) => v,
            Err(violations) => return violations,
        };

    let scan: MotionScan = serde_json::from_value(val).unwrap_or_default();
    let evidence = unneutralised_motion(&scan);
    if evidence.is_empty() {
        return vec![];
    }

    let shown: Vec<String> = evidence
        .iter()
        .take(MAX_EVIDENCE_IN_MESSAGE)
        .map(MotionEvidence::describe)
        .collect();
    let more = evidence.len().saturating_sub(MAX_EVIDENCE_IN_MESSAGE);
    let more_text = if more > 0 {
        format!(" and {more} more")
    } else {
        String::new()
    };

    vec![Violation::new(
        REDUCED_MOTION_RULE.id,
        REDUCED_MOTION_RULE.name,
        REDUCED_MOTION_RULE.level,
        Severity::Medium,
        format!(
            "Stylesheets define motion that is not reduced under `prefers-reduced-motion: reduce` ({}{more_text}). Users with vestibular disorders may experience nausea or dizziness. WCAG 2.3.3 is Level AAA.",
            shown.join("; ")
        ),
        "stylesheet",
    )
    .with_fix(
        "Wrap motion animations/transitions in `@media (prefers-reduced-motion: no-preference) { ... }` or add a `@media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; } }` reset.",
    )
    .with_rule_id(REDUCED_MOTION_RULE.axe_id)
    .with_help_url(REDUCED_MOTION_RULE.help_url)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyframes(name: &str, props: &[&str]) -> KeyframesScan {
        KeyframesScan {
            name: name.into(),
            props: props.iter().map(|p| p.to_string()).collect(),
        }
    }

    fn animation(selector: &str, name: &str) -> AnimationScan {
        AnimationScan {
            selector: selector.into(),
            names: vec![name.into()],
        }
    }

    fn transition(selector: &str, props: &[&str]) -> TransitionScan {
        TransitionScan {
            selector: selector.into(),
            props: props.iter().map(|p| p.to_string()).collect(),
        }
    }

    #[test]
    fn colour_and_opacity_transitions_are_no_motion() {
        let scan = MotionScan {
            transitions: vec![
                transition("a", &["color", "background-color"]),
                transition(".fade", &["opacity"]),
            ],
            ..Default::default()
        };
        assert!(unneutralised_motion(&scan).is_empty());
    }

    #[test]
    fn transform_transition_without_override_is_motion() {
        let scan = MotionScan {
            transitions: vec![transition(".card", &["transform", "opacity"])],
            ..Default::default()
        };
        let ev = unneutralised_motion(&scan);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].property, "transform");
    }

    #[test]
    fn transition_all_counts_only_with_motion_in_state_rule() {
        let mut scan = MotionScan {
            transitions: vec![transition("a", &["all"])],
            state_props: vec!["color".into()],
            ..Default::default()
        };
        assert!(unneutralised_motion(&scan).is_empty());
        scan.state_props.push("transform".into());
        assert_eq!(unneutralised_motion(&scan).len(), 1);
    }

    #[test]
    fn animation_counts_only_with_motion_keyframes() {
        let mut scan = MotionScan {
            keyframes: vec![keyframes("pulse", &["opacity", "color"])],
            animations: vec![animation(".dot", "pulse")],
            ..Default::default()
        };
        assert!(unneutralised_motion(&scan).is_empty());
        scan.keyframes.push(keyframes("spin", &["transform"]));
        scan.animations.push(animation(".spinner", "spin"));
        let ev = unneutralised_motion(&scan);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].selector, ".spinner");
    }

    #[test]
    fn unknown_keyframes_are_no_evidence() {
        let scan = MotionScan {
            animations: vec![animation(".x", "from-cross-origin-sheet")],
            ..Default::default()
        };
        assert!(unneutralised_motion(&scan).is_empty());
    }

    #[test]
    fn universal_reduce_reset_neutralises_everything() {
        let scan = MotionScan {
            keyframes: vec![keyframes("slide", &["margin-left"])],
            animations: vec![animation(".banner", "slide")],
            transitions: vec![transition(".card", &["transform"])],
            overrides: vec![ReduceOverride {
                selector: "*, ::before, ::after".into(),
                animation_duration_ms: Some(0.01),
                transition_duration_ms: Some(0.01),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(unneutralised_motion(&scan).is_empty());
    }

    #[test]
    fn override_must_hit_selector_and_kind() {
        let mut scan = MotionScan {
            keyframes: vec![keyframes("spin", &["transform"])],
            animations: vec![animation(".spinner", "spin")],
            overrides: vec![ReduceOverride {
                selector: ".other".into(),
                animation_name: "none".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(unneutralised_motion(&scan).len(), 1);

        // Richtiger Selektor, aber nur die Transition abgeschaltet.
        scan.overrides[0].selector = ".spinner".into();
        scan.overrides[0].animation_name = String::new();
        scan.overrides[0].transition_property = "none".into();
        assert_eq!(unneutralised_motion(&scan).len(), 1);

        scan.overrides[0].animation_name = "none".into();
        assert!(unneutralised_motion(&scan).is_empty());
    }

    #[test]
    fn long_override_duration_does_not_neutralise() {
        let scan = MotionScan {
            transitions: vec![transition(".card", &["transform"])],
            overrides: vec![ReduceOverride {
                selector: ".card".into(),
                transition_duration_ms: Some(300.0),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(unneutralised_motion(&scan).len(), 1);
    }
}
