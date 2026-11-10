//! Rendering-Probes: Messungen an der laufenden Seite für die geteilten Regeln
//! `frames/cross-origin`, `orientation/root-rotated`, `text-spacing/clipped`,
//! `focus/obscured(-enhanced)`, `focus/indicator-missing` und
//! `auth/paste-blocked` (a11y-dom 0.29, `Rendering::frame_reachable` und
//! Nachbarn).
//!
//! # Regel dieser Datei
//!
//! Eine Messung ist **gemessen** oder **nicht gemessen**, nie dazwischen. Ein
//! Durchgang, der scheitert (Ausnahme im Skript, Zeitüberschreitung, kein
//! konstruierbares Stylesheet, Überschreibung kam am Element nicht an), liefert
//! `None` für die ganze Messung; die geteilte Regel macht daraus
//! `NotRun::CapabilityMissing` und nie ein Bestehen. Ein Element ohne Zeile in
//! einer gelaufenen Messung ist ebenfalls „nicht gemessen".
//!
//! # Wie die Werte zurückkommen
//!
//! Jedes Skript (`js/probes/*.js`, mit dem Vorspann `common.js`) liefert
//! `{ els, json }`: die Elemente als Array und die Zeilen als JSON-Text, die
//! Elemente über ihren Index nennend. Ein Aufruf holt das Ergebnis, ein
//! weiterer die Elementliste, dann je Element ein `DOM.describeNode` für die
//! Backend-ID -- derselbe Weg wie bei den Überdeckungs- und Animationsdurchgängen
//! (`fetch_element_ids`). Über die Backend-ID finden die Werte an jedes
//! Dokument der Seite, auch an die der iframes.
//!
//! # Was die Messungen berühren
//!
//! Lesend sind Rahmen und Wurzeldrehung. Einfügen (Handler laufen, `value`
//! wird für einen Moment geschrieben), Fokus (Fokus und Scrollposition) und
//! Textabstand (ein adoptiertes Stylesheet, solange gemessen wird) verändern die
//! Seite und stellen den Zustand in einem `finally` wieder her. Was sich nicht
//! zurücknehmen lässt -- ein `focus`-Handler der Seite, der selbst etwas tut --,
//! bleibt.

use std::collections::HashMap;

use a11y_dom::{BoxTransform, FocusCover, PasteProbe, SpacingClip};
use chromiumoxide::cdp::browser_protocol::dom::DescribeNodeParams;
use chromiumoxide::cdp::js_protocol::runtime::{
    EvaluateParams, GetPropertiesParams, ReleaseObjectGroupParams, TimeDelta,
};
use chromiumoxide::Page;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::error::{AuditError, Result};

const COMMON_JS: &str = include_str!("js/probes/common.js");
const FRAMES_JS: &str = include_str!("js/probes/frames.js");
const TRANSFORMS_JS: &str = include_str!("js/probes/transforms.js");
const PASTE_JS: &str = include_str!("js/probes/paste.js");
const FOCUS_JS: &str = include_str!("js/probes/focus.js");
const SPACING_JS: &str = include_str!("js/probes/spacing.js");

/// Obergrenze für das Skript einer Messung. Eine Seite, deren Handler hängt,
/// hielte sonst den ganzen Audit an.
const PROBE_TIMEOUT_MS: f64 = 15_000.0;

const GROUP: &str = "ams-probe";

/// Welche Messungen ein Lauf macht. Die Zuordnung zu den Regeln (Stufe und
/// `[rules]`-Filter) steht in `wcag::shared::probe_selection`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProbeSelection {
    /// `frames/cross-origin` -- lesend.
    pub frames: bool,
    /// `orientation/root-rotated` -- lesend.
    pub transforms: bool,
    /// `auth/paste-blocked` -- berührt die Seite.
    pub paste: bool,
    /// `focus/obscured`, `focus/obscured-enhanced`, `focus/indicator-missing`
    /// -- berührt die Seite.
    pub focus: bool,
    /// `text-spacing/clipped` -- berührt die Seite.
    pub spacing: bool,
}

impl ProbeSelection {
    pub const NONE: ProbeSelection = ProbeSelection {
        frames: false,
        transforms: false,
        paste: false,
        focus: false,
        spacing: false,
    };
    pub const ALL: ProbeSelection = ProbeSelection {
        frames: true,
        transforms: true,
        paste: true,
        focus: true,
        spacing: true,
    };
}

