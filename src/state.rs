//! Shared application state: library, undo, runtime presses, editor selection.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::input::{InputAction, InputMsg};
use crate::model::{
    AlignEdge, DrawerTab, KeyConfig, KeyShape, KeyStyle, LayoutLibrary, LayoutTemplateId,
    PressEffect, ProfileConfig, VisualTheme, LIBRARY_SCHEMA_VERSION,
};
use crate::persist::{self, save_library};
use crate::platform;
use crate::templates::{create_key, create_profile_from_template, theme_style};

const MAX_HISTORY: usize = 50;

pub struct AppState {
    pub library: LayoutLibrary,
    pub past: Vec<ProfileConfig>,
    pub future: Vec<ProfileConfig>,
    pub active_keys: HashSet<String>,
    pub press_counts: HashMap<String, u32>,
    pub stick_axes: HashMap<String, (f32, f32)>,
    pub selected: HashSet<String>,
    pub capturing: Option<String>,
    pub suppress_shortcuts_until: Instant,
    pub drawer: Option<DrawerTab>,
    pub style_scope_all: bool,
    pub kps_history: Vec<Instant>,
    pub kps: usize,
    pub dirty: bool,
    pub last_save: Instant,
    pub status: Option<(String, Instant)>,
    pub overlay_open: bool,
    pub editor_request_focus: bool,
    pub canvas_size: egui::Vec2,
    /// Drag/resize session for editor canvas.
    pub drag: Option<DragSession>,
}

