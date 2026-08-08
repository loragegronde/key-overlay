//! Transparent HUD — runs as its own root window (`key-overlay --overlay`).
//!
//! Secondary egui viewports stay black on Windows even with glow; a separate
//! process with a transparent root viewport is the reliable approach.
//!
//! The window is sized to the layout bounding box (plus padding) so you can
//! place a tight HUD anywhere on screen.
//!
//! Lock / visibility are driven by `hud-control.json` (written by editor global
//! hotkeys) so Ctrl+Shift+L works even when this window is click-through.

use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant, SystemTime};

use eframe::egui::{self, Align2, CentralPanel, Color32, FontId, Frame, Pos2, Sense, Vec2};

use crate::hud_control::{self, HudControl};
use crate::input::{apply_egui_presses, start_listener, InputMsg};
use crate::model::{KeyConfig, HOTKEY_TOGGLE_LOCK};
use crate::persist::{self, load_library};
use crate::platform;
use crate::state::AppState;
use crate::ui::widgets::{key_rect, paint_key};

const PAD: f32 = 12.0;
/// Reserved bottom strip for the drag hint (kept even when locked so size is stable).
const BOTTOM_PAD: f32 = 28.0;
const MIN_W: f32 = 80.0;
const MIN_H: f32 = 48.0;

pub struct OverlayApp {
    state: AppState,
    input_rx: Receiver<InputMsg>,
    last_reload: Instant,
    last_control_poll: Instant,
    layout_mtime: Option<SystemTime>,
    control_mtime: Option<SystemTime>,
    control: HudControl,
    last_native_flags: Instant,
    flags_dirty: bool,
    last_show: Option<bool>,
    last_passthrough: Option<bool>,
    /// Top-left of content in layout coordinates (keys drawn relative to this).
    content_origin: Pos2,
    last_size: Vec2,
}

