//! Window control and layout persistence exposed to the frontend.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use crate::app_filter;
use crate::{EDITOR_WINDOW, OVERLAY_WINDOW};

const LAYOUT_FILE: &str = "layout.json";

fn overlay_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window(OVERLAY_WINDOW)
        .ok_or_else(|| format!("no window labelled '{OVERLAY_WINDOW}'"))
}

fn editor_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window(EDITOR_WINDOW)
        .ok_or_else(|| format!("no window labelled '{EDITOR_WINDOW}'"))
}

/// Makes the overlay ignore the cursor so clicks land on whatever is behind it.
#[tauri::command]
pub async fn toggle_click_through(app: AppHandle, enabled: bool) -> Result<(), String> {
    overlay_window(&app)?
        .set_ignore_cursor_events(enabled)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn set_always_on_top(app: AppHandle, enabled: bool) -> Result<(), String> {
    overlay_window(&app)?
        .set_always_on_top(enabled)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn minimize_window(app: AppHandle) -> Result<(), String> {
    editor_window(&app)?.minimize().map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// Closes only the editor. The overlay (and tray) keep running if the overlay
/// has been launched.
#[tauri::command]
pub async fn close_editor(app: AppHandle) -> Result<(), String> {
    if let Some(editor) = app.get_webview_window(EDITOR_WINDOW) {
        editor.close().map_err(|err| err.to_string())?;
    }
    Ok(())
}

/// Shows the editor window again (e.g. from the tray).
#[tauri::command]
pub async fn open_editor(app: AppHandle) -> Result<(), String> {
    let editor = editor_window(&app)?;
    editor.show().map_err(|err| err.to_string())?;
    editor.set_focus().map_err(|err| err.to_string())?;
    Ok(())
}

/// Launches the HUD overlay: marks it live, shows it (subject to app filter),
/// and puts it in click-through mode. Emits `overlay-launched` so the overlay
/// webview reloads the saved profile.
#[tauri::command]
pub async fn launch_overlay(app: AppHandle, positioning: bool) -> Result<(), String> {
    let overlay = overlay_window(&app)?;

    app_filter::set_overlay_live(true);
    app_filter::set_manual_visible(true);

    overlay
        .set_always_on_top(true)
        .map_err(|err| err.to_string())?;

    // Positioning mode keeps the window interactive so the user can drag it.
    overlay
        .set_ignore_cursor_events(!positioning)
        .map_err(|err| err.to_string())?;

    app_filter::apply_visibility(&app);

    let _ = app.emit("overlay-launched", ());
    let _ = app.emit("profile-changed", ());

    Ok(())
}

/// After positioning, lock the overlay into click-through HUD mode.
#[tauri::command]
pub async fn finish_positioning(app: AppHandle) -> Result<(), String> {
    overlay_window(&app)?
        .set_ignore_cursor_events(true)
        .map_err(|err| err.to_string())?;
    let _ = app.emit("positioning-finished", ());
    Ok(())
}

/// Updates the app-scoped visibility filter used by the foreground watcher.
#[tauri::command]
pub async fn set_app_filter(
    app: AppHandle,
    enabled: bool,
    match_text: String,
) -> Result<(), String> {
    app_filter::set_filter(enabled, match_text);
    app_filter::apply_visibility(&app);
    Ok(())
}

fn layout_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(LAYOUT_FILE))
        .map_err(|err| format!("could not resolve the app config directory: {err}"))
}

fn write_json(path: &Path, config: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("could not create {}: {err}", parent.display()))?;
    }

    let serialised =
        serde_json::to_string_pretty(config).map_err(|err| format!("invalid layout: {err}"))?;

    fs::write(path, serialised).map_err(|err| format!("could not write {}: {err}", path.display()))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {err}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|err| format!("{} is not valid JSON: {err}", path.display()))
}

#[tauri::command]
pub async fn save_layout(app: AppHandle, config: Value) -> Result<(), String> {
    write_json(&layout_path(&app)?, &config)?;
    // Keep the live HUD in sync with editor saves.
    let _ = app.emit("profile-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn load_layout(app: AppHandle) -> Result<Option<Value>, String> {
    let path = layout_path(&app)?;

    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map(Some)
            .map_err(|err| format!("{} is not valid JSON: {err}", path.display())),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("could not read {}: {err}", path.display())),
    }
}

#[tauri::command]
pub async fn export_profile(path: String, config: Value) -> Result<(), String> {
    write_json(Path::new(&path), &config)
}

#[tauri::command]
pub async fn import_profile(path: String) -> Result<Value, String> {
    read_json(Path::new(&path))
}
