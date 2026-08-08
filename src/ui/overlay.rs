//! Transparent HUD viewport.

use egui::{Align2, CentralPanel, Color32, Context, FontId, Frame, Sense, Vec2};

use crate::model::HOTKEY_TOGGLE_LOCK;
use crate::platform;
use crate::state::AppState;
use crate::ui::widgets::paint_key;

pub const OVERLAY_VIEWPORT: &str = "overlay_hud";

pub fn show_overlay(ctx: &Context, state: &mut AppState) {
    let show = platform::SHOULD_SHOW_OVERLAY.load(std::sync::atomic::Ordering::SeqCst)
        || platform::is_positioning();

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Key Overlay HUD")
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_taskbar(false)
        .with_inner_size([720.0, 420.0])
        .with_visible(show);

    if platform::is_positioning() {
        viewport = viewport.with_mouse_passthrough(false);
    } else {
        viewport = viewport.with_mouse_passthrough(true);
    }

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of(OVERLAY_VIEWPORT),
        viewport,
        |ctx, _class| {
            // Keep native flags in sync on Windows.
            platform::apply_native_window_flags("Key Overlay HUD");

            let opacity = state.profile().window_opacity;
            CentralPanel::default()
                .frame(Frame::none().fill(Color32::TRANSPARENT))
                .show(ctx, |ui| {
                    ui.set_opacity(opacity);
                    let full = ui.max_rect();

                    if platform::is_positioning() {
                        let drag = ui.interact(full, ui.id().with("overlay-drag"), Sense::drag());
                        if drag.dragged() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        ui.painter().text(
                            full.center_bottom() - Vec2::new(0.0, 18.0),
                            Align2::CENTER_CENTER,
                            format!("Drag anywhere to move · {HOTKEY_TOGGLE_LOCK} to lock"),
                            FontId::proportional(12.0),
                            Color32::from_rgba_unmultiplied(255, 255, 255, 180),
                        );
                    }

                    if state.profile().show_kps_meter {
                        ui.painter().text(
                            full.right_top() + Vec2::new(-12.0, 12.0),
                            Align2::RIGHT_TOP,
                            format!("{} KPS", state.kps),
                            FontId::monospace(16.0),
                            Color32::from_rgb(34, 211, 238),
                        );
                    }

                    for key in &state.profile().keys.clone() {
                        paint_key(ui, state, key, false);
                    }
                });

            // Close request on overlay should hide, not quit.
            if ctx.input(|i| i.viewport().close_requested()) {
                platform::set_manual_visible(false);
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        },
    );
}
