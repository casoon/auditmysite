//! Keyboard input via CDP `Input.dispatchKeyEvent`.
//!
//! For accessibility journeys we need *real* keyboard events — not synthetic
//! DOM clicks. Real events traverse the full eventing pipeline (keydown →
//! input → keyup) and are what assistive technologies actually observe.
//!
//! Phase 1 ships the primitives. Higher-level journeys in `src/a11y_journey/`
//! sequence them.

use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use chromiumoxide::keys::get_key_definition;
use chromiumoxide::Page;

use crate::error::{AuditError, Result};

/// Press a named key (e.g. `"Tab"`, `"Enter"`, `"Escape"`, `"ArrowDown"`).
///
/// Uses the W3C key names as defined in chromiumoxide's US keyboard layout.
/// Dispatches both `keyDown` and `keyUp` events.
pub async fn press(page: &Page, key: &str) -> Result<()> {
    press_with_modifiers(page, key, 0).await
}

/// Press a named key while holding the given CDP modifier bitmask.
/// Modifiers: 1=Alt, 2=Ctrl, 4=Meta/Cmd, 8=Shift.
pub async fn press_with_modifiers(page: &Page, key: &str, modifiers: i64) -> Result<()> {
    let def = get_key_definition(key).ok_or_else(|| AuditError::InteractionFailed {
        reason: format!("Unknown key: {key}"),
    })?;

    let mut cmd = DispatchKeyEventParams::builder();

    // Mirror chromiumoxide's own `press_key` logic: include `text` for
    // printable single-character keys so the page sees an `input` event.
    let key_down_event_type = if let Some(txt) = def.text {
        cmd = cmd.text(txt);
        DispatchKeyEventType::KeyDown
    } else if def.key.len() == 1 {
        cmd = cmd.text(def.key);
        DispatchKeyEventType::KeyDown
    } else {
        DispatchKeyEventType::RawKeyDown
    };

    cmd = cmd
        .key(def.key)
        .code(def.code)
        .windows_virtual_key_code(def.key_code)
        .native_virtual_key_code(def.key_code)
        .modifiers(modifiers);

    let key_down = cmd
        .clone()
        .r#type(key_down_event_type)
        .build()
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("KeyDown build failed: {e}"),
        })?;
    page.execute(key_down)
        .await
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("KeyDown dispatch failed: {e}"),
        })?;

    let key_up = cmd
        .r#type(DispatchKeyEventType::KeyUp)
        .build()
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("KeyUp build failed: {e}"),
        })?;
    page.execute(key_up)
        .await
        .map_err(|e| AuditError::InteractionFailed {
            reason: format!("KeyUp dispatch failed: {e}"),
        })?;

    Ok(())
}

/// Press the `Tab` key once.
pub async fn press_tab(page: &Page) -> Result<()> {
    press(page, "Tab").await
}

/// Press `Shift+Tab`.
pub async fn press_shift_tab(page: &Page) -> Result<()> {
    press_with_modifiers(page, "Tab", 8).await
}

/// Press `Enter`.
pub async fn press_enter(page: &Page) -> Result<()> {
    press(page, "Enter").await
}

/// Press `Escape`.
pub async fn press_escape(page: &Page) -> Result<()> {
    press(page, "Escape").await
}

/// Press an arrow key. `direction` is `"Up"`, `"Down"`, `"Left"`, or `"Right"`.
pub async fn press_arrow(page: &Page, direction: &str) -> Result<()> {
    let key = arrow_key(direction).ok_or_else(|| AuditError::InteractionFailed {
        reason: format!("Invalid arrow direction: {direction}"),
    })?;
    press(page, key).await
}

fn arrow_key(direction: &str) -> Option<&'static str> {
    match direction {
        "Up" => Some("ArrowUp"),
        "Down" => Some("ArrowDown"),
        "Left" => Some("ArrowLeft"),
        "Right" => Some("ArrowRight"),
        _ => None,
    }
}

/// Type a string of printable characters by dispatching key events per
/// character. For forms where a real `input` event is required.
pub async fn type_text(page: &Page, text: &str) -> Result<()> {
    for ch in text.chars() {
        let s = ch.to_string();
        press(page, &s).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::arrow_key;

    #[test]
    fn arrow_key_maps_supported_directions_to_w3c_key_names() {
        assert_eq!(arrow_key("Up"), Some("ArrowUp"));
        assert_eq!(arrow_key("Down"), Some("ArrowDown"));
        assert_eq!(arrow_key("Left"), Some("ArrowLeft"));
        assert_eq!(arrow_key("Right"), Some("ArrowRight"));
    }

    #[test]
    fn arrow_key_rejects_unknown_directions() {
        for direction in ["", "up", "Forward", "ArrowUp"] {
            assert_eq!(arrow_key(direction), None);
        }
    }
}