/// Ein Vermerk zu einer Messung, die nicht oder nicht ganz gelaufen ist.
/// Kommt als `reason` an den Ausführungsvermerk der betroffenen Regeln.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeNote {
    /// Die Messung: `frames`, `transforms`, `paste`, `focus` oder `spacing`.
    pub probe: &'static str,
    pub text: String,
}

/// Messwerte an einem Element der Fokusmessung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusMeasure {
    pub points: u32,
    pub covered: u32,
    /// Backend-ID des obersten überdeckenden Elements.
    pub blocker: Option<i64>,
    /// Ob sich beim Fokussieren ein Stil änderte, der als Indikator taugt;
    /// `None`, wo der Fokus nicht auf dem Element landete (inert, vom Skript
    /// wieder genommen) -- dann auch ohne Abtastpunkte.
    pub indicator: Option<bool>,
}

/// Die Ergebnisse aller Messungen eines Laufs, je Backend-ID. `None` heißt
/// „Messung nicht gelaufen oder gescheitert".
#[derive(Debug, Clone, Default)]
pub struct Probes {
    pub frames: Option<HashMap<i64, bool>>,
    pub transforms: Option<HashMap<i64, BoxTransform>>,
    pub paste: Option<HashMap<i64, PasteProbe>>,
    pub focus: Option<HashMap<i64, FocusMeasure>>,
    pub spacing: Option<HashMap<i64, SpacingClip>>,
    pub notes: Vec<ProbeNote>,
}

impl Probes {
    pub(crate) fn focus_cover(&self, backend: i64) -> Option<FocusCover> {
        let m = self.focus.as_ref()?.get(&backend)?;
        Some(FocusCover {
            points: m.points,
            covered: m.covered,
            // Die Kennung im Arena-Baum setzt der Aufrufer ein.
            blocker: None,
        })
    }

    /// Wie viele Elemente jede Messung gefüllt hat; `None` = nicht gemessen.
    pub fn filled(&self) -> ProbeCounts {
        ProbeCounts {
            frames: self.frames.as_ref().map(HashMap::len),
            transforms: self.transforms.as_ref().map(HashMap::len),
            paste: self.paste.as_ref().map(HashMap::len),
            focus: self.focus.as_ref().map(HashMap::len),
            spacing: self.spacing.as_ref().map(HashMap::len),
        }
    }

    pub fn notes(&self) -> &[ProbeNote] {
        &self.notes
    }
}

/// Zeilen je Messung; `None` = nicht gemessen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeCounts {
    pub frames: Option<usize>,
    pub transforms: Option<usize>,
    pub paste: Option<usize>,
    pub focus: Option<usize>,
    pub spacing: Option<usize>,
}

// ── Ablauf ───────────────────────────────────────────────────────────────────

/// Führt die gewählten Messungen aus. Die lesenden zuerst, dann die
/// berührenden; scheitert eine, bleibt nur ihr Feld leer.
pub async fn run_probes(page: &Page, selection: ProbeSelection) -> Probes {
    let mut probes = Probes::default();
    if selection.frames {
        probes.frames = finish(&mut probes.notes, "frames", measure_frames(page).await);
    }
    if selection.transforms {
        probes.transforms = finish(
            &mut probes.notes,
            "transforms",
            measure_transforms(page).await,
        );
    }
    if selection.paste {
        probes.paste = finish(&mut probes.notes, "paste", measure_paste(page).await);
    }
    if selection.focus {
        probes.focus = finish(&mut probes.notes, "focus", measure_focus(page).await);
    }
    if selection.spacing {
        probes.spacing = finish(&mut probes.notes, "spacing", measure_spacing(page).await);
    }
    probes
}

/// Ergebnis einer Messung: die Werte und ein Vermerk, wenn nicht alles
/// gemessen wurde (Deckel).
struct Measured<T> {
    values: HashMap<i64, T>,
    note: Option<String>,
}

fn finish<T>(
    notes: &mut Vec<ProbeNote>,
    probe: &'static str,
    result: Result<Measured<T>>,
) -> Option<HashMap<i64, T>> {
    match result {
        Ok(measured) => {
            if let Some(text) = measured.note {
                notes.push(ProbeNote { probe, text });
            }
            Some(measured.values)
        }
        Err(e) => {
            warn!("Messung {probe} gescheitert: {e}");
            notes.push(ProbeNote {
                probe,
                text: format!("probe_failed: {e}"),
            });
            None
        }
    }
}

