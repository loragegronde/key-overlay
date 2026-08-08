//! Persisted layout schema (v4 library), matching the former TypeScript types.

use serde::{Deserialize, Serialize};

pub const PROFILE_SCHEMA_VERSION: u32 = 4;
pub const LIBRARY_SCHEMA_VERSION: u32 = 4;
pub const DEFAULT_GRID_SIZE: f32 = 10.0;

pub const HOTKEY_TOGGLE_VISIBILITY: &str = "Ctrl+Shift+O";
pub const HOTKEY_TOGGLE_LOCK: &str = "Ctrl+Shift+L";
pub const HOTKEY_OPEN_EDITOR: &str = "Ctrl+Shift+E";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PressEffect {
    #[default]
    Glow,
    #[serde(rename = "glow-pulse")]
    GlowPulse,
    #[serde(rename = "key-drop")]
    KeyDrop,
    #[serde(rename = "border-ripple")]
    BorderRipple,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum VisualTheme {
    #[default]
    Cyberpunk,
    Glassmorphism,
    #[serde(rename = "retro-arcade")]
    RetroArcade,
    #[serde(rename = "stealth-minimal")]
    StealthMinimal,
    #[serde(rename = "rgb-wave")]
    RgbWave,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutTemplateId {
    #[serde(rename = "full-100")]
    Full100,
    Tkl,
    #[serde(rename = "60-percent")]
    SixtyPercent,
    #[serde(rename = "wasd-gaming")]
    #[default]
    WasdGaming,
    #[serde(rename = "fighting-arcade")]
    FightingArcade,
    #[serde(rename = "streamer-hud")]
    StreamerHud,
    Controller,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeyShape {
    #[default]
    Rectangle,
    Circle,
    Stick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyStyle {
    pub background_color: String,
    pub border_color: String,
    pub active_glow_color: String,
    pub text_color: String,
    pub border_radius: f32,
    pub opacity: f32,
    pub font_family: String,
    pub font_size: f32,
    pub press_effect: PressEffect,
    pub show_label: bool,
    pub show_press_count: bool,
    pub show_duration: bool,
}

impl Default for KeyStyle {
    fn default() -> Self {
        Self {
            background_color: "rgba(20, 20, 40, 0.75)".into(),
            border_color: "rgba(0, 255, 255, 0.5)".into(),
            active_glow_color: "rgba(0, 255, 255, 0.9)".into(),
            text_color: "#ffffff".into(),
            border_radius: 8.0,
            opacity: 1.0,
            font_family: "JetBrains Mono".into(),
            font_size: 14.0,
            press_effect: PressEffect::Glow,
            show_label: true,
            show_press_count: false,
            show_duration: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyConfig {
    pub id: String,
    pub code: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub shape: KeyShape,
    pub rotation: f32,
    pub scale: f32,
    pub style: KeyStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileConfig {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub template_id: LayoutTemplateId,
    pub keys: Vec<KeyConfig>,
    pub global_theme: VisualTheme,
    pub show_kps_meter: bool,
    pub window_opacity: f32,
    pub snap_to_grid: bool,
    pub grid_size: f32,
    pub target_app_enabled: bool,
    pub target_app_match: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutLibrary {
    pub version: u32,
    pub active_id: String,
    pub profiles: Vec<ProfileConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignEdge {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerTab {
    Visuals,
    Themes,
    Animations,
    Layouts,
    Settings,
}
