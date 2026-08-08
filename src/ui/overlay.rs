//! Transparent HUD — runs as its own root window (`key-overlay --overlay`).
//!
//! Secondary egui viewports stay black on Windows even with glow; a separate
//! process with a transparent root viewport is the reliable approach.

use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, CentralPanel, Color32, FontId, Frame, Sense, Vec2};

use crate::input::{start_listener, InputMsg};
use crate::model::HOTKEY_TOGGLE_LOCK;
use crate::persist::load_library;
use crate::platform;
use crate::state::AppState;
use crate::ui::widgets::paint_key;

pub struct OverlayApp {
    state: AppState,
    input_rx: Receiver<InputMsg>,
    last_reload: Instant,
    last_native_flags: Instant,
    positioning: bool,
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

        Self {
            state,
            input_rx: start_listener(),
            last_reload: Instant::now(),
            last_native_flags: Instant::now(),
            positioning: true,
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
        self.state.tick_kps();

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

        let show = platform::SHOULD_SHOW_OVERLAY.load(std::sync::atomic::Ordering::SeqCst)
            || self.positioning;
        if !show {
            // Keep process alive but hide.
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
        CentralPanel::default()
            .frame(Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.set_opacity(opacity);
                let full = ui.max_rect();

                if self.positioning {
                    let drag = ui.interact(full, ui.id().with("overlay-drag"), Sense::drag());
                    if drag.dragged() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    ui.painter().text(
                        full.center_bottom() - Vec2::new(0.0, 18.0),
                        Align2::CENTER_CENTER,
                        format!("Drag anywhere to move · {HOTKEY_TOGGLE_LOCK} to lock"),
                        FontId::proportional(12.0),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 200),
                    );
                }

                if self.state.profile().show_kps_meter {
                    ui.painter().text(
                        full.right_top() + Vec2::new(-12.0, 12.0),
                        Align2::RIGHT_TOP,
                        format!("{} KPS", self.state.kps),
                        FontId::monospace(16.0),
                        Color32::from_rgb(34, 211, 238),
                    );
                }

                for key in &self.state.profile().keys.clone() {
                    paint_key(ui, &self.state, key, false);
                }
            });

        if ctx.input(|i| i.viewport().close_requested()) {
            // Quit overlay process.
            std::process::exit(0);
        }

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

pub fn run_overlay() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Key Overlay HUD")
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_taskbar(false)
            .with_inner_size([720.0, 420.0]),
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