/// Was ein Skript geliefert hat: Backend-IDs seiner Elemente (nach Index in
/// `els`; `None`, wo `describeNode` scheiterte) und der JSON-Text.
struct ScriptOutput {
    ids: Vec<Option<i64>>,
    json: serde_json::Value,
}

async fn run_script(page: &Page, pass: &str, body: &str) -> Result<ScriptOutput> {
    let failed = |what: &str, e: String| AuditError::AXTreeExtractionFailed {
        reason: format!("{pass}: {what}: {e}"),
    };
    let expression = format!("(function() {{\n{COMMON_JS}\n{body}\n}})()");
    let params = EvaluateParams::builder()
        .expression(expression)
        .object_group(GROUP)
        .return_by_value(false)
        .timeout(TimeDelta::new(PROBE_TIMEOUT_MS))
        .build()
        .map_err(|e| failed("Parameter", e))?;
    let result = async {
        let evaluated = page
            .execute(params)
            .await
            .map_err(|e| failed("Skript", e.to_string()))?;
        if let Some(exception) = &evaluated.result.exception_details {
            let detail = exception
                .exception
                .as_ref()
                .and_then(|e| e.description.clone())
                .unwrap_or_else(|| exception.text.clone());
            return Err(failed("Skript", detail));
        }
        let Some(result_id) = evaluated.result.result.object_id.clone() else {
            return Err(failed("Skript", "kein Ergebnis".to_string()));
        };
        let properties = page
            .execute(
                GetPropertiesParams::builder()
                    .object_id(result_id)
                    .own_properties(true)
                    .build()
                    .map_err(|e| failed("Parameter", e))?,
            )
            .await
            .map_err(|e| failed("Ergebnis", e.to_string()))?;
        let mut els = None;
        let mut json = None;
        for p in &properties.result.result {
            match (p.name.as_str(), p.value.as_ref()) {
                ("els", Some(v)) => els = v.object_id.clone(),
                ("json", Some(v)) => json = v.value.clone(),
                _ => {}
            }
        }
        let (Some(els), Some(serde_json::Value::String(json))) = (els, json) else {
            return Err(failed("Skript", "Ergebnis ohne els/json".to_string()));
        };
        let json: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| failed("JSON", e.to_string()))?;

        let listed = page
            .execute(
                GetPropertiesParams::builder()
                    .object_id(els)
                    .own_properties(true)
                    .build()
                    .map_err(|e| failed("Parameter", e))?,
            )
            .await
            .map_err(|e| failed("Elementliste", e.to_string()))?;
        let mut indexed: Vec<(usize, String)> = listed
            .result
            .result
            .iter()
            .filter_map(|p| Some((p.name.parse().ok()?, p.value.as_ref()?.object_id.clone()?)))
            .map(|(i, id): (usize, _)| (i, id.inner().clone()))
            .collect();
        indexed.sort_by_key(|(i, _)| *i);
        let len = indexed.last().map_or(0, |(i, _)| i + 1);
        let describes = indexed.iter().map(|(_, id)| {
            page.execute(
                DescribeNodeParams::builder()
                    .object_id(
                        chromiumoxide::cdp::js_protocol::runtime::RemoteObjectId::new(id.clone()),
                    )
                    .build(),
            )
        });
        let mut ids = vec![None; len];
        for ((i, _), described) in indexed
            .iter()
            .zip(futures::future::join_all(describes).await)
        {
            ids[*i] = described
                .ok()
                .map(|d| *d.result.node.backend_node_id.inner());
        }
        Ok(ScriptOutput { ids, json })
    }
    .await;
    if let Err(e) = page.execute(ReleaseObjectGroupParams::new(GROUP)).await {
        debug!("Objektgruppe {GROUP} nicht freigegeben: {e}");
    }
    result
}

fn rows<T: for<'de> Deserialize<'de>>(json: &serde_json::Value, pass: &str) -> Result<Vec<T>> {
    serde_json::from_value(json.get("rows").cloned().unwrap_or_default()).map_err(|e| {
        AuditError::AXTreeExtractionFailed {
            reason: format!("{pass}: Zeilen: {e}"),
        }
    })
}

