//! Cross-process HUD lock / visibility control.
//!
//! The editor and overlay are separate processes. Global hotkeys live in the
//! editor (or overlay fallback); both sides read/write this small JSON file so
//! lock and visibility always apply to the HUD.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::persist::layout_path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HudControl {
    pub locked: bool,
    pub visible: bool,
    /// When true, HUD ignores key highlights (editor text field has focus).
    #[serde(default)]
    pub suppress_input: bool,
    /// Bumped on every write so consumers notice rapid toggles even if mtime stalls.
    #[serde(default)]
    pub rev: u64,
}

impl Default for HudControl {
    fn default() -> Self {
        Self {
            locked: false,
            visible: true,
            suppress_input: false,
            rev: 0,
        }
    }
}

fn control_path() -> PathBuf {
    layout_path()
        .parent()
        .map(|p| p.join("hud-control.json"))
        .unwrap_or_else(|| PathBuf::from("hud-control.json"))
}

pub fn load() -> HudControl {
    let path = control_path();
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => HudControl::default(),
    }
}

pub fn save(control: &HudControl) -> Result<(), String> {
    let path = control_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string(control).map_err(|e| e.to_string())?;
    // Atomic-ish replace reduces torn reads in the overlay process.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).or_else(|_| fs::write(&path, json)).map_err(|e| e.to_string())
}

pub fn mtime() -> Option<SystemTime> {
    fs::metadata(control_path()).and_then(|m| m.modified()).ok()
}

pub fn reset_for_place() {
    let mut c = load();
    c.locked = false;
    c.visible = true;
    c.suppress_input = false;
    c.rev = c.rev.wrapping_add(1);
    let _ = save(&c);
}

/// Pause HUD key highlighting while the editor is typing in a text field.
pub fn set_suppress_input(suppress: bool) {
    let mut c = load();
    if c.suppress_input == suppress {
        return;
    }
    c.suppress_input = suppress;
    c.rev = c.rev.wrapping_add(1);
    let _ = save(&c);
}

pub fn toggle_lock() -> HudControl {
    let mut c = load();
    c.locked = !c.locked;
    c.rev = c.rev.wrapping_add(1);
    let _ = save(&c);
    c
}

pub fn toggle_visible() -> HudControl {
    let mut c = load();
    c.visible = !c.visible;
    c.rev = c.rev.wrapping_add(1);
    let _ = save(&c);
    c
}
