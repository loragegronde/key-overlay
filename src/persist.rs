//! Load / save / migrate `layout.json` (v4 library).

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde_json::Value;

use crate::model::{
    LayoutLibrary, LayoutTemplateId, PressEffect, ProfileConfig, VisualTheme,
    LIBRARY_SCHEMA_VERSION, PROFILE_SCHEMA_VERSION,
};
use crate::templates::{create_default_library, create_profile_from_template, default_key_style};

pub fn layout_path() -> PathBuf {
    let dirs = ProjectDirs::from("com", "keyoverlay", "key-overlay")
        .or_else(|| ProjectDirs::from("com", "keyoverlay", "app"));
    // Prefer the historical Tauri identifier path when possible.
    if let Some(base) = std::env::var_os("APPDATA") {
        let legacy = PathBuf::from(base)
            .join("com.keyoverlay.app")
            .join("layout.json");
        if legacy.exists() {
            return legacy;
        }
        // Still write/read under the same historical folder on Windows.
        return PathBuf::from(
            std::env::var_os("APPDATA").expect("APPDATA checked above"),
        )
        .join("com.keyoverlay.app")
        .join("layout.json");
    }
    if let Some(dirs) = dirs {
        return dirs.config_dir().join("layout.json");
    }
    PathBuf::from("layout.json")
}

pub fn load_library() -> LayoutLibrary {
    let path = layout_path();
    match fs::read_to_string(&path) {
        Ok(raw) => match serde_json::from_str::<Value>(&raw) {
            Ok(value) => normalize_library(&value).unwrap_or_else(create_default_library),
            Err(err) => {
                eprintln!("could not parse layout.json: {err}");
                create_default_library()
            }
        },
        Err(_) => create_default_library(),
    }
}

