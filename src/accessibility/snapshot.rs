//! Die Aufnahme eines Zeitpunkts im Browser.
//!
//! Die Struktur einer Aufnahme — [`AXSnapshot`], [`FocusSnapshot`] — liegt im
//! Crate `a11y_perception`. Hier steht nur, wie sie zustande kommt: über CDP,
//! also mit einem Browser. Das ist die Grenze zwischen Erhebung und Berechnung.

use a11y_perception::{AXSnapshot, FocusSnapshot};
use chromiumoxide::Page;

/// Nimmt den aktuellen Zustand der Seite auf: Accessibility-Tree, Fokus, URL
/// und Titel.
///
/// Das ist der Eingang des Journey-Layers: was Assistive Technology zu diesem
/// Zeitpunkt überhaupt wahrnehmen kann. Zwei Aufnahmen um eine Handlung herum
/// ergeben über `a11y_perception::AXTreeDiff` die wahrnehmbare Änderung.
///
/// Der Aufrufer sorgt dafür, dass die Seite vorher zur Ruhe gekommen ist
/// (`interaction::stability::settle`) — sonst nimmt die Aufnahme einen
/// Zwischenzustand auf.
pub async fn capture(
    page: &Page,
    label: impl Into<String>,
    timestamp_ms: u64,
) -> crate::error::Result<AXSnapshot> {
    let tree = super::extractor::extract_ax_tree(page).await?;
    let mut focus: FocusSnapshot = crate::interaction::focus::capture_focus(page).await?;
    // Die Backend-ID des fokussierten Knotens wird hier und nur hier aufgelöst:
    // sie kostet zwei zusätzliche CDP-Aufrufe und wird nur gebraucht, wo eine
    // Aufnahme gegen den Baum gestellt wird.
    focus.active_backend_node_id = crate::interaction::focus::capture_focus_backend_id(page).await;
    focus.ax_node_id = focus
        .active_backend_node_id
        .and_then(|id| tree.node_by_backend_id(id).map(|node| node.node_id.clone()));
    let url = page.url().await.ok().flatten().unwrap_or_default();
    let document_title = page.get_title().await.ok().flatten().unwrap_or_default();
    Ok(AXSnapshot::new(
        label,
        url,
        document_title,
        timestamp_ms,
        tree,
        focus,
    ))
}
