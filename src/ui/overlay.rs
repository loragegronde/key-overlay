//! Transparent HUD — runs as its own root window (`key-overlay --overlay`).
//!
//! Secondary egui viewports stay black on Windows even with glow; a separate
//! process with a transparent root viewport is the reliable approach.
//!
//! The window is sized to the layout bounding box (plus padding) so you can
//! place a tight HUD anywhere on screen.

use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, CentralPanel, Color32, FontId, Frame, Pos2, Sense, Vec2};

use crate::input::{start_listener, InputMsg};
use crate::model::{KeyConfig, HOTKEY_TOGGLE_LOCK};
use crate::persist::load_library;
use crate::platform;
use crate::state::AppState;
use crate::ui::widgets::{key_rect, paint_key};

const PAD: f32 = 12.0;
const HINT_H: f32 = 28.0;
const MIN_W: f32 = 80.0;
const MIN_H: f32 = 48.0;

pub struct OverlayApp {
    state: AppState,
    input_rx: Receiver<InputMsg>,
    last_reload: Instant,
    last_native_flags: Instant,
    positioning: bool,
    /// Top-left of content in layout coordinates (keys drawn relative to this).
    content_origin: Pos2,
    last_size: Vec2,
}

impl OverlayApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        platform::start_watcher();
        platform::set_overlay_live(true);
        platform::set_manual_visible(true);
        platform::set_positioning(true);
        platform::set_click_through(false);

        let state = AppState::load();
        platform::set_filter(
            state.profile().target_app_enabled,
            state.profile().target_app_match.clone(),
        );
        platform::recompute();

        let (size, origin) = content_metrics(&state.profile().keys, true);

        Self {
            state,
            input_rx: start_listener(),
            last_reload: Instant::now(),
            last_native_flags: Instant::now(),
            positioning: true,
            content_origin: origin,
            last_size: size,
        }
    }

    fn sync_window_size(&mut self, ctx: &egui::Context) {
        let (size, origin) = content_metrics(&self.state.profile().keys, self.positioning);
        self.content_origin = origin;
        if (size.x - self.last_size.x).abs() > 0.5 || (size.y - self.last_size.y).abs() > 0.5 {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            self.last_size = size;
        }
    }
}

impl eframe::App for OverlayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.input_rx.try_recv() {
            self.state.handle_input(msg);
        }

        // Hotkeys while the HUD is focused.
        ctx.input(|i| {
            let chord = i.modifiers.ctrl && i.modifiers.shift;
            if chord && i.key_pressed(egui::Key::L) {
                self.positioning = false;
                platform::finish_positioning();
            }
            if chord && i.key_pressed(egui::Key::O) {
                platform::toggle_manual_visible();
            }
        });

        // Pick up editor saves.
        if self.last_reload.elapsed() > Duration::from_millis(500) {
            let library = load_library();
            self.state.set_library(library);
            platform::set_filter(
                self.state.profile().target_app_enabled,
                self.state.profile().target_app_match.clone(),
            );
            self.last_reload = Instant::now();
        }

        self.sync_window_size(ctx);

        let show = platform::SHOULD_SHOW_OVERLAY.load(std::sync::atomic::Ordering::SeqCst)
            || self.positioning;
        if !show {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(
            !self.positioning && platform::is_click_through(),
        ));

        if self.last_native_flags.elapsed() > Duration::from_millis(200) {
            platform::apply_native_window_flags("Key Overlay HUD");
            self.last_native_flags = Instant::now();
        }

        let opacity = self.state.profile().window_opacity;
        let origin = self.content_origin;
        let positioning = self.positioning;

        CentralPanel::default()
            .frame(Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.set_opacity(opacity);
                let full = ui.max_rect();

                if positioning {
                    let drag = ui.interact(full, ui.id().with("overlay-drag"), Sense::drag());
                    if drag.dragged() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    ui.painter().text(
                        full.center_bottom() - Vec2::new(0.0, 8.0),
                        Align2::CENTER_CENTER,
                        format!("Drag to move · {HOTKEY_TOGGLE_LOCK} to lock"),
                        FontId::proportional(11.0),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 200),
                    );
                }

                for key in &self.state.profile().keys.clone() {
                    let mut k = key.clone();
                    k.x = key.x - origin.x + full.min.x;
                    k.y = key.y - origin.y + full.min.y;
                    paint_key(ui, &self.state, &k, false);
                }
            });

        if ctx.input(|i| i.viewport().close_requested()) {
            std::process::exit(0);
        }

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

/// Returns `(window_size, content_origin)` where keys are drawn at
/// `key.pos - content_origin + window_min`.
fn content_metrics(keys: &[KeyConfig], positioning: bool) -> (Vec2, Pos2) {
    let hint = if positioning { HINT_H } else { 0.0 };
    if keys.is_empty() {
        return (Vec2::new(220.0, 100.0 + hint), Pos2::ZERO);
    }

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for key in keys {
        let r = key_rect(key);
        min_x = min_x.min(r.min.x);
        min_y = min_y.min(r.min.y);
        max_x = max_x.max(r.max.x);
        max_y = max_y.max(r.max.y);
    }

    let origin = Pos2::new(min_x - PAD, min_y - PAD);
    let w = (max_x - min_x + PAD * 2.0).max(MIN_W);
    let h = (max_y - min_y + PAD * 2.0 + hint).max(MIN_H + hint);
    (Vec2::new(w, h), origin)
}

pub fn run_overlay() -> eframe::Result<()> {
    let library = load_library();
    let keys = library
        .profiles
        .iter()
        .find(|p| p.id == library.active_id)
        .map(|p| p.keys.as_slice())
        .unwrap_or(&[]);
    let (size, _) = content_metrics(keys, true);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Key Overlay HUD")
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_taskbar(false)
            .with_inner_size([size.x, size.y])
            .with_resizable(false),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Key Overlay HUD",
        options,
        Box::new(|cc| Ok(Box::new(OverlayApp::new(cc)))),
    )
}
