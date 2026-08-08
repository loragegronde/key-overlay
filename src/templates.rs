//! Layout presets and visual themes.

use crate::model::{
    KeyConfig, KeyShape, KeyStyle, LayoutLibrary, LayoutTemplateId, PressEffect, ProfileConfig,
    VisualTheme, DEFAULT_GRID_SIZE, LIBRARY_SCHEMA_VERSION, PROFILE_SCHEMA_VERSION,
};

pub fn default_key_style() -> KeyStyle {
    KeyStyle::default()
}

fn style() -> KeyStyle {
    KeyStyle::default()
}

fn style_bg(bg: &str) -> KeyStyle {
    let mut s = style();
    s.background_color = bg.into();
    s
}

use std::sync::atomic::{AtomicU64, Ordering};

static KEY_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> String {
    let n = KEY_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("key-{n}-{}", chrono::Utc::now().timestamp_millis())
}

pub fn create_key(
    code: &str,
    label: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    shape: KeyShape,
    style: KeyStyle,
) -> KeyConfig {
    KeyConfig {
        id: next_id(),
        code: code.into(),
        label: label.into(),
        x,
        y,
        width,
        height,
        shape,
        rotation: 0.0,
        scale: 1.0,
        style,
    }
}

fn key(code: &str, label: &str, x: f32, y: f32, w: f32, h: f32) -> KeyConfig {
    create_key(code, label, x, y, w, h, KeyShape::Rectangle, style())
}

fn circle(code: &str, label: &str, x: f32, y: f32, size: f32, bg: Option<&str>) -> KeyConfig {
    let s = bg.map(style_bg).unwrap_or_else(style);
    create_key(code, label, x, y, size, size, KeyShape::Circle, s)
}

pub struct TemplateMeta {
    pub id: LayoutTemplateId,
    pub name: &'static str,
    pub description: &'static str,
}

pub fn all_templates() -> Vec<TemplateMeta> {
    vec![
        TemplateMeta {
            id: LayoutTemplateId::WasdGaming,
            name: "WASD Gaming",
            description: "FPS movement cluster + common binds",
        },
        TemplateMeta {
            id: LayoutTemplateId::FightingArcade,
            name: "Arcade Stick",
            description: "6-button fighting game layout",
        },
        TemplateMeta {
            id: LayoutTemplateId::StreamerHud,
            name: "Streamer HUD",
            description: "Minimal overlay for stream displays",
        },
        TemplateMeta {
            id: LayoutTemplateId::Controller,
            name: "Controller",
            description: "Xbox-style pad with joysticks",
        },
        TemplateMeta {
            id: LayoutTemplateId::SixtyPercent,
            name: "60%",
            description: "Compact alphanumeric block",
        },
        TemplateMeta {
            id: LayoutTemplateId::Tkl,
            name: "TKL",
            description: "Tenkeyless-style cluster",
        },
        TemplateMeta {
            id: LayoutTemplateId::Full100,
            name: "Full 100%",
            description: "Extended keyboard overview",
        },
        TemplateMeta {
            id: LayoutTemplateId::Custom,
            name: "Custom",
            description: "Start from scratch",
        },
    ]
}