impl OverlayApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        platform::start_watcher();
        platform::set_overlay_live(true);

        let control = hud_control::load();
        apply_control_to_platform(&control);

        let state = AppState::load();
        platform::set_filter(
            state.profile().target_app_enabled,
            state.profile().target_app_match.clone(),
        );
        platform::recompute();

        let (size, origin) = content_metrics(&state.profile().keys);

        Self {
            state,
            input_rx: start_listener(),
            last_reload: Instant::now(),
            last_control_poll: Instant::now(),
            layout_mtime: persist::layout_mtime(),
            control_mtime: hud_control::mtime(),
            control,
            last_native_flags: Instant::now(),
            flags_dirty: true,
            last_show: None,
            last_passthrough: None,
            content_origin: origin,
            last_size: size,
        }
    }

    fn sync_control(&mut self) {
        let mtime = hud_control::mtime();
        // Reload on mtime change, and poll often enough that same-second toggles
        // still apply via `rev` while the HUD is click-through.
        let due = mtime != self.control_mtime
            || self.last_control_poll.elapsed() > Duration::from_millis(50);
        if !due {
            return;
        }
        self.last_control_poll = Instant::now();
        self.control_mtime = mtime;
        let next = hud_control::load();
        if next.rev != self.control.rev
            || next.locked != self.control.locked
            || next.visible != self.control.visible
            || next.suppress_input != self.control.suppress_input
        {
            self.control = next;
            apply_control_to_platform(&self.control);
            self.flags_dirty = true;
        }
    }

    fn sync_layout(&mut self) {
        if self.last_reload.elapsed() < Duration::from_millis(400) {
            return;
        }
        self.last_reload = Instant::now();
        let mtime = persist::layout_mtime();
        if mtime == self.layout_mtime {
            return;
        }
        self.layout_mtime = mtime;
        let library = load_library();
        self.state.set_library(library);
        platform::set_filter(
            self.state.profile().target_app_enabled,
            self.state.profile().target_app_match.clone(),
        );
    }

    fn sync_window_size(&mut self, ctx: &egui::Context) {
        let (size, origin) = content_metrics(&self.state.profile().keys);
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
            if self.control.suppress_input {
                // Editor is typing in a text field — don't flash HUD keys.
                if let InputMsg::StickAxes { .. } = msg {
                    self.state.handle_input(msg);
                }
            } else {
                self.state.handle_input(msg);
            }
        }
        if self.control.suppress_input {
            self.state.active_keys.clear();
        } else if !self.control.locked {
            // When the HUD still has focus (unlocked), light keys from egui too.
            apply_egui_presses(ctx, &mut self.state);
        }

        // Local fallback when this window still has focus (global hotkeys also
        // write hud-control.json from the editor process).
        ctx.input(|i| {
            let chord = i.modifiers.ctrl && i.modifiers.shift;
            if chord && i.key_pressed(egui::Key::L) {
                hud_control::toggle_lock();
            }
            if chord && i.key_pressed(egui::Key::O) {
                hud_control::toggle_visible();
            }
        });

        self.sync_control();
        self.sync_layout();
        self.sync_window_size(ctx);

        // Unlocked (positioning): always show so you can place the HUD.
        // Locked: honor target-app filter + manual visibility.
        let positioning = !self.control.locked;
        let filtered =
            platform::SHOULD_SHOW_OVERLAY.load(std::sync::atomic::Ordering::SeqCst);
        let show = self.control.visible && (positioning || filtered);

        if self.last_show != Some(show) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(show));
            self.last_show = Some(show);
        }

        let passthrough = self.control.locked;
        if self.last_passthrough != Some(passthrough) {
            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(passthrough));
            self.last_passthrough = Some(passthrough);
            self.flags_dirty = true;
        }

        if self.flags_dirty || self.last_native_flags.elapsed() > Duration::from_secs(2) {
            platform::apply_native_window_flags("Key Overlay HUD");
            self.last_native_flags = Instant::now();
            self.flags_dirty = false;
        }

        let opacity = self.state.profile().window_opacity;
        let origin = self.content_origin;

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
                        format!("Drag to move · {HOTKEY_TOGGLE_LOCK} lock/unlock"),
                        FontId::proportional(11.0),
                        Color32::from_rgba_unmultiplied(255, 255, 255, 200),
                    );
                }

                for key in &self.state.profile().keys {
                    let mut k = key.clone();
                    k.x = key.x - origin.x + full.min.x;
                    k.y = key.y - origin.y + full.min.y;
                    paint_key(ui, &self.state, &k, false);
                }
            });

        if ctx.input(|i| i.viewport().close_requested()) {
            std::process::exit(0);
        }

        // Repaint quickly while keys are held; idle a bit slower to cut CPU.
        let busy = !self.state.active_keys.is_empty() || positioning;
        ctx.request_repaint_after(Duration::from_millis(if busy { 8 } else { 33 }));
    }
}

fn apply_control_to_platform(control: &HudControl) {
    platform::set_manual_visible(control.visible);
    if control.locked {
        platform::finish_positioning();
    } else {
        platform::set_positioning(true);
        platform::set_click_through(false);
    }
    platform::recompute();
}

/// Returns `(window_size, content_origin)` where keys are drawn at
/// `key.pos - content_origin + window_min`.
fn content_metrics(keys: &[KeyConfig]) -> (Vec2, Pos2) {
    if keys.is_empty() {
        return (Vec2::new(220.0, 100.0 + BOTTOM_PAD), Pos2::ZERO);
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
    let h = (max_y - min_y + PAD * 2.0 + BOTTOM_PAD).max(MIN_H + BOTTOM_PAD);
    (Vec2::new(w, h), origin)
}

pub fn run_overlay() -> eframe::Result<()> {
    // Ensure control file exists so editor hotkeys have a target immediately.
    if hud_control::mtime().is_none() {
        hud_control::reset_for_place();
    }

    let library = load_library();
    let keys = library
        .profiles
        .iter()
        .find(|p| p.id == library.active_id)
        .map(|p| p.keys.as_slice())
        .unwrap_or(&[]);
    let (size, _) = content_metrics(keys);

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