fn skipped(json: &serde_json::Value, what: &str, cap: &str) -> Option<String> {
    let n = json.get("skipped")?.as_u64().filter(|n| *n > 0)?;
    Some(format!("probe_capped: {n} {what} over the cap of {cap}"))
}

// ── Frames ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FrameRow {
    e: usize,
    r: bool,
}

async fn measure_frames(page: &Page) -> Result<Measured<bool>> {
    let out = run_script(page, "Rahmendurchgang", FRAMES_JS).await?;
    let mut values = HashMap::new();
    for row in rows::<FrameRow>(&out.json, "Rahmendurchgang")? {
        if let Some(Some(id)) = out.ids.get(row.e) {
            values.insert(*id, row.r);
        }
    }
    Ok(Measured { values, note: None })
}

// ── Transform ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TransformRow {
    e: usize,
    t: String,
    r: String,
}

async fn measure_transforms(page: &Page) -> Result<Measured<BoxTransform>> {
    let out = run_script(page, "Drehungsdurchgang", TRANSFORMS_JS).await?;
    let mut values = HashMap::new();
    for row in rows::<TransformRow>(&out.json, "Drehungsdurchgang")? {
        if let Some(Some(id)) = out.ids.get(row.e) {
            // Eine leere Zeichenkette heißt: der Browser kennt die Eigenschaft
            // nicht -- nicht gemessen, nicht „keine Drehung".
            let some = |s: String| (!s.trim().is_empty()).then_some(s);
            values.insert(
                *id,
                BoxTransform {
                    transform: some(row.t),
                    rotate: some(row.r),
                },
            );
        }
    }
    Ok(Measured { values, note: None })
}

// ── Einfügen ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PasteRow {
    e: usize,
    c: bool,
    i: bool,
}

async fn measure_paste(page: &Page) -> Result<Measured<PasteProbe>> {
    let out = run_script(page, "Einfügedurchgang", PASTE_JS).await?;
    let mut values = HashMap::new();
    for row in rows::<PasteRow>(&out.json, "Einfügedurchgang")? {
        if let Some(Some(id)) = out.ids.get(row.e) {
            values.insert(
                *id,
                PasteProbe {
                    cancelled: row.c,
                    inserted: row.i,
                },
            );
        }
    }
    let note = skipped(&out.json, "fields", "50");
    Ok(Measured { values, note })
}

// ── Fokus ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FocusRow {
    e: usize,
    p: u32,
    c: u32,
    v: Option<bool>,
    b: i64,
}

async fn measure_focus(page: &Page) -> Result<Measured<FocusMeasure>> {
    let out = run_script(page, "Fokusdurchgang", FOCUS_JS).await?;
    let mut values = HashMap::new();
    for row in rows::<FocusRow>(&out.json, "Fokusdurchgang")? {
        let Some(Some(id)) = out.ids.get(row.e) else {
            continue;
        };
        let blocker = usize::try_from(row.b)
            .ok()
            .and_then(|b| out.ids.get(b).copied().flatten());
        values.insert(
            *id,
            FocusMeasure {
                points: row.p,
                covered: row.c,
                blocker,
                indicator: row.v,
            },
        );
    }
    let note = skipped(&out.json, "controls", "300");
    Ok(Measured { values, note })
}

// ── Textabstand ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SpacingRow {
    e: usize,
    b: bool,
    a: bool,
}

async fn measure_spacing(page: &Page) -> Result<Measured<SpacingClip>> {
    let out = run_script(page, "Textabstandsdurchgang", SPACING_JS).await?;
    if let Some(why) = out.json.get("failed").and_then(|f| f.as_str()) {
        return Err(AuditError::AXTreeExtractionFailed {
            reason: format!("Textabstandsdurchgang: Überschreibung nicht angewendet ({why})"),
        });
    }
    let mut values = HashMap::new();
    for row in rows::<SpacingRow>(&out.json, "Textabstandsdurchgang")? {
        if let Some(Some(id)) = out.ids.get(row.e) {
            values.insert(
                *id,
                SpacingClip {
                    before: row.b,
                    after: row.a,
                },
            );
        }
    }
    let note = skipped(&out.json, "elements", "5000");
    Ok(Measured { values, note })
}