fn template_keys(id: LayoutTemplateId) -> Vec<KeyConfig> {
    match id {
        LayoutTemplateId::WasdGaming => vec![
            key("KeyW", "W", 120.0, 80.0, 64.0, 64.0),
            key("KeyA", "A", 50.0, 150.0, 64.0, 64.0),
            key("KeyS", "S", 120.0, 150.0, 64.0, 64.0),
            key("KeyD", "D", 190.0, 150.0, 64.0, 64.0),
            key("Space", "Space", 280.0, 150.0, 120.0, 48.0),
            key("ShiftLeft", "Shift", 280.0, 80.0, 80.0, 48.0),
            key("ControlLeft", "Ctrl", 280.0, 210.0, 80.0, 48.0),
            key("KeyE", "E", 420.0, 80.0, 56.0, 56.0),
            key("KeyR", "R", 420.0, 150.0, 56.0, 56.0),
            key("KeyQ", "Q", 420.0, 220.0, 56.0, 56.0),
            circle("Mouseleft", "LMB", 520.0, 100.0, 72.0, None),
            circle("Mouseright", "RMB", 520.0, 190.0, 72.0, None),
        ],
        LayoutTemplateId::FightingArcade => vec![
            key("ArrowUp", "↑", 80.0, 60.0, 56.0, 56.0),
            key("ArrowLeft", "←", 20.0, 120.0, 56.0, 56.0),
            key("ArrowDown", "↓", 80.0, 120.0, 56.0, 56.0),
            key("ArrowRight", "→", 140.0, 120.0, 56.0, 56.0),
            circle("KeyU", "LP", 280.0, 80.0, 64.0, Some("rgba(239,68,68,0.7)")),
            circle("KeyI", "MP", 360.0, 80.0, 64.0, Some("rgba(249,115,22,0.7)")),
            circle("KeyO", "HP", 440.0, 80.0, 64.0, Some("rgba(234,179,8,0.7)")),
            circle("KeyJ", "LK", 280.0, 160.0, 64.0, Some("rgba(59,130,246,0.7)")),
            circle("KeyK", "MK", 360.0, 160.0, 64.0, Some("rgba(139,92,246,0.7)")),
            circle("KeyL", "HK", 440.0, 160.0, 64.0, Some("rgba(236,72,153,0.7)")),
        ],
        LayoutTemplateId::StreamerHud => {
            let mut hotkey = key("KeyW", "Hot", 40.0, 40.0, 100.0, 48.0);
            hotkey.style.show_press_count = true;
            hotkey.style.background_color = "rgba(0,0,0,0.5)".into();
            vec![
                hotkey,
                key("KeyM", "Mic", 160.0, 40.0, 64.0, 48.0),
                key("KeyC", "Cam", 240.0, 40.0, 64.0, 48.0),
                key("F1", "Scene", 320.0, 40.0, 72.0, 48.0),
            ]
        }
        LayoutTemplateId::Controller => {
            let mut ls = create_key(
                "PadLS",
                "LS",
                60.0,
                160.0,
                96.0,
                96.0,
                KeyShape::Stick,
                {
                    let mut s = style();
                    s.border_radius = 48.0;
                    s
                },
            );
            let mut rs = create_key(
                "PadRS",
                "RS",
                280.0,
                260.0,
                96.0,
                96.0,
                KeyShape::Stick,
                {
                    let mut s = style();
                    s.border_radius = 48.0;
                    s
                },
            );
            // silence unused mut warnings by using them
            ls.label = "LS".into();
            rs.label = "RS".into();
            vec![
                key("PadLB", "LB", 40.0, 40.0, 72.0, 40.0),
                key("PadLT", "LT", 40.0, 90.0, 72.0, 40.0),
                key("PadRB", "RB", 400.0, 40.0, 72.0, 40.0),
                key("PadRT", "RT", 400.0, 90.0, 72.0, 40.0),
                ls,
                rs,
                circle("PadY", "Y", 440.0, 160.0, 56.0, Some("rgba(234,179,8,0.75)")),
                circle("PadX", "X", 390.0, 210.0, 56.0, Some("rgba(59,130,246,0.75)")),
                circle("PadB", "B", 490.0, 210.0, 56.0, Some("rgba(239,68,68,0.75)")),
                circle("PadA", "A", 440.0, 260.0, 56.0, Some("rgba(34,197,94,0.75)")),
                key("PadUp", "↑", 200.0, 160.0, 44.0, 44.0),
                key("PadLeft", "←", 156.0, 204.0, 44.0, 44.0),
                key("PadDown", "↓", 200.0, 248.0, 44.0, 44.0),
                key("PadRight", "→", 244.0, 204.0, 44.0, 44.0),
                key("PadBack", "Back", 220.0, 120.0, 56.0, 32.0),
                key("PadStart", "Start", 286.0, 120.0, 56.0, 32.0),
            ]
        }
        LayoutTemplateId::SixtyPercent | LayoutTemplateId::Tkl | LayoutTemplateId::Full100 => {
            // Compact QWERTY row cluster (lighter than a full mechanical dump).
            let mut keys = Vec::new();
            let row1 = ["KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP"];
            let lab1 = ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"];
            for (i, (c, l)) in row1.iter().zip(lab1).enumerate() {
                keys.push(key(c, l, 40.0 + i as f32 * 58.0, 80.0, 52.0, 52.0));
            }
            let row2 = ["KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL"];
            let lab2 = ["A", "S", "D", "F", "G", "H", "J", "K", "L"];
            for (i, (c, l)) in row2.iter().zip(lab2).enumerate() {
                keys.push(key(c, l, 60.0 + i as f32 * 58.0, 140.0, 52.0, 52.0));
            }
            let row3 = ["KeyZ", "KeyX", "KeyC", "KeyV", "KeyB", "KeyN", "KeyM"];
            let lab3 = ["Z", "X", "C", "V", "B", "N", "M"];
            for (i, (c, l)) in row3.iter().zip(lab3).enumerate() {
                keys.push(key(c, l, 80.0 + i as f32 * 58.0, 200.0, 52.0, 52.0));
            }
            keys.push(key("Space", "Space", 160.0, 260.0, 220.0, 48.0));
            if matches!(id, LayoutTemplateId::Tkl | LayoutTemplateId::Full100) {
                keys.push(key("ArrowUp", "↑", 520.0, 200.0, 48.0, 48.0));
                keys.push(key("ArrowLeft", "←", 470.0, 250.0, 48.0, 48.0));
                keys.push(key("ArrowDown", "↓", 520.0, 250.0, 48.0, 48.0));
                keys.push(key("ArrowRight", "→", 570.0, 250.0, 48.0, 48.0));
            }
            if matches!(id, LayoutTemplateId::Full100) {
                for i in 0..10 {
                    keys.push(key(
                        &format!("Numpad{i}"),
                        &format!("{i}"),
                        650.0 + (i % 3) as f32 * 52.0,
                        80.0 + (i / 3) as f32 * 52.0,
                        48.0,
                        48.0,
                    ));
                }
            }
            keys
        }
        LayoutTemplateId::Custom => vec![],
    }
}

