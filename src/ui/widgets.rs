//! Shared key / stick painting for editor + overlay.

use egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};

use crate::color::{parse_color, pressed_fill};
use crate::model::{KeyConfig, KeyShape, PressEffect};
use crate::state::AppState;

pub fn key_rect(key: &KeyConfig) -> Rect {
    Rect::from_min_size(
        Pos2::new(key.x, key.y),
        Vec2::new(key.width * key.scale, key.height * key.scale),
    )
}

pub fn paint_key(
    ui: &mut Ui,
    state: &AppState,
    key: &KeyConfig,
    interactive: bool,
) -> egui::Response {
    let rect = key_rect(key);
    let sense = if interactive {
        Sense::click_and_drag()
    } else {
        Sense::hover()
    };
    let response = ui.allocate_rect(rect, sense);

    let active = state.active_keys.contains(&key.code);
    let selected = state.selected.contains(&key.id);
    let capturing = state.capturing.as_deref() == Some(key.id.as_str());

    let bg = if active {
        pressed_fill(&key.style.background_color, &key.style.active_glow_color, 0.55).to_egui()
    } else {
        let mut c = parse_color(&key.style.background_color);
        c.a *= key.style.opacity;
        c.to_egui()
    };
    let border = if active {
        parse_color(&key.style.active_glow_color).to_egui()
    } else {
        parse_color(&key.style.border_color).to_egui()
    };

    let rounding = match key.shape {
        KeyShape::Circle | KeyShape::Stick => Rounding::same(rect.width().min(rect.height()) * 0.5),
        KeyShape::Rectangle => Rounding::same(key.style.border_radius),
    };

    let painter = ui.painter();
    let mut draw_rect = rect;
    if active {
        match key.style.press_effect {
            PressEffect::KeyDrop => {
                draw_rect = draw_rect.translate(Vec2::new(0.0, 3.0));
                let shrink = draw_rect.width() * 0.06;
                draw_rect = draw_rect.shrink(shrink);
            }
            PressEffect::Glow | PressEffect::GlowPulse => {
                let glow = parse_color(&key.style.active_glow_color).to_egui();
                painter.rect(
                    draw_rect.expand(6.0),
                    rounding,
                    Color32::from_rgba_unmultiplied(glow.r(), glow.g(), glow.b(), 40),
                    Stroke::NONE,
                );
            }
            PressEffect::BorderRipple => {
                painter.rect_stroke(
                    draw_rect.expand(4.0),
                    rounding,
                    Stroke::new(2.0_f32, border.gamma_multiply(0.5)),
                );
            }
            PressEffect::None => {}
        }
    }

    painter.rect(draw_rect, rounding, bg, Stroke::new(1.5_f32, border));

    if key.shape == KeyShape::Stick {
        let (sx, sy) = state.stick_axes.get(&key.code).copied().unwrap_or((0.0, 0.0));
        let well = draw_rect.shrink(draw_rect.width() * 0.18);
        painter.circle_stroke(
            well.center(),
            well.width() * 0.5,
            Stroke::new(1.0_f32, border.gamma_multiply(0.5)),
        );
        let travel = draw_rect.width() * 0.28;
        let knob_c = draw_rect.center() + Vec2::new(sx * travel, sy * travel);
        let knob_r = draw_rect.width() * 0.21;
        let knob_col = parse_color(&key.style.active_glow_color).to_egui();
        painter.circle_filled(
            knob_c,
            knob_r,
            if active {
                knob_col
            } else {
                Color32::from_rgba_unmultiplied(knob_col.r(), knob_col.g(), knob_col.b(), 120)
            },
        );
        painter.circle_stroke(knob_c, knob_r, Stroke::new(1.5_f32, knob_col));
        if key.style.show_label {
            painter.text(
                Pos2::new(draw_rect.center().x, draw_rect.bottom() - 10.0),
                Align2::CENTER_CENTER,
                if capturing { "…" } else { &key.label },
                FontId::monospace(10.0),
                parse_color(&key.style.text_color).to_egui(),
            );
        }
    } else if key.style.show_label {
        let mut label = if capturing {
            "…".to_string()
        } else {
            key.label.clone()
        };
        if key.style.show_press_count {
            if let Some(n) = state.press_counts.get(&key.code) {
                if *n > 0 {
                    label = format!("{label}\n{n}");
                }
            }
        }
        painter.text(
            draw_rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::monospace(key.style.font_size),
            parse_color(&key.style.text_color).to_egui(),
        );
    }

    if selected {
        painter.rect_stroke(rect.expand(2.0), rounding, Stroke::new(2.0_f32, Color32::from_rgb(34, 211, 238)));
    }
    if capturing {
        painter.rect_stroke(rect.expand(2.0), rounding, Stroke::new(2.0_f32, Color32::from_rgb(251, 191, 36)));
    }

    response
}

pub fn paint_resize_handles(ui: &mut Ui, rect: Rect) {
    let size = 8.0;
    let painter = ui.painter();
    for pos in [
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    ] {
        let r = Rect::from_center_size(pos, Vec2::splat(size));
        painter.circle_filled(r.center(), size * 0.5, Color32::from_rgb(34, 211, 238));
        painter.circle_stroke(r.center(), size * 0.5, Stroke::new(1.0_f32, Color32::BLACK));
    }
}