pub enum DragSession {
    Move {
        start: egui::Pos2,
        origins: HashMap<String, egui::Pos2>,
    },
    Resize {
        id: String,
        handle: ResizeHandle,
        start: egui::Pos2,
        origin: egui::Rect,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ResizeHandle {
    Nw,
    Ne,
    Sw,
    Se,
}

impl AppState {
    pub fn load() -> Self {
        let library = persist::load_library();
        Self {
            library,
            past: Vec::new(),
            future: Vec::new(),
            active_keys: HashSet::new(),
            press_counts: HashMap::new(),
            stick_axes: HashMap::new(),
            selected: HashSet::new(),
            capturing: None,
            suppress_shortcuts_until: Instant::now(),
            drawer: Some(DrawerTab::Layouts),
            style_scope_all: true,
            kps_history: Vec::new(),
            kps: 0,
            dirty: false,
            last_save: Instant::now(),
            status: None,
            overlay_open: false,
            editor_request_focus: false,
            canvas_size: egui::vec2(800.0, 560.0),
            drag: None,
        }
    }

    pub fn set_library(&mut self, library: LayoutLibrary) {
        self.library = library;
        if !self.library.profiles.iter().any(|p| p.id == self.library.active_id) {
            if let Some(first) = self.library.profiles.first() {
                self.library.active_id = first.id.clone();
            }
        }
    }

    pub fn profile(&self) -> &ProfileConfig {
        self.library
            .profiles
            .iter()
            .find(|p| p.id == self.library.active_id)
            .or_else(|| self.library.profiles.first())
            .expect("library always has a profile")
    }

    pub fn profile_mut(&mut self) -> &mut ProfileConfig {
        let id = self.library.active_id.clone();
        let idx = self
            .library
            .profiles
            .iter()
            .position(|p| p.id == id)
            .unwrap_or(0);
        &mut self.library.profiles[idx]
    }

    fn touch(&mut self) {
        let now = chrono::Utc::now().to_rfc3339();
        self.profile_mut().updated_at = now;
        self.dirty = true;
    }

    pub fn push_history(&mut self) {
        let snap = self.profile().clone();
        self.past.push(snap);
        if self.past.len() > MAX_HISTORY {
            self.past.remove(0);
        }
        self.future.clear();
    }

    pub fn touch_profile(&mut self) {
        self.touch();
    }

    pub fn undo(&mut self) {
        let Some(prev) = self.past.pop() else { return };
        let current = self.profile().clone();
        self.future.insert(0, current);
        if self.future.len() > MAX_HISTORY {
            self.future.pop();
        }
        self.replace_active_profile(prev);
        self.selected.clear();
        self.capturing = None;
    }

    pub fn redo(&mut self) {
        if self.future.is_empty() {
            return;
        }
        let next = self.future.remove(0);
        let current = self.profile().clone();
        self.past.push(current);
        self.replace_active_profile(next);
        self.selected.clear();
        self.capturing = None;
    }

    fn replace_active_profile(&mut self, profile: ProfileConfig) {
        let id = profile.id.clone();
        if let Some(slot) = self.library.profiles.iter_mut().find(|p| p.id == id) {
            *slot = profile;
        } else {
            self.library.profiles.push(profile);
        }
        self.library.active_id = id;
        self.dirty = true;
    }

    pub fn flush_save(&mut self) {
        if !self.dirty {
            return;
        }
        self.library.version = LIBRARY_SCHEMA_VERSION;
        if let Err(err) = save_library(&self.library) {
            self.flash(format!("save failed: {err}"));
        } else {
            self.dirty = false;
            self.last_save = Instant::now();
        }
    }

    pub fn autosave_tick(&mut self) {
        if self.dirty && self.last_save.elapsed() > Duration::from_millis(400) {
            self.flush_save();
        }
    }

    pub fn flash(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), Instant::now()));
    }

    pub fn handle_input(&mut self, msg: InputMsg) {
        match msg {
            InputMsg::StickAxes { code, x, y } => {
                self.stick_axes.insert(code, (x, y));
            }
            InputMsg::Key {
                code,
                label,
                action,
                ..
            } => {
                if action == InputAction::Down {
                    if let Some(id) = self.capturing.clone() {
                        if self.bind_suppressed() {
                            return;
                        }
                        self.push_history();
                        if let Some(key) = self.profile_mut().keys.iter_mut().find(|k| k.id == id) {
                            let keep = !key.label.is_empty()
                                && key.label != "New Key"
                                && key.label != key.code
                                && key.label != "…";
                            key.code = code;
                            if !keep {
                                key.label = if label == " " {
                                    "Space".into()
                                } else {
                                    label
                                };
                            }
                        }
                        self.capturing = None;
                        self.suppress_shortcuts_until =
                            Instant::now() + Duration::from_millis(400);
                        self.touch();
                        return;
                    }
                    if self.active_keys.contains(&code) {
                        return;
                    }
                    self.active_keys.insert(code.clone());
                    *self.press_counts.entry(code).or_insert(0) += 1;
                    let now = Instant::now();
                    self.kps_history.retain(|t| now.duration_since(*t) < Duration::from_secs(1));
                    self.kps_history.push(now);
                    self.kps = self.kps_history.len();
                } else {
                    self.active_keys.remove(&code);
                }
            }
        }
    }

    pub fn tick_kps(&mut self) {
        let now = Instant::now();
        let before = self.kps_history.len();
        self.kps_history.retain(|t| now.duration_since(*t) < Duration::from_secs(1));
        if self.kps_history.len() != before {
            self.kps = self.kps_history.len();
        }
    }

    pub fn create_layout(&mut self) {
        self.flush_into_library_list();
        let n = self.library.profiles.len() + 1;
        let fresh = create_profile_from_template(LayoutTemplateId::Custom, Some(format!("Layout {n}")));
        self.library.active_id = fresh.id.clone();
        self.library.profiles.push(fresh);
        self.past.clear();
        self.future.clear();
        self.selected.clear();
        self.dirty = true;
    }

    pub fn duplicate_layout(&mut self, id: Option<&str>) {
        self.flush_into_library_list();
        let id = id.unwrap_or(&self.library.active_id).to_string();
        let Some(source) = self.library.profiles.iter().find(|p| p.id == id).cloned() else {
            return;
        };
        let mut copy = source;
        copy.id = format!("profile-{}", uuid::Uuid::new_v4());
        copy.name = format!("{} copy", copy.name);
        copy.template_id = LayoutTemplateId::Custom;
        let now = chrono::Utc::now().to_rfc3339();
        copy.created_at = now.clone();
        copy.updated_at = now;
        for k in &mut copy.keys {
            k.id = format!("{}-{}", k.id, uuid::Uuid::new_v4());
        }
        self.library.active_id = copy.id.clone();
        self.library.profiles.push(copy);
        self.past.clear();
        self.future.clear();
        self.selected.clear();
        self.dirty = true;
    }

    pub fn delete_layout(&mut self, id: &str) {
        if self.library.profiles.len() <= 1 {
            return;
        }
        self.flush_into_library_list();
        self.library.profiles.retain(|p| p.id != id);
        if self.library.active_id == id {
            self.library.active_id = self.library.profiles[0].id.clone();
        }
        self.past.clear();
        self.future.clear();
        self.selected.clear();
        self.dirty = true;
    }

    pub fn switch_layout(&mut self, id: &str) {
        if id == self.library.active_id {
            return;
        }
        self.flush_into_library_list();
        if self.library.profiles.iter().any(|p| p.id == id) {
            self.library.active_id = id.into();
            self.past.clear();
            self.future.clear();
            self.selected.clear();
            self.active_keys.clear();
            self.dirty = true;
        }
    }

    pub fn rename_layout(&mut self, id: &str, name: String) {
        let name = name.trim().to_string();
        if name.is_empty() {
            return;
        }
        if let Some(p) = self.library.profiles.iter_mut().find(|p| p.id == id) {
            p.name = name;
            p.updated_at = chrono::Utc::now().to_rfc3339();
            self.dirty = true;
        }
    }

    fn flush_into_library_list(&mut self) {
        // profile is already the entry in library.profiles
        self.library.version = LIBRARY_SCHEMA_VERSION;
    }

    pub fn load_template(&mut self, id: LayoutTemplateId) {
        self.push_history();
        let keep_id = self.profile().id.clone();
        let keep_created = self.profile().created_at.clone();
        let show_kps = self.profile().show_kps_meter;
        let opacity = self.profile().window_opacity;
        let snap = self.profile().snap_to_grid;
        let grid = self.profile().grid_size;
        let filter_on = self.profile().target_app_enabled;
        let filter = self.profile().target_app_match.clone();
        let theme = self.profile().global_theme;
        let mut next = create_profile_from_template(id, None);
        next.id = keep_id;
        next.created_at = keep_created;
        next.show_kps_meter = show_kps;
        next.window_opacity = opacity;
        next.snap_to_grid = snap;
        next.grid_size = grid;
        next.target_app_enabled = filter_on;
        next.target_app_match = filter;
        next.global_theme = theme;
        *self.profile_mut() = next;
        self.selected.clear();
        self.touch();
    }

    pub fn apply_theme(&mut self, theme: VisualTheme) {
        self.push_history();
        let style = theme_style(theme);
        for k in &mut self.profile_mut().keys {
            k.style.background_color = style.background_color.clone();
            k.style.border_color = style.border_color.clone();
            k.style.active_glow_color = style.active_glow_color.clone();
            k.style.text_color = style.text_color.clone();
            k.style.border_radius = style.border_radius;
            k.style.opacity = style.opacity;
            k.style.press_effect = style.press_effect;
            if !style.font_family.is_empty() {
                k.style.font_family = style.font_family.clone();
            }
        }
        self.profile_mut().global_theme = theme;
        self.touch();
    }

    pub fn add_key(&mut self) {
        self.push_history();
        let (cw, ch) = (self.canvas_size.x, self.canvas_size.y);
        let key = create_key(
            "KeyA",
            "New Key",
            (cw / 2.0 - 28.0).max(24.0),
            (ch / 2.0 - 28.0).max(24.0),
            56.0,
            56.0,
            KeyShape::Rectangle,
            KeyStyle::default(),
        );
        let id = key.id.clone();
        self.profile_mut().keys.push(key);
        self.selected = HashSet::from([id.clone()]);
        self.capturing = Some(id);
        self.suppress_shortcuts_until = Instant::now() + Duration::from_millis(300);
        self.touch();
    }

    pub fn add_mouse_zone(&mut self, which: &str) {
        self.push_history();
        let (code, label) = match which {
            "right" => ("Mouseright", "RMB"),
            "middle" => ("Mousemiddle", "MMB"),
            _ => ("Mouseleft", "LMB"),
        };
        let (cw, ch) = (self.canvas_size.x, self.canvas_size.y);
        let key = create_key(
            code,
            label,
            (cw / 2.0 - 36.0).max(24.0),
            (ch / 2.0 - 36.0).max(24.0),
            72.0,
            72.0,
            KeyShape::Circle,
            KeyStyle::default(),
        );
        let id = key.id.clone();
        self.profile_mut().keys.push(key);
        self.selected = HashSet::from([id]);
        self.touch();
    }

    pub fn add_joystick(&mut self, stick: &str) {
        self.push_history();
        let (code, label) = if stick == "PadRS" {
            ("PadRS", "RS")
        } else {
            ("PadLS", "LS")
        };
        let (cw, ch) = (self.canvas_size.x, self.canvas_size.y);
        let mut style = KeyStyle::default();
        style.border_radius = 44.0;
        style.show_press_count = false;
        style.show_duration = false;
        let key = create_key(
            code,
            label,
            (cw / 2.0 - 40.0).max(24.0),
            (ch / 2.0 - 40.0).max(24.0),
            88.0,
            88.0,
            KeyShape::Stick,
            style,
        );
        let id = key.id.clone();
        self.profile_mut().keys.push(key);
        self.selected = HashSet::from([id]);
        self.touch();
    }

    pub fn add_controller_pad(&mut self) {
        self.push_history();
        let pad = create_profile_from_template(LayoutTemplateId::Controller, None);
        let mut ids = HashSet::new();
        for mut k in pad.keys {
            k.x += 40.0;
            k.y += 40.0;
            k.id = format!("{}-{}", k.id, uuid::Uuid::new_v4());
            ids.insert(k.id.clone());
            self.profile_mut().keys.push(k);
        }
        self.selected = ids;
        self.touch();
    }

    pub fn remove_selected(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        self.push_history();
        let sel = self.selected.clone();
        self.profile_mut().keys.retain(|k| !sel.contains(&k.id));
        self.selected.clear();
        self.capturing = None;
        self.touch();
    }

    pub fn nudge_selected(&mut self, dx: f32, dy: f32) {
        if self.selected.is_empty() || (dx == 0.0 && dy == 0.0) {
            return;
        }
        self.push_history();
        let sel = self.selected.clone();
        let cw = self.canvas_size.x;
        let ch = self.canvas_size.y;
        for k in self.profile_mut().keys.iter_mut() {
            if !sel.contains(&k.id) {
                continue;
            }
            let w = k.width * k.scale;
            let h = k.height * k.scale;
            k.x = (k.x + dx).clamp(0.0, (cw - w).max(0.0)).round();
            k.y = (k.y + dy).clamp(0.0, (ch - h).max(0.0)).round();
        }
        self.touch();
    }

    pub fn align_selected(&mut self, edge: AlignEdge) {
        let selected: Vec<KeyConfig> = self
            .profile()
            .keys
            .iter()
            .filter(|k| self.selected.contains(&k.id))
            .cloned()
            .collect();
        if selected.len() < 2 {
            return;
        }
        self.push_history();
        let left = selected.iter().map(|k| k.x).fold(f32::INFINITY, f32::min);
        let right = selected
            .iter()
            .map(|k| k.x + k.width)
            .fold(f32::NEG_INFINITY, f32::max);
        let top = selected.iter().map(|k| k.y).fold(f32::INFINITY, f32::min);
        let bottom = selected
            .iter()
            .map(|k| k.y + k.height)
            .fold(f32::NEG_INFINITY, f32::max);
        let sel = self.selected.clone();
        for k in self.profile_mut().keys.iter_mut() {
            if !sel.contains(&k.id) {
                continue;
            }
            match edge {
                AlignEdge::Left => k.x = left,
                AlignEdge::Right => k.x = right - k.width,
                AlignEdge::Center => k.x = ((left + right) / 2.0 - k.width / 2.0).round(),
                AlignEdge::Top => k.y = top,
                AlignEdge::Bottom => k.y = bottom - k.height,
                AlignEdge::Middle => k.y = ((top + bottom) / 2.0 - k.height / 2.0).round(),
            }
        }
        self.touch();
    }

    pub fn update_selected_style(&mut self, f: impl FnMut(&mut KeyStyle)) {
        let ids: HashSet<String> = if self.style_scope_all || self.selected.is_empty() {
            self.profile().keys.iter().map(|k| k.id.clone()).collect()
        } else {
            self.selected.clone()
        };
        // Style sliders spam — don't push history every frame; caller can decide.
        let mut f = f;
        for k in self.profile_mut().keys.iter_mut() {
            if ids.contains(&k.id) {
                f(&mut k.style);
            }
        }
        self.profile_mut().global_theme = VisualTheme::Custom;
        self.touch();
    }

    pub fn place_overlay(&mut self) {
        self.flush_save();
        platform::set_overlay_live(true);
        platform::set_manual_visible(true);
        platform::set_positioning(true);
        platform::set_filter(
            self.profile().target_app_enabled,
            self.profile().target_app_match.clone(),
        );

        // Secondary egui viewports can't be transparent on Windows (black window).
        // Spawn the HUD as its own process so it owns a transparent root window.
        match std::env::current_exe() {
            Ok(exe) => match std::process::Command::new(exe).arg("--overlay").spawn() {
                Ok(_) => {
                    self.overlay_open = true;
                    self.flash("Overlay launched — drag it, then Ctrl+Shift+L to lock");
                }
                Err(err) => self.flash(format!("could not launch overlay: {err}")),
            },
            Err(err) => self.flash(format!("could not find executable: {err}")),
        }
    }

    pub fn shortcuts_blocked(&self) -> bool {
        self.capturing.is_some() || self.bind_suppressed()
    }

    /// True briefly after starting a rebind so the opening click isn't assigned.
    pub fn bind_suppressed(&self) -> bool {
        Instant::now() < self.suppress_shortcuts_until
    }

    pub fn effect_label(effect: PressEffect) -> &'static str {
        match effect {
            PressEffect::Glow => "Glow",
            PressEffect::GlowPulse => "Glow Pulse",
            PressEffect::KeyDrop => "Key Drop",
            PressEffect::BorderRipple => "Border Ripple",
            PressEffect::None => "None",
        }
    }
}