pub fn template_name(id: LayoutTemplateId) -> &'static str {
    all_templates()
        .into_iter()
        .find(|t| t.id == id)
        .map(|t| t.name)
        .unwrap_or("Custom")
}

pub fn create_profile_from_template(id: LayoutTemplateId, name: Option<String>) -> ProfileConfig {
    let now = chrono::Utc::now().to_rfc3339();
    let keys = template_keys(id);
    ProfileConfig {
        version: PROFILE_SCHEMA_VERSION,
        id: format!("profile-{}", uuid::Uuid::new_v4()),
        name: name.unwrap_or_else(|| template_name(id).into()),
        template_id: id,
        keys,
        global_theme: VisualTheme::Cyberpunk,
        window_opacity: 1.0,
        snap_to_grid: false,
        grid_size: DEFAULT_GRID_SIZE,
        target_app_enabled: false,
        target_app_match: String::new(),
        created_at: now.clone(),
        updated_at: now,
    }
}

pub fn create_default_library() -> LayoutLibrary {
    let profile = create_profile_from_template(LayoutTemplateId::WasdGaming, None);
    LayoutLibrary {
        version: LIBRARY_SCHEMA_VERSION,
        active_id: profile.id.clone(),
        profiles: vec![profile],
    }
}

pub fn theme_style(theme: VisualTheme) -> KeyStyle {
    let mut s = style();
    match theme {
        VisualTheme::Cyberpunk => {
            s.background_color = "rgba(10, 10, 30, 0.8)".into();
            s.border_color = "rgba(255, 0, 128, 0.8)".into();
            s.active_glow_color = "rgba(0, 255, 255, 0.9)".into();
            s.text_color = "#ff0080".into();
            s.press_effect = PressEffect::Glow;
        }
        VisualTheme::Glassmorphism => {
            s.background_color = "rgba(255, 255, 255, 0.1)".into();
            s.border_color = "rgba(255, 255, 255, 0.3)".into();
            s.active_glow_color = "rgba(255, 255, 255, 0.5)".into();
            s.text_color = "#ffffff".into();
            s.border_radius = 12.0;
            s.opacity = 0.7;
            s.press_effect = PressEffect::BorderRipple;
        }
        VisualTheme::RetroArcade => {
            s.background_color = "rgba(20, 0, 40, 0.85)".into();
            s.border_color = "rgba(255, 215, 0, 0.9)".into();
            s.active_glow_color = "rgba(255, 100, 0, 0.9)".into();
            s.text_color = "#ffd700".into();
            s.border_radius = 4.0;
            s.font_family = "Orbitron".into();
            s.press_effect = PressEffect::GlowPulse;
        }
        VisualTheme::StealthMinimal => {
            s.background_color = "rgba(12, 12, 14, 0.55)".into();
            s.border_color = "rgba(255, 255, 255, 0.14)".into();
            s.active_glow_color = "rgba(235, 235, 240, 0.75)".into();
            s.text_color = "#e6e6ea".into();
            s.border_radius = 6.0;
            s.opacity = 0.9;
            s.press_effect = PressEffect::KeyDrop;
        }
        VisualTheme::RgbWave => {
            s.background_color = "rgba(0, 0, 0, 0.6)".into();
            s.border_color = "rgba(255, 255, 255, 0.2)".into();
            s.active_glow_color = "rgba(168, 85, 247, 0.9)".into();
            s.text_color = "#ffffff".into();
            s.border_radius = 10.0;
            s.press_effect = PressEffect::GlowPulse;
        }
        VisualTheme::Custom => {}
    }
    s
}

pub fn theme_name(theme: VisualTheme) -> &'static str {
    match theme {
        VisualTheme::Cyberpunk => "Cyberpunk",
        VisualTheme::Glassmorphism => "Glassmorphism",
        VisualTheme::RetroArcade => "Retro Arcade",
        VisualTheme::StealthMinimal => "Stealth Minimal",
        VisualTheme::RgbWave => "RGB Wave",
        VisualTheme::Custom => "Custom",
    }
}