pub fn save_library(library: &LayoutLibrary) -> Result<(), String> {
    let path = layout_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut out = library.clone();
    out.version = LIBRARY_SCHEMA_VERSION;
    for p in &mut out.profiles {
        p.version = PROFILE_SCHEMA_VERSION;
    }
    let json = serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn export_profile(path: &std::path::Path, profile: &ProfileConfig) -> Result<(), String> {
    let mut p = profile.clone();
    p.version = PROFILE_SCHEMA_VERSION;
    let json = serde_json::to_string_pretty(&p).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn import_profile(path: &std::path::Path) -> Result<ProfileConfig, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    normalize_profile(&value).ok_or_else(|| "that file is not a Key Overlay profile".into())
}

fn str_field(v: &Value, key: &str, fallback: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn num_field(v: &Value, key: &str, fallback: f64) -> f64 {
    v.get(key).and_then(|x| x.as_f64()).unwrap_or(fallback)
}

fn bool_field(v: &Value, key: &str, fallback: bool) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(fallback)
}

fn normalize_library(raw: &Value) -> Option<LayoutLibrary> {
    if let Some(profiles) = raw.get("profiles").and_then(|p| p.as_array()) {
        let version = num_field(raw, "version", 0.0) as u32;
        if version > LIBRARY_SCHEMA_VERSION {
            return None;
        }
        let profiles: Vec<ProfileConfig> = profiles
            .iter()
            .filter_map(normalize_profile)
            .collect();
        if profiles.is_empty() {
            return None;
        }
        let active_id = raw
            .get("activeId")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .filter(|id| profiles.iter().any(|p| p.id == *id))
            .unwrap_or_else(|| profiles[0].id.clone());
        return Some(LayoutLibrary {
            version: LIBRARY_SCHEMA_VERSION,
            active_id,
            profiles,
        });
    }
    let single = normalize_profile(raw)?;
    Some(LayoutLibrary {
        version: LIBRARY_SCHEMA_VERSION,
        active_id: single.id.clone(),
        profiles: vec![single],
    })
}

fn normalize_profile(raw: &Value) -> Option<ProfileConfig> {
    if !raw.is_object() {
        return None;
    }
    if raw.get("profiles").is_some() {
        return None;
    }
    let keys = raw.get("keys")?.as_array()?;
    let version = num_field(raw, "version", 0.0) as u32;
    if version > PROFILE_SCHEMA_VERSION {
        return None;
    }
    let now = chrono::Utc::now().to_rfc3339();
    let keys = keys
        .iter()
        .enumerate()
        .filter_map(|(i, k)| normalize_key(k, i))
        .collect();
    Some(ProfileConfig {
        version: PROFILE_SCHEMA_VERSION,
        id: str_field(raw, "id", &format!("profile-{}", chrono::Utc::now().timestamp_millis())),
        name: str_field(raw, "name", "My Layout"),
        template_id: parse_template(raw.get("templateId").and_then(|x| x.as_str())),
        keys,
        global_theme: parse_theme(raw.get("globalTheme").and_then(|x| x.as_str())),
        show_kps_meter: bool_field(raw, "showKpsMeter", true),
        window_opacity: (num_field(raw, "windowOpacity", 1.0) as f32).clamp(0.1, 1.0),
        snap_to_grid: bool_field(raw, "snapToGrid", false),
        grid_size: (num_field(raw, "gridSize", DEFAULT_GRID as f64) as f32).clamp(2.0, 100.0),
        target_app_enabled: bool_field(raw, "targetAppEnabled", false),
        target_app_match: str_field(raw, "targetAppMatch", ""),
        created_at: str_field(raw, "createdAt", &now),
        updated_at: str_field(raw, "updatedAt", &now),
    })
}

const DEFAULT_GRID: f32 = 10.0;

fn normalize_key(raw: &Value, index: usize) -> Option<crate::model::KeyConfig> {
    let code = raw.get("code")?.as_str()?.to_string();
    let style = raw.get("style").map(normalize_style).unwrap_or_default();
    Some(crate::model::KeyConfig {
        id: str_field(raw, "id", &format!("key-{index}-{}", chrono::Utc::now().timestamp_millis())),
        label: str_field(raw, "label", &code),
        code,
        x: num_field(raw, "x", 100.0) as f32,
        y: num_field(raw, "y", 100.0) as f32,
        width: num_field(raw, "width", 56.0) as f32,
        height: num_field(raw, "height", 56.0) as f32,
        shape: parse_shape(raw.get("shape").and_then(|x| x.as_str())),
        rotation: num_field(raw, "rotation", 0.0) as f32,
        scale: num_field(raw, "scale", 1.0) as f32,
        style,
    })
}

fn normalize_style(raw: &Value) -> crate::model::KeyStyle {
    let d = default_key_style();
    crate::model::KeyStyle {
        background_color: str_field(raw, "backgroundColor", &d.background_color),
        border_color: str_field(raw, "borderColor", &d.border_color),
        active_glow_color: str_field(raw, "activeGlowColor", &d.active_glow_color),
        text_color: str_field(raw, "textColor", &d.text_color),
        border_radius: num_field(raw, "borderRadius", d.border_radius as f64) as f32,
        opacity: num_field(raw, "opacity", d.opacity as f64) as f32,
        font_family: str_field(raw, "fontFamily", &d.font_family),
        font_size: num_field(raw, "fontSize", d.font_size as f64) as f32,
        press_effect: parse_effect(raw.get("pressEffect").and_then(|x| x.as_str())),
        show_label: bool_field(raw, "showLabel", d.show_label),
        show_press_count: bool_field(raw, "showPressCount", d.show_press_count),
        show_duration: bool_field(raw, "showDuration", d.show_duration),
    }
}

fn parse_shape(v: Option<&str>) -> crate::model::KeyShape {
    match v {
        Some("circle") => crate::model::KeyShape::Circle,
        Some("stick") => crate::model::KeyShape::Stick,
        _ => crate::model::KeyShape::Rectangle,
    }
}

fn parse_effect(v: Option<&str>) -> PressEffect {
    match v {
        Some("glow-pulse") | Some("pulse") => PressEffect::GlowPulse,
        Some("key-drop") | Some("bounce") => PressEffect::KeyDrop,
        Some("border-ripple") | Some("ripple") => PressEffect::BorderRipple,
        Some("none") => PressEffect::None,
        Some("glow") | Some("trail") | _ => PressEffect::Glow,
    }
}

fn parse_theme(v: Option<&str>) -> VisualTheme {
    match v {
        Some("glassmorphism") => VisualTheme::Glassmorphism,
        Some("retro-arcade") => VisualTheme::RetroArcade,
        Some("stealth-minimal") | Some("neumorphism") => VisualTheme::StealthMinimal,
        Some("rgb-wave") | Some("rgb-gradient") => VisualTheme::RgbWave,
        Some("custom") => VisualTheme::Custom,
        _ => VisualTheme::Cyberpunk,
    }
}

fn parse_template(v: Option<&str>) -> LayoutTemplateId {
    match v {
        Some("full-100") => LayoutTemplateId::Full100,
        Some("tkl") => LayoutTemplateId::Tkl,
        Some("60-percent") => LayoutTemplateId::SixtyPercent,
        Some("fighting-arcade") => LayoutTemplateId::FightingArcade,
        Some("streamer-hud") => LayoutTemplateId::StreamerHud,
        Some("controller") => LayoutTemplateId::Controller,
        Some("custom") => LayoutTemplateId::Custom,
        _ => LayoutTemplateId::WasdGaming,
    }
}

#[allow(dead_code)]
pub fn ensure_active(library: &mut LayoutLibrary) -> &ProfileConfig {
    if library.profiles.is_empty() {
        *library = create_default_library();
    }
    if !library.profiles.iter().any(|p| p.id == library.active_id) {
        library.active_id = library.profiles[0].id.clone();
    }
    library
        .profiles
        .iter()
        .find(|p| p.id == library.active_id)
        .unwrap()
}

#[allow(dead_code)]
pub fn default_profile() -> ProfileConfig {
    create_profile_from_template(LayoutTemplateId::WasdGaming, None)
}
